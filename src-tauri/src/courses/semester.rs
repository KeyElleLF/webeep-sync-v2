//! Semester of a course (§6.2), as a guess with its source so the UI can say how sure it is.
//!
//! Real data (2026-11-09): no custom fields; `startdate` is mostly the late-July default;
//! a course without material is not necessarily a 2nd-semester one (Machine Learning, 1st
//! semester, has no dated files). The best signal is when the course files were uploaded.
//! Order: manual choice > study plan (`study_plan`) > file dates > `startdate` (only if outside
//! July–August) > unknown.

use chrono::{DateTime, Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

/// Day from which the 2nd semester is the current one (lessons start late February).
const SECOND_PERIOD_START: (u32, u32) = (2, 15);
/// The 1st semester runs from here to the end of January (uploads start in August).
const FIRST_UPLOADS_FROM: (u32, u32) = (8, 15);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Semester {
    First,
    Second,
}

impl Semester {
    pub fn as_number(self) -> u8 {
        match self {
            Self::First => 1,
            Self::Second => 2,
        }
    }

    pub fn from_number(n: u8) -> Option<Self> {
        match n {
            1 => Some(Self::First),
            2 => Some(Self::Second),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SemesterSource {
    Manual,
    /// Imported from the study plan receipt (PDF).
    StudyPlan,
    FileDates,
    StartDate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SemesterGuess {
    pub semester: Semester,
    pub source: SemesterSource,
}

/// Files of a course uploaded during each semester of its academic year.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Activity {
    pub first: u32,
    pub second: u32,
}

/// The semester whose lessons or exams are going on: 1st from September to mid-February.
pub fn current_period(today: NaiveDate) -> Semester {
    let (month, day) = SECOND_PERIOD_START;
    let in_second = (today.month(), today.day()) >= (month, day) && today.month() < 9;
    if in_second {
        Semester::Second
    } else {
        Semester::First
    }
}

/// Counts the files modified in each semester of `academic_year` (`2026-27`). Files older
/// than the year (reused from past editions) say nothing and are ignored.
pub fn count_activity(
    timemodified: impl IntoIterator<Item = i64>,
    academic_year: &str,
) -> Activity {
    let Some(start) = academic_year.get(..4).and_then(|y| y.parse::<i32>().ok()) else {
        return Activity::default();
    };
    let first_from = date(start, FIRST_UPLOADS_FROM);
    let second_from = date(start + 1, SECOND_PERIOD_START);
    let year_end = date(start + 1, (8, 15));
    let mut activity = Activity::default();
    for day in timemodified.into_iter().filter_map(to_date) {
        match (first_from, second_from, year_end) {
            (Some(a), Some(b), _) if day >= a && day < b => activity.first += 1,
            (_, Some(b), Some(c)) if day >= b && day < c => activity.second += 1,
            _ => {}
        }
    }
    activity
}

/// Best guess, or `None` when no signal is reliable.
pub fn guess_semester(
    manual: Option<Semester>,
    plan: Option<Semester>,
    activity: Option<Activity>,
    startdate: i64,
) -> Option<SemesterGuess> {
    let guess = |semester, source| Some(SemesterGuess { semester, source });
    if let Some(semester) = manual {
        return guess(semester, SemesterSource::Manual);
    }
    if let Some(semester) = plan {
        return guess(semester, SemesterSource::StudyPlan);
    }
    match activity {
        Some(a) if a.first > a.second => return guess(Semester::First, SemesterSource::FileDates),
        Some(a) if a.second > a.first => {
            return guess(Semester::Second, SemesterSource::FileDates);
        }
        _ => {}
    }
    // Late July/August is the default creation date, not a real start.
    match to_date(startdate)?.month() {
        9..=12 => guess(Semester::First, SemesterSource::StartDate),
        2..=4 => guess(Semester::Second, SemesterSource::StartDate),
        _ => None,
    }
}

fn date(year: i32, (month, day): (u32, u32)) -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt(year, month, day)
}

fn to_date(unix: i64) -> Option<NaiveDate> {
    (unix > 0)
        .then(|| DateTime::from_timestamp(unix, 0))
        .flatten()
        .map(|t| t.date_naive())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ts(y: i32, m: u32, d: u32) -> i64 {
        NaiveDate::from_ymd_opt(y, m, d)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp()
    }

    fn day(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn periods_follow_the_polimi_calendar() {
        assert_eq!(current_period(day(2026, 11, 9)), Semester::First);
        assert_eq!(current_period(day(2027, 2, 14)), Semester::First);
        assert_eq!(current_period(day(2027, 2, 15)), Semester::Second);
        assert_eq!(current_period(day(2027, 7, 20)), Semester::Second);
        assert_eq!(current_period(day(2027, 9, 1)), Semester::First);
    }

    #[test]
    fn counts_uploads_per_semester_ignoring_old_files() {
        let files = [
            ts(2026, 7, 20), // reused from last year: ignored
            ts(2026, 9, 20), // 1st
            ts(2026, 10, 1), // 1st
            ts(2027, 3, 10), // 2nd
            ts(2027, 10, 1), // next year: ignored
            0,               // unknown date
        ];
        assert_eq!(
            count_activity(files, "2026-27"),
            Activity {
                first: 2,
                second: 1
            }
        );
        assert_eq!(count_activity(files, "CCS"), Activity::default());
    }

    #[test]
    fn manual_choice_wins() {
        let activity = Some(Activity {
            first: 10,
            second: 0,
        });
        let guess = guess_semester(Some(Semester::Second), None, activity, 0).unwrap();
        assert_eq!(guess.semester, Semester::Second);
        assert_eq!(guess.source, SemesterSource::Manual);
    }

    #[test]
    fn study_plan_beats_file_dates_but_not_a_manual_choice() {
        let activity = Some(Activity {
            first: 10,
            second: 0,
        });
        let plan = Some(Semester::Second);
        let guess = guess_semester(None, plan, activity, ts(2026, 9, 14)).unwrap();
        assert_eq!(
            (guess.semester, guess.source),
            (Semester::Second, SemesterSource::StudyPlan)
        );
        let manual = guess_semester(Some(Semester::First), plan, activity, 0).unwrap();
        assert_eq!(manual.source, SemesterSource::Manual);
    }

    #[test]
    fn file_dates_beat_the_start_date() {
        let activity = Some(Activity {
            first: 0,
            second: 3,
        });
        let guess = guess_semester(None, None, activity, ts(2026, 9, 14)).unwrap();
        assert_eq!(guess.semester, Semester::Second);
        assert_eq!(guess.source, SemesterSource::FileDates);
    }

    #[test]
    fn start_date_is_used_only_outside_the_default_summer_dates() {
        let none = Some(Activity::default());
        assert_eq!(
            guess_semester(None, None, none, ts(2026, 9, 14)).map(|g| g.semester),
            Some(Semester::First)
        );
        assert_eq!(
            guess_semester(None, None, none, ts(2026, 2, 20)).map(|g| g.semester),
            Some(Semester::Second)
        );
        assert_eq!(guess_semester(None, None, none, ts(2026, 7, 29)), None);
        assert_eq!(guess_semester(None, None, None, 0), None);
    }

    #[test]
    fn real_first_semester_courses_are_recognized_from_their_files() {
        use crate::moodle::types::CourseSection;
        use crate::sync::tree::{TreeOptions, build_tree};
        use crate::test_support::fixture;
        for name in [
            "data_mining",
            "data_information_quality",
            "formal_languages_compilers",
            "technologies_information_systems",
            "uncertainty_ai",
        ] {
            let sections: Vec<CourseSection> =
                serde_json::from_str(&fixture(&format!("contents/{name}.json"))).unwrap();
            let tree = build_tree(&sections, TreeOptions::default());
            let activity = count_activity(tree.files.iter().map(|f| f.timemodified), "2026-27");
            let guess = guess_semester(None, None, Some(activity), 0);
            assert_eq!(guess.map(|g| g.semester), Some(Semester::First), "{name}");
        }
    }
}
