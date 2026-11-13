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

/// Where the academic year of a course stands today.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CourseYear {
    /// The current academic year.
    Current,
    /// Last academic year, during its September exam session: the new year has started
    /// (lessons begin mid-September) but exams of last year are still being taken.
    LastSession,
    /// Older, or last year after September.
    Past,
}

pub fn derive_state(
    manual: Option<ManualState>,
    year: CourseYear,
    semester: Option<Semester>,
    period: Semester,
) -> CourseState {
    match manual {
        Some(ManualState::Passed) => return CourseState::Passed,
        Some(ManualState::Ignored) => return CourseState::Ignored,
        None => {}
    }
    match year {
        CourseYear::Current => {}
        // Not marked as passed: its exam may still be taken in September.
        CourseYear::LastSession => return CourseState::ExamPending,
        CourseYear::Past => return CourseState::Past,
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
            derive_state(
                Some(ManualState::Passed),
                CourseYear::Current,
                Some(First),
                First
            ),
            CourseState::Passed
        );
        assert_eq!(
            derive_state(Some(ManualState::Ignored), CourseYear::Past, None, First),
            CourseState::Ignored
        );
    }

    #[test]
    fn other_years_are_past() {
        assert_eq!(
            derive_state(None, CourseYear::Past, Some(First), First),
            CourseState::Past
        );
    }

    #[test]
    fn last_years_courses_wait_for_the_september_exam_session() {
        let pending = CourseState::ExamPending;
        assert_eq!(
            derive_state(None, CourseYear::LastSession, Some(First), First),
            pending
        );
        assert_eq!(
            derive_state(None, CourseYear::LastSession, Some(Second), First),
            pending
        );
        assert_eq!(
            derive_state(None, CourseYear::LastSession, None, First),
            pending
        );
        assert_eq!(
            derive_state(
                Some(ManualState::Passed),
                CourseYear::LastSession,
                None,
                First
            ),
            CourseState::Passed
        );
    }

    #[test]
    fn current_year_depends_on_the_semester() {
        assert_eq!(
            derive_state(None, CourseYear::Current, Some(First), First),
            CourseState::Current
        );
        assert_eq!(
            derive_state(None, CourseYear::Current, Some(Second), First),
            CourseState::Upcoming
        );
        assert_eq!(
            derive_state(None, CourseYear::Current, Some(First), Second),
            CourseState::ExamPending
        );
        assert_eq!(
            derive_state(None, CourseYear::Current, Some(Second), Second),
            CourseState::Current
        );
        assert_eq!(
            derive_state(None, CourseYear::Current, None, Second),
            CourseState::Current
        );
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
