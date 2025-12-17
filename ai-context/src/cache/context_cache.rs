/**
 * Context Cache - Comprehensive caching system for AI context components
 * 
 * This module implements a multi-level caching system for various AI context
 * components including semantic contexts, query results, ASTs, and relationship graphs.
 */

use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};
use std::time::{Duration, Instant};
use std::hash::{Hash, Hasher};
use serde::{Deserialize, Serialize};
use lru::LruCache;

use crate::types::{SemanticContext, CompleteAIContext, QueryIntent, AST, RelationshipGraph};
use crate::error::CheckpointError;

/// Multi-level context cache with different strategies for different content types
pub struct ContextCache {
    // Semantic context cache - frequently accessed, medium-term storage
    semantic_cache: Arc<RwLock<LruCache<String, CachedSemanticContext>>>,
    
    // Query result cache - short-term but high-impact
    query_cache: Arc<RwLock<LruCache<String, CachedQueryResult>>>,
    
    // AST cache - expensive to compute, long-term storage
    ast_cache: Arc<RwLock<LruCache<String, CachedAST>>>,
    
    // Relationship graph cache - medium-term, relationship data
    relationship_cache: Arc<RwLock<LruCache<String, CachedRelationshipGraph>>>,
    
    // Relevance score cache - short-term, frequently updated
    relevance_cache: Arc<RwLock<LruCache<String, CachedRelevanceScore>>>,
    
    // Cache statistics and metrics
    stats: Arc<Mutex<CacheStatistics>>,
    
    // Cache configuration
    config: CacheConfig,
    
    // Cache warming service
    warming_service: Option<Arc<CacheWarmingService>>,
}

impl ContextCache {
    pub fn new(config: CacheConfig) -> Self {
        let warming_service = if config.enable_cache_warming {
            Some(Arc::new(CacheWarmingService::new(config.clone())))
        } else {
            None
        };

        Self {
            semantic_cache: Arc::new(RwLock::new(LruCache::new(config.semantic_cache_size))),
            query_cache: Arc::new(RwLock::new(LruCache::new(config.query_cache_size))),
            ast_cache: Arc::new(RwLock::new(LruCache::new(config.ast_cache_size))),
            relationship_cache: Arc::new(RwLock::new(LruCache::new(config.relationship_cache_size))),
            relevance_cache: Arc::new(RwLock::new(LruCache::new(config.relevance_cache_size))),
            stats: Arc::new(Mutex::new(CacheStatistics::default())),
            config,
            warming_service,
        }
    }

    /// Get or compute semantic context with caching
    pub fn get_or_compute_semantic_context<F>(
        &self,
        key: &str,
        compute: F
    ) -> Result<SemanticContext, CheckpointError>
    where
        F: FnOnce() -> Result<SemanticContext, CheckpointError>
    {
        // Try to get from cache first
        if let Some(cached) = self.get_semantic_context(key) {
            self.record_cache_hit("semantic");
            return Ok(cached.context);
        }

        // Cache miss - compute the context
        self.record_cache_miss("semantic");
        let computed = compute()?;
        
        // Store in cache
        self.put_semantic_context(key, &computed);
        
        Ok(computed)
    }

    /// Get or compute query result with caching
    pub fn get_or_compute_query_context<F>(
        &self,
        query: &str,
        workspace: &str,
        max_tokens: u32,
        compute: F
    ) -> Result<CompleteAIContext, CheckpointError>
    where
        F: FnOnce() -> Result<CompleteAIContext, CheckpointError>
    {
        let cache_key = self.generate_query_cache_key(query, workspace, max_tokens);
        
        // Try to get from cache first
        if let Some(cached) = self.get_query_result(&cache_key) {
            if !self.is_query_result_expired(&cached) {
                self.record_cache_hit("query");
                return Ok(cached.context);
            }
        }

        // Cache miss or expired - compute the result
        self.record_cache_miss("query");
        let computed = compute()?;
        
        // Store in cache
        self.put_query_result(&cache_key, &computed);
        
        Ok(computed)
    }

