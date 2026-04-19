//! Node.js bindings for the checkpoint system

use crate::changeset_tracker::{ChangesetTracker, OperationMode};
use crate::config::CheckpointConfig;
use crate::manager::CheckpointManager;
use crate::types::*;
use neon::prelude::*;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Global checkpoint manager instance - using Lazy for thread safety
use once_cell::sync::Lazy;
static MANAGER: Lazy<Arc<Mutex<Option<CheckpointManager>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

// ========================================
// Utility Functions
// ========================================

/// Parse an array of file paths from JavaScript
fn parse_path_array(cx: &mut FunctionContext, array: Handle<JsArray>) -> NeonResult<Vec<PathBuf>> {
    let length = array.len(cx);
    let mut paths = Vec::new();
    
    for i in 0..length {
        let item = array.get::<JsString, _, _>(cx, i)?;
        let path_str = item.value(cx);
        paths.push(PathBuf::from(path_str));
    }
    
    Ok(paths)
}

/// Get an optional string property from a JavaScript object
fn get_optional_string(cx: &mut FunctionContext, obj: &Handle<JsObject>, key: &str) -> NeonResult<Option<String>> {
    match obj.get::<JsString, _, _>(cx, key) {
        Ok(value) => Ok(Some(value.value(cx))),
        Err(_) => Ok(None),
    }
}

/// Get an optional string array property from a JavaScript object
fn get_optional_string_array(cx: &mut FunctionContext, obj: &Handle<JsObject>, key: &str) -> NeonResult<Option<Vec<String>>> {
    match obj.get::<JsArray, _, _>(cx, key) {
        Ok(array) => {
            let length = array.len(cx);
            let mut strings = Vec::new();
            
            for i in 0..length {
                let item = array.get::<JsString, _, _>(cx, i)?;
                strings.push(item.value(cx));
            }
            
            Ok(Some(strings))
        }
        Err(_) => Ok(None),
    }
}

/// Get an optional number property from a JavaScript object
fn get_optional_number(cx: &mut FunctionContext, obj: &Handle<JsObject>, key: &str) -> NeonResult<Option<f64>> {
    match obj.get::<JsNumber, _, _>(cx, key) {
        Ok(value) => Ok(Some(value.value(cx))),
        Err(_) => Ok(None),
    }
}

/// Create a checkpoint manager
pub fn create_checkpoint_manager(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let config_obj = cx.argument::<JsObject>(0)?;
    let workspace_path_str = cx.argument::<JsString>(1)?.value(&mut cx);
    
    // Parse configuration from JS object
    let config = parse_config(&mut cx, config_obj)?;
    
    // Parse workspace path and session ID
    let workspace_path = PathBuf::from(workspace_path_str);
    let session_id = Uuid::new_v4();
    
    match CheckpointManager::new(config, workspace_path, session_id) {
        Ok(manager) => {
            let mut global_manager = MANAGER.lock().unwrap();
            *global_manager = Some(manager);
            Ok(cx.boolean(true))
        }
        Err(e) => cx.throw_error(format!("Failed to create checkpoint manager: {}", e)),
    }
}

/// Create a checkpoint
pub fn create_checkpoint(mut cx: FunctionContext) -> JsResult<JsString> {
    let workspace_path = cx.argument::<JsString>(0)?.value(&mut cx);
    let description = cx.argument::<JsString>(1)?.value(&mut cx);
    let tags_array = cx.argument::<JsArray>(2)?;
    
    // Parse tags
    let tags = parse_string_array(&mut cx, tags_array)?;
    
    let options = CheckpointOptions {
        description: Some(description),
        tags,
        ..Default::default()
    };
    
    with_manager(&mut cx, |manager| {
        manager.create_checkpoint(options)
    }).map(|checkpoint_id| cx.string(checkpoint_id.to_string()))
}

