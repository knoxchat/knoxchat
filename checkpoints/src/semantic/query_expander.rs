//! Query Expansion and Refinement System
//!
//! This module automatically expands queries to find related context beyond exact matches,
//! using semantic understanding, synonyms, and graph traversal.

use super::knowledge_graph::{EdgeType, KnowledgeGraph};
use crate::error::Result;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Query expander for intelligent context discovery
pub struct QueryExpander {
    knowledge_graph: Arc<KnowledgeGraph>,
    expansion_config: ExpansionConfig,
    synonym_dictionary: HashMap<String, Vec<String>>,
}

/// Configuration for query expansion
#[derive(Debug, Clone)]
pub struct ExpansionConfig {
    pub max_expansion_depth: usize,
    pub max_related_entities: usize,
    pub include_synonyms: bool,
    pub include_related_types: bool,
    pub include_usage_patterns: bool,
    pub similarity_threshold: f64,
}

impl Default for ExpansionConfig {
    fn default() -> Self {
        Self {
            max_expansion_depth: 2,
            max_related_entities: 20,
            include_synonyms: true,
            include_related_types: true,
            include_usage_patterns: true,
            similarity_threshold: 0.6,
        }
    }
}

/// Expanded query with additional entities
#[derive(Debug, Clone)]
pub struct ExpandedQuery {
    pub original_query: String,
    pub original_entities: Vec<String>,
    pub expanded_entities: Vec<ExpandedEntity>,
    pub suggested_refinements: Vec<QueryRefinement>,
    pub expansion_strategy: String,
}

/// Entity discovered through expansion
#[derive(Debug, Clone)]
pub struct ExpandedEntity {
    pub name: String,
    pub expansion_reason: ExpansionReason,
    pub relevance_score: f64,
    pub relationship_path: Vec<String>,
}

/// Reason for entity expansion
#[derive(Debug, Clone, PartialEq)]
pub enum ExpansionReason {
    Synonym,
    Related,
    Dependency,
    UsagePattern,
    TypeRelationship,
    CallChain,
    InterfaceImplementation,
}

/// Query refinement suggestion
#[derive(Debug, Clone)]
pub struct QueryRefinement {
    pub refined_query: String,
    pub reasoning: String,
    pub expected_improvement: f64,
}

