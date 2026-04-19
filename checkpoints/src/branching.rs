//! Phase 8.2: Checkpoint Branching & Merging
//!
//! Provides branch management for checkpoints, allowing users to create
//! divergent timelines from any checkpoint and merge them back together
//! with conflict detection and resolution.

use crate::db::CheckpointDatabase;
use crate::error::{CheckpointError, Result};
use crate::storage::CheckpointStorage;
use crate::types::*;

use chrono::Utc;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

/// Manages checkpoint branches and merging
pub struct BranchManager {
    database: Arc<CheckpointDatabase>,
    storage: Arc<CheckpointStorage>,
    session_id: SessionId,
}

impl BranchManager {
    /// Create a new branch manager
    pub fn new(
        database: Arc<CheckpointDatabase>,
        storage: Arc<CheckpointStorage>,
        session_id: SessionId,
    ) -> Self {
        Self {
            database,
            storage,
            session_id,
        }
    }

    /// Create a new branch from a specific checkpoint
    pub fn create_branch(
        &self,
        name: &str,
        base_checkpoint_id: &CheckpointId,
        description: &str,
    ) -> Result<Branch> {
        // Verify the base checkpoint exists
        let _base = self
            .database
            .get_checkpoint(base_checkpoint_id)?
            .ok_or_else(|| {
                CheckpointError::checkpoint_not_found(base_checkpoint_id.to_string())
            })?;

        let branch = Branch {
            id: format!("branch-{}", Uuid::new_v4()),
            name: name.to_string(),
            session_id: self.session_id,
            base_checkpoint_id: *base_checkpoint_id,
            head_checkpoint_id: Some(*base_checkpoint_id),
            created_at: Utc::now(),
            description: description.to_string(),
            is_default: false,
            metadata: HashMap::new(),
        };

        self.database.create_branch(&branch)?;

        log::info!(
            "Created branch '{}' ({}) from checkpoint {}",
            branch.name,
            branch.id,
            base_checkpoint_id
        );

        Ok(branch)
    }

    /// List all branches for the current session
    pub fn list_branches(&self) -> Result<Vec<Branch>> {
        self.database.list_branches(&self.session_id)
    }

    /// Get a branch by ID
    pub fn get_branch(&self, branch_id: &str) -> Result<Option<Branch>> {
        self.database.get_branch(branch_id)
    }

    /// Delete a branch
    pub fn delete_branch(&self, branch_id: &str) -> Result<()> {
        let branch = self.database.get_branch(branch_id)?;
        if let Some(b) = branch {
            if b.is_default {
                return Err(CheckpointError::validation(
                    "Cannot delete the default branch",
                ));
            }
        }
        self.database.delete_branch(branch_id)
    }

    /// Update the head checkpoint of a branch
    pub fn update_branch_head(
        &self,
        branch_id: &str,
        checkpoint_id: &CheckpointId,
    ) -> Result<()> {
        self.database.update_branch_head(branch_id, checkpoint_id)
    }

    /// List checkpoints on a specific branch
    pub fn list_branch_checkpoints(
        &self,
        branch_id: &str,
        limit: Option<usize>,
    ) -> Result<Vec<Checkpoint>> {
        self.database.list_branch_checkpoints(branch_id, limit)
    }

    /// Merge source branch into target branch
    pub fn merge_branches(
        &self,
        source_branch_id: &str,
        target_branch_id: &str,
        strategy: MergeStrategy,
    ) -> Result<MergeResult> {
        let source_branch = self
            .database
            .get_branch(source_branch_id)?
            .ok_or_else(|| {
                CheckpointError::generic(format!("Source branch not found: {}", source_branch_id))
            })?;

        let target_branch = self
            .database
            .get_branch(target_branch_id)?
            .ok_or_else(|| {
                CheckpointError::generic(format!("Target branch not found: {}", target_branch_id))
            })?;

        // Get head checkpoints
        let source_head_id = source_branch.head_checkpoint_id.ok_or_else(|| {
            CheckpointError::validation("Source branch has no checkpoints")
        })?;
        let target_head_id = target_branch.head_checkpoint_id.ok_or_else(|| {
            CheckpointError::validation("Target branch has no checkpoints")
        })?;

        let source_head = self
            .database
            .get_checkpoint(&source_head_id)?
            .ok_or_else(|| CheckpointError::checkpoint_not_found(source_head_id.to_string()))?;
        let target_head = self
            .database
            .get_checkpoint(&target_head_id)?
            .ok_or_else(|| CheckpointError::checkpoint_not_found(target_head_id.to_string()))?;

        // Find common ancestor
        let base_checkpoint_id = self
            .database
            .find_common_ancestor(source_branch_id, target_branch_id)?;

        let base_checkpoint = if let Some(base_id) = base_checkpoint_id {
            self.database.get_checkpoint(&base_id)?
        } else {
            None
        };

        // Perform 3-way merge
        let (merged_changes, conflicts) = self.three_way_merge(
            base_checkpoint.as_ref(),
            &source_head,
            &target_head,
            strategy,
        );

        if !conflicts.is_empty() && matches!(strategy, MergeStrategy::ThreeWay) {
            // Report conflicts without creating checkpoint
            return Ok(MergeResult {
                success: false,
                merge_checkpoint_id: None,
                merged_files: merged_changes.iter().map(|c| c.path.clone()).collect(),
                conflicts,
                strategy,
            });
        }

        // Create merge checkpoint on target branch
        let merge_checkpoint_id = Uuid::new_v4();
        let merge_description = format!(
            "Merge branch '{}' into '{}'",
            source_branch.name, target_branch.name
        );

        let merge_checkpoint = Checkpoint {
            id: merge_checkpoint_id,
            session_id: self.session_id,
            description: merge_description,
            created_at: Utc::now(),
            file_changes: merged_changes.clone(),
            file_inventory: target_head.file_inventory.clone(),
            files_affected: merged_changes.len(),
            size_bytes: merged_changes.iter().map(|c| c.size_bytes).sum(),
            tags: vec!["merge".to_string()],
            metadata: HashMap::from([
                ("source_branch".to_string(), source_branch.name.clone()),
                ("target_branch".to_string(), target_branch.name.clone()),
                ("merge_strategy".to_string(), format!("{:?}", strategy)),
            ]),
            parent_checkpoint_id: Some(target_head_id),
            is_full_snapshot: true, // Merges are always full snapshots
            delta_depth: 0,
            branch_id: Some(target_branch_id.to_string()),
        };

        // Store the merge checkpoint
        self.storage.store_checkpoint(&merge_checkpoint)?;
        self.database
            .create_incremental_checkpoint(&merge_checkpoint)?;

        // Update target branch head
        self.database
            .update_branch_head(target_branch_id, &merge_checkpoint_id)?;

        log::info!(
            "Merged branch '{}' into '{}', checkpoint: {}",
            source_branch.name,
            target_branch.name,
            merge_checkpoint_id
        );

        Ok(MergeResult {
            success: true,
            merge_checkpoint_id: Some(merge_checkpoint_id),
            merged_files: merged_changes.iter().map(|c| c.path.clone()).collect(),
            conflicts: Vec::new(),
            strategy,
        })
    }

