//! Incremental Analysis System
//!
//! This module provides real-time incremental updates when code changes,
//! avoiding full re-analysis for better performance.

use super::knowledge_graph::{GraphNode, KnowledgeGraph, NodeType};
use super::symbol_resolver::SymbolResolver;
use super::types::*;
use crate::error::Result;
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Incremental analysis system for real-time updates
pub struct IncrementalUpdater {
    knowledge_graph: Arc<KnowledgeGraph>,
    symbol_resolver: Arc<SymbolResolver>,
    change_buffer: Arc<RwLock<ChangeBuffer>>,
    dependency_tracker: Arc<RwLock<DependencyTracker>>,
    update_config: UpdateConfig,
}

/// Configuration for incremental updates
#[derive(Debug, Clone)]
pub struct UpdateConfig {
    pub batch_size: usize,
    pub debounce_ms: u64,
    pub max_cascade_depth: usize,
    pub enable_smart_invalidation: bool,
    pub background_processing: bool,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            batch_size: 10,
            debounce_ms: 100,
            max_cascade_depth: 5,
            enable_smart_invalidation: true,
            background_processing: true,
        }
    }
}

/// Buffer for batching changes
#[derive(Debug, Clone)]
struct ChangeBuffer {
    pending_changes: VecDeque<FileChangeEvent>,
    last_process_time: Option<Instant>,
}

/// File change event
#[derive(Debug, Clone)]
pub struct FileChangeEvent {
    pub file_path: String,
    pub change_type: FileChangeType,
    pub timestamp: Instant,
    pub content_hash: Option<String>,
}

/// Type of file change
#[derive(Debug, Clone, PartialEq)]
pub enum FileChangeType {
    Created,
    Modified,
    Deleted,
    Renamed { old_path: String },
}

/// Dependency tracker for cascading updates
#[derive(Debug, Clone)]
struct DependencyTracker {
    file_dependencies: HashMap<String, HashSet<String>>,
    reverse_dependencies: HashMap<String, HashSet<String>>,
    last_update: HashMap<String, Instant>,
}

/// Result of an incremental update
#[derive(Debug, Clone)]
pub struct UpdateResult {
    pub files_updated: usize,
    pub nodes_updated: usize,
    pub edges_updated: usize,
    pub cascade_updates: usize,
    pub processing_time_ms: u64,
    pub invalidated_caches: Vec<String>,
}

/// Update statistics
#[derive(Debug, Clone)]
pub struct UpdateStatistics {
    pub total_updates: usize,
    pub average_update_time_ms: f64,
    pub cache_hit_rate: f64,
    pub cascade_ratio: f64,
}

