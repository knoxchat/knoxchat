//! Context Building Pipeline
//!
//! Core pipeline for building AI context from checkpoints and semantic analysis

use super::*;
use crate::manager::CheckpointManager;
use crate::semantic::types::IntentAnalysis;
use crate::semantic::{AIContextCheckpoint, SemanticAnalyzer, SemanticContext};

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
// use tokio::sync::RwLock;

/// Options for context building
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextOptions {
    /// Maximum tokens in the generated context
    pub max_tokens: usize,
    /// Include evolution/history context
    pub include_evolution: bool,
    /// Include code examples
    pub include_examples: bool,
    /// Priority threshold for relevance filtering
    pub priority_threshold: f64,
    /// Enable parallel processing
    pub enable_parallel: bool,
    /// Specific files to focus on
    pub focus_files: Option<Vec<String>>,
    /// Exclude certain patterns
    pub exclude_patterns: Option<Vec<String>>,
}

impl Default for ContextOptions {
    fn default() -> Self {
        Self {
            max_tokens: 16000,
            include_evolution: true,
            include_examples: true,
            priority_threshold: 0.3,
            enable_parallel: true,
            focus_files: None,
            exclude_patterns: None,
        }
    }
}

/// Complete AI context structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteAIContext {
    /// Core files with semantic information
    pub core_files: Vec<ContextFile>,
    /// Architectural context
    pub architecture: ArchitecturalContext,
    /// Relationship context
    pub relationships: RelationshipContext,
    /// Historical context
    pub history: HistoryContext,
    /// Code examples
    pub examples: Vec<ExampleContext>,
    /// Context metadata
    pub metadata: ContextBuildMetadata,
}

/// Individual file context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextFile {
    /// File path relative to workspace
    pub path: String,
    /// Complete file content
    pub content: String,
    /// Programming language
    pub language: String,
    /// File encoding
    pub encoding: String,
    /// File size in bytes
    pub size_bytes: usize,
    /// Last modification time
    pub last_modified: chrono::DateTime<chrono::Utc>,
    /// Semantic information extracted from file
    pub semantic_info: FileSemanticInfo,
    /// Relevance score for this query
    pub relevance_score: f64,
}

/// Semantic information for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSemanticInfo {
    /// Function definitions
    pub functions: Vec<FunctionInfo>,
    /// Class definitions
    pub classes: Vec<ClassInfo>,
    /// Interface definitions
    pub interfaces: Vec<InterfaceInfo>,
    /// Type definitions
    pub types: Vec<TypeInfo>,
    /// Import statements
    pub imports: Vec<ImportInfo>,
    /// Export statements
    pub exports: Vec<ExportInfo>,
    /// Complexity metrics
    pub complexity_metrics: ComplexityMetrics,
}

/// Function information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub parameters: Vec<ParameterInfo>,
    pub return_type: Option<String>,
    pub visibility: String,
    pub is_async: bool,
    pub documentation: Option<String>,
    pub complexity: u32,
    pub calls: Vec<String>,
    pub called_by: Vec<String>,
    pub start_line: u32,
    pub end_line: u32,
}

/// Class information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassInfo {
    pub name: String,
    pub extends: Option<String>,
    pub implements: Vec<String>,
    pub methods: Vec<String>,
    pub properties: Vec<PropertyInfo>,
    pub visibility: String,
    pub design_patterns: Vec<String>,
    pub start_line: u32,
    pub end_line: u32,
}

/// Interface information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceInfo {
    pub name: String,
    pub extends: Vec<String>,
    pub methods: Vec<MethodSignature>,
    pub properties: Vec<PropertySignature>,
    pub start_line: u32,
    pub end_line: u32,
}

/// Type information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeInfo {
    pub name: String,
    pub kind: String,
    pub definition: String,
    pub generic_parameters: Vec<String>,
    pub start_line: u32,
    pub end_line: u32,
}

/// Import information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportInfo {
    pub module: String,
    pub imported_items: Vec<String>,
    pub alias: Option<String>,
    pub is_default: bool,
    pub is_namespace: bool,
    pub line_number: u32,
}

/// Export information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportInfo {
    pub exported_items: Vec<String>,
    pub is_default: bool,
    pub alias: Option<String>,
    pub line_number: u32,
}

