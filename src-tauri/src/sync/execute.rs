//! Applies the planned actions of one course: downloads (bounded by a shared semaphore),
//! adoptions, renames and conflict copies. Failures are collected per file and never stop
//! the rest of the course. A cancelled sync starts no new download; the ones already running
//! finish (each file is written atomically, so nothing is left half done).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::sync::download::{Fetcher, Precondition, Target};
use crate::sync::local::{local_meta, safe_join};
use crate::sync::paths::extension;
use crate::sync::plan::{Action, BaselineEntry, DownloadReason, LocalMeta};
use crate::sync::tree::RemoteFile;

const UNSAFE_PATH: &str = "percorso non valido";
/// Conflict copies of the same file on the same day before giving up.
const MAX_COPIES: u32 = 99;
const VANISHED: &str = "il file non è più presente sul disco";

/// Called with (files done, files to transfer) of the course, for the progress shown in the UI.
pub type FileProgress<'a> = dyn Fn(usize, usize) + Send + Sync + 'a;

/// A file that could not be synced, for the summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FileError {
    pub path: String,
    pub message: String,
}

/// What happened to one course, shown in the sync summary.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseReport {
    /// New files, or tracked files deleted locally and downloaded again.
    pub downloaded: Vec<String>,
    pub updated: Vec<String>,
    pub adopted: usize,
    pub renamed: usize,
    pub kept_local: Vec<String>,
    /// Paths of the remote copies saved next to locally modified files.
    pub conflicts: Vec<String>,
    pub removed_remotely: Vec<String>,
    pub errors: Vec<FileError>,
}

#[derive(Debug, Default)]
pub struct CourseOutcome {
    /// Baseline entries to save (inserted or replaced).
    pub baseline: Vec<(String, BaselineEntry)>,
    /// Files the teacher removed from WeBeep: reported this once, then no longer tracked, so the
    /// copy on disk becomes the user's (never touched again, not even by the cleanup).
    pub forgotten: Vec<String>,
    pub report: CourseReport,
}

/// Result of one action, before it is folded into the outcome.
#[derive(Debug)]
enum Done {
    Downloaded {
        key: String,
        path: String,
        reason: DownloadReason,
        meta: LocalMeta,
    },
    Conflict {
        key: String,
        path: String,
        copy: String,
        meta: LocalMeta,
    },
    Adopted {
        key: String,
        path: String,
        meta: LocalMeta,
    },
    Renamed {
        key: String,
        to: String,
    },
    KeptLocal(String),
    RemovedRemotely {
        key: String,
        path: String,
    },
    Failed(FileError),
    /// Not started because the sync was cancelled: nothing changed, nothing to report.
    Cancelled,
}

/// Shared context of a course run.
pub struct CourseRun<'a> {
    pub fetcher: &'a Fetcher,
    pub limit: Arc<Semaphore>,
    /// Folder of the course; action paths are relative to it.
    pub root: &'a Path,
    pub remote: &'a [RemoteFile],
    pub baseline: &'a HashMap<String, BaselineEntry>,
    /// Set by "Annulla": downloads not yet started are skipped.
    pub cancel: Arc<AtomicBool>,
    pub on_files: Option<&'a FileProgress<'a>>,
}

