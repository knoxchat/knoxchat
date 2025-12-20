//! Unified AI Context Engine
//! 
//! Main engine that orchestrates all AI context components and provides a unified interface

use super::*;
use crate::manager::CheckpointManager;
use crate::semantic::SemanticAnalyzer;
use crate::ai_context::context_builder::{ArchitecturalContext, ProjectStructure, DependencyGraph, RelationshipContext, CallGraph, TypeHierarchy, ImportGraph, HistoryContext, ContextBuildMetadata};

use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// The main unified AI context engine
pub struct UnifiedAIContextEngine {
    /// Checkpoint manager
    checkpoint_manager: Arc<CheckpointManager>,
    /// Enhanced AI context manager from existing system
    ai_context_manager: Arc<crate::ai_context_manager::AIContextManager>,
    /// Context building pipeline
    context_builder: Arc<ContextBuildingPipeline>,
    /// Cache manager
    cache_manager: Arc<ContextCacheManager>,
    /// Performance monitor
    performance_monitor: Arc<PerformanceMonitor>,
    /// Configuration
    config: Arc<RwLock<AIContextConfig>>,
    /// Engine state
    state: Arc<RwLock<EngineState>>,
}

/// Engine state information
#[derive(Debug, Clone)]
struct EngineState {
    /// Whether the engine is initialized
    initialized: bool,
    /// Number of active operations
    active_operations: usize,
    /// Last operation timestamp
    last_operation: Option<chrono::DateTime<chrono::Utc>>,
    /// Engine statistics
    statistics: EngineStatistics,
}

/// Engine performance statistics
#[derive(Debug, Clone, Default)]
struct EngineStatistics {
    /// Total number of context builds
    total_context_builds: u64,
    /// Total number of cache hits
    cache_hits: u64,
    /// Total number of cache misses
    cache_misses: u64,
    /// Average build time in milliseconds
    average_build_time_ms: f64,
    /// Total number of errors
    total_errors: u64,
}

impl UnifiedAIContextEngine {
    /// Create a new unified AI context engine
    pub async fn new(
        checkpoint_manager: CheckpointManager,
        config: AIContextConfig,
    ) -> Result<Self> {
        let checkpoint_manager = Arc::new(checkpoint_manager);
        // Create semantic analyzer
        let semantic_analyzer = Arc::new(SemanticAnalyzer::new()
            .map_err(|e| AIContextError::ConfigurationError(format!("Failed to create semantic analyzer: {}", e)))?);
        
        // Create AI context manager (simplified for now)
        // let ai_context_manager = Arc::new(crate::ai_context_manager::AIContextManager::new((*checkpoint_manager).clone())
        //     .map_err(|e| AIContextError::ConfigurationError(format!("Failed to create AI context manager: {}", e)))?);
        
        // Create context building pipeline
        let context_builder = Arc::new(ContextBuildingPipeline::new(
            checkpoint_manager.clone(),
            semantic_analyzer,
            config.clone(),
        )?);
        
        // Create cache manager
        let cache_manager = Arc::new(ContextCacheManager::new(config.cache_config.clone()));
        
        // Create performance monitor
        let performance_monitor = Arc::new(PerformanceMonitor::new(config.performance_config.clone())?);
        
        // Initialize engine state
        let state = Arc::new(RwLock::new(EngineState {
            initialized: true,
            active_operations: 0,
            last_operation: None,
            statistics: EngineStatistics::default(),
        }));
        
        Ok(Self {
            checkpoint_manager: checkpoint_manager.clone(),
            ai_context_manager: Arc::new(crate::ai_context_manager::AIContextManager::new(
                crate::manager::CheckpointManager::new(
                    crate::config::CheckpointConfig::default(),
                    std::path::PathBuf::from("."),
                    uuid::Uuid::new_v4()
                )?
            )?),
            context_builder,
            cache_manager,
            performance_monitor,
            config: Arc::new(RwLock::new(config)),
            state,
        })
    }

