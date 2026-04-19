//! Design Pattern Detection
//!
//! This module detects design patterns, anti-patterns, and code smells in the codebase,
//! providing architectural insights and refactoring recommendations.

use super::knowledge_graph::{EdgeType, GraphNode, KnowledgeGraph, NodeType};
use super::types::*;
use crate::error::Result;
use std::collections::HashMap;
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
#[derive(Debug, Clone, PartialEq, PartialOrd)]
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
        let mut detected = Vec::new();
        let class_nodes = self.knowledge_graph.get_nodes_by_type(NodeType::Class);

        for node in class_nodes {
            let mut confidence = 0.0;
            let name_lower = node.name.to_lowercase();

            // Check naming hints
            if name_lower.contains("emitter")
                || name_lower.contains("publisher")
                || name_lower.contains("eventbus")
                || name_lower.contains("dispatcher")
            {
                confidence += 0.4;
            }

            // Check metadata for subscribe/emit-like methods
            let methods = self.get_method_names_from_metadata(&node);
            if methods.iter().any(|m| {
                m.contains("subscribe")
                    || m.contains("addEventListener")
                    || m.contains("on")
                    || m.contains("addListener")
            }) {
                confidence += 0.3;
            }
            if methods.iter().any(|m| {
                m.contains("emit")
                    || m.contains("notify")
                    || m.contains("publish")
                    || m.contains("dispatch")
            }) {
                confidence += 0.3;
            }

            if confidence >= 0.6 {
                detected.push(DetectedPattern {
                    pattern_name: "Observer".to_string(),
                    pattern_type: PatternType::Observer,
                    confidence,
                    entities: vec![PatternEntity {
                        name: node.name.clone(),
                        role: "Subject / EventEmitter".to_string(),
                        location: node.location.clone(),
                    }],
                    description: "Defines a one-to-many dependency between objects so that when one changes state, all dependents are notified".to_string(),
                    benefits: vec![
                        "Loose coupling between subject and observers".to_string(),
                        "Supports broadcast communication".to_string(),
                    ],
                    drawbacks: vec![
                        "Can lead to unexpected updates".to_string(),
                        "Memory leaks if observers aren't removed".to_string(),
                    ],
                    recommendation: Some(PatternRecommendation {
                        action: RecommendationAction::Accept,
                        reasoning: "Observer pattern is appropriate for event-driven communication".to_string(),
                        implementation_steps: Vec::new(),
                        estimated_effort: EffortLevel::Low,
                    }),
                });
            }
        }

        Ok(detected)
    }

    /// Detect Strategy pattern
    fn detect_strategy(&self) -> Result<Vec<DetectedPattern>> {
        let mut detected = Vec::new();
        let interface_nodes = self.knowledge_graph.get_nodes_by_type(NodeType::Interface);

        for iface in &interface_nodes {
            let iface_lower = iface.name.to_lowercase();
            if !iface_lower.contains("strategy")
                && !iface_lower.contains("policy")
                && !iface_lower.contains("algorithm")
                && !iface_lower.contains("handler")
            {
                continue;
            }

            // Look for classes that implement this interface
            let all_classes = self.knowledge_graph.get_nodes_by_type(NodeType::Class);
            let implementors: Vec<&GraphNode> = all_classes
                .iter()
                .filter(|c| {
                    self.knowledge_graph
                        .get_outgoing_edges(&c.id)
                        .iter()
                        .any(|e| e.edge_type == EdgeType::Implements && e.to == iface.id)
                })
                .collect();

            // Strategy pattern typically has 2+ implementations
            if implementors.len() >= 2 {
                let mut entities = vec![PatternEntity {
                    name: iface.name.clone(),
                    role: "Strategy Interface".to_string(),
                    location: iface.location.clone(),
                }];
                for imp in &implementors {
                    entities.push(PatternEntity {
                        name: imp.name.clone(),
                        role: "Concrete Strategy".to_string(),
                        location: imp.location.clone(),
                    });
                }

                detected.push(DetectedPattern {
                    pattern_name: "Strategy".to_string(),
                    pattern_type: PatternType::Strategy,
                    confidence: 0.85,
                    entities,
                    description: format!(
                        "Interface {} with {} interchangeable implementations",
                        iface.name,
                        implementors.len()
                    ),
                    benefits: vec![
                        "Algorithms can be swapped at runtime".to_string(),
                        "Open/Closed principle".to_string(),
                    ],
                    drawbacks: vec!["Clients must be aware of different strategies".to_string()],
                    recommendation: Some(PatternRecommendation {
                        action: RecommendationAction::Accept,
                        reasoning: "Well-structured strategy pattern".to_string(),
                        implementation_steps: Vec::new(),
                        estimated_effort: EffortLevel::Low,
                    }),
                });
            }
        }

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
            let param_count = self.count_parameters(&node);

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

    /// Detect Feature Envy — a method uses another class's data more than its own
    fn detect_feature_envy(&self) -> Result<Vec<DetectedPattern>> {
        let mut detected = Vec::new();
        let function_nodes = self.knowledge_graph.get_nodes_by_type(NodeType::Function);

        for node in function_nodes {
            let outgoing = self.knowledge_graph.get_outgoing_edges(&node.id);

            // Group call/use edges by target file
            let mut external_calls: HashMap<String, usize> = HashMap::new();
            let mut own_file_calls = 0usize;

            for edge in &outgoing {
                if edge.edge_type == EdgeType::Calls || edge.edge_type == EdgeType::Uses {
                    if let Some(target) = self.knowledge_graph.get_node(&edge.to) {
                        if target.file_path == node.file_path {
                            own_file_calls += 1;
                        } else {
                            *external_calls.entry(target.file_path.clone()).or_default() += 1;
                        }
                    }
                }
            }

            // If any single external file is referenced more than own-file references
            for (ext_file, ext_count) in &external_calls {
                if *ext_count > own_file_calls && *ext_count >= 3 {
                    detected.push(DetectedPattern {
                        pattern_name: "Feature Envy".to_string(),
                        pattern_type: PatternType::FeatureEnvy,
                        confidence: (*ext_count as f64 / (*ext_count + own_file_calls) as f64)
                            .min(0.95),
                        entities: vec![PatternEntity {
                            name: node.name.clone(),
                            role: "Envious Method".to_string(),
                            location: node.location.clone(),
                        }],
                        description: format!(
                            "Method {} references {} {} times but its own file only {} times",
                            node.name, ext_file, ext_count, own_file_calls
                        ),
                        benefits: Vec::new(),
                        drawbacks: vec![
                            "Suggests the method belongs in another class".to_string(),
                            "Increases coupling between modules".to_string(),
                        ],
                        recommendation: Some(PatternRecommendation {
                            action: RecommendationAction::Refactor,
                            reasoning: format!(
                                "Consider moving this method to {}",
                                ext_file
                            ),
                            implementation_steps: vec![
                                "Move the method to the class it references most".to_string(),
                                "Update callers to use the new location".to_string(),
                            ],
                            estimated_effort: EffortLevel::Medium,
                        }),
                    });
                    break; // one detection per method
                }
            }
        }

        Ok(detected)
    }

    /// Detect Data Class — classes with fields but no meaningful behaviour
    fn detect_data_class(&self) -> Result<Vec<DetectedPattern>> {
        let mut detected = Vec::new();
        let class_nodes = self.knowledge_graph.get_nodes_by_type(NodeType::Class);

        for node in class_nodes {
            let method_count = self.count_methods(&node);
            let field_count = self.count_fields(&node);

            // A data class has many fields but very few (or only getter/setter) methods
            if field_count >= 3 && method_count <= field_count {
                let methods = self.get_method_names_from_metadata(&node);
                let getter_setter_count = methods
                    .iter()
                    .filter(|m| {
                        m.starts_with("get")
                            || m.starts_with("set")
                            || m.starts_with("is")
                            || m.starts_with("has")
                    })
                    .count();

                // Most methods are getters/setters → data class
                if getter_setter_count >= methods.len().saturating_sub(1) {
                    detected.push(DetectedPattern {
                        pattern_name: "Data Class".to_string(),
                        pattern_type: PatternType::DataClass,
                        confidence: 0.8,
                        entities: vec![PatternEntity {
                            name: node.name.clone(),
                            role: "Data Class".to_string(),
                            location: node.location.clone(),
                        }],
                        description: format!(
                            "Class {} has {} fields and {} methods (mostly accessors)",
                            node.name, field_count, method_count
                        ),
                        benefits: Vec::new(),
                        drawbacks: vec![
                            "Anemic domain model".to_string(),
                            "Behavior scattered across other classes".to_string(),
                        ],
                        recommendation: Some(PatternRecommendation {
                            action: RecommendationAction::Review,
                            reasoning:
                                "Consider moving related behaviour into this class".to_string(),
                            implementation_steps: vec![
                                "Identify methods in other classes that operate on this data"
                                    .to_string(),
                                "Move them into this class to create a richer domain model"
                                    .to_string(),
                            ],
                            estimated_effort: EffortLevel::Medium,
                        }),
                    });
                }
            }
        }

        Ok(detected)
    }

    // Helper methods

    fn has_static_instance(&self, node: &GraphNode) -> bool {
        // Check metadata for static fields containing "instance"
        if let Some(fields) = node.metadata.get("fields") {
            if let Some(arr) = fields.as_array() {
                return arr.iter().any(|f| {
                    let name = f.get("name").and_then(|n| n.as_str()).unwrap_or("");
                    let is_static = f.get("is_static").and_then(|s| s.as_bool()).unwrap_or(false);
                    is_static && name.to_lowercase().contains("instance")
                });
            }
        }
        // Fallback: check outgoing edges for a self-typed static field
        let outgoing = self.knowledge_graph.get_outgoing_edges(&node.id);
        outgoing
            .iter()
            .any(|e| e.edge_type == EdgeType::Contains && e.to.contains("instance"))
    }

    fn has_private_constructor(&self, node: &GraphNode) -> bool {
        if let Some(methods) = node.metadata.get("methods") {
            if let Some(arr) = methods.as_array() {
                return arr.iter().any(|m| {
                    let name = m.get("name").and_then(|n| n.as_str()).unwrap_or("");
                    let vis = m.get("visibility").and_then(|v| v.as_str()).unwrap_or("");
                    (name == "constructor" || name == "new") && vis == "private"
                });
            }
        }
        false
    }

    fn has_get_instance_method(&self, node: &GraphNode) -> bool {
        let methods = self.get_method_names_from_metadata(node);
        methods.iter().any(|m| {
            let lower = m.to_lowercase();
            lower == "getinstance"
                || lower == "get_instance"
                || lower == "instance"
                || lower == "shared"
        })
    }

    fn looks_like_factory(&self, node: &GraphNode) -> bool {
        let lower = node.name.to_lowercase();
        if lower.contains("factory") || lower.contains("creator") {
            return true;
        }
        // Also check for methods named "create*"
        let methods = self.get_method_names_from_metadata(node);
        methods.iter().filter(|m| m.starts_with("create")).count() >= 2
    }

    fn count_methods(&self, node: &GraphNode) -> usize {
        // First try metadata
        if let Some(methods) = node.metadata.get("methods") {
            if let Some(arr) = methods.as_array() {
                return arr.len();
            }
        }
        // Fallback: count outgoing Contains edges to Function nodes
        let outgoing = self.knowledge_graph.get_outgoing_edges(&node.id);
        outgoing
            .iter()
            .filter(|e| {
                e.edge_type == EdgeType::Contains
                    && self
                        .knowledge_graph
                        .get_node(&e.to)
                        .map(|n| n.node_type == NodeType::Function)
                        .unwrap_or(false)
            })
            .count()
    }

    fn count_fields(&self, node: &GraphNode) -> usize {
        // First try metadata
        if let Some(fields) = node.metadata.get("fields") {
            if let Some(arr) = fields.as_array() {
                return arr.len();
            }
        }
        // Fallback: count outgoing Contains edges to Variable/Constant nodes
        let outgoing = self.knowledge_graph.get_outgoing_edges(&node.id);
        outgoing
            .iter()
            .filter(|e| {
                e.edge_type == EdgeType::Contains
                    && self
                        .knowledge_graph
                        .get_node(&e.to)
                        .map(|n| {
                            n.node_type == NodeType::Variable || n.node_type == NodeType::Constant
                        })
                        .unwrap_or(false)
            })
            .count()
    }

    fn count_parameters(&self, node: &GraphNode) -> usize {
        if let Some(params) = node.metadata.get("parameters") {
            if let Some(arr) = params.as_array() {
                return arr.len();
            }
            if let Some(n) = params.as_u64() {
                return n as usize;
            }
        }
        0
    }

    fn count_dependencies(&self, node: &GraphNode) -> usize {
        self.knowledge_graph.get_outgoing_edges(&node.id).len()
    }

    /// Get method names from node metadata
    fn get_method_names_from_metadata(&self, node: &GraphNode) -> Vec<String> {
        if let Some(methods) = node.metadata.get("methods") {
            if let Some(arr) = methods.as_array() {
                return arr
                    .iter()
                    .filter_map(|m| {
                        m.get("name")
                            .and_then(|n| n.as_str())
                            .map(|s| s.to_string())
                    })
                    .collect();
            }
        }
        // Fallback: query contained function nodes
        let outgoing = self.knowledge_graph.get_outgoing_edges(&node.id);
        outgoing
            .iter()
            .filter(|e| e.edge_type == EdgeType::Contains)
            .filter_map(|e| {
                self.knowledge_graph.get_node(&e.to).and_then(|n| {
                    if n.node_type == NodeType::Function {
                        Some(n.name)
                    } else {
                        None
                    }
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::knowledge_graph::NodeType;
    use crate::semantic::types::CodeLocation;

    fn make_graph_node(name: &str, node_type: NodeType, metadata: HashMap<String, serde_json::Value>) -> GraphNode {
        GraphNode {
            id: name.to_string(),
            node_type,
            name: name.to_string(),
            file_path: "test.ts".to_string(),
            location: CodeLocation {
                file_path: std::path::PathBuf::from("test.ts"),
                start_line: 1,
                end_line: 10,
                start_column: 0,
                end_column: 0,
            },
            metadata,
            checkpoint_id: None,
        }
    }

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

    #[test]
    fn test_detect_observer_by_name() {
        let graph = Arc::new(KnowledgeGraph::new());
        let mut metadata = HashMap::new();
        metadata.insert("methods".to_string(), serde_json::json!([
            {"name": "on"}, {"name": "emit"}, {"name": "removeListener"}
        ]));
        let node = make_graph_node("EventEmitter", NodeType::Class, metadata);
        let _ = graph.add_node(node);
        let detector = PatternDetector::new(graph.clone());
        let result = detector.detect_observer().unwrap();
        assert!(!result.is_empty(), "Should detect observer pattern from EventEmitter class name + methods");
    }

    #[test]
    fn test_detect_data_class() {
        let graph = Arc::new(KnowledgeGraph::new());
        let mut metadata = HashMap::new();
        metadata.insert("fields".to_string(), serde_json::json!(["name", "age", "email", "phone"]));
        metadata.insert("methods".to_string(), serde_json::json!(["toString"]));
        let node = make_graph_node("PersonDTO", NodeType::Class, metadata);
        let _ = graph.add_node(node);
        let detector = PatternDetector::new(graph.clone());
        let result = detector.detect_data_class().unwrap();
        assert!(!result.is_empty(), "Should detect data class (4 fields, 1 method)");
    }

    #[test]
    fn test_count_methods_from_metadata() {
        let graph = Arc::new(KnowledgeGraph::new());
        let detector = PatternDetector::new(graph);
        let mut metadata = HashMap::new();
        metadata.insert("methods".to_string(), serde_json::json!(["a", "b", "c"]));
        let node = make_graph_node("TestClass", NodeType::Class, metadata);
        assert_eq!(detector.count_methods(&node), 3);
    }

    #[test]
    fn test_count_fields_from_metadata() {
        let graph = Arc::new(KnowledgeGraph::new());
        let detector = PatternDetector::new(graph);
        let mut metadata = HashMap::new();
        metadata.insert("fields".to_string(), serde_json::json!(["x", "y"]));
        let node = make_graph_node("TestClass", NodeType::Class, metadata);
        assert_eq!(detector.count_fields(&node), 2);
    }

    #[test]
    fn test_looks_like_factory() {
        let graph = Arc::new(KnowledgeGraph::new());
        let detector = PatternDetector::new(graph);
        let mut metadata = HashMap::new();
        metadata.insert("methods".to_string(), serde_json::json!(["createButton", "createInput", "destroy"]));
        let node = make_graph_node("WidgetFactory", NodeType::Class, metadata);
        assert!(detector.looks_like_factory(&node), "Should detect factory pattern from name + create* methods");
    }
}
