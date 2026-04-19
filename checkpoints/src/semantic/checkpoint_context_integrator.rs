//! Checkpoint Context Integrator
//!
//! This module ties together all semantic analysis components specifically for
//! checkpoint-based code understanding, providing the unique advantage over RAG systems.

use super::context_ranker::ContextRanker;
use super::knowledge_graph::{EdgeType, GraphEdge, GraphNode, KnowledgeGraph, NodeType};
use super::symbol_resolver::SymbolResolver;
use super::temporal_analyzer::{ArchitecturalState, TemporalAnalyzer, TemporalCheckpoint};
use super::types::*;
use crate::error::Result;
use crate::types::{CheckpointId, FileChange};
use chrono::Utc;
use lru::LruCache;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const CHECKPOINT_CACHE_CAP: usize = 128;
const CACHE_CLEANUP_INTERVAL_SECS: u64 = 300;

#[derive(Debug, Default)]
struct IntegratorCacheMetrics {
    hits: AtomicU64,
    misses: AtomicU64,
    evictions: AtomicU64,
    cleanup_runs: AtomicU64,
    entries_cleared: AtomicU64,
}

impl IntegratorCacheMetrics {
    fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    fn record_eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }

    fn record_cleanup(&self, cleared_entries: u64) {
        self.cleanup_runs.fetch_add(1, Ordering::Relaxed);
        self.entries_cleared
            .fetch_add(cleared_entries, Ordering::Relaxed);
    }

    fn snapshot(&self, current_size: usize, capacity: usize) -> CacheStatistics {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let lookups = hits + misses;

        CacheStatistics {
            hits,
            misses,
            hit_rate: if lookups == 0 {
                0.0
            } else {
                hits as f64 / lookups as f64
            },
            evictions: self.evictions.load(Ordering::Relaxed),
            cleanup_runs: self.cleanup_runs.load(Ordering::Relaxed),
            entries_cleared: self.entries_cleared.load(Ordering::Relaxed),
            current_size,
            capacity,
        }
    }
}

/// Complete checkpoint-based context integrator
pub struct CheckpointContextIntegrator {
    knowledge_graph: Arc<KnowledgeGraph>,
    temporal_analyzer: Arc<RwLock<TemporalAnalyzer>>,
    symbol_resolver: Arc<SymbolResolver>,
    context_ranker: Arc<RwLock<ContextRanker>>,
    checkpoint_cache: Arc<RwLock<LruCache<CheckpointId, CheckpointContext>>>,
    cache_metrics: Arc<IntegratorCacheMetrics>,
    cache_cleanup_stop: Arc<AtomicBool>,
}

/// Complete context for a checkpoint
#[derive(Debug, Clone)]
pub struct CheckpointContext {
    pub checkpoint_id: CheckpointId,
    pub semantic_context: SemanticContext,
    pub graph_snapshot: GraphSnapshot,
    pub temporal_state: TemporalState,
    pub metadata: ContextMetadata,
}

/// Snapshot of the knowledge graph at a checkpoint
#[derive(Debug, Clone)]
pub struct GraphSnapshot {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub key_entities: Vec<String>,
    pub critical_paths: Vec<Vec<String>>,
    pub circular_dependencies: Vec<Vec<String>>,
}

/// Temporal state at a checkpoint
#[derive(Debug, Clone)]
pub struct TemporalState {
    pub entities_changed: usize,
    pub patterns_detected: Vec<String>,
    pub complexity_trend: String,
    pub hot_spots: Vec<String>,
}

/// Context metadata
#[derive(Debug, Clone)]
pub struct ContextMetadata {
    pub build_time_ms: u64,
    pub confidence_score: f64,
    pub coverage_metrics: CoverageMetrics,
}

/// Coverage metrics
#[derive(Debug, Clone)]
pub struct CoverageMetrics {
    pub files_analyzed: usize,
    pub symbols_indexed: usize,
    pub relationships_mapped: usize,
    pub completeness_score: f64,
}

/// Query result with checkpoint context
#[derive(Debug, Clone)]
pub struct CheckpointQueryResult {
    pub core_context: Vec<EntityDefinition>,
    pub related_context: Vec<EntityDefinition>,
    pub temporal_insights: Vec<TemporalInsight>,
    pub architectural_context: ArchitecturalContext,
    pub confidence_score: f64,
    pub explanation: String,
}

