//! File tracking and change detection for the checkpoint system

use crate::config::CheckpointConfig;
use crate::error::{CheckpointError, Result};
use crate::types::*;
use ignore::{Walk, WalkBuilder};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Tracks file changes in a workspace
pub struct FileTracker {
    config: CheckpointConfig,
    workspace_path: PathBuf,
    previous_state: HashMap<PathBuf, FileState>,

    /// Performance metrics
    scan_stats: ScanStatistics,
}

/// State of a file at a point in time
#[derive(Debug, Clone)]
pub struct FileState {
    hash: String,
    size: u64,
    permissions: Option<u32>,
}

/// Performance statistics for file scanning
#[derive(Debug, Clone, Default)]
pub struct ScanStatistics {
    pub total_scans: u64,
    pub total_files_scanned: u64,
    pub total_directories_scanned: u64,
    pub last_scan_duration_ms: u64,
    pub average_scan_duration_ms: u64,
    pub files_skipped_size: u64,
    pub files_skipped_depth: u64,
}

impl FileTracker {
    /// Create a new file tracker
    pub fn new(config: CheckpointConfig, workspace_path: PathBuf) -> Self {
        Self {
            config,
            workspace_path,
            previous_state: HashMap::new(),
            scan_stats: ScanStatistics::default(),
        }
    }

    /// Get performance statistics
    pub fn get_statistics(&self) -> &ScanStatistics {
        &self.scan_stats
    }

    /// Capture the current state of all tracked files with performance tracking
    pub fn capture_current_state(&mut self) -> Result<HashMap<PathBuf, FileState>> {
        let start_time = std::time::Instant::now();
        let mut current_state = HashMap::new();
        let mut files_scanned = 0u64;
        let mut dirs_scanned = 0u64;
        let mut files_skipped_size = 0u64;

        for entry in self.build_walker()? {
            let entry =
                entry.map_err(|e| CheckpointError::file_system(format!("Walk error: {}", e)))?;
            let path = entry.path();

            if path.is_dir() {
                dirs_scanned += 1;
            } else if path.is_file() {
                files_scanned += 1;

                if let Some(relative_path) = path.strip_prefix(&self.workspace_path).ok() {
                    // Check file size limit before processing
                    if let Ok(metadata) = fs::metadata(path) {
                        if metadata.len() > self.config.max_file_size_bytes {
                            files_skipped_size += 1;
                            if self.config.verbose_logging {
                                log::debug!(
                                    "Skipping large file: {} ({} bytes > {} bytes)",
                                    relative_path.display(),
                                    metadata.len(),
                                    self.config.max_file_size_bytes
                                );
                            }
                            continue;
                        }
                    }

                    if self.should_track_file(relative_path) {
                        match self.get_file_state(path) {
                            Ok(state) => {
                                current_state.insert(relative_path.to_path_buf(), state);
                            }
                            Err(e) => {
                                log::warn!(
                                    "Failed to get state for file {}: {}",
                                    path.display(),
                                    e
                                );
                            }
                        }
                    }
                }
            }
        }

        // Update performance statistics if enabled
        if self.config.enable_performance_metrics {
            let duration_ms = start_time.elapsed().as_millis() as u64;
            self.update_scan_statistics(
                duration_ms,
                files_scanned,
                dirs_scanned,
                files_skipped_size,
            );

            if self.config.verbose_logging {
                log::info!(
                    "Scan completed: {} files in {} directories ({} ms, {} files skipped by size)",
                    files_scanned,
                    dirs_scanned,
                    duration_ms,
                    files_skipped_size
                );
            }
        }

        Ok(current_state)
    }

    /// Update scan statistics
    fn update_scan_statistics(
        &mut self,
        duration_ms: u64,
        files: u64,
        dirs: u64,
        skipped_size: u64,
    ) {
        self.scan_stats.total_scans += 1;
        self.scan_stats.total_files_scanned += files;
        self.scan_stats.total_directories_scanned += dirs;
        self.scan_stats.last_scan_duration_ms = duration_ms;
        self.scan_stats.files_skipped_size += skipped_size;

        // Calculate rolling average
        let total = self.scan_stats.total_scans;
        self.scan_stats.average_scan_duration_ms =
            (self.scan_stats.average_scan_duration_ms * (total - 1) + duration_ms) / total;
    }

