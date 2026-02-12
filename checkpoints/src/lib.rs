//! # Knox Enterprise Checkpoint System
//!
//! A powerful checkpoint system that automatically tracks and manages file changes
//! made by AI agents, enabling users to restore their codebase to previous states.

pub mod bindings_simple;
pub mod changeset_tracker;
pub mod config;
pub mod db;
pub mod enterprise;
pub mod error;
pub mod file_tracker;
pub mod manager;
pub mod restoration;
pub mod semantic;
pub mod storage;
pub mod types;
pub mod utils;

// Re-exports for convenience
pub use changeset_tracker::{ChangesetTracker, OperationMode};
pub use config::CheckpointConfig;
pub use db::CheckpointDatabase;
pub use error::{CheckpointError, Result};
pub use file_tracker::FileTracker;
pub use manager::CheckpointManager;
pub use restoration::CheckpointRestoration;
pub use semantic::{types::*, SemanticAnalyzer};
pub use storage::CheckpointStorage;
pub use types::*;

use neon::prelude::*;
use once_cell::sync::OnceCell;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Global configuration storage
static GLOBAL_CONFIG: OnceCell<Arc<Mutex<CheckpointConfig>>> = OnceCell::new();

/// Get the global configuration, or default if not set
fn get_config_or_default() -> CheckpointConfig {
    if let Some(config_arc) = GLOBAL_CONFIG.get() {
        if let Ok(config) = config_arc.lock() {
            return config.clone();
        }
    }
    CheckpointConfig::default()
}

/// Set the global configuration from Node.js
fn set_config(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let config_obj = cx.argument::<JsObject>(0)?;

    let mut config = CheckpointConfig::default();

    // Extract configuration values from the JavaScript object
    if let Ok(max_checkpoints) = config_obj.get::<JsNumber, _, _>(&mut cx, "maxCheckpoints") {
        config.max_checkpoints = max_checkpoints.value(&mut cx) as usize;
    }

    if let Ok(retention_days) = config_obj.get::<JsNumber, _, _>(&mut cx, "retentionDays") {
        config.retention_days = retention_days.value(&mut cx) as i64;
    }

    if let Ok(max_storage_bytes) = config_obj.get::<JsNumber, _, _>(&mut cx, "maxStorageBytes") {
        config.max_storage_bytes = max_storage_bytes.value(&mut cx) as u64;
    }

    if let Ok(max_files_per_checkpoint) =
        config_obj.get::<JsNumber, _, _>(&mut cx, "maxFilesPerCheckpoint")
    {
        config.max_files_per_checkpoint = max_files_per_checkpoint.value(&mut cx) as usize;
    }

    if let Ok(enable_compression) = config_obj.get::<JsBoolean, _, _>(&mut cx, "enableCompression")
    {
        config.enable_compression = enable_compression.value(&mut cx);
    }

    // Parse tracked extensions array
    if let Ok(tracked_extensions_array) =
        config_obj.get::<JsArray, _, _>(&mut cx, "trackedExtensions")
    {
        let length = tracked_extensions_array.len(&mut cx);
        let mut tracked_extensions = Vec::new();

        for i in 0..length {
            if let Ok(extension) = tracked_extensions_array.get::<JsString, _, _>(&mut cx, i) {
                tracked_extensions.push(extension.value(&mut cx));
            }
        }

        config.tracked_extensions = tracked_extensions;
    }

    // Parse auto cleanup setting
    if let Ok(auto_cleanup) = config_obj.get::<JsBoolean, _, _>(&mut cx, "autoCleanup") {
        config.auto_cleanup = auto_cleanup.value(&mut cx);
    }

    // Parse cleanup interval hours
    if let Ok(cleanup_interval_hours) =
        config_obj.get::<JsNumber, _, _>(&mut cx, "cleanupIntervalHours")
    {
        config.cleanup_interval_hours = cleanup_interval_hours.value(&mut cx) as u64;
    }

    // Store the configuration globally
    let config_arc = Arc::new(Mutex::new(config.clone()));
    if GLOBAL_CONFIG.set(config_arc).is_err() {
        // If already set, update the existing config
        if let Some(existing_config_arc) = GLOBAL_CONFIG.get() {
            if let Ok(mut existing_config) = existing_config_arc.lock() {
                *existing_config = config;
            }
        }
    }

    Ok(cx.boolean(true))
}

/// Simple checkpoint creation function for Node.js
fn create_simple_checkpoint(mut cx: FunctionContext) -> JsResult<JsString> {
    let description = cx.argument::<JsString>(0)?.value(&mut cx);

    // Create a checkpoint manager with configured values
    let config = get_config_or_default();
    let manager = match CheckpointManager::new(config, PathBuf::from("."), uuid::Uuid::new_v4()) {
        Ok(manager) => manager,
        Err(e) => {
            return cx.throw_error(format!("Failed to create checkpoint manager: {}", e));
        }
    };

    // Create a simple checkpoint
    let checkpoint_id = match manager.create_simple_checkpoint(&description) {
        Ok(id) => id.to_string(),
        Err(e) => {
            log::warn!("Failed to create checkpoint: {}", e);
            // For compatibility, still return a UUID even if creation fails
            uuid::Uuid::new_v4().to_string()
        }
    };

    Ok(cx.string(checkpoint_id))
}

