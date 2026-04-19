//! Utility functions for the checkpoint system

use crate::error::{CheckpointError, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Get the knox global path, similar to getKnoxGlobalPath utility
pub fn get_knox_global_path(debug_mode: bool) -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| CheckpointError::file_system("Unable to get home directory"))?;

    if debug_mode {
        Ok(home.join(".knox-debug"))
    } else {
        Ok(home.join(".knox"))
    }
}

/// Format file size in human readable format
pub fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    const THRESHOLD: f64 = 1024.0;

    if bytes == 0 {
        return "0 B".to_string();
    }

    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= THRESHOLD && unit_index < UNITS.len() - 1 {
        size /= THRESHOLD;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Calculate the relative path between two paths
pub fn relative_path_from(path: &Path, base: &Path) -> Result<PathBuf> {
    path.strip_prefix(base)
        .map(|p| p.to_path_buf())
        .map_err(|_| {
            CheckpointError::file_system(format!(
                "Path {} is not relative to {}",
                path.display(),
                base.display()
            ))
        })
}

/// Ensure a directory exists, creating it if necessary
pub fn ensure_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path).map_err(|e| {
            CheckpointError::file_system(format!(
                "Failed to create directory {}: {}",
                path.display(),
                e
            ))
        })?;
    } else if !path.is_dir() {
        return Err(CheckpointError::file_system(format!(
            "Path {} exists but is not a directory",
            path.display()
        )));
    }
    Ok(())
}

/// Check if a path is safe to write to (not system directories, etc.)
pub fn is_safe_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();

    // Check for dangerous system paths
    let dangerous_patterns = [
        "/system",
        "/windows",
        "/program files",
        "/boot",
        "c:\\windows",
        "c:\\system32",
        "/usr/bin",
        "/bin",
        "/sbin",
        "/etc",
        "/var",
        "/tmp",
        "/dev",
        "/proc",
        "/sys",
    ];

    for pattern in &dangerous_patterns {
        if path_str.starts_with(pattern) {
            return false;
        }
    }

    // Check for relative path traversal
    for component in path.components() {
        if component.as_os_str() == ".." {
            return false;
        }
    }

    true
}

/// Sanitize a filename by removing or replacing invalid characters
pub fn sanitize_filename(filename: &str) -> String {
    let invalid_chars = ['<', '>', ':', '"', '|', '?', '*', '/', '\\'];
    let mut sanitized = String::new();

    for ch in filename.chars() {
        if invalid_chars.contains(&ch) || ch.is_control() {
            sanitized.push('_');
        } else {
            sanitized.push(ch);
        }
    }

    // Trim and ensure it's not empty
    let sanitized = sanitized.trim();
    if sanitized.is_empty() {
        "unnamed".to_string()
    } else {
        sanitized.to_string()
    }
}

/// Check if two file paths refer to the same file
pub fn paths_equal(path1: &Path, path2: &Path) -> bool {
    match (path1.canonicalize(), path2.canonicalize()) {
        (Ok(p1), Ok(p2)) => p1 == p2,
        _ => path1 == path2, // Fallback to simple comparison
    }
}

/// Get file extension in lowercase
pub fn get_file_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
}

/// Check if a file is likely to be a text file based on its extension
pub fn is_text_file(path: &Path) -> bool {
    const TEXT_EXTENSIONS: &[&str] = &[
        "txt", "md", "json", "yaml", "yml", "toml", "xml", "html", "css", "js", "ts", "jsx", "tsx",
        "py", "rs", "go", "java", "cpp", "c", "h", "cs", "php", "rb", "swift", "kt", "scala", "sh",
        "bat", "ps1", "sql", "r", "m", "mm", "pl", "pm", "tcl", "vbs", "asm", "s", "f", "f90",
        "pas", "pp", "inc", "lpr", "lfm", "dpr", "dpk", "cfg", "ini", "conf", "log", "csv", "tsv",
        "rtf", "tex", "bib", "cls", "sty", "dtx", "ins",
    ];

    if let Some(ext) = get_file_extension(path) {
        TEXT_EXTENSIONS.contains(&ext.as_str())
    } else {
        // Check common extensionless text files
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            matches!(
                filename.to_lowercase().as_str(),
                "readme"
                    | "license"
                    | "changelog"
                    | "makefile"
                    | "dockerfile"
                    | "gemfile"
                    | "rakefile"
                    | "podfile"
                    | "brewfile"
                    | "procfile"
            )
        } else {
            false
        }
    }
}

/// Generate a short hash from a string (for display purposes)
pub fn short_hash(input: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let hash = hasher.finish();

    format!("{:08x}", hash & 0xFFFFFFFF)
}

