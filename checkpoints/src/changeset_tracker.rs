//! Smart changeset-based file tracking for AI agent interactions
//!
//! This module implements a Git-like changeset tracking system that:
//! - Only tracks files that AI agents actually modify
//! - Uses file watchers instead of full filesystem scans
//! - Focuses on deltas rather than full file states
//! - Distinguishes between agent mode and chat mode interactions

use crate::config::CheckpointConfig;
use crate::error::{CheckpointError, Result};
use crate::types::*;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Mode of operation for the checkpoint system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationMode {
    /// Agent mode - AI is actively making changes that should be tracked
    Agent,
    /// Chat mode - User is chatting, changes should not be tracked
    Chat,
    /// Manual mode - User is manually making changes outside of AI
    Manual,
}

/// Represents a single file change with minimal memory footprint
#[derive(Debug, Clone)]
pub struct ChangesetEntry {
    pub file_path: PathBuf,
    pub change_type: ChangeType,
    pub timestamp: SystemTime,
    pub content_hash: String,
    pub size_bytes: u64,
    pub agent_session_id: Option<SessionId>,
    /// Only store actual content for small files or critical changes
    pub content: Option<String>,
}

/// Tracks only AI-generated changes with minimal memory usage
pub struct ChangesetTracker {
    config: CheckpointConfig,
    workspace_path: PathBuf,

    /// Current operation mode
    current_mode: Arc<Mutex<OperationMode>>,

    /// Active agent session (only track changes during agent sessions)
    active_agent_session: Arc<Mutex<Option<SessionId>>>,

    /// Pending changes that haven't been checkpointed yet
    pending_changes: Arc<Mutex<HashMap<PathBuf, ChangesetEntry>>>,

    /// Files currently being watched
    watched_files: Arc<Mutex<HashSet<PathBuf>>>,

    /// Baseline file states (to detect Created vs Modified)
    baseline_file_states: Arc<Mutex<HashMap<PathBuf, String>>>,

    /// File system watcher
    watcher: Option<RecommendedWatcher>,
    file_events: Arc<Mutex<Receiver<notify::Result<Event>>>>,

    /// Performance tracking
    stats: Arc<Mutex<ChangesetStats>>,
}

#[derive(Debug, Default)]
pub struct ChangesetStats {
    pub files_tracked: usize,
    pub changes_detected: usize,
    pub memory_usage_bytes: usize,
    pub last_scan_duration_ms: u64,
}

