//! Simplified Node.js bindings for the checkpoint system

use crate::config::CheckpointConfig;
use crate::manager::CheckpointManager;
use crate::types::*;
use neon::prelude::*;
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::sync::Mutex;
use uuid::Uuid;

/// Global configuration storage using thread-safe Lazy
static GLOBAL_STATE: Lazy<Mutex<Option<(CheckpointConfig, PathBuf, Uuid)>>> =
    Lazy::new(|| Mutex::new(None));

/// Create a checkpoint manager configuration
pub fn create_checkpoint_manager(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let config_obj = cx.argument::<JsObject>(0)?;
    let workspace_path_str = cx.argument::<JsString>(1)?.value(&mut cx);

    // Parse basic configuration
    let mut config = CheckpointConfig::default();

    // Extract storage path
    if let Ok(storage_path) = config_obj.get::<JsString, _, _>(&mut cx, "storagePath") {
        config.storage_path = PathBuf::from(storage_path.value(&mut cx));
    }

    // Extract max checkpoints
    if let Ok(max_checkpoints) = config_obj.get::<JsNumber, _, _>(&mut cx, "maxCheckpoints") {
        config.max_checkpoints = max_checkpoints.value(&mut cx) as usize;
    }

    // Extract retention days
    if let Ok(retention_days) = config_obj.get::<JsNumber, _, _>(&mut cx, "retentionDays") {
        config.retention_days = retention_days.value(&mut cx) as i64;
    }

    // Parse workspace path
    let workspace_path = PathBuf::from(workspace_path_str);
    let session_id = Uuid::new_v4();

    // Create and initialize manager to ensure session exists in database
    let mut manager =
        match CheckpointManager::new(config.clone(), workspace_path.clone(), session_id) {
            Ok(manager) => manager,
            Err(e) => return cx.throw_error(format!("Failed to create manager: {}", e)),
        };

    // Initialize session in database
    if let Err(e) = manager.init_session(workspace_path.clone()) {
        return cx.throw_error(format!("Failed to initialize session: {}", e));
    }

    // Store configuration globally (thread-safe)
    let mut global_state = GLOBAL_STATE.lock().unwrap();
    *global_state = Some((config, workspace_path, session_id));

    Ok(cx.boolean(true))
}

/// Helper function to create a manager instance with the existing session
fn create_manager() -> Result<CheckpointManager, String> {
    let global_state = GLOBAL_STATE.lock().unwrap();
    let (config, workspace_path, session_id) = global_state
        .as_ref()
        .ok_or("Configuration not initialized. Call createCheckpointManager first.")?;

    // Create manager with existing session ID (session already exists in database)
    CheckpointManager::new(config.clone(), workspace_path.clone(), *session_id)
        .map_err(|e| format!("Failed to create manager: {}", e))
}

/// Create a checkpoint using the simple method
pub fn create_agent_checkpoint(mut cx: FunctionContext) -> JsResult<JsString> {
    let options_obj = cx.argument::<JsObject>(0)?;

    // Parse description
    let description = if let Ok(desc) = options_obj.get::<JsString, _, _>(&mut cx, "description") {
        Some(desc.value(&mut cx))
    } else {
        Some("AI agent changes".to_string())
    };

    let manager = match create_manager() {
        Ok(manager) => manager,
        Err(e) => return cx.throw_error(e),
    };

    let options = CheckpointOptions {
        description,
        tags: vec!["agent".to_string()],
        ..Default::default()
    };

    match manager.create_agent_checkpoint(options) {
        Ok(checkpoint_id) => Ok(cx.string(checkpoint_id.to_string())),
        Err(e) => cx.throw_error(format!("Failed to create agent checkpoint: {}", e)),
    }
}

