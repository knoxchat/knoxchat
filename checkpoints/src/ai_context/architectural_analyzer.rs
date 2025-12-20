//! Architectural Analysis Module
//!
//! Analyzes architectural patterns, dependencies, and system structure

use super::*;
use crate::semantic::{AIContextCheckpoint, SemanticContext};

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Architectural analyzer
pub struct ArchitecturalAnalyzer {
    /// Pattern detectors
    pattern_detectors: Vec<PatternDetector>,
    /// Layer detection rules
    layer_rules: Vec<LayerRule>,
    /// Dependency analyzers
    dependency_analyzers: Vec<DependencyAnalyzer>,
}

/// Architectural patterns detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalPatterns {
    /// Detected patterns
    pub patterns: Vec<DetectedPattern>,
    /// Pattern confidence scores
    pub confidence_scores: HashMap<String, f64>,
    /// Pattern relationships
    pub pattern_relationships: Vec<PatternRelationship>,
}

/// Detected architectural pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPattern {
    /// Pattern name
    pub name: String,
    /// Pattern type
    pub pattern_type: PatternType,
    /// Pattern description
    pub description: String,
    /// Components that implement this pattern
    pub components: Vec<String>,
    /// Confidence in detection
    pub confidence: f64,
    /// Evidence for the pattern
    pub evidence: Vec<String>,
}

/// Types of architectural patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    Layered,
    MVC,
    Repository,
    Factory,
    Singleton,
    Observer,
    Strategy,
    Facade,
    Adapter,
    Microservices,
    EventDriven,
    CQRS,
    DependencyInjection,
    Other(String),
}

/// Relationship between patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternRelationship {
    /// Source pattern
    pub from: String,
    /// Target pattern
    pub to: String,
    /// Relationship type
    pub relationship_type: RelationshipType,
    /// Strength of relationship
    pub strength: f64,
}

/// Types of pattern relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipType {
    Implements,
    Uses,
    Extends,
    Contains,
    ComplementedBy,
}

/// Pattern detector
#[derive(Debug, Clone)]
struct PatternDetector {
    /// Pattern name
    pattern_name: String,
    /// Pattern type
    pattern_type: PatternType,
    /// Detection rules
    rules: Vec<DetectionRule>,
    /// Minimum confidence threshold
    threshold: f64,
}

/// Detection rule for patterns
#[derive(Debug, Clone)]
struct DetectionRule {
    /// Rule description
    _description: String,
    /// Rule weight
    weight: f64,
    /// Rule matcher
    matcher: RuleMatcher,
}

/// Rule matching criteria
#[derive(Debug, Clone)]
#[allow(unused)]
enum RuleMatcher {
    /// Class name pattern
    ClassNamePattern(String),
    /// Method name pattern
    MethodNamePattern(String),
    /// Inheritance pattern
    InheritancePattern(String),
    /// Directory structure pattern
    DirectoryPattern(String),
    /// Interface implementation pattern
    InterfacePattern(String),
    /// Dependency pattern
    DependencyPattern(String),
}

/// Layer detection rule
#[derive(Debug, Clone)]
struct LayerRule {
    /// Layer name
    layer_name: String,
    /// Layer type
    layer_type: String,
    /// Directory patterns
    directory_patterns: Vec<String>,
    /// Class name patterns
    class_patterns: Vec<String>,
    /// Typical responsibilities
    responsibilities: Vec<String>,
}

/// Dependency analyzer
#[derive(Debug, Clone)]
struct DependencyAnalyzer {
    /// Analyzer name
    _name: String,
    /// Analysis type
    _analysis_type: DependencyAnalysisType,
    /// Analysis rules
    _rules: Vec<String>,
}

/// Types of dependency analysis
#[derive(Debug, Clone)]
#[allow(unused)]
enum DependencyAnalysisType {
    CircularDependency,
    LayerViolation,
    UnusedDependency,
    CouplingAnalysis,
    CohesionAnalysis,
}

impl ArchitecturalAnalyzer {
    /// Create a new architectural analyzer
    pub fn new() -> Result<Self> {
        let mut analyzer = Self {
            pattern_detectors: Vec::new(),
            layer_rules: Vec::new(),
            dependency_analyzers: Vec::new(),
        };

        analyzer.initialize_pattern_detectors();
        analyzer.initialize_layer_rules();
        analyzer.initialize_dependency_analyzers();

        Ok(analyzer)
    }

