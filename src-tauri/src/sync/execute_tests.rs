use std::time::Duration;

use url::Url;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::*;
use crate::auth::token::WsToken;
use crate::sync::download::RetryPolicy;

const TM: i64 = 1_700_000_000;
const FILES: &str = "/webservice/pluginfile.php/f/";

struct Fixture {
    server: MockServer,
    dir: tempfile::TempDir,
    fetcher: Fetcher,
}

impl Fixture {
    async fn new() -> Self {
        let server = MockServer::start().await;
        let retry = RetryPolicy {
            attempts: 1,
            base_delay: Duration::from_millis(1),
        };
        let site = Url::parse(&server.uri()).unwrap();
        let fetcher = Fetcher::new(site, WsToken::new("t"), retry).unwrap();
        Self {
            server,
            dir: tempfile::tempdir().unwrap(),
            fetcher,
        }
    }

    async fn serve(&self, name: &str, body: &[u8]) {
        Mock::given(method("GET"))
            .and(path(format!("{FILES}{name}")))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(body.to_vec()))
            .mount(&self.server)
            .await;
    }

    fn remote(&self, key: &str, rel: &str, size: u64) -> RemoteFile {
        let name = rel.rsplit('/').next().unwrap_or(rel);
        RemoteFile {
            module_id: 1,
            remote_key: key.to_owned(),
            local_path: rel.split('/').map(str::to_owned).collect(),
            fileurl: format!("{}{FILES}{name}", self.server.uri()),
            filesize: size,
            timemodified: TM,
        }
    }

    fn write(&self, rel: &str, body: &[u8]) {
        let dest = safe_join(self.dir.path(), rel).unwrap();
        std::fs::create_dir_all(dest.parent().unwrap()).unwrap();
        std::fs::write(dest, body).unwrap();
    }

    fn read(&self, rel: &str) -> Option<Vec<u8>> {
        std::fs::read(safe_join(self.dir.path(), rel).unwrap()).ok()
    }

    async fn run(
        &self,
        remote: &[RemoteFile],
        baseline: &HashMap<String, BaselineEntry>,
        actions: Vec<Action>,
    ) -> CourseOutcome {
        let run = CourseRun {
            fetcher: &self.fetcher,
            limit: Arc::new(Semaphore::new(2)),
            root: self.dir.path(),
            remote,
            baseline,
        };
        run.execute(actions).await
    }
}

fn download(file: &RemoteFile, reason: DownloadReason) -> Action {
    Action::Download {
        key: file.remote_key.clone(),
        path: file.local_path.join("/"),
        url: file.fileurl.clone(),
        reason,
    }
}

#[tokio::test]
async fn downloads_adopts_and_reports_the_rest() {
    let fx = Fixture::new().await;
    fx.serve("new.pdf", b"nuovo").await;
    fx.serve("upd.pdf", b"aggiornato").await;
    fx.write("Materiali/old.pdf", b"v1");
    let new = fx.remote("1:/new.pdf", "Materiali/new.pdf", 5);
    let upd = fx.remote("1:/upd.pdf", "upd.pdf", 10);
    let old = fx.remote("2:/old.pdf", "Materiali/old.pdf", 2);
    let actions = vec![
        download(&new, DownloadReason::New),
        download(&upd, DownloadReason::Updated),
        Action::Adopt {
            key: old.remote_key.clone(),
            path: "Materiali/old.pdf".to_owned(),
        },
        Action::KeepLocal {
            key: "3:/n.pdf".to_owned(),
            path: "n.pdf".to_owned(),
        },
        Action::RemovedRemotely {
            key: "4:/r.pdf".to_owned(),
            path: "r.pdf".to_owned(),
        },
    ];

    let outcome = fx.run(&[new, upd, old], &HashMap::new(), actions).await;

    assert_eq!(fx.read("Materiali/new.pdf").as_deref(), Some(&b"nuovo"[..]));
    let report = &outcome.report;
    assert_eq!(report.downloaded, ["Materiali/new.pdf"]);
    assert_eq!(report.updated, ["upd.pdf"]);
    assert_eq!(report.adopted, 1);
    assert_eq!(report.kept_local, ["n.pdf"]);
    assert_eq!(report.removed_remotely, ["r.pdf"]);
    assert!(report.errors.is_empty(), "{:?}", report.errors);

    let saved: HashMap<_, _> = outcome.baseline.into_iter().collect();
    assert_eq!(saved.len(), 3);
    let entry = &saved["1:/new.pdf"];
    assert_eq!(entry.local_path, "Materiali/new.pdf");
    assert_eq!((entry.remote_filesize, entry.remote_timemodified), (5, TM));
    assert_eq!((entry.local_size, entry.local_mtime), (5, TM));
    assert_eq!(saved["2:/old.pdf"].local_size, 2);
}

#[tokio::test]
async fn conflict_keeps_the_original_and_saves_a_copy() {
    let fx = Fixture::new().await;
    fx.serve("slides.pdf", b"versione docente").await;
    fx.write("slides.pdf", b"annotate");
    let file = fx.remote("1:/slides.pdf", "slides.pdf", 16);
    let copy = "slides (WeBeep aggiornato 2026-11-09).pdf";
    let actions = vec![Action::Conflict {
        key: file.remote_key.clone(),
        path: "slides.pdf".to_owned(),
        conflict_path: copy.to_owned(),
        url: file.fileurl.clone(),
    }];

    let outcome = fx.run(&[file], &HashMap::new(), actions).await;

    assert_eq!(fx.read("slides.pdf").as_deref(), Some(&b"annotate"[..]));
    assert_eq!(fx.read(copy).as_deref(), Some(&b"versione docente"[..]));
    assert_eq!(outcome.report.conflicts, [copy]);
    let (_, entry) = &outcome.baseline[0];
    assert_eq!(entry.local_path, "slides.pdf");
    assert_eq!((entry.local_size, entry.local_mtime), (16, TM));
}

