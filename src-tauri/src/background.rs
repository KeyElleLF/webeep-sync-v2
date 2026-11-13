//! Periodic work while the app window is open: automatic sync (§7), announcement checks
//! every 15 minutes and deadline reminders (§8). There is no background mode: closing the
//! window quits the app and stops all of this (user decision, 2026-11-09).

use std::collections::HashSet;
use std::time::{Duration, SystemTime};

use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;
use tracing::{info, warn};

use crate::commands::shared::{fetch_courses, locked};
use crate::commands::sync::{clients, notify_new_files, notify_new_links, run_sync};
use crate::commands::{CommandError, CommandResult};
use crate::notify::announcements::{self, by_course};
use crate::notify::deadlines::{self, Deadline, due_reminders};
use crate::notify::html::preview;
use crate::state::AppState;
use crate::store::notify::Announcement;
use crate::sync::local::unix_seconds;

/// First check soon after opening: the app is usually open only for a short while.
const STARTUP_DELAY: Duration = Duration::from_secs(5);
const TICK: Duration = Duration::from_secs(60);
/// §8: announcements every 15 minutes.
const ANNOUNCEMENTS_EVERY: i64 = 15 * 60;
const DEADLINES_EVERY: i64 = 60 * 60;
pub const ANNOUNCEMENTS_EVENT: &str = "announcements-updated";

/// When each periodic job last ran (unix seconds).
#[derive(Debug, Default)]
struct Timers {
    announcements: Option<i64>,
    deadlines: Option<i64>,
}

pub fn start(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(STARTUP_DELAY).await;
        let mut timers = Timers::default();
        loop {
            tick(&app, &mut timers).await;
            tokio::time::sleep(TICK).await;
        }
    });
}

/// Whether a job last run at `last` should run again.
pub fn is_due(last: Option<i64>, now: i64, every_secs: i64) -> bool {
    last.is_none_or(|last| now - last >= every_secs)
}

async fn tick(app: &AppHandle, timers: &mut Timers) {
    if !matches!(crate::auth::store::load(), Ok(Some(_))) {
        return;
    }
    let now = unix_seconds(SystemTime::now());
    if is_due(timers.announcements, now, ANNOUNCEMENTS_EVERY) {
        timers.announcements = Some(now);
        if let Err(err) = check_announcements(app, now).await {
            warn!("announcements check failed: {}", err.message);
        }
    }
    if is_due(timers.deadlines, now, DEADLINES_EVERY) {
        timers.deadlines = Some(now);
        if let Err(err) = remind_deadlines(app, now).await {
            warn!("deadlines check failed: {}", err.message);
        }
    }
    if autosync_due(app, now) {
        match run_sync(app).await {
            Ok(summaries) => {
                notify_new_files(app, &summaries);
                notify_new_links(app, &summaries);
            }
            Err(err) => warn!("automatic sync failed: {}", err.message),
        }
    }
}

fn autosync_due(app: &AppHandle, now: i64) -> bool {
    let state = app.state::<AppState>();
    let Ok(store) = locked(&state) else {
        return false;
    };
    let Ok(settings) = store.settings() else {
        return false;
    };
    let has_root = matches!(
        store.setting(crate::commands::shared::SYNC_ROOT_KEY),
        Ok(Some(_))
    );
    let last = store
        .setting(crate::commands::shared::LAST_SYNC_KEY)
        .ok()
        .flatten()
        .and_then(|v| v.parse().ok());
    // Never synced by hand yet: the first sync is the user's choice.
    settings.autosync
        && has_root
        && last.is_some()
        && is_due(last, now, i64::from(settings.autosync_minutes) * 60)
}

/// Checks the announcements of the synced courses and notifies the new ones.
pub async fn check_announcements(app: &AppHandle, now: i64) -> CommandResult<Vec<Announcement>> {
    let state = app.state::<AppState>();
    let (client, _) = clients()?;
    let courses = fetch_courses(&client, &state).await?;
    let synced: Vec<_> = courses.iter().filter(|c| c.selected).collect();
    let ids: Vec<u64> = synced.iter().map(|c| c.id).collect();
    let fresh = announcements::check(&client, &state.store, &ids, now)
        .await
        .map_err(|e| CommandError::new(e.to_string()))?;
    let muted = locked(&state)?.muted_courses()?;
    for (course_id, items) in by_course(&fresh) {
        if muted.contains(&course_id) {
            continue;
        }
        let name = synced
            .iter()
            .find(|c| c.id == course_id)
            .map_or("WeBeep", |c| c.name.as_str());
        notify(
            app,
            &announcement_title(name, items.len()),
            &announcement_body(&items),
        );
    }
    let _ = app.emit(ANNOUNCEMENTS_EVENT, fresh.len());
    if !fresh.is_empty() {
        info!("{} new announcements", fresh.len());
    }
    Ok(fresh)
}

