//! Unified AI Context System
//!
//! This module provides a high-performance, unified AI context system that merges
//! checkpoint management with semantic analysis and context building capabilities.

pub mod architectural_analyzer;
pub mod context_builder;
pub mod error_handling;
pub mod intent_analyzer;
pub mod performance_monitor;
pub mod production_config;
pub mod query_analyzer;
pub mod relevance_engine;
// pub mod production_ffi; // Disabled due to threading constraints
pub mod sync_ffi;
// pub mod async_ffi; // Disabled pending API fixes
// pub mod unified_engine; // Temporarily disabled due to compilation issues

// Re-export key types and structs
pub use architectural_analyzer::{ArchitecturalAnalyzer, ArchitecturalPatterns};
pub use context_builder::{CompleteAIContext, ContextBuildingPipeline, ContextOptions};
pub use error_handling::{ContextBuildStage, ErrorContext, ErrorHandler, RecoveryStrategy};
pub use intent_analyzer::{ChangeIntent, IntentAnalyzer, IntentPattern};
pub use performance_monitor::{MetricsCollector, PerformanceMetrics, PerformanceMonitor};
pub use production_config::ProductionAIContextConfig;
pub use query_analyzer::{QueryAnalyzer, QueryIntent, QueryScope, QueryType};
pub use relevance_engine::{RelevanceScore, RelevanceScorer, ScoredCheckpoint};
// pub use unified_engine::UnifiedAIContextEngine; // Temporarily disabled

// Import core types from parent modules
use crate::types::*;
// use crate::semantic::{SemanticAnalyzer, SemanticContext, AIContextCheckpoint};

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// Configuration for the AI context system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIContextConfig {
    /// Maximum number of tokens in generated context
    pub max_tokens: usize,
    /// Enable incremental context updates
    pub enable_incremental_updates: bool,
    /// Enable parallel processing
    pub enable_parallel_processing: bool,
    /// Cache configuration
    pub cache_config: CacheConfig,
    /// Performance monitoring configuration
    pub performance_config: PerformanceConfig,
}

impl Default for AIContextConfig {
    fn default() -> Self {
        Self {
            max_tokens: 16000,
            enable_incremental_updates: true,
            enable_parallel_processing: true,
            cache_config: CacheConfig::default(),
            performance_config: PerformanceConfig::default(),
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum number of cached contexts
    pub max_cached_contexts: usize,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Enable semantic similarity caching
    pub enable_semantic_caching: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_cached_contexts: 1000,
            cache_ttl_seconds: 3600, // 1 hour
            enable_semantic_caching: true,
        }
    }
}

/// Performance monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable detailed performance metrics
    pub enable_detailed_metrics: bool,
    /// Metrics collection interval in seconds
    pub metrics_interval_seconds: u64,
    /// Enable memory usage tracking
    pub enable_memory_tracking: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_detailed_metrics: true,
            metrics_interval_seconds: 60,
            enable_memory_tracking: true,
        }
    }
}

// Use the comprehensive AIContextError from error_handling module
pub use error_handling::AIContextError;
pub type Result<T> = std::result::Result<T, AIContextError>;

/// Core AI context result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIContextResult {
    /// The complete AI context
    pub context: CompleteAIContext,
    /// Metadata about context generation
    pub metadata: ContextMetadata,
    /// Performance metrics
    pub performance: PerformanceMetrics,
    /// Cache information
    pub cache_info: CacheInfo,
}

/// Context generation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMetadata {
    /// Checkpoints used to build context
    pub checkpoints_used: Vec<CheckpointId>,
    /// Context type
    pub context_type: String,
    /// Overall confidence score
    pub confidence_score: f64,
    /// Estimated token count
    pub token_count: usize,
    /// Number of semantic entities included
    pub semantic_entities_count: usize,
    /// Architectural patterns identified
    pub architectural_patterns: Vec<String>,
    /// Build timestamp
    pub generated_at: chrono::DateTime<chrono::Utc>,
    /// Build duration
    pub build_duration: Duration,
}

