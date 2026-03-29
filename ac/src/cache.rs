//! LRU Cache implementation for autocomplete using rusqlite
//!
//! This module provides a thread-safe LRU cache that stores autocomplete
//! completions in a SQLite database for persistence across sessions.

use crate::error::Result;
use crate::types::CacheEntry;

use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::Arc;

/// LRU Cache for autocomplete completions
pub struct AutocompleteLruCache {
    connection: Arc<Mutex<Connection>>,
    capacity: usize,
}

impl AutocompleteLruCache {
    /// Create a new autocomplete cache
    pub fn new<P: AsRef<Path>>(db_path: P, capacity: usize) -> Result<Self> {
        let connection = Connection::open(&db_path)?;

        // Set pragmas for performance
        connection.execute_batch(
            r#"
            PRAGMA busy_timeout = 3000;
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA cache_size = 10000;
            PRAGMA temp_store = MEMORY;
            "#,
        )?;

        let cache = Self {
            connection: Arc::new(Mutex::new(connection)),
            capacity,
        };

        cache.initialize_schema()?;
        Ok(cache)
    }

    /// Initialize database schema
    fn initialize_schema(&self) -> Result<()> {
        let conn = self.connection.lock();
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS cache (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )
            "#,
            [],
        )?;

        // Create index on timestamp for efficient LRU eviction
        conn.execute(
            r#"
            CREATE INDEX IF NOT EXISTS idx_cache_timestamp ON cache(timestamp)
            "#,
            [],
        )?;

        Ok(())
    }

    /// Get a completion from the cache
    ///
    /// If the query is "co" and we have "c" -> "ontinue" in the cache,
    /// we should return "ntinue" as the completion.
    pub fn get(&self, prefix: &str) -> Result<Option<String>> {
        let conn = self.connection.lock();

        // Truncate the pattern to avoid SQL injection
        let truncated_prefix = Self::truncate_like_pattern(prefix);

        // Find the longest matching key
        let result: Option<(String, String)> = conn
            .query_row(
                "SELECT key, value FROM cache WHERE ? LIKE key || '%' ORDER BY LENGTH(key) DESC LIMIT 1",
                params![truncated_prefix],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;

        if let Some((key, value)) = result {
            // Validate that the cached completion is valid for this prefix
            if value.starts_with(&prefix[key.len()..]) {
                // Update timestamp
                conn.execute(
                    "UPDATE cache SET timestamp = ? WHERE key = ?",
                    params![Self::current_timestamp(), prefix],
                )?;

                // Return the completion, truncated to what's not already there
                return Ok(Some(value[prefix.len() - key.len()..].to_string()));
            }
        }

        Ok(None)
    }

    /// Put a completion into the cache
    pub fn put(&self, prefix: &str, completion: &str) -> Result<()> {
        let conn = self.connection.lock();

        conn.execute("BEGIN TRANSACTION", [])?;

        let result = (|| -> Result<()> {
            // Check if key already exists
            let exists: bool = conn
                .query_row(
                    "SELECT 1 FROM cache WHERE key = ?",
                    params![prefix],
                    |_| Ok(true),
                )
                .optional()?
                .unwrap_or(false);

            if exists {
                // Update existing entry
                conn.execute(
                    "UPDATE cache SET value = ?, timestamp = ? WHERE key = ?",
                    params![completion, Self::current_timestamp(), prefix],
                )?;
            } else {
                // Check if we're at capacity
                let count: i64 = conn.query_row("SELECT COUNT(*) FROM cache", [], |row| row.get(0))?;

                if count as usize >= self.capacity {
                    // Evict the oldest entry (LRU)
                    conn.execute(
                        "DELETE FROM cache WHERE key = (SELECT key FROM cache ORDER BY timestamp ASC LIMIT 1)",
                        [],
                    )?;
                }

                // Insert new entry
                conn.execute(
                    "INSERT INTO cache (key, value, timestamp) VALUES (?, ?, ?)",
                    params![prefix, completion, Self::current_timestamp()],
                )?;
            }

            Ok(())
        })();

        match result {
            Ok(_) => {
                conn.execute("COMMIT", [])?;
                Ok(())
            }
            Err(e) => {
                conn.execute("ROLLBACK", [])?;
                Err(e)
            }
        }
    }

    /// Clear all entries from the cache
    pub fn clear(&self) -> Result<()> {
        let conn = self.connection.lock();
        conn.execute("DELETE FROM cache", [])?;
        Ok(())
    }

    /// Get the number of entries in the cache
    pub fn len(&self) -> Result<usize> {
        let conn = self.connection.lock();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM cache", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }

    /// Get cache statistics
    pub fn stats(&self) -> Result<Vec<CacheEntry>> {
        let conn = self.connection.lock();
        let mut stmt = conn.prepare("SELECT key, value, timestamp FROM cache ORDER BY timestamp DESC LIMIT 100")?;

        let entries = stmt
            .query_map([], |row| {
                Ok(CacheEntry {
                    key: row.get(0)?,
                    value: row.get(1)?,
                    timestamp: row.get(2)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    /// Truncate a SQLite LIKE pattern to prevent injection
    fn truncate_like_pattern(pattern: &str) -> String {
        pattern
            .chars()
            .filter(|c| *c != '%' && *c != '_')
            .take(1000)
            .collect()
    }

    /// Get the current timestamp in milliseconds
    fn current_timestamp() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_cache_put_and_get() {
        let temp_file = NamedTempFile::new().unwrap();
        let cache = AutocompleteLruCache::new(temp_file.path(), 10).unwrap();

        cache.put("const", "const foo = 'bar'").unwrap();
        let result = cache.get("const").unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_cache_lru_eviction() {
        let temp_file = NamedTempFile::new().unwrap();
        let cache = AutocompleteLruCache::new(temp_file.path(), 3).unwrap();

        cache.put("a", "apple").unwrap();
        cache.put("b", "banana").unwrap();
        cache.put("c", "cherry").unwrap();
        
        assert_eq!(cache.len().unwrap(), 3);

        // Adding one more should evict "a"
        cache.put("d", "date").unwrap();
        
        assert_eq!(cache.len().unwrap(), 3);
        assert!(cache.get("a").unwrap().is_none());
        assert!(cache.get("d").unwrap().is_some());
    }

    #[test]
    fn test_cache_prefix_matching() {
        let temp_file = NamedTempFile::new().unwrap();
        let cache = AutocompleteLruCache::new(temp_file.path(), 10).unwrap();

        cache.put("c", "ontinue").unwrap();
        
        // Should find "c" and return "ntinue" (removing the "co" that's already there)
        let result = cache.get("co").unwrap();
        assert_eq!(result, Some("ntinue".to_string()));
    }
}

