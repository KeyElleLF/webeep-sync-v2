//! Phase 0 dump: saves raw JSON responses from WeBeep so the real data shapes can be studied
//! and turned into redacted test fixtures. Raw dumps contain personal data and are gitignored.
//!
//! Every step is isolated: a failing call is recorded in the report and the dump goes on.

use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::Value;

use crate::moodle::client::{MoodleClient, Param, array_param, param};
use crate::moodle::error::MoodleError;

/// Course code used to locate the reference course with nested folders (§5).
pub const REFERENCE_COURSE_CODE: &str = "056892";
/// Keys removed from dumped responses because they are credentials.
const CREDENTIAL_KEYS: &[&str] = &["userprivateaccesskey", "token", "privatetoken"];
const TIMELINE_CLASSIFICATIONS: &[&str] = &["inprogress", "past", "future", "all"];
const MAX_DUMPED_FORUMS: usize = 5;
/// Deadlines from 30 days ago to 60 days ahead.
const DEADLINE_DUMP_PAST: i64 = 30 * 86_400;
const DEADLINE_DUMP_FUTURE: i64 = 60 * 86_400;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DumpReport {
    pub directory: String,
    pub files: Vec<String>,
    pub errors: Vec<String>,
    pub moodle_release: Option<String>,
    pub reference_course_ids: Vec<u64>,
}

struct Dumper<'a> {
    client: &'a MoodleClient,
    dir: PathBuf,
    files: Vec<String>,
    errors: Vec<String>,
}

impl Dumper<'_> {
    /// Calls `function`, saves the response as `<name>.json` and returns it.
    async fn save(&mut self, name: &str, function: &str, params: &[Param]) -> Option<Value> {
        match self.client.call_raw(function, params).await {
            Ok(value) => {
                self.write(name, &value).await;
                Some(value)
            }
            Err(err) => {
                self.errors.push(format!("{function} ({name}): {err}"));
                None
            }
        }
    }

    async fn write(&mut self, name: &str, value: &Value) {
        let file = format!("{name}.json");
        let written = match serde_json::to_vec_pretty(value) {
            Ok(bytes) => tokio::fs::write(self.dir.join(&file), bytes).await,
            Err(err) => Err(std::io::Error::other(err)),
        };
        match written {
            Ok(()) => self.files.push(file),
            Err(err) => self.errors.push(format!("scrittura di {file}: {err}")),
        }
    }
}

/// Runs the Phase 0 dump into a new timestamped folder under `base_dir`.
pub async fn run(client: &MoodleClient, base_dir: &Path) -> Result<DumpReport, MoodleError> {
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let dir = base_dir.join(stamp);
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| MoodleError::Config(format!("cartella di dump non creabile: {e}")))?;
    let mut dumper = Dumper {
        client,
        dir,
        files: Vec::new(),
        errors: Vec::new(),
    };

    // Without site info we have no user id: treat it as fatal (usually an expired token).
    let site_info = client
        .call_raw("core_webservice_get_site_info", &[])
        .await?;
    dumper
        .write("site_info", &without_credentials(&site_info))
        .await;
    let release = site_info["release"].as_str().map(str::to_owned);

    let lists = dump_course_lists(&mut dumper, &site_info).await;
    let reference_ids = lists
        .all
        .as_ref()
        .map(|all| find_courses_by_code(all, REFERENCE_COURSE_CODE))
        .unwrap_or_default();
    dump_course_details(&mut dumper, &lists).await;

    Ok(DumpReport {
        directory: dumper.dir.display().to_string(),
        files: dumper.files,
        errors: dumper.errors,
        moodle_release: release,
        reference_course_ids: reference_ids,
    })
}

/// Timeline responses needed by the later dump steps.
struct CourseLists {
    all: Option<Value>,
    inprogress: Option<Value>,
}

/// Dumps every course list and returns the `all` and `inprogress` timelines.
async fn dump_course_lists(dumper: &mut Dumper<'_>, site_info: &Value) -> CourseLists {
    if let Some(userid) = site_info["userid"].as_u64() {
        let params = [param("userid", userid)];
        dumper
            .save(
                "enrol_users_courses",
                "core_enrol_get_users_courses",
                &params,
            )
            .await;
    }
    let mut lists = CourseLists {
        all: None,
        inprogress: None,
    };
    for classification in TIMELINE_CLASSIFICATIONS {
        let params = [
            param("classification", classification),
            param("offset", 0),
            param("limit", 0),
            param("sort", "fullname"),
        ];
        let name = format!("timeline_{classification}");
        let function = "core_course_get_enrolled_courses_by_timeline_classification";
        let response = dumper.save(&name, function, &params).await;
        match *classification {
            "all" => lists.all = response,
            "inprogress" => lists.inprogress = response,
            _ => {}
        }
    }
    lists
}

