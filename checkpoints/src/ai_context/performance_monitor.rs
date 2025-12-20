//! Performance Monitoring Module
//!
//! Monitors and tracks performance metrics for the AI context system

use super::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Performance monitoring system
pub struct PerformanceMonitor {
    /// Configuration
    config: PerformanceConfig,
    /// Metrics collector
    metrics_collector: Arc<MetricsCollector>,
    /// Performance thresholds
    thresholds: PerformanceThresholds,
    /// Monitoring state
    state: Arc<RwLock<MonitoringState>>,
}

/// Metrics collection system
pub struct MetricsCollector {
    /// Build time metrics
    build_times: Arc<RwLock<Vec<Duration>>>,
    /// Memory usage samples
    memory_samples: Arc<RwLock<Vec<MemorySample>>>,
    /// Operation counters
    operation_counters: OperationCounters,
    /// Error tracking
    error_tracker: Arc<RwLock<ErrorTracker>>,
}

/// Performance thresholds for alerts
#[derive(Debug, Clone)]
struct PerformanceThresholds {
    /// Maximum acceptable build time in milliseconds
    max_build_time_ms: u64,
    /// Maximum memory usage in MB
    max_memory_usage_mb: f64,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            max_build_time_ms: 5000,     // 5 seconds
            max_memory_usage_mb: 1000.0, // 1GB
        }
    }
}

/// Monitoring state
#[derive(Debug, Clone)]
pub struct MonitoringState {
    /// Whether monitoring is active
    active: bool,
    /// Number of alerts generated
    alerts_generated: u64,
}

/// Memory usage sample
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySample {
    /// Timestamp of sample
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Memory usage in MB
    pub memory_mb: f64,
    /// Memory type (heap, stack, etc.)
    pub memory_type: String,
}

/// Operation counters
#[derive(Debug)]
struct OperationCounters {
    /// Total context builds
    context_builds: AtomicU64,
    /// Cache hits
    cache_hits: AtomicU64,
    /// Cache misses
    cache_misses: AtomicU64,
    /// Query analyses
    query_analyses: AtomicU64,
    /// Semantic analyses
    semantic_analyses: AtomicU64,
    /// Errors
    errors: AtomicU64,
}

impl Default for OperationCounters {
    fn default() -> Self {
        Self {
            context_builds: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            query_analyses: AtomicU64::new(0),
            semantic_analyses: AtomicU64::new(0),
            errors: AtomicU64::new(0),
        }
    }
}

/// Error tracking
#[derive(Debug, Clone, Default)]
struct ErrorTracker {
    /// Recent errors
    recent_errors: Vec<ErrorEntry>,
    /// Error counts by type
    error_counts: std::collections::HashMap<String, u64>,
    /// Last error time
    last_error_time: Option<chrono::DateTime<chrono::Utc>>,
}

/// Error entry
#[derive(Debug, Clone)]
struct ErrorEntry {
    /// Timestamp
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// Performance metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Build time in milliseconds
    pub build_time_ms: u64,
    /// Whether this was a cache hit
    pub cache_hit: bool,
    /// Memory usage in MB
    pub memory_usage_mb: f64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Number of checkpoints analyzed
    pub checkpoints_analyzed: usize,
    /// Number of files processed
    pub files_processed: usize,
    /// Number of entities extracted
    pub entities_extracted: usize,
}

/// Comprehensive performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    /// Report generation time
    pub generated_at: chrono::DateTime<chrono::Utc>,
    /// Summary statistics
    pub summary: PerformanceSummary,
    /// Build time statistics
    pub build_times: BuildTimeStats,
    /// Memory usage statistics
    pub memory_stats: MemoryStats,
    /// Cache performance
    pub cache_performance: CachePerformanceStats,
    /// Error statistics
    pub error_stats: ErrorStats,
    /// Performance alerts
    pub alerts: Vec<PerformanceAlert>,
}

/// Performance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    /// Total operations
    pub total_operations: u64,
    /// Average response time
    pub avg_response_time_ms: f64,
    /// Success rate
    pub success_rate: f64,
    /// Overall health score (0-100)
    pub health_score: f64,
}

