//! Relevance Engine
//!
//! Scores and ranks context elements by relevance to queries

use super::*;
use crate::ai_context::query_analyzer::EntityType;
use crate::semantic::{AIContextCheckpoint, SemanticContext};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Relevance scoring engine
pub struct RelevanceScorer {
    /// Weights for different scoring factors
    weights: RelevanceWeights,
    /// Semantic similarity calculator
    similarity_calculator: SemanticSimilarityCalculator,
}

/// Weights for relevance scoring components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelevanceWeights {
    pub semantic: f64,
    pub temporal: f64,
    pub architectural: f64,
    pub dependency: f64,
    pub usage: f64,
}

impl Default for RelevanceWeights {
    fn default() -> Self {
        Self {
            semantic: 0.4,
            temporal: 0.2,
            architectural: 0.2,
            dependency: 0.1,
            usage: 0.1,
        }
    }
}

/// Detailed relevance score breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelevanceScore {
    /// Semantic similarity score
    pub semantic: f64,
    /// Temporal relevance score
    pub temporal: f64,
    /// Architectural relevance score
    pub architectural: f64,
    /// Dependency relevance score
    pub dependency: f64,
    /// Usage pattern relevance score
    pub usage: f64,
    /// Composite weighted score
    pub composite: f64,
    /// Confidence in the scoring
    pub confidence: f64,
    /// Human-readable reasoning
    pub reasoning: String,
}

/// Checkpoint with its relevance score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredCheckpoint {
    /// The checkpoint
    pub checkpoint: AIContextCheckpoint,
    /// Relevance score
    pub score: RelevanceScore,
}

/// Semantic similarity calculator
struct SemanticSimilarityCalculator {
    /// TF-IDF vectors cache
    tfidf_cache: std::cell::RefCell<HashMap<String, Vec<f64>>>,
}

impl RelevanceScorer {
    /// Create a new relevance scorer
    pub fn new() -> Result<Self> {
        Ok(Self {
            weights: RelevanceWeights::default(),
            similarity_calculator: SemanticSimilarityCalculator::new(),
        })
    }

    /// Score checkpoint relevance to a query intent
    pub async fn score_checkpoint_relevance(
        &self,
        checkpoint: &AIContextCheckpoint,
        query_intent: &QueryIntent,
    ) -> Result<RelevanceScore> {
        // Calculate individual scores
        let semantic_score = self
            .calculate_semantic_score(checkpoint, query_intent)
            .await?;
        let temporal_score = self
            .calculate_temporal_score(checkpoint, query_intent)
            .await?;
        let architectural_score = self
            .calculate_architectural_score(checkpoint, query_intent)
            .await?;
        let dependency_score = self
            .calculate_dependency_score(checkpoint, query_intent)
            .await?;
        let usage_score = self.calculate_usage_score(checkpoint, query_intent).await?;

        // Calculate composite score
        let composite = self.weights.semantic * semantic_score
            + self.weights.temporal * temporal_score
            + self.weights.architectural * architectural_score
            + self.weights.dependency * dependency_score
            + self.weights.usage * usage_score;

        // Calculate confidence based on available data
        let confidence = self.calculate_scoring_confidence(checkpoint, query_intent);

        // Generate reasoning
        let reasoning = self.generate_reasoning(
            semantic_score,
            temporal_score,
            architectural_score,
            dependency_score,
            usage_score,
            composite,
        );

        Ok(RelevanceScore {
            semantic: semantic_score,
            temporal: temporal_score,
            architectural: architectural_score,
            dependency: dependency_score,
            usage: usage_score,
            composite,
            confidence,
            reasoning,
        })
    }

