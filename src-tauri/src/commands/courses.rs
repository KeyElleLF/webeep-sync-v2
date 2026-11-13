//! Course list commands: listing, selection and folder of each course.

use serde::Serialize;
use tauri::State;
use tracing::info;

use super::shared::{fetch_courses, locked, saved_root, today};
use super::{CommandError, CommandResult};
use crate::courses::archived::{PassedSuggestion, known_from, passed_suggestions};
use crate::courses::{CourseInfo, assign_folders, current_academic_year};
use crate::state::AppState;
use crate::sync::folders::{FolderChange, change_course_folder};

/// A course in the list, with the folder it is (or would be) synced to.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseRow {
    #[serde(flatten)]
    pub info: CourseInfo,
    pub folder: String,
}

/// The course list plus the "did you pass it?" suggestions.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseListing {
    pub courses: Vec<CourseRow>,
    pub suggestions: Vec<PassedSuggestion>,
}

#[tauri::command]
pub async fn list_courses(state: State<'_, AppState>) -> CommandResult<CourseListing> {
    let client = super::session::saved_client()?;
    let courses = fetch_courses(&client, &state).await?;
    let (folders, suggestions) = {
        let mut store = locked(&state)?;
        let folders = assign_folders(&courses, &store.course_folders()?);
        let year = current_academic_year(today());
        let suggestions = passed_suggestions(&store.known_courses()?, &courses, &year);
        // Vanished courses stay remembered: the suggestion remains until answered.
        store.remember_courses(&known_from(&courses))?;
        (folders, suggestions)
    };
    let courses = courses
        .into_iter()
        .map(|info| CourseRow {
            folder: folders.get(&info.id).cloned().unwrap_or_default(),
            info,
        })
        .collect();
    Ok(CourseListing {
        courses,
        suggestions,
    })
}

#[tauri::command]
pub fn set_course_selected(
    state: State<'_, AppState>,
    course_id: u64,
    selected: bool,
) -> CommandResult<()> {
    Ok(locked(&state)?.set_course_selected(course_id, selected)?)
}

/// Gives a course another folder: moves it on disk, or points to an existing one.
#[tauri::command]
pub fn set_course_folder(
    state: State<'_, AppState>,
    course_id: u64,
    folder: String,
) -> CommandResult<FolderChange> {
    let Some(_guard) = state.begin_sync() else {
        return Err(CommandError::new(
            "attendi la fine della sincronizzazione per cambiare cartella",
        ));
    };
    let root = saved_root(&state)?;
    let store = locked(&state)?;
    let change = change_course_folder(&root, &store, course_id, &folder)?;
    info!("course {course_id} folder changed: {change:?}");
    Ok(change)
}
