//! The study plan receipt ("Ricevuta di presentazione piano degli studi", a PDF from the
//! Polimi online services) says the semester of every teaching; so does the "Piano degli
//! studi" page, for those who did not keep the receipt (`career_page`). Pure part: plan text →
//! entries, and the lookup used by the course list. Reading the PDF is in `plan_file`.
//!
//! One line per teaching, the same for every student:
//! `2026 - 2° anno 1° semestre 056892 DATA MINING 5 CFU - Monodisc.`
//! The name may continue on the next line, or be only on the next line.

use std::collections::HashMap;
use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

use super::career_page::parse_career_page;
use super::semester::Semester;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanEntry {
    /// Teaching code, six digits (`056892`).
    pub code: String,
    pub name: String,
    /// First solar year of the academic year (`2026` = 2026-27).
    pub year: i32,
    pub semester: Semester,
}

/// `2026 - 2° anno 1° semestre 056892 <rest>`: the degree sign may be missing or garbled.
static TEACHING_LINE: LazyLock<Option<Regex>> = LazyLock::new(|| {
    Regex::new(r"^\s*(\d{4})\s*-\s*\d\D{0,3}\s*anno\s+([12])\D{0,3}\s*semestre\s+(\d{6})\b(.*)$")
        .ok()
});
/// Credits and everything after them (`5 CFU - Monodisc.`).
static CREDITS: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"\s+\d+(?:[.,]\d+)?\s*CFU\b.*$").ok());

/// Limits for a hostile file: a real plan has a few dozen teachings with short names.
pub(super) const MAX_ENTRIES: usize = 200;
const MAX_NAME_CHARS: usize = 120;

/// Teachings read from a plan text, in order: the plan receipt, or else the "Piano degli
/// studi" page of the online services (printed or copied, see `career_page`).
pub fn parse_plan(text: &str) -> Vec<PlanEntry> {
    let receipt = parse_receipt(text);
    if receipt.is_empty() {
        return parse_career_page(text);
    }
    receipt
}

/// The receipt: one line per teaching. Lines that are not teachings are skipped.
fn parse_receipt(text: &str) -> Vec<PlanEntry> {
    let Some(line_re) = TEACHING_LINE.as_ref() else {
        return Vec::new();
    };
    let lines: Vec<&str> = text.lines().collect();
    lines
        .iter()
        .enumerate()
        .filter_map(|(i, line)| {
            let caps = line_re.captures(line)?;
            let rest = caps[4].trim();
            let name = if rest.is_empty() {
                next_text_line(&lines[i + 1..])?
            } else {
                rest
            };
            Some(PlanEntry {
                code: caps[3].to_owned(),
                name: teaching_name(name),
                year: caps[1].parse().ok()?,
                semester: Semester::from_number(caps[2].parse().ok()?)?,
            })
        })
        .take(MAX_ENTRIES)
        .collect()
}

fn next_text_line<'a>(lines: &[&'a str]) -> Option<&'a str> {
    lines.iter().map(|l| l.trim()).find(|l| !l.is_empty())
}

/// The name without credits and trailing dashes.
pub(super) fn teaching_name(text: &str) -> String {
    let without_credits = CREDITS
        .as_ref()
        .map_or(text.into(), |re| re.replace(text, ""));
    without_credits
        .trim()
        .trim_end_matches(['-', ' '])
        .chars()
        .take(MAX_NAME_CHARS)
        .collect()
}

/// Lookup of a course in the plan: by code first, then by name.
#[derive(Debug, Clone, Default)]
pub struct PlanIndex {
    by_code: HashMap<String, Semester>,
    by_name: HashMap<String, Semester>,
}

impl PlanIndex {
    pub fn new(entries: &[PlanEntry]) -> Self {
        let mut index = Self::default();
        // The first entry of a code or name wins, like the order of the plan.
        for e in entries {
            index.by_code.entry(e.code.clone()).or_insert(e.semester);
            index
                .by_name
                .entry(normalize_name(&e.name))
                .or_insert(e.semester);
        }
        index
    }

    /// Semester of a WeBeep course, from its codes (own and parent) or its name. The same
    /// teaching may have a different code on WeBeep (e.g. a module of an integrated course).
    pub fn semester(&self, codes: &[&str], name: &str) -> Option<Semester> {
        codes
            .iter()
            .find_map(|code| self.by_code.get(*code))
            .or_else(|| self.by_name.get(&normalize_name(name)))
            .copied()
    }
}

