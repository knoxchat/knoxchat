//! AST parsing interfaces and implementations

use super::types::*;
use crate::error::Result;
use std::path::Path;

/// Main AST parser that coordinates language-specific parsers
pub struct ASTParser {
    // Configuration and shared resources
}

impl ASTParser {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
}

/// Trait for language-specific parsers
pub trait LanguageParser {
    /// Parse file content into an AST
    fn parse_file(&self, content: &str, file_path: &Path) -> Result<super::analyzer::AST>;

    /// Extract semantic symbols from AST
    fn extract_symbols(&self, ast: &super::analyzer::AST) -> Result<Vec<EntityDefinition>>;

    /// Extract import statements
    fn extract_imports(&self, ast: &super::analyzer::AST) -> Result<Vec<ImportStatement>>;

    /// Extract export statements
    fn extract_exports(&self, ast: &super::analyzer::AST) -> Result<Vec<ExportStatement>>;

    /// Build call graph from AST
    fn build_call_graph(&self, _ast: &super::analyzer::AST) -> Result<Vec<CallChain>> {
        // Default implementation - can be overridden
        Ok(Vec::new())
    }

    /// Analyze dependencies from AST
    fn analyze_dependencies(&self, _ast: &super::analyzer::AST) -> Result<Vec<DependencyEdge>> {
        // Default implementation - can be overridden
        Ok(Vec::new())
    }
}
