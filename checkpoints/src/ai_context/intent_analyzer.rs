//! Intent Analysis Module
//!
//! Analyzes code change intent and patterns for better context understanding

use super::*;
use crate::semantic::{IntentAnalysis, SemanticContext};
use crate::types::FileChange;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Intent analyzer for code changes
pub struct IntentAnalyzer {
    /// Pattern matchers for different change intents
    intent_patterns: HashMap<ChangeIntent, Vec<String>>,
    /// Refactoring pattern detectors
    refactoring_detectors: Vec<RefactoringDetector>,
}

/// Types of change intents
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ChangeIntent {
    FeatureAddition,
    BugFix,
    Refactoring,
    Performance,
    Optimization,
    Security,
    SecurityEnhancement,
    Documentation,
    Testing,
    Maintenance,
    Configuration,
    Migration,
    Unknown,
}

/// Intent pattern for matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentPattern {
    /// Intent type
    pub intent: ChangeIntent,
    /// Pattern to match
    pub pattern: String,
    /// Weight of this pattern
    pub weight: f64,
    /// Context where this pattern applies
    pub context: PatternContext,
}

/// Context where pattern applies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternContext {
    CommitMessage,
    FileContent,
    FileName,
    FileExtension,
    Any,
}

/// Refactoring detector
#[derive(Debug, Clone)]
struct RefactoringDetector {
    /// Type of refactoring
    refactoring_type: String,
    /// Detection patterns
    patterns: Vec<String>,
    /// Minimum confidence threshold
    threshold: f64,
}

impl IntentAnalyzer {
    /// Create a new intent analyzer
    pub fn new() -> Result<Self> {
        let mut analyzer = Self {
            intent_patterns: HashMap::new(),
            refactoring_detectors: Vec::new(),
        };

        analyzer.initialize_patterns();
        analyzer.initialize_refactoring_detectors();

        Ok(analyzer)
    }

    /// Analyze intent from file changes and semantic context
    pub async fn analyze_intent(
        &self,
        file_changes: &[FileChange],
        semantic_context: &SemanticContext,
        description: &str,
    ) -> Result<IntentAnalysis> {
        // Analyze change intent
        let change_intent = self
            .analyze_change_intent(file_changes, description)
            .await?;

        // Identify affected features
        let affected_features = self
            .identify_affected_features(file_changes, semantic_context)
            .await?;

        // Detect design patterns and use them in the result
        let design_patterns = self
            .detect_design_patterns(file_changes, semantic_context)
            .await?;

        // Extract architectural decisions and use them in the result
        let architectural_decisions = self
            .extract_architectural_decisions(description, &change_intent)
            .await?;

        // Detect refactoring type and use it in the result
        let refactoring_type = self
            .detect_refactoring_type(file_changes, description)
            .await?;

        // Calculate confidence
        let confidence =
            self.calculate_intent_confidence(&change_intent, file_changes, description);

        // Convert to semantic intent representation for better analysis
        let _semantic_intent = self.convert_to_semantic_intent(change_intent.clone());

        // Convert to semantic types
        let semantic_change_intent = match change_intent {
            ChangeIntent::FeatureAddition => {
                crate::semantic::types::ChangeIntent::FeatureAddition {
                    feature_name: "detected".to_string(),
                    scope: "local".to_string(),
                }
            }
            ChangeIntent::BugFix => crate::semantic::types::ChangeIntent::BugFix {
                issue_description: "detected bug fix".to_string(),
                affected_components: vec!["unknown".to_string()],
            },
            ChangeIntent::Refactoring => crate::semantic::types::ChangeIntent::Refactoring {
                refactoring_pattern: "detected".to_string(),
                reason: "code improvement".to_string(),
            },
            ChangeIntent::Optimization => crate::semantic::types::ChangeIntent::Optimization {
                target_metric: "performance".to_string(),
                expected_improvement: "unknown".to_string(),
            },
            ChangeIntent::SecurityEnhancement => {
                crate::semantic::types::ChangeIntent::SecurityEnhancement {
                    vulnerability_type: "unknown".to_string(),
                    mitigation: "security improvement".to_string(),
                }
            }
            ChangeIntent::Performance => crate::semantic::types::ChangeIntent::Optimization {
                target_metric: "performance".to_string(),
                expected_improvement: "performance improvement".to_string(),
            },
            ChangeIntent::Security => crate::semantic::types::ChangeIntent::SecurityEnhancement {
                vulnerability_type: "security".to_string(),
                mitigation: "security improvement".to_string(),
            },
            ChangeIntent::Documentation => crate::semantic::types::ChangeIntent::Documentation {
                doc_type: "documentation".to_string(),
            },
            ChangeIntent::Testing => crate::semantic::types::ChangeIntent::Testing {
                test_type: "test".to_string(),
                coverage_area: "unknown".to_string(),
            },
            ChangeIntent::Maintenance => crate::semantic::types::ChangeIntent::Maintenance {
                maintenance_type: "code cleanup".to_string(),
            },
            ChangeIntent::Configuration => crate::semantic::types::ChangeIntent::Configuration {
                config_type: "configuration".to_string(),
                purpose: "configuration change".to_string(),
            },
            ChangeIntent::Migration => crate::semantic::types::ChangeIntent::Maintenance {
                maintenance_type: "migration".to_string(),
            },
            ChangeIntent::Unknown => crate::semantic::types::ChangeIntent::Maintenance {
                maintenance_type: "unknown".to_string(),
            },
        };

        // Convert design patterns to semantic types
        let semantic_design_patterns: Vec<crate::semantic::types::DetectedPattern> =
            design_patterns
                .into_iter()
                .map(|_| crate::semantic::types::DetectedPattern {
                    name: "Singleton".to_string(),
                    pattern: crate::semantic::types::DesignPattern::Singleton,
                    confidence: 0.7,
                    locations: Vec::new(),
                    description: None,
                })
                .collect();

        // Convert architectural decisions to semantic types
        let semantic_architectural_decisions: Vec<crate::semantic::types::ArchitecturalDecision> =
            architectural_decisions
                .into_iter()
                .map(|_| crate::semantic::types::ArchitecturalDecision {
                    decision: "design decision".to_string(),
                    reasoning: "architectural decision reasoning".to_string(),
                    alternatives: vec![],
                    tradeoffs: vec![],
                    impact: "low".to_string(),
                })
                .collect();

        // Convert refactoring type
        let semantic_refactoring_type =
            refactoring_type.map(|_| crate::semantic::types::RefactoringType::ExtractMethod);

        Ok(crate::semantic::types::IntentAnalysis {
            change_intent: semantic_change_intent,
            affected_features,
            design_patterns_used: semantic_design_patterns,
            architectural_decisions: semantic_architectural_decisions,
            refactoring_type: semantic_refactoring_type,
            confidence,
        })
    }