impl QueryExpander {
    /// Create a new query expander
    pub fn new(knowledge_graph: Arc<KnowledgeGraph>) -> Self {
        Self {
            knowledge_graph,
            expansion_config: ExpansionConfig::default(),
            synonym_dictionary: Self::build_default_synonyms(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(knowledge_graph: Arc<KnowledgeGraph>, config: ExpansionConfig) -> Self {
        Self {
            knowledge_graph,
            expansion_config: config,
            synonym_dictionary: Self::build_default_synonyms(),
        }
    }

    /// Expand a query to find related entities
    pub fn expand_query(&self, query: &str, entities: &[String]) -> Result<ExpandedQuery> {
        let mut expanded_entities = Vec::new();

        // Expand through synonyms
        if self.expansion_config.include_synonyms {
            expanded_entities.extend(self.expand_via_synonyms(entities));
        }

        // Expand through graph relationships
        expanded_entities.extend(self.expand_via_relationships(entities)?);

        // Expand through type relationships
        if self.expansion_config.include_related_types {
            expanded_entities.extend(self.expand_via_types(entities)?);
        }

        // Expand through usage patterns
        if self.expansion_config.include_usage_patterns {
            expanded_entities.extend(self.expand_via_usage_patterns(entities)?);
        }

        // Remove duplicates and sort by relevance
        expanded_entities = self.deduplicate_and_sort(expanded_entities);

        // Limit to max entities
        expanded_entities.truncate(self.expansion_config.max_related_entities);

        // Generate refinement suggestions
        let suggested_refinements = self.generate_refinements(query, entities, &expanded_entities);

        Ok(ExpandedQuery {
            original_query: query.to_string(),
            original_entities: entities.to_vec(),
            expanded_entities,
            suggested_refinements,
            expansion_strategy: self.describe_strategy(),
        })
    }

    /// Expand through synonym discovery
    fn expand_via_synonyms(&self, entities: &[String]) -> Vec<ExpandedEntity> {
        let mut expanded = Vec::new();

        for entity in entities {
            if let Some(synonyms) = self.synonym_dictionary.get(&entity.to_lowercase()) {
                for synonym in synonyms {
                    expanded.push(ExpandedEntity {
                        name: synonym.clone(),
                        expansion_reason: ExpansionReason::Synonym,
                        relevance_score: 0.9, // Synonyms are highly relevant
                        relationship_path: vec![entity.clone(), synonym.clone()],
                    });
                }
            }

            // Also find linguistic variations
            expanded.extend(self.find_linguistic_variations(entity));
        }

        expanded
    }

    /// Expand through knowledge graph relationships
    fn expand_via_relationships(&self, entities: &[String]) -> Result<Vec<ExpandedEntity>> {
        let mut expanded = Vec::new();

        for entity in entities {
            // Find nodes in the graph
            let matching_nodes = self.find_matching_nodes(entity);

            for node in matching_nodes {
                // Get neighbors up to expansion depth
                let reachable = self
                    .knowledge_graph
                    .get_reachable_nodes(&node.id, Some(self.expansion_config.max_expansion_depth));

                for related_node in reachable {
                    if related_node.id != node.id {
                        // Find the relationship type
                        let edges = self.knowledge_graph.get_outgoing_edges(&node.id);
                        let edge = edges.iter().find(|e| e.to == related_node.id);

                        let (reason, relevance) = if let Some(edge) = edge {
                            match edge.edge_type {
                                EdgeType::Calls => (ExpansionReason::CallChain, 0.8),
                                EdgeType::DependsOn => (ExpansionReason::Dependency, 0.7),
                                EdgeType::Implements => {
                                    (ExpansionReason::InterfaceImplementation, 0.85)
                                }
                                _ => (ExpansionReason::Related, 0.6),
                            }
                        } else {
                            (ExpansionReason::Related, 0.5)
                        };

                        expanded.push(ExpandedEntity {
                            name: related_node.name.clone(),
                            expansion_reason: reason,
                            relevance_score: relevance,
                            relationship_path: vec![entity.clone(), related_node.name.clone()],
                        });
                    }
                }
            }
        }

        Ok(expanded)
    }

    /// Expand through type relationships
    fn expand_via_types(&self, entities: &[String]) -> Result<Vec<ExpandedEntity>> {
        let mut expanded = Vec::new();

        for entity in entities {
            // Find class/interface nodes
            let class_nodes = self
                .knowledge_graph
                .get_nodes_by_type(super::knowledge_graph::NodeType::Class);

            for node in class_nodes {
                if node.name.contains(entity) {
                    // Find implementing classes
                    let incoming = self.knowledge_graph.get_incoming_edges(&node.id);

                    for edge in incoming {
                        if edge.edge_type == EdgeType::Implements {
                            if let Some(implementing_node) =
                                self.knowledge_graph.get_node(&edge.from)
                            {
                                expanded.push(ExpandedEntity {
                                    name: implementing_node.name.clone(),
                                    expansion_reason: ExpansionReason::TypeRelationship,
                                    relevance_score: 0.75,
                                    relationship_path: vec![
                                        entity.clone(),
                                        implementing_node.name.clone(),
                                    ],
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(expanded)
    }

    /// Expand through usage patterns
    fn expand_via_usage_patterns(&self, entities: &[String]) -> Result<Vec<ExpandedEntity>> {
        let mut expanded = Vec::new();
        let mut pattern_frequency: HashMap<String, usize> = HashMap::new();

        // Analyze co-occurrence patterns
        for entity in entities {
            let matching_nodes = self.find_matching_nodes(entity);

            for node in matching_nodes {
                // Get neighbors
                let neighbors = self.knowledge_graph.get_neighbors(&node.id);

                for neighbor in neighbors {
                    *pattern_frequency.entry(neighbor.name.clone()).or_insert(0) += 1;
                }
            }
        }

        // Entities that co-occur frequently are likely related
        for (name, frequency) in pattern_frequency {
            if frequency >= 2 && !entities.contains(&name) {
                let relevance = (frequency as f64 / entities.len() as f64).min(1.0);

                if relevance >= self.expansion_config.similarity_threshold {
                    expanded.push(ExpandedEntity {
                        name,
                        expansion_reason: ExpansionReason::UsagePattern,
                        relevance_score: relevance,
                        relationship_path: vec!["pattern".to_string()],
                    });
                }
            }
        }

        Ok(expanded)
    }

    /// Find linguistic variations of a term
    fn find_linguistic_variations(&self, entity: &str) -> Vec<ExpandedEntity> {
        let mut variations = Vec::new();

        // Plural/singular variations
        if let Some(stripped) = entity.strip_suffix('s') {
            variations.push(ExpandedEntity {
                name: stripped.to_string(),
                expansion_reason: ExpansionReason::Synonym,
                relevance_score: 0.85,
                relationship_path: vec![entity.to_string()],
            });
        } else {
            variations.push(ExpandedEntity {
                name: format!("{}s", entity),
                expansion_reason: ExpansionReason::Synonym,
                relevance_score: 0.85,
                relationship_path: vec![entity.to_string()],
            });
        }

        // Verb forms (e.g., create -> creator, created, creating)
        variations.extend(self.generate_verb_forms(entity));

        // Common prefixes/suffixes
        variations.extend(self.generate_prefix_suffix_variations(entity));

        variations
    }

    /// Generate verb form variations
    fn generate_verb_forms(&self, word: &str) -> Vec<ExpandedEntity> {
        let mut forms = Vec::new();

        if word.len() < 4 {
            return forms;
        }

        // Add common suffixes
        let suffixes = vec!["er", "ed", "ing", "tion", "tor", "able"];

        for suffix in suffixes {
            forms.push(ExpandedEntity {
                name: format!("{}{}", word, suffix),
                expansion_reason: ExpansionReason::Synonym,
                relevance_score: 0.7,
                relationship_path: vec![word.to_string()],
            });
        }

        forms
    }

    /// Generate prefix/suffix variations
    fn generate_prefix_suffix_variations(&self, word: &str) -> Vec<ExpandedEntity> {
        let mut variations = Vec::new();

        // Common prefixes
        let prefixes = vec!["get", "set", "is", "has", "can", "should"];

        for prefix in prefixes {
            variations.push(ExpandedEntity {
                name: format!("{}{}", prefix, capitalize_first(word)),
                expansion_reason: ExpansionReason::Synonym,
                relevance_score: 0.75,
                relationship_path: vec![word.to_string()],
            });
        }

        // Common suffixes
        let suffixes = vec!["Service", "Manager", "Handler", "Controller", "Repository"];

        for suffix in suffixes {
            variations.push(ExpandedEntity {
                name: format!("{}{}", capitalize_first(word), suffix),
                expansion_reason: ExpansionReason::Synonym,
                relevance_score: 0.7,
                relationship_path: vec![word.to_string()],
            });
        }

        variations
    }

    /// Find nodes matching an entity name
    fn find_matching_nodes(&self, entity: &str) -> Vec<super::knowledge_graph::GraphNode> {
        let mut matches = Vec::new();

        // Check all node types
        for node_type in [
            super::knowledge_graph::NodeType::Function,
            super::knowledge_graph::NodeType::Class,
            super::knowledge_graph::NodeType::Interface,
            super::knowledge_graph::NodeType::Type,
        ] {
            let nodes = self.knowledge_graph.get_nodes_by_type(node_type);

            for node in nodes {
                if node.name.to_lowercase().contains(&entity.to_lowercase()) {
                    matches.push(node);
                }
            }
        }

        matches
    }

    /// Remove duplicates and sort by relevance
    fn deduplicate_and_sort(&self, mut entities: Vec<ExpandedEntity>) -> Vec<ExpandedEntity> {
        let mut seen = HashSet::new();
        let mut unique = Vec::new();

        for entity in entities.drain(..) {
            if seen.insert(entity.name.clone()) {
                unique.push(entity);
            }
        }

        // Sort by relevance score (descending)
        unique.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

        unique
    }

    /// Generate query refinement suggestions
    fn generate_refinements(
        &self,
        query: &str,
        entities: &[String],
        expanded: &[ExpandedEntity],
    ) -> Vec<QueryRefinement> {
        let mut refinements = Vec::new();

        // Suggest including top expanded entities
        if !expanded.is_empty() {
            let top_expanded: Vec<_> = expanded.iter().take(3).map(|e| e.name.clone()).collect();

            refinements.push(QueryRefinement {
                refined_query: format!("{} including {}", query, top_expanded.join(", ")),
                reasoning: "Include highly related entities discovered through graph traversal"
                    .to_string(),
                expected_improvement: 0.3,
            });
        }

        // Suggest narrowing if too many entities
        if entities.len() > 5 {
            refinements.push(QueryRefinement {
                refined_query: format!("{} (focus on core entities)", query),
                reasoning: "Query has many entities, consider narrowing scope".to_string(),
                expected_improvement: 0.2,
            });
        }

        // Suggest broadening if too few results
        if entities.len() == 1 {
            refinements.push(QueryRefinement {
                refined_query: format!("{} and related functionality", query),
                reasoning: "Single entity query, consider broadening to related context"
                    .to_string(),
                expected_improvement: 0.25,
            });
        }

        refinements
    }

    /// Describe the expansion strategy used
    fn describe_strategy(&self) -> String {
        let mut strategies = Vec::new();

        if self.expansion_config.include_synonyms {
            strategies.push("synonyms");
        }
        if self.expansion_config.include_related_types {
            strategies.push("type relationships");
        }
        if self.expansion_config.include_usage_patterns {
            strategies.push("usage patterns");
        }

        format!(
            "Multi-strategy expansion using {} with depth {} and max {} entities",
            strategies.join(" + "),
            self.expansion_config.max_expansion_depth,
            self.expansion_config.max_related_entities
        )
    }

    /// Build default synonym dictionary
    fn build_default_synonyms() -> HashMap<String, Vec<String>> {
        let mut dict = HashMap::new();

        // Common programming synonyms
        dict.insert(
            "get".to_string(),
            vec![
                "fetch".to_string(),
                "retrieve".to_string(),
                "obtain".to_string(),
            ],
        );
        dict.insert(
            "set".to_string(),
            vec![
                "update".to_string(),
                "assign".to_string(),
                "put".to_string(),
            ],
        );
        dict.insert(
            "create".to_string(),
            vec![
                "make".to_string(),
                "build".to_string(),
                "initialize".to_string(),
                "new".to_string(),
            ],
        );
        dict.insert(
            "delete".to_string(),
            vec![
                "remove".to_string(),
                "destroy".to_string(),
                "clear".to_string(),
            ],
        );
        dict.insert(
            "auth".to_string(),
            vec![
                "authentication".to_string(),
                "authorize".to_string(),
                "login".to_string(),
            ],
        );
        dict.insert(
            "user".to_string(),
            vec![
                "account".to_string(),
                "profile".to_string(),
                "member".to_string(),
            ],
        );
        dict.insert(
            "data".to_string(),
            vec![
                "information".to_string(),
                "content".to_string(),
                "record".to_string(),
            ],
        );
        dict.insert(
            "save".to_string(),
            vec![
                "store".to_string(),
                "persist".to_string(),
                "write".to_string(),
            ],
        );
        dict.insert(
            "load".to_string(),
            vec!["read".to_string(), "fetch".to_string(), "get".to_string()],
        );
        dict.insert(
            "validate".to_string(),
            vec![
                "check".to_string(),
                "verify".to_string(),
                "confirm".to_string(),
            ],
        );

        dict
    }

    /// Add custom synonym
    pub fn add_synonym(&mut self, word: String, synonyms: Vec<String>) {
        self.synonym_dictionary.insert(word, synonyms);
    }
}

// Helper functions

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capitalize_first() {
        assert_eq!(capitalize_first("hello"), "Hello");
        assert_eq!(capitalize_first("World"), "World");
        assert_eq!(capitalize_first(""), "");
    }

    #[test]
    fn test_default_synonyms() {
        let synonyms = QueryExpander::build_default_synonyms();
        assert!(synonyms.contains_key("get"));
        assert!(synonyms.contains_key("create"));
    }

    #[test]
    fn test_expander_creation() {
        let graph = Arc::new(KnowledgeGraph::new());
        let expander = QueryExpander::new(graph);

        assert_eq!(expander.expansion_config.max_expansion_depth, 2);
    }
}