    /// Score semantic context relevance to query intent
    pub async fn score_semantic_relevance(
        &self,
        semantic_context: &SemanticContext,
        query_intent: &QueryIntent,
    ) -> Result<f64> {
        let mut relevance_score = 0.0;
        let mut match_count = 0;

        // Score entity matches
        for entity in &query_intent.entities {
            match entity.entity_type {
                EntityType::Function => {
                    if semantic_context.functions.contains_key(&entity.name) {
                        relevance_score += entity.confidence;
                        match_count += 1;
                    }
                }
                EntityType::Class => {
                    if semantic_context.classes.contains_key(&entity.name) {
                        relevance_score += entity.confidence;
                        match_count += 1;
                    }
                }
                EntityType::Interface => {
                    if semantic_context.interfaces.contains_key(&entity.name) {
                        relevance_score += entity.confidence;
                        match_count += 1;
                    }
                }
                EntityType::Type => {
                    if semantic_context.types.contains_key(&entity.name) {
                        relevance_score += entity.confidence;
                        match_count += 1;
                    }
                }
                _ => {} // Handle other types as needed
            }
        }

        // Normalize by number of matches or total entities
        if query_intent.entities.is_empty() {
            Ok(0.5) // Neutral score when no entities
        } else if match_count > 0 {
            // Weight by match ratio - higher score for more matches
            let match_ratio = match_count as f64 / query_intent.entities.len() as f64;
            Ok((relevance_score / query_intent.entities.len() as f64) * (1.0 + match_ratio))
        } else {
            Ok(0.0) // No matches found
        }
    }

    /// Calculate semantic similarity score
    async fn calculate_semantic_score(
        &self,
        checkpoint: &AIContextCheckpoint,
        query_intent: &QueryIntent,
    ) -> Result<f64> {
        // Entity matching score
        let entity_score = self.score_entity_matches(&checkpoint.semantic_context, query_intent)?;

        // Text similarity score using TF-IDF
        let text_similarity = self.similarity_calculator.calculate_text_similarity(
            &query_intent.original_query,
            &checkpoint.base_checkpoint.description,
        )?;

        // Combine scores
        Ok((entity_score * 0.7) + (text_similarity * 0.3))
    }

    /// Calculate temporal relevance score
    async fn calculate_temporal_score(
        &self,
        checkpoint: &AIContextCheckpoint,
        _query_intent: &QueryIntent,
    ) -> Result<f64> {
        let now = chrono::Utc::now();
        let checkpoint_age = now.signed_duration_since(checkpoint.base_checkpoint.created_at);

        // Score based on recency - newer checkpoints get higher scores
        let days_old = checkpoint_age.num_days() as f64;

        // Exponential decay function
        let score = (-days_old / 30.0).exp(); // Half-life of 30 days

        Ok(score.max(0.0).min(1.0))
    }

    /// Calculate architectural relevance score
    async fn calculate_architectural_score(
        &self,
        checkpoint: &AIContextCheckpoint,
        query_intent: &QueryIntent,
    ) -> Result<f64> {
        let mut score: f64 = 0.0;

        // Check if query is architecture-related
        if query_intent.query_type == QueryType::Architecture {
            score += 0.5;
        }

        // Check architectural impact significance
        match checkpoint.architectural_impact.significance {
            crate::semantic::types::ArchitecturalSignificance::High => score += 0.3,
            crate::semantic::types::ArchitecturalSignificance::Critical => score += 0.4,
            crate::semantic::types::ArchitecturalSignificance::Medium => score += 0.2,
            crate::semantic::types::ArchitecturalSignificance::Low => score += 0.1,
        }

        // Check for architectural patterns in query
        let query_lower = query_intent.original_query.to_lowercase();
        let architectural_keywords = ["pattern", "architecture", "design", "structure"];
        for keyword in &architectural_keywords {
            if query_lower.contains(keyword) {
                score += 0.1;
            }
        }

        Ok(score.min(1.0))
    }

