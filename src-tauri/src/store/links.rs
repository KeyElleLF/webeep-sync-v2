//! External links of the courses, kept between syncs to list them and to spot new ones.

use std::collections::HashSet;

use rusqlite::{Row, Transaction, params};
use serde::Serialize;

use super::{Store, StoreError};
use crate::sync::links::{LinkKind, classify, is_web_url};
use crate::sync::tree::ExternalLink;

/// A link as shown in the "Link" page. The name is the one the teacher gave on WeBeep.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseLink {
    pub id: i64,
    pub course_id: u64,
    pub module_id: u64,
    pub name: String,
    pub section: String,
    pub url: String,
    pub kind: LinkKind,
    pub first_seen: i64,
    /// False for a link that appeared after the first sync of its course and was not opened.
    pub seen: bool,
}

type LinkKey = (u64, String);

/// Limits against a broken or hostile course page: the data is shown and sent as notifications.
const MAX_URL_LEN: usize = 2048;
const MAX_TEXT_CHARS: usize = 300;
const MAX_LINKS_PER_COURSE: usize = 200;

impl Store {
    /// Replaces the links of a course with the ones just read from WeBeep (non-web or too long
    /// links are dropped, names cut, at most 200 per course). Returns the links that were not there before; the first time a course is
    /// saved nothing counts as new, otherwise every link would look new.
    pub fn save_links(
        &mut self,
        course_id: u64,
        links: &[ExternalLink],
        now: i64,
    ) -> Result<Vec<ExternalLink>, StoreError> {
        let bounded = bounded_links(links);
        let web: Vec<&ExternalLink> = bounded.iter().collect();
        let first_save = !self.links_saved_before(course_id)?;
        let known = self.link_keys(course_id)?;
        let current: HashSet<LinkKey> = web.iter().map(|l| key(l)).collect();
        let tx = self.conn.transaction()?;
        forget_links(&tx, course_id, known.difference(&current))?;
        upsert_links(&tx, course_id, &web, now, first_save)?;
        tx.execute(
            "INSERT OR IGNORE INTO course_link_import (course_id) VALUES (?1)",
            params![course_id],
        )?;
        tx.commit()?;
        if first_save {
            return Ok(Vec::new());
        }
        Ok(web
            .into_iter()
            .filter(|l| !known.contains(&key(l)))
            .cloned()
            .collect())
    }

    /// All links, by course and in the order of the course page.
    pub fn links(&self) -> Result<Vec<CourseLink>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, course_id, module_id, name, section, url, first_seen, seen \
             FROM course_link ORDER BY course_id, position",
        )?;
        let rows = stmt.query_map([], link_from_row)?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Marks one link (or all, with `None`) as seen.
    pub fn mark_links_seen(&self, id: Option<i64>) -> Result<(), StoreError> {
        match id {
            Some(id) => self
                .conn
                .execute("UPDATE course_link SET seen = 1 WHERE id = ?1", params![id])?,
            None => self.conn.execute("UPDATE course_link SET seen = 1", [])?,
        };
        Ok(())
    }

    fn links_saved_before(&self, course_id: u64) -> Result<bool, StoreError> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM course_link_import WHERE course_id = ?1",
            params![course_id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    fn link_keys(&self, course_id: u64) -> Result<HashSet<LinkKey>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT module_id, url FROM course_link WHERE course_id = ?1")?;
        let rows = stmt.query_map(params![course_id], |row| Ok((row.get(0)?, row.get(1)?)))?;
        Ok(rows.collect::<Result<_, _>>()?)
    }
}

fn bounded_links(links: &[ExternalLink]) -> Vec<ExternalLink> {
    links
        .iter()
        .filter(|l| l.url.len() <= MAX_URL_LEN && is_web_url(&l.url))
        .take(MAX_LINKS_PER_COURSE)
        .map(|l| ExternalLink {
            module_id: l.module_id,
            section: truncated(&l.section),
            name: truncated(&l.name),
            url: l.url.clone(),
        })
        .collect()
}

