//! Cleanup of a passed course (§6.5). The decision of *what* to remove is pure and tested;
//! removal always goes through the Recycle Bin, and only empty folders are deleted.
//!
//! Safety rules:
//! - only files in the baseline (downloaded by the app) are candidates: files the user added
//!   by hand are never touched;
//! - files changed since we wrote them (annotated slides) are excluded unless the user
//!   explicitly includes them;
//! - a file that cannot be moved (open in another program) is skipped and reported.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::sync::execute::FileError;
use crate::sync::local::{local_meta, safe_join};
use crate::sync::plan::{BaselineEntry, LocalMeta};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupFile {
    #[serde(skip)]
    pub key: String,
    pub path: String,
    pub size: u64,
    /// Size and date seen when planning: the file is moved only if still exactly so.
    #[serde(skip)]
    pub seen: LocalMeta,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CleanupPlan {
    /// Untouched downloads: removed when the user confirms.
    pub files: Vec<CleanupFile>,
    /// Changed since download: removed only if the user asks for it too.
    pub modified: Vec<CleanupFile>,
    /// Not found on disk: forgotten, once confirmed absent.
    pub gone: Vec<CleanupFile>,
}

impl CleanupPlan {
    pub fn bytes(&self) -> u64 {
        self.files.iter().map(|f| f.size).sum()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupReport {
    pub trashed: usize,
    pub freed: u64,
    /// Modified files left in place.
    pub kept: usize,
    pub skipped: Vec<FileError>,
}

pub fn plan_cleanup(
    baseline: &HashMap<String, BaselineEntry>,
    local: &HashMap<String, LocalMeta>,
) -> CleanupPlan {
    let mut plan = CleanupPlan::default();
    for (key, entry) in baseline {
        let seen = local.get(&entry.local_path).copied();
        let file = CleanupFile {
            key: key.clone(),
            path: entry.local_path.clone(),
            size: seen.map_or(0, |m| m.size),
            seen: seen.unwrap_or_default(),
        };
        match seen {
            None => plan.gone.push(file),
            Some(m) if m.size == entry.local_size && m.mtime == entry.local_mtime => {
                plan.files.push(file);
            }
            Some(_) => plan.modified.push(file),
        }
    }
    for list in [&mut plan.files, &mut plan.modified, &mut plan.gone] {
        list.sort_by(|a, b| a.path.cmp(&b.path));
    }
    plan
}

/// Moves the planned files to the bin with `trash`, then removes the folders left empty.
/// Returns the report and the baseline keys to forget.
pub fn execute_cleanup(
    course_root: &Path,
    plan: &CleanupPlan,
    include_modified: bool,
    trash: impl Fn(&Path) -> Result<(), String>,
) -> (CleanupReport, Vec<String>) {
    let mut report = CleanupReport::default();
    // Unreadable is not absent: only a confirmed missing file is forgotten.
    let mut forget: Vec<String> = plan
        .gone
        .iter()
        .filter(|f| safe_join(course_root, &f.path).is_some_and(|p| is_absent(&p)))
        .map(|f| f.key.clone())
        .collect();
    let mut parents = HashSet::new();
    let extra: &[CleanupFile] = if include_modified {
        &plan.modified
    } else {
        &[]
    };
    report.kept = plan.modified.len() - extra.len();
    for file in plan.files.iter().chain(extra) {
        match trash_one(course_root, file, &trash) {
            Ok(path) => {
                report.trashed += 1;
                report.freed += file.size;
                forget.push(file.key.clone());
                parents.extend(path.parent().map(Path::to_path_buf));
            }
            Err(message) => report.skipped.push(FileError {
                path: file.path.clone(),
                message,
            }),
        }
    }
    remove_empty_dirs(course_root, parents);
    (report, forget)
}

fn trash_one(
    root: &Path,
    file: &CleanupFile,
    trash: &impl Fn(&Path) -> Result<(), String>,
) -> Result<PathBuf, String> {
    let path = safe_join(root, &file.path).ok_or("percorso non valido")?;
    // Re-check right before: size and date must still be the ones that were planned.
    let now = local_meta(&path).ok_or("il file non c'è più")?;
    if now != file.seen {
        return Err("il file è cambiato nel frattempo: non è stato toccato".to_owned());
    }
    trash(&path)?;
    Ok(path)
}

fn is_absent(path: &Path) -> bool {
    matches!(
        std::fs::symlink_metadata(path),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound
    )
}

/// Removes `dirs` and their ancestors up to and including `root`, only while empty.
fn remove_empty_dirs(root: &Path, dirs: HashSet<PathBuf>) {
    let mut ordered: Vec<PathBuf> = dirs
        .into_iter()
        .flat_map(|dir| {
            dir.ancestors()
                .take_while(|a| a.starts_with(root))
                .map(Path::to_path_buf)
                .collect::<Vec<_>>()
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    // Deepest first, so children go before their parents.
    ordered.sort_by_key(|d| std::cmp::Reverse(d.components().count()));
    for dir in ordered {
        // `remove_dir` fails on non-empty folders: user files keep their folder.
        let _ = std::fs::remove_dir(&dir);
    }
}

/// The real bin of the operating system.
pub fn system_trash(path: &Path) -> Result<(), String> {
    trash::delete(path).map_err(|e| e.to_string())
}

#[cfg(test)]
#[path = "cleanup_tests.rs"]
mod tests;
