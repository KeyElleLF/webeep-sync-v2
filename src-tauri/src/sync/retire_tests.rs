use super::*;
use crate::sync::local::local_meta;
use crate::sync::plan::BaselineEntry;

const KEY: &str = "056892";

struct Env {
    dir: tempfile::TempDir,
    store: Store,
}

impl Env {
    /// Two courses of the same teaching; only the first has downloaded files.
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let mut store = Store::open_in_memory().unwrap();
        store.set_course_folder(1, "DATA MINING").unwrap();
        let course = dir.path().join("DATA MINING");
        std::fs::create_dir_all(&course).unwrap();
        let mut entries = Vec::new();
        for (name, body) in [("a.pdf", &b"aaa"[..]), ("b.pdf", &b"bb"[..])] {
            std::fs::write(course.join(name), body).unwrap();
            let meta = local_meta(&course.join(name)).unwrap();
            let entry = BaselineEntry {
                local_path: name.to_owned(),
                remote_timemodified: 1,
                remote_filesize: meta.size,
                local_size: meta.size,
                local_mtime: meta.mtime,
            };
            entries.push((format!("k:{name}"), entry));
        }
        store.save_baseline(1, &entries).unwrap();
        store.set_course_selected(1, true).unwrap();
        Self { dir, store }
    }

    fn group() -> Vec<GroupCourse> {
        vec![
            GroupCourse {
                id: 1,
                name: "DATA MINING".to_owned(),
            },
            GroupCourse {
                id: 2,
                name: "DATA MINING".to_owned(),
            },
        ]
    }

    fn mark(&mut self, state: Option<ManualState>, policy: CleanupPolicy) -> StateOutcome {
        let group = Self::group();
        let change = StateChange {
            key: KEY,
            state,
            group: &group,
            policy,
            root: Some(self.dir.path()),
            now: 100,
        };
        set_teaching_state(&mut self.store, &change, &remove).unwrap()
    }

    fn course_file(&self, name: &str) -> std::path::PathBuf {
        self.dir.path().join("DATA MINING").join(name)
    }
}

fn remove(path: &Path) -> Result<(), String> {
    std::fs::remove_file(path).map_err(|e| e.to_string())
}

#[test]
fn passed_with_ask_queues_only_courses_with_files() {
    let mut env = Env::new();

    let outcome = env.mark(Some(ManualState::Passed), CleanupPolicy::Ask);

    assert_eq!(outcome.queued, 1);
    let pending = env.store.pending_cleanups().unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].course_id, 1);
    assert!(env.store.course_selection().unwrap().is_empty());
    assert!(env.course_file("a.pdf").exists(), "nothing deleted yet");
}

#[test]
fn reactivating_drops_the_pending_cleanup() {
    let mut env = Env::new();
    env.mark(Some(ManualState::Passed), CleanupPolicy::Ask);

    env.mark(None, CleanupPolicy::Ask);

    assert!(env.store.pending_cleanups().unwrap().is_empty());
    assert_eq!(env.store.teaching_prefs().unwrap()[KEY].state, None);
}

#[test]
fn passed_with_always_cleans_untouched_files_now() {
    let mut env = Env::new();
    std::fs::write(env.course_file("b.pdf"), b"annotated").unwrap();

    let outcome = env.mark(Some(ManualState::Passed), CleanupPolicy::Always);

    assert_eq!(outcome.cleaned.len(), 1);
    assert_eq!(outcome.cleaned[0].trashed, 1);
    assert_eq!(outcome.cleaned[0].kept, 1);
    assert!(!env.course_file("a.pdf").exists());
    assert!(env.course_file("b.pdf").exists(), "modified file kept");
    let left: Vec<String> = env.store.baseline(1).unwrap().into_keys().collect();
    assert_eq!(left, ["k:b.pdf"]);
}

#[test]
fn passed_with_never_touches_nothing() {
    let mut env = Env::new();

    let outcome = env.mark(Some(ManualState::Passed), CleanupPolicy::Never);

    assert_eq!(outcome, StateOutcome::default());
    assert!(env.store.pending_cleanups().unwrap().is_empty());
    assert!(env.course_file("a.pdf").exists());
}

#[test]
fn confirming_without_a_pending_request_touches_nothing() {
    let mut env = Env::new();

    let report = confirm_pending_cleanup(&mut env.store, env.dir.path(), 1, true, &remove).unwrap();

    assert_eq!(report, None);
    assert!(env.course_file("a.pdf").exists());
    assert_eq!(env.store.baseline(1).unwrap().len(), 2);
}

#[test]
fn preview_then_confirmed_cleanup_with_modified_files() {
    let mut env = Env::new();
    env.mark(Some(ManualState::Passed), CleanupPolicy::Ask);
    std::fs::write(env.course_file("b.pdf"), b"annotated").unwrap();

    let preview = preview(&env.store, env.dir.path(), 1, "DATA MINING").unwrap();
    assert_eq!((preview.files, preview.bytes), (1, 3));
    assert_eq!(preview.modified.len(), 1);

    let report = confirm_pending_cleanup(&mut env.store, env.dir.path(), 1, true, &remove)
        .unwrap()
        .unwrap();

    assert_eq!(report.trashed, 2);
    assert!(
        !env.dir.path().join("DATA MINING").exists(),
        "empty course folder removed"
    );
    assert!(env.store.baseline(1).unwrap().is_empty());
    assert!(env.store.pending_cleanups().unwrap().is_empty());
}
