/**
 * Incremental Update Manager - Efficient incremental semantic context updates
 * 
 * This module handles incremental updates to semantic context when files change,
 * avoiding full recomputation and maintaining consistency across the system.
 */

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock, Mutex};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::time::sleep;

use crate::types::{SemanticContext, FileChange, ChangeType, DependencyGraph, RelationshipGraph};
use crate::cache::ContextCache;
use crate::parsers::LanguageParser;
use crate::error::CheckpointError;

/// Manages incremental updates to semantic context
pub struct IncrementalUpdateManager {
    // Core components
    language_parsers: HashMap<String, Arc<dyn LanguageParser>>,
    context_cache: Arc<ContextCache>,
    dependency_tracker: Arc<RwLock<DependencyTracker>>,
    
    // Update processing
    update_queue: Arc<Mutex<UpdateQueue>>,
    processing_state: Arc<RwLock<ProcessingState>>,
    
    // Change monitoring
    file_monitor: Option<FileMonitor>,
    change_detector: ChangeDetector,
    
    // Configuration
    config: IncrementalConfig,
    
    // Statistics
    stats: Arc<Mutex<UpdateStatistics>>,
}

impl IncrementalUpdateManager {
    pub fn new(
        language_parsers: HashMap<String, Arc<dyn LanguageParser>>,
        context_cache: Arc<ContextCache>,
        config: IncrementalConfig
    ) -> Self {
        Self {
            language_parsers,
            context_cache,
            dependency_tracker: Arc::new(RwLock::new(DependencyTracker::new())),
            update_queue: Arc::new(Mutex::new(UpdateQueue::new())),
            processing_state: Arc::new(RwLock::new(ProcessingState::new())),
            file_monitor: None,
            change_detector: ChangeDetector::new(),
            config,
            stats: Arc::new(Mutex::new(UpdateStatistics::default())),
        }
    }

    /// Start the incremental update manager
    pub async fn start(&mut self) -> Result<(), CheckpointError> {
        // Start file monitoring if enabled
        if self.config.enable_file_monitoring {
            self.file_monitor = Some(FileMonitor::new(self.config.clone()).await?);
        }

        // Start update processing loop
        self.start_update_processor().await?;

        Ok(())
    }

    /// Process a file change incrementally
    pub async fn process_file_change(
        &self,
        file_change: &FileChange,
        workspace_context: &SemanticContext
    ) -> Result<IncrementalUpdateResult, CheckpointError> {
        let start_time = Instant::now();

        // Determine the scope of changes needed
        let change_scope = self.analyze_change_scope(file_change, workspace_context).await?;

        // Create update operation
        let update_op = UpdateOperation {
            id: self.generate_update_id(),
            file_change: file_change.clone(),
            change_scope,
            created_at: Instant::now(),
            priority: self.calculate_update_priority(file_change),
            retry_count: 0,
        };

        // Queue the update
        self.queue_update(update_op.clone()).await?;

        // Process immediately if high priority, otherwise let background processor handle it
        let result = if update_op.priority >= self.config.immediate_processing_threshold {
            self.process_update_operation(&update_op, workspace_context).await?
        } else {
            // Return pending result, background processor will handle it
            IncrementalUpdateResult {
                operation_id: update_op.id.clone(),
                processed_files: vec![file_change.path.clone()],
                affected_dependencies: HashSet::new(),
                cache_invalidations: HashSet::new(),
                processing_time: Duration::from_millis(0),
                status: UpdateStatus::Queued,
                semantic_changes: SemanticChanges::default(),
            }
        };

        // Update statistics
        self.record_update_stats(&result, start_time.elapsed()).await;

        Ok(result)
    }