/// Parameter information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterInfo {
    pub name: String,
    pub param_type: String,
    pub is_optional: bool,
    pub default_value: Option<String>,
}

/// Property information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyInfo {
    pub name: String,
    pub property_type: String,
    pub visibility: String,
    pub is_static: bool,
    pub is_readonly: bool,
}

/// Method signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodSignature {
    pub name: String,
    pub parameters: Vec<ParameterInfo>,
    pub return_type: Option<String>,
    pub is_async: bool,
}

/// Property signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertySignature {
    pub name: String,
    pub property_type: String,
    pub is_optional: bool,
    pub is_readonly: bool,
}

/// Complexity metrics for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    /// Cyclomatic complexity
    pub cyclomatic_complexity: u32,
    /// Lines of code
    pub lines_of_code: u32,
    /// Number of functions
    pub function_count: u32,
    /// Number of classes
    pub class_count: u32,
    /// Average function length
    pub average_function_length: f64,
}

/// Architectural context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalContext {
    /// Project structure overview
    pub project_structure: ProjectStructure,
    /// Design patterns used
    pub patterns_used: Vec<DesignPattern>,
    /// Dependency graph
    pub dependency_graph: DependencyGraph,
    /// Architectural layers
    pub layers: Vec<ArchitecturalLayer>,
}

/// Project structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStructure {
    /// Root directories
    pub root_directories: Vec<String>,
    /// Module structure
    pub modules: Vec<ModuleInfo>,
    /// Package dependencies
    pub dependencies: Vec<DependencyInfo>,
}

/// Module information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub path: String,
    pub module_type: String,
    pub exports: Vec<String>,
    pub dependencies: Vec<String>,
}

/// Dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInfo {
    pub name: String,
    pub version: String,
    pub dependency_type: DependencyType,
}

/// Dependency type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    Production,
    Development,
    Peer,
    Optional,
}

/// Design pattern information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignPattern {
    pub name: String,
    pub pattern_type: PatternType,
    pub description: String,
    pub locations: Vec<String>,
    pub confidence: f64,
}

/// Pattern type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    Creational,
    Structural,
    Behavioral,
    Architectural,
}

/// Dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub nodes: Vec<DependencyNode>,
    pub edges: Vec<DependencyEdge>,
    pub cycles: Vec<Vec<String>>,
}

/// Dependency graph node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyNode {
    pub id: String,
    pub node_type: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Dependency graph edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    pub from: String,
    pub to: String,
    pub edge_type: String,
    pub strength: f64,
}

/// Architectural layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalLayer {
    pub name: String,
    pub layer_type: String,
    pub components: Vec<String>,
    pub responsibilities: Vec<String>,
}

/// Relationship context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipContext {
    /// Call graph
    pub call_graph: CallGraph,
    /// Type hierarchy
    pub type_hierarchy: TypeHierarchy,
    /// Import graph
    pub import_graph: ImportGraph,
    /// Usage patterns
    pub usage_patterns: Vec<UsagePattern>,
}

/// Call graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallGraph {
    pub functions: Vec<CallGraphNode>,
    pub relationships: Vec<CallRelationship>,
}

/// Call graph node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallGraphNode {
    pub name: String,
    pub file: String,
    pub complexity: u32,
    pub calls_count: u32,
    pub called_by_count: u32,
}

/// Call relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallRelationship {
    pub caller: String,
    pub called: String,
    pub call_type: String,
    pub frequency: u32,
}

/// Type hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeHierarchy {
    pub root_types: Vec<String>,
    pub inheritance_chains: Vec<InheritanceChain>,
    pub interface_implementations: Vec<InterfaceImplementation>,
}

/// Inheritance chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InheritanceChain {
    pub base: String,
    pub derived: Vec<String>,
    pub depth: u32,
}

/// Interface implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceImplementation {
    pub interface: String,
    pub implementations: Vec<String>,
}

/// Import graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportGraph {
    pub modules: Vec<ImportGraphNode>,
    pub dependencies: Vec<ImportDependency>,
}

/// Import graph node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportGraphNode {
    pub module: String,
    pub exports: Vec<String>,
    pub imports: Vec<String>,
}

/// Import dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportDependency {
    pub from: String,
    pub to: String,
    pub imported_items: Vec<String>,
}

