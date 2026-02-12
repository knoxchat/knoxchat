//! Storage layer for checkpoint system
//!
//! This module handles the physical storage of checkpoint data with support for
//! compression, deduplication, and efficient file management.

use crate::config::CheckpointConfig;
use crate::error::{CheckpointError, Result};
use crate::types::*;

// use base64::{Engine as _, engine::general_purpose}; // Unused for now
use chrono::{DateTime, Utc};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use parking_lot::RwLock;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Storage manager for checkpoint data
#[derive(Clone)]
pub struct CheckpointStorage {
    config: CheckpointConfig,
    content_cache: Arc<RwLock<HashMap<String, CachedContent>>>,
    dedup_index: Arc<RwLock<HashMap<String, String>>>, // content_hash -> file_path
}

/// Cached content with metadata
#[derive(Debug, Clone)]
struct CachedContent {
    content: Vec<u8>,
    compressed: bool,
    last_accessed: DateTime<Utc>,
    access_count: u64,
}

impl CheckpointStorage {
    /// Create a new storage manager
    pub fn new(config: CheckpointConfig) -> Result<Self> {
        let storage = Self {
            config,
            content_cache: Arc::new(RwLock::new(HashMap::new())),
            dedup_index: Arc::new(RwLock::new(HashMap::new())),
        };

        storage.initialize_storage_directories()?;
        storage.load_deduplication_index()?;

        Ok(storage)
    }

    /// Initialize storage directories
    fn initialize_storage_directories(&self) -> Result<()> {
        fs::create_dir_all(&self.config.data_path())?;
        fs::create_dir_all(&self.config.backup_path())?;
        fs::create_dir_all(self.config.data_path().join("content"))?;
        fs::create_dir_all(self.config.data_path().join("dedup"))?;
        Ok(())
    }

    /// Load deduplication index from disk
    fn load_deduplication_index(&self) -> Result<()> {
        let index_path = self.config.data_path().join("dedup_index.json");

        if index_path.exists() {
            let content = fs::read_to_string(&index_path)?;
            let index: HashMap<String, String> = serde_json::from_str(&content).map_err(|e| {
                CheckpointError::generic(format!("Failed to parse dedup index: {}", e))
            })?;

            *self.dedup_index.write() = index;
        }

        Ok(())
    }

    /// Save deduplication index to disk
    fn save_deduplication_index(&self) -> Result<()> {
        let index_path = self.config.data_path().join("dedup_index.json");
        let index = self.dedup_index.read();
        let content = serde_json::to_string_pretty(&*index)?;
        fs::write(&index_path, content)?;
        Ok(())
    }

    /// Store checkpoint data
    pub fn store_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()> {
        let checkpoint_dir = self
            .config
            .data_path()
            .join("checkpoints")
            .join(checkpoint.id.to_string());
        fs::create_dir_all(&checkpoint_dir)?;

        // Store checkpoint metadata
        let metadata_path = checkpoint_dir.join("metadata.json");
        let metadata_json = serde_json::to_string_pretty(checkpoint)?;
        fs::write(&metadata_path, metadata_json)?;

        // Store file changes with deduplication
        for (i, file_change) in checkpoint.file_changes.iter().enumerate() {
            self.store_file_change(&checkpoint_dir, i, file_change)?;
        }

        Ok(())
    }

    /// Store a single file change with deduplication
    fn store_file_change(
        &self,
        checkpoint_dir: &Path,
        index: usize,
        file_change: &FileChange,
    ) -> Result<()> {
        let change_dir = checkpoint_dir.join(format!("file_{:04}", index));
        fs::create_dir_all(&change_dir)?;

        // Store file change metadata
        let metadata = serde_json::to_string_pretty(file_change)?;
        fs::write(change_dir.join("metadata.json"), metadata)?;

        // Store original content if present
        if let Some(ref content) = file_change.original_content {
            let content_hash = self.calculate_content_hash(content);
            self.store_content_with_dedup(&content_hash, content.as_bytes(), CompressionType::Lz4)?;
            fs::write(change_dir.join("original_hash"), &content_hash)?;
        }

        // Store new content if present
        if let Some(ref content) = file_change.new_content {
            let content_hash = self.calculate_content_hash(content);
            self.store_content_with_dedup(&content_hash, content.as_bytes(), CompressionType::Lz4)?;
            fs::write(change_dir.join("new_hash"), &content_hash)?;
        }

        Ok(())
    }

