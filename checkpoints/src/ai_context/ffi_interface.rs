//! FFI Interface for AI Context System
//! 
//! Provides Node.js/Neon.js bindings for the unified AI context engine

use super::*;
use crate::manager::CheckpointManager;
use neon::prelude::*;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

/// FFI wrapper for the unified AI context engine  
pub struct AIContextEngineFFI {
    /// Configuration for the engine
    config: AIContextConfig,
    /// Runtime for async operations
    runtime: Runtime,
}

impl AIContextEngineFFI {
    /// Create a new FFI wrapper
    pub fn new(config: AIContextConfig) -> Result<Self> {
        let runtime = Runtime::new()
            .map_err(|e| AIContextError::FFIInterfaceError(format!("Failed to create runtime: {}", e)))?;
        
        Ok(Self {
            config,
            runtime,
        })
    }

    /// Build context for a query
    pub fn build_context_for_query(
        &self,
        query: &str,
        options: Option<ContextOptions>,
    ) -> Result<AIContextResult> {
        self.runtime.block_on(async {
            // Create checkpoint manager
            let checkpoint_manager = CheckpointManager::new(
                crate::config::CheckpointConfig::default(),
                std::path::PathBuf::from("."),
                uuid::Uuid::new_v4()
            ).map_err(|e| AIContextError::FFIInterfaceError(format!("Failed to create checkpoint manager: {}", e)))?;
            
            // Create unified engine
            let engine = UnifiedAIContextEngine::new(checkpoint_manager, self.config.clone()).await?;
            
            // Build context
            engine.build_context_for_query(query, options).await
        })
    }

    /// Get context for a checkpoint
    pub fn get_context_for_checkpoint(&self, checkpoint_id: &str) -> Result<AIContextResult> {
        self.runtime.block_on(async {
            // Create checkpoint manager
            let checkpoint_manager = CheckpointManager::new(
                crate::config::CheckpointConfig::default(),
                std::path::PathBuf::from("."),
                uuid::Uuid::new_v4()
            ).map_err(|e| AIContextError::FFIInterfaceError(format!("Failed to create checkpoint manager: {}", e)))?;
            
            // Create unified engine
            let engine = UnifiedAIContextEngine::new(checkpoint_manager, self.config.clone()).await?;
            
            // Get context
            engine.get_context_for_checkpoint(checkpoint_id).await
        })
    }

    /// Analyze query intent
    pub fn analyze_query_intent(&self, query: &str) -> Result<QueryIntent> {
        self.runtime.block_on(async {
            let query_analyzer = QueryAnalyzer::new()?;
            query_analyzer.analyze_intent(query).await
        })
    }

    /// Get performance metrics (simplified for now)
    pub fn get_performance_metrics(&self) -> Result<EnginePerformanceMetrics> {
        Ok(EnginePerformanceMetrics {
            total_context_builds: 0,
            cache_hit_ratio: 0.0,
            average_build_time_ms: 0.0,
            active_operations: 0,
            cache_stats: CacheStats {
                context_cache_size: 0,
                intent_cache_size: 0,
                similarity_cache_size: 0,
                max_cache_size: 1000,
                cache_hit_ratio: 0.0,
            },
            memory_usage_mb: 0.0,
            uptime_seconds: 0,
        })
    }
}

// Implement Finalize for Neon.js boxing
impl Finalize for AIContextEngineFFI {}

// Neon.js export functions

/// Initialize the AI context engine
pub fn js_initialize_engine(mut cx: FunctionContext) -> JsResult<JsObject> {
    let config_obj = cx.argument::<JsObject>(0)?;
    
    // Parse configuration from JavaScript object
    let config = parse_config_from_js(&mut cx, config_obj)?;
    
    // Create engine wrapper
    let engine = AIContextEngineFFI::new(config)
        .or_else(|e| cx.throw_error(format!("Failed to create engine: {}", e)))?;
    
    // Store engine in context for later use
    let this = cx.this().downcast::<JsObject, _>(&mut cx).or_throw(&mut cx)?;
    this.set(&mut cx, "engine", cx.boxed(engine))?;
    
    // Return success object
    let result = cx.empty_object();
    result.set(&mut cx, "success", cx.boolean(true))?;
    Ok(result)
}

