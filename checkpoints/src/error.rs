//! Error types for the checkpoint system

use std::path::PathBuf;
use thiserror::Error;

/// Result type for checkpoint operations
pub type Result<T> = std::result::Result<T, CheckpointError>;

/// Errors that can occur in the checkpoint system
#[derive(Error, Debug)]
pub enum CheckpointError {
    /// Database operation failed
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    /// File system operation failed
    #[error("File system error: {message}")]
    FileSystem { message: String },

    /// I/O operation failed
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Directory walking error
    #[error("Directory walk error: {0}")]
    WalkDir(#[from] walkdir::Error),

    /// File system watcher error
    #[error("File watcher error: {0}")]
    Notify(#[from] notify::Error),

    /// System time error
    #[error("System time error: {0}")]
    SystemTime(#[from] std::time::SystemTimeError),

    /// Serialization/deserialization failed
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Checkpoint not found
    #[error("Checkpoint not found: {id}")]
    CheckpointNotFound { id: String },

    /// Session not found
    #[error("Session not found: {id}")]
    SessionNotFound { id: String },

    /// File not found
    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },

    /// File access denied
    #[error("Access denied: {path}")]
    AccessDenied { path: PathBuf },

    /// Invalid configuration
    #[error("Invalid configuration: {message}")]
    InvalidConfig { message: String },

    /// Storage limit exceeded
    #[error("Storage limit exceeded: {current} bytes (limit: {limit} bytes)")]
    StorageLimitExceeded { current: u64, limit: u64 },

    /// File limit exceeded
    #[error("File limit exceeded: {current} files (limit: {limit} files)")]
    FileLimitExceeded { current: usize, limit: usize },

    /// Restoration failed
    #[error("Restoration failed: {message}")]
    RestorationFailed { message: String },

    /// Compression/decompression failed
    #[error("Compression error: {message}")]
    Compression { message: String },

    /// Validation failed
    #[error("Validation error: {message}")]
    Validation { message: String },

    /// Concurrent operation conflict
    #[error("Concurrent operation conflict: {message}")]
    Conflict { message: String },

    /// Generic error with context
    #[error("Checkpoint error: {message}")]
    Generic { message: String },
}

impl CheckpointError {
    /// Create a file system error
    pub fn file_system<S: Into<String>>(message: S) -> Self {
        Self::FileSystem {
            message: message.into(),
        }
    }

    /// Create a file not found error
    pub fn file_not_found<P: Into<PathBuf>>(path: P) -> Self {
        Self::FileNotFound { path: path.into() }
    }

    /// Create an access denied error
    pub fn access_denied<P: Into<PathBuf>>(path: P) -> Self {
        Self::AccessDenied { path: path.into() }
    }

    /// Create an invalid config error
    pub fn invalid_config<S: Into<String>>(message: S) -> Self {
        Self::InvalidConfig {
            message: message.into(),
        }
    }

    /// Create a restoration failed error
    pub fn restoration_failed<S: Into<String>>(message: S) -> Self {
        Self::RestorationFailed {
            message: message.into(),
        }
    }

    /// Create a compression error
    pub fn compression<S: Into<String>>(message: S) -> Self {
        Self::Compression {
            message: message.into(),
        }
    }

    /// Create a validation error
    pub fn validation<S: Into<String>>(message: S) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    /// Create a conflict error
    pub fn conflict<S: Into<String>>(message: S) -> Self {
        Self::Conflict {
            message: message.into(),
        }
    }

    /// Create a generic error
    pub fn generic<S: Into<String>>(message: S) -> Self {
        Self::Generic {
            message: message.into(),
        }
    }

    /// Create a checkpoint not found error
    pub fn checkpoint_not_found<S: Into<String>>(id: S) -> Self {
        Self::CheckpointNotFound { id: id.into() }
    }

    /// Alias for checkpoint_not_found
    pub fn not_found<S: Into<String>>(id: S) -> Self {
        Self::CheckpointNotFound { id: id.into() }
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::Database(_) => false,
            Self::FileSystem { .. } => true,
            Self::Io(_) => true,
            Self::WalkDir(_) => true,
            Self::Serialization(_) => false,
            Self::CheckpointNotFound { .. } => false,
            Self::SessionNotFound { .. } => false,
            Self::FileNotFound { .. } => true,
            Self::AccessDenied { .. } => false,
            Self::InvalidConfig { .. } => false,
            Self::StorageLimitExceeded { .. } => true,
            Self::FileLimitExceeded { .. } => true,
            Self::RestorationFailed { .. } => true,
            Self::Compression { .. } => true,
            Self::Validation { .. } => false,
            Self::Conflict { .. } => true,
            Self::Generic { .. } => true,
            Self::Notify(_) => true,
            Self::SystemTime(_) => true,
        }
    }

    /// Get error category for logging/metrics
    pub fn category(&self) -> &'static str {
        match self {
            Self::Database(_) => "database",
            Self::FileSystem { .. } => "filesystem",
            Self::Io(_) => "io",
            Self::WalkDir(_) => "filesystem",
            Self::Serialization(_) => "serialization",
            Self::CheckpointNotFound { .. } => "not_found",
            Self::SessionNotFound { .. } => "not_found",
            Self::FileNotFound { .. } => "not_found",
            Self::AccessDenied { .. } => "access",
            Self::InvalidConfig { .. } => "config",
            Self::StorageLimitExceeded { .. } => "limits",
            Self::FileLimitExceeded { .. } => "limits",
            Self::RestorationFailed { .. } => "restoration",
            Self::Compression { .. } => "compression",
            Self::Validation { .. } => "validation",
            Self::Conflict { .. } => "conflict",
            Self::Generic { .. } => "generic",
            Self::Notify(_) => "filesystem",
            Self::SystemTime(_) => "system",
        }
    }
}