    /// Process multiple file changes in batch
    pub async fn process_batch_changes(
        &self,
        file_changes: &[FileChange],
        workspace_context: &SemanticContext
    ) -> Result<BatchUpdateResult, CheckpointError> {
        let start_time = Instant::now();

        // Group changes by dependency relationships to optimize processing order
        let grouped_changes = self.group_changes_by_dependencies(file_changes, workspace_context).await?;

        let mut batch_result = BatchUpdateResult {
            operation_id: self.generate_update_id(),
            individual_results: Vec::new(),
            total_processing_time: Duration::from_millis(0),
            overall_status: UpdateStatus::InProgress,
        };

        // Process each group
        for group in grouped_changes {
            for file_change in group.changes {
                let result = self.process_file_change(&file_change, workspace_context).await?;
                batch_result.individual_results.push(result);
            }
        }

        batch_result.total_processing_time = start_time.elapsed();
        batch_result.overall_status = if batch_result.individual_results.iter().all(|r| r.status == UpdateStatus::Completed) {
            UpdateStatus::Completed
        } else {
            UpdateStatus::Failed
        };

        Ok(batch_result)
    }

    /// Get pending updates count
    pub async fn get_pending_updates_count(&self) -> usize {
        if let Ok(queue) = self.update_queue.lock() {
            queue.operations.len()
        } else {
            0
        }
    }

    /// Get update statistics
    pub async fn get_statistics(&self) -> UpdateStatistics {
        if let Ok(stats) = self.stats.lock() {
            stats.clone()
        } else {
            UpdateStatistics::default()
        }
    }

    /// Force process all pending updates
    pub async fn flush_pending_updates(&self, workspace_context: &SemanticContext) -> Result<(), CheckpointError> {
        let pending_operations = {
            if let Ok(mut queue) = self.update_queue.lock() {
                let ops = queue.operations.clone();
                queue.operations.clear();
                ops
            } else {
                return Err(CheckpointError::LockError("Failed to acquire update queue lock".to_string()));
            }
        };

        for operation in pending_operations {
            self.process_update_operation(&operation, workspace_context).await?;
        }

        Ok(())
    }

    // Private implementation methods

    async fn analyze_change_scope(
        &self,
        file_change: &FileChange,
        workspace_context: &SemanticContext
    ) -> Result<ChangeScope, CheckpointError> {
        let mut scope = ChangeScope {
            direct_file: file_change.path.clone(),
            affected_files: HashSet::new(),
            dependency_changes: HashSet::new(),
            semantic_impact: SemanticImpact::Low,
        };

        match file_change.change_type {
            ChangeType::Created => {
                scope.semantic_impact = SemanticImpact::Medium;
                scope.affected_files = self.find_files_importing(&file_change.path, workspace_context).await?;
            }
            ChangeType::Modified => {
                scope.semantic_impact = self.assess_modification_impact(file_change, workspace_context).await?;
                scope.affected_files = self.find_dependent_files(&file_change.path, workspace_context).await?;
            }
            ChangeType::Deleted => {
                scope.semantic_impact = SemanticImpact::High;
                scope.affected_files = self.find_files_importing(&file_change.path, workspace_context).await?;
            }
            _ => {}
        }

        // Analyze dependency changes
        scope.dependency_changes = self.analyze_dependency_changes(file_change, workspace_context).await?;

        Ok(scope)
    }

    async fn assess_modification_impact(
        &self,
        file_change: &FileChange,
        workspace_context: &SemanticContext
    ) -> Result<SemanticImpact, CheckpointError> {
        // Parse the old and new content to determine semantic changes
        let old_content = file_change.old_content.as_ref().unwrap_or(&String::new());
        let new_content = file_change.new_content.as_ref().unwrap_or(&String::new());

        // Get appropriate language parser
        let parser = self.get_parser_for_file(&file_change.path)?;

        // Parse both versions
        let old_ast = parser.parse_file(old_content, &file_change.path)?;
        let new_ast = parser.parse_file(new_content, &file_change.path)?;

        // Compare symbols
        let old_symbols = parser.extract_symbols(&old_ast);
        let new_symbols = parser.extract_symbols(&new_ast);

        // Analyze the differences
        let symbol_changes = self.compare_symbols(&old_symbols, &new_symbols);

        // Determine impact based on changes
        if symbol_changes.has_signature_changes() {
            Ok(SemanticImpact::High)
        } else if symbol_changes.has_structural_changes() {
            Ok(SemanticImpact::Medium)
        } else {
            Ok(SemanticImpact::Low)
        }
    }

