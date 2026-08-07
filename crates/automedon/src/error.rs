use std::time::Duration;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("unknown harness adapter: {0}")]
    UnknownAdapter(String),

    #[error("harness not found on PATH: {0}")]
    HarnessNotFound(String),

    #[error("session already finished")]
    SessionFinished,

    #[error("session has no active turn")]
    NoActiveTurn,

    #[error("expect timed out after {timeout:?}: {predicate}")]
    ExpectTimeout {
        timeout: Duration,
        predicate: String,
    },

    #[error("expect failed: {0}")]
    ExpectFailed(String),

    #[error("process exited with status {code:?}: {stderr}")]
    ProcessFailed { code: Option<i32>, stderr: String },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("script error: {0}")]
    Script(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<anyhow::Error> for Error {
    fn from(value: anyhow::Error) -> Self {
        Error::Other(value.to_string())
    }
}