/// Build time statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildTimeStats {
    /// Average build time
    pub average_ms: f64,
    /// Median build time
    pub median_ms: f64,
    /// 95th percentile
    pub p95_ms: f64,
    /// 99th percentile
    pub p99_ms: f64,
    /// Fastest build time
    pub fastest_ms: f64,
    /// Slowest build time
    pub slowest_ms: f64,
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Current memory usage
    pub current_mb: f64,
    /// Peak memory usage
    pub peak_mb: f64,
    /// Average memory usage
    pub average_mb: f64,
    /// Memory growth rate (MB per hour)
    pub growth_rate_mb_per_hour: f64,
}

/// Cache performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachePerformanceStats {
    /// Hit ratio
    pub hit_ratio: f64,
    /// Total hits
    pub total_hits: u64,
    /// Total misses
    pub total_misses: u64,
    /// Cache effectiveness score
    pub effectiveness_score: f64,
}

/// Error statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStats {
    /// Total errors
    pub total_errors: u64,
    /// Error rate (errors per hour)
    pub error_rate_per_hour: f64,
    /// Most common error types
    pub common_error_types: Vec<(String, u64)>,
    /// Recent error trend
    pub recent_trend: String,
}

/// Performance alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    /// Alert level
    pub level: AlertLevel,
    /// Alert message
    pub message: String,
    /// Metric that triggered the alert
    pub metric: String,
    /// Current value
    pub current_value: f64,
    /// Threshold value
    pub threshold_value: f64,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Suggested actions
    pub suggested_actions: Vec<String>,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertLevel {
    Info,
    Warning,
    Error,
    Critical,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new(config: PerformanceConfig) -> Result<Self> {
        let metrics_collector = Arc::new(MetricsCollector::new());
        let thresholds = PerformanceThresholds::default();

        let state = Arc::new(RwLock::new(MonitoringState {
            active: true,
            alerts_generated: 0,
        }));

        Ok(Self {
            config,
            metrics_collector,
            thresholds,
            state,
        })
    }

    /// Start a build timer
    pub async fn start_build_timer(&self) -> BuildTimer {
        BuildTimer::new(self.metrics_collector.clone())
    }

    /// Record a build time
    pub async fn record_build_time(&self, duration: Duration) {
        self.metrics_collector.record_build_time(duration).await;

        // Check for performance alerts
        if duration.as_millis() as u64 > self.thresholds.max_build_time_ms {
            self.generate_alert(
                AlertLevel::Warning,
                "Build time exceeded threshold".to_string(),
                "build_time_ms".to_string(),
                duration.as_millis() as f64,
                self.thresholds.max_build_time_ms as f64,
            )
            .await;
        }
    }

    /// Record a cache hit
    pub async fn record_cache_hit(&self) {
        self.metrics_collector.increment_cache_hits();
    }

    /// Record a cache miss
    pub async fn record_cache_miss(&self) {
        self.metrics_collector.increment_cache_misses();
    }

    /// Record an error
    pub async fn record_error(&self, error: &AIContextError, context: Option<String>) {
        self.metrics_collector.record_error(error, context).await;
        self.metrics_collector.increment_errors();
    }

    /// Record memory usage
    pub async fn record_memory_usage(&self, memory_mb: f64, memory_type: String) {
        self.metrics_collector
            .record_memory_usage(memory_mb, memory_type)
            .await;

        // Check for memory alerts
        if memory_mb > self.thresholds.max_memory_usage_mb {
            self.generate_alert(
                AlertLevel::Error,
                "Memory usage exceeded threshold".to_string(),
                "memory_usage_mb".to_string(),
                memory_mb,
                self.thresholds.max_memory_usage_mb,
            )
            .await;
        }
    }

    /// Get current performance metrics
    pub async fn get_current_metrics(&self) -> PerformanceMetrics {
        let build_times = self.metrics_collector.build_times.read().await;
        let memory_samples = self.metrics_collector.memory_samples.read().await;

        let avg_build_time = if build_times.is_empty() {
            0
        } else {
            build_times.iter().map(|d| d.as_millis()).sum::<u128>() as u64
                / build_times.len() as u64
        };

        let current_memory = memory_samples
            .last()
            .map(|sample| sample.memory_mb)
            .unwrap_or(0.0);

        let cache_hits = self
            .metrics_collector
            .operation_counters
            .cache_hits
            .load(Ordering::Relaxed);
        let cache_misses = self
            .metrics_collector
            .operation_counters
            .cache_misses
            .load(Ordering::Relaxed);
        let total_cache_ops = cache_hits + cache_misses;
        let cache_hit_ratio = if total_cache_ops > 0 {
            cache_hits as f64 / total_cache_ops as f64
        } else {
            0.0
        };

        PerformanceMetrics {
            build_time_ms: avg_build_time,
            cache_hit: cache_hit_ratio > 0.5, // Simplified
            memory_usage_mb: current_memory,
            cpu_usage_percent: 0.0,  // Would be calculated from system metrics
            checkpoints_analyzed: 0, // Would be tracked separately
            files_processed: 0,      // Would be tracked separately
            entities_extracted: 0,   // Would be tracked separately
        }
    }

    /// Generate a comprehensive performance report
    pub async fn generate_report(&self) -> PerformanceReport {
        let build_times = self.metrics_collector.build_times.read().await;
        let memory_samples = self.metrics_collector.memory_samples.read().await;
        let error_tracker = self.metrics_collector.error_tracker.read().await;

        // Calculate build time statistics
        let build_time_stats = self.calculate_build_time_stats(&build_times);

        // Calculate memory statistics
        let memory_stats = self.calculate_memory_stats(&memory_samples);

        // Calculate cache performance
        let cache_performance = self.calculate_cache_performance();

        // Calculate error statistics
        let error_stats = self.calculate_error_stats(&error_tracker);

        // Calculate summary
        let summary = self.calculate_summary(&build_time_stats, &cache_performance, &error_stats);

        // Get current alerts
        let alerts = vec![]; // Would maintain an alerts list

        PerformanceReport {
            generated_at: chrono::Utc::now(),
            summary,
            build_times: build_time_stats,
            memory_stats,
            cache_performance,
            error_stats,
            alerts,
        }
    }

    /// Optimize performance based on current metrics
    pub async fn optimize_performance(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        let metrics = self.get_current_metrics().await;

        // Build time recommendations
        if metrics.build_time_ms > 2000 {
            recommendations
                .push("Consider enabling parallel processing for context building".to_string());
            recommendations.push("Optimize semantic analysis algorithms".to_string());
        }

        // Memory recommendations
        if metrics.memory_usage_mb > 500.0 {
            recommendations.push("Implement more aggressive cache eviction policies".to_string());
            recommendations
                .push("Consider streaming context building for large codebases".to_string());
        }

        // Cache recommendations
        if !metrics.cache_hit {
            recommendations.push("Review cache key generation strategy".to_string());
            recommendations.push("Increase cache size limits".to_string());
        }

        recommendations
    }

    /// Generate a performance alert
    async fn generate_alert(
        &self,
        level: AlertLevel,
        message: String,
        metric: String,
        current_value: f64,
        threshold_value: f64,
    ) {
        let alert = PerformanceAlert {
            level: level.clone(),
            message: message.clone(),
            metric,
            current_value,
            threshold_value,
            timestamp: chrono::Utc::now(),
            suggested_actions: self.get_suggested_actions(&level, &message),
        };

        // In a real implementation, this would be sent to an alerting system
        log::warn!("Performance Alert: {:?}", alert);

        // Update alert counter
        {
            let mut state = self.state.write().await;
            state.alerts_generated += 1;
        }
    }

    /// Get suggested actions for an alert
    fn get_suggested_actions(&self, level: &AlertLevel, message: &str) -> Vec<String> {
        let mut actions = Vec::new();

        match level {
            AlertLevel::Warning => {
                actions.push("Monitor the situation".to_string());
                actions.push("Consider optimizations if trend continues".to_string());
            }
            AlertLevel::Error => {
                actions.push("Investigate immediately".to_string());
                actions.push("Check system resources".to_string());
            }
            AlertLevel::Critical => {
                actions.push("Take immediate action".to_string());
                actions.push("Consider scaling resources".to_string());
            }
            _ => {}
        }

        if message.contains("memory") {
            actions.push("Clear caches".to_string());
            actions.push("Restart if necessary".to_string());
        }

        if message.contains("build time") {
            actions.push("Enable parallel processing".to_string());
            actions.push("Optimize query complexity".to_string());
        }

        actions
    }

    // Helper methods for statistics calculation

    fn calculate_build_time_stats(&self, build_times: &[Duration]) -> BuildTimeStats {
        if build_times.is_empty() {
            return BuildTimeStats {
                average_ms: 0.0,
                median_ms: 0.0,
                p95_ms: 0.0,
                p99_ms: 0.0,
                fastest_ms: 0.0,
                slowest_ms: 0.0,
            };
        }

        let mut times_ms: Vec<f64> = build_times.iter().map(|d| d.as_millis() as f64).collect();
        times_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let average_ms = times_ms.iter().sum::<f64>() / times_ms.len() as f64;
        let median_ms = times_ms[times_ms.len() / 2];
        let p95_ms = times_ms[(times_ms.len() as f64 * 0.95) as usize];
        let p99_ms = times_ms[(times_ms.len() as f64 * 0.99) as usize];
        let fastest_ms = times_ms[0];
        let slowest_ms = times_ms[times_ms.len() - 1];

        BuildTimeStats {
            average_ms,
            median_ms,
            p95_ms,
            p99_ms,
            fastest_ms,
            slowest_ms,
        }
    }

    fn calculate_memory_stats(&self, memory_samples: &[MemorySample]) -> MemoryStats {
        if memory_samples.is_empty() {
            return MemoryStats {
                current_mb: 0.0,
                peak_mb: 0.0,
                average_mb: 0.0,
                growth_rate_mb_per_hour: 0.0,
            };
        }

        let current_mb = memory_samples.last().unwrap().memory_mb;
        let peak_mb = memory_samples
            .iter()
            .map(|s| s.memory_mb)
            .fold(0.0, f64::max);
        let average_mb =
            memory_samples.iter().map(|s| s.memory_mb).sum::<f64>() / memory_samples.len() as f64;

        // Calculate growth rate (simplified)
        let growth_rate_mb_per_hour = if memory_samples.len() > 1 {
            let first = &memory_samples[0];
            let last = &memory_samples[memory_samples.len() - 1];
            let time_diff_hours = (last.timestamp - first.timestamp).num_minutes() as f64 / 60.0;
            if time_diff_hours > 0.0 {
                (last.memory_mb - first.memory_mb) / time_diff_hours
            } else {
                0.0
            }
        } else {
            0.0
        };

        MemoryStats {
            current_mb,
            peak_mb,
            average_mb,
            growth_rate_mb_per_hour,
        }
    }

    fn calculate_cache_performance(&self) -> CachePerformanceStats {
        let hits = self
            .metrics_collector
            .operation_counters
            .cache_hits
            .load(Ordering::Relaxed);
        let misses = self
            .metrics_collector
            .operation_counters
            .cache_misses
            .load(Ordering::Relaxed);
        let total = hits + misses;

        let hit_ratio = if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        };

        let effectiveness_score = if hit_ratio >= 0.8 {
            100.0
        } else if hit_ratio >= 0.6 {
            80.0
        } else if hit_ratio >= 0.4 {
            60.0
        } else {
            40.0
        };

        CachePerformanceStats {
            hit_ratio,
            total_hits: hits,
            total_misses: misses,
            effectiveness_score,
        }
    }

    fn calculate_error_stats(&self, error_tracker: &ErrorTracker) -> ErrorStats {
        let total_errors = error_tracker.recent_errors.len() as u64;

        // Calculate error rate (simplified)
        let error_rate_per_hour = if !error_tracker.recent_errors.is_empty() {
            let now = chrono::Utc::now();
            let one_hour_ago = now - chrono::Duration::hours(1);
            let recent_errors = error_tracker
                .recent_errors
                .iter()
                .filter(|e| e.timestamp > one_hour_ago)
                .count();
            recent_errors as f64
        } else {
            0.0
        };

        // Get common error types
        let mut common_error_types: Vec<(String, u64)> = error_tracker
            .error_counts
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        common_error_types.sort_by(|a, b| b.1.cmp(&a.1));
        common_error_types.truncate(5);

        // Determine trend (simplified)
        let recent_trend = if error_rate_per_hour > 5.0 {
            "Increasing".to_string()
        } else if error_rate_per_hour < 1.0 {
            "Stable".to_string()
        } else {
            "Moderate".to_string()
        };

        ErrorStats {
            total_errors,
            error_rate_per_hour,
            common_error_types,
            recent_trend,
        }
    }

    fn calculate_summary(
        &self,
        build_times: &BuildTimeStats,
        cache_perf: &CachePerformanceStats,
        error_stats: &ErrorStats,
    ) -> PerformanceSummary {
        let total_operations = self
            .metrics_collector
            .operation_counters
            .context_builds
            .load(Ordering::Relaxed);
        let avg_response_time_ms = build_times.average_ms;

        let success_rate = if total_operations > 0 {
            1.0 - (error_stats.total_errors as f64 / total_operations as f64)
        } else {
            1.0
        };

        // Calculate health score
        let mut health_score: f64 = 100.0;
        if avg_response_time_ms > 1000.0 {
            health_score -= 20.0;
        }
        if cache_perf.hit_ratio < 0.7 {
            health_score -= 15.0;
        }
        if success_rate < 0.95 {
            health_score -= 25.0;
        }
        if error_stats.error_rate_per_hour > 5.0 {
            health_score -= 20.0;
        }

        PerformanceSummary {
            total_operations,
            avg_response_time_ms,
            success_rate,
            health_score: health_score.max(0.0),
        }
    }

    /// Get current configuration
    pub fn get_config(&self) -> &PerformanceConfig {
        &self.config
    }

    /// Check if monitoring should be active based on configuration
    pub async fn should_monitor(&self) -> bool {
        if !self.config.enable_detailed_metrics {
            return false;
        }

        let state = self.state.read().await;
        state.active
    }

    /// Update configuration
    pub async fn update_config(&mut self, new_config: PerformanceConfig) {
        self.config = new_config;

        // Update monitoring state based on new config
        if !self.config.enable_detailed_metrics {
            let mut state = self.state.write().await;
            state.active = false;
        }
    }

    /// Check thresholds and generate alerts if needed
    pub async fn check_thresholds(&self, metrics: &PerformanceMetrics) -> Vec<PerformanceAlert> {
        let mut alerts = Vec::new();

        // Check build time threshold
        if metrics.build_time_ms > self.thresholds.max_build_time_ms {
            alerts.push(PerformanceAlert {
                level: AlertLevel::Warning,
                message: "Build time exceeded threshold".to_string(),
                metric: "build_time_ms".to_string(),
                current_value: metrics.build_time_ms as f64,
                threshold_value: self.thresholds.max_build_time_ms as f64,
                timestamp: chrono::Utc::now(),
                suggested_actions: vec![
                    "Consider optimizing query complexity".to_string(),
                    "Check for performance bottlenecks".to_string(),
                ],
            });
        }

        // Check memory usage threshold
        if metrics.memory_usage_mb > self.thresholds.max_memory_usage_mb {
            alerts.push(PerformanceAlert {
                level: AlertLevel::Warning,
                message: "Memory usage exceeded threshold".to_string(),
                metric: "memory_usage_mb".to_string(),
                current_value: metrics.memory_usage_mb,
                threshold_value: self.thresholds.max_memory_usage_mb,
                timestamp: chrono::Utc::now(),
                suggested_actions: vec![
                    "Clear caches".to_string(),
                    "Reduce context size".to_string(),
                ],
            });
        }

        // Note: Cache hit ratio checking disabled as PerformanceMetrics only has cache_hit boolean
        // TODO: Implement cache hit ratio tracking in PerformanceMetrics
        /*
        if metrics.cache_hit_ratio < self.thresholds.min_cache_hit_ratio {
            alerts.push(PerformanceAlert {
                level: AlertLevel::Warning,
                message: "Cache hit ratio below threshold".to_string(),
                metric: "cache_hit_ratio".to_string(),
                current_value: metrics.cache_hit_ratio,
                threshold_value: self.thresholds.min_cache_hit_ratio,
                timestamp: chrono::Utc::now(),
                suggested_actions: vec![
                    "Review cache configuration".to_string(),
                    "Optimize cache key generation".to_string(),
                ],
            });
        }
        */

        alerts
    }

    /// Update monitoring state
    pub async fn update_monitoring_state<F>(&self, updater: F)
    where
        F: FnOnce(&mut MonitoringState),
    {
        let mut state = self.state.write().await;
        updater(&mut state);
    }
}