    async fn find_dependent_files(
        &self,
        file_path: &PathBuf,
        workspace_context: &SemanticContext
    ) -> Result<HashSet<PathBuf>, CheckpointError> {
        let mut dependents = HashSet::new();

        if let Ok(tracker) = self.dependency_tracker.read() {
            if let Some(deps) = tracker.get_dependents(file_path) {
                dependents.extend(deps.clone());
            }
        }

        Ok(dependents)
    }

    async fn find_files_importing(
        &self,
        file_path: &PathBuf,
        workspace_context: &SemanticContext
    ) -> Result<HashSet<PathBuf>, CheckpointError> {
        let mut importing_files = HashSet::new();

        // Search through workspace context for import statements
        for (path, context) in &workspace_context.file_contexts {
            for import in &context.imports {
                if self.import_references_file(&import.module_path, file_path) {
                    importing_files.insert(path.clone());
                }
            }
        }

        Ok(importing_files)
    }

    async fn analyze_dependency_changes(
        &self,
        file_change: &FileChange,
        workspace_context: &SemanticContext
    ) -> Result<HashSet<DependencyChange>, CheckpointError> {
        let mut dependency_changes = HashSet::new();

        if let Some(new_content) = &file_change.new_content {
            let parser = self.get_parser_for_file(&file_change.path)?;
            let ast = parser.parse_file(new_content, &file_change.path)?;
            let dependencies = parser.analyze_dependencies(&ast);

            // Compare with previous dependencies if available
            if let Some(old_content) = &file_change.old_content {
                let old_ast = parser.parse_file(old_content, &file_change.path)?;
                let old_dependencies = parser.analyze_dependencies(&old_ast);

                // Find added dependencies
                for dep in &dependencies {
                    if !old_dependencies.contains(dep) {
                        dependency_changes.insert(DependencyChange {
                            change_type: DependencyChangeType::Added,
                            source: file_change.path.clone(),
                            target: PathBuf::from(&dep.target),
                            dependency_type: dep.dependency_type.clone(),
                        });
                    }
                }

                // Find removed dependencies
                for dep in &old_dependencies {
                    if !dependencies.contains(dep) {
                        dependency_changes.insert(DependencyChange {
                            change_type: DependencyChangeType::Removed,
                            source: file_change.path.clone(),
                            target: PathBuf::from(&dep.target),
                            dependency_type: dep.dependency_type.clone(),
                        });
                    }
                }
            }
        }

        Ok(dependency_changes)
    }

    async fn process_update_operation(
        &self,
        operation: &UpdateOperation,
        workspace_context: &SemanticContext
    ) -> Result<IncrementalUpdateResult, CheckpointError> {
        let start_time = Instant::now();

        // Mark as in progress
        self.set_processing_status(&operation.id, UpdateStatus::InProgress).await;

        let mut result = IncrementalUpdateResult {
            operation_id: operation.id.clone(),
            processed_files: vec![operation.file_change.path.clone()],
            affected_dependencies: HashSet::new(),
            cache_invalidations: HashSet::new(),
            processing_time: Duration::from_millis(0),
            status: UpdateStatus::InProgress,
            semantic_changes: SemanticChanges::default(),
        };

        // Process the main file change
        match self.update_semantic_context_for_file(&operation.file_change, workspace_context).await {
            Ok(changes) => {
                result.semantic_changes = changes;

                // Invalidate relevant caches
                self.context_cache.invalidate_file_caches(&operation.file_change.path.to_string_lossy());
                result.cache_invalidations.insert(operation.file_change.path.clone());

                // Process affected files
                for affected_file in &operation.change_scope.affected_files {
                    if let Err(e) = self.update_dependent_file(affected_file, &operation.file_change).await {
                        println!("Warning: Failed to update dependent file {:?}: {}", affected_file, e);
                    } else {
                        result.processed_files.push(affected_file.clone());
                        result.cache_invalidations.insert(affected_file.clone());
                    }
                }

                // Update dependency tracking
                if let Ok(mut tracker) = self.dependency_tracker.write() {
                    for dep_change in &operation.change_scope.dependency_changes {
                        tracker.update_dependency(dep_change);
                    }
                }

                result.affected_dependencies = operation.change_scope.dependency_changes.clone();
                result.status = UpdateStatus::Completed;
            }
            Err(e) => {
                result.status = UpdateStatus::Failed;
                return Err(e);
            }
        }

        result.processing_time = start_time.elapsed();
        self.set_processing_status(&operation.id, result.status.clone()).await;

        Ok(result)
    }

