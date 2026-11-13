//! Course list: academic-year filter, manual selection and stable folder names.
//!
//! Active courses (§4.5-bis) are the timeline courses whose `coursecategory` is the current
//! academic year. Manual choices always win over that default.

pub mod archived;
pub mod career_page;
pub mod name;
pub mod plan_file;
pub mod semester;
pub mod state;
pub mod study_plan;

use std::collections::{HashMap, HashSet};

use chrono::{Datelike, NaiveDate};
use serde::Serialize;

use crate::moodle::types::TimelineCourse;
use crate::store::courses::TeachingPrefs;
use crate::sync::paths::sanitize_component;
use name::{CourseName, parse_fullname};
use semester::{Activity, Semester, SemesterGuess, current_period, guess_semester};
use state::{CourseState, CourseYear, derive_state};
use study_plan::PlanIndex;

/// The academic year starts in September.
const ACADEMIC_YEAR_START_MONTH: u32 = 9;
/// Last month of the September exam session, which still belongs to the previous year.
const LAST_SESSION_MONTH: u32 = 9;

/// A course as shown in the course list.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseInfo {
    pub id: u64,
    pub fullname: String,
    pub name: String,
    pub code: Option<String>,
    /// Grouping key for the same teaching (§6.3): the parent code when present.
    pub teaching_code: Option<String>,
    /// Key of the teaching choices: the teaching code, or `course:<id>` without one.
    pub teaching_key: String,
    pub state: CourseState,
    pub semester: Option<SemesterGuess>,
    /// Parent teaching of a module course, and section or cohort (`sezione A`).
    pub parent_name: Option<String>,
    pub section: Option<String>,
    pub teachers: Vec<String>,
    pub academic_year: String,
    /// Course of the current academic year (the default filter).
    pub current: bool,
    pub selected: bool,
    pub viewurl: String,
}

/// `2026-27` from September 2026 to August 2027.
pub fn current_academic_year(today: NaiveDate) -> String {
    let start = if today.month() >= ACADEMIC_YEAR_START_MONTH {
        today.year()
    } else {
        today.year() - 1
    };
    academic_year_from(start)
}

/// `2026` → `"2026-27"`, the format of `coursecategory`.
fn academic_year_from(start: i32) -> String {
    format!("{start}-{:02}", (start + 1).rem_euclid(100))
}

/// Whether a course of academic year `category` is current, in last year's September exam
/// session, or past.
pub fn course_year(category: &str, today: NaiveDate) -> CourseYear {
    if category == current_academic_year(today) {
        return CourseYear::Current;
    }
    let last_year = academic_year_from(today.year() - 1);
    if today.month() == LAST_SESSION_MONTH && category == last_year {
        return CourseYear::LastSession;
    }
    CourseYear::Past
}

/// What the user decided and what earlier syncs learned, used to build the list.
#[derive(Debug, Default)]
pub struct ListInputs {
    /// Explicit sync choices per course: they win over the state.
    pub selection: HashMap<u64, bool>,
    pub teachings: HashMap<String, TeachingPrefs>,
    pub activity: HashMap<u64, Activity>,
    /// Semesters from the imported study plan (empty if none).
    pub plan: PlanIndex,
}

/// Builds the course list with state, semester guess and selection of every course.
pub fn course_list(
    courses: &[TimelineCourse],
    inputs: &ListInputs,
    today: NaiveDate,
) -> Vec<CourseInfo> {
    let year = current_academic_year(today);
    let period = current_period(today);
    courses
        .iter()
        .map(|course| {
            let parsed = parse_fullname(&course.fullname);
            let current = course.coursecategory == year;
            let course_year = course_year(&course.coursecategory, today);
            let teaching_key = parsed
                .teaching_code()
                .map_or_else(|| format!("course:{}", course.id), str::to_owned);
            let prefs = inputs
                .teachings
                .get(&teaching_key)
                .copied()
                .unwrap_or_default();
            let activity = inputs.activity.get(&course.id).copied();
            let plan = plan_semester(&inputs.plan, &parsed);
            let semester = guess_semester(prefs.semester, plan, activity, course.startdate);
            let state = derive_state(
                prefs.state,
                course_year,
                semester.map(|g| g.semester),
                period,
            );
            CourseInfo {
                id: course.id,
                fullname: course.fullname.clone(),
                teaching_code: parsed.teaching_code().map(str::to_owned),
                teaching_key,
                state,
                semester,
                name: parsed.name,
                code: parsed.code,
                parent_name: parsed.parent_name,
                section: parsed.section,
                teachers: parsed.teachers,
                academic_year: course.coursecategory.clone(),
                current,
                selected: inputs
                    .selection
                    .get(&course.id)
                    .copied()
                    .unwrap_or_else(|| state.synced_by_default()),
                viewurl: course.viewurl.clone(),
            }
        })
        .collect()
}