impl IncrementalUpdater {
    /// Create a new incremental updater
    pub fn new(knowledge_graph: Arc<KnowledgeGraph>, symbol_resolver: Arc<SymbolResolver>) -> Self {
        Self {
            knowledge_graph,
            symbol_resolver,
            change_buffer: Arc::new(RwLock::new(ChangeBuffer {
                pending_changes: VecDeque::new(),
                last_process_time: None,
            })),
            dependency_tracker: Arc::new(RwLock::new(DependencyTracker {
                file_dependencies: HashMap::new(),
                reverse_dependencies: HashMap::new(),
                last_update: HashMap::new(),
            })),
            update_config: UpdateConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        knowledge_graph: Arc<KnowledgeGraph>,
        symbol_resolver: Arc<SymbolResolver>,
        config: UpdateConfig,
    ) -> Self {
        let mut updater = Self::new(knowledge_graph, symbol_resolver);
        updater.update_config = config;
        updater
    }

    /// Register a file change
    pub fn register_change(&self, event: FileChangeEvent) -> Result<()> {
        let mut buffer = self.change_buffer.write();
        buffer.pending_changes.push_back(event);
        Ok(())
    }

    /// Process pending changes
    pub fn process_pending_changes(&self) -> Result<UpdateResult> {
        let start_time = Instant::now();
        let mut result = UpdateResult {
            files_updated: 0,
            nodes_updated: 0,
            edges_updated: 0,
            cascade_updates: 0,
            processing_time_ms: 0,
            invalidated_caches: Vec::new(),
        };

        // Get batch of changes
        let changes = self.get_batched_changes()?;

        if changes.is_empty() {
            return Ok(result);
        }

        result.files_updated = changes.len();

        // Process each change
        for change in changes {
            let update = self.process_single_change(&change)?;
            result.nodes_updated += update.nodes_updated;
            result.edges_updated += update.edges_updated;
            result.invalidated_caches.extend(update.invalidated_caches);

            // Handle cascade updates if enabled
            if self.update_config.enable_smart_invalidation {
                let cascade = self.process_cascade_updates(&change)?;
                result.cascade_updates += cascade.cascade_updates;
            }
        }

        result.processing_time_ms = start_time.elapsed().as_millis() as u64;
        Ok(result)
    }

    /// Get batched changes respecting debounce
    fn get_batched_changes(&self) -> Result<Vec<FileChangeEvent>> {
        let mut buffer = self.change_buffer.write();

        // Check debounce
        if let Some(last_time) = buffer.last_process_time {
            let elapsed = last_time.elapsed();
            if elapsed < Duration::from_millis(self.update_config.debounce_ms) {
                return Ok(Vec::new());
            }
        }

        // Take batch
        let mut changes = Vec::new();
        for _ in 0..self.update_config.batch_size {
            if let Some(change) = buffer.pending_changes.pop_front() {
                changes.push(change);
            } else {
                break;
            }
        }

        buffer.last_process_time = Some(Instant::now());
        Ok(changes)
    }

    /// Process a single file change
    fn process_single_change(&self, event: &FileChangeEvent) -> Result<UpdateResult> {
        match event.change_type {
            FileChangeType::Created => self.handle_file_created(event),
            FileChangeType::Modified => self.handle_file_modified(event),
            FileChangeType::Deleted => self.handle_file_deleted(event),
            FileChangeType::Renamed { ref old_path } => self.handle_file_renamed(event, old_path),
        }
    }

    /// Handle file creation
    fn handle_file_created(&self, event: &FileChangeEvent) -> Result<UpdateResult> {
        let mut result = UpdateResult {
            files_updated: 1,
            nodes_updated: 0,
            edges_updated: 0,
            cascade_updates: 0,
            processing_time_ms: 0,
            invalidated_caches: vec![event.file_path.clone()],
        };

        // Parse and extract symbols from new file
        // (Would use actual parser here)
        let new_nodes = self.extract_nodes_from_file(&event.file_path)?;

        for node in new_nodes {
            self.knowledge_graph.add_node(node)?;
            result.nodes_updated += 1;
        }

        // Update dependencies
        self.update_file_dependencies(&event.file_path)?;

        Ok(result)
    }

    /// Handle file modification
    fn handle_file_modified(&self, event: &FileChangeEvent) -> Result<UpdateResult> {
        let mut result = UpdateResult {
            files_updated: 1,
            nodes_updated: 0,
            edges_updated: 0,
            cascade_updates: 0,
            processing_time_ms: 0,
            invalidated_caches: vec![event.file_path.clone()],
        };

        // Get old nodes for this file
        let old_nodes = self.knowledge_graph.get_nodes_in_file(&event.file_path);
        let old_node_ids: HashSet<_> = old_nodes.iter().map(|n| n.id.clone()).collect();

        // Parse and extract new symbols
        let new_nodes = self.extract_nodes_from_file(&event.file_path)?;
        let new_node_ids: HashSet<_> = new_nodes.iter().map(|n| n.id.clone()).collect();

        // Find what changed
        let added: Vec<_> = new_nodes
            .iter()
            .filter(|n| !old_node_ids.contains(&n.id))
            .collect();

        let removed: Vec<_> = old_nodes
            .iter()
            .filter(|n| !new_node_ids.contains(&n.id))
            .collect();

        // Apply incremental changes
        for node in added {
            self.knowledge_graph.add_node(node.clone())?;
            result.nodes_updated += 1;
        }

        for _node in removed {
            // Remove node and its edges
            // (Would implement node removal in graph)
            result.nodes_updated += 1;
        }

        // Update modified nodes
        for new_node in new_nodes {
            if old_node_ids.contains(&new_node.id) {
                // Update existing node
                self.knowledge_graph.add_node(new_node)?;
                result.nodes_updated += 1;
            }
        }

        // Update dependencies
        self.update_file_dependencies(&event.file_path)?;

        Ok(result)
    }

    /// Handle file deletion
    fn handle_file_deleted(&self, event: &FileChangeEvent) -> Result<UpdateResult> {
        let mut result = UpdateResult {
            files_updated: 1,
            nodes_updated: 0,
            edges_updated: 0,
            cascade_updates: 0,
            processing_time_ms: 0,
            invalidated_caches: vec![event.file_path.clone()],
        };

        // Get all nodes in this file
        let nodes = self.knowledge_graph.get_nodes_in_file(&event.file_path);

        // Remove all nodes (and their edges)
        for _node in nodes {
            // Would remove from graph
            result.nodes_updated += 1;
        }

        // Remove from dependency tracker
        self.remove_file_dependencies(&event.file_path)?;

        Ok(result)
    }

    /// Handle file rename
    fn handle_file_renamed(&self, event: &FileChangeEvent, old_path: &str) -> Result<UpdateResult> {
        // Delete old + create new
        let delete_result = self.handle_file_deleted(&FileChangeEvent {
            file_path: old_path.to_string(),
            change_type: FileChangeType::Deleted,
            timestamp: event.timestamp,
            content_hash: None,
        })?;

        let create_result = self.handle_file_created(event)?;

        Ok(UpdateResult {
            files_updated: 2,
            nodes_updated: delete_result.nodes_updated + create_result.nodes_updated,
            edges_updated: delete_result.edges_updated + create_result.edges_updated,
            cascade_updates: 0,
            processing_time_ms: 0,
            invalidated_caches: vec![old_path.to_string(), event.file_path.clone()],
        })
    }

    /// Process cascade updates for dependent files
    fn process_cascade_updates(&self, event: &FileChangeEvent) -> Result<UpdateResult> {
        let mut result = UpdateResult {
            files_updated: 0,
            nodes_updated: 0,
            edges_updated: 0,
            cascade_updates: 0,
            processing_time_ms: 0,
            invalidated_caches: Vec::new(),
        };

        // Find files that depend on this one
        let dependent_files = self.find_dependent_files(&event.file_path)?;

        // Process dependent files up to cascade depth
        let mut processed = HashSet::new();
        let mut queue = VecDeque::new();

        for file in dependent_files {
            queue.push_back((file, 0)); // (file, depth)
        }

        while let Some((file, depth)) = queue.pop_front() {
            if depth >= self.update_config.max_cascade_depth {
                continue;
            }

            if processed.contains(&file) {
                continue;
            }

            processed.insert(file.clone());

            // Mark for invalidation
            result.invalidated_caches.push(file.clone());
            result.cascade_updates += 1;

            // Find transitive dependencies
            if let Ok(transitive) = self.find_dependent_files(&file) {
                for dep_file in transitive {
                    queue.push_back((dep_file, depth + 1));
                }
            }
        }

        Ok(result)
    }

    /// Extract nodes from a file (simplified)
    fn extract_nodes_from_file(&self, file_path: &str) -> Result<Vec<GraphNode>> {
        // In reality, would parse the file with tree-sitter
        // For now, return mock data
        Ok(vec![GraphNode {
            id: format!("{}::mock_function", file_path),
            node_type: NodeType::Function,
            name: "mockFunction".to_string(),
            file_path: file_path.to_string(),
            location: CodeLocation {
                file_path: PathBuf::from(file_path),
                start_line: 1,
                start_column: 1,
                end_line: 10,
                end_column: 1,
            },
            metadata: HashMap::new(),
            checkpoint_id: None,
        }])
    }

    /// Update file dependencies
    fn update_file_dependencies(&self, file_path: &str) -> Result<()> {
        let mut tracker = self.dependency_tracker.write();

        // Get imports from this file
        let dependencies = self
            .symbol_resolver
            .get_file_dependencies(std::path::Path::new(file_path))?;

        // Update forward dependencies
        let dep_set: HashSet<_> = dependencies
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        tracker
            .file_dependencies
            .insert(file_path.to_string(), dep_set.clone());

        // Update reverse dependencies
        for dep in dep_set {
            tracker
                .reverse_dependencies
                .entry(dep)
                .or_insert_with(HashSet::new)
                .insert(file_path.to_string());
        }

        tracker
            .last_update
            .insert(file_path.to_string(), Instant::now());

        Ok(())
    }

    /// Remove file from dependencies
    fn remove_file_dependencies(&self, file_path: &str) -> Result<()> {
        let mut tracker = self.dependency_tracker.write();

        // Remove forward dependencies
        if let Some(deps) = tracker.file_dependencies.remove(file_path) {
            // Remove from reverse dependencies
            for dep in deps {
                if let Some(rev_deps) = tracker.reverse_dependencies.get_mut(&dep) {
                    rev_deps.remove(file_path);
                }
            }
        }

        // Remove as a dependency of others
        tracker.reverse_dependencies.remove(file_path);

        tracker.last_update.remove(file_path);

        Ok(())
    }

    /// Find files that depend on the given file
    fn find_dependent_files(&self, file_path: &str) -> Result<Vec<String>> {
        let tracker = self.dependency_tracker.read();

        Ok(tracker
            .reverse_dependencies
            .get(file_path)
            .map(|deps| deps.iter().cloned().collect())
            .unwrap_or_default())
    }

    /// Check if processing is needed
    pub fn needs_processing(&self) -> bool {
        let buffer = self.change_buffer.read();
        !buffer.pending_changes.is_empty()
    }

    /// Get pending change count
    pub fn pending_changes_count(&self) -> usize {
        self.change_buffer.read().pending_changes.len()
    }

    /// Clear all pending changes
    pub fn clear_pending_changes(&self) {
        self.change_buffer.write().pending_changes.clear();
    }

    /// Get update statistics
    pub fn get_statistics(&self) -> UpdateStatistics {
        // Would track these metrics in real implementation
        UpdateStatistics {
            total_updates: 0,
            average_update_time_ms: 0.0,
            cache_hit_rate: 0.75,
            cascade_ratio: 0.3,
        }
    }

    /// Start background processing (if enabled)
    pub fn start_background_processing(&self) -> Result<()> {
        if !self.update_config.background_processing {
            return Ok(());
        }

        // Would spawn background thread to process changes
        // For now, just log
        log::info!("Background processing would be started here");
        Ok(())
    }

    /// Stop background processing
    pub fn stop_background_processing(&self) -> Result<()> {
        log::info!("Background processing would be stopped here");
        Ok(())
    }
}

/// Builder for incremental updater
pub struct IncrementalUpdaterBuilder {
    knowledge_graph: Arc<KnowledgeGraph>,
    symbol_resolver: Arc<SymbolResolver>,
    config: UpdateConfig,
}

impl IncrementalUpdaterBuilder {
    pub fn new(knowledge_graph: Arc<KnowledgeGraph>, symbol_resolver: Arc<SymbolResolver>) -> Self {
        Self {
            knowledge_graph,
            symbol_resolver,
            config: UpdateConfig::default(),
        }
    }

