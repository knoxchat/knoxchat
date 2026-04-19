//! Maps relationships between code entities

use super::types::*;
use crate::error::Result;
use crate::types::FileChange;
use lru::LruCache;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const DEFAULT_CACHE_CAP: usize = 256;
const CACHE_CLEANUP_INTERVAL_SECS: u64 = 300;

#[derive(Debug, Default)]
struct RelationshipCacheMetrics {
    hits: AtomicU64,
    misses: AtomicU64,
    evictions: AtomicU64,
    cleanup_runs: AtomicU64,
    entries_cleared: AtomicU64,
}

impl RelationshipCacheMetrics {
    fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    fn record_eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }

    fn record_cleanup(&self, cleared_entries: u64) {
        self.cleanup_runs.fetch_add(1, Ordering::Relaxed);
        self.entries_cleared
            .fetch_add(cleared_entries, Ordering::Relaxed);
    }

    fn snapshot(&self, current_size: usize, capacity: usize) -> CacheStatistics {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let lookups = hits + misses;

        CacheStatistics {
            hits,
            misses,
            hit_rate: if lookups == 0 {
                0.0
            } else {
                hits as f64 / lookups as f64
            },
            evictions: self.evictions.load(Ordering::Relaxed),
            cleanup_runs: self.cleanup_runs.load(Ordering::Relaxed),
            entries_cleared: self.entries_cleared.load(Ordering::Relaxed),
            current_size,
            capacity,
        }
    }
}

/// Maps relationships and dependencies between code entities
pub struct RelationshipMapper {
    // LRU cache for call graph results
    call_graph_cache: Arc<Mutex<LruCache<String, Vec<CallChain>>>>,
    cache_metrics: Arc<RelationshipCacheMetrics>,
    cache_cleanup_stop: Arc<AtomicBool>,
}

