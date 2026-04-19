use crate::config::CheckpointConfig;
use crate::manager::CheckpointManager;
use crate::restoration::RestoreResult;
use crate::types::{
    AISessionMetrics, CheckpointAnalysis, CheckpointOptions, ConflictResolution,
    RestoreOptions, SyncStatus,
};
use chrono::{Duration, Utc};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use uuid::Uuid;

fn write_workspace_file(workspace: &Path, relative_path: &str, contents: &str) {
    let full_path = workspace.join(relative_path);
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).expect("create parent directories");
    }
    fs::write(full_path, contents).expect("write workspace file");
}

fn read_workspace_file(workspace: &Path, relative_path: &str) -> String {
    fs::read_to_string(workspace.join(relative_path)).expect("read workspace file")
}

fn build_test_manager(
    storage_root: &Path,
    workspace_root: &Path,
    session_id: Uuid,
) -> CheckpointManager {
    let mut config = CheckpointConfig::default();
    config.storage_path = storage_root.to_path_buf();
    config.auto_cleanup = false;
    config.enable_file_watcher = false;
    config.enable_performance_metrics = true;
    config.max_checkpoints = 100;
    config.max_files_per_checkpoint = 500;
    config.max_storage_bytes = 50_000_000;

    let mut manager = CheckpointManager::new(config, workspace_root.to_path_buf(), session_id)
        .expect("create checkpoint manager");
    manager
        .init_session(workspace_root.to_path_buf())
        .expect("initialize session");
    manager
}

fn overwrite_restore_options() -> RestoreOptions {
    RestoreOptions {
        create_backup: false,
        require_backup: false,
        restore_permissions: false,
        restore_timestamps: false,
        conflict_resolution: ConflictResolution::Overwrite,
        ..Default::default()
    }
}

#[test]
fn checkpoint_lifecycle_records_restore_and_storage_metrics() {
    let workspace_dir = TempDir::new().expect("workspace tempdir");
    let storage_dir = TempDir::new().expect("storage tempdir");

    write_workspace_file(workspace_dir.path(), "src/main.ts", "export const value = 1;\n");
    write_workspace_file(workspace_dir.path(), "README.md", "# checkpoint suite\n");

    let manager = build_test_manager(storage_dir.path(), workspace_dir.path(), Uuid::new_v4());
    let checkpoint_id = manager
        .create_checkpoint(CheckpointOptions {
            description: Some("Initial snapshot".to_string()),
            ..Default::default()
        })
        .expect("create checkpoint");

    let checkpoints = manager
        .list_checkpoints(Some(10))
        .expect("list checkpoints after create");
    assert_eq!(checkpoints.len(), 1);
    assert_eq!(checkpoints[0].id, checkpoint_id);

    write_workspace_file(workspace_dir.path(), "src/main.ts", "export const value = 99;\n");
    let restore_result: RestoreResult = manager
        .restore_checkpoint(&checkpoint_id, overwrite_restore_options())
        .expect("restore checkpoint");

    assert!(restore_result.success);
    assert_eq!(
        read_workspace_file(workspace_dir.path(), "src/main.ts"),
        "export const value = 1;\n"
    );

    manager
        .record_storage_snapshot()
        .expect("record storage snapshot");
    let storage_usage = manager.get_storage_usage().expect("get storage usage");
    assert!(storage_usage.total_bytes > 0);
    assert!(storage_usage.checkpoint_count >= 1);

    let dashboard = manager
        .get_performance_dashboard(30)
        .expect("load performance dashboard");
    assert!(!dashboard.storage_history.is_empty());
    assert_eq!(dashboard.restoration_events.len(), 1);
    assert_eq!(dashboard.restoration_events[0].checkpoint_id, checkpoint_id);
    assert!(dashboard.summary.total_restorations >= 1);

    manager
        .delete_checkpoint(&checkpoint_id)
        .expect("delete checkpoint");
    assert!(
        manager
            .list_checkpoints(Some(10))
            .expect("list checkpoints after delete")
            .is_empty()
    );
}

