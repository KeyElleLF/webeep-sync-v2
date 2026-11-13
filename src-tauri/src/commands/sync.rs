//! The sync command.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::Semaphore;
use tracing::info;

use super::shared::{LAST_SYNC_KEY, fetch_courses, locked, saved_root, today};
use super::{CommandError, CommandResult};
use crate::auth::{self, store as keyring};
use crate::courses::{CourseInfo, assign_folders};
use crate::moodle::client::MoodleClient;
use crate::moodle::error::MoodleError;
use crate::notify::links::new_links_notice;
use crate::state::AppState;
use crate::sync::download::{Fetcher, RetryPolicy};
use crate::sync::local::unix_seconds;
use crate::sync::run::{CourseJob, CourseSummary, SyncContext};
use crate::sync::tree::TreeOptions;

const PROGRESS_EVENT: &str = "sync-progress";
const FINISHED_EVENT: &str = "sync-finished";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Progress {
    index: usize,
    total: usize,
    name: String,
}

#[tauri::command]
pub async fn sync_now(app: AppHandle) -> CommandResult<Vec<CourseSummary>> {
    run_sync(&app).await
}

/// One full sync, shared by the button and the background scheduler. Also tells the UI
/// (`sync-finished`) so a window opened later shows the result.
pub async fn run_sync(app: &AppHandle) -> CommandResult<Vec<CourseSummary>> {
    let state = app.state::<AppState>();
    let Some(_guard) = state.begin_sync() else {
        return Err(CommandError::new("una sincronizzazione è già in corso"));
    };
    let summaries = sync_selected(app, &state).await?;
    let _ = app.emit(FINISHED_EVENT, &summaries);
    Ok(summaries)
}

/// Notification "12 nuovi file in 3 corsi", if anything arrived.
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
    let title = format!("{files} file nuovi o aggiornati");
    crate::background::notify(app, &title, &names.join(", "));
}

/// Notification for links that teachers added (skipping muted courses).
pub fn notify_new_links(app: &AppHandle, summaries: &[CourseSummary]) {
    let state = app.state::<AppState>();
    let muted = locked(&state)
        .ok()
        .and_then(|store| store.muted_courses().ok())
        .unwrap_or_default();
    if let Some((title, body)) = new_links_notice(summaries, &muted) {
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
    };
    let total = jobs.len();
    info!("sync started: {total} courses");
    let summaries = context
        .sync_all(&jobs, |index, course| {
            let name = course.name.clone();
            let _ = app.emit(PROGRESS_EVENT, Progress { index, total, name });
        })
        .await?;
    let now = unix_seconds(SystemTime::now());
    locked(state)?.set_setting(LAST_SYNC_KEY, &now.to_string())?;
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
