use serde_json::{Value, json};
use url::Url;
use wiremock::matchers::{body_string_contains, method};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::*;
use crate::auth::token::WsToken;
use crate::test_support::fixture;

const COURSE: u64 = 24741;
const FORUM: u64 = 900;
const NOW: i64 = 1_800_000_000;

fn discussions_fixture() -> Value {
    serde_json::from_str(&fixture("notify/discussions.json")).unwrap()
}

fn forums(count: u64) -> Value {
    json!([
        {"id": FORUM, "course": COURSE, "type": "news", "numdiscussions": count},
        {"id": 901, "course": COURSE, "type": "general", "numdiscussions": 50}
    ])
}

async fn mock(server: &MockServer, function: &str, body: Value) {
    server.reset().await;
    Mock::given(method("POST"))
        .and(body_string_contains(format!("wsfunction={function}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(server)
        .await;
}

async fn setup() -> (MockServer, MoodleClient, Mutex<Store>) {
    let server = MockServer::start().await;
    let site = Url::parse(&format!("{}/", server.uri())).unwrap();
    let client = MoodleClient::new(&site, WsToken::new("t")).unwrap();
    (server, client, Mutex::new(Store::open_in_memory().unwrap()))
}

async fn serve(server: &MockServer, count: u64, discussions: Value) {
    server.reset().await;
    for (function, body) in [
        ("mod_forum_get_forums_by_courses", forums(count)),
        ("mod_forum_get_forum_discussions", discussions),
    ] {
        Mock::given(method("POST"))
            .and(body_string_contains(format!("wsfunction={function}")))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(server)
            .await;
    }
}

#[tokio::test]
async fn first_import_is_silent_then_new_posts_are_reported() {
    let (server, client, store) = setup().await;
    serve(&server, 3, discussions_fixture()).await;

    let first = check(&client, &store, &[COURSE], NOW).await.unwrap();
    assert!(first.is_empty(), "history is not notified");
    assert_eq!(store.lock().unwrap().announcements().unwrap().len(), 3);

    // Unchanged count: no discussions call at all.
    mock(&server, "mod_forum_get_forums_by_courses", forums(3)).await;
    assert!(
        check(&client, &store, &[COURSE], NOW)
            .await
            .unwrap()
            .is_empty()
    );

    let mut more = discussions_fixture();
    let new_post = json!({"discussion": 4242, "subject": "Nuovo", "message": "<p>Ciao</p><script>x()</script>",
        "created": NOW, "modified": NOW, "userfullname": "DOCENTE A", "pinned": false});
    more["discussions"]
        .as_array_mut()
        .unwrap()
        .insert(0, new_post);
    serve(&server, 4, more).await;

    let fresh = check(&client, &store, &[COURSE], NOW).await.unwrap();
    assert_eq!(fresh.len(), 1);
    assert_eq!(fresh[0].id, 4242);
    assert_eq!(fresh[0].message, "<p>Ciao</p>");
    assert!(!fresh[0].read);
}

#[tokio::test]
async fn nothing_to_check_without_courses() {
    let (_server, client, store) = setup().await;
    assert!(check(&client, &store, &[], NOW).await.unwrap().is_empty());
}

#[test]
fn first_import_keeps_recent_posts_unread() {
    let response: DiscussionsResponse =
        serde_json::from_str(&fixture("notify/discussions.json")).unwrap();
    let d = &response.discussions[0];
    assert!(to_announcement(COURSE, d, true, d.created + 86_400 * 30).read);
    assert!(!to_announcement(COURSE, d, true, d.created + 3600).read);
    assert!(!to_announcement(COURSE, d, false, d.created + 86_400 * 30).read);
}
