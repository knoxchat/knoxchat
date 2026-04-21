//! Storage layer for checkpoint system
//!
//! This module handles the physical storage of checkpoint data with support for
//! compression, deduplication, and efficient file management.

use crate::config::CheckpointConfig;
use crate::error::{CheckpointError, Result};
use crate::types::*;

// use base64::{Engine as _, engine::general_purpose}; // Unused for now
use chrono::Utc;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use lru::LruCache;
use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use parking_lot::RwLock;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Maximum number of entries in the content cache
const CONTENT_CACHE_CAP: usize = 512;

/// Storage manager for checkpoint data
#[derive(Clone)]
pub struct CheckpointStorage {
    config: CheckpointConfig,
    content_cache: Arc<RwLock<LruCache<String, CachedContent>>>,
    dedup_index: Arc<RwLock<HashMap<String, String>>>, // content_hash -> file_path
    /// Reference counts for content-addressable blobs: content_hash -> refcount
    content_refcounts: Arc<RwLock<HashMap<String, u64>>>,
    /// Tracks cumulative compression metrics for stats
    compression_metrics: Arc<RwLock<CompressionMetrics>>,
}

/// Cached content with metadata
#[derive(Debug, Clone)]
struct CachedContent {
    content: Vec<u8>,
    compressed: bool,
}

/// Tracks compression and deduplication metrics
#[derive(Debug, Clone, Default)]
struct CompressionMetrics {
    /// Total bytes before compression across all stored content
    total_original_bytes: u64,
    /// Total bytes after compression across all stored content
    total_compressed_bytes: u64,
    /// Total bytes saved by deduplication (content that was already stored)
    total_dedup_savings_bytes: u64,
}