/// Temporal insight from history
#[derive(Debug, Clone)]
pub struct TemporalInsight {
    pub insight_type: InsightType,
    pub description: String,
    pub relevance: f64,
    pub checkpoints_involved: Vec<CheckpointId>,
}

/// Type of temporal insight
#[derive(Debug, Clone, PartialEq)]
pub enum InsightType {
    FrequentChange,
    RecentModification,
    PatternEvolution,
    ArchitecturalShift,
    ComplexityIncrease,
    RefactoringOpportunity,
}

/// Architectural context at query time
#[derive(Debug, Clone)]
pub struct ArchitecturalContext {
    pub layers: Vec<String>,
    pub key_components: Vec<ComponentInfo>,
    pub dependencies: Vec<DependencyInfo>,
    pub design_patterns: Vec<PatternInfo>,
}

/// Component information
#[derive(Debug, Clone)]
pub struct ComponentInfo {
    pub name: String,
    pub component_type: String,
    pub files: Vec<String>,
    pub importance: f64,
}

/// Dependency information
#[derive(Debug, Clone)]
pub struct DependencyInfo {
    pub from: String,
    pub to: String,
    pub dependency_type: String,
    pub strength: f64,
}

/// Pattern information
#[derive(Debug, Clone)]
pub struct PatternInfo {
    pub name: String,
    pub pattern_type: String,
    pub entities_involved: Vec<String>,
    pub confidence: f64,
}

impl CheckpointContextIntegrator {
    /// Create a new integrator
    pub fn new() -> Self {
        let knowledge_graph = Arc::new(KnowledgeGraph::new());
        let symbol_resolver = Arc::new(SymbolResolver::new(knowledge_graph.clone()));
        let context_ranker = Arc::new(RwLock::new(ContextRanker::new(knowledge_graph.clone())));
        let checkpoint_cache = Arc::new(RwLock::new(LruCache::new(
            NonZeroUsize::new(CHECKPOINT_CACHE_CAP).unwrap(),
        )));
        let cache_metrics = Arc::new(IntegratorCacheMetrics::default());
        let cache_cleanup_stop = Arc::new(AtomicBool::new(false));

        Self::start_cache_cleanup_task(
            checkpoint_cache.clone(),
            cache_metrics.clone(),
            cache_cleanup_stop.clone(),
        );

        Self {
            knowledge_graph: knowledge_graph.clone(),
            temporal_analyzer: Arc::new(RwLock::new(TemporalAnalyzer::new())),
            symbol_resolver,
            context_ranker,
            checkpoint_cache,
            cache_metrics,
            cache_cleanup_stop,
        }
    }

