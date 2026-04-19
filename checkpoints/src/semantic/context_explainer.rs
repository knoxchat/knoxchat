//! Context Explanation System
//!
//! This module provides detailed explanations for why specific context was included.

use super::context_ranker::RankedContext;
use super::knowledge_graph::KnowledgeGraph;
use crate::error::Result;
use std::collections::HashSet;
use std::sync::Arc;

/// Context explainer that generates detailed reasoning
pub struct ContextExplainer {
    knowledge_graph: Arc<KnowledgeGraph>,
    explanation_config: ExplanationConfig,
}

/// Configuration for explanations
#[derive(Debug, Clone)]
pub struct ExplanationConfig {
    pub include_score_breakdown: bool,
    pub include_relationship_chain: bool,
    pub include_temporal_context: bool,
    pub max_chain_length: usize,
    pub verbosity: ExplanationVerbosity,
}

/// Verbosity level for explanations
#[derive(Debug, Clone, PartialEq)]
pub enum ExplanationVerbosity {
    Minimal,  // Just the reason
    Standard, // Reason + key factors
    Detailed, // Full breakdown
    Debug,    // Everything including internal scores
}

impl Default for ExplanationConfig {
    fn default() -> Self {
        Self {
            include_score_breakdown: true,
            include_relationship_chain: true,
            include_temporal_context: true,
            max_chain_length: 5,
            verbosity: ExplanationVerbosity::Standard,
        }
    }
}

/// Complete explanation for a context item
#[derive(Debug, Clone)]
pub struct ContextExplanation {
    pub entity_name: String,
    pub primary_reason: String,
    pub score_breakdown: ScoreBreakdown,
    pub relationship_chain: Vec<RelationshipStep>,
    pub temporal_factors: Vec<TemporalFactor>,
    pub alternatives_considered: Vec<AlternativeContext>,
    pub confidence: f64,
}

/// Breakdown of scoring factors
#[derive(Debug, Clone)]
pub struct ScoreBreakdown {
    pub total_score: f64,
    pub relevance_contribution: f64,
    pub importance_contribution: f64,
    pub centrality_contribution: f64,
    pub recency_contribution: f64,
    pub diversity_contribution: f64,
    pub factors_explanation: Vec<(String, f64, String)>,
}

/// Step in a relationship chain
#[derive(Debug, Clone)]
pub struct RelationshipStep {
    pub from_entity: String,
    pub to_entity: String,
    pub relationship_type: String,
    pub strength: f64,
    pub reasoning: String,
}

/// Temporal factor affecting inclusion
#[derive(Debug, Clone)]
pub struct TemporalFactor {
    pub factor_type: TemporalFactorType,
    pub description: String,
    pub impact: f64,
    pub evidence: Vec<String>,
}

/// Type of temporal factor
#[derive(Debug, Clone, PartialEq)]
pub enum TemporalFactorType {
    RecentChange,
    FrequentModification,
    PatternEvolution,
    ArchitecturalShift,
    StabilityIndicator,
}

/// Alternative context that was considered but not included
#[derive(Debug, Clone)]
pub struct AlternativeContext {
    pub entity_name: String,
    pub score: f64,
    pub exclusion_reason: String,
}

/// Explanation for query results
#[derive(Debug, Clone)]
pub struct QueryExplanation {
    pub query: String,
    pub query_interpretation: QueryInterpretation,
    pub context_selection_strategy: String,
    pub included_explanations: Vec<ContextExplanation>,
    pub exclusion_summary: ExclusionSummary,
    pub coverage_analysis: CoverageAnalysis,
    pub recommendations: Vec<String>,
}

/// How the query was interpreted
#[derive(Debug, Clone)]
pub struct QueryInterpretation {
    pub detected_entities: Vec<String>,
    pub inferred_intent: String,
    pub scope: String,
    pub confidence: f64,
}

/// Summary of excluded context
#[derive(Debug, Clone)]
pub struct ExclusionSummary {
    pub total_excluded: usize,
    pub excluded_by_score: usize,
    pub excluded_by_token_limit: usize,
    pub excluded_by_diversity: usize,
    pub top_excluded: Vec<AlternativeContext>,
}