/// At most `MAX_TEXT_CHARS` characters (never cut inside a character).
fn truncated(text: &str) -> String {
    text.chars().take(MAX_TEXT_CHARS).collect()
}

fn key(link: &ExternalLink) -> LinkKey {
    (link.module_id, link.url.clone())
}

fn forget_links<'a>(
    tx: &Transaction<'_>,
    course_id: u64,
    gone: impl Iterator<Item = &'a LinkKey>,
) -> Result<(), StoreError> {
    let mut stmt =
        tx.prepare("DELETE FROM course_link WHERE course_id = ?1 AND module_id = ?2 AND url = ?3")?;
    for (module_id, url) in gone {
        stmt.execute(params![course_id, module_id, url])?;
    }
    Ok(())
}

/// Inserts new links (seen only on the first save of the course) and refreshes the name,
/// section and position of known ones, keeping their seen flag.
fn upsert_links(
    tx: &Transaction<'_>,
    course_id: u64,
    links: &[&ExternalLink],
    now: i64,
    seen: bool,
) -> Result<(), StoreError> {
    let mut stmt = tx.prepare(
        "INSERT INTO course_link (course_id, module_id, url, name, section, position, \
         first_seen, seen) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8) \
         ON CONFLICT (course_id, module_id, url) DO UPDATE SET name = excluded.name, \
         section = excluded.section, position = excluded.position",
    )?;
    for (position, link) in links.iter().enumerate() {
        stmt.execute(params![
            course_id,
            link.module_id,
            link.url,
            link.name,
            link.section,
            position as i64,
            now,
            seen
        ])?;
    }
    Ok(())
}

