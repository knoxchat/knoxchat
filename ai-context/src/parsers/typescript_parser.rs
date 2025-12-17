/**
 * TypeScript/JavaScript Parser Implementation
 * 
 * This module implements the LanguageParser trait for TypeScript and JavaScript
 * using Tree-sitter for accurate AST parsing.
 */

use super::language_parser::*;
use std::path::Path;
use std::collections::HashMap;

/// TypeScript/JavaScript parser implementation
pub struct TypeScriptParser {
    // In a real implementation, this would use tree-sitter-typescript
    // For now, we'll implement a simplified parser
}

impl TypeScriptParser {
    pub fn new() -> Self {
        Self {}
    }

    /// Extract function parameters from AST node
    fn extract_function_parameters(&self, node: &ASTNode) -> Vec<Parameter> {
        let mut parameters = Vec::new();
        
        // Look for parameter list in function node
        for child in &node.children {
            if child.node_type == "parameters" {
                for param_node in &child.children {
                    if param_node.node_type == "parameter" {
                        let param = self.parse_parameter(param_node);
                        parameters.push(param);
                    }
                }
                break;
            }
        }
        
        parameters
    }

    /// Parse a parameter node
    fn parse_parameter(&self, node: &ASTNode) -> Parameter {
        let mut name = String::new();
        let mut param_type = None;
        let mut is_optional = false;
        let mut default_value = None;

        for child in &node.children {
            match child.node_type.as_str() {
                "identifier" => name = child.text.clone(),
                "type_annotation" => param_type = Some(self.extract_type_from_annotation(child)),
                "optional_parameter" => is_optional = true,
                "default_parameter" => default_value = Some(child.text.clone()),
                _ => {}
            }
        }

        Parameter {
            name,
            param_type,
            is_optional,
            default_value,
        }
    }

    /// Extract return type from function node
    fn extract_return_type(&self, node: &ASTNode) -> Option<String> {
        for child in &node.children {
            if child.node_type == "type_annotation" {
                return Some(self.extract_type_from_annotation(child));
            }
        }
        None
    }

    /// Extract type from type annotation
    fn extract_type_from_annotation(&self, node: &ASTNode) -> String {
        // Simplified type extraction
        for child in &node.children {
            if child.node_type != ":" {
                return child.text.clone();
            }
        }
        "unknown".to_string()
    }

    /// Extract visibility from modifiers
    fn extract_visibility(&self, node: &ASTNode) -> Visibility {
        for child in &node.children {
            match child.text.as_str() {
                "private" => return Visibility::Private,
                "protected" => return Visibility::Protected,
                "public" => return Visibility::Public,
                _ => {}
            }
        }
        Visibility::Public // Default in TypeScript
    }

    /// Check if function is async
    fn is_async_function(&self, node: &ASTNode) -> bool {
        for child in &node.children {
            if child.text == "async" {
                return true;
            }
        }
        false
    }

    /// Check if function is generator
    fn is_generator_function(&self, node: &ASTNode) -> bool {
        node.text.contains("function*") || node.text.contains("*")
    }

    /// Calculate cyclomatic complexity (simplified)
    fn calculate_complexity(&self, node: &ASTNode) -> u32 {
        let mut complexity = 1; // Base complexity
        
        self.calculate_complexity_recursive(node, &mut complexity);
        complexity
    }

    fn calculate_complexity_recursive(&self, node: &ASTNode, complexity: &mut u32) {
        match node.node_type.as_str() {
            "if_statement" | "while_statement" | "for_statement" | 
            "switch_statement" | "catch_clause" | "conditional_expression" => {
                *complexity += 1;
            }
            "case_clause" => *complexity += 1,
            "logical_expression" if node.text.contains("&&") || node.text.contains("||") => {
                *complexity += 1;
            }
            _ => {}
        }

        for child in &node.children {
            self.calculate_complexity_recursive(child, complexity);
        }
    }