/// Usage pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePattern {
    pub pattern: String,
    pub description: String,
    pub frequency: u32,
    pub locations: Vec<String>,
    pub confidence: f64,
}

/// Historical context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryContext {
    /// Timeline of changes
    pub change_timeline: Vec<ChangeEvent>,
    /// Architectural decisions
    pub architectural_decisions: Vec<ArchitecturalDecision>,
    /// Refactoring history
    pub refactoring_history: Vec<RefactoringEvent>,
}

/// Change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub checkpoint_id: String,
    pub description: String,
    pub files_changed: Vec<String>,
    pub change_type: String,
    pub impact: String,
}

/// Architectural decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalDecision {
    pub decision: String,
    pub reasoning: String,
    pub alternatives: Vec<String>,
    pub trade_offs: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Refactoring event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringEvent {
    pub refactoring_type: String,
    pub description: String,
    pub files_affected: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub reasoning: String,
}

/// Example context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleContext {
    pub description: String,
    pub code_example: String,
    pub surrounding_context: String,
    pub applicable_patterns: Vec<String>,
    pub confidence: f64,
}

/// Context build metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBuildMetadata {
    /// Number of checkpoints analyzed
    pub checkpoints_analyzed: usize,
    /// Number of files included
    pub files_included: usize,
    /// Total lines of code included
    pub total_lines_of_code: usize,
    /// Build strategy used
    pub build_strategy: String,
    /// Token estimation
    pub estimated_tokens: usize,
}

/// Context building pipeline
pub struct ContextBuildingPipeline {
    /// Checkpoint manager
    checkpoint_manager: Arc<CheckpointManager>,
    /// Semantic analyzer
    semantic_analyzer: Arc<SemanticAnalyzer>,
    /// Query analyzer
    query_analyzer: Arc<super::QueryAnalyzer>,
    /// Relevance scorer
    relevance_scorer: Arc<super::RelevanceScorer>,
    /// Intent analyzer
    intent_analyzer: Arc<super::IntentAnalyzer>,
    /// Architectural analyzer
    architectural_analyzer: Arc<super::ArchitecturalAnalyzer>,
    /// Performance monitor
    performance_monitor: Arc<super::PerformanceMonitor>,
    /// Configuration
    config: AIContextConfig,
}

impl ContextBuildingPipeline {
    /// Create a new context building pipeline
    pub fn new(
        checkpoint_manager: Arc<CheckpointManager>,
        semantic_analyzer: Arc<SemanticAnalyzer>,
        config: AIContextConfig,
    ) -> Result<Self> {
        let query_analyzer = Arc::new(super::QueryAnalyzer::new()?);
        let relevance_scorer = Arc::new(super::RelevanceScorer::new()?);
        let intent_analyzer = Arc::new(super::IntentAnalyzer::new()?);
        let architectural_analyzer = Arc::new(super::ArchitecturalAnalyzer::new()?);
        let performance_monitor = Arc::new(super::PerformanceMonitor::new(
            config.performance_config.clone(),
        )?);

        Ok(Self {
            checkpoint_manager,
            semantic_analyzer,
            query_analyzer,
            relevance_scorer,
            intent_analyzer,
            architectural_analyzer,
            performance_monitor,
            config,
        })
    }

