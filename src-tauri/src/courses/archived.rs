//! Noticing a probably passed exam (§6.5): the assigned course of a teaching disappears from
//! the enrolments (archived after the exam) while a self-enrolled course with the same code
//! is still there. This is only a suggestion: nothing changes until the user confirms.

use std::collections::HashSet;

use serde::Serialize;

use super::CourseInfo;
use super::state::CourseState;
use crate::store::courses::KnownCourse;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PassedSuggestion {
    /// The course that disappeared.
    pub course_id: u64,
    pub name: String,
    pub teaching_key: String,
}

pub fn passed_suggestions(
    known: &[KnownCourse],
    listed: &[CourseInfo],
    current_year: &str,
) -> Vec<PassedSuggestion> {
    let listed_ids: HashSet<u64> = listed.iter().map(|c| c.id).collect();
    let mut seen_keys = HashSet::new();
    known
        .iter()
        .filter(|k| !k.dismissed && k.academic_year == current_year)
        .filter(|k| !listed_ids.contains(&k.course_id))
        .filter(|k| {
            let siblings: Vec<&CourseInfo> = listed
                .iter()
                .filter(|c| c.teaching_key == k.teaching_key)
                .collect();
            !siblings.is_empty() && siblings.iter().all(|c| c.state != CourseState::Passed)
        })
        .filter(|k| seen_keys.insert(k.teaching_key.clone()))
        .map(|k| PassedSuggestion {
            course_id: k.course_id,
            name: k.name.clone(),
            teaching_key: k.teaching_key.clone(),
        })
        .collect()
}

/// What to remember about the listed courses.
pub fn known_from(listed: &[CourseInfo]) -> Vec<KnownCourse> {
    listed
        .iter()
        .map(|c| KnownCourse {
            course_id: c.id,
            teaching_key: c.teaching_key.clone(),
            academic_year: c.academic_year.clone(),
            name: c.name.clone(),
            dismissed: false,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const YEAR: &str = "2026-27";

    fn listed(id: u64, key: &str, state: CourseState) -> CourseInfo {
        CourseInfo {
            id,
            name: format!("corso {id}"),
            teaching_key: key.to_owned(),
            academic_year: YEAR.to_owned(),
            state,
            ..CourseInfo::default()
        }
    }

    fn known(id: u64, key: &str, year: &str, dismissed: bool) -> KnownCourse {
        KnownCourse {
            course_id: id,
            teaching_key: key.to_owned(),
            academic_year: year.to_owned(),
            name: "DATA MINING".to_owned(),
            dismissed,
        }
    }

    #[test]
    fn archived_assigned_course_with_a_remaining_sibling_is_suggested() {
        let now = [listed(2, "056892", CourseState::Current)];
        let before = [
            known(1, "056892", YEAR, false),
            known(2, "056892", YEAR, false),
        ];

        let suggestions = passed_suggestions(&before, &now, YEAR);

        assert_eq!(
            suggestions,
            [PassedSuggestion {
                course_id: 1,
                name: "DATA MINING".to_owned(),
                teaching_key: "056892".to_owned()
            }]
        );
    }

    #[test]
    fn no_suggestion_without_a_sibling_or_when_dismissed_or_old_or_passed() {
        let now = [
            listed(2, "056892", CourseState::Current),
            listed(4, "097683", CourseState::Passed),
        ];
        let before = [
            known(9, "000001", YEAR, false), // no sibling left: just unenrolled
            known(1, "056892", YEAR, true),  // already dismissed
            known(5, "056892", "2025-26", false), // previous year: expected to go
            known(3, "097683", YEAR, false), // already marked passed
        ];
        assert!(passed_suggestions(&before, &now, YEAR).is_empty());
    }
}
