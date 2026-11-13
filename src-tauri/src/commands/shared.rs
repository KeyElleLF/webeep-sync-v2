//! Helpers shared by the course, settings and sync commands.

use std::path::PathBuf;
use std::sync::MutexGuard;

use chrono::{Local, NaiveDate};
use tauri::State;

use super::{CommandError, CommandResult};
use crate::courses::study_plan::PlanIndex;
use crate::courses::{CourseInfo, ListInputs, course_list};
use crate::moodle::client::{MoodleClient, param};
use crate::moodle::types::TimelineResponse;
use crate::state::AppState;
use crate::store::Store;

pub const SYNC_ROOT_KEY: &str = "sync_root";
pub const LAST_SYNC_KEY: &str = "last_sync";
const TIMELINE_FUNCTION: &str = "core_course_get_enrolled_courses_by_timeline_classification";

pub fn today() -> NaiveDate {
    Local::now().date_naive()
}

pub fn locked<'a>(state: &'a State<'_, AppState>) -> CommandResult<MutexGuard<'a, Store>> {
    state
        .store
        .lock()
        .map_err(|_| CommandError::new("database locale non disponibile"))
}

/// The saved sync root, checked to still be an existing directory.
pub fn saved_root(state: &State<'_, AppState>) -> CommandResult<PathBuf> {
    let root = locked(state)?
        .setting(SYNC_ROOT_KEY)?
        .ok_or_else(|| CommandError::new("scegli prima la cartella di destinazione"))?;
    Ok(PathBuf::from(valid_root(PathBuf::from(root))?))
}

/// The sync root must be an existing absolute directory.
pub fn valid_root(path: PathBuf) -> CommandResult<String> {
    if !path.is_absolute() || !path.is_dir() {
        return Err(CommandError::new(
            "la cartella di destinazione non esiste più: sceglila di nuovo",
        ));
    }
    path.into_os_string()
        .into_string()
        .map_err(|_| CommandError::new("il percorso della cartella contiene caratteri non validi"))
}

/// All visible enrolments with the default selection applied.
pub async fn fetch_courses(
    client: &MoodleClient,
    state: &State<'_, AppState>,
) -> CommandResult<Vec<CourseInfo>> {
    let params = [
        param("classification", "all"),
        param("offset", 0),
        param("limit", 0),
        param("sort", "fullname"),
    ];
    let response: TimelineResponse = client.call(TIMELINE_FUNCTION, &params).await?;
    let inputs = {
        let store = locked(state)?;
        ListInputs {
            selection: store.course_selection()?,
            teachings: store.teaching_prefs()?,
            activity: store.course_activity()?,
            plan: PlanIndex::new(&store.study_plan()?),
        }
    };
    Ok(course_list(&response.courses, &inputs, today()))
}
