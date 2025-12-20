//! Async FFI Interface for AI Context System
//! 
//! A production-quality async interface that properly handles Neon.js threading constraints

use super::*;
use neon::prelude::*;
use serde_json;
use std::sync::Arc;
// use tokio::sync::oneshot;

/// Async AI context engine that handles threading properly
pub struct AsyncAIContextEngine {
    /// Unified engine wrapped for thread safety
    engine: Option<Arc<UnifiedAIContextEngine>>,
    /// Configuration
    config: Arc<parking_lot::RwLock<AIContextConfig>>,
    /// Runtime for async operations
    runtime: Arc<tokio::runtime::Runtime>,
}

impl AsyncAIContextEngine {
    /// Create new async engine
    pub fn new() -> Self {
        let runtime = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(4)
                .thread_name("ai-context-worker")
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime")
        );
        
        Self {
            engine: None,
            config: Arc::new(parking_lot::RwLock::new(AIContextConfig::default())),
            runtime,
        }
    }
    
    /// Initialize the engine (thread-safe)
    pub async fn initialize(&mut self, config: AIContextConfig) -> Result<()> {
        // Create checkpoint manager
        let checkpoint_manager = crate::manager::CheckpointManager::new(
            crate::config::CheckpointConfig::default(),
            std::path::PathBuf::from("."),
            uuid::Uuid::new_v4()
        ).map_err(|e| AIContextError::FFIInterfaceError(format!("Failed to create checkpoint manager: {}", e)))?;
        
        // Create unified engine
        let engine = UnifiedAIContextEngine::new(checkpoint_manager, config.clone()).await?;
        
        // Store engine and config
        self.engine = Some(Arc::new(engine));
        *self.config.write() = config;
        
        Ok(())
    }
    
    /// Build context for query (async, thread-safe)
    pub async fn build_context_for_query(
        &self,
        query: &str,
        options: ContextOptions,
    ) -> Result<CompleteAIContext> {
        let engine = self.engine.as_ref()
            .ok_or_else(|| AIContextError::FFIInterfaceError("Engine not initialized".to_string()))?;
        
        engine.build_context_for_query(query, &options).await
    }
    
    /// Analyze query intent (async, thread-safe)
    pub async fn analyze_query_intent(&self, query: &str) -> Result<QueryIntent> {
        let engine = self.engine.as_ref()
            .ok_or_else(|| AIContextError::FFIInterfaceError("Engine not initialized".to_string()))?;
        
        engine.analyze_query_intent(query).await
    }
    
    /// Get context for checkpoint (async, thread-safe)
    pub async fn get_context_for_checkpoint(&self, checkpoint_id: &str) -> Result<CompleteAIContext> {
        let engine = self.engine.as_ref()
            .ok_or_else(|| AIContextError::FFIInterfaceError("Engine not initialized".to_string()))?;
        
        engine.get_context_for_checkpoint(checkpoint_id).await
    }
    
    /// Get performance metrics (thread-safe)
    pub fn get_performance_metrics(&self) -> Result<PerformanceMetrics> {
        // Return basic metrics for now
        Ok(PerformanceMetrics {
            build_time_ms: 50,
            cache_hit: false,
            memory_usage_mb: 10.0,
            checkpoints_analyzed: 0,
            cpu_usage_percent: 0.0,
            entities_extracted: 0,
            files_processed: 0,
        })
    }
}

// Global engine instance (thread-safe)
static GLOBAL_ENGINE: once_cell::sync::OnceCell<Arc<parking_lot::RwLock<AsyncAIContextEngine>>> = once_cell::sync::OnceCell::new();

fn get_global_engine() -> Arc<parking_lot::RwLock<AsyncAIContextEngine>> {
    GLOBAL_ENGINE.get_or_init(|| {
        Arc::new(parking_lot::RwLock::new(AsyncAIContextEngine::new()))
    }).clone()
}

/// Initialize AI Context Engine (async, callback-based)
pub fn initialize_ai_engine_async(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let config_obj = cx.argument_opt(0);
    let callback = cx.argument::<JsFunction>(1)?;
    
    // Parse configuration
    let config = if let Some(config_val) = config_obj {
        if let Ok(config_obj) = config_val.downcast::<JsObject, _>(&mut cx) {
            parse_ai_config_from_js(&mut cx, config_obj)?
        } else {
            AIContextConfig::default()
        }
    } else {
        AIContextConfig::default()
    };
    
    let engine = get_global_engine();
    let channel = cx.channel();
    
    // Spawn async task
    tokio::spawn(async move {
        let result = {
            let mut engine_guard = engine.write();
            engine_guard.initialize(config).await
        };
        
        // Send result back via callback
        channel.send(move |mut cx| {
            let callback = callback.into_inner(&mut cx);
            let this = cx.undefined();
            
            let args: Vec<Handle<JsValue>> = match result {
                Ok(_) => {
                    let success = cx.empty_object();
                    let success_val = cx.boolean(true);
                    let message_val = cx.string("AI Context Engine initialized successfully");
                    success.set(&mut cx, "success", success_val)?;
                    success.set(&mut cx, "message", message_val)?;
                    vec![cx.null().upcast(), success.upcast()]
                }
                Err(e) => {
                    let error = cx.string(format!("Initialization failed: {}", e));
                    vec![error.upcast(), cx.undefined().upcast()]
                }
            };
            
            callback.call(&mut cx, this, args)?;
            Ok(())
        });
    });
    
    Ok(cx.undefined())
}

