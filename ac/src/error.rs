//! Error types for the autocomplete system

use thiserror::Error;

pub type Result<T> = std::result::Result<T, AutocompleteError>;

#[derive(Debug, Error)]
pub enum AutocompleteError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("String UTF-8 error: {0}")]
    StringUtf8(#[from] std::string::FromUtf8Error),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Neon error: {0}")]
    Neon(String),
}