fn announcement_title(course: &str, count: usize) -> String {
    match count {
        1 => format!("{course}: nuovo avviso"),
        n => format!("{course}: {n} nuovi avvisi"),
    }
}

fn announcement_body(items: &[&Announcement]) -> String {
    match items {
        [one] => format!("{} — {}", one.subject, preview(&one.message)),
        many => many
            .iter()
            .map(|a| a.subject.as_str())
            .collect::<Vec<_>>()
            .join(" · "),
    }
}

async fn remind_deadlines(app: &AppHandle, now: i64) -> CommandResult<()> {
    let state = app.state::<AppState>();
    let (client, _) = clients()?;
    let upcoming = deadlines::fetch(&client, now).await?;
    let (reminded, muted): (HashSet<u64>, HashSet<u64>) = {
        let store = locked(&state)?;
        (store.reminded_deadlines()?, store.muted_courses()?)
    };
    for deadline in due_reminders(&upcoming, now, &reminded, &muted) {
        notify(app, &deadline_title(deadline, now), &deadline.course_name);
        locked(&state)?.set_reminded(deadline.id, now)?;
    }
    Ok(())
}

fn deadline_title(deadline: &Deadline, now: i64) -> String {
    match ((deadline.due - now) / 3_600).max(1) {
        1 => format!("Tra meno di 2 ore: {}", deadline.name),
        hours => format!("Tra {hours} ore: {}", deadline.name),
    }
}

/// Shows a system notification, if the user wants them.
pub fn notify(app: &AppHandle, title: &str, body: &str) {
    let state = app.state::<AppState>();
    let enabled = locked(&state)
        .ok()
        .and_then(|s| s.settings().ok())
        .is_none_or(|s| s.notifications);
    if !enabled {
        return;
    }
    if let Err(err) = app.notification().builder().title(title).body(body).show() {
        warn!("notification failed: {err}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jobs_run_first_time_and_then_after_their_interval() {
        assert!(is_due(None, 1_000, 900));
        assert!(!is_due(Some(500), 1_000, 900));
        assert!(is_due(Some(100), 1_000, 900));
    }

    #[test]
    fn titles_and_bodies_are_readable() {
        let item = |subject: &str| Announcement {
            id: 1,
            course_id: 1,
            subject: subject.to_owned(),
            message: "<p>Lezione <b>annullata</b></p>".to_owned(),
            author: String::new(),
            created: 0,
            modified: 0,
            pinned: false,
            attachments: Vec::new(),
            read: false,
        };
        assert_eq!(
            announcement_title("DATA MINING", 1),
            "DATA MINING: nuovo avviso"
        );
        assert_eq!(
            announcement_title("DATA MINING", 3),
            "DATA MINING: 3 nuovi avvisi"
        );
        let one = item("Lezione di domani");
        assert_eq!(
            announcement_body(&[&one]),
            "Lezione di domani — Lezione annullata"
        );
        let two = item("Esame");
        assert_eq!(
            announcement_body(&[&one, &two]),
            "Lezione di domani · Esame"
        );
        let deadline = Deadline {
            id: 1,
            name: "Homework 2 è in scadenza".to_owned(),
            activity: String::new(),
            kind: "assign".to_owned(),
            course_id: 1,
            course_name: "ML".to_owned(),
            due: 10 * 3_600,
            overdue: false,
            url: String::new(),
        };
        assert_eq!(
            deadline_title(&deadline, 0),
            "Tra 10 ore: Homework 2 è in scadenza"
        );
        assert_eq!(
            deadline_title(&deadline, deadline.due - 60),
            "Tra meno di 2 ore: Homework 2 è in scadenza"
        );
    }
}