fn link_from_row(row: &Row<'_>) -> rusqlite::Result<CourseLink> {
    let name: String = row.get(3)?;
    let section: String = row.get(4)?;
    let url: String = row.get(5)?;
    Ok(CourseLink {
        id: row.get(0)?,
        course_id: row.get(1)?,
        module_id: row.get(2)?,
        kind: classify(&name, &section, &url),
        name,
        section,
        url,
        first_seen: row.get(6)?,
        seen: row.get(7)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn link(module_id: u64, name: &str, url: &str) -> ExternalLink {
        ExternalLink {
            module_id,
            section: "Teaching Materials".to_owned(),
            name: name.to_owned(),
            url: url.to_owned(),
        }
    }

    fn slides() -> ExternalLink {
        link(1, "Slides", "https://polimi365-my.sharepoint.com/:f:/a")
    }

    fn recordings() -> ExternalLink {
        link(
            2,
            "Archivio registrazioni",
            "https://aunicalogin.polimi.it/x",
        )
    }

    fn names(store: &Store) -> Vec<String> {
        store.links().unwrap().into_iter().map(|l| l.name).collect()
    }

    #[test]
    fn first_save_of_a_course_has_no_new_links_and_all_are_seen() {
        let mut store = Store::open_in_memory().unwrap();
        let new = store.save_links(7, &[slides(), recordings()], 100).unwrap();

        assert!(new.is_empty());
        let saved = store.links().unwrap();
        assert_eq!(saved.len(), 2);
        assert!(
            saved
                .iter()
                .all(|l| l.seen && l.course_id == 7 && l.first_seen == 100)
        );
        assert_eq!(saved[0].kind, LinkKind::Material);
        assert_eq!(saved[1].kind, LinkKind::Recording);
    }

    #[test]
    fn later_saves_report_added_links_and_forget_removed_ones() {
        let mut store = Store::open_in_memory().unwrap();
        store.save_links(7, &[slides(), recordings()], 100).unwrap();
        let exams = link(3, "Past exams", "https://drive.google.com/x");

        let new = store
            .save_links(7, &[exams.clone(), slides()], 200)
            .unwrap();

        assert_eq!(new, [exams]);
        let saved = store.links().unwrap();
        assert_eq!(
            names(&store),
            ["Past exams", "Slides"],
            "order of the course page"
        );
        assert!(!saved[0].seen && saved[0].first_seen == 200);
        assert!(saved[1].seen && saved[1].first_seen == 100);
    }

    #[test]
    fn a_renamed_link_keeps_its_id_and_seen_flag() {
        let mut store = Store::open_in_memory().unwrap();
        store.save_links(7, &[slides()], 100).unwrap();
        let id = store.links().unwrap()[0].id;
        let renamed = ExternalLink {
            name: "Slides 2026".to_owned(),
            ..slides()
        };

        assert!(store.save_links(7, &[renamed], 200).unwrap().is_empty());
        let saved = &store.links().unwrap()[0];
        assert_eq!(
            (saved.id, saved.name.as_str(), saved.seen),
            (id, "Slides 2026", true)
        );
    }

    #[test]
    fn a_new_url_for_the_same_module_is_a_new_link() {
        let mut store = Store::open_in_memory().unwrap();
        store.save_links(7, &[slides()], 100).unwrap();
        let moved = link(1, "Slides", "https://polimi365-my.sharepoint.com/:f:/b");

        assert_eq!(
            store
                .save_links(7, std::slice::from_ref(&moved), 200)
                .unwrap(),
            [moved]
        );
        assert_eq!(store.links().unwrap().len(), 1);
    }

    #[test]
    fn each_course_has_its_own_first_save() {
        let mut store = Store::open_in_memory().unwrap();
        store.save_links(7, &[slides()], 100).unwrap();
        assert!(store.save_links(8, &[slides()], 100).unwrap().is_empty());
        // A course saved once with no links still counts: the next link is new.
        store.save_links(9, &[], 100).unwrap();
        assert_eq!(store.save_links(9, &[slides()], 200).unwrap(), [slides()]);
    }

    #[test]
    fn links_that_are_not_web_links_are_never_stored() {
        let mut store = Store::open_in_memory().unwrap();
        let bad = link(4, "Strano", "javascript:alert(1)");
        store.save_links(7, &[slides()], 100).unwrap();

        assert!(
            store
                .save_links(7, &[slides(), bad], 200)
                .unwrap()
                .is_empty()
        );
        assert_eq!(names(&store), ["Slides"]);
    }

    #[test]
    fn oversized_links_are_bounded() {
        let mut store = Store::open_in_memory().unwrap();
        let long_name = ExternalLink {
            name: "è".repeat(MAX_TEXT_CHARS + 50),
            ..slides()
        };
        let long_url = link(
            2,
            "Lungo",
            &format!("https://example.org/{}", "a".repeat(MAX_URL_LEN)),
        );
        store.save_links(7, &[long_name, long_url], 100).unwrap();

        let saved = store.links().unwrap();
        assert_eq!(saved.len(), 1, "a URL longer than the limit is dropped");
        assert_eq!(saved[0].name.chars().count(), MAX_TEXT_CHARS);

        let many: Vec<ExternalLink> = (0..MAX_LINKS_PER_COURSE as u64 + 10)
            .map(|i| link(i, "x", &format!("https://example.org/{i}")))
            .collect();
        store.save_links(8, &many, 100).unwrap();
        let count = store
            .links()
            .unwrap()
            .iter()
            .filter(|l| l.course_id == 8)
            .count();
        assert_eq!(count, MAX_LINKS_PER_COURSE);
    }

    #[test]
    fn links_are_marked_seen_one_at_a_time_or_all_together() {
        let mut store = Store::open_in_memory().unwrap();
        store.save_links(7, &[], 100).unwrap();
        store.save_links(7, &[slides(), recordings()], 200).unwrap();
        let first = store.links().unwrap()[0].id;

        store.mark_links_seen(Some(first)).unwrap();
        let seen: Vec<bool> = store.links().unwrap().iter().map(|l| l.seen).collect();
        assert_eq!(seen, [true, false]);
        store.mark_links_seen(None).unwrap();
        assert!(store.links().unwrap().iter().all(|l| l.seen));
    }
}
