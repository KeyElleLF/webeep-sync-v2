//! Announcements ("Avvisi"): the `news` forum of each synced course.
//!
//! To stay light on WeBeep, a forum's discussions are fetched only when its discussion count
//! changed since the last check: a normal check is a single call. The first time a forum is
//! seen its history is imported without notifications (older posts already read).

use std::collections::HashMap;
use std::sync::Mutex;

use crate::moodle::client::{MoodleClient, array_param, param};
use crate::moodle::error::MoodleError;
use crate::moodle::notify_types::{Discussion, DiscussionsResponse, Forum};
use crate::notify::html::sanitize;
use crate::store::notify::{Announcement, Attachment};
use crate::store::{Store, StoreError};

const NEWS_FORUM: &str = "news";
const DISCUSSIONS_PER_CHECK: u32 = 20;
/// On first import, posts younger than this stay unread (but are not notified).
const RECENT_SECS: i64 = 7 * 86_400;

#[derive(Debug, thiserror::Error)]
pub enum NotifyError {
    #[error(transparent)]
    Moodle(#[from] MoodleError),
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error("database locale non disponibile")]
    Locked,
}

/// Checks the announcement forums of `course_ids`; returns the new announcements to notify.
pub async fn check(
    client: &MoodleClient,
    store: &Mutex<Store>,
    course_ids: &[u64],
    now: i64,
) -> Result<Vec<Announcement>, NotifyError> {
    if course_ids.is_empty() {
        return Ok(Vec::new());
    }
    let forums: Vec<Forum> = client
        .call(
            "mod_forum_get_forums_by_courses",
            &array_param("courseids", course_ids),
        )
        .await?;
    let counts = with(store, |s| s.forum_counts())?;
    let mut fresh = Vec::new();
    for forum in forums.iter().filter(|f| f.kind == NEWS_FORUM) {
        let previous = counts.get(&forum.id).copied();
        if previous == Some(forum.numdiscussions) {
            continue;
        }
        let discussions = fetch_discussions(client, forum.id).await?;
        let first_import = previous.is_none();
        let items: Vec<Announcement> = discussions
            .iter()
            .map(|d| to_announcement(forum.course, d, first_import, now))
            .collect();
        let new_ids = with(store, |s| s.save_announcements(&items))?;
        with(store, |s| s.set_forum_count(forum.id, forum.numdiscussions))?;
        if !first_import {
            fresh.extend(items.into_iter().filter(|a| new_ids.contains(&a.id)));
        }
    }
    Ok(fresh)
}

async fn fetch_discussions(
    client: &MoodleClient,
    forum_id: u64,
) -> Result<Vec<Discussion>, MoodleError> {
    let params = [
        param("forumid", forum_id),
        param("page", 0),
        param("perpage", DISCUSSIONS_PER_CHECK),
    ];
    let response: DiscussionsResponse = client
        .call("mod_forum_get_forum_discussions", &params)
        .await?;
    Ok(response.discussions)
}

/// Pure conversion: sanitized message, attachments, initial read flag.
pub fn to_announcement(
    course_id: u64,
    d: &Discussion,
    first_import: bool,
    now: i64,
) -> Announcement {
    let old = now - d.created > RECENT_SECS;
    Announcement {
        id: d.discussion,
        course_id,
        subject: d.subject.clone(),
        message: sanitize(&d.message),
        author: d.userfullname.clone(),
        created: d.created,
        modified: d.modified,
        pinned: d.pinned,
        attachments: d
            .attachments
            .iter()
            .map(|a| Attachment {
                filename: a.filename.clone(),
                filesize: a.filesize,
                fileurl: a.fileurl.clone(),
            })
            .collect(),
        read: first_import && old,
    }
}

/// Announcements grouped by course, for one notification per course.
pub fn by_course(items: &[Announcement]) -> HashMap<u64, Vec<&Announcement>> {
    let mut grouped: HashMap<u64, Vec<&Announcement>> = HashMap::new();
    for item in items {
        grouped.entry(item.course_id).or_default().push(item);
    }
    grouped
}

fn with<T>(
    store: &Mutex<Store>,
    f: impl FnOnce(&mut Store) -> Result<T, StoreError>,
) -> Result<T, NotifyError> {
    let mut guard = store.lock().map_err(|_| NotifyError::Locked)?;
    Ok(f(&mut guard)?)
}

#[cfg(test)]
#[path = "announcements_tests.rs"]
mod tests;
