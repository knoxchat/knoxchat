//! Type definitions for autocomplete system

use serde::{Deserialize, Serialize};

/// Autocomplete snippet types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AutocompleteSnippetType {
    Code,
    Diff,
    Clipboard,
}

/// Base autocomplete snippet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutocompleteCodeSnippet {
    pub filepath: String,
    pub content: String,
    #[serde(rename = "type")]
    pub snippet_type: AutocompleteSnippetType,
}

/// Position in a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

/// Range in a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

/// Recently edited range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentlyEditedRange {
    pub filepath: String,
    pub range: Range,
    pub timestamp: i64,
    pub lines: Vec<String>,
    pub symbols: Vec<String>,
}

/// Autocomplete input from Node.js
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutocompleteInput {
    pub is_untitled_file: bool,
    pub completion_id: String,
    pub filepath: String,
    pub pos: Position,
    pub recently_visited_ranges: Vec<AutocompleteCodeSnippet>,
    pub recently_edited_ranges: Vec<RecentlyEditedRange>,
    pub manually_pass_file_contents: Option<String>,
    pub manually_pass_prefix: Option<String>,
    pub selected_completion_info: Option<SelectedCompletionInfo>,
    pub inject_details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedCompletionInfo {
    pub text: String,
    pub range: Range,
}

/// Tab autocomplete options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TabAutocompleteOptions {
    pub disable: bool,
    pub use_file_suffix: bool,
    pub max_prompt_tokens: u32,
    pub debounce_delay: u32,
    pub max_suffix_percentage: f64,
    pub prefix_percentage: f64,
    pub transform: bool,
    pub template: Option<String>,
    pub multiline_completions: String, // "always", "never", "auto"
    pub sliding_window_prefix_percentage: f64,
    pub sliding_window_size: u32,
    pub use_cache: bool,
    pub only_my_code: bool,
    pub use_recently_edited: bool,
    pub use_imports: bool,
}

impl Default for TabAutocompleteOptions {
    fn default() -> Self {
        Self {
            disable: false,
            use_file_suffix: true,
            max_prompt_tokens: 1024,
            debounce_delay: 300,
            max_suffix_percentage: 0.25,
            prefix_percentage: 0.75,
            transform: true,
            template: None,
            multiline_completions: "auto".to_string(),
            sliding_window_prefix_percentage: 0.75,
            sliding_window_size: 512,
            use_cache: true,
            only_my_code: false,
            use_recently_edited: true,
            use_imports: true,
        }
    }
}

/// Autocomplete outcome
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutocompleteOutcome {
    pub accepted: Option<bool>,
    pub time: u64,
    pub prefix: String,
    pub suffix: String,
    pub prompt: String,
    pub completion: String,
    pub model_provider: String,
    pub model_name: String,
    pub completion_options: serde_json::Value,
    pub cache_hit: bool,
    pub num_lines: usize,
    pub filepath: String,
    pub git_repo: Option<String>,
    pub completion_id: String,
    pub unique_id: String,
    pub timestamp: i64,
}

/// Language information for autocomplete
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutocompleteLanguageInfo {
    pub name: String,
    pub top_level_keywords: Vec<String>,
    pub comment_start: String,
    pub comment_end: Option<String>,
    pub line_comment: String,
    pub block_comment: Option<(String, String)>,
}

impl AutocompleteLanguageInfo {
    pub fn for_filepath(filepath: &str) -> Self {
        let extension = std::path::Path::new(filepath)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match extension {
            "ts" | "tsx" | "js" | "jsx" => Self {
                name: "typescript".to_string(),
                top_level_keywords: vec![
                    "function".to_string(),
                    "class".to_string(),
                    "interface".to_string(),
                    "type".to_string(),
                    "const".to_string(),
                    "let".to_string(),
                    "var".to_string(),
                    "import".to_string(),
                    "export".to_string(),
                ],
                comment_start: "//".to_string(),
                comment_end: None,
                line_comment: "//".to_string(),
                block_comment: Some(("/*".to_string(), "*/".to_string())),
            },
            "py" => Self {
                name: "python".to_string(),
                top_level_keywords: vec![
                    "def".to_string(),
                    "class".to_string(),
                    "import".to_string(),
                    "from".to_string(),
                    "async".to_string(),
                ],
                comment_start: "#".to_string(),
                comment_end: None,
                line_comment: "#".to_string(),
                block_comment: Some(("\"\"\"".to_string(), "\"\"\"".to_string())),
            },
            "rs" => Self {
                name: "rust".to_string(),
                top_level_keywords: vec![
                    "fn".to_string(),
                    "struct".to_string(),
                    "enum".to_string(),
                    "impl".to_string(),
                    "trait".to_string(),
                    "mod".to_string(),
                    "use".to_string(),
                    "pub".to_string(),
                ],
                comment_start: "//".to_string(),
                comment_end: None,
                line_comment: "//".to_string(),
                block_comment: Some(("/*".to_string(), "*/".to_string())),
            },
            "go" => Self {
                name: "go".to_string(),
                top_level_keywords: vec![
                    "func".to_string(),
                    "type".to_string(),
                    "package".to_string(),
                    "import".to_string(),
                    "var".to_string(),
                    "const".to_string(),
                ],
                comment_start: "//".to_string(),
                comment_end: None,
                line_comment: "//".to_string(),
                block_comment: Some(("/*".to_string(), "*/".to_string())),
            },
            "java" => Self {
                name: "java".to_string(),
                top_level_keywords: vec![
                    "class".to_string(),
                    "interface".to_string(),
                    "enum".to_string(),
                    "package".to_string(),
                    "import".to_string(),
                    "public".to_string(),
                    "private".to_string(),
                ],
                comment_start: "//".to_string(),
                comment_end: None,
                line_comment: "//".to_string(),
                block_comment: Some(("/*".to_string(), "*/".to_string())),
            },
            _ => Self {
                name: "unknown".to_string(),
                top_level_keywords: vec![],
                comment_start: "//".to_string(),
                comment_end: None,
                line_comment: "//".to_string(),
                block_comment: None,
            },
        }
    }
}

/// Cache entry for autocomplete
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub key: String,
    pub value: String,
    pub timestamp: i64,
}

/// Statistics for the autocomplete cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub total_size_bytes: u64,
}