/// Folder names for `courses`: names already assigned never change (renaming a course
/// folder would orphan its files); new courses get the short name, disambiguated by
/// teacher or academic year when another course already uses it (case-insensitively).
pub fn assign_folders(
    courses: &[CourseInfo],
    assigned: &HashMap<u64, String>,
) -> HashMap<u64, String> {
    let mut result = assigned.clone();
    let mut owners: HashMap<String, Option<&str>> = HashMap::new();
    for course in courses {
        if let Some(folder) = assigned.get(&course.id) {
            owners.insert(folder.to_lowercase(), Some(&course.academic_year));
        }
    }
    for folder in assigned.values() {
        owners.entry(folder.to_lowercase()).or_insert(None);
    }
    let mut new: Vec<&CourseInfo> = courses
        .iter()
        .filter(|c| !assigned.contains_key(&c.id))
        .collect();
    // Current courses first, so they keep the plain name.
    new.sort_by_key(|c| (!c.current, c.id));
    for course in new {
        let folder = pick_folder(course, &owners);
        owners.insert(folder.to_lowercase(), Some(&course.academic_year));
        result.insert(course.id, folder);
    }
    result
}

fn pick_folder(course: &CourseInfo, owners: &HashMap<String, Option<&str>>) -> String {
    let base = sanitize_component(&course.name);
    let holder_year = owners.get(&base.to_lowercase()).copied().flatten();
    let teacher = course.teachers.join(", ");
    let by_teacher = format!("{base} ({teacher})");
    let by_year = format!("{base} [{}]", course.academic_year);
    let both = format!("{base} ({teacher}) [{}]", course.academic_year);
    let same_year = holder_year == Some(course.academic_year.as_str());
    let ordered = if same_year {
        [base.clone(), by_teacher, by_year, both]
    } else {
        [base.clone(), by_year, by_teacher, both]
    };
    let taken: HashSet<&String> = owners.keys().collect();
    ordered
        .into_iter()
        .map(|candidate| sanitize_component(&candidate))
        .find(|candidate| !taken.contains(&candidate.to_lowercase()))
        .unwrap_or_else(|| sanitize_component(&format!("{base} ({})", course.id)))
}

