//! Pure helpers that turn remote names into safe local file and folder names (Windows rules
//! apply everywhere, so a folder synced on Windows can be copied to macOS/Linux and back).

/// Maximum length, in characters, of a single path component. Keeps full paths well under
/// Windows' 260-character limit for typical course structures.
pub const MAX_COMPONENT_CHARS: usize = 100;
/// Replacement for characters that are not allowed in file names.
const REPLACEMENT: char = '_';
/// Characters forbidden in Windows file names.
const FORBIDDEN: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
/// Device names reserved by Windows, with or without extension.
const RESERVED: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];
/// Longest extension recognised as such (e.g. `ipynb`).
const MAX_EXTENSION_CHARS: usize = 5;

/// Known MIME types and the extension to add when a file was uploaded without one.
const MIME_EXTENSIONS: &[(&str, &str)] = &[
    ("application/zip", "zip"),
    ("application/x-zip-compressed", "zip"),
    ("application/pdf", "pdf"),
    ("application/x-rar-compressed", "rar"),
    ("application/vnd.rar", "rar"),
    ("application/x-7z-compressed", "7z"),
    ("application/gzip", "gz"),
    ("application/x-tar", "tar"),
    ("application/x-ipynb+json", "ipynb"),
    ("application/msword", "doc"),
    (
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "docx",
    ),
    ("application/vnd.ms-powerpoint", "ppt"),
    (
        "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "pptx",
    ),
    ("application/vnd.ms-excel", "xls"),
    (
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "xlsx",
    ),
    ("text/plain", "txt"),
    ("text/csv", "csv"),
    ("text/x-python", "py"),
    ("image/png", "png"),
    ("image/jpeg", "jpg"),
    ("video/mp4", "mp4"),
];

/// Makes `name` a valid file or folder name on Windows, macOS and Linux.
pub fn sanitize_component(name: &str) -> String {
    let replaced: String = name
        .chars()
        .map(|c| {
            if c.is_control() || FORBIDDEN.contains(&c) {
                REPLACEMENT
            } else {
                c
            }
        })
        .collect();
    let trimmed = trim_name(&replaced);
    if trimmed.is_empty() {
        return REPLACEMENT.to_string();
    }
    truncate(&escape_reserved(trimmed))
}

/// Appends an extension derived from `mimetype` when `filename` has none.
pub fn ensure_extension(filename: &str, mimetype: Option<&str>) -> String {
    if extension(filename).is_some() {
        return filename.to_owned();
    }
    let Some(mime) = mimetype.map(normalize_mime) else {
        return filename.to_owned();
    };
    match MIME_EXTENSIONS.iter().find(|(known, _)| *known == mime) {
        Some((_, ext)) => format!("{}.{ext}", filename.trim_end_matches('.')),
        None => filename.to_owned(),
    }
}

/// Returns the extension of `filename` (without dot) if it looks like a real one.
pub fn extension(filename: &str) -> Option<&str> {
    let (stem, ext) = filename.rsplit_once('.')?;
    let is_extension = !stem.is_empty()
        && !ext.is_empty()
        && ext.len() <= MAX_EXTENSION_CHARS
        && ext.chars().all(|c| c.is_ascii_alphanumeric());
    is_extension.then_some(ext)
}

/// Leading whitespace is confusing, trailing dots and spaces are invalid on Windows.
fn trim_name(name: &str) -> &str {
    name.trim_start()
        .trim_end_matches(|c: char| c == '.' || c.is_whitespace())
}

fn normalize_mime(mime: &str) -> String {
    mime.split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
}

/// `CON` → `CON_`, `nul.txt` → `nul_.txt`: Windows reserves device names even with extension.
fn escape_reserved(name: &str) -> String {
    let (stem, rest) = match name.find('.') {
        Some(dot) => name.split_at(dot),
        None => (name, ""),
    };
    let is_reserved = RESERVED.iter().any(|r| r.eq_ignore_ascii_case(stem));
    if is_reserved {
        format!("{stem}{REPLACEMENT}{rest}")
    } else {
        name.to_owned()
    }
}