/// Analysis of context coverage
#[derive(Debug, Clone)]
pub struct CoverageAnalysis {
    pub query_entities_covered: usize,
    pub query_entities_total: usize,
    pub files_covered: usize,
    pub architectural_layers_covered: Vec<String>,
    pub gaps: Vec<String>,
}

impl ContextExplainer {
    /// Create a new context explainer
    pub fn new(knowledge_graph: Arc<KnowledgeGraph>) -> Self {
        Self {
            knowledge_graph,
            explanation_config: ExplanationConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(knowledge_graph: Arc<KnowledgeGraph>, config: ExplanationConfig) -> Self {
        Self {
            knowledge_graph,
            explanation_config: config,
        }
    }

    /// Explain why a specific context item was included
    pub fn explain_context_item(
        &self,
        ranked_context: &RankedContext,
        query_entities: &[String],
    ) -> Result<ContextExplanation> {
        let entity_name = ranked_context.entity.name().to_string();

        // Generate primary reason
        let primary_reason = self.generate_primary_reason(ranked_context, query_entities);

        // Build score breakdown
        let score_breakdown = self.build_score_breakdown(ranked_context);

        // Find relationship chain to query entities
        let relationship_chain = self.find_relationship_chain(&entity_name, query_entities)?;

        // Identify temporal factors
        let temporal_factors = self.identify_temporal_factors(ranked_context);

        // Find alternatives that were considered
        let alternatives_considered = Vec::new(); // Would be populated from ranking process

        Ok(ContextExplanation {
            entity_name,
            primary_reason,
            score_breakdown,
            relationship_chain,
            temporal_factors,
            alternatives_considered,
            confidence: ranked_context.rank_score,
        })
    }

    /// Explain the entire query result
    pub fn explain_query_result(
        &self,
        query: &str,
        query_entities: &[String],
        included_items: &[RankedContext],
        excluded_items: &[RankedContext],
    ) -> Result<QueryExplanation> {
        // Interpret the query
        let query_interpretation = self.interpret_query(query, query_entities);

        // Determine selection strategy
        let context_selection_strategy = self.determine_selection_strategy(included_items);

        // Explain each included item
        let included_explanations: Vec<_> = included_items
            .iter()
            .map(|item| self.explain_context_item(item, query_entities))
            .collect::<Result<Vec<_>>>()?;

        // Build exclusion summary
        let exclusion_summary = self.build_exclusion_summary(excluded_items);

        // Analyze coverage
        let coverage_analysis = self.analyze_coverage(query_entities, included_items);

        // Generate recommendations
        let recommendations = self.generate_recommendations(&coverage_analysis, &exclusion_summary);

        Ok(QueryExplanation {
            query: query.to_string(),
            query_interpretation,
            context_selection_strategy,
            included_explanations,
            exclusion_summary,
            coverage_analysis,
            recommendations,
        })
    }

    /// Generate formatted explanation text
    pub fn format_explanation(&self, explanation: &ContextExplanation) -> String {
        let mut output = String::new();

        match self.explanation_config.verbosity {
            ExplanationVerbosity::Minimal => {
                output.push_str(&format!(
                    "✓ {}: {}\n",
                    explanation.entity_name, explanation.primary_reason
                ));
            }
            ExplanationVerbosity::Standard => {
                output.push_str(&format!("✓ {}\n", explanation.entity_name));
                output.push_str(&format!("  Reason: {}\n", explanation.primary_reason));
                output.push_str(&format!(
                    "  Score: {:.2}\n",
                    explanation.score_breakdown.total_score
                ));

                if !explanation.relationship_chain.is_empty() {
                    output.push_str("  Connections:\n");
                    for step in explanation.relationship_chain.iter().take(3) {
                        output.push_str(&format!(
                            "    • {} → {} ({})\n",
                            step.from_entity, step.to_entity, step.relationship_type
                        ));
                    }
                }
            }
            ExplanationVerbosity::Detailed => {
                output.push_str(&format!(
                    "✓ {} (confidence: {:.2})\n",
                    explanation.entity_name, explanation.confidence
                ));
                output.push_str(&format!(
                    "\n  Primary Reason:\n    {}\n",
                    explanation.primary_reason
                ));

                output.push_str(&format!(
                    "\n  Score Breakdown (total: {:.2}):\n",
                    explanation.score_breakdown.total_score
                ));
                for (factor, score, desc) in &explanation.score_breakdown.factors_explanation {
                    output.push_str(&format!("    • {}: {:.2} - {}\n", factor, score, desc));
                }

                if !explanation.relationship_chain.is_empty() {
                    output.push_str("\n  Relationship Chain:\n");
                    for step in &explanation.relationship_chain {
                        output.push_str(&format!(
                            "    {} → {} [{}] (strength: {:.2})\n      {}\n",
                            step.from_entity,
                            step.to_entity,
                            step.relationship_type,
                            step.strength,
                            step.reasoning
                        ));
                    }
                }

                if !explanation.temporal_factors.is_empty() {
                    output.push_str("\n  Temporal Factors:\n");
                    for factor in &explanation.temporal_factors {
                        output.push_str(&format!(
                            "    • {:?}: {} (impact: {:.2})\n",
                            factor.factor_type, factor.description, factor.impact
                        ));
                    }
                }
            }
            ExplanationVerbosity::Debug => {
                output.push_str(&format!("{:#?}\n", explanation));
            }
        }

        output
    }

    /// Format query explanation
    pub fn format_query_explanation(&self, explanation: &QueryExplanation) -> String {
        let mut output = String::new();

        output.push_str("╔══════════════════════════════════════════════════════════════╗\n");
        output.push_str(&format!(
            "║ Query Explanation: {:<44} ║\n",
            truncate(&explanation.query, 44)
        ));
        output.push_str("╚══════════════════════════════════════════════════════════════╝\n\n");

        // Query interpretation
        output.push_str("📊 Query Interpretation:\n");
        output.push_str(&format!(
            "  Intent: {}\n",
            explanation.query_interpretation.inferred_intent
        ));
        output.push_str(&format!(
            "  Scope: {}\n",
            explanation.query_interpretation.scope
        ));
        output.push_str(&format!(
            "  Entities: {}\n",
            explanation
                .query_interpretation
                .detected_entities
                .join(", ")
        ));
        output.push_str(&format!(
            "  Confidence: {:.2}\n\n",
            explanation.query_interpretation.confidence
        ));

        // Selection strategy
        output.push_str(&format!(
            "🎯 Selection Strategy:\n  {}\n\n",
            explanation.context_selection_strategy
        ));

        // Included items
        output.push_str(&format!(
            "✅ Included Context ({} items):\n\n",
            explanation.included_explanations.len()
        ));
        for (i, item_explanation) in explanation.included_explanations.iter().enumerate() {
            output.push_str(&format!(
                "{}. {}",
                i + 1,
                self.format_explanation(item_explanation)
            ));
            output.push('\n');
        }

        // Coverage analysis
        output.push_str("📈 Coverage Analysis:\n");
        output.push_str(&format!(
            "  Entities covered: {}/{}\n",
            explanation.coverage_analysis.query_entities_covered,
            explanation.coverage_analysis.query_entities_total
        ));
        output.push_str(&format!(
            "  Files included: {}\n",
            explanation.coverage_analysis.files_covered
        ));
        output.push_str(&format!(
            "  Layers covered: {}\n",
            explanation
                .coverage_analysis
                .architectural_layers_covered
                .join(", ")
        ));

        if !explanation.coverage_analysis.gaps.is_empty() {
            output.push_str(&format!(
                "  Gaps: {}\n",
                explanation.coverage_analysis.gaps.join(", ")
            ));
        }

        // Exclusion summary
        output.push_str("\n❌ Exclusion Summary:\n");
        output.push_str(&format!(
            "  Total excluded: {}\n",
            explanation.exclusion_summary.total_excluded
        ));
        output.push_str(&format!(
            "  By score: {}\n",
            explanation.exclusion_summary.excluded_by_score
        ));
        output.push_str(&format!(
            "  By token limit: {}\n",
            explanation.exclusion_summary.excluded_by_token_limit
        ));

        // Recommendations
        if !explanation.recommendations.is_empty() {
            output.push_str("\n💡 Recommendations:\n");
            for rec in &explanation.recommendations {
                output.push_str(&format!("  • {}\n", rec));
            }
        }

        output
    }

    // Private helper methods

    fn generate_primary_reason(
        &self,
        ranked_context: &RankedContext,
        query_entities: &[String],
    ) -> String {
        let mut reasons = Vec::new();

        if ranked_context.relevance_score > 0.7 {
            let matching_entities: Vec<_> = query_entities
                .iter()
                .filter(|e| ranked_context.entity.name().contains(*e))
                .collect();

            if !matching_entities.is_empty() {
                reasons.push(format!(
                    "Directly matches query entities: {}",
                    matching_entities
                        .iter()
                        .map(|e| format!("'{}'", e))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            } else {
                reasons.push("High semantic relevance to query".to_string());
            }
        }

        if ranked_context.importance_score > 0.7 {
            if ranked_context.entity.visibility() == "public" {
                reasons.push("Public API with high importance".to_string());
            } else {
                reasons.push("Critical component in codebase".to_string());
            }
        }

        if ranked_context.centrality_score > 0.7 {
            reasons.push("Central node in code dependency graph".to_string());
        }

        if reasons.is_empty() {
            reasons.push("Included for contextual completeness".to_string());
        }

        reasons.join("; ")
    }

    fn build_score_breakdown(&self, ranked_context: &RankedContext) -> ScoreBreakdown {
        let factors = vec![
            (
                "Relevance".to_string(),
                ranked_context.relevance_score,
                "Match to query entities and intent".to_string(),
            ),
            (
                "Importance".to_string(),
                ranked_context.importance_score,
                "Visibility, documentation, and structural importance".to_string(),
            ),
            (
                "Centrality".to_string(),
                ranked_context.centrality_score,
                "Position in dependency graph".to_string(),
            ),
            (
                "Recency".to_string(),
                ranked_context.recency_score,
                "Recent modifications and activity".to_string(),
            ),
            (
                "Diversity".to_string(),
                ranked_context.diversity_score,
                "Contribution to overall context diversity".to_string(),
            ),
        ];

        ScoreBreakdown {
            total_score: ranked_context.rank_score,
            relevance_contribution: ranked_context.relevance_score * 0.4,
            importance_contribution: ranked_context.importance_score * 0.3,
            centrality_contribution: ranked_context.centrality_score * 0.2,
            recency_contribution: ranked_context.recency_score * 0.1,
            diversity_contribution: ranked_context.diversity_score * 0.1,
            factors_explanation: factors,
        }
    }

    fn find_relationship_chain(
        &self,
        entity_name: &str,
        query_entities: &[String],
    ) -> Result<Vec<RelationshipStep>> {
        let mut chain = Vec::new();

        for query_entity in query_entities {
            // Find path in knowledge graph
            let path = self
                .knowledge_graph
                .find_shortest_path(entity_name, query_entity);

            if let Some(nodes) = path {
                for i in 0..nodes.len().saturating_sub(1) {
                    let from = &nodes[i];
                    let to = &nodes[i + 1];

                    // Find edge between these nodes
                    let edges = self.knowledge_graph.get_outgoing_edges(&from.id);
                    if let Some(edge) = edges.iter().find(|e| e.to == to.id) {
                        chain.push(RelationshipStep {
                            from_entity: from.name.clone(),
                            to_entity: to.name.clone(),
                            relationship_type: format!("{:?}", edge.edge_type),
                            strength: edge.weight,
                            reasoning: format!(
                                "{} is connected to {} via {}",
                                from.name,
                                to.name,
                                format!("{:?}", edge.edge_type).to_lowercase()
                            ),
                        });
                    }
                }

                if chain.len() >= self.explanation_config.max_chain_length {
                    break;
                }
            }
        }

        Ok(chain)
    }

    fn identify_temporal_factors(&self, _ranked_context: &RankedContext) -> Vec<TemporalFactor> {
        // Would analyze temporal data to identify factors
        Vec::new()
    }

    fn interpret_query(&self, query: &str, query_entities: &[String]) -> QueryInterpretation {
        let intent = if query.contains("how") || query.contains("what") {
            "Explanation"
        } else if query.contains("implement") || query.contains("create") {
            "Implementation"
        } else if query.contains("bug") || query.contains("fix") {
            "Debugging"
        } else {
            "General"
        };

        QueryInterpretation {
            detected_entities: query_entities.to_vec(),
            inferred_intent: intent.to_string(),
            scope: "Module".to_string(),
            confidence: 0.8,
        }
    }

    fn determine_selection_strategy(&self, included_items: &[RankedContext]) -> String {
        let avg_score =
            included_items.iter().map(|i| i.rank_score).sum::<f64>() / included_items.len() as f64;

        format!(
            "Greedy selection with diversity constraints (avg score: {:.2})",
            avg_score
        )
    }

    fn build_exclusion_summary(&self, excluded_items: &[RankedContext]) -> ExclusionSummary {
        let by_score = excluded_items.iter().filter(|i| i.rank_score < 0.3).count();
        let by_token = excluded_items.len() - by_score;

        let top_excluded: Vec<_> = excluded_items
            .iter()
            .take(5)
            .map(|item| AlternativeContext {
                entity_name: item.entity.name().to_string(),
                score: item.rank_score,
                exclusion_reason: if item.rank_score < 0.3 {
                    "Score below threshold".to_string()
                } else {
                    "Token budget limit".to_string()
                },
            })
            .collect();

        ExclusionSummary {
            total_excluded: excluded_items.len(),
            excluded_by_score: by_score,
            excluded_by_token_limit: by_token,
            excluded_by_diversity: 0,
            top_excluded,
        }
    }

    fn analyze_coverage(
        &self,
        query_entities: &[String],
        included_items: &[RankedContext],
    ) -> CoverageAnalysis {
        let mut covered = 0;
        let mut files = HashSet::new();

        for item in included_items {
            for query_entity in query_entities {
                if item.entity.name().contains(query_entity) {
                    covered += 1;
                    break;
                }
            }
            files.insert(item.entity.location().file_path.clone());
        }

        CoverageAnalysis {
            query_entities_covered: covered,
            query_entities_total: query_entities.len(),
            files_covered: files.len(),
            architectural_layers_covered: vec!["Business Logic".to_string()],
            gaps: Vec::new(),
        }
    }

    fn generate_recommendations(
        &self,
        coverage: &CoverageAnalysis,
        exclusion: &ExclusionSummary,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if coverage.query_entities_covered < coverage.query_entities_total {
            recommendations.push(format!(
                "Only {}/{} query entities covered. Consider refining query or increasing token budget.",
                coverage.query_entities_covered, coverage.query_entities_total
            ));
        }

        if exclusion.excluded_by_token_limit > 0 {
            recommendations.push(format!(
                "{} relevant items excluded due to token limit. Consider increasing max_tokens.",
                exclusion.excluded_by_token_limit
            ));
        }

        if coverage.files_covered < 3 {
            recommendations.push("Limited file coverage. Results may be incomplete.".to_string());
        }

        recommendations
    }
}

// Helper functions

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explainer_creation() {
        let graph = Arc::new(KnowledgeGraph::new());
        let explainer = ContextExplainer::new(graph);
        assert_eq!(
            explainer.explanation_config.verbosity,
            ExplanationVerbosity::Standard
        );
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("Hello", 10), "Hello");
        assert_eq!(truncate("Hello World!", 8), "Hello...");
    }
}