    /// Extract documentation from comments
    fn extract_documentation(&self, node: &ASTNode) -> Option<String> {
        // Look for JSDoc comments or regular comments above the node
        // This is a simplified implementation
        if node.text.contains("/**") {
            // Extract JSDoc comment
            let lines: Vec<&str> = node.text.lines().collect();
            let mut doc_lines = Vec::new();
            let mut in_jsdoc = false;

            for line in lines {
                let trimmed = line.trim();
                if trimmed.starts_with("/**") {
                    in_jsdoc = true;
                    continue;
                }
                if trimmed.ends_with("*/") {
                    break;
                }
                if in_jsdoc && trimmed.starts_with("*") {
                    doc_lines.push(trimmed.trim_start_matches("*").trim());
                }
            }

            if !doc_lines.is_empty() {
                return Some(doc_lines.join("\n"));
            }
        }
        None
    }

    /// Parse class methods
    fn parse_class_methods(&self, class_node: &ASTNode) -> Vec<FunctionSymbol> {
        let mut methods = Vec::new();

        for child in &class_node.children {
            if child.node_type == "method_definition" || child.node_type == "method_signature" {
                if let Some(method) = self.parse_method(child) {
                    methods.push(method);
                }
            }
        }

        methods
    }

    /// Parse a method node
    fn parse_method(&self, node: &ASTNode) -> Option<FunctionSymbol> {
        let mut name = String::new();
        let mut location = node.location.clone();

        // Extract method name
        for child in &node.children {
            if child.node_type == "property_name" || child.node_type == "identifier" {
                name = child.text.clone();
                break;
            }
        }

        if name.is_empty() {
            return None;
        }

        Some(FunctionSymbol {
            name,
            parameters: self.extract_function_parameters(node),
            return_type: self.extract_return_type(node),
            location,
            visibility: self.extract_visibility(node),
            is_async: self.is_async_function(node),
            is_generator: self.is_generator_function(node),
            complexity: self.calculate_complexity(node),
            documentation: self.extract_documentation(node),
        })
    }

    /// Parse class properties
    fn parse_class_properties(&self, class_node: &ASTNode) -> Vec<PropertySymbol> {
        let mut properties = Vec::new();

        for child in &class_node.children {
            if child.node_type == "property_definition" || child.node_type == "property_signature" {
                if let Some(property) = self.parse_property(child) {
                    properties.push(property);
                }
            }
        }

        properties
    }

    /// Parse a property node
    fn parse_property(&self, node: &ASTNode) -> Option<PropertySymbol> {
        let mut name = String::new();
        let mut property_type = None;
        let mut initial_value = None;
        let mut is_readonly = false;
        let mut is_static = false;

        // Extract property information
        for child in &node.children {
            match child.node_type.as_str() {
                "property_name" | "identifier" => name = child.text.clone(),
                "type_annotation" => property_type = Some(self.extract_type_from_annotation(child)),
                "initializer" => initial_value = Some(child.text.clone()),
                "readonly" => is_readonly = true,
                "static" => is_static = true,
                _ => {}
            }
        }

        if name.is_empty() {
            return None;
        }

        Some(PropertySymbol {
            name,
            property_type,
            location: node.location.clone(),
            visibility: self.extract_visibility(node),
            is_readonly,
            is_static,
            initial_value,
        })
    }

    /// Build call graph from function calls
    fn build_call_graph_from_ast(&self, ast: &AST) -> CallGraph {
        let mut graph = CallGraph::new();
        let mut node_id_counter = 0;
        let mut function_nodes = HashMap::new();

        // First pass: collect all function definitions
        self.collect_function_nodes(ast, &mut graph, &mut function_nodes, &mut node_id_counter);

        // Second pass: find function calls and create edges
        self.collect_function_calls(ast, &mut graph, &function_nodes);

        graph
    }

    fn collect_function_nodes(
        &self,
        ast: &AST,
        graph: &mut CallGraph,
        function_nodes: &mut HashMap<String, String>,
        node_id_counter: &mut u32,
    ) {
        self.collect_function_nodes_recursive(&ast.root_node, graph, function_nodes, node_id_counter);
    }

    fn collect_function_nodes_recursive(
        &self,
        node: &ASTNode,
        graph: &mut CallGraph,
        function_nodes: &mut HashMap<String, String>,
        node_id_counter: &mut u32,
    ) {
        if node.node_type == "function_declaration" || node.node_type == "method_definition" {
            if let Some(name) = self.extract_function_name(node) {
                let node_id = format!("node_{}", node_id_counter);
                *node_id_counter += 1;

                let call_node = CallNode {
                    id: node_id.clone(),
                    name: name.clone(),
                    node_type: if node.node_type == "method_definition" {
                        CallNodeType::Method
                    } else {
                        CallNodeType::Function
                    },
                    location: node.location.clone(),
                };

                graph.add_node(call_node);
                function_nodes.insert(name, node_id);
            }
        }

        for child in &node.children {
            self.collect_function_nodes_recursive(child, graph, function_nodes, node_id_counter);
        }
    }

