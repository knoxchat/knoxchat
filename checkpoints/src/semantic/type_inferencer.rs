//! Type Inference Engine
//!
//! This module infers types for dynamic languages and enhances type understanding
//! for static languages, providing better code understanding for AI context.

use super::knowledge_graph::{GraphNode, KnowledgeGraph};
use super::types::*;
use crate::error::Result;
use std::collections::HashMap;
use std::sync::Arc;

/// Type inference engine
pub struct TypeInferencer {
    knowledge_graph: Arc<KnowledgeGraph>,
    config: TypeInferenceConfig,
    type_registry: HashMap<String, TypeInfo>,
}

/// Configuration for type inference
#[derive(Debug, Clone)]
pub struct TypeInferenceConfig {
    pub infer_function_returns: bool,
    pub infer_variable_types: bool,
    pub infer_generic_types: bool,
    pub use_flow_analysis: bool,
    pub confidence_threshold: f64,
}

impl Default for TypeInferenceConfig {
    fn default() -> Self {
        Self {
            infer_function_returns: true,
            infer_variable_types: true,
            infer_generic_types: true,
            use_flow_analysis: true,
            confidence_threshold: 0.7,
        }
    }
}

/// Type information
#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub name: String,
    pub type_kind: TypeKind,
    pub constraints: Vec<TypeConstraint>,
    pub generic_params: Vec<String>,
    pub properties: HashMap<String, PropertyType>,
    pub methods: HashMap<String, MethodSignature>,
}

/// Kind of type
#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    Primitive(PrimitiveType),
    Class(String),
    Interface(String),
    Union(Vec<String>),
    Intersection(Vec<String>),
    Array(Box<TypeKind>),
    Tuple(Vec<TypeKind>),
    Function(FunctionType),
    Generic(String, Vec<TypeKind>),
    Unknown,
    Any,
    Never,
}

/// Primitive types
#[derive(Debug, Clone, PartialEq)]
pub enum PrimitiveType {
    Number,
    String,
    Boolean,
    Null,
    Undefined,
    Symbol,
    BigInt,
}

/// Function type
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionType {
    pub parameters: Vec<ParameterType>,
    pub return_type: Box<TypeKind>,
    pub is_async: bool,
    pub is_generator: bool,
}

/// Parameter type
#[derive(Debug, Clone, PartialEq)]
pub struct ParameterType {
    pub name: String,
    pub param_type: TypeKind,
    pub optional: bool,
    pub rest: bool,
}

/// Property type
#[derive(Debug, Clone)]
pub struct PropertyType {
    pub name: String,
    pub prop_type: TypeKind,
    pub readonly: bool,
    pub optional: bool,
}

/// Method signature
#[derive(Debug, Clone)]
pub struct MethodSignature {
    pub name: String,
    pub parameters: Vec<ParameterType>,
    pub return_type: TypeKind,
    pub visibility: Visibility,
}

/// Visibility level
#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
}

/// Type constraint
#[derive(Debug, Clone)]
pub struct TypeConstraint {
    pub constraint_type: ConstraintType,
    pub target_type: TypeKind,
}

/// Type of constraint
#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintType {
    Extends,
    Implements,
    SuperType,
    SubType,
}

/// Type inference result
#[derive(Debug, Clone)]
pub struct TypeInferenceResult {
    pub inferred_types: HashMap<String, InferredType>,
    pub type_errors: Vec<TypeError>,
    pub type_warnings: Vec<TypeWarning>,
    pub confidence_scores: HashMap<String, f64>,
}

/// Inferred type
#[derive(Debug, Clone)]
pub struct InferredType {
    pub identifier: String,
    pub inferred_type: TypeKind,
    pub confidence: f64,
    pub evidence: Vec<TypeEvidence>,
    pub location: CodeLocation,
}

/// Type evidence
#[derive(Debug, Clone)]
pub struct TypeEvidence {
    pub source: EvidenceSource,
    pub evidence_type: TypeKind,
    pub strength: f64,
    pub location: CodeLocation,
}

