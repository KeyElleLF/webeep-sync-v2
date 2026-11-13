//! Pure three-way sync planning: baseline (what we downloaded) × local disk × remote (WeBeep)
//! → actions. No I/O: the caller passes a snapshot of the local files it found.
//!
//! Rules (§7):
//! - only the remote changed → download and replace;
//! - only the local file changed (annotated slides) → never touch it, and keep the *original*
//!   baseline, so a later remote update is detected as a conflict instead of overwriting;
//! - both changed → save the remote copy next to it, `name (WeBeep aggiornato YYYY-MM-DD).ext`;
//! - removed by the teacher → report only, never delete;
//! - a file already on disk and never tracked (e.g. downloaded by the v1) is adopted when its
//!   size matches, so it is not downloaded again.

use std::collections::{HashMap, HashSet};

use chrono::NaiveDate;

use crate::sync::paths::extension;
use crate::sync::tree::RemoteFile;

/// What we recorded when we last wrote a file (one entry per remote key).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaselineEntry {
    /// Path relative to the course folder, components joined with `/`.
    pub local_path: String,
    pub remote_timemodified: i64,
    pub remote_filesize: u64,
    pub local_size: u64,
    pub local_mtime: i64,
}

/// Size and modification time of a file currently on disk.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LocalMeta {
    pub size: u64,
    pub mtime: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadReason {
    New,
    Missing,
    Updated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Download {
        key: String,
        path: String,
        url: String,
        reason: DownloadReason,
    },
    /// The file is already on disk: record it in the baseline without downloading.
    Adopt { key: String, path: String },
    /// Changed locally only: leave it alone.
    KeepLocal { key: String, path: String },
    /// Changed on both sides (or an untracked local file differs): save the remote copy aside.
    Conflict {
        key: String,
        path: String,
        conflict_path: String,
        url: String,
    },
    /// The local path changed (module renamed, …) and the old file is untouched: move it.
    Rename {
        key: String,
        from: String,
        to: String,
    },
    /// Tracked file no longer on WeBeep: reported, never deleted.
    RemovedRemotely { key: String, path: String },
}

/// Plans the actions for one course. `local` maps paths (as in [`BaselineEntry::local_path`])
/// to the files found on disk; `today` dates conflict copies.
pub fn plan(
    remote: &[RemoteFile],
    baseline: &HashMap<String, BaselineEntry>,
    local: &HashMap<String, LocalMeta>,
    today: NaiveDate,
) -> Vec<Action> {
    let mut actions: Vec<Action> = remote
        .iter()
        .filter_map(|file| plan_file(file, baseline.get(&file.remote_key), local, today))
        .collect();
    let present: HashSet<&str> = remote.iter().map(|f| f.remote_key.as_str()).collect();
    let mut removed: Vec<Action> = baseline
        .iter()
        .filter(|(key, _)| !present.contains(key.as_str()))
        .map(|(key, entry)| Action::RemovedRemotely {
            key: key.clone(),
            path: entry.local_path.clone(),
        })
        .collect();
    removed.sort_by(|a, b| action_path(a).cmp(action_path(b)));
    actions.extend(removed);
    actions
}

/// `slides.pdf` → `slides (WeBeep aggiornato 2026-11-09).pdf`
pub fn conflict_path(path: &str, today: NaiveDate) -> String {
    let (dir, name) = match path.rsplit_once('/') {
        Some((dir, name)) => (format!("{dir}/"), name),
        None => (String::new(), path),
    };
    let tag = format!("WeBeep aggiornato {}", today.format("%Y-%m-%d"));
    match extension(name) {
        Some(ext) => {
            let stem = &name[..name.len() - ext.len() - 1];
            format!("{dir}{stem} ({tag}).{ext}")
        }
        None => format!("{dir}{name} ({tag})"),
    }
}

fn plan_file(
    file: &RemoteFile,
    entry: Option<&BaselineEntry>,
    local: &HashMap<String, LocalMeta>,
    today: NaiveDate,
) -> Option<Action> {
    let path = file.local_path.join("/");
    match entry {
        None => Some(plan_untracked(file, path.clone(), local.get(&path), today)),
        Some(entry) if entry.local_path != path => {
            Some(plan_moved(file, entry, path, local, today))
        }
        Some(entry) => plan_tracked(file, entry, path.clone(), local.get(&path), today),
    }
}

