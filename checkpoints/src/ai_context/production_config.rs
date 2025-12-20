//! Production Configuration Management for AI Context System
//!
//! Provides comprehensive configuration management with environment-specific settings,
//! validation, and hot-reloading capabilities.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::Duration;

/// Production-ready AI context configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionAIContextConfig {
    /// Core system configuration
    pub system: SystemConfig,
    /// Performance and optimization settings
    pub performance: PerformanceConfig,
    /// Caching configuration
    pub cache: CacheConfig,
    /// Logging and monitoring configuration
    pub monitoring: MonitoringConfig,
    /// Security and validation settings
    pub security: SecurityConfig,
    /// Feature flags
    pub features: FeatureConfig,
    /// Environment-specific overrides
    pub environment: EnvironmentConfig,
}

/// Core system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    /// Maximum number of tokens in generated context
    pub max_tokens: usize,
    /// Maximum number of files to analyze in a single request
    pub max_files_per_request: usize,
    /// Timeout for context building operations
    pub context_build_timeout_ms: u64,
    /// Maximum memory usage threshold (MB)
    pub max_memory_usage_mb: f64,
    /// Enable incremental context updates
    pub enable_incremental_updates: bool,
    /// Enable parallel processing
    pub enable_parallel_processing: bool,
    /// Number of worker threads for parallel processing
    pub worker_thread_count: usize,
}

/// Performance and optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable detailed performance metrics
    pub enable_detailed_metrics: bool,
    /// Performance metrics collection interval (seconds)
    pub metrics_collection_interval_sec: u64,
    /// Enable performance profiling
    pub enable_profiling: bool,
    /// CPU usage threshold for throttling (percentage)
    pub cpu_throttle_threshold: f64,
    /// Memory usage threshold for cleanup (percentage)
    pub memory_cleanup_threshold: f64,
    /// Enable automatic optimization
    pub enable_auto_optimization: bool,
    /// Context build time warning threshold (ms)
    pub build_time_warning_threshold_ms: u64,
}

/// Caching configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum number of cached contexts
    pub max_cached_contexts: usize,
    /// Maximum number of cached intents
    pub max_cached_intents: usize,
    /// Cache TTL for contexts (seconds)
    pub context_cache_ttl_sec: u64,
    /// Cache TTL for intents (seconds)
    pub intent_cache_ttl_sec: u64,
    /// Enable cache compression
    pub enable_cache_compression: bool,
    /// Cache cleanup interval (seconds)
    pub cache_cleanup_interval_sec: u64,
    /// Cache hit ratio threshold for warnings
    pub cache_hit_ratio_warning_threshold: f64,
    /// Enable persistent cache storage
    pub enable_persistent_cache: bool,
    /// Persistent cache file path
    pub persistent_cache_path: Option<String>,
}

/// Monitoring and logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable detailed logging
    pub enable_detailed_logging: bool,
    /// Log level (error, warn, info, debug, trace)
    pub log_level: String,
    /// Enable performance monitoring
    pub enable_performance_monitoring: bool,
    /// Enable error tracking
    pub enable_error_tracking: bool,
    /// Enable usage analytics
    pub enable_usage_analytics: bool,
    /// Metrics export interval (seconds)
    pub metrics_export_interval_sec: u64,
    /// Log rotation configuration
    pub log_rotation: LogRotationConfig,
    /// Health check configuration
    pub health_check: HealthCheckConfig,
}

/// Log rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotationConfig {
    /// Maximum log file size (MB)
    pub max_file_size_mb: f64,
    /// Maximum number of log files to keep
    pub max_files: usize,
    /// Enable log compression
    pub compress_rotated_logs: bool,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Enable health checks
    pub enabled: bool,
    /// Health check interval (seconds)
    pub interval_sec: u64,
    /// Health check timeout (ms)
    pub timeout_ms: u64,
    /// Endpoints to check
    pub endpoints: Vec<String>,
}

