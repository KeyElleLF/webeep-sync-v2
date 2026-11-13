//! Minimal client for Moodle's REST web service (`/webservice/rest/server.php`).

use std::fmt;
use std::time::Duration;

use serde::de::DeserializeOwned;
use serde_json::Value;
use url::Url;

use super::error::MoodleError;
use crate::auth::token::WsToken;

/// Path of the REST endpoint, relative to the site root.
const REST_PATH: &str = "webservice/rest/server.php";
/// Per-request timeout. The v1 used 10 s, which proved too tight under load.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
/// Moodle's error code for an expired or revoked token.
const INVALID_TOKEN_CODE: &str = "invalidtoken";

/// Identifies the app to WeBeep without any personal data.
pub fn user_agent() -> String {
    format!("WeBeepSyncV2/{}", env!("CARGO_PKG_VERSION"))
}

/// A form parameter sent along with a web service call.
pub type Param = (String, String);

/// Builds a scalar form parameter.
pub fn param(name: &str, value: impl ToString) -> Param {
    (name.to_owned(), value.to_string())
}

/// Builds Moodle's array encoding: `name[0]=a&name[1]=b`.
pub fn array_param<T: ToString>(name: &str, values: &[T]) -> Vec<Param> {
    values
        .iter()
        .enumerate()
        .map(|(i, v)| (format!("{name}[{i}]"), v.to_string()))
        .collect()
}

pub struct MoodleClient {
    http: reqwest::Client,
    endpoint: Url,
    token: WsToken,
}

impl fmt::Debug for MoodleClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MoodleClient")
            .field("endpoint", &self.endpoint.as_str())
            .field("token", &self.token)
            .finish()
    }
}

impl MoodleClient {
    /// `site_root` is the Moodle base URL, e.g. `https://webeep.polimi.it/`.
    pub fn new(site_root: &Url, token: WsToken) -> Result<Self, MoodleError> {
        let endpoint = site_root
            .join(REST_PATH)
            .map_err(|e| MoodleError::Config(e.to_string()))?;
        let http = reqwest::Client::builder()
            .user_agent(user_agent())
            .timeout(REQUEST_TIMEOUT)
            .connect_timeout(CONNECT_TIMEOUT)
            // A 307/308 would replay the POST body, token included, to another host.
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        Ok(Self {
            http,
            endpoint,
            token,
        })
    }

    /// Calls `function` and returns the raw JSON, after checking for Moodle exceptions.
    pub async fn call_raw(&self, function: &str, params: &[Param]) -> Result<Value, MoodleError> {
        let form = self.build_form(function, params);
        let response = self
            .http
            .post(self.endpoint.clone())
            .form(&form)
            .send()
            .await?;
        let status = response.status();
        if !status.is_success() {
            return Err(MoodleError::Status(status.as_u16()));
        }
        let body: Value = serde_json::from_slice(&response.bytes().await?)?;
        check_exception(body)
    }

    /// Calls `function` and deserializes the response into `T`.
    pub async fn call<T: DeserializeOwned>(
        &self,
        function: &str,
        params: &[Param],
    ) -> Result<T, MoodleError> {
        let value = self.call_raw(function, params).await?;
        Ok(serde_json::from_value(value)?)
    }

    fn build_form(&self, function: &str, params: &[Param]) -> Vec<Param> {
        let standard = [
            param("wstoken", self.token.expose()),
            param("wsfunction", function),
            param("moodlewsrestformat", "json"),
            param("moodlewssettingfilter", "true"),
            param("moodlewssettinglang", "it"),
        ];
        standard.into_iter().chain(params.iter().cloned()).collect()
    }
}

/// Moodle reports failures as HTTP 200 with `{"exception", "errorcode", "message"}`.
fn check_exception(body: Value) -> Result<Value, MoodleError> {
    let Some(errorcode) = body
        .as_object()
        .filter(|obj| obj.contains_key("exception"))
        .map(|obj| text_field(obj, "errorcode"))
    else {
        return Ok(body);
    };
    if errorcode == INVALID_TOKEN_CODE {
        return Err(MoodleError::InvalidToken);
    }
    let message = body
        .as_object()
        .map(|obj| text_field(obj, "message"))
        .unwrap_or_default();
    Err(MoodleError::Api { errorcode, message })
}

