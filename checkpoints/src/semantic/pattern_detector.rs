//! Design Pattern Detection
//!
//! This module detects design patterns, anti-patterns, and code smells in the codebase,
//! providing architectural insights and refactoring recommendations.

use super::knowledge_graph::{GraphNode, KnowledgeGraph, NodeType};
use super::types::*;
use crate::error::Result;
use std::sync::Arc;

/// Pattern detector for design patterns and anti-patterns
pub struct PatternDetector {
    knowledge_graph: Arc<KnowledgeGraph>,
    config: PatternDetectionConfig,
}

/// Configuration for pattern detection
#[derive(Debug, Clone)]
pub struct PatternDetectionConfig {
    pub enable_design_patterns: bool,
    pub enable_anti_patterns: bool,
    pub enable_code_smells: bool,
    pub confidence_threshold: f64,
}

impl Default for PatternDetectionConfig {
    fn default() -> Self {
        Self {
            enable_design_patterns: true,
            enable_anti_patterns: true,
            enable_code_smells: true,
            confidence_threshold: 0.7,
        }
    }
}

/// Detected pattern
#[derive(Debug, Clone)]
pub struct DetectedPattern {
    pub pattern_name: String,
    pub pattern_type: PatternType,
    pub confidence: f64,
    pub entities: Vec<PatternEntity>,
    pub description: String,
    pub benefits: Vec<String>,
    pub drawbacks: Vec<String>,
    pub recommendation: Option<PatternRecommendation>,
}

/// Type of pattern
#[derive(Debug, Clone, PartialEq)]
pub enum PatternType {
    // Creational patterns
    Singleton,
    Factory,
    AbstractFactory,
    Builder,
    Prototype,

    // Structural patterns
    Adapter,
    Bridge,
    Composite,
    Decorator,
    Facade,
    Flyweight,
    Proxy,

    // Behavioral patterns
    ChainOfResponsibility,
    Command,
    Iterator,
    Mediator,
    Memento,
    Observer,
    State,
    Strategy,
    Template,
    Visitor,

    // Architectural patterns
    MVC,
    MVP,
    MVVM,
    Repository,
    ServiceLayer,

    // Anti-patterns
    GodObject,
    SpaghettiCode,
    LavaFlow,
    GoldenHammer,
    CopyPasteProgramming,

    // Code smells
    LongMethod,
    LargeClass,
    LongParameterList,
    FeatureEnvy,
    DataClass,
}

/// Entity involved in pattern
#[derive(Debug, Clone)]
pub struct PatternEntity {
    pub name: String,
    pub role: String,
    pub location: CodeLocation,
}

/// Pattern recommendation
#[derive(Debug, Clone)]
pub struct PatternRecommendation {
    pub action: RecommendationAction,
    pub reasoning: String,
    pub implementation_steps: Vec<String>,
    pub estimated_effort: EffortLevel,
}

/// Recommendation action
#[derive(Debug, Clone, PartialEq)]
pub enum RecommendationAction {
    Adopt,    // Good pattern, should use
    Refactor, // Anti-pattern, should fix
    Review,   // Needs review
    Accept,   // Known pattern, acceptable
    Improve,  // Can be improved
}

/// Effort level
#[derive(Debug, Clone, PartialEq)]
pub enum EffortLevel {
    Trivial,
    Low,
    Medium,
    High,
    VeryHigh,
}