/// Security and validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable input validation
    pub enable_input_validation: bool,
    /// Maximum query length
    pub max_query_length: usize,
    /// Enable rate limiting
    pub enable_rate_limiting: bool,
    /// Rate limit (requests per minute)
    pub rate_limit_requests_per_minute: usize,
    /// Enable API key authentication
    pub enable_api_key_auth: bool,
    /// Allowed file extensions for analysis
    pub allowed_file_extensions: Vec<String>,
    /// Blocked file patterns (regex)
    pub blocked_file_patterns: Vec<String>,
    /// Enable content sanitization
    pub enable_content_sanitization: bool,
}

/// Feature flags configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    /// Enable experimental features
    pub enable_experimental_features: bool,
    /// Enable advanced semantic analysis
    pub enable_advanced_semantic_analysis: bool,
    /// Enable machine learning enhancements
    pub enable_ml_enhancements: bool,
    /// Enable real-time collaboration features
    pub enable_realtime_collaboration: bool,
    /// Enable context streaming
    pub enable_context_streaming: bool,
    /// Enable predictive context loading
    pub enable_predictive_loading: bool,
    /// Enable architectural pattern detection
    pub enable_pattern_detection: bool,
}

/// Environment-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Environment name (development, staging, production)
    pub environment: String,
    /// Debug mode enabled
    pub debug_mode: bool,
    /// Development-only features enabled
    pub dev_features_enabled: bool,
    /// Testing mode enabled
    pub test_mode: bool,
    /// Production optimizations enabled
    pub production_optimizations: bool,
    /// Environment-specific overrides
    pub overrides: std::collections::HashMap<String, serde_json::Value>,
}

impl Default for ProductionAIContextConfig {
    fn default() -> Self {
        Self {
            system: SystemConfig {
                max_tokens: 16000,
                max_files_per_request: 100,
                context_build_timeout_ms: 30000,
                max_memory_usage_mb: 512.0,
                enable_incremental_updates: true,
                enable_parallel_processing: true,
                worker_thread_count: num_cpus::get(),
            },
            performance: PerformanceConfig {
                enable_detailed_metrics: true,
                metrics_collection_interval_sec: 60,
                enable_profiling: false,
                cpu_throttle_threshold: 80.0,
                memory_cleanup_threshold: 85.0,
                enable_auto_optimization: true,
                build_time_warning_threshold_ms: 1000,
            },
            cache: CacheConfig {
                max_cached_contexts: 1000,
                max_cached_intents: 5000,
                context_cache_ttl_sec: 3600,
                intent_cache_ttl_sec: 1800,
                enable_cache_compression: true,
                cache_cleanup_interval_sec: 300,
                cache_hit_ratio_warning_threshold: 0.7,
                enable_persistent_cache: true,
                persistent_cache_path: Some("./cache/ai_context_cache.db".to_string()),
            },
            monitoring: MonitoringConfig {
                enable_detailed_logging: true,
                log_level: "info".to_string(),
                enable_performance_monitoring: true,
                enable_error_tracking: true,
                enable_usage_analytics: true,
                metrics_export_interval_sec: 60,
                log_rotation: LogRotationConfig {
                    max_file_size_mb: 100.0,
                    max_files: 10,
                    compress_rotated_logs: true,
                },
                health_check: HealthCheckConfig {
                    enabled: true,
                    interval_sec: 30,
                    timeout_ms: 5000,
                    endpoints: vec![],
                },
            },
            security: SecurityConfig {
                enable_input_validation: true,
                max_query_length: 10000,
                enable_rate_limiting: true,
                rate_limit_requests_per_minute: 100,
                enable_api_key_auth: false,
                allowed_file_extensions: vec![
                    "ts".to_string(),
                    "tsx".to_string(),
                    "js".to_string(),
                    "jsx".to_string(),
                    "py".to_string(),
                    "rs".to_string(),
                    "java".to_string(),
                    "cpp".to_string(),
                    "c".to_string(),
                    "h".to_string(),
                    "hpp".to_string(),
                    "cs".to_string(),
                    "go".to_string(),
                    "php".to_string(),
                    "rb".to_string(),
                    "swift".to_string(),
                    "kt".to_string(),
                    "scala".to_string(),
                    "md".to_string(),
                    "json".to_string(),
                    "yaml".to_string(),
                    "yml".to_string(),
                    "toml".to_string(),
                    "xml".to_string(),
                ],
                blocked_file_patterns: vec![
                    r".*\.exe$".to_string(),
                    r".*\.dll$".to_string(),
                    r".*\.so$".to_string(),
                    r".*\.dylib$".to_string(),
                    r".*node_modules.*".to_string(),
                    r".*\.git.*".to_string(),
                    r".*target/.*".to_string(),
                    r".*build/.*".to_string(),
                    r".*dist/.*".to_string(),
                ],
                enable_content_sanitization: true,
            },
            features: FeatureConfig {
                enable_experimental_features: false,
                enable_advanced_semantic_analysis: true,
                enable_ml_enhancements: false,
                enable_realtime_collaboration: false,
                enable_context_streaming: true,
                enable_predictive_loading: true,
                enable_pattern_detection: true,
            },
            environment: EnvironmentConfig {
                environment: "production".to_string(),
                debug_mode: false,
                dev_features_enabled: false,
                test_mode: false,
                production_optimizations: true,
                overrides: std::collections::HashMap::new(),
            },
        }
    }
}

