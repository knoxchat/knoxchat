//! Main checkpoint manager that orchestrates all checkpoint operations

use crate::changeset_tracker::{ChangesetTracker, OperationMode};
use crate::config::CheckpointConfig;
use crate::db::CheckpointDatabase;
use crate::error::{CheckpointError, Result};
use crate::file_tracker::FileTracker;
use crate::restoration::CheckpointRestoration;
use crate::storage::CheckpointStorage;
use crate::types::*;
use chrono::Utc;

use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

/// Main checkpoint manager that orchestrates all checkpoint operations
pub struct CheckpointManager {
    config: CheckpointConfig,
    database: Arc<CheckpointDatabase>,
    storage: Arc<CheckpointStorage>,
    file_tracker: Arc<RwLock<FileTracker>>,
    changeset_tracker: Arc<ChangesetTracker>,
    restoration: Arc<CheckpointRestoration>,
    current_session_id: SessionId,
    workspace_path: PathBuf,
    performance_tracker: Arc<RwLock<PerformanceTracker>>,
    /// Whether to use the new changeset-based tracking (agent mode only)
    use_changeset_tracking: bool,
}

/// Tracks performance metrics for operations
#[derive(Debug, Default)]
struct PerformanceTracker {
    checkpoint_creation_times: Vec<f64>,
    restoration_times: Vec<f64>,
}

impl CheckpointManager {
    /// Create a new checkpoint manager with smart changeset tracking
    pub fn new(
        config: CheckpointConfig,
        workspace_path: PathBuf,
        session_id: SessionId,
    ) -> Result<Self> {
        let database = Arc::new(CheckpointDatabase::new(
            config.storage_path.join("checkpoints.db"),
        )?);
        let storage = Arc::new(CheckpointStorage::new(config.clone())?);
        let file_tracker = Arc::new(RwLock::new(FileTracker::new(
            config.clone(),
            workspace_path.clone(),
        )));
        let changeset_tracker = Arc::new(ChangesetTracker::new(
            config.clone(),
            workspace_path.clone(),
        )?);
        let restoration = Arc::new(CheckpointRestoration::new(
            config.clone(),
            (*storage).clone(),
            workspace_path.clone(),
        ));

        Ok(Self {
            config: config.clone(),
            database,
            storage,
            file_tracker,
            changeset_tracker,
            restoration,
            current_session_id: session_id,
            workspace_path,
            performance_tracker: Arc::new(RwLock::new(PerformanceTracker::default())),
            use_changeset_tracking: true, // Enable by default for better performance
        })
    }

    /// Start an agent session for tracking AI-generated changes
    pub fn start_agent_session(&self, session_id: SessionId) -> Result<()> {
        if self.use_changeset_tracking {
            self.changeset_tracker.start_agent_session(session_id);
            log::info!(
                "Started agent session for changeset tracking: {}",
                session_id
            );
        }
        Ok(())
    }

    /// Stop the current agent session
    pub fn stop_agent_session(&self) -> Result<()> {
        if self.use_changeset_tracking {
            self.changeset_tracker.stop_agent_session();
            log::info!("Stopped agent session for changeset tracking");
        }
        Ok(())
    }

    /// Set the operation mode for the checkpoint system
    pub fn set_operation_mode(&self, mode: OperationMode) -> Result<()> {
        if self.use_changeset_tracking {
            self.changeset_tracker.set_mode(mode);
            log::info!("Set checkpoint operation mode to {:?}", mode);
        }
        Ok(())
    }

    /// Notify the system that AI is about to work on specific files
    pub fn track_ai_files(&self, file_paths: &[PathBuf]) -> Result<()> {
        if self.use_changeset_tracking {
            self.changeset_tracker.watch_files(file_paths)?;
            log::debug!("Added {} files to AI tracking", file_paths.len());
        }
        Ok(())
    }

