//! Login window: opens `launch.php` in a dedicated, private webview, lets the user complete
//! the Polimi SSO and intercepts the `moodlemobile://token=...` redirect.
//!
//! The window loads a remote URL and has no capability attached, so it cannot call any
//! Tauri command (§12).

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent};
use thiserror::Error;
use tokio::sync::oneshot;
use tracing::{debug, warn};
use url::Url;

use super::token::{TokenError, WsToken, parse_launch_redirect};
use super::{SIGNATURE_ROOTS, URL_SCHEME, launch_url, new_passport};

const LOGIN_LABEL: &str = "login";
const LOGIN_TITLE: &str = "Accedi a WeBeep";
const WINDOW_SIZE: (f64, f64) = (1000.0, 720.0);
const SITE_HOST: &str = "webeep.polimi.it";
/// Pages reached after a successful login when Shibboleth ignores the return URL.
const POST_LOGIN_PATHS: &[&str] = &["/my", "/my/", "/my/index.php", "/my/courses.php"];
/// Guards against redirect loops between `/my/` and `launch.php`.
const MAX_LAUNCH_REDIRECTS: u32 = 3;
/// Exact prefix of the redirect issued by `launch.php`.
const TOKEN_PREFIX: &str = "moodlemobile://token=";
const LAUNCH_PATH: &str = "/admin/tool/mobile/launch.php";
const BLANK_PAGE: &str = "about:blank";

