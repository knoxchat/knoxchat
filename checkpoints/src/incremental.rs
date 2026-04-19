//! Phase 8.1: Incremental Checkpointing
//!
//! Provides delta-based checkpointing where only changed files are stored
//! relative to a parent checkpoint. Includes chain reconstruction for
//! restoration and periodic full snapshots to limit chain length.

use crate::db::CheckpointDatabase;
use crate::error::{CheckpointError, Result};
use crate::types::*;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Manages incremental (delta-based) checkpointing
pub struct IncrementalCheckpointManager {
    config: IncrementalConfig,
    database: Arc<CheckpointDatabase>,
}

impl IncrementalCheckpointManager {
    /// Create a new incremental checkpoint manager
    pub fn new(
        config: IncrementalConfig,
        database: Arc<CheckpointDatabase>,
    ) -> Self {
        Self {
            config,
            database,
        }
    }

    /// Determine whether the next checkpoint should be a full snapshot or delta
    pub fn should_create_full_snapshot(
        &self,
        session_id: &SessionId,
        branch_id: Option<&str>,
    ) -> Result<bool> {
        if !self.config.enabled {
            return Ok(true); // Always full snapshot when incremental is disabled
        }

        // Check how many deltas since last full snapshot
        let delta_count = self
            .database
            .count_since_last_full_snapshot(session_id, branch_id)?;

        // Force full snapshot if chain is too long
        if delta_count >= self.config.max_chain_length {
            log::info!(
                "Delta chain length {} >= max {}, forcing full snapshot",
                delta_count,
                self.config.max_chain_length
            );
            return Ok(true);
        }

        // Force full snapshot at regular intervals
        if delta_count > 0 && delta_count % self.config.full_snapshot_interval == 0 {
            log::info!(
                "Reached full snapshot interval at {} deltas",
                delta_count
            );
            return Ok(true);
        }

        Ok(false)
    }

    /// Compute the delta (changed files only) between current state and parent checkpoint
    pub fn compute_delta(
        &self,
        current_changes: &[FileChange],
        parent_checkpoint: &Checkpoint,
    ) -> Vec<FileChange> {
        let parent_hashes: HashMap<&PathBuf, &str> = parent_checkpoint
            .file_changes
            .iter()
            .map(|fc| (&fc.path, fc.content_hash.as_str()))
            .collect();

        current_changes
            .iter()
            .filter(|change| {
                match &change.change_type {
                    ChangeType::Created => true, // New files always included
                    ChangeType::Deleted => true,  // Deletions always included
                    ChangeType::Modified => {
                        // Only include if content actually changed from parent
                        match parent_hashes.get(&change.path) {
                            Some(parent_hash) => *parent_hash != change.content_hash,
                            None => true, // File not in parent, include it
                        }
                    }
                    ChangeType::Renamed { .. } | ChangeType::Moved { .. } => true,
                }
            })
            .cloned()
            .collect()
    }

    /// Reconstruct a complete checkpoint state from its delta chain
    pub fn reconstruct_from_chain(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<ReconstructedCheckpoint> {
        // Get the delta chain (full snapshot first, then deltas in order)
        let chain = self.database.get_delta_chain(checkpoint_id)?;

        if chain.is_empty() {
            return Err(CheckpointError::checkpoint_not_found(
                checkpoint_id.to_string(),
            ));
        }

        let mut file_states: HashMap<PathBuf, ReconstructedFileState> = HashMap::new();
        let mut total_chain_size = 0u64;

        // Apply each checkpoint in the chain, starting from the full snapshot
        for checkpoint in &chain {
            total_chain_size += checkpoint.size_bytes;

            for change in &checkpoint.file_changes {
                match &change.change_type {
                    ChangeType::Created | ChangeType::Modified => {
                        file_states.insert(
                            change.path.clone(),
                            ReconstructedFileState {
                                path: change.path.clone(),
                                content: change.new_content.clone(),
                                content_hash: change.content_hash.clone(),
                                size_bytes: change.size_bytes,
                                encoding: change.encoding,
                            },
                        );
                    }
                    ChangeType::Deleted => {
                        file_states.remove(&change.path);
                    }
                    ChangeType::Renamed { from } => {
                        if let Some(state) = file_states.remove(from) {
                            let mut new_state = state;
                            new_state.path = change.path.clone();
                            if let Some(content) = &change.new_content {
                                new_state.content = Some(content.clone());
                            }
                            file_states.insert(change.path.clone(), new_state);
                        }
                    }
                    ChangeType::Moved { from } => {
                        if let Some(state) = file_states.remove(from) {
                            let mut new_state = state;
                            new_state.path = change.path.clone();
                            file_states.insert(change.path.clone(), new_state);
                        }
                    }
                }
            }
        }

        // Use the file inventory from the last checkpoint in the chain
        let last_checkpoint = chain.last().unwrap();
        let file_inventory = last_checkpoint.file_inventory.clone();

        Ok(ReconstructedCheckpoint {
            file_states,
            file_inventory,
            chain_length: chain.len() as u32,
            total_chain_size_bytes: total_chain_size,
        })
    }

    /// Get the current delta depth for the next checkpoint
    pub fn get_next_delta_depth(
        &self,
        session_id: &SessionId,
        branch_id: Option<&str>,
    ) -> Result<u32> {
        if let Some(latest) = self
            .database
            .get_latest_checkpoint(session_id, branch_id)?
        {
            if latest.is_full_snapshot {
                Ok(1) // First delta after a full snapshot
            } else {
                Ok(latest.delta_depth + 1)
            }
        } else {
            Ok(0) // No checkpoints yet, will be full snapshot
        }
    }

    /// Get the parent checkpoint ID for the next incremental checkpoint
    pub fn get_parent_checkpoint_id(
        &self,
        session_id: &SessionId,
        branch_id: Option<&str>,
    ) -> Result<Option<CheckpointId>> {
        if let Some(latest) = self
            .database
            .get_latest_checkpoint(session_id, branch_id)?
        {
            Ok(Some(latest.id))
        } else {
            Ok(None)
        }
    }
}
