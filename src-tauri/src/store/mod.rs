//! Local state in SQLite: the sync baseline, manual course selection and settings.
//!
//! Schema changes are append-only migrations tracked with `PRAGMA user_version`.

pub mod courses;
pub mod history;
pub mod links;
pub mod notify;
pub mod settings;
pub mod study_plan;

use std::collections::HashMap;
use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};
use thiserror::Error;

use crate::sync::plan::BaselineEntry;

/// Each entry upgrades the schema by one version. Never edit a released migration.
const MIGRATIONS: &[&str] = &[
    r"
    CREATE TABLE baseline (
        course_id           INTEGER NOT NULL,
        remote_key          TEXT    NOT NULL,
        local_path          TEXT    NOT NULL,
        remote_timemodified INTEGER NOT NULL,
        remote_filesize     INTEGER NOT NULL,
        local_size          INTEGER NOT NULL,
        local_mtime         INTEGER NOT NULL,
        PRIMARY KEY (course_id, remote_key)
    );
    CREATE TABLE course_selection (
        course_id INTEGER PRIMARY KEY,
        selected  INTEGER NOT NULL
    );
    CREATE TABLE course_folder (
        course_id INTEGER PRIMARY KEY,
        folder    TEXT    NOT NULL
    );
    CREATE TABLE settings (
        key   TEXT PRIMARY KEY,
        value TEXT NOT NULL
    );
",
    r"
    CREATE TABLE teaching (
        key       TEXT PRIMARY KEY,
        state     TEXT,
        semester  INTEGER
    );
    CREATE TABLE course_activity (
        course_id INTEGER PRIMARY KEY,
        first     INTEGER NOT NULL,
        second    INTEGER NOT NULL
    );
    CREATE TABLE pending_cleanup (
        course_id INTEGER PRIMARY KEY,
        name      TEXT    NOT NULL,
        created   INTEGER NOT NULL
    );
    CREATE TABLE known_course (
        course_id     INTEGER PRIMARY KEY,
        teaching_key  TEXT    NOT NULL,
        academic_year TEXT    NOT NULL,
        name          TEXT    NOT NULL,
        dismissed     INTEGER NOT NULL DEFAULT 0
    );
",
    r"
    CREATE TABLE forum_state (
        forum_id       INTEGER PRIMARY KEY,
        numdiscussions INTEGER NOT NULL
    );
    CREATE TABLE announcement (
        id          INTEGER PRIMARY KEY,
        course_id   INTEGER NOT NULL,
        subject     TEXT    NOT NULL,
        message     TEXT    NOT NULL,
        author      TEXT    NOT NULL,
        created     INTEGER NOT NULL,
        modified    INTEGER NOT NULL,
        pinned      INTEGER NOT NULL,
        attachments TEXT    NOT NULL,
        read        INTEGER NOT NULL DEFAULT 0
    );
    CREATE INDEX announcement_course ON announcement (course_id);
    CREATE TABLE deadline_reminder (
        event_id INTEGER PRIMARY KEY,
        reminded INTEGER NOT NULL
    );
    CREATE TABLE course_mute (
        course_id INTEGER PRIMARY KEY
    );
",
    r"
    CREATE TABLE course_link (
        id         INTEGER PRIMARY KEY,
        course_id  INTEGER NOT NULL,
        module_id  INTEGER NOT NULL,
        url        TEXT    NOT NULL,
        name       TEXT    NOT NULL,
        section    TEXT    NOT NULL,
        position   INTEGER NOT NULL,
        first_seen INTEGER NOT NULL,
        seen       INTEGER NOT NULL DEFAULT 0,
        UNIQUE (course_id, module_id, url)
    );
    CREATE TABLE course_link_import (
        course_id INTEGER PRIMARY KEY
    );
",
    r"
    CREATE TABLE study_plan (
        code     TEXT    NOT NULL,
        name     TEXT    NOT NULL,
        year     INTEGER NOT NULL,
        semester INTEGER NOT NULL,
        position INTEGER NOT NULL
    );
",
    r"
    CREATE TABLE sync_run (
        id       INTEGER PRIMARY KEY AUTOINCREMENT,
        finished INTEGER NOT NULL
    );
    CREATE TABLE sync_file (
        run_id    INTEGER NOT NULL,
        course_id INTEGER NOT NULL,
        kind      TEXT    NOT NULL,
        path      TEXT    NOT NULL
    );
    CREATE INDEX sync_file_run ON sync_file (run_id);
",
];

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("errore del database locale: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error(
        "il database locale è di una versione più recente dell'app ({0}): aggiorna WeBeep Sync"
    )]
    TooNew(i64),
}

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        Self::init(Connection::open(path)?)
    }

    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self, StoreError> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(mut conn: Connection) -> Result<Self, StoreError> {
        conn.pragma_update(None, "journal_mode", "WAL")?;
        migrate(&mut conn)?;
        Ok(Self { conn })
    }

    /// Baseline of one course, keyed by remote key.
    pub fn baseline(&self, course_id: u64) -> Result<HashMap<String, BaselineEntry>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT remote_key, local_path, remote_timemodified, remote_filesize, local_size, \
             local_mtime FROM baseline WHERE course_id = ?1",
        )?;
        let rows = stmt.query_map(params![course_id], |row| {
            let entry = BaselineEntry {
                local_path: row.get(1)?,
                remote_timemodified: row.get(2)?,
                remote_filesize: row.get(3)?,
                local_size: row.get(4)?,
                local_mtime: row.get(5)?,
            };
            Ok((row.get::<_, String>(0)?, entry))
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Inserts or replaces baseline entries of one course in a single transaction.
    pub fn save_baseline(
        &mut self,
        course_id: u64,
        entries: &[(String, BaselineEntry)],
    ) -> Result<(), StoreError> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO baseline (course_id, remote_key, local_path, \
                 remote_timemodified, remote_filesize, local_size, local_mtime) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            )?;
            for (key, e) in entries {
                stmt.execute(params![
                    course_id,
                    key,
                    e.local_path,
                    e.remote_timemodified,
                    e.remote_filesize,
                    e.local_size,
                    e.local_mtime
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Forgets some files of a course (moved to the bin by a cleanup).
    pub fn forget_baseline(&mut self, course_id: u64, keys: &[String]) -> Result<(), StoreError> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt =
                tx.prepare("DELETE FROM baseline WHERE course_id = ?1 AND remote_key = ?2")?;
            for key in keys {
                stmt.execute(params![course_id, key])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Forgets what was downloaded for a course (its folder now points elsewhere).
    pub fn clear_baseline(&self, course_id: u64) -> Result<(), StoreError> {
        self.conn.execute(
            "DELETE FROM baseline WHERE course_id = ?1",
            params![course_id],
        )?;
        Ok(())
    }

    /// Courses the user explicitly selected (`true`) or excluded (`false`). Courses missing
    /// here follow the default filter.
    pub fn course_selection(&self) -> Result<HashMap<u64, bool>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT course_id, selected FROM course_selection")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    pub fn set_course_selected(&self, course_id: u64, selected: bool) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO course_selection (course_id, selected) VALUES (?1, ?2)",
            params![course_id, selected],
        )?;
        Ok(())
    }

    /// Folder names already given to courses (relative to the sync root).
    pub fn course_folders(&self) -> Result<HashMap<u64, String>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT course_id, folder FROM course_folder")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    pub fn set_course_folder(&self, course_id: u64, folder: &str) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO course_folder (course_id, folder) VALUES (?1, ?2)",
            params![course_id, folder],
        )?;
        Ok(())
    }

    /// Version of the database schema (number of migrations applied).
    pub fn schema_version(&self) -> Result<i64, StoreError> {
        Ok(self
            .conn
            .pragma_query_value(None, "user_version", |row| row.get(0))?)
    }

    /// Files the app downloaded, and in how many courses (for the diagnostic file).
    pub fn baseline_counts(&self) -> Result<(u64, u64), StoreError> {
        Ok(self.conn.query_row(
            "SELECT COUNT(*), COUNT(DISTINCT course_id) FROM baseline",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?)
    }

    pub fn setting(&self, key: &str) -> Result<Option<String>, StoreError> {
        Ok(self
            .conn
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()?)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }
}

fn migrate(conn: &mut Connection) -> Result<(), StoreError> {
    let current: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    let latest = i64::try_from(MIGRATIONS.len()).unwrap_or(i64::MAX);
    if current > latest {
        return Err(StoreError::TooNew(current));
    }
    let tx = conn.transaction()?;
    for (version, sql) in MIGRATIONS.iter().enumerate().skip(current as usize) {
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", version as i64 + 1)?;
    }
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(path: &str, size: u64) -> BaselineEntry {
        BaselineEntry {
            local_path: path.to_owned(),
            remote_timemodified: 1_700_000_000,
            remote_filesize: size,
            local_size: size,
            local_mtime: 1_700_000_000,
        }
    }

    #[test]
    fn baseline_round_trips_per_course() {
        let mut store = Store::open_in_memory().unwrap();
        store
            .save_baseline(1, &[("1:/a.pdf".to_owned(), entry("Materiali/a.pdf", 10))])
            .unwrap();
        store
            .save_baseline(2, &[("2:/b.pdf".to_owned(), entry("b.pdf", 20))])
            .unwrap();

        let first = store.baseline(1).unwrap();
        assert_eq!(first.len(), 1);
        assert_eq!(first["1:/a.pdf"], entry("Materiali/a.pdf", 10));
        assert!(store.baseline(3).unwrap().is_empty());
    }

    #[test]
    fn counts_and_schema_version_for_diagnostics() {
        let mut store = Store::open_in_memory().unwrap();
        assert_eq!(store.baseline_counts().unwrap(), (0, 0));
        store
            .save_baseline(
                1,
                &[
                    ("a".to_owned(), entry("a.pdf", 1)),
                    ("b".to_owned(), entry("b.pdf", 1)),
                ],
            )
            .unwrap();
        store
            .save_baseline(2, &[("c".to_owned(), entry("c.pdf", 1))])
            .unwrap();
        assert_eq!(store.baseline_counts().unwrap(), (3, 2));
        assert_eq!(store.schema_version().unwrap(), MIGRATIONS.len() as i64);
    }

    #[test]
    fn saving_an_existing_key_replaces_it() {
        let mut store = Store::open_in_memory().unwrap();
        let key = "1:/a.pdf".to_owned();
        store
            .save_baseline(1, &[(key.clone(), entry("a.pdf", 10))])
            .unwrap();
        store
            .save_baseline(1, &[(key.clone(), entry("nuova/a.pdf", 11))])
            .unwrap();
        assert_eq!(store.baseline(1).unwrap()[&key], entry("nuova/a.pdf", 11));
    }

    #[test]
    fn settings_and_selection_round_trip() {
        let store = Store::open_in_memory().unwrap();
        assert_eq!(store.setting("sync_root").unwrap(), None);
        store.set_setting("sync_root", "C:\\polimi").unwrap();
        store.set_setting("sync_root", "D:\\polimi").unwrap();
        assert_eq!(
            store.setting("sync_root").unwrap().as_deref(),
            Some("D:\\polimi")
        );

        store.set_course_selected(24741, true).unwrap();
        store.set_course_selected(23056, false).unwrap();
        store.set_course_selected(23056, true).unwrap();
        let selection = store.course_selection().unwrap();
        assert_eq!(selection, HashMap::from([(24741, true), (23056, true)]));

        store.set_course_folder(24741, "DATA MINING").unwrap();
        assert_eq!(
            store.course_folders().unwrap(),
            HashMap::from([(24741, "DATA MINING".to_owned())])
        );
    }

    #[test]
    fn reopening_a_file_keeps_data_and_schema() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.sqlite");
        {
            let store = Store::open(&path).unwrap();
            store.set_setting("k", "v").unwrap();
        }
        let store = Store::open(&path).unwrap();
        assert_eq!(store.setting("k").unwrap().as_deref(), Some("v"));
    }

    #[test]
    fn refuses_a_database_from_a_newer_app() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.sqlite");
        {
            let conn = Connection::open(&path).unwrap();
            conn.pragma_update(None, "user_version", 99).unwrap();
        }
        assert!(matches!(Store::open(&path), Err(StoreError::TooNew(99))));
    }
}