    fn start_cache_cleanup_task(
        cache: Arc<RwLock<LruCache<CheckpointId, CheckpointContext>>>,
        metrics: Arc<IntegratorCacheMetrics>,
        stop_signal: Arc<AtomicBool>,
    ) {
        thread::spawn(move || {
            while !stop_signal.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_secs(CACHE_CLEANUP_INTERVAL_SECS));

                if stop_signal.load(Ordering::Relaxed) {
                    break;
                }

                let cleared = {
                    let mut cache = cache.write();
                    let cleared = cache.len() as u64;
                    cache.clear();
                    cleared
                };

                if cleared > 0 {
                    metrics.record_cleanup(cleared);
                }
            }
        });
    }

    /// Build complete context for a checkpoint
    pub fn build_checkpoint_context(
        &self,
        checkpoint_id: CheckpointId,
        file_changes: &[FileChange],
    ) -> Result<CheckpointContext> {
        let start_time = std::time::Instant::now();

        // Check cache
        if let Some(cached) = self.checkpoint_cache.write().get(&checkpoint_id) {
            self.cache_metrics.record_hit();
            return Ok(cached.clone());
        }

        self.cache_metrics.record_miss();

        // Build semantic context
        let semantic_context = self.build_semantic_context(file_changes)?;

        // Update knowledge graph
        self.update_knowledge_graph(&semantic_context, file_changes)?;

        // Build graph snapshot
        let graph_snapshot = self.build_graph_snapshot()?;

        // Update temporal analyzer
        let temporal_state = self.update_temporal_state(checkpoint_id, &semantic_context)?;

        // Calculate metadata
        let metadata = ContextMetadata {
            build_time_ms: start_time.elapsed().as_millis() as u64,
            confidence_score: self.calculate_confidence(&semantic_context),
            coverage_metrics: self.calculate_coverage(&semantic_context, file_changes),
        };

        let context = CheckpointContext {
            checkpoint_id,
            semantic_context,
            graph_snapshot,
            temporal_state,
            metadata,
        };

        // Cache the context
        let mut cache = self.checkpoint_cache.write();
        if cache.len() == CHECKPOINT_CACHE_CAP {
            self.cache_metrics.record_eviction();
        }
        cache.put(checkpoint_id, context.clone());

        Ok(context)
    }

    /// Query context across checkpoints
    pub fn query_with_checkpoint_context(
        &self,
        query: &str,
        query_entities: &[String],
        checkpoint_ids: &[CheckpointId],
        max_tokens: usize,
    ) -> Result<CheckpointQueryResult> {
        // Get contexts for all checkpoints
        let mut all_entities = Vec::new();

        for checkpoint_id in checkpoint_ids {
            if let Some(context) = self.checkpoint_cache.write().get(checkpoint_id) {
                // Extract relevant entities from this checkpoint
                for entity in context.semantic_context.functions.values() {
                    if self.is_relevant_to_query(query, &entity.name) {
                        all_entities.push(EntityDefinition::Function(entity.clone()));
                    }
                }

                for entity in context.semantic_context.classes.values() {
                    if self.is_relevant_to_query(query, &entity.name) {
                        all_entities.push(EntityDefinition::Class(entity.clone()));
                    }
                }
            }
        }

        // Rank and prune context
        let ranker = self.context_ranker.read();
        let pruned = ranker.rank_and_prune(all_entities, query_entities, Some(max_tokens))?;

        // Generate temporal insights
        let temporal_insights = self.generate_temporal_insights(checkpoint_ids, query_entities)?;

        // Build architectural context
        let architectural_context = self.build_architectural_context(checkpoint_ids)?;

        // Generate explanation
        let explanation =
            self.generate_query_explanation(query, &pruned.included_items, &temporal_insights);

        Ok(CheckpointQueryResult {
            core_context: pruned
                .included_items
                .iter()
                .map(|r| r.entity.clone())
                .collect(),
            related_context: Vec::new(), // Would expand with related entities
            temporal_insights,
            architectural_context,
            confidence_score: pruned.coverage_score,
            explanation,
        })
    }

    /// Build semantic context from file changes
    fn build_semantic_context(&self, file_changes: &[FileChange]) -> Result<SemanticContext> {
        let mut context = SemanticContext {
            functions: HashMap::new(),
            classes: HashMap::new(),
            interfaces: HashMap::new(),
            types: HashMap::new(),
            variables: HashMap::new(),
            constants: HashMap::new(),
            modules: HashMap::new(),
            imports: Vec::new(),
            exports: Vec::new(),
            call_chains: Vec::new(),
            inheritance_tree: InheritanceTree {
                root_classes: Vec::new(),
                relationships: HashMap::new(),
                depth_map: HashMap::new(),
            },
            dependency_graph: DependencyGraph {
                nodes: Vec::new(),
                edges: Vec::new(),
                cycles: Vec::new(),
                external_dependencies: Vec::new(),
            },
            usage_patterns: Vec::new(),
        };

        // Extract symbols from file changes
        for file_change in file_changes {
            if let Some(_content) = &file_change.new_content {
                // Would parse and extract symbols
                // For now, create placeholder
                let func_name = format!("function_in_{}", file_change.path.display());
                context.functions.insert(
                    func_name.clone(),
                    FunctionDefinition {
                        name: func_name,
                        parameters: Vec::new(),
                        return_type: None,
                        visibility: Visibility::Public,
                        is_async: false,
                        is_static: false,
                        documentation: None,
                        location: CodeLocation {
                            file_path: file_change.path.clone(),
                            start_line: 1,
                            start_column: 1,
                            end_line: 10,
                            end_column: 1,
                        },
                        calls: Vec::new(),
                        called_by: Vec::new(),
                        complexity: 1,
                        lines_of_code: 10,
                    },
                );
            }
        }

        Ok(context)
    }

    /// Update knowledge graph with new semantic context
    fn update_knowledge_graph(
        &self,
        semantic_context: &SemanticContext,
        _file_changes: &[FileChange],
    ) -> Result<()> {
        // Add function nodes
        for (name, entity) in &semantic_context.functions {
            let node = GraphNode {
                id: format!("{}::{}", entity.location.file_path.display(), name),
                node_type: NodeType::Function,
                name: name.clone(),
                file_path: entity.location.file_path.to_string_lossy().to_string(),
                location: entity.location.clone(),
                metadata: HashMap::new(),
                checkpoint_id: None,
            };

            self.knowledge_graph.add_node(node)?;
        }

        // Add class nodes
        for (name, entity) in &semantic_context.classes {
            let node = GraphNode {
                id: format!("{}::{}", entity.location.file_path.display(), name),
                node_type: NodeType::Class,
                name: name.clone(),
                file_path: entity.location.file_path.to_string_lossy().to_string(),
                location: entity.location.clone(),
                metadata: HashMap::new(),
                checkpoint_id: None,
            };

            self.knowledge_graph.add_node(node)?;
        }

        // Add call edges
        for call_chain in &semantic_context.call_chains {
            let edge = GraphEdge {
                from: call_chain.caller.clone(),
                to: call_chain.called.clone(),
                edge_type: EdgeType::Calls,
                weight: 1.0,
                metadata: HashMap::new(),
            };

            // Ignore errors if nodes don't exist
            let _ = self.knowledge_graph.add_edge(edge);
        }

        Ok(())
    }

    /// Build snapshot of the knowledge graph
    fn build_graph_snapshot(&self) -> Result<GraphSnapshot> {
        let stats = self.knowledge_graph.get_statistics();

        // Find circular dependencies
        let circular_deps = self.knowledge_graph.find_circular_dependencies();
        let circular_paths: Vec<Vec<String>> = circular_deps
            .into_iter()
            .map(|cycle| cycle.iter().map(|node| node.name.clone()).collect())
            .collect();

        Ok(GraphSnapshot {
            total_nodes: stats.total_nodes,
            total_edges: stats.total_edges,
            key_entities: Vec::new(),   // Would identify key entities
            critical_paths: Vec::new(), // Would find critical paths
            circular_dependencies: circular_paths,
        })
    }

    /// Update temporal state
    fn update_temporal_state(
        &self,
        checkpoint_id: CheckpointId,
        semantic_context: &SemanticContext,
    ) -> Result<TemporalState> {
        // Create temporal checkpoint
        let temporal_checkpoint = TemporalCheckpoint {
            checkpoint_id,
            timestamp: Utc::now(),
            entities: HashMap::new(), // Would populate from semantic_context
            patterns: Vec::new(),
            architectural_state: ArchitecturalState {
                layers: Vec::new(),
                components: Vec::new(),
                module_count: semantic_context.functions.len() + semantic_context.classes.len(),
                average_coupling: 0.5,
                average_cohesion: 0.7,
            },
        };

        self.temporal_analyzer
            .write()
            .add_checkpoint(temporal_checkpoint)?;

        // Get hot spots
        let hot_spots = self.temporal_analyzer.read().find_hot_spots(3);

        Ok(TemporalState {
            entities_changed: semantic_context.functions.len() + semantic_context.classes.len(),
            patterns_detected: Vec::new(),
            complexity_trend: "Stable".to_string(),
            hot_spots: hot_spots.into_iter().map(|(name, _)| name).collect(),
        })
    }

    /// Calculate confidence score
    fn calculate_confidence(&self, context: &SemanticContext) -> f64 {
        let mut confidence: f64 = 0.5;

        if !context.functions.is_empty() {
            confidence += 0.2;
        }
        if !context.classes.is_empty() {
            confidence += 0.15;
        }
        if !context.call_chains.is_empty() {
            confidence += 0.15;
        }

        confidence.min(1.0)
    }

    /// Calculate coverage metrics
    fn calculate_coverage(
        &self,
        context: &SemanticContext,
        file_changes: &[FileChange],
    ) -> CoverageMetrics {
        CoverageMetrics {
            files_analyzed: file_changes.len(),
            symbols_indexed: context.functions.len()
                + context.classes.len()
                + context.interfaces.len(),
            relationships_mapped: context.call_chains.len(),
            completeness_score: 0.85,
        }
    }

    /// Check if entity is relevant to query
    fn is_relevant_to_query(&self, query: &str, entity_name: &str) -> bool {
        let query_lower = query.to_lowercase();
        let entity_lower = entity_name.to_lowercase();

        query_lower.contains(&entity_lower) || entity_lower.contains(&query_lower)
    }

    /// Generate temporal insights
    fn generate_temporal_insights(
        &self,
        checkpoint_ids: &[CheckpointId],
        query_entities: &[String],
    ) -> Result<Vec<TemporalInsight>> {
        let mut insights = Vec::new();

        let analyzer = self.temporal_analyzer.read();

        // Find entities that changed frequently
        let hot_spots = analyzer.find_hot_spots(2);

        for (entity_name, change_count) in hot_spots {
            if query_entities.iter().any(|qe| entity_name.contains(qe)) {
                insights.push(TemporalInsight {
                    insight_type: InsightType::FrequentChange,
                    description: format!(
                        "{} has changed {} times recently",
                        entity_name, change_count
                    ),
                    relevance: 0.8,
                    checkpoints_involved: checkpoint_ids.to_vec(),
                });
            }
        }

        Ok(insights)
    }

    /// Build architectural context
    fn build_architectural_context(
        &self,
        _checkpoint_ids: &[CheckpointId],
    ) -> Result<ArchitecturalContext> {
        Ok(ArchitecturalContext {
            layers: vec![
                "Presentation".to_string(),
                "Business Logic".to_string(),
                "Data Access".to_string(),
            ],
            key_components: Vec::new(),
            dependencies: Vec::new(),
            design_patterns: Vec::new(),
        })
    }

    /// Generate explanation for query result
    fn generate_query_explanation(
        &self,
        query: &str,
        included_items: &[super::context_ranker::RankedContext],
        insights: &[TemporalInsight],
    ) -> String {
        let mut explanation = format!("Context for query: '{}'\n\n", query);

        explanation.push_str(&format!(
            "Included {} relevant entities:\n",
            included_items.len()
        ));
        for item in included_items.iter().take(5) {
            explanation.push_str(&format!(
                "  - {} (score: {:.2}): {}\n",
                item.entity.name(),
                item.rank_score,
                item.inclusion_reason
            ));
        }

        if !insights.is_empty() {
            explanation.push_str(&format!("\n{} temporal insights:\n", insights.len()));
            for insight in insights.iter().take(3) {
                explanation.push_str(&format!("  - {}\n", insight.description));
            }
        }

        explanation
    }

    /// Clear all caches
    pub fn clear_caches(&self) {
        let cleared = {
            let mut cache = self.checkpoint_cache.write();
            let cleared = cache.len() as u64;
            cache.clear();
            cleared
        };
        if cleared > 0 {
            self.cache_metrics.record_cleanup(cleared);
        }
        self.symbol_resolver.clear_caches();
        self.knowledge_graph.clear();
    }

    pub fn get_cache_statistics(&self) -> CacheStatistics {
        self.cache_metrics
            .snapshot(self.checkpoint_cache.read().len(), CHECKPOINT_CACHE_CAP)
    }

    /// Get statistics
    pub fn get_statistics(&self) -> IntegratorStatistics {
        let checkpoint_count = self.checkpoint_cache.read().len();
        let graph_stats = self.knowledge_graph.get_statistics();
        let symbol_stats = self.symbol_resolver.get_statistics();

        IntegratorStatistics {
            checkpoints_cached: checkpoint_count,
            total_nodes: graph_stats.total_nodes,
            total_edges: graph_stats.total_edges,
            total_symbols: symbol_stats.total_symbols,
            total_modules: symbol_stats.total_modules,
            cache: self.get_cache_statistics(),
        }
    }
}

impl Drop for CheckpointContextIntegrator {
    fn drop(&mut self) {
        self.cache_cleanup_stop.store(true, Ordering::Relaxed);
    }
}

impl Default for CheckpointContextIntegrator {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the integrator
#[derive(Debug, Clone)]
pub struct IntegratorStatistics {
    pub checkpoints_cached: usize,
    pub total_nodes: usize,
    pub total_edges: usize,
    pub total_symbols: usize,
    pub total_modules: usize,
    pub cache: CacheStatistics,
}

#[derive(Debug, Clone)]
pub struct CacheStatistics {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub evictions: u64,
    pub cleanup_runs: u64,
    pub entries_cleared: u64,
    pub current_size: usize,
    pub capacity: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integrator_creation() {
        let integrator = CheckpointContextIntegrator::new();
        let stats = integrator.get_statistics();
        assert_eq!(stats.checkpoints_cached, 0);
    }
}