/// Restore a checkpoint
pub fn restore_checkpoint(mut cx: FunctionContext) -> JsResult<JsObject> {
    let checkpoint_id_str = cx.argument::<JsString>(0)?.value(&mut cx);
    let options_obj = cx.argument::<JsObject>(1)?;
    
    // Parse checkpoint ID
    let checkpoint_id = match Uuid::parse_str(&checkpoint_id_str) {
        Ok(id) => id,
        Err(_) => return cx.throw_error("Invalid checkpoint ID format"),
    };
    
    // Parse restore options
    let restore_options = parse_restore_options(&mut cx, options_obj)?;
    
    with_manager(&mut cx, |manager| {
        manager.restore_checkpoint(&checkpoint_id, restore_options)
    }).and_then(|result| {
        let js_result = cx.empty_object();
        
        let success = cx.boolean(result.success);
        js_result.set(&mut cx, "success", success)?;
        
        let restored_files = create_path_array(&mut cx, &result.restored_files)?;
        js_result.set(&mut cx, "restoredFiles", restored_files)?;
        
        let failed_files = create_failed_files_array(&mut cx, &result.failed_files)?;
        js_result.set(&mut cx, "failedFiles", failed_files)?;
        
        let created_files = create_path_array(&mut cx, &result.created_files)?;
        js_result.set(&mut cx, "createdFiles", created_files)?;
        
        let modified_files = create_path_array(&mut cx, &result.modified_files)?;
        js_result.set(&mut cx, "modifiedFiles", modified_files)?;
        
        let deleted_files = create_path_array(&mut cx, &result.deleted_files)?;
        js_result.set(&mut cx, "deletedFiles", deleted_files)?;
        
        if let Some(backup_id) = result.backup_checkpoint_id {
            let backup_id_str = cx.string(backup_id.to_string());
            js_result.set(&mut cx, "backupCheckpointId", backup_id_str)?;
        }
        
        // Include conflict details so the caller can prompt user and re-invoke
        let conflicts_array = JsArray::new(&mut cx, result.conflicts.len() as u32);
        for (i, conflict) in result.conflicts.iter().enumerate() {
            let conflict_obj = cx.empty_object();
            let path_str = cx.string(conflict.file_path.to_string_lossy());
            conflict_obj.set(&mut cx, "path", path_str)?;
            let conflict_type = cx.string(format!("{:?}", conflict.conflict_type));
            conflict_obj.set(&mut cx, "type", conflict_type)?;
            let resolution = cx.string(format!("{:?}", conflict.resolution));
            conflict_obj.set(&mut cx, "resolution", resolution)?;
            conflicts_array.set(&mut cx, i as u32, conflict_obj)?;
        }
        js_result.set(&mut cx, "conflicts", conflicts_array)?;
        
        Ok(js_result)
    })
}

/// List checkpoints
pub fn list_checkpoints(mut cx: FunctionContext) -> JsResult<JsArray> {
    let limit = if cx.len() > 0 {
        Some(cx.argument::<JsNumber>(0)?.value(&mut cx) as usize)
    } else {
        None
    };
    
    let manager = get_manager(&mut cx)?;
    let manager = manager.lock().unwrap();
    
    match manager.list_checkpoints(limit) {
        Ok(checkpoints) => {
            let js_array = JsArray::new(&mut cx, checkpoints.len() as u32);
            
            for (i, checkpoint) in checkpoints.iter().enumerate() {
                let js_checkpoint = create_checkpoint_object(&mut cx, checkpoint)?;
                js_array.set(&mut cx, i as u32, js_checkpoint)?;
            }
            
            Ok(js_array)
        }
        Err(e) => cx.throw_error(format!("Failed to list checkpoints: {}", e)),
    }
}

/// Delete a checkpoint
pub fn delete_checkpoint(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let checkpoint_id_str = cx.argument::<JsString>(0)?.value(&mut cx);
    
    let manager = get_manager(&mut cx)?;
    
    let checkpoint_id = match Uuid::parse_str(&checkpoint_id_str) {
        Ok(id) => id,
        Err(_) => return cx.throw_error("Invalid checkpoint ID format"),
    };
    
    let manager = manager.lock().unwrap();
    
    match manager.delete_checkpoint(&checkpoint_id) {
        Ok(()) => Ok(cx.boolean(true)),
        Err(e) => cx.throw_error(format!("Failed to delete checkpoint: {}", e)),
    }
}