    /// Get or compute AST with caching
    pub fn get_or_compute_ast<F>(
        &self,
        file_path: &str,
        content_hash: &str,
        compute: F
    ) -> Result<AST, CheckpointError>
    where
        F: FnOnce() -> Result<AST, CheckpointError>
    {
        let cache_key = format!("{}:{}", file_path, content_hash);
        
        // Try to get from cache first
        if let Some(cached) = self.get_ast(&cache_key) {
            self.record_cache_hit("ast");
            return Ok(cached.ast);
        }

        // Cache miss - compute the AST
        self.record_cache_miss("ast");
        let computed = compute()?;
        
        // Store in cache
        self.put_ast(&cache_key, &computed);
        
        Ok(computed)
    }

    /// Get or compute relationship graph with caching
    pub fn get_or_compute_relationship_graph<F>(
        &self,
        workspace: &str,
        files_hash: &str,
        compute: F
    ) -> Result<RelationshipGraph, CheckpointError>
    where
        F: FnOnce() -> Result<RelationshipGraph, CheckpointError>
    {
        let cache_key = format!("{}:{}", workspace, files_hash);
        
        // Try to get from cache first
        if let Some(cached) = self.get_relationship_graph(&cache_key) {
            if !self.is_relationship_graph_expired(&cached) {
                self.record_cache_hit("relationship");
                return Ok(cached.graph);
            }
        }

        // Cache miss or expired - compute the graph
        self.record_cache_miss("relationship");
        let computed = compute()?;
        
        // Store in cache
        self.put_relationship_graph(&cache_key, &computed);
        
        Ok(computed)
    }

    /// Get cached relevance score
    pub fn get_relevance_score(&self, query_hash: &str, checkpoint_id: &str) -> Option<f64> {
        let cache_key = format!("{}:{}", query_hash, checkpoint_id);
        
        if let Ok(cache) = self.relevance_cache.read() {
            if let Some(cached) = cache.peek(&cache_key) {
                if !self.is_relevance_score_expired(cached) {
                    self.record_cache_hit("relevance");
                    return Some(cached.score);
                }
            }
        }
        
        self.record_cache_miss("relevance");
        None
    }

    /// Put relevance score in cache
    pub fn put_relevance_score(&self, query_hash: &str, checkpoint_id: &str, score: f64) {
        let cache_key = format!("{}:{}", query_hash, checkpoint_id);
        let cached_score = CachedRelevanceScore {
            score,
            cached_at: Instant::now(),
            access_count: 1,
        };

        if let Ok(mut cache) = self.relevance_cache.write() {
            cache.put(cache_key, cached_score);
        }
    }

    /// Invalidate cache entries for a specific file
    pub fn invalidate_file_caches(&self, file_path: &str) {
        // Invalidate AST cache entries for this file
        if let Ok(mut cache) = self.ast_cache.write() {
            let keys_to_remove: Vec<String> = cache.iter()
                .filter(|(key, _)| key.starts_with(file_path))
                .map(|(key, _)| key.clone())
                .collect();
            
            for key in keys_to_remove {
                cache.pop(&key);
            }
        }

        // Invalidate semantic context cache entries
        if let Ok(mut cache) = self.semantic_cache.write() {
            let keys_to_remove: Vec<String> = cache.iter()
                .filter(|(key, _)| key.contains(file_path))
                .map(|(key, _)| key.clone())
                .collect();
            
            for key in keys_to_remove {
                cache.pop(&key);
            }
        }

        // Invalidate relationship graphs (they might be affected by file changes)
        if let Ok(mut cache) = self.relationship_cache.write() {
            cache.clear(); // Conservative approach - clear all relationship graphs
        }
    }

    /// Warm up cache with frequently accessed data
    pub async fn warm_cache(&self, workspace: &str) -> Result<(), CheckpointError> {
        if let Some(warming_service) = &self.warming_service {
            warming_service.warm_cache(workspace, self).await?;
        }
        Ok(())
    }