/// Shortens `name` to [`MAX_COMPONENT_CHARS`] characters, keeping a real extension.
fn truncate(name: &str) -> String {
    if name.chars().count() <= MAX_COMPONENT_CHARS {
        return name.to_owned();
    }
    let suffix = extension(name)
        .map(|ext| format!(".{ext}"))
        .unwrap_or_default();
    let stem_chars = MAX_COMPONENT_CHARS - suffix.chars().count();
    let stem: String = name.chars().take(stem_chars).collect();
    format!("{stem}{suffix}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_forbidden_characters() {
        assert_eq!(
            sanitize_component(r#"a<b>c:d"e/f\g|h?i*j"#),
            "a_b_c_d_e_f_g_h_i_j"
        );
    }

    #[test]
    fn replaces_control_characters() {
        assert_eq!(sanitize_component("a\tb\nc"), "a_b_c");
    }

    #[test]
    fn keeps_unicode_and_spaces() {
        assert_eq!(
            sanitize_component("Probabilità e Statistica"),
            "Probabilità e Statistica"
        );
    }

    #[test]
    fn trims_trailing_dots_and_spaces() {
        assert_eq!(sanitize_component("  Slides . . "), "Slides");
    }

    #[test]
    fn escapes_reserved_device_names() {
        assert_eq!(sanitize_component("CON"), "CON_");
        assert_eq!(sanitize_component("nul.txt"), "nul_.txt");
        assert_eq!(sanitize_component("Com1.pdf"), "Com1_.pdf");
        assert_eq!(sanitize_component("CONSOLE.pdf"), "CONSOLE.pdf");
    }

    #[test]
    fn empty_or_dot_only_names_become_placeholder() {
        assert_eq!(sanitize_component(""), "_");
        assert_eq!(sanitize_component(" .. "), "_");
    }

    #[test]
    fn truncates_long_names_keeping_extension() {
        let long = format!("{}.pdf", "x".repeat(300));
        let result = sanitize_component(&long);
        assert_eq!(result.chars().count(), MAX_COMPONENT_CHARS);
        assert!(result.ends_with("x.pdf"));
    }

    #[test]
    fn truncation_counts_characters_not_bytes() {
        let long = "è".repeat(300);
        assert_eq!(
            sanitize_component(&long).chars().count(),
            MAX_COMPONENT_CHARS
        );
    }

    #[test]
    fn detects_real_extensions_only() {
        assert_eq!(extension("slides.pdf"), Some("pdf"));
        assert_eq!(extension("notebook.ipynb"), Some("ipynb"));
        assert_eq!(extension("06 Classification"), None);
        assert_eq!(extension("Data Series v2.0 notes"), None);
        assert_eq!(extension(".hidden"), None);
        assert_eq!(extension("archive."), None);
    }

    #[test]
    fn adds_extension_to_zip_without_one() {
        assert_eq!(
            ensure_extension("06 Classification", Some("application/zip")),
            "06 Classification.zip"
        );
    }

    #[test]
    fn keeps_existing_extension() {
        assert_eq!(
            ensure_extension("09 Explainability.zip", Some("application/zip")),
            "09 Explainability.zip"
        );
        assert_eq!(
            ensure_extension("notes.PDF", Some("application/pdf")),
            "notes.PDF"
        );
    }

    #[test]
    fn leaves_name_alone_when_mimetype_is_unknown_or_missing() {
        assert_eq!(ensure_extension("README", None), "README");
        assert_eq!(
            ensure_extension("README", Some("application/x-unknown")),
            "README"
        );
    }

    #[test]
    fn mimetype_match_ignores_parameters_and_case() {
        assert_eq!(
            ensure_extension("data", Some("Text/CSV; charset=utf-8")),
            "data.csv"
        );
    }
}