#[tokio::test]
async fn conflict_copies_never_overwrite_each_other() {
    let fx = Fixture::new().await;
    fx.serve("slides.pdf", b"terza versione!!").await;
    let copy = "slides (WeBeep aggiornato 2026-11-09).pdf";
    fx.write(copy, b"seconda versione, annotata");
    let file = fx.remote("1:/slides.pdf", "slides.pdf", 16);
    let actions = vec![Action::Conflict {
        key: file.remote_key.clone(),
        path: "slides.pdf".to_owned(),
        conflict_path: copy.to_owned(),
        url: file.fileurl.clone(),
    }];

    let outcome = fx.run(&[file], &HashMap::new(), actions).await;

    let second = "slides (WeBeep aggiornato 2026-11-09) (2).pdf";
    assert_eq!(outcome.report.conflicts, [second]);
    assert_eq!(
        fx.read(copy).as_deref(),
        Some(&b"seconda versione, annotata"[..])
    );
    assert_eq!(fx.read(second).as_deref(), Some(&b"terza versione!!"[..]));
}

#[tokio::test]
async fn an_update_never_replaces_a_file_edited_since_the_plan() {
    let fx = Fixture::new().await;
    fx.serve("a.pdf", b"new").await;
    fx.write("a.pdf", b"annotated after the plan");
    let file = fx.remote("1:/a.pdf", "a.pdf", 3);
    // The baseline describes the file as it was when we wrote it, not as it is now.
    let baseline = HashMap::from([(
        "1:/a.pdf".to_owned(),
        BaselineEntry {
            local_path: "a.pdf".to_owned(),
            remote_timemodified: TM - 1,
            remote_filesize: 3,
            local_size: 3,
            local_mtime: TM - 1,
        },
    )]);
    let actions = vec![download(&file, DownloadReason::Updated)];

    let outcome = fx.run(&[file], &baseline, actions).await;

    assert_eq!(
        fx.read("a.pdf").as_deref(),
        Some(&b"annotated after the plan"[..])
    );
    assert_eq!(outcome.report.errors.len(), 1);
    assert!(outcome.baseline.is_empty());
}

#[tokio::test]
async fn rename_moves_the_file_and_the_baseline() {
    let fx = Fixture::new().await;
    fx.write("Vecchia/a.pdf", b"aaa");
    let old_entry = BaselineEntry {
        local_path: "Vecchia/a.pdf".to_owned(),
        remote_timemodified: TM,
        remote_filesize: 3,
        local_size: 3,
        local_mtime: 42,
    };
    let baseline = HashMap::from([("1:/a.pdf".to_owned(), old_entry.clone())]);
    let actions = vec![Action::Rename {
        key: "1:/a.pdf".to_owned(),
        from: "Vecchia/a.pdf".to_owned(),
        to: "Nuova/a.pdf".to_owned(),
    }];

    let outcome = fx.run(&[], &baseline, actions).await;

    assert_eq!(fx.read("Vecchia/a.pdf"), None);
    assert_eq!(fx.read("Nuova/a.pdf").as_deref(), Some(&b"aaa"[..]));
    assert_eq!(outcome.report.renamed, 1);
    let expected = BaselineEntry {
        local_path: "Nuova/a.pdf".to_owned(),
        ..old_entry
    };
    assert_eq!(outcome.baseline, vec![("1:/a.pdf".to_owned(), expected)]);
}

#[tokio::test]
async fn rename_never_overwrites() {
    let fx = Fixture::new().await;
    fx.write("a.pdf", b"old");
    fx.write("b.pdf", b"user file");
    let actions = vec![Action::Rename {
        key: "k".to_owned(),
        from: "a.pdf".to_owned(),
        to: "b.pdf".to_owned(),
    }];

    let outcome = fx.run(&[], &HashMap::new(), actions).await;

    assert_eq!(fx.read("b.pdf").as_deref(), Some(&b"user file"[..]));
    assert_eq!(outcome.report.errors.len(), 1);
    assert!(outcome.baseline.is_empty());
}

#[tokio::test]
async fn a_failed_file_does_not_stop_the_others() {
    let fx = Fixture::new().await;
    fx.serve("ok.pdf", b"ok").await;
    // `missing.pdf` is not served: wiremock answers 404.
    let ok = fx.remote("1:/ok.pdf", "ok.pdf", 2);
    let missing = fx.remote("1:/missing.pdf", "missing.pdf", 2);
    let escaping = Action::Download {
        key: "x".to_owned(),
        path: "../fuori.pdf".to_owned(),
        url: ok.fileurl.clone(),
        reason: DownloadReason::New,
    };
    let actions = vec![
        download(&missing, DownloadReason::New),
        download(&ok, DownloadReason::New),
        escaping,
        Action::Adopt {
            key: "y".to_owned(),
            path: "sparito.pdf".to_owned(),
        },
    ];

    let outcome = fx.run(&[ok, missing], &HashMap::new(), actions).await;

    assert_eq!(outcome.report.downloaded, ["ok.pdf"]);
    let mut failed: Vec<&str> = outcome
        .report
        .errors
        .iter()
        .map(|e| e.path.as_str())
        .collect();
    failed.sort_unstable();
    assert_eq!(failed, ["../fuori.pdf", "missing.pdf", "sparito.pdf"]);
    assert_eq!(outcome.baseline.len(), 1);
    assert!(!fx.dir.path().parent().unwrap().join("fuori.pdf").exists());
}