/// Clean up old checkpoints
pub fn cleanup_old_checkpoints(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let manager = get_manager(&mut cx)?;
    let manager = manager.lock().unwrap();
    
    match manager.cleanup_old_checkpoints() {
        Ok(deleted_count) => Ok(cx.number(deleted_count as f64)),
        Err(e) => cx.throw_error(format!("Failed to cleanup checkpoints: {}", e)),
    }
}

/// Run storage garbage collection to remove orphaned content blobs
pub fn run_storage_gc(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let manager = get_manager(&mut cx)?;
    let manager = manager.lock().unwrap();
    
    match manager.run_storage_gc() {
        Ok(freed_bytes) => Ok(cx.number(freed_bytes as f64)),
        Err(e) => cx.throw_error(format!("Failed to run storage GC: {}", e)),
    }
}

/// Get checkpoint statistics
pub fn get_checkpoint_stats(mut cx: FunctionContext) -> JsResult<JsObject> {
    let manager = get_manager(&mut cx)?;
    let manager = manager.lock().unwrap();
    
    match manager.get_stats() {
        Ok(stats) => create_stats_object(&mut cx, &stats),
        Err(e) => cx.throw_error(format!("Failed to get stats: {}", e)),
    }
}

/// Helper function to execute with the global manager
fn with_manager<F, R>(cx: &mut FunctionContext, f: F) -> NeonResult<R>
where
    F: FnOnce(&CheckpointManager) -> Result<R, CheckpointError>,
{
    let manager_guard = MANAGER.lock().unwrap();
    if let Some(manager) = manager_guard.as_ref() {
        match f(manager) {
            Ok(result) => Ok(result),
            Err(e) => cx.throw_error(format!("Manager operation failed: {}", e)),
        }
    } else {
        cx.throw_error("Checkpoint manager not initialized. Call createCheckpointManager first.")
    }
}

/// Parse configuration from JS object
fn parse_config(cx: &mut FunctionContext, obj: Handle<JsObject>) -> NeonResult<CheckpointConfig> {
    let mut config = CheckpointConfig::default();
    
    // Parse storage path
    if let Ok(storage_path) = obj.get::<JsString, _, _>(cx, "storagePath") {
        config.storage_path = PathBuf::from(storage_path.value(cx));
    }
    
    // Parse debug mode
    if let Ok(debug_mode) = obj.get::<JsBoolean, _, _>(cx, "debugMode") {
        config.debug_mode = debug_mode.value(cx);
        if config.debug_mode {
            config.storage_path = CheckpointConfig::default_storage_path(true);
        }
    }
    
    // Parse max checkpoints
    if let Ok(max_checkpoints) = obj.get::<JsNumber, _, _>(cx, "maxCheckpoints") {
        config.max_checkpoints = max_checkpoints.value(cx) as usize;
    }
    
    // Parse retention days
    if let Ok(retention_days) = obj.get::<JsNumber, _, _>(cx, "retentionDays") {
        config.retention_days = retention_days.value(cx) as i64;
    }
    
    // Parse max storage bytes
    if let Ok(max_storage) = obj.get::<JsNumber, _, _>(cx, "maxStorageBytes") {
        config.max_storage_bytes = max_storage.value(cx) as u64;
    }
    
    // Parse compression setting
    if let Ok(enable_compression) = obj.get::<JsBoolean, _, _>(cx, "enableCompression") {
        config.enable_compression = enable_compression.value(cx);
    }
    
    // Parse tracked extensions
    if let Ok(extensions_array) = obj.get::<JsArray, _, _>(cx, "trackedExtensions") {
        config.tracked_extensions = parse_string_array(cx, extensions_array)?;
    }
    
    Ok(config)
}