    /// Build complete AI context for a query
    pub async fn build_context(
        &self,
        query: &str,
        options: ContextOptions,
    ) -> Result<CompleteAIContext> {
        let start_time = Instant::now();
        self.performance_monitor.start_build_timer().await;

        // Step 1: Analyze query intent
        let query_intent = self.query_analyzer.analyze_intent(query).await?;

        // Step 2: Select relevant checkpoints
        let relevant_checkpoints = self.select_relevant_checkpoints(&query_intent).await?;

        // Step 3: Build semantic context
        let semantic_context = self
            .build_semantic_context(&relevant_checkpoints, &options)
            .await?;

        // Step 4: Score and filter content by relevance
        let scored_content = self
            .score_and_filter_content(&semantic_context, &query_intent, &options)
            .await?;

        // Step 5: Build architectural context
        let architectural_context = self.build_architectural_context(&scored_content).await?;

        // Step 6: Build relationship context
        let relationship_context = self.build_relationship_context(&scored_content).await?;

        // Step 7: Build historical context (if enabled)
        let history_context = if options.include_evolution {
            self.build_history_context(&relevant_checkpoints).await?
        } else {
            HistoryContext {
                change_timeline: vec![],
                architectural_decisions: vec![],
                refactoring_history: vec![],
            }
        };

        // Step 8: Generate examples (if enabled)
        let examples = if options.include_examples {
            self.generate_examples(&scored_content, &query_intent)
                .await?
        } else {
            vec![]
        };

        // Step 9: Create core files context
        let core_files = self.create_core_files_context(&scored_content).await?;

        // Step 10: Calculate metadata
        let metadata = self
            .calculate_build_metadata(&core_files, &relevant_checkpoints, &options)
            .await?;

        let build_duration = start_time.elapsed();
        self.performance_monitor
            .record_build_time(build_duration)
            .await;

        Ok(CompleteAIContext {
            core_files,
            architecture: architectural_context,
            relationships: relationship_context,
            history: history_context,
            examples,
            metadata,
        })
    }

    /// Select relevant checkpoints based on query intent
    async fn select_relevant_checkpoints(
        &self,
        query_intent: &super::QueryIntent,
    ) -> Result<Vec<AIContextCheckpoint>> {
        // Get all available checkpoints
        let all_checkpoints = self
            .checkpoint_manager
            .list_checkpoints(None)
            .map_err(|e| {
                AIContextError::context_building_failed(
                    &format!("Failed to list checkpoints: {}", e),
                    "unknown",
                    ContextBuildStage::CheckpointSelection,
                )
            })?;

        // Convert to AI context checkpoints and score relevance
        let mut scored_checkpoints: Vec<(crate::semantic::types::AIContextCheckpoint, f64)> =
            Vec::new();

        for base_checkpoint in all_checkpoints {
            // Create AI context checkpoint from base checkpoint
            let ai_checkpoint = crate::semantic::types::AIContextCheckpoint {
                base_checkpoint: base_checkpoint.clone(),
                semantic_context: crate::semantic::types::SemanticContext::default(),
                intent_analysis: crate::semantic::types::IntentAnalysis::default(),
                architectural_impact: crate::semantic::types::ArchitecturalImpact::default(),
                code_relationships: crate::semantic::types::CodeRelationships::default(),
                confidence_score: 0.5,
            };

            // Score relevance using query intent analysis
            let relevance_score = self
                .calculate_checkpoint_relevance(&ai_checkpoint, query_intent)
                .await
                .unwrap_or(0.0);

            if relevance_score > 0.1 {
                // Only include checkpoints with meaningful relevance
                scored_checkpoints.push((ai_checkpoint, relevance_score));
            }
        }

        // Sort by relevance score (descending)
        scored_checkpoints
            .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored_checkpoints.truncate(20); // Limit to top 20 checkpoints

        // Extract just the checkpoints
        Ok(scored_checkpoints
            .into_iter()
            .map(|(checkpoint, _)| checkpoint)
            .collect())
    }

    /// Calculate relevance score for a checkpoint based on query intent
    async fn calculate_checkpoint_relevance(
        &self,
        checkpoint: &AIContextCheckpoint,
        query_intent: &super::QueryIntent,
    ) -> Result<f64> {
        let mut total_score = 0.0;
        let mut score_count = 0;

        // Score based on entities in query
        for entity in &query_intent.entities {
            let entity_score = match entity.entity_type {
                super::query_analyzer::EntityType::Function => {
                    if checkpoint
                        .base_checkpoint
                        .description
                        .to_lowercase()
                        .contains(&entity.name.to_lowercase())
                    {
                        entity.confidence
                    } else {
                        0.0
                    }
                }
                super::query_analyzer::EntityType::Class => {
                    if checkpoint
                        .base_checkpoint
                        .description
                        .to_lowercase()
                        .contains(&entity.name.to_lowercase())
                    {
                        entity.confidence * 0.8 // Classes are slightly less specific
                    } else {
                        0.0
                    }
                }
                _ => 0.0,
            };

            if entity_score > 0.0 {
                total_score += entity_score;
                score_count += 1;
            }
        }

        // Score based on keywords in query
        let query_lower = query_intent.original_query.to_lowercase();
        let description_lower = checkpoint.base_checkpoint.description.to_lowercase();

        // Simple keyword matching - can be enhanced with TF-IDF
        let keywords: Vec<&str> = query_lower.split_whitespace().collect();
        let mut keyword_matches = 0;

        for keyword in &keywords {
            if keyword.len() > 3 && description_lower.contains(keyword) {
                keyword_matches += 1;
            }
        }

        if keyword_matches > 0 {
            total_score += (keyword_matches as f64 / keywords.len() as f64) * 0.5;
            score_count += 1;
        }

        // Return average score or 0 if no matches
        Ok(if score_count > 0 {
            total_score / score_count as f64
        } else {
            0.0
        })
    }

