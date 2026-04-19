//! Restoration system for checkpoint recovery
//!
//! This module handles the restoration of files from checkpoints with support for
//! conflict detection, selective restoration, and backup creation.

use crate::config::CheckpointConfig;
use crate::error::{CheckpointError, Result};
use crate::storage::CheckpointStorage;
use crate::types::*;

use chrono::Utc;
use std::fs::{self, Permissions};
use std::path::{Path, PathBuf};

/// Restoration engine for checkpoint recovery
pub struct CheckpointRestoration {
    storage: CheckpointStorage,
    workspace_path: PathBuf,
}

/// Result of a restoration operation
#[derive(Debug, Clone)]
pub struct RestoreResult {
    pub success: bool,
    pub restored_files: Vec<PathBuf>,
    pub failed_files: Vec<(PathBuf, String)>,
    pub created_files: Vec<PathBuf>,
    pub modified_files: Vec<PathBuf>,
    pub deleted_files: Vec<PathBuf>,
    pub backup_checkpoint_id: Option<CheckpointId>,
    pub conflicts: Vec<ConflictInfo>,
}

/// Information about a restoration conflict
#[derive(Debug, Clone)]
pub struct ConflictInfo {
    pub file_path: PathBuf,
    pub conflict_type: ConflictType,
    pub current_content: Option<String>,
    pub checkpoint_content: Option<String>,
    pub resolution: ConflictResolution,
}

/// Types of conflicts during restoration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictType {
    /// File was modified since checkpoint
    FileModified,
    /// File was deleted since checkpoint
    FileDeleted,
    /// File was created since checkpoint
    FileCreated,
    /// Permission conflict
    PermissionDenied,
    /// File is locked or in use
    FileLocked,
}

impl CheckpointRestoration {
    /// Create a new restoration engine
    pub fn new(
        _config: CheckpointConfig,
        storage: CheckpointStorage,
        workspace_path: PathBuf,
    ) -> Self {
        Self {
            storage,
            workspace_path,
        }
    }

    /// Restore a checkpoint with the given options
    pub fn restore_checkpoint(
        &self,
        checkpoint_id: &CheckpointId,
        options: &RestoreOptions,
    ) -> Result<RestoreResult> {
        // Load the checkpoint
        let checkpoint = self
            .storage
            .load_checkpoint(checkpoint_id)?
            .ok_or_else(|| CheckpointError::CheckpointNotFound {
                id: checkpoint_id.to_string(),
            })?;

        let mut result = RestoreResult {
            success: true,
            restored_files: Vec::new(),
            failed_files: Vec::new(),
            created_files: Vec::new(),
            modified_files: Vec::new(),
            deleted_files: Vec::new(),
            backup_checkpoint_id: None,
            conflicts: Vec::new(),
        };

        // Create backup if requested
        if options.create_backup {
            match self.create_pre_restore_backup() {
                Ok(backup_id) => result.backup_checkpoint_id = Some(backup_id),
                Err(e) => {
                    log::warn!("Failed to create pre-restore backup: {}", e);
                    if options.require_backup {
                        return Err(CheckpointError::restoration_failed(
                            "Failed to create pre-restore backup (require_backup=true)",
                        ));
                    }
                }
            }
        }

        // Filter files if specific files are requested
        let files_to_restore = if options.include_files.is_empty() {
            checkpoint.file_changes.clone()
        } else {
            checkpoint
                .file_changes
                .iter()
                .filter(|change| {
                    options
                        .include_files
                        .iter()
                        .any(|pattern| self.path_matches_pattern(&change.path, pattern))
                })
                .cloned()
                .collect()
        };

        // Exclude files if specified
        let files_to_restore: Vec<_> = files_to_restore
            .iter()
            .filter(|change| {
                !options
                    .exclude_files
                    .iter()
                    .any(|pattern| self.path_matches_pattern(&change.path, pattern))
            })
            .cloned()
            .collect();

        // Detect conflicts before restoration
        let conflicts = self.detect_conflicts(&files_to_restore, options)?;
        result.conflicts = conflicts.clone();

        // Dry-run mode: return conflicts without modifying any files
        if options.dry_run {
            log::info!(
                "Dry-run complete: detected {} conflicts for {} files",
                conflicts.len(),
                files_to_restore.len()
            );
            return Ok(result);
        }

        // Create a set of conflicted file paths for quick lookup
        let conflicted_paths: std::collections::HashSet<_> =
            conflicts.iter().map(|c| c.file_path.clone()).collect();

        // Handle conflicts based on resolution strategy
        let mut resolved_files = self.resolve_conflicts(conflicts, options)?;

        // Add non-conflicted files from checkpoint to the restoration list
        for file_change in &files_to_restore {
            if !conflicted_paths.contains(&file_change.path) {
                resolved_files.push(file_change.clone());
            }
        }

        // Perform the actual restoration
        for file_change in &resolved_files {
            match self.restore_file_change(file_change, options) {
                Ok(restore_type) => {
                    result.restored_files.push(file_change.path.clone());
                    match restore_type {
                        FileRestoreType::Created => {
                            result.created_files.push(file_change.path.clone())
                        }
                        FileRestoreType::Modified => {
                            result.modified_files.push(file_change.path.clone())
                        }
                        FileRestoreType::Deleted => {
                            result.deleted_files.push(file_change.path.clone())
                        }
                    }
                }
                Err(e) => {
                    result
                        .failed_files
                        .push((file_change.path.clone(), e.to_string()));
                    result.success = false;
                }
            }
        }

        // CRITICAL: Remove files that don't exist in checkpoint inventory
        // This ensures complete workspace restoration to checkpoint state
        self.cleanup_files_not_in_inventory(&checkpoint, &mut result)?;

        // Validate checksums if requested
        if options.validate_checksums {
            self.validate_restored_files(&result.restored_files, &resolved_files)?;
        }

        // Clean up backup after successful, validated restoration
        if result.success {
            if let Some(backup_id) = &result.backup_checkpoint_id {
                log::info!("Restoration succeeded; backup {} retained for safety", backup_id);
            }
        }

        Ok(result)
    }