/// Uppercase letters and digits only, single spaces: `Model Identification - 2nd Module` and
/// `MODEL IDENTIFICATION 2ND MODULE` are the same name.
pub fn normalize_name(name: &str) -> String {
    let spaced: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect();
    spaced
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::fixture;
    use Semester::{First, Second};

    fn plan() -> Vec<PlanEntry> {
        parse_plan(&fixture("study_plan/plan_text.txt"))
    }

    fn entry<'a>(entries: &'a [PlanEntry], code: &str) -> &'a PlanEntry {
        entries.iter().find(|e| e.code == code).unwrap()
    }

    #[test]
    fn reads_every_teaching_of_the_real_plan() {
        let entries = plan();
        assert_eq!(entries.len(), 20);
        let first = entries.iter().filter(|e| e.semester == First).count();
        assert_eq!((first, entries.len() - first), (11, 9));
        assert_eq!(
            entry(&entries, "056892"),
            &PlanEntry {
                code: "056892".to_owned(),
                name: "DATA MINING".to_owned(),
                year: 2026,
                semester: First,
            }
        );
        assert_eq!(
            entry(&entries, "097683").semester,
            Second,
            "Machine Learning"
        );
    }

    #[test]
    fn names_are_cut_before_the_credits_or_taken_from_the_next_line() {
        let entries = plan();
        assert_eq!(
            entry(&entries, "052537").name,
            "TECHNOLOGIES FOR INFORMATION SYSTEMS"
        );
        assert_eq!(
            entry(&entries, "055633").name,
            "COMPUTER SECURITY - UIC 587"
        );
        assert_eq!(
            entry(&entries, "091023").name,
            "BUSINESS INFORMATION SYSTEMS 1"
        );
        assert_eq!(
            entry(&entries, "090038").name,
            "MODEL IDENTIFICATION AND DATA ANALYSIS - 2ND MODULE"
        );
        assert_eq!(entry(&entries, "090038").year, 2025);
    }

    #[test]
    fn text_without_teachings_gives_nothing() {
        assert!(parse_plan("").is_empty());
        assert!(parse_plan("Nome: X\nMatricola: 1\nTotale crediti richiesti: 40").is_empty());
        assert!(parse_plan("2026 - 2° anno 3° semestre 056892 DATA MINING 5 CFU").is_empty());
    }

    #[test]
    fn a_hostile_text_is_bounded() {
        let long = "X".repeat(500);
        let text: String = (0..300)
            .map(|i| format!("2026 - 2° anno 1° semestre {i:06} {long} 5 CFU\n"))
            .collect();
        let entries = parse_plan(&text);
        assert_eq!(entries.len(), MAX_ENTRIES);
        assert_eq!(entries[0].name.chars().count(), MAX_NAME_CHARS);
    }

    #[test]
    fn the_degree_symbol_may_be_missing_or_garbled() {
        let entries = parse_plan("2026 - 2 anno 2� semestre 090958 COMPUTER GRAPHICS 5 CFU -");
        assert_eq!(entries.len(), 1);
        assert_eq!(
            (entries[0].name.as_str(), entries[0].semester),
            ("COMPUTER GRAPHICS", Second)
        );
    }

    #[test]
    fn courses_are_found_by_code_then_by_name() {
        let index = PlanIndex::new(&plan());
        assert_eq!(index.semester(&["056892"], "Whatever"), Some(First));
        // Own code first, then the parent one.
        assert_eq!(index.semester(&["000000", "097683"], "x"), Some(Second));
        // Different code on WeBeep, same name (punctuation and case do not matter).
        assert_eq!(
            index.semester(
                &["051589", "051587"],
                "Model Identification and Data Analysis 2nd module"
            ),
            Some(Second)
        );
        assert_eq!(index.semester(&["123456"], "Not In The Plan"), None);
        assert_eq!(
            PlanIndex::default().semester(&["056892"], "DATA MINING"),
            None
        );
    }

    #[test]
    fn names_are_normalized() {
        assert_eq!(
            normalize_name("  Computer  Security - UIC 587 "),
            "COMPUTER SECURITY UIC 587"
        );
        assert_eq!(normalize_name("Data-Mining"), "DATA MINING");
    }
}