    /// Create a checkpoint using the smart changeset system (agent mode only)
    pub fn create_agent_checkpoint(&self, options: CheckpointOptions) -> Result<CheckpointId> {
        if !self.use_changeset_tracking {
            // Fall back to traditional method
            return self.create_checkpoint_traditional(options);
        }

        let start_time = Instant::now();

        // Process any pending file system events
        self.changeset_tracker.process_events()?;

        // Get changes from the changeset tracker
        let changeset_entries = self.changeset_tracker.consume_pending_changes();

        if changeset_entries.is_empty() && options.description.is_none() {
            return Err(CheckpointError::validation(
                "No AI-generated changes detected for checkpoint",
            ));
        }

        // Convert changeset entries to file changes
        let file_changes = self
            .changeset_tracker
            .changeset_to_file_changes(&changeset_entries)?;

        // Apply file limits
        let limited_changes = if let Some(max_files) = options.max_files {
            file_changes.into_iter().take(max_files).collect()
        } else {
            file_changes
        };

        // Calculate total size
        let total_size = limited_changes
            .iter()
            .map(|change| change.size_bytes)
            .sum::<u64>();

        // CRITICAL: Capture complete file inventory for proper restoration
        let file_inventory = self.capture_complete_file_inventory()?;

        // Create checkpoint
        let checkpoint_id = Uuid::new_v4();
        let description = options
            .description
            .unwrap_or_else(|| self.generate_ai_description(&limited_changes));

        let checkpoint = Checkpoint {
            id: checkpoint_id,
            session_id: self.current_session_id,
            description,
            created_at: Utc::now(),
            file_changes: limited_changes.clone(),
            file_inventory, // Complete snapshot of all files
            files_affected: limited_changes.len(),
            size_bytes: total_size,
            tags: options.tags,
            metadata: options.metadata,
        };

        // Store checkpoint data
        self.storage.store_checkpoint(&checkpoint)?;

        // Store checkpoint metadata in database
        self.database.create_checkpoint(&checkpoint)?;

        // Record performance metrics
        let duration = start_time.elapsed().as_secs_f64() * 1000.0;
        {
            let mut perf = self.performance_tracker.write();
            perf.checkpoint_creation_times.push(duration);
        }

        log::info!(
            "Created agent checkpoint {} with {} changes in {:.2}ms",
            checkpoint_id,
            limited_changes.len(),
            duration
        );

        Ok(checkpoint_id)
    }

    /// Check if there are pending changes from AI activity
    pub fn has_ai_changes(&self) -> bool {
        if self.use_changeset_tracking {
            self.changeset_tracker.has_pending_changes()
        } else {
            false
        }
    }

    /// Get statistics about the changeset tracking system
    pub fn get_changeset_stats(&self) -> Option<serde_json::Value> {
        if self.use_changeset_tracking {
            let stats = self.changeset_tracker.get_stats();
            Some(serde_json::json!({
                "files_tracked": stats.files_tracked,
                "changes_detected": stats.changes_detected,
                "memory_usage_bytes": stats.memory_usage_bytes,
                "last_scan_duration_ms": stats.last_scan_duration_ms
            }))
        } else {
            None
        }
    }

    /// Generate a description for AI-generated changes
    fn generate_ai_description(&self, file_changes: &[FileChange]) -> String {
        let created_count = file_changes
            .iter()
            .filter(|c| c.change_type == ChangeType::Created)
            .count();
        let modified_count = file_changes
            .iter()
            .filter(|c| c.change_type == ChangeType::Modified)
            .count();
        let deleted_count = file_changes
            .iter()
            .filter(|c| c.change_type == ChangeType::Deleted)
            .count();

        let mut parts = Vec::new();
        if created_count > 0 {
            parts.push(format!("{} created", created_count));
        }
        if modified_count > 0 {
            parts.push(format!("{} modified", modified_count));
        }
        if deleted_count > 0 {
            parts.push(format!("{} deleted", deleted_count));
        }

        if parts.is_empty() {
            "AI agent checkpoint".to_string()
        } else {
            format!("AI: {} files {}", file_changes.len(), parts.join(", "))
        }
    }