/// Parse restore options from JS object
fn parse_restore_options(cx: &mut FunctionContext, obj: Handle<JsObject>) -> NeonResult<RestoreOptions> {
    let mut options = RestoreOptions::default();
    
    if let Ok(create_backup) = obj.get::<JsBoolean, _, _>(cx, "createBackup") {
        options.create_backup = create_backup.value(cx);
    }
    
    if let Ok(restore_permissions) = obj.get::<JsBoolean, _, _>(cx, "restorePermissions") {
        options.restore_permissions = restore_permissions.value(cx);
    }
    
    if let Ok(show_progress) = obj.get::<JsBoolean, _, _>(cx, "showProgress") {
        // Note: show_progress is not in RestoreOptions, used by caller
    }
    
    if let Ok(specific_files_array) = obj.get::<JsArray, _, _>(cx, "specificFiles") {
        let file_paths = parse_string_array(cx, specific_files_array)?;
        options.include_files = file_paths.into_iter().map(PathBuf::from).collect();
    }
    
    if let Ok(conflict_resolution) = obj.get::<JsString, _, _>(cx, "conflictResolution") {
        options.conflict_resolution = match conflict_resolution.value(cx).as_str() {
            "skip" => ConflictResolution::Skip,
            "overwrite" => ConflictResolution::Overwrite,
            "backup" => ConflictResolution::Backup,
            "prompt" => ConflictResolution::Prompt,
            _ => ConflictResolution::Prompt,
        };
    }
    
    if let Ok(prompt_fallback) = obj.get::<JsString, _, _>(cx, "promptFallback") {
        options.prompt_fallback = match prompt_fallback.value(cx).as_str() {
            "skip" => ConflictResolution::Skip,
            "overwrite" => ConflictResolution::Overwrite,
            "backup" => ConflictResolution::Backup,
            _ => ConflictResolution::Backup,
        };
    }
    
    if let Ok(dry_run) = obj.get::<JsBoolean, _, _>(cx, "dryRun") {
        options.dry_run = dry_run.value(cx);
    }
    
    // Parse per-file resolutions: { "path/to/file": "overwrite", ... }
    if let Ok(per_file_obj) = obj.get::<JsObject, _, _>(cx, "perFileResolutions") {
        if let Ok(keys) = per_file_obj.get_own_property_names(cx) {
            let length = keys.len(cx);
            for i in 0..length {
                if let Ok(key) = keys.get::<JsString, _, _>(cx, i) {
                    let file_path = PathBuf::from(key.value(cx));
                    if let Ok(resolution_str) = per_file_obj.get::<JsString, _, _>(cx, key.value(cx).as_str()) {
                        let resolution = match resolution_str.value(cx).as_str() {
                            "skip" => ConflictResolution::Skip,
                            "overwrite" => ConflictResolution::Overwrite,
                            "backup" => ConflictResolution::Backup,
                            _ => ConflictResolution::Skip,
                        };
                        options.per_file_resolutions.insert(file_path, resolution);
                    }
                }
            }
        }
    }
    
    Ok(options)
}

/// Parse array of strings from JS
fn parse_string_array(cx: &mut FunctionContext, array: Handle<JsArray>) -> NeonResult<Vec<String>> {
    let length = array.len(cx);
    let mut result = Vec::new();
    
    for i in 0..length {
        let item: Handle<JsString> = array.get(cx, i)?;
        result.push(item.value(cx));
    }
    
    Ok(result)
}

/// Create JS array from path vector
fn create_path_array<'a>(cx: &'a mut FunctionContext<'a>, paths: &[PathBuf]) -> NeonResult<Handle<'a, JsArray>> {
    let js_array = JsArray::new(cx, paths.len() as u32);
    
    for (i, path) in paths.iter().enumerate() {
        let path_str = cx.string(path.to_string_lossy());
        js_array.set(cx, i as u32, path_str)?;
    }
    
    Ok(js_array)
}

/// Create JS array for failed files
fn create_failed_files_array<'a>(cx: &'a mut FunctionContext<'a>, failed: &[(PathBuf, String)]) -> NeonResult<Handle<'a, JsArray>> {
    let js_array = JsArray::new(cx, failed.len() as u32);
    
    for (i, (path, error)) in failed.iter().enumerate() {
        let js_obj = cx.empty_object();
        let path_str = cx.string(path.to_string_lossy());
        let error_str = cx.string(error);
        
        js_obj.set(cx, "path", path_str)?;
        js_obj.set(cx, "error", error_str)?;
        
        js_array.set(cx, i as u32, js_obj)?;
    }
    
    Ok(js_array)
}

