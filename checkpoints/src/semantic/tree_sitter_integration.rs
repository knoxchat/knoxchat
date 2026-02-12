//! Tree-sitter integration for real AST parsing
//!
//! This module provides concrete tree-sitter parser implementations for multiple languages,
//! enabling deep syntactic and semantic analysis of code.

use super::analyzer::AST;
use super::ast_parser::LanguageParser;
use super::types::*;
use crate::error::{CheckpointError, Result};
use std::collections::HashMap;
use std::path::Path;

// Re-export tree-sitter types
pub use tree_sitter::{Language, Node, Parser, Query, QueryCursor, Tree};

/// Tree-sitter based language parser
pub struct TreeSitterParser {
    parser: Parser,
    language: Language,
    language_name: String,
    query_patterns: QueryPatterns,
}

/// Query patterns for extracting semantic information
struct QueryPatterns {
    function_query: Option<String>,
    class_query: Option<String>,
    import_query: Option<String>,
    export_query: Option<String>,
    call_query: Option<String>,
}

impl TreeSitterParser {
    /// Create a new tree-sitter parser for a specific language
    pub fn new(language: Language, language_name: &str) -> Result<Self> {
        let mut parser = Parser::new();
        parser
            .set_language(language)
            .map_err(|e| CheckpointError::parsing(format!("Failed to set language: {}", e)))?;

        let query_patterns = Self::get_query_patterns(language_name);

        Ok(Self {
            parser,
            language,
            language_name: language_name.to_string(),
            query_patterns,
        })
    }

    /// Get language-specific query patterns
    fn get_query_patterns(language: &str) -> QueryPatterns {
        match language {
            "typescript" | "tsx" => QueryPatterns {
                function_query: Some(
                    r#"
                    (function_declaration
                        name: (identifier) @function.name
                        parameters: (formal_parameters) @function.params
                        return_type: (type_annotation)? @function.return_type
                        body: (statement_block) @function.body)
                    
                    (method_definition
                        name: (property_identifier) @method.name
                        parameters: (formal_parameters) @method.params
                        return_type: (type_annotation)? @method.return_type
                        body: (statement_block) @method.body)
                    
                    (arrow_function
                        parameters: (formal_parameters) @arrow.params
                        return_type: (type_annotation)? @arrow.return_type
                        body: (_) @arrow.body)
                    "#
                    .to_string(),
                ),
                class_query: Some(
                    r#"
                    (class_declaration
                        name: (type_identifier) @class.name
                        type_parameters: (type_parameters)? @class.type_params
                        superclass: (extends_clause)? @class.extends
                        implements: (implements_clause)? @class.implements
                        body: (class_body) @class.body)
                    
                    (interface_declaration
                        name: (type_identifier) @interface.name
                        type_parameters: (type_parameters)? @interface.type_params
                        extends: (extends_clause)? @interface.extends
                        body: (interface_body) @interface.body)
                    "#
                    .to_string(),
                ),
                import_query: Some(
                    r#"
                    (import_statement
                        source: (string) @import.source
                        (import_clause)? @import.clause)
                    "#
                    .to_string(),
                ),
                export_query: Some(
                    r#"
                    (export_statement) @export
                    "#
                    .to_string(),
                ),
                call_query: Some(
                    r#"
                    (call_expression
                        function: (_) @call.function
                        arguments: (arguments) @call.arguments)
                    "#
                    .to_string(),
                ),
            },
            "rust" => QueryPatterns {
                function_query: Some(
                    r#"
                    (function_item
                        name: (identifier) @function.name
                        parameters: (parameters) @function.params
                        return_type: (type)? @function.return_type
                        body: (block) @function.body)
                    
                    (impl_item
                        type: (type_identifier) @impl.type
                        body: (declaration_list) @impl.body)
                    "#
                    .to_string(),
                ),
                class_query: Some(
                    r#"
                    (struct_item
                        name: (type_identifier) @struct.name
                        type_parameters: (type_parameters)? @struct.type_params
                        body: (_) @struct.body)
                    
                    (enum_item
                        name: (type_identifier) @enum.name
                        type_parameters: (type_parameters)? @enum.type_params
                        body: (enum_variant_list) @enum.body)
                    
                    (trait_item
                        name: (type_identifier) @trait.name
                        type_parameters: (type_parameters)? @trait.type_params
                        body: (declaration_list) @trait.body)
                    "#
                    .to_string(),
                ),
                import_query: Some(
                    r#"
                    (use_declaration) @import
                    "#
                    .to_string(),
                ),
                export_query: Some(
                    r#"
                    (visibility_modifier) @export
                    "#
                    .to_string(),
                ),
                call_query: Some(
                    r#"
                    (call_expression
                        function: (_) @call.function
                        arguments: (arguments) @call.arguments)
                    "#
                    .to_string(),
                ),
            },
            "python" => QueryPatterns {
                function_query: Some(
                    r#"
                    (function_definition
                        name: (identifier) @function.name
                        parameters: (parameters) @function.params
                        return_type: (type)? @function.return_type
                        body: (block) @function.body)
                    "#
                    .to_string(),
                ),
                class_query: Some(
                    r#"
                    (class_definition
                        name: (identifier) @class.name
                        superclasses: (argument_list)? @class.bases
                        body: (block) @class.body)
                    "#
                    .to_string(),
                ),
                import_query: Some(
                    r#"
                    (import_statement) @import
                    (import_from_statement) @import.from
                    "#
                    .to_string(),
                ),
                export_query: None,
                call_query: Some(
                    r#"
                    (call
                        function: (_) @call.function
                        arguments: (argument_list) @call.arguments)
                    "#
                    .to_string(),
                ),
            },
            _ => QueryPatterns {
                function_query: None,
                class_query: None,
                import_query: None,
                export_query: None,
                call_query: None,
            },
        }
    }

