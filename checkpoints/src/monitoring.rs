//! Phase 8.5: Performance Monitoring Dashboard
//!
//! Tracks storage usage, checkpoint creation frequency, restoration
//! success/failure rates, and AI session productivity metrics.

use crate::db::CheckpointDatabase;
use crate::error::Result;
use crate::types::*;

use chrono::{Duration, Utc};
use std::path::Path;
use std::sync::Arc;

/// Performance monitoring dashboard manager
pub struct PerformanceMonitor {
    database: Arc<CheckpointDatabase>,
    /// Storage root path for calculating disk usage
    storage_path: std::path::PathBuf,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new(database: Arc<CheckpointDatabase>, storage_path: std::path::PathBuf) -> Self {
        Self {
            database,
            storage_path,
        }
    }

    // ========================================
    // Storage Usage
    // ========================================

    /// Get current storage usage snapshot
    pub fn get_current_storage_usage(&self) -> Result<StorageUsageSnapshot> {
        let stats = self.database.get_stats()?;

        // Calculate database file size
        let db_path = self.storage_path.join("checkpoints.db");
        let database_bytes = std::fs::metadata(&db_path)
            .map(|m| m.len())
            .unwrap_or(0);

        // Calculate content blob storage size
        let content_dir = self.storage_path.join("content");
        let (checkpoint_data_bytes, blob_count) = Self::dir_size_and_count(&content_dir);

        Ok(StorageUsageSnapshot {
            timestamp: Utc::now(),
            total_bytes: database_bytes + checkpoint_data_bytes,
            checkpoint_data_bytes,
            database_bytes,
            blob_count,
            checkpoint_count: stats.total_checkpoints as u64,
        })
    }

    /// Get storage usage history (from periodic snapshots stored in DB)
    pub fn get_storage_history(&self, days: u32) -> Result<Vec<StorageUsageSnapshot>> {
        let since = Utc::now() - Duration::days(days as i64);
        self.database.get_storage_snapshots(since)
    }

    /// Record a storage usage snapshot (called periodically)
    pub fn record_storage_snapshot(&self) -> Result<()> {
        let snapshot = self.get_current_storage_usage()?;
        self.database.insert_storage_snapshot(&snapshot)
    }

    /// Calculate directory size and file count recursively
    fn dir_size_and_count(path: &Path) -> (u64, u64) {
        let mut total_size: u64 = 0;
        let mut count: u64 = 0;

        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        total_size += metadata.len();
                        count += 1;
                    } else if metadata.is_dir() {
                        let (sub_size, sub_count) = Self::dir_size_and_count(&entry.path());
                        total_size += sub_size;
                        count += sub_count;
                    }
                }
            }
        }

        (total_size, count)
    }

    // ========================================
    // Checkpoint Creation Frequency
    // ========================================

    /// Get checkpoint creation frequency (bucketed by hour for recent, by day for older)
    pub fn get_creation_frequency(&self, days: u32) -> Result<Vec<CreationFrequencyPoint>> {
        self.database.get_creation_frequency(days)
    }

    // ========================================
    // Restoration Events
    // ========================================

    /// Record a restoration event
    pub fn record_restoration_event(&self, event: &RestorationEvent) -> Result<()> {
        self.database.insert_restoration_event(event)
    }

    /// Get restoration events history
    pub fn get_restoration_events(&self, limit: usize) -> Result<Vec<RestorationEvent>> {
        self.database.get_restoration_events(limit)
    }

    /// Get restoration success rate
    pub fn get_restoration_success_rate(&self) -> Result<f64> {
        let events = self.database.get_restoration_events(1000)?;
        if events.is_empty() {
            return Ok(1.0);
        }
        let successes = events.iter().filter(|e| e.success).count();
        Ok(successes as f64 / events.len() as f64)
    }

    // ========================================
    // AI Session Metrics
    // ========================================

    /// Record AI session metrics when a session ends
    pub fn record_ai_session_metrics(&self, metrics: &AISessionMetrics) -> Result<()> {
        self.database.insert_ai_session_metrics(metrics)
    }

    /// Get AI session metrics history
    pub fn get_ai_session_metrics(&self, limit: usize) -> Result<Vec<AISessionMetrics>> {
        self.database.get_ai_session_metrics(limit)
    }

    // ========================================
    // Full Dashboard
    // ========================================

    /// Build the complete performance dashboard
    pub fn get_dashboard(&self, history_days: u32) -> Result<PerformanceDashboard> {
        let current_storage = self.get_current_storage_usage()?;
        let storage_history = self.get_storage_history(history_days)?;
        let creation_frequency = self.get_creation_frequency(history_days)?;
        let restoration_events = self.get_restoration_events(100)?;
        let ai_session_metrics = self.get_ai_session_metrics(50)?;

        // Calculate summary
        let total_restorations = restoration_events.len() as u64;
        let successful_restorations = restoration_events.iter().filter(|e| e.success).count() as u64;
        let restoration_success_rate = if total_restorations > 0 {
            successful_restorations as f64 / total_restorations as f64
        } else {
            1.0
        };

        let avg_creation_time_ms = {
            let perf_metrics = self.database.get_performance_metrics("checkpoint_create", 100)?;
            if perf_metrics.is_empty() {
                0.0
            } else {
                perf_metrics.iter().map(|m| m.1).sum::<f64>() / perf_metrics.len() as f64
            }
        };

        let avg_restoration_time_ms = if restoration_events.is_empty() {
            0.0
        } else {
            restoration_events.iter().map(|e| e.duration_ms).sum::<f64>()
                / restoration_events.len() as f64
        };

        let total_ai_sessions = ai_session_metrics.len() as u64;
        let avg_changes_per_session = if ai_session_metrics.is_empty() {
            0.0
        } else {
            ai_session_metrics.iter().map(|m| m.files_changed as f64).sum::<f64>()
                / ai_session_metrics.len() as f64
        };

        let total_rollbacks: u64 = ai_session_metrics.iter().map(|m| m.rollbacks).sum();

        let total_checkpoints_created: u64 =
            creation_frequency.iter().map(|p| p.count).sum();

        let summary = DashboardSummary {
            total_checkpoints_created,
            total_restorations,
            restoration_success_rate,
            avg_creation_time_ms,
            avg_restoration_time_ms,
            total_ai_sessions,
            avg_changes_per_session,
            total_rollbacks,
        };

        Ok(PerformanceDashboard {
            storage_history,
            creation_frequency,
            restoration_events,
            ai_session_metrics,
            current_storage,
            summary,
        })
    }
}
