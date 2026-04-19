//! Context Ranking and Pruning System
//!
//! This module implements intelligent ranking and pruning of context to fit within
//! token limits while maximizing relevance and informativeness.

use super::knowledge_graph::{GraphNode, KnowledgeGraph};
use super::types::*;
use crate::error::Result;
use std::collections::HashSet;
use std::sync::Arc;

/// Context ranker for intelligent context selection
pub struct ContextRanker {
    knowledge_graph: Arc<KnowledgeGraph>,
    ranking_config: RankingConfig,
}

/// Configuration for ranking algorithm
#[derive(Debug, Clone)]
pub struct RankingConfig {
    pub max_tokens: usize,
    pub min_confidence_threshold: f64,
    pub diversity_weight: f64,
    pub relevance_weight: f64,
    pub recency_weight: f64,
    pub centrality_weight: f64,
}

impl Default for RankingConfig {
    fn default() -> Self {
        Self {
            max_tokens: 8000,
            min_confidence_threshold: 0.3,
            diversity_weight: 0.2,
            relevance_weight: 0.4,
            recency_weight: 0.2,
            centrality_weight: 0.2,
        }
    }
}

/// Ranked context item
#[derive(Debug, Clone)]
pub struct RankedContext {
    pub entity: EntityDefinition,
    pub rank_score: f64,
    pub relevance_score: f64,
    pub importance_score: f64,
    pub recency_score: f64,
    pub centrality_score: f64,
    pub diversity_score: f64,
    pub estimated_tokens: usize,
    pub inclusion_reason: String,
}

/// Context pruning result
#[derive(Debug, Clone)]
pub struct PrunedContext {
    pub included_items: Vec<RankedContext>,
    pub excluded_items: Vec<RankedContext>,
    pub total_tokens: usize,
    pub coverage_score: f64,
    pub diversity_score: f64,
}