impl CourseRun<'_> {
    pub async fn execute(&self, actions: Vec<Action>) -> CourseOutcome {
        let remote: HashMap<&str, &RemoteFile> = self
            .remote
            .iter()
            .map(|f| (f.remote_key.as_str(), f))
            .collect();
        let mut transfers = JoinSet::new();
        let mut done = Vec::new();
        for action in actions {
            match action {
                Action::Download { .. } | Action::Conflict { .. } => {
                    match self.prepare(action, &remote) {
                        Ok((action, target)) => {
                            transfers.spawn(self.transfer(action, target));
                        }
                        Err(failure) => done.push(failure),
                    }
                }
                other => done.push(self.apply_local(other)),
            }
        }
        let total = transfers.len();
        let report = |finished: usize| {
            if let Some(on_files) = self.on_files {
                on_files(finished, total);
            }
        };
        if total > 0 {
            report(0);
        }
        let mut finished = 0;
        while let Some(joined) = transfers.join_next().await {
            done.push(joined.unwrap_or_else(|err| failed("?", &err.to_string())));
            finished += 1;
            report(finished);
        }
        self.fold(done)
    }

    /// Resolves the destination of a download and what it must look like when replaced.
    /// Conflict copies get a free name, so an existing file is never overwritten.
    fn prepare(
        &self,
        action: Action,
        remote: &HashMap<&str, &RemoteFile>,
    ) -> Result<(Action, Target), Done> {
        let (key, url) = match &action {
            Action::Download { key, url, .. } | Action::Conflict { key, url, .. } => {
                (key.clone(), url.clone())
            }
            _ => return Err(failed("?", "azione inattesa")),
        };
        let (timemodified, size) = remote
            .get(key.as_str())
            .map_or((0, 0), |f| (f.timemodified, f.filesize));
        let (action, dest, precondition) = match action {
            Action::Download {
                ref path,
                reason: DownloadReason::Updated,
                ..
            } => {
                let dest = self.resolve(path)?;
                (action.clone(), dest, self.unchanged_since_baseline(&key))
            }
            Action::Download { ref path, .. } => {
                let dest = self.resolve(path)?;
                (action.clone(), dest, Precondition::Absent)
            }
            Action::Conflict {
                key,
                path,
                conflict_path,
                url,
            } => {
                let (copy, dest) = free_copy(self.root, &conflict_path)
                    .ok_or_else(|| failed(&conflict_path, UNSAFE_PATH))?;
                let renamed = Action::Conflict {
                    key,
                    path,
                    conflict_path: copy,
                    url,
                };
                (renamed, dest, Precondition::Absent)
            }
            other => return Err(failed(action_path(&other), "azione inattesa")),
        };
        let target = Target {
            url,
            dest,
            timemodified,
            size,
            precondition,
        };
        Ok((action, target))
    }

    fn resolve(&self, path: &str) -> Result<PathBuf, Done> {
        safe_join(self.root, path).ok_or_else(|| failed(path, UNSAFE_PATH))
    }

    /// An update may only replace the file as we last wrote it.
    fn unchanged_since_baseline(&self, key: &str) -> Precondition {
        self.baseline
            .get(key)
            .map_or(Precondition::Absent, |entry| {
                Precondition::Unchanged(LocalMeta {
                    size: entry.local_size,
                    mtime: entry.local_mtime,
                })
            })
    }

    /// A future that downloads a file or a conflict copy once a permit is free.
    fn transfer(
        &self,
        action: Action,
        target: Target,
    ) -> impl Future<Output = Done> + Send + 'static {
        let fetcher = self.fetcher.clone();
        let limit = Arc::clone(&self.limit);
        let cancel = Arc::clone(&self.cancel);
        async move {
            let shown = match &action {
                Action::Conflict { conflict_path, .. } => conflict_path.clone(),
                other => action_path(other).to_owned(),
            };
            let Ok(_permit) = limit.acquire_owned().await else {
                return failed(&shown, "sincronizzazione interrotta");
            };
            if cancel.load(Ordering::Acquire) {
                return Done::Cancelled;
            }
            match fetcher.fetch(&target).await {
                Ok(meta) => transferred(action, meta),
                Err(err) => failed(&shown, &err.to_string()),
            }
        }
    }

    fn apply_local(&self, action: Action) -> Done {
        match action {
            Action::Adopt { key, path } => {
                match safe_join(self.root, &path).and_then(|p| local_meta(&p)) {
                    Some(meta) => Done::Adopted { key, path, meta },
                    None => failed(&path, VANISHED),
                }
            }
            Action::Rename { key, from, to } => match rename(self.root, &from, &to) {
                Ok(()) => Done::Renamed { key, to },
                Err(message) => failed(&to, &message),
            },
            Action::KeepLocal { path, .. } => Done::KeptLocal(path),
            Action::RemovedRemotely { key, path } => Done::RemovedRemotely { key, path },
            Action::Download { path, .. } | Action::Conflict { path, .. } => {
                failed(&path, "azione inattesa")
            }
        }
    }

    fn fold(&self, done: Vec<Done>) -> CourseOutcome {
        let remote: HashMap<&str, &RemoteFile> = self
            .remote
            .iter()
            .map(|f| (f.remote_key.as_str(), f))
            .collect();
        let mut outcome = CourseOutcome::default();
        for item in done {
            if let Some(entry) = baseline_after(&item, &remote, self.baseline) {
                outcome.baseline.push(entry);
            }
            if let Done::RemovedRemotely { key, .. } = &item {
                outcome.forgotten.push(key.clone());
            }
            record(&mut outcome.report, item);
        }
        outcome
    }
}