impl MetricsCollector {
    fn new() -> Self {
        Self {
            build_times: Arc::new(RwLock::new(Vec::new())),
            memory_samples: Arc::new(RwLock::new(Vec::new())),
            operation_counters: OperationCounters::default(),
            error_tracker: Arc::new(RwLock::new(ErrorTracker::default())),
        }
    }

    async fn record_build_time(&self, duration: Duration) {
        let mut build_times = self.build_times.write().await;
        build_times.push(duration);

        // Keep only last 1000 build times
        if build_times.len() > 1000 {
            build_times.remove(0);
        }

        self.operation_counters
            .context_builds
            .fetch_add(1, Ordering::Relaxed);
    }

    async fn record_memory_usage(&self, memory_mb: f64, memory_type: String) {
        let mut memory_samples = self.memory_samples.write().await;
        memory_samples.push(MemorySample {
            timestamp: chrono::Utc::now(),
            memory_mb,
            memory_type,
        });

        // Keep only last 1000 samples
        if memory_samples.len() > 1000 {
            memory_samples.remove(0);
        }
    }

    async fn record_error(&self, error: &AIContextError, _context: Option<String>) {
        let mut error_tracker = self.error_tracker.write().await;

        let error_type = format!("{:?}", error)
            .split('(')
            .next()
            .unwrap_or("Unknown")
            .to_string();

        error_tracker.recent_errors.push(ErrorEntry {
            timestamp: chrono::Utc::now(),
        });

        // Keep only last 100 errors
        if error_tracker.recent_errors.len() > 100 {
            error_tracker.recent_errors.remove(0);
        }

        *error_tracker.error_counts.entry(error_type).or_insert(0) += 1;
        error_tracker.last_error_time = Some(chrono::Utc::now());
    }