fn text_field(obj: &serde_json::Map<String, Value>, key: &str) -> String {
    obj.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_json::json;
    use wiremock::matchers::{body_string_contains, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const TOKEN: &str = "abcdef0123456789";

    async fn client_for(server: &MockServer) -> MoodleClient {
        let root = Url::parse(&format!("{}/", server.uri())).unwrap();
        MoodleClient::new(&root, WsToken::new(TOKEN)).unwrap()
    }

    fn rest_mock(body: Value) -> Mock {
        Mock::given(method("POST"))
            .and(path("/webservice/rest/server.php"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
    }

    #[test]
    fn array_param_uses_indexed_names() {
        let params = array_param("courseids", &[10, 20]);
        assert_eq!(
            params,
            vec![
                ("courseids[0]".to_owned(), "10".to_owned()),
                ("courseids[1]".to_owned(), "20".to_owned()),
            ]
        );
    }

    #[tokio::test]
    async fn sends_standard_form_fields_and_params() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/webservice/rest/server.php"))
            .and(body_string_contains(format!("wstoken={TOKEN}")))
            .and(body_string_contains("wsfunction=core_course_get_contents"))
            .and(body_string_contains("moodlewsrestformat=json"))
            .and(body_string_contains("courseid=42"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([{"id": 1}])))
            .expect(1)
            .mount(&server)
            .await;

        let client = client_for(&server).await;
        let value = client
            .call_raw("core_course_get_contents", &[param("courseid", 42)])
            .await
            .unwrap();
        assert_eq!(value, json!([{"id": 1}]));
    }

    #[tokio::test]
    async fn deserializes_typed_response() {
        #[derive(Deserialize)]
        struct SiteInfo {
            userid: u64,
        }
        let server = MockServer::start().await;
        rest_mock(json!({"userid": 7, "fullname": "X"}))
            .mount(&server)
            .await;

        let info: SiteInfo = client_for(&server)
            .await
            .call("core_webservice_get_site_info", &[])
            .await
            .unwrap();
        assert_eq!(info.userid, 7);
    }

    #[tokio::test]
    async fn maps_invalidtoken_exception() {
        let server = MockServer::start().await;
        rest_mock(json!({
            "exception": "moodle_exception",
            "errorcode": "invalidtoken",
            "message": "Invalid token - token not found"
        }))
        .mount(&server)
        .await;

        let result = client_for(&server).await.call_raw("any", &[]).await;
        assert!(matches!(result, Err(MoodleError::InvalidToken)));
    }

    #[tokio::test]
    async fn maps_other_exceptions_to_api_error() {
        let server = MockServer::start().await;
        rest_mock(json!({
            "exception": "required_capability_exception",
            "errorcode": "nopermissions",
            "message": "Sorry"
        }))
        .mount(&server)
        .await;

        let result = client_for(&server).await.call_raw("any", &[]).await;
        match result {
            Err(MoodleError::Api { errorcode, message }) => {
                assert_eq!(errorcode, "nopermissions");
                assert_eq!(message, "Sorry");
            }
            other => panic!("unexpected result: {other:?}"),
        }
    }

    #[tokio::test]
    async fn maps_http_failure_status() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&server)
            .await;

        let result = client_for(&server).await.call_raw("any", &[]).await;
        assert!(matches!(result, Err(MoodleError::Status(503))));
    }

    #[tokio::test]
    async fn reports_non_json_body_as_decode_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_string("<html>"))
            .mount(&server)
            .await;

        let result = client_for(&server).await.call_raw("any", &[]).await;
        assert!(matches!(result, Err(MoodleError::Decode(_))));
    }

    #[tokio::test]
    async fn debug_output_never_contains_token() {
        let server = MockServer::start().await;
        let rendered = format!("{:?}", client_for(&server).await);
        assert!(!rendered.contains(TOKEN));
    }
}