/// The baseline entry an applied action leaves behind, if it changes one.
fn baseline_after(
    done: &Done,
    remote: &HashMap<&str, &RemoteFile>,
    baseline: &HashMap<String, BaselineEntry>,
) -> Option<(String, BaselineEntry)> {
    let from_remote = |key: &str, path: &str, meta: &LocalMeta| {
        let file = remote.get(key)?;
        Some((
            key.to_owned(),
            BaselineEntry {
                local_path: path.to_owned(),
                remote_timemodified: file.timemodified,
                remote_filesize: file.filesize,
                local_size: meta.size,
                local_mtime: meta.mtime,
            },
        ))
    };
    match done {
        Done::Downloaded {
            key, path, meta, ..
        }
        | Done::Adopted { key, path, meta } => from_remote(key, path, meta),
        // Tracking the copy's metadata against the original path keeps the edited original
        // "changed locally only" until WeBeep changes again: no conflict copy every sync.
        Done::Conflict {
            key, path, meta, ..
        } => from_remote(key, path, meta),
        Done::Renamed { key, to } => baseline.get(key).map(|old| {
            let moved = BaselineEntry {
                local_path: to.clone(),
                ..old.clone()
            };
            (key.clone(), moved)
        }),
        Done::KeptLocal(_) | Done::RemovedRemotely { .. } | Done::Failed(_) | Done::Cancelled => {
            None
        }
    }
}

fn record(report: &mut CourseReport, done: Done) {
    match done {
        Done::Downloaded {
            path,
            reason: DownloadReason::Updated,
            ..
        } => report.updated.push(path),
        Done::Downloaded { path, .. } => report.downloaded.push(path),
        Done::Conflict { copy, .. } => report.conflicts.push(copy),
        Done::Adopted { .. } => report.adopted += 1,
        Done::Renamed { .. } => report.renamed += 1,
        Done::KeptLocal(path) => report.kept_local.push(path),
        Done::RemovedRemotely { path, .. } => report.removed_remotely.push(path),
        Done::Failed(error) => report.errors.push(error),
        Done::Cancelled => {}
    }
}

fn transferred(action: Action, meta: LocalMeta) -> Done {
    match action {
        Action::Download {
            key, path, reason, ..
        } => Done::Downloaded {
            key,
            path,
            reason,
            meta,
        },
        Action::Conflict {
            key,
            path,
            conflict_path,
            ..
        } => Done::Conflict {
            key,
            path,
            copy: conflict_path,
            meta,
        },
        _ => failed("?", "azione inattesa"),
    }
}

/// Moves an untouched file to its new path, never over an existing file.
fn rename(root: &Path, from: &str, to: &str) -> Result<(), String> {
    let (Some(src), Some(dest)) = (safe_join(root, from), safe_join(root, to)) else {
        return Err(UNSAFE_PATH.to_owned());
    };
    if dest.exists() {
        return Err("esiste già un file con il nuovo nome".to_owned());
    }
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::rename(&src, &dest).map_err(|e| e.to_string())
}

/// The first free name among `copy`, `copy (2)`, `copy (3)`, … (before the extension).
fn free_copy(root: &Path, copy: &str) -> Option<(String, PathBuf)> {
    let (dir, name) = match copy.rsplit_once('/') {
        Some((dir, name)) => (format!("{dir}/"), name),
        None => (String::new(), copy),
    };
    let (stem, ext) = match extension(name) {
        Some(ext) => (&name[..name.len() - ext.len() - 1], format!(".{ext}")),
        None => (name, String::new()),
    };
    (1..=MAX_COPIES).find_map(|n| {
        let candidate = match n {
            1 => copy.to_owned(),
            n => format!("{dir}{stem} ({n}){ext}"),
        };
        let dest = safe_join(root, &candidate)?;
        (!dest.exists()).then_some((candidate, dest))
    })
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

fn failed(path: &str, message: &str) -> Done {
    Done::Failed(FileError {
        path: path.to_owned(),
        message: crate::i18n::localize(message),
    })
}

#[cfg(test)]
#[path = "execute_tests.rs"]
mod tests;
