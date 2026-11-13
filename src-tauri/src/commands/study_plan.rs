//! Import of the study plan (receipt PDF, printed page or pasted table), which sets the
//! semester of the courses.

use std::path::PathBuf;
use std::time::SystemTime;

use serde::Serialize;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;
use tokio::sync::oneshot;
use tracing::info;

use super::shared::locked;
use super::{CommandError, CommandResult};
use crate::courses::plan_file::read_plan_pdf;
use crate::courses::semester::Semester;
use crate::courses::study_plan::{PlanEntry, parse_plan};
use crate::state::AppState;
use crate::sync::local::unix_seconds;

/// The pasted table is a few KB; much more means the whole page or something else.
const MAX_PASTED_BYTES: usize = 200 * 1024;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanStatus {
    pub teachings: usize,
    pub first: usize,
    pub second: usize,
    pub imported_at: i64,
}

fn status(entries: &[PlanEntry], imported_at: i64) -> PlanStatus {
    let first = entries
        .iter()
        .filter(|e| e.semester == Semester::First)
        .count();
    PlanStatus {
        teachings: entries.len(),
        first,
        second: entries.len() - first,
        imported_at,
    }
}

/// The loaded plan, if any (no network).
#[tauri::command]
pub fn study_plan_status(state: State<'_, AppState>) -> CommandResult<Option<PlanStatus>> {
    let store = locked(&state)?;
    let Some(imported_at) = store.study_plan_imported()? else {
        return Ok(None);
    };
    Ok(Some(status(&store.study_plan()?, imported_at)))
}

/// Imports a PDF dropped on the window (its path comes from the drop event).
#[tauri::command]
pub fn import_study_plan(state: State<'_, AppState>, path: String) -> CommandResult<PlanStatus> {
    let path = PathBuf::from(path);
    if !path.is_absolute() {
        return Err(CommandError::new("percorso del file non valido"));
    }
    import(&state, &path)
}

/// Imports the table of the "Piano degli studi" page copied and pasted by the user.
#[tauri::command]
pub fn import_study_plan_text(
    state: State<'_, AppState>,
    text: String,
) -> CommandResult<PlanStatus> {
    if text.len() > MAX_PASTED_BYTES {
        return Err(CommandError::new(
            "il testo è troppo lungo: incolla solo la tabella degli insegnamenti",
        ));
    }
    save(&state, parse_plan(&text))
}

/// Opens the file picker (PDF only) and imports the chosen file; `None` if cancelled.
#[tauri::command]
pub async fn choose_study_plan(
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<Option<PlanStatus>> {
    let (tx, rx) = oneshot::channel();
    app.dialog()
        .file()
        .set_title(crate::i18n::localize(
            "Scegli la ricevuta del piano degli studi (PDF)",
        ))
        .add_filter("PDF", &["pdf"])
        .pick_file(move |picked| {
            let _ = tx.send(picked);
        });
    let Some(picked) = rx.await.ok().flatten() else {
        return Ok(None);
    };
    let path = picked
        .into_path()
        .map_err(|_| CommandError::new("file non valido"))?;
    import(&state, &path).map(Some)
}

/// Forgets the plan: semesters go back to the automatic guess.
#[tauri::command]
pub fn clear_study_plan(state: State<'_, AppState>) -> CommandResult<()> {
    locked(&state)?.clear_study_plan()?;
    info!("study plan removed");
    Ok(())
}

fn import(state: &State<'_, AppState>, path: &std::path::Path) -> CommandResult<PlanStatus> {
    save(state, read_plan_pdf(path)?)
}

fn save(state: &State<'_, AppState>, entries: Vec<PlanEntry>) -> CommandResult<PlanStatus> {
    if entries.is_empty() {
        return Err(CommandError::new(
            "non ho trovato insegnamenti: copia la tabella della pagina Piano degli studi",
        ));
    }
    let now = unix_seconds(SystemTime::now());
    locked(state)?.replace_study_plan(&entries, now)?;
    info!("study plan imported: {} teachings", entries.len());
    Ok(status(&entries, now))
}