    fn collect_function_calls(&self, ast: &AST, graph: &mut CallGraph, function_nodes: &HashMap<String, String>) {
        self.collect_function_calls_recursive(&ast.root_node, graph, function_nodes, None);
    }

    fn collect_function_calls_recursive(
        &self,
        node: &ASTNode,
        graph: &mut CallGraph,
        function_nodes: &HashMap<String, String>,
        current_function: Option<&str>,
    ) {
        let mut current_func = current_function;

        // Update current function if we're in a function
        if node.node_type == "function_declaration" || node.node_type == "method_definition" {
            if let Some(name) = self.extract_function_name(node) {
                current_func = Some(&name);
            }
        }

        // Look for function calls
        if node.node_type == "call_expression" {
            if let Some(called_function) = self.extract_called_function_name(node) {
                if let (Some(from), Some(to)) = (current_func, function_nodes.get(&called_function)) {
                    if let Some(from_id) = function_nodes.get(from) {
                        let edge = CallEdge {
                            from: from_id.clone(),
                            to: to.clone(),
                            call_type: CallType::Direct,
                            location: node.location.clone(),
                        };
                        graph.add_edge(edge);
                    }
                }
            }
        }

        for child in &node.children {
            self.collect_function_calls_recursive(child, graph, function_nodes, current_func);
        }
    }

    fn extract_function_name(&self, node: &ASTNode) -> Option<String> {
        for child in &node.children {
            if child.node_type == "identifier" || child.node_type == "property_name" {
                return Some(child.text.clone());
            }
        }
        None
    }

    fn extract_called_function_name(&self, node: &ASTNode) -> Option<String> {
        for child in &node.children {
            if child.node_type == "identifier" || child.node_type == "member_expression" {
                return Some(child.text.clone());
            }
        }
        None
    }

    /// Analyze imports and dependencies
    fn analyze_dependencies_from_ast(&self, ast: &AST) -> Vec<Dependency> {
        let mut dependencies = Vec::new();
        self.analyze_dependencies_recursive(&ast.root_node, &mut dependencies);
        dependencies
    }

    fn analyze_dependencies_recursive(&self, node: &ASTNode, dependencies: &mut Vec<Dependency>) {
        match node.node_type.as_str() {
            "import_statement" => {
                if let Some(dep) = self.parse_import_dependency(node) {
                    dependencies.push(dep);
                }
            }
            "extends_clause" => {
                if let Some(dep) = self.parse_inheritance_dependency(node) {
                    dependencies.push(dep);
                }
            }
            "implements_clause" => {
                if let Some(dep) = self.parse_implements_dependency(node) {
                    dependencies.push(dep);
                }
            }
            _ => {}
        }

        for child in &node.children {
            self.analyze_dependencies_recursive(child, dependencies);
        }
    }

    fn parse_import_dependency(&self, node: &ASTNode) -> Option<Dependency> {
        // Simplified import parsing
        for child in &node.children {
            if child.node_type == "string" {
                return Some(Dependency {
                    source: "current_file".to_string(),
                    target: child.text.trim_matches('"').trim_matches('\'').to_string(),
                    dependency_type: DependencyType::Import,
                    location: node.location.clone(),
                });
            }
        }
        None
    }

    fn parse_inheritance_dependency(&self, node: &ASTNode) -> Option<Dependency> {
        for child in &node.children {
            if child.node_type == "identifier" {
                return Some(Dependency {
                    source: "current_class".to_string(),
                    target: child.text.clone(),
                    dependency_type: DependencyType::Inheritance,
                    location: node.location.clone(),
                });
            }
        }
        None
    }

    fn parse_implements_dependency(&self, node: &ASTNode) -> Option<Dependency> {
        for child in &node.children {
            if child.node_type == "identifier" {
                return Some(Dependency {
                    source: "current_class".to_string(),
                    target: child.text.clone(),
                    dependency_type: DependencyType::Usage,
                    location: node.location.clone(),
                });
            }
        }
        None
    }

