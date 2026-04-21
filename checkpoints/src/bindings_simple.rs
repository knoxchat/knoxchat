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

    // Helper: safely extract an optional boolean property
    macro_rules! get_opt_bool {
        ($obj:expr, $cx:expr, $key:expr) => {{
            let val = $obj.get::<JsValue, _, _>($cx, $key)?;
            val.downcast::<JsBoolean, _>($cx).ok().map(|b| b.value($cx))
        }};
    }

    // Parse createBackup
    if let Some(v) = get_opt_bool!(options_obj, &mut cx, "createBackup") {
        restore_options.create_backup = v;
    }

    // Parse restorePermissions
    if let Some(v) = get_opt_bool!(options_obj, &mut cx, "restorePermissions") {
        restore_options.restore_permissions = v;
    }

    // Parse restoreTimestamps
    if let Some(v) = get_opt_bool!(options_obj, &mut cx, "restoreTimestamps") {
        restore_options.restore_timestamps = v;
    }

    // Parse validateChecksums
    if let Some(v) = get_opt_bool!(options_obj, &mut cx, "validateChecksums") {
        restore_options.validate_checksums = v;
    }

    // Parse dryRun
    if let Some(v) = get_opt_bool!(options_obj, &mut cx, "dryRun") {
        restore_options.dry_run = v;
    }

    // Parse conflictResolution
    {
        let val = options_obj.get::<JsValue, _, _>(&mut cx, "conflictResolution")?;
        if let Ok(conflict_resolution) = val.downcast::<JsString, _>(&mut cx) {
            let resolution_str = conflict_resolution.value(&mut cx);
            restore_options.conflict_resolution = match resolution_str.as_str() {
                "skip" => ConflictResolution::Skip,
                "overwrite" => ConflictResolution::Overwrite,
                "backup" => ConflictResolution::Backup,
                "prompt" => ConflictResolution::Prompt,
                _ => ConflictResolution::Overwrite, // Default
            };
        }
    }

    // Parse includeFiles
    {
        let val = options_obj.get::<JsValue, _, _>(&mut cx, "includeFiles")?;
        if let Ok(include_files) = val.downcast::<JsArray, _>(&mut cx) {
            let length = include_files.len(&mut cx);
            let mut files = Vec::new();
            for i in 0..length {
                if let Ok(file) = include_files.get::<JsString, _, _>(&mut cx, i) {
                    files.push(PathBuf::from(file.value(&mut cx)));
                }
            }
            restore_options.include_files = files;
        }
    }

    // Parse excludeFiles
    {
        let val = options_obj.get::<JsValue, _, _>(&mut cx, "excludeFiles")?;
        if let Ok(exclude_files) = val.downcast::<JsArray, _>(&mut cx) {
            let length = exclude_files.len(&mut cx);
            let mut files = Vec::new();
            for i in 0..length {
                if let Ok(file) = exclude_files.get::<JsString, _, _>(&mut cx, i) {
                    files.push(PathBuf::from(file.value(&mut cx)));
                }
            }
            restore_options.exclude_files = files;
        }
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
    let limit = if !cx.is_empty() {
        cx.argument::<JsNumber>(0)?.value(&mut cx) as usize
    } else {
        100
    };

    let manager = match create_manager() {
        Ok(m) => m,
        Err(e) => return cx.throw_error(e),
    };

    match manager.list_checkpoints(Some(limit)) {
        Ok(checkpoints) => {
            let arr = JsArray::new(&mut cx, checkpoints.len());
            for (i, cp) in checkpoints.iter().enumerate() {
                let obj = cx.empty_object();
                let id = cx.string(cp.id.to_string());
                obj.set(&mut cx, "id", id)?;
                let desc = cx.string(&cp.description);
                obj.set(&mut cx, "description", desc)?;
                let created = cx.string(cp.created_at.to_rfc3339());
                obj.set(&mut cx, "createdAt", created)?;
                let files = cx.number(cp.files_affected as f64);
                obj.set(&mut cx, "filesAffected", files)?;
                let size = cx.number(cp.size_bytes as f64);
                obj.set(&mut cx, "sizeBytes", size)?;
                let tags_arr = JsArray::new(&mut cx, cp.tags.len());
                for (j, tag) in cp.tags.iter().enumerate() {
                    let t = cx.string(tag);
                    tags_arr.set(&mut cx, j as u32, t)?;
                }
                obj.set(&mut cx, "tags", tags_arr)?;
                arr.set(&mut cx, i as u32, obj)?;
            }
            Ok(arr)
        }
        Err(e) => cx.throw_error(format!("Failed to list checkpoints: {}", e)),
    }
}

pub fn delete_checkpoint(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let checkpoint_id_str = cx.argument::<JsString>(0)?.value(&mut cx);
    let checkpoint_id = match Uuid::parse_str(&checkpoint_id_str) {
        Ok(id) => id,
        Err(_) => return cx.throw_error(format!("Invalid checkpoint ID: {}", checkpoint_id_str)),
    };

    let manager = match create_manager() {
        Ok(m) => m,
        Err(e) => return cx.throw_error(e),
    };

    match manager.delete_checkpoint(&checkpoint_id) {
        Ok(_) => Ok(cx.boolean(true)),
        Err(e) => cx.throw_error(format!("Failed to delete checkpoint: {}", e)),
    }
}

pub fn cleanup_old_checkpoints(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let manager = match create_manager() {
        Ok(m) => m,
        Err(e) => return cx.throw_error(e),
    };

    match manager.cleanup_old_checkpoints() {
        Ok(removed) => Ok(cx.number(removed as f64)),
        Err(e) => cx.throw_error(format!("Failed to cleanup: {}", e)),
    }
}

pub fn get_checkpoint_stats(mut cx: FunctionContext) -> JsResult<JsObject> {
    let manager = match create_manager() {
        Ok(m) => m,
        Err(e) => return cx.throw_error(e),
    };

    match manager.get_stats() {
        Ok(stats) => {
            let result = cx.empty_object();
            let total = cx.number(stats.total_checkpoints as f64);
            result.set(&mut cx, "totalCheckpoints", total)?;
            let sessions = cx.number(stats.total_sessions as f64);
            result.set(&mut cx, "totalSessions", sessions)?;
            let size = cx.number(stats.total_storage_bytes as f64);
            result.set(&mut cx, "totalSizeBytes", size)?;
            let avg = cx.number(stats.avg_checkpoint_size as f64);
            result.set(&mut cx, "avgCheckpointSize", avg)?;
            let ratio = cx.number(stats.compression_ratio);
            result.set(&mut cx, "compressionRatio", ratio)?;
            let dedup = cx.number(stats.deduplication_savings as f64);
            result.set(&mut cx, "deduplicationSavingsBytes", dedup)?;
            Ok(result)
        }
        Err(e) => cx.throw_error(format!("Failed to get stats: {}", e)),
    }
}

/// Run storage garbage collection to remove orphaned content blobs
pub fn run_storage_gc(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let manager = match create_manager() {
        Ok(m) => m,
        Err(e) => return cx.throw_error(e),
    };

    match manager.run_storage_gc() {
        Ok(freed_bytes) => Ok(cx.number(freed_bytes as f64)),
        Err(e) => cx.throw_error(format!("Failed to run storage GC: {}", e)),
    }
}