    fn increment_cache_hits(&self) {
        self.operation_counters
            .cache_hits
            .fetch_add(1, Ordering::Relaxed);
    }

    fn increment_cache_misses(&self) {
        self.operation_counters
            .cache_misses
            .fetch_add(1, Ordering::Relaxed);
    }

    fn increment_errors(&self) {
        self.operation_counters
            .errors
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Record a query analysis operation
    pub fn record_query_analysis(&self) {
        self.operation_counters
            .query_analyses
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Record a semantic analysis operation
    pub fn record_semantic_analysis(&self) {
        self.operation_counters
            .semantic_analyses
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Get operation counts
    pub fn get_operation_counts(&self) -> (u64, u64, u64, u64, u64, u64, u64) {
        (
            self.operation_counters
                .context_builds
                .load(Ordering::Relaxed),
            self.operation_counters.cache_hits.load(Ordering::Relaxed),
            self.operation_counters.cache_misses.load(Ordering::Relaxed),
            self.operation_counters
                .query_analyses
                .load(Ordering::Relaxed),
            self.operation_counters
                .semantic_analyses
                .load(Ordering::Relaxed),
            self.operation_counters.errors.load(Ordering::Relaxed),
            0, // placeholder for additional counter
        )
    }
}

/// Build timer for measuring operation duration
pub struct BuildTimer {
    start_time: Instant,
    metrics_collector: Arc<MetricsCollector>,
}

impl BuildTimer {
    fn new(metrics_collector: Arc<MetricsCollector>) -> Self {
        Self {
            start_time: Instant::now(),
            metrics_collector,
        }
    }

    /// Finish timing and record the duration
    pub async fn finish(self) -> Duration {
        let duration = self.start_time.elapsed();
        self.metrics_collector.record_build_time(duration).await;
        duration
    }
}
