//! What each sync brought, course by course, kept for 30 days for the "Oggi" page: new and
//! updated files and the copies saved next to files the user had changed.

use rusqlite::params;
use serde::Serialize;

use super::{Store, StoreError};
use crate::sync::execute::CourseReport;

/// How long the history is kept.
pub const HISTORY_DAYS: i64 = 30;
const DAY: i64 = 86_400;

/// What happened to one file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FileChange {
    /// New on WeBeep, or deleted locally and downloaded again.
    Downloaded,
    Updated,
    /// The new version saved next to a file the user had changed.
    Conflict,
}

impl FileChange {
    fn as_str(self) -> &'static str {
        match self {
            Self::Downloaded => "downloaded",
            Self::Updated => "updated",
            Self::Conflict => "conflict",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "downloaded" => Some(Self::Downloaded),
            "updated" => Some(Self::Updated),
            "conflict" => Some(Self::Conflict),
            _ => None,
        }
    }
}

/// One file of a sync. The path is inside the course folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryFile {
    pub course_id: u64,
    pub change: FileChange,
    pub path: String,
}

/// A sync that brought something: when it ended (Unix seconds) and its files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncRecord {
    pub finished: i64,
    pub files: Vec<HistoryFile>,
}

/// The files a course report lists, in the order of the summary.
pub fn history_files(course_id: u64, report: &CourseReport) -> Vec<HistoryFile> {
    let entries = [
        (FileChange::Downloaded, &report.downloaded),
        (FileChange::Updated, &report.updated),
        (FileChange::Conflict, &report.conflicts),
    ];
    entries
        .into_iter()
        .flat_map(|(change, paths)| {
            paths.iter().map(move |path| HistoryFile {
                course_id,
                change,
                path: path.clone(),
            })
        })
        .collect()
}

impl Store {
    /// Saves what a sync brought (a sync that brought nothing leaves no trace) and forgets the
    /// syncs older than 30 days.
    pub fn record_sync(&mut self, finished: i64, files: &[HistoryFile]) -> Result<(), StoreError> {
        let tx = self.conn.transaction()?;
        let oldest = finished - HISTORY_DAYS * DAY;
        tx.execute(
            "DELETE FROM sync_file WHERE run_id IN (SELECT id FROM sync_run WHERE finished < ?1)",
            params![oldest],
        )?;
        tx.execute("DELETE FROM sync_run WHERE finished < ?1", params![oldest])?;
        if !files.is_empty() {
            tx.execute(
                "INSERT INTO sync_run (finished) VALUES (?1)",
                params![finished],
            )?;
            let run = tx.last_insert_rowid();
            let mut insert = tx.prepare(
                "INSERT INTO sync_file (run_id, course_id, kind, path) VALUES (?1, ?2, ?3, ?4)",
            )?;
            for file in files {
                insert.execute(params![
                    run,
                    file.course_id,
                    file.change.as_str(),
                    file.path
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// The syncs that ended at `since` or later, newest first, each with its files in the order
    /// they were saved.
    pub fn sync_history(&self, since: i64) -> Result<Vec<SyncRecord>, StoreError> {
        let mut statement = self.conn.prepare(
            "SELECT r.id, r.finished, f.course_id, f.kind, f.path \
             FROM sync_run r JOIN sync_file f ON f.run_id = r.id \
             WHERE r.finished >= ?1 ORDER BY r.finished DESC, r.id DESC, f.rowid",
        )?;
        let rows = statement.query_map(params![since], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, u64>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        })?;
        let mut records: Vec<SyncRecord> = Vec::new();
        let mut current = None;
        for row in rows {
            let (run, finished, course_id, kind, path) = row?;
            // A kind written by a newer version of the app is skipped, not an error.
            let Some(change) = FileChange::parse(&kind) else {
                continue;
            };
            if current != Some(run) {
                records.push(SyncRecord {
                    finished,
                    files: Vec::new(),
                });
                current = Some(run);
            }
            if let Some(record) = records.last_mut() {
                record.files.push(HistoryFile {
                    course_id,
                    change,
                    path,
                });
            }
        }
        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(course_id: u64, change: FileChange, path: &str) -> HistoryFile {
        HistoryFile {
            course_id,
            change,
            path: path.to_owned(),
        }
    }

    #[test]
    fn a_sync_that_brought_nothing_leaves_no_trace() {
        let mut store = Store::open_in_memory().unwrap();
        store.record_sync(1_000, &[]).unwrap();
        assert!(store.sync_history(0).unwrap().is_empty());
    }

    #[test]
    fn syncs_come_back_newest_first_with_their_files_in_order() {
        let mut store = Store::open_in_memory().unwrap();
        let first = [file(
            1,
            FileChange::Downloaded,
            "Syllabus/08 Clustering.pdf",
        )];
        let second = [
            file(1, FileChange::Downloaded, "Exams/Esame.pdf"),
            file(4, FileChange::Updated, "Lab 5.pdf"),
            file(
                4,
                FileChange::Conflict,
                "Lezione 5 (WeBeep aggiornato 2026-11-24).pdf",
            ),
        ];
        store.record_sync(1_000, &first).unwrap();
        store.record_sync(2_000, &second).unwrap();
        let history = store.sync_history(0).unwrap();
        assert_eq!(
            history,
            vec![
                SyncRecord {
                    finished: 2_000,
                    files: second.to_vec()
                },
                SyncRecord {
                    finished: 1_000,
                    files: first.to_vec()
                },
            ]
        );
    }

    #[test]
    fn only_syncs_since_the_given_time_are_read() {
        let mut store = Store::open_in_memory().unwrap();
        store
            .record_sync(1_000, &[file(1, FileChange::Downloaded, "a.pdf")])
            .unwrap();
        store
            .record_sync(5_000, &[file(1, FileChange::Downloaded, "b.pdf")])
            .unwrap();
        let history = store.sync_history(5_000).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].files[0].path, "b.pdf");
    }

    #[test]
    fn syncs_older_than_thirty_days_are_forgotten() {
        let mut store = Store::open_in_memory().unwrap();
        let old = 1_000;
        let now = old + HISTORY_DAYS * DAY + 1;
        store
            .record_sync(old, &[file(1, FileChange::Downloaded, "old.pdf")])
            .unwrap();
        // Even a sync that brought nothing clears the old ones.
        store.record_sync(now, &[]).unwrap();
        assert!(store.sync_history(0).unwrap().is_empty());
    }

    #[test]
    fn a_report_lists_new_then_updated_files_then_conflict_copies() {
        let report = CourseReport {
            downloaded: vec!["new.pdf".to_owned()],
            updated: vec!["changed.pdf".to_owned()],
            conflicts: vec!["copy (WeBeep aggiornato 2026-11-24).pdf".to_owned()],
            kept_local: vec!["annotated.pdf".to_owned()],
            ..CourseReport::default()
        };
        assert_eq!(
            history_files(7, &report),
            vec![
                file(7, FileChange::Downloaded, "new.pdf"),
                file(7, FileChange::Updated, "changed.pdf"),
                file(
                    7,
                    FileChange::Conflict,
                    "copy (WeBeep aggiornato 2026-11-24).pdf"
                ),
            ]
        );
    }
}