    /// Build AI context for a query
    pub async fn build_context_for_query(
        &self,
        query: &str,
        options: Option<ContextOptions>,
    ) -> Result<AIContextResult> {
        let start_time = Instant::now();
        let operation_id = uuid::Uuid::new_v4().to_string();
        
        // Increment active operations
        {
            let mut state = self.state.write().await;
            state.active_operations += 1;
            state.last_operation = Some(chrono::Utc::now());
        }
        
        // Check cache first
        let cache_key = self.generate_cache_key(query, &options);
        if let Some(cached_result) = self.cache_manager.get_context(&cache_key).await {
            // Update statistics
            {
                let mut state = self.state.write().await;
                state.active_operations -= 1;
                state.statistics.cache_hits += 1;
            }
            
            return Ok(cached_result);
        }
        
        // Cache miss - build context
        let result = self.build_context_internal(query, options.unwrap_or_default(), &operation_id).await;
        
        // Update statistics
        {
            let mut state = self.state.write().await;
            state.active_operations -= 1;
            state.statistics.total_context_builds += 1;
            state.statistics.cache_misses += 1;
            
            match &result {
                Ok(_) => {
                    let build_time_ms = start_time.elapsed().as_millis() as f64;
                    state.statistics.average_build_time_ms = 
                        (state.statistics.average_build_time_ms * (state.statistics.total_context_builds - 1) as f64 + build_time_ms) 
                        / state.statistics.total_context_builds as f64;
                }
                Err(_) => {
                    state.statistics.total_errors += 1;
                }
            }
        }
        
        // Cache successful results
        if let Ok(ref result) = result {
            self.cache_manager.cache_context(cache_key, result.clone()).await;
        }
        
        result
    }

    /// Get AI context for a specific checkpoint
    pub async fn get_context_for_checkpoint(
        &self,
        checkpoint_id: &str,
    ) -> Result<AIContextResult> {
        let start_time = Instant::now();
        
        // Check cache first
        let cache_key = format!("checkpoint:{}", checkpoint_id);
        if let Some(cached_result) = self.cache_manager.get_context(&cache_key).await {
            return Ok(cached_result);
        }
        
        // Get checkpoint from AI context manager
        let ai_checkpoint = self.ai_context_manager.get_ai_checkpoint(
            uuid::Uuid::parse_str(checkpoint_id)
                .map_err(|e| AIContextError::ContextBuildingFailed(format!("Invalid checkpoint ID: {}", e)))?
        )
        .map_err(|e| AIContextError::ContextBuildingFailed(format!("Failed to get checkpoint: {}", e)))?
        .ok_or_else(|| AIContextError::ContextBuildingFailed("Checkpoint not found".to_string()))?;
        
        // Convert AI checkpoint to complete context
        let complete_context = self.convert_ai_checkpoint_to_context(&ai_checkpoint).await?;
        
        // Create metadata
        let metadata = ContextMetadata {
            checkpoints_used: vec![uuid::Uuid::parse_str(checkpoint_id).unwrap()],
            context_type: "checkpoint".to_string(),
            confidence_score: ai_checkpoint.confidence_score,
            token_count: self.estimate_token_count(&complete_context),
            semantic_entities_count: self.count_semantic_entities(&complete_context),
            architectural_patterns: self.extract_architectural_patterns(&complete_context),
            generated_at: chrono::Utc::now(),
            build_duration: start_time.elapsed(),
        };
        
        // Create performance metrics
        let performance_metrics = PerformanceMetrics {
            build_time_ms: start_time.elapsed().as_millis() as u64,
            cache_hit: false,
            memory_usage_mb: 0.0, // Would be calculated
            cpu_usage_percent: 0.0, // Would be calculated
            checkpoints_analyzed: 1,
            files_processed: complete_context.core_files.len(),
            entities_extracted: metadata.semantic_entities_count,
        };
        
        // Create cache info
        let cache_info = CacheInfo {
            cache_hit: false,
            cache_key: cache_key.clone(),
            cached_at: None,
            ttl_remaining: None,
        };
        
        let result = AIContextResult {
            context: complete_context,
            metadata,
            performance: performance_metrics,
            cache_info,
        };
        
        // Cache the result
        self.cache_manager.cache_context(cache_key, result.clone()).await;
        
        Ok(result)
    }

