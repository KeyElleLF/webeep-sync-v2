use thiserror::Error;

/// Errors raised while talking to the Moodle web service.
///
/// Messages are user-facing (Italian) and must never contain the token.
#[derive(Debug, Error)]
pub enum MoodleError {
    #[error("sessione scaduta: effettua di nuovo l'accesso")]
    InvalidToken,
    #[error("WeBeep ha risposto con un errore ({errorcode}): {message}")]
    Api { errorcode: String, message: String },
    #[error("WeBeep ha risposto con lo stato HTTP {0}")]
    Status(u16),
    #[error("errore di rete verso WeBeep: {0}")]
    Network(String),
    #[error("risposta di WeBeep non riconosciuta: {0}")]
    Decode(String),
    #[error("configurazione del client non valida: {0}")]
    Config(String),
}

impl From<reqwest::Error> for MoodleError {
    fn from(err: reqwest::Error) -> Self {
        // `without_url` keeps query strings (which may carry `token=`) out of messages.
        Self::Network(err.without_url().to_string())
    }
}

impl From<serde_json::Error> for MoodleError {
    fn from(err: serde_json::Error) -> Self {
        Self::Decode(err.to_string())
    }
}
