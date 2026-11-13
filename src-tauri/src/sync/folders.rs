//! Changing the folder of a course.
//!
//! - The old folder exists and the new one does not: the folder is moved on disk and the
//!   baseline (paths relative to the course folder) stays valid.
//! - The new folder already exists (e.g. a v1 folder with the full course name): the course
//!   simply points there and its baseline is cleared, so the next sync adopts the files with
//!   matching size and date. The old folder is left untouched.

use std::path::Path;

use serde::Serialize;
use thiserror::Error;

use crate::store::{Store, StoreError};
use crate::sync::local::safe_join;
use crate::sync::paths::sanitize_component;

#[derive(Debug, Error)]
pub enum FolderError {
    #[error("il nome della cartella non è valido")]
    Invalid,
    #[error("la cartella «{0}» è già usata da un altro corso")]
    Taken(String),
    #[error("impossibile spostare la cartella: {0}")]
    Move(String),
    #[error(transparent)]
    Store(#[from] StoreError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FolderChange {
    /// Nothing changed.
    Same,
    /// The existing folder was renamed on disk.
    Moved,
    /// The course now uses a folder that already existed (or neither exists yet).
    Repointed,
}

/// Validates `requested` (it must already be a valid folder name) and frees of conflicts
/// with other courses, case-insensitively.
pub fn validate_folder(
    requested: &str,
    course_id: u64,
    store: &Store,
) -> Result<String, FolderError> {
    let name = requested.trim();
    if name.is_empty() || sanitize_component(name) != name {
        return Err(FolderError::Invalid);
    }
    let lower = name.to_lowercase();
    let taken = store
        .course_folders()?
        .into_iter()
        .any(|(id, folder)| id != course_id && folder.to_lowercase() == lower);
    if taken {
        return Err(FolderError::Taken(name.to_owned()));
    }
    Ok(name.to_owned())
}

pub fn change_course_folder(
    root: &Path,
    store: &Store,
    course_id: u64,
    requested: &str,
) -> Result<FolderChange, FolderError> {
    let new = validate_folder(requested, course_id, store)?;
    let old = store.course_folders()?.remove(&course_id);
    if old.as_deref() == Some(new.as_str()) {
        return Ok(FolderChange::Same);
    }
    let new_path = safe_join(root, &new).ok_or(FolderError::Invalid)?;
    let old_path = old.as_deref().and_then(|old| safe_join(root, old));
    let change = match old_path {
        // A case-only rename is a move too: on Windows `exists` sees the same folder. On
        // case-sensitive systems a different existing folder is never replaced.
        Some(old_path)
            if old_path.is_dir() && (!new_path.exists() || same_folder(&old_path, &new_path)) =>
        {
            std::fs::rename(&old_path, &new_path).map_err(|e| FolderError::Move(e.to_string()))?;
            FolderChange::Moved
        }
        _ => {
            store.clear_baseline(course_id)?;
            FolderChange::Repointed
        }
    };
    store.set_course_folder(course_id, &new)?;
    Ok(change)
}

/// Whether two paths name the same folder on disk (e.g. differing only in case on Windows).
fn same_folder(a: &Path, b: &Path) -> bool {
    match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::plan::BaselineEntry;

    fn setup() -> (tempfile::TempDir, Store) {
        let mut store = Store::open_in_memory().unwrap();
        store.set_course_folder(1, "DATA MINING").unwrap();
        store.set_course_folder(2, "MACHINE LEARNING").unwrap();
        let entry = BaselineEntry {
            local_path: "a.pdf".to_owned(),
            remote_timemodified: 1,
            remote_filesize: 1,
            local_size: 1,
            local_mtime: 1,
        };
        store.save_baseline(1, &[("k".to_owned(), entry)]).unwrap();
        (tempfile::tempdir().unwrap(), store)
    }

    #[test]
    fn moves_the_folder_and_keeps_the_baseline() {
        let (dir, store) = setup();
        std::fs::create_dir(dir.path().join("DATA MINING")).unwrap();
        std::fs::write(dir.path().join("DATA MINING").join("a.pdf"), b"x").unwrap();

        let change = change_course_folder(dir.path(), &store, 1, "Data Mining 2026").unwrap();

        assert_eq!(change, FolderChange::Moved);
        assert!(dir.path().join("Data Mining 2026").join("a.pdf").exists());
        assert!(!dir.path().join("DATA MINING").exists());
        assert_eq!(store.course_folders().unwrap()[&1], "Data Mining 2026");
        assert_eq!(store.baseline(1).unwrap().len(), 1);
    }

    #[test]
    fn case_only_rename_moves_the_folder() {
        let (dir, store) = setup();
        std::fs::create_dir(dir.path().join("DATA MINING")).unwrap();

        let change = change_course_folder(dir.path(), &store, 1, "Data Mining").unwrap();

        assert_eq!(change, FolderChange::Moved);
        assert_eq!(store.course_folders().unwrap()[&1], "Data Mining");
        assert_eq!(store.baseline(1).unwrap().len(), 1);
    }

    #[test]
    fn points_to_an_existing_folder_and_clears_the_baseline() {
        let (dir, store) = setup();
        std::fs::create_dir(dir.path().join("DATA MINING")).unwrap();
        let v1 = "056892 - DATA MINING (DOCENTE M) [2026-27]";
        std::fs::create_dir(dir.path().join(v1)).unwrap();

        let change = change_course_folder(dir.path(), &store, 1, v1).unwrap();

        assert_eq!(change, FolderChange::Repointed);
        assert!(
            dir.path().join("DATA MINING").exists(),
            "old folder untouched"
        );
        assert_eq!(store.course_folders().unwrap()[&1], v1);
        assert!(store.baseline(1).unwrap().is_empty());
    }

    #[test]
    fn same_name_changes_nothing() {
        let (dir, store) = setup();
        let change = change_course_folder(dir.path(), &store, 1, " DATA MINING ").unwrap();
        assert_eq!(change, FolderChange::Same);
        assert_eq!(store.baseline(1).unwrap().len(), 1);
    }

    #[test]
    fn refuses_invalid_or_taken_names() {
        let (dir, store) = setup();
        for bad in ["", "a/b", "..", "CON", "x:y", "fine."] {
            assert!(
                matches!(
                    change_course_folder(dir.path(), &store, 1, bad),
                    Err(FolderError::Invalid)
                ),
                "{bad}"
            );
        }
        assert!(matches!(
            change_course_folder(dir.path(), &store, 1, "machine learning"),
            Err(FolderError::Taken(_))
        ));
        assert_eq!(store.course_folders().unwrap()[&1], "DATA MINING");
    }
}
