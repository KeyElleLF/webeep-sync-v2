//! Announcement HTML is written by teachers and comes from the network: it is untrusted.
//! It is sanitized with an allow-list before reaching the webview (which also has a strict
//! CSP), and reduced to plain text for system notifications.

use std::collections::HashSet;
use std::sync::LazyLock;

use ammonia::Builder;

/// Formatting seen in real announcements (`p`, `a`, `div`, `strong`, `span`, `br`, `pre`) plus
/// common harmless ones. No images: the CSP would block remote ones anyway.
const ALLOWED_TAGS: &[&str] = &[
    "a",
    "b",
    "blockquote",
    "br",
    "code",
    "div",
    "em",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "hr",
    "i",
    "li",
    "ol",
    "p",
    "pre",
    "s",
    "small",
    "span",
    "strong",
    "sub",
    "sup",
    "table",
    "tbody",
    "td",
    "th",
    "thead",
    "tr",
    "u",
    "ul",
];
const ALLOWED_SCHEMES: &[&str] = &["http", "https", "mailto"];
const PREVIEW_CHARS: usize = 160;

static SANITIZER: LazyLock<Builder<'static>> = LazyLock::new(|| {
    let mut builder = Builder::default();
    builder
        .tags(ALLOWED_TAGS.iter().copied().collect::<HashSet<_>>())
        .generic_attributes(HashSet::new())
        .tag_attributes(
            [("a", ["href"].into_iter().collect::<HashSet<_>>())]
                .into_iter()
                .collect(),
        )
        .url_schemes(ALLOWED_SCHEMES.iter().copied().collect())
        .link_rel(Some("noopener noreferrer"));
    builder
});

/// Safe HTML to show in the app.
pub fn sanitize(html: &str) -> String {
    SANITIZER.clean(html).to_string()
}

/// A short plain-text excerpt (for notifications).
pub fn preview(html: &str) -> String {
    let text = Builder::empty().clean(html).to_string();
    let text = decode_basic_entities(&text);
    let words: Vec<&str> = text.split_whitespace().collect();
    let joined = words.join(" ");
    if joined.chars().count() <= PREVIEW_CHARS {
        return joined;
    }
    let cut: String = joined.chars().take(PREVIEW_CHARS).collect();
    format!("{}…", cut.trim_end())
}

/// `ammonia` escapes text; notifications want it readable.
fn decode_basic_entities(text: &str) -> String {
    text.replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_formatting_and_safe_links() {
        let html = r#"<p>Ciao <strong>a tutti</strong>, vedi <a href="https://webeep.polimi.it/x" target="_blank" class="c">qui</a></p>"#;
        assert_eq!(
            sanitize(html),
            r#"<p>Ciao <strong>a tutti</strong>, vedi <a href="https://webeep.polimi.it/x" rel="noopener noreferrer">qui</a></p>"#
        );
    }

    #[test]
    fn removes_anything_executable() {
        let html = r#"<p onclick="alert(1)">x</p><script>alert(2)</script><img src=x onerror="alert(3)"><a href="javascript:alert(4)">y</a><iframe src="https://evil"></iframe><style>p{}</style>"#;
        let clean = sanitize(html);
        for bad in [
            "onclick",
            "script",
            "onerror",
            "img",
            "javascript",
            "iframe",
            "style",
            "alert",
        ] {
            assert!(!clean.contains(bad), "{bad} in {clean}");
        }
        assert!(clean.contains("<p>x</p>"));
    }

    #[test]
    fn preview_is_plain_short_text() {
        let html = "<p>Lezione&nbsp;di <b>domani</b> annullata &amp; spostata</p>\n<p>Saluti</p>";
        assert_eq!(
            preview(html),
            "Lezione di domani annullata & spostata Saluti"
        );
        let long = format!("<p>{}</p>", "parola ".repeat(100));
        let cut = preview(&long);
        assert!(cut.ends_with('…'));
        assert!(cut.chars().count() <= PREVIEW_CHARS + 1);
    }

    #[test]
    fn real_messages_survive_sanitizing() {
        use crate::moodle::notify_types::DiscussionsResponse;
        use crate::test_support::fixture;
        let response: DiscussionsResponse =
            serde_json::from_str(&fixture("notify/discussions.json")).unwrap();
        for d in response.discussions {
            let clean = sanitize(&d.message);
            assert!(clean.contains("Testo dell'avviso") || clean.contains("Testo dell&#39;avviso"));
        }
    }
}
