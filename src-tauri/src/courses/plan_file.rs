//! Reading the study plan PDF that the user picked or dropped. The file is only read: its
//! text is turned into teachings (`study_plan::parse_plan`) and nothing else is kept.

use std::fs::File;
use std::io::Read;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Component, Path, Prefix};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use thiserror::Error;

use super::study_plan::{PlanEntry, parse_plan};

/// A plan receipt is a few KB; anything this big is something else.
const MAX_PDF_BYTES: u64 = 5 * 1024 * 1024;
/// The PDF library recurses: a big stack keeps a deeply nested (hostile) file from overflowing.
const EXTRACT_STACK_BYTES: usize = 64 * 1024 * 1024;
/// A real receipt takes milliseconds; a file that loops is given up (its thread is left behind).
const EXTRACT_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Debug, Error)]
pub enum PlanFileError {
    #[error("scegli un file PDF")]
    NotPdf,
    #[error("copia il PDF sul tuo PC: i file nelle cartelle di rete non vengono aperti")]
    NetworkPath,
    #[error("il file è troppo grande per essere la ricevuta del piano degli studi")]
    TooLarge,
    #[error("impossibile leggere il file: {0}")]
    Io(#[from] std::io::Error),
    #[error("il PDF non si riesce a leggere")]
    Unreadable,
    #[error(
        "nel PDF non ci sono insegnamenti: serve la ricevuta del piano degli studi o la pagina Piano degli studi salvata in PDF"
    )]
    NoTeachings,
}

pub fn read_plan_pdf(path: &Path) -> Result<Vec<PlanEntry>, PlanFileError> {
    // Even opening a file on a network share would contact the server.
    if is_network_path(path) {
        return Err(PlanFileError::NetworkPath);
    }
    let is_pdf = path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("pdf"));
    if !is_pdf {
        return Err(PlanFileError::NotPdf);
    }
    let file = File::open(path)?;
    let meta = file.metadata()?;
    if !meta.is_file() {
        return Err(PlanFileError::NotPdf);
    }
    if meta.len() > MAX_PDF_BYTES {
        return Err(PlanFileError::TooLarge);
    }
    // Size checked on the open file, and the read itself is capped too.
    let mut bytes = Vec::new();
    file.take(MAX_PDF_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_PDF_BYTES {
        return Err(PlanFileError::TooLarge);
    }
    plan_from_pdf(bytes)
}

pub fn plan_from_pdf(bytes: Vec<u8>) -> Result<Vec<PlanEntry>, PlanFileError> {
    let text = extract_text(bytes)?;
    let entries = parse_plan(&text);
    if entries.is_empty() {
        return Err(PlanFileError::NoTeachings);
    }
    Ok(entries)
}

/// Text of the PDF, read on its own thread: a panic of the library becomes `Unreadable`, a
/// deep file gets a large stack and a file that never finishes is given up after a timeout.
fn extract_text(bytes: Vec<u8>) -> Result<String, PlanFileError> {
    let (tx, rx) = mpsc::channel();
    thread::Builder::new()
        .name("study-plan-pdf".to_owned())
        .stack_size(EXTRACT_STACK_BYTES)
        .spawn(move || {
            let text = catch_unwind(AssertUnwindSafe(|| {
                pdf_extract::extract_text_from_mem(&bytes)
            }));
            let _ = tx.send(text.ok().and_then(Result::ok));
        })
        .map_err(|_| PlanFileError::Unreadable)?;
    match rx.recv_timeout(EXTRACT_TIMEOUT) {
        Ok(Some(text)) => Ok(text),
        _ => Err(PlanFileError::Unreadable),
    }
}