/// Analyze codebase semantics for a set of file changes
///
/// Arguments:
///   0: JsArray of { path: string, content: string, changeType: string }
///
/// Returns a JsObject with:
///   functions, classes, interfaces, types (counts),
///   imports (list of module strings),
///   layers (affected architectural layers),
///   significance (Low/Medium/High/Critical)
pub fn analyze_semantics(mut cx: FunctionContext) -> JsResult<JsObject> {
    use crate::semantic::analyzer::SemanticAnalyzer;
    use crate::types::{ChangeType, FileChange};

    let files_array = cx.argument::<JsArray>(0)?;
    let len = files_array.len(&mut cx);

    let mut file_changes = Vec::new();
    for i in 0..len {
        let item = files_array.get::<JsObject, _, _>(&mut cx, i)?;
        let path_val = item.get::<JsValue, _, _>(&mut cx, "path")?;
        let path_str = path_val
            .downcast::<JsString, _>(&mut cx)
            .ok()
            .map(|v| v.value(&mut cx))
            .unwrap_or_default();
        let content_val = item.get::<JsValue, _, _>(&mut cx, "content")?;
        let content = content_val
            .downcast::<JsString, _>(&mut cx)
            .ok()
            .map(|v| v.value(&mut cx));
        let ct_val = item.get::<JsValue, _, _>(&mut cx, "changeType")?;
        let change_type_str = ct_val
            .downcast::<JsString, _>(&mut cx)
            .ok()
            .map(|v| v.value(&mut cx))
            .unwrap_or_else(|| "modified".to_string());

        let change_type = match change_type_str.as_str() {
            "created" => ChangeType::Created,
            "deleted" => ChangeType::Deleted,
            _ => ChangeType::Modified,
        };

        file_changes.push(FileChange {
            path: PathBuf::from(&path_str),
            change_type,
            original_content: None,
            new_content: content,
            size_bytes: 0,
            content_hash: String::new(),
            permissions: None,
            modified_at: chrono::Utc::now(),
            encoding: crate::types::FileEncoding::Utf8,
            compressed: false,
        });
    }

    let analyzer = match SemanticAnalyzer::new() {
        Ok(a) => a,
        Err(e) => return cx.throw_error(format!("Failed to create analyzer: {}", e)),
    };

    let context = match analyzer.analyze_codebase(&file_changes) {
        Ok(ctx) => ctx,
        Err(e) => return cx.throw_error(format!("Failed to analyze: {}", e)),
    };

    let impact = analyzer
        .analyze_architectural_impact(&file_changes, &context)
        .ok();

    let result = cx.empty_object();

    // Counts
    let func_count = cx.number(context.functions.len() as f64);
    result.set(&mut cx, "functionCount", func_count)?;
    let class_count = cx.number(context.classes.len() as f64);
    result.set(&mut cx, "classCount", class_count)?;
    let iface_count = cx.number(context.interfaces.len() as f64);
    result.set(&mut cx, "interfaceCount", iface_count)?;
    let type_count = cx.number(context.types.len() as f64);
    result.set(&mut cx, "typeCount", type_count)?;

    // Import modules
    let imports_arr = JsArray::new(&mut cx, context.imports.len());
    for (i, imp) in context.imports.iter().enumerate() {
        let s = cx.string(&imp.module);
        imports_arr.set(&mut cx, i as u32, s)?;
    }
    result.set(&mut cx, "imports", imports_arr)?;

    // Function names
    let func_names = JsArray::new(&mut cx, context.functions.len());
    for (i, name) in context.functions.keys().enumerate() {
        let s = cx.string(name);
        func_names.set(&mut cx, i as u32, s)?;
    }
    result.set(&mut cx, "functionNames", func_names)?;

    // Class names
    let class_names = JsArray::new(&mut cx, context.classes.len());
    for (i, name) in context.classes.keys().enumerate() {
        let s = cx.string(name);
        class_names.set(&mut cx, i as u32, s)?;
    }
    result.set(&mut cx, "classNames", class_names)?;

    // Architectural impact
    if let Some(ref imp) = impact {
        let sig = cx.string(format!("{:?}", imp.significance));
        result.set(&mut cx, "significance", sig)?;

        let layers_arr = JsArray::new(&mut cx, imp.layers_affected.len());
        for (i, layer) in imp.layers_affected.iter().enumerate() {
            let s = cx.string(format!("{:?}", layer));
            layers_arr.set(&mut cx, i as u32, s)?;
        }
        result.set(&mut cx, "layersAffected", layers_arr)?;
    }

    Ok(result)
}

// ========================================
// Phase 8.1: Incremental Checkpoint Bindings
// ========================================

/// Create an incremental checkpoint (only stores changed files vs parent)
pub fn create_incremental_checkpoint(mut cx: FunctionContext) -> JsResult<JsString> {
    let options_obj = cx.argument::<JsObject>(0)?;

    let description = options_obj
        .get::<JsString, _, _>(&mut cx, "description")
        .ok()
        .map(|v| v.value(&mut cx));

    let tags: Vec<String> = options_obj
        .get::<JsArray, _, _>(&mut cx, "tags")
        .ok()
        .map(|arr| {
            let len = arr.len(&mut cx);
            (0..len)
                .filter_map(|i| {
                    arr.get::<JsString, _, _>(&mut cx, i)
                        .ok()
                        .map(|s| s.value(&mut cx))
                })
                .collect()
        })
        .unwrap_or_default();

    let manager = match create_manager() {
        Ok(m) => m,
        Err(e) => return cx.throw_error(e),
    };

    let options = CheckpointOptions {
        description,
        tags,
        ..Default::default()
    };

    match manager.create_incremental_checkpoint(options) {
        Ok(id) => Ok(cx.string(id.to_string())),
        Err(e) => cx.throw_error(format!("Failed to create incremental checkpoint: {}", e)),
    }
}

/// Reconstruct a checkpoint from its delta chain
pub fn reconstruct_checkpoint(mut cx: FunctionContext) -> JsResult<JsObject> {
    let checkpoint_id_str = cx.argument::<JsString>(0)?.value(&mut cx);
    let checkpoint_id = match Uuid::parse_str(&checkpoint_id_str) {
        Ok(id) => id,
        Err(_) => return cx.throw_error("Invalid checkpoint ID"),
    };

    let manager = match create_manager() {
        Ok(m) => m,
        Err(e) => return cx.throw_error(e),
    };

    match manager.reconstruct_checkpoint(&checkpoint_id) {
        Ok(reconstructed) => {
            let result = cx.empty_object();

            let chain_length = cx.number(reconstructed.chain_length as f64);
            result.set(&mut cx, "chainLength", chain_length)?;

            let total_size = cx.number(reconstructed.total_chain_size_bytes as f64);
            result.set(&mut cx, "totalChainSizeBytes", total_size)?;

            let file_count = cx.number(reconstructed.file_states.len() as f64);
            result.set(&mut cx, "fileCount", file_count)?;

            let files_arr = JsArray::new(&mut cx, reconstructed.file_states.len());
            for (i, (path, state)) in reconstructed.file_states.iter().enumerate() {
                let file_obj = cx.empty_object();
                let path_str = cx.string(path.to_string_lossy());
                file_obj.set(&mut cx, "path", path_str)?;
                let hash = cx.string(&state.content_hash);
                file_obj.set(&mut cx, "contentHash", hash)?;
                let size = cx.number(state.size_bytes as f64);
                file_obj.set(&mut cx, "sizeBytes", size)?;
                files_arr.set(&mut cx, i as u32, file_obj)?;
            }
            result.set(&mut cx, "files", files_arr)?;

            Ok(result)
        }
        Err(e) => cx.throw_error(format!("Failed to reconstruct checkpoint: {}", e)),
    }
}