    async fn update_semantic_context_for_file(
        &self,
        file_change: &FileChange,
        workspace_context: &SemanticContext
    ) -> Result<SemanticChanges, CheckpointError> {
        let mut changes = SemanticChanges::default();

        if let Some(new_content) = &file_change.new_content {
            let parser = self.get_parser_for_file(&file_change.path)?;
            
            // Parse the new content
            let ast = parser.parse_file(new_content, &file_change.path)?;
            let new_symbols = parser.extract_symbols(&ast);
            let new_dependencies = parser.analyze_dependencies(&ast);

            // Compare with old symbols if available
            if let Some(old_content) = &file_change.old_content {
                let old_ast = parser.parse_file(old_content, &file_change.path)?;
                let old_symbols = parser.extract_symbols(&old_ast);

                changes = self.compare_symbols(&old_symbols, &new_symbols);
            }

            // Update the semantic context
            // This would update the actual semantic context in the workspace
            // For now, we'll just record the changes
        }

        Ok(changes)
    }

    async fn update_dependent_file(
        &self,
        dependent_file: &PathBuf,
        original_change: &FileChange
    ) -> Result<(), CheckpointError> {
        // Update semantic context for dependent files
        // This might involve re-analyzing imports, updating call graphs, etc.
        
        // Invalidate caches for the dependent file
        self.context_cache.invalidate_file_caches(&dependent_file.to_string_lossy());

        Ok(())
    }

    async fn group_changes_by_dependencies(
        &self,
        file_changes: &[FileChange],
        workspace_context: &SemanticContext
    ) -> Result<Vec<ChangeGroup>, CheckpointError> {
        let mut groups = Vec::new();
        let mut processed = HashSet::new();

        for file_change in file_changes {
            if processed.contains(&file_change.path) {
                continue;
            }

            let mut group = ChangeGroup {
                changes: vec![file_change.clone()],
                dependency_level: 0,
            };

            // Find all changes that depend on this one
            let dependents = self.find_dependent_files(&file_change.path, workspace_context).await?;
            
            for dependent in dependents {
                if let Some(dependent_change) = file_changes.iter().find(|c| c.path == dependent) {
                    if !processed.contains(&dependent) {
                        group.changes.push(dependent_change.clone());
                        processed.insert(dependent.clone());
                    }
                }
            }

            processed.insert(file_change.path.clone());
            groups.push(group);
        }

        // Sort groups by dependency level (independent files first)
        groups.sort_by_key(|g| g.dependency_level);

        Ok(groups)
    }

    async fn queue_update(&self, operation: UpdateOperation) -> Result<(), CheckpointError> {
        if let Ok(mut queue) = self.update_queue.lock() {
            queue.add_operation(operation);
            Ok(())
        } else {
            Err(CheckpointError::LockError("Failed to acquire update queue lock".to_string()))
        }
    }