    /// Build semantic context from checkpoints
    async fn build_semantic_context(
        &self,
        checkpoints: &[AIContextCheckpoint],
        options: &ContextOptions,
    ) -> Result<Vec<(AIContextCheckpoint, SemanticContext)>> {
        if options.enable_parallel && self.config.enable_parallel_processing {
            // Parallel processing
            let results: Result<Vec<_>> = checkpoints
                .par_iter()
                .map(|checkpoint| {
                    // Extract semantic context
                    Ok((checkpoint.clone(), checkpoint.semantic_context.clone()))
                })
                .collect();

            results
        } else {
            // Sequential processing
            let mut results = Vec::new();
            for checkpoint in checkpoints {
                results.push((checkpoint.clone(), checkpoint.semantic_context.clone()));
            }
            Ok(results)
        }
    }

    /// Score and filter content by relevance
    async fn score_and_filter_content(
        &self,
        semantic_contexts: &[(AIContextCheckpoint, SemanticContext)],
        query_intent: &super::QueryIntent,
        options: &ContextOptions,
    ) -> Result<Vec<(AIContextCheckpoint, SemanticContext, f64)>> {
        let mut scored_contexts = Vec::new();

        for (checkpoint, semantic_context) in semantic_contexts {
            let relevance_score = self
                .relevance_scorer
                .score_semantic_relevance(semantic_context, query_intent)
                .await?;

            if relevance_score >= options.priority_threshold {
                scored_contexts.push((
                    checkpoint.clone(),
                    semantic_context.clone(),
                    relevance_score,
                ));
            }
        }

        // Sort by relevance score
        scored_contexts.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        Ok(scored_contexts)
    }

    /// Build architectural context
    async fn build_architectural_context(
        &self,
        scored_content: &[(AIContextCheckpoint, SemanticContext, f64)],
    ) -> Result<ArchitecturalContext> {
        // Analyze architectural patterns across all content
        let patterns = self
            .architectural_analyzer
            .analyze_patterns(scored_content)
            .await?;
        let project_structure = self
            .architectural_analyzer
            .analyze_project_structure(scored_content)
            .await?;
        let dependency_graph = self
            .architectural_analyzer
            .build_dependency_graph(scored_content)
            .await?;
        let layers = self
            .architectural_analyzer
            .identify_layers(scored_content)
            .await?;

        Ok(ArchitecturalContext {
            project_structure,
            patterns_used: patterns,
            dependency_graph,
            layers,
        })
    }

    /// Build relationship context
    async fn build_relationship_context(
        &self,
        scored_content: &[(AIContextCheckpoint, SemanticContext, f64)],
    ) -> Result<RelationshipContext> {
        // Build call graph
        let call_graph = self.build_call_graph(scored_content).await?;

        // Build type hierarchy
        let type_hierarchy = self.build_type_hierarchy(scored_content).await?;

        // Build import graph
        let import_graph = self.build_import_graph(scored_content).await?;

        // Identify usage patterns
        let usage_patterns = self.identify_usage_patterns(scored_content).await?;

        Ok(RelationshipContext {
            call_graph,
            type_hierarchy,
            import_graph,
            usage_patterns,
        })
    }

