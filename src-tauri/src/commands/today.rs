//! The "Oggi" page: what the last syncs brought, when the app was last opened, and showing a
//! downloaded file in File Explorer.

use std::time::SystemTime;

use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

use super::shared::{locked, saved_root};
use super::{CommandError, CommandResult};
use crate::state::AppState;
use crate::store::history::{HISTORY_DAYS, SyncRecord};
use crate::sync::local::{safe_join, unix_seconds};

/// When "Oggi" was last shown: what arrived after it is marked as new.
const TODAY_SEEN_KEY: &str = "today_seen";
const DAY: i64 = 86_400;

/// The syncs of the last `days` days (1 to 30) that brought files, newest first.
#[tauri::command]
pub fn sync_history(state: State<'_, AppState>, days: u32) -> CommandResult<Vec<SyncRecord>> {
    let days = i64::from(days).clamp(1, HISTORY_DAYS);
    let since = unix_seconds(SystemTime::now()) - days * DAY;
    Ok(locked(&state)?.sync_history(since)?)
}

/// Returns when the app was last opened on "Oggi" (Unix seconds, none the first time) and
/// records now as the new time. Called once when the app starts.
#[tauri::command]
pub fn mark_today_opened(state: State<'_, AppState>) -> CommandResult<Option<i64>> {
    let store = locked(&state)?;
    let previous = store
        .setting(TODAY_SEEN_KEY)?
        .and_then(|value| value.parse().ok());
    store.set_setting(TODAY_SEEN_KEY, &unix_seconds(SystemTime::now()).to_string())?;
    Ok(previous)
}

/// Opens File Explorer on the course folder with the file selected. The path is the one in the
/// sync history, inside the course folder: anything that could leave it is refused.
#[tauri::command]
pub fn reveal_course_file(
    app: AppHandle,
    state: State<'_, AppState>,
    course_id: u64,
    path: String,
) -> CommandResult<()> {
    let root = saved_root(&state)?;
    let folder = locked(&state)?
        .course_folders()?
        .remove(&course_id)
        .ok_or_else(|| CommandError::new("il corso non ha ancora una cartella: sincronizzalo"))?;
    let course_dir = safe_join(&root, &folder)
        .ok_or_else(|| CommandError::new(format!("nome di cartella non valido: «{folder}»")))?;
    let file = safe_join(&course_dir, &path)
        .ok_or_else(|| CommandError::new(format!("percorso non valido: «{path}»")))?;
    if !file.is_file() {
        return Err(CommandError::new(
            "il file non c'è più: è stato spostato o eliminato",
        ));
    }
    app.opener()
        .reveal_item_in_dir(&file)
        .map_err(|e| CommandError::new(format!("impossibile mostrare il file: {e}")))
}