/// Create JS object from checkpoint
fn create_checkpoint_object<'a>(cx: &'a mut FunctionContext<'a>, checkpoint: &Checkpoint) -> NeonResult<Handle<'a, JsObject>> {
    let js_obj = cx.empty_object();
    
    let id = cx.string(checkpoint.id.to_string());
    js_obj.set(cx, "id", id)?;
    
    let session_id = cx.string(checkpoint.session_id.to_string());
    js_obj.set(cx, "sessionId", session_id)?;
    
    let description = cx.string(&checkpoint.description);
    js_obj.set(cx, "description", description)?;
    
    let created_at = cx.string(checkpoint.created_at.to_rfc3339());
    js_obj.set(cx, "createdAt", created_at)?;
    
    let files_affected = cx.number(checkpoint.files_affected as f64);
    js_obj.set(cx, "filesAffected", files_affected)?;
    
    let size_bytes = cx.number(checkpoint.size_bytes as f64);
    js_obj.set(cx, "sizeBytes", size_bytes)?;
    
    let short_summary = cx.string(checkpoint.short_summary());
    js_obj.set(cx, "shortSummary", short_summary)?;
    
    // Add tags array
    let tags_array = JsArray::new(cx, checkpoint.tags.len() as u32);
    for (i, tag) in checkpoint.tags.iter().enumerate() {
        let tag_str = cx.string(tag);
        tags_array.set(cx, i as u32, tag_str)?;
    }
    js_obj.set(cx, "tags", tags_array)?;
    
    Ok(js_obj)
}

/// Create JS object from stats
fn create_stats_object<'a>(cx: &'a mut FunctionContext<'a>, stats: &CheckpointStats) -> NeonResult<Handle<'a, JsObject>> {
    let js_obj = cx.empty_object();
    
    let total_checkpoints = cx.number(stats.total_checkpoints as f64);
    js_obj.set(cx, "totalCheckpoints", total_checkpoints)?;
    
    let session_checkpoints = cx.number(stats.session_checkpoints as f64);
    js_obj.set(cx, "sessionCheckpoints", session_checkpoints)?;
    
    let total_storage_bytes = cx.number(stats.total_storage_bytes as f64);
    js_obj.set(cx, "totalStorageBytes", total_storage_bytes)?;
    
    let average_size_bytes = cx.number(stats.average_size_bytes as f64);
    js_obj.set(cx, "averageSizeBytes", average_size_bytes)?;
    
    let total_files_tracked = cx.number(stats.total_files_tracked as f64);
    js_obj.set(cx, "totalFilesTracked", total_files_tracked)?;
    
    if let Some(oldest) = stats.oldest_checkpoint {
        let oldest_str = cx.string(oldest.to_rfc3339());
        js_obj.set(cx, "oldestCheckpoint", oldest_str)?;
    }
    
    if let Some(newest) = stats.newest_checkpoint {
        let newest_str = cx.string(newest.to_rfc3339());
        js_obj.set(cx, "newestCheckpoint", newest_str)?;
    }
    
    Ok(js_obj)
}

// ========================================
// Smart Checkpoint System Bindings
// ========================================

/// Start an agent session for smart checkpoint tracking
pub fn start_agent_session(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let session_id_str = cx.argument::<JsString>(0)?.value(&mut cx);
    
    let manager = get_manager(&mut cx)?;
    let mut manager = manager.lock().unwrap();
    
    // Parse session ID
    let session_id = match Uuid::parse_str(&session_id_str) {
        Ok(id) => id,
        Err(_) => return cx.throw_error("Invalid session ID format"),
    };
    
    match manager.start_agent_session(session_id) {
        Ok(_) => Ok(cx.boolean(true)),
        Err(e) => cx.throw_error(format!("Failed to start agent session: {}", e)),
    }
}

/// Stop the current agent session
pub fn stop_agent_session(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let manager = get_manager(&mut cx)?;
    let mut manager = manager.lock().unwrap();
    
    match manager.stop_agent_session() {
        Ok(_) => Ok(cx.boolean(true)),
        Err(e) => cx.throw_error(format!("Failed to stop agent session: {}", e)),
    }
}