    /// Analyze query intent without building full context
    pub async fn analyze_query_intent(&self, query: &str) -> Result<QueryIntent> {
        // Check intent cache first
        let cache_key = format!("intent:{}", self.hash_string(query));
        if let Some(cached_intent) = self.cache_manager.get_intent(&cache_key).await {
            return Ok(cached_intent);
        }
        
        // Create query analyzer if not part of pipeline
        let query_analyzer = QueryAnalyzer::new()?;
        let intent = query_analyzer.analyze_intent(query).await?;
        
        // Cache the intent
        self.cache_manager.cache_intent(cache_key, intent.clone()).await;
        
        Ok(intent)
    }

    /// Get semantic similarity for a query
    pub async fn get_semantic_similarity(
        &self,
        query: &str,
        threshold: f64,
    ) -> Result<Vec<ScoredCheckpoint>> {
        // Analyze query intent first
        let query_intent = self.analyze_query_intent(query).await?;
        
        // Search for similar checkpoints using AI context manager
        let similar_checkpoints = self.ai_context_manager.search_checkpoints_by_semantics(
            &query_intent.entities.iter().map(|e| e.name.clone()).collect::<Vec<_>>(),
            threshold,
        )
        .map_err(|e| AIContextError::RelevanceScoringFailed(e.to_string()))?;
        
        // Convert to scored checkpoints
        let mut scored_checkpoints = Vec::new();
        for checkpoint in similar_checkpoints {
            let relevance_score = RelevanceScore {
                semantic: 0.8, // Would be calculated properly
                temporal: 0.6,
                architectural: 0.7,
                dependency: 0.5,
                usage: 0.4,
                composite: 0.65,
                confidence: checkpoint.confidence_score,
                reasoning: "Semantic similarity match".to_string(),
            };
            
            scored_checkpoints.push(ScoredCheckpoint {
                checkpoint,
                score: relevance_score,
            });
        }
        
        Ok(scored_checkpoints)
    }

    /// Subscribe to real-time context updates
    pub async fn subscribe_to_context_updates<F>(&self, _callback: F) -> Result<String>
    where
        F: Fn(ContextUpdate) + Send + Sync + 'static,
    {
        // This would implement a subscription system for real-time updates
        // For now, return a subscription ID
        Ok(uuid::Uuid::new_v4().to_string())
    }

    /// Update context incrementally with new changes
    pub async fn update_context_incremental(
        &self,
        changes: &[crate::types::FileChange],
    ) -> Result<()> {
        // Use AI context manager's incremental update capability
        for change in changes {
            // Find checkpoints that contain this file
            let all_checkpoints = self.checkpoint_manager.list_checkpoints(None)
                .map_err(|e| AIContextError::ContextBuildingFailed(e.to_string()))?;
            
            for checkpoint in all_checkpoints {
                if checkpoint.file_changes.iter().any(|fc| fc.path == change.path) {
                    self.ai_context_manager.update_semantic_context_incremental(
                        checkpoint.id,
                        &[change.clone()],
                    )
                    .map_err(|e| AIContextError::ContextBuildingFailed(e.to_string()))?;
                }
            }
        }
        
        // Invalidate relevant caches
        // This is simplified - in practice would be more targeted
        self.cache_manager.clear_all().await;
        
        Ok(())
    }