    /// Traditional checkpoint creation (fallback)
    fn create_checkpoint_traditional(&self, options: CheckpointOptions) -> Result<CheckpointId> {
        let start_time = Instant::now();

        // Detect file changes using traditional method
        let file_changes = {
            let mut tracker = self.file_tracker.write();
            tracker.detect_changes()?
        };

        if file_changes.is_empty() && options.description.is_none() {
            // No changes to checkpoint
            return Err(CheckpointError::validation(
                "No changes detected for checkpoint",
            ));
        }

        // Apply file limits
        let limited_changes = if let Some(max_files) = options.max_files {
            file_changes.into_iter().take(max_files).collect()
        } else {
            file_changes
        };

        // Calculate total size
        let total_size = limited_changes
            .iter()
            .map(|change| change.size_bytes)
            .sum::<u64>();

        // CRITICAL: Capture complete file inventory for proper restoration
        let file_inventory = self.capture_complete_file_inventory()?;

        // Create checkpoint
        let checkpoint_id = Uuid::new_v4();
        let description = options
            .description
            .unwrap_or_else(|| self.generate_auto_description(&limited_changes));

        let checkpoint = Checkpoint {
            id: checkpoint_id,
            session_id: self.current_session_id,
            description,
            created_at: Utc::now(),
            file_changes: limited_changes.clone(),
            file_inventory, // Complete snapshot of all files
            files_affected: limited_changes.len(),
            size_bytes: total_size,
            tags: options.tags,
            metadata: options.metadata,
        };

        // Store checkpoint data
        self.storage.store_checkpoint(&checkpoint)?;

        // Store checkpoint metadata in database
        self.database.create_checkpoint(&checkpoint)?;

        // Record performance metrics
        let duration = start_time.elapsed().as_secs_f64() * 1000.0;
        {
            let mut perf = self.performance_tracker.write();
            perf.checkpoint_creation_times.push(duration);
        }

        log::info!(
            "Created traditional checkpoint {} with {} changes in {:.2}ms",
            checkpoint_id,
            limited_changes.len(),
            duration
        );

        Ok(checkpoint_id)
    }

    /// Generate automatic description for traditional checkpoints
    fn generate_auto_description(&self, file_changes: &[FileChange]) -> String {
        let created_count = file_changes
            .iter()
            .filter(|c| c.change_type == ChangeType::Created)
            .count();
        let modified_count = file_changes
            .iter()
            .filter(|c| c.change_type == ChangeType::Modified)
            .count();
        let deleted_count = file_changes
            .iter()
            .filter(|c| c.change_type == ChangeType::Deleted)
            .count();

        let mut parts = Vec::new();
        if created_count > 0 {
            parts.push(format!("{} created", created_count));
        }
        if modified_count > 0 {
            parts.push(format!("{} modified", modified_count));
        }
        if deleted_count > 0 {
            parts.push(format!("{} deleted", deleted_count));
        }

        if parts.is_empty() {
            "Automatic checkpoint".to_string()
        } else {
            format!("{} files {}", file_changes.len(), parts.join(", "))
        }
    }

