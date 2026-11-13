//! Token persistence in the OS credential store (Windows Credential Manager, macOS Keychain,
//! Linux Secret Service). The token never touches disk in plain text and is never logged.

use keyring::Entry;
use thiserror::Error;

use super::token::WsToken;

const KEYRING_SERVICE: &str = "it.webeepsync.app";
const KEYRING_ACCOUNT: &str = "webeep-ws-token";

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("impossibile accedere al portachiavi di sistema: {0}")]
    Keyring(String),
}

impl From<keyring::Error> for StoreError {
    fn from(err: keyring::Error) -> Self {
        Self::Keyring(err.to_string())
    }
}

fn entry() -> Result<Entry, StoreError> {
    Ok(Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)?)
}

pub fn save(token: &WsToken) -> Result<(), StoreError> {
    Ok(entry()?.set_password(token.expose())?)
}

/// Returns `Ok(None)` when no token has been saved yet.
pub fn load() -> Result<Option<WsToken>, StoreError> {
    match entry()?.get_password() {
        Ok(value) => Ok(Some(WsToken::new(value))),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(err) => Err(err.into()),
    }
}

/// Deletes the saved token. Succeeds if there was nothing to delete.
pub fn clear() -> Result<(), StoreError> {
    match entry()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(err) => Err(err.into()),
    }
}