impl PatternDetector {
    /// Create a new pattern detector
    pub fn new(knowledge_graph: Arc<KnowledgeGraph>) -> Self {
        Self {
            knowledge_graph,
            config: PatternDetectionConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        knowledge_graph: Arc<KnowledgeGraph>,
        config: PatternDetectionConfig,
    ) -> Self {
        Self {
            knowledge_graph,
            config,
        }
    }

    /// Detect all patterns in codebase
    pub fn detect_patterns(&self) -> Result<Vec<DetectedPattern>> {
        let mut detected = Vec::new();

        if self.config.enable_design_patterns {
            detected.extend(self.detect_design_patterns()?);
        }

        if self.config.enable_anti_patterns {
            detected.extend(self.detect_anti_patterns()?);
        }

        if self.config.enable_code_smells {
            detected.extend(self.detect_code_smells()?);
        }

        // Filter by confidence threshold
        detected.retain(|p| p.confidence >= self.config.confidence_threshold);

        Ok(detected)
    }

    /// Detect design patterns
    fn detect_design_patterns(&self) -> Result<Vec<DetectedPattern>> {
        let mut patterns = Vec::new();

        // Detect Singleton pattern
        patterns.extend(self.detect_singleton()?);

        // Detect Factory pattern
        patterns.extend(self.detect_factory()?);

        // Detect Observer pattern
        patterns.extend(self.detect_observer()?);

        // Detect Strategy pattern
        patterns.extend(self.detect_strategy()?);

        // Additional patterns would be implemented similarly...

        Ok(patterns)
    }

    /// Detect anti-patterns
    fn detect_anti_patterns(&self) -> Result<Vec<DetectedPattern>> {
        let mut patterns = Vec::new();

        // Detect God Object
        patterns.extend(self.detect_god_object()?);

        // Detect Long Method
        patterns.extend(self.detect_long_method()?);

        // Detect Large Class
        patterns.extend(self.detect_large_class()?);

        Ok(patterns)
    }

    /// Detect code smells
    fn detect_code_smells(&self) -> Result<Vec<DetectedPattern>> {
        let mut smells = Vec::new();

        // Detect Long Parameter List
        smells.extend(self.detect_long_parameter_list()?);

        // Detect Feature Envy
        smells.extend(self.detect_feature_envy()?);

        // Detect Data Class
        smells.extend(self.detect_data_class()?);

        Ok(smells)
    }

    /// Detect Singleton pattern
    fn detect_singleton(&self) -> Result<Vec<DetectedPattern>> {
        let mut detected = Vec::new();
        let class_nodes = self.knowledge_graph.get_nodes_by_type(NodeType::Class);

        for node in class_nodes {
            let mut confidence = 0.0;
            let mut reasons = Vec::new();

            // Check for static instance
            if self.has_static_instance(&node) {
                confidence += 0.4;
                reasons.push("Has static instance field");
            }

            // Check for private constructor
            if self.has_private_constructor(&node) {
                confidence += 0.3;
                reasons.push("Has private constructor");
            }

            // Check for getInstance method
            if self.has_get_instance_method(&node) {
                confidence += 0.3;
                reasons.push("Has getInstance method");
            }

            if confidence >= 0.7 {
                detected.push(DetectedPattern {
                    pattern_name: "Singleton".to_string(),
                    pattern_type: PatternType::Singleton,
                    confidence,
                    entities: vec![PatternEntity {
                        name: node.name.clone(),
                        role: "Singleton".to_string(),
                        location: node.location.clone(),
                    }],
                    description: "Ensures a class has only one instance and provides global access to it".to_string(),
                    benefits: vec![
                        "Controlled access to sole instance".to_string(),
                        "Reduced namespace pollution".to_string(),
                    ],
                    drawbacks: vec![
                        "Difficult to test".to_string(),
                        "Hidden dependencies".to_string(),
                        "Violates Single Responsibility Principle".to_string(),
                    ],
                    recommendation: Some(PatternRecommendation {
                        action: RecommendationAction::Review,
                        reasoning: "Singleton pattern detected. Consider dependency injection as alternative".to_string(),
                        implementation_steps: vec![
                            "Consider using dependency injection instead".to_string(),
                            "Make testability a priority".to_string(),
                        ],
                        estimated_effort: EffortLevel::Medium,
                    }),
                });
            }
        }

        Ok(detected)
    }

    /// Detect Factory pattern
    fn detect_factory(&self) -> Result<Vec<DetectedPattern>> {
        let mut detected = Vec::new();
        let class_nodes = self.knowledge_graph.get_nodes_by_type(NodeType::Class);

        for node in class_nodes {
            if self.looks_like_factory(&node) {
                detected.push(DetectedPattern {
                    pattern_name: "Factory".to_string(),
                    pattern_type: PatternType::Factory,
                    confidence: 0.8,
                    entities: vec![PatternEntity {
                        name: node.name.clone(),
                        role: "Factory".to_string(),
                        location: node.location.clone(),
                    }],
                    description: "Creates objects without specifying exact class".to_string(),
                    benefits: vec!["Loose coupling".to_string(), "Easier to extend".to_string()],
                    drawbacks: vec!["Can become complex".to_string()],
                    recommendation: Some(PatternRecommendation {
                        action: RecommendationAction::Accept,
                        reasoning: "Well-implemented factory pattern".to_string(),
                        implementation_steps: Vec::new(),
                        estimated_effort: EffortLevel::Low,
                    }),
                });
            }
        }

        Ok(detected)
    }

    /// Detect Observer pattern
    fn detect_observer(&self) -> Result<Vec<DetectedPattern>> {
        let detected = Vec::new();
        // Implementation would check for:
        // - Subject with list of observers
        // - Notify method
        // - Observer interface/trait
        // - Multiple implementations of observer
        Ok(detected)
    }

    /// Detect Strategy pattern
    fn detect_strategy(&self) -> Result<Vec<DetectedPattern>> {
        let detected = Vec::new();
        // Implementation would check for:
        // - Strategy interface
        // - Multiple implementations
        // - Context class using strategy
        Ok(detected)
    }

    /// Detect God Object anti-pattern
    fn detect_god_object(&self) -> Result<Vec<DetectedPattern>> {
        let mut detected = Vec::new();
        let class_nodes = self.knowledge_graph.get_nodes_by_type(NodeType::Class);

        for node in class_nodes {
            let method_count = self.count_methods(&node);
            let field_count = self.count_fields(&node);
            let dependency_count = self.count_dependencies(&node);

            // God object heuristics
            if method_count > 20 || field_count > 15 || dependency_count > 10 {
                let confidence = ((method_count as f64 / 20.0).min(1.0)
                    + (field_count as f64 / 15.0).min(1.0)
                    + (dependency_count as f64 / 10.0).min(1.0))
                    / 3.0;

                detected.push(DetectedPattern {
                    pattern_name: "God Object".to_string(),
                    pattern_type: PatternType::GodObject,
                    confidence,
                    entities: vec![PatternEntity {
                        name: node.name.clone(),
                        role: "God Object".to_string(),
                        location: node.location.clone(),
                    }],
                    description: format!(
                        "Class with too many responsibilities ({} methods, {} fields, {} dependencies)",
                        method_count, field_count, dependency_count
                    ),
                    benefits: Vec::new(),
                    drawbacks: vec![
                        "Violates Single Responsibility Principle".to_string(),
                        "Hard to maintain and test".to_string(),
                        "High coupling".to_string(),
                    ],
                    recommendation: Some(PatternRecommendation {
                        action: RecommendationAction::Refactor,
                        reasoning: "Class has too many responsibilities and should be split".to_string(),
                        implementation_steps: vec![
                            "Identify cohesive groups of methods and fields".to_string(),
                            "Extract these groups into separate classes".to_string(),
                            "Use composition instead of a single large class".to_string(),
                        ],
                        estimated_effort: EffortLevel::High,
                    }),
                });
            }
        }

        Ok(detected)
    }

    /// Detect Long Method code smell
    fn detect_long_method(&self) -> Result<Vec<DetectedPattern>> {
        let mut detected = Vec::new();
        let function_nodes = self.knowledge_graph.get_nodes_by_type(NodeType::Function);

        for node in function_nodes {
            let line_count = node.location.end_line - node.location.start_line;

            if line_count > 50 {
                detected.push(DetectedPattern {
                    pattern_name: "Long Method".to_string(),
                    pattern_type: PatternType::LongMethod,
                    confidence: (line_count as f64 / 100.0).min(1.0),
                    entities: vec![PatternEntity {
                        name: node.name.clone(),
                        role: "Long Method".to_string(),
                        location: node.location.clone(),
                    }],
                    description: format!("Method is {} lines long", line_count),
                    benefits: Vec::new(),
                    drawbacks: vec![
                        "Hard to understand".to_string(),
                        "Difficult to reuse".to_string(),
                        "Hard to test".to_string(),
                    ],
                    recommendation: Some(PatternRecommendation {
                        action: RecommendationAction::Refactor,
                        reasoning: "Method should be broken into smaller, focused methods"
                            .to_string(),
                        implementation_steps: vec![
                            "Identify logical sections in the method".to_string(),
                            "Extract each section into a separate method".to_string(),
                            "Give extracted methods meaningful names".to_string(),
                        ],
                        estimated_effort: EffortLevel::Medium,
                    }),
                });
            }
        }

        Ok(detected)
    }

    /// Detect Large Class code smell
    fn detect_large_class(&self) -> Result<Vec<DetectedPattern>> {
        let mut detected = Vec::new();
        let class_nodes = self.knowledge_graph.get_nodes_by_type(NodeType::Class);

        for node in class_nodes {
            let line_count = node.location.end_line - node.location.start_line;

            if line_count > 300 {
                detected.push(DetectedPattern {
                    pattern_name: "Large Class".to_string(),
                    pattern_type: PatternType::LargeClass,
                    confidence: (line_count as f64 / 500.0).min(1.0),
                    entities: vec![PatternEntity {
                        name: node.name.clone(),
                        role: "Large Class".to_string(),
                        location: node.location.clone(),
                    }],
                    description: format!("Class is {} lines long", line_count),
                    benefits: Vec::new(),
                    drawbacks: vec![
                        "Hard to understand".to_string(),
                        "Probably has multiple responsibilities".to_string(),
                        "Difficult to maintain".to_string(),
                    ],
                    recommendation: Some(PatternRecommendation {
                        action: RecommendationAction::Refactor,
                        reasoning: "Class is too large and should be split".to_string(),
                        implementation_steps: vec![
                            "Identify cohesive groups of functionality".to_string(),
                            "Extract these groups into separate classes".to_string(),
                            "Consider using composition".to_string(),
                        ],
                        estimated_effort: EffortLevel::High,
                    }),
                });
            }
        }

        Ok(detected)
    }

    /// Detect Long Parameter List
    fn detect_long_parameter_list(&self) -> Result<Vec<DetectedPattern>> {
        let mut detected = Vec::new();
        let function_nodes = self.knowledge_graph.get_nodes_by_type(NodeType::Function);

        for node in function_nodes {
            // Would extract parameter count from actual parsing
            // For now, placeholder
            let param_count = 0; // Would be extracted from node metadata

            if param_count > 5 {
                detected.push(DetectedPattern {
                    pattern_name: "Long Parameter List".to_string(),
                    pattern_type: PatternType::LongParameterList,
                    confidence: 0.9,
                    entities: vec![PatternEntity {
                        name: node.name.clone(),
                        role: "Function".to_string(),
                        location: node.location.clone(),
                    }],
                    description: format!("Function has {} parameters", param_count),
                    benefits: Vec::new(),
                    drawbacks: vec!["Hard to call".to_string(), "Hard to understand".to_string()],
                    recommendation: Some(PatternRecommendation {
                        action: RecommendationAction::Refactor,
                        reasoning: "Consider parameter object or builder pattern".to_string(),
                        implementation_steps: vec![
                            "Group related parameters into an object".to_string(),
                            "Use builder pattern for optional parameters".to_string(),
                        ],
                        estimated_effort: EffortLevel::Low,
                    }),
                });
            }
        }

        Ok(detected)
    }

    /// Detect Feature Envy
    fn detect_feature_envy(&self) -> Result<Vec<DetectedPattern>> {
        // Would analyze method calls to other classes
        // If a method uses another class more than its own, it's feature envy
        Ok(Vec::new())
    }

    /// Detect Data Class
    fn detect_data_class(&self) -> Result<Vec<DetectedPattern>> {
        // Would check for classes with only getters/setters and no behavior
        Ok(Vec::new())
    }

    // Helper methods

    fn has_static_instance(&self, _node: &GraphNode) -> bool {
        // Would check for static instance field
        false
    }

    fn has_private_constructor(&self, _node: &GraphNode) -> bool {
        // Would check for private constructor
        false
    }

    fn has_get_instance_method(&self, _node: &GraphNode) -> bool {
        // Would check for getInstance method
        false
    }

    fn looks_like_factory(&self, node: &GraphNode) -> bool {
        node.name.contains("Factory") || node.name.contains("Creator")
    }

    fn count_methods(&self, _node: &GraphNode) -> usize {
        // Would count methods in class
        0
    }

    fn count_fields(&self, _node: &GraphNode) -> usize {
        // Would count fields in class
        0
    }

    fn count_dependencies(&self, node: &GraphNode) -> usize {
        self.knowledge_graph.get_outgoing_edges(&node.id).len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let graph = Arc::new(KnowledgeGraph::new());
        let detector = PatternDetector::new(graph);
        assert!(detector.config.enable_design_patterns);
    }

    #[test]
    fn test_effort_levels() {
        assert!(EffortLevel::Low < EffortLevel::High);
        assert!(EffortLevel::Trivial < EffortLevel::VeryHigh);
    }
}