/// Get checkpoint configuration
fn get_config(mut cx: FunctionContext) -> JsResult<JsObject> {
    let config = get_config_or_default();
    let js_obj = cx.empty_object();

    let max_checkpoints = cx.number(config.max_checkpoints as f64);
    js_obj.set(&mut cx, "maxCheckpoints", max_checkpoints)?;

    let retention_days = cx.number(config.retention_days as f64);
    js_obj.set(&mut cx, "retentionDays", retention_days)?;

    let max_storage_bytes = cx.number(config.max_storage_bytes as f64);
    js_obj.set(&mut cx, "maxStorageBytes", max_storage_bytes)?;

    let max_files_per_checkpoint = cx.number(config.max_files_per_checkpoint as f64);
    js_obj.set(&mut cx, "maxFilesPerCheckpoint", max_files_per_checkpoint)?;

    let enable_compression = cx.boolean(config.enable_compression);
    js_obj.set(&mut cx, "enableCompression", enable_compression)?;

    // Add tracked extensions array
    let tracked_extensions_array = JsArray::new(&mut cx, config.tracked_extensions.len());
    for (i, extension) in config.tracked_extensions.iter().enumerate() {
        let extension_str = cx.string(extension);
        tracked_extensions_array.set(&mut cx, i as u32, extension_str)?;
    }
    js_obj.set(&mut cx, "trackedExtensions", tracked_extensions_array)?;

    // Add auto cleanup setting
    let auto_cleanup = cx.boolean(config.auto_cleanup);
    js_obj.set(&mut cx, "autoCleanup", auto_cleanup)?;

    // Add cleanup interval hours
    let cleanup_interval_hours = cx.number(config.cleanup_interval_hours as f64);
    js_obj.set(&mut cx, "cleanupIntervalHours", cleanup_interval_hours)?;

    Ok(js_obj)
}

/// Create a checkpoint manager instance
fn create_manager(mut cx: FunctionContext) -> JsResult<JsString> {
    let workspace_path_str = cx.argument::<JsString>(0)?.value(&mut cx);
    let workspace_path = std::path::PathBuf::from(workspace_path_str);

    let config = get_config_or_default();
    let mut manager = match CheckpointManager::new(config, PathBuf::from("."), uuid::Uuid::new_v4())
    {
        Ok(manager) => manager,
        Err(e) => {
            return cx.throw_error(format!("Failed to create checkpoint manager: {}", e));
        }
    };

    // Initialize session
    let session_id = match manager.init_session(workspace_path) {
        Ok(id) => id,
        Err(e) => {
            return cx.throw_error(format!("Failed to initialize session: {}", e));
        }
    };

    Ok(cx.string(session_id.to_string()))
}

/// Get checkpoint statistics
fn get_stats(mut cx: FunctionContext) -> JsResult<JsObject> {
    let config = get_config_or_default();
    let manager = match CheckpointManager::new(config, PathBuf::from("."), uuid::Uuid::new_v4()) {
        Ok(manager) => manager,
        Err(e) => {
            return cx.throw_error(format!("Failed to create checkpoint manager: {}", e));
        }
    };

    let stats = match manager.get_statistics() {
        Ok(stats) => stats,
        Err(e) => {
            return cx.throw_error(format!("Failed to get statistics: {}", e));
        }
    };

    let js_obj = cx.empty_object();

    let total_checkpoints = cx.number(stats.total_checkpoints as f64);
    js_obj.set(&mut cx, "totalCheckpoints", total_checkpoints)?;

    let total_sessions = cx.number(stats.total_sessions as f64);
    js_obj.set(&mut cx, "totalSessions", total_sessions)?;

    let total_storage_bytes = cx.number(stats.total_storage_bytes as f64);
    js_obj.set(&mut cx, "totalStorageBytes", total_storage_bytes)?;

    let avg_checkpoint_size = cx.number(stats.avg_checkpoint_size as f64);
    js_obj.set(&mut cx, "avgCheckpointSize", avg_checkpoint_size)?;

    let files_tracked = cx.number(stats.files_tracked as f64);
    js_obj.set(&mut cx, "filesTracked", files_tracked)?;

    Ok(js_obj)
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    // Original checkpoint functions
    cx.export_function("createSimpleCheckpoint", create_simple_checkpoint)?;
    cx.export_function("getConfig", get_config)?;
    cx.export_function("setConfig", set_config)?;
    cx.export_function("createManager", create_manager)?;
    cx.export_function("getStats", get_stats)?;

    // Advanced checkpoint functions from bindings_simple.rs
    cx.export_function(
        "createCheckpointManager",
        crate::bindings_simple::create_checkpoint_manager,
    )?;
    cx.export_function(
        "createCheckpoint",
        crate::bindings_simple::create_checkpoint,
    )?;
    cx.export_function(
        "restoreCheckpoint",
        crate::bindings_simple::restore_checkpoint,
    )?;
    cx.export_function("listCheckpoints", crate::bindings_simple::list_checkpoints)?;
    cx.export_function(
        "deleteCheckpoint",
        crate::bindings_simple::delete_checkpoint,
    )?;
    cx.export_function(
        "cleanupOldCheckpoints",
        crate::bindings_simple::cleanup_old_checkpoints,
    )?;
    cx.export_function(
        "getCheckpointStats",
        crate::bindings_simple::get_checkpoint_stats,
    )?;

    // Smart checkpoint system functions
    cx.export_function(
        "startAgentSession",
        crate::bindings_simple::start_agent_session,
    )?;
    cx.export_function(
        "stopAgentSession",
        crate::bindings_simple::stop_agent_session,
    )?;
    cx.export_function(
        "setOperationMode",
        crate::bindings_simple::set_operation_mode,
    )?;
    cx.export_function("trackAIFiles", crate::bindings_simple::track_ai_files)?;
    cx.export_function(
        "createAgentCheckpoint",
        crate::bindings_simple::create_agent_checkpoint,
    )?;
    cx.export_function("hasAIChanges", crate::bindings_simple::has_ai_changes)?;
    cx.export_function(
        "getChangesetStats",
        crate::bindings_simple::get_changeset_stats,
    )?;

    Ok(())
}
