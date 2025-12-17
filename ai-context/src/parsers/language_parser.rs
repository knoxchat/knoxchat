/**
 * Language Parser Trait and Common Types
 * 
 * This module defines the common interface and types for language-specific
 * AST parsers used in semantic analysis.
 */

use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};

/// Common trait for all language parsers
pub trait LanguageParser: Send + Sync {
    /// Parse a file and return its AST
    fn parse_file(&self, content: &str, file_path: &Path) -> Result<AST, ParserError>;
    
    /// Extract symbols from an AST
    fn extract_symbols(&self, ast: &AST) -> Vec<Symbol>;
    
    /// Build call graph from an AST
    fn build_call_graph(&self, ast: &AST) -> CallGraph;
    
    /// Analyze dependencies from an AST
    fn analyze_dependencies(&self, ast: &AST) -> Vec<Dependency>;
    
    /// Get supported file extensions
    fn supported_extensions(&self) -> Vec<&'static str>;
    
    /// Get language name
    fn language_name(&self) -> &'static str;
}

/// Abstract Syntax Tree representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AST {
    pub file_path: String,
    pub language: String,
    pub root_node: ASTNode,
    pub source_map: SourceMap,
}

impl AST {
    pub fn new(file_path: String, language: String, root_node: ASTNode) -> Self {
        Self {
            file_path,
            language,
            root_node,
            source_map: SourceMap::default(),
        }
    }

    /// Query nodes by pattern
    pub fn query_nodes(&self, pattern: &str) -> Vec<&ASTNode> {
        self.root_node.find_nodes_by_pattern(pattern)
    }

    /// Get all nodes of a specific type
    pub fn get_nodes_by_type(&self, node_type: &str) -> Vec<&ASTNode> {
        self.root_node.find_nodes_by_type(node_type)
    }
}

/// AST Node representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ASTNode {
    pub node_type: String,
    pub text: String,
    pub children: Vec<ASTNode>,
    pub location: Location,
    pub attributes: HashMap<String, String>,
}

impl ASTNode {
    pub fn new(node_type: String, text: String, location: Location) -> Self {
        Self {
            node_type,
            text,
            children: Vec::new(),
            location,
            attributes: HashMap::new(),
        }
    }

    pub fn add_child(&mut self, child: ASTNode) {
        self.children.push(child);
    }

    pub fn add_attribute(&mut self, key: String, value: String) {
        self.attributes.insert(key, value);
    }

    /// Find nodes matching a pattern
    pub fn find_nodes_by_pattern(&self, pattern: &str) -> Vec<&ASTNode> {
        let mut results = Vec::new();
        self.find_nodes_by_pattern_recursive(pattern, &mut results);
        results
    }

    /// Find nodes by type
    pub fn find_nodes_by_type(&self, node_type: &str) -> Vec<&ASTNode> {
        let mut results = Vec::new();
        self.find_nodes_by_type_recursive(node_type, &mut results);
        results
    }

    fn find_nodes_by_pattern_recursive(&self, pattern: &str, results: &mut Vec<&ASTNode>) {
        // Simple pattern matching - in production would use more sophisticated matching
        if self.node_type.contains(pattern) || self.text.contains(pattern) {
            results.push(self);
        }
        
        for child in &self.children {
            child.find_nodes_by_pattern_recursive(pattern, results);
        }
    }

    fn find_nodes_by_type_recursive(&self, node_type: &str, results: &mut Vec<&ASTNode>) {
        if self.node_type == node_type {
            results.push(self);
        }
        
        for child in &self.children {
            child.find_nodes_by_type_recursive(node_type, results);
        }
    }
}

/// Source location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub line: u32,
    pub column: u32,
    pub byte_offset: u32,
    pub length: u32,
}

impl Location {
    pub fn new(line: u32, column: u32, byte_offset: u32, length: u32) -> Self {
        Self { line, column, byte_offset, length }
    }
}

/// Source map for mapping AST nodes to source positions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SourceMap {
    pub line_starts: Vec<u32>,
    pub total_lines: u32,
}

/// Symbol extracted from code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Symbol {
    Function(FunctionSymbol),
    Class(ClassSymbol),
    Interface(InterfaceSymbol),
    Variable(VariableSymbol),
    Type(TypeSymbol),
    Constant(ConstantSymbol),
    Import(ImportSymbol),
    Export(ExportSymbol),
}

impl Symbol {
    pub fn name(&self) -> &str {
        match self {
            Symbol::Function(f) => &f.name,
            Symbol::Class(c) => &c.name,
            Symbol::Interface(i) => &i.name,
            Symbol::Variable(v) => &v.name,
            Symbol::Type(t) => &t.name,
            Symbol::Constant(c) => &c.name,
            Symbol::Import(i) => &i.name,
            Symbol::Export(e) => &e.name,
        }
    }

    pub fn location(&self) -> &Location {
        match self {
            Symbol::Function(f) => &f.location,
            Symbol::Class(c) => &c.location,
            Symbol::Interface(i) => &i.location,
            Symbol::Variable(v) => &v.location,
            Symbol::Type(t) => &t.location,
            Symbol::Constant(c) => &c.location,
            Symbol::Import(i) => &i.location,
            Symbol::Export(e) => &e.location,
        }
    }
}

/// Function symbol information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSymbol {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<String>,
    pub location: Location,
    pub visibility: Visibility,
    pub is_async: bool,
    pub is_generator: bool,
    pub complexity: u32,
    pub documentation: Option<String>,
}

