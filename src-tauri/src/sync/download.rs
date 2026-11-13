//! Atomic download of one WeBeep file: write `<name>.webeep-part`, set the modification time
//! to WeBeep's `timemodified`, check the destination did not change meanwhile, then rename
//! over it.
//!
//! The token is appended as `token=` only when the request is built, only for the site's
//! `pluginfile.php` URLs, and never appears in errors or logs.

use std::path::{Path, PathBuf};
use std::time::Duration;

use thiserror::Error;
use tokio::io::AsyncWriteExt;
use url::Url;

use crate::auth::token::WsToken;
use crate::moodle::client::user_agent;
use crate::sync::local::{local_meta, meta_of, system_time};
use crate::sync::plan::LocalMeta;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
/// No total timeout (files can be hundreds of MB), but a stalled transfer fails.
const READ_TIMEOUT: Duration = Duration::from_secs(60);
/// Distinctive, so it never clobbers a user file named `<name>.part`.
const PART_SUFFIX: &str = ".webeep-part";
const TOKEN_PARAM: &str = "token";
/// The only endpoint that receives the token.
const DOWNLOAD_PATH: &str = "/webservice/pluginfile.php/";
const TOO_MANY_REQUESTS: u16 = 429;

#[derive(Debug, Clone, Copy)]
pub struct RetryPolicy {
    /// Total attempts, the first included.
    pub attempts: u32,
    /// Delay before the 2nd attempt; doubles each time.
    pub base_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            attempts: 3,
            base_delay: Duration::from_secs(2),
        }
    }
}

/// What the destination must look like right before it is replaced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Precondition {
    /// Nothing there (new file, conflict copy).
    Absent,
    /// Missing, or still exactly as we last wrote it.
    Unchanged(LocalMeta),
}

impl Precondition {
    fn holds(self, current: Option<LocalMeta>) -> bool {
        match (self, current) {
            (_, None) => true,
            (Self::Absent, Some(_)) => false,
            (Self::Unchanged(expected), Some(meta)) => meta == expected,
        }
    }
}

/// One file to download.
#[derive(Debug, Clone)]
pub struct Target {
    pub url: String,
    pub dest: PathBuf,
    pub timemodified: i64,
    /// Expected size in bytes; 0 when unknown (files linked from HTML).
    pub size: u64,
    pub precondition: Precondition,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DownloadError {
    #[error("WeBeep ha risposto con lo stato HTTP {0}")]
    Status(u16),
    #[error("errore di rete durante il download: {0}")]
    Network(String),
    #[error("impossibile scrivere «{path}»: {message}")]
    Io { path: String, message: String },
    #[error("link non scaricabile: non punta a un file di WeBeep")]
    ForeignUrl,
    #[error("WeBeep ha inviato {received} byte invece di {expected}: file non salvato")]
    SizeMismatch { expected: u64, received: u64 },
    #[error("il file è stato modificato durante la sincronizzazione: non è stato toccato")]
    ChangedMeanwhile,
}

impl DownloadError {
    fn is_transient(&self) -> bool {
        match self {
            Self::Network(_) => true,
            Self::Status(code) => *code == TOO_MANY_REQUESTS || *code >= 500,
            Self::Io { .. } | Self::ForeignUrl | Self::SizeMismatch { .. } => false,
            Self::ChangedMeanwhile => false,
        }
    }

    fn io(path: &Path, err: std::io::Error) -> Self {
        Self::Io {
            path: path.display().to_string(),
            message: err.to_string(),
        }
    }
}

impl From<reqwest::Error> for DownloadError {
    fn from(err: reqwest::Error) -> Self {
        Self::Network(err.without_url().to_string())
    }
}

/// Downloads files of one site with one token. Cheap to clone.
#[derive(Clone)]
pub struct Fetcher {
    http: reqwest::Client,
    site: Url,
    token: WsToken,
    retry: RetryPolicy,
}

impl Fetcher {
    pub fn new(site: Url, token: WsToken, retry: RetryPolicy) -> Result<Self, DownloadError> {
        let http = reqwest::Client::builder()
            .user_agent(user_agent())
            .connect_timeout(CONNECT_TIMEOUT)
            .read_timeout(READ_TIMEOUT)
            // A redirect could carry the token-bearing request elsewhere.
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        Ok(Self {
            http,
            site,
            token,
            retry,
        })
    }