    /// Detect changes between previous and current state
    pub fn detect_changes(&mut self) -> Result<Vec<FileChange>> {
        let current_state = self.capture_current_state()?;
        let mut changes = Vec::new();

        // Find modified and deleted files
        for (path, prev_state) in &self.previous_state {
            if let Some(current_state) = current_state.get(path) {
                // File exists in both states
                if prev_state.hash != current_state.hash {
                    // File was modified
                    changes.push(self.create_file_change(
                        path.clone(),
                        ChangeType::Modified,
                        Some(prev_state),
                        Some(current_state),
                    )?);
                }
            } else {
                // File was deleted
                changes.push(self.create_file_change(
                    path.clone(),
                    ChangeType::Deleted,
                    Some(prev_state),
                    None,
                )?);
            }
        }

        // Find created files
        for (path, current_state) in &current_state {
            if !self.previous_state.contains_key(path) {
                // File was created
                changes.push(self.create_file_change(
                    path.clone(),
                    ChangeType::Created,
                    None,
                    Some(current_state),
                )?);
            }
        }

        // Update previous state
        self.previous_state = current_state;

        Ok(changes)
    }

    /// Create a file change record
    fn create_file_change(
        &self,
        file_path: PathBuf,
        change_type: ChangeType,
        prev_state: Option<&FileState>,
        current_state: Option<&FileState>,
    ) -> Result<FileChange> {
        let full_path = self.workspace_path.join(&file_path);

        let (original_content, original_hash, _original_size) = if let Some(state) = prev_state {
            let content = if change_type != ChangeType::Deleted {
                None // Don't read content for non-deleted files in prev state
            } else {
                None // We don't have the content for deleted files
            };
            (content, Some(state.hash.clone()), Some(state.size))
        } else {
            (None, None, None)
        };

        let (modified_content, modified_hash, _modified_size) = if let Some(state) = current_state {
            let content = if change_type != ChangeType::Deleted {
                Some(self.read_file_content(&full_path)?)
            } else {
                None
            };
            (content, Some(state.hash.clone()), Some(state.size))
        } else {
            (None, None, None)
        };

        let permissions = current_state
            .or(prev_state)
            .and_then(|state| state.permissions);

        use chrono::Utc;

        Ok(FileChange {
            path: file_path,
            change_type,
            original_content,
            new_content: modified_content,
            size_bytes: current_state
                .map(|s| s.size)
                .or(prev_state.map(|s| s.size))
                .unwrap_or(0),
            content_hash: modified_hash.or(original_hash).unwrap_or_default(),
            permissions,
            modified_at: Utc::now(),
            encoding: FileEncoding::Utf8,
            compressed: false,
        })
    }

    /// Get the current state of a file
    fn get_file_state(&self, path: &Path) -> Result<FileState> {
        let metadata = fs::metadata(path)?;
        let content = self.read_file_content(path)?;
        let hash = self.calculate_hash(&content);

        let permissions = {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                Some(metadata.permissions().mode())
            }
            #[cfg(not(unix))]
            {
                None
            }
        };

        Ok(FileState {
            hash,
            size: metadata.len(),
            permissions,
        })
    }

    /// Read file content as string
    fn read_file_content(&self, path: &Path) -> Result<String> {
        fs::read_to_string(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::InvalidData {
                CheckpointError::file_system(format!("File is not valid UTF-8: {}", path.display()))
            } else {
                CheckpointError::from(e)
            }
        })
    }

    /// Calculate SHA-256 hash of content
    fn calculate_hash(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Check if a file should be tracked
    fn should_track_file(&self, path: &Path) -> bool {
        // Check file extension
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            if !self.config.should_track_extension(extension) {
                return false;
            }
        } else {
            // Files without extension - check if it's a common text file
            if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
                match file_name.to_lowercase().as_str() {
                    "readme" | "license" | "changelog" | "makefile" | "dockerfile" => return true,
                    _ => return false,
                }
            }
            return false;
        }

        // Check against ignore patterns
        let path_str = path.to_string_lossy();
        for pattern in &self.config.global_ignore_patterns {
            if glob_match(pattern, &path_str) {
                return false;
            }
        }

        true
    }

    /// Build a file walker with proper ignore rules
    fn build_walker(&self) -> Result<Walk> {
        let mut builder = WalkBuilder::new(&self.workspace_path);

        // Add custom ignore patterns
        for pattern in &self.config.global_ignore_patterns {
            builder.add_ignore(&format!("**/{}", pattern));
        }

        // Respect .gitignore and .knoxignore files
        builder.add_custom_ignore_filename(".knoxignore");

        // Don't follow symlinks
        builder.follow_links(false);

        // Skip hidden files and directories
        builder.hidden(false);

        Ok(builder.build())
    }

    /// Set the previous state (useful for initialization)
    pub fn set_previous_state(&mut self, state: HashMap<PathBuf, FileState>) {
        self.previous_state = state;
    }

    /// Get files that match specific patterns
    pub fn get_files_matching(&self, patterns: &[String]) -> Result<Vec<PathBuf>> {
        let mut matching_files = Vec::new();

        for entry in self.build_walker()? {
            let entry =
                entry.map_err(|e| CheckpointError::file_system(format!("Walk error: {}", e)))?;
            let path = entry.path();

            if path.is_file() {
                if let Some(relative_path) = path.strip_prefix(&self.workspace_path).ok() {
                    let path_str = relative_path.to_string_lossy();

                    for pattern in patterns {
                        if glob_match(pattern, &path_str) {
                            matching_files.push(relative_path.to_path_buf());
                            break;
                        }
                    }
                }
            }
        }

        Ok(matching_files)
    }

    /// Load file content with the given hash
    pub fn load_file_content(&self, file_path: &Path, hash: &str) -> Result<Option<String>> {
        let full_path = self.workspace_path.join(file_path);

        if full_path.exists() {
            let content = self.read_file_content(&full_path)?;
            let current_hash = self.calculate_hash(&content);

            if current_hash == hash {
                Ok(Some(content))
            } else {
                Ok(None) // File has changed since checkpoint
            }
        } else {
            Ok(None) // File doesn't exist
        }
    }
}