    /// Build historical context
    async fn build_history_context(
        &self,
        checkpoints: &[AIContextCheckpoint],
    ) -> Result<HistoryContext> {
        let mut change_timeline = Vec::new();
        let mut architectural_decisions = Vec::new();
        let mut refactoring_history = Vec::new();

        for checkpoint in checkpoints {
            // Create change event
            let change_event = ChangeEvent {
                timestamp: checkpoint.base_checkpoint.created_at,
                checkpoint_id: checkpoint.base_checkpoint.id.to_string(),
                description: checkpoint.base_checkpoint.description.clone(),
                files_changed: checkpoint
                    .base_checkpoint
                    .file_changes
                    .iter()
                    .map(|fc| fc.path.to_string_lossy().to_string())
                    .collect(),
                change_type: self.classify_change_type(&checkpoint.intent_analysis),
                impact: format!("{:?}", checkpoint.architectural_impact.significance),
            };
            change_timeline.push(change_event);

            // Extract architectural decisions
            for decision in &checkpoint.intent_analysis.architectural_decisions {
                architectural_decisions.push(decision.clone());
            }

            // Extract refactoring events
            if let Some(refactoring_type) = &checkpoint.intent_analysis.refactoring_type {
                let refactoring_event = RefactoringEvent {
                    refactoring_type: format!("{:?}", refactoring_type),
                    description: checkpoint.base_checkpoint.description.clone(),
                    files_affected: checkpoint
                        .base_checkpoint
                        .file_changes
                        .iter()
                        .map(|fc| fc.path.to_string_lossy().to_string())
                        .collect(),
                    timestamp: checkpoint.base_checkpoint.created_at,
                    reasoning: format!("Confidence: {:.2}", checkpoint.intent_analysis.confidence),
                };
                refactoring_history.push(refactoring_event);
            }
        }

        // Sort by timestamp
        change_timeline.sort_by_key(|e| e.timestamp);
        // architectural_decisions.sort_by_key(|d| d.timestamp); // Skip sorting for now
        refactoring_history.sort_by_key(|r| r.timestamp);

        Ok(HistoryContext {
            change_timeline,
            architectural_decisions: vec![], // TODO: Convert between types
            refactoring_history,
        })
    }

    /// Generate code examples
    async fn generate_examples(
        &self,
        scored_content: &[(AIContextCheckpoint, SemanticContext, f64)],
        _query_intent: &super::QueryIntent,
    ) -> Result<Vec<ExampleContext>> {
        let mut examples = Vec::new();

        // For now, create simple examples from high-scoring functions
        for (checkpoint, semantic_context, score) in scored_content.iter().take(5) {
            if *score > 0.7 {
                for (func_name, func_def) in semantic_context.functions.iter().take(2) {
                    // This would be more sophisticated in a real implementation
                    let example = ExampleContext {
                        description: format!("Example from {}", checkpoint.base_checkpoint.description),
                        code_example: format!("// Function: {}\n// Parameters: {:?}\n// Return: {:?}\n// Relevance: {:.2}", func_name, func_def.parameters, func_def.return_type, score),
                        surrounding_context: "High relevance function".to_string(),
                        applicable_patterns: vec!["function-pattern".to_string()],
                        confidence: *score,
                    };
                    examples.push(example);
                }
            }
        }

        Ok(examples)
    }

    /// Create core files context
    async fn create_core_files_context(
        &self,
        scored_content: &[(AIContextCheckpoint, SemanticContext, f64)],
    ) -> Result<Vec<ContextFile>> {
        let mut core_files = Vec::new();
        let mut seen_files = HashSet::new();

        for (checkpoint, semantic_context, relevance_score) in scored_content {
            for file_change in &checkpoint.base_checkpoint.file_changes {
                if seen_files.contains(&file_change.path) {
                    continue;
                }
                seen_files.insert(file_change.path.clone());

                if let Some(content) = &file_change.new_content {
                    let semantic_info = self
                        .extract_file_semantic_info(
                            &file_change.path.to_string_lossy(),
                            semantic_context,
                        )
                        .await?;

                    let context_file = ContextFile {
                        path: file_change.path.to_string_lossy().to_string(),
                        content: content.clone(),
                        language: self.detect_language(&file_change.path.to_string_lossy()),
                        encoding: format!("{:?}", file_change.encoding),
                        size_bytes: file_change.size_bytes as usize,
                        last_modified: file_change.modified_at,
                        semantic_info,
                        relevance_score: *relevance_score,
                    };

                    core_files.push(context_file);
                }
            }
        }

        // Sort by relevance score
        core_files.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(core_files)
    }

    // Helper methods...