    /// Perform a 3-way merge between base, source, and target checkpoints
    fn three_way_merge(
        &self,
        base: Option<&Checkpoint>,
        source: &Checkpoint,
        target: &Checkpoint,
        strategy: MergeStrategy,
    ) -> (Vec<FileChange>, Vec<MergeConflict>) {
        let mut merged_changes = Vec::new();
        let mut conflicts = Vec::new();

        // Build file maps
        let base_files: HashMap<PathBuf, &FileChange> = base
            .map(|b| {
                b.file_changes
                    .iter()
                    .map(|fc| (fc.path.clone(), fc))
                    .collect()
            })
            .unwrap_or_default();

        let source_files: HashMap<PathBuf, &FileChange> = source
            .file_changes
            .iter()
            .map(|fc| (fc.path.clone(), fc))
            .collect();

        let target_files: HashMap<PathBuf, &FileChange> = target
            .file_changes
            .iter()
            .map(|fc| (fc.path.clone(), fc))
            .collect();

        // Collect all file paths
        let mut all_paths: HashSet<PathBuf> = HashSet::new();
        all_paths.extend(source_files.keys().cloned());
        all_paths.extend(target_files.keys().cloned());
        all_paths.extend(base_files.keys().cloned());

        for path in all_paths {
            let base_change = base_files.get(&path);
            let source_change = source_files.get(&path);
            let target_change = target_files.get(&path);

            match (base_change, source_change, target_change) {
                // Only source changed — take source
                (_, Some(src), None) => {
                    merged_changes.push((*src).clone());
                }
                // Only target changed — keep target
                (_, None, Some(tgt)) => {
                    merged_changes.push((*tgt).clone());
                }
                // Both changed — check for conflict
                (_, Some(src), Some(tgt)) => {
                    if src.content_hash == tgt.content_hash {
                        // Same change, no conflict
                        merged_changes.push((*src).clone());
                    } else {
                        // Conflict! Handle based on strategy
                        match strategy {
                            MergeStrategy::SourceWins => {
                                merged_changes.push((*src).clone());
                            }
                            MergeStrategy::TargetWins => {
                                merged_changes.push((*tgt).clone());
                            }
                            MergeStrategy::ThreeWay => {
                                let conflict_type =
                                    self.classify_conflict(base_change, source_change, target_change);
                                conflicts.push(MergeConflict {
                                    file_path: path.clone(),
                                    source_content: src.new_content.clone(),
                                    target_content: tgt.new_content.clone(),
                                    base_content: base_change
                                        .and_then(|b| b.new_content.clone()),
                                    conflict_type,
                                });
                            }
                        }
                    }
                }
                // Neither changed — nothing to do
                (_, None, None) => {}
            }
        }

        (merged_changes, conflicts)
    }

    /// Classify the type of merge conflict
    fn classify_conflict(
        &self,
        _base: Option<&&FileChange>,
        source: Option<&&FileChange>,
        target: Option<&&FileChange>,
    ) -> MergeConflictType {
        let source_deleted = source
            .map(|s| matches!(s.change_type, ChangeType::Deleted))
            .unwrap_or(false);
        let target_deleted = target
            .map(|t| matches!(t.change_type, ChangeType::Deleted))
            .unwrap_or(false);
        let source_created = source
            .map(|s| matches!(s.change_type, ChangeType::Created))
            .unwrap_or(false);
        let target_created = target
            .map(|t| matches!(t.change_type, ChangeType::Created))
            .unwrap_or(false);

        if source_created && target_created {
            MergeConflictType::BothCreated
        } else if source_deleted && !target_deleted {
            MergeConflictType::DeletedModified
        } else if !source_deleted && target_deleted {
            MergeConflictType::ModifiedDeleted
        } else {
            MergeConflictType::BothModified
        }
    }
}