    /// Simplified AST parsing (in real implementation would use tree-sitter)
    fn parse_typescript_content(&self, content: &str, file_path: &Path) -> Result<AST, ParserError> {
        // This is a very simplified parser for demonstration
        // In production, this would use tree-sitter-typescript
        
        let mut root_node = ASTNode::new(
            "program".to_string(),
            content.to_string(),
            Location::new(1, 0, 0, content.len() as u32),
        );

        // Simple parsing logic (very basic)
        let lines: Vec<&str> = content.lines().enumerate().collect::<Vec<_>>().iter().map(|(_, line)| *line).collect();
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            let location = Location::new((line_num + 1) as u32, 0, 0, line.len() as u32);

            if trimmed.starts_with("function ") || trimmed.starts_with("async function ") {
                let func_node = self.parse_function_declaration(trimmed, location);
                root_node.add_child(func_node);
            } else if trimmed.starts_with("class ") {
                let class_node = self.parse_class_declaration(trimmed, location);
                root_node.add_child(class_node);
            } else if trimmed.starts_with("interface ") {
                let interface_node = self.parse_interface_declaration(trimmed, location);
                root_node.add_child(interface_node);
            } else if trimmed.starts_with("import ") || trimmed.starts_with("import{") {
                let import_node = self.parse_import_statement(trimmed, location);
                root_node.add_child(import_node);
            }
        }

        Ok(AST::new(
            file_path.to_string_lossy().to_string(),
            "typescript".to_string(),
            root_node,
        ))
    }

    fn parse_function_declaration(&self, line: &str, location: Location) -> ASTNode {
        ASTNode::new("function_declaration".to_string(), line.to_string(), location)
    }

    fn parse_class_declaration(&self, line: &str, location: Location) -> ASTNode {
        ASTNode::new("class_declaration".to_string(), line.to_string(), location)
    }

    fn parse_interface_declaration(&self, line: &str, location: Location) -> ASTNode {
        ASTNode::new("interface_declaration".to_string(), line.to_string(), location)
    }

    fn parse_import_statement(&self, line: &str, location: Location) -> ASTNode {
        ASTNode::new("import_statement".to_string(), line.to_string(), location)
    }
}

impl LanguageParser for TypeScriptParser {
    fn parse_file(&self, content: &str, file_path: &Path) -> Result<AST, ParserError> {
        self.parse_typescript_content(content, file_path)
    }

    fn extract_symbols(&self, ast: &AST) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        self.extract_symbols_recursive(&ast.root_node, &mut symbols);
        symbols
    }

    fn build_call_graph(&self, ast: &AST) -> CallGraph {
        self.build_call_graph_from_ast(ast)
    }

    fn analyze_dependencies(&self, ast: &AST) -> Vec<Dependency> {
        self.analyze_dependencies_from_ast(ast)
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["ts", "tsx", "js", "jsx"]
    }

    fn language_name(&self) -> &'static str {
        "TypeScript"
    }
}

impl TypeScriptParser {
    fn extract_symbols_recursive(&self, node: &ASTNode, symbols: &mut Vec<Symbol>) {
        match node.node_type.as_str() {
            "function_declaration" => {
                if let Some(symbol) = self.extract_function_symbol(node) {
                    symbols.push(Symbol::Function(symbol));
                }
            }
            "class_declaration" => {
                if let Some(symbol) = self.extract_class_symbol(node) {
                    symbols.push(Symbol::Class(symbol));
                }
            }
            "interface_declaration" => {
                if let Some(symbol) = self.extract_interface_symbol(node) {
                    symbols.push(Symbol::Interface(symbol));
                }
            }
            "import_statement" => {
                if let Some(symbol) = self.extract_import_symbol(node) {
                    symbols.push(Symbol::Import(symbol));
                }
            }
            "variable_declaration" => {
                if let Some(symbol) = self.extract_variable_symbol(node) {
                    symbols.push(Symbol::Variable(symbol));
                }
            }
            _ => {}
        }

        for child in &node.children {
            self.extract_symbols_recursive(child, symbols);
        }
    }