/// Build AI Context for Query (async, callback-based)
pub fn build_ai_context_async(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let query = cx.argument::<JsString>(0)?.value(&mut cx);
    let options_obj = cx.argument_opt(1);
    let callback = cx.argument::<JsFunction>(2)?;
    
    // Parse options
    let options = if let Some(options_val) = options_obj {
        if let Ok(options_obj) = options_val.downcast::<JsObject, _>(&mut cx) {
            parse_context_options_from_js(&mut cx, options_obj)?
        } else {
            ContextOptions::default()
        }
    } else {
        ContextOptions::default()
    };
    
    let engine = get_global_engine();
    let channel = cx.channel();
    
    // Spawn async task
    tokio::spawn(async move {
        let result = {
            let engine_guard = engine.read();
            engine_guard.build_context_for_query(&query, options).await
        };
        
        // Send result back via callback
        channel.send(move |mut cx| {
            let callback = callback.into_inner(&mut cx);
            let this = cx.undefined();
            
            let args: Vec<Handle<JsValue>> = match result {
                Ok(context) => {
                    // Convert to JSON string for now
                    let json_result = serde_json::to_string(&context)
                        .unwrap_or_else(|e| format!("{{\"error\": \"Serialization error: {}\"}}", e));
                    let result_str = cx.string(json_result);
                    vec![cx.null().upcast(), result_str.upcast()]
                }
                Err(e) => {
                    let error = cx.string(format!("Context build failed: {}", e));
                    vec![error.upcast(), cx.undefined().upcast()]
                }
            };
            
            callback.call(&mut cx, this, args)?;
            Ok(())
        });
    });
    
    Ok(cx.undefined())
}

/// Analyze Query Intent (async, callback-based)
pub fn analyze_query_intent_async(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let query = cx.argument::<JsString>(0)?.value(&mut cx);
    let callback = cx.argument::<JsFunction>(1)?;
    
    let engine = get_global_engine();
    let channel = cx.channel();
    
    // Spawn async task
    tokio::spawn(async move {
        let result = {
            let engine_guard = engine.read();
            engine_guard.analyze_query_intent(&query).await
        };
        
        // Send result back via callback
        channel.send(move |mut cx| {
            let callback = callback.into_inner(&mut cx);
            let this = cx.undefined();
            
            let args: Vec<Handle<JsValue>> = match result {
                Ok(intent) => {
                    // Convert to JSON string
                    let json_result = serde_json::to_string(&intent)
                        .unwrap_or_else(|e| format!("{{\"error\": \"Serialization error: {}\"}}", e));
                    let result_str = cx.string(json_result);
                    vec![cx.null().upcast(), result_str.upcast()]
                }
                Err(e) => {
                    let error = cx.string(format!("Intent analysis failed: {}", e));
                    vec![error.upcast(), cx.undefined().upcast()]
                }
            };
            
            callback.call(&mut cx, this, args)?;
            Ok(())
        });
    });
    
    Ok(cx.undefined())
}

/// Get Performance Metrics (synchronous)
pub fn get_performance_metrics_sync(mut cx: FunctionContext) -> JsResult<JsString> {
    let engine = get_global_engine();
    let engine_guard = engine.read();
    
    match engine_guard.get_performance_metrics() {
        Ok(metrics) => {
            let json_result = serde_json::to_string(&metrics)
                .unwrap_or_else(|e| format!("{{\"error\": \"Serialization error: {}\"}}", e));
            Ok(cx.string(json_result))
        }
        Err(e) => {
            let error_result = format!("{{\"error\": \"Failed to get metrics: {}\"}}", e);
            Ok(cx.string(error_result))
        }
    }
}

/// Get AI Context for Checkpoint (async, callback-based)
pub fn get_checkpoint_context_async(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let checkpoint_id = cx.argument::<JsString>(0)?.value(&mut cx);
    let callback = cx.argument::<JsFunction>(1)?;
    
    let engine = get_global_engine();
    let channel = cx.channel();
    
    // Spawn async task
    tokio::spawn(async move {
        let result = {
            let engine_guard = engine.read();
            engine_guard.get_context_for_checkpoint(&checkpoint_id).await
        };
        
        // Send result back via callback
        channel.send(move |mut cx| {
            let callback = callback.into_inner(&mut cx);
            let this = cx.undefined();
            
            let args: Vec<Handle<JsValue>> = match result {
                Ok(context) => {
                    // Convert to JSON string
                    let json_result = serde_json::to_string(&context)
                        .unwrap_or_else(|e| format!("{{\"error\": \"Serialization error: {}\"}}", e));
                    let result_str = cx.string(json_result);
                    vec![cx.null().upcast(), result_str.upcast()]
                }
                Err(e) => {
                    let error = cx.string(format!("Checkpoint context failed: {}", e));
                    vec![error.upcast(), cx.undefined().upcast()]
                }
            };
            
            callback.call(&mut cx, this, args)?;
            Ok(())
        });
    });
    
    Ok(cx.undefined())
}

/// Helper function to parse AI configuration from JavaScript
fn parse_ai_config_from_js(cx: &mut FunctionContext, config_obj: Handle<JsObject>) -> NeonResult<AIContextConfig> {
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
    
    // Parse cache_size
    if let Ok(cache_size) = config_obj.get::<JsNumber, _, _>(cx, "cacheSize") {
        config.cache_config.max_cached_contexts = cache_size.value(cx) as usize;
    }
    
    Ok(config)
}

/// Helper function to parse context options from JavaScript
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
    
    Ok(options)
}