    /// Calculate dependency relevance score
    async fn calculate_dependency_score(
        &self,
        checkpoint: &AIContextCheckpoint,
        query_intent: &QueryIntent,
    ) -> Result<f64> {
        let mut score: f64 = 0.0;

        // Check if any query entities are in the dependency graph
        for entity in &query_intent.entities {
            // Check if entity appears in dependency relationships
            let entity_name = &entity.name;

            // Check direct dependencies
            for dep in &checkpoint.code_relationships.direct_dependencies {
                if dep.to_lowercase().contains(&entity_name.to_lowercase()) {
                    score += entity.confidence * 0.3;
                    break;
                }
            }

            // Check dependents (reverse dependencies)
            for dep in &checkpoint.code_relationships.dependents {
                if dep.to_lowercase().contains(&entity_name.to_lowercase()) {
                    score += entity.confidence * 0.2;
                    break;
                }
            }
        }

        // Boost score for queries about dependencies
        let query_lower = query_intent.original_query.to_lowercase();
        if query_lower.contains("depend")
            || query_lower.contains("import")
            || query_lower.contains("require")
        {
            score += 0.3;
        }

        Ok(score.min(1.0))
    }

    /// Calculate usage pattern relevance score
    async fn calculate_usage_score(
        &self,
        checkpoint: &AIContextCheckpoint,
        query_intent: &QueryIntent,
    ) -> Result<f64> {
        let mut score: f64 = 0.0;

        // Check for usage-related queries
        let query_lower = query_intent.original_query.to_lowercase();
        let usage_keywords = ["how to use", "example", "usage", "call", "invoke"];
        for keyword in &usage_keywords {
            if query_lower.contains(keyword) {
                score += 0.2;
                break;
            }
        }

        // Check if checkpoint has usage patterns
        if !checkpoint.semantic_context.usage_patterns.is_empty() {
            score += 0.3;
        }

        // Check for call chains
        if !checkpoint.semantic_context.call_chains.is_empty() {
            score += 0.2;
        }

        Ok(score.min(1.0))
    }

    /// Score entity matches between checkpoint and query
    fn score_entity_matches(
        &self,
        semantic_context: &SemanticContext,
        query_intent: &QueryIntent,
    ) -> Result<f64> {
        let mut total_score = 0.0;
        let mut match_count = 0;

        for entity in &query_intent.entities {
            let entity_exists = match entity.entity_type {
                EntityType::Function => semantic_context.functions.contains_key(&entity.name),
                EntityType::Class => semantic_context.classes.contains_key(&entity.name),
                EntityType::Interface => semantic_context.interfaces.contains_key(&entity.name),
                EntityType::Type => semantic_context.types.contains_key(&entity.name),
                EntityType::Constant => semantic_context.constants.contains_key(&entity.name),
                _ => false,
            };

            if entity_exists {
                total_score += entity.confidence;
                match_count += 1;
            }
        }

        if query_intent.entities.is_empty() {
            Ok(0.5) // Neutral score for queries without specific entities
        } else if match_count > 0 {
            // Weight by match ratio and average confidence
            let match_ratio = match_count as f64 / query_intent.entities.len() as f64;
            let avg_score = total_score / query_intent.entities.len() as f64;
            Ok(avg_score * (1.0 + match_ratio))
        } else {
            Ok(0.0) // No entity matches found
        }
    }

    /// Calculate confidence in the scoring
    fn calculate_scoring_confidence(
        &self,
        checkpoint: &AIContextCheckpoint,
        query_intent: &QueryIntent,
    ) -> f64 {
        let mut confidence = 0.5; // Base confidence

        // Boost confidence based on query intent confidence
        confidence += query_intent.confidence * 0.2;

        // Boost confidence based on checkpoint confidence
        confidence += checkpoint.confidence_score * 0.2;

        // Boost confidence if we have specific entities to match
        if !query_intent.entities.is_empty() {
            confidence += 0.1;
        }

        // Boost confidence if checkpoint has rich semantic context
        if !checkpoint.semantic_context.functions.is_empty()
            || !checkpoint.semantic_context.classes.is_empty()
        {
            confidence += 0.1;
        }

        confidence.min(1.0)
    }