    /// Parse source code into tree
    pub fn parse_str(&mut self, source: &str) -> Result<Tree> {
        self.parser
            .parse(source, None)
            .ok_or_else(|| CheckpointError::parsing("Failed to parse source code".to_string()))
    }

    /// Extract all functions from the tree
    fn extract_functions(&self, tree: &Tree, source: &str) -> Result<Vec<EntityDefinition>> {
        let mut functions = Vec::new();

        if let Some(query_str) = &self.query_patterns.function_query {
            if let Ok(query) = Query::new(self.language, query_str) {
                let mut cursor = QueryCursor::new();
                let matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

                for match_ in matches {
                    let mut function = EntityDefinition {
                        name: String::new(),
                        entity_type: EntityType::Function,
                        location: CodeLocation {
                            file_path: String::new(),
                            start_line: 0,
                            start_column: 0,
                            end_line: 0,
                            end_column: 0,
                        },
                        visibility: "public".to_string(),
                        documentation: None,
                        metadata: HashMap::new(),
                    };

                    for capture in match_.captures {
                        let node = capture.node;
                        let capture_name = query.capture_names()[capture.index as usize];

                        if capture_name.contains("name") {
                            function.name =
                                node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                        }

                        function.location = CodeLocation {
                            file_path: String::new(),
                            start_line: node.start_position().row + 1,
                            start_column: node.start_position().column + 1,
                            end_line: node.end_position().row + 1,
                            end_column: node.end_position().column + 1,
                        };
                    }

                    if !function.name.is_empty() {
                        functions.push(function);
                    }
                }
            }
        }

        Ok(functions)
    }

    /// Extract all classes/structs from the tree
    fn extract_classes(&self, tree: &Tree, source: &str) -> Result<Vec<EntityDefinition>> {
        let mut classes = Vec::new();

        if let Some(query_str) = &self.query_patterns.class_query {
            if let Ok(query) = Query::new(self.language, query_str) {
                let mut cursor = QueryCursor::new();
                let matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

                for match_ in matches {
                    let mut class = EntityDefinition {
                        name: String::new(),
                        entity_type: EntityType::Class,
                        location: CodeLocation {
                            file_path: String::new(),
                            start_line: 0,
                            start_column: 0,
                            end_line: 0,
                            end_column: 0,
                        },
                        visibility: "public".to_string(),
                        documentation: None,
                        metadata: HashMap::new(),
                    };

                    for capture in match_.captures {
                        let node = capture.node;
                        let capture_name = query.capture_names()[capture.index as usize];

                        if capture_name.contains("name") {
                            class.name =
                                node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                        }

                        class.location = CodeLocation {
                            file_path: String::new(),
                            start_line: node.start_position().row + 1,
                            start_column: node.start_position().column + 1,
                            end_line: node.end_position().row + 1,
                            end_column: node.end_position().column + 1,
                        };
                    }

                    if !class.name.is_empty() {
                        classes.push(class);
                    }
                }
            }
        }

        Ok(classes)
    }

    /// Extract imports from the tree
    fn extract_imports_from_tree(&self, tree: &Tree, source: &str) -> Result<Vec<ImportStatement>> {
        let mut imports = Vec::new();

        if let Some(query_str) = &self.query_patterns.import_query {
            if let Ok(query) = Query::new(self.language, query_str) {
                let mut cursor = QueryCursor::new();
                let matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

                for match_ in matches {
                    for capture in match_.captures {
                        let node = capture.node;
                        let text = node.utf8_text(source.as_bytes()).unwrap_or("");

                        imports.push(ImportStatement {
                            module_name: text.to_string(),
                            imported_items: vec![],
                            import_type: ImportType::Named,
                            location: CodeLocation {
                                file_path: String::new(),
                                start_line: node.start_position().row + 1,
                                start_column: node.start_position().column + 1,
                                end_line: node.end_position().row + 1,
                                end_column: node.end_position().column + 1,
                            },
                        });
                    }
                }
            }
        }

        Ok(imports)
    }