impl ChangesetTracker {
    /// Create a new changeset tracker
    pub fn new(config: CheckpointConfig, workspace_path: PathBuf) -> Result<Self> {
        let (sender, receiver): (
            Sender<notify::Result<Event>>,
            Receiver<notify::Result<Event>>,
        ) = channel();

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                if let Err(e) = sender.send(res) {
                    log::error!("Failed to send file system event: {}", e);
                }
            },
            Config::default(),
        )?;

        // Start watching the workspace
        watcher.watch(&workspace_path, RecursiveMode::Recursive)?;

        Ok(Self {
            config,
            workspace_path,
            current_mode: Arc::new(Mutex::new(OperationMode::Chat)),
            active_agent_session: Arc::new(Mutex::new(None)),
            pending_changes: Arc::new(Mutex::new(HashMap::new())),
            watched_files: Arc::new(Mutex::new(HashSet::new())),
            baseline_file_states: Arc::new(Mutex::new(HashMap::new())),
            watcher: Some(watcher),
            file_events: Arc::new(Mutex::new(receiver)),
            stats: Arc::new(Mutex::new(ChangesetStats::default())),
        })
    }

    /// Set the current operation mode
    pub fn set_mode(&self, mode: OperationMode) {
        let mut current_mode = self.current_mode.lock().unwrap();
        *current_mode = mode;

        log::info!("Changeset tracker mode changed to {:?}", mode);

        // Clear pending changes when switching from agent mode to avoid
        // tracking user changes as AI changes
        if mode != OperationMode::Agent {
            let mut pending = self.pending_changes.lock().unwrap();
            pending.clear();
            log::debug!("Cleared pending changes due to mode switch");
        }
    }

    /// Start tracking changes for an agent session
    pub fn start_agent_session(&self, session_id: SessionId) {
        let mut active_session = self.active_agent_session.lock().unwrap();
        *active_session = Some(session_id);
        self.set_mode(OperationMode::Agent);

        log::info!("Started agent session tracking: {}", session_id);
    }

    /// Stop tracking the current agent session
    pub fn stop_agent_session(&self) {
        let mut active_session = self.active_agent_session.lock().unwrap();
        *active_session = None;
        self.set_mode(OperationMode::Chat);

        log::info!("Stopped agent session tracking");
    }

    /// Add specific files to watch (called when AI starts working on them)
    pub fn watch_files(&self, file_paths: &[PathBuf]) -> Result<()> {
        let mut watched = self.watched_files.lock().unwrap();
        let mut baseline = self.baseline_file_states.lock().unwrap();

        for path in file_paths {
            let full_path = self.workspace_path.join(path);

            // Capture baseline state if file exists
            if full_path.exists() {
                if let Ok(content) = fs::read_to_string(&full_path) {
                    let hash = self.calculate_hash(&content);
                    baseline.insert(path.clone(), hash);
                }

                if self.should_track_file(path) {
                    watched.insert(path.clone());
                    log::debug!("Added file to watch list with baseline: {}", path.display());
                }
            } else {
                // File doesn't exist yet - don't add to baseline so we know it's new if created
                if self.should_track_file(path) {
                    watched.insert(path.clone());
                    log::debug!("Added non-existent file to watch list: {}", path.display());
                }
            }
        }

        Ok(())
    }

    /// Process pending file system events
    pub fn process_events(&self) -> Result<()> {
        let current_mode = *self.current_mode.lock().unwrap();

        // Only process events in agent mode
        if current_mode != OperationMode::Agent {
            return Ok(());
        }

        let receiver = self.file_events.lock().unwrap();

        // Process all pending events (non-blocking)
        while let Ok(event_result) = receiver.try_recv() {
            match event_result {
                Ok(event) => {
                    self.handle_file_event(event)?;
                }
                Err(e) => {
                    log::warn!("File system event error: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Handle a single file system event
    fn handle_file_event(&self, event: Event) -> Result<()> {
        let current_mode = *self.current_mode.lock().unwrap();
        let active_session = *self.active_agent_session.lock().unwrap();

        // Only process events in agent mode with an active session
        if current_mode != OperationMode::Agent || active_session.is_none() {
            return Ok(());
        }

        let session_id = active_session.unwrap();

        for path in event.paths {
            if let Ok(relative_path) = path.strip_prefix(&self.workspace_path) {
                if self.should_track_file(&relative_path) {
                    self.process_file_change(&relative_path, session_id)?;
                }
            }
        }

        Ok(())
    }

    /// Process a change to a specific file
    fn process_file_change(&self, file_path: &Path, session_id: SessionId) -> Result<()> {
        let full_path = self.workspace_path.join(file_path);

        // Determine change type based on baseline and current state
        let baseline = self.baseline_file_states.lock().unwrap();
        let change_type = if !full_path.exists() {
            // File doesn't exist anymore - it was deleted
            ChangeType::Deleted
        } else if !baseline.contains_key(file_path) {
            // File exists but wasn't in baseline - it was created
            ChangeType::Created
        } else {
            // File exists and was in baseline - it was modified
            ChangeType::Modified
        };

        // Create changeset entry
        let entry = self.create_changeset_entry(file_path, change_type.clone(), session_id)?;

        // Store in pending changes
        let mut pending = self.pending_changes.lock().unwrap();
        pending.insert(file_path.to_path_buf(), entry);

        // Update stats
        let mut stats = self.stats.lock().unwrap();
        stats.changes_detected += 1;

        log::debug!(
            "Tracked change: {} ({:?})",
            file_path.display(),
            change_type
        );

        Ok(())
    }

    /// Create a changeset entry for a file change
    fn create_changeset_entry(
        &self,
        file_path: &Path,
        change_type: ChangeType,
        session_id: SessionId,
    ) -> Result<ChangesetEntry> {
        let full_path = self.workspace_path.join(file_path);
        let timestamp = SystemTime::now();

        let (content_hash, size_bytes, content) = if full_path.exists() {
            let metadata = fs::metadata(&full_path)?;
            let size = metadata.len();

            // Only read content for small files or specific file types
            let should_store_content = size < 10_000 || // Small files
                self.is_critical_file_type(file_path);

            let content = if should_store_content {
                Some(fs::read_to_string(&full_path).map_err(|e| {
                    CheckpointError::file_system(format!(
                        "Failed to read file {}: {}",
                        full_path.display(),
                        e
                    ))
                })?)
            } else {
                None
            };

            let hash = if let Some(ref content_str) = content {
                self.calculate_hash(content_str)
            } else {
                // For large files, use file metadata as a quick hash
                format!(
                    "meta_{}_{}_{}",
                    size,
                    metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs(),
                    file_path.to_string_lossy()
                )
            };

            (hash, size, content)
        } else {
            // File was deleted
            (String::new(), 0, None)
        };

        Ok(ChangesetEntry {
            file_path: file_path.to_path_buf(),
            change_type,
            timestamp,
            content_hash,
            size_bytes,
            agent_session_id: Some(session_id),
            content,
        })
    }

    /// Get all pending changes and clear the list
    pub fn consume_pending_changes(&self) -> Vec<ChangesetEntry> {
        let mut pending = self.pending_changes.lock().unwrap();
        let changes: Vec<ChangesetEntry> = pending.values().cloned().collect();
        pending.clear();

        // Update baseline: remove deleted files, update baseline for created/modified files
        let mut baseline = self.baseline_file_states.lock().unwrap();
        for change in &changes {
            match change.change_type {
                ChangeType::Deleted => {
                    // Remove from baseline as file no longer exists
                    baseline.remove(&change.file_path);
                }
                ChangeType::Created | ChangeType::Modified => {
                    // Update baseline with new hash
                    baseline.insert(change.file_path.clone(), change.content_hash.clone());
                }
                _ => {}
            }
        }

        log::info!("Consumed {} pending changes for checkpoint", changes.len());
        changes
    }

    /// Get pending changes without consuming them
    pub fn get_pending_changes(&self) -> Vec<ChangesetEntry> {
        let pending = self.pending_changes.lock().unwrap();
        pending.values().cloned().collect()
    }

    /// Check if there are any pending changes
    pub fn has_pending_changes(&self) -> bool {
        let pending = self.pending_changes.lock().unwrap();
        !pending.is_empty()
    }

    /// Convert changeset entries to FileChange format for compatibility
    pub fn changeset_to_file_changes(&self, entries: &[ChangesetEntry]) -> Result<Vec<FileChange>> {
        let mut file_changes = Vec::new();

        for entry in entries {
            let file_change = FileChange {
                path: entry.file_path.clone(),
                change_type: entry.change_type.clone(),
                original_content: None, // We don't track original content for memory efficiency
                new_content: entry.content.clone(),
                size_bytes: entry.size_bytes,
                content_hash: entry.content_hash.clone(),
                permissions: None, // Could be added if needed
                modified_at: chrono::DateTime::from(entry.timestamp),
                encoding: FileEncoding::Utf8,
                compressed: false,
            };

            file_changes.push(file_change);
        }

        Ok(file_changes)
    }

    /// Get current tracking statistics
    pub fn get_stats(&self) -> ChangesetStats {
        let stats = self.stats.lock().unwrap();
        let pending = self.pending_changes.lock().unwrap();
        let watched = self.watched_files.lock().unwrap();

        ChangesetStats {
            files_tracked: watched.len(),
            changes_detected: stats.changes_detected,
            memory_usage_bytes: pending.len() * std::mem::size_of::<ChangesetEntry>(),
            last_scan_duration_ms: stats.last_scan_duration_ms,
        }
    }

    /// Check if a file should be tracked based on configuration
    fn should_track_file(&self, path: &Path) -> bool {
        // Check file extension
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            if !self.config.should_track_extension(extension) {
                return false;
            }
        }

        // Check against ignore patterns
        let path_str = path.to_string_lossy();
        for pattern in &self.config.global_ignore_patterns {
            if self.matches_pattern(pattern, &path_str) {
                return false;
            }
        }

        true
    }

    /// Check if this is a critical file type that should always have content stored
    fn is_critical_file_type(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            matches!(
                extension.to_lowercase().as_str(),
                "json" | "yaml" | "yml" | "toml" | "xml" | "config" | "env"
            )
        } else {
            false
        }
    }

    /// Calculate SHA-256 hash of content
    fn calculate_hash(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Simple pattern matching
    fn matches_pattern(&self, pattern: &str, text: &str) -> bool {
        // Simplified glob matching - could be enhanced
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                return text.starts_with(parts[0]) && text.ends_with(parts[1]);
            }
        }
        pattern == text
    }
}

impl Drop for ChangesetTracker {
    fn drop(&mut self) {
        if let Some(watcher) = self.watcher.take() {
            drop(watcher);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::Duration;
    use tempfile::TempDir;

    fn create_test_tracker() -> (ChangesetTracker, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = CheckpointConfig::default();
        let tracker = ChangesetTracker::new(config, temp_dir.path().to_path_buf()).unwrap();
        (tracker, temp_dir)
    }

    #[test]
    fn test_mode_switching() {
        let (tracker, _temp_dir) = create_test_tracker();

        // Should start in chat mode
        assert_eq!(*tracker.current_mode.lock().unwrap(), OperationMode::Chat);

        // Switch to agent mode
        tracker.set_mode(OperationMode::Agent);
        assert_eq!(*tracker.current_mode.lock().unwrap(), OperationMode::Agent);
    }

    #[test]
    fn test_agent_session_tracking() {
        let (tracker, _temp_dir) = create_test_tracker();
        let session_id = Uuid::new_v4();

        // Start agent session
        tracker.start_agent_session(session_id);
        assert_eq!(
            *tracker.active_agent_session.lock().unwrap(),
            Some(session_id)
        );
        assert_eq!(*tracker.current_mode.lock().unwrap(), OperationMode::Agent);

        // Stop agent session
        tracker.stop_agent_session();
        assert_eq!(*tracker.active_agent_session.lock().unwrap(), None);
        assert_eq!(*tracker.current_mode.lock().unwrap(), OperationMode::Chat);
    }

    #[test]
    fn test_file_tracking() {
        let (tracker, temp_dir) = create_test_tracker();
        let session_id = Uuid::new_v4();

        // Create a test file
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "Hello, world!").unwrap();

        // Start agent session and track the file
        tracker.start_agent_session(session_id);
        tracker.watch_files(&[PathBuf::from("test.txt")]).unwrap();

        // Modify the file
        fs::write(&test_file, "Hello, Rust!").unwrap();

        // Process events
        std::thread::sleep(Duration::from_millis(100)); // Give file system time
        tracker.process_events().unwrap();

        // Check for pending changes
        assert!(tracker.has_pending_changes());

        let changes = tracker.get_pending_changes();
        assert!(!changes.is_empty());
    }

    #[test]
    fn test_changeset_to_file_changes_conversion() {
        let (tracker, _temp_dir) = create_test_tracker();

        let entry = ChangesetEntry {
            file_path: PathBuf::from("test.txt"),
            change_type: ChangeType::Modified,
            timestamp: SystemTime::now(),
            content_hash: "abc123".to_string(),
            size_bytes: 100,
            agent_session_id: Some(Uuid::new_v4()),
            content: Some("test content".to_string()),
        };

        let file_changes = tracker.changeset_to_file_changes(&[entry]).unwrap();
        assert_eq!(file_changes.len(), 1);
        assert_eq!(file_changes[0].path, PathBuf::from("test.txt"));
        assert_eq!(file_changes[0].change_type, ChangeType::Modified);
    }
}
