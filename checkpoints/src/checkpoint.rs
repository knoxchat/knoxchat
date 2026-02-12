//! Individual checkpoint operations and utilities

use crate::types::*;
use crate::error::{CheckpointError, Result};
use chrono::Utc;
use serde_json;
use std::path::Path;
use uuid::Uuid;

impl Checkpoint {
    /// Create a new checkpoint with the given parameters
    pub fn new(
        session_id: SessionId,
        description: String,
        file_changes: Vec<FileChange>,
    ) -> Self {
        let files_affected = file_changes.len();
        let size_bytes = file_changes
            .iter()
            .map(|change| {
                let original_size = change.original_size.unwrap_or(0);
                let modified_size = change.modified_size.unwrap_or(0);
                original_size + modified_size
            })
            .sum();
        
        Self {
            id: Uuid::new_v4(),
            session_id,
            description,
            created_at: Utc::now(),
            file_changes,
            files_affected,
            size_bytes,
            tags: Vec::new(),
            metadata: std::collections::HashMap::new(),
        }
    }
    
    /// Add a tag to the checkpoint
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
    
    /// Remove a tag from the checkpoint
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
    }
    
    /// Check if the checkpoint has a specific tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(&tag.to_string())
    }
    
    /// Add metadata to the checkpoint
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
    
    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
    
    /// Get all files that were created in this checkpoint
    pub fn created_files(&self) -> Vec<&FileChange> {
        self.file_changes
            .iter()
            .filter(|change| change.change_type == ChangeType::Created)
            .collect()
    }
    
    /// Get all files that were modified in this checkpoint
    pub fn modified_files(&self) -> Vec<&FileChange> {
        self.file_changes
            .iter()
            .filter(|change| change.change_type == ChangeType::Modified)
            .collect()
    }
    
    /// Get all files that were deleted in this checkpoint
    pub fn deleted_files(&self) -> Vec<&FileChange> {
        self.file_changes
            .iter()
            .filter(|change| change.change_type == ChangeType::Deleted)
            .collect()
    }
    
    /// Get all files that were moved in this checkpoint
    pub fn moved_files(&self) -> Vec<&FileChange> {
        self.file_changes
            .iter()
            .filter(|change| change.change_type == ChangeType::Moved)
            .collect()
    }
    
    /// Get the total size change in bytes
    pub fn total_size_delta(&self) -> i64 {
        self.file_changes
            .iter()
            .map(|change| change.size_delta())
            .sum()
    }
    
    /// Check if this checkpoint affects a specific file
    pub fn affects_file(&self, file_path: &Path) -> bool {
        self.file_changes
            .iter()
            .any(|change| change.file_path == file_path)
    }
    
    /// Get the change for a specific file, if any
    pub fn get_file_change(&self, file_path: &Path) -> Option<&FileChange> {
        self.file_changes
            .iter()
            .find(|change| change.file_path == file_path)
    }
    
    /// Export checkpoint to JSON
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(CheckpointError::from)
    }
    
    /// Import checkpoint from JSON
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(CheckpointError::from)
    }
    
    /// Validate the checkpoint data
    pub fn validate(&self) -> Result<()> {
        // Check that files_affected matches actual file changes
        if self.files_affected != self.file_changes.len() {
            return Err(CheckpointError::validation(
                "files_affected count doesn't match actual file changes"
            ));
        }
        
        // Check that all file changes have valid paths
        for change in &self.file_changes {
            if change.file_path.as_os_str().is_empty() {
                return Err(CheckpointError::validation(
                    "file change has empty path"
                ));
            }
            
            // Check that content and hashes are consistent
            match change.change_type {
                ChangeType::Created => {
                    if change.original_content.is_some() || change.original_hash.is_some() {
                        return Err(CheckpointError::validation(
                            "created file should not have original content or hash"
                        ));
                    }
                    if change.modified_content.is_none() && change.modified_hash.is_none() {
                        return Err(CheckpointError::validation(
                            "created file should have modified content or hash"
                        ));
                    }
                }
                ChangeType::Deleted => {
                    if change.modified_content.is_some() || change.modified_hash.is_some() {
                        return Err(CheckpointError::validation(
                            "deleted file should not have modified content or hash"
                        ));
                    }
                    if change.original_content.is_none() && change.original_hash.is_none() {
                        return Err(CheckpointError::validation(
                            "deleted file should have original content or hash"
                        ));
                    }
                }
                ChangeType::Modified => {
                    if (change.original_content.is_none() && change.original_hash.is_none()) ||
                       (change.modified_content.is_none() && change.modified_hash.is_none()) {
                        return Err(CheckpointError::validation(
                            "modified file should have both original and modified content or hashes"
                        ));
                    }
                }
                ChangeType::Moved => {
                    // For moved files, we need at least the new location
                    if change.modified_content.is_none() && change.modified_hash.is_none() {
                        return Err(CheckpointError::validation(
                            "moved file should have modified content or hash"
                        ));
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Get a human-readable age string
    pub fn age_string(&self) -> String {
        let now = Utc::now();
        let duration = now.signed_duration_since(self.created_at);
        
        if duration.num_days() > 0 {
            format!("{} days ago", duration.num_days())
        } else if duration.num_hours() > 0 {
            format!("{} hours ago", duration.num_hours())
        } else if duration.num_minutes() > 0 {
            format!("{} minutes ago", duration.num_minutes())
        } else {
            "Just now".to_string()
        }
    }
    
    /// Get file extensions affected by this checkpoint
    pub fn affected_extensions(&self) -> Vec<String> {
        let mut extensions = std::collections::HashSet::new();
        
        for change in &self.file_changes {
            if let Some(extension) = change.file_path.extension() {
                if let Some(ext_str) = extension.to_str() {
                    extensions.insert(ext_str.to_lowercase());
                }
            }
        }
        
        let mut result: Vec<String> = extensions.into_iter().collect();
        result.sort();
        result
    }
    
    /// Check if this is a large checkpoint (affects many files or large size)
    pub fn is_large(&self) -> bool {
        self.files_affected > 50 || self.size_bytes > 10_000_000 // 10MB
    }
    
    /// Get complexity score (higher = more complex changes)
    pub fn complexity_score(&self) -> f64 {
        let mut score = 0.0;
        
        // Base score from number of files
        score += self.files_affected as f64 * 0.1;
        
        // Add score based on change types
        for change in &self.file_changes {
            match change.change_type {
                ChangeType::Created => score += 0.5,
                ChangeType::Modified => score += 1.0,
                ChangeType::Deleted => score += 0.8,
                ChangeType::Moved => score += 1.2,
            }
        }
        
        // Add score based on size
        score += (self.size_bytes as f64 / 1000.0).min(10.0);
        
        // Add score based on number of different extensions
        score += self.affected_extensions().len() as f64 * 0.2;
        
        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    fn create_test_file_change(change_type: ChangeType, path: &str) -> FileChange {
        FileChange {
            file_path: PathBuf::from(path),
            change_type,
            original_content: match change_type {
                ChangeType::Created => None,
                _ => Some("original content".to_string()),
            },
            modified_content: match change_type {
                ChangeType::Deleted => None,
                _ => Some("modified content".to_string()),
            },
            original_hash: match change_type {
                ChangeType::Created => None,
                _ => Some("original_hash".to_string()),
            },
            modified_hash: match change_type {
                ChangeType::Deleted => None,
                _ => Some("modified_hash".to_string()),
            },
            original_size: match change_type {
                ChangeType::Created => None,
                _ => Some(100),
            },
            modified_size: match change_type {
                ChangeType::Deleted => None,
                _ => Some(150),
            },
            permissions: Some(0o644),
        }
    }
    
    #[test]
    fn test_checkpoint_creation() {
        let session_id = Uuid::new_v4();
        let file_changes = vec![
            create_test_file_change(ChangeType::Created, "new_file.txt"),
            create_test_file_change(ChangeType::Modified, "existing_file.txt"),
        ];
        
        let checkpoint = Checkpoint::new(
            session_id,
            "Test checkpoint".to_string(),
            file_changes,
        );
        
        assert_eq!(checkpoint.session_id, session_id);
        assert_eq!(checkpoint.description, "Test checkpoint");
        assert_eq!(checkpoint.files_affected, 2);
        assert!(checkpoint.size_bytes > 0);
    }
    
    #[test]
    fn test_checkpoint_tags() {
        let session_id = Uuid::new_v4();
        let mut checkpoint = Checkpoint::new(
            session_id,
            "Test".to_string(),
            vec![],
        );
        
        checkpoint.add_tag("important".to_string());
        checkpoint.add_tag("feature".to_string());
        checkpoint.add_tag("important".to_string()); // Duplicate
        
        assert_eq!(checkpoint.tags.len(), 2);
        assert!(checkpoint.has_tag("important"));
        assert!(checkpoint.has_tag("feature"));
        assert!(!checkpoint.has_tag("bug"));
        
        checkpoint.remove_tag("important");
        assert!(!checkpoint.has_tag("important"));
        assert_eq!(checkpoint.tags.len(), 1);
    }
    
    #[test]
    fn test_checkpoint_validation() {
        let session_id = Uuid::new_v4();
        
        // Valid checkpoint
        let valid_checkpoint = Checkpoint::new(
            session_id,
            "Valid".to_string(),
            vec![create_test_file_change(ChangeType::Modified, "test.txt")],
        );
        assert!(valid_checkpoint.validate().is_ok());
        
        // Invalid checkpoint - wrong files_affected count
        let mut invalid_checkpoint = valid_checkpoint.clone();
        invalid_checkpoint.files_affected = 999;
        assert!(invalid_checkpoint.validate().is_err());
    }
    
    #[test]
    fn test_checkpoint_filtering() {
        let session_id = Uuid::new_v4();
        let file_changes = vec![
            create_test_file_change(ChangeType::Created, "new1.txt"),
            create_test_file_change(ChangeType::Created, "new2.txt"),
            create_test_file_change(ChangeType::Modified, "mod1.txt"),
            create_test_file_change(ChangeType::Deleted, "del1.txt"),
        ];
        
        let checkpoint = Checkpoint::new(session_id, "Test".to_string(), file_changes);
        
        assert_eq!(checkpoint.created_files().len(), 2);
        assert_eq!(checkpoint.modified_files().len(), 1);
        assert_eq!(checkpoint.deleted_files().len(), 1);
        assert_eq!(checkpoint.moved_files().len(), 0);
    }
    
    #[test]
    fn test_checkpoint_serialization() {
        let session_id = Uuid::new_v4();
        let checkpoint = Checkpoint::new(
            session_id,
            "Test".to_string(),
            vec![create_test_file_change(ChangeType::Modified, "test.txt")],
        );
        
        let json = checkpoint.to_json().unwrap();
        let restored = Checkpoint::from_json(&json).unwrap();
        
        assert_eq!(checkpoint.id, restored.id);
        assert_eq!(checkpoint.session_id, restored.session_id);
        assert_eq!(checkpoint.description, restored.description);
        assert_eq!(checkpoint.files_affected, restored.files_affected);
    }
    
    #[test]
    fn test_complexity_score() {
        let session_id = Uuid::new_v4();
        
        // Simple checkpoint
        let simple = Checkpoint::new(
            session_id,
            "Simple".to_string(),
            vec![create_test_file_change(ChangeType::Created, "test.txt")],
        );
        
        // Complex checkpoint
        let complex = Checkpoint::new(
            session_id,
            "Complex".to_string(),
            vec![
                create_test_file_change(ChangeType::Created, "test1.txt"),
                create_test_file_change(ChangeType::Modified, "test2.js"),
                create_test_file_change(ChangeType::Deleted, "test3.py"),
                create_test_file_change(ChangeType::Moved, "test4.rs"),
            ],
        );
        
        assert!(complex.complexity_score() > simple.complexity_score());
    }
}