impl CheckpointStorage {
    /// Create a new storage manager
    pub fn new(config: CheckpointConfig) -> Result<Self> {
        let storage = Self {
            config,
            content_cache: Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(CONTENT_CACHE_CAP).unwrap(),
            ))),
            dedup_index: Arc::new(RwLock::new(HashMap::new())),
            content_refcounts: Arc::new(RwLock::new(HashMap::new())),
            compression_metrics: Arc::new(RwLock::new(CompressionMetrics::default())),
        };

        storage.initialize_storage_directories()?;
        storage.load_deduplication_index()?;
        storage.load_refcount_index()?;

        Ok(storage)
    }

    /// Initialize storage directories
    fn initialize_storage_directories(&self) -> Result<()> {
        fs::create_dir_all(self.config.data_path())?;
        fs::create_dir_all(self.config.backup_path())?;
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

    /// Load reference count index from disk
    fn load_refcount_index(&self) -> Result<()> {
        let index_path = self.config.data_path().join("refcount_index.json");

        if index_path.exists() {
            let content = fs::read_to_string(&index_path)?;
            let index: HashMap<String, u64> = serde_json::from_str(&content).map_err(|e| {
                CheckpointError::generic(format!("Failed to parse refcount index: {}", e))
            })?;

            *self.content_refcounts.write() = index;
        }

        Ok(())
    }

    /// Save reference count index to disk
    fn save_refcount_index(&self) -> Result<()> {
        let index_path = self.config.data_path().join("refcount_index.json");
        let index = self.content_refcounts.read();
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
        let original_size = content.len() as u64;

        // Check if content already exists (dedup hit)
        {
            let dedup_index = self.dedup_index.read();
            if dedup_index.contains_key(content_hash) {
                // Content already stored — increment refcount and record dedup savings
                let mut refcounts = self.content_refcounts.write();
                let count = refcounts.entry(content_hash.to_string()).or_insert(1);
                *count += 1;

                let mut metrics = self.compression_metrics.write();
                metrics.total_dedup_savings_bytes += original_size;

                return Ok(());
            }
        }

        // Compress content
        let compressed_content = self.compress_content(content, compression)?;
        let compressed_size = compressed_content.len() as u64;

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

        // Initialize refcount to 1
        {
            let mut refcounts = self.content_refcounts.write();
            refcounts.insert(content_hash.to_string(), 1);
        }

        // Update compression metrics
        {
            let mut metrics = self.compression_metrics.write();
            metrics.total_original_bytes += original_size;
            metrics.total_compressed_bytes += compressed_size;
        }

        // Cache the content
        {
            let mut cache = self.content_cache.write();
            cache.put(
                content_hash.to_string(),
                CachedContent {
                    content: compressed_content,
                    compressed: true,
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
        // Check cache first (LruCache::get automatically promotes to most-recently-used)
        {
            let mut cache = self.content_cache.write();
            if let Some(cached) = cache.get(content_hash) {
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
                    cache.put(
                        content_hash.to_string(),
                        CachedContent {
                            content: compressed_content,
                            compressed: true,
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
            // Collect content hashes referenced by this checkpoint before removing
            let content_hashes = self.collect_checkpoint_content_hashes(&checkpoint_dir);

            // Decrement refcounts and remove unreferenced content blobs
            for hash in &content_hashes {
                self.release_content_ref(hash);
            }

            // Remove the checkpoint directory (metadata + hash pointer files)
            fs::remove_dir_all(&checkpoint_dir)?;
        }

        Ok(())
    }

    /// Collect all content hashes referenced by a checkpoint directory
    fn collect_checkpoint_content_hashes(&self, checkpoint_dir: &Path) -> Vec<String> {
        let mut hashes = Vec::new();
        let mut index = 0;

        loop {
            let change_dir = checkpoint_dir.join(format!("file_{:04}", index));
            if !change_dir.exists() {
                break;
            }

            for hash_file in &["original_hash", "new_hash"] {
                let hash_path = change_dir.join(hash_file);
                if let Ok(hash) = fs::read_to_string(&hash_path) {
                    let hash = hash.trim().to_string();
                    if !hash.is_empty() {
                        hashes.push(hash);
                    }
                }
            }
            index += 1;
        }

        hashes
    }

    /// Decrement refcount for a content hash; remove blob if it reaches zero
    fn release_content_ref(&self, content_hash: &str) {
        let should_delete = {
            let mut refcounts = self.content_refcounts.write();
            if let Some(count) = refcounts.get_mut(content_hash) {
                *count = count.saturating_sub(1);
                if *count == 0 {
                    refcounts.remove(content_hash);
                    true
                } else {
                    false
                }
            } else {
                // No refcount tracked — legacy data; remove the blob
                true
            }
        };

        if should_delete {
            // Remove from content store
            let content_path = self.get_content_path(content_hash);
            if content_path.exists() {
                if let Err(e) = fs::remove_file(&content_path) {
                    log::warn!("Failed to remove unreferenced content blob {}: {}", content_hash, e);
                } else {
                    log::debug!("Removed unreferenced content blob: {}", content_hash);
                }
            }

            // Remove from dedup index
            {
                let mut dedup_index = self.dedup_index.write();
                dedup_index.remove(content_hash);
            }

            // Remove from cache
            {
                let mut cache = self.content_cache.write();
                cache.pop(content_hash);
            }
        }
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
        // With LruCache, eviction is automatic. We can resize to trigger eviction
        // of least-recently-used entries, or just report current cache size.
        let _freed_bytes = {
            let cache = self.content_cache.read();
            // LruCache handles eviction automatically — report current usage
            cache.iter().map(|(_, c)| c.content.len() as u64).sum::<u64>()
        };

        // Resize the cache to force eviction if it's over capacity
        // (LruCache already handles this on put, so this is a no-op in practice)
        Ok(0)
    }

    /// Get storage statistics
    pub fn get_storage_stats(&self) -> Result<StorageStats> {
        let data_dir = &self.config.data_path();
        let total_size = self.calculate_directory_size(data_dir)?;

        let cache_size = {
            let cache = self.content_cache.read();
            cache.iter().map(|(_, c)| c.content.len() as u64).sum()
        };

        let dedup_entries = {
            let index = self.dedup_index.read();
            index.len()
        };

        let metrics = self.compression_metrics.read();
        let compression_ratio = if metrics.total_original_bytes > 0 {
            metrics.total_compressed_bytes as f64 / metrics.total_original_bytes as f64
        } else {
            1.0 // no data yet
        };

        Ok(StorageStats {
            total_size_bytes: total_size,
            cache_size_bytes: cache_size,
            dedup_entries,
            compression_ratio,
            deduplication_savings_bytes: metrics.total_dedup_savings_bytes,
        })
    }

    /// Run garbage collection: remove orphaned content blobs with zero references
    pub fn run_gc(&self) -> Result<u64> {
        let mut freed_bytes = 0u64;
        let content_dir = self.config.data_path().join("content");

        if !content_dir.exists() {
            return Ok(0);
        }

        let refcounts = self.content_refcounts.read();
        let dedup_index = self.dedup_index.read();

        // Walk content directory and find blobs not in the refcount index
        for entry in walkdir::WalkDir::new(&content_dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                // Reconstruct hash from path: content/<first2chars>/<rest>
                if let (Some(parent), Some(filename)) = (
                    entry.path().parent().and_then(|p| p.file_name()),
                    entry.path().file_name(),
                ) {
                    let hash = format!(
                        "{}{}",
                        parent.to_string_lossy(),
                        filename.to_string_lossy()
                    );

                    // If hash has zero or no refcount and is not in dedup_index, remove it
                    let is_orphaned = !refcounts.contains_key(&hash) && !dedup_index.contains_key(&hash);
                    if is_orphaned {
                        if let Ok(meta) = entry.metadata() {
                            freed_bytes += meta.len();
                        }
                        if let Err(e) = fs::remove_file(entry.path()) {
                            log::warn!("GC: failed to remove orphaned blob: {}", e);
                        } else {
                            log::debug!("GC: removed orphaned blob {}", hash);
                        }
                    }
                }
            }
        }

        drop(refcounts);
        drop(dedup_index);

        log::info!("GC complete: freed {} bytes", freed_bytes);
        Ok(freed_bytes)
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

    /// Flush deduplication index and refcount index to disk
    pub fn flush(&self) -> Result<()> {
        self.save_deduplication_index()?;
        self.save_refcount_index()
    }
}

/// Storage statistics
#[derive(Debug, Clone)]
pub struct StorageStats {
    pub total_size_bytes: u64,
    pub cache_size_bytes: u64,
    pub dedup_entries: usize,
    pub compression_ratio: f64,
    pub deduplication_savings_bytes: u64,
}

impl Drop for CheckpointStorage {
    fn drop(&mut self) {
        // Save deduplication index and refcount index on drop
        let _ = self.save_deduplication_index();
        let _ = self.save_refcount_index();
    }
}
