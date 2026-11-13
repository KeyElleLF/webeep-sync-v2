//! Typed subsets of Moodle web service responses. Only the fields we use are declared.

use serde::Deserialize;

/// `core_webservice_get_site_info`
#[derive(Debug, Clone, Deserialize)]
pub struct SiteInfo {
    pub fullname: String,
    #[serde(default)]
    pub release: Option<String>,
}

/// A section of `core_course_get_contents`.
///
/// Moodle 4.5 subsections are delegated sections: `component == "mod_subsection"` and
/// `itemid` is the instance id of the `subsection` module that shows them.
#[derive(Debug, Clone, Deserialize)]
pub struct CourseSection {
    pub id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub section: u32,
    #[serde(default)]
    pub component: Option<String>,
    #[serde(default)]
    pub itemid: Option<u64>,
    #[serde(default)]
    pub uservisible: Option<bool>,
    /// Section summary (HTML): may link files.
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub modules: Vec<CourseModule>,
}

/// A course module (activity or resource) inside a section.
#[derive(Debug, Clone, Deserialize)]
pub struct CourseModule {
    pub id: u64,
    #[serde(default)]
    pub instance: Option<u64>,
    #[serde(default)]
    pub name: String,
    pub modname: String,
    #[serde(default)]
    pub uservisible: Option<bool>,
    /// Module intro (HTML); for labels it is the whole content. May link files.
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub contents: Vec<ModuleContent>,
}

/// A file or URL attached to a module.
#[derive(Debug, Clone, Deserialize)]
pub struct ModuleContent {
    /// `"file"`, `"url"` or `"content"`. Missing in assignment attachments: always files.
    #[serde(rename = "type", default = "file_kind")]
    pub kind: String,
    #[serde(default)]
    pub filename: String,
    /// `"/"` or `"/sub/dir/"`; `null` for URLs.
    #[serde(default)]
    pub filepath: Option<String>,
    #[serde(default)]
    pub filesize: u64,
    #[serde(default)]
    pub fileurl: String,
    #[serde(default)]
    pub timemodified: Option<i64>,
    #[serde(default)]
    pub mimetype: Option<String>,
}

fn file_kind() -> String {
    "file".to_owned()
}

/// `core_course_get_enrolled_courses_by_timeline_classification`
#[derive(Debug, Clone, Deserialize)]
pub struct TimelineResponse {
    #[serde(default)]
    pub courses: Vec<TimelineCourse>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TimelineCourse {
    pub id: u64,
    pub fullname: String,
    /// Academic year (`2026-27`), or `CCS` for degree-programme communities.
    #[serde(default)]
    pub coursecategory: String,
    /// Mostly the late-July creation default: a weak semester signal.
    #[serde(default)]
    pub startdate: i64,
    #[serde(default)]
    pub viewurl: String,
}

/// `mod_assign_get_assignments`
#[derive(Debug, Clone, Deserialize)]
pub struct AssignmentsResponse {
    #[serde(default)]
    pub courses: Vec<AssignmentCourse>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssignmentCourse {
    #[serde(default)]
    pub assignments: Vec<Assignment>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Assignment {
    /// Course module id: matches `CourseModule::id` of the `assign` module.
    pub cmid: u64,
    /// Files attached to the assignment description (not in `core_course_get_contents`).
    #[serde(default)]
    pub introattachments: Vec<ModuleContent>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::fixture;

    const CONTENT_FIXTURES: &[&str] = &[
        "contents/data_mining.json",
        "contents/data_information_quality.json",
        "contents/formal_languages_compilers.json",
        "contents/technologies_information_systems.json",
        "contents/recommender_systems.json",
        "contents/uncertainty_ai.json",
        "contents/machine_learning.json",
    ];

    #[test]
    fn every_contents_fixture_deserializes() {
        for name in CONTENT_FIXTURES {
            let sections: Vec<CourseSection> =
                serde_json::from_str(&fixture(name)).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert!(!sections.is_empty(), "{name}");
        }
    }

    #[test]
    fn folder_files_keep_their_subpath() {
        let sections: Vec<CourseSection> =
            serde_json::from_str(&fixture("contents/data_mining.json")).unwrap();
        let nested = sections
            .iter()
            .flat_map(|s| &s.modules)
            .flat_map(|m| &m.contents)
            .find(|c| c.filename == "06 Classification")
            .unwrap();
        assert_eq!(
            nested.filepath.as_deref(),
            Some("/Notebooks/Optional Notebooks/")
        );
        assert_eq!(nested.mimetype.as_deref(), Some("application/zip"));
    }
}
