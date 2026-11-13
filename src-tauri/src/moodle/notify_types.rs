//! Typed subsets of the forum and calendar responses used by announcements and deadlines.
//! Shapes verified on the real dump of 2026-11-09 (`fixtures/notify/`).

use serde::Deserialize;

use super::types::ModuleContent;

/// `mod_forum_get_forums_by_courses` (an array of these).
#[derive(Debug, Clone, Deserialize)]
pub struct Forum {
    pub id: u64,
    pub course: u64,
    /// `"news"` for the announcements forum ("Annunci" / "Avvisi").
    #[serde(rename = "type", default)]
    pub kind: String,
    #[serde(default)]
    pub numdiscussions: u64,
}

/// `mod_forum_get_forum_discussions`
#[derive(Debug, Clone, Deserialize)]
pub struct DiscussionsResponse {
    #[serde(default)]
    pub discussions: Vec<Discussion>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Discussion {
    /// Discussion id (stable across edits of the first post).
    pub discussion: u64,
    #[serde(default)]
    pub subject: String,
    /// HTML of the first post: untrusted, sanitized before display.
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub created: i64,
    #[serde(default)]
    pub modified: i64,
    #[serde(default)]
    pub userfullname: String,
    #[serde(default)]
    pub pinned: bool,
    /// Present only when the post has attachments.
    #[serde(default)]
    pub attachments: Vec<ModuleContent>,
}

/// `core_calendar_get_action_events_by_timesort`
#[derive(Debug, Clone, Deserialize)]
pub struct ActionEventsResponse {
    #[serde(default)]
    pub events: Vec<ActionEvent>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActionEvent {
    pub id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub activityname: String,
    #[serde(default)]
    pub modulename: String,
    /// When it is due (or closes).
    pub timesort: i64,
    #[serde(default)]
    pub overdue: bool,
    #[serde(default)]
    pub url: String,
    pub course: EventCourse,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventCourse {
    pub id: u64,
    #[serde(default)]
    pub fullname: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::fixture;

    #[test]
    fn real_discussions_deserialize() {
        let response: DiscussionsResponse =
            serde_json::from_str(&fixture("notify/discussions.json")).unwrap();
        assert_eq!(response.discussions.len(), 3);
        let first = &response.discussions[0];
        assert!(first.discussion > 0 && first.created > 0);
        assert!(first.message.contains("<p>"));
        assert!(first.attachments.is_empty());
    }

    #[test]
    fn real_action_events_deserialize() {
        let response: ActionEventsResponse =
            serde_json::from_str(&fixture("notify/action_events.json")).unwrap();
        assert_eq!(response.events.len(), 4);
        let kinds: Vec<&str> = response
            .events
            .iter()
            .map(|e| e.modulename.as_str())
            .collect();
        assert_eq!(kinds, ["quiz", "quiz", "quiz", "assign"]);
        assert!(
            response
                .events
                .iter()
                .all(|e| e.timesort > 0 && e.course.id > 0)
        );
    }

    #[test]
    fn attachments_deserialize_when_present() {
        let json = r#"{"discussions":[{"discussion":7,"subject":"s","message":"m",
            "attachments":[{"filename":"a.pdf","filepath":"/","filesize":3,
            "fileurl":"https://webeep.polimi.it/webservice/pluginfile.php/1/mod_forum/attachment/2/a.pdf",
            "timemodified":1,"mimetype":"application/pdf"}]}]}"#;
        let response: DiscussionsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.discussions[0].attachments[0].filename, "a.pdf");
    }
}
