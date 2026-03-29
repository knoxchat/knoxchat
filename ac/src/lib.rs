//! # Knox Enterprise Autocomplete System
//! 
//! A high-performance autocomplete system built in Rust with FFI bindings for Node.js.
//! This module provides intelligent code completion with caching, context awareness,
//! and support for multiple programming languages.

pub mod cache;
pub mod context;
pub mod error;
pub mod ffi;
pub mod types;
pub mod utils;

// Re-exports for convenience
pub use cache::AutocompleteLruCache;
pub use context::ContextRetrievalService;
pub use error::{AutocompleteError, Result};
pub use types::*;

use neon::prelude::*;

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    // Cache functions
    cx.export_function("initializeCache", ffi::initialize_cache)?;
    cx.export_function("cacheGet", ffi::cache_get)?;
    cx.export_function("cachePut", ffi::cache_put)?;
    cx.export_function("cacheClear", ffi::cache_clear)?;
    cx.export_function("cacheStats", ffi::cache_stats)?;

    // Context functions
    cx.export_function("initializeContext", ffi::initialize_context)?;
    cx.export_function("updateImports", ffi::update_imports)?;
    cx.export_function("getImportSnippets", ffi::get_import_snippets)?;

    // Utility functions
    cx.export_function("pruneLinesFromTop", ffi::prune_lines_from_top)?;
    cx.export_function("pruneLinesFromBottom", ffi::prune_lines_from_bottom)?;
    cx.export_function("countTokens", ffi::count_tokens)?;
    cx.export_function("extractSymbols", ffi::extract_symbols)?;
    cx.export_function("getLanguageInfo", ffi::get_language_info)?;
    cx.export_function("normalizeLineEndings", ffi::normalize_line_endings)?;

    Ok(())
}

