//! Session commands: status, login, logout.

use serde::Serialize;
use tauri::AppHandle;
use tracing::info;

use super::{CommandError, CommandResult};
use crate::auth::{self, store, token::WsToken, window};
use crate::moodle::client::MoodleClient;
use crate::moodle::error::MoodleError;
use crate::moodle::types::SiteInfo;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    pub logged_in: bool,
    pub fullname: Option<String>,
    pub moodle_release: Option<String>,
}

impl SessionInfo {
    fn logged_out() -> Self {
        Self {
            logged_in: false,
            fullname: None,
            moodle_release: None,
        }
    }
}

fn client_for(token: WsToken) -> Result<MoodleClient, MoodleError> {
    let root = auth::site_url().map_err(|e| MoodleError::Config(e.to_string()))?;
    MoodleClient::new(&root, token)
}

/// Loads the saved token and builds a client, or fails with `InvalidToken` if absent.
pub fn saved_client() -> Result<MoodleClient, CommandError> {
    let token = store::load()?.ok_or(MoodleError::InvalidToken)?;
    Ok(client_for(token)?)
}

/// Validates `token` against WeBeep. An expired token is removed from the keyring.
async fn session_for(token: WsToken) -> CommandResult<SessionInfo> {
    let client = client_for(token)?;
    match client
        .call::<SiteInfo>("core_webservice_get_site_info", &[])
        .await
    {
        Ok(info) => Ok(SessionInfo {
            logged_in: true,
            fullname: Some(info.fullname),
            moodle_release: info.release,
        }),
        Err(MoodleError::InvalidToken) => {
            info!("saved token rejected by WeBeep, clearing it");
            store::clear()?;
            Ok(SessionInfo::logged_out())
        }
        Err(err) => Err(err.into()),
    }
}

#[tauri::command]
pub async fn auth_status() -> CommandResult<SessionInfo> {
    match store::load()? {
        Some(token) => session_for(token).await,
        None => Ok(SessionInfo::logged_out()),
    }
}

#[tauri::command]
pub async fn login(app: AppHandle) -> CommandResult<SessionInfo> {
    let token = window::login(&app).await?;
    store::save(&token)?;
    info!("login completed");
    session_for(token).await
}

#[tauri::command]
pub async fn logout() -> CommandResult<()> {
    store::clear()?;
    info!("logged out");
    Ok(())
}
