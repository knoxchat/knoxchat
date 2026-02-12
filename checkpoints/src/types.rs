//! Core types for the checkpoint system

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Unique identifier for a checkpoint
pub type CheckpointId = Uuid;

/// Unique identifier for a session
pub type SessionId = Uuid;

/// File hash type
pub type FileHash = String;

/// Checkpoint metadata and file changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Unique identifier for this checkpoint
    pub id: CheckpointId,

    /// Session this checkpoint belongs to
    pub session_id: SessionId,

    /// Human-readable description of changes
    pub description: String,

    /// When this checkpoint was created
    pub created_at: DateTime<Utc>,

    /// Files that were changed in this checkpoint
    pub file_changes: Vec<FileChange>,

    /// COMPLETE inventory of ALL files in workspace at checkpoint time
    /// This is CRITICAL for proper restoration - files not in this list should be removed
    pub file_inventory: Vec<PathBuf>,

    /// Total number of files affected
    pub files_affected: usize,

    /// Size of checkpoint data in bytes
    pub size_bytes: u64,

    /// Tags for categorization
    pub tags: Vec<String>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Represents a change to a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    /// Path to the file (relative to workspace root)
    pub path: PathBuf,

    /// Type of change
    pub change_type: ChangeType,

    /// Original file content (for modified/deleted files)
    pub original_content: Option<String>,

    /// New file content (for created/modified files)  
    pub new_content: Option<String>,

    /// File size in bytes
    pub size_bytes: u64,

    /// File hash for deduplication
    pub content_hash: FileHash,

    /// File permissions (Unix-style)
    pub permissions: Option<u32>,

    /// Last modified timestamp
    pub modified_at: DateTime<Utc>,

    /// Encoding used for the file content
    pub encoding: FileEncoding,

    /// Whether the content was compressed
    pub compressed: bool,
}

/// Type of change made to a file
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    /// File was created
    Created,
    /// File was modified
    Modified,
    /// File was deleted
    Deleted,
    /// File was renamed
    Renamed { from: PathBuf },
    /// File was moved
    Moved { from: PathBuf },
}

/// File encoding types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileEncoding {
    /// UTF-8 text file
    Utf8,
    /// ASCII text file
    Ascii,
    /// Binary file (base64 encoded)
    Binary,
    /// Unknown encoding
    Unknown,
}

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session identifier
    pub id: SessionId,

    /// Workspace path for this session
    pub workspace_path: PathBuf,

    /// When the session was created
    pub created_at: DateTime<Utc>,

    /// When the session was last accessed
    pub last_accessed: DateTime<Utc>,

    /// Number of checkpoints in this session
    pub checkpoint_count: usize,

    /// Total size of all checkpoints in bytes
    pub total_size_bytes: u64,

    /// Session metadata
    pub metadata: HashMap<String, String>,
}

/// Statistics for checkpoint system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointStats {
    /// Total number of checkpoints
    pub total_checkpoints: usize,

    /// Total number of sessions
    pub total_sessions: usize,

    /// Total storage used in bytes
    pub total_storage_bytes: u64,

    /// Average checkpoint size
    pub avg_checkpoint_size: u64,

    /// Number of files tracked
    pub files_tracked: usize,

    /// Compression ratio (0.0 to 1.0)
    pub compression_ratio: f64,

    /// Deduplication savings in bytes
    pub deduplication_savings: u64,

    /// Last cleanup timestamp
    pub last_cleanup: Option<DateTime<Utc>>,

    /// Performance metrics
    pub performance: PerformanceMetrics,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Average time to create checkpoint (milliseconds)
    pub avg_creation_time_ms: f64,

    /// Average time to restore checkpoint (milliseconds)
    pub avg_restoration_time_ms: f64,

    /// Database query performance (queries per second)
    pub db_queries_per_second: f64,

    /// File I/O performance (MB/s)
    pub file_io_mbps: f64,

    /// Memory usage in MB
    pub memory_usage_mb: f64,
}

/// Checkpoint restoration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreOptions {
    /// Whether to create a backup before restoring
    pub create_backup: bool,

    /// Whether to restore file permissions
    pub restore_permissions: bool,

    /// Whether to restore timestamps
    pub restore_timestamps: bool,

    /// Files to include (empty means all)
    pub include_files: Vec<PathBuf>,

    /// Files to exclude
    pub exclude_files: Vec<PathBuf>,

    /// How to handle conflicts
    pub conflict_resolution: ConflictResolution,

    /// Whether to validate checksums
    pub validate_checksums: bool,
}

/// How to resolve conflicts during restoration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// Skip conflicting files
    Skip,
    /// Overwrite existing files
    Overwrite,
    /// Create backup copies
    Backup,
    /// Prompt user for decision
    Prompt,
}

/// Backup information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    /// Unique backup identifier
    pub id: Uuid,

    /// Path to backup file
    pub path: PathBuf,

    /// When backup was created
    pub created_at: DateTime<Utc>,

    /// Size of backup in bytes
    pub size_bytes: u64,

    /// Checkpoints included in backup
    pub checkpoint_ids: Vec<CheckpointId>,

    /// Backup format version
    pub format_version: u32,

    /// Compression used
    pub compression_type: CompressionType,
}

