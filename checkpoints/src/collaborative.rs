//! Phase 8.4: Collaborative Checkpoints
//!
//! Provides multi-user checkpoint sharing, cross-machine sync,
//! and compliance audit trail functionality.

use crate::db::CheckpointDatabase;
use crate::error::{CheckpointError, Result};
use crate::storage::CheckpointStorage;
use crate::types::*;

use chrono::Utc;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

/// Manages collaborative checkpoint features
pub struct CollaborativeManager {
    database: Arc<CheckpointDatabase>,
    storage: Arc<CheckpointStorage>,
    /// Current user identity
    current_user: CollaboratorInfo,
}

impl CollaborativeManager {
    /// Create a new collaborative manager
    pub fn new(
        database: Arc<CheckpointDatabase>,
        storage: Arc<CheckpointStorage>,
        user_id: &str,
        display_name: &str,
        machine_id: &str,
    ) -> Self {
        Self {
            database,
            storage,
            current_user: CollaboratorInfo {
                user_id: user_id.to_string(),
                display_name: display_name.to_string(),
                machine_id: machine_id.to_string(),
                last_seen: Utc::now(),
            },
        }
    }

    // ========================================
    // Multi-User Checkpoint Sharing
    // ========================================

    /// Export checkpoints as a shareable bundle
    pub fn share_checkpoints(
        &self,
        checkpoint_ids: &[CheckpointId],
        description: &str,
    ) -> Result<SharedCheckpointBundle> {
        if checkpoint_ids.is_empty() {
            return Err(CheckpointError::validation("No checkpoints to share"));
        }

        // Collect checkpoint data
        let mut checkpoints_data: Vec<Checkpoint> = Vec::new();
        for id in checkpoint_ids {
            if let Some(cp) = self.database.get_checkpoint(id)? {
                checkpoints_data.push(cp);
            } else {
                return Err(CheckpointError::not_found(format!(
                    "Checkpoint {} not found",
                    id
                )));
            }
        }

        // Serialize the checkpoint data
        let data = serde_json::to_vec(&checkpoints_data).map_err(|e| {
            CheckpointError::generic(format!("Failed to serialize checkpoints: {}", e))
        })?;

        let bundle = SharedCheckpointBundle {
            id: Uuid::new_v4().to_string(),
            checkpoint_ids: checkpoint_ids.to_vec(),
            shared_by: self.current_user.clone(),
            shared_at: Utc::now(),
            description: description.to_string(),
            format_version: 1,
            data,
        };

        // Store the share event in DB
        self.database.record_share_event(&bundle)?;

        // Log audit trail
        self.log_audit(
            "share_checkpoints",
            "checkpoint_bundle",
            &bundle.id,
            HashMap::from([
                ("count".to_string(), checkpoint_ids.len().to_string()),
                ("description".to_string(), description.to_string()),
            ]),
            AuditOutcome::Success,
        )?;

        log::info!(
            "Shared {} checkpoints as bundle {}",
            checkpoint_ids.len(),
            bundle.id
        );

        Ok(bundle)
    }

    /// Import a shared checkpoint bundle
    pub fn import_shared_bundle(&self, bundle: &SharedCheckpointBundle) -> Result<Vec<CheckpointId>> {
        // Deserialize the checkpoint data
        let checkpoints: Vec<Checkpoint> = serde_json::from_slice(&bundle.data).map_err(|e| {
            CheckpointError::generic(format!("Failed to deserialize bundle: {}", e))
        })?;

        let mut imported_ids = Vec::new();

        for checkpoint in &checkpoints {
            // Check if checkpoint already exists (dedup by ID)
            if self.database.get_checkpoint(&checkpoint.id)?.is_some() {
                log::debug!("Checkpoint {} already exists, skipping", checkpoint.id);
                continue;
            }

            if self.database.get_session(&checkpoint.session_id)?.is_none() {
                let imported_session = Session {
                    id: checkpoint.session_id,
                    workspace_path: PathBuf::from(format!(
                        "shared-{}",
                        bundle.shared_by.machine_id
                    )),
                    created_at: checkpoint.created_at,
                    last_accessed: checkpoint.created_at,
                    checkpoint_count: 0,
                    total_size_bytes: 0,
                    metadata: HashMap::from([
                        ("imported".to_string(), "true".to_string()),
                        (
                            "shared_by".to_string(),
                            bundle.shared_by.display_name.clone(),
                        ),
                    ]),
                };
                self.database.create_session(&imported_session)?;
            }

            // Store the checkpoint
            self.storage.store_checkpoint(checkpoint)?;
            self.database.create_checkpoint(checkpoint)?;
            imported_ids.push(checkpoint.id);
        }

        // Log audit trail
        self.log_audit(
            "import_shared_bundle",
            "checkpoint_bundle",
            &bundle.id,
            HashMap::from([
                ("imported_count".to_string(), imported_ids.len().to_string()),
                ("total_in_bundle".to_string(), checkpoints.len().to_string()),
                ("shared_by".to_string(), bundle.shared_by.display_name.clone()),
            ]),
            AuditOutcome::Success,
        )?;

        log::info!(
            "Imported {} checkpoints from bundle {} (shared by {})",
            imported_ids.len(),
            bundle.id,
            bundle.shared_by.display_name
        );

        Ok(imported_ids)
    }

