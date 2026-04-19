//! Symbol extraction from AST nodes

use super::types::*;
use crate::error::Result;

/// Extracts semantic symbols from parsed AST nodes.
///
/// This extractor walks the simplified ASTNode tree (already populated by
/// tree-sitter via the language parsers) and converts nodes into typed
/// entity definitions.
pub struct SymbolExtractor {
    // Configuration and caches
}

impl SymbolExtractor {
    pub fn new() -> Self {
        Self {}
    }

    /// Extract all symbols from an AST
    pub fn extract_symbols(&self, ast: &super::analyzer::AST) -> Result<Vec<EntityDefinition>> {
        let mut symbols = Vec::new();
        self.extract_symbols_recursive(&ast.root, &mut symbols)?;
        Ok(symbols)
    }

    fn extract_symbols_recursive(
        &self,
        node: &super::analyzer::ASTNode,
        symbols: &mut Vec<EntityDefinition>,
    ) -> Result<()> {
        match node.node_type.as_str() {
            // TS/JS and Rust function-like nodes
            "function_declaration" | "function_item" | "function_definition"
            | "method_definition" | "method_declaration" | "arrow_function" => {
                if let Some(function) = self.extract_function(node)? {
                    symbols.push(EntityDefinition::Function(function));
                }
            }
            // Class / struct declarations
            "class_declaration" | "class_definition" | "struct_item" => {
                if let Some(class) = self.extract_class(node)? {
                    symbols.push(EntityDefinition::Class(class));
                }
            }
            // Interface / trait declarations
            "interface_declaration" | "trait_item" => {
                if let Some(interface) = self.extract_interface(node)? {
                    symbols.push(EntityDefinition::Interface(interface));
                }
            }
            // Type alias
            "type_alias_declaration" | "type_item" => {
                if let Some(type_def) = self.extract_type(node)? {
                    symbols.push(EntityDefinition::Type(type_def));
                }
            }
            // Variable / constant declarations
            "lexical_declaration" | "variable_declaration" | "const_item" | "static_item" => {
                if let Some(constant) = self.extract_constant(node)? {
                    symbols.push(EntityDefinition::Constant(constant));
                }
            }
            _ => {}
        }

        // Recurse into children
        for child in &node.children {
            self.extract_symbols_recursive(child, symbols)?;
        }

        Ok(())
    }

    /// Extract function definition from an ASTNode.
    ///
    /// The ASTNode contains the original source text and children populated by
    /// tree-sitter. We look for a child with node_type "identifier" or "name"
    /// to get the function name, "formal_parameters" / "parameters" for params, etc.
    fn extract_function(
        &self,
        node: &super::analyzer::ASTNode,
    ) -> Result<Option<FunctionDefinition>> {
        let name = self
            .find_name_child(node)
            .unwrap_or_else(|| "<anonymous>".to_string());

        let is_async = node
            .children
            .iter()
            .any(|c| c.node_type == "async" || c.text.starts_with("async"));

        let parameters = self.extract_parameters(node);

        let return_type = node
            .children
            .iter()
            .find(|c| {
                c.node_type == "type_annotation"
                    || c.node_type == "return_type"
                    || c.node_type.contains("return_type")
            })
            .map(|c| c.text.trim_start_matches(": ").to_string());

        let lines = node.location.end_line.saturating_sub(node.location.start_line) + 1;

        Ok(Some(FunctionDefinition {
            name,
            parameters,
            return_type,
            visibility: Visibility::Public,
            is_async,
            is_static: false,
            documentation: None,
            location: node.location.clone(),
            calls: Vec::new(),
            called_by: Vec::new(),
            complexity: 1,
            lines_of_code: lines,
        }))
    }

    /// Extract class/struct definition
    fn extract_class(&self, node: &super::analyzer::ASTNode) -> Result<Option<ClassDefinition>> {
        let name = self.find_name_child(node).unwrap_or_default();

        let extends = node
            .children
            .iter()
            .find(|c| {
                c.node_type == "class_heritage"
                    || c.node_type == "superclass"
                    || c.node_type == "extends_clause"
            })
            .and_then(|c| self.find_name_child(c));

        // Collect method names from body
        let methods: Vec<String> = node
            .children
            .iter()
            .filter(|c| {
                c.node_type == "class_body"
                    || c.node_type == "declaration_list"
                    || c.node_type == "field_declaration_list"
            })
            .flat_map(|body| body.children.iter())
            .filter(|c| {
                c.node_type == "method_definition"
                    || c.node_type == "function_item"
                    || c.node_type == "method_declaration"
            })
            .filter_map(|m| self.find_name_child(m))
            .collect();

        Ok(Some(ClassDefinition {
            name,
            extends,
            implements: Vec::new(),
            properties: Vec::new(),
            methods,
            visibility: Visibility::Public,
            is_abstract: node
                .children
                .iter()
                .any(|c| c.node_type == "abstract" || c.text == "abstract"),
            is_static: false,
            documentation: None,
            location: node.location.clone(),
            design_patterns: Vec::new(),
        }))
    }

