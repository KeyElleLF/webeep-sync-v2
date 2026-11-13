//! The diagnostic file attached to bug reports: app and system info, settings, a few counts and
//! the latest log lines, with anything personal or secret hidden. Pure: the command gathers the
//! input (`commands::support`), this builds the text the user sees before saving it.

use std::sync::LazyLock;

use regex::Regex;

/// Enough to cover a few days of normal use.
pub const MAX_LOG_LINES: usize = 1500;

pub struct ReportInput<'a> {
    pub version: &'a str,
    pub system: &'a str,
    pub language: &'a str,
    pub created: &'a str,
    pub settings: &'a str,
    pub schema_version: i64,
    pub tracked_files: u64,
    pub courses_with_files: u64,
    pub plan_teachings: Option<usize>,
    /// Log files, oldest first: (file name, content).
    pub logs: &'a [(String, String)],
    /// The user's home folder, replaced with `~` everywhere.
    pub home: Option<&'a str>,
}

/// `token=…` and `wstoken=…` in URLs.
static TOKEN_PARAM: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r#"(?i)\b((?:ws)?token)=[^&\s"'<>]+"#).ok());
/// Moodle tokens are 32 hexadecimal digits.
static HEX_TOKEN: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"\b[0-9a-fA-F]{32}\b").ok());
static EMAIL: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9-]+(?:\.[A-Za-z0-9-]+)+").ok());

pub fn build_report(input: &ReportInput<'_>) -> String {
    let plan = input
        .plan_teachings
        .map_or_else(|| "none".to_owned(), |n| format!("{n} teachings"));
    let mut lines = vec![
        "WeBeep Sync - diagnostics / diagnostica".to_owned(),
        format!("Version: {}", input.version),
        format!("System: {}", input.system),
        format!("Interface language: {}", input.language),
        format!("Created: {}", input.created),
        format!("Settings: {}", input.settings),
        format!("Database schema: {}", input.schema_version),
        format!(
            "Downloaded files tracked: {} (in {} courses)",
            input.tracked_files, input.courses_with_files
        ),
        format!("Study plan: {plan}"),
        "Hidden: home folder (shown as ~), tokens, email addresses.".to_owned(),
        String::new(),
        format!("--- Log (last {MAX_LOG_LINES} lines at most) ---"),
    ];
    let log = latest_log_lines(input.logs);
    if log.is_empty() {
        lines.push("(no log)".to_owned());
    }
    lines.extend(log);
    redact(&(lines.join("\n") + "\n"), input.home)
}

/// The last `MAX_LOG_LINES` lines over all files, each file introduced by its name.
fn latest_log_lines(logs: &[(String, String)]) -> Vec<String> {
    let total: usize = logs
        .iter()
        .map(|(_, content)| content.lines().count())
        .sum();
    let mut skip = total.saturating_sub(MAX_LOG_LINES);
    let mut out = Vec::new();
    for (name, content) in logs {
        let lines: Vec<&str> = content.lines().collect();
        let dropped = skip.min(lines.len());
        skip -= dropped;
        if dropped == lines.len() {
            continue;
        }
        out.push(format!("== {name} =="));
        out.extend(lines[dropped..].iter().map(|line| (*line).to_owned()));
    }
    out
}

/// Hides the home folder, tokens and email addresses.
pub fn redact(text: &str, home: Option<&str>) -> String {
    let mut text = text.to_owned();
    if let Some(re) = home.filter(|h| !h.is_empty()).and_then(home_pattern) {
        text = re.replace_all(&text, "~").into_owned();
    }
    if let Some(re) = TOKEN_PARAM.as_ref() {
        text = re.replace_all(&text, "$1=***").into_owned();
    }
    if let Some(re) = HEX_TOKEN.as_ref() {
        text = re.replace_all(&text, "***").into_owned();
    }
    if let Some(re) = EMAIL.as_ref() {
        text = re.replace_all(&text, "***@***").into_owned();
    }
    text
}

/// The home folder in any case, with `\` or `/` as separator.
fn home_pattern(home: &str) -> Option<Regex> {
    let trimmed = home.trim_end_matches(['\\', '/']);
    let parts: Vec<String> = trimmed.split(['\\', '/']).map(regex::escape).collect();
    Regex::new(&format!(r"(?i){}", parts.join(r"[\\/]"))).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOME: &str = r"C:\Users\mario";

    fn input<'a>(logs: &'a [(String, String)]) -> ReportInput<'a> {
        ReportInput {
            version: "0.1.3",
            system: "windows x86_64",
            language: "it",
            created: "2026-11-13 12:00",
            settings: "download 4, flatten true",
            schema_version: 5,
            tracked_files: 436,
            courses_with_files: 7,
            plan_teachings: Some(20),
            logs,
            home: Some(HOME),
        }
    }

    #[test]
    fn the_home_folder_is_hidden_in_any_spelling() {
        let text = r"C:\Users\mario\Desktop\polimi c:\users\MARIO\x C:/Users/mario/y";
        assert_eq!(redact(text, Some(HOME)), r"~\Desktop\polimi ~\x ~/y");
    }

    #[test]
    fn tokens_and_emails_are_hidden() {
        let text = "GET /webservice/pluginfile.php/1/a.pdf?token=abc123&forcedownload=1 \
                    wstoken=zzz 0123456789abcdef0123456789abcdef mario.rossi@mail.polimi.it";
        let hidden = redact(text, None);
        assert_eq!(
            hidden,
            "GET /webservice/pluginfile.php/1/a.pdf?token=***&forcedownload=1 \
             wstoken=*** *** ***@***"
        );
    }

    #[test]
    fn the_report_has_info_counts_and_only_the_latest_log_lines() {
        let old: String = (0..MAX_LOG_LINES).map(|i| format!("old {i}\n")).collect();
        let logs = vec![
            ("webeep-sync.2026-11-12.log".to_owned(), old),
            (
                "webeep-sync.2026-11-13.log".to_owned(),
                "INFO sync started in C:\\Users\\mario\\Desktop\nWARN course 1 failed\n".to_owned(),
            ),
        ];
        let report = build_report(&input(&logs));

        for expected in [
            "0.1.3",
            "windows x86_64",
            "436",
            "20",
            "download 4",
            "sync started in ~\\Desktop",
        ] {
            assert!(report.contains(expected), "missing {expected}");
        }
        assert!(!report.contains("mario"));
        assert!(!report.contains("old 0\n"), "oldest lines dropped");
        assert!(report.contains("old 1499") && report.contains("WARN course 1 failed"));
        let log_lines = report.lines().filter(|l| l.starts_with("old ")).count();
        assert_eq!(log_lines, MAX_LOG_LINES - 2);
    }

    #[test]
    fn a_missing_plan_and_empty_logs_are_said_plainly() {
        let report = build_report(&ReportInput {
            plan_teachings: None,
            ..input(&[])
        });
        assert!(report.contains("none"));
        assert!(report.contains("(no log)"));
    }
}
