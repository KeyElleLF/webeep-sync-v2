//! Per-teaching choices (passed, ignored, semester), course activity, pending cleanups and
//! the courses seen so far (to notice archived ones).

use std::collections::HashMap;

use rusqlite::params;

use super::{Store, StoreError};
use crate::courses::semester::{Activity, Semester};
use crate::courses::state::ManualState;

const PASSED: &str = "passed";
const IGNORED: &str = "ignored";

/// What the user decided for a teaching (all its courses, every academic year).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TeachingPrefs {
    pub state: Option<ManualState>,
    pub semester: Option<Semester>,
}

/// A course we listed before, to notice when it disappears from the enrolments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnownCourse {
    pub course_id: u64,
    pub teaching_key: String,
    pub academic_year: String,
    pub name: String,
    pub dismissed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingCleanup {
    pub course_id: u64,
    pub name: String,
}

impl Store {
    pub fn teaching_prefs(&self) -> Result<HashMap<String, TeachingPrefs>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT key, state, semester FROM teaching")?;
        let rows = stmt.query_map([], |row| {
            let state: Option<String> = row.get(1)?;
            let semester: Option<u8> = row.get(2)?;
            let prefs = TeachingPrefs {
                state: state.as_deref().and_then(parse_state),
                semester: semester.and_then(Semester::from_number),
            };
            Ok((row.get::<_, String>(0)?, prefs))
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    pub fn set_teaching_state(
        &self,
        key: &str,
        state: Option<ManualState>,
    ) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO teaching (key, state) VALUES (?1, ?2) \
             ON CONFLICT(key) DO UPDATE SET state = excluded.state",
            params![key, state.map(state_name)],
        )?;
        Ok(())
    }

    pub fn set_teaching_semester(
        &self,
        key: &str,
        semester: Option<Semester>,
    ) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO teaching (key, semester) VALUES (?1, ?2) \
             ON CONFLICT(key) DO UPDATE SET semester = excluded.semester",
            params![key, semester.map(Semester::as_number)],
        )?;
        Ok(())
    }

    /// Drops an explicit selection, so the course follows its state again.
    pub fn clear_course_selection(&self, course_id: u64) -> Result<(), StoreError> {
        self.conn.execute(
            "DELETE FROM course_selection WHERE course_id = ?1",
            params![course_id],
        )?;
        Ok(())
    }

    pub fn course_activity(&self) -> Result<HashMap<u64, Activity>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT course_id, first, second FROM course_activity")?;
        let rows = stmt.query_map([], |row| {
            let activity = Activity {
                first: row.get(1)?,
                second: row.get(2)?,
            };
            Ok((row.get(0)?, activity))
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    pub fn save_activity(&self, course_id: u64, activity: Activity) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO course_activity (course_id, first, second) \
             VALUES (?1, ?2, ?3)",
            params![course_id, activity.first, activity.second],
        )?;
        Ok(())
    }

    pub fn add_pending_cleanup(
        &self,
        course_id: u64,
        name: &str,
        created: i64,
    ) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO pending_cleanup (course_id, name, created) VALUES (?1, ?2, ?3)",
            params![course_id, name, created],
        )?;
        Ok(())
    }

    pub fn remove_pending_cleanup(&self, course_id: u64) -> Result<(), StoreError> {
        self.conn.execute(
            "DELETE FROM pending_cleanup WHERE course_id = ?1",
            params![course_id],
        )?;
        Ok(())
    }

    pub fn pending_cleanups(&self) -> Result<Vec<PendingCleanup>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT course_id, name FROM pending_cleanup ORDER BY created, course_id")?;
        let rows = stmt.query_map([], |row| {
            Ok(PendingCleanup {
                course_id: row.get(0)?,
                name: row.get(1)?,
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    pub fn known_courses(&self) -> Result<Vec<KnownCourse>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT course_id, teaching_key, academic_year, name, dismissed FROM known_course",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(KnownCourse {
                course_id: row.get(0)?,
                teaching_key: row.get(1)?,
                academic_year: row.get(2)?,
                name: row.get(3)?,
                dismissed: row.get(4)?,
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Records the listed courses (keeping the `dismissed` flag of known ones).
    pub fn remember_courses(&mut self, courses: &[KnownCourse]) -> Result<(), StoreError> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO known_course (course_id, teaching_key, academic_year, name) \
                 VALUES (?1, ?2, ?3, ?4) ON CONFLICT(course_id) DO UPDATE SET \
                 teaching_key = excluded.teaching_key, academic_year = excluded.academic_year, \
                 name = excluded.name",
            )?;
            for c in courses {
                stmt.execute(params![
                    c.course_id,
                    c.teaching_key,
                    c.academic_year,
                    c.name
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn dismiss_known_course(&self, course_id: u64) -> Result<(), StoreError> {
        self.conn.execute(
            "UPDATE known_course SET dismissed = 1 WHERE course_id = ?1",
            params![course_id],
        )?;
        Ok(())
    }
}

fn state_name(state: ManualState) -> &'static str {
    match state {
        ManualState::Passed => PASSED,
        ManualState::Ignored => IGNORED,
    }
}

fn parse_state(name: &str) -> Option<ManualState> {
    match name {
        PASSED => Some(ManualState::Passed),
        IGNORED => Some(ManualState::Ignored),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn teaching_state_and_semester_are_independent() {
        let store = Store::open_in_memory().unwrap();
        store
            .set_teaching_semester("056892", Some(Semester::Second))
            .unwrap();
        store
            .set_teaching_state("056892", Some(ManualState::Passed))
            .unwrap();
        store
            .set_teaching_state("097683", Some(ManualState::Ignored))
            .unwrap();
        store.set_teaching_state("097683", None).unwrap();

        let prefs = store.teaching_prefs().unwrap();
        assert_eq!(
            prefs["056892"],
            TeachingPrefs {
                state: Some(ManualState::Passed),
                semester: Some(Semester::Second)
            }
        );
        assert_eq!(prefs["097683"], TeachingPrefs::default());
    }

    #[test]
    fn activity_and_pending_cleanups_round_trip() {
        let store = Store::open_in_memory().unwrap();
        let activity = Activity {
            first: 3,
            second: 1,
        };
        store.save_activity(7, activity).unwrap();
        assert_eq!(store.course_activity().unwrap()[&7], activity);

        store.add_pending_cleanup(2, "B", 20).unwrap();
        store.add_pending_cleanup(1, "A", 10).unwrap();
        let names: Vec<String> = store
            .pending_cleanups()
            .unwrap()
            .into_iter()
            .map(|p| p.name)
            .collect();
        assert_eq!(names, ["A", "B"]);
        store.remove_pending_cleanup(1).unwrap();
        assert_eq!(store.pending_cleanups().unwrap().len(), 1);
    }

    #[test]
    fn remembering_keeps_dismissals() {
        let mut store = Store::open_in_memory().unwrap();
        let course = KnownCourse {
            course_id: 1,
            teaching_key: "056892".into(),
            academic_year: "2026-27".into(),
            name: "DATA MINING".into(),
            dismissed: false,
        };
        store
            .remember_courses(std::slice::from_ref(&course))
            .unwrap();
        store.dismiss_known_course(1).unwrap();
        store.remember_courses(&[course]).unwrap();
        assert!(store.known_courses().unwrap()[0].dismissed);
    }

    #[test]
    fn clearing_a_selection_restores_the_default() {
        let store = Store::open_in_memory().unwrap();
        store.set_course_selected(5, true).unwrap();
        store.clear_course_selection(5).unwrap();
        assert!(store.course_selection().unwrap().is_empty());
    }
}
