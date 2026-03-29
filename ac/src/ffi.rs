//! FFI bindings for Node.js using Neon
//!
//! This module provides the Node.js interface for the Rust autocomplete system.

use crate::cache::AutocompleteLruCache;
use crate::context::ContextRetrievalService;
use crate::types::*;
use crate::utils;

use neon::prelude::*;
use once_cell::sync::OnceCell;
use parking_lot::Mutex;
use std::sync::Arc;

/// Global cache instance
static GLOBAL_CACHE: OnceCell<Arc<Mutex<AutocompleteLruCache>>> = OnceCell::new();

/// Global context service instance
static GLOBAL_CONTEXT: OnceCell<Arc<Mutex<ContextRetrievalService>>> = OnceCell::new();

/// Initialize the autocomplete cache
pub fn initialize_cache(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let capacity = cx.argument::<JsNumber>(0)?.value(&mut cx) as usize;

    let cache_path = match utils::get_autocomplete_cache_path() {
        Ok(path) => path,
        Err(e) => return cx.throw_error(format!("Failed to get cache path: {}", e)),
    };

    let cache = match AutocompleteLruCache::new(cache_path, capacity) {
        Ok(c) => c,
        Err(e) => return cx.throw_error(format!("Failed to create cache: {}", e)),
    };

    if GLOBAL_CACHE.set(Arc::new(Mutex::new(cache))).is_err() {
        return cx.throw_error("Cache already initialized");
    }

    Ok(cx.boolean(true))
}

/// Get a completion from the cache
pub fn cache_get(mut cx: FunctionContext) -> JsResult<JsValue> {
    let prefix = cx.argument::<JsString>(0)?.value(&mut cx);

    let cache = match GLOBAL_CACHE.get() {
        Some(c) => c,
        None => return cx.throw_error("Cache not initialized"),
    };

    let result = match cache.lock().get(&prefix) {
        Ok(r) => r,
        Err(e) => return cx.throw_error(format!("Cache get failed: {}", e)),
    };

    match result {
        Some(completion) => Ok(cx.string(completion).upcast()),
        None => Ok(cx.undefined().upcast()),
    }
}

/// Put a completion into the cache
pub fn cache_put(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let prefix = cx.argument::<JsString>(0)?.value(&mut cx);
    let completion = cx.argument::<JsString>(1)?.value(&mut cx);

    let cache = match GLOBAL_CACHE.get() {
        Some(c) => c,
        None => return cx.throw_error("Cache not initialized"),
    };

    if let Err(e) = cache.lock().put(&prefix, &completion) {
        return cx.throw_error(format!("Cache put failed: {}", e));
    }

    Ok(cx.boolean(true))
}

/// Clear the cache
pub fn cache_clear(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let cache = match GLOBAL_CACHE.get() {
        Some(c) => c,
        None => return cx.throw_error("Cache not initialized"),
    };

    if let Err(e) = cache.lock().clear() {
        return cx.throw_error(format!("Cache clear failed: {}", e));
    }

    Ok(cx.boolean(true))
}

/// Get cache statistics
pub fn cache_stats(mut cx: FunctionContext) -> JsResult<JsObject> {
    let cache = match GLOBAL_CACHE.get() {
        Some(c) => c,
        None => return cx.throw_error("Cache not initialized"),
    };

    let cache_lock = cache.lock();
    let len = match cache_lock.len() {
        Ok(l) => l,
        Err(e) => return cx.throw_error(format!("Failed to get cache length: {}", e)),
    };

    let obj = cx.empty_object();
    let total_entries = cx.number(len as f64);
    obj.set(&mut cx, "totalEntries", total_entries)?;

    Ok(obj)
}

/// Initialize the context service
pub fn initialize_context(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let context_service = ContextRetrievalService::new();

    if GLOBAL_CONTEXT.set(Arc::new(Mutex::new(context_service))).is_err() {
        return cx.throw_error("Context service already initialized");
    }

    Ok(cx.boolean(true))
}

/// Update import definitions for a file
pub fn update_imports(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let filepath = cx.argument::<JsString>(0)?.value(&mut cx);
    let content = cx.argument::<JsString>(1)?.value(&mut cx);

    let context = match GLOBAL_CONTEXT.get() {
        Some(c) => c,
        None => return cx.throw_error("Context service not initialized"),
    };

    if let Err(e) = context.lock().update_import_definitions(&filepath, &content) {
        return cx.throw_error(format!("Failed to update imports: {}", e));
    }

    Ok(cx.boolean(true))
}

