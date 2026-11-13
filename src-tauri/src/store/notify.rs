//! Announcements, forum counters, deadline reminders and muted courses.

use std::collections::{HashMap, HashSet};

use rusqlite::{Row, params};
use serde::{Deserialize, Serialize};

use super::{Store, StoreError};

/// A file attached to an announcement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub filename: String,
    pub filesize: u64,
    /// Web service URL, without token.
    pub fileurl: String,
}

/// An announcement as stored and shown in the "Bacheca".
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Announcement {
    /// Moodle discussion id.
    pub id: u64,
    pub course_id: u64,
    pub subject: String,
    /// Sanitized HTML.
    pub message: String,
    pub author: String,
    pub created: i64,
    pub modified: i64,
    pub pinned: bool,
    pub attachments: Vec<Attachment>,
    pub read: bool,
}

const SELECT_ANNOUNCEMENT: &str = "SELECT id, course_id, subject, message, author, created, \
    modified, pinned, attachments, read FROM announcement";

impl Store {
    pub fn forum_counts(&self) -> Result<HashMap<u64, u64>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT forum_id, numdiscussions FROM forum_state")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    pub fn set_forum_count(&self, forum_id: u64, count: u64) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO forum_state (forum_id, numdiscussions) VALUES (?1, ?2)",
            params![forum_id, count],
        )?;
        Ok(())
    }

    /// Inserts new announcements and updates known ones (keeping their read flag).
    /// Returns the ids that were not known before.
    pub fn save_announcements(&mut self, items: &[Announcement]) -> Result<Vec<u64>, StoreError> {
        let known = self.announcement_ids()?;
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO announcement (id, course_id, subject, message, author, created, \
                 modified, pinned, attachments, read) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10) ON CONFLICT(id) DO UPDATE SET \
                 subject = excluded.subject, message = excluded.message, \
                 author = excluded.author, modified = excluded.modified, \
                 pinned = excluded.pinned, attachments = excluded.attachments",
            )?;
            for a in items {
                let attachments = serde_json::to_string(&a.attachments).unwrap_or_default();
                stmt.execute(params![
                    a.id,
                    a.course_id,
                    a.subject,
                    a.message,
                    a.author,
                    a.created,
                    a.modified,
                    a.pinned,
                    attachments,
                    a.read
                ])?;
            }
        }
        tx.commit()?;
        Ok(items
            .iter()
            .map(|a| a.id)
            .filter(|id| !known.contains(id))
            .collect())
    }

    /// Newest first.
    pub fn announcements(&self) -> Result<Vec<Announcement>, StoreError> {
        let mut stmt = self.conn.prepare(&format!(
            "{SELECT_ANNOUNCEMENT} ORDER BY pinned DESC, created DESC"
        ))?;
        let rows = stmt.query_map([], announcement_from_row)?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Marks one announcement (or all, with `None`) as read.
    pub fn mark_read(&self, id: Option<u64>) -> Result<(), StoreError> {
        match id {
            Some(id) => self.conn.execute(
                "UPDATE announcement SET read = 1 WHERE id = ?1",
                params![id],
            )?,
            None => self.conn.execute("UPDATE announcement SET read = 1", [])?,
        };
        Ok(())
    }

    pub fn reminded_deadlines(&self) -> Result<HashSet<u64>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT event_id FROM deadline_reminder")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    pub fn set_reminded(&self, event_id: u64, when: i64) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO deadline_reminder (event_id, reminded) VALUES (?1, ?2)",
            params![event_id, when],
        )?;
        Ok(())
    }

    pub fn muted_courses(&self) -> Result<HashSet<u64>, StoreError> {
        let mut stmt = self.conn.prepare("SELECT course_id FROM course_mute")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    pub fn set_muted(&self, course_id: u64, muted: bool) -> Result<(), StoreError> {
        if muted {
            self.conn.execute(
                "INSERT OR IGNORE INTO course_mute (course_id) VALUES (?1)",
                params![course_id],
            )?;
        } else {
            self.conn.execute(
                "DELETE FROM course_mute WHERE course_id = ?1",
                params![course_id],
            )?;
        }
        Ok(())
    }

    fn announcement_ids(&self) -> Result<HashSet<u64>, StoreError> {
        let mut stmt = self.conn.prepare("SELECT id FROM announcement")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        Ok(rows.collect::<Result<_, _>>()?)
    }
}

fn announcement_from_row(row: &Row<'_>) -> rusqlite::Result<Announcement> {
    let attachments: String = row.get(8)?;
    Ok(Announcement {
        id: row.get(0)?,
        course_id: row.get(1)?,
        subject: row.get(2)?,
        message: row.get(3)?,
        author: row.get(4)?,
        created: row.get(5)?,
        modified: row.get(6)?,
        pinned: row.get(7)?,
        attachments: serde_json::from_str(&attachments).unwrap_or_default(),
        read: row.get(9)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: u64, created: i64, read: bool) -> Announcement {
        Announcement {
            id,
            course_id: 1,
            subject: format!("Avviso {id}"),
            message: "<p>x</p>".to_owned(),
            author: "DOCENTE A".to_owned(),
            created,
            modified: created,
            pinned: false,
            attachments: vec![Attachment {
                filename: "a.pdf".to_owned(),
                filesize: 3,
                fileurl: "https://webeep.polimi.it/webservice/pluginfile.php/1/a.pdf".to_owned(),
            }],
            read,
        }
    }

    #[test]
    fn saving_reports_only_new_ids_and_keeps_read_flags() {
        let mut store = Store::open_in_memory().unwrap();
        assert_eq!(
            store.save_announcements(&[item(1, 10, false)]).unwrap(),
            [1]
        );
        store.mark_read(Some(1)).unwrap();

        let edited = Announcement {
            subject: "Corretto".to_owned(),
            ..item(1, 10, false)
        };
        let new = store
            .save_announcements(&[edited, item(2, 20, false)])
            .unwrap();

        assert_eq!(new, [2]);
        let all = store.announcements().unwrap();
        assert_eq!(
            all.iter().map(|a| a.id).collect::<Vec<_>>(),
            [2, 1],
            "newest first"
        );
        assert!(all[1].read && all[1].subject == "Corretto");
        assert_eq!(all[0].attachments[0].filename, "a.pdf");
        store.mark_read(None).unwrap();
        assert!(store.announcements().unwrap().iter().all(|a| a.read));
    }

    #[test]
    fn counters_reminders_and_mutes_round_trip() {
        let store = Store::open_in_memory().unwrap();
        store.set_forum_count(5, 3).unwrap();
        assert_eq!(store.forum_counts().unwrap()[&5], 3);
        store.set_reminded(9, 100).unwrap();
        assert!(store.reminded_deadlines().unwrap().contains(&9));
        store.set_muted(7, true).unwrap();
        store.set_muted(7, true).unwrap();
        assert!(store.muted_courses().unwrap().contains(&7));
        store.set_muted(7, false).unwrap();
        assert!(store.muted_courses().unwrap().is_empty());
    }
}
