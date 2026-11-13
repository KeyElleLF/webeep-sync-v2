//! The imported study plan: only teaching codes, names and semesters (nothing personal).

use rusqlite::params;

use super::{Store, StoreError};
use crate::courses::semester::Semester;
use crate::courses::study_plan::PlanEntry;

const IMPORTED_KEY: &str = "study_plan_imported";

impl Store {
    /// Replaces the plan. Manual semester choices are dropped: importing the plan is the
    /// user's way of setting every semester at once (they can still change one by hand).
    pub fn replace_study_plan(
        &mut self,
        entries: &[PlanEntry],
        now: i64,
    ) -> Result<(), StoreError> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM study_plan", [])?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO study_plan (code, name, year, semester, position) \
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )?;
            for (position, e) in entries.iter().enumerate() {
                stmt.execute(params![
                    e.code,
                    e.name,
                    e.year,
                    e.semester.as_number(),
                    position as i64
                ])?;
            }
        }
        tx.execute("UPDATE teaching SET semester = NULL", [])?;
        tx.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![IMPORTED_KEY, now.to_string()],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn study_plan(&self) -> Result<Vec<PlanEntry>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT code, name, year, semester FROM study_plan ORDER BY position")?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i32>(2)?,
                row.get::<_, u8>(3)?,
            ))
        })?;
        let mut entries = Vec::new();
        for row in rows {
            let (code, name, year, semester) = row?;
            if let Some(semester) = Semester::from_number(semester) {
                entries.push(PlanEntry {
                    code,
                    name,
                    year,
                    semester,
                });
            }
        }
        Ok(entries)
    }

    /// Unix time of the last import, if a plan is loaded.
    pub fn study_plan_imported(&self) -> Result<Option<i64>, StoreError> {
        Ok(self
            .setting(IMPORTED_KEY)?
            .and_then(|value| value.parse().ok()))
    }

    /// Forgets the plan: semesters go back to the automatic guess.
    pub fn clear_study_plan(&mut self) -> Result<(), StoreError> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM study_plan", [])?;
        tx.execute("DELETE FROM settings WHERE key = ?1", params![IMPORTED_KEY])?;
        tx.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(code: &str, semester: Semester) -> PlanEntry {
        PlanEntry {
            code: code.to_owned(),
            name: format!("CORSO {code}"),
            year: 2026,
            semester,
        }
    }

    #[test]
    fn a_new_plan_replaces_the_old_one_in_order() {
        let mut store = Store::open_in_memory().unwrap();
        assert!(store.study_plan().unwrap().is_empty());
        assert_eq!(store.study_plan_imported().unwrap(), None);

        store
            .replace_study_plan(&[entry("000001", Semester::First)], 100)
            .unwrap();
        let plan = [
            entry("056892", Semester::First),
            entry("097683", Semester::Second),
        ];
        store.replace_study_plan(&plan, 200).unwrap();

        assert_eq!(store.study_plan().unwrap(), plan);
        assert_eq!(store.study_plan_imported().unwrap(), Some(200));
    }

    #[test]
    fn importing_drops_manual_semesters_but_keeps_other_choices() {
        let mut store = Store::open_in_memory().unwrap();
        store
            .set_teaching_semester("097683", Some(Semester::First))
            .unwrap();
        store
            .set_teaching_state("097683", Some(crate::courses::state::ManualState::Passed))
            .unwrap();

        store
            .replace_study_plan(&[entry("097683", Semester::Second)], 100)
            .unwrap();

        let prefs = store.teaching_prefs().unwrap()["097683"];
        assert_eq!(prefs.semester, None);
        assert!(prefs.state.is_some(), "passed stays passed");
    }

    #[test]
    fn clearing_forgets_the_plan() {
        let mut store = Store::open_in_memory().unwrap();
        store
            .replace_study_plan(&[entry("056892", Semester::First)], 100)
            .unwrap();
        store.clear_study_plan().unwrap();
        assert!(store.study_plan().unwrap().is_empty());
        assert_eq!(store.study_plan_imported().unwrap(), None);
    }
}
