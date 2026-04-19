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

/// Unique identifier for a branch
pub type BranchId = String;

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

    // === Phase 8.1: Incremental Checkpointing ===

    /// Parent checkpoint ID for delta chains (None = full snapshot)
    #[serde(default)]
    pub parent_checkpoint_id: Option<CheckpointId>,

    /// Whether this is a full snapshot (vs incremental delta)
    #[serde(default = "default_true")]
    pub is_full_snapshot: bool,

    /// Depth of this checkpoint in the delta chain (0 = full snapshot)
    #[serde(default)]
    pub delta_depth: u32,

    // === Phase 8.2: Branching ===

    /// Branch this checkpoint belongs to
    #[serde(default)]
    pub branch_id: Option<BranchId>,
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

    /// When true (default), a failed backup aborts restoration.
    /// When false, a backup failure is a non-fatal warning.
    #[serde(default = "default_true")]
    pub require_backup: bool,

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

    /// Per-file conflict resolutions (overrides `conflict_resolution` for specific files).
    /// Enables a two-pass restore: first pass detects conflicts and returns them,
    /// caller prompts user, second pass provides per-file decisions here.
    #[serde(default)]
    pub per_file_resolutions: HashMap<PathBuf, ConflictResolution>,

    /// Fallback strategy when `Prompt` is selected but interactive prompting is
    /// not available (e.g. in headless / Rust-only context). Defaults to `Backup`.
    #[serde(default = "ConflictResolution::backup_default")]
    pub prompt_fallback: ConflictResolution,

    /// When true, perform a dry-run: detect and return conflicts without
    /// actually restoring any files. Useful for the first pass of a two-pass flow.
    #[serde(default)]
    pub dry_run: bool,
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
    /// Prompt user for decision (in Rust context, falls back to `prompt_fallback`)
    Prompt,
}

impl ConflictResolution {
    /// Default fallback for prompt mode
    fn backup_default() -> Self {
        ConflictResolution::Backup
    }
}

fn default_true() -> bool {
    true
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

// ========================================
// Phase 8.1: Incremental Checkpointing Types
// ========================================

/// Configuration for incremental checkpointing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncrementalConfig {
    /// Enable incremental (delta) checkpoints
    pub enabled: bool,
    /// Maximum delta chain length before forcing a full snapshot
    pub max_chain_length: u32,
    /// Create a full snapshot every N checkpoints regardless of chain length
    pub full_snapshot_interval: u32,
}

impl Default for IncrementalConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_chain_length: 10,
            full_snapshot_interval: 20,
        }
    }
}

/// Result of reconstructing a checkpoint from its delta chain
#[derive(Debug, Clone)]
pub struct ReconstructedCheckpoint {
    /// The final reconstructed file states
    pub file_states: HashMap<PathBuf, ReconstructedFileState>,
    /// The complete file inventory
    pub file_inventory: Vec<PathBuf>,
    /// Number of deltas applied in reconstruction
    pub chain_length: u32,
    /// Total size of all deltas in the chain
    pub total_chain_size_bytes: u64,
}

/// A reconstructed file state from the delta chain
#[derive(Debug, Clone)]
pub struct ReconstructedFileState {
    pub path: PathBuf,
    pub content: Option<String>,
    pub content_hash: FileHash,
    pub size_bytes: u64,
    pub encoding: FileEncoding,
}

// ========================================
// Phase 8.2: Branching & Merging Types
// ========================================

/// A checkpoint branch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    /// Unique branch identifier
    pub id: BranchId,
    /// Human-readable branch name
    pub name: String,
    /// Session this branch belongs to
    pub session_id: SessionId,
    /// Checkpoint from which this branch was created
    pub base_checkpoint_id: CheckpointId,
    /// The latest (head) checkpoint on this branch
    pub head_checkpoint_id: Option<CheckpointId>,
    /// When this branch was created
    pub created_at: DateTime<Utc>,
    /// Description of the branch purpose
    pub description: String,
    /// Whether this is the default/main branch
    pub is_default: bool,
    /// Branch metadata
    pub metadata: HashMap<String, String>,
}

/// Result of merging two branches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    /// Whether the merge succeeded
    pub success: bool,
    /// The merge checkpoint ID (if created)
    pub merge_checkpoint_id: Option<CheckpointId>,
    /// Files that were merged successfully
    pub merged_files: Vec<PathBuf>,
    /// Files with merge conflicts
    pub conflicts: Vec<MergeConflict>,
    /// Strategy used for the merge
    pub strategy: MergeStrategy,
}

