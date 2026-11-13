use std::time::Duration;

use serde_json::json;
use url::Url;
use wiremock::matchers::{body_string_contains, method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::*;
use crate::auth::token::WsToken;
use crate::sync::download::RetryPolicy;
use crate::test_support::fixture;

const DIQ: u64 = 24951;
const BROKEN: u64 = 999;
const REAL_SITE: &str = "https://webeep.polimi.it";

struct Env {
    server: MockServer,
    client: MoodleClient,
    fetcher: Fetcher,
    store: Mutex<Store>,
    dir: tempfile::TempDir,
}

impl Env {
    async fn new() -> Self {
        let server = MockServer::start().await;
        let site = Url::parse(&format!("{}/", server.uri())).unwrap();
        let retry = RetryPolicy {
            attempts: 1,
            base_delay: Duration::from_millis(1),
        };
        Self {
            client: MoodleClient::new(&site, WsToken::new("t")).unwrap(),
            fetcher: Fetcher::new(site, WsToken::new("t"), retry).unwrap(),
            store: Mutex::new(Store::open_in_memory().unwrap()),
            dir: tempfile::tempdir().unwrap(),
            server,
        }
    }

    /// Serves the real DIQ structure (links rewritten to the mock) and every file.
    async fn serve_course(&self) {
        let contents = fixture("contents/data_information_quality.json")
            .replace(REAL_SITE, &self.server.uri());
        // Every file is served as `data`: the declared sizes must match or it is refused.
        let contents = regex::Regex::new(r#""filesize":\s*\d+"#)
            .unwrap()
            .replace_all(&contents, r#""filesize": 4"#)
            .into_owned();
        self.rest("core_course_get_contents", &format!("courseid={DIQ}"))
            .respond_with(ResponseTemplate::new(200).set_body_string(contents))
            .mount(&self.server)
            .await;
        Mock::given(method("GET"))
            .and(path_regex("^/webservice/pluginfile.php/"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(b"data".to_vec()))
            .mount(&self.server)
            .await;
        self.rest("mod_assign_get_assignments", "")
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"courses": []})))
            .mount(&self.server)
            .await;
    }

    fn rest(&self, function: &str, body: &str) -> wiremock::MockBuilder {
        Mock::given(method("POST"))
            .and(path("/webservice/rest/server.php"))
            .and(body_string_contains(format!("wsfunction={function}")))
            .and(body_string_contains(body))
    }

    fn context(&self) -> SyncContext<'_> {
        SyncContext {
            client: &self.client,
            fetcher: &self.fetcher,
            store: &self.store,
            root: self.dir.path(),
            limit: Arc::new(Semaphore::new(4)),
            today: NaiveDate::from_ymd_opt(2026, 11, 9).unwrap(),
            now: 1_794_000_000,
            options: TreeOptions::default(),
        }
    }
}

fn course(id: u64, name: &str) -> CourseInfo {
    CourseInfo {
        id,
        fullname: name.to_owned(),
        name: name.to_owned(),
        academic_year: "2026-27".to_owned(),
        current: true,
        selected: true,
        ..CourseInfo::default()
    }
}

fn exception(errorcode: &str) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({
        "exception": "moodle_exception", "errorcode": errorcode, "message": "x"
    }))
}