impl RelationshipMapper {
    pub fn new() -> Self {
        let call_graph_cache = Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(DEFAULT_CACHE_CAP).unwrap(),
        )));
        let cache_metrics = Arc::new(RelationshipCacheMetrics::default());
        let cache_cleanup_stop = Arc::new(AtomicBool::new(false));

        Self::start_cache_cleanup_task(
            call_graph_cache.clone(),
            cache_metrics.clone(),
            cache_cleanup_stop.clone(),
        );

        Self {
            call_graph_cache,
            cache_metrics,
            cache_cleanup_stop,
        }
    }

    fn start_cache_cleanup_task(
        cache: Arc<Mutex<LruCache<String, Vec<CallChain>>>>,
        metrics: Arc<RelationshipCacheMetrics>,
        stop_signal: Arc<AtomicBool>,
    ) {
        thread::spawn(move || {
            while !stop_signal.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_secs(CACHE_CLEANUP_INTERVAL_SECS));

                if stop_signal.load(Ordering::Relaxed) {
                    break;
                }

                let cleared = {
                    let mut cache = cache.lock();
                    let cleared = cache.len() as u64;
                    cache.clear();
                    cleared
                };

                if cleared > 0 {
                    metrics.record_cleanup(cleared);
                }
            }
        });
    }

    /// Clear the call graph cache
    pub fn clear_cache(&self) {
        let cleared = {
            let mut cache = self.call_graph_cache.lock();
            let cleared = cache.len() as u64;
            cache.clear();
            cleared
        };
        if cleared > 0 {
            self.cache_metrics.record_cleanup(cleared);
        }
    }

    /// Remove cached call chains for a specific file
    pub fn invalidate_file_cache(&self, file_path: &Path) {
        let file_key = file_path.to_string_lossy().to_string();
        self.call_graph_cache.lock().pop(&file_key);
    }

    /// Build call chains from AST
    pub fn build_call_chains(
        &self,
        ast: &super::analyzer::AST,
        file_path: &Path,
    ) -> Result<Vec<CallChain>> {
        let file_key = file_path.to_string_lossy().to_string();

        // Check cache first
        if let Some(cached_chains) = self.call_graph_cache.lock().get(&file_key) {
            self.cache_metrics.record_hit();
            return Ok(cached_chains.clone());
        }

        self.cache_metrics.record_miss();

        let mut call_chains = Vec::new();

        // Traverse AST to find function calls
        self.find_call_expressions(&ast.root, file_path, &mut call_chains)?;

        // Cache the results
        let mut cache = self.call_graph_cache.lock();
        if cache.len() == DEFAULT_CACHE_CAP {
            self.cache_metrics.record_eviction();
        }
        cache.put(file_key, call_chains.clone());

        Ok(call_chains)
    }

    pub fn get_cache_statistics(&self) -> CacheStatistics {
        self.cache_metrics
            .snapshot(self.call_graph_cache.lock().len(), DEFAULT_CACHE_CAP)
    }

    /// Build inheritance tree from class definitions
    pub fn build_inheritance_tree(
        &self,
        classes: &HashMap<String, ClassDefinition>,
    ) -> Result<InheritanceTree> {
        let mut root_classes = Vec::new();
        let mut relationships = HashMap::new();
        let mut depth_map = HashMap::new();

        // Find root classes (no parent)
        for (name, class_def) in classes {
            if class_def.extends.is_none() {
                root_classes.push(name.clone());
                depth_map.insert(name.clone(), 0);
            }
        }

        // Build parent-child relationships
        for (name, class_def) in classes {
            if let Some(parent) = &class_def.extends {
                relationships
                    .entry(parent.clone())
                    .or_insert_with(Vec::new)
                    .push(name.clone());

                // Calculate depth
                let parent_depth = depth_map.get(parent).unwrap_or(&0);
                depth_map.insert(name.clone(), parent_depth + 1);
            }
        }

        Ok(InheritanceTree {
            root_classes,
            relationships,
            depth_map,
        })
    }

    /// Identify usage patterns in code
    pub fn identify_usage_patterns(
        &self,
        ast: &super::analyzer::AST,
        file_path: &Path,
    ) -> Result<Vec<UsagePattern>> {
        let mut patterns = Vec::new();

        // Look for common patterns
        patterns.extend(self.find_singleton_patterns(ast, file_path)?);
        patterns.extend(self.find_factory_patterns(ast, file_path)?);
        patterns.extend(self.find_observer_patterns(ast, file_path)?);

        Ok(patterns)
    }

    /// Build dependency graph from file changes and semantic context
    pub fn build_dependency_graph(
        &self,
        file_changes: &[FileChange],
        semantic_context: &SemanticContext,
    ) -> Result<DependencyGraph> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut external_dependencies = Vec::new();

        // Create nodes for each file
        for file_change in file_changes {
            let node = DependencyNode {
                id: file_change.path.to_string_lossy().to_string(),
                node_type: DependencyNodeType::File,
                location: None,
                metadata: HashMap::new(),
            };
            nodes.push(node);
        }

        // Create edges based on imports
        for import in &semantic_context.imports {
            let edge = DependencyEdge {
                from: import.location.file_path.to_string_lossy().to_string(),
                to: import.module.clone(),
                edge_type: DependencyType::Import,
                strength: 1.0,
            };
            edges.push(edge);

            // Check if it's an external dependency
            if self.is_external_dependency(&import.module) {
                let external_dep = ExternalDependency {
                    name: import.module.clone(),
                    version: None,
                    source: self.detect_dependency_source(&import.module),
                    usage_locations: vec![import.location.clone()],
                };
                external_dependencies.push(external_dep);
            }
        }

        // Detect cycles (simplified implementation)
        let cycles = self.detect_cycles(&nodes, &edges)?;

        Ok(DependencyGraph {
            nodes,
            edges,
            cycles,
            external_dependencies,
        })
    }

    // Private helper methods

    fn find_call_expressions(
        &self,
        node: &super::analyzer::ASTNode,
        file_path: &Path,
        call_chains: &mut Vec<CallChain>,
    ) -> Result<()> {
        // Check if current node is a call expression
        if node.node_type == "call_expression" {
            let call_chain = self.parse_call_expression(node, file_path)?;
            call_chains.push(call_chain);
        }

        // Recursively check children
        for child in &node.children {
            self.find_call_expressions(child, file_path, call_chains)?;
        }

        Ok(())
    }

    fn parse_call_expression(
        &self,
        node: &super::analyzer::ASTNode,
        file_path: &Path,
    ) -> Result<CallChain> {
        // Extract caller and called function names
        let caller = self
            .find_containing_function(node)
            .unwrap_or("global".to_string());
        let called = self.extract_called_function_name(node)?;

        // Create a location that includes the file path context
        let mut location = node.location.clone();
        location.file_path = file_path.to_path_buf();

        Ok(CallChain {
            caller,
            called,
            call_type: CallType::Direct,
            location,
            is_async: node.text.contains("await"),
            parameters_passed: self.extract_call_parameters(node),
        })
    }

    fn find_containing_function(&self, node: &super::analyzer::ASTNode) -> Option<String> {
        // Look for function definition in the node's text content
        // This is a simplified implementation - in a real scenario,
        // we would traverse up the parent nodes in the AST
        if node.text.contains("function ") {
            // Extract function name from text (simplified)
            if let Some(start) = node.text.find("function ") {
                let after_function = &node.text[start + 9..]; // "function ".len() = 9
                if let Some(end) = after_function.find('(') {
                    let function_name = after_function[..end].trim();
                    if !function_name.is_empty() {
                        return Some(function_name.to_string());
                    }
                }
            }
        }

        // Check for arrow functions or method definitions
        if node.text.contains("=>") || node.text.contains(": function") {
            // Try to extract identifier before arrow or colon
            if let Some(arrow_pos) = node.text.find("=>") {
                let before_arrow = &node.text[..arrow_pos];
                if let Some(equals_pos) = before_arrow.rfind('=') {
                    let identifier = before_arrow[..equals_pos].trim();
                    if let Some(last_word) = identifier.split_whitespace().last() {
                        return Some(last_word.to_string());
                    }
                }
            }
        }

        // Default to global scope if no containing function found
        Some("global".to_string())
    }

    fn extract_called_function_name(&self, node: &super::analyzer::ASTNode) -> Result<String> {
        // Look for the function being called
        for child in &node.children {
            if child.node_type == "identifier" || child.node_type == "member_expression" {
                return Ok(child.text.clone());
            }
        }
        Ok("unknown".to_string())
    }

    fn extract_call_parameters(&self, node: &super::analyzer::ASTNode) -> Vec<String> {
        let mut parameters = Vec::new();

        for child in &node.children {
            if child.node_type == "arguments" {
                for arg in &child.children {
                    parameters.push(arg.text.clone());
                }
                break;
            }
        }

        parameters
    }

    fn find_singleton_patterns(
        &self,
        ast: &super::analyzer::AST,
        file_path: &Path,
    ) -> Result<Vec<UsagePattern>> {
        let mut patterns = Vec::new();

        // Look for singleton pattern indicators
        if ast.root.text.contains("getInstance") {
            patterns.push(UsagePattern {
                pattern_type: PatternType::CreationalPattern,
                description: format!("Singleton pattern detected in {}", file_path.display()),
                frequency: 1,
                locations: vec![ast.root.location.clone()],
                confidence: 0.8,
            });
        }

        // Also look for private constructors and static instances
        if ast.root.text.contains("private constructor")
            || (ast.root.text.contains("static") && ast.root.text.contains("instance"))
        {
            patterns.push(UsagePattern {
                pattern_type: PatternType::CreationalPattern,
                description: format!(
                    "Singleton implementation pattern in {}",
                    file_path.display()
                ),
                frequency: 1,
                locations: vec![ast.root.location.clone()],
                confidence: 0.7,
            });
        }

        Ok(patterns)
    }

    fn find_factory_patterns(
        &self,
        ast: &super::analyzer::AST,
        file_path: &Path,
    ) -> Result<Vec<UsagePattern>> {
        let mut patterns = Vec::new();

        // Look for factory pattern indicators
        if ast.root.text.contains("create") && ast.root.text.contains("Factory") {
            patterns.push(UsagePattern {
                pattern_type: PatternType::CreationalPattern,
                description: format!("Factory pattern detected in {}", file_path.display()),
                frequency: 1,
                locations: vec![ast.root.location.clone()],
                confidence: 0.7,
            });
        }

        // Look for builder patterns in the same file
        if file_path.to_string_lossy().contains("builder") || ast.root.text.contains("Builder") {
            patterns.push(UsagePattern {
                pattern_type: PatternType::CreationalPattern,
                description: format!("Builder pattern detected in {}", file_path.display()),
                frequency: 1,
                locations: vec![ast.root.location.clone()],
                confidence: 0.6,
            });
        }

        Ok(patterns)
    }

    fn find_observer_patterns(
        &self,
        ast: &super::analyzer::AST,
        file_path: &Path,
    ) -> Result<Vec<UsagePattern>> {
        let mut patterns = Vec::new();

        // Look for observer pattern indicators
        if ast.root.text.contains("addEventListener") || ast.root.text.contains("subscribe") {
            patterns.push(UsagePattern {
                pattern_type: PatternType::BehavioralPattern,
                description: format!("Observer pattern detected in {}", file_path.display()),
                frequency: 1,
                locations: vec![ast.root.location.clone()],
                confidence: 0.6,
            });
        }

        // Check for event emitter patterns based on file context
        if file_path.to_string_lossy().contains("event")
            || ast.root.text.contains("emit")
            || ast.root.text.contains("EventEmitter")
        {
            patterns.push(UsagePattern {
                pattern_type: PatternType::BehavioralPattern,
                description: format!("Event emitter pattern in {}", file_path.display()),
                frequency: 1,
                locations: vec![ast.root.location.clone()],
                confidence: 0.7,
            });
        }

        Ok(patterns)
    }

    fn is_external_dependency(&self, module_name: &str) -> bool {
        // Check if module is external (not relative import)
        !module_name.starts_with('.') && !module_name.starts_with('/')
    }

    fn detect_dependency_source(&self, module_name: &str) -> DependencySource {
        // Simple heuristics to detect dependency source
        if module_name.starts_with('@') || self.is_known_npm_package(module_name) {
            DependencySource::NPM
        } else if module_name.contains("java") || module_name.contains("org.") {
            DependencySource::Maven
        } else if module_name.contains("System.") || module_name.contains("Microsoft.") {
            DependencySource::NuGet
        } else {
            DependencySource::Unknown
        }
    }

    fn is_known_npm_package(&self, module_name: &str) -> bool {
        // List of common npm packages
        let known_packages = [
            "react",
            "vue",
            "angular",
            "lodash",
            "express",
            "axios",
            "typescript",
        ];
        known_packages.iter().any(|&pkg| module_name.contains(pkg))
    }

    fn detect_cycles(
        &self,
        nodes: &[DependencyNode],
        edges: &[DependencyEdge],
    ) -> Result<Vec<Vec<String>>> {
        // Simplified cycle detection using DFS
        let mut cycles = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut recursion_stack = std::collections::HashSet::new();

        for node in nodes {
            if !visited.contains(&node.id) {
                self.dfs_cycle_detection(
                    &node.id,
                    edges,
                    &mut visited,
                    &mut recursion_stack,
                    &mut cycles,
                    &mut Vec::new(),
                );
            }
        }

        Ok(cycles)
    }

    fn dfs_cycle_detection(
        &self,
        node_id: &str,
        edges: &[DependencyEdge],
        visited: &mut std::collections::HashSet<String>,
        recursion_stack: &mut std::collections::HashSet<String>,
        cycles: &mut Vec<Vec<String>>,
        current_path: &mut Vec<String>,
    ) {
        visited.insert(node_id.to_string());
        recursion_stack.insert(node_id.to_string());
        current_path.push(node_id.to_string());

        // Find all outgoing edges from this node
        for edge in edges {
            if edge.from == node_id {
                if recursion_stack.contains(&edge.to) {
                    // Cycle detected
                    if let Some(cycle_start) = current_path.iter().position(|n| n == &edge.to) {
                        let cycle = current_path[cycle_start..].to_vec();
                        cycles.push(cycle);
                    }
                } else if !visited.contains(&edge.to) {
                    self.dfs_cycle_detection(
                        &edge.to,
                        edges,
                        visited,
                        recursion_stack,
                        cycles,
                        current_path,
                    );
                }
            }
        }

        recursion_stack.remove(node_id);
        current_path.pop();
    }
}

impl Drop for RelationshipMapper {
    fn drop(&mut self) {
        self.cache_cleanup_stop.store(true, Ordering::Relaxed);
    }
}

#[derive(Debug, Clone)]
pub struct CacheStatistics {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub evictions: u64,
    pub cleanup_runs: u64,
    pub entries_cleared: u64,
    pub current_size: usize,
    pub capacity: usize,
}

impl Default for RelationshipMapper {
    fn default() -> Self {
        Self::new()
    }
}
