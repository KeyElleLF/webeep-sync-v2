//! Phase 0 developer command: dump raw WeBeep responses for analysis.

use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use super::{CommandError, CommandResult};
use crate::commands::session::saved_client;
use crate::diagnostics::dump::{self, DumpReport};
use crate::moodle::client::ContentLanguage;

/// In debug builds dumps go to `<repo>/fixtures/raw` (gitignored); in release to app data.
fn dump_base_dir(app: &AppHandle) -> CommandResult<PathBuf> {
    if cfg!(debug_assertions) {
        return Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("fixtures")
            .join("raw"));
    }
    let data_dir = app.path().app_local_data_dir().map_err(|e| CommandError {
        message: format!("cartella dati dell'app non disponibile: {e}"),
    })?;
    Ok(data_dir.join("dumps"))
}

/// `lang`: content language of the dump, `it` (default) or `en`.
#[tauri::command]
pub async fn dump_phase0(app: AppHandle, lang: Option<String>) -> CommandResult<DumpReport> {
    if !cfg!(debug_assertions) {
        return Err(CommandError {
            message: "il dump è disponibile solo nelle build di sviluppo".to_owned(),
        });
    }
    let language = match lang.as_deref() {
        None | Some("it") => ContentLanguage::Italian,
        Some("en") => ContentLanguage::English,
        Some(_) => return Err(CommandError::new("lingua del dump non valida")),
    };
    let client = saved_client()?.with_language(language);
    let base_dir = dump_base_dir(&app)?;
    Ok(dump::run(&client, &base_dir).await?)
}