/// Never tracked: download, adopt a matching file, or keep a different one untouched.
fn plan_untracked(
    file: &RemoteFile,
    path: String,
    on_disk: Option<&LocalMeta>,
    today: NaiveDate,
) -> Action {
    match on_disk {
        None => download(file, path, DownloadReason::New),
        // Linked files have an unknown size (0): whatever is there is taken as the file.
        Some(meta) if is_same_file(file, meta) => Action::Adopt {
            key: file.remote_key.clone(),
            path,
        },
        Some(_) => conflict(file, path, today),
    }
}

fn plan_tracked(
    file: &RemoteFile,
    entry: &BaselineEntry,
    path: String,
    on_disk: Option<&LocalMeta>,
    today: NaiveDate,
) -> Option<Action> {
    let Some(meta) = on_disk else {
        return Some(download(file, path, DownloadReason::Missing));
    };
    match (remote_changed(file, entry), local_changed(meta, entry)) {
        (false, false) => None,
        (true, false) => Some(download(file, path, DownloadReason::Updated)),
        // The baseline is deliberately kept: a later remote update becomes a conflict.
        (false, true) => Some(Action::KeepLocal {
            key: file.remote_key.clone(),
            path,
        }),
        (true, true) => Some(conflict(file, path, today)),
    }
}

/// The local path changed: move an untouched file, otherwise leave the old one alone.
fn plan_moved(
    file: &RemoteFile,
    entry: &BaselineEntry,
    path: String,
    local: &HashMap<String, LocalMeta>,
    today: NaiveDate,
) -> Action {
    if let Some(existing) = local.get(&path) {
        return plan_untracked(file, path, Some(existing), today);
    }
    let old_untouched = local
        .get(&entry.local_path)
        .is_some_and(|meta| !local_changed(meta, entry));
    if old_untouched && !remote_changed(file, entry) {
        return Action::Rename {
            key: file.remote_key.clone(),
            from: entry.local_path.clone(),
            to: path,
        };
    }
    download(file, path, DownloadReason::New)
}

/// An untracked file on disk is taken as the remote one only if size and modification time
/// both match (the v1 sets `mtime = timemodified`: verified on 76 of 76 real files). A file of
/// equal size but another date (e.g. a re-exported PDF) becomes a conflict instead, so a
/// later update can never overwrite it. Linked files have unknown size and date (0): whatever
/// is there is taken as the file.
fn is_same_file(file: &RemoteFile, meta: &LocalMeta) -> bool {
    let size_ok = file.filesize == 0 || meta.size == file.filesize;
    let date_ok = file.timemodified <= 0 || meta.mtime == file.timemodified;
    size_ok && date_ok
}

fn remote_changed(file: &RemoteFile, entry: &BaselineEntry) -> bool {
    file.timemodified != entry.remote_timemodified || file.filesize != entry.remote_filesize
}

fn local_changed(meta: &LocalMeta, entry: &BaselineEntry) -> bool {
    meta.size != entry.local_size || meta.mtime != entry.local_mtime
}

fn download(file: &RemoteFile, path: String, reason: DownloadReason) -> Action {
    Action::Download {
        key: file.remote_key.clone(),
        path,
        url: file.fileurl.clone(),
        reason,
    }
}

fn conflict(file: &RemoteFile, path: String, today: NaiveDate) -> Action {
    Action::Conflict {
        key: file.remote_key.clone(),
        conflict_path: conflict_path(&path, today),
        path,
        url: file.fileurl.clone(),
    }
}