/// Set the operation mode for the checkpoint system
pub fn set_operation_mode(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let mode_str = cx.argument::<JsString>(0)?.value(&mut cx);
    
    let mode = match mode_str.as_str() {
        "Agent" => OperationMode::Agent,
        "Chat" => OperationMode::Chat,
        "Manual" => OperationMode::Manual,
        _ => return cx.throw_error("Invalid operation mode. Must be 'Agent', 'Chat', or 'Manual'"),
    };
    
    let manager = get_manager(&mut cx)?;
    let mut manager = manager.lock().unwrap();
    
    match manager.set_operation_mode(mode) {
        Ok(_) => Ok(cx.boolean(true)),
        Err(e) => cx.throw_error(format!("Failed to set operation mode: {}", e)),
    }
}

/// Track specific files that AI will modify
pub fn track_ai_files(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let files_array = cx.argument::<JsArray>(0)?;
    
    let manager = get_manager(&mut cx)?;
    let mut manager = manager.lock().unwrap();
    
    // Parse file paths
    let file_paths = parse_path_array(&mut cx, files_array)?;
    
    match manager.track_ai_files(&file_paths) {
        Ok(_) => Ok(cx.boolean(true)),
        Err(e) => cx.throw_error(format!("Failed to track AI files: {}", e)),
    }
}

/// Create a checkpoint using the smart agent system
pub fn create_agent_checkpoint(mut cx: FunctionContext) -> JsResult<JsString> {
    let options_obj = cx.argument::<JsObject>(0)?;
    
    let manager = get_manager(&mut cx)?;
    let mut manager = manager.lock().unwrap();
    
    // Parse options
    let description = get_optional_string(&mut cx, &options_obj, "description")?;
    let tags = get_optional_string_array(&mut cx, &options_obj, "tags")?;
    let max_files = get_optional_number(&mut cx, &options_obj, "maxFiles")?.map(|n| n as usize);
    
    let options = CheckpointOptions {
        description,
        tags: tags.unwrap_or_default(),
        max_files,
        ..Default::default()
    };
    
    match manager.create_agent_checkpoint(options) {
        Ok(checkpoint_id) => Ok(cx.string(checkpoint_id.to_string())),
        Err(e) => cx.throw_error(format!("Failed to create agent checkpoint: {}", e)),
    }
}

/// Check if there are pending AI changes
pub fn has_ai_changes(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let manager = get_manager(&mut cx)?;
    let manager = manager.lock().unwrap();
    
    Ok(cx.boolean(manager.has_ai_changes()))
}

/// Get statistics about the smart checkpoint system
pub fn get_changeset_stats(mut cx: FunctionContext) -> JsResult<JsObject> {
    let manager = get_manager(&mut cx)?;
    let manager = manager.lock().unwrap();
    
    match manager.get_changeset_stats() {
        Some(stats) => {
            let js_obj = cx.empty_object();
            
            if let Some(files_tracked) = stats.get("files_tracked").and_then(|v| v.as_u64()) {
                let files_tracked = cx.number(files_tracked as f64);
                js_obj.set(&mut cx, "filesTracked", files_tracked)?;
            }
            
            if let Some(changes_detected) = stats.get("changes_detected").and_then(|v| v.as_u64()) {
                let changes_detected = cx.number(changes_detected as f64);
                js_obj.set(&mut cx, "changesDetected", changes_detected)?;
            }
            
            if let Some(memory_usage) = stats.get("memory_usage_bytes").and_then(|v| v.as_u64()) {
                let memory_usage = cx.number(memory_usage as f64);
                js_obj.set(&mut cx, "memoryUsageBytes", memory_usage)?;
            }
            
            if let Some(scan_duration) = stats.get("last_scan_duration_ms").and_then(|v| v.as_u64()) {
                let scan_duration = cx.number(scan_duration as f64);
                js_obj.set(&mut cx, "lastScanDurationMs", scan_duration)?;
            }
            
            Ok(js_obj)
        }
        None => {
            let js_obj = cx.empty_object();
            let files_tracked = cx.number(0.0);
            js_obj.set(&mut cx, "filesTracked", files_tracked)?;
            Ok(js_obj)
        }
    }
}