    /// Create a backup of current state before restoration.
    /// Captures workspace files so the user can roll back if needed.
    fn create_pre_restore_backup(&self) -> Result<CheckpointId> {
        let backup_id = uuid::Uuid::new_v4();
        let backup_dir = self.workspace_path.join(".checkpoints").join("backups").join(backup_id.to_string());

        fs::create_dir_all(&backup_dir).map_err(|e| {
            CheckpointError::generic(format!("Failed to create backup directory: {}", e))
        })?;

        // Walk workspace and back up files (skipping hidden dirs / backup dir itself)
        let mut file_count: usize = 0;
        for entry in walkdir::WalkDir::new(&self.workspace_path)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.') && name != "node_modules" && name != "target"
            })
        {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            if !entry.file_type().is_file() {
                continue;
            }
            let rel = match entry.path().strip_prefix(&self.workspace_path) {
                Ok(r) => r,
                Err(_) => continue,
            };
            let dest = backup_dir.join(rel);
            if let Some(parent) = dest.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if fs::copy(entry.path(), &dest).is_ok() {
                file_count += 1;
            }
        }

        // Verify backup by checking file count
        if file_count == 0 {
            log::warn!("Pre-restore backup {} contains 0 files", backup_id);
        } else {
            log::info!("Pre-restore backup {} created with {} files", backup_id, file_count);
        }

        Ok(backup_id)
    }

    /// Detect conflicts that would occur during restoration
    fn detect_conflicts(
        &self,
        file_changes: &[FileChange],
        options: &RestoreOptions,
    ) -> Result<Vec<ConflictInfo>> {
        let mut conflicts = Vec::new();

        for file_change in file_changes {
            let full_path = self.workspace_path.join(&file_change.path);

            match file_change.change_type {
                ChangeType::Created => {
                    if full_path.exists() {
                        // File exists but checkpoint wants to create it
                        let current_content = self.read_file_safely(&full_path);
                        conflicts.push(ConflictInfo {
                            file_path: file_change.path.clone(),
                            conflict_type: ConflictType::FileCreated,
                            current_content,
                            checkpoint_content: file_change.new_content.clone(),
                            resolution: options.conflict_resolution,
                        });
                    }
                }
                ChangeType::Modified => {
                    if !full_path.exists() {
                        // File doesn't exist but checkpoint wants to modify it
                        conflicts.push(ConflictInfo {
                            file_path: file_change.path.clone(),
                            conflict_type: ConflictType::FileDeleted,
                            current_content: None,
                            checkpoint_content: file_change.new_content.clone(),
                            resolution: options.conflict_resolution,
                        });
                    } else {
                        // Check if file was modified since checkpoint
                        if let Some(current_content) = self.read_file_safely(&full_path) {
                            let current_hash = self.calculate_content_hash(&current_content);
                            if current_hash != file_change.content_hash {
                                conflicts.push(ConflictInfo {
                                    file_path: file_change.path.clone(),
                                    conflict_type: ConflictType::FileModified,
                                    current_content: Some(current_content),
                                    checkpoint_content: file_change.new_content.clone(),
                                    resolution: options.conflict_resolution,
                                });
                            }
                        }
                    }
                }
                ChangeType::Deleted => {
                    if full_path.exists() {
                        // File exists but checkpoint wants to delete it
                        let current_content = self.read_file_safely(&full_path);
                        conflicts.push(ConflictInfo {
                            file_path: file_change.path.clone(),
                            conflict_type: ConflictType::FileModified,
                            current_content,
                            checkpoint_content: None,
                            resolution: options.conflict_resolution,
                        });
                    }
                }
                ChangeType::Renamed { .. } | ChangeType::Moved { .. } => {
                    // Handle move/rename conflicts
                    if full_path.exists() {
                        let current_content = self.read_file_safely(&full_path);
                        conflicts.push(ConflictInfo {
                            file_path: file_change.path.clone(),
                            conflict_type: ConflictType::FileModified,
                            current_content,
                            checkpoint_content: file_change.new_content.clone(),
                            resolution: options.conflict_resolution,
                        });
                    }
                }
            }
        }

        Ok(conflicts)
    }

    /// Resolve conflicts based on the resolution strategy
    fn resolve_conflicts(
        &self,
        conflicts: Vec<ConflictInfo>,
        options: &RestoreOptions,
    ) -> Result<Vec<FileChange>> {
        let mut resolved_files = Vec::new();

        for conflict in conflicts {
            // Check for per-file resolution override first
            let resolution = options
                .per_file_resolutions
                .get(&conflict.file_path)
                .copied()
                .unwrap_or(conflict.resolution);

            // If this is a dry-run, skip actual resolution — conflicts are
            // already recorded in the result for the caller to inspect.
            if options.dry_run {
                log::info!(
                    "Dry-run: would resolve conflict for {} with {:?}",
                    conflict.file_path.display(),
                    resolution
                );
                continue;
            }

            match resolution {
                ConflictResolution::Skip => {
                    log::info!("Skipping conflicted file: {}", conflict.file_path.display());
                    // Don't add to resolved_files
                }
                ConflictResolution::Overwrite => {
                    log::info!(
                        "Overwriting conflicted file: {}",
                        conflict.file_path.display()
                    );

                    // Determine the appropriate change type based on the conflict
                    let change_type = if conflict.checkpoint_content.is_none() {
                        // Checkpoint wants to delete this file
                        ChangeType::Deleted
                    } else if conflict.current_content.is_none() {
                        // File doesn't exist, checkpoint wants to create it
                        ChangeType::Created
                    } else {
                        // File exists and checkpoint wants to modify it
                        ChangeType::Modified
                    };

                    // Create a file change for restoration
                    let size_bytes = conflict
                        .checkpoint_content
                        .as_ref()
                        .map(|c| c.len() as u64)
                        .unwrap_or(0);
                    let content_hash = conflict
                        .checkpoint_content
                        .as_ref()
                        .map(|c| self.calculate_content_hash(c))
                        .unwrap_or_default();
                    let file_change = FileChange {
                        path: conflict.file_path,
                        change_type,
                        original_content: conflict.current_content,
                        new_content: conflict.checkpoint_content,
                        size_bytes,
                        content_hash,
                        permissions: None,
                        modified_at: Utc::now(),
                        encoding: FileEncoding::Utf8,
                        compressed: false,
                    };
                    resolved_files.push(file_change);
                }
                ConflictResolution::Backup => {
                    log::info!(
                        "Creating backup for conflicted file: {}",
                        conflict.file_path.display()
                    );
                    // Create backup copy
                    self.create_conflict_backup(&conflict)?;

                    // Determine the appropriate change type based on the conflict
                    let change_type = if conflict.checkpoint_content.is_none() {
                        // Checkpoint wants to delete this file
                        ChangeType::Deleted
                    } else if conflict.current_content.is_none() {
                        // File doesn't exist, checkpoint wants to create it
                        ChangeType::Created
                    } else {
                        // File exists and checkpoint wants to modify it
                        ChangeType::Modified
                    };

                    // Create a file change for restoration
                    let size_bytes = conflict
                        .checkpoint_content
                        .as_ref()
                        .map(|c| c.len() as u64)
                        .unwrap_or(0);
                    let content_hash = conflict
                        .checkpoint_content
                        .as_ref()
                        .map(|c| self.calculate_content_hash(c))
                        .unwrap_or_default();
                    let file_change = FileChange {
                        path: conflict.file_path,
                        change_type,
                        original_content: conflict.current_content,
                        new_content: conflict.checkpoint_content,
                        size_bytes,
                        content_hash,
                        permissions: None,
                        modified_at: Utc::now(),
                        encoding: FileEncoding::Utf8,
                        compressed: false,
                    };
                    resolved_files.push(file_change);
                }
                ConflictResolution::Prompt => {
                    // Interactive prompting is handled by the caller (e.g. VSCode extension).
                    // In the Rust-only context, fall back to the configured prompt_fallback strategy.
                    let fallback = options.prompt_fallback;
                    log::info!(
                        "Prompt resolution requested for {} — using fallback strategy: {:?}",
                        conflict.file_path.display(),
                        fallback
                    );

                    match fallback {
                        ConflictResolution::Skip => {
                            log::info!("Fallback: skipping conflicted file: {}", conflict.file_path.display());
                        }
                        ConflictResolution::Overwrite | ConflictResolution::Prompt => {
                            // Treat Prompt-fallback-to-Prompt as Overwrite to avoid infinite loop
                            let change_type = if conflict.checkpoint_content.is_none() {
                                ChangeType::Deleted
                            } else if conflict.current_content.is_none() {
                                ChangeType::Created
                            } else {
                                ChangeType::Modified
                            };
                            let size_bytes = conflict.checkpoint_content.as_ref().map(|c| c.len() as u64).unwrap_or(0);
                            let content_hash = conflict.checkpoint_content.as_ref().map(|c| self.calculate_content_hash(c)).unwrap_or_default();
                            resolved_files.push(FileChange {
                                path: conflict.file_path,
                                change_type,
                                original_content: conflict.current_content,
                                new_content: conflict.checkpoint_content,
                                size_bytes,
                                content_hash,
                                permissions: None,
                                modified_at: Utc::now(),
                                encoding: FileEncoding::Utf8,
                                compressed: false,
                            });
                        }
                        ConflictResolution::Backup => {
                            self.create_conflict_backup(&conflict)?;
                            let change_type = if conflict.checkpoint_content.is_none() {
                                ChangeType::Deleted
                            } else if conflict.current_content.is_none() {
                                ChangeType::Created
                            } else {
                                ChangeType::Modified
                            };
                            let size_bytes = conflict.checkpoint_content.as_ref().map(|c| c.len() as u64).unwrap_or(0);
                            let content_hash = conflict.checkpoint_content.as_ref().map(|c| self.calculate_content_hash(c)).unwrap_or_default();
                            resolved_files.push(FileChange {
                                path: conflict.file_path,
                                change_type,
                                original_content: conflict.current_content,
                                new_content: conflict.checkpoint_content,
                                size_bytes,
                                content_hash,
                                permissions: None,
                                modified_at: Utc::now(),
                                encoding: FileEncoding::Utf8,
                                compressed: false,
                            });
                        }
                    }
                }
            }
        }

        Ok(resolved_files)
    }

    /// Create a backup copy of a conflicted file
    fn create_conflict_backup(&self, conflict: &ConflictInfo) -> Result<()> {
        if let Some(ref current_content) = conflict.current_content {
            let backup_path = self.get_backup_path(&conflict.file_path);

            // Ensure backup directory exists
            if let Some(parent) = backup_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(&backup_path, current_content)?;
            log::info!("Created conflict backup: {}", backup_path.display());
        }

        Ok(())
    }

    /// Get backup path for a conflicted file
    fn get_backup_path(&self, file_path: &Path) -> PathBuf {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!(
            "{}.backup.{}",
            file_path.file_name().unwrap_or_default().to_string_lossy(),
            timestamp
        );

        if let Some(parent) = file_path.parent() {
            self.workspace_path.join(parent).join(backup_name)
        } else {
            self.workspace_path.join(backup_name)
        }
    }

    /// Restore a single file change
    fn restore_file_change(
        &self,
        file_change: &FileChange,
        options: &RestoreOptions,
    ) -> Result<FileRestoreType> {
        let full_path = self.workspace_path.join(&file_change.path);

        match file_change.change_type {
            ChangeType::Created => {
                if let Some(ref content) = file_change.new_content {
                    // Ensure parent directory exists
                    if let Some(parent) = full_path.parent() {
                        fs::create_dir_all(parent)?;
                    }

                    fs::write(&full_path, content)?;

                    // Restore permissions if requested
                    if options.restore_permissions {
                        if let Some(permissions) = file_change.permissions {
                            self.set_file_permissions(&full_path, permissions)?;
                        }
                    }

                    // Restore timestamps if requested
                    if options.restore_timestamps {
                        self.set_file_timestamp(&full_path, &file_change.modified_at)?;
                    }

                    Ok(FileRestoreType::Created)
                } else {
                    Err(CheckpointError::restoration_failed(
                        "No content for created file",
                    ))
                }
            }
            ChangeType::Modified => {
                if let Some(ref content) = file_change.new_content {
                    fs::write(&full_path, content)?;

                    // Restore permissions if requested
                    if options.restore_permissions {
                        if let Some(permissions) = file_change.permissions {
                            self.set_file_permissions(&full_path, permissions)?;
                        }
                    }

                    // Restore timestamps if requested
                    if options.restore_timestamps {
                        self.set_file_timestamp(&full_path, &file_change.modified_at)?;
                    }

                    Ok(FileRestoreType::Modified)
                } else {
                    Err(CheckpointError::restoration_failed(
                        "No content for modified file",
                    ))
                }
            }
            ChangeType::Deleted => {
                if full_path.exists() {
                    fs::remove_file(&full_path)?;

                    // Try to remove empty parent directories
                    if let Some(parent) = full_path.parent() {
                        self.remove_empty_directories(parent)?;
                    }

                    Ok(FileRestoreType::Deleted)
                } else {
                    // File already doesn't exist, that's fine
                    Ok(FileRestoreType::Deleted)
                }
            }
            ChangeType::Renamed { ref from } => {
                let from_path = self.workspace_path.join(from);
                if from_path.exists() {
                    // Ensure target directory exists
                    if let Some(parent) = full_path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::rename(&from_path, &full_path)?;
                    Ok(FileRestoreType::Modified)
                } else {
                    Err(CheckpointError::restoration_failed(format!(
                        "Source file for rename not found: {}",
                        from_path.display()
                    )))
                }
            }
            ChangeType::Moved { ref from } => {
                let from_path = self.workspace_path.join(from);
                if from_path.exists() {
                    // Ensure target directory exists
                    if let Some(parent) = full_path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::rename(&from_path, &full_path)?;
                    Ok(FileRestoreType::Modified)
                } else {
                    Err(CheckpointError::restoration_failed(format!(
                        "Source file for move not found: {}",
                        from_path.display()
                    )))
                }
            }
        }
    }

    /// Set file permissions (Unix only)
    fn set_file_permissions(&self, file_path: &Path, permissions: u32) -> Result<()> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = Permissions::from_mode(permissions);
            fs::set_permissions(file_path, perms)?;
        }

        #[cfg(not(unix))]
        {
            // On Windows, we can't set Unix-style permissions
            log::warn!(
                "Cannot set Unix permissions on Windows: {}",
                file_path.display()
            );
        }

        Ok(())
    }

    /// Set file timestamp
    fn set_file_timestamp(
        &self,
        file_path: &Path,
        timestamp: &chrono::DateTime<Utc>,
    ) -> Result<()> {
        use std::fs::FileTimes;
        use std::time::{Duration, UNIX_EPOCH};

        let ts_secs = timestamp.timestamp() as u64;
        let system_time = UNIX_EPOCH + Duration::from_secs(ts_secs);

        let file = fs::File::options().write(true).open(file_path)?;
        let times = FileTimes::new()
            .set_accessed(system_time)
            .set_modified(system_time);

        file.set_times(times).map_err(|e| {
            log::warn!("Could not set timestamp for {}: {}", file_path.display(), e);
            CheckpointError::generic(format!("Failed to set timestamp: {}", e))
        })?;

        Ok(())
    }

    /// Validate that restored files match their expected checksums
    fn validate_restored_files(
        &self,
        restored_files: &[PathBuf],
        expected_changes: &[FileChange],
    ) -> Result<()> {
        for file_change in expected_changes {
            if restored_files.contains(&file_change.path) {
                let full_path = self.workspace_path.join(&file_change.path);

                if full_path.exists() {
                    if let Some(content) = self.read_file_safely(&full_path) {
                        let actual_hash = self.calculate_content_hash(&content);
                        if actual_hash != file_change.content_hash {
                            return Err(CheckpointError::validation(format!(
                                "Checksum mismatch for restored file: {}",
                                file_change.path.display()
                            )));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Safely read file content, returning None on error
    fn read_file_safely(&self, path: &Path) -> Option<String> {
        fs::read_to_string(path).ok()
    }

    /// Calculate content hash
    fn calculate_content_hash(&self, content: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Check if a path matches a pattern
    fn path_matches_pattern(&self, path: &Path, pattern: &Path) -> bool {
        // Simple pattern matching - in production, you'd want more sophisticated glob matching
        path == pattern || path.starts_with(pattern)
    }

    /// Remove empty directories recursively up to the workspace root
    fn remove_empty_directories(&self, dir_path: &Path) -> Result<()> {
        // Don't remove anything outside the workspace
        if !dir_path.starts_with(&self.workspace_path) {
            return Ok(());
        }

        // Don't remove the workspace root itself
        if dir_path == self.workspace_path {
            return Ok(());
        }

        // Check if directory exists and is empty
        match fs::read_dir(dir_path) {
            Ok(mut entries) => {
                if entries.next().is_none() {
                    // Directory is empty, try to remove it
                    match fs::remove_dir(dir_path) {
                        Ok(_) => {
                            log::info!("Removed empty directory: {}", dir_path.display());

                            // Recursively try to remove parent directories
                            if let Some(parent) = dir_path.parent() {
                                let _ = self.remove_empty_directories(parent);
                            }
                        }
                        Err(e) => {
                            log::debug!("Could not remove directory {}: {}", dir_path.display(), e);
                        }
                    }
                }
            }
            Err(_) => {
                // Directory doesn't exist or can't be read, that's fine
            }
        }

        Ok(())
    }

    /// Remove files that exist in workspace but not in checkpoint inventory
    /// This is CRITICAL for proper restoration to exact checkpoint state
    fn cleanup_files_not_in_inventory(
        &self,
        checkpoint: &crate::types::Checkpoint,
        result: &mut RestoreResult,
    ) -> Result<()> {
        use std::collections::HashSet;
        use walkdir::WalkDir;

        // Create a set of files that should exist (from checkpoint inventory)
        let inventory_set: HashSet<PathBuf> = checkpoint.file_inventory.iter().cloned().collect();

        log::info!(
            "Checkpoint inventory contains {} files",
            inventory_set.len()
        );

        // Walk the workspace and find files that shouldn't exist
        for entry in WalkDir::new(&self.workspace_path)
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
                if let Ok(relative_path) = entry.path().strip_prefix(&self.workspace_path) {
                    // If file is not in inventory, it should be removed
                    if !inventory_set.contains(relative_path) {
                        let full_path = self.workspace_path.join(relative_path);
                        match fs::remove_file(&full_path) {
                            Ok(_) => {
                                log::info!(
                                    "Removed extra file not in checkpoint: {}",
                                    relative_path.display()
                                );
                                result.deleted_files.push(relative_path.to_path_buf());

                                // Try to remove empty parent directories
                                if let Some(parent) = full_path.parent() {
                                    let _ = self.remove_empty_directories(parent);
                                }
                            }
                            Err(e) => {
                                log::warn!(
                                    "Failed to remove extra file {}: {}",
                                    relative_path.display(),
                                    e
                                );
                                result
                                    .failed_files
                                    .push((relative_path.to_path_buf(), e.to_string()));
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

/// Type of file restoration operation
#[derive(Debug, Clone, PartialEq, Eq)]
enum FileRestoreType {
    Created,
    Modified,
    Deleted,
}

impl Default for RestoreOptions {
    fn default() -> Self {
        Self {
            create_backup: true,
            require_backup: true,
            restore_permissions: true,
            restore_timestamps: true,
            include_files: Vec::new(),
            exclude_files: Vec::new(),
            conflict_resolution: ConflictResolution::Prompt,
            validate_checksums: true,
            per_file_resolutions: std::collections::HashMap::new(),
            prompt_fallback: ConflictResolution::Backup,
            dry_run: false,
        }
    }
}