// ========================================
// Phase 8.2: Branch Management Bindings
// ========================================

/// Create a new branch from a checkpoint
pub fn create_branch(mut cx: FunctionContext) -> JsResult<JsObject> {
    let name = cx.argument::<JsString>(0)?.value(&mut cx);
    let base_checkpoint_id_str = cx.argument::<JsString>(1)?.value(&mut cx);
    let description = if cx.len() > 2 {
        cx.argument::<JsString>(2)?.value(&mut cx)
    } else {
        String::new()
    };

    let base_checkpoint_id = match Uuid::parse_str(&base_checkpoint_id_str) {
        Ok(id) => id,
        Err(_) => return cx.throw_error("Invalid checkpoint ID"),
    };

    let manager = match create_manager() {
        Ok(m) => m,
        Err(e) => return cx.throw_error(e),
    };

    match manager.create_branch(&name, &base_checkpoint_id, &description) {
        Ok(branch) => {
            let result = cx.empty_object();
            let id = cx.string(&branch.id);
            result.set(&mut cx, "id", id)?;
            let name = cx.string(&branch.name);
            result.set(&mut cx, "name", name)?;
            let base_id = cx.string(branch.base_checkpoint_id.to_string());
            result.set(&mut cx, "baseCheckpointId", base_id)?;
            let created = cx.string(branch.created_at.to_rfc3339());
            result.set(&mut cx, "createdAt", created)?;
            let desc = cx.string(&branch.description);
            result.set(&mut cx, "description", desc)?;
            let is_default = cx.boolean(branch.is_default);
            result.set(&mut cx, "isDefault", is_default)?;
            Ok(result)
        }
        Err(e) => cx.throw_error(format!("Failed to create branch: {}", e)),
    }
}

/// List all branches
pub fn list_branches(mut cx: FunctionContext) -> JsResult<JsArray> {
    let manager = match create_manager() {
        Ok(m) => m,
        Err(e) => return cx.throw_error(e),
    };

    match manager.list_branches() {
        Ok(branches) => {
            let arr = JsArray::new(&mut cx, branches.len());
            for (i, branch) in branches.iter().enumerate() {
                let obj = cx.empty_object();
                let id = cx.string(&branch.id);
                obj.set(&mut cx, "id", id)?;
                let name = cx.string(&branch.name);
                obj.set(&mut cx, "name", name)?;
                let base_id = cx.string(branch.base_checkpoint_id.to_string());
                obj.set(&mut cx, "baseCheckpointId", base_id)?;
                let head_id_val = if let Some(hid) = branch.head_checkpoint_id {
                    cx.string(hid.to_string()).upcast::<JsValue>()
                } else {
                    cx.null().upcast::<JsValue>()
                };
                obj.set(&mut cx, "headCheckpointId", head_id_val)?;
                let created = cx.string(branch.created_at.to_rfc3339());
                obj.set(&mut cx, "createdAt", created)?;
                let desc = cx.string(&branch.description);
                obj.set(&mut cx, "description", desc)?;
                let is_default = cx.boolean(branch.is_default);
                obj.set(&mut cx, "isDefault", is_default)?;
                arr.set(&mut cx, i as u32, obj)?;
            }
            Ok(arr)
        }
        Err(e) => cx.throw_error(format!("Failed to list branches: {}", e)),
    }
}

/// Switch to a branch
pub fn switch_branch(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let branch_id = cx.argument::<JsString>(0)?.value(&mut cx);

    let manager = match create_manager() {
        Ok(m) => m,
        Err(e) => return cx.throw_error(e),
    };

    match manager.switch_branch(&branch_id) {
        Ok(_) => Ok(cx.boolean(true)),
        Err(e) => cx.throw_error(format!("Failed to switch branch: {}", e)),
    }
}

/// Delete a branch
pub fn delete_branch(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let branch_id = cx.argument::<JsString>(0)?.value(&mut cx);

    let manager = match create_manager() {
        Ok(m) => m,
        Err(e) => return cx.throw_error(e),
    };

    match manager.delete_branch(&branch_id) {
        Ok(_) => Ok(cx.boolean(true)),
        Err(e) => cx.throw_error(format!("Failed to delete branch: {}", e)),
    }
}

/// Merge two branches
pub fn merge_branches(mut cx: FunctionContext) -> JsResult<JsObject> {
    let source_branch_id = cx.argument::<JsString>(0)?.value(&mut cx);
    let target_branch_id = cx.argument::<JsString>(1)?.value(&mut cx);
    let strategy_str = if cx.len() > 2 {
        cx.argument::<JsString>(2)?.value(&mut cx)
    } else {
        "ThreeWay".to_string()
    };

    let strategy = match strategy_str.as_str() {
        "SourceWins" => MergeStrategy::SourceWins,
        "TargetWins" => MergeStrategy::TargetWins,
        _ => MergeStrategy::ThreeWay,
    };

    let manager = match create_manager() {
        Ok(m) => m,
        Err(e) => return cx.throw_error(e),
    };

    match manager.merge_branches(&source_branch_id, &target_branch_id, strategy) {
        Ok(merge_result) => {
            let result = cx.empty_object();
            let success = cx.boolean(merge_result.success);
            result.set(&mut cx, "success", success)?;

            if let Some(merge_id) = merge_result.merge_checkpoint_id {
                let id_str = cx.string(merge_id.to_string());
                result.set(&mut cx, "mergeCheckpointId", id_str)?;
            }

            let merged_arr = JsArray::new(&mut cx, merge_result.merged_files.len());
            for (i, path) in merge_result.merged_files.iter().enumerate() {
                let s = cx.string(path.to_string_lossy());
                merged_arr.set(&mut cx, i as u32, s)?;
            }
            result.set(&mut cx, "mergedFiles", merged_arr)?;

            let conflicts_arr = JsArray::new(&mut cx, merge_result.conflicts.len());
            for (i, conflict) in merge_result.conflicts.iter().enumerate() {
                let obj = cx.empty_object();
                let path = cx.string(conflict.file_path.to_string_lossy());
                obj.set(&mut cx, "filePath", path)?;
                let ctype = cx.string(format!("{:?}", conflict.conflict_type));
                obj.set(&mut cx, "conflictType", ctype)?;
                if let Some(ref src) = conflict.source_content {
                    let s = cx.string(src);
                    obj.set(&mut cx, "sourceContent", s)?;
                }
                if let Some(ref tgt) = conflict.target_content {
                    let s = cx.string(tgt);
                    obj.set(&mut cx, "targetContent", s)?;
                }
                if let Some(ref base) = conflict.base_content {
                    let s = cx.string(base);
                    obj.set(&mut cx, "baseContent", s)?;
                }
                conflicts_arr.set(&mut cx, i as u32, obj)?;
            }
            result.set(&mut cx, "conflicts", conflicts_arr)?;

            let strat = cx.string(format!("{:?}", merge_result.strategy));
            result.set(&mut cx, "strategy", strat)?;

            Ok(result)
        }
        Err(e) => cx.throw_error(format!("Failed to merge branches: {}", e)),
    }
}

// ========================================
// Phase 8.3: AI Analysis Bindings
// ========================================