    async fn extract_file_semantic_info(
        &self,
        _file_path: &str,
        _semantic_context: &SemanticContext,
    ) -> Result<FileSemanticInfo> {
        // Simplified implementation - would extract from semantic context
        Ok(FileSemanticInfo {
            functions: vec![],
            classes: vec![],
            interfaces: vec![],
            types: vec![],
            imports: vec![],
            exports: vec![],
            complexity_metrics: ComplexityMetrics {
                cyclomatic_complexity: 0,
                lines_of_code: 0,
                function_count: 0,
                class_count: 0,
                average_function_length: 0.0,
            },
        })
    }

    fn detect_language(&self, file_path: &str) -> String {
        match std::path::Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
        {
            Some("ts") | Some("tsx") => "typescript".to_string(),
            Some("js") | Some("jsx") => "javascript".to_string(),
            Some("rs") => "rust".to_string(),
            Some("py") => "python".to_string(),
            Some("go") => "go".to_string(),
            Some("java") => "java".to_string(),
            Some("cpp") | Some("cc") | Some("cxx") => "cpp".to_string(),
            Some("c") => "c".to_string(),
            Some("h") | Some("hpp") => "c".to_string(),
            _ => "unknown".to_string(),
        }
    }

    fn classify_change_type(&self, _intent_analysis: &IntentAnalysis) -> String {
        // Simplified implementation
        "modification".to_string()
    }

    async fn calculate_build_metadata(
        &self,
        core_files: &[ContextFile],
        checkpoints: &[AIContextCheckpoint],
        _options: &ContextOptions,
    ) -> Result<ContextBuildMetadata> {
        let total_lines_of_code = core_files.iter().map(|f| f.content.lines().count()).sum();

        let estimated_tokens = total_lines_of_code * 4; // Rough estimation

        Ok(ContextBuildMetadata {
            checkpoints_analyzed: checkpoints.len(),
            files_included: core_files.len(),
            total_lines_of_code,
            build_strategy: "semantic_relevance".to_string(),
            estimated_tokens,
        })
    }

    // Placeholder implementations for complex methods
    async fn build_call_graph(
        &self,
        _scored_content: &[(AIContextCheckpoint, SemanticContext, f64)],
    ) -> Result<CallGraph> {
        Ok(CallGraph {
            functions: vec![],
            relationships: vec![],
        })
    }

    async fn build_type_hierarchy(
        &self,
        _scored_content: &[(AIContextCheckpoint, SemanticContext, f64)],
    ) -> Result<TypeHierarchy> {
        Ok(TypeHierarchy {
            root_types: vec![],
            inheritance_chains: vec![],
            interface_implementations: vec![],
        })
    }

    async fn build_import_graph(
        &self,
        _scored_content: &[(AIContextCheckpoint, SemanticContext, f64)],
    ) -> Result<ImportGraph> {
        Ok(ImportGraph {
            modules: vec![],
            dependencies: vec![],
        })
    }

    async fn identify_usage_patterns(
        &self,
        _scored_content: &[(AIContextCheckpoint, SemanticContext, f64)],
    ) -> Result<Vec<UsagePattern>> {
        Ok(vec![])
    }

    /// Enhanced semantic analysis using the semantic analyzer
    pub async fn perform_enhanced_semantic_analysis(
        &self,
        checkpoints: &[AIContextCheckpoint],
    ) -> Result<Vec<(AIContextCheckpoint, SemanticContext)>> {
        let mut results = Vec::new();

        for checkpoint in checkpoints {
            // Use the semantic analyzer to enhance context
            let enhanced_context = self
                .semantic_analyzer
                .analyze_codebase(&checkpoint.base_checkpoint.file_changes)
                .unwrap_or_else(|_| checkpoint.semantic_context.clone());

            results.push((checkpoint.clone(), enhanced_context));
        }

        Ok(results)
    }

    /// Perform intent analysis using the intent analyzer
    pub async fn perform_intent_analysis(
        &self,
        file_changes: &[crate::types::FileChange],
        semantic_context: &crate::semantic::types::SemanticContext,
        description: &str,
    ) -> Result<crate::semantic::types::IntentAnalysis> {
        self.intent_analyzer
            .analyze_intent(file_changes, semantic_context, description)
            .await
    }
}
