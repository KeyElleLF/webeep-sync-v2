//! The sync command.

use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::Semaphore;
use tracing::{info, warn};

use super::shared::{LAST_SYNC_KEY, fetch_courses, locked, saved_root, today};
use super::{CommandError, CommandResult};
use crate::auth::{self, store as keyring};
use crate::courses::{CourseInfo, assign_folders};
use crate::i18n::Language;
use crate::moodle::client::MoodleClient;
use crate::moodle::error::MoodleError;
use crate::notify::links::new_links_notice;
use crate::state::AppState;
use crate::store::history::{HistoryFile, history_files};
use crate::sync::download::{Fetcher, RetryPolicy};
use crate::sync::local::unix_seconds;
use crate::sync::run::{CourseJob, CourseSummary, SyncContext};
use crate::sync::tree::TreeOptions;

const PROGRESS_EVENT: &str = "sync-progress";
const FINISHED_EVENT: &str = "sync-finished";
/// Sent when any sync (button or automatic) starts and when one fails, so the UI never thinks a
/// sync is still running after it stopped.
const STARTED_EVENT: &str = "sync-started";
const FAILED_EVENT: &str = "sync-failed";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Progress {
    index: usize,
    total: usize,
    name: String,
    /// Files transferred / to transfer in the current course (0 / 0 until its plan is known).
    files_done: usize,
    files_total: usize,
}

#[tauri::command]
pub async fn sync_now(app: AppHandle) -> CommandResult<Vec<CourseSummary>> {
    run_sync(&app).await
}

/// "Annulla": the running sync (manual or automatic) stops after the downloads in progress.
#[tauri::command]
pub fn cancel_sync(state: State<'_, AppState>) {
    info!("sync cancel requested");
    state.cancel_sync();
}

/// One full sync, shared by the button and the background scheduler. Also tells the UI
/// (`sync-finished`) so a window opened later shows the result.
pub async fn run_sync(app: &AppHandle) -> CommandResult<Vec<CourseSummary>> {
    let state = app.state::<AppState>();
    let Some(_guard) = state.begin_sync() else {
        return Err(CommandError::new("una sincronizzazione è già in corso"));
    };
    state.cancel.store(false, Ordering::Release);
    let _ = app.emit(STARTED_EVENT, ());
    match sync_selected(app, &state).await {
        Ok(summaries) => {
            let _ = app.emit(FINISHED_EVENT, &summaries);
            Ok(summaries)
        }
        Err(err) => {
            let _ = app.emit(FAILED_EVENT, &err);
            Err(err)
        }
    }
}

/// Notification "12 file nuovi o aggiornati" with the course names, if anything arrived.
pub fn notify_new_files(app: &AppHandle, summaries: &[CourseSummary]) {
    let courses: Vec<&CourseSummary> = summaries
        .iter()
        .filter(|s| !s.report.downloaded.is_empty() || !s.report.updated.is_empty())
        .collect();
    let files: usize = courses
        .iter()
        .map(|s| s.report.downloaded.len() + s.report.updated.len())
        .sum();
    if files == 0 {
        return;
    }
    let names: Vec<&str> = courses.iter().map(|s| s.name.as_str()).collect();
    let title = match (crate::i18n::current(), files) {
        (Language::Italian, 1) => "1 file nuovo o aggiornato".to_owned(),
        (Language::Italian, n) => format!("{n} file nuovi o aggiornati"),
        (Language::English, 1) => "1 new or updated file".to_owned(),
        (Language::English, n) => format!("{n} new or updated files"),
    };
    crate::background::notify(app, &title, &names.join(", "));
}

/// Notification for links that teachers added (skipping muted courses).
pub fn notify_new_links(app: &AppHandle, summaries: &[CourseSummary]) {
    let state = app.state::<AppState>();
    let muted = locked(&state)
        .ok()
        .and_then(|store| store.muted_courses().ok())
        .unwrap_or_default();
    if let Some((title, body)) = new_links_notice(summaries, &muted, crate::i18n::current()) {
        crate::background::notify(app, &title, &body);
    }
}

async fn sync_selected(
    app: &AppHandle,
    state: &State<'_, AppState>,
) -> CommandResult<Vec<CourseSummary>> {
    let root = saved_root(state)?;
    let settings = locked(state)?.settings()?;
    let (client, fetcher) = clients()?;
    let selected: Vec<CourseInfo> = fetch_courses(&client, state)
        .await?
        .into_iter()
        .filter(|course| course.selected)
        .collect();
    let folders = persist_folders(state, &selected)?;
    let jobs: Vec<CourseJob<'_>> = selected
        .iter()
        .filter_map(|course| {
            let folder = folders.get(&course.id)?;
            Some(CourseJob { course, folder })
        })
        .collect();
    let context = SyncContext {
        client: &client,
        fetcher: &fetcher,
        store: &state.store,
        root: &root,
        limit: Arc::new(Semaphore::new(usize::from(settings.download_concurrency))),
        today: today(),
        now: unix_seconds(SystemTime::now()),
        options: TreeOptions {
            flatten_materials: settings.flatten_materials,
        },
        cancel: Arc::clone(&state.cancel),
        on_files: None,
    };
    let total = jobs.len();
    info!("sync started: {total} courses");
    let current = Mutex::new(Progress {
        index: 0,
        total,
        name: String::new(),
        files_done: 0,
        files_total: 0,
    });
    let emit = |update: &dyn Fn(&mut Progress)| {
        if let Ok(mut progress) = current.lock() {
            update(&mut progress);
            let _ = app.emit(PROGRESS_EVENT, &*progress);
        }
    };
    let on_files = |done: usize, of: usize| {
        emit(&|p| {
            p.files_done = done;
            p.files_total = of;
        });
    };
    let context = SyncContext {
        on_files: Some(&on_files),
        ..context
    };
    let summaries = context
        .sync_all(&jobs, |index, course| {
            emit(&|p| {
                p.index = index;
                p.name = course.name.clone();
                p.files_done = 0;
                p.files_total = 0;
            });
        })
        .await?;
    let now = unix_seconds(SystemTime::now());
    let files: Vec<HistoryFile> = summaries
        .iter()
        .flat_map(|s| history_files(s.id, &s.report))
        .collect();
    let mut store = locked(state)?;
    store.set_setting(LAST_SYNC_KEY, &now.to_string())?;
    // The history only feeds the "Oggi" page: a failure there must not fail the sync.
    if let Err(err) = store.record_sync(now, &files) {
        warn!("sync history not saved: {err}");
    }
    info!("sync finished");
    Ok(summaries)
}

/// REST client and downloader sharing the saved token.
pub fn clients() -> CommandResult<(MoodleClient, Fetcher)> {
    let token = keyring::load()?.ok_or(MoodleError::InvalidToken)?;
    let site = auth::site_url().map_err(|e| MoodleError::Config(e.to_string()))?;
    let client = MoodleClient::new(&site, token.clone())?;
    let fetcher = Fetcher::new(site, token, RetryPolicy::default())?;
    Ok((client, fetcher))
}

/// Assigns folders to new courses and saves them, so they never change afterwards.
fn persist_folders(
    state: &State<'_, AppState>,
    courses: &[CourseInfo],
) -> CommandResult<HashMap<u64, String>> {
    let store = locked(state)?;
    let assigned = store.course_folders()?;
    let folders = assign_folders(courses, &assigned);
    for (id, folder) in &folders {
        if !assigned.contains_key(id) {
            store.set_course_folder(*id, folder)?;
        }
    }
    Ok(folders)
}