/// Analyze a checkpoint for risks, impact, and auto-generated description
pub fn analyze_checkpoint(mut cx: FunctionContext) -> JsResult<JsObject> {
    let checkpoint_id_str = cx.argument::<JsString>(0)?.value(&mut cx);
    let checkpoint_id = match Uuid::parse_str(&checkpoint_id_str) {
        Ok(id) => id,
        Err(_) => return cx.throw_error("Invalid checkpoint ID"),
    };

    let manager = match create_manager() {
        Ok(m) => m,
        Err(e) => return cx.throw_error(e),
    };

    match manager.analyze_checkpoint(&checkpoint_id) {
        Ok(analysis) => {
            let result = cx.empty_object();

            // Generated description
            let desc = cx.string(&analysis.generated_description);
            result.set(&mut cx, "generatedDescription", desc)?;

            // Risk assessment
            let risk_obj = cx.empty_object();
            let level = cx.string(format!("{:?}", analysis.risk_assessment.level));
            risk_obj.set(&mut cx, "level", level)?;
            let score = cx.number(analysis.risk_assessment.score);
            risk_obj.set(&mut cx, "score", score)?;

            let factors_arr = JsArray::new(&mut cx, analysis.risk_assessment.factors.len());
            for (i, factor) in analysis.risk_assessment.factors.iter().enumerate() {
                let obj = cx.empty_object();
                let cat = cx.string(format!("{:?}", factor.category));
                obj.set(&mut cx, "category", cat)?;
                let desc = cx.string(&factor.description);
                obj.set(&mut cx, "description", desc)?;
                let weight = cx.number(factor.weight);
                obj.set(&mut cx, "weight", weight)?;
                let files_arr = JsArray::new(&mut cx, factor.affected_files.len());
                for (j, path) in factor.affected_files.iter().enumerate() {
                    let s = cx.string(path.to_string_lossy());
                    files_arr.set(&mut cx, j as u32, s)?;
                }
                obj.set(&mut cx, "affectedFiles", files_arr)?;
                factors_arr.set(&mut cx, i as u32, obj)?;
            }
            risk_obj.set(&mut cx, "factors", factors_arr)?;

            let recs_arr = JsArray::new(&mut cx, analysis.risk_assessment.recommendations.len());
            for (i, rec) in analysis.risk_assessment.recommendations.iter().enumerate() {
                let s = cx.string(rec);
                recs_arr.set(&mut cx, i as u32, s)?;
            }
            risk_obj.set(&mut cx, "recommendations", recs_arr)?;
            result.set(&mut cx, "riskAssessment", risk_obj)?;

            // Impact analysis
            let impact_obj = cx.empty_object();

            let features_arr = JsArray::new(&mut cx, analysis.impact_analysis.affected_features.len());
            for (i, feature) in analysis.impact_analysis.affected_features.iter().enumerate() {
                let obj = cx.empty_object();
                let name = cx.string(&feature.name);
                obj.set(&mut cx, "name", name)?;
                let level = cx.string(format!("{:?}", feature.impact_level));
                obj.set(&mut cx, "impactLevel", level)?;
                let files_arr = JsArray::new(&mut cx, feature.changed_files.len());
                for (j, path) in feature.changed_files.iter().enumerate() {
                    let s = cx.string(path.to_string_lossy());
                    files_arr.set(&mut cx, j as u32, s)?;
                }
                obj.set(&mut cx, "changedFiles", files_arr)?;
                features_arr.set(&mut cx, i as u32, obj)?;
            }
            impact_obj.set(&mut cx, "affectedFeatures", features_arr)?;

            let layers_arr = JsArray::new(&mut cx, analysis.impact_analysis.affected_layers.len());
            for (i, layer) in analysis.impact_analysis.affected_layers.iter().enumerate() {
                let s = cx.string(layer);
                layers_arr.set(&mut cx, i as u32, s)?;
            }
            impact_obj.set(&mut cx, "affectedLayers", layers_arr)?;

            let scope = cx.string(format!("{:?}", analysis.impact_analysis.scope));
            impact_obj.set(&mut cx, "scope", scope)?;

            result.set(&mut cx, "impactAnalysis", impact_obj)?;

            // Grouping suggestion
            if let Some(ref grouping) = analysis.grouping_suggestion {
                let group_obj = cx.empty_object();
                let name = cx.string(&grouping.group_name);
                group_obj.set(&mut cx, "groupName", name)?;
                let rationale = cx.string(&grouping.rationale);
                group_obj.set(&mut cx, "rationale", rationale)?;
                let confidence = cx.number(grouping.confidence);
                group_obj.set(&mut cx, "confidence", confidence)?;
                let ids_arr = JsArray::new(&mut cx, grouping.checkpoint_ids.len());
                for (i, id) in grouping.checkpoint_ids.iter().enumerate() {
                    let s = cx.string(id.to_string());
                    ids_arr.set(&mut cx, i as u32, s)?;
                }
                group_obj.set(&mut cx, "checkpointIds", ids_arr)?;
                result.set(&mut cx, "groupingSuggestion", group_obj)?;
            }

            Ok(result)
        }
        Err(e) => cx.throw_error(format!("Failed to analyze checkpoint: {}", e)),
    }
}

/// Suggest checkpoint groupings
pub fn suggest_checkpoint_groups(mut cx: FunctionContext) -> JsResult<JsArray> {
    let limit = if !cx.is_empty() {
        cx.argument::<JsNumber>(0)?.value(&mut cx) as usize
    } else {
        50
    };

    let manager = match create_manager() {
        Ok(m) => m,
        Err(e) => return cx.throw_error(e),
    };

    match manager.suggest_checkpoint_groups(limit) {
        Ok(suggestions) => {
            let arr = JsArray::new(&mut cx, suggestions.len());
            for (i, suggestion) in suggestions.iter().enumerate() {
                let obj = cx.empty_object();
                let name = cx.string(&suggestion.group_name);
                obj.set(&mut cx, "groupName", name)?;
                let rationale = cx.string(&suggestion.rationale);
                obj.set(&mut cx, "rationale", rationale)?;
                let confidence = cx.number(suggestion.confidence);
                obj.set(&mut cx, "confidence", confidence)?;
                let ids_arr = JsArray::new(&mut cx, suggestion.checkpoint_ids.len());
                for (j, id) in suggestion.checkpoint_ids.iter().enumerate() {
                    let s = cx.string(id.to_string());
                    ids_arr.set(&mut cx, j as u32, s)?;
                }
                obj.set(&mut cx, "checkpointIds", ids_arr)?;
                arr.set(&mut cx, i as u32, obj)?;
            }
            Ok(arr)
        }
        Err(e) => cx.throw_error(format!("Failed to suggest groups: {}", e)),
    }
}

// ========================================
// Phase 8.4: Collaborative Checkpoint Bindings
// ========================================

/// Share checkpoints as a bundle
pub fn share_checkpoints(mut cx: FunctionContext) -> JsResult<JsObject> {
    let ids_arr = cx.argument::<JsArray>(0)?;
    let description = cx.argument::<JsString>(1)?.value(&mut cx);

    let len = ids_arr.len(&mut cx);
    let mut checkpoint_ids = Vec::new();
    for i in 0..len {
        let id_str = ids_arr.get::<JsString, _, _>(&mut cx, i)?.value(&mut cx);
        match Uuid::parse_str(&id_str) {
            Ok(id) => checkpoint_ids.push(id),
            Err(_) => return cx.throw_error(format!("Invalid checkpoint ID: {}", id_str)),
        }
    }

    let manager = match create_manager() {
        Ok(m) => m,
        Err(e) => return cx.throw_error(e),
    };

    match manager.share_checkpoints(&checkpoint_ids, &description) {
        Ok(bundle) => {
            let result = cx.empty_object();
            let id = cx.string(&bundle.id);
            result.set(&mut cx, "id", id)?;
            let desc = cx.string(&bundle.description);
            result.set(&mut cx, "description", desc)?;
            let shared_at = cx.string(bundle.shared_at.to_rfc3339());
            result.set(&mut cx, "sharedAt", shared_at)?;
            let count = cx.number(bundle.checkpoint_ids.len() as f64);
            result.set(&mut cx, "checkpointCount", count)?;
            let shared_by = cx.string(&bundle.shared_by.display_name);
            result.set(&mut cx, "sharedBy", shared_by)?;
            Ok(result)
        }
        Err(e) => cx.throw_error(format!("Failed to share checkpoints: {}", e)),
    }
}