    /// Downloads `target`, retrying transient failures. On error the destination is untouched.
    pub async fn fetch(&self, target: &Target) -> Result<LocalMeta, DownloadError> {
        let request_url = self.authorized(&target.url)?;
        let mut attempt = 0;
        loop {
            match self.fetch_once(&request_url, target).await {
                Err(err) if err.is_transient() && attempt + 1 < self.retry.attempts => {
                    tracing::debug!("download retry {}: {err}", attempt + 1);
                    tokio::time::sleep(self.retry.base_delay * 2_u32.pow(attempt)).await;
                    attempt += 1;
                }
                result => return result,
            }
        }
    }

    /// Adds the token, refusing anything but the site's download endpoint.
    fn authorized(&self, url: &str) -> Result<Url, DownloadError> {
        let mut parsed = Url::parse(url).map_err(|_| DownloadError::ForeignUrl)?;
        if parsed.origin() != self.site.origin() || !parsed.path().starts_with(DOWNLOAD_PATH) {
            return Err(DownloadError::ForeignUrl);
        }
        let kept: Vec<(String, String)> = parsed
            .query_pairs()
            .filter(|(name, _)| name != TOKEN_PARAM)
            .map(|(name, value)| (name.into_owned(), value.into_owned()))
            .collect();
        parsed
            .query_pairs_mut()
            .clear()
            .extend_pairs(kept)
            .append_pair(TOKEN_PARAM, self.token.expose());
        Ok(parsed)
    }

    async fn fetch_once(&self, url: &Url, target: &Target) -> Result<LocalMeta, DownloadError> {
        let dest = target.dest.as_path();
        let part = part_path(dest);
        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| DownloadError::io(parent, e))?;
        }
        let written = match self.write_part(url, &part, target).await {
            Ok(()) => replace(&part, target).await,
            Err(err) => Err(err),
        };
        if let Err(err) = written {
            // Best effort: a leftover part file is harmless and overwritten next time.
            let _ = tokio::fs::remove_file(&part).await;
            return Err(err);
        }
        let metadata = tokio::fs::metadata(dest)
            .await
            .map_err(|e| DownloadError::io(dest, e))?;
        Ok(meta_of(&metadata))
    }

    async fn write_part(
        &self,
        url: &Url,
        part: &Path,
        target: &Target,
    ) -> Result<(), DownloadError> {
        let mut response = self.http.get(url.clone()).send().await?;
        let status = response.status();
        if !status.is_success() {
            return Err(DownloadError::Status(status.as_u16()));
        }
        let io_err = |e| DownloadError::io(part, e);
        let mut file = tokio::fs::File::create(part).await.map_err(io_err)?;
        let mut received: u64 = 0;
        while let Some(chunk) = response.chunk().await? {
            file.write_all(&chunk).await.map_err(io_err)?;
            received += chunk.len() as u64;
        }
        // An error page served with 200 must not replace a good file.
        if target.size > 0 && received != target.size {
            return Err(DownloadError::SizeMismatch {
                expected: target.size,
                received,
            });
        }
        file.flush().await.map_err(io_err)?;
        let file = file.into_std().await;
        if let Some(modified) = system_time(target.timemodified) {
            file.set_modified(modified).map_err(io_err)?;
        }
        file.sync_all().map_err(io_err)
    }
}

/// Renames the part file over the destination, unless the user changed it meanwhile.
async fn replace(part: &Path, target: &Target) -> Result<(), DownloadError> {
    let dest = target.dest.as_path();
    if !target.precondition.holds(local_meta(dest)) {
        return Err(DownloadError::ChangedMeanwhile);
    }
    // A destination open in another program (Windows) makes the rename fail.
    tokio::fs::rename(part, dest)
        .await
        .map_err(|e| DownloadError::io(dest, e))
}

