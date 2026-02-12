//! Symbol Resolution System
//!
//! This module resolves symbols across files, handling imports, exports,
//! and building complete dependency chains and call graphs.

use super::knowledge_graph::{EdgeType, KnowledgeGraph, NodeType};
use super::types::*;
use crate::error::{CheckpointError, Result};
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Symbol resolver for cross-file resolution
pub struct SymbolResolver {
    knowledge_graph: Arc<KnowledgeGraph>,
    symbol_table: Arc<RwLock<SymbolTable>>,
    module_registry: Arc<RwLock<ModuleRegistry>>,
    import_resolution_cache: Arc<RwLock<HashMap<String, ResolutionResult>>>,
}

/// Global symbol table
#[derive(Debug, Clone)]
pub struct SymbolTable {
    /// Map from fully qualified name to symbol
    symbols: HashMap<String, SymbolEntry>,
    /// Map from file path to symbols defined in that file
    file_symbols: HashMap<String, Vec<String>>,
    /// Map from symbol name to all possible definitions (for ambiguous resolution)
    name_index: HashMap<String, Vec<String>>,
}

/// Entry in the symbol table
#[derive(Debug, Clone)]
pub struct SymbolEntry {
    pub fully_qualified_name: String,
    pub simple_name: String,
    pub symbol_type: SymbolType,
    pub definition_location: CodeLocation,
    pub visibility: SymbolVisibility,
    pub module_path: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Type of symbol
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolType {
    Function,
    Class,
    Interface,
    Type,
    Variable,
    Constant,
    Module,
    Namespace,
}

/// Visibility of a symbol
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolVisibility {
    Public,
    Protected,
    Private,
    Internal,
}

/// Module registry tracking all modules
#[derive(Debug, Clone)]
pub struct ModuleRegistry {
    modules: HashMap<String, ModuleInfo>,
    path_to_module: HashMap<PathBuf, String>,
}

/// Information about a module
#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub module_path: Vec<String>,
    pub file_path: PathBuf,
    pub exports: Vec<ExportEntry>,
    pub imports: Vec<ImportEntry>,
}

/// Export entry
#[derive(Debug, Clone)]
pub struct ExportEntry {
    pub name: String,
    pub original_name: Option<String>,
    pub symbol_type: SymbolType,
    pub is_default: bool,
}

/// Import entry
#[derive(Debug, Clone)]
pub struct ImportEntry {
    pub imported_name: String,
    pub original_name: Option<String>,
    pub source_module: String,
    pub resolved_path: Option<PathBuf>,
}

/// Result of symbol resolution
#[derive(Debug, Clone)]
pub struct ResolutionResult {
    pub symbol: SymbolEntry,
    pub resolution_path: Vec<String>,
    pub confidence: f64,
}