    async fn start_update_processor(&self) -> Result<(), CheckpointError> {
        // Start background task to process queued updates
        let queue = Arc::clone(&self.update_queue);
        let processing_state = Arc::clone(&self.processing_state);
        let config = self.config.clone();

        tokio::spawn(async move {
            loop {
                // Process queued updates
                let operations = {
                    if let Ok(mut queue_guard) = queue.lock() {
                        let ops = queue_guard.get_next_batch(config.batch_size);
                        for op in &ops {
                            queue_guard.remove_operation(&op.id);
                        }
                        ops
                    } else {
                        Vec::new()
                    }
                };

                if operations.is_empty() {
                    sleep(Duration::from_millis(config.processing_interval_ms)).await;
                    continue;
                }

                // Process each operation
                for operation in operations {
                    // This would need access to workspace_context
                    // In a real implementation, this would be handled differently
                    println!("Processing queued update: {}", operation.id);
                }

                sleep(Duration::from_millis(config.processing_interval_ms)).await;
            }
        });

        Ok(())
    }

    // Helper methods

    fn get_parser_for_file(&self, file_path: &PathBuf) -> Result<&Arc<dyn LanguageParser>, CheckpointError> {
        let extension = file_path.extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| CheckpointError::ParseError("No file extension found".to_string()))?;

        self.language_parsers.get(extension)
            .ok_or_else(|| CheckpointError::UnsupportedLanguage(extension.to_string()))
    }

    fn compare_symbols(&self, old_symbols: &[crate::parsers::Symbol], new_symbols: &[crate::parsers::Symbol]) -> SemanticChanges {
        let mut changes = SemanticChanges::default();

        // Find added symbols
        for new_symbol in new_symbols {
            if !old_symbols.iter().any(|old| old.name() == new_symbol.name()) {
                changes.added_symbols.push(new_symbol.name().to_string());
            }
        }

        // Find removed symbols
        for old_symbol in old_symbols {
            if !new_symbols.iter().any(|new| new.name() == old_symbol.name()) {
                changes.removed_symbols.push(old_symbol.name().to_string());
            }
        }

        // Find modified symbols (simplified check)
        for new_symbol in new_symbols {
            if let Some(old_symbol) = old_symbols.iter().find(|old| old.name() == new_symbol.name()) {
                // This is a simplified comparison - in practice would be more sophisticated
                if std::mem::discriminant(old_symbol) != std::mem::discriminant(new_symbol) {
                    changes.modified_symbols.push(new_symbol.name().to_string());
                }
            }
        }

        changes
    }

    fn import_references_file(&self, import_path: &str, file_path: &PathBuf) -> bool {
        // Simplified check - in practice would handle relative paths, module resolution, etc.
        import_path.contains(&file_path.to_string_lossy().to_string())
    }

    fn generate_update_id(&self) -> String {
        format!("update_{}", chrono::Utc::now().timestamp_millis())
    }

    fn calculate_update_priority(&self, file_change: &FileChange) -> u32 {
        match file_change.change_type {
            ChangeType::Created | ChangeType::Deleted => 100, // High priority
            ChangeType::Modified => 50, // Medium priority
            _ => 10, // Low priority
        }
    }

    async fn set_processing_status(&self, operation_id: &str, status: UpdateStatus) {
        if let Ok(mut state) = self.processing_state.write() {
            state.set_status(operation_id, status);
        }
    }

    async fn record_update_stats(&self, result: &IncrementalUpdateResult, processing_time: Duration) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.total_updates += 1;
            stats.total_processing_time += processing_time;
            
            match result.status {
                UpdateStatus::Completed => stats.successful_updates += 1,
                UpdateStatus::Failed => stats.failed_updates += 1,
                _ => {}
            }

            stats.total_files_processed += result.processed_files.len() as u64;
            stats.total_cache_invalidations += result.cache_invalidations.len() as u64;
        }
    }
}

// Supporting data structures and types

#[derive(Debug, Clone)]
struct UpdateOperation {
    id: String,
    file_change: FileChange,
    change_scope: ChangeScope,
    created_at: Instant,
    priority: u32,
    retry_count: u32,
}