    fn extract_function_symbol(&self, node: &ASTNode) -> Option<FunctionSymbol> {
        // Extract function name from the text (simplified)
        let name = if let Some(name_start) = node.text.find("function ") {
            let after_function = &node.text[name_start + 9..];
            if let Some(paren_pos) = after_function.find('(') {
                after_function[..paren_pos].trim().to_string()
            } else {
                return None;
            }
        } else {
            return None;
        };

        Some(FunctionSymbol {
            name,
            parameters: self.extract_function_parameters(node),
            return_type: self.extract_return_type(node),
            location: node.location.clone(),
            visibility: self.extract_visibility(node),
            is_async: self.is_async_function(node),
            is_generator: self.is_generator_function(node),
            complexity: self.calculate_complexity(node),
            documentation: self.extract_documentation(node),
        })
    }

    fn extract_class_symbol(&self, node: &ASTNode) -> Option<ClassSymbol> {
        // Extract class name (simplified)
        let name = if let Some(name_start) = node.text.find("class ") {
            let after_class = &node.text[name_start + 6..];
            let name_end = after_class.find(' ')
                .or_else(|| after_class.find('{'))
                .unwrap_or(after_class.len());
            after_class[..name_end].trim().to_string()
        } else {
            return None;
        };

        Some(ClassSymbol {
            name,
            extends: None, // Simplified - would extract from extends clause
            implements: Vec::new(), // Simplified - would extract from implements clause
            methods: self.parse_class_methods(node),
            properties: self.parse_class_properties(node),
            location: node.location.clone(),
            visibility: self.extract_visibility(node),
            is_abstract: node.text.contains("abstract "),
            documentation: self.extract_documentation(node),
        })
    }

    fn extract_interface_symbol(&self, node: &ASTNode) -> Option<InterfaceSymbol> {
        // Extract interface name (simplified)
        let name = if let Some(name_start) = node.text.find("interface ") {
            let after_interface = &node.text[name_start + 10..];
            let name_end = after_interface.find(' ')
                .or_else(|| after_interface.find('{'))
                .unwrap_or(after_interface.len());
            after_interface[..name_end].trim().to_string()
        } else {
            return None;
        };

        Some(InterfaceSymbol {
            name,
            extends: Vec::new(), // Simplified
            methods: Vec::new(), // Simplified
            properties: Vec::new(), // Simplified
            location: node.location.clone(),
            documentation: self.extract_documentation(node),
        })
    }

    fn extract_import_symbol(&self, node: &ASTNode) -> Option<ImportSymbol> {
        // Simplified import parsing
        let text = &node.text;
        if let Some(from_pos) = text.find(" from ") {
            let import_part = &text[..from_pos];
            let module_part = &text[from_pos + 6..];
            
            let module_path = module_part.trim_matches('"').trim_matches('\'').trim_end_matches(';').to_string();
            
            Some(ImportSymbol {
                name: "import".to_string(),
                module_path,
                imported_items: Vec::new(), // Simplified
                location: node.location.clone(),
                is_default: false, // Simplified
                alias: None,
            })
        } else {
            None
        }
    }

    fn extract_variable_symbol(&self, node: &ASTNode) -> Option<VariableSymbol> {
        // Simplified variable extraction
        Some(VariableSymbol {
            name: "variable".to_string(), // Simplified
            var_type: None,
            location: node.location.clone(),
            visibility: Visibility::Public,
            is_mutable: true,
            initial_value: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_typescript_parser_creation() {
        let parser = TypeScriptParser::new();
        assert_eq!(parser.language_name(), "TypeScript");
        assert!(parser.supported_extensions().contains(&"ts"));
    }

    #[test]
    fn test_simple_function_parsing() {
        let parser = TypeScriptParser::new();
        let content = "function hello() { return 'world'; }";
        let path = PathBuf::from("test.ts");
        
        let result = parser.parse_file(content, &path);
        assert!(result.is_ok());
        
        let ast = result.unwrap();
        assert_eq!(ast.language, "typescript");
    }

    #[test]
    fn test_symbol_extraction() {
        let parser = TypeScriptParser::new();
        let content = "function test() {}";
        let path = PathBuf::from("test.ts");
        
        let ast = parser.parse_file(content, &path).unwrap();
        let symbols = parser.extract_symbols(&ast);
        
        assert!(!symbols.is_empty());
    }
}
