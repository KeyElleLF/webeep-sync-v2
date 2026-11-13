//! Phase 2 commands: passed/ignored teachings, semester override, archived-course
//! suggestions and the cleanup of passed courses.

use std::time::SystemTime;

use tauri::State;

use super::shared::{locked, saved_root};
use super::{CommandError, CommandResult};
use crate::courses::semester::Semester;
use crate::courses::state::ManualState;
use crate::state::AppState;
use crate::sync::cleanup::{CleanupReport, system_trash};
use crate::sync::local::unix_seconds;
use crate::sync::retire::{self, CleanupPreview, GroupCourse, StateChange, StateOutcome};

const BUSY: &str = "attendi la fine della sincronizzazione";

/// Marks a teaching (all its courses) as passed or ignored, or back to automatic (`null`).
#[tauri::command]
pub fn set_teaching_state(
    state: State<'_, AppState>,
    teaching_key: String,
    teaching_state: Option<ManualState>,
) -> CommandResult<StateOutcome> {
    let Some(_guard) = state.begin_sync() else {
        return Err(CommandError::new(BUSY));
    };
    let root = saved_root(&state).ok();
    let mut store = locked(&state)?;
    let group: Vec<GroupCourse> = store
        .known_courses()?
        .into_iter()
        .filter(|c| c.teaching_key == teaching_key)
        .map(|c| GroupCourse {
            id: c.course_id,
            name: c.name,
        })
        .collect();
    let change = StateChange {
        key: &teaching_key,
        state: teaching_state,
        group: &group,
        policy: store.settings()?.cleanup_policy,
        root: root.as_deref(),
        now: unix_seconds(SystemTime::now()),
    };
    Ok(retire::set_teaching_state(
        &mut store,
        &change,
        &system_trash,
    )?)
}

/// Forces the semester of a teaching (1 or 2), or back to the automatic guess (`null`).
#[tauri::command]
pub fn set_teaching_semester(
    state: State<'_, AppState>,
    teaching_key: String,
    semester: Option<u8>,
) -> CommandResult<()> {
    let semester = match semester {
        None => None,
        Some(n) => {
            Some(Semester::from_number(n).ok_or_else(|| CommandError::new("semestre non valido"))?)
        }
    };
    Ok(locked(&state)?.set_teaching_semester(&teaching_key, semester)?)
}

/// "No, I have not passed it": the suggestion is not shown again.
#[tauri::command]
pub fn dismiss_suggestion(state: State<'_, AppState>, course_id: u64) -> CommandResult<()> {
    Ok(locked(&state)?.dismiss_known_course(course_id)?)
}

/// Cleanups waiting for an answer, with what each would remove.
#[tauri::command]
pub fn pending_cleanups(state: State<'_, AppState>) -> CommandResult<Vec<CleanupPreview>> {
    let Ok(root) = saved_root(&state) else {
        return Ok(Vec::new());
    };
    let store = locked(&state)?;
    let pending = store.pending_cleanups()?;
    let previews = pending
        .iter()
        .map(|p| retire::preview(&store, &root, p.course_id, &p.name))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(previews)
}

/// "Yes": moves the files to the Recycle Bin.
#[tauri::command]
pub fn confirm_cleanup(
    state: State<'_, AppState>,
    course_id: u64,
    include_modified: bool,
) -> CommandResult<CleanupReport> {
    let Some(_guard) = state.begin_sync() else {
        return Err(CommandError::new(BUSY));
    };
    let root = saved_root(&state)?;
    let mut store = locked(&state)?;
    retire::confirm_pending_cleanup(
        &mut store,
        &root,
        course_id,
        include_modified,
        &system_trash,
    )?
    .ok_or_else(|| CommandError::new("nessuna pulizia in attesa per questo corso"))
}

/// "No": keeps the files and does not ask again.
#[tauri::command]
pub fn decline_cleanup(state: State<'_, AppState>, course_id: u64) -> CommandResult<()> {
    Ok(locked(&state)?.remove_pending_cleanup(course_id)?)
}