/// The course in the study plan, by its own code, its parent code or its name.
fn plan_semester(plan: &PlanIndex, parsed: &CourseName) -> Option<Semester> {
    let codes: Vec<&str> = [parsed.code.as_deref(), parsed.parent_code.as_deref()]
        .into_iter()
        .flatten()
        .collect();
    plan.semester(&codes, &parsed.name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moodle::types::TimelineResponse;
    use crate::test_support::fixture;
    use semester::SemesterSource;
    use state::ManualState;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn timeline() -> Vec<TimelineCourse> {
        let response: TimelineResponse =
            serde_json::from_str(&fixture("courses/timeline_all.json")).unwrap();
        response.courses
    }

    fn info(id: u64, name: &str, teacher: &str, year: &str, current: bool) -> CourseInfo {
        CourseInfo {
            id,
            name: name.to_owned(),
            teachers: vec![teacher.to_owned()],
            academic_year: year.to_owned(),
            current,
            selected: true,
            ..CourseInfo::default()
        }
    }

    fn list(inputs: &ListInputs) -> Vec<CourseInfo> {
        course_list(&timeline(), inputs, date(2026, 11, 9))
    }

    fn find(list: &[CourseInfo], id: u64) -> &CourseInfo {
        list.iter().find(|c| c.id == id).unwrap()
    }

    #[test]
    fn the_study_plan_sets_the_semester_of_every_current_course() {
        let entries = study_plan::parse_plan(&fixture("study_plan/plan_text.txt"));
        let inputs = ListInputs {
            plan: PlanIndex::new(&entries),
            ..ListInputs::default()
        };
        let list = list(&inputs);
        let current: Vec<&CourseInfo> = list.iter().filter(|c| c.current).collect();

        // Every course of the year is in the plan (one by name: MIDA 2nd module).
        assert!(current.iter().all(|c| {
            c.semester
                .is_some_and(|g| g.source == SemesterSource::StudyPlan)
        }));
        let mida = current
            .iter()
            .find(|c| c.code.as_deref() == Some("051589"))
            .unwrap();
        assert_eq!(
            mida.state,
            CourseState::Upcoming,
            "2nd semester, seen in November"
        );
        let mining = current
            .iter()
            .find(|c| c.code.as_deref() == Some("056892"))
            .unwrap();
        assert_eq!(mining.state, CourseState::Current);
        let upcoming = current
            .iter()
            .filter(|c| c.state == CourseState::Upcoming)
            .count();
        assert_eq!(upcoming, 8);
    }

    #[test]
    fn september_is_still_last_years_exam_session() {
        let on = |m, d| date(2027, m, d);
        assert_eq!(course_year("2026-27", on(8, 31)), CourseYear::Current);
        assert_eq!(course_year("2026-27", on(9, 10)), CourseYear::LastSession);
        assert_eq!(course_year("2027-28", on(9, 10)), CourseYear::Current);
        assert_eq!(course_year("2025-26", on(9, 10)), CourseYear::Past);
        assert_eq!(course_year("2026-27", on(10, 1)), CourseYear::Past);
        assert_eq!(course_year("CCS", on(9, 10)), CourseYear::Past);
    }

    #[test]
    fn last_years_courses_stay_synced_until_the_end_of_september() {
        let in_september = course_list(&timeline(), &ListInputs::default(), date(2027, 9, 10));
        let in_october = course_list(&timeline(), &ListInputs::default(), date(2027, 10, 1));
        let last_year = |list: &[CourseInfo]| -> Vec<(CourseState, bool)> {
            list.iter()
                .filter(|c| c.academic_year == "2026-27")
                .map(|c| (c.state, c.selected))
                .collect()
        };

        let september = last_year(&in_september);
        assert_eq!(september.len(), 16);
        assert!(
            september
                .iter()
                .all(|&s| s == (CourseState::ExamPending, true))
        );
        assert!(
            last_year(&in_october)
                .iter()
                .all(|&s| s == (CourseState::Past, false))
        );
        let older = in_september.iter().filter(|c| c.academic_year == "2025-26");
        assert!(older.clone().count() > 0);
        assert!(
            older
                .into_iter()
                .all(|c| c.state == CourseState::Past && !c.selected)
        );
    }

    #[test]
    fn academic_year_changes_in_september() {
        assert_eq!(current_academic_year(date(2026, 11, 9)), "2026-27");
        assert_eq!(current_academic_year(date(2027, 8, 31)), "2026-27");
        assert_eq!(current_academic_year(date(2027, 9, 1)), "2027-28");
        assert_eq!(current_academic_year(date(2099, 10, 1)), "2099-00");
    }

    #[test]
    fn real_list_selects_only_the_current_year() {
        let list = list(&ListInputs::default());
        assert_eq!(list.len(), 33);
        let selected: Vec<&CourseInfo> = list.iter().filter(|c| c.selected).collect();
        assert_eq!(selected.len(), 16);
        assert!(selected.iter().all(|c| c.academic_year == "2026-27"));
        let community = list.iter().find(|c| c.academic_year == "CCS").unwrap();
        assert!(!community.current && !community.selected);
        let dm = list.iter().find(|c| c.id == 24741).unwrap();
        assert_eq!(dm.name, "DATA MINING");
        assert_eq!(dm.code.as_deref(), Some("056892"));
    }

    #[test]
    fn manual_choices_win() {
        let inputs = ListInputs {
            selection: HashMap::from([(24741, false), (20961, true)]),
            ..ListInputs::default()
        };
        let list = list(&inputs);
        assert!(!find(&list, 24741).selected);
        assert!(find(&list, 20961).selected);
    }

    #[test]
    fn passed_teaching_covers_every_year_and_stops_syncing() {
        // Machine Learning 097683: 2025-26 (20961) and 2026-27 (23056).
        let inputs = ListInputs {
            teachings: HashMap::from([(
                "097683".to_owned(),
                TeachingPrefs {
                    state: Some(ManualState::Passed),
                    semester: None,
                },
            )]),
            ..ListInputs::default()
        };
        let list = list(&inputs);
        for id in [20961, 23056] {
            assert_eq!(find(&list, id).state, CourseState::Passed);
            assert!(!find(&list, id).selected);
        }
    }

    #[test]
    fn module_courses_group_by_parent_code() {
        let list = list(&ListInputs::default());
        // 051589 is a module of 051587; 090037 has no parent.
        assert_eq!(find(&list, 25817).teaching_key, "051587");
        assert_eq!(find(&list, 25040).teaching_key, "090037");
        assert_eq!(find(&list, 2588).teaching_key, "course:2588");
    }

    #[test]
    fn semester_from_activity_and_states() {
        let inputs = ListInputs {
            activity: HashMap::from([
                (
                    24741,
                    Activity {
                        first: 30,
                        second: 0,
                    },
                ),
                (
                    24537,
                    Activity {
                        first: 0,
                        second: 2,
                    },
                ),
            ]),
            ..ListInputs::default()
        };
        let list = list(&inputs);
        let dm = find(&list, 24741);
        assert_eq!(dm.state, CourseState::Current);
        assert_eq!(
            dm.semester.map(|g| g.source),
            Some(SemesterSource::FileDates)
        );
        // A 2nd-semester course in November is upcoming, and still synced.
        let wds = find(&list, 24537);
        assert_eq!(wds.state, CourseState::Upcoming);
        assert!(wds.selected);
        assert_eq!(find(&list, 20961).state, CourseState::Past);
    }

    #[test]
    fn folders_use_the_short_name() {
        let courses = [info(1, "DATA MINING", "DOCENTE M", "2026-27", true)];
        let folders = assign_folders(&courses, &HashMap::new());
        assert_eq!(folders[&1], "DATA MINING");
    }

    #[test]
    fn same_year_duplicates_get_the_teacher() {
        let courses = [
            info(2, "DATA BASES 2", "DOCENTE R", "2025-26", false),
            info(1, "DATA BASES 2", "DOCENTE Q", "2025-26", false),
        ];
        let folders = assign_folders(&courses, &HashMap::new());
        assert_eq!(folders[&1], "DATA BASES 2");
        assert_eq!(folders[&2], "DATA BASES 2 (DOCENTE R)");
    }

    #[test]
    fn older_years_get_the_year_and_current_keeps_the_name() {
        let courses = [
            info(10, "MACHINE LEARNING", "DOCENTE X", "2025-26", false),
            info(20, "MACHINE LEARNING", "DOCENTE X", "2026-27", true),
        ];
        let folders = assign_folders(&courses, &HashMap::new());
        assert_eq!(folders[&20], "MACHINE LEARNING");
        assert_eq!(folders[&10], "MACHINE LEARNING [2025-26]");
    }

    #[test]
    fn assigned_folders_never_change() {
        let courses = [
            info(20, "MACHINE LEARNING", "DOCENTE X", "2026-27", true),
            info(10, "Machine Learning", "DOCENTE X", "2025-26", false),
        ];
        let assigned = HashMap::from([(10, "Machine Learning".to_owned())]);
        let folders = assign_folders(&courses, &assigned);
        assert_eq!(folders[&10], "Machine Learning");
        assert_eq!(folders[&20], "MACHINE LEARNING [2026-27]");
    }

    #[test]
    fn folder_names_are_sanitized() {
        let courses = [info(1, "A/B: C?", "DOCENTE", "2026-27", true)];
        let folders = assign_folders(&courses, &HashMap::new());
        assert_eq!(folders[&1], "A_B_ C_");
    }
}
