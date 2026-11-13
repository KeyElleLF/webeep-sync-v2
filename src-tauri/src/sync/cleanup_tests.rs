use std::path::Path;

use super::*;
use crate::sync::local::snapshot;

/// A course folder on disk plus the baseline the app would have recorded for it.
struct Course {
    dir: tempfile::TempDir,
    bin: tempfile::TempDir,
    baseline: HashMap<String, BaselineEntry>,
}

impl Course {
    fn new() -> Self {
        Self {
            dir: tempfile::tempdir().unwrap(),
            bin: tempfile::tempdir().unwrap(),
            baseline: HashMap::new(),
        }
    }

    fn root(&self) -> &Path {
        self.dir.path()
    }

    /// A file downloaded by the app (recorded in the baseline as written).
    fn downloaded(&mut self, rel: &str, body: &[u8]) {
        self.write(rel, body);
        let meta = local_meta(&safe_join(self.root(), rel).unwrap()).unwrap();
        self.baseline.insert(
            format!("key:{rel}"),
            BaselineEntry {
                local_path: rel.to_owned(),
                remote_timemodified: 1,
                remote_filesize: meta.size,
                local_size: meta.size,
                local_mtime: meta.mtime,
            },
        );
    }

    fn write(&self, rel: &str, body: &[u8]) {
        let path = safe_join(self.root(), rel).unwrap();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }

    fn exists(&self, rel: &str) -> bool {
        safe_join(self.root(), rel).unwrap().exists()
    }

    fn plan(&self) -> CleanupPlan {
        let paths: Vec<&str> = self
            .baseline
            .values()
            .map(|e| e.local_path.as_str())
            .collect();
        plan_cleanup(&self.baseline, &snapshot(self.root(), paths))
    }

    /// Fake bin: moves the file into a temporary folder instead of the real Recycle Bin.
    fn run(&self, include_modified: bool) -> (CleanupReport, Vec<String>) {
        let bin = self.bin.path().to_path_buf();
        let counter = std::sync::atomic::AtomicUsize::new(0);
        execute_cleanup(self.root(), &self.plan(), include_modified, |path| {
            let n = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            std::fs::rename(path, bin.join(n.to_string())).map_err(|e| e.to_string())
        })
    }
}

#[test]
fn plans_untouched_modified_and_gone_files() {
    let mut course = Course::new();
    course.downloaded("Materiali/a.pdf", b"aaa");
    course.downloaded("b.pdf", b"bb");
    course.downloaded("gone.pdf", b"g");
    course.write("b.pdf", b"bb + annotations");
    std::fs::remove_file(course.root().join("gone.pdf")).unwrap();
    course.write("mine.txt", b"user notes");

    let plan = course.plan();

    let paths = |files: &[CleanupFile]| files.iter().map(|f| f.path.clone()).collect::<Vec<_>>();
    assert_eq!(paths(&plan.files), ["Materiali/a.pdf"]);
    assert_eq!(paths(&plan.modified), ["b.pdf"]);
    assert_eq!(paths(&plan.gone), ["gone.pdf"]);
    assert_eq!(plan.bytes(), 3);
}

#[test]
fn a_file_changed_after_the_plan_with_the_same_size_is_not_moved() {
    let mut course = Course::new();
    course.downloaded("a.pdf", b"aaa");
    let plan = course.plan();
    // Same size, different date: e.g. replaced by another file in the meantime.
    let path = course.root().join("a.pdf");
    std::fs::write(&path, b"zzz").unwrap();
    let later = std::time::SystemTime::now() + std::time::Duration::from_secs(120);
    std::fs::File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_modified(later)
        .unwrap();

    let (report, forget) = execute_cleanup(course.root(), &plan, false, |_| {
        panic!("must not be called")
    });

    assert_eq!(report.skipped.len(), 1);
    assert!(forget.is_empty());
    assert!(course.exists("a.pdf"));
}

#[test]
fn a_gone_file_that_reappeared_is_not_forgotten() {
    let mut course = Course::new();
    course.downloaded("a.pdf", b"aaa");
    std::fs::remove_file(course.root().join("a.pdf")).unwrap();
    let plan = course.plan();
    course.write("a.pdf", b"back again");

    let (_, forget) = execute_cleanup(course.root(), &plan, false, |_| Ok(()));

    assert!(forget.is_empty());
}

#[test]
fn trashes_downloads_keeps_user_files_and_modified_ones() {
    let mut course = Course::new();
    course.downloaded("Materiali/a.pdf", b"aaa");
    course.downloaded("Esercizi/x.zip", b"xx");
    course.downloaded("b.pdf", b"bb");
    course.write("b.pdf", b"annotated");
    course.write("Esercizi/mie soluzioni.txt", b"mine");

    let (report, forget) = course.run(false);

    assert_eq!(report.trashed, 2);
    assert_eq!(report.freed, 5);
    assert_eq!(report.kept, 1);
    assert!(report.skipped.is_empty());
    assert!(!course.exists("Materiali/a.pdf"));
    assert!(!course.exists("Materiali"), "empty folder removed");
    assert!(
        course.exists("Esercizi/mie soluzioni.txt"),
        "user file kept"
    );
    assert!(course.exists("b.pdf"), "modified file kept");
    let mut forget = forget;
    forget.sort();
    assert_eq!(forget, ["key:Esercizi/x.zip", "key:Materiali/a.pdf"]);
    // The course folder still holds user files: it stays.
    assert!(course.root().exists());
}

#[test]
fn modified_files_go_only_when_asked() {
    let mut course = Course::new();
    course.downloaded("b.pdf", b"bb");
    course.write("b.pdf", b"annotated");

    let (report, forget) = course.run(true);

    assert_eq!(report.trashed, 1);
    assert_eq!(report.kept, 0);
    assert_eq!(forget, ["key:b.pdf"]);
    assert!(!course.exists("b.pdf"));
}

#[test]
fn an_emptied_course_folder_is_removed() {
    let mut course = Course::new();
    course.downloaded("Sezione/Modulo/a.pdf", b"a");

    course.run(false);

    assert!(!course.root().exists());
}

#[test]
fn a_file_that_cannot_be_moved_is_skipped() {
    let mut course = Course::new();
    course.downloaded("a.pdf", b"a");
    course.downloaded("b.pdf", b"b");

    let (report, forget) = execute_cleanup(course.root(), &course.plan(), false, |path| {
        if path.ends_with("a.pdf") {
            Err("file aperto in un altro programma".to_owned())
        } else {
            std::fs::remove_file(path).map_err(|e| e.to_string())
        }
    });

    assert_eq!(report.trashed, 1);
    assert_eq!(report.skipped.len(), 1);
    assert_eq!(report.skipped[0].path, "a.pdf");
    assert_eq!(forget, ["key:b.pdf"]);
    assert!(course.exists("a.pdf"));
}