#[derive(Debug, Clone)]
struct ChangeScope {
    direct_file: PathBuf,
    affected_files: HashSet<PathBuf>,
    dependency_changes: HashSet<DependencyChange>,
    semantic_impact: SemanticImpact,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct DependencyChange {
    change_type: DependencyChangeType,
    source: PathBuf,
    target: PathBuf,
    dependency_type: crate::parsers::DependencyType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum DependencyChangeType {
    Added,
    Removed,
    Modified,
}

#[derive(Debug, Clone)]
enum SemanticImpact {
    Low,    // Comments, formatting, minor changes
    Medium, // New functions, classes, but no signature changes
    High,   // Signature changes, deletions, major structural changes
}

#[derive(Debug, Clone, Default)]
struct SemanticChanges {
    added_symbols: Vec<String>,
    removed_symbols: Vec<String>,
    modified_symbols: Vec<String>,
    signature_changes: Vec<String>,
    structural_changes: Vec<String>,
}

impl SemanticChanges {
    fn has_signature_changes(&self) -> bool {
        !self.signature_changes.is_empty()
    }

    fn has_structural_changes(&self) -> bool {
        !self.structural_changes.is_empty() || !self.added_symbols.is_empty() || !self.removed_symbols.is_empty()
    }
}

struct UpdateQueue {
    operations: Vec<UpdateOperation>,
}

impl UpdateQueue {
    fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    fn add_operation(&mut self, operation: UpdateOperation) {
        // Insert based on priority (higher priority first)
        let insert_pos = self.operations
            .iter()
            .position(|op| op.priority < operation.priority)
            .unwrap_or(self.operations.len());
        
        self.operations.insert(insert_pos, operation);
    }

    fn get_next_batch(&self, batch_size: usize) -> Vec<UpdateOperation> {
        self.operations.iter().take(batch_size).cloned().collect()
    }

    fn remove_operation(&mut self, operation_id: &str) {
        self.operations.retain(|op| op.id != operation_id);
    }
}

struct ProcessingState {
    status_map: HashMap<String, UpdateStatus>,
}

impl ProcessingState {
    fn new() -> Self {
        Self {
            status_map: HashMap::new(),
        }
    }

    fn set_status(&mut self, operation_id: &str, status: UpdateStatus) {
        self.status_map.insert(operation_id.to_string(), status);
    }

    fn get_status(&self, operation_id: &str) -> Option<&UpdateStatus> {
        self.status_map.get(operation_id)
    }
}

struct DependencyTracker {
    dependencies: HashMap<PathBuf, HashSet<PathBuf>>, // file -> files it depends on
    dependents: HashMap<PathBuf, HashSet<PathBuf>>,   // file -> files that depend on it
}

impl DependencyTracker {
    fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
        }
    }

    fn update_dependency(&mut self, change: &DependencyChange) {
        match change.change_type {
            DependencyChangeType::Added => {
                self.dependencies
                    .entry(change.source.clone())
                    .or_insert_with(HashSet::new)
                    .insert(change.target.clone());
                
                self.dependents
                    .entry(change.target.clone())
                    .or_insert_with(HashSet::new)
                    .insert(change.source.clone());
            }
            DependencyChangeType::Removed => {
                if let Some(deps) = self.dependencies.get_mut(&change.source) {
                    deps.remove(&change.target);
                }
                if let Some(deps) = self.dependents.get_mut(&change.target) {
                    deps.remove(&change.source);
                }
            }
            DependencyChangeType::Modified => {
                // For modified dependencies, we might need to re-analyze
                // For now, treat as no-op
            }
        }
    }

    fn get_dependents(&self, file: &PathBuf) -> Option<&HashSet<PathBuf>> {
        self.dependents.get(file)
    }

    fn get_dependencies(&self, file: &PathBuf) -> Option<&HashSet<PathBuf>> {
        self.dependencies.get(file)
    }
}