    /// Store content with deduplication
    fn store_content_with_dedup(
        &self,
        content_hash: &str,
        content: &[u8],
        compression: CompressionType,
    ) -> Result<()> {
        // Check if content already exists
        {
            let dedup_index = self.dedup_index.read();
            if dedup_index.contains_key(content_hash) {
                // Content already stored, just return
                return Ok(());
            }
        }

        // Compress content
        let compressed_content = self.compress_content(content, compression)?;

        // Store compressed content
        let content_path = self.get_content_path(content_hash);
        fs::create_dir_all(content_path.parent().unwrap())?;
        fs::write(&content_path, &compressed_content)?;

        // Update deduplication index
        {
            let mut dedup_index = self.dedup_index.write();
            dedup_index.insert(
                content_hash.to_string(),
                content_path.to_string_lossy().to_string(),
            );
        }

        // Cache the content
        {
            let mut cache = self.content_cache.write();
            cache.insert(
                content_hash.to_string(),
                CachedContent {
                    content: compressed_content,
                    compressed: true,
                    last_accessed: Utc::now(),
                    access_count: 1,
                },
            );
        }

        Ok(())
    }

    /// Load checkpoint data
    pub fn load_checkpoint(&self, checkpoint_id: &CheckpointId) -> Result<Option<Checkpoint>> {
        let checkpoint_dir = self
            .config
            .data_path()
            .join("checkpoints")
            .join(checkpoint_id.to_string());

        if !checkpoint_dir.exists() {
            return Ok(None);
        }

        // Load checkpoint metadata
        let metadata_path = checkpoint_dir.join("metadata.json");
        let metadata_content = fs::read_to_string(&metadata_path)?;
        let mut checkpoint: Checkpoint = serde_json::from_str(&metadata_content)?;

        // Load file changes
        let mut file_changes = Vec::new();
        let mut index = 0;

        loop {
            let change_dir = checkpoint_dir.join(format!("file_{:04}", index));
            if !change_dir.exists() {
                break;
            }

            let file_change = self.load_file_change(&change_dir)?;
            file_changes.push(file_change);
            index += 1;
        }

        checkpoint.file_changes = file_changes;
        Ok(Some(checkpoint))
    }

    /// Load a single file change
    fn load_file_change(&self, change_dir: &Path) -> Result<FileChange> {
        // Load metadata
        let metadata_content = fs::read_to_string(change_dir.join("metadata.json"))?;
        let mut file_change: FileChange = serde_json::from_str(&metadata_content)?;

        // Load original content if hash exists
        let original_hash_path = change_dir.join("original_hash");
        if original_hash_path.exists() {
            let content_hash = fs::read_to_string(&original_hash_path)?;
            if let Some(content) = self.load_content(&content_hash)? {
                file_change.original_content = Some(String::from_utf8(content).map_err(|e| {
                    CheckpointError::generic(format!("Invalid UTF-8 content: {}", e))
                })?);
            }
        }

        // Load new content if hash exists
        let new_hash_path = change_dir.join("new_hash");
        if new_hash_path.exists() {
            let content_hash = fs::read_to_string(&new_hash_path)?;
            if let Some(content) = self.load_content(&content_hash)? {
                file_change.new_content = Some(String::from_utf8(content).map_err(|e| {
                    CheckpointError::generic(format!("Invalid UTF-8 content: {}", e))
                })?);
            }
        }

        Ok(file_change)
    }

