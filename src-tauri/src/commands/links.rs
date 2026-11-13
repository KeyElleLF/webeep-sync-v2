//! The "Link" page: external links of the courses and their folders.

use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

use super::shared::{locked, saved_root};
use super::{CommandError, CommandResult};
use crate::state::AppState;
use crate::store::links::CourseLink;
use crate::sync::local::safe_join;

/// Links saved by the syncs, by course (no network).
#[tauri::command]
pub fn list_links(state: State<'_, AppState>) -> CommandResult<Vec<CourseLink>> {
    Ok(locked(&state)?.links()?)
}

/// Marks one link as seen (it was opened), or all of them with `null`.
#[tauri::command]
pub fn mark_links_seen(state: State<'_, AppState>, id: Option<i64>) -> CommandResult<()> {
    Ok(locked(&state)?.mark_links_seen(id)?)
}

/// Opens the folder of a course in Explorer, creating it if nothing was downloaded yet (a
/// course with only links has no files), so that files saved by hand go to the right place.
#[tauri::command]
pub fn open_course_folder(
    app: AppHandle,
    state: State<'_, AppState>,
    course_id: u64,
) -> CommandResult<()> {
    let root = saved_root(&state)?;
    let folder = locked(&state)?
        .course_folders()?
        .remove(&course_id)
        .ok_or_else(|| CommandError::new("il corso non ha ancora una cartella: sincronizzalo"))?;
    let path = safe_join(&root, &folder)
        .ok_or_else(|| CommandError::new(format!("nome di cartella non valido: «{folder}»")))?;
    std::fs::create_dir_all(&path)
        .map_err(|e| CommandError::new(format!("impossibile creare la cartella del corso: {e}")))?;
    let path = path.into_os_string().into_string().map_err(|_| {
        CommandError::new("il percorso della cartella contiene caratteri non validi")
    })?;
    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|e| CommandError::new(format!("impossibile aprire la cartella: {e}")))
}