    /// Analyze architectural patterns in the codebase
    pub async fn analyze_patterns(
        &self,
        scored_content: &[(AIContextCheckpoint, SemanticContext, f64)],
    ) -> Result<Vec<crate::ai_context::context_builder::DesignPattern>> {
        let mut detected_patterns = Vec::new();

        // Collect all semantic contexts for analysis
        let all_contexts: Vec<&SemanticContext> = scored_content
            .iter()
            .map(|(_, context, _)| context)
            .collect();

        // Run pattern detectors
        for detector in &self.pattern_detectors {
            if let Some(pattern) = self.detect_pattern(detector, &all_contexts).await? {
                detected_patterns.push(crate::ai_context::context_builder::DesignPattern {
                    name: pattern.name,
                    pattern_type: self.convert_pattern_type(pattern.pattern_type),
                    description: pattern.description,
                    locations: pattern.components,
                    confidence: pattern.confidence,
                });
            }
        }

        Ok(detected_patterns)
    }

    /// Analyze project structure
    pub async fn analyze_project_structure(
        &self,
        scored_content: &[(AIContextCheckpoint, SemanticContext, f64)],
    ) -> Result<crate::ai_context::context_builder::ProjectStructure> {
        let mut root_directories = HashSet::new();
        let mut modules = Vec::new();
        let mut dependencies = Vec::new();

        // Extract directory structure from file paths
        for (checkpoint, semantic_context, _) in scored_content {
            for file_change in &checkpoint.base_checkpoint.file_changes {
                let path = std::path::Path::new(&file_change.path);

                // Extract root directory
                if let Some(root) = path.components().next() {
                    if let std::path::Component::Normal(dir) = root {
                        if let Some(dir_str) = dir.to_str() {
                            root_directories.insert(dir_str.to_string());
                        }
                    }
                }

                // Create module info
                if let Some(parent) = path.parent() {
                    let module_path = parent.to_string_lossy().to_string();
                    let exports = self.extract_exports_from_context(semantic_context);
                    let deps = self.extract_dependencies_from_context(semantic_context);

                    modules.push(crate::ai_context::context_builder::ModuleInfo {
                        path: module_path,
                        module_type: self.infer_module_type(&file_change.path.to_string_lossy()),
                        exports,
                        dependencies: deps,
                    });
                }
            }
        }

        // Extract dependencies (simplified)
        dependencies.push(crate::ai_context::context_builder::DependencyInfo {
            name: "example-dependency".to_string(),
            version: "1.0.0".to_string(),
            dependency_type: crate::ai_context::context_builder::DependencyType::Production,
        });

        Ok(crate::ai_context::context_builder::ProjectStructure {
            root_directories: root_directories.into_iter().collect(),
            modules,
            dependencies,
        })
    }

    /// Build dependency graph
    pub async fn build_dependency_graph(
        &self,
        scored_content: &[(AIContextCheckpoint, SemanticContext, f64)],
    ) -> Result<crate::ai_context::context_builder::DependencyGraph> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        #[allow(unused_assignments)]
        let mut cycles = Vec::new();

        // Create nodes for each module/class
        for (checkpoint, semantic_context, score) in scored_content {
            // Add nodes for classes with metadata including score and checkpoint info
            for (class_name, _class_def) in &semantic_context.classes {
                let mut metadata = HashMap::new();
                metadata.insert(
                    "score".to_string(),
                    serde_json::Value::String(score.to_string()),
                );
                metadata.insert(
                    "checkpoint_id".to_string(),
                    serde_json::Value::String(checkpoint.base_checkpoint.id.to_string()),
                );
                metadata.insert(
                    "checkpoint_desc".to_string(),
                    serde_json::Value::String(checkpoint.base_checkpoint.description.clone()),
                );

                nodes.push(crate::ai_context::context_builder::DependencyNode {
                    id: class_name.clone(),
                    node_type: "class".to_string(),
                    metadata,
                });
            }

            // Add nodes for functions
            for (func_name, _func_def) in &semantic_context.functions {
                nodes.push(crate::ai_context::context_builder::DependencyNode {
                    id: func_name.clone(),
                    node_type: "function".to_string(),
                    metadata: HashMap::new(),
                });
            }
        }