/// Simple glob pattern matching
fn glob_match(pattern: &str, text: &str) -> bool {
    // This is a simplified glob matcher
    // For production, you might want to use a proper glob library
    if pattern.contains("**") {
        let parts: Vec<&str> = pattern.split("**").collect();
        if parts.len() == 2 {
            let prefix = parts[0];
            let suffix = parts[1];

            return text.starts_with(prefix) && text.ends_with(suffix);
        }
    }

    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            let prefix = parts[0];
            let suffix = parts[1];

            return text.starts_with(prefix) && text.ends_with(suffix);
        }
    }

    pattern == text
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_tracker() -> (FileTracker, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = CheckpointConfig::default();
        let tracker = FileTracker::new(config, temp_dir.path().to_path_buf());
        (tracker, temp_dir)
    }

    #[test]
    fn test_file_state_capture() {
        let (mut tracker, temp_dir) = create_test_tracker();

        // Create a test file
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "Hello, world!").unwrap();

        let state = tracker.capture_current_state().unwrap();
        assert_eq!(state.len(), 1);

        let file_state = state.get(&PathBuf::from("test.txt")).unwrap();
        assert_eq!(file_state.size, 13);
        assert!(!file_state.hash.is_empty());
    }

    #[test]
    fn test_change_detection() {
        let (mut tracker, temp_dir) = create_test_tracker();

        // Create initial file
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "Hello, world!").unwrap();

        // Capture initial state
        let initial_state = tracker.capture_current_state().unwrap();
        tracker.set_previous_state(initial_state);

        // Modify the file
        fs::write(&test_file, "Hello, Rust!").unwrap();

        // Detect changes
        let changes = tracker.detect_changes().unwrap();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].change_type, ChangeType::Modified);
        assert_eq!(changes[0].file_path, PathBuf::from("test.txt"));
    }

    #[test]
    fn test_file_creation_detection() {
        let (mut tracker, temp_dir) = create_test_tracker();

        // Capture empty state
        let initial_state = tracker.capture_current_state().unwrap();
        tracker.set_previous_state(initial_state);

        // Create a new file
        let test_file = temp_dir.path().join("new_file.txt");
        fs::write(&test_file, "New content").unwrap();

        // Detect changes
        let changes = tracker.detect_changes().unwrap();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].change_type, ChangeType::Created);
        assert_eq!(changes[0].file_path, PathBuf::from("new_file.txt"));
    }

    #[test]
    fn test_file_deletion_detection() {
        let (mut tracker, temp_dir) = create_test_tracker();

        // Create initial file
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "Hello, world!").unwrap();

        // Capture initial state
        let initial_state = tracker.capture_current_state().unwrap();
        tracker.set_previous_state(initial_state);

        // Delete the file
        fs::remove_file(&test_file).unwrap();

        // Detect changes
        let changes = tracker.detect_changes().unwrap();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].change_type, ChangeType::Deleted);
        assert_eq!(changes[0].file_path, PathBuf::from("test.txt"));
    }

    #[test]
    fn test_glob_matching() {
        assert!(glob_match("*.txt", "test.txt"));
        assert!(glob_match("**/*.js", "src/components/test.js"));
        assert!(glob_match(
            "node_modules/**",
            "node_modules/package/index.js"
        ));
        assert!(!glob_match("*.js", "test.txt"));
    }
}