/// Build context for a query
pub fn js_build_context_for_query(mut cx: FunctionContext) -> JsResult<JsObject> {
    let query = cx.argument::<JsString>(0)?.value(&mut cx);
    let options_obj = cx.argument_opt(1);
    
    // Parse options if provided
    let options = if let Some(opts) = options_obj {
        if let Ok(obj) = opts.downcast::<JsObject, _>(&mut cx) {
            Some(parse_context_options_from_js(&mut cx, obj)?)
        } else {
            None
        }
    } else {
        None
    };
    
    // Get engine from context
    let this = cx.this().downcast::<JsObject, _>(&mut cx).or_throw(&mut cx)?;
    let engine = this.get::<JsBox<AIContextEngineFFI>, _, _>(&mut cx, "engine")?;
    
    // Build context
    let result = engine.build_context_for_query(&query, options)
        .or_else(|e| cx.throw_error(format!("Failed to build context: {}", e)))?;
    
    // Convert result to JavaScript object
    convert_ai_context_result_to_js(&mut cx, result)
}

/// Get context for a checkpoint
pub fn js_get_context_for_checkpoint(mut cx: FunctionContext) -> JsResult<JsObject> {
    let checkpoint_id = cx.argument::<JsString>(0)?.value(&mut cx);
    
    // Get engine from context
    let this = cx.this().downcast::<JsObject, _>(&mut cx).or_throw(&mut cx)?;
    let engine = this.get::<JsBox<AIContextEngineFFI>, _, _>(&mut cx, "engine")?;
    
    // Get context
    let result = engine.get_context_for_checkpoint(&checkpoint_id)
        .or_else(|e| cx.throw_error(format!("Failed to get checkpoint context: {}", e)))?;
    
    // Convert result to JavaScript object
    convert_ai_context_result_to_js(&mut cx, result)
}

/// Analyze query intent
pub fn js_analyze_query_intent(mut cx: FunctionContext) -> JsResult<JsObject> {
    let query = cx.argument::<JsString>(0)?.value(&mut cx);
    
    // Get engine from context
    let this = cx.this().downcast::<JsObject, _>(&mut cx).or_throw(&mut cx)?;
    let engine = this.get::<JsBox<AIContextEngineFFI>, _, _>(&mut cx, "engine")?;
    
    // Analyze intent
    let intent = engine.analyze_query_intent(&query)
        .or_else(|e| cx.throw_error(format!("Failed to analyze query intent: {}", e)))?;
    
    // Convert to JavaScript object
    convert_query_intent_to_js(&mut cx, intent)
}

/// Get performance metrics
pub fn js_get_performance_metrics(mut cx: FunctionContext) -> JsResult<JsObject> {
    // Get engine from context
    let this = cx.this().downcast::<JsObject, _>(&mut cx).or_throw(&mut cx)?;
    let engine = this.get::<JsBox<AIContextEngineFFI>, _, _>(&mut cx, "engine")?;
    
    // Get metrics
    let metrics = engine.get_performance_metrics()
        .or_else(|e| cx.throw_error(format!("Failed to get performance metrics: {}", e)))?;
    
    // Convert to JavaScript object
    convert_performance_metrics_to_js(&mut cx, metrics)
}

// Helper functions for JavaScript object conversion

/// Parse configuration from JavaScript object
fn parse_config_from_js(cx: &mut FunctionContext, config_obj: Handle<JsObject>) -> NeonResult<AIContextConfig> {
    let mut config = AIContextConfig::default();
    
    // Parse max_tokens
    if let Ok(max_tokens) = config_obj.get::<JsNumber, _, _>(cx, "maxTokens") {
        config.max_tokens = max_tokens.value(cx) as usize;
    }
    
    // Parse enable_incremental_updates
    if let Ok(enable_incremental) = config_obj.get::<JsBoolean, _, _>(cx, "enableIncrementalUpdates") {
        config.enable_incremental_updates = enable_incremental.value(cx);
    }
    
    // Parse enable_parallel_processing
    if let Ok(enable_parallel) = config_obj.get::<JsBoolean, _, _>(cx, "enableParallelProcessing") {
        config.enable_parallel_processing = enable_parallel.value(cx);
    }
    
    Ok(config)
}