/// Class symbol information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassSymbol {
    pub name: String,
    pub extends: Option<String>,
    pub implements: Vec<String>,
    pub methods: Vec<FunctionSymbol>,
    pub properties: Vec<PropertySymbol>,
    pub location: Location,
    pub visibility: Visibility,
    pub is_abstract: bool,
    pub documentation: Option<String>,
}

/// Interface symbol information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceSymbol {
    pub name: String,
    pub extends: Vec<String>,
    pub methods: Vec<MethodSignature>,
    pub properties: Vec<PropertySignature>,
    pub location: Location,
    pub documentation: Option<String>,
}

/// Variable symbol information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableSymbol {
    pub name: String,
    pub var_type: Option<String>,
    pub location: Location,
    pub visibility: Visibility,
    pub is_mutable: bool,
    pub initial_value: Option<String>,
}

/// Type symbol information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeSymbol {
    pub name: String,
    pub definition: String,
    pub location: Location,
    pub generic_parameters: Vec<String>,
    pub documentation: Option<String>,
}

/// Constant symbol information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstantSymbol {
    pub name: String,
    pub const_type: Option<String>,
    pub value: String,
    pub location: Location,
    pub visibility: Visibility,
}

/// Import symbol information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportSymbol {
    pub name: String,
    pub module_path: String,
    pub imported_items: Vec<String>,
    pub location: Location,
    pub is_default: bool,
    pub alias: Option<String>,
}

/// Export symbol information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSymbol {
    pub name: String,
    pub exported_type: ExportType,
    pub location: Location,
    pub is_default: bool,
}

/// Function parameter information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub param_type: Option<String>,
    pub is_optional: bool,
    pub default_value: Option<String>,
}

/// Property symbol information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertySymbol {
    pub name: String,
    pub property_type: Option<String>,
    pub location: Location,
    pub visibility: Visibility,
    pub is_readonly: bool,
    pub is_static: bool,
    pub initial_value: Option<String>,
}

/// Method signature information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodSignature {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<String>,
    pub location: Location,
}

/// Property signature information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertySignature {
    pub name: String,
    pub property_type: Option<String>,
    pub location: Location,
    pub is_readonly: bool,
}

/// Symbol visibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
    Package,
}

/// Export type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportType {
    Function,
    Class,
    Interface,
    Type,
    Variable,
    Constant,
}

/// Call graph representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallGraph {
    pub nodes: Vec<CallNode>,
    pub edges: Vec<CallEdge>,
}

impl CallGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: CallNode) {
        self.nodes.push(node);
    }

    pub fn add_edge(&mut self, edge: CallEdge) {
        self.edges.push(edge);
    }
}

/// Node in call graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallNode {
    pub id: String,
    pub name: String,
    pub node_type: CallNodeType,
    pub location: Location,
}

/// Edge in call graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallEdge {
    pub from: String,
    pub to: String,
    pub call_type: CallType,
    pub location: Location,
}

/// Type of call node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CallNodeType {
    Function,
    Method,
    Constructor,
    StaticMethod,
}

/// Type of function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CallType {
    Direct,
    Indirect,
    Virtual,
    Static,
}

/// Dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub source: String,
    pub target: String,
    pub dependency_type: DependencyType,
    pub location: Location,
}

/// Type of dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    Import,
    Inheritance,
    Composition,
    Aggregation,
    Usage,
}

/// Parser error types
#[derive(Debug, Clone)]
pub enum ParserError {
    SyntaxError { message: String, location: Location },
    UnsupportedLanguage(String),
    FileNotFound(String),
    IOError(String),
    TreeSitterError(String),
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserError::SyntaxError { message, location } => {
                write!(f, "Syntax error at {}:{}: {}", location.line, location.column, message)
            }
            ParserError::UnsupportedLanguage(lang) => write!(f, "Unsupported language: {}", lang),
            ParserError::FileNotFound(path) => write!(f, "File not found: {}", path),
            ParserError::IOError(msg) => write!(f, "IO error: {}", msg),
            ParserError::TreeSitterError(msg) => write!(f, "Tree-sitter error: {}", msg),
        }
    }
}

impl std::error::Error for ParserError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_node_creation() {
        let location = Location::new(1, 0, 0, 10);
        let node = ASTNode::new("function".to_string(), "test()".to_string(), location.clone());
        
        assert_eq!(node.node_type, "function");
        assert_eq!(node.text, "test()");
        assert_eq!(node.location.line, 1);
    }

    #[test]
    fn test_symbol_name() {
        let func_symbol = Symbol::Function(FunctionSymbol {
            name: "testFunction".to_string(),
            parameters: Vec::new(),
            return_type: None,
            location: Location::new(1, 0, 0, 10),
            visibility: Visibility::Public,
            is_async: false,
            is_generator: false,
            complexity: 1,
            documentation: None,
        });
        
        assert_eq!(func_symbol.name(), "testFunction");
    }

    #[test]
    fn test_call_graph_creation() {
        let mut graph = CallGraph::new();
        
        let node = CallNode {
            id: "func1".to_string(),
            name: "function1".to_string(),
            node_type: CallNodeType::Function,
            location: Location::new(1, 0, 0, 10),
        };
        
        graph.add_node(node);
        assert_eq!(graph.nodes.len(), 1);
    }
}