    /// Generate human-readable reasoning for the score
    fn generate_reasoning(
        &self,
        semantic: f64,
        temporal: f64,
        architectural: f64,
        dependency: f64,
        usage: f64,
        composite: f64,
    ) -> String {
        let mut reasons = Vec::new();

        if semantic > 0.7 {
            reasons.push("High semantic similarity");
        } else if semantic > 0.4 {
            reasons.push("Moderate semantic similarity");
        }

        if temporal > 0.8 {
            reasons.push("Recent checkpoint");
        } else if temporal < 0.2 {
            reasons.push("Older checkpoint");
        }

        if architectural > 0.5 {
            reasons.push("Architecturally relevant");
        }

        if dependency > 0.5 {
            reasons.push("Has dependency relationships");
        }

        if usage > 0.5 {
            reasons.push("Contains usage patterns");
        }

        let score_description = if composite > 0.8 {
            "Very high relevance"
        } else if composite > 0.6 {
            "High relevance"
        } else if composite > 0.4 {
            "Moderate relevance"
        } else if composite > 0.2 {
            "Low relevance"
        } else {
            "Very low relevance"
        };

        if reasons.is_empty() {
            score_description.to_string()
        } else {
            format!("{}: {}", score_description, reasons.join(", "))
        }
    }
}

impl SemanticSimilarityCalculator {
    fn new() -> Self {
        Self {
            tfidf_cache: std::cell::RefCell::new(HashMap::new()),
        }
    }

    /// Calculate text similarity using TF-IDF
    fn calculate_text_similarity(&self, text1: &str, text2: &str) -> Result<f64> {
        // Check cache first
        let cache_key1 = format!("tfidf_{}", text1);
        let cache_key2 = format!("tfidf_{}", text2);

        let vec1 = if let Some(cached) = self.tfidf_cache.borrow().get(&cache_key1) {
            cached.clone()
        } else {
            let vec = self.calculate_tfidf_vector(text1);
            self.tfidf_cache
                .borrow_mut()
                .insert(cache_key1, vec.clone());
            vec
        };

        let vec2 = if let Some(cached) = self.tfidf_cache.borrow().get(&cache_key2) {
            cached.clone()
        } else {
            let vec = self.calculate_tfidf_vector(text2);
            self.tfidf_cache
                .borrow_mut()
                .insert(cache_key2, vec.clone());
            vec
        };

        // Calculate cosine similarity between TF-IDF vectors
        Ok(self.cosine_similarity(&vec1, &vec2))
    }

    /// Calculate TF-IDF vector for text
    fn calculate_tfidf_vector(&self, text: &str) -> Vec<f64> {
        // Simplified TF-IDF - in production would use proper IDF calculation
        let words = self.tokenize(text);
        let mut term_freq = std::collections::HashMap::new();

        // Calculate term frequencies
        for word in &words {
            *term_freq.entry(word.clone()).or_insert(0.0) += 1.0;
        }

        // Normalize by document length
        let doc_length = words.len() as f64;
        let mut tfidf_vec = Vec::new();

        // Create a simple vector representation (first 100 most common words)
        for word in &words {
            let tf = term_freq.get(word).unwrap_or(&0.0) / doc_length;
            let idf = 1.0; // Simplified - would calculate proper IDF in production
            tfidf_vec.push(tf * idf);
        }

        // Pad or truncate to fixed size for consistency
        tfidf_vec.resize(100, 0.0);
        tfidf_vec
    }

    /// Calculate cosine similarity between two vectors
    fn cosine_similarity(&self, vec1: &[f64], vec2: &[f64]) -> f64 {
        let dot_product: f64 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();
        let norm1: f64 = vec1.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm2: f64 = vec2.iter().map(|x| x * x).sum::<f64>().sqrt();

        if norm1 == 0.0 || norm2 == 0.0 {
            0.0
        } else {
            dot_product / (norm1 * norm2)
        }
    }

    /// Simple tokenization
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .filter(|word| word.len() > 2) // Filter out short words
            .map(|word| {
                word.trim_matches(|c: char| !c.is_alphanumeric())
                    .to_string()
            })
            .filter(|word| !word.is_empty())
            .collect()
    }
}