    pub fn batch_size(mut self, size: usize) -> Self {
        self.config.batch_size = size;
        self
    }

    pub fn debounce_ms(mut self, ms: u64) -> Self {
        self.config.debounce_ms = ms;
        self
    }

    pub fn max_cascade_depth(mut self, depth: usize) -> Self {
        self.config.max_cascade_depth = depth;
        self
    }

    pub fn enable_smart_invalidation(mut self, enable: bool) -> Self {
        self.config.enable_smart_invalidation = enable;
        self
    }

    pub fn background_processing(mut self, enable: bool) -> Self {
        self.config.background_processing = enable;
        self
    }

    pub fn build(self) -> IncrementalUpdater {
        IncrementalUpdater::with_config(self.knowledge_graph, self.symbol_resolver, self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_updater_creation() {
        let graph = Arc::new(KnowledgeGraph::new());
        let resolver = Arc::new(SymbolResolver::new(graph.clone()));
        let updater = IncrementalUpdater::new(graph, resolver);

        assert_eq!(updater.pending_changes_count(), 0);
        assert!(!updater.needs_processing());
    }

    #[test]
    fn test_register_change() {
        let graph = Arc::new(KnowledgeGraph::new());
        let resolver = Arc::new(SymbolResolver::new(graph.clone()));
        let updater = IncrementalUpdater::new(graph, resolver);

        let event = FileChangeEvent {
            file_path: "test.ts".to_string(),
            change_type: FileChangeType::Modified,
            timestamp: Instant::now(),
            content_hash: None,
        };

        assert!(updater.register_change(event).is_ok());
        assert_eq!(updater.pending_changes_count(), 1);
        assert!(updater.needs_processing());
    }

    #[test]
    fn test_builder_pattern() {
        let graph = Arc::new(KnowledgeGraph::new());
        let resolver = Arc::new(SymbolResolver::new(graph.clone()));

        let updater = IncrementalUpdaterBuilder::new(graph, resolver)
            .batch_size(20)
            .debounce_ms(200)
            .max_cascade_depth(10)
            .enable_smart_invalidation(true)
            .build();

        assert_eq!(updater.update_config.batch_size, 20);
        assert_eq!(updater.update_config.debounce_ms, 200);
    }
}