/// List shared checkpoint bundles
pub fn list_shared_bundles(mut cx: FunctionContext) -> JsResult<JsArray> {
    let manager = match create_manager() {
        Ok(m) => m,
        Err(e) => return cx.throw_error(e),
    };

    match manager.list_shared_bundles() {
        Ok(bundles) => {
            let arr = JsArray::new(&mut cx, bundles.len());
            for (i, bundle) in bundles.iter().enumerate() {
                let obj = cx.empty_object();
                let id = cx.string(&bundle.id);
                obj.set(&mut cx, "id", id)?;
                let desc = cx.string(&bundle.description);
                obj.set(&mut cx, "description", desc)?;
                let shared_at = cx.string(bundle.shared_at.to_rfc3339());
                obj.set(&mut cx, "sharedAt", shared_at)?;
                let count = cx.number(bundle.checkpoint_ids.len() as f64);
                obj.set(&mut cx, "checkpointCount", count)?;
                let shared_by = cx.string(&bundle.shared_by.display_name);
                obj.set(&mut cx, "sharedBy", shared_by)?;
                let machine = cx.string(&bundle.shared_by.machine_id);
                obj.set(&mut cx, "machineId", machine)?;
                arr.set(&mut cx, i as u32, obj)?;
            }
            Ok(arr)
        }
        Err(e) => cx.throw_error(format!("Failed to list shared bundles: {}", e)),
    }
}

/// Get compliance audit trail
pub fn get_audit_trail(mut cx: FunctionContext) -> JsResult<JsArray> {
    let limit = if !cx.is_empty() {
        cx.argument::<JsNumber>(0)?.value(&mut cx) as usize
    } else {
        100
    };

    let action_filter: Option<String> = if cx.len() > 1 {
        cx.argument::<JsString>(1).ok().map(|s| s.value(&mut cx))
    } else {
        None
    };

    let manager = match create_manager() {
        Ok(m) => m,
        Err(e) => return cx.throw_error(e),
    };

    match manager.get_audit_trail(limit, action_filter.as_deref()) {
        Ok(records) => {
            let arr = JsArray::new(&mut cx, records.len());
            for (i, record) in records.iter().enumerate() {
                let obj = cx.empty_object();
                let id = cx.string(&record.id);
                obj.set(&mut cx, "id", id)?;
                let ts = cx.string(record.timestamp.to_rfc3339());
                obj.set(&mut cx, "timestamp", ts)?;
                let user = cx.string(&record.user_id);
                obj.set(&mut cx, "userId", user)?;
                let machine = cx.string(&record.machine_id);
                obj.set(&mut cx, "machineId", machine)?;
                let action = cx.string(&record.action);
                obj.set(&mut cx, "action", action)?;
                let res_type = cx.string(&record.resource_type);
                obj.set(&mut cx, "resourceType", res_type)?;
                let res_id = cx.string(&record.resource_id);
                obj.set(&mut cx, "resourceId", res_id)?;
                let outcome = cx.string(format!("{:?}", record.outcome));
                obj.set(&mut cx, "outcome", outcome)?;
                let details = cx.string(serde_json::to_string(&record.details).unwrap_or_default());
                obj.set(&mut cx, "details", details)?;
                arr.set(&mut cx, i as u32, obj)?;
            }
            Ok(arr)
        }
        Err(e) => cx.throw_error(format!("Failed to get audit trail: {}", e)),
    }
}

// ========================================
// Phase 8.5: Performance Monitoring Bindings
// ========================================