/// Cache information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheInfo {
    /// Whether this result was served from cache
    pub cache_hit: bool,
    /// Cache key used
    pub cache_key: String,
    /// Cache timestamp
    pub cached_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Cache TTL remaining
    pub ttl_remaining: Option<Duration>,
}

/// Unified cache manager for AI context system
pub struct ContextCacheManager {
    /// Context cache
    context_cache: Arc<RwLock<HashMap<String, (AIContextResult, Instant)>>>,
    /// Query intent cache
    intent_cache: Arc<RwLock<HashMap<String, (QueryIntent, Instant)>>>,
    /// Semantic similarity cache
    similarity_cache: Arc<RwLock<HashMap<String, (Vec<ScoredCheckpoint>, Instant)>>>,
    /// Configuration
    config: CacheConfig,
}

impl ContextCacheManager {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            context_cache: Arc::new(RwLock::new(HashMap::new())),
            intent_cache: Arc::new(RwLock::new(HashMap::new())),
            similarity_cache: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Get cached context result
    pub async fn get_context(&self, key: &str) -> Option<AIContextResult> {
        let cache = self.context_cache.read().await;

        if let Some((result, timestamp)) = cache.get(key) {
            if timestamp.elapsed().as_secs() < self.config.cache_ttl_seconds {
                return Some(result.clone());
            }
        }

        None
    }

    /// Cache context result
    pub async fn cache_context(&self, key: String, result: AIContextResult) {
        let mut cache = self.context_cache.write().await;

        // Remove expired entries if cache is full
        if cache.len() >= self.config.max_cached_contexts {
            let _now = Instant::now();
            cache.retain(|_, (_, timestamp)| {
                timestamp.elapsed().as_secs() < self.config.cache_ttl_seconds
            });

            // If still full, remove oldest entries
            if cache.len() >= self.config.max_cached_contexts {
                let keys_to_remove: Vec<_> = {
                    let mut entries: Vec<_> = cache.iter().collect();
                    entries.sort_by_key(|(_, (_, timestamp))| timestamp);
                    let to_remove = entries
                        .len()
                        .saturating_sub(self.config.max_cached_contexts / 2);
                    entries
                        .into_iter()
                        .take(to_remove)
                        .map(|(key, _)| key.clone())
                        .collect()
                };

                for key in keys_to_remove {
                    cache.remove(&key);
                }
            }
        }

        cache.insert(key, (result, Instant::now()));
    }

    /// Get cached query intent
    pub async fn get_intent(&self, key: &str) -> Option<QueryIntent> {
        let cache = self.intent_cache.read().await;

        if let Some((intent, timestamp)) = cache.get(key) {
            if timestamp.elapsed().as_secs() < self.config.cache_ttl_seconds {
                return Some(intent.clone());
            }
        }

        None
    }

    /// Cache query intent
    pub async fn cache_intent(&self, key: String, intent: QueryIntent) {
        let mut cache = self.intent_cache.write().await;
        cache.insert(key, (intent, Instant::now()));
    }

    /// Clear all caches
    pub async fn clear_all(&self) {
        let mut context_cache = self.context_cache.write().await;
        let mut intent_cache = self.intent_cache.write().await;
        let mut similarity_cache = self.similarity_cache.write().await;

        context_cache.clear();
        intent_cache.clear();
        similarity_cache.clear();
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        let context_cache = self.context_cache.read().await;
        let intent_cache = self.intent_cache.read().await;
        let similarity_cache = self.similarity_cache.read().await;

        CacheStats {
            context_cache_size: context_cache.len(),
            intent_cache_size: intent_cache.len(),
            similarity_cache_size: similarity_cache.len(),
            max_cache_size: self.config.max_cached_contexts,
            cache_hit_ratio: 0.0, // Would be calculated from metrics
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub context_cache_size: usize,
    pub intent_cache_size: usize,
    pub similarity_cache_size: usize,
    pub max_cache_size: usize,
    pub cache_hit_ratio: f64,
}
