//! Pure merge of assignment attachments (`mod_assign_get_assignments`) into the course
//! structure, so `tree` handles them like the files of any other module.

use std::collections::HashMap;

use crate::moodle::types::{Assignment, CourseSection};

const ASSIGN_MODNAME: &str = "assign";

/// Returns a copy of `sections` where each `assign` module also lists its intro attachments.
pub fn with_assignment_files(
    sections: &[CourseSection],
    assignments: &[Assignment],
) -> Vec<CourseSection> {
    let by_module: HashMap<u64, &Assignment> = assignments.iter().map(|a| (a.cmid, a)).collect();
    sections
        .iter()
        .map(|section| {
            let mut section = section.clone();
            for module in section.modules.iter_mut() {
                if module.modname != ASSIGN_MODNAME {
                    continue;
                }
                if let Some(assignment) = by_module.get(&module.id) {
                    module
                        .contents
                        .extend(assignment.introattachments.iter().cloned());
                }
            }
            section
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moodle::types::AssignmentsResponse;
    use crate::sync::tree::{TreeOptions, build_tree};
    use crate::test_support::fixture;
    use serde_json::json;

    fn sections() -> Vec<CourseSection> {
        serde_json::from_value(json!([
            {"id": 1, "name": "Consegne", "modules": [
                {"id": 50, "name": "Homework 1", "modname": "assign"},
                {"id": 51, "name": "Homework 2", "modname": "assign"}
            ]}
        ]))
        .unwrap()
    }

    fn assignments() -> Vec<Assignment> {
        serde_json::from_value(json!([
            {"cmid": 50, "introattachments": [
                {"filename": "testo.pdf", "filepath": "/", "filesize": 12, "timemodified": 7,
                 "fileurl": "https://webeep.polimi.it/webservice/pluginfile.php/1/mod_assign/introattachment/0/testo.pdf",
                 "mimetype": "application/pdf"}
            ]},
            {"cmid": 99, "introattachments": []}
        ]))
        .unwrap()
    }

    #[test]
    fn attachments_become_files_of_the_assign_module() {
        let merged = with_assignment_files(&sections(), &assignments());
        let tree = build_tree(&merged, TreeOptions::default());
        let paths: Vec<String> = tree.files.iter().map(|f| f.local_path.join("/")).collect();
        assert_eq!(paths, vec!["Consegne/Homework 1/testo.pdf"]);
        assert_eq!(tree.files[0].filesize, 12);
    }

    #[test]
    fn input_sections_are_not_modified() {
        let original = sections();
        let _ = with_assignment_files(&original, &assignments());
        assert!(original[0].modules[0].contents.is_empty());
    }

    #[test]
    fn real_assignments_without_attachments_change_nothing() {
        let response: AssignmentsResponse =
            serde_json::from_str(&fixture("assignments/inprogress.json")).unwrap();
        // All assignments together, as the sync passes them: matching is by module id.
        let assignments: Vec<_> = response
            .courses
            .into_iter()
            .flat_map(|c| c.assignments)
            .collect();
        assert!(!assignments.is_empty());
        let sections: Vec<CourseSection> =
            serde_json::from_str(&fixture("contents/uncertainty_ai.json")).unwrap();
        let merged = with_assignment_files(&sections, &assignments);
        let before = build_tree(&sections, TreeOptions::default());
        assert_eq!(build_tree(&merged, TreeOptions::default()), before);
    }
}