    /// Capture complete file inventory for restoration
    /// This scans the entire workspace and returns a list of ALL tracked files
    fn capture_complete_file_inventory(&self) -> Result<Vec<PathBuf>> {
        use walkdir::WalkDir;

        let mut inventory = Vec::new();
        let workspace_path = &self.workspace_path;

        // Walk the workspace directory
        for entry in WalkDir::new(workspace_path)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                // Skip common ignored directories
                let file_name = e.file_name().to_string_lossy();
                !file_name.starts_with('.')
                    && file_name != "node_modules"
                    && file_name != "target"
                    && file_name != "dist"
                    && file_name != "build"
                    && file_name != "out"
            })
        {
            let entry = entry?;
            if entry.file_type().is_file() {
                if let Ok(relative_path) = entry.path().strip_prefix(workspace_path) {
                    // Only include text files that we typically track
                    if self.should_track_file(relative_path) {
                        inventory.push(relative_path.to_path_buf());
                    }
                }
            }
        }

        log::info!("Captured file inventory: {} files", inventory.len());
        Ok(inventory)
    }

    /// Check if a file should be tracked based on extension
    fn should_track_file(&self, path: &std::path::Path) -> bool {
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            matches!(
                ext_str.as_str(),
                "js" | "jsx"
                    | "ts"
                    | "tsx"
                    | "py"
                    | "java"
                    | "cpp"
                    | "c"
                    | "cs"
                    | "go"
                    | "rs"
                    | "php"
                    | "rb"
                    | "swift"
                    | "kt"
                    | "html"
                    | "css"
                    | "scss"
                    | "json"
                    | "yaml"
                    | "yml"
                    | "md"
                    | "txt"
                    | "sh"
                    | "bat"
                    | "ps1"
                    | "xml"
                    | "toml"
                    | "ini"
                    | "cfg"
                    | "conf"
                    | "sql"
            )
        } else {
            false
        }
    }

    /// Initialize a session for a specific workspace
    pub fn init_session(&mut self, workspace_path: PathBuf) -> Result<SessionId> {
        self.workspace_path = workspace_path.clone();

        // Create new session
        let session = Session {
            id: self.current_session_id,
            workspace_path: workspace_path.clone(),
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            checkpoint_count: 0,
            total_size_bytes: 0,
            metadata: std::collections::HashMap::new(),
        };

        // Store session in database
        self.database.create_session(&session)?;

        // Update file tracker workspace
        {
            let mut tracker = self.file_tracker.write();
            *tracker = FileTracker::new(self.config.clone(), workspace_path);
        }

        // Initialize restoration system with new workspace
        self.restoration = Arc::new(CheckpointRestoration::new(
            self.config.clone(),
            (*self.storage).clone(),
            self.workspace_path.clone(),
        ));

        Ok(self.current_session_id)
    }

    /// Create a checkpoint with the given options
    pub fn create_checkpoint(&self, options: CheckpointOptions) -> Result<CheckpointId> {
        let start_time = Instant::now();

        // Detect file changes
        let file_changes = {
            let mut tracker = self.file_tracker.write();
            tracker.detect_changes()?
        };

        if file_changes.is_empty() && options.description.is_none() {
            // No changes to checkpoint
            return Err(CheckpointError::validation(
                "No changes detected for checkpoint",
            ));
        }

        // Apply file limits
        let limited_changes = if let Some(max_files) = options.max_files {
            file_changes.into_iter().take(max_files).collect()
        } else {
            file_changes
        };

        // Calculate total size
        let total_size = limited_changes
            .iter()
            .map(|change| change.size_bytes)
            .sum::<u64>();

        // CRITICAL: Capture complete file inventory for proper restoration
        let file_inventory = self.capture_complete_file_inventory()?;

        // Create checkpoint
        let checkpoint_id = Uuid::new_v4();
        let description = options
            .description
            .unwrap_or_else(|| self.generate_auto_description(&limited_changes));

        let checkpoint = Checkpoint {
            id: checkpoint_id,
            session_id: self.current_session_id,
            description,
            created_at: Utc::now(),
            file_changes: limited_changes.clone(),
            file_inventory, // Complete snapshot of all files
            files_affected: limited_changes.len(),
            size_bytes: total_size,
            tags: options.tags,
            metadata: options.metadata,
        };

        // Store checkpoint data
        self.storage.store_checkpoint(&checkpoint)?;

        // Store checkpoint metadata in database
        self.database.create_checkpoint(&checkpoint)?;

        // Record performance metrics
        let duration = start_time.elapsed().as_secs_f64() * 1000.0;
        {
            let mut perf = self.performance_tracker.write();
            perf.checkpoint_creation_times.push(duration);
        }

        self.database.record_performance_metric(
            "checkpoint_creation",
            duration,
            Some(total_size),
            std::collections::HashMap::from([(
                "files_affected".to_string(),
                limited_changes.len().to_string(),
            )]),
        )?;

        Ok(checkpoint_id)
    }

    /// Create a simple checkpoint with minimal options
    pub fn create_simple_checkpoint(&self, description: &str) -> Result<CheckpointId> {
        let options = CheckpointOptions {
            description: Some(description.to_string()),
            ..Default::default()
        };

        self.create_checkpoint(options)
    }

    /// Restore a checkpoint
    pub fn restore_checkpoint(
        &self,
        checkpoint_id: &CheckpointId,
        options: RestoreOptions,
    ) -> Result<crate::restoration::RestoreResult> {
        let start_time = Instant::now();

        // Perform restoration
        let result = self
            .restoration
            .restore_checkpoint(checkpoint_id, &options)?;

        // Record performance metrics
        let duration = start_time.elapsed().as_secs_f64() * 1000.0;
        {
            let mut perf = self.performance_tracker.write();
            perf.restoration_times.push(duration);
        }

        self.database.record_performance_metric(
            "checkpoint_restoration",
            duration,
            None,
            std::collections::HashMap::from([
                (
                    "files_restored".to_string(),
                    result.restored_files.len().to_string(),
                ),
                ("conflicts".to_string(), result.conflicts.len().to_string()),
            ]),
        )?;

        // Log audit entry
        self.database.log_audit_action(
            AuditAction::CheckpointRestored,
            "user",
            &checkpoint_id.to_string(),
            std::collections::HashMap::from([
                ("success".to_string(), result.success.to_string()),
                (
                    "files_restored".to_string(),
                    result.restored_files.len().to_string(),
                ),
            ]),
            if result.success {
                AuditResult::Success
            } else {
                AuditResult::Failed
            },
        )?;

        Ok(result)
    }

    /// Get a checkpoint by ID
    pub fn get_checkpoint(&self, checkpoint_id: &CheckpointId) -> Result<Option<Checkpoint>> {
        self.database.get_checkpoint(checkpoint_id)
    }

    /// List checkpoints for the current session
    pub fn list_checkpoints(&self, limit: Option<usize>) -> Result<Vec<Checkpoint>> {
        self.database
            .list_checkpoints(&self.current_session_id, limit)
    }

    /// Delete a checkpoint
    pub fn delete_checkpoint(&self, checkpoint_id: &CheckpointId) -> Result<()> {
        // Delete from storage
        self.storage.delete_checkpoint(checkpoint_id)?;

        // Delete from database
        self.database.delete_checkpoint(checkpoint_id)?;

        Ok(())
    }

    /// Get checkpoint statistics
    pub fn get_statistics(&self) -> Result<CheckpointStats> {
        self.database.get_stats()
    }

    /// Clean up old checkpoints based on retention policy
    pub fn cleanup_old_checkpoints(&self) -> Result<usize> {
        let deleted_count = self
            .database
            .cleanup_old_checkpoints(self.config.retention_days)?;

        // Also cleanup unused content in storage
        let _freed_bytes = self.storage.cleanup_unused_content()?;

        Ok(deleted_count)
    }

    /// Create a backup of checkpoints
    pub fn create_backup(
        &self,
        checkpoint_ids: &[CheckpointId],
        backup_path: &std::path::Path,
    ) -> Result<BackupInfo> {
        self.storage.create_backup(checkpoint_ids, backup_path)
    }

    /// Restore from backup
    pub fn restore_backup(&self, backup_path: &std::path::Path) -> Result<Vec<CheckpointId>> {
        self.storage.restore_backup(backup_path)
    }

    /// Get performance metrics
    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        let perf = self.performance_tracker.read();

        let avg_creation_time = if perf.checkpoint_creation_times.is_empty() {
            0.0
        } else {
            perf.checkpoint_creation_times.iter().sum::<f64>()
                / perf.checkpoint_creation_times.len() as f64
        };

        let avg_restoration_time = if perf.restoration_times.is_empty() {
            0.0
        } else {
            perf.restoration_times.iter().sum::<f64>() / perf.restoration_times.len() as f64
        };

        PerformanceMetrics {
            avg_creation_time_ms: avg_creation_time,
            avg_restoration_time_ms: avg_restoration_time,
            db_queries_per_second: 1000.0, // Placeholder
            file_io_mbps: 100.0,           // Placeholder
            memory_usage_mb: 50.0,         // Placeholder
        }
    }

    /// Get current session ID
    pub fn current_session_id(&self) -> SessionId {
        self.current_session_id
    }

    /// Get workspace path
    pub fn workspace_path(&self) -> &PathBuf {
        &self.workspace_path
    }

    /// Flush all pending operations
    pub fn flush(&self) -> Result<()> {
        self.storage.flush()?;
        Ok(())
    }
}