    /// Get comprehensive cache statistics
    pub fn get_statistics(&self) -> CacheStatistics {
        if let Ok(stats) = self.stats.lock() {
            stats.clone()
        } else {
            CacheStatistics::default()
        }
    }

    /// Clear all caches
    pub fn clear_all(&self) {
        if let Ok(mut cache) = self.semantic_cache.write() {
            cache.clear();
        }
        if let Ok(mut cache) = self.query_cache.write() {
            cache.clear();
        }
        if let Ok(mut cache) = self.ast_cache.write() {
            cache.clear();
        }
        if let Ok(mut cache) = self.relationship_cache.write() {
            cache.clear();
        }
        if let Ok(mut cache) = self.relevance_cache.write() {
            cache.clear();
        }
        
        // Reset statistics
        if let Ok(mut stats) = self.stats.lock() {
            *stats = CacheStatistics::default();
        }
    }

    /// Get memory usage estimation
    pub fn get_memory_usage(&self) -> MemoryUsage {
        let mut usage = MemoryUsage::default();

        if let Ok(cache) = self.semantic_cache.read() {
            usage.semantic_cache_mb = (cache.len() * 1024) / (1024 * 1024); // Rough estimate
        }
        if let Ok(cache) = self.query_cache.read() {
            usage.query_cache_mb = (cache.len() * 2048) / (1024 * 1024); // Rough estimate
        }
        if let Ok(cache) = self.ast_cache.read() {
            usage.ast_cache_mb = (cache.len() * 512) / (1024 * 1024); // Rough estimate
        }
        if let Ok(cache) = self.relationship_cache.read() {
            usage.relationship_cache_mb = (cache.len() * 1024) / (1024 * 1024); // Rough estimate
        }
        if let Ok(cache) = self.relevance_cache.read() {
            usage.relevance_cache_mb = (cache.len() * 64) / (1024 * 1024); // Rough estimate
        }

        usage.total_mb = usage.semantic_cache_mb + usage.query_cache_mb + 
                        usage.ast_cache_mb + usage.relationship_cache_mb + usage.relevance_cache_mb;

        usage
    }

    // Private helper methods

    fn get_semantic_context(&self, key: &str) -> Option<CachedSemanticContext> {
        if let Ok(mut cache) = self.semantic_cache.write() {
            if let Some(cached) = cache.get(key) {
                cached.access_count += 1;
                cached.last_accessed = Instant::now();
                return Some(cached.clone());
            }
        }
        None
    }

    fn put_semantic_context(&self, key: &str, context: &SemanticContext) {
        let cached_context = CachedSemanticContext {
            context: context.clone(),
            cached_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 1,
        };

        if let Ok(mut cache) = self.semantic_cache.write() {
            cache.put(key.to_string(), cached_context);
        }
    }

    fn get_query_result(&self, key: &str) -> Option<CachedQueryResult> {
        if let Ok(mut cache) = self.query_cache.write() {
            if let Some(cached) = cache.get(key) {
                cached.access_count += 1;
                cached.last_accessed = Instant::now();
                return Some(cached.clone());
            }
        }
        None
    }

    fn put_query_result(&self, key: &str, context: &CompleteAIContext) {
        let cached_result = CachedQueryResult {
            context: context.clone(),
            cached_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 1,
        };

        if let Ok(mut cache) = self.query_cache.write() {
            cache.put(key.to_string(), cached_result);
        }
    }

    fn get_ast(&self, key: &str) -> Option<CachedAST> {
        if let Ok(mut cache) = self.ast_cache.write() {
            if let Some(cached) = cache.get(key) {
                cached.access_count += 1;
                cached.last_accessed = Instant::now();
                return Some(cached.clone());
            }
        }
        None
    }