/// Get the full performance dashboard
pub fn get_performance_dashboard(mut cx: FunctionContext) -> JsResult<JsObject> {
    let history_days = if !cx.is_empty() {
        cx.argument::<JsNumber>(0)?.value(&mut cx) as u32
    } else {
        30
    };

    let manager = match create_manager() {
        Ok(m) => m,
        Err(e) => return cx.throw_error(e),
    };

    match manager.get_performance_dashboard(history_days) {
        Ok(dashboard) => {
            let result = cx.empty_object();

            // Current storage
            let storage_obj = cx.empty_object();
            let total = cx.number(dashboard.current_storage.total_bytes as f64);
            storage_obj.set(&mut cx, "totalBytes", total)?;
            let cp_data = cx.number(dashboard.current_storage.checkpoint_data_bytes as f64);
            storage_obj.set(&mut cx, "checkpointDataBytes", cp_data)?;
            let db_bytes = cx.number(dashboard.current_storage.database_bytes as f64);
            storage_obj.set(&mut cx, "databaseBytes", db_bytes)?;
            let blobs = cx.number(dashboard.current_storage.blob_count as f64);
            storage_obj.set(&mut cx, "blobCount", blobs)?;
            let cp_count = cx.number(dashboard.current_storage.checkpoint_count as f64);
            storage_obj.set(&mut cx, "checkpointCount", cp_count)?;
            result.set(&mut cx, "currentStorage", storage_obj)?;

            // Storage history
            let history_arr = JsArray::new(&mut cx, dashboard.storage_history.len());
            for (i, snap) in dashboard.storage_history.iter().enumerate() {
                let obj = cx.empty_object();
                let ts = cx.string(snap.timestamp.to_rfc3339());
                obj.set(&mut cx, "timestamp", ts)?;
                let total = cx.number(snap.total_bytes as f64);
                obj.set(&mut cx, "totalBytes", total)?;
                let cp = cx.number(snap.checkpoint_count as f64);
                obj.set(&mut cx, "checkpointCount", cp)?;
                history_arr.set(&mut cx, i as u32, obj)?;
            }
            result.set(&mut cx, "storageHistory", history_arr)?;

            // Creation frequency
            let freq_arr = JsArray::new(&mut cx, dashboard.creation_frequency.len());
            for (i, point) in dashboard.creation_frequency.iter().enumerate() {
                let obj = cx.empty_object();
                let bucket = cx.string(point.bucket.to_rfc3339());
                obj.set(&mut cx, "bucket", bucket)?;
                let count = cx.number(point.count as f64);
                obj.set(&mut cx, "count", count)?;
                freq_arr.set(&mut cx, i as u32, obj)?;
            }
            result.set(&mut cx, "creationFrequency", freq_arr)?;

            // Restoration events
            let events_arr = JsArray::new(&mut cx, dashboard.restoration_events.len());
            for (i, event) in dashboard.restoration_events.iter().enumerate() {
                let obj = cx.empty_object();
                let ts = cx.string(event.timestamp.to_rfc3339());
                obj.set(&mut cx, "timestamp", ts)?;
                let cp_id = cx.string(event.checkpoint_id.to_string());
                obj.set(&mut cx, "checkpointId", cp_id)?;
                let success = cx.boolean(event.success);
                obj.set(&mut cx, "success", success)?;
                let dur = cx.number(event.duration_ms);
                obj.set(&mut cx, "durationMs", dur)?;
                let restored = cx.number(event.files_restored as f64);
                obj.set(&mut cx, "filesRestored", restored)?;
                let failed = cx.number(event.files_failed as f64);
                obj.set(&mut cx, "filesFailed", failed)?;
                if let Some(ref err) = event.error {
                    let e = cx.string(err);
                    obj.set(&mut cx, "error", e)?;
                }
                events_arr.set(&mut cx, i as u32, obj)?;
            }
            result.set(&mut cx, "restorationEvents", events_arr)?;

            // AI session metrics
            let sessions_arr = JsArray::new(&mut cx, dashboard.ai_session_metrics.len());
            for (i, metrics) in dashboard.ai_session_metrics.iter().enumerate() {
                let obj = cx.empty_object();
                let sid = cx.string(metrics.session_id.to_string());
                obj.set(&mut cx, "sessionId", sid)?;
                let started = cx.string(metrics.started_at.to_rfc3339());
                obj.set(&mut cx, "startedAt", started)?;
                if let Some(ended) = metrics.ended_at {
                    let e = cx.string(ended.to_rfc3339());
                    obj.set(&mut cx, "endedAt", e)?;
                }
                let fc = cx.number(metrics.files_changed as f64);
                obj.set(&mut cx, "filesChanged", fc)?;
                let la = cx.number(metrics.lines_added as f64);
                obj.set(&mut cx, "linesAdded", la)?;
                let ld = cx.number(metrics.lines_deleted as f64);
                obj.set(&mut cx, "linesDeleted", ld)?;
                let cc = cx.number(metrics.checkpoints_created as f64);
                obj.set(&mut cx, "checkpointsCreated", cc)?;
                let rb = cx.number(metrics.rollbacks as f64);
                obj.set(&mut cx, "rollbacks", rb)?;
                let dur = cx.number(metrics.duration_seconds);
                obj.set(&mut cx, "durationSeconds", dur)?;
                sessions_arr.set(&mut cx, i as u32, obj)?;
            }
            result.set(&mut cx, "aiSessionMetrics", sessions_arr)?;

            // Summary
            let summary = cx.empty_object();
            let tc = cx.number(dashboard.summary.total_checkpoints_created as f64);
            summary.set(&mut cx, "totalCheckpointsCreated", tc)?;
            let tr = cx.number(dashboard.summary.total_restorations as f64);
            summary.set(&mut cx, "totalRestorations", tr)?;
            let rsr = cx.number(dashboard.summary.restoration_success_rate);
            summary.set(&mut cx, "restorationSuccessRate", rsr)?;
            let act = cx.number(dashboard.summary.avg_creation_time_ms);
            summary.set(&mut cx, "avgCreationTimeMs", act)?;
            let art = cx.number(dashboard.summary.avg_restoration_time_ms);
            summary.set(&mut cx, "avgRestorationTimeMs", art)?;
            let tas = cx.number(dashboard.summary.total_ai_sessions as f64);
            summary.set(&mut cx, "totalAiSessions", tas)?;
            let acps = cx.number(dashboard.summary.avg_changes_per_session);
            summary.set(&mut cx, "avgChangesPerSession", acps)?;
            let trb = cx.number(dashboard.summary.total_rollbacks as f64);
            summary.set(&mut cx, "totalRollbacks", trb)?;
            result.set(&mut cx, "summary", summary)?;

            Ok(result)
        }
        Err(e) => cx.throw_error(format!("Failed to get dashboard: {}", e)),
    }
}

/// Get current storage usage
pub fn get_storage_usage(mut cx: FunctionContext) -> JsResult<JsObject> {
    let manager = match create_manager() {
        Ok(m) => m,
        Err(e) => return cx.throw_error(e),
    };

    match manager.get_storage_usage() {
        Ok(snapshot) => {
            let result = cx.empty_object();
            let ts = cx.string(snapshot.timestamp.to_rfc3339());
            result.set(&mut cx, "timestamp", ts)?;
            let total = cx.number(snapshot.total_bytes as f64);
            result.set(&mut cx, "totalBytes", total)?;
            let cp_data = cx.number(snapshot.checkpoint_data_bytes as f64);
            result.set(&mut cx, "checkpointDataBytes", cp_data)?;
            let db_bytes = cx.number(snapshot.database_bytes as f64);
            result.set(&mut cx, "databaseBytes", db_bytes)?;
            let blobs = cx.number(snapshot.blob_count as f64);
            result.set(&mut cx, "blobCount", blobs)?;
            let cp_count = cx.number(snapshot.checkpoint_count as f64);
            result.set(&mut cx, "checkpointCount", cp_count)?;
            Ok(result)
        }
        Err(e) => cx.throw_error(format!("Failed to get storage usage: {}", e)),
    }
}

/// Record a storage snapshot for trend tracking
pub fn record_storage_snapshot(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let manager = match create_manager() {
        Ok(m) => m,
        Err(e) => return cx.throw_error(e),
    };

    match manager.record_storage_snapshot() {
        Ok(_) => Ok(cx.boolean(true)),
        Err(e) => cx.throw_error(format!("Failed to record snapshot: {}", e)),
    }
}

/// Record a restoration event for performance monitoring.
pub fn record_restoration_event(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let event_obj = cx.argument::<JsObject>(0)?;

    let checkpoint_id = event_obj
        .get::<JsString, _, _>(&mut cx, "checkpointId")?
        .value(&mut cx);
    let checkpoint_id = match Uuid::parse_str(&checkpoint_id) {
        Ok(id) => id,
        Err(_) => return cx.throw_error("Invalid checkpointId for restoration event"),
    };

    let success = event_obj
        .get::<JsBoolean, _, _>(&mut cx, "success")?
        .value(&mut cx);
    let duration_ms = event_obj
        .get::<JsNumber, _, _>(&mut cx, "durationMs")?
        .value(&mut cx);
    let files_restored = event_obj
        .get::<JsNumber, _, _>(&mut cx, "filesRestored")?
        .value(&mut cx) as u64;
    let files_failed = event_obj
        .get::<JsNumber, _, _>(&mut cx, "filesFailed")?
        .value(&mut cx) as u64;

    let timestamp = {
        let val = event_obj.get::<JsValue, _, _>(&mut cx, "timestamp")?;
        if let Ok(ts) = val.downcast::<JsString, _>(&mut cx) {
            chrono::DateTime::parse_from_rfc3339(&ts.value(&mut cx))
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now())
        } else {
            chrono::Utc::now()
        }
    };

    let error = {
        let val = event_obj.get::<JsValue, _, _>(&mut cx, "error")?;
        if let Ok(s) = val.downcast::<JsString, _>(&mut cx) {
            Some(s.value(&mut cx))
        } else {
            None
        }
    };

    let manager = match create_manager() {
        Ok(m) => m,
        Err(e) => return cx.throw_error(e),
    };

    let event = RestorationEvent {
        timestamp,
        checkpoint_id,
        success,
        duration_ms,
        files_restored,
        files_failed,
        error,
    };

    match manager.record_restoration_event(&event) {
        Ok(_) => Ok(cx.boolean(true)),
        Err(e) => cx.throw_error(format!("Failed to record restoration event: {}", e)),
    }
}