impl ProductionAIContextConfig {
    /// Load configuration from file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Load configuration from environment variables
    pub fn load_from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let mut config = Self::default();

        // System configuration from environment
        if let Ok(max_tokens) = std::env::var("AI_CONTEXT_MAX_TOKENS") {
            config.system.max_tokens = max_tokens.parse()?;
        }

        if let Ok(timeout) = std::env::var("AI_CONTEXT_TIMEOUT_MS") {
            config.system.context_build_timeout_ms = timeout.parse()?;
        }

        if let Ok(parallel) = std::env::var("AI_CONTEXT_ENABLE_PARALLEL") {
            config.system.enable_parallel_processing = parallel.parse()?;
        }

        // Environment configuration
        if let Ok(env) = std::env::var("AI_CONTEXT_ENVIRONMENT") {
            config.environment.environment = env;
        }

        if let Ok(debug) = std::env::var("AI_CONTEXT_DEBUG") {
            config.environment.debug_mode = debug.parse()?;
        }

        // Cache configuration
        if let Ok(cache_size) = std::env::var("AI_CONTEXT_CACHE_SIZE") {
            config.cache.max_cached_contexts = cache_size.parse()?;
        }

        if let Ok(cache_ttl) = std::env::var("AI_CONTEXT_CACHE_TTL") {
            config.cache.context_cache_ttl_sec = cache_ttl.parse()?;
        }

        // Performance configuration
        if let Ok(metrics) = std::env::var("AI_CONTEXT_ENABLE_METRICS") {
            config.performance.enable_detailed_metrics = metrics.parse()?;
        }

        // Logging configuration
        if let Ok(log_level) = std::env::var("AI_CONTEXT_LOG_LEVEL") {
            config.monitoring.log_level = log_level;
        }

        config.validate()?;
        Ok(config)
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Validate system configuration
        if self.system.max_tokens == 0 {
            return Err("max_tokens must be greater than 0".into());
        }

        if self.system.max_files_per_request == 0 {
            return Err("max_files_per_request must be greater than 0".into());
        }

        if self.system.context_build_timeout_ms == 0 {
            return Err("context_build_timeout_ms must be greater than 0".into());
        }

        if self.system.worker_thread_count == 0 {
            return Err("worker_thread_count must be greater than 0".into());
        }

        // Validate performance configuration
        if self.performance.cpu_throttle_threshold <= 0.0
            || self.performance.cpu_throttle_threshold > 100.0
        {
            return Err("cpu_throttle_threshold must be between 0 and 100".into());
        }

        if self.performance.memory_cleanup_threshold <= 0.0
            || self.performance.memory_cleanup_threshold > 100.0
        {
            return Err("memory_cleanup_threshold must be between 0 and 100".into());
        }

        // Validate cache configuration
        if self.cache.max_cached_contexts == 0 {
            return Err("max_cached_contexts must be greater than 0".into());
        }

        if self.cache.context_cache_ttl_sec == 0 {
            return Err("context_cache_ttl_sec must be greater than 0".into());
        }