    fn put_ast(&self, key: &str, ast: &AST) {
        let cached_ast = CachedAST {
            ast: ast.clone(),
            cached_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 1,
        };

        if let Ok(mut cache) = self.ast_cache.write() {
            cache.put(key.to_string(), cached_ast);
        }
    }

    fn get_relationship_graph(&self, key: &str) -> Option<CachedRelationshipGraph> {
        if let Ok(mut cache) = self.relationship_cache.write() {
            if let Some(cached) = cache.get(key) {
                cached.access_count += 1;
                cached.last_accessed = Instant::now();
                return Some(cached.clone());
            }
        }
        None
    }

    fn put_relationship_graph(&self, key: &str, graph: &RelationshipGraph) {
        let cached_graph = CachedRelationshipGraph {
            graph: graph.clone(),
            cached_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 1,
        };

        if let Ok(mut cache) = self.relationship_cache.write() {
            cache.put(key.to_string(), cached_graph);
        }
    }

    fn generate_query_cache_key(&self, query: &str, workspace: &str, max_tokens: u32) -> String {
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        query.hash(&mut hasher);
        workspace.hash(&mut hasher);
        max_tokens.hash(&mut hasher);
        
        format!("query:{:x}", hasher.finish())
    }

    fn is_query_result_expired(&self, cached: &CachedQueryResult) -> bool {
        cached.cached_at.elapsed() > self.config.query_cache_ttl
    }

    fn is_relationship_graph_expired(&self, cached: &CachedRelationshipGraph) -> bool {
        cached.cached_at.elapsed() > self.config.relationship_cache_ttl
    }

    fn is_relevance_score_expired(&self, cached: &CachedRelevanceScore) -> bool {
        cached.cached_at.elapsed() > self.config.relevance_cache_ttl
    }

    fn record_cache_hit(&self, cache_type: &str) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.total_hits += 1;
            match cache_type {
                "semantic" => stats.semantic_hits += 1,
                "query" => stats.query_hits += 1,
                "ast" => stats.ast_hits += 1,
                "relationship" => stats.relationship_hits += 1,
                "relevance" => stats.relevance_hits += 1,
                _ => {}
            }
        }
    }

    fn record_cache_miss(&self, cache_type: &str) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.total_misses += 1;
            match cache_type {
                "semantic" => stats.semantic_misses += 1,
                "query" => stats.query_misses += 1,
                "ast" => stats.ast_misses += 1,
                "relationship" => stats.relationship_misses += 1,
                "relevance" => stats.relevance_misses += 1,
                _ => {}
            }
        }
    }
}

/// Cache warming service for proactive cache population
pub struct CacheWarmingService {
    config: CacheConfig,
}

impl CacheWarmingService {
    pub fn new(config: CacheConfig) -> Self {
        Self { config }
    }

    pub async fn warm_cache(&self, workspace: &str, cache: &ContextCache) -> Result<(), CheckpointError> {
        // Warm up frequently accessed semantic contexts
        self.warm_semantic_contexts(workspace, cache).await?;
        
        // Warm up common query patterns
        self.warm_query_patterns(workspace, cache).await?;
        
        // Warm up ASTs for recently modified files
        self.warm_recent_asts(workspace, cache).await?;
        
        Ok(())
    }

    async fn warm_semantic_contexts(&self, _workspace: &str, _cache: &ContextCache) -> Result<(), CheckpointError> {
        // Implementation would identify frequently accessed semantic contexts
        // and preload them into cache
        Ok(())
    }

    async fn warm_query_patterns(&self, _workspace: &str, _cache: &ContextCache) -> Result<(), CheckpointError> {
        // Implementation would identify common query patterns
        // and preload their results
        Ok(())
    }

    async fn warm_recent_asts(&self, _workspace: &str, _cache: &ContextCache) -> Result<(), CheckpointError> {
        // Implementation would identify recently modified files
        // and preload their ASTs
        Ok(())
    }
}

// Cached data structures