/// Record AI session metrics for performance monitoring.
pub fn record_ai_session_metrics(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let metrics_obj = cx.argument::<JsObject>(0)?;

    let session_id = metrics_obj
        .get::<JsString, _, _>(&mut cx, "sessionId")?
        .value(&mut cx);
    let session_id = match Uuid::parse_str(&session_id) {
        Ok(id) => id,
        Err(_) => return cx.throw_error("Invalid sessionId for AI session metrics"),
    };

    let started_at = metrics_obj
        .get::<JsString, _, _>(&mut cx, "startedAt")?
        .value(&mut cx);
    let started_at = match chrono::DateTime::parse_from_rfc3339(&started_at) {
        Ok(value) => value.with_timezone(&chrono::Utc),
        Err(error) => {
            return cx.throw_error(format!(
                "Invalid startedAt timestamp for AI session metrics: {}",
                error
            ))
        }
    };

    let ended_at = {
        let val = metrics_obj.get::<JsValue, _, _>(&mut cx, "endedAt")?;
        if let Ok(value) = val.downcast::<JsString, _>(&mut cx) {
            let ended_at_raw = value.value(&mut cx);
            Some(match chrono::DateTime::parse_from_rfc3339(&ended_at_raw) {
                Ok(parsed) => parsed.with_timezone(&chrono::Utc),
                Err(error) => {
                    return cx.throw_error(format!(
                        "Invalid endedAt timestamp for AI session metrics: {}",
                        error
                    ))
                }
            })
        } else {
            None
        }
    };

    let files_changed = metrics_obj
        .get::<JsNumber, _, _>(&mut cx, "filesChanged")?
        .value(&mut cx) as u64;
    let lines_added = metrics_obj
        .get::<JsNumber, _, _>(&mut cx, "linesAdded")?
        .value(&mut cx) as u64;
    let lines_deleted = metrics_obj
        .get::<JsNumber, _, _>(&mut cx, "linesDeleted")?
        .value(&mut cx) as u64;
    let checkpoints_created = metrics_obj
        .get::<JsNumber, _, _>(&mut cx, "checkpointsCreated")?
        .value(&mut cx) as u64;
    let rollbacks = metrics_obj
        .get::<JsNumber, _, _>(&mut cx, "rollbacks")?
        .value(&mut cx) as u64;
    let duration_seconds = metrics_obj
        .get::<JsNumber, _, _>(&mut cx, "durationSeconds")?
        .value(&mut cx);

    let manager = match create_manager() {
        Ok(m) => m,
        Err(e) => return cx.throw_error(e),
    };

    let metrics = AISessionMetrics {
        session_id,
        started_at,
        ended_at,
        files_changed,
        lines_added,
        lines_deleted,
        checkpoints_created,
        rollbacks,
        duration_seconds,
    };

    match manager.record_ai_session_metrics(&metrics) {
        Ok(_) => Ok(cx.boolean(true)),
        Err(e) => cx.throw_error(format!("Failed to record AI session metrics: {}", e)),
    }
}

// ========================================
// Phase 2.6: Semantic Analysis NAPI Bindings
// ========================================

/// Analyze the semantic diff between two sets of file changes.
///
/// Arguments:
///   0: JsArray of { path: string, content: string, changeType: string }
///
/// Returns a JsObject with intent analysis, code relationships, and architectural impact.
pub fn analyze_checkpoint_diff(mut cx: FunctionContext) -> JsResult<JsObject> {
    use crate::semantic::analyzer::SemanticAnalyzer;

    let files_array = cx.argument::<JsArray>(0)?;
    let file_changes = parse_file_changes_array(&mut cx, &files_array)?;

    let analyzer = match SemanticAnalyzer::new() {
        Ok(a) => a,
        Err(e) => return cx.throw_error(format!("Failed to create analyzer: {}", e)),
    };

    let context = match analyzer.analyze_codebase(&file_changes) {
        Ok(ctx) => ctx,
        Err(e) => return cx.throw_error(format!("Analyze failed: {}", e)),
    };

    let result = cx.empty_object();

    // Intent analysis
    if let Ok(intent) = analyzer.analyze_intent(&file_changes, &context) {
        let intent_obj = cx.empty_object();
        let change_intent = cx.string(format!("{:?}", intent.change_intent));
        intent_obj.set(&mut cx, "changeIntent", change_intent)?;

        let features = JsArray::new(&mut cx, intent.affected_features.len());
        for (i, feat) in intent.affected_features.iter().enumerate() {
            let s = cx.string(feat);
            features.set(&mut cx, i as u32, s)?;
        }
        intent_obj.set(&mut cx, "affectedFeatures", features)?;

        let confidence = cx.number(intent.confidence);
        intent_obj.set(&mut cx, "confidence", confidence)?;

        if let Some(ref refactoring) = intent.refactoring_type {
            let rt = cx.string(format!("{:?}", refactoring));
            intent_obj.set(&mut cx, "refactoringType", rt)?;
        }

        result.set(&mut cx, "intent", intent_obj)?;
    }

    // Code relationships
    if let Ok(rels) = analyzer.build_code_relationships(&file_changes, &context) {
        let rels_obj = cx.empty_object();

        let direct = JsArray::new(&mut cx, rels.direct_dependencies.len());
        for (i, dep) in rels.direct_dependencies.iter().enumerate() {
            let s = cx.string(dep);
            direct.set(&mut cx, i as u32, s)?;
        }
        rels_obj.set(&mut cx, "directDependencies", direct)?;

        let transitive = JsArray::new(&mut cx, rels.transitive_dependencies.len());
        for (i, dep) in rels.transitive_dependencies.iter().enumerate() {
            let s = cx.string(dep);
            transitive.set(&mut cx, i as u32, s)?;
        }
        rels_obj.set(&mut cx, "transitiveDependencies", transitive)?;

        let dependents = JsArray::new(&mut cx, rels.dependents.len());
        for (i, dep) in rels.dependents.iter().enumerate() {
            let s = cx.string(dep);
            dependents.set(&mut cx, i as u32, s)?;
        }
        rels_obj.set(&mut cx, "dependents", dependents)?;

        result.set(&mut cx, "relationships", rels_obj)?;
    }

    // Architectural impact
    if let Ok(impact) = analyzer.analyze_architectural_impact(&file_changes, &context) {
        let impact_obj = cx.empty_object();
        let sig = cx.string(format!("{:?}", impact.significance));
        impact_obj.set(&mut cx, "significance", sig)?;

        let layers = JsArray::new(&mut cx, impact.layers_affected.len());
        for (i, layer) in impact.layers_affected.iter().enumerate() {
            let s = cx.string(format!("{:?}", layer));
            layers.set(&mut cx, i as u32, s)?;
        }
        impact_obj.set(&mut cx, "layersAffected", layers)?;

        result.set(&mut cx, "architecturalImpact", impact_obj)?;
    }

    Ok(result)
}

