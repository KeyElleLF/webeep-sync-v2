//! State of a course (§6.1): where it shows up and whether it is synced by default.
//!
//! `upcoming` extends the original model: a 2nd-semester course seen during the 1st
//! semester (usually still empty). It is synced, so its material arrives as soon as it is
//! published.

use serde::{Deserialize, Serialize};

use super::semester::Semester;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CourseState {
    /// Semester in progress.
    #[default]
    Current,
    /// Next semester of the current academic year.
    Upcoming,
    /// 1st-semester course during the 2nd semester: exam possibly still to take.
    ExamPending,
    /// Marked as passed by the user.
    Passed,
    /// Previous academic years (or community courses).
    Past,
    /// Excluded by the user.
    Ignored,
}

impl CourseState {
    /// Whether the course is synced when the user made no explicit choice.
    pub fn synced_by_default(self) -> bool {
        matches!(self, Self::Current | Self::Upcoming | Self::ExamPending)
    }
}

/// States the user sets by hand on a whole teaching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ManualState {
    Passed,
    Ignored,
}

pub fn derive_state(
    manual: Option<ManualState>,
    current_year: bool,
    semester: Option<Semester>,
    period: Semester,
) -> CourseState {
    match manual {
        Some(ManualState::Passed) => return CourseState::Passed,
        Some(ManualState::Ignored) => return CourseState::Ignored,
        None => {}
    }
    if !current_year {
        return CourseState::Past;
    }
    match (semester, period) {
        (Some(Semester::First), Semester::Second) => CourseState::ExamPending,
        (Some(Semester::Second), Semester::First) => CourseState::Upcoming,
        _ => CourseState::Current,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Semester::{First, Second};

    #[test]
    fn manual_states_win() {
        assert_eq!(
            derive_state(Some(ManualState::Passed), true, Some(First), First),
            CourseState::Passed
        );
        assert_eq!(
            derive_state(Some(ManualState::Ignored), false, None, First),
            CourseState::Ignored
        );
    }

    #[test]
    fn other_years_are_past() {
        assert_eq!(
            derive_state(None, false, Some(First), First),
            CourseState::Past
        );
    }

    #[test]
    fn current_year_depends_on_the_semester() {
        assert_eq!(
            derive_state(None, true, Some(First), First),
            CourseState::Current
        );
        assert_eq!(
            derive_state(None, true, Some(Second), First),
            CourseState::Upcoming
        );
        assert_eq!(
            derive_state(None, true, Some(First), Second),
            CourseState::ExamPending
        );
        assert_eq!(
            derive_state(None, true, Some(Second), Second),
            CourseState::Current
        );
        assert_eq!(derive_state(None, true, None, Second), CourseState::Current);
    }

    #[test]
    fn only_active_states_are_synced_by_default() {
        let synced: Vec<CourseState> = [
            CourseState::Current,
            CourseState::Upcoming,
            CourseState::ExamPending,
            CourseState::Passed,
            CourseState::Past,
            CourseState::Ignored,
        ]
        .into_iter()
        .filter(|s| s.synced_by_default())
        .collect();
        assert_eq!(
            synced,
            [
                CourseState::Current,
                CourseState::Upcoming,
                CourseState::ExamPending
            ]
        );
    }
}