/// Parse context options from JavaScript object
fn parse_context_options_from_js(cx: &mut FunctionContext, options_obj: Handle<JsObject>) -> NeonResult<ContextOptions> {
    let mut options = ContextOptions::default();
    
    // Parse max_tokens
    if let Ok(max_tokens) = options_obj.get::<JsNumber, _, _>(cx, "maxTokens") {
        options.max_tokens = max_tokens.value(cx) as usize;
    }
    
    // Parse include_evolution
    if let Ok(include_evolution) = options_obj.get::<JsBoolean, _, _>(cx, "includeEvolution") {
        options.include_evolution = include_evolution.value(cx);
    }
    
    // Parse include_examples
    if let Ok(include_examples) = options_obj.get::<JsBoolean, _, _>(cx, "includeExamples") {
        options.include_examples = include_examples.value(cx);
    }
    
    // Parse priority_threshold
    if let Ok(priority_threshold) = options_obj.get::<JsNumber, _, _>(cx, "priorityThreshold") {
        options.priority_threshold = priority_threshold.value(cx);
    }
    
    Ok(options)
}

/// Convert AI context result to JavaScript object
fn convert_ai_context_result_to_js<'a>(cx: &mut FunctionContext<'a>, result: AIContextResult) -> NeonResult<Handle<'a, JsObject>> {
    let obj = cx.empty_object();
    
    // Convert context
    let context_obj = convert_complete_ai_context_to_js(cx, result.context)?;
    obj.set(cx, "context", context_obj)?;
    
    // Convert metadata
    let metadata_obj = convert_context_metadata_to_js(cx, result.metadata)?;
    obj.set(cx, "metadata", metadata_obj)?;
    
    // Convert performance metrics
    let performance_obj = convert_performance_metrics_to_js(cx, result.performance)?;
    obj.set(cx, "performance", performance_obj)?;
    
    // Convert cache info
    let cache_obj = convert_cache_info_to_js(cx, result.cache_info)?;
    obj.set(cx, "cacheInfo", cache_obj)?;
    
    Ok(obj)
}

/// Convert complete AI context to JavaScript object
fn convert_complete_ai_context_to_js<'a>(cx: &mut FunctionContext<'a>, context: CompleteAIContext) -> NeonResult<Handle<'a, JsObject>> {
    let obj = cx.empty_object();
    
    // Convert core files
    let core_files = cx.empty_array();
    for (i, file) in context.core_files.iter().enumerate() {
        let file_obj = convert_context_file_to_js(cx, file)?;
        core_files.set(cx, i as u32, file_obj)?;
    }
    obj.set(cx, "coreFiles", core_files)?;
    
    // Convert architecture
    let architecture_obj = convert_architectural_context_to_js(cx, context.architecture)?;
    obj.set(cx, "architecture", architecture_obj)?;
    
    // Convert relationships
    let relationships_obj = convert_relationship_context_to_js(cx, context.relationships)?;
    obj.set(cx, "relationships", relationships_obj)?;
    
    // Convert history
    let history_obj = convert_history_context_to_js(cx, context.history)?;
    obj.set(cx, "history", history_obj)?;
    
    // Convert examples
    let examples = cx.empty_array();
    for (i, example) in context.examples.iter().enumerate() {
        let example_obj = convert_example_context_to_js(cx, example)?;
        examples.set(cx, i as u32, example_obj)?;
    }
    obj.set(cx, "examples", examples)?;
    
    // Convert metadata
    let metadata_obj = convert_context_build_metadata_to_js(cx, context.metadata)?;
    obj.set(cx, "metadata", metadata_obj)?;
    
    Ok(obj)
}

