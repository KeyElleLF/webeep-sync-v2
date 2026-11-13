//! Phase 3 commands: announcements board, deadlines, muted courses.

use std::time::SystemTime;

use tauri::{AppHandle, State};

use super::CommandResult;
use super::shared::locked;
use super::sync::clients;
use crate::background::check_announcements;
use crate::notify::deadlines::{self, Deadline};
use crate::state::AppState;
use crate::store::notify::Announcement;
use crate::sync::local::unix_seconds;

/// Announcements saved so far, newest first (no network).
#[tauri::command]
pub fn list_announcements(state: State<'_, AppState>) -> CommandResult<Vec<Announcement>> {
    Ok(locked(&state)?.announcements()?)
}

/// Asks WeBeep for new announcements now, then returns the whole list.
#[tauri::command]
pub async fn refresh_announcements(app: AppHandle) -> CommandResult<Vec<Announcement>> {
    check_announcements(&app, unix_seconds(SystemTime::now())).await?;
    let state = tauri::Manager::state::<AppState>(&app);
    Ok(locked(&state)?.announcements()?)
}

/// Marks one announcement as read, or all of them with `null`.
#[tauri::command]
pub fn mark_announcements_read(state: State<'_, AppState>, id: Option<u64>) -> CommandResult<()> {
    Ok(locked(&state)?.mark_read(id)?)
}

#[tauri::command]
pub async fn list_deadlines() -> CommandResult<Vec<Deadline>> {
    let (client, _) = clients()?;
    Ok(deadlines::fetch(&client, unix_seconds(SystemTime::now())).await?)
}

#[tauri::command]
pub fn muted_courses(state: State<'_, AppState>) -> CommandResult<Vec<u64>> {
    Ok(locked(&state)?.muted_courses()?.into_iter().collect())
}

/// No notifications for this course (announcements and deadlines).
#[tauri::command]
pub fn set_course_muted(
    state: State<'_, AppState>,
    course_id: u64,
    muted: bool,
) -> CommandResult<()> {
    Ok(locked(&state)?.set_muted(course_id, muted)?)
}