/// A merge conflict between two branches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeConflict {
    /// Path of the conflicting file
    pub file_path: PathBuf,
    /// Content from the source branch
    pub source_content: Option<String>,
    /// Content from the target branch
    pub target_content: Option<String>,
    /// Content from the common ancestor (base)
    pub base_content: Option<String>,
    /// Type of conflict
    pub conflict_type: MergeConflictType,
}

/// Types of merge conflicts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MergeConflictType {
    /// Both branches modified the same file differently
    BothModified,
    /// Source modified a file that target deleted
    ModifiedDeleted,
    /// Source deleted a file that target modified
    DeletedModified,
    /// Both branches created a file at the same path
    BothCreated,
}

/// Strategy for merging branches
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[derive(Default)]
pub enum MergeStrategy {
    /// Attempt automatic 3-way merge, report conflicts
    #[default]
    ThreeWay,
    /// Source branch wins on conflicts
    SourceWins,
    /// Target branch wins on conflicts
    TargetWins,
}


// ========================================
// Phase 8.3: AI-Powered Analysis Types
// ========================================

/// Result of AI-powered checkpoint analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointAnalysis {
    /// Auto-generated description of the changes
    pub generated_description: String,
    /// Risk assessment for this checkpoint
    pub risk_assessment: RiskAssessment,
    /// Impact analysis - which features/areas are affected
    pub impact_analysis: ImpactAnalysis,
    /// Suggested checkpoint grouping
    pub grouping_suggestion: Option<GroupingSuggestion>,
}

/// Risk assessment for a checkpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    /// Overall risk level
    pub level: RiskLevel,
    /// Risk score from 0.0 (safe) to 1.0 (dangerous)
    pub score: f64,
    /// Specific risk factors identified
    pub factors: Vec<RiskFactor>,
    /// Recommended actions to mitigate risk
    pub recommendations: Vec<String>,
}

/// Risk levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// A specific risk factor found in the checkpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    /// Category of risk
    pub category: RiskCategory,
    /// Description of the risk
    pub description: String,
    /// Files involved
    pub affected_files: Vec<PathBuf>,
    /// Weight of this factor (0.0 to 1.0)
    pub weight: f64,
}

/// Categories of risk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskCategory {
    /// Changes to configuration files
    ConfigChange,
    /// Changes to security-sensitive files
    SecuritySensitive,
    /// Large-scale deletions
    MassDeletion,
    /// Changes to build/deploy infrastructure
    InfrastructureChange,
    /// API/interface changes
    ApiChange,
    /// Database migration changes
    DatabaseChange,
    /// Changes to many interconnected files
    HighCoupling,
    /// Changes to files with no tests
    Untested,
}

/// Impact analysis for a checkpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAnalysis {
    /// Feature areas affected by the changes
    pub affected_features: Vec<AffectedFeature>,
    /// Architectural layers touched
    pub affected_layers: Vec<String>,
    /// Estimated scope of changes
    pub scope: ChangeScope,
    /// Files transitively affected (downstream dependencies)
    pub transitive_impact: Vec<PathBuf>,
}

/// A feature area affected by changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectedFeature {
    /// Name of the feature area
    pub name: String,
    /// How much of this feature area is affected
    pub impact_level: ImpactLevel,
    /// Files in this feature area that changed
    pub changed_files: Vec<PathBuf>,
}

/// Impact levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ImpactLevel {
    Minor,
    Moderate,
    Major,
}

/// Scope of changes
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ChangeScope {
    /// Single function/method
    Function,
    /// Single file
    File,
    /// Single module/component
    Module,
    /// Multiple modules
    CrossModule,
    /// Architectural changes
    Architecture,
}

/// Suggestion for grouping related checkpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupingSuggestion {
    /// Suggested group name
    pub group_name: String,
    /// Checkpoint IDs that belong together
    pub checkpoint_ids: Vec<CheckpointId>,
    /// Why these checkpoints are related
    pub rationale: String,
    /// Confidence in this grouping (0.0 to 1.0)
    pub confidence: f64,
}

// ========================================
// Phase 8.4: Collaborative Checkpoint Types
// ========================================

/// A user/collaborator identity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaboratorInfo {
    /// Unique user identifier
    pub user_id: String,
    /// Display name
    pub display_name: String,
    /// Machine/device identifier
    pub machine_id: String,
    /// When this user was last seen
    pub last_seen: DateTime<Utc>,
}

