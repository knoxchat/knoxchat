//! Utility functions for the autocomplete system

use crate::error::Result;
use std::path::{Path, PathBuf};

/// Get the Knox global path for storing data
pub fn get_knox_global_path() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| {
        crate::error::AutocompleteError::Internal("Could not find home directory".to_string())
    })?;

    let knox_path = home_dir.join(".knox");
    
    // Create the directory if it doesn't exist
    if !knox_path.exists() {
        std::fs::create_dir_all(&knox_path)?;
    }

    Ok(knox_path)
}

/// Get the autocomplete cache database path
pub fn get_autocomplete_cache_path() -> Result<PathBuf> {
    let knox_path = get_knox_global_path()?;
    Ok(knox_path.join("autocompleteCache.sqlite"))
}

/// Prune lines from the top of text to fit within token limit
pub fn prune_lines_from_top(text: &str, max_tokens: u32) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut total_tokens = 0;
    let mut result = Vec::new();

    // Rough estimation: 4 chars = 1 token
    let tokens_per_char = 0.25;

    for line in lines.iter().rev() {
        let line_tokens = (line.len() as f64 * tokens_per_char) as u32;
        if total_tokens + line_tokens > max_tokens && !result.is_empty() {
            break;
        }
        result.push(*line);
        total_tokens += line_tokens;
    }

    result.reverse();
    result.join("\n")
}

/// Prune lines from the bottom of text to fit within token limit
pub fn prune_lines_from_bottom(text: &str, max_tokens: u32) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut total_tokens = 0;
    let mut result = Vec::new();

    // Rough estimation: 4 chars = 1 token
    let tokens_per_char = 0.25;

    for line in lines.iter() {
        let line_tokens = (line.len() as f64 * tokens_per_char) as u32;
        if total_tokens + line_tokens > max_tokens && !result.is_empty() {
            break;
        }
        result.push(*line);
        total_tokens += line_tokens;
    }

    result.join("\n")
}

/// Count tokens in text (rough estimation)
pub fn count_tokens(text: &str) -> u32 {
    // Rough estimation: 4 chars = 1 token
    (text.len() as f64 * 0.25) as u32
}

/// Extract symbols from text (identifiers, function names, etc.)
pub fn extract_symbols(text: &str) -> Vec<String> {
    let re = regex::Regex::new(r"\b[a-zA-Z_][a-zA-Z0-9_]*\b").unwrap();
    re.find_iter(text)
        .map(|m| m.as_str().to_string())
        .collect()
}

/// Get the file extension from a path
pub fn get_file_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
}

/// Check if a file is a supported source code file
pub fn is_source_file(path: &Path) -> bool {
    let supported_extensions = vec![
        "ts", "tsx", "js", "jsx", "py", "rs", "go", "java", "c", "cpp", "h", "hpp",
        "cs", "rb", "php", "swift", "kt", "scala", "clj", "ex", "exs",
    ];

    if let Some(ext) = get_file_extension(path) {
        supported_extensions.contains(&ext.as_str())
    } else {
        false
    }
}

/// Normalize line endings to \n
pub fn normalize_line_endings(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

/// Get the current timestamp in milliseconds
pub fn current_timestamp_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prune_lines_from_top() {
        let text = "line1\nline2\nline3\nline4\nline5";
        let pruned = prune_lines_from_top(text, 10);
        assert!(pruned.lines().count() <= 5);
    }

    #[test]
    fn test_count_tokens() {
        let text = "Hello world";
        let tokens = count_tokens(text);
        assert!(tokens > 0);
    }

    #[test]
    fn test_extract_symbols() {
        let text = "const foo = bar.baz()";
        let symbols = extract_symbols(text);
        assert!(symbols.contains(&"foo".to_string()));
        assert!(symbols.contains(&"bar".to_string()));
        assert!(symbols.contains(&"baz".to_string()));
    }

    #[test]
    fn test_normalize_line_endings() {
        let text = "line1\r\nline2\rline3\n";
        let normalized = normalize_line_endings(text);
        assert_eq!(normalized, "line1\nline2\nline3\n");
    }
}

