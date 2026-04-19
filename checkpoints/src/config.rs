//! Configuration for the checkpoint system

use crate::error::{CheckpointError, Result};
use chrono::Duration;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for the checkpoint system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointConfig {
    /// Base directory for storing checkpoints
    pub storage_path: PathBuf,

    /// Whether we're in debug mode (affects storage path)
    pub debug_mode: bool,

    /// Maximum number of checkpoints to keep
    pub max_checkpoints: usize,

    /// Maximum age of checkpoints before cleanup
    pub retention_days: i64,

    /// Maximum storage size for all checkpoints (bytes)
    pub max_storage_bytes: u64,

    /// Maximum number of files per checkpoint
    pub max_files_per_checkpoint: usize,

    /// Whether to compress checkpoint data
    pub enable_compression: bool,

    /// File extensions to track (empty means all text files)
    pub tracked_extensions: Vec<String>,

    /// Global ignore patterns
    pub global_ignore_patterns: Vec<String>,

    /// Whether to automatically cleanup old checkpoints
    pub auto_cleanup: bool,

    /// Cleanup interval in hours
    pub cleanup_interval_hours: u64,

    /// Database connection pool size
    pub db_pool_size: u32,

    /// Enable detailed logging
    pub verbose_logging: bool,

    /// Maximum directory depth to scan (prevents infinite recursion)
    pub max_scan_depth: usize,

    /// Maximum file size to track in bytes (larger files are skipped)
    pub max_file_size_bytes: u64,

    /// Enable file system watcher for real-time tracking
    pub enable_file_watcher: bool,

    /// Enable performance metrics collection
    pub enable_performance_metrics: bool,
}

impl Default for CheckpointConfig {
    fn default() -> Self {
        Self {
            storage_path: Self::default_storage_path(false),
            debug_mode: false,
            max_checkpoints: 1000,
            retention_days: 7,
            max_storage_bytes: 1_000_000_000, // 1GB
            max_files_per_checkpoint: 100,
            enable_compression: true,
            tracked_extensions: vec![
                "ts".to_string(),
                "tsx".to_string(),
                "js".to_string(),
                "jsx".to_string(),
                "py".to_string(),
                "java".to_string(),
                "cpp".to_string(),
                "c".to_string(),
                "cs".to_string(),
                "go".to_string(),
                "rs".to_string(),
                "php".to_string(),
                "rb".to_string(),
                "swift".to_string(),
                "kt".to_string(),
                "html".to_string(),
                "css".to_string(),
                "scss".to_string(),
                "json".to_string(),
                "yaml".to_string(),
                "yml".to_string(),
                "md".to_string(),
                "txt".to_string(),
            ],
            global_ignore_patterns: vec![
                "**/.git/**".to_string(),
                "**/node_modules/**".to_string(),
                "**/target/**".to_string(),
                "**/.knox/**".to_string(),
                "**/.knox-debug/**".to_string(),
                "**/__pycache__/**".to_string(),
                "**/.pytest_cache/**".to_string(),
                "**/.vscode/**".to_string(),
                "**/.idea/**".to_string(),
                "*.log".to_string(),
                "*.tmp".to_string(),
                "*.swp".to_string(),
                "*.bak".to_string(),
                "*.db".to_string(),
                "*.sqlite".to_string(),
                "*.bin".to_string(),
                "*.exe".to_string(),
                "*.dll".to_string(),
                "*.so".to_string(),
                "*.dylib".to_string(),
                "*.png".to_string(),
                "*.jpg".to_string(),
                "*.jpeg".to_string(),
                "*.gif".to_string(),
                "*.svg".to_string(),
                "*.ico".to_string(),
                "*.pdf".to_string(),
                "*.zip".to_string(),
                "*.tar".to_string(),
                "*.gz".to_string(),
            ],
            auto_cleanup: true,
            cleanup_interval_hours: 24,
            db_pool_size: 10,
            verbose_logging: false,
            max_scan_depth: 10,
            max_file_size_bytes: 1_048_576, // 1MB
            enable_file_watcher: true,
            enable_performance_metrics: true,
        }
    }
}