/// A shared checkpoint bundle for cross-machine/user sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedCheckpointBundle {
    /// Unique bundle ID
    pub id: String,
    /// Checkpoint IDs included in this bundle
    pub checkpoint_ids: Vec<CheckpointId>,
    /// Who shared the bundle
    pub shared_by: CollaboratorInfo,
    /// When it was shared
    pub shared_at: DateTime<Utc>,
    /// Description of the shared set
    pub description: String,
    /// Export format version
    pub format_version: u32,
    /// Serialized checkpoint data (checkpoints + file changes)
    pub data: Vec<u8>,
}

/// Status of a sync operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncStatus {
    /// No sync in progress
    Idle,
    /// Currently syncing
    InProgress,
    /// Last sync succeeded
    Succeeded { at: DateTime<Utc> },
    /// Last sync failed
    Failed { at: DateTime<Utc>, error: String },
}

/// A record in the compliance audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    /// Unique record ID
    pub id: String,
    /// When the event occurred
    pub timestamp: DateTime<Utc>,
    /// Who performed the action
    pub user_id: String,
    /// Machine where action was performed
    pub machine_id: String,
    /// What action was taken
    pub action: String,
    /// What resource was affected
    pub resource_type: String,
    /// Resource identifier (e.g. checkpoint ID)
    pub resource_id: String,
    /// Detailed context
    pub details: HashMap<String, String>,
    /// Outcome of the action
    pub outcome: AuditOutcome,
}

/// Outcome of an audited action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditOutcome {
    Success,
    Failure(String),
    PartialSuccess(String),
}

// ========================================
// Phase 8.5: Performance Monitoring Types
// ========================================

/// Snapshot of storage usage at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageUsageSnapshot {
    /// When this snapshot was taken
    pub timestamp: DateTime<Utc>,
    /// Total storage bytes used
    pub total_bytes: u64,
    /// Storage used by checkpoint data
    pub checkpoint_data_bytes: u64,
    /// Storage used by database
    pub database_bytes: u64,
    /// Number of stored content blobs
    pub blob_count: u64,
    /// Number of checkpoints
    pub checkpoint_count: u64,
}

/// Checkpoint creation frequency data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreationFrequencyPoint {
    /// Time bucket (hour/day)
    pub bucket: DateTime<Utc>,
    /// Number of checkpoints created in this bucket
    pub count: u64,
    /// Breakdown by type (manual, auto, ai)
    pub by_type: HashMap<String, u64>,
}

/// Restoration event for tracking success/failure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestorationEvent {
    /// When the restoration happened
    pub timestamp: DateTime<Utc>,
    /// Checkpoint that was restored
    pub checkpoint_id: CheckpointId,
    /// Whether it succeeded
    pub success: bool,
    /// Duration in milliseconds
    pub duration_ms: f64,
    /// Number of files restored
    pub files_restored: u64,
    /// Number of files that failed
    pub files_failed: u64,
    /// Error message if failed
    pub error: Option<String>,
}

/// AI session productivity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AISessionMetrics {
    /// Session identifier
    pub session_id: SessionId,
    /// When the session started
    pub started_at: DateTime<Utc>,
    /// When the session ended (None if still active)
    pub ended_at: Option<DateTime<Utc>>,
    /// Total files changed during session
    pub files_changed: u64,
    /// Total lines added
    pub lines_added: u64,
    /// Total lines deleted
    pub lines_deleted: u64,
    /// Number of checkpoints created during session
    pub checkpoints_created: u64,
    /// Number of rollbacks performed
    pub rollbacks: u64,
    /// Session duration in seconds
    pub duration_seconds: f64,
}

/// Aggregated performance dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceDashboard {
    /// Storage usage over time
    pub storage_history: Vec<StorageUsageSnapshot>,
    /// Checkpoint creation frequency
    pub creation_frequency: Vec<CreationFrequencyPoint>,
    /// Restoration events log
    pub restoration_events: Vec<RestorationEvent>,
    /// AI session metrics
    pub ai_session_metrics: Vec<AISessionMetrics>,
    /// Current storage usage
    pub current_storage: StorageUsageSnapshot,
    /// Summary statistics
    pub summary: DashboardSummary,
}

/// Summary statistics for the dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSummary {
    /// Total checkpoints ever created
    pub total_checkpoints_created: u64,
    /// Total restorations performed
    pub total_restorations: u64,
    /// Restoration success rate (0.0 to 1.0)
    pub restoration_success_rate: f64,
    /// Average checkpoint creation time in ms
    pub avg_creation_time_ms: f64,
    /// Average restoration time in ms
    pub avg_restoration_time_ms: f64,
    /// Total AI sessions
    pub total_ai_sessions: u64,
    /// Average changes per AI session
    pub avg_changes_per_session: f64,
    /// Total rollbacks across all sessions
    pub total_rollbacks: u64,
}
