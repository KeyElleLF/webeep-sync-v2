//! Orchestration of a sync: course list → per course get_contents → tree → plan → execute →
//! baseline. Each course is isolated: its failure is reported and the next one runs. Only an
//! invalid token stops everything, since every later call would fail too.

use std::path::Path;
use std::sync::{Arc, Mutex};

use chrono::NaiveDate;
use serde::Serialize;
use thiserror::Error;
use tokio::sync::Semaphore;

use crate::courses::CourseInfo;
use crate::courses::semester::count_activity;
use crate::moodle::client::{MoodleClient, array_param, param};
use crate::moodle::error::MoodleError;
use crate::moodle::types::{Assignment, AssignmentsResponse, CourseSection};
use crate::store::{Store, StoreError};
use crate::sync::assign::with_assignment_files;
use crate::sync::download::Fetcher;
use crate::sync::execute::{CourseReport, CourseRun};
use crate::sync::local::{safe_join, snapshot};
use crate::sync::plan::plan;
use crate::sync::tree::{ExternalLink, TreeOptions, build_tree};

#[derive(Debug, Error)]
pub enum SyncError {
    #[error(transparent)]
    Moodle(#[from] MoodleError),
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error("database locale non disponibile")]
    StoreLocked,
    #[error("nome di cartella non valido: «{0}»")]
    BadFolder(String),
}

/// Result of one course, for the summary.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseSummary {
    pub id: u64,
    pub name: String,
    pub folder: String,
    pub report: CourseReport,
    pub external_links: Vec<ExternalLink>,
    /// Links that appeared since the previous sync (none on the first sync of a course).
    pub new_links: Vec<ExternalLink>,
    /// Set when the course could not be synced at all.
    pub error: Option<String>,
}

/// Everything a sync needs. `root` is the folder that contains the course folders.
pub struct SyncContext<'a> {
    pub client: &'a MoodleClient,
    pub fetcher: &'a Fetcher,
    pub store: &'a Mutex<Store>,
    pub root: &'a Path,
    pub limit: Arc<Semaphore>,
    pub today: NaiveDate,
    /// Unix seconds, for when a link was first seen.
    pub now: i64,
    pub options: TreeOptions,
}

/// A course to sync together with its (already assigned) folder.
pub struct CourseJob<'a> {
    pub course: &'a CourseInfo,
    pub folder: &'a str,
}

impl SyncContext<'_> {
    /// Syncs `jobs` in order, calling `on_course` before each one.
    pub async fn sync_all(
        &self,
        jobs: &[CourseJob<'_>],
        mut on_course: impl FnMut(usize, &CourseInfo),
    ) -> Result<Vec<CourseSummary>, SyncError> {
        let assignments = self.assignments(jobs).await?;
        let mut summaries = Vec::with_capacity(jobs.len());
        for (index, job) in jobs.iter().enumerate() {
            on_course(index, job.course);
            let summary = match self.sync_course(job, &assignments).await {
                Ok(summary) => summary,
                Err(SyncError::Moodle(MoodleError::InvalidToken)) => {
                    return Err(MoodleError::InvalidToken.into());
                }
                Err(err) => {
                    tracing::warn!("course {} failed: {err}", job.course.id);
                    failed_course(job, err.to_string())
                }
            };
            summaries.push(summary);
        }
        Ok(summaries)
    }

    /// Assignment attachments of all courses in one call. Failing here only loses the
    /// attachments, so it is not fatal (except for an invalid token).
    async fn assignments(&self, jobs: &[CourseJob<'_>]) -> Result<Vec<Assignment>, SyncError> {
        let ids: Vec<u64> = jobs.iter().map(|job| job.course.id).collect();
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let params = array_param("courseids", &ids);
        match self
            .client
            .call::<AssignmentsResponse>("mod_assign_get_assignments", &params)
            .await
        {
            Ok(response) => Ok(response
                .courses
                .into_iter()
                .flat_map(|c| c.assignments)
                .collect()),
            Err(MoodleError::InvalidToken) => Err(MoodleError::InvalidToken.into()),
            Err(err) => {
                tracing::warn!("assignments unavailable: {err}");
                Ok(Vec::new())
            }
        }
    }

    async fn sync_course(
        &self,
        job: &CourseJob<'_>,
        assignments: &[Assignment],
    ) -> Result<CourseSummary, SyncError> {
        let course_id = job.course.id;
        let course_root = safe_join(self.root, job.folder)
            .ok_or_else(|| SyncError::BadFolder(job.folder.to_owned()))?;
        let sections: Vec<CourseSection> = self
            .client
            .call("core_course_get_contents", &[param("courseid", course_id)])
            .await?;
        let tree = build_tree(&with_assignment_files(&sections, assignments), self.options);
        let baseline = self.with_store(|store| store.baseline(course_id))?;
        let tracked = tree.files.iter().map(|f| f.local_path.join("/"));
        let previous = baseline.values().map(|e| e.local_path.clone());
        let paths: Vec<String> = tracked.chain(previous).collect();
        let local = snapshot(&course_root, paths.iter().map(String::as_str));
        let actions = plan(&tree.files, &baseline, &local, self.today);
        let run = CourseRun {
            fetcher: self.fetcher,
            limit: Arc::clone(&self.limit),
            root: &course_root,
            remote: &tree.files,
            baseline: &baseline,
        };
        let outcome = run.execute(actions).await;
        self.with_store(|store| store.save_baseline(course_id, &outcome.baseline))?;
        // When the files were uploaded tells the semester (see `courses::semester`).
        let activity = count_activity(
            tree.files.iter().map(|f| f.timemodified),
            &job.course.academic_year,
        );
        self.with_store(|store| store.save_activity(course_id, activity))?;
        let new_links =
            self.with_store(|store| store.save_links(course_id, &tree.external_links, self.now))?;
        Ok(CourseSummary {
            id: course_id,
            name: job.course.name.clone(),
            folder: job.folder.to_owned(),
            report: outcome.report,
            external_links: tree.external_links,
            new_links,
            error: None,
        })
    }

    fn with_store<T>(
        &self,
        f: impl FnOnce(&mut Store) -> Result<T, StoreError>,
    ) -> Result<T, SyncError> {
        let mut store = self.store.lock().map_err(|_| SyncError::StoreLocked)?;
        Ok(f(&mut store)?)
    }
}

fn failed_course(job: &CourseJob<'_>, message: String) -> CourseSummary {
    CourseSummary {
        id: job.course.id,
        name: job.course.name.clone(),
        folder: job.folder.to_owned(),
        report: CourseReport::default(),
        external_links: Vec::new(),
        new_links: Vec::new(),
        error: Some(message),
    }
}

#[cfg(test)]
#[path = "run_tests.rs"]
mod tests;