/// Convert context file to JavaScript object
fn convert_context_file_to_js(cx: &mut FunctionContext, file: &ContextFile) -> NeonResult<Handle<JsObject>> {
    let obj = cx.empty_object();
    
    obj.set(cx, "path", cx.string(&file.path))?;
    obj.set(cx, "content", cx.string(&file.content))?;
    obj.set(cx, "language", cx.string(&file.language))?;
    obj.set(cx, "encoding", cx.string(&file.encoding))?;
    obj.set(cx, "sizeBytes", cx.number(file.size_bytes as f64))?;
    obj.set(cx, "relevanceScore", cx.number(file.relevance_score))?;
    
    // Convert semantic info
    let semantic_obj = convert_file_semantic_info_to_js(cx, &file.semantic_info)?;
    obj.set(cx, "semanticInfo", semantic_obj)?;
    
    Ok(obj)
}

/// Convert file semantic info to JavaScript object
fn convert_file_semantic_info_to_js(cx: &mut FunctionContext, info: &FileSemanticInfo) -> NeonResult<Handle<JsObject>> {
    let obj = cx.empty_object();
    
    // Convert functions
    let functions = cx.empty_array();
    for (i, func) in info.functions.iter().enumerate() {
        let func_obj = convert_function_info_to_js(cx, func)?;
        functions.set(cx, i as u32, func_obj)?;
    }
    obj.set(cx, "functions", functions)?;
    
    // Convert classes
    let classes = cx.empty_array();
    for (i, class) in info.classes.iter().enumerate() {
        let class_obj = convert_class_info_to_js(cx, class)?;
        classes.set(cx, i as u32, class_obj)?;
    }
    obj.set(cx, "classes", classes)?;
    
    // Convert interfaces
    let interfaces = cx.empty_array();
    for (i, interface) in info.interfaces.iter().enumerate() {
        let interface_obj = convert_interface_info_to_js(cx, interface)?;
        interfaces.set(cx, i as u32, interface_obj)?;
    }
    obj.set(cx, "interfaces", interfaces)?;
    
    // Convert types
    let types = cx.empty_array();
    for (i, type_info) in info.types.iter().enumerate() {
        let type_obj = convert_type_info_to_js(cx, type_info)?;
        types.set(cx, i as u32, type_obj)?;
    }
    obj.set(cx, "types", types)?;
    
    Ok(obj)
}

// Simplified conversion functions (would be more complete in practice)

fn convert_function_info_to_js(cx: &mut FunctionContext, func: &FunctionInfo) -> NeonResult<Handle<JsObject>> {
    let obj = cx.empty_object();
    obj.set(cx, "name", cx.string(&func.name))?;
    obj.set(cx, "complexity", cx.number(func.complexity as f64))?;
    obj.set(cx, "isAsync", cx.boolean(func.is_async))?;
    Ok(obj)
}

fn convert_class_info_to_js(cx: &mut FunctionContext, class: &ClassInfo) -> NeonResult<Handle<JsObject>> {
    let obj = cx.empty_object();
    obj.set(cx, "name", cx.string(&class.name))?;
    obj.set(cx, "visibility", cx.string(&class.visibility))?;
    Ok(obj)
}

fn convert_interface_info_to_js(cx: &mut FunctionContext, interface: &InterfaceInfo) -> NeonResult<Handle<JsObject>> {
    let obj = cx.empty_object();
    obj.set(cx, "name", cx.string(&interface.name))?;
    Ok(obj)
}

fn convert_type_info_to_js(cx: &mut FunctionContext, type_info: &TypeInfo) -> NeonResult<Handle<JsObject>> {
    let obj = cx.empty_object();
    obj.set(cx, "name", cx.string(&type_info.name))?;
    obj.set(cx, "kind", cx.string(&type_info.kind))?;
    Ok(obj)
}

fn convert_architectural_context_to_js(cx: &mut FunctionContext, _architecture: ArchitecturalContext) -> NeonResult<Handle<JsObject>> {
    // Simplified implementation
    let obj = cx.empty_object();
    obj.set(cx, "patterns", cx.empty_array())?;
    Ok(obj)
}