        // Create edges based on dependencies
        for (_, semantic_context, _) in scored_content {
            for import in &semantic_context.imports {
                for imported_item in &import.imported_items {
                    edges.push(crate::ai_context::context_builder::DependencyEdge {
                        from: import.module.clone(),
                        to: imported_item.clone(),
                        edge_type: "import".to_string(),
                        strength: 1.0,
                    });
                }
            }
        }

        // Detect cycles (simplified)
        cycles = self.detect_dependency_cycles(&nodes, &edges);

        Ok(crate::ai_context::context_builder::DependencyGraph {
            nodes,
            edges,
            cycles,
        })
    }

    /// Identify architectural layers
    pub async fn identify_layers(
        &self,
        scored_content: &[(AIContextCheckpoint, SemanticContext, f64)],
    ) -> Result<Vec<crate::ai_context::context_builder::ArchitecturalLayer>> {
        let mut layers = Vec::new();
        let mut layer_components: HashMap<String, Vec<String>> = HashMap::new();

        // Apply layer detection rules
        for rule in &self.layer_rules {
            let mut components = Vec::new();

            for (checkpoint, semantic_context, _) in scored_content {
                // Check directory patterns
                for file_change in &checkpoint.base_checkpoint.file_changes {
                    for pattern in &rule.directory_patterns {
                        if file_change.path.to_string_lossy().contains(pattern) {
                            components.push(file_change.path.to_string_lossy().to_string());
                        }
                    }
                }

                // Check class patterns
                for (class_name, _) in &semantic_context.classes {
                    for pattern in &rule.class_patterns {
                        if class_name.contains(pattern) {
                            components.push(class_name.clone());
                        }
                    }
                }
            }

            if !components.is_empty() {
                layer_components.insert(rule.layer_name.clone(), components);
            }
        }

        // Create architectural layers
        for (layer_name, components) in layer_components {
            if let Some(rule) = self.layer_rules.iter().find(|r| r.layer_name == layer_name) {
                layers.push(crate::ai_context::context_builder::ArchitecturalLayer {
                    name: layer_name,
                    layer_type: rule.layer_type.clone(),
                    components,
                    responsibilities: rule.responsibilities.clone(),
                });
            }
        }

        Ok(layers)
    }

    // Private helper methods

    /// Detect a specific pattern
    async fn detect_pattern(
        &self,
        detector: &PatternDetector,
        contexts: &[&SemanticContext],
    ) -> Result<Option<DetectedPattern>> {
        let mut total_score = 0.0;
        let mut evidence = Vec::new();
        let mut components = Vec::new();

        // Apply detection rules
        for rule in &detector.rules {
            let (rule_score, rule_evidence, rule_components) =
                self.apply_detection_rule(rule, contexts)?;
            total_score += rule_score;
            evidence.extend(rule_evidence);
            components.extend(rule_components);
        }

        // Calculate confidence
        let max_possible_score: f64 = detector.rules.iter().map(|r| r.weight).sum();
        let confidence = if max_possible_score > 0.0 {
            total_score / max_possible_score
        } else {
            0.0
        };

        // Check if pattern is detected
        if confidence >= detector.threshold {
            Ok(Some(DetectedPattern {
                name: detector.pattern_name.clone(),
                pattern_type: detector.pattern_type.clone(),
                description: format!(
                    "Detected {} pattern with {:.1}% confidence",
                    detector.pattern_name,
                    confidence * 100.0
                ),
                components,
                confidence,
                evidence,
            }))
        } else {
            Ok(None)
        }
    }

    /// Apply a detection rule
    fn apply_detection_rule(
        &self,
        rule: &DetectionRule,
        contexts: &[&SemanticContext],
    ) -> Result<(f64, Vec<String>, Vec<String>)> {
        let mut score = 0.0;
        let mut evidence = Vec::new();
        let mut components = Vec::new();

        match &rule.matcher {
            RuleMatcher::ClassNamePattern(pattern) => {
                for context in contexts {
                    for (class_name, _) in &context.classes {
                        if class_name.contains(pattern) {
                            score += rule.weight;
                            evidence
                                .push(format!("Class {} matches pattern {}", class_name, pattern));
                            components.push(class_name.clone());
                        }
                    }
                }
            }
            RuleMatcher::MethodNamePattern(pattern) => {
                for context in contexts {
                    for (func_name, _) in &context.functions {
                        if func_name.contains(pattern) {
                            score += rule.weight * 0.5; // Methods are less indicative than classes
                            evidence
                                .push(format!("Method {} matches pattern {}", func_name, pattern));
                            components.push(func_name.clone());
                        }
                    }
                }
            }
            RuleMatcher::InterfacePattern(pattern) => {
                for context in contexts {
                    for (interface_name, _) in &context.interfaces {
                        if interface_name.contains(pattern) {
                            score += rule.weight;
                            evidence.push(format!(
                                "Interface {} matches pattern {}",
                                interface_name, pattern
                            ));
                            components.push(interface_name.clone());
                        }
                    }
                }
            }
            // Add more matcher implementations...
            _ => {
                // Simplified implementation for other matchers
            }
        }

        Ok((score, evidence, components))
    }

    /// Extract exports from semantic context
    fn extract_exports_from_context(&self, context: &SemanticContext) -> Vec<String> {
        let mut exports = Vec::new();

        for export in &context.exports {
            exports.extend(export.exported_items.clone());
        }

        // Add class names as exports
        for (class_name, _) in &context.classes {
            exports.push(class_name.clone());
        }

        // Add function names as exports
        for (func_name, _) in &context.functions {
            exports.push(func_name.clone());
        }

        exports
    }

    /// Extract dependencies from semantic context
    fn extract_dependencies_from_context(&self, context: &SemanticContext) -> Vec<String> {
        let mut dependencies = Vec::new();

        for import in &context.imports {
            dependencies.push(import.module.clone());
        }

        dependencies
    }

    /// Infer module type from file path
    fn infer_module_type(&self, file_path: &str) -> String {
        if file_path.contains("/test/")
            || file_path.contains(".test.")
            || file_path.contains(".spec.")
        {
            "test".to_string()
        } else if file_path.contains("/config/") || file_path.ends_with(".config.js") {
            "configuration".to_string()
        } else if file_path.contains("/types/") || file_path.ends_with(".d.ts") {
            "types".to_string()
        } else if file_path.contains("/utils/") || file_path.contains("/helpers/") {
            "utility".to_string()
        } else {
            "source".to_string()
        }
    }

    /// Detect dependency cycles
    fn detect_dependency_cycles(
        &self,
        nodes: &[crate::ai_context::context_builder::DependencyNode],
        edges: &[crate::ai_context::context_builder::DependencyEdge],
    ) -> Vec<Vec<String>> {
        // Simplified cycle detection using DFS
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        // Build adjacency list
        let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();
        for edge in edges {
            adj_list
                .entry(edge.from.clone())
                .or_insert_with(Vec::new)
                .push(edge.to.clone());
        }

        // Run DFS for each unvisited node
        for node in nodes {
            if !visited.contains(&node.id) {
                let mut path = Vec::new();
                if self.dfs_cycle_detection(
                    &node.id,
                    &adj_list,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                ) {
                    cycles.push(path);
                }
            }
        }

        cycles
    }

    /// DFS cycle detection helper
    fn dfs_cycle_detection(
        &self,
        node: &str,
        adj_list: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> bool {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(neighbors) = adj_list.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if self.dfs_cycle_detection(neighbor, adj_list, visited, rec_stack, path) {
                        return true;
                    }
                } else if rec_stack.contains(neighbor) {
                    // Found a cycle
                    if let Some(cycle_start) = path.iter().position(|x| x == neighbor) {
                        path.drain(0..cycle_start);
                    }
                    return true;
                }
            }
        }

        rec_stack.remove(node);
        path.pop();
        false
    }

    /// Convert pattern type to context builder format
    fn convert_pattern_type(
        &self,
        pattern_type: PatternType,
    ) -> crate::ai_context::context_builder::PatternType {
        match pattern_type {
            PatternType::Factory | PatternType::Singleton => {
                crate::ai_context::context_builder::PatternType::Creational
            }
            PatternType::Facade | PatternType::Adapter => {
                crate::ai_context::context_builder::PatternType::Structural
            }
            PatternType::Observer | PatternType::Strategy => {
                crate::ai_context::context_builder::PatternType::Behavioral
            }
            PatternType::Layered | PatternType::MVC | PatternType::Microservices => {
                crate::ai_context::context_builder::PatternType::Architectural
            }
            _ => crate::ai_context::context_builder::PatternType::Architectural,
        }
    }

    /// Initialize pattern detectors
    fn initialize_pattern_detectors(&mut self) {
        // MVC Pattern Detector
        self.pattern_detectors.push(PatternDetector {
            pattern_name: "MVC".to_string(),
            pattern_type: PatternType::MVC,
            rules: vec![
                DetectionRule {
                    _description: "Controller classes".to_string(),
                    weight: 1.0,
                    matcher: RuleMatcher::ClassNamePattern("Controller".to_string()),
                },
                DetectionRule {
                    _description: "Model classes".to_string(),
                    weight: 1.0,
                    matcher: RuleMatcher::ClassNamePattern("Model".to_string()),
                },
                DetectionRule {
                    _description: "View classes".to_string(),
                    weight: 1.0,
                    matcher: RuleMatcher::ClassNamePattern("View".to_string()),
                },
            ],
            threshold: 0.6,
        });

        // Repository Pattern Detector
        self.pattern_detectors.push(PatternDetector {
            pattern_name: "Repository".to_string(),
            pattern_type: PatternType::Repository,
            rules: vec![
                DetectionRule {
                    _description: "Repository classes".to_string(),
                    weight: 1.0,
                    matcher: RuleMatcher::ClassNamePattern("Repository".to_string()),
                },
                DetectionRule {
                    _description: "Repository interfaces".to_string(),
                    weight: 0.8,
                    matcher: RuleMatcher::InterfacePattern("Repository".to_string()),
                },
            ],
            threshold: 0.7,
        });

        // Add more pattern detectors...
    }

    /// Initialize layer detection rules
    fn initialize_layer_rules(&mut self) {
        self.layer_rules.push(LayerRule {
            layer_name: "Presentation".to_string(),
            layer_type: "UI".to_string(),
            directory_patterns: vec![
                "ui".to_string(),
                "views".to_string(),
                "components".to_string(),
            ],
            class_patterns: vec![
                "View".to_string(),
                "Component".to_string(),
                "Controller".to_string(),
            ],
            responsibilities: vec!["User interface".to_string(), "User interaction".to_string()],
        });

        self.layer_rules.push(LayerRule {
            layer_name: "Business".to_string(),
            layer_type: "Logic".to_string(),
            directory_patterns: vec![
                "business".to_string(),
                "services".to_string(),
                "logic".to_string(),
            ],
            class_patterns: vec![
                "Service".to_string(),
                "Manager".to_string(),
                "Handler".to_string(),
            ],
            responsibilities: vec!["Business rules".to_string(), "Domain logic".to_string()],
        });

        self.layer_rules.push(LayerRule {
            layer_name: "Data".to_string(),
            layer_type: "Persistence".to_string(),
            directory_patterns: vec![
                "data".to_string(),
                "repositories".to_string(),
                "dao".to_string(),
            ],
            class_patterns: vec![
                "Repository".to_string(),
                "DAO".to_string(),
                "Entity".to_string(),
            ],
            responsibilities: vec!["Data access".to_string(), "Data persistence".to_string()],
        });
    }

    /// Initialize dependency analyzers
    fn initialize_dependency_analyzers(&mut self) {
        self.dependency_analyzers.push(DependencyAnalyzer {
            _name: "Circular Dependency Detector".to_string(),
            _analysis_type: DependencyAnalysisType::CircularDependency,
            _rules: vec!["Detect import cycles".to_string()],
        });

        self.dependency_analyzers.push(DependencyAnalyzer {
            _name: "Layer Violation Detector".to_string(),
            _analysis_type: DependencyAnalysisType::LayerViolation,
            _rules: vec!["Check layer dependencies".to_string()],
        });
    }
}