fn action_path(action: &Action) -> &str {
    match action {
        Action::Download { path, .. }
        | Action::Adopt { path, .. }
        | Action::KeepLocal { path, .. }
        | Action::Conflict { path, .. }
        | Action::RemovedRemotely { path, .. } => path,
        Action::Rename { to, .. } => to,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &str = "10:/slides.pdf";
    const PATH: &str = "Lezioni/slides.pdf";
    const URL: &str = "https://webeep.polimi.it/webservice/pluginfile.php/1/slides.pdf";

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, 9).unwrap()
    }

    fn remote(timemodified: i64, filesize: u64) -> RemoteFile {
        RemoteFile {
            module_id: 10,
            remote_key: KEY.to_owned(),
            local_path: vec!["Lezioni".to_owned(), "slides.pdf".to_owned()],
            fileurl: URL.to_owned(),
            filesize,
            timemodified,
        }
    }

    fn base(path: &str) -> HashMap<String, BaselineEntry> {
        HashMap::from([(
            KEY.to_owned(),
            BaselineEntry {
                local_path: path.to_owned(),
                remote_timemodified: 100,
                remote_filesize: 500,
                local_size: 500,
                local_mtime: 100,
            },
        )])
    }

    fn disk(entries: &[(&str, u64, i64)]) -> HashMap<String, LocalMeta> {
        entries
            .iter()
            .map(|(p, size, mtime)| {
                (
                    (*p).to_owned(),
                    LocalMeta {
                        size: *size,
                        mtime: *mtime,
                    },
                )
            })
            .collect()
    }

    fn run(
        remote: &[RemoteFile],
        baseline: &HashMap<String, BaselineEntry>,
        local: &HashMap<String, LocalMeta>,
    ) -> Vec<Action> {
        plan(remote, baseline, local, today())
    }

    fn download(reason: DownloadReason) -> Action {
        Action::Download {
            key: KEY.to_owned(),
            path: PATH.to_owned(),
            url: URL.to_owned(),
            reason,
        }
    }

    fn conflict() -> Action {
        Action::Conflict {
            key: KEY.to_owned(),
            path: PATH.to_owned(),
            conflict_path: "Lezioni/slides (WeBeep aggiornato 2026-11-09).pdf".to_owned(),
            url: URL.to_owned(),
        }
    }

    // --- Untracked files -----------------------------------------------------------------

    #[test]
    fn new_remote_file_is_downloaded() {
        let actions = run(&[remote(100, 500)], &HashMap::new(), &HashMap::new());
        assert_eq!(actions, vec![download(DownloadReason::New)]);
    }

    #[test]
    fn untracked_local_file_with_same_size_and_date_is_adopted() {
        let actions = run(
            &[remote(100, 500)],
            &HashMap::new(),
            &disk(&[(PATH, 500, 100)]),
        );
        assert_eq!(
            actions,
            vec![Action::Adopt {
                key: KEY.to_owned(),
                path: PATH.to_owned()
            }]
        );
    }

    #[test]
    fn untracked_local_file_with_same_size_but_other_date_is_a_conflict() {
        let actions = run(
            &[remote(100, 500)],
            &HashMap::new(),
            &disk(&[(PATH, 500, 42)]),
        );
        assert_eq!(actions, vec![conflict()]);
    }

    #[test]
    fn untracked_local_file_with_other_size_is_never_overwritten() {
        let actions = run(
            &[remote(100, 500)],
            &HashMap::new(),
            &disk(&[(PATH, 777, 42)]),
        );
        assert_eq!(actions, vec![conflict()]);
    }

    #[test]
    fn linked_file_of_unknown_size_is_adopted_when_present() {
        let actions = run(&[remote(0, 0)], &HashMap::new(), &disk(&[(PATH, 777, 42)]));
        assert!(matches!(actions[0], Action::Adopt { .. }));
    }

    // --- Tracked files: the three-way cases ----------------------------------------------

    #[test]
    fn nothing_to_do_when_nothing_changed() {
        let actions = run(&[remote(100, 500)], &base(PATH), &disk(&[(PATH, 500, 100)]));
        assert!(actions.is_empty(), "{actions:?}");
    }

    #[test]
    fn remote_change_only_replaces_the_file() {
        let actions = run(&[remote(200, 600)], &base(PATH), &disk(&[(PATH, 500, 100)]));
        assert_eq!(actions, vec![download(DownloadReason::Updated)]);
    }

    #[test]
    fn remote_change_detected_by_size_alone() {
        let actions = run(&[remote(100, 600)], &base(PATH), &disk(&[(PATH, 500, 100)]));
        assert_eq!(actions, vec![download(DownloadReason::Updated)]);
    }

    #[test]
    fn local_change_only_is_left_alone() {
        let actions = run(&[remote(100, 500)], &base(PATH), &disk(&[(PATH, 900, 300)]));
        assert_eq!(
            actions,
            vec![Action::KeepLocal {
                key: KEY.to_owned(),
                path: PATH.to_owned()
            }]
        );
    }

    #[test]
    fn local_mtime_change_alone_counts_as_local_change() {
        let actions = run(&[remote(100, 500)], &base(PATH), &disk(&[(PATH, 500, 300)]));
        assert!(matches!(actions[0], Action::KeepLocal { .. }));
    }

    #[test]
    fn change_on_both_sides_saves_remote_copy_aside() {
        let actions = run(&[remote(200, 600)], &base(PATH), &disk(&[(PATH, 900, 300)]));
        assert_eq!(actions, vec![conflict()]);
    }

    #[test]
    fn deleted_local_file_is_downloaded_again() {
        let actions = run(&[remote(100, 500)], &base(PATH), &HashMap::new());
        assert_eq!(actions, vec![download(DownloadReason::Missing)]);
    }

    #[test]
    fn file_removed_by_teacher_is_reported_not_deleted() {
        let actions = run(&[], &base(PATH), &disk(&[(PATH, 500, 100)]));
        assert_eq!(
            actions,
            vec![Action::RemovedRemotely {
                key: KEY.to_owned(),
                path: PATH.to_owned()
            }]
        );
    }

    // --- Path changes --------------------------------------------------------------------

    #[test]
    fn untouched_file_follows_a_path_change() {
        let old = "Vecchia/slides.pdf";
        let actions = run(&[remote(100, 500)], &base(old), &disk(&[(old, 500, 100)]));
        assert_eq!(
            actions,
            vec![Action::Rename {
                key: KEY.to_owned(),
                from: old.to_owned(),
                to: PATH.to_owned()
            }]
        );
    }

    #[test]
    fn modified_file_stays_put_on_path_change_and_remote_goes_to_new_path() {
        let old = "Vecchia/slides.pdf";
        let actions = run(&[remote(100, 500)], &base(old), &disk(&[(old, 900, 300)]));
        assert_eq!(actions, vec![download(DownloadReason::New)]);
    }

    #[test]
    fn path_change_onto_an_existing_file_does_not_overwrite_it() {
        let old = "Vecchia/slides.pdf";
        let local = disk(&[(old, 500, 100), (PATH, 777, 1)]);
        let actions = run(&[remote(100, 500)], &base(old), &local);
        assert_eq!(actions, vec![conflict()]);
    }

    // --- Conflict names ------------------------------------------------------------------

    #[test]
    fn conflict_name_keeps_extension_and_folders() {
        assert_eq!(
            conflict_path("A/B/notes.final.pdf", today()),
            "A/B/notes.final (WeBeep aggiornato 2026-11-09).pdf"
        );
        assert_eq!(
            conflict_path("README", today()),
            "README (WeBeep aggiornato 2026-11-09)"
        );
    }

    #[test]
    fn actions_follow_remote_order_then_removals() {
        let mut second = remote(100, 500);
        second.remote_key = "11:/b.pdf".to_owned();
        second.local_path = vec!["b.pdf".to_owned()];
        let mut gone = base(PATH);
        let entry = gone.remove(KEY).unwrap();
        gone.insert(
            "99:/old.pdf".to_owned(),
            BaselineEntry {
                local_path: "old.pdf".to_owned(),
                ..entry
            },
        );
        let actions = run(&[remote(100, 500), second], &gone, &HashMap::new());
        let kinds: Vec<&str> = actions
            .iter()
            .map(|a| match a {
                Action::Download { path, .. } | Action::RemovedRemotely { path, .. } => {
                    path.as_str()
                }
                _ => "?",
            })
            .collect();
        assert_eq!(kinds, vec![PATH, "b.pdf", "old.pdf"]);
    }
}