    /// Extract interface/trait definition
    fn extract_interface(
        &self,
        node: &super::analyzer::ASTNode,
    ) -> Result<Option<InterfaceDefinition>> {
        let name = self.find_name_child(node).unwrap_or_default();

        // Extract method signatures from the body
        let methods: Vec<MethodSignature> = node
            .children
            .iter()
            .filter(|c| c.node_type == "object_type" || c.node_type == "declaration_list")
            .flat_map(|body| body.children.iter())
            .filter(|c| {
                c.node_type == "method_signature"
                    || c.node_type == "function_item"
                    || c.node_type == "method_definition"
            })
            .filter_map(|m| {
                let mname = self.find_name_child(m)?;
                Some(MethodSignature {
                    name: mname,
                    parameters: self.extract_parameters(m),
                    return_type: None,
                    is_async: false,
                    documentation: None,
                })
            })
            .collect();

        // Extract property signatures
        let properties: Vec<PropertySignature> = node
            .children
            .iter()
            .filter(|c| c.node_type == "object_type" || c.node_type == "declaration_list")
            .flat_map(|body| body.children.iter())
            .filter(|c| c.node_type == "property_signature")
            .filter_map(|p| {
                let pname = self.find_name_child(p)?;
                Some(PropertySignature {
                    name: pname,
                    prop_type: String::new(),
                    is_optional: p.children.iter().any(|c| c.text == "?"),
                    is_readonly: p
                        .children
                        .iter()
                        .any(|c| c.node_type == "readonly" || c.text == "readonly"),
                    documentation: None,
                })
            })
            .collect();

        Ok(Some(InterfaceDefinition {
            name,
            extends: Vec::new(),
            methods,
            properties,
            visibility: Visibility::Public,
            documentation: None,
            location: node.location.clone(),
        }))
    }

    /// Extract type alias
    fn extract_type(&self, node: &super::analyzer::ASTNode) -> Result<Option<TypeDefinition>> {
        let name = self.find_name_child(node).unwrap_or_default();

        // The definition text is usually the child after "="
        let definition = node
            .children
            .iter()
            .find(|c| {
                c.node_type != "identifier"
                    && c.node_type != "type_identifier"
                    && c.node_type != "type"
                    && c.node_type != "type_parameters"
                    && !c.text.is_empty()
            })
            .map(|c| c.text.clone())
            .unwrap_or_default();

        // Detect type kind from definition shape
        let type_kind = if definition.contains('|') {
            TypeKind::Union
        } else if definition.contains('&') {
            TypeKind::Intersection
        } else if definition.contains("enum") {
            TypeKind::Enum
        } else {
            TypeKind::Alias
        };

        Ok(Some(TypeDefinition {
            name,
            type_kind,
            definition,
            generic_parameters: Vec::new(),
            visibility: Visibility::Public,
            documentation: None,
            location: node.location.clone(),
        }))
    }

    /// Extract constant / variable declaration
    fn extract_constant(
        &self,
        node: &super::analyzer::ASTNode,
    ) -> Result<Option<ConstantDefinition>> {
        let name = self.find_name_child(node).unwrap_or_default();
        if name.is_empty() {
            return Ok(None);
        }

        let value = node
            .children
            .iter()
            .find(|c| {
                c.node_type != "identifier"
                    && c.node_type != "type_identifier"
                    && c.node_type != "const"
                    && c.node_type != "let"
                    && c.node_type != "var"
                    && !c.text.is_empty()
            })
            .map(|c| c.text.clone())
            .unwrap_or_default();

        let const_type = node
            .children
            .iter()
            .find(|c| c.node_type == "type_annotation")
            .map(|c| c.text.trim_start_matches(": ").to_string())
            .unwrap_or_else(|| "unknown".to_string());

        Ok(Some(ConstantDefinition {
            name,
            value,
            const_type,
            visibility: Visibility::Public,
            documentation: None,
            location: node.location.clone(),
        }))
    }

    // ── Helpers ────────────────────────────────────────────────────────────────