#[tokio::test]
async fn first_sync_downloads_everything_and_second_does_nothing() {
    let env = Env::new().await;
    env.serve_course().await;
    let diq = course(DIQ, "DATA AND INFORMATION QUALITY");
    let jobs = [CourseJob {
        course: &diq,
        folder: "DATA AND INFORMATION QUALITY",
    }];

    let first = env.context().sync_all(&jobs, |_, _| {}).await.unwrap();

    let report = &first[0].report;
    assert_eq!(first[0].error, None);
    assert!(report.errors.is_empty(), "{:?}", report.errors);
    // The real course keeps all its material in a `page` (the v1 bug): 21 files.
    assert_eq!(report.downloaded.len(), 21);
    let course_dir = env.dir.path().join("DATA AND INFORMATION QUALITY");
    let first_file = safe_join(&course_dir, &report.downloaded[0]).unwrap();
    assert_eq!(std::fs::read(first_file).unwrap(), b"data");
    assert_eq!(env.store.lock().unwrap().baseline(DIQ).unwrap().len(), 21);
    // Uploaded September–November 2026: 1st semester activity.
    let activity = env.store.lock().unwrap().course_activity().unwrap()[&DIQ];
    assert!(activity.first > 0 && activity.second == 0, "{activity:?}");

    // External links are kept for the "Link" page; on the first sync none is new.
    let links = env.store.lock().unwrap().links().unwrap();
    assert!(!links.is_empty() && links.len() == first[0].external_links.len());
    assert!(first[0].new_links.is_empty());

    let second = env.context().sync_all(&jobs, |_, _| {}).await.unwrap();
    assert_eq!(second[0].report, CourseReport::default());
    assert!(second[0].new_links.is_empty());
}

#[tokio::test]
async fn links_added_since_the_previous_sync_are_reported_as_new() {
    let env = Env::new().await;
    env.serve_course().await;
    // The course was synced before, when it had no links.
    env.store.lock().unwrap().save_links(DIQ, &[], 1).unwrap();
    let diq = course(DIQ, "DATA AND INFORMATION QUALITY");
    let jobs = [CourseJob {
        course: &diq,
        folder: "DATA AND INFORMATION QUALITY",
    }];

    let summaries = env.context().sync_all(&jobs, |_, _| {}).await.unwrap();

    assert!(!summaries[0].new_links.is_empty());
    assert_eq!(summaries[0].new_links, summaries[0].external_links);
    let links = env.store.lock().unwrap().links().unwrap();
    assert!(
        links
            .iter()
            .all(|l| !l.seen && l.first_seen == 1_794_000_000)
    );
}

#[tokio::test]
async fn a_broken_course_does_not_stop_the_others() {
    let env = Env::new().await;
    env.serve_course().await;
    env.rest("core_course_get_contents", &format!("courseid={BROKEN}"))
        .respond_with(exception("nopermissions"))
        .mount(&env.server)
        .await;
    let broken = course(BROKEN, "ROTTO");
    let diq = course(DIQ, "DIQ");
    let jobs = [
        CourseJob {
            course: &broken,
            folder: "ROTTO",
        },
        CourseJob {
            course: &diq,
            folder: "DIQ",
        },
    ];
    let mut seen = Vec::new();

    let summaries = env
        .context()
        .sync_all(&jobs, |i, c| seen.push((i, c.id)))
        .await
        .unwrap();

    assert_eq!(seen, [(0, BROKEN), (1, DIQ)]);
    assert!(
        summaries[0]
            .error
            .as_deref()
            .unwrap()
            .contains("nopermissions")
    );
    assert_eq!(summaries[1].report.downloaded.len(), 21);
}

#[tokio::test]
async fn an_invalid_token_stops_the_sync() {
    let env = Env::new().await;
    Mock::given(method("POST"))
        .respond_with(exception("invalidtoken"))
        .mount(&env.server)
        .await;
    let diq = course(DIQ, "DIQ");
    let jobs = [CourseJob {
        course: &diq,
        folder: "DIQ",
    }];

    let result = env.context().sync_all(&jobs, |_, _| {}).await;

    assert!(matches!(
        result,
        Err(SyncError::Moodle(MoodleError::InvalidToken))
    ));
}

#[tokio::test]
async fn an_unsafe_folder_is_refused() {
    let env = Env::new().await;
    env.serve_course().await;
    let diq = course(DIQ, "DIQ");
    let jobs = [CourseJob {
        course: &diq,
        folder: "..",
    }];

    let summaries = env.context().sync_all(&jobs, |_, _| {}).await.unwrap();

    assert!(summaries[0].error.is_some());
    assert!(summaries[0].report.downloaded.is_empty());
}
