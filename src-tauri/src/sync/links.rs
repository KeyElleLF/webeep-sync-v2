//! Pure classification of the external links of a course (`url` modules pointing outside
//! WeBeep): course material in cloud folders, lesson recordings, everything else.

use serde::Serialize;
use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LinkKind {
    Material,
    Recording,
    Other,
}

/// Words (lowercase, also as part of a word) that mark lesson recordings. Checked first, so
/// "Recordings Archive" in a "Teaching Materials" section is a recording.
const RECORDING_WORDS: &[&str] = &[
    "registrazion",
    "lezioni registrate",
    "videolezion",
    "video lezion",
    "recording",
    "recorded",
];
/// Live and recorded lessons.
const RECORDING_HOSTS: &[&str] = &["webex.com"];
const MATERIAL_WORDS: &[&str] = &[
    "slide", "material", "lucidi", "dispens", "notes", "appunti", "esercit", "exercise", "notebook",
];
/// Where teachers keep folders of slides and notes.
const STORAGE_HOSTS: &[&str] = &[
    "sharepoint.com",
    "onedrive.live.com",
    "1drv.ms",
    "drive.google.com",
    "docs.google.com",
    "dropbox.com",
    "github.com",
];

/// Decides the group of a link from its name, its section and where it points.
pub fn classify(name: &str, section: &str, url: &str) -> LinkKind {
    let text = format!("{name}\n{section}").to_lowercase();
    let host = Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(str::to_lowercase))
        .unwrap_or_default();
    if contains_any(&text, RECORDING_WORDS) || on_any_host(&host, RECORDING_HOSTS) {
        LinkKind::Recording
    } else if contains_any(&text, MATERIAL_WORDS) || on_any_host(&host, STORAGE_HOSTS) {
        LinkKind::Material
    } else {
        LinkKind::Other
    }
}

/// Only web links are kept: a teacher could paste anything in a `url` module.
pub fn is_web_url(url: &str) -> bool {
    Url::parse(url).is_ok_and(|u| matches!(u.scheme(), "http" | "https") && u.host().is_some())
}

fn contains_any(text: &str, words: &[&str]) -> bool {
    words.iter().any(|word| text.contains(word))
}

/// `host` is one of `hosts` or a subdomain of it (never just a name that starts the same).
fn on_any_host(host: &str, hosts: &[&str]) -> bool {
    hosts
        .iter()
        .any(|h| host == *h || host.strip_suffix(h).is_some_and(|rest| rest.ends_with('.')))
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONEDRIVE: &str = "https://polimi365-my.sharepoint.com/:f:/g/personal/x/abc";
    const RECORDINGS: &str =
        "https://aunicalogin.polimi.it/aunicalogin/aunicalogin.jsp?id_servizio=2263";
    const WEBEX: &str = "https://politecnicomilano.webex.com/meet/someone";

    #[test]
    fn slides_folder_on_onedrive_is_material() {
        let kind = classify(
            "Slides and Additional Materials",
            "Teaching Materials",
            ONEDRIVE,
        );
        assert_eq!(kind, LinkKind::Material);
    }

    #[test]
    fn cloud_storage_is_material_whatever_the_name() {
        for url in [
            ONEDRIVE,
            "https://onedrive.live.com/?id=1",
            "https://1drv.ms/f/s!abc",
            "https://drive.google.com/drive/folders/1",
            "https://docs.google.com/presentation/d/1",
            "https://www.dropbox.com/sh/abc",
            "https://github.com/someone/notebooks",
        ] {
            assert_eq!(
                classify("Link", "Generale", url),
                LinkKind::Material,
                "{url}"
            );
        }
    }

    #[test]
    fn material_words_in_name_or_section_mean_material() {
        let site = "https://example.org/course";
        for (name, section) in [
            ("Lucidi del corso", "Generale"),
            ("Dispense", "Generale"),
            ("Lecture notes", "Generale"),
            ("Appunti", "Generale"),
            ("Esercitazioni", "Generale"),
            ("Course page", "Materiale didattico"),
        ] {
            assert_eq!(classify(name, section, site), LinkKind::Material, "{name}");
        }
    }

    #[test]
    fn recordings_archive_and_webex_are_recordings() {
        assert_eq!(
            classify("Archivio registrazioni", "Registrazioni", RECORDINGS),
            LinkKind::Recording
        );
        assert_eq!(
            classify("Accesso all'aula virtuale", "Introduzione", WEBEX),
            LinkKind::Recording
        );
    }

    #[test]
    fn recording_words_win_over_material_section_and_cloud_host() {
        assert_eq!(
            classify("Recordings Archive", "Teaching Materials", RECORDINGS),
            LinkKind::Recording
        );
        assert_eq!(
            classify("Link to pre-recorded lessons", "Registrazioni", ONEDRIVE),
            LinkKind::Recording
        );
        assert_eq!(
            classify("Recording link for the lesson", "Materiali", WEBEX),
            LinkKind::Recording
        );
    }

    #[test]
    fn competitions_and_websites_are_other() {
        assert_eq!(
            classify(
                "Competition 2025",
                "Course Competition",
                "https://www.kaggle.com/c/x"
            ),
            LinkKind::Other
        );
        assert_eq!(
            classify(
                "Website of the Lab",
                "The Lab",
                "https://lab.deib.polimi.it/"
            ),
            LinkKind::Other
        );
    }

    #[test]
    fn host_must_really_be_the_storage_service() {
        let fake = "https://sharepoint.com.example.org/folder";
        assert_eq!(classify("Link", "Generale", fake), LinkKind::Other);
    }

    #[test]
    fn recommender_systems_links_are_grouped_as_expected() {
        use crate::moodle::types::CourseSection;
        use crate::sync::tree::{TreeOptions, build_tree};
        use crate::test_support::fixture;

        let raw = fixture("contents/recommender_systems.json");
        let sections: Vec<CourseSection> = serde_json::from_str(&raw).unwrap();
        let tree = build_tree(&sections, TreeOptions::default());
        let kind_of = |name: &str| {
            let link = tree.external_links.iter().find(|l| l.name == name).unwrap();
            classify(&link.name, &link.section, &link.url)
        };

        assert_eq!(tree.external_links.len(), 14);
        assert_eq!(
            kind_of("Slides and Additional Materials"),
            LinkKind::Material
        );
        assert_eq!(kind_of("Coding Notebooks"), LinkKind::Material);
        assert_eq!(kind_of("Recordings Archive"), LinkKind::Recording);
        assert_eq!(kind_of("Competition 2025"), LinkKind::Other);
        assert_eq!(
            kind_of("Website of the ReMAP Lab at Politecnico di Milano"),
            LinkKind::Other
        );
        assert!(tree.external_links.iter().all(|l| is_web_url(&l.url)));
    }

    #[test]
    fn only_http_and_https_links_are_web_urls() {
        assert!(is_web_url(ONEDRIVE));
        assert!(is_web_url("http://example.org"));
        for bad in [
            "javascript:alert(1)",
            "file:///C:/x",
            "ftp://x.org/a",
            "not a url",
            "",
        ] {
            assert!(!is_web_url(bad), "{bad}");
        }
    }
}
