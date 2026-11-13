//! Main window helpers. The app has no tray icon and no background mode: closing the
//! window quits it (user decision, 2026-11-09).

use tauri::{AppHandle, Manager};

const MAIN_WINDOW: &str = "main";

/// Brings the running window to the front (a second launch calls this).
pub fn show_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}
