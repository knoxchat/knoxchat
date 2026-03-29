//! Context retrieval service for autocomplete
//!
//! This module handles retrieving relevant code context for autocomplete,
//! including imports, recently edited files, and semantic analysis.

use crate::error::Result;
use crate::types::{AutocompleteCodeSnippet, AutocompleteSnippetType, RecentlyEditedRange};
use crate::utils;

use std::collections::HashMap;
use std::path::Path;

/// Context retrieval service
pub struct ContextRetrievalService {
    // Cache for import definitions
    import_cache: HashMap<String, ImportDefinitions>,
}

/// Import definitions for a file
#[derive(Debug, Clone)]
pub struct ImportDefinitions {
    pub imports: HashMap<String, Vec<AutocompleteCodeSnippet>>,
}

impl ContextRetrievalService {
    /// Create a new context retrieval service
    pub fn new() -> Self {
        Self {
            import_cache: HashMap::new(),
        }
    }

    /// Get snippets from import definitions
    pub fn get_snippets_from_imports(
        &mut self,
        filepath: &str,
        prefix: &str,
        suffix: &str,
        use_imports: bool,
    ) -> Result<Vec<AutocompleteCodeSnippet>> {
        if !use_imports {
            return Ok(Vec::new());
        }

        let mut snippets = Vec::new();

        // Extract symbols from text around cursor
        let text_around_cursor = format!("{}{}", prefix, suffix);
        let symbols = utils::extract_symbols(&text_around_cursor);

        // Get import definitions for this file
        if let Some(file_info) = self.import_cache.get(filepath) {
            for symbol in symbols {
                if let Some(definitions) = file_info.imports.get(&symbol) {
                    snippets.extend(definitions.clone());
                }
            }
        }

        Ok(snippets)
    }

    /// Get snippets from recently edited ranges
    pub fn get_snippets_from_recently_edited(
        &self,
        recently_edited: &[RecentlyEditedRange],
        current_filepath: &str,
    ) -> Result<Vec<AutocompleteCodeSnippet>> {
        let mut snippets = Vec::new();

        for edited in recently_edited {
            // Skip the current file
            if edited.filepath == current_filepath {
                continue;
            }

            let content = edited.lines.join("\n");
            snippets.push(AutocompleteCodeSnippet {
                filepath: edited.filepath.clone(),
                content,
                snippet_type: AutocompleteSnippetType::Code,
            });
        }

        Ok(snippets)
    }

    /// Update import definitions for a file
    pub fn update_import_definitions(
        &mut self,
        filepath: &str,
        content: &str,
    ) -> Result<()> {
        let imports = self.parse_imports(filepath, content)?;
        self.import_cache.insert(filepath.to_string(), imports);
        Ok(())
    }

    /// Parse imports from file content
    fn parse_imports(&self, filepath: &str, content: &str) -> Result<ImportDefinitions> {
        let mut imports: HashMap<String, Vec<AutocompleteCodeSnippet>> = HashMap::new();

        let extension = Path::new(filepath)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match extension {
            "ts" | "tsx" | "js" | "jsx" => {
                self.parse_typescript_imports(content, &mut imports)?;
            }
            "py" => {
                self.parse_python_imports(content, &mut imports)?;
            }
            "rs" => {
                self.parse_rust_imports(content, &mut imports)?;
            }
            _ => {}
        }

        Ok(ImportDefinitions { imports })
    }

    /// Parse TypeScript/JavaScript imports
    fn parse_typescript_imports(
        &self,
        content: &str,
        imports: &mut HashMap<String, Vec<AutocompleteCodeSnippet>>,
    ) -> Result<()> {
        // Simple regex-based parsing for imports
        let import_re = regex::Regex::new(r#"import\s+(?:\{([^}]+)\}|(\w+))\s+from\s+['"]([^'"]+)['"]"#).unwrap();

        for cap in import_re.captures_iter(content) {
            let _module = cap.get(3).map(|m| m.as_str()).unwrap_or("");
            
            if let Some(named_imports) = cap.get(1) {
                // Named imports: import { foo, bar } from 'module'
                for symbol in named_imports.as_str().split(',') {
                    let symbol = symbol.trim().to_string();
                    imports.entry(symbol.clone()).or_default();
                }
            } else if let Some(default_import) = cap.get(2) {
                // Default import: import foo from 'module'
                let symbol = default_import.as_str().to_string();
                imports.entry(symbol.clone()).or_default();
            }
        }

        Ok(())
    }

    /// Parse Python imports
    fn parse_python_imports(
        &self,
        content: &str,
        imports: &mut HashMap<String, Vec<AutocompleteCodeSnippet>>,
    ) -> Result<()> {
        // Simple regex-based parsing for imports
        let import_re = regex::Regex::new(r"(?:from\s+(\S+)\s+)?import\s+(.+)").unwrap();

        for cap in import_re.captures_iter(content) {
            let imported = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            
            for symbol in imported.split(',') {
                let symbol = symbol.trim().split_whitespace().next().unwrap_or("");
                if !symbol.is_empty() {
                    imports.entry(symbol.to_string()).or_default();
                }
            }
        }

        Ok(())
    }

    /// Parse Rust imports
    fn parse_rust_imports(
        &self,
        content: &str,
        imports: &mut HashMap<String, Vec<AutocompleteCodeSnippet>>,
    ) -> Result<()> {
        // Simple regex-based parsing for use statements
        let use_re = regex::Regex::new(r"use\s+(?:[\w:]+::)?(?:\{([^}]+)\}|(\w+))").unwrap();

        for cap in use_re.captures_iter(content) {
            if let Some(braced_imports) = cap.get(1) {
                // use module::{foo, bar}
                for symbol in braced_imports.as_str().split(',') {
                    let symbol = symbol.trim().to_string();
                    imports.entry(symbol.clone()).or_default();
                }
            } else if let Some(single_import) = cap.get(2) {
                // use module::foo
                let symbol = single_import.as_str().to_string();
                imports.entry(symbol.clone()).or_default();
            }
        }

        Ok(())
    }

    /// Clear the import cache
    pub fn clear_cache(&mut self) {
        self.import_cache.clear();
    }
}

impl Default for ContextRetrievalService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_typescript_imports() {
        let content = r#"
            import { foo, bar } from 'module1';
            import baz from 'module2';
        "#;

        let service = ContextRetrievalService::new();
        let mut imports = HashMap::new();
        service.parse_typescript_imports(content, &mut imports).unwrap();

        assert!(imports.contains_key("foo"));
        assert!(imports.contains_key("bar"));
        assert!(imports.contains_key("baz"));
    }

    #[test]
    fn test_parse_python_imports() {
        let content = r#"
            from module1 import foo, bar
            import baz
        "#;

        let service = ContextRetrievalService::new();
        let mut imports = HashMap::new();
        service.parse_python_imports(content, &mut imports).unwrap();

        assert!(imports.contains_key("foo"));
        assert!(imports.contains_key("bar"));
        assert!(imports.contains_key("baz"));
    }

    #[test]
    fn test_parse_rust_imports() {
        let content = r#"
            use module::{foo, bar};
            use other::baz;
        "#;

        let service = ContextRetrievalService::new();
        let mut imports = HashMap::new();
        service.parse_rust_imports(content, &mut imports).unwrap();

        assert!(imports.contains_key("foo"));
        assert!(imports.contains_key("bar"));
        assert!(imports.contains_key("baz"));
    }
}