    /// Extract exports from the tree
    fn extract_exports_from_tree(&self, tree: &Tree, source: &str) -> Result<Vec<ExportStatement>> {
        let mut exports = Vec::new();

        if let Some(query_str) = &self.query_patterns.export_query {
            if let Ok(query) = Query::new(self.language, query_str) {
                let mut cursor = QueryCursor::new();
                let matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

                for match_ in matches {
                    for capture in match_.captures {
                        let node = capture.node;

                        exports.push(ExportStatement {
                            exported_items: vec![],
                            export_type: ExportType::Named,
                            location: CodeLocation {
                                file_path: String::new(),
                                start_line: node.start_position().row + 1,
                                start_column: node.start_position().column + 1,
                                end_line: node.end_position().row + 1,
                                end_column: node.end_position().column + 1,
                            },
                        });
                    }
                }
            }
        }

        Ok(exports)
    }

    /// Build call graph from the tree
    fn build_call_graph_from_tree(&self, tree: &Tree, source: &str) -> Result<Vec<CallChain>> {
        let mut call_chains = Vec::new();

        if let Some(query_str) = &self.query_patterns.call_query {
            if let Ok(query) = Query::new(self.language, query_str) {
                let mut cursor = QueryCursor::new();
                let matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

                for match_ in matches {
                    for capture in match_.captures {
                        let node = capture.node;
                        let capture_name = query.capture_names()[capture.index as usize];

                        if capture_name.contains("function") {
                            let function_name =
                                node.utf8_text(source.as_bytes()).unwrap_or("").to_string();

                            call_chains.push(CallChain {
                                caller: String::new(), // Would need context to determine
                                called: function_name,
                                call_type: CallType::Direct,
                                location: CodeLocation {
                                    file_path: String::new(),
                                    start_line: node.start_position().row + 1,
                                    start_column: node.start_position().column + 1,
                                    end_line: node.end_position().row + 1,
                                    end_column: node.end_position().column + 1,
                                },
                                context: None,
                            });
                        }
                    }
                }
            }
        }

        Ok(call_chains)
    }
}

impl LanguageParser for TreeSitterParser {
    fn parse_file(&self, content: &str, file_path: &Path) -> Result<AST> {
        let mut parser = Parser::new();
        parser
            .set_language(self.language)
            .map_err(|e| CheckpointError::parsing(format!("Failed to set language: {}", e)))?;

        let tree = parser
            .parse(content, None)
            .ok_or_else(|| CheckpointError::parsing("Failed to parse file".to_string()))?;

        Ok(AST {
            root: super::analyzer::ASTNode {
                node_type: "root".to_string(),
                text: content.to_string(),
                children: Vec::new(),
                location: CodeLocation {
                    file_path: file_path.to_string_lossy().to_string(),
                    start_line: 1,
                    start_column: 1,
                    end_line: content.lines().count(),
                    end_column: content.lines().last().map(|l| l.len()).unwrap_or(0),
                },
            },
            file_path: file_path.to_path_buf(),
        })
    }

    fn extract_symbols(&self, _ast: &AST) -> Result<Vec<EntityDefinition>> {
        // This would use the tree from parse_file
        Ok(Vec::new())
    }

    fn extract_imports(&self, _ast: &AST) -> Result<Vec<ImportStatement>> {
        Ok(Vec::new())
    }

    fn extract_exports(&self, _ast: &AST) -> Result<Vec<ExportStatement>> {
        Ok(Vec::new())
    }

    fn build_call_graph(&self, _ast: &AST) -> Result<Vec<CallChain>> {
        Ok(Vec::new())
    }
}

/// Factory for creating language-specific parsers
pub struct TreeSitterParserFactory;

impl TreeSitterParserFactory {
    /// Create a parser for the given language
    pub fn create_parser(language: &str) -> Result<Box<dyn LanguageParser + Send + Sync>> {
        match language {
            "typescript" | "tsx" => Ok(Box::new(TreeSitterParser::new(
                tree_sitter_typescript::language_typescript(),
                "typescript",
            )?)),
            "javascript" | "jsx" => Ok(Box::new(TreeSitterParser::new(
                tree_sitter_javascript::language(),
                "javascript",
            )?)),
            "rust" => Ok(Box::new(TreeSitterParser::new(
                tree_sitter_rust::language(),
                "rust",
            )?)),
            "python" => Ok(Box::new(TreeSitterParser::new(
                tree_sitter_python::language(),
                "python",
            )?)),
            _ => Err(CheckpointError::validation(format!(
                "Unsupported language: {}",
                language
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typescript_parser() {
        let parser = TreeSitterParserFactory::create_parser("typescript");
        assert!(parser.is_ok());
    }

    #[test]
    fn test_rust_parser() {
        let parser = TreeSitterParserFactory::create_parser("rust");
        assert!(parser.is_ok());
    }

    #[test]
    fn test_python_parser() {
        let parser = TreeSitterParserFactory::create_parser("python");
        assert!(parser.is_ok());
    }
}