    /// Load content by hash with caching
    pub fn load_content(&self, content_hash: &str) -> Result<Option<Vec<u8>>> {
        // Check cache first
        {
            let mut cache = self.content_cache.write();
            if let Some(cached) = cache.get_mut(content_hash) {
                cached.last_accessed = Utc::now();
                cached.access_count += 1;

                return if cached.compressed {
                    Ok(Some(self.decompress_content(
                        &cached.content,
                        CompressionType::Lz4,
                    )?))
                } else {
                    Ok(Some(cached.content.clone()))
                };
            }
        }

        // Load from disk
        let dedup_index = self.dedup_index.read();
        if let Some(file_path) = dedup_index.get(content_hash) {
            let path = PathBuf::from(file_path);
            if path.exists() {
                let compressed_content = fs::read(&path)?;
                let content = self.decompress_content(&compressed_content, CompressionType::Lz4)?;

                // Update cache
                {
                    let mut cache = self.content_cache.write();
                    cache.insert(
                        content_hash.to_string(),
                        CachedContent {
                            content: compressed_content,
                            compressed: true,
                            last_accessed: Utc::now(),
                            access_count: 1,
                        },
                    );
                }

                Ok(Some(content))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Delete checkpoint data
    pub fn delete_checkpoint(&self, checkpoint_id: &CheckpointId) -> Result<()> {
        let checkpoint_dir = self
            .config
            .data_path()
            .join("checkpoints")
            .join(checkpoint_id.to_string());

        if checkpoint_dir.exists() {
            // TODO: Implement reference counting for deduplication
            // For now, we just remove the checkpoint directory
            fs::remove_dir_all(&checkpoint_dir)?;
        }

        Ok(())
    }

    /// Compress content using the specified algorithm
    fn compress_content(&self, content: &[u8], compression: CompressionType) -> Result<Vec<u8>> {
        match compression {
            CompressionType::None => Ok(content.to_vec()),
            CompressionType::Lz4 => Ok(compress_prepend_size(content)),
            CompressionType::Gzip => {
                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(content)?;
                Ok(encoder.finish()?)
            }
            CompressionType::Zstd => {
                // For now, fall back to LZ4 since zstd requires additional dependencies
                Ok(compress_prepend_size(content))
            }
        }
    }

    /// Decompress content using the specified algorithm
    fn decompress_content(
        &self,
        compressed: &[u8],
        compression: CompressionType,
    ) -> Result<Vec<u8>> {
        match compression {
            CompressionType::None => Ok(compressed.to_vec()),
            CompressionType::Lz4 => decompress_size_prepended(compressed).map_err(|e| {
                CheckpointError::compression(format!("LZ4 decompression failed: {}", e))
            }),
            CompressionType::Gzip => {
                let mut decoder = GzDecoder::new(compressed);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed)?;
                Ok(decompressed)
            }
            CompressionType::Zstd => {
                // For now, fall back to LZ4
                decompress_size_prepended(compressed).map_err(|e| {
                    CheckpointError::compression(format!("LZ4 decompression failed: {}", e))
                })
            }
        }
    }

    /// Calculate content hash
    fn calculate_content_hash(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Get storage path for content hash
    fn get_content_path(&self, content_hash: &str) -> PathBuf {
        // Use first two characters of hash for directory structure
        let dir = &content_hash[0..2];
        let filename = &content_hash[2..];
        self.config
            .data_path()
            .join("content")
            .join(dir)
            .join(filename)
    }

    /// Create backup of checkpoint data
    pub fn create_backup(
        &self,
        checkpoint_ids: &[CheckpointId],
        backup_path: &Path,
    ) -> Result<BackupInfo> {
        let backup_id = uuid::Uuid::new_v4();
        let backup_file = backup_path.join(format!("{}.backup", backup_id));

        // Create tar-like backup format
        let mut backup_data = Vec::new();

        for checkpoint_id in checkpoint_ids {
            if let Some(checkpoint) = self.load_checkpoint(checkpoint_id)? {
                let checkpoint_json = serde_json::to_string(&checkpoint)?;
                backup_data.extend_from_slice(checkpoint_json.as_bytes());
                backup_data.push(b'\n');
            }
        }

        // Compress backup
        let compressed_backup = self.compress_content(&backup_data, CompressionType::Gzip)?;
        fs::write(&backup_file, &compressed_backup)?;

        let backup_info = BackupInfo {
            id: backup_id,
            path: backup_file,
            created_at: Utc::now(),
            size_bytes: compressed_backup.len() as u64,
            checkpoint_ids: checkpoint_ids.to_vec(),
            format_version: 1,
            compression_type: CompressionType::Gzip,
        };

        Ok(backup_info)
    }

    /// Restore from backup
    pub fn restore_backup(&self, backup_path: &Path) -> Result<Vec<CheckpointId>> {
        let compressed_backup = fs::read(backup_path)?;
        let backup_data = self.decompress_content(&compressed_backup, CompressionType::Gzip)?;
        let backup_content = String::from_utf8(backup_data)
            .map_err(|e| CheckpointError::generic(format!("Invalid backup format: {}", e)))?;

        let mut restored_checkpoints = Vec::new();

        for line in backup_content.lines() {
            if !line.is_empty() {
                let checkpoint: Checkpoint = serde_json::from_str(line)?;
                self.store_checkpoint(&checkpoint)?;
                restored_checkpoints.push(checkpoint.id);
            }
        }

        Ok(restored_checkpoints)
    }

    /// Clean up unused content (garbage collection)
    pub fn cleanup_unused_content(&self) -> Result<u64> {
        // This would implement reference counting and cleanup unused content
        // For now, just clean up cache
        let mut freed_bytes = 0u64;

        {
            let mut cache = self.content_cache.write();
            let cutoff_time = Utc::now() - chrono::Duration::hours(24);

            cache.retain(|_, cached| {
                if cached.last_accessed < cutoff_time && cached.access_count < 2 {
                    freed_bytes += cached.content.len() as u64;
                    false
                } else {
                    true
                }
            });
        }

        Ok(freed_bytes)
    }

    /// Get storage statistics
    pub fn get_storage_stats(&self) -> Result<StorageStats> {
        let data_dir = &self.config.data_path();
        let total_size = self.calculate_directory_size(data_dir)?;

        let cache_size = {
            let cache = self.content_cache.read();
            cache.values().map(|c| c.content.len() as u64).sum()
        };

        let dedup_entries = {
            let index = self.dedup_index.read();
            index.len()
        };

        Ok(StorageStats {
            total_size_bytes: total_size,
            cache_size_bytes: cache_size,
            dedup_entries,
            compression_ratio: 0.7, // Placeholder
        })
    }

    /// Calculate total size of a directory
    fn calculate_directory_size(&self, dir: &Path) -> Result<u64> {
        let mut total_size = 0u64;

        if dir.exists() {
            for entry in walkdir::WalkDir::new(dir) {
                let entry = entry
                    .map_err(|e| CheckpointError::file_system(format!("Walk error: {}", e)))?;
                if entry.file_type().is_file() {
                    total_size += entry.metadata()?.len();
                }
            }
        }

        Ok(total_size)
    }

    /// Flush deduplication index to disk
    pub fn flush(&self) -> Result<()> {
        self.save_deduplication_index()
    }
}

/// Storage statistics
#[derive(Debug, Clone)]
pub struct StorageStats {
    pub total_size_bytes: u64,
    pub cache_size_bytes: u64,
    pub dedup_entries: usize,
    pub compression_ratio: f64,
}

impl Drop for CheckpointStorage {
    fn drop(&mut self) {
        // Save deduplication index on drop
        let _ = self.save_deduplication_index();
    }
}