struct FileMonitor {
    // File system monitoring would be implemented here
    // Using something like notify-rs for real file system events
}

impl FileMonitor {
    async fn new(_config: IncrementalConfig) -> Result<Self, CheckpointError> {
        // Initialize file system monitoring
        Ok(Self {})
    }
}

struct ChangeDetector {
    // Change detection logic
}

impl ChangeDetector {
    fn new() -> Self {
        Self {}
    }
}

struct ChangeGroup {
    changes: Vec<FileChange>,
    dependency_level: u32,
}

// Public result types

#[derive(Debug, Clone)]
pub struct IncrementalUpdateResult {
    pub operation_id: String,
    pub processed_files: Vec<PathBuf>,
    pub affected_dependencies: HashSet<DependencyChange>,
    pub cache_invalidations: HashSet<PathBuf>,
    pub processing_time: Duration,
    pub status: UpdateStatus,
    pub semantic_changes: SemanticChanges,
}

#[derive(Debug, Clone)]
pub struct BatchUpdateResult {
    pub operation_id: String,
    pub individual_results: Vec<IncrementalUpdateResult>,
    pub total_processing_time: Duration,
    pub overall_status: UpdateStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UpdateStatus {
    Queued,
    InProgress,
    Completed,
    Failed,
}

// Configuration

#[derive(Debug, Clone)]
pub struct IncrementalConfig {
    pub enable_file_monitoring: bool,
    pub immediate_processing_threshold: u32,
    pub batch_size: usize,
    pub processing_interval_ms: u64,
    pub max_retry_attempts: u32,
    pub dependency_analysis_depth: u32,
}

impl Default for IncrementalConfig {
    fn default() -> Self {
        Self {
            enable_file_monitoring: true,
            immediate_processing_threshold: 80,
            batch_size: 10,
            processing_interval_ms: 1000,
            max_retry_attempts: 3,
            dependency_analysis_depth: 5,
        }
    }
}

// Statistics

#[derive(Debug, Clone, Default)]
pub struct UpdateStatistics {
    pub total_updates: u64,
    pub successful_updates: u64,
    pub failed_updates: u64,
    pub total_processing_time: Duration,
    pub total_files_processed: u64,
    pub total_cache_invalidations: u64,
    pub average_processing_time: Duration,
}

impl UpdateStatistics {
    pub fn calculate_averages(&mut self) {
        if self.total_updates > 0 {
            self.average_processing_time = self.total_processing_time / self.total_updates as u32;
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_updates > 0 {
            self.successful_updates as f64 / self.total_updates as f64
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_queue_priority() {
        let mut queue = UpdateQueue::new();
        
        let low_priority_op = UpdateOperation {
            id: "low".to_string(),
            file_change: FileChange::default(),
            change_scope: ChangeScope {
                direct_file: PathBuf::new(),
                affected_files: HashSet::new(),
                dependency_changes: HashSet::new(),
                semantic_impact: SemanticImpact::Low,
            },
            created_at: Instant::now(),
            priority: 10,
            retry_count: 0,
        };

        let high_priority_op = UpdateOperation {
            id: "high".to_string(),
            file_change: FileChange::default(),
            change_scope: ChangeScope {
                direct_file: PathBuf::new(),
                affected_files: HashSet::new(),
                dependency_changes: HashSet::new(),
                semantic_impact: SemanticImpact::High,
            },
            created_at: Instant::now(),
            priority: 100,
            retry_count: 0,
        };

        queue.add_operation(low_priority_op);
        queue.add_operation(high_priority_op);

        let batch = queue.get_next_batch(1);
        assert_eq!(batch[0].id, "high");
    }

    #[test]
    fn test_semantic_changes() {
        let mut changes = SemanticChanges::default();
        changes.added_symbols.push("newFunction".to_string());
        changes.signature_changes.push("existingFunction".to_string());

        assert!(changes.has_structural_changes());
        assert!(changes.has_signature_changes());
    }
}