/// Dumps course details for every course, plus contents, assignments and forums of the
/// in-progress ones. Calls are sequential on purpose, to keep the load on WeBeep low.
async fn dump_course_details(dumper: &mut Dumper<'_>, lists: &CourseLists) {
    let all_ids = lists
        .all
        .as_ref()
        .map(timeline_course_ids)
        .unwrap_or_default();
    if !all_ids.is_empty() {
        let ids = all_ids
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let params = [param("field", "ids"), param("value", ids)];
        dumper
            .save(
                "courses_by_field",
                "core_course_get_courses_by_field",
                &params,
            )
            .await;
    }
    let active_ids = lists
        .inprogress
        .as_ref()
        .map(timeline_course_ids)
        .unwrap_or_default();
    if active_ids.is_empty() {
        return;
    }
    for id in &active_ids {
        let params = [param("courseid", id)];
        dumper
            .save(
                &format!("contents_{id}"),
                "core_course_get_contents",
                &params,
            )
            .await;
    }
    let course_ids = array_param("courseids", &active_ids);
    dumper
        .save(
            "assignments_inprogress",
            "mod_assign_get_assignments",
            &course_ids,
        )
        .await;
    let forums = dumper
        .save(
            "forums_inprogress",
            "mod_forum_get_forums_by_courses",
            &course_ids,
        )
        .await;
    dump_announcements(dumper, forums.as_ref()).await;
    dump_deadlines(dumper).await;
}

/// Discussions of the first few announcement forums that have any (Phase 3).
async fn dump_announcements(dumper: &mut Dumper<'_>, forums: Option<&Value>) {
    let ids: Vec<u64> = forums
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|f| f["type"] == "news" && f["numdiscussions"].as_u64().unwrap_or(0) > 0)
        .filter_map(|f| f["id"].as_u64())
        .take(MAX_DUMPED_FORUMS)
        .collect();
    for id in ids {
        let params = [param("forumid", id), param("page", 0), param("perpage", 10)];
        dumper
            .save(
                &format!("discussions_{id}"),
                "mod_forum_get_forum_discussions",
                &params,
            )
            .await;
    }
}

/// Deadlines of the next weeks, as shown in the dashboard timeline block.
async fn dump_deadlines(dumper: &mut Dumper<'_>) {
    let now = chrono::Utc::now().timestamp();
    let params = [
        param("timesortfrom", now - DEADLINE_DUMP_PAST),
        param("timesortto", now + DEADLINE_DUMP_FUTURE),
        param("limitnum", 50),
    ];
    dumper
        .save(
            "action_events",
            "core_calendar_get_action_events_by_timesort",
            &params,
        )
        .await;
}

/// `site_info` carries `userprivateaccesskey`, an auto-login credential: never write it.
pub fn without_credentials(site_info: &Value) -> Value {
    match site_info {
        Value::Object(map) => Value::Object(
            map.iter()
                .filter(|(key, _)| !CREDENTIAL_KEYS.contains(&key.as_str()))
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
        ),
        other => other.clone(),
    }
}

/// Course ids from a `..._by_timeline_classification` response (`{"courses": [...]}`).
pub fn timeline_course_ids(response: &Value) -> Vec<u64> {
    courses(response).filter_map(|c| c["id"].as_u64()).collect()
}

/// Ids of the courses whose `fullname` starts with `<code> - `.
pub fn find_courses_by_code(response: &Value, code: &str) -> Vec<u64> {
    let prefix = format!("{code} - ");
    courses(response)
        .filter(|c| {
            c["fullname"]
                .as_str()
                .is_some_and(|n| n.starts_with(&prefix))
        })
        .filter_map(|c| c["id"].as_u64())
        .collect()
}

fn courses(response: &Value) -> impl Iterator<Item = &Value> {
    response["courses"].as_array().into_iter().flatten()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn timeline() -> Value {
        json!({
            "courses": [
                {"id": 1, "fullname": "056892 - DATA MINING (DOCENTE A) [2026-27]"},
                {"id": 2, "fullname": "051589 - MODEL IDENTIFICATION (DOCENTE B) [2026-27]"},
                {"id": 3, "fullname": "056892 - DATA MINING (DOCENTE C) [2026-27]"},
                {"id": 4, "fullname": "1056892 - OTHER (DOCENTE D) [2026-27]"}
            ],
            "nextoffset": 0
        })
    }

    #[test]
    fn strips_private_access_key_from_site_info() {
        let info = json!({"userid": 1, "userprivateaccesskey": "secret", "release": "4.5"});
        let clean = without_credentials(&info);
        assert_eq!(clean, json!({"userid": 1, "release": "4.5"}));
    }

    #[test]
    fn extracts_all_course_ids() {
        assert_eq!(timeline_course_ids(&timeline()), vec![1, 2, 3, 4]);
    }

    #[test]
    fn finds_every_course_with_the_code_prefix() {
        assert_eq!(find_courses_by_code(&timeline(), "056892"), vec![1, 3]);
    }

    #[test]
    fn handles_unexpected_shapes() {
        assert!(timeline_course_ids(&json!([])).is_empty());
        assert!(find_courses_by_code(&json!({"courses": null}), "056892").is_empty());
    }
}
