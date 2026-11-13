//! Pure extraction of WeBeep file links from HTML (labels, module intros, section summaries)
//! and from `url` modules.
//!
//! Only `<a href>` links count: `<img src>` images in labels are decoration (banners, photos)
//! and are not downloaded. Links are converted to the web service form
//! (`/webservice/pluginfile.php/...`), which accepts `token=` at download time.

use std::collections::HashSet;
use std::sync::LazyLock;

use percent_encoding::percent_decode_str;
use regex::Regex;
use url::Url;

const SITE_HOST: &str = "webeep.polimi.it";
const PLUGINFILE_PATH: &str = "/pluginfile.php/";
const WEBSERVICE_PLUGINFILE_PATH: &str = "/webservice/pluginfile.php/";

/// `href` attribute of an anchor tag, single or double quoted.
static ANCHOR_HREF: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r#"(?is)<a\b[^>]*?\bhref\s*=\s*(?:"([^"]*)"|'([^']*)')"#).ok());

/// A WeBeep file linked from HTML or from a `url` module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkedFile {
    /// Web service URL, without token.
    pub url: String,
    /// Decoded last path segment.
    pub filename: String,
}

/// Files of WeBeep linked with `<a href>` in `html`, without duplicates, in document order.
pub fn linked_files(html: &str) -> Vec<LinkedFile> {
    let Some(anchor_href) = ANCHOR_HREF.as_ref() else {
        return Vec::new();
    };
    let mut seen = HashSet::new();
    anchor_href
        .captures_iter(html)
        .filter_map(|caps| caps.get(1).or_else(|| caps.get(2)))
        .filter_map(|href| webservice_file(&decode_entities(href.as_str())))
        .filter(|file| seen.insert(file.url.clone()))
        .collect()
}

/// Converts a WeBeep `pluginfile.php` URL to its web service form. Returns `None` for
/// anything else (other hosts, other WeBeep pages, malformed URLs).
pub fn webservice_file(raw: &str) -> Option<LinkedFile> {
    let mut url = Url::parse(raw.trim()).ok()?;
    let is_site = url.host_str() == Some(SITE_HOST) && matches!(url.scheme(), "http" | "https");
    if !is_site {
        return None;
    }
    let path = url.path().to_owned();
    let rest = path
        .strip_prefix(WEBSERVICE_PLUGINFILE_PATH)
        .or_else(|| path.strip_prefix(PLUGINFILE_PATH))?;
    let encoded_name = rest.rsplit('/').next().filter(|name| !name.is_empty())?;
    let filename = percent_decode_str(encoded_name)
        .decode_utf8()
        .ok()?
        .into_owned();
    url.set_scheme("https").ok()?;
    url.set_path(&format!("{WEBSERVICE_PLUGINFILE_PATH}{rest}"));
    Some(LinkedFile {
        url: url.to_string(),
        filename,
    })
}

/// HTML attributes escape `&` in query strings.
fn decode_entities(href: &str) -> String {
    href.replace("&amp;", "&").replace("&#38;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moodle::types::CourseSection;
    use crate::test_support::fixture;

    const WS: &str = "https://webeep.polimi.it/webservice/pluginfile.php";

    #[test]
    fn anchor_to_pluginfile_becomes_webservice_link() {
        let html = r#"<p>Vedi <a href="https://webeep.polimi.it/pluginfile.php/42/mod_label/intro/Esercizi%201.pdf">qui</a></p>"#;
        assert_eq!(
            linked_files(html),
            vec![LinkedFile {
                url: format!("{WS}/42/mod_label/intro/Esercizi%201.pdf"),
                filename: "Esercizi 1.pdf".to_owned(),
            }]
        );
    }

    #[test]
    fn images_are_decoration_and_ignored() {
        let html =
            r#"<img src="https://webeep.polimi.it/pluginfile.php/1/mod_label/intro/banner.jpg">"#;
        assert!(linked_files(html).is_empty());
    }

    #[test]
    fn real_labels_only_contain_decorative_images() {
        let sections: Vec<CourseSection> =
            serde_json::from_str(&fixture("contents/recommender_systems.json")).unwrap();
        let found: Vec<LinkedFile> = sections
            .iter()
            .flat_map(|s| &s.modules)
            .filter(|m| m.modname == "label")
            .flat_map(|m| linked_files(m.description.as_deref().unwrap_or_default()))
            .collect();
        assert!(found.is_empty(), "{found:?}");
    }

    #[test]
    fn external_and_non_file_links_are_ignored() {
        let html = r#"
            <a href="https://drive.google.com/file/d/abc/view">Drive</a>
            <a href="https://webeep.polimi.it/mod/resource/view.php?id=5">Risorsa</a>
            <a href="https://evil.example/pluginfile.php/1/x.pdf">Finto</a>
            <a href="mailto:redacted@example.com">Mail</a>"#;
        assert!(linked_files(html).is_empty());
    }

    #[test]
    fn handles_quotes_case_entities_and_duplicates() {
        let html = r#"
            <A class='btn' HREF='https://webeep.polimi.it/pluginfile.php/7/a/b.zip?forcedownload=1&amp;x=2'>uno</A>
            <a href="https://webeep.polimi.it/pluginfile.php/7/a/b.zip?forcedownload=1&amp;x=2">due</a>"#;
        let files = linked_files(html);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].url, format!("{WS}/7/a/b.zip?forcedownload=1&x=2"));
        assert_eq!(files[0].filename, "b.zip");
    }

    #[test]
    fn webservice_and_http_links_are_normalized() {
        let ws = webservice_file(&format!("{WS}/9/f/c.pdf")).unwrap();
        assert_eq!(ws.url, format!("{WS}/9/f/c.pdf"));
        let http = webservice_file("http://webeep.polimi.it/pluginfile.php/9/f/c.pdf").unwrap();
        assert_eq!(http.url, format!("{WS}/9/f/c.pdf"));
    }

    #[test]
    fn rejects_links_without_a_file_name() {
        assert_eq!(
            webservice_file("https://webeep.polimi.it/pluginfile.php/9/f/"),
            None
        );
        assert_eq!(webservice_file("not a url"), None);
    }

    #[test]
    fn decodes_multibyte_names() {
        let file =
            webservice_file("https://webeep.polimi.it/pluginfile.php/1/a/Probabilit%C3%A0.pdf")
                .unwrap();
        assert_eq!(file.filename, "Probabilità.pdf");
    }
}