        if self.cache.cache_hit_ratio_warning_threshold < 0.0
            || self.cache.cache_hit_ratio_warning_threshold > 1.0
        {
            return Err("cache_hit_ratio_warning_threshold must be between 0 and 1".into());
        }

        // Validate security configuration
        if self.security.max_query_length == 0 {
            return Err("max_query_length must be greater than 0".into());
        }

        if self.security.rate_limit_requests_per_minute == 0 {
            return Err("rate_limit_requests_per_minute must be greater than 0".into());
        }

        // Validate log level
        match self.monitoring.log_level.to_lowercase().as_str() {
            "error" | "warn" | "info" | "debug" | "trace" => {}
            _ => return Err("log_level must be one of: error, warn, info, debug, trace".into()),
        }

        // Validate environment
        match self.environment.environment.to_lowercase().as_str() {
            "development" | "staging" | "production" | "test" => {}
            _ => {
                return Err(
                    "environment must be one of: development, staging, production, test".into(),
                )
            }
        }

        Ok(())
    }

    /// Get configuration for specific environment
    pub fn for_environment(env: &str) -> Self {
        let mut config = Self::default();

        match env.to_lowercase().as_str() {
            "development" => {
                config.environment.environment = "development".to_string();
                config.environment.debug_mode = true;
                config.environment.dev_features_enabled = true;
                config.environment.production_optimizations = false;
                config.monitoring.log_level = "debug".to_string();
                config.performance.enable_profiling = true;
                config.features.enable_experimental_features = true;
                config.cache.max_cached_contexts = 100;
                config.system.max_tokens = 8000;
            }
            "staging" => {
                config.environment.environment = "staging".to_string();
                config.environment.debug_mode = false;
                config.environment.dev_features_enabled = false;
                config.environment.production_optimizations = true;
                config.monitoring.log_level = "info".to_string();
                config.performance.enable_profiling = false;
                config.features.enable_experimental_features = false;
                config.cache.max_cached_contexts = 500;
                config.system.max_tokens = 12000;
            }
            "production" => {
                // Production defaults are already set in Default impl
            }
            "test" => {
                config.environment.environment = "test".to_string();
                config.environment.debug_mode = false;
                config.environment.test_mode = true;
                config.environment.production_optimizations = false;
                config.monitoring.log_level = "warn".to_string();
                config.performance.enable_detailed_metrics = false;
                config.cache.max_cached_contexts = 10;
                config.system.max_tokens = 4000;
                config.system.context_build_timeout_ms = 5000;
            }
            _ => {
                // Use default production configuration
            }
        }

        config
    }

    /// Get timeout duration
    pub fn get_context_build_timeout(&self) -> Duration {
        Duration::from_millis(self.system.context_build_timeout_ms)
    }

    /// Get cache TTL duration for contexts
    pub fn get_context_cache_ttl(&self) -> Duration {
        Duration::from_secs(self.cache.context_cache_ttl_sec)
    }

    /// Get cache TTL duration for intents
    pub fn get_intent_cache_ttl(&self) -> Duration {
        Duration::from_secs(self.cache.intent_cache_ttl_sec)
    }

    /// Check if feature is enabled
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        match feature {
            "experimental" => self.features.enable_experimental_features,
            "advanced_semantic" => self.features.enable_advanced_semantic_analysis,
            "ml_enhancements" => self.features.enable_ml_enhancements,
            "realtime_collaboration" => self.features.enable_realtime_collaboration,
            "context_streaming" => self.features.enable_context_streaming,
            "predictive_loading" => self.features.enable_predictive_loading,
            "pattern_detection" => self.features.enable_pattern_detection,
            _ => false,
        }
    }

    /// Check if file extension is allowed
    pub fn is_file_extension_allowed(&self, extension: &str) -> bool {
        self.security
            .allowed_file_extensions
            .contains(&extension.to_lowercase())
    }

    /// Check if file path matches blocked patterns
    pub fn is_file_path_blocked(&self, path: &str) -> bool {
        for pattern in &self.security.blocked_file_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                if regex.is_match(path) {
                    return true;
                }
            }
        }
        false
    }
}