    /// List shared bundles for this session
    pub fn list_shared_bundles(&self) -> Result<Vec<SharedCheckpointBundle>> {
        self.database.list_shared_bundles()
    }

    // ========================================
    // Checkpoint Sync Across Machines
    // ========================================

    /// Export all checkpoints since a given timestamp for sync
    pub fn export_for_sync(
        &self,
        since: chrono::DateTime<Utc>,
    ) -> Result<SharedCheckpointBundle> {
        self.database.set_sync_status(&SyncStatus::InProgress)?;

        // Get all checkpoints created after the timestamp
        let checkpoints = self.database.list_checkpoints_since(since)?;

        if checkpoints.is_empty() {
            self.database.set_sync_status(&SyncStatus::Idle)?;
            return Err(CheckpointError::validation("No new checkpoints to sync"));
        }

        let checkpoint_ids: Vec<CheckpointId> = checkpoints.iter().map(|c| c.id).collect();

        let data = serde_json::to_vec(&checkpoints).map_err(|e| {
            CheckpointError::generic(format!("Failed to serialize for sync: {}", e))
        })?;

        let bundle = SharedCheckpointBundle {
            id: format!("sync-{}", Uuid::new_v4()),
            checkpoint_ids,
            shared_by: self.current_user.clone(),
            shared_at: Utc::now(),
            description: format!(
                "Sync export from {} since {}",
                self.current_user.machine_id,
                since.to_rfc3339()
            ),
            format_version: 1,
            data,
        };

        self.log_audit(
            "export_for_sync",
            "sync_bundle",
            &bundle.id,
            HashMap::from([
                ("checkpoint_count".to_string(), bundle.checkpoint_ids.len().to_string()),
                ("since".to_string(), since.to_rfc3339()),
            ]),
            AuditOutcome::Success,
        )?;

        self.database
            .set_sync_status(&SyncStatus::Succeeded { at: Utc::now() })?;

        Ok(bundle)
    }

    /// Import a sync bundle from another machine
    pub fn import_sync_bundle(&self, bundle: &SharedCheckpointBundle) -> Result<SyncStatus> {
        self.database.set_sync_status(&SyncStatus::InProgress)?;

        match self.import_shared_bundle(bundle) {
            Ok(imported_ids) => {
                log::info!(
                    "Sync import complete: {} new checkpoints from {}",
                    imported_ids.len(),
                    bundle.shared_by.machine_id
                );
                let status = SyncStatus::Succeeded { at: Utc::now() };
                self.database.set_sync_status(&status)?;
                Ok(status)
            }
            Err(e) => {
                let status = SyncStatus::Failed {
                    at: Utc::now(),
                    error: e.to_string(),
                };

                self.log_audit(
                    "import_sync_bundle",
                    "sync_bundle",
                    &bundle.id,
                    HashMap::from([("error".to_string(), e.to_string())]),
                    AuditOutcome::Failure(e.to_string()),
                )?;

                self.database.set_sync_status(&status)?;

                Ok(status)
            }
        }
    }

    /// Get current sync status
    pub fn get_sync_status(&self) -> Result<SyncStatus> {
        self.database.get_sync_status()
    }

    // ========================================
    // Audit Trail for Compliance
    // ========================================

    /// Log an audit action
    pub fn log_audit(
        &self,
        action: &str,
        resource_type: &str,
        resource_id: &str,
        details: HashMap<String, String>,
        outcome: AuditOutcome,
    ) -> Result<()> {
        let record = AuditRecord {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            user_id: self.current_user.user_id.clone(),
            machine_id: self.current_user.machine_id.clone(),
            action: action.to_string(),
            resource_type: resource_type.to_string(),
            resource_id: resource_id.to_string(),
            details,
            outcome,
        };

        self.database.insert_audit_record(&record)
    }

    /// Get audit trail records
    pub fn get_audit_trail(
        &self,
        limit: usize,
        action_filter: Option<&str>,
    ) -> Result<Vec<AuditRecord>> {
        self.database.get_audit_trail(limit, action_filter)
    }

    /// Get audit trail for a specific resource
    pub fn get_resource_audit_trail(
        &self,
        resource_type: &str,
        resource_id: &str,
    ) -> Result<Vec<AuditRecord>> {
        self.database
            .get_resource_audit_trail(resource_type, resource_id)
    }

    /// Get current user info
    pub fn current_user(&self) -> &CollaboratorInfo {
        &self.current_user
    }
}
