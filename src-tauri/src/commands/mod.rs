//! Thin Tauri commands: validate input, delegate to services, map errors to UI messages.

pub mod courses;
pub mod dump;
pub mod links;
pub mod notify;
pub mod session;
pub mod settings;
pub mod shared;
pub mod study_plan;
pub mod sync;
pub mod teaching;

use serde::Serialize;

/// Error returned to the frontend: a user-facing Italian message, never containing secrets.
#[derive(Debug, Serialize)]
pub struct CommandError {
    pub message: String,
}

impl CommandError {
    /// `message` is written in Italian and shown in the language of the interface.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: crate::i18n::localize(&message.into()),
        }
    }
}

impl<E: std::error::Error> From<E> for CommandError {
    fn from(err: E) -> Self {
        Self::new(err.to_string())
    }
}

pub type CommandResult<T> = Result<T, CommandError>;