/// Start an agent session
pub fn start_agent_session(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let session_id_str = cx.argument::<JsString>(0)?.value(&mut cx);

    // Try to parse as UUID, if that fails, generate a new UUID from the string
    let session_id = match Uuid::parse_str(&session_id_str) {
        Ok(id) => id,
        Err(_) => {
            // Generate a deterministic UUID from the string
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            std::hash::Hash::hash(&session_id_str, &mut hasher);
            let hash = std::hash::Hasher::finish(&hasher);

            // Create a UUID from the hash (this is a simple approach)
            let bytes = [
                (hash >> 56) as u8,
                (hash >> 48) as u8,
                (hash >> 40) as u8,
                (hash >> 32) as u8,
                (hash >> 24) as u8,
                (hash >> 16) as u8,
                (hash >> 8) as u8,
                hash as u8,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ];
            Uuid::from_bytes(bytes)
        }
    };

    let manager = match create_manager() {
        Ok(manager) => manager,
        Err(e) => return cx.throw_error(e),
    };

    match manager.start_agent_session(session_id) {
        Ok(_) => Ok(cx.boolean(true)),
        Err(e) => cx.throw_error(format!("Failed to start agent session: {}", e)),
    }
}

/// Stop agent session
pub fn stop_agent_session(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let manager = match create_manager() {
        Ok(manager) => manager,
        Err(e) => return cx.throw_error(e),
    };

    match manager.stop_agent_session() {
        Ok(_) => Ok(cx.boolean(true)),
        Err(e) => cx.throw_error(format!("Failed to stop agent session: {}", e)),
    }
}

/// Set operation mode
pub fn set_operation_mode(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let mode_str = cx.argument::<JsString>(0)?.value(&mut cx);

    let mode = match mode_str.as_str() {
        "Agent" => crate::changeset_tracker::OperationMode::Agent,
        "Chat" => crate::changeset_tracker::OperationMode::Chat,
        "Manual" => crate::changeset_tracker::OperationMode::Manual,
        _ => return cx.throw_error("Invalid operation mode"),
    };

    let manager = match create_manager() {
        Ok(manager) => manager,
        Err(e) => return cx.throw_error(e),
    };

    match manager.set_operation_mode(mode) {
        Ok(_) => Ok(cx.boolean(true)),
        Err(e) => cx.throw_error(format!("Failed to set operation mode: {}", e)),
    }
}

/// Track AI files
pub fn track_ai_files(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let files_array = cx.argument::<JsArray>(0)?;

    // Parse file paths
    let length = files_array.len(&mut cx);
    let mut file_paths = Vec::new();

    for i in 0..length {
        let item = files_array.get::<JsString, _, _>(&mut cx, i)?;
        file_paths.push(PathBuf::from(item.value(&mut cx)));
    }

    let manager = match create_manager() {
        Ok(manager) => manager,
        Err(e) => return cx.throw_error(e),
    };

    match manager.track_ai_files(&file_paths) {
        Ok(_) => Ok(cx.boolean(true)),
        Err(e) => cx.throw_error(format!("Failed to track AI files: {}", e)),
    }
}

/// Check if there are AI changes
pub fn has_ai_changes(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let manager = match create_manager() {
        Ok(manager) => manager,
        Err(e) => return cx.throw_error(e),
    };

    Ok(cx.boolean(manager.has_ai_changes()))
}

/// Get changeset stats
pub fn get_changeset_stats(mut cx: FunctionContext) -> JsResult<JsObject> {
    let manager = match create_manager() {
        Ok(manager) => manager,
        Err(e) => return cx.throw_error(e),
    };

    let js_obj = cx.empty_object();

    if let Some(stats) = manager.get_changeset_stats() {
        if let Some(files_tracked) = stats.get("files_tracked").and_then(|v| v.as_u64()) {
            let files_tracked = cx.number(files_tracked as f64);
            js_obj.set(&mut cx, "filesTracked", files_tracked)?;
        }

        if let Some(changes_detected) = stats.get("changes_detected").and_then(|v| v.as_u64()) {
            let changes_detected = cx.number(changes_detected as f64);
            js_obj.set(&mut cx, "changesDetected", changes_detected)?;
        }
    } else {
        // Return default stats
        let files_tracked = cx.number(0.0);
        js_obj.set(&mut cx, "filesTracked", files_tracked)?;
        let changes_detected = cx.number(0.0);
        js_obj.set(&mut cx, "changesDetected", changes_detected)?;
    }

    Ok(js_obj)
}

