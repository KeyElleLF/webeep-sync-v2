mod auth;
mod background;
mod commands;
mod courses;
mod diagnostics;
mod i18n;
mod logging;
mod moodle;
mod notify;
mod state;
mod store;
mod sync;
#[cfg(test)]
mod test_support;
mod window;

use tauri::{App, Manager};

fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    // `%LOCALAPPDATA%\it.webeepsync.app\logs` on Windows.
    let log_dir = app.path().app_log_dir().ok();
    if let Some(dir) = &log_dir {
        let _ = std::fs::create_dir_all(dir);
    }
    app.manage(logging::init(log_dir.as_deref()));
    tracing::info!("WeBeep Sync {} started", env!("CARGO_PKG_VERSION"));
    let state = state::AppState::open(app.handle())?;
    commands::settings::load_language(&state);
    app.manage(state);
    // Periodic checks run only while the window is open: closing it quits the app.
    background::start(app.handle().clone());
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let result = tauri::Builder::default()
        // Must be first: a second launch just shows the running window.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            window::show_main(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        // Updates from the GitHub releases (signed; see plugins.updater in tauri.conf.json).
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(setup)
        .invoke_handler(tauri::generate_handler![
            commands::session::auth_status,
            commands::session::login,
            commands::session::logout,
            commands::dump::dump_phase0,
            commands::courses::list_courses,
            commands::courses::set_course_selected,
            commands::courses::set_course_folder,
            commands::settings::sync_root,
            commands::settings::choose_sync_root,
            commands::settings::get_settings,
            commands::settings::last_sync,
            commands::settings::set_settings,
            commands::settings::get_language,
            commands::settings::set_language,
            commands::sync::sync_now,
            commands::sync::cancel_sync,
            commands::teaching::set_teaching_state,
            commands::teaching::set_teaching_semester,
            commands::teaching::dismiss_suggestion,
            commands::teaching::pending_cleanups,
            commands::teaching::confirm_cleanup,
            commands::teaching::decline_cleanup,
            commands::notify::list_announcements,
            commands::notify::refresh_announcements,
            commands::notify::mark_announcements_read,
            commands::notify::list_deadlines,
            commands::notify::muted_courses,
            commands::notify::set_course_muted,
            commands::links::list_links,
            commands::links::mark_links_seen,
            commands::links::open_course_folder,
            commands::today::sync_history,
            commands::today::mark_today_opened,
            commands::today::reveal_course_file,
            commands::study_plan::study_plan_status,
            commands::study_plan::import_study_plan,
            commands::study_plan::import_study_plan_text,
            commands::study_plan::choose_study_plan,
            commands::study_plan::clear_study_plan,
            commands::support::app_info,
            commands::support::diagnostics_preview,
            commands::support::save_diagnostics,
        ])
        .run(tauri::generate_context!());
    if let Err(err) = result {
        tracing::error!("application error: {err}");
        std::process::exit(1);
    }
}