/// Get snippets from imports
pub fn get_import_snippets(mut cx: FunctionContext) -> JsResult<JsArray> {
    let filepath = cx.argument::<JsString>(0)?.value(&mut cx);
    let prefix = cx.argument::<JsString>(1)?.value(&mut cx);
    let suffix = cx.argument::<JsString>(2)?.value(&mut cx);
    let use_imports = cx.argument::<JsBoolean>(3)?.value(&mut cx);

    let context = match GLOBAL_CONTEXT.get() {
        Some(c) => c,
        None => return cx.throw_error("Context service not initialized"),
    };

    let snippets = match context.lock().get_snippets_from_imports(&filepath, &prefix, &suffix, use_imports) {
        Ok(s) => s,
        Err(e) => return cx.throw_error(format!("Failed to get import snippets: {}", e)),
    };

    let js_array = JsArray::new(&mut cx, snippets.len());

    for (i, snippet) in snippets.iter().enumerate() {
        let obj = cx.empty_object();
        
        let filepath_str = cx.string(&snippet.filepath);
        obj.set(&mut cx, "filepath", filepath_str)?;
        
        let content_str = cx.string(&snippet.content);
        obj.set(&mut cx, "content", content_str)?;
        
        let type_str = cx.string("code");
        obj.set(&mut cx, "type", type_str)?;
        
        js_array.set(&mut cx, i as u32, obj)?;
    }

    Ok(js_array)
}

/// Prune lines from top
pub fn prune_lines_from_top(mut cx: FunctionContext) -> JsResult<JsString> {
    let text = cx.argument::<JsString>(0)?.value(&mut cx);
    let max_tokens = cx.argument::<JsNumber>(1)?.value(&mut cx) as u32;

    let pruned = utils::prune_lines_from_top(&text, max_tokens);
    Ok(cx.string(pruned))
}

/// Prune lines from bottom
pub fn prune_lines_from_bottom(mut cx: FunctionContext) -> JsResult<JsString> {
    let text = cx.argument::<JsString>(0)?.value(&mut cx);
    let max_tokens = cx.argument::<JsNumber>(1)?.value(&mut cx) as u32;

    let pruned = utils::prune_lines_from_bottom(&text, max_tokens);
    Ok(cx.string(pruned))
}

/// Count tokens
pub fn count_tokens(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let text = cx.argument::<JsString>(0)?.value(&mut cx);
    let tokens = utils::count_tokens(&text);
    Ok(cx.number(tokens as f64))
}

/// Extract symbols from text
pub fn extract_symbols(mut cx: FunctionContext) -> JsResult<JsArray> {
    let text = cx.argument::<JsString>(0)?.value(&mut cx);
    let symbols = utils::extract_symbols(&text);

    let js_array = JsArray::new(&mut cx, symbols.len());
    for (i, symbol) in symbols.iter().enumerate() {
        let symbol_str = cx.string(symbol);
        js_array.set(&mut cx, i as u32, symbol_str)?;
    }

    Ok(js_array)
}

/// Get language info for a file
pub fn get_language_info(mut cx: FunctionContext) -> JsResult<JsObject> {
    let filepath = cx.argument::<JsString>(0)?.value(&mut cx);
    let lang_info = AutocompleteLanguageInfo::for_filepath(&filepath);

    let obj = cx.empty_object();
    
    let name = cx.string(&lang_info.name);
    obj.set(&mut cx, "name", name)?;
    
    let comment_start = cx.string(&lang_info.comment_start);
    obj.set(&mut cx, "commentStart", comment_start)?;
    
    let line_comment = cx.string(&lang_info.line_comment);
    obj.set(&mut cx, "lineComment", line_comment)?;
    
    // Top level keywords array
    let keywords_array = JsArray::new(&mut cx, lang_info.top_level_keywords.len());
    for (i, keyword) in lang_info.top_level_keywords.iter().enumerate() {
        let keyword_str = cx.string(keyword);
        keywords_array.set(&mut cx, i as u32, keyword_str)?;
    }
    obj.set(&mut cx, "topLevelKeywords", keywords_array)?;

    Ok(obj)
}

/// Normalize line endings
pub fn normalize_line_endings(mut cx: FunctionContext) -> JsResult<JsString> {
    let text = cx.argument::<JsString>(0)?.value(&mut cx);
    let normalized = utils::normalize_line_endings(&text);
    Ok(cx.string(normalized))
}

