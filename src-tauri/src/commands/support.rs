//! Help for bug reports: app info for the pre-filled GitHub issue, and the diagnostic file
//! (shown to the user before saving, see `diagnostics::report`).

use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;
use tokio::sync::oneshot;

use super::shared::locked;
use super::{CommandError, CommandResult};
use crate::diagnostics::report::{ReportInput, build_report};
use crate::i18n::{self, localize, pick};
use crate::state::AppState;

/// Log files are small; anything bigger is cut to its end.
const MAX_LOG_FILE_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: String,
    pub os: String,
    pub arch: String,
}

#[tauri::command]
pub fn app_info() -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_owned(),
        os: std::env::consts::OS.to_owned(),
        arch: std::env::consts::ARCH.to_owned(),
    }
}

/// The diagnostic text, to show the user before saving it.
#[tauri::command]
pub fn diagnostics_preview(app: AppHandle, state: State<'_, AppState>) -> CommandResult<String> {
    diagnostics_text(&app, &state)
}

/// Saves the diagnostic text where the user chooses; `None` if cancelled.
#[tauri::command]
pub async fn save_diagnostics(
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<Option<String>> {
    let text = diagnostics_text(&app, &state)?;
    let name = format!(
        "{}-{}.txt",
        pick(
            i18n::current(),
            "webeep-sync-diagnostica",
            "webeep-sync-diagnostics"
        ),
        chrono::Local::now().format("%Y-%m-%d")
    );
    let (tx, rx) = oneshot::channel();
    app.dialog()
        .file()
        .set_title(localize("Salva il file di diagnostica"))
        .set_file_name(&name)
        .add_filter("Text", &["txt"])
        .save_file(move |picked| {
            let _ = tx.send(picked);
        });
    let Some(picked) = rx.await.ok().flatten() else {
        return Ok(None);
    };
    let path = picked
        .into_path()
        .map_err(|_| CommandError::new("percorso del file non valido"))?;
    std::fs::write(&path, text)
        .map_err(|e| CommandError::new(format!("impossibile salvare il file: {e}")))?;
    Ok(Some(path.display().to_string()))
}

fn diagnostics_text(app: &AppHandle, state: &State<'_, AppState>) -> CommandResult<String> {
    let (settings, schema, (files, courses), plan) = {
        let store = locked(state)?;
        let plan = match store.study_plan_imported()? {
            Some(_) => Some(store.study_plan()?.len()),
            None => None,
        };
        (
            store.settings()?,
            store.schema_version()?,
            store.baseline_counts()?,
            plan,
        )
    };
    let settings_text = format!(
        "{} downloads at a time, flatten Materiali {}, cleanup {:?}, autosync {} every {} min, notifications {}",
        settings.download_concurrency,
        settings.flatten_materials,
        settings.cleanup_policy,
        settings.autosync,
        settings.autosync_minutes,
        settings.notifications
    );
    let info = app_info();
    let system = format!("{} {}", info.os, info.arch);
    let created = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
    let logs = app
        .path()
        .app_log_dir()
        .map(|dir| read_logs(&dir))
        .unwrap_or_default();
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .ok();
    Ok(build_report(&ReportInput {
        version: &info.version,
        system: &system,
        language: i18n::current().code(),
        created: &created,
        settings: &settings_text,
        schema_version: schema,
        tracked_files: files,
        courses_with_files: courses,
        plan_teachings: plan,
        logs: &logs,
        home: home.as_deref(),
    }))
}

/// The app's daily log files, oldest first.
fn read_logs(dir: &Path) -> Vec<(String, String)> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("webeep-sync") && n.ends_with(".log"))
        })
        .collect();
    files.sort();
    files
        .into_iter()
        .filter_map(|path| {
            let name = path.file_name()?.to_string_lossy().into_owned();
            let bytes = std::fs::read(&path).ok()?;
            let start = bytes.len().saturating_sub(MAX_LOG_FILE_BYTES);
            Some((name, String::from_utf8_lossy(&bytes[start..]).into_owned()))
        })
        .collect()
}