impl CheckpointConfig {
    /// Create a new configuration with debug mode
    pub fn debug() -> Self {
        Self {
            storage_path: Self::default_storage_path(true),
            debug_mode: true,
            verbose_logging: true,
            ..Default::default()
        }
    }

    /// Get the default storage path based on the getKnoxGlobalPath utility
    pub fn default_storage_path(debug_mode: bool) -> PathBuf {
        let home = dirs::home_dir().expect("Unable to get home directory");
        if debug_mode {
            home.join(".knox-debug").join("checkpoints")
        } else {
            home.join(".knox").join("checkpoints")
        }
    }

    /// Get the database file path
    pub fn database_path(&self) -> PathBuf {
        self.storage_path.join("checkpoints.db")
    }

    /// Get the data directory for checkpoint files
    pub fn data_path(&self) -> PathBuf {
        self.storage_path.join("data")
    }

    /// Get the backup directory
    pub fn backup_path(&self) -> PathBuf {
        self.storage_path.join("backups")
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.max_checkpoints == 0 {
            return Err(CheckpointError::invalid_config(
                "max_checkpoints must be greater than 0",
            ));
        }

        if self.retention_days < 1 {
            return Err(CheckpointError::invalid_config(
                "retention_days must be at least 1",
            ));
        }

        if self.max_storage_bytes == 0 {
            return Err(CheckpointError::invalid_config(
                "max_storage_bytes must be greater than 0",
            ));
        }

        if self.max_files_per_checkpoint == 0 {
            return Err(CheckpointError::invalid_config(
                "max_files_per_checkpoint must be greater than 0",
            ));
        }

        if self.cleanup_interval_hours == 0 {
            return Err(CheckpointError::invalid_config(
                "cleanup_interval_hours must be greater than 0",
            ));
        }

        if self.db_pool_size == 0 {
            return Err(CheckpointError::invalid_config(
                "db_pool_size must be greater than 0",
            ));
        }

        if self.max_scan_depth == 0 {
            return Err(CheckpointError::invalid_config(
                "max_scan_depth must be greater than 0",
            ));
        }

        if self.max_file_size_bytes < 1024 {
            return Err(CheckpointError::invalid_config(
                "max_file_size_bytes must be at least 1024 (1KB)",
            ));
        }

        Ok(())
    }

    /// Create directories if they don't exist
    pub fn ensure_directories(&self) -> Result<()> {
        std::fs::create_dir_all(&self.storage_path).map_err(|e| {
            CheckpointError::file_system(format!("Failed to create storage directory: {}", e))
        })?;

        std::fs::create_dir_all(self.data_path()).map_err(|e| {
            CheckpointError::file_system(format!("Failed to create data directory: {}", e))
        })?;

        std::fs::create_dir_all(self.backup_path()).map_err(|e| {
            CheckpointError::file_system(format!("Failed to create backup directory: {}", e))
        })?;

        Ok(())
    }

    /// Get retention duration
    pub fn retention_duration(&self) -> Duration {
        Duration::days(self.retention_days)
    }

    /// Get cleanup interval duration
    pub fn cleanup_interval_duration(&self) -> Duration {
        Duration::hours(self.cleanup_interval_hours as i64)
    }

    /// Check if a file extension should be tracked
    pub fn should_track_extension(&self, extension: &str) -> bool {
        if self.tracked_extensions.is_empty() {
            // If no specific extensions are configured, track common text files
            matches!(
                extension.to_lowercase().as_str(),
                "txt"
                    | "md"
                    | "json"
                    | "yaml"
                    | "yml"
                    | "toml"
                    | "xml"
                    | "html"
                    | "css"
                    | "js"
                    | "ts"
                    | "py"
                    | "rs"
                    | "go"
                    | "java"
                    | "cpp"
                    | "c"
                    | "h"
                    | "cs"
                    | "php"
                    | "rb"
                    | "swift"
                    | "kt"
                    | "scala"
                    | "sh"
                    | "bat"
                    | "ps1"
            )
        } else {
            self.tracked_extensions
                .iter()
                .any(|ext| ext.eq_ignore_ascii_case(extension))
        }
    }

    /// Load configuration from file
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_json::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
