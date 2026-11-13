//! Parsing of Polimi course full names:
//! `<code> - <NAME> (<TEACHERS>) {<parent code> - <PARENT> [sezione X]} [<YYYY-YY>]`.
//!
//! Real-data quirks (§6.1-bis): the teachers are the *last* parenthesized group (names can
//! contain parentheses too), several teachers are comma-separated, the space before the year
//! may be missing, and community courses ("Ingegneria Informatica") follow no format at all.

/// The parts of a course full name. Missing parts are `None` / empty.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CourseName {
    pub code: Option<String>,
    pub name: String,
    pub teachers: Vec<String>,
    pub parent_code: Option<String>,
    pub parent_name: Option<String>,
    pub section: Option<String>,
    pub academic_year: Option<String>,
}

impl CourseName {
    /// Grouping key for the same teaching (§6.3): the parent code when present.
    pub fn teaching_code(&self) -> Option<&str> {
        self.parent_code.as_deref().or(self.code.as_deref())
    }
}

pub fn parse_fullname(fullname: &str) -> CourseName {
    let rest = fullname.trim();
    let (rest, academic_year) = strip_group(rest, '[', ']');
    let academic_year = academic_year.filter(|year| is_academic_year(year));
    let (rest, parent) = strip_group(rest, '{', '}');
    let (rest, teachers) = strip_group(rest, '(', ')');
    let (code, name) = split_code(rest);
    let (parent_code, parent_name, section) = parent.map(parse_parent).unwrap_or_default();
    // Without a code this is not a regular course name: keep it whole.
    let (name, teachers) = match (&code, teachers) {
        (None, Some(t)) => (format!("{name} ({t})"), Vec::new()),
        (_, teachers) => (name, teachers.map(split_teachers).unwrap_or_default()),
    };
    CourseName {
        code,
        name,
        teachers,
        parent_code,
        parent_name,
        section,
        academic_year,
    }
}

/// Removes a trailing `open … close` group, returning the rest (trimmed) and the content.
fn strip_group(text: &str, open: char, close: char) -> (&str, Option<String>) {
    let Some(without_close) = text.strip_suffix(close) else {
        return (text, None);
    };
    match without_close.rfind(open) {
        Some(start) => (
            text[..start].trim_end(),
            Some(without_close[start + open.len_utf8()..].trim().to_owned()),
        ),
        None => (text, None),
    }
}

/// `056892 - DATA MINING` → (`056892`, `DATA MINING`).
fn split_code(text: &str) -> (Option<String>, String) {
    match text.split_once(" - ") {
        Some((code, name)) if !code.is_empty() && code.chars().all(|c| c.is_ascii_digit()) => {
            (Some(code.to_owned()), name.trim().to_owned())
        }
        _ => (None, text.trim().to_owned()),
    }
}

/// `051587 - MODEL IDENTIFICATION [sezione A]` → code, name, section.
fn parse_parent(parent: String) -> (Option<String>, Option<String>, Option<String>) {
    let (rest, section) = strip_group(&parent, '[', ']');
    let (code, name) = split_code(rest);
    (code, Some(name).filter(|n| !n.is_empty()), section)
}

fn split_teachers(teachers: String) -> Vec<String> {
    teachers
        .split(',')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(str::to_owned)
        .collect()
}

/// `2026-27`
fn is_academic_year(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() == 7
        && bytes[4] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(i, b)| i == 4 || b.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::fixture;

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn parses_a_plain_course() {
        let parsed = parse_fullname("056892 - DATA MINING (DOCENTE M) [2026-27]");
        assert_eq!(
            parsed,
            CourseName {
                code: Some("056892".into()),
                name: "DATA MINING".into(),
                teachers: strings(&["DOCENTE M"]),
                academic_year: Some("2026-27".into()),
                ..CourseName::default()
            }
        );
        assert_eq!(parsed.teaching_code(), Some("056892"));
    }

    #[test]
    fn parses_module_with_parent_and_section() {
        let parsed = parse_fullname(
            "051589 - MODEL IDENTIFICATION AND DATA ANALYSIS - 2ND MODULE (DOCENTE A) \
             {051587 - MODEL IDENTIFICATION AND DATA ANALYSIS [sezione A]} [2026-27]",
        );
        assert_eq!(parsed.code.as_deref(), Some("051589"));
        assert_eq!(
            parsed.name,
            "MODEL IDENTIFICATION AND DATA ANALYSIS - 2ND MODULE"
        );
        assert_eq!(parsed.parent_code.as_deref(), Some("051587"));
        assert_eq!(
            parsed.parent_name.as_deref(),
            Some("MODEL IDENTIFICATION AND DATA ANALYSIS")
        );
        assert_eq!(parsed.section.as_deref(), Some("sezione A"));
        assert_eq!(parsed.teaching_code(), Some("051587"));
    }

    #[test]
    fn teacher_is_the_last_parenthesized_group() {
        let parsed = parse_fullname(
            "052511 - SISTEMI INFORMATIVI (PER IL SETTORE DELL'INFORMAZIONE) (DOCENTE B) [2023-24]",
        );
        assert_eq!(
            parsed.name,
            "SISTEMI INFORMATIVI (PER IL SETTORE DELL'INFORMAZIONE)"
        );
        assert_eq!(parsed.teachers, strings(&["DOCENTE B"]));
    }

    #[test]
    fn handles_missing_space_and_several_teachers() {
        let parsed =
            parse_fullname("054443 - SOFTWARE ENGINEERING 2 (DOCENTE G, DOCENTE Y)[2024-25]");
        assert_eq!(parsed.name, "SOFTWARE ENGINEERING 2");
        assert_eq!(parsed.teachers, strings(&["DOCENTE G", "DOCENTE Y"]));
        assert_eq!(parsed.academic_year.as_deref(), Some("2024-25"));
    }

    #[test]
    fn keeps_unformatted_names_whole() {
        assert_eq!(
            parse_fullname("Ingegneria Informatica"),
            CourseName {
                name: "Ingegneria Informatica".into(),
                ..CourseName::default()
            }
        );
        let odd = parse_fullname("Laboratorio (gruppo 2)");
        assert_eq!(odd.name, "Laboratorio (gruppo 2)");
        assert!(odd.teachers.is_empty());
    }

    #[test]
    fn every_real_course_name_parses() {
        let timeline: serde_json::Value =
            serde_json::from_str(&fixture("courses/timeline_all.json")).unwrap();
        for course in timeline["courses"].as_array().unwrap() {
            let fullname = course["fullname"].as_str().unwrap();
            let parsed = parse_fullname(fullname);
            assert!(!parsed.name.is_empty(), "{fullname}");
            if course["coursecategory"] == "CCS" {
                continue;
            }
            assert!(parsed.code.is_some(), "{fullname}");
            assert_eq!(parsed.teachers.len(), 1, "{fullname}");
            assert_eq!(
                parsed.academic_year.as_deref(),
                course["coursecategory"].as_str(),
                "{fullname}"
            );
        }
    }
}