fn part_path(dest: &Path) -> PathBuf {
    let mut name = dest.as_os_str().to_owned();
    name.push(PART_SUFFIX);
    PathBuf::from(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const SECRET: &str = "s3cr3tt0k3n";
    const FILE_PATH: &str = "/webservice/pluginfile.php/1/mod_folder/content/0/a.pdf";
    const TIMEMODIFIED: i64 = 1_700_000_000;

    fn fast_retry() -> RetryPolicy {
        RetryPolicy {
            attempts: 3,
            base_delay: Duration::from_millis(1),
        }
    }

    fn fetcher(server: &MockServer) -> Fetcher {
        let site = Url::parse(&server.uri()).unwrap();
        Fetcher::new(site, WsToken::new(SECRET), fast_retry()).unwrap()
    }

    fn target(server: &MockServer, dest: &Path) -> Target {
        Target {
            url: format!("{}{FILE_PATH}?forcedownload=1", server.uri()),
            dest: dest.to_path_buf(),
            timemodified: 0,
            size: 0,
            precondition: Precondition::Absent,
        }
    }

    async fn serve(server: &MockServer, body: &[u8]) {
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(body.to_vec()))
            .mount(server)
            .await;
    }

    #[tokio::test]
    async fn writes_file_with_remote_mtime_and_sends_token() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(FILE_PATH))
            .and(query_param("token", SECRET))
            .and(query_param("forcedownload", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(b"%PDF-data".to_vec()))
            .expect(1)
            .mount(&server)
            .await;
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("Corso").join("Materiali").join("a.pdf");
        let file = Target {
            timemodified: TIMEMODIFIED,
            size: 9,
            ..target(&server, &dest)
        };

        let meta = fetcher(&server).fetch(&file).await.unwrap();

        assert_eq!(std::fs::read(&dest).unwrap(), b"%PDF-data");
        assert_eq!(
            meta,
            LocalMeta {
                size: 9,
                mtime: TIMEMODIFIED
            }
        );
        assert_eq!(local_meta(&dest), Some(meta));
        assert!(!part_path(&dest).exists());
    }

    #[tokio::test]
    async fn replaces_a_file_still_as_we_wrote_it() {
        let server = MockServer::start().await;
        serve(&server, b"new").await;
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("a.pdf");
        std::fs::write(&dest, b"old content").unwrap();
        let file = Target {
            precondition: Precondition::Unchanged(local_meta(&dest).unwrap()),
            ..target(&server, &dest)
        };

        fetcher(&server).fetch(&file).await.unwrap();

        assert_eq!(std::fs::read(&dest).unwrap(), b"new");
    }

    #[tokio::test]
    async fn never_replaces_a_file_changed_during_the_sync() {
        let server = MockServer::start().await;
        serve(&server, b"new").await;
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("a.pdf");
        std::fs::write(&dest, b"old").unwrap();
        let planned = local_meta(&dest).unwrap();
        // The user annotates the file after the plan was made.
        std::fs::write(&dest, b"old + annotations").unwrap();
        let file = Target {
            precondition: Precondition::Unchanged(planned),
            ..target(&server, &dest)
        };

        let err = fetcher(&server).fetch(&file).await.unwrap_err();

        assert_eq!(err, DownloadError::ChangedMeanwhile);
        assert_eq!(std::fs::read(&dest).unwrap(), b"old + annotations");
        assert!(!part_path(&dest).exists());
    }

    #[tokio::test]
    async fn never_replaces_a_file_that_appeared_meanwhile() {
        let server = MockServer::start().await;
        serve(&server, b"new").await;
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("a.pdf");
        std::fs::write(&dest, b"user file").unwrap();

        let err = fetcher(&server)
            .fetch(&target(&server, &dest))
            .await
            .unwrap_err();

        assert_eq!(err, DownloadError::ChangedMeanwhile);
        assert_eq!(std::fs::read(&dest).unwrap(), b"user file");
    }

    #[tokio::test]
    async fn rejects_a_body_of_the_wrong_size() {
        let server = MockServer::start().await;
        serve(&server, b"<html>errore</html>").await;
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("a.pdf");
        let file = Target {
            size: 1000,
            ..target(&server, &dest)
        };

        let err = fetcher(&server).fetch(&file).await.unwrap_err();

        assert_eq!(
            err,
            DownloadError::SizeMismatch {
                expected: 1000,
                received: 19
            }
        );
        assert!(!dest.exists());
        assert!(!part_path(&dest).exists());
    }

    #[tokio::test]
    async fn http_error_leaves_destination_untouched() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(404))
            .expect(1)
            .mount(&server)
            .await;
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("a.pdf");
        std::fs::write(&dest, b"annotated").unwrap();
        let file = Target {
            precondition: Precondition::Unchanged(local_meta(&dest).unwrap()),
            ..target(&server, &dest)
        };

        let err = fetcher(&server).fetch(&file).await.unwrap_err();

        assert_eq!(err, DownloadError::Status(404));
        assert_eq!(std::fs::read(&dest).unwrap(), b"annotated");
        assert!(!part_path(&dest).exists());
    }

    #[tokio::test]
    async fn retries_transient_server_errors() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(503))
            .up_to_n_times(2)
            .with_priority(1)
            .mount(&server)
            .await;
        serve(&server, b"ok").await;
        let dir = tempfile::tempdir().unwrap();

        let meta = fetcher(&server)
            .fetch(&target(&server, &dir.path().join("a.pdf")))
            .await
            .unwrap();

        assert_eq!(meta.size, 2);
    }

    #[tokio::test]
    async fn gives_up_after_the_last_attempt() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(500))
            .expect(3)
            .mount(&server)
            .await;
        let dir = tempfile::tempdir().unwrap();

        let err = fetcher(&server)
            .fetch(&target(&server, &dir.path().join("a.pdf")))
            .await
            .unwrap_err();

        assert_eq!(err, DownloadError::Status(500));
    }

    #[tokio::test]
    async fn never_sends_the_token_to_another_origin() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200))
            .expect(0)
            .mount(&server)
            .await;
        let site = Url::parse("https://webeep.polimi.it/").unwrap();
        let fetcher = Fetcher::new(site, WsToken::new(SECRET), fast_retry()).unwrap();
        let dir = tempfile::tempdir().unwrap();

        let err = fetcher
            .fetch(&target(&server, &dir.path().join("a.pdf")))
            .await
            .unwrap_err();

        assert_eq!(err, DownloadError::ForeignUrl);
    }

    #[test]
    fn token_goes_only_to_the_download_endpoint() {
        let site = Url::parse("https://webeep.polimi.it/").unwrap();
        let fetcher = Fetcher::new(site, WsToken::new(SECRET), fast_retry()).unwrap();
        let url = fetcher
            .authorized("https://webeep.polimi.it/webservice/pluginfile.php/1/a.pdf?token=old&x=1")
            .unwrap();
        assert_eq!(url.query(), Some(format!("x=1&token={SECRET}").as_str()));
        for other in [
            "https://webeep.polimi.it/course/view.php?id=1",
            "https://webeep.polimi.it/pluginfile.php/1/a.pdf",
            "http://webeep.polimi.it/webservice/pluginfile.php/1/a.pdf",
        ] {
            assert_eq!(
                fetcher.authorized(other),
                Err(DownloadError::ForeignUrl),
                "{other}"
            );
        }
    }

    #[test]
    fn preconditions() {
        let meta = LocalMeta { size: 1, mtime: 2 };
        let other = LocalMeta { size: 1, mtime: 3 };
        assert!(Precondition::Absent.holds(None));
        assert!(!Precondition::Absent.holds(Some(meta)));
        assert!(Precondition::Unchanged(meta).holds(Some(meta)));
        assert!(Precondition::Unchanged(meta).holds(None));
        assert!(!Precondition::Unchanged(meta).holds(Some(other)));
    }

    #[tokio::test]
    async fn network_errors_do_not_leak_the_token() {
        // Nothing listens on a port that was bound and released.
        let port = std::net::TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let site = Url::parse(&format!("http://127.0.0.1:{port}/")).unwrap();
        // Windows retries refused connections for ~2 s: one attempt keeps the test fast.
        let once = RetryPolicy {
            attempts: 1,
            ..fast_retry()
        };
        let fetcher = Fetcher::new(site, WsToken::new(SECRET), once).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let file = Target {
            url: format!("http://127.0.0.1:{port}{FILE_PATH}"),
            dest: dir.path().join("a.pdf"),
            timemodified: 0,
            size: 0,
            precondition: Precondition::Absent,
        };

        let err = fetcher.fetch(&file).await.unwrap_err();

        assert!(matches!(err, DownloadError::Network(_)), "{err:?}");
        assert!(!err.to_string().contains(SECRET));
        assert!(!format!("{err:?}").contains(SECRET));
    }
}