    /// Find the name of a node by searching its children for an identifier
    fn find_name_child(&self, node: &super::analyzer::ASTNode) -> Option<String> {
        // Prefer child named "name" by tree-sitter field names; since our
        // ASTNode only stores kind, we match on identifier-like node types.
        for child in &node.children {
            match child.node_type.as_str() {
                "identifier" | "type_identifier" | "property_identifier"
                | "shorthand_property_identifier" => {
                    let text = child.text.trim().to_string();
                    if !text.is_empty() {
                        return Some(text);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// Extract parameters from a function-like ASTNode
    fn extract_parameters(&self, node: &super::analyzer::ASTNode) -> Vec<Parameter> {
        node.children
            .iter()
            .filter(|c| {
                c.node_type == "formal_parameters"
                    || c.node_type == "parameters"
                    || c.node_type == "parameter_list"
            })
            .flat_map(|params| params.children.iter())
            .filter(|p| {
                p.node_type == "required_parameter"
                    || p.node_type == "optional_parameter"
                    || p.node_type == "parameter"
                    || p.node_type == "identifier"
            })
            .map(|p| {
                let pname = self
                    .find_name_child(p)
                    .unwrap_or_else(|| p.text.trim().to_string());
                let ptype = p
                    .children
                    .iter()
                    .find(|c| c.node_type == "type_annotation")
                    .map(|c| c.text.trim_start_matches(": ").to_string())
                    .unwrap_or_default();

                Parameter {
                    name: pname,
                    param_type: ptype,
                    is_optional: p.node_type == "optional_parameter",
                    default_value: p
                        .children
                        .iter()
                        .find(|c| c.node_type == "default_value")
                        .map(|c| c.text.clone()),
                    documentation: None,
                }
            })
            .collect()
    }
}

impl Default for SymbolExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::analyzer::ASTNode;
    use super::super::types::CodeLocation;

    fn make_ast_node(kind: &str, text: &str, children: Vec<ASTNode>) -> ASTNode {
        ASTNode {
            node_type: kind.to_string(),
            text: text.to_string(),
            children,
            location: CodeLocation {
                file_path: std::path::PathBuf::from("test.ts"),
                start_line: 1,
                end_line: 1,
                start_column: 0,
                end_column: text.len() as u32,
            },
        }
    }

    #[test]
    fn test_extract_function_with_name() {
        let extractor = SymbolExtractor::new();
        let node = make_ast_node(
            "function_declaration",
            "function greet(name: string): string {}",
            vec![
                make_ast_node("identifier", "greet", vec![]),
                make_ast_node("formal_parameters", "(name: string)", vec![
                    make_ast_node("required_parameter", "name: string", vec![]),
                ]),
            ],
        );
        let def = extractor.extract_function(&node).unwrap().unwrap();
        assert_eq!(def.name, "greet");
        assert_eq!(def.parameters.len(), 1);
    }

    #[test]
    fn test_extract_class_with_methods() {
        let extractor = SymbolExtractor::new();
        let node = make_ast_node(
            "class_declaration",
            "class UserService { addUser() {} getUser() {} }",
            vec![
                make_ast_node("type_identifier", "UserService", vec![]),
                make_ast_node("class_body", "{ addUser() {} getUser() {} }", vec![
                    make_ast_node("method_definition", "addUser() {}", vec![
                        make_ast_node("property_identifier", "addUser", vec![]),
                    ]),
                    make_ast_node("method_definition", "getUser() {}", vec![
                        make_ast_node("property_identifier", "getUser", vec![]),
                    ]),
                ]),
            ],
        );
        let def = extractor.extract_class(&node).unwrap().unwrap();
        assert_eq!(def.name, "UserService");
        assert_eq!(def.methods.len(), 2);
    }

    #[test]
    fn test_extract_interface() {
        let extractor = SymbolExtractor::new();
        let node = make_ast_node(
            "interface_declaration",
            "interface Serializable { serialize(): string; }",
            vec![
                make_ast_node("type_identifier", "Serializable", vec![]),
                make_ast_node("object_type", "{ serialize(): string; }", vec![
                    make_ast_node("method_signature", "serialize(): string", vec![
                        make_ast_node("identifier", "serialize", vec![]),
                    ]),
                ]),
            ],
        );
        let def = extractor.extract_interface(&node).unwrap().unwrap();
        assert_eq!(def.name, "Serializable");
        assert!(!def.methods.is_empty());
    }

    #[test]
    fn test_extract_constant() {
        let extractor = SymbolExtractor::new();
        let node = make_ast_node(
            "lexical_declaration",
            "const MAX_SIZE = 100",
            vec![
                make_ast_node("identifier", "MAX_SIZE", vec![]),
                make_ast_node("number", "100", vec![]),
            ],
        );
        let def = extractor.extract_constant(&node).unwrap().unwrap();
        assert_eq!(def.name, "MAX_SIZE");
    }

    #[test]
    fn test_extract_type_union() {
        let extractor = SymbolExtractor::new();
        let node = make_ast_node(
            "type_alias_declaration",
            "type Result = Success | Error",
            vec![
                make_ast_node("type_identifier", "Result", vec![]),
                make_ast_node("union_type", "Success | Error", vec![]),
            ],
        );
        let def = extractor.extract_type(&node).unwrap().unwrap();
        assert_eq!(def.name, "Result");
    }

    #[test]
    fn test_find_name_child_checks_identifiers() {
        let extractor = SymbolExtractor::new();
        let node = make_ast_node("function_declaration", "fn process()", vec![
            make_ast_node("identifier", "process", vec![]),
        ]);
        let name = extractor.find_name_child(&node);
        assert_eq!(name, Some("process".to_string()));
    }
}