#[derive(Debug, Error)]
pub enum LoginError {
    #[error("accesso annullato: la finestra di login è stata chiusa")]
    Cancelled,
    #[error("una finestra di login è già aperta")]
    AlreadyOpen,
    #[error(transparent)]
    Token(#[from] TokenError),
    #[error("impossibile aprire la finestra di login: {0}")]
    Window(String),
}

/// What to do with a navigation inside the login webview.
#[derive(Debug, PartialEq, Eq)]
pub enum NavDecision {
    Allow,
    /// Not https, or a token redirect not coming from `launch.php`: refuse it.
    Block,
    /// The `launch.php` redirect carrying the token was reached.
    Token,
    /// Login completed on the dashboard: go to `launch.php` to obtain the token.
    RedirectToLaunch,
}

/// Decides how to handle `url`, given the last page the webview was allowed to load.
pub fn decide(url: &Url, previous: Option<&Url>) -> NavDecision {
    if url.scheme() == URL_SCHEME {
        let has_token_prefix = url.as_str().starts_with(TOKEN_PREFIX);
        return if has_token_prefix && previous.is_some_and(is_launch_page) {
            NavDecision::Token
        } else {
            NavDecision::Block
        };
    }
    if url.as_str() == BLANK_PAGE {
        return NavDecision::Allow;
    }
    if url.scheme() != "https" {
        return NavDecision::Block;
    }
    if is_site(url) && POST_LOGIN_PATHS.contains(&url.path()) {
        return NavDecision::RedirectToLaunch;
    }
    NavDecision::Allow
}

fn is_site(url: &Url) -> bool {
    url.scheme() == "https" && url.host_str() == Some(SITE_HOST)
}

fn is_launch_page(url: &Url) -> bool {
    is_site(url) && url.path() == LAUNCH_PATH
}

enum Outcome {
    Redirect(String),
    Closed,
}

type OutcomeSlot = Arc<Mutex<Option<oneshot::Sender<Outcome>>>>;

fn deliver(slot: &OutcomeSlot, outcome: Outcome) {
    let sender = match slot.lock() {
        Ok(mut guard) => guard.take(),
        Err(poisoned) => poisoned.into_inner().take(),
    };
    if let Some(sender) = sender {
        // The receiver only disappears if `login` was dropped; nothing left to notify.
        let _ = sender.send(outcome);
    }
}

/// Runs the interactive login and returns a verified token.
pub async fn login(app: &AppHandle) -> Result<WsToken, LoginError> {
    if let Some(existing) = app.get_webview_window(LOGIN_LABEL) {
        if let Err(err) = existing.set_focus() {
            warn!("cannot focus login window: {err}");
        }
        return Err(LoginError::AlreadyOpen);
    }
    let passport = new_passport();
    let launch = launch_url(&passport).map_err(|e| LoginError::Window(e.to_string()))?;
    let (tx, rx) = oneshot::channel();
    let slot: OutcomeSlot = Arc::new(Mutex::new(Some(tx)));

    let window = build_window(app, launch, slot.clone())?;
    window.on_window_event(move |event| {
        if matches!(event, WindowEvent::Destroyed) {
            deliver(&slot, Outcome::Closed);
        }
    });

    match rx.await.unwrap_or(Outcome::Closed) {
        Outcome::Closed => Err(LoginError::Cancelled),
        Outcome::Redirect(raw) => {
            if let Err(err) = window.close() {
                warn!("cannot close login window: {err}");
            }
            Ok(parse_launch_redirect(&raw, &passport, SIGNATURE_ROOTS)?)
        }
    }
}

/// Everything the navigation callback needs, shared for the window's lifetime.
struct NavState {
    app: AppHandle,
    launch: Url,
    slot: OutcomeSlot,
    redirects: AtomicU32,
    previous: Mutex<Option<Url>>,
}

fn build_window(
    app: &AppHandle,
    launch: Url,
    slot: OutcomeSlot,
) -> Result<WebviewWindow, LoginError> {
    let state = NavState {
        app: app.clone(),
        launch: launch.clone(),
        slot,
        redirects: AtomicU32::new(0),
        previous: Mutex::new(None),
    };
    WebviewWindowBuilder::new(app, LOGIN_LABEL, WebviewUrl::External(launch))
        .title(LOGIN_TITLE)
        .inner_size(WINDOW_SIZE.0, WINDOW_SIZE.1)
        // Private session: no cookies shared with the system browser, nothing left on logout.
        .incognito(true)
        .on_navigation(move |url| handle_navigation(url, &state))
        .build()
        .map_err(|e| LoginError::Window(e.to_string()))
}

fn handle_navigation(url: &Url, state: &NavState) -> bool {
    let mut previous = match state.previous.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    match decide(url, previous.as_ref()) {
        NavDecision::Allow => {
            debug!(host = url.host_str(), path = url.path(), "login navigation");
            show_host_in_title(
                state.app.clone(),
                url.host_str().unwrap_or_default().to_owned(),
            );
            *previous = Some(url.clone());
            true
        }
        NavDecision::Block => {
            warn!(
                scheme = url.scheme(),
                host = url.host_str(),
                "login navigation blocked"
            );
            false
        }
        NavDecision::Token => {
            debug!("token redirect intercepted");
            deliver(&state.slot, Outcome::Redirect(url.as_str().to_owned()));
            false
        }
        NavDecision::RedirectToLaunch => {
            if state.redirects.fetch_add(1, Ordering::SeqCst) >= MAX_LAUNCH_REDIRECTS {
                warn!("too many redirects to launch.php, leaving the dashboard open");
                return true;
            }
            navigate_later(state.app.clone(), state.launch.clone());
            false
        }
    }
}

/// The login window has no address bar: show the current site in the title instead, so a
/// redirect to an unexpected domain is visible to the user.
fn show_host_in_title(app: AppHandle, host: String) {
    tauri::async_runtime::spawn(async move {
        let Some(window) = app.get_webview_window(LOGIN_LABEL) else {
            return;
        };
        if let Err(err) = window.set_title(&format!("{LOGIN_TITLE} — {host}")) {
            warn!("cannot update login window title: {err}");
        }
    });
}

/// Navigating from inside the navigation callback could deadlock: defer it.
fn navigate_later(app: AppHandle, target: Url) {
    tauri::async_runtime::spawn(async move {
        let Some(window) = app.get_webview_window(LOGIN_LABEL) else {
            return;
        };
        if let Err(err) = window.navigate(target) {
            warn!("cannot navigate login window to launch.php: {err}");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(s: &str) -> Url {
        Url::parse(s).unwrap()
    }

    const LAUNCH: &str = "https://webeep.polimi.it/admin/tool/mobile/launch.php?service=x";

    #[test]
    fn token_redirect_right_after_launch_page_is_accepted() {
        let launch = url(LAUNCH);
        let token = url("moodlemobile://token=abc==");
        assert_eq!(decide(&token, Some(&launch)), NavDecision::Token);
    }

    #[test]
    fn token_redirect_from_elsewhere_is_blocked() {
        let token = url("moodlemobile://token=abc==");
        let other = url("https://evil.example/admin/tool/mobile/launch.php");
        assert_eq!(decide(&token, None), NavDecision::Block);
        assert_eq!(decide(&token, Some(&other)), NavDecision::Block);
    }

    #[test]
    fn custom_scheme_without_exact_prefix_is_blocked() {
        let launch = url(LAUNCH);
        let tricky = url("moodlemobile://x?token=abc");
        assert_eq!(decide(&tricky, Some(&launch)), NavDecision::Block);
    }

    #[test]
    fn non_https_pages_are_blocked() {
        for page in [
            "http://webeep.polimi.it/my/",
            "file:///C:/x.html",
            "javascript:alert(1)",
            "data:text/html,hi",
        ] {
            assert_eq!(decide(&url(page), None), NavDecision::Block, "{page}");
        }
    }

    #[test]
    fn dashboard_triggers_launch_redirect() {
        for page in [
            "https://webeep.polimi.it/my/",
            "https://webeep.polimi.it/my/courses.php",
        ] {
            assert_eq!(
                decide(&url(page), None),
                NavDecision::RedirectToLaunch,
                "{page}"
            );
        }
    }

    #[test]
    fn sso_pages_are_allowed() {
        for page in [
            "https://aunicalogin.polimi.it/aunicalogin/aunicalogin.jsp",
            "https://webeep.polimi.it/auth/shibboleth/index.php",
            LAUNCH,
            "https://evil.example/my/",
            "about:blank",
        ] {
            assert_eq!(decide(&url(page), None), NavDecision::Allow, "{page}");
        }
    }
}