/// Source of evidence
#[derive(Debug, Clone, PartialEq)]
pub enum EvidenceSource {
    Assignment,
    FunctionCall,
    PropertyAccess,
    ReturnStatement,
    Parameter,
    Annotation,
    Usage,
}

/// Type error
#[derive(Debug, Clone)]
pub struct TypeError {
    pub error_type: TypeErrorKind,
    pub expected: TypeKind,
    pub found: TypeKind,
    pub location: CodeLocation,
    pub message: String,
}

/// Type error kind
#[derive(Debug, Clone, PartialEq)]
pub enum TypeErrorKind {
    Mismatch,
    MissingProperty,
    InvalidOperation,
    ArgumentCountMismatch,
    IncompatibleAssignment,
}

/// Type warning
#[derive(Debug, Clone)]
pub struct TypeWarning {
    pub warning_type: TypeWarningKind,
    pub location: CodeLocation,
    pub message: String,
    pub suggestion: Option<String>,
}

/// Type warning kind
#[derive(Debug, Clone, PartialEq)]
pub enum TypeWarningKind {
    ImplicitAny,
    PossibleNull,
    UnusedVariable,
    UnsafeOperation,
}

impl TypeInferencer {
    /// Create a new type inferencer
    pub fn new(knowledge_graph: Arc<KnowledgeGraph>) -> Self {
        Self {
            knowledge_graph,
            config: TypeInferenceConfig::default(),
            type_registry: HashMap::new(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(knowledge_graph: Arc<KnowledgeGraph>, config: TypeInferenceConfig) -> Self {
        Self {
            knowledge_graph,
            config,
            type_registry: HashMap::new(),
        }
    }

    /// Infer types for entire codebase
    pub fn infer_types(&mut self) -> Result<TypeInferenceResult> {
        let mut inferred_types = HashMap::new();
        let mut type_errors = Vec::new();
        let mut type_warnings = Vec::new();
        let mut confidence_scores = HashMap::new();

        // Infer function return types
        if self.config.infer_function_returns {
            let function_types = self.infer_function_types()?;
            inferred_types.extend(function_types);
        }

        // Infer variable types
        if self.config.infer_variable_types {
            let variable_types = self.infer_variable_types()?;
            inferred_types.extend(variable_types);
        }

        // Validate and find errors
        let (errors, warnings) = self.validate_types(&inferred_types)?;
        type_errors.extend(errors);
        type_warnings.extend(warnings);

        // Calculate confidence scores
        for (id, inferred) in &inferred_types {
            confidence_scores.insert(id.clone(), inferred.confidence);
        }

        Ok(TypeInferenceResult {
            inferred_types,
            type_errors,
            type_warnings,
            confidence_scores,
        })
    }

    /// Infer function return types
    fn infer_function_types(&self) -> Result<HashMap<String, InferredType>> {
        let mut inferred = HashMap::new();
        let function_nodes = self
            .knowledge_graph
            .get_nodes_by_type(super::knowledge_graph::NodeType::Function);

        for node in function_nodes {
            let return_type = self.infer_function_return_type(&node)?;

            inferred.insert(
                node.id.clone(),
                InferredType {
                    identifier: node.name.clone(),
                    inferred_type: return_type.clone(),
                    confidence: 0.8,
                    evidence: vec![TypeEvidence {
                        source: EvidenceSource::ReturnStatement,
                        evidence_type: return_type,
                        strength: 0.8,
                        location: node.location.clone(),
                    }],
                    location: node.location.clone(),
                },
            );
        }

        Ok(inferred)
    }

    /// Infer function return type
    fn infer_function_return_type(&self, _node: &GraphNode) -> Result<TypeKind> {
        // Would analyze return statements and infer type
        // For now, placeholder
        Ok(TypeKind::Unknown)
    }

    /// Infer variable types
    fn infer_variable_types(&self) -> Result<HashMap<String, InferredType>> {
        let inferred = HashMap::new();

        // Would analyze variable assignments and usage
        // Build type constraints and solve

        Ok(inferred)
    }

    /// Infer type from expression
    pub fn infer_expression_type(&self, expression: &str) -> Result<TypeKind> {
        // Literal patterns
        if expression.starts_with('"') || expression.starts_with('\'') {
            return Ok(TypeKind::Primitive(PrimitiveType::String));
        }

        if expression.parse::<i64>().is_ok() {
            return Ok(TypeKind::Primitive(PrimitiveType::Number));
        }

        if expression == "true" || expression == "false" {
            return Ok(TypeKind::Primitive(PrimitiveType::Boolean));
        }

        if expression == "null" {
            return Ok(TypeKind::Primitive(PrimitiveType::Null));
        }

        if expression == "undefined" {
            return Ok(TypeKind::Primitive(PrimitiveType::Undefined));
        }

        // Array literal
        if expression.starts_with('[') && expression.ends_with(']') {
            return Ok(TypeKind::Array(Box::new(TypeKind::Unknown)));
        }

        // Object literal
        if expression.starts_with('{') && expression.ends_with('}') {
            return Ok(TypeKind::Class("Object".to_string()));
        }

        // Function expression
        if expression.contains("=>") || expression.starts_with("function") {
            return Ok(TypeKind::Function(FunctionType {
                parameters: Vec::new(),
                return_type: Box::new(TypeKind::Unknown),
                is_async: expression.contains("async"),
                is_generator: expression.contains("*"),
            }));
        }

        Ok(TypeKind::Unknown)
    }

    /// Unify types
    pub fn unify_types(&self, type1: &TypeKind, type2: &TypeKind) -> Result<TypeKind> {
        match (type1, type2) {
            (TypeKind::Unknown, t) | (t, TypeKind::Unknown) => Ok(t.clone()),
            (TypeKind::Any, _) | (_, TypeKind::Any) => Ok(TypeKind::Any),

            (TypeKind::Primitive(p1), TypeKind::Primitive(p2)) if p1 == p2 => {
                Ok(TypeKind::Primitive(p1.clone()))
            }

            (TypeKind::Array(t1), TypeKind::Array(t2)) => {
                let unified = self.unify_types(t1, t2)?;
                Ok(TypeKind::Array(Box::new(unified)))
            }

            (t1, t2) if t1 == t2 => Ok(t1.clone()),

            _ => Ok(TypeKind::Union(vec![
                format!("{:?}", type1),
                format!("{:?}", type2),
            ])),
        }
    }

    /// Validate types and find errors
    fn validate_types(
        &self,
        inferred_types: &HashMap<String, InferredType>,
    ) -> Result<(Vec<TypeError>, Vec<TypeWarning>)> {
        let errors = Vec::new();
        let mut warnings = Vec::new();

        for (_, inferred) in inferred_types {
            // Check for low confidence
            if inferred.confidence < self.config.confidence_threshold {
                warnings.push(TypeWarning {
                    warning_type: TypeWarningKind::ImplicitAny,
                    location: inferred.location.clone(),
                    message: format!(
                        "Low confidence ({:.2}) in type inference for '{}'",
                        inferred.confidence, inferred.identifier
                    ),
                    suggestion: Some("Consider adding explicit type annotation".to_string()),
                });
            }

            // Check for Union with Unknown
            if matches!(inferred.inferred_type, TypeKind::Union(_)) {
                warnings.push(TypeWarning {
                    warning_type: TypeWarningKind::UnsafeOperation,
                    location: inferred.location.clone(),
                    message: format!("Union type inferred for '{}'", inferred.identifier),
                    suggestion: Some("Narrow type with type guards".to_string()),
                });
            }
        }

        Ok((errors, warnings))
    }

    /// Check type compatibility
    pub fn is_compatible(&self, source: &TypeKind, target: &TypeKind) -> bool {
        match (source, target) {
            (TypeKind::Any, _) | (_, TypeKind::Any) => true,
            (TypeKind::Unknown, _) | (_, TypeKind::Unknown) => true,
            (TypeKind::Never, _) => true,

            (TypeKind::Primitive(p1), TypeKind::Primitive(p2)) => p1 == p2,

            (TypeKind::Array(t1), TypeKind::Array(t2)) => self.is_compatible(t1, t2),

            (TypeKind::Union(types), _target) => {
                types.iter().all(|_t| {
                    // Simplified - would need proper type checking
                    true
                })
            }

            (_source, TypeKind::Union(types)) => {
                types.iter().any(|_t| {
                    // Simplified - would need proper type checking
                    true
                })
            }

            _ => source == target,
        }
    }

    /// Register built-in types
    pub fn register_builtin_types(&mut self) {
        // Register common built-in types
        self.register_type(TypeInfo {
            name: "String".to_string(),
            type_kind: TypeKind::Primitive(PrimitiveType::String),
            constraints: Vec::new(),
            generic_params: Vec::new(),
            properties: HashMap::new(),
            methods: HashMap::new(),
        });

        self.register_type(TypeInfo {
            name: "Number".to_string(),
            type_kind: TypeKind::Primitive(PrimitiveType::Number),
            constraints: Vec::new(),
            generic_params: Vec::new(),
            properties: HashMap::new(),
            methods: HashMap::new(),
        });

        // ... more built-ins
    }

    /// Register a type
    pub fn register_type(&mut self, type_info: TypeInfo) {
        self.type_registry.insert(type_info.name.clone(), type_info);
    }

    /// Get type info
    pub fn get_type_info(&self, name: &str) -> Option<&TypeInfo> {
        self.type_registry.get(name)
    }

    /// Format type for display
    pub fn format_type(&self, type_kind: &TypeKind) -> String {
        match type_kind {
            TypeKind::Primitive(p) => format!("{:?}", p),
            TypeKind::Class(name) => name.clone(),
            TypeKind::Interface(name) => name.clone(),
            TypeKind::Array(t) => format!("Array<{}>", self.format_type(t)),
            TypeKind::Tuple(types) => {
                let formatted: Vec<_> = types.iter().map(|t| self.format_type(t)).collect();
                format!("[{}]", formatted.join(", "))
            }
            TypeKind::Union(types) => types.join(" | "),
            TypeKind::Intersection(types) => types.join(" & "),
            TypeKind::Function(ft) => {
                let params: Vec<_> = ft
                    .parameters
                    .iter()
                    .map(|p| format!("{}: {}", p.name, self.format_type(&p.param_type)))
                    .collect();
                format!(
                    "({}) => {}",
                    params.join(", "),
                    self.format_type(&ft.return_type)
                )
            }
            TypeKind::Generic(name, params) => {
                let formatted: Vec<_> = params.iter().map(|t| self.format_type(t)).collect();
                format!("{}<{}>", name, formatted.join(", "))
            }
            TypeKind::Unknown => "unknown".to_string(),
            TypeKind::Any => "any".to_string(),
            TypeKind::Never => "never".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inferencer_creation() {
        let graph = Arc::new(KnowledgeGraph::new());
        let inferencer = TypeInferencer::new(graph);
        assert!(inferencer.config.infer_function_returns);
    }

    #[test]
    fn test_infer_literal_types() {
        let graph = Arc::new(KnowledgeGraph::new());
        let inferencer = TypeInferencer::new(graph);

        assert!(matches!(
            inferencer.infer_expression_type("\"hello\"").unwrap(),
            TypeKind::Primitive(PrimitiveType::String)
        ));

        assert!(matches!(
            inferencer.infer_expression_type("42").unwrap(),
            TypeKind::Primitive(PrimitiveType::Number)
        ));

        assert!(matches!(
            inferencer.infer_expression_type("true").unwrap(),
            TypeKind::Primitive(PrimitiveType::Boolean)
        ));
    }

    #[test]
    fn test_format_type() {
        let graph = Arc::new(KnowledgeGraph::new());
        let inferencer = TypeInferencer::new(graph);

        let type_kind = TypeKind::Array(Box::new(TypeKind::Primitive(PrimitiveType::String)));
        assert_eq!(inferencer.format_type(&type_kind), "Array<String>");
    }
}
