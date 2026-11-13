//! Settings commands: destination folder and sync options.

use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;
use tokio::sync::oneshot;
use tracing::info;

use super::shared::{LAST_SYNC_KEY, SYNC_ROOT_KEY, locked, valid_root};
use super::{CommandError, CommandResult};
use crate::state::AppState;
use crate::store::settings::Settings;

#[tauri::command]
pub fn sync_root(state: State<'_, AppState>) -> CommandResult<Option<String>> {
    Ok(locked(&state)?.setting(SYNC_ROOT_KEY)?)
}

/// Opens the system folder picker and saves the choice. `None` if the user cancels.
#[tauri::command]
pub async fn choose_sync_root(
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<Option<String>> {
    let (tx, rx) = oneshot::channel();
    app.dialog()
        .file()
        .set_title("Scegli la cartella in cui sincronizzare i corsi")
        .pick_folder(move |picked| {
            let _ = tx.send(picked);
        });
    let Some(picked) = rx.await.ok().flatten() else {
        return Ok(None);
    };
    let path = picked
        .into_path()
        .map_err(|_| CommandError::new("cartella non valida"))?;
    let root = valid_root(path)?;
    locked(&state)?.set_setting(SYNC_ROOT_KEY, &root)?;
    info!("sync root changed");
    Ok(Some(root))
}

/// Unix time of the last completed sync, if any.
#[tauri::command]
pub fn last_sync(state: State<'_, AppState>) -> CommandResult<Option<i64>> {
    Ok(locked(&state)?
        .setting(LAST_SYNC_KEY)?
        .and_then(|v| v.parse().ok()))
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> CommandResult<Settings> {
    Ok(locked(&state)?.settings()?)
}

/// Saves the settings (out-of-range values are clamped) and returns what was stored.
#[tauri::command]
pub fn set_settings(state: State<'_, AppState>, settings: Settings) -> CommandResult<Settings> {
    Ok(locked(&state)?.save_settings(settings)?)
}