    /// Get performance metrics for the engine
    pub async fn get_performance_metrics(&self) -> Result<EnginePerformanceMetrics> {
        let state = self.state.read().await;
        let cache_stats = self.cache_manager.get_stats().await;
        
        Ok(EnginePerformanceMetrics {
            total_context_builds: state.statistics.total_context_builds,
            cache_hit_ratio: if state.statistics.cache_hits + state.statistics.cache_misses > 0 {
                state.statistics.cache_hits as f64 / (state.statistics.cache_hits + state.statistics.cache_misses) as f64
            } else {
                0.0
            },
            average_build_time_ms: state.statistics.average_build_time_ms,
            active_operations: state.active_operations,
            cache_stats,
            memory_usage_mb: 0.0, // Would be calculated
            uptime_seconds: chrono::Utc::now().timestamp() as u64, // Simplified
        })
    }

    /// Optimize caches and performance
    pub async fn optimize_cache(&self) -> Result<OptimizationReport> {
        // Clear expired entries
        self.cache_manager.clear_all().await;
        
        // This would implement more sophisticated optimization
        Ok(OptimizationReport {
            cache_entries_removed: 0,
            memory_freed_mb: 0.0,
            optimization_time_ms: 0,
            recommendations: vec![],
        })
    }

    /// Shutdown the engine gracefully
    pub async fn shutdown(&self) -> Result<()> {
        // Wait for active operations to complete
        loop {
            let active_ops = {
                let state = self.state.read().await;
                state.active_operations
            };
            
            if active_ops == 0 {
                break;
            }
            
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        
        // Clear caches
        self.cache_manager.clear_all().await;
        
        // Mark as not initialized
        {
            let mut state = self.state.write().await;
            state.initialized = false;
        }
        
        Ok(())
    }

    // Private helper methods

    /// Build context internally
    async fn build_context_internal(
        &self,
        query: &str,
        options: ContextOptions,
        _operation_id: &str,
    ) -> Result<AIContextResult> {
        let start_time = Instant::now();
        
        // Build complete context using the pipeline
        let complete_context = self.context_builder.build_context(query, options.clone()).await?;
        
        // Create metadata
        let metadata = ContextMetadata {
            checkpoints_used: vec![], // Would be populated by pipeline
            context_type: "query".to_string(),
            confidence_score: 0.8, // Would be calculated
            token_count: self.estimate_token_count(&complete_context),
            semantic_entities_count: self.count_semantic_entities(&complete_context),
            architectural_patterns: self.extract_architectural_patterns(&complete_context),
            generated_at: chrono::Utc::now(),
            build_duration: start_time.elapsed(),
        };
        
        // Create performance metrics
        let performance_metrics = PerformanceMetrics {
            build_time_ms: start_time.elapsed().as_millis() as u64,
            cache_hit: false,
            memory_usage_mb: 0.0, // Would be calculated
            cpu_usage_percent: 0.0, // Would be calculated
            checkpoints_analyzed: 0, // Would be populated
            files_processed: complete_context.core_files.len(),
            entities_extracted: metadata.semantic_entities_count,
        };
        
        // Create cache info
        let cache_info = CacheInfo {
            cache_hit: false,
            cache_key: self.generate_cache_key(query, &Some(options)),
            cached_at: None,
            ttl_remaining: None,
        };
        
        Ok(AIContextResult {
            context: complete_context,
            metadata,
            performance: performance_metrics,
            cache_info,
        })
    }

    /// Convert AI checkpoint to complete context
    async fn convert_ai_checkpoint_to_context(
        &self,
        _ai_checkpoint: &crate::semantic::AIContextCheckpoint,
    ) -> Result<CompleteAIContext> {
        // This is a simplified conversion - would be more comprehensive in practice
        let core_files = vec![]; // Would extract from checkpoint
        let architecture = ArchitecturalContext {
            project_structure: ProjectStructure {
                root_directories: vec![],
                modules: vec![],
                dependencies: vec![],
            },
            patterns_used: vec![],
            dependency_graph: DependencyGraph {
                nodes: vec![],
                edges: vec![],
                cycles: vec![],
            },
            layers: vec![],
        };
        
        Ok(CompleteAIContext {
            core_files,
            architecture,
            relationships: RelationshipContext {
                call_graph: CallGraph { functions: vec![], relationships: vec![] },
                type_hierarchy: TypeHierarchy { root_types: vec![], inheritance_chains: vec![], interface_implementations: vec![] },
                import_graph: ImportGraph { modules: vec![], dependencies: vec![] },
                usage_patterns: vec![],
            },
            history: HistoryContext {
                change_timeline: vec![],
                architectural_decisions: vec![],
                refactoring_history: vec![],
            },
            examples: vec![],
            metadata: ContextBuildMetadata {
                checkpoints_analyzed: 1,
                files_included: 0,
                total_lines_of_code: 0,
                build_strategy: "checkpoint".to_string(),
                estimated_tokens: 0,
            },
        })
    }

    /// Generate cache key for query and options
    fn generate_cache_key(&self, query: &str, options: &Option<ContextOptions>) -> String {
        let options_hash = if let Some(opts) = options {
            format!("{:?}", opts)
        } else {
            "default".to_string()
        };
        
        format!("query:{}:{}", self.hash_string(query), self.hash_string(&options_hash))
    }

    /// Simple hash function for strings
    fn hash_string(&self, s: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Estimate token count for context
    fn estimate_token_count(&self, context: &CompleteAIContext) -> usize {
        // Simple estimation - 4 characters per token on average
        let total_chars: usize = context.core_files.iter()
            .map(|f| f.content.len())
            .sum();
        
        total_chars / 4
    }

    /// Count semantic entities in context
    fn count_semantic_entities(&self, context: &CompleteAIContext) -> usize {
        context.core_files.iter()
            .map(|f| f.semantic_info.functions.len() + 
                     f.semantic_info.classes.len() + 
                     f.semantic_info.interfaces.len() +
                     f.semantic_info.types.len())
            .sum()
    }

    /// Extract architectural patterns from context
    fn extract_architectural_patterns(&self, context: &CompleteAIContext) -> Vec<String> {
        context.architecture.patterns_used.iter()
            .map(|p| p.name.clone())
            .collect()
    }
    
    // Note: get_performance_metrics method already exists above, removed duplicate
    
    /// Update engine configuration
    pub async fn update_configuration(&self, new_config: AIContextConfig) -> Result<()> {
        let mut config = self.config.write().await;
        *config = new_config;
        
        // Update performance monitor configuration if needed
        // Update performance monitor configuration
        // self.performance_monitor.update_config(config.performance_config.clone()).await;
        
        Ok(())
    }
    
    /// Get current configuration
    pub async fn get_configuration(&self) -> AIContextConfig {
        let config = self.config.read().await;
        config.clone()
    }
    
    /// Check if performance monitoring is enabled
    pub async fn is_performance_monitoring_enabled(&self) -> bool {
        let config = self.config.read().await;
        config.performance_config.enable_detailed_metrics
    }
    
    /// Record performance metrics for an operation
    pub async fn record_operation_metrics(&self, _operation_type: &str, duration_ms: f64) -> Result<()> {
        if self.is_performance_monitoring_enabled().await {
            // Record metrics using the performance monitor
            self.performance_monitor.record_build_time(
                std::time::Duration::from_millis(duration_ms as u64)
            ).await;
        }
        Ok(())
    }
}

/// Context update for real-time notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextUpdate {
    /// Type of update
    pub update_type: String,
    /// Affected checkpoint IDs
    pub checkpoint_ids: Vec<String>,
    /// Update timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Update details
    pub details: serde_json::Value,
}

/// Engine performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnginePerformanceMetrics {
    pub total_context_builds: u64,
    pub cache_hit_ratio: f64,
    pub average_build_time_ms: f64,
    pub active_operations: usize,
    pub cache_stats: CacheStats,
    pub memory_usage_mb: f64,
    pub uptime_seconds: u64,
}

/// Optimization report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationReport {
    pub cache_entries_removed: usize,
    pub memory_freed_mb: f64,
    pub optimization_time_ms: u64,
    pub recommendations: Vec<String>,
}
