//! Notification for links that teachers added since the previous sync.

use std::collections::HashSet;

use crate::i18n::{Language, pick};
use crate::sync::run::CourseSummary;

/// Lines listed in the notification; the others are only counted.
const MAX_LINES: usize = 5;

/// Title and body of the notification, or `None` if no unmuted course has new links.
pub fn new_links_notice(
    summaries: &[CourseSummary],
    muted: &HashSet<u64>,
    language: Language,
) -> Option<(String, String)> {
    let items: Vec<(&str, &str)> = summaries
        .iter()
        .filter(|s| !muted.contains(&s.id))
        .flat_map(|s| {
            s.new_links
                .iter()
                .map(|l| (s.name.as_str(), l.name.as_str()))
        })
        .collect();
    match items.as_slice() {
        [] => None,
        [(course, link)] => {
            let title = pick(language, "Nuovo link in", "New link in");
            Some((format!("{title} {course}"), (*link).to_owned()))
        }
        _ => {
            let mut lines: Vec<String> = items
                .iter()
                .take(MAX_LINES)
                .map(|(c, l)| format!("{c}: {l}"))
                .collect();
            if items.len() > MAX_LINES {
                let more = items.len() - MAX_LINES;
                lines.push(match language {
                    Language::Italian => format!("e altri {more}"),
                    Language::English => format!("and {more} more"),
                });
            }
            let new = pick(language, "nuovi link", "new links");
            Some((format!("{} {new}", items.len()), lines.join("\n")))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::execute::CourseReport;
    use crate::sync::tree::ExternalLink;

    fn summary(id: u64, name: &str, links: &[&str]) -> CourseSummary {
        CourseSummary {
            id,
            name: name.to_owned(),
            folder: name.to_owned(),
            report: CourseReport::default(),
            external_links: Vec::new(),
            new_links: links
                .iter()
                .map(|n| ExternalLink {
                    module_id: 1,
                    section: "Materiali".to_owned(),
                    name: (*n).to_owned(),
                    url: "https://example.org".to_owned(),
                })
                .collect(),
            error: None,
        }
    }

    #[test]
    fn nothing_new_means_no_notification() {
        let summaries = [summary(1, "Data Mining", &[])];
        assert_eq!(
            new_links_notice(&summaries, &HashSet::new(), Language::Italian),
            None
        );
    }

    #[test]
    fn one_new_link_names_course_and_link() {
        let summaries = [summary(1, "Recommender Systems", &["Slides"])];
        assert_eq!(
            new_links_notice(&summaries, &HashSet::new(), Language::Italian),
            Some((
                "Nuovo link in Recommender Systems".to_owned(),
                "Slides".to_owned()
            ))
        );
    }

    #[test]
    fn a_flood_of_links_lists_five_and_counts_the_rest() {
        let names: Vec<String> = (1..=8).map(|i| format!("Link {i}")).collect();
        let names: Vec<&str> = names.iter().map(String::as_str).collect();
        let summaries = [summary(1, "Corso", &names)];

        let (title, body) =
            new_links_notice(&summaries, &HashSet::new(), Language::Italian).unwrap();

        assert_eq!(title, "8 nuovi link");
        assert_eq!(body.lines().count(), 6);
        assert!(body.ends_with("e altri 3"));
    }

    #[test]
    fn notifications_follow_the_interface_language() {
        let one = [summary(1, "Data Mining", &["Slides"])];
        let (title, _) = new_links_notice(&one, &HashSet::new(), Language::English).unwrap();
        assert_eq!(title, "New link in Data Mining");

        let names: Vec<String> = (1..=7).map(|i| format!("Link {i}")).collect();
        let names: Vec<&str> = names.iter().map(String::as_str).collect();
        let many = [summary(1, "Data Mining", &names)];
        let (title, body) = new_links_notice(&many, &HashSet::new(), Language::English).unwrap();
        assert_eq!(title, "7 new links");
        assert!(body.ends_with("and 2 more"));
    }

    #[test]
    fn several_new_links_are_listed_and_muted_courses_skipped() {
        let summaries = [
            summary(1, "Recommender Systems", &["Slides", "Past exams"]),
            summary(2, "Data Mining", &["Notebook"]),
            summary(3, "Silenziato", &["Ignorato"]),
        ];
        let (title, body) =
            new_links_notice(&summaries, &HashSet::from([3]), Language::Italian).unwrap();
        assert_eq!(title, "3 nuovi link");
        assert_eq!(
            body,
            "Recommender Systems: Slides\nRecommender Systems: Past exams\nData Mining: Notebook"
        );
    }
}
