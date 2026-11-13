//! Deadlines (assignments due, quizzes closing) from the dashboard timeline, and which of
//! them deserve a reminder.

use std::collections::HashSet;

use serde::Serialize;

use crate::courses::name::parse_fullname;
use crate::moodle::client::{MoodleClient, param};
use crate::moodle::error::MoodleError;
use crate::moodle::notify_types::{ActionEvent, ActionEventsResponse};

/// §8: deadlines of the next 14 days.
pub const LOOKAHEAD_SECS: i64 = 14 * 86_400;
/// Reminder sent once, when a deadline is less than this far away.
pub const REMINDER_SECS: i64 = 24 * 3_600;
const MAX_EVENTS: u32 = 50;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Deadline {
    pub id: u64,
    /// "Problem statement generation è in scadenza".
    pub name: String,
    pub activity: String,
    /// `assign`, `quiz`, …
    pub kind: String,
    pub course_id: u64,
    pub course_name: String,
    pub due: i64,
    pub overdue: bool,
    /// Page on WeBeep.
    pub url: String,
}

pub async fn fetch(client: &MoodleClient, now: i64) -> Result<Vec<Deadline>, MoodleError> {
    let params = [
        param("timesortfrom", now),
        param("timesortto", now + LOOKAHEAD_SECS),
        param("limitnum", MAX_EVENTS),
    ];
    let response: ActionEventsResponse = client
        .call("core_calendar_get_action_events_by_timesort", &params)
        .await?;
    Ok(to_deadlines(&response.events))
}

/// Sorted by due date.
pub fn to_deadlines(events: &[ActionEvent]) -> Vec<Deadline> {
    let mut deadlines: Vec<Deadline> = events
        .iter()
        .map(|e| Deadline {
            id: e.id,
            name: e.name.clone(),
            activity: e.activityname.clone(),
            kind: e.modulename.clone(),
            course_id: e.course.id,
            course_name: parse_fullname(&e.course.fullname).name,
            due: e.timesort,
            overdue: e.overdue,
            url: e.url.clone(),
        })
        .collect();
    deadlines.sort_by_key(|d| (d.due, d.id));
    deadlines
}

/// Deadlines within the reminder window, not overdue, not reminded yet, not muted.
pub fn due_reminders<'a>(
    deadlines: &'a [Deadline],
    now: i64,
    reminded: &HashSet<u64>,
    muted: &HashSet<u64>,
) -> Vec<&'a Deadline> {
    deadlines
        .iter()
        .filter(|d| !d.overdue && d.due > now && d.due - now <= REMINDER_SECS)
        .filter(|d| !reminded.contains(&d.id) && !muted.contains(&d.course_id))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::fixture;

    fn real() -> Vec<Deadline> {
        let response: ActionEventsResponse =
            serde_json::from_str(&fixture("notify/action_events.json")).unwrap();
        to_deadlines(&response.events)
    }

    #[test]
    fn real_events_become_sorted_deadlines_with_short_course_names() {
        let deadlines = real();
        assert_eq!(deadlines.len(), 4);
        assert!(deadlines.windows(2).all(|w| w[0].due <= w[1].due));
        assert!(
            deadlines
                .iter()
                .all(|d| d.course_name == "UNCERTAINTY IN ARTIFICIAL INTELLIGENCE")
        );
        assert!(deadlines.iter().any(|d| d.kind == "assign"));
    }

    #[test]
    fn reminds_once_within_the_window() {
        let deadlines = real();
        let target = &deadlines[0];
        let now = target.due - 3_600;
        let none = HashSet::new();

        let due = due_reminders(&deadlines, now, &none, &none);
        assert!(due.iter().any(|d| d.id == target.id));

        let reminded = HashSet::from([target.id]);
        assert!(
            !due_reminders(&deadlines, now, &reminded, &none)
                .iter()
                .any(|d| d.id == target.id)
        );
        let muted = HashSet::from([target.course_id]);
        assert!(due_reminders(&deadlines, now, &none, &muted).is_empty());
        // Too early, or already past.
        assert!(
            !due_reminders(&deadlines, target.due - 2 * REMINDER_SECS, &none, &none)
                .iter()
                .any(|d| d.id == target.id)
        );
        assert!(
            !due_reminders(&deadlines, target.due + 1, &none, &none)
                .iter()
                .any(|d| d.id == target.id)
        );
    }
}