/// Get a symbol map for a given file content.
///
/// Arguments:
///   0: JsString — file path (used for language detection)
///   1: JsString — file content
///
/// Returns a JsObject with extracted symbols (functions, classes, interfaces, types).
pub fn get_symbol_map(mut cx: FunctionContext) -> JsResult<JsObject> {
    use crate::semantic::analyzer::SemanticAnalyzer;

    let file_path = cx.argument::<JsString>(0)?.value(&mut cx);
    let file_content = cx.argument::<JsString>(1)?.value(&mut cx);

    let analyzer = match SemanticAnalyzer::new() {
        Ok(a) => a,
        Err(e) => return cx.throw_error(format!("Failed to create analyzer: {}", e)),
    };

    let file_changes = vec![FileChange {
        path: PathBuf::from(&file_path),
        change_type: ChangeType::Modified,
        original_content: None,
        new_content: Some(file_content),
        size_bytes: 0,
        content_hash: String::new(),
        permissions: None,
        modified_at: chrono::Utc::now(),
        encoding: crate::types::FileEncoding::Utf8,
        compressed: false,
    }];

    let context = match analyzer.analyze_codebase(&file_changes) {
        Ok(ctx) => ctx,
        Err(e) => return cx.throw_error(format!("Analyze failed: {}", e)),
    };

    let result = cx.empty_object();

    // Functions
    let func_arr = JsArray::new(&mut cx, context.functions.len());
    for (i, (name, _)) in context.functions.iter().enumerate() {
        let s = cx.string(name);
        func_arr.set(&mut cx, i as u32, s)?;
    }
    result.set(&mut cx, "functions", func_arr)?;

    // Classes
    let class_arr = JsArray::new(&mut cx, context.classes.len());
    for (i, (name, _)) in context.classes.iter().enumerate() {
        let s = cx.string(name);
        class_arr.set(&mut cx, i as u32, s)?;
    }
    result.set(&mut cx, "classes", class_arr)?;

    // Interfaces
    let iface_arr = JsArray::new(&mut cx, context.interfaces.len());
    for (i, (name, _)) in context.interfaces.iter().enumerate() {
        let s = cx.string(name);
        iface_arr.set(&mut cx, i as u32, s)?;
    }
    result.set(&mut cx, "interfaces", iface_arr)?;

    // Types
    let type_arr = JsArray::new(&mut cx, context.types.len());
    for (i, (name, _)) in context.types.iter().enumerate() {
        let s = cx.string(name);
        type_arr.set(&mut cx, i as u32, s)?;
    }
    result.set(&mut cx, "types", type_arr)?;

    // Imports
    let imports_arr = JsArray::new(&mut cx, context.imports.len());
    for (i, imp) in context.imports.iter().enumerate() {
        let s = cx.string(&imp.module);
        imports_arr.set(&mut cx, i as u32, s)?;
    }
    result.set(&mut cx, "imports", imports_arr)?;

    // Exports
    let export_names: Vec<String> = context
        .exports
        .iter()
        .flat_map(|exp| {
            if exp.exported_items.is_empty() {
                exp.alias.clone().into_iter().collect::<Vec<String>>()
            } else {
                exp.exported_items.clone()
            }
        })
        .collect();
    let exports_arr = JsArray::new(&mut cx, export_names.len());
    for (i, name) in export_names.iter().enumerate() {
        let s = cx.string(name);
        exports_arr.set(&mut cx, i as u32, s)?;
    }
    result.set(&mut cx, "exports", exports_arr)?;

    Ok(result)
}

/// Get a dependency graph for a set of files.
///
/// Arguments:
///   0: JsArray of { path: string, content: string, changeType: string }
///
/// Returns a JsObject with nodes, edges, and cycles.
pub fn get_dependency_graph(mut cx: FunctionContext) -> JsResult<JsObject> {
    use crate::semantic::analyzer::SemanticAnalyzer;
    use crate::semantic::RelationshipMapper;

    let files_array = cx.argument::<JsArray>(0)?;
    let file_changes = parse_file_changes_array(&mut cx, &files_array)?;

    let analyzer = match SemanticAnalyzer::new() {
        Ok(a) => a,
        Err(e) => return cx.throw_error(format!("Failed to create analyzer: {}", e)),
    };

    let context = match analyzer.analyze_codebase(&file_changes) {
        Ok(ctx) => ctx,
        Err(e) => return cx.throw_error(format!("Analyze failed: {}", e)),
    };

    let mapper = RelationshipMapper::new();
    let graph = match mapper.build_dependency_graph(&file_changes, &context) {
        Ok(g) => g,
        Err(e) => return cx.throw_error(format!("Graph build failed: {}", e)),
    };

    let result = cx.empty_object();

    // Nodes
    let nodes_arr = JsArray::new(&mut cx, graph.nodes.len());
    for (i, node) in graph.nodes.iter().enumerate() {
        let node_obj = cx.empty_object();
        let id = cx.string(&node.id);
        node_obj.set(&mut cx, "id", id)?;
        let node_type = cx.string(format!("{:?}", node.node_type));
        node_obj.set(&mut cx, "type", node_type)?;
        nodes_arr.set(&mut cx, i as u32, node_obj)?;
    }
    result.set(&mut cx, "nodes", nodes_arr)?;

    // Edges
    let edges_arr = JsArray::new(&mut cx, graph.edges.len());
    for (i, edge) in graph.edges.iter().enumerate() {
        let edge_obj = cx.empty_object();
        let from = cx.string(&edge.from);
        edge_obj.set(&mut cx, "from", from)?;
        let to = cx.string(&edge.to);
        edge_obj.set(&mut cx, "to", to)?;
        let edge_type = cx.string(format!("{:?}", edge.edge_type));
        edge_obj.set(&mut cx, "type", edge_type)?;
        let strength = cx.number(edge.strength);
        edge_obj.set(&mut cx, "strength", strength)?;
        edges_arr.set(&mut cx, i as u32, edge_obj)?;
    }
    result.set(&mut cx, "edges", edges_arr)?;

    // Cycles
    let cycles_arr = JsArray::new(&mut cx, graph.cycles.len());
    for (i, cycle) in graph.cycles.iter().enumerate() {
        let cycle_arr = JsArray::new(&mut cx, cycle.len());
        for (j, node_id) in cycle.iter().enumerate() {
            let s = cx.string(node_id);
            cycle_arr.set(&mut cx, j as u32, s)?;
        }
        cycles_arr.set(&mut cx, i as u32, cycle_arr)?;
    }
    result.set(&mut cx, "cycles", cycles_arr)?;

    Ok(result)
}

/// Helper: parse a JsArray of file change objects into Vec<FileChange>
fn parse_file_changes_array(cx: &mut FunctionContext, arr: &JsArray) -> NeonResult<Vec<FileChange>> {

    let len = arr.len(cx);
    let mut file_changes = Vec::with_capacity(len as usize);

    for i in 0..len {
        let item = arr.get::<JsObject, _, _>(cx, i)?;
        let path_str = item
            .get::<JsValue, _, _>(cx, "path")?
            .downcast::<JsString, _>(cx)
            .ok()
            .map(|v| v.value(cx))
            .unwrap_or_default();
        let content = item
            .get::<JsValue, _, _>(cx, "content")?
            .downcast::<JsString, _>(cx)
            .ok()
            .map(|v| v.value(cx));
        let change_type_str = item
            .get::<JsValue, _, _>(cx, "changeType")?
            .downcast::<JsString, _>(cx)
            .ok()
            .map(|v| v.value(cx))
            .unwrap_or_else(|| "modified".to_string());

        let change_type = match change_type_str.as_str() {
            "created" => ChangeType::Created,
            "deleted" => ChangeType::Deleted,
            _ => ChangeType::Modified,
        };

        file_changes.push(FileChange {
            path: PathBuf::from(&path_str),
            change_type,
            original_content: None,
            new_content: content,
            size_bytes: 0,
            content_hash: String::new(),
            permissions: None,
            modified_at: chrono::Utc::now(),
            encoding: crate::types::FileEncoding::Utf8,
            compressed: false,
        });
    }

    Ok(file_changes)
}