#[derive(Debug, Clone)]
struct CachedSemanticContext {
    context: SemanticContext,
    cached_at: Instant,
    last_accessed: Instant,
    access_count: u32,
}

#[derive(Debug, Clone)]
struct CachedQueryResult {
    context: CompleteAIContext,
    cached_at: Instant,
    last_accessed: Instant,
    access_count: u32,
}

#[derive(Debug, Clone)]
struct CachedAST {
    ast: AST,
    cached_at: Instant,
    last_accessed: Instant,
    access_count: u32,
}

#[derive(Debug, Clone)]
struct CachedRelationshipGraph {
    graph: RelationshipGraph,
    cached_at: Instant,
    last_accessed: Instant,
    access_count: u32,
}

#[derive(Debug, Clone)]
struct CachedRelevanceScore {
    score: f64,
    cached_at: Instant,
    access_count: u32,
}

// Configuration and statistics

#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub semantic_cache_size: usize,
    pub query_cache_size: usize,
    pub ast_cache_size: usize,
    pub relationship_cache_size: usize,
    pub relevance_cache_size: usize,
    pub query_cache_ttl: Duration,
    pub relationship_cache_ttl: Duration,
    pub relevance_cache_ttl: Duration,
    pub enable_cache_warming: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            semantic_cache_size: 1000,
            query_cache_size: 500,
            ast_cache_size: 2000,
            relationship_cache_size: 100,
            relevance_cache_size: 5000,
            query_cache_ttl: Duration::from_secs(3600), // 1 hour
            relationship_cache_ttl: Duration::from_secs(7200), // 2 hours
            relevance_cache_ttl: Duration::from_secs(1800), // 30 minutes
            enable_cache_warming: true,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CacheStatistics {
    pub total_hits: u64,
    pub total_misses: u64,
    pub semantic_hits: u64,
    pub semantic_misses: u64,
    pub query_hits: u64,
    pub query_misses: u64,
    pub ast_hits: u64,
    pub ast_misses: u64,
    pub relationship_hits: u64,
    pub relationship_misses: u64,
    pub relevance_hits: u64,
    pub relevance_misses: u64,
}

impl CacheStatistics {
    pub fn hit_rate(&self) -> f64 {
        if self.total_hits + self.total_misses == 0 {
            0.0
        } else {
            self.total_hits as f64 / (self.total_hits + self.total_misses) as f64
        }
    }

    pub fn semantic_hit_rate(&self) -> f64 {
        if self.semantic_hits + self.semantic_misses == 0 {
            0.0
        } else {
            self.semantic_hits as f64 / (self.semantic_hits + self.semantic_misses) as f64
        }
    }

    pub fn query_hit_rate(&self) -> f64 {
        if self.query_hits + self.query_misses == 0 {
            0.0
        } else {
            self.query_hits as f64 / (self.query_hits + self.query_misses) as f64
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MemoryUsage {
    pub semantic_cache_mb: usize,
    pub query_cache_mb: usize,
    pub ast_cache_mb: usize,
    pub relationship_cache_mb: usize,
    pub relevance_cache_mb: usize,
    pub total_mb: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_creation() {
        let config = CacheConfig::default();
        let cache = ContextCache::new(config);
        
        let stats = cache.get_statistics();
        assert_eq!(stats.total_hits, 0);
        assert_eq!(stats.total_misses, 0);
    }

    #[test]
    fn test_cache_key_generation() {
        let config = CacheConfig::default();
        let cache = ContextCache::new(config);
        
        let key1 = cache.generate_query_cache_key("test query", "/workspace", 8000);
        let key2 = cache.generate_query_cache_key("test query", "/workspace", 8000);
        let key3 = cache.generate_query_cache_key("different query", "/workspace", 8000);
        
        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_memory_usage_calculation() {
        let config = CacheConfig::default();
        let cache = ContextCache::new(config);
        
        let usage = cache.get_memory_usage();
        assert_eq!(usage.total_mb, 0); // Empty cache
    }
}
