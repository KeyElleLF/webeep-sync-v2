//! Passed and ignored teachings, and the cleanup of passed courses (§6.5).
//!
//! Marking a teaching as passed applies to all its courses (every academic year, every
//! teacher, §6.3), stops syncing them and, depending on the setting, queues a cleanup for
//! startup, runs it right away (untouched files only) or does nothing.

use std::path::Path;

use serde::Serialize;

use crate::courses::state::ManualState;
use crate::store::settings::CleanupPolicy;
use crate::store::{Store, StoreError};
use crate::sync::cleanup::{CleanupFile, CleanupReport, execute_cleanup, plan_cleanup};
use crate::sync::local::{safe_join, snapshot};

/// A course of the teaching: id and display name.
#[derive(Debug, Clone)]
pub struct GroupCourse {
    pub id: u64,
    pub name: String,
}

/// What a cleanup would do, for the confirmation dialog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupPreview {
    pub course_id: u64,
    pub name: String,
    pub files: usize,
    pub bytes: u64,
    /// Files the user changed: kept unless explicitly included.
    pub modified: Vec<CleanupFile>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StateOutcome {
    /// Cleanups queued for the confirmation dialog.
    pub queued: usize,
    /// Cleanups already done (policy "always").
    pub cleaned: Vec<CleanupReport>,
}

/// A change of teaching state and its context.
pub struct StateChange<'a> {
    pub key: &'a str,
    pub state: Option<ManualState>,
    /// Courses of the teaching, from every academic year.
    pub group: &'a [GroupCourse],
    pub policy: CleanupPolicy,
    /// Sync root, if chosen: needed to clean up right away.
    pub root: Option<&'a Path>,
    pub now: i64,
}

pub fn set_teaching_state(
    store: &mut Store,
    change: &StateChange<'_>,
    trash: &dyn Fn(&Path) -> Result<(), String>,
) -> Result<StateOutcome, StoreError> {
    let StateChange {
        key,
        state,
        group,
        policy,
        root,
        now,
    } = *change;
    store.set_teaching_state(key, state)?;
    let mut outcome = StateOutcome::default();
    for course in group {
        // Back to the default of the new state.
        store.clear_course_selection(course.id)?;
        if state != Some(ManualState::Passed) {
            store.remove_pending_cleanup(course.id)?;
            continue;
        }
        if store.baseline(course.id)?.is_empty() {
            continue;
        }
        match (policy, root) {
            (CleanupPolicy::Ask, _) | (CleanupPolicy::Always, None) => {
                store.add_pending_cleanup(course.id, &course.name, now)?;
                outcome.queued += 1;
            }
            (CleanupPolicy::Always, Some(root)) => {
                let report = run_cleanup(store, root, course.id, false, trash)?;
                outcome.cleaned.push(report);
            }
            (CleanupPolicy::Never, _) => {}
        }
    }
    Ok(outcome)
}

pub fn preview(
    store: &Store,
    root: &Path,
    course_id: u64,
    name: &str,
) -> Result<CleanupPreview, StoreError> {
    let plan = course_plan(store, root, course_id)?;
    Ok(CleanupPreview {
        course_id,
        name: name.to_owned(),
        files: plan.files.len(),
        bytes: plan.bytes(),
        modified: plan.modified,
    })
}

/// The user's "Yes": runs only if a cleanup of that course is really waiting for an answer,
/// so a stale or buggy frontend can never empty an active course. `None` otherwise.
pub fn confirm_pending_cleanup(
    store: &mut Store,
    root: &Path,
    course_id: u64,
    include_modified: bool,
    trash: &dyn Fn(&Path) -> Result<(), String>,
) -> Result<Option<CleanupReport>, StoreError> {
    let pending = store
        .pending_cleanups()?
        .iter()
        .any(|p| p.course_id == course_id);
    if !pending {
        return Ok(None);
    }
    run_cleanup(store, root, course_id, include_modified, trash).map(Some)
}

/// Moves the course files to the bin, forgets them and closes the pending request.
pub fn run_cleanup(
    store: &mut Store,
    root: &Path,
    course_id: u64,
    include_modified: bool,
    trash: &dyn Fn(&Path) -> Result<(), String>,
) -> Result<CleanupReport, StoreError> {
    let plan = course_plan(store, root, course_id)?;
    let folder = store.course_folders()?.remove(&course_id);
    let report = match folder.and_then(|f| safe_join(root, &f)) {
        Some(course_root) => {
            let (report, forget) = execute_cleanup(&course_root, &plan, include_modified, trash);
            store.forget_baseline(course_id, &forget)?;
            report
        }
        None => CleanupReport::default(),
    };
    store.remove_pending_cleanup(course_id)?;
    tracing::info!(
        "cleanup of course {course_id}: {} trashed, {} kept, {} skipped",
        report.trashed,
        report.kept,
        report.skipped.len()
    );
    Ok(report)
}

fn course_plan(
    store: &Store,
    root: &Path,
    course_id: u64,
) -> Result<crate::sync::cleanup::CleanupPlan, StoreError> {
    let baseline = store.baseline(course_id)?;
    let folder = store.course_folders()?.remove(&course_id);
    let Some(course_root) = folder.and_then(|f| safe_join(root, &f)) else {
        return Ok(Default::default());
    };
    let paths: Vec<&str> = baseline.values().map(|e| e.local_path.as_str()).collect();
    Ok(plan_cleanup(&baseline, &snapshot(&course_root, paths)))
}

#[cfg(test)]
#[path = "retire_tests.rs"]
mod tests;