fn convert_relationship_context_to_js(cx: &mut FunctionContext, _relationships: RelationshipContext) -> NeonResult<Handle<JsObject>> {
    // Simplified implementation
    let obj = cx.empty_object();
    obj.set(cx, "callGraph", cx.empty_object())?;
    Ok(obj)
}

fn convert_history_context_to_js(cx: &mut FunctionContext, _history: HistoryContext) -> NeonResult<Handle<JsObject>> {
    // Simplified implementation
    let obj = cx.empty_object();
    obj.set(cx, "changeTimeline", cx.empty_array())?;
    Ok(obj)
}

fn convert_example_context_to_js(cx: &mut FunctionContext, example: &ExampleContext) -> NeonResult<Handle<JsObject>> {
    let obj = cx.empty_object();
    obj.set(cx, "description", cx.string(&example.description))?;
    obj.set(cx, "codeExample", cx.string(&example.code_example))?;
    obj.set(cx, "confidence", cx.number(example.confidence))?;
    Ok(obj)
}

fn convert_context_build_metadata_to_js(cx: &mut FunctionContext, metadata: ContextBuildMetadata) -> NeonResult<Handle<JsObject>> {
    let obj = cx.empty_object();
    obj.set(cx, "checkpointsAnalyzed", cx.number(metadata.checkpoints_analyzed as f64))?;
    obj.set(cx, "filesIncluded", cx.number(metadata.files_included as f64))?;
    obj.set(cx, "estimatedTokens", cx.number(metadata.estimated_tokens as f64))?;
    Ok(obj)
}

fn convert_context_metadata_to_js(cx: &mut FunctionContext, metadata: ContextMetadata) -> NeonResult<Handle<JsObject>> {
    let obj = cx.empty_object();
    obj.set(cx, "contextType", cx.string(&metadata.context_type))?;
    obj.set(cx, "confidenceScore", cx.number(metadata.confidence_score))?;
    obj.set(cx, "tokenCount", cx.number(metadata.token_count as f64))?;
    Ok(obj)
}

fn convert_performance_metrics_to_js(cx: &mut FunctionContext, metrics: PerformanceMetrics) -> NeonResult<Handle<JsObject>> {
    let obj = cx.empty_object();
    obj.set(cx, "buildTimeMs", cx.number(metrics.build_time_ms as f64))?;
    obj.set(cx, "cacheHit", cx.boolean(metrics.cache_hit))?;
    obj.set(cx, "memoryUsageMb", cx.number(metrics.memory_usage_mb))?;
    Ok(obj)
}

fn convert_cache_info_to_js(cx: &mut FunctionContext, cache_info: CacheInfo) -> NeonResult<Handle<JsObject>> {
    let obj = cx.empty_object();
    obj.set(cx, "cacheHit", cx.boolean(cache_info.cache_hit))?;
    obj.set(cx, "cacheKey", cx.string(&cache_info.cache_key))?;
    Ok(obj)
}

fn convert_query_intent_to_js(cx: &mut FunctionContext, intent: QueryIntent) -> NeonResult<Handle<JsObject>> {
    let obj = cx.empty_object();
    obj.set(cx, "originalQuery", cx.string(&intent.original_query))?;
    obj.set(cx, "queryType", cx.string(&format!("{:?}", intent.query_type)))?;
    obj.set(cx, "scope", cx.string(&format!("{:?}", intent.scope)))?;
    obj.set(cx, "confidence", cx.number(intent.confidence))?;
    Ok(obj)
}

// Export main entry point for Neon.js
#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("initializeEngine", js_initialize_engine)?;
    cx.export_function("buildContextForQuery", js_build_context_for_query)?;
    cx.export_function("getContextForCheckpoint", js_get_context_for_checkpoint)?;
    cx.export_function("analyzeQueryIntent", js_analyze_query_intent)?;
    cx.export_function("getPerformanceMetrics", js_get_performance_metrics)?;
    Ok(())
}
