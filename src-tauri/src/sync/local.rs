//! Local disk side of the sync: resolving relative paths safely and reading file metadata.

use std::collections::HashMap;
use std::fs::Metadata;
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::sync::plan::LocalMeta;

/// Joins a relative path (`/`-separated, as in the baseline) to `root`. Returns `None` for
/// anything that could escape `root`: absolute paths, drive prefixes, `.` and `..`.
pub fn safe_join(root: &Path, relative: &str) -> Option<PathBuf> {
    let mut joined = root.to_path_buf();
    for part in relative.split('/') {
        let mut components = Path::new(part).components();
        match (components.next(), components.next()) {
            // `:` and `\` never survive `sanitize_component`; rejecting them keeps drive
            // prefixes out on every platform.
            (Some(Component::Normal(name)), None) if !part.contains([':', '\\']) => {
                joined.push(name)
            }
            _ => return None,
        }
    }
    Some(joined)
}

/// Size and modification time (whole seconds) of a regular file, `None` if missing.
pub fn local_meta(path: &Path) -> Option<LocalMeta> {
    let metadata = std::fs::metadata(path).ok()?;
    metadata.is_file().then(|| meta_of(&metadata))
}

pub fn meta_of(metadata: &Metadata) -> LocalMeta {
    LocalMeta {
        size: metadata.len(),
        mtime: metadata.modified().map(unix_seconds).unwrap_or_default(),
    }
}

/// Metadata of the given relative paths under `root`, skipping the missing ones.
pub fn snapshot<'a>(
    root: &Path,
    paths: impl IntoIterator<Item = &'a str>,
) -> HashMap<String, LocalMeta> {
    paths
        .into_iter()
        .filter_map(|rel| {
            let meta = local_meta(&safe_join(root, rel)?)?;
            Some((rel.to_owned(), meta))
        })
        .collect()
}

pub fn unix_seconds(time: SystemTime) -> i64 {
    match time.duration_since(UNIX_EPOCH) {
        Ok(after) => i64::try_from(after.as_secs()).unwrap_or(i64::MAX),
        Err(before) => -i64::try_from(before.duration().as_secs()).unwrap_or(i64::MAX),
    }
}

/// `None` for non-positive timestamps (unknown dates of linked files).
pub fn system_time(unix: i64) -> Option<SystemTime> {
    let secs = u64::try_from(unix).ok().filter(|s| *s > 0)?;
    UNIX_EPOCH.checked_add(Duration::from_secs(secs))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_normal_components() {
        let root = Path::new("C:\\polimi");
        assert_eq!(
            safe_join(root, "Data Mining/Materiali/a.pdf"),
            Some(root.join("Data Mining").join("Materiali").join("a.pdf"))
        );
    }

    #[test]
    fn rejects_escaping_paths() {
        let root = Path::new("C:\\polimi");
        for bad in [
            "../x",
            "a/../../x",
            "./x",
            "",
            "a//b",
            "C:/Windows/x",
            "a\\..\\x",
            "/abs",
        ] {
            assert_eq!(safe_join(root, bad), None, "{bad}");
        }
    }

    #[test]
    fn snapshot_reads_only_existing_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("sub")).unwrap();
        std::fs::write(dir.path().join("sub").join("a.txt"), b"hello").unwrap();

        let found = snapshot(dir.path(), ["sub/a.txt", "sub/missing.txt", "sub", "../x"]);

        assert_eq!(found.len(), 1);
        assert_eq!(found["sub/a.txt"].size, 5);
        assert!(found["sub/a.txt"].mtime > 0);
    }

    #[test]
    fn converts_unix_times_both_ways() {
        let time = system_time(1_700_000_000).unwrap();
        assert_eq!(unix_seconds(time), 1_700_000_000);
        assert_eq!(system_time(0), None);
        assert_eq!(system_time(-5), None);
        assert_eq!(unix_seconds(UNIX_EPOCH - Duration::from_secs(3)), -3);
    }
}