// Placeholder functions for other bindings
pub fn create_checkpoint(mut cx: FunctionContext) -> JsResult<JsString> {
    // Redirect to createAgentCheckpoint for now
    let description = cx.argument::<JsString>(1)?.value(&mut cx);

    let manager = match create_manager() {
        Ok(manager) => manager,
        Err(e) => return cx.throw_error(e),
    };

    let options = CheckpointOptions {
        description: Some(description),
        tags: vec!["manual".to_string()],
        ..Default::default()
    };

    match manager.create_checkpoint(options) {
        Ok(checkpoint_id) => Ok(cx.string(checkpoint_id.to_string())),
        Err(e) => cx.throw_error(format!("Failed to create checkpoint: {}", e)),
    }
}

pub fn restore_checkpoint(mut cx: FunctionContext) -> JsResult<JsObject> {
    let checkpoint_id_str = cx.argument::<JsString>(0)?.value(&mut cx);
    let options_obj = cx.argument::<JsObject>(1)?;

    // Parse checkpoint ID
    let checkpoint_id = match Uuid::parse_str(&checkpoint_id_str) {
        Ok(id) => id,
        Err(_) => return cx.throw_error("Invalid checkpoint ID format"),
    };

    // Parse restore options
    let mut restore_options = RestoreOptions::default();

    // Parse createBackup
    if let Ok(create_backup) = options_obj.get::<JsBoolean, _, _>(&mut cx, "createBackup") {
        restore_options.create_backup = create_backup.value(&mut cx);
    }

    // Parse restorePermissions
    if let Ok(restore_permissions) =
        options_obj.get::<JsBoolean, _, _>(&mut cx, "restorePermissions")
    {
        restore_options.restore_permissions = restore_permissions.value(&mut cx);
    }

    // Parse restoreTimestamps
    if let Ok(restore_timestamps) = options_obj.get::<JsBoolean, _, _>(&mut cx, "restoreTimestamps")
    {
        restore_options.restore_timestamps = restore_timestamps.value(&mut cx);
    }

    // Parse validateChecksums
    if let Ok(validate_checksums) = options_obj.get::<JsBoolean, _, _>(&mut cx, "validateChecksums")
    {
        restore_options.validate_checksums = validate_checksums.value(&mut cx);
    }

    // Parse conflictResolution
    if let Ok(conflict_resolution) =
        options_obj.get::<JsString, _, _>(&mut cx, "conflictResolution")
    {
        let resolution_str = conflict_resolution.value(&mut cx);
        restore_options.conflict_resolution = match resolution_str.as_str() {
            "skip" => ConflictResolution::Skip,
            "overwrite" => ConflictResolution::Overwrite,
            "backup" => ConflictResolution::Backup,
            "prompt" => ConflictResolution::Prompt,
            _ => ConflictResolution::Overwrite, // Default
        };
    }

    // Parse includeFiles
    if let Ok(include_files) = options_obj.get::<JsArray, _, _>(&mut cx, "includeFiles") {
        let length = include_files.len(&mut cx);
        let mut files = Vec::new();
        for i in 0..length {
            if let Ok(file) = include_files.get::<JsString, _, _>(&mut cx, i) {
                files.push(PathBuf::from(file.value(&mut cx)));
            }
        }
        restore_options.include_files = files;
    }

    // Parse excludeFiles
    if let Ok(exclude_files) = options_obj.get::<JsArray, _, _>(&mut cx, "excludeFiles") {
        let length = exclude_files.len(&mut cx);
        let mut files = Vec::new();
        for i in 0..length {
            if let Ok(file) = exclude_files.get::<JsString, _, _>(&mut cx, i) {
                files.push(PathBuf::from(file.value(&mut cx)));
            }
        }
        restore_options.exclude_files = files;
    }

    // Create manager and perform restoration
    let manager = match create_manager() {
        Ok(manager) => manager,
        Err(e) => return cx.throw_error(e),
    };

    let result = match manager.restore_checkpoint(&checkpoint_id, restore_options) {
        Ok(result) => result,
        Err(e) => return cx.throw_error(format!("Failed to restore checkpoint: {}", e)),
    };

    // Build result object
    let js_result = cx.empty_object();

    // success field
    let success = cx.boolean(result.success);
    js_result.set(&mut cx, "success", success)?;

    // restoredFiles array
    let restored_files = cx.empty_array();
    for (i, file_path) in result.restored_files.iter().enumerate() {
        let path_str = cx.string(file_path.to_string_lossy());
        restored_files.set(&mut cx, i as u32, path_str)?;
    }
    js_result.set(&mut cx, "restoredFiles", restored_files)?;

    // createdFiles array
    let created_files = cx.empty_array();
    for (i, file_path) in result.created_files.iter().enumerate() {
        let path_str = cx.string(file_path.to_string_lossy());
        created_files.set(&mut cx, i as u32, path_str)?;
    }
    js_result.set(&mut cx, "createdFiles", created_files)?;

    // modifiedFiles array
    let modified_files = cx.empty_array();
    for (i, file_path) in result.modified_files.iter().enumerate() {
        let path_str = cx.string(file_path.to_string_lossy());
        modified_files.set(&mut cx, i as u32, path_str)?;
    }
    js_result.set(&mut cx, "modifiedFiles", modified_files)?;

    // deletedFiles array
    let deleted_files = cx.empty_array();
    for (i, file_path) in result.deleted_files.iter().enumerate() {
        let path_str = cx.string(file_path.to_string_lossy());
        deleted_files.set(&mut cx, i as u32, path_str)?;
    }
    js_result.set(&mut cx, "deletedFiles", deleted_files)?;

    // failedFiles array
    let failed_files = cx.empty_array();
    for (i, (file_path, error)) in result.failed_files.iter().enumerate() {
        let item = cx.empty_object();
        let path_str = cx.string(file_path.to_string_lossy());
        let error_str = cx.string(error);
        item.set(&mut cx, "path", path_str)?;
        item.set(&mut cx, "error", error_str)?;
        failed_files.set(&mut cx, i as u32, item)?;
    }
    js_result.set(&mut cx, "failedFiles", failed_files)?;

    // conflicts array
    let conflicts_array = cx.empty_array();
    for (i, conflict) in result.conflicts.iter().enumerate() {
        let item = cx.empty_object();
        let path_str = cx.string(conflict.file_path.to_string_lossy());
        let type_str = cx.string(format!("{:?}", conflict.conflict_type));
        item.set(&mut cx, "path", path_str)?;
        item.set(&mut cx, "type", type_str)?;
        conflicts_array.set(&mut cx, i as u32, item)?;
    }
    js_result.set(&mut cx, "conflicts", conflicts_array)?;

    // backupCheckpointId (optional)
    if let Some(backup_id) = result.backup_checkpoint_id {
        let backup_id_str = cx.string(backup_id.to_string());
        js_result.set(&mut cx, "backupCheckpointId", backup_id_str)?;
    }

    Ok(js_result)
}

pub fn list_checkpoints(mut cx: FunctionContext) -> JsResult<JsArray> {
    cx.throw_error("List functionality not implemented yet")
}

pub fn delete_checkpoint(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    cx.throw_error("Delete functionality not implemented yet")
}

pub fn cleanup_old_checkpoints(mut cx: FunctionContext) -> JsResult<JsNumber> {
    cx.throw_error("Cleanup functionality not implemented yet")
}

pub fn get_checkpoint_stats(mut cx: FunctionContext) -> JsResult<JsObject> {
    cx.throw_error("Stats functionality not implemented yet")
}
