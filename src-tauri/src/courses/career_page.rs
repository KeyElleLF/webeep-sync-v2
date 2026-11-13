//! The study plan as shown in "Servizi online > Carriera didattica > Piano degli studi", for
//! students without the plan receipt: the page saved as PDF (Ctrl+P) or its table copied and
//! pasted. Both give, for every teaching, the cells in this order (empty cells vanish, in the PDF
//! each value comes after its label and long names wrap):
//! `2026/27  055812 Programma dettagliato  DIGITAL FORENSICS AND CYBERCRIME  INF  T2A  E  2° Sem`
//! then exam state, credits, teachers, exam date and grade (only the first part is read).

use std::sync::LazyLock;

use regex::Regex;

use super::semester::Semester;
use super::study_plan::{MAX_ENTRIES, PlanEntry, teaching_name};

/// One teaching in the page text joined on one line: academic year (optional), code, optional
/// labels, name, then three one-word cells (degree course, PSPA, Posins) and the period.
static ROW: LazyLock<Option<Regex>> = LazyLock::new(|| {
    Regex::new(concat!(
        r"(?:\b(\d{4})/\d{2}\s+(?:Codice ins\.\s+)?)?",
        r"\b(\d{6})\b\s+",
        r"(?:Programma dettagliato\s+)?(?:Descrizione\s+)?",
        r"(.{1,150}?)\s+",
        r"(?:Corso laurea\s+)?\S+\s+(?:PSPA\s+)?\S+\s+(?:Posins\s+)?\S+\s+",
        r"(?:Periodo\s+)?([12])\s*°?\s*Sem\b",
    ))
    .ok()
});
/// A six-digit number inside a name means a row was swallowed: such a match is dropped.
static CODE: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(r"\b\d{6}\b").ok());
const TABLE_HEADER: &str = "Codice ins";

/// Teachings read from the page text. Only what follows the table header ("Codice ins.") is
/// read when the header is there: above it the PDF has the student's data (with a six-digit
/// student number that must never look like a teaching code).
pub fn parse_career_page(text: &str) -> Vec<PlanEntry> {
    let (Some(row), Some(code)) = (ROW.as_ref(), CODE.as_ref()) else {
        return Vec::new();
    };
    let joined = joined_lines(text);
    let table = joined
        .find(TABLE_HEADER)
        .map_or(joined.as_str(), |at| &joined[at + TABLE_HEADER.len()..]);
    row.captures_iter(table)
        .filter_map(|caps| {
            let name = teaching_name(&caps[3]);
            if name.is_empty() || code.is_match(&name) {
                return None;
            }
            Some(PlanEntry {
                code: caps[2].to_owned(),
                name,
                year: caps
                    .get(1)
                    .and_then(|y| y.as_str().parse().ok())
                    .unwrap_or(0),
                semester: Semester::from_number(caps[4].parse().ok()?)?,
            })
        })
        .take(MAX_ENTRIES)
        .collect()
}

/// All lines on one line with single spaces, without the browser's print header and footer
/// (they can fall in the middle of a row at a page break).
fn joined_lines(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .filter(|line| !is_print_margin(line))
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>()
        .join(" ")
}

/// `12/11/26, 20:32 Carriera studente`, the page address, `Carriera didattica v. 2.3.0 …`.
fn is_print_margin(line: &str) -> bool {
    let bytes = line.as_bytes();
    let printed_at = bytes.len() > 15
        && bytes[2] == b'/'
        && bytes[5] == b'/'
        && line.get(8..).is_some_and(|rest| rest.starts_with(", "));
    printed_at || line.starts_with("http") || line.contains("Carriera didattica v.")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::courses::study_plan::parse_plan;
    use crate::test_support::fixture;
    use std::collections::BTreeMap;

    fn semesters(entries: &[PlanEntry]) -> BTreeMap<String, Semester> {
        entries
            .iter()
            .map(|e| (e.code.clone(), e.semester))
            .collect()
    }

    fn receipt() -> Vec<PlanEntry> {
        parse_plan(&fixture("study_plan/plan_text.txt"))
    }

    fn entry<'a>(entries: &'a [PlanEntry], code: &str) -> &'a PlanEntry {
        entries.iter().find(|e| e.code == code).unwrap()
    }

    #[test]
    fn the_pasted_table_says_the_same_as_the_receipt() {
        let entries = parse_career_page(&fixture("study_plan/career_page_pasted.txt"));
        assert_eq!(entries.len(), 20);
        assert_eq!(semesters(&entries), semesters(&receipt()));
        assert_eq!(
            entry(&entries, "090038").name,
            "MODEL IDENTIFICATION AND DATA ANALYSIS - 2ND MODULE"
        );
        assert_eq!(entry(&entries, "056892").year, 2026);
        assert_eq!(
            entry(&entries, "089183").year,
            2025,
            "passed exam, read too"
        );
    }

    #[test]
    fn the_printed_page_says_the_same_as_the_receipt() {
        let entries = parse_career_page(&fixture("study_plan/career_page_pdf.txt"));
        assert_eq!(entries.len(), 20);
        assert_eq!(semesters(&entries), semesters(&receipt()));
        // Wrapped names are joined back.
        assert_eq!(
            entry(&entries, "052537").name,
            "TECHNOLOGIES FOR INFORMATION SYSTEMS"
        );
        assert_eq!(
            entry(&entries, "055633").name,
            "COMPUTER SECURITY - UIC 587"
        );
        assert_eq!(entry(&entries, "089183").name, "DATA BASES 2");
    }

    #[test]
    fn parse_plan_reads_every_format() {
        let pasted = parse_plan(&fixture("study_plan/career_page_pasted.txt"));
        let printed = parse_plan(&fixture("study_plan/career_page_pdf.txt"));
        assert_eq!(semesters(&pasted), semesters(&receipt()));
        assert_eq!(semesters(&printed), semesters(&receipt()));
    }

    #[test]
    fn numbers_above_the_table_are_never_teachings() {
        let text = "Matricola 123456 Nome STUDENTE PROVA INF T2A E 1° Sem\n\
                    Anno corso AA Codice ins. Descrizione Periodo\n\
                    2 2026/27 056892 Programma dettagliato DATA MINING INF T2A E 1° Sem N 5.0";
        let entries = parse_career_page(text);
        assert_eq!(entries.len(), 1);
        assert_eq!(
            (entries[0].code.as_str(), entries[0].name.as_str()),
            ("056892", "DATA MINING")
        );
    }

    #[test]
    fn a_selection_without_the_header_is_read_too() {
        let text =
            "2\n2026/27\n090958 Programma dettagliato\nCOMPUTER GRAPHICS\nINF\nT2A\nE\n2° Sem\nN";
        let entries = parse_career_page(text);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].semester, Semester::Second);
    }

    #[test]
    fn accented_text_never_breaks_the_reading() {
        for text in [
            "12/11/2è, x",
            "àààààààààààààààà",
            "ab/cd/èè, 20:32 Carriera studente",
        ] {
            assert!(parse_career_page(text).is_empty());
        }
    }

    #[test]
    fn unrelated_text_gives_nothing() {
        assert!(parse_career_page("").is_empty());
        assert!(parse_career_page("Lezione 3: 123456 studenti, 2° Sem").is_empty());
    }
}