#[test]
fn advanced_features_support_incremental_branching_analysis_and_collaboration() {
    let workspace_dir = TempDir::new().expect("workspace tempdir");
    let storage_dir = TempDir::new().expect("storage tempdir");
    let imported_storage_dir = TempDir::new().expect("imported storage tempdir");
    let session_id = Uuid::new_v4();

    write_workspace_file(workspace_dir.path(), "src/app.ts", "export function greet() {\n  return 'hello';\n}\n");

    let manager = build_test_manager(storage_dir.path(), workspace_dir.path(), session_id);
    let imported_manager = build_test_manager(
        imported_storage_dir.path(),
        workspace_dir.path(),
        Uuid::new_v4(),
    );
    let base_checkpoint = manager
        .create_checkpoint(CheckpointOptions {
            description: Some("Base checkpoint".to_string()),
            ..Default::default()
        })
        .expect("create base checkpoint");

    let branch = manager
        .create_branch("feature/collab", &base_checkpoint, "feature branch")
        .expect("create branch");
    manager
        .switch_branch(&branch.id)
        .expect("switch branch");
    assert_eq!(manager.current_branch_id().as_deref(), Some(branch.id.as_str()));

    write_workspace_file(
        workspace_dir.path(),
        "src/app.ts",
        "export function greet() {\n  return 'hello from branch';\n}\n",
    );

    let incremental_checkpoint = manager
        .create_incremental_checkpoint(CheckpointOptions {
            description: Some("Branch delta".to_string()),
            ..Default::default()
        })
        .expect("create incremental checkpoint");

    let reconstructed = manager
        .reconstruct_checkpoint(&incremental_checkpoint)
        .expect("reconstruct checkpoint chain");
    assert!(reconstructed.chain_length <= 1);
    assert_eq!(
        reconstructed
            .file_states
            .get(&PathBuf::from("src/app.ts"))
            .and_then(|state| state.content.clone())
            .as_deref(),
        Some("export function greet() {\n  return 'hello from branch';\n}\n")
    );

    let branch_checkpoints = manager
        .list_branch_checkpoints(&branch.id, Some(10))
        .expect("list branch checkpoints");
    assert!(branch_checkpoints.iter().any(|checkpoint| checkpoint.id == incremental_checkpoint));

    let analysis: CheckpointAnalysis = manager
        .analyze_checkpoint(&incremental_checkpoint)
        .expect("analyze checkpoint");
    assert!(!analysis.generated_description.is_empty());
    assert!(analysis.risk_assessment.score >= 0.0);

    let shared_bundle = manager
        .share_checkpoints(&[base_checkpoint, incremental_checkpoint], "Phase 8 share")
        .expect("share checkpoints");
    assert_eq!(shared_bundle.checkpoint_ids.len(), 2);
    assert_eq!(
        manager
            .list_shared_bundles()
            .expect("list shared bundles")
            .len(),
        1
    );
    assert!(manager
        .get_audit_trail(20, Some("share_checkpoints"))
        .expect("get audit trail")
        .iter()
        .any(|record| record.resource_id == shared_bundle.id));

    let imported_ids = imported_manager
        .import_shared_bundle(&shared_bundle)
        .expect("import shared bundle");
    assert_eq!(imported_ids.len(), 2);
    assert!(imported_manager
        .get_checkpoint(&base_checkpoint)
        .expect("get imported checkpoint")
        .is_some());

    let sync_bundle = manager
        .export_for_sync(Utc::now() - Duration::hours(1))
        .expect("export sync bundle");
    let sync_status = imported_manager
        .import_sync_bundle(&sync_bundle)
        .expect("import sync bundle");
    assert!(matches!(sync_status, SyncStatus::Succeeded { .. }));
    assert!(matches!(
        imported_manager.get_sync_status().expect("get sync status"),
        SyncStatus::Succeeded { .. }
    ));

    manager
        .record_ai_session_metrics(&AISessionMetrics {
            session_id: Uuid::new_v4(),
            started_at: Utc::now() - Duration::minutes(5),
            ended_at: Some(Utc::now()),
            files_changed: 1,
            lines_added: 3,
            lines_deleted: 1,
            checkpoints_created: 2,
            rollbacks: 1,
            duration_seconds: 300.0,
        })
        .expect("record ai session metrics");

    let dashboard = manager
        .get_performance_dashboard(30)
        .expect("load advanced dashboard");
    assert!(dashboard.summary.total_checkpoints_created >= 2);
    assert!(dashboard.summary.total_ai_sessions >= 1);
    assert!(dashboard.summary.total_rollbacks >= 1);
}

#[test]
fn implementation_plan_contains_no_unchecked_items() {
    let plan_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("imple-checkpoint.md");
    let plan = fs::read_to_string(plan_path).expect("read implementation plan");

    let unchecked: Vec<_> = plan
        .lines()
        .filter(|line| line.trim_start().starts_with("- [ ]"))
        .collect();

    assert!(
        unchecked.is_empty(),
        "implementation plan still has unchecked items: {:?}",
        unchecked
    );
}