/// Compression types supported
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionType {
    /// No compression
    None,
    /// LZ4 compression (fast)
    Lz4,
    /// Gzip compression (good ratio)
    Gzip,
    /// Zstd compression (best of both)
    Zstd,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unique entry identifier
    pub id: Uuid,

    /// When the action occurred
    pub timestamp: DateTime<Utc>,

    /// Type of action performed
    pub action: AuditAction,

    /// User/system that performed the action
    pub actor: String,

    /// Resource affected (checkpoint ID, file path, etc.)
    pub resource: String,

    /// Additional details
    pub details: HashMap<String, String>,

    /// Result of the action
    pub result: AuditResult,
}

/// Types of auditable actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditAction {
    /// Checkpoint created
    CheckpointCreated,
    /// Checkpoint restored
    CheckpointRestored,
    /// Checkpoint deleted
    CheckpointDeleted,
    /// Session started
    SessionStarted,
    /// Session ended
    SessionEnded,
    /// Cleanup performed
    CleanupPerformed,
    /// Backup created
    BackupCreated,
    /// Backup restored
    BackupRestored,
    /// Configuration changed
    ConfigurationChanged,
}

/// Result of an audited action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditResult {
    /// Action succeeded
    Success,
    /// Action failed
    Failed,
    /// Action was partial success
    Partial,
}

/// Configuration for checkpoint creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointOptions {
    /// Custom description for the checkpoint
    pub description: Option<String>,

    /// Tags to apply to the checkpoint
    pub tags: Vec<String>,

    /// Whether to compress the checkpoint data
    pub compress: bool,

    /// Maximum number of files to include
    pub max_files: Option<usize>,

    /// File patterns to exclude
    pub exclude_patterns: Vec<String>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Default for CheckpointOptions {
    fn default() -> Self {
        Self {
            description: None,
            tags: Vec::new(),
            compress: true,
            max_files: Some(100),
            exclude_patterns: vec![
                "node_modules/**".to_string(),
                ".git/**".to_string(),
                "target/**".to_string(),
                "*.log".to_string(),
                "*.tmp".to_string(),
            ],
            metadata: HashMap::new(),
        }
    }
}

impl ChangeType {
    /// Get a short string representation for display
    pub fn as_short_str(&self) -> &'static str {
        match self {
            ChangeType::Created => "+",
            ChangeType::Modified => "=",
            ChangeType::Deleted => "-",
            ChangeType::Renamed { .. } => "→",
            ChangeType::Moved { .. } => "↗",
        }
    }

    /// Get a descriptive string for display
    pub fn as_display_str(&self) -> &'static str {
        match self {
            ChangeType::Created => "created",
            ChangeType::Modified => "modified",
            ChangeType::Deleted => "deleted",
            ChangeType::Renamed { .. } => "renamed",
            ChangeType::Moved { .. } => "moved",
        }
    }
}

impl FileChange {
    /// Get the final file path after any moves
    pub fn final_path(&self) -> &PathBuf {
        &self.path
    }

    /// Check if this change involves content modification
    pub fn has_content_change(&self) -> bool {
        matches!(self.change_type, ChangeType::Created | ChangeType::Modified)
    }

    /// Get the size change in bytes (positive = increased, negative = decreased)
    pub fn size_delta(&self) -> i64 {
        match self.change_type {
            ChangeType::Created => self.size_bytes as i64,
            ChangeType::Deleted => -(self.size_bytes as i64),
            ChangeType::Modified => 0, // Would need original size to calculate
            _ => 0,
        }
    }
}

impl Checkpoint {
    /// Generate a human-readable description based on file changes
    pub fn generate_description(&self) -> String {
        let mut created = 0;
        let mut modified = 0;
        let mut deleted = 0;
        let mut moved = 0;

        for change in &self.file_changes {
            match change.change_type {
                ChangeType::Created => created += 1,
                ChangeType::Modified => modified += 1,
                ChangeType::Deleted => deleted += 1,
                ChangeType::Renamed { .. } | ChangeType::Moved { .. } => moved += 1,
            }
        }

        let mut parts = Vec::new();

        if created > 0 {
            parts.push(format!(
                "created {} file{}",
                created,
                if created == 1 { "" } else { "s" }
            ));
        }
        if modified > 0 {
            parts.push(format!(
                "modified {} file{}",
                modified,
                if modified == 1 { "" } else { "s" }
            ));
        }
        if deleted > 0 {
            parts.push(format!(
                "deleted {} file{}",
                deleted,
                if deleted == 1 { "" } else { "s" }
            ));
        }
        if moved > 0 {
            parts.push(format!(
                "moved {} file{}",
                moved,
                if moved == 1 { "" } else { "s" }
            ));
        }

        if parts.is_empty() {
            "No changes".to_string()
        } else {
            format!("Agent response - {}", parts.join(", "))
        }
    }

    /// Get a short summary for display in UI
    pub fn short_summary(&self) -> String {
        let created = self
            .file_changes
            .iter()
            .filter(|c| c.change_type == ChangeType::Created)
            .count();
        let modified = self
            .file_changes
            .iter()
            .filter(|c| c.change_type == ChangeType::Modified)
            .count();
        let deleted = self
            .file_changes
            .iter()
            .filter(|c| c.change_type == ChangeType::Deleted)
            .count();

        let mut parts = Vec::new();
        if created > 0 {
            parts.push(format!("+{}", created));
        }
        if modified > 0 {
            parts.push(format!("={}", modified));
        }
        if deleted > 0 {
            parts.push(format!("-{}", deleted));
        }

        if parts.is_empty() {
            "No changes".to_string()
        } else {
            parts.join(" ")
        }
    }
}