impl ContextRanker {
    /// Create a new context ranker
    pub fn new(knowledge_graph: Arc<KnowledgeGraph>) -> Self {
        Self {
            knowledge_graph,
            ranking_config: RankingConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(knowledge_graph: Arc<KnowledgeGraph>, config: RankingConfig) -> Self {
        Self {
            knowledge_graph,
            ranking_config: config,
        }
    }

    /// Rank and select context items within token budget
    pub fn rank_and_prune(
        &self,
        candidates: Vec<EntityDefinition>,
        query_entities: &[String],
        max_tokens: Option<usize>,
    ) -> Result<PrunedContext> {
        let token_budget = max_tokens.unwrap_or(self.ranking_config.max_tokens);

        // Calculate scores for all candidates
        let mut ranked_items: Vec<RankedContext> = candidates
            .into_iter()
            .map(|entity| self.calculate_ranking_scores(&entity, query_entities))
            .collect();

        // Sort by rank score (descending)
        ranked_items.sort_by(|a, b| b.rank_score.partial_cmp(&a.rank_score).unwrap());

        // Apply greedy selection with diversity constraint
        let (included, excluded) = self.greedy_selection_with_diversity(ranked_items, token_budget);

        // Calculate coverage and diversity metrics
        let coverage_score = self.calculate_coverage_score(&included, query_entities);
        let diversity_score = self.calculate_diversity_score(&included);
        let total_tokens: usize = included.iter().map(|item| item.estimated_tokens).sum();

        Ok(PrunedContext {
            included_items: included,
            excluded_items: excluded,
            total_tokens,
            coverage_score,
            diversity_score,
        })
    }

    /// Calculate comprehensive ranking scores for an entity
    fn calculate_ranking_scores(
        &self,
        entity: &EntityDefinition,
        query_entities: &[String],
    ) -> RankedContext {
        let relevance_score = self.calculate_relevance_score(entity, query_entities);
        let importance_score = self.calculate_importance_score(entity);
        let recency_score = 0.5; // Would be calculated from temporal data
        let centrality_score = self.calculate_centrality_score(entity);
        let diversity_score = 0.5; // Calculated during selection

        // Weighted composite score
        let rank_score = relevance_score * self.ranking_config.relevance_weight
            + importance_score
                * (1.0
                    - self.ranking_config.relevance_weight
                    - self.ranking_config.recency_weight
                    - self.ranking_config.centrality_weight)
            + recency_score * self.ranking_config.recency_weight
            + centrality_score * self.ranking_config.centrality_weight;

        RankedContext {
            entity: entity.clone(),
            rank_score,
            relevance_score,
            importance_score,
            recency_score,
            centrality_score,
            diversity_score,
            estimated_tokens: self.estimate_tokens(entity),
            inclusion_reason: self.generate_inclusion_reason(
                relevance_score,
                importance_score,
                centrality_score,
            ),
        }
    }

    /// Calculate relevance score based on query entities
    fn calculate_relevance_score(
        &self,
        entity: &EntityDefinition,
        query_entities: &[String],
    ) -> f64 {
        if query_entities.is_empty() {
            return 0.5;
        }

        let mut relevance: f64 = 0.0;

        // Check if entity name matches query
        for query_entity in query_entities {
            if entity
                .name()
                .to_lowercase()
                .contains(&query_entity.to_lowercase())
            {
                relevance += 0.4;
            }

            // Check if entity name is similar
            if self.fuzzy_match(entity.name(), query_entity) {
                relevance += 0.3;
            }

            // Check if entity is in same file as query entities
            // (would need more context to implement)
        }

        relevance.min(1.0)
    }

    /// Calculate importance score based on structural properties
    fn calculate_importance_score(&self, entity: &EntityDefinition) -> f64 {
        let mut importance = 0.0;

        // Public visibility is more important
        if entity.visibility() == "public" {
            importance += 0.3;
        }

        // Entities with documentation are more important
        if entity.documentation().is_some() {
            importance += 0.2;
        }

        // Calculate based on graph centrality
        let node_id = format!(
            "{}::{}",
            entity.location().file_path.display(),
            entity.name()
        );
        let neighbors = self.knowledge_graph.get_neighbors(&node_id);
        let degree_centrality = (neighbors.len() as f64) / 100.0; // Normalize
        importance += degree_centrality.min(0.5);

        importance.min(1.0)
    }

    /// Calculate centrality score in the knowledge graph
    fn calculate_centrality_score(&self, entity: &EntityDefinition) -> f64 {
        let node_id = format!(
            "{}::{}",
            entity.location().file_path.display(),
            entity.name()
        );

        // Degree centrality
        let in_edges = self.knowledge_graph.get_incoming_edges(&node_id);
        let out_edges = self.knowledge_graph.get_outgoing_edges(&node_id);
        let degree = in_edges.len() + out_edges.len();

        // Normalize (assuming max degree of 100)
        (degree as f64 / 100.0).min(1.0)
    }

    /// Greedy selection with diversity constraint
    fn greedy_selection_with_diversity(
        &self,
        mut ranked_items: Vec<RankedContext>,
        token_budget: usize,
    ) -> (Vec<RankedContext>, Vec<RankedContext>) {
        let mut included = Vec::new();
        let mut excluded = Vec::new();
        let mut current_tokens = 0;
        let mut selected_files = HashSet::new();
        let mut selected_types = HashSet::new();

        while let Some(item) = ranked_items.pop() {
            // Check if adding this item would exceed budget
            if current_tokens + item.estimated_tokens > token_budget {
                excluded.push(item);
                continue;
            }

            // Check minimum confidence threshold
            if item.rank_score < self.ranking_config.min_confidence_threshold {
                excluded.push(item);
                continue;
            }

            // Calculate diversity bonus
            let file_path = item.entity.location().file_path.clone();
            let entity_type = item.entity.entity_type_name().to_string();

            let diversity_bonus = if !selected_files.contains(&file_path) {
                0.1
            } else {
                0.0
            } + if !selected_types.contains(&entity_type) {
                0.1
            } else {
                0.0
            };

            // Apply diversity bonus to rank score
            let adjusted_score =
                item.rank_score + diversity_bonus * self.ranking_config.diversity_weight;

            // Re-insert with adjusted score if needed
            if diversity_bonus > 0.0 {
                let mut adjusted_item = item.clone();
                adjusted_item.rank_score = adjusted_score;
                adjusted_item.diversity_score = diversity_bonus;

                // Check if we should still include it
                if adjusted_score >= self.ranking_config.min_confidence_threshold {
                    current_tokens += adjusted_item.estimated_tokens;
                    selected_files.insert(file_path.clone());
                    selected_types.insert(entity_type.clone());
                    included.push(adjusted_item);
                } else {
                    excluded.push(adjusted_item);
                }
            } else {
                current_tokens += item.estimated_tokens;
                selected_files.insert(file_path.clone());
                selected_types.insert(entity_type.clone());
                included.push(item);
            }
        }

        // Re-sort included by rank score
        included.sort_by(|a, b| b.rank_score.partial_cmp(&a.rank_score).unwrap());

        (included, excluded)
    }

    /// Calculate coverage score
    fn calculate_coverage_score(
        &self,
        included: &[RankedContext],
        query_entities: &[String],
    ) -> f64 {
        if query_entities.is_empty() {
            return 1.0;
        }

        let mut covered_entities = 0;

        for query_entity in query_entities {
            for item in included {
                if item.entity.name().contains(query_entity) {
                    covered_entities += 1;
                    break;
                }
            }
        }

        covered_entities as f64 / query_entities.len() as f64
    }

    /// Calculate diversity score for selected items
    fn calculate_diversity_score(&self, included: &[RankedContext]) -> f64 {
        if included.is_empty() {
            return 0.0;
        }

        let mut unique_files = HashSet::new();
        let mut unique_types = HashSet::new();

        for item in included {
            unique_files.insert(item.entity.location().file_path.clone());
            unique_types.insert(item.entity.entity_type_name().to_string());
        }

        // Diversity is the ratio of unique files and types to total items
        let file_diversity = unique_files.len() as f64 / included.len() as f64;
        let type_diversity = unique_types.len() as f64 / included.len().min(4) as f64;

        (file_diversity + type_diversity) / 2.0
    }

    /// Estimate token count for an entity
    fn estimate_tokens(&self, entity: &EntityDefinition) -> usize {
        // Simple estimation: name + location + metadata
        let base_tokens = 50;
        let name_tokens = entity.name().len() / 4;
        let doc_tokens = entity
            .documentation()
            .as_ref()
            .map(|d| d.len() / 4)
            .unwrap_or(0);

        base_tokens + name_tokens + doc_tokens
    }

    /// Generate inclusion reason
    fn generate_inclusion_reason(
        &self,
        relevance: f64,
        importance: f64,
        centrality: f64,
    ) -> String {
        let mut reasons = Vec::new();

        if relevance > 0.7 {
            reasons.push("high relevance to query");
        }
        if importance > 0.7 {
            reasons.push("high importance");
        }
        if centrality > 0.7 {
            reasons.push("central to codebase");
        }

        if reasons.is_empty() {
            reasons.push("included for context");
        }

        reasons.join(", ")
    }

    /// Fuzzy match two strings
    fn fuzzy_match(&self, s1: &str, s2: &str) -> bool {
        let s1_lower = s1.to_lowercase();
        let s2_lower = s2.to_lowercase();

        // Simple fuzzy matching: check for common substrings
        s1_lower.contains(&s2_lower) || s2_lower.contains(&s1_lower)
    }

    /// Optimize context for specific query type
    pub fn optimize_for_query_type(&mut self, query_type: &str) {
        match query_type {
            "implementation" => {
                self.ranking_config.relevance_weight = 0.5;
                self.ranking_config.centrality_weight = 0.3;
            }
            "debugging" => {
                self.ranking_config.relevance_weight = 0.6;
                self.ranking_config.recency_weight = 0.3;
            }
            "architecture" => {
                self.ranking_config.centrality_weight = 0.5;
                self.ranking_config.diversity_weight = 0.3;
            }
            "refactoring" => {
                self.ranking_config.relevance_weight = 0.4;
                self.ranking_config.centrality_weight = 0.4;
            }
            _ => {
                // Use default weights
            }
        }
    }

    /// Expand context with related entities
    pub fn expand_with_related(
        &self,
        core_context: &[RankedContext],
        remaining_tokens: usize,
    ) -> Vec<RankedContext> {
        let mut expanded = Vec::new();
        let mut used_tokens = 0;

        // For each core context item, find related entities
        for item in core_context {
            let node_id = format!(
                "{}::{}",
                item.entity.location().file_path.display(),
                item.entity.name()
            );
            let neighbors = self.knowledge_graph.get_neighbors(&node_id);

            for neighbor in neighbors {
                if used_tokens >= remaining_tokens {
                    break;
                }

                // Convert GraphNode to EntityDefinition
                let entity = self.graph_node_to_entity(&neighbor);
                let ranked = self.calculate_ranking_scores(&entity, &[]);

                // Only include if has reasonable score
                if ranked.rank_score > 0.3 {
                    used_tokens += ranked.estimated_tokens;
                    expanded.push(ranked);
                }
            }

            if used_tokens >= remaining_tokens {
                break;
            }
        }

        expanded
    }

    /// Convert GraphNode to EntityDefinition
    fn graph_node_to_entity(&self, node: &GraphNode) -> EntityDefinition {
        // Create a basic entity based on node type
        match node.node_type {
            super::knowledge_graph::NodeType::Function => {
                EntityDefinition::Function(FunctionDefinition {
                    name: node.name.clone(),
                    parameters: Vec::new(),
                    return_type: None,
                    visibility: Visibility::Public,
                    is_async: false,
                    is_static: false,
                    documentation: None,
                    location: node.location.clone(),
                    calls: Vec::new(),
                    called_by: Vec::new(),
                    complexity: 1,
                    lines_of_code: 0,
                })
            }
            super::knowledge_graph::NodeType::Class => EntityDefinition::Class(ClassDefinition {
                name: node.name.clone(),
                extends: None,
                implements: Vec::new(),
                properties: Vec::new(),
                methods: Vec::new(),
                visibility: Visibility::Public,
                is_abstract: false,
                is_static: false,
                documentation: None,
                location: node.location.clone(),
                design_patterns: Vec::new(),
            }),
            _ => {
                // Default to a simple function for other types
                EntityDefinition::Function(FunctionDefinition {
                    name: node.name.clone(),
                    parameters: Vec::new(),
                    return_type: None,
                    visibility: Visibility::Public,
                    is_async: false,
                    is_static: false,
                    documentation: None,
                    location: node.location.clone(),
                    calls: Vec::new(),
                    called_by: Vec::new(),
                    complexity: 1,
                    lines_of_code: 0,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_context_ranker_creation() {
        let graph = Arc::new(KnowledgeGraph::new());
        let ranker = ContextRanker::new(graph);
        assert_eq!(ranker.ranking_config.max_tokens, 8000);
    }

    #[test]
    fn test_estimate_tokens() {
        let graph = Arc::new(KnowledgeGraph::new());
        let ranker = ContextRanker::new(graph);

        let entity = EntityDefinition::Function(FunctionDefinition {
            name: "testFunction".to_string(),
            parameters: vec![],
            return_type: None,
            visibility: Visibility::Public,
            is_async: false,
            is_static: false,
            documentation: Some("Test documentation".to_string()),
            location: CodeLocation {
                file_path: PathBuf::from("test.ts"),
                start_line: 1,
                start_column: 1,
                end_line: 10,
                end_column: 1,
            },
            calls: vec![],
            called_by: vec![],
            complexity: 1,
            lines_of_code: 10,
        });

        let tokens = ranker.estimate_tokens(&entity);
        assert!(tokens > 0);
    }
}