/// Format a timestamp as a human-readable string
pub fn format_timestamp(timestamp: chrono::DateTime<chrono::Utc>) -> String {
    let local: chrono::DateTime<chrono::Local> = timestamp.into();
    local.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Calculate the depth of a path (number of components)
pub fn path_depth(path: &Path) -> usize {
    path.components().count()
}

/// Get the common prefix of multiple paths
pub fn common_path_prefix(paths: &[PathBuf]) -> Option<PathBuf> {
    if paths.is_empty() {
        return None;
    }

    let mut common = paths[0].clone();

    for path in paths.iter().skip(1) {
        let mut new_common = PathBuf::new();

        for (c1, c2) in common.components().zip(path.components()) {
            if c1 == c2 {
                new_common.push(c1);
            } else {
                break;
            }
        }

        common = new_common;

        if common.as_os_str().is_empty() {
            break;
        }
    }

    if common.as_os_str().is_empty() {
        None
    } else {
        Some(common)
    }
}

/// Truncate a string to a maximum length, adding ellipsis if needed
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        "...".to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(1023), "1023 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1048576), "1.0 MB");
        assert_eq!(format_file_size(1073741824), "1.0 GB");
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("normal_file.txt"), "normal_file.txt");
        assert_eq!(sanitize_filename("file<>:\"|?*.txt"), "file_______.txt");
        assert_eq!(sanitize_filename("   "), "unnamed");
        assert_eq!(sanitize_filename(""), "unnamed");
        assert_eq!(
            sanitize_filename("file\nwith\tcontrol"),
            "file_with_control"
        );
    }

    #[test]
    fn test_is_safe_path() {
        assert!(is_safe_path(Path::new("safe/path/file.txt")));
        assert!(is_safe_path(Path::new("/home/user/documents/file.txt")));

        assert!(!is_safe_path(Path::new("/system/file.txt")));
        assert!(!is_safe_path(Path::new("C:\\Windows\\file.txt")));
        assert!(!is_safe_path(Path::new("../../../etc/passwd")));
        assert!(!is_safe_path(Path::new("safe/../../../etc/passwd")));
    }

    #[test]
    fn test_get_file_extension() {
        assert_eq!(
            get_file_extension(Path::new("file.txt")),
            Some("txt".to_string())
        );
        assert_eq!(
            get_file_extension(Path::new("file.TXT")),
            Some("txt".to_string())
        );
        assert_eq!(get_file_extension(Path::new("file")), None);
        assert_eq!(
            get_file_extension(Path::new("path/file.js")),
            Some("js".to_string())
        );
    }

    #[test]
    fn test_is_text_file() {
        assert!(is_text_file(Path::new("file.txt")));
        assert!(is_text_file(Path::new("script.js")));
        assert!(is_text_file(Path::new("README")));
        assert!(is_text_file(Path::new("Makefile")));

        assert!(!is_text_file(Path::new("image.png")));
        assert!(!is_text_file(Path::new("binary.exe")));
    }

    #[test]
    fn test_ensure_dir_exists() {
        let temp_dir = TempDir::new().unwrap();
        let new_dir = temp_dir.path().join("new_directory");

        assert!(!new_dir.exists());
        ensure_dir_exists(&new_dir).unwrap();
        assert!(new_dir.exists());
        assert!(new_dir.is_dir());

        // Should not fail if directory already exists
        ensure_dir_exists(&new_dir).unwrap();
    }

    #[test]
    fn test_relative_path_from() {
        let base = Path::new("/home/user");
        let path = Path::new("/home/user/documents/file.txt");

        let relative = relative_path_from(path, base).unwrap();
        assert_eq!(relative, Path::new("documents/file.txt"));
    }

    #[test]
    fn test_common_path_prefix() {
        let paths = vec![
            PathBuf::from("/home/user/docs/file1.txt"),
            PathBuf::from("/home/user/docs/file2.txt"),
            PathBuf::from("/home/user/docs/subdir/file3.txt"),
        ];

        let common = common_path_prefix(&paths).unwrap();
        assert_eq!(common, Path::new("/home/user/docs"));

        let different_paths = vec![
            PathBuf::from("/home/user1/file.txt"),
            PathBuf::from("/var/log/file.txt"),
        ];

        let common = common_path_prefix(&different_paths).unwrap();
        assert_eq!(common, Path::new("/"));
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("short", 10), "short");
        assert_eq!(
            truncate_string("this is a very long string", 10),
            "this is..."
        );
        assert_eq!(truncate_string("abc", 3), "abc");
        assert_eq!(truncate_string("abcd", 3), "...");
    }

    #[test]
    fn test_short_hash() {
        let hash1 = short_hash("test string 1");
        let hash2 = short_hash("test string 2");
        let hash3 = short_hash("test string 1"); // Same as hash1

        assert_ne!(hash1, hash2);
        assert_eq!(hash1, hash3);
        assert_eq!(hash1.len(), 8); // 8 hex characters
    }

    #[test]
    fn test_path_depth() {
        assert_eq!(path_depth(Path::new("file.txt")), 1);
        assert_eq!(path_depth(Path::new("dir/file.txt")), 2);
        assert_eq!(path_depth(Path::new("/home/user/docs/file.txt")), 5);
        assert_eq!(path_depth(Path::new("")), 0);
    }
}