    /// Analyze change intent from files and description
    async fn analyze_change_intent(
        &self,
        file_changes: &[FileChange],
        description: &str,
    ) -> Result<ChangeIntent> {
        let mut intent_scores: HashMap<ChangeIntent, f64> = HashMap::new();

        // Initialize scores
        for intent in [
            ChangeIntent::FeatureAddition,
            ChangeIntent::BugFix,
            ChangeIntent::Refactoring,
            ChangeIntent::Performance,
            ChangeIntent::Security,
            ChangeIntent::Documentation,
            ChangeIntent::Testing,
            ChangeIntent::Maintenance,
            ChangeIntent::Configuration,
            ChangeIntent::Migration,
        ] {
            intent_scores.insert(intent, 0.0);
        }

        // Analyze description
        self.analyze_description_for_intent(description, &mut intent_scores);

        // Analyze file changes
        self.analyze_file_changes_for_intent(file_changes, &mut intent_scores);

        // Find highest scoring intent
        let best_intent = intent_scores
            .into_iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(intent, _)| intent)
            .unwrap_or(ChangeIntent::Unknown);

        Ok(best_intent)
    }

    /// Analyze description text for intent patterns
    fn analyze_description_for_intent(
        &self,
        description: &str,
        intent_scores: &mut HashMap<ChangeIntent, f64>,
    ) {
        let description_lower = description.to_lowercase();

        // Check for each intent pattern
        for (intent, patterns) in &self.intent_patterns {
            for pattern in patterns {
                if description_lower.contains(&pattern.to_lowercase()) {
                    *intent_scores.entry(intent.clone()).or_insert(0.0) += 1.0;
                }
            }
        }

        // Additional heuristics
        if description_lower.contains("fix") || description_lower.contains("bug") {
            *intent_scores.entry(ChangeIntent::BugFix).or_insert(0.0) += 2.0;
        }

        if description_lower.contains("add")
            || description_lower.contains("implement")
            || description_lower.contains("feature")
        {
            *intent_scores
                .entry(ChangeIntent::FeatureAddition)
                .or_insert(0.0) += 2.0;
        }

        if description_lower.contains("refactor") || description_lower.contains("restructure") {
            *intent_scores
                .entry(ChangeIntent::Refactoring)
                .or_insert(0.0) += 2.0;
        }

        if description_lower.contains("performance") || description_lower.contains("optimize") {
            *intent_scores
                .entry(ChangeIntent::Performance)
                .or_insert(0.0) += 2.0;
        }

        if description_lower.contains("security") || description_lower.contains("vulnerability") {
            *intent_scores.entry(ChangeIntent::Security).or_insert(0.0) += 2.0;
        }
    }

    /// Analyze file changes for intent patterns
    fn analyze_file_changes_for_intent(
        &self,
        file_changes: &[FileChange],
        intent_scores: &mut HashMap<ChangeIntent, f64>,
    ) {
        for file_change in file_changes {
            // Analyze file name
            let file_name = std::path::Path::new(&file_change.path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");

            // Test files indicate testing intent
            if file_name.contains("test") || file_name.contains("spec") {
                *intent_scores.entry(ChangeIntent::Testing).or_insert(0.0) += 1.0;
            }

            // Config files indicate configuration intent
            if file_name.ends_with(".config.js")
                || file_name.ends_with(".json")
                || file_name.ends_with(".yaml")
                || file_name.ends_with(".yml")
            {
                *intent_scores
                    .entry(ChangeIntent::Configuration)
                    .or_insert(0.0) += 1.0;
            }

            // Documentation files
            if file_name.ends_with(".md")
                || file_name.ends_with(".rst")
                || file_name.ends_with(".txt")
                || file_change.path.to_string_lossy().contains("/docs/")
            {
                *intent_scores
                    .entry(ChangeIntent::Documentation)
                    .or_insert(0.0) += 1.0;
            }

            // Analyze file content changes if available
            if let (Some(old_content), Some(new_content)) =
                (&file_change.original_content, &file_change.new_content)
            {
                self.analyze_content_changes(old_content, new_content, intent_scores);
            }
        }
    }

    /// Analyze content changes for intent patterns
    fn analyze_content_changes(
        &self,
        old_content: &str,
        new_content: &str,
        intent_scores: &mut HashMap<ChangeIntent, f64>,
    ) {
        let old_lines: Vec<&str> = old_content.lines().collect();
        let new_lines: Vec<&str> = new_content.lines().collect();

        // Simple diff analysis
        let added_lines: Vec<&str> = new_lines
            .iter()
            .filter(|line| !old_lines.contains(line))
            .copied()
            .collect();

        for line in added_lines {
            let line_lower = line.to_lowercase();

            // Look for performance optimizations
            if line_lower.contains("optimize")
                || line_lower.contains("cache")
                || line_lower.contains("async")
                || line_lower.contains("parallel")
            {
                *intent_scores
                    .entry(ChangeIntent::Performance)
                    .or_insert(0.0) += 0.5;
            }

            // Look for security additions
            if line_lower.contains("security")
                || line_lower.contains("auth")
                || line_lower.contains("validate")
                || line_lower.contains("sanitize")
            {
                *intent_scores.entry(ChangeIntent::Security).or_insert(0.0) += 0.5;
            }

            // Look for error handling (often bug fixes)
            if line_lower.contains("try")
                || line_lower.contains("catch")
                || line_lower.contains("error")
                || line_lower.contains("exception")
            {
                *intent_scores.entry(ChangeIntent::BugFix).or_insert(0.0) += 0.3;
            }

            // Look for new functions/methods (feature addition)
            if line_lower.trim_start().starts_with("function")
                || line_lower.trim_start().starts_with("async function")
                || line_lower.trim_start().starts_with("def ")
            {
                *intent_scores
                    .entry(ChangeIntent::FeatureAddition)
                    .or_insert(0.0) += 0.5;
            }
        }
    }

    /// Identify affected features from file changes
    async fn identify_affected_features(
        &self,
        file_changes: &[FileChange],
        semantic_context: &SemanticContext,
    ) -> Result<Vec<String>> {
        let mut features = Vec::new();

        // Extract features from file paths
        for file_change in file_changes {
            let path_str = file_change.path.to_string_lossy();
            let path_components: Vec<&str> = path_str.split('/').collect();

            // Look for feature directories
            for component in path_components {
                if component != "src"
                    && component != "lib"
                    && component != "test"
                    && component.len() > 2
                    && !component.ends_with(".ts")
                    && !component.ends_with(".js")
                {
                    features.push(component.to_string());
                }
            }
        }

        // Extract features from semantic context
        for (class_name, _class_def) in &semantic_context.classes {
            // Extract feature from class names (e.g., UserService -> User)
            if let Some(feature) = self.extract_feature_from_name(class_name) {
                features.push(feature);
            }
        }

        for (func_name, _func_def) in &semantic_context.functions {
            // Extract feature from function names
            if let Some(feature) = self.extract_feature_from_name(func_name) {
                features.push(feature);
            }
        }

        // Remove duplicates and sort
        features.sort();
        features.dedup();

        Ok(features)
    }

    /// Extract feature name from class/function names
    fn extract_feature_from_name(&self, name: &str) -> Option<String> {
        // Simple heuristic: extract prefix before Service, Manager, Controller, etc.
        let suffixes = [
            "Service",
            "Manager",
            "Controller",
            "Handler",
            "Provider",
            "Repository",
        ];

        for suffix in &suffixes {
            if name.ends_with(suffix) && name.len() > suffix.len() {
                return Some(name[..name.len() - suffix.len()].to_string());
            }
        }

        None
    }

    /// Detect design patterns in changes
    async fn detect_design_patterns(
        &self,
        _file_changes: &[FileChange],
        semantic_context: &SemanticContext,
    ) -> Result<Vec<crate::ai_context::context_builder::DesignPattern>> {
        let mut patterns = Vec::new();

        // Detect Singleton pattern
        for (class_name, _class_def) in &semantic_context.classes {
            if class_name.contains("Singleton") || class_name.ends_with("Instance") {
                patterns.push(crate::ai_context::context_builder::DesignPattern {
                    name: "Singleton".to_string(),
                    pattern_type: crate::ai_context::context_builder::PatternType::Creational,
                    description: "Singleton pattern detected".to_string(),
                    locations: vec![class_name.clone()],
                    confidence: 0.7,
                });
            }
        }

        // Detect Factory pattern
        for (class_name, _class_def) in &semantic_context.classes {
            if class_name.contains("Factory") || class_name.contains("Builder") {
                patterns.push(crate::ai_context::context_builder::DesignPattern {
                    name: "Factory".to_string(),
                    pattern_type: crate::ai_context::context_builder::PatternType::Creational,
                    description: "Factory pattern detected".to_string(),
                    locations: vec![class_name.clone()],
                    confidence: 0.8,
                });
            }
        }

        // Detect Observer pattern
        for (class_name, _class_def) in &semantic_context.classes {
            if class_name.contains("Observer")
                || class_name.contains("Listener")
                || class_name.contains("Subscriber")
            {
                patterns.push(crate::ai_context::context_builder::DesignPattern {
                    name: "Observer".to_string(),
                    pattern_type: crate::ai_context::context_builder::PatternType::Behavioral,
                    description: "Observer pattern detected".to_string(),
                    locations: vec![class_name.clone()],
                    confidence: 0.6,
                });
            }
        }

        Ok(patterns)
    }

    /// Extract architectural decisions from description
    async fn extract_architectural_decisions(
        &self,
        description: &str,
        change_intent: &ChangeIntent,
    ) -> Result<Vec<crate::ai_context::context_builder::ArchitecturalDecision>> {
        let mut decisions = Vec::new();

        // Look for architectural decision indicators
        let decision_keywords = ["decided", "chosen", "adopted", "migrated", "switched"];
        let description_lower = description.to_lowercase();

        for keyword in &decision_keywords {
            if description_lower.contains(keyword) {
                let decision_text = if description.len() > 100 {
                    format!("{}...", &description[..100])
                } else {
                    description.to_string()
                };

                decisions.push(crate::ai_context::context_builder::ArchitecturalDecision {
                    decision: decision_text,
                    reasoning: format!("Inferred from change intent: {:?}", change_intent),
                    alternatives: vec![], // Would be extracted with more sophisticated analysis
                    trade_offs: vec![],   // Would be extracted with more sophisticated analysis
                    timestamp: chrono::Utc::now(),
                });
                break;
            }
        }

        Ok(decisions)
    }

    /// Detect refactoring type
    async fn detect_refactoring_type(
        &self,
        file_changes: &[FileChange],
        description: &str,
    ) -> Result<Option<String>> {
        let description_lower = description.to_lowercase();

        // Check for explicit refactoring mentions
        if description_lower.contains("refactor") {
            // Try to identify specific refactoring type
            if description_lower.contains("extract") {
                return Ok(Some("Extract Method".to_string()));
            } else if description_lower.contains("rename") {
                return Ok(Some("Rename".to_string()));
            } else if description_lower.contains("move") {
                return Ok(Some("Move".to_string()));
            } else {
                return Ok(Some("General Refactoring".to_string()));
            }
        }

        // Detect refactoring patterns from file changes
        for detector in &self.refactoring_detectors {
            if self.matches_refactoring_pattern(&detector, file_changes, description) {
                return Ok(Some(detector.refactoring_type.clone()));
            }
        }

        Ok(None)
    }

    /// Check if changes match a refactoring pattern
    fn matches_refactoring_pattern(
        &self,
        detector: &RefactoringDetector,
        _file_changes: &[FileChange],
        description: &str,
    ) -> bool {
        let description_lower = description.to_lowercase();

        let mut matches = 0;
        for pattern in &detector.patterns {
            if description_lower.contains(&pattern.to_lowercase()) {
                matches += 1;
            }
        }

        (matches as f64 / detector.patterns.len() as f64) >= detector.threshold
    }

    /// Calculate confidence in intent analysis
    fn calculate_intent_confidence(
        &self,
        change_intent: &ChangeIntent,
        file_changes: &[FileChange],
        description: &str,
    ) -> f64 {
        let mut confidence: f64 = 0.3; // Base confidence

        // Boost confidence for explicit intent keywords
        let description_lower = description.to_lowercase();
        if matches!(change_intent, ChangeIntent::BugFix) && description_lower.contains("fix") {
            confidence += 0.3;
        }
        if matches!(change_intent, ChangeIntent::FeatureAddition)
            && description_lower.contains("add")
        {
            confidence += 0.3;
        }
        if matches!(change_intent, ChangeIntent::Refactoring)
            && description_lower.contains("refactor")
        {
            confidence += 0.4;
        }

        // Boost confidence based on number of files changed
        if file_changes.len() == 1 {
            confidence += 0.1; // Single file changes are often clearer
        } else if file_changes.len() > 10 {
            confidence -= 0.1; // Large changes are often mixed intent
        }

        // Boost confidence for clear descriptions
        if description.len() > 20 && description.len() < 200 {
            confidence += 0.1;
        }

        confidence.min(1.0)
    }

    /// Convert change intent to semantic intent format
    fn convert_to_semantic_intent(
        &self,
        change_intent: ChangeIntent,
    ) -> std::collections::HashMap<String, serde_json::Value> {
        let mut intent_map = std::collections::HashMap::new();
        intent_map.insert(
            "type".to_string(),
            serde_json::Value::String(format!("{:?}", change_intent)),
        );
        intent_map.insert(
            "confidence".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.8).unwrap()),
        );
        intent_map
    }

    /// Initialize intent patterns
    fn initialize_patterns(&mut self) {
        // Feature addition patterns
        self.intent_patterns.insert(
            ChangeIntent::FeatureAddition,
            vec![
                "add".to_string(),
                "implement".to_string(),
                "create".to_string(),
                "feature".to_string(),
                "new".to_string(),
            ],
        );

        // Bug fix patterns
        self.intent_patterns.insert(
            ChangeIntent::BugFix,
            vec![
                "fix".to_string(),
                "bug".to_string(),
                "error".to_string(),
                "issue".to_string(),
                "problem".to_string(),
            ],
        );

        // Refactoring patterns
        self.intent_patterns.insert(
            ChangeIntent::Refactoring,
            vec![
                "refactor".to_string(),
                "restructure".to_string(),
                "reorganize".to_string(),
                "cleanup".to_string(),
                "improve".to_string(),
            ],
        );

        // Add patterns for other intents...
    }

    /// Initialize refactoring detectors
    fn initialize_refactoring_detectors(&mut self) {
        self.refactoring_detectors.push(RefactoringDetector {
            refactoring_type: "Extract Method".to_string(),
            patterns: vec!["extract".to_string(), "method".to_string()],
            threshold: 0.5,
        });

        self.refactoring_detectors.push(RefactoringDetector {
            refactoring_type: "Rename".to_string(),
            patterns: vec!["rename".to_string(), "name".to_string()],
            threshold: 0.5,
        });

        // Add more refactoring detectors...
    }
}