impl SymbolResolver {
    /// Create a new symbol resolver
    pub fn new(knowledge_graph: Arc<KnowledgeGraph>) -> Self {
        Self {
            knowledge_graph,
            symbol_table: Arc::new(RwLock::new(SymbolTable::new())),
            module_registry: Arc::new(RwLock::new(ModuleRegistry::new())),
            import_resolution_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a symbol in the symbol table
    pub fn register_symbol(&self, entry: SymbolEntry) -> Result<()> {
        let mut symbol_table = self.symbol_table.write();

        // Add to main symbols map
        symbol_table
            .symbols
            .insert(entry.fully_qualified_name.clone(), entry.clone());

        // Update file index
        symbol_table
            .file_symbols
            .entry(
                entry
                    .definition_location
                    .file_path
                    .to_string_lossy()
                    .to_string(),
            )
            .or_insert_with(Vec::new)
            .push(entry.fully_qualified_name.clone());

        // Update name index
        symbol_table
            .name_index
            .entry(entry.simple_name.clone())
            .or_insert_with(Vec::new)
            .push(entry.fully_qualified_name);

        Ok(())
    }

    /// Register a module
    pub fn register_module(&self, module_info: ModuleInfo) -> Result<()> {
        let mut registry = self.module_registry.write();

        let module_name = module_info.module_path.join("::");
        registry
            .path_to_module
            .insert(module_info.file_path.clone(), module_name.clone());
        registry.modules.insert(module_name, module_info);

        Ok(())
    }

    /// Resolve a symbol reference
    pub fn resolve_symbol(
        &self,
        symbol_name: &str,
        context_file: &Path,
        context_scope: &[String],
    ) -> Result<ResolutionResult> {
        // Check cache first
        let cache_key = format!(
            "{}:{}:{:?}",
            symbol_name,
            context_file.display(),
            context_scope
        );

        if let Some(cached) = self.import_resolution_cache.read().get(&cache_key) {
            return Ok(cached.clone());
        }

        // Try different resolution strategies
        let result = self
            .resolve_in_current_scope(symbol_name, context_scope)
            .or_else(|_| self.resolve_in_imports(symbol_name, context_file))
            .or_else(|_| self.resolve_in_same_file(symbol_name, context_file))
            .or_else(|_| self.resolve_globally(symbol_name))?;

        // Cache the result
        self.import_resolution_cache
            .write()
            .insert(cache_key, result.clone());

        Ok(result)
    }

    /// Resolve in current scope
    fn resolve_in_current_scope(
        &self,
        symbol_name: &str,
        scope: &[String],
    ) -> Result<ResolutionResult> {
        let symbol_table = self.symbol_table.read();

        // Build fully qualified name from scope
        let mut fqn = scope.to_vec();
        fqn.push(symbol_name.to_string());
        let fqn_str = fqn.join("::");

        if let Some(symbol) = symbol_table.symbols.get(&fqn_str) {
            return Ok(ResolutionResult {
                symbol: symbol.clone(),
                resolution_path: fqn,
                confidence: 1.0,
            });
        }

        Err(CheckpointError::validation(format!(
            "Symbol {} not found in scope",
            symbol_name
        )))
    }

    /// Resolve through imports
    fn resolve_in_imports(
        &self,
        symbol_name: &str,
        context_file: &Path,
    ) -> Result<ResolutionResult> {
        let registry = self.module_registry.read();

        // Get module for context file
        let module_name = registry
            .path_to_module
            .get(context_file)
            .ok_or_else(|| CheckpointError::validation("Module not found for file".to_string()))?;

        let module_info = registry
            .modules
            .get(module_name)
            .ok_or_else(|| CheckpointError::validation("Module info not found".to_string()))?;

        // Search through imports
        for import in &module_info.imports {
            if import.imported_name == symbol_name {
                // Try to resolve the import
                if let Some(resolved_path) = &import.resolved_path {
                    return self.resolve_in_same_file(
                        import.original_name.as_deref().unwrap_or(symbol_name),
                        resolved_path,
                    );
                }
            }
        }

        Err(CheckpointError::validation(format!(
            "Symbol {} not found in imports",
            symbol_name
        )))
    }

    /// Resolve in same file
    fn resolve_in_same_file(
        &self,
        symbol_name: &str,
        file_path: &Path,
    ) -> Result<ResolutionResult> {
        let symbol_table = self.symbol_table.read();

        let file_path_str = file_path.to_string_lossy().to_string();

        if let Some(symbols_in_file) = symbol_table.file_symbols.get(&file_path_str) {
            for fqn in symbols_in_file {
                if let Some(symbol) = symbol_table.symbols.get(fqn) {
                    if symbol.simple_name == symbol_name {
                        return Ok(ResolutionResult {
                            symbol: symbol.clone(),
                            resolution_path: symbol.module_path.clone(),
                            confidence: 0.9,
                        });
                    }
                }
            }
        }

        Err(CheckpointError::validation(format!(
            "Symbol {} not found in file",
            symbol_name
        )))
    }

    /// Resolve globally
    fn resolve_globally(&self, symbol_name: &str) -> Result<ResolutionResult> {
        let symbol_table = self.symbol_table.read();

        if let Some(possible_symbols) = symbol_table.name_index.get(symbol_name) {
            if possible_symbols.len() == 1 {
                // Unambiguous resolution
                let fqn = &possible_symbols[0];
                if let Some(symbol) = symbol_table.symbols.get(fqn) {
                    return Ok(ResolutionResult {
                        symbol: symbol.clone(),
                        resolution_path: symbol.module_path.clone(),
                        confidence: 0.7,
                    });
                }
            } else if !possible_symbols.is_empty() {
                // Ambiguous - return first public symbol with lower confidence
                for fqn in possible_symbols {
                    if let Some(symbol) = symbol_table.symbols.get(fqn) {
                        if symbol.visibility == SymbolVisibility::Public {
                            return Ok(ResolutionResult {
                                symbol: symbol.clone(),
                                resolution_path: symbol.module_path.clone(),
                                confidence: 0.5,
                            });
                        }
                    }
                }
            }
        }

        Err(CheckpointError::validation(format!(
            "Symbol {} not found globally",
            symbol_name
        )))
    }

    /// Build complete call graph across files
    pub fn build_complete_call_graph(&self) -> Result<Vec<CallChain>> {
        let mut call_chains = Vec::new();
        let symbol_table = self.symbol_table.read();

        // Iterate through all function symbols
        for (fqn, symbol) in &symbol_table.symbols {
            if symbol.symbol_type == SymbolType::Function {
                // Get all calls from this function using knowledge graph
                let node_id = format!(
                    "{}::{}",
                    symbol.definition_location.file_path.display(),
                    symbol.simple_name
                );

                let outgoing_edges = self.knowledge_graph.get_outgoing_edges(&node_id);

                for edge in outgoing_edges {
                    if edge.edge_type == EdgeType::Calls {
                        call_chains.push(CallChain {
                            caller: fqn.clone(),
                            called: edge.to.clone(),
                            call_type: CallType::Direct,
                            location: symbol.definition_location.clone(),
                            is_async: false,
                            parameters_passed: Vec::new(),
                        });
                    }
                }
            }
        }

        Ok(call_chains)
    }

    /// Get all dependencies for a file
    pub fn get_file_dependencies(&self, file_path: &Path) -> Result<Vec<PathBuf>> {
        let registry = self.module_registry.read();

        let module_name = registry
            .path_to_module
            .get(file_path)
            .ok_or_else(|| CheckpointError::validation("Module not found".to_string()))?;

        let module_info = registry
            .modules
            .get(module_name)
            .ok_or_else(|| CheckpointError::validation("Module info not found".to_string()))?;

        let dependencies: Vec<PathBuf> = module_info
            .imports
            .iter()
            .filter_map(|import| import.resolved_path.clone())
            .collect();

        Ok(dependencies)
    }

    /// Get all dependents of a file
    pub fn get_file_dependents(&self, file_path: &Path) -> Result<Vec<PathBuf>> {
        let mut dependents = Vec::new();
        let registry = self.module_registry.read();

        let target_module_name = registry
            .path_to_module
            .get(file_path)
            .ok_or_else(|| CheckpointError::validation("Module not found".to_string()))?;

        // Search all modules for imports from target module
        for (module_path, module_info) in &registry.modules {
            if module_path != target_module_name {
                for import in &module_info.imports {
                    if import.source_module == *target_module_name {
                        dependents.push(module_info.file_path.clone());
                        break;
                    }
                }
            }
        }

        Ok(dependents)
    }

    /// Build complete dependency graph
    pub fn build_dependency_graph(&self) -> Result<HashMap<PathBuf, Vec<PathBuf>>> {
        let mut graph = HashMap::new();
        let registry = self.module_registry.read();

        for module_info in registry.modules.values() {
            let dependencies: Vec<PathBuf> = module_info
                .imports
                .iter()
                .filter_map(|import| import.resolved_path.clone())
                .collect();

            graph.insert(module_info.file_path.clone(), dependencies);
        }

        Ok(graph)
    }

    /// Detect circular dependencies
    pub fn detect_circular_dependencies(&self) -> Result<Vec<Vec<PathBuf>>> {
        let dep_graph = self.build_dependency_graph()?;
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for file in dep_graph.keys() {
            if !visited.contains(file) {
                self.detect_cycles_dfs(
                    file,
                    &dep_graph,
                    &mut visited,
                    &mut rec_stack,
                    &mut Vec::new(),
                    &mut cycles,
                );
            }
        }

        Ok(cycles)
    }

    /// DFS helper for cycle detection
    fn detect_cycles_dfs(
        &self,
        node: &PathBuf,
        graph: &HashMap<PathBuf, Vec<PathBuf>>,
        visited: &mut HashSet<PathBuf>,
        rec_stack: &mut HashSet<PathBuf>,
        current_path: &mut Vec<PathBuf>,
        cycles: &mut Vec<Vec<PathBuf>>,
    ) {
        visited.insert(node.clone());
        rec_stack.insert(node.clone());
        current_path.push(node.clone());

        if let Some(dependencies) = graph.get(node) {
            for dep in dependencies {
                if !visited.contains(dep) {
                    self.detect_cycles_dfs(dep, graph, visited, rec_stack, current_path, cycles);
                } else if rec_stack.contains(dep) {
                    // Found a cycle
                    let cycle_start = current_path.iter().position(|p| p == dep).unwrap();
                    cycles.push(current_path[cycle_start..].to_vec());
                }
            }
        }

        current_path.pop();
        rec_stack.remove(node);
    }

    /// Get all symbols defined in a file
    pub fn get_file_symbols(&self, file_path: &Path) -> Vec<SymbolEntry> {
        let symbol_table = self.symbol_table.read();
        let file_path_str = file_path.to_string_lossy().to_string();

        symbol_table
            .file_symbols
            .get(&file_path_str)
            .map(|fqns| {
                fqns.iter()
                    .filter_map(|fqn| symbol_table.symbols.get(fqn).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all references to a symbol
    pub fn find_all_references(&self, symbol_fqn: &str) -> Vec<CodeLocation> {
        let mut references = Vec::new();

        // Use knowledge graph to find all references
        let symbol_nodes: Vec<_> = self
            .knowledge_graph
            .get_nodes_by_type(NodeType::Function) // Would need to check all types
            .into_iter()
            .filter(|node| node.name == symbol_fqn)
            .collect();

        for node in symbol_nodes {
            let incoming_edges = self.knowledge_graph.get_incoming_edges(&node.id);

            for edge in incoming_edges {
                if edge.edge_type == EdgeType::References {
                    if let Some(referring_node) = self.knowledge_graph.get_node(&edge.from) {
                        references.push(referring_node.location);
                    }
                }
            }
        }

        references
    }

    /// Clear all caches
    pub fn clear_caches(&self) {
        self.import_resolution_cache.write().clear();
    }

    /// Get symbol table statistics
    pub fn get_statistics(&self) -> SymbolTableStatistics {
        let symbol_table = self.symbol_table.read();
        let registry = self.module_registry.read();

        SymbolTableStatistics {
            total_symbols: symbol_table.symbols.len(),
            total_modules: registry.modules.len(),
            files_indexed: symbol_table.file_symbols.len(),
            ambiguous_symbols: symbol_table
                .name_index
                .values()
                .filter(|v| v.len() > 1)
                .count(),
        }
    }
}

impl SymbolTable {
    fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            file_symbols: HashMap::new(),
            name_index: HashMap::new(),
        }
    }
}

impl ModuleRegistry {
    fn new() -> Self {
        Self {
            modules: HashMap::new(),
            path_to_module: HashMap::new(),
        }
    }
}

/// Statistics about the symbol table
#[derive(Debug, Clone)]
pub struct SymbolTableStatistics {
    pub total_symbols: usize,
    pub total_modules: usize,
    pub files_indexed: usize,
    pub ambiguous_symbols: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_resolver_creation() {
        let graph = Arc::new(KnowledgeGraph::new());
        let resolver = SymbolResolver::new(graph);
        let stats = resolver.get_statistics();
        assert_eq!(stats.total_symbols, 0);
    }

    #[test]
    fn test_register_symbol() {
        let graph = Arc::new(KnowledgeGraph::new());
        let resolver = SymbolResolver::new(graph);

        let entry = SymbolEntry {
            fully_qualified_name: "module::function".to_string(),
            simple_name: "function".to_string(),
            symbol_type: SymbolType::Function,
            definition_location: CodeLocation {
                file_path: "test.ts".to_string(),
                start_line: 1,
                start_column: 1,
                end_line: 10,
                end_column: 1,
            },
            visibility: SymbolVisibility::Public,
            module_path: vec!["module".to_string()],
            metadata: HashMap::new(),
        };

        assert!(resolver.register_symbol(entry).is_ok());
        let stats = resolver.get_statistics();
        assert_eq!(stats.total_symbols, 1);
    }
}