fn is_network_path(path: &Path) -> bool {
    match path.components().next() {
        Some(Component::Prefix(prefix)) => matches!(
            prefix.kind(),
            Prefix::UNC(..) | Prefix::VerbatimUNC(..) | Prefix::DeviceNS(..)
        ),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::courses::semester::Semester;
    use lopdf::content::{Content, Operation};
    use lopdf::{Document, Object, Stream, StringFormat, dictionary};

    /// A one-page PDF with these lines, in Helvetica with the Windows encoding (° is 0xB0).
    fn pdf_with_lines(lines: &[&str]) -> Vec<u8> {
        let mut doc = Document::with_version("1.5");
        let pages_id = doc.new_object_id();
        let font_id = doc.add_object(dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Helvetica",
            "Encoding" => "WinAnsiEncoding",
        });
        let resources_id = doc.add_object(dictionary! {
            "Font" => dictionary! { "F1" => font_id },
        });
        let mut operations = vec![Operation::new("BT", vec![])];
        for (i, line) in lines.iter().enumerate() {
            let y = 800 - 14 * i as i64;
            let bytes: Vec<u8> = line.chars().map(|c| c as u32 as u8).collect();
            operations.push(Operation::new("Tf", vec!["F1".into(), 10.into()]));
            operations.push(Operation::new(
                "Tm",
                vec![1.into(), 0.into(), 0.into(), 1.into(), 40.into(), y.into()],
            ));
            operations.push(Operation::new(
                "Tj",
                vec![Object::String(bytes, StringFormat::Literal)],
            ));
        }
        operations.push(Operation::new("ET", vec![]));
        let content = Content { operations };
        let content_id = doc.add_object(Stream::new(dictionary! {}, content.encode().unwrap()));
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "Contents" => content_id,
        });
        doc.objects.insert(
            pages_id,
            Object::Dictionary(dictionary! {
                "Type" => "Pages",
                "Kids" => vec![page_id.into()],
                "Count" => 1,
                "Resources" => resources_id,
                "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
            }),
        );
        let catalog_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        doc.trailer.set("Root", catalog_id);
        let mut bytes = Vec::new();
        doc.save_to(&mut bytes).unwrap();
        bytes
    }

    #[test]
    fn reads_the_teachings_of_a_plan_pdf() {
        let pdf = pdf_with_lines(&[
            "RICEVUTA DI PRESENTAZIONE PIANO DEGLI STUDI 2026",
            "Nome: STUDENTE",
            "Insegnamenti del piano",
            "2026 - 2° anno 1° semestre 056892 DATA MINING 5 CFU - Monodisc.",
            "2026 - 2° anno 2° semestre 090958 COMPUTER GRAPHICS 5 CFU -",
        ]);

        let entries = plan_from_pdf(pdf).unwrap();

        let read: Vec<(&str, &str, Semester)> = entries
            .iter()
            .map(|e| (e.code.as_str(), e.name.as_str(), e.semester))
            .collect();
        assert_eq!(
            read,
            [
                ("056892", "DATA MINING", Semester::First),
                ("090958", "COMPUTER GRAPHICS", Semester::Second)
            ]
        );
    }

    /// The real receipt has personal data, so it is never in the repo: run this by hand with
    /// `WEBEEP_PLAN_PDF=<path> cargo test real_plan_pdf -- --ignored --nocapture`.
    #[test]
    #[ignore = "needs a real plan PDF in WEBEEP_PLAN_PDF"]
    fn real_plan_pdf() {
        let path = std::env::var("WEBEEP_PLAN_PDF").unwrap();
        let entries = read_plan_pdf(Path::new(&path)).unwrap();
        for e in &entries {
            println!("{} {} {:?} {}", e.year, e.code, e.semester, e.name);
        }
        assert!(!entries.is_empty());
    }

    #[test]
    fn a_pdf_without_teachings_is_refused() {
        let pdf = pdf_with_lines(&["Slides of lesson 3", "Nothing to see here"]);
        assert!(matches!(
            plan_from_pdf(pdf),
            Err(PlanFileError::NoTeachings)
        ));
    }

    #[test]
    fn broken_or_random_bytes_are_unreadable_not_a_crash() {
        assert!(matches!(
            plan_from_pdf(Vec::new()),
            Err(PlanFileError::Unreadable)
        ));
        assert!(matches!(
            plan_from_pdf(b"%PDF-1.5\n garbage \x00\xff".to_vec()),
            Err(PlanFileError::Unreadable)
        ));
    }

    #[cfg(windows)]
    #[test]
    fn network_paths_are_refused_before_touching_them() {
        for unc in [
            r"\\server\share\piano.pdf",
            r"\\?\UNC\server\share\piano.pdf",
        ] {
            assert!(matches!(
                read_plan_pdf(Path::new(unc)),
                Err(PlanFileError::NetworkPath)
            ));
        }
        assert!(!is_network_path(Path::new(r"\\?\C:\Users\x\piano.pdf")));
        assert!(!is_network_path(Path::new(r"C:\Users\x\piano.pdf")));
    }

    #[test]
    fn only_pdf_files_of_a_sensible_size_are_read() {
        let dir = tempfile::tempdir().unwrap();
        let text = dir.path().join("piano.txt");
        std::fs::write(&text, "x").unwrap();
        assert!(matches!(read_plan_pdf(&text), Err(PlanFileError::NotPdf)));
        assert!(matches!(
            read_plan_pdf(dir.path()),
            Err(PlanFileError::NotPdf)
        ));
        let missing = dir.path().join("manca.pdf");
        assert!(matches!(read_plan_pdf(&missing), Err(PlanFileError::Io(_))));

        let big = dir.path().join("enorme.PDF");
        let file = std::fs::File::create(&big).unwrap();
        file.set_len(MAX_PDF_BYTES + 1).unwrap();
        assert!(matches!(read_plan_pdf(&big), Err(PlanFileError::TooLarge)));

        let good = dir.path().join("Piano.pdf");
        let lines = ["2026 - 2° anno 1° semestre 056892 DATA MINING 5 CFU"];
        std::fs::write(&good, pdf_with_lines(&lines)).unwrap();
        assert_eq!(read_plan_pdf(&good).unwrap().len(), 1);
    }
}
