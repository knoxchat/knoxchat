//! Database layer for checkpoint storage
//!
//! This module provides a SQLite-based database layer for storing checkpoint metadata,
//! file changes, sessions, and audit logs. It's designed for enterprise use with
//! performance, reliability, and scalability in mind.

use crate::error::Result;
use crate::types::*;

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

/// Database connection pool for thread-safe access
pub struct CheckpointDatabase {
    connection: Arc<RwLock<Connection>>,
}

impl CheckpointDatabase {
    /// Create a new database instance
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let connection = Connection::open(&db_path)?;

        // Enable WAL mode for better concurrency
        // PRAGMA statements can return results, so we need to handle them properly
        connection.execute_batch(
            r#"
            PRAGMA journal_mode=WAL;
            PRAGMA synchronous=NORMAL;
            PRAGMA cache_size=10000;
            PRAGMA temp_store=MEMORY;
            PRAGMA mmap_size=268435456;
        "#,
        )?;

        let db = Self {
            connection: Arc::new(RwLock::new(connection)),
        };

        db.initialize_schema()?;
        Ok(db)
    }

    /// Initialize database schema
    fn initialize_schema(&self) -> Result<()> {
        let conn = self.connection.write();

        // Sessions table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                workspace_path TEXT NOT NULL,
                created_at TEXT NOT NULL,
                last_accessed TEXT NOT NULL,
                checkpoint_count INTEGER DEFAULT 0,
                total_size_bytes INTEGER DEFAULT 0,
                metadata TEXT DEFAULT '{}'
            )
            "#,
            [],
        )?;

        // Checkpoints table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS checkpoints (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                description TEXT NOT NULL,
                created_at TEXT NOT NULL,
                files_affected INTEGER NOT NULL,
                size_bytes INTEGER NOT NULL,
                tags TEXT DEFAULT '[]',
                metadata TEXT DEFAULT '{}',
                FOREIGN KEY (session_id) REFERENCES sessions (id) ON DELETE CASCADE
            )
            "#,
            [],
        )?;

        // File changes table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS file_changes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                checkpoint_id TEXT NOT NULL,
                path TEXT NOT NULL,
                change_type TEXT NOT NULL,
                original_content TEXT,
                new_content TEXT,
                size_bytes INTEGER NOT NULL,
                content_hash TEXT NOT NULL,
                permissions INTEGER,
                modified_at TEXT NOT NULL,
                encoding TEXT NOT NULL,
                compressed INTEGER DEFAULT 0,
                FOREIGN KEY (checkpoint_id) REFERENCES checkpoints (id) ON DELETE CASCADE
            )
            "#,
            [],
        )?;

        // Audit log table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS audit_log (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                action TEXT NOT NULL,
                actor TEXT NOT NULL,
                resource TEXT NOT NULL,
                details TEXT DEFAULT '{}',
                result TEXT NOT NULL
            )
            "#,
            [],
        )?;

        // Performance metrics table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS performance_metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                operation TEXT NOT NULL,
                duration_ms REAL NOT NULL,
                size_bytes INTEGER DEFAULT 0,
                metadata TEXT DEFAULT '{}'
            )
            "#,
            [],
        )?;

        // Create indexes for better query performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sessions_workspace ON sessions (workspace_path)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_checkpoints_session ON checkpoints (session_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_checkpoints_created ON checkpoints (created_at)",
            [],
        )?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_file_changes_checkpoint ON file_changes (checkpoint_id)", [])?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_file_changes_path ON file_changes (path)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_log (timestamp)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_audit_action ON audit_log (action)",
            [],
        )?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_performance_timestamp ON performance_metrics (timestamp)", [])?;

        Ok(())
    }

    /// Create a new session
    pub fn create_session(&self, session: &Session) -> Result<()> {
        let conn = self.connection.write();

        conn.execute(
            r#"
            INSERT INTO sessions (
                id, workspace_path, created_at, last_accessed, 
                checkpoint_count, total_size_bytes, metadata
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                session.id.to_string(),
                session.workspace_path.to_string_lossy(),
                session.created_at.to_rfc3339(),
                session.last_accessed.to_rfc3339(),
                session.checkpoint_count,
                session.total_size_bytes as i64,
                serde_json::to_string(&session.metadata)?
            ],
        )?;

        self.log_audit_action(
            AuditAction::SessionStarted,
            "system",
            &session.id.to_string(),
            HashMap::new(),
            AuditResult::Success,
        )?;

        Ok(())
    }

    /// Get a session by ID
    pub fn get_session(&self, session_id: &SessionId) -> Result<Option<Session>> {
        let conn = self.connection.read();

        let session = conn
            .query_row(
                "SELECT id, workspace_path, created_at, last_accessed, checkpoint_count, total_size_bytes, metadata FROM sessions WHERE id = ?1",
                params![session_id.to_string()],
                |row| {
                    Ok(Session {
                        id: Uuid::parse_str(&row.get::<_, String>(0)?)
                            .map_err(|_| rusqlite::Error::InvalidColumnType(0, "Invalid UUID".to_string(), rusqlite::types::Type::Text))?,
                        workspace_path: row.get::<_, String>(1)?.into(),
                        created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                            .map_err(|_| rusqlite::Error::InvalidColumnType(2, "Invalid datetime".to_string(), rusqlite::types::Type::Text))?
                            .with_timezone(&Utc),
                        last_accessed: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                            .map_err(|_| rusqlite::Error::InvalidColumnType(3, "Invalid datetime".to_string(), rusqlite::types::Type::Text))?
                            .with_timezone(&Utc),
                        checkpoint_count: row.get::<_, i32>(4)? as usize,
                        total_size_bytes: row.get::<_, i64>(5)? as u64,
                        metadata: serde_json::from_str(&row.get::<_, String>(6)?)
                            .map_err(|_| rusqlite::Error::InvalidColumnType(6, "Invalid JSON".to_string(), rusqlite::types::Type::Text))?,
                    })
                },
            )
            .optional()?;

        Ok(session)
    }

    /// Update session last accessed time
    pub fn update_session_access(&self, session_id: &SessionId) -> Result<()> {
        let conn = self.connection.write();

        conn.execute(
            "UPDATE sessions SET last_accessed = ?1 WHERE id = ?2",
            params![Utc::now().to_rfc3339(), session_id.to_string()],
        )?;

        Ok(())
    }

    /// Create a new checkpoint
    pub fn create_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()> {
        let mut conn = self.connection.write();
        let tx = conn.transaction()?;

        // Insert checkpoint
        tx.execute(
            r#"
            INSERT INTO checkpoints (
                id, session_id, description, created_at, 
                files_affected, size_bytes, tags, metadata
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                checkpoint.id.to_string(),
                checkpoint.session_id.to_string(),
                checkpoint.description,
                checkpoint.created_at.to_rfc3339(),
                checkpoint.files_affected,
                checkpoint.size_bytes as i64,
                serde_json::to_string(&checkpoint.tags)?,
                serde_json::to_string(&checkpoint.metadata)?
            ],
        )?;

        // Insert file changes
        for file_change in &checkpoint.file_changes {
            tx.execute(
                r#"
                INSERT INTO file_changes (
                    checkpoint_id, path, change_type, original_content, new_content,
                    size_bytes, content_hash, permissions, modified_at, encoding, compressed
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                "#,
                params![
                    checkpoint.id.to_string(),
                    file_change.path.to_string_lossy(),
                    serde_json::to_string(&file_change.change_type)?,
                    file_change.original_content,
                    file_change.new_content,
                    file_change.size_bytes as i64,
                    file_change.content_hash,
                    file_change.permissions.map(|p| p as i64),
                    file_change.modified_at.to_rfc3339(),
                    serde_json::to_string(&file_change.encoding)?,
                    if file_change.compressed { 1 } else { 0 }
                ],
            )?;
        }

        // Update session statistics
        tx.execute(
            r#"
            UPDATE sessions 
            SET checkpoint_count = checkpoint_count + 1,
                total_size_bytes = total_size_bytes + ?1,
                last_accessed = ?2
            WHERE id = ?3
            "#,
            params![
                checkpoint.size_bytes as i64,
                Utc::now().to_rfc3339(),
                checkpoint.session_id.to_string()
            ],
        )?;

        tx.commit()?;

        // Log audit entry
        self.log_audit_action(
            AuditAction::CheckpointCreated,
            "system",
            &checkpoint.id.to_string(),
            HashMap::from([
                (
                    "files_affected".to_string(),
                    checkpoint.files_affected.to_string(),
                ),
                ("size_bytes".to_string(), checkpoint.size_bytes.to_string()),
            ]),
            AuditResult::Success,
        )?;

        Ok(())
    }

    /// Get a checkpoint by ID
    pub fn get_checkpoint(&self, checkpoint_id: &CheckpointId) -> Result<Option<Checkpoint>> {
        let conn = self.connection.read();

        // Get checkpoint metadata
        let checkpoint_row = conn
            .query_row(
                r#"
                SELECT id, session_id, description, created_at, files_affected, 
                       size_bytes, tags, metadata
                FROM checkpoints WHERE id = ?1
                "#,
                params![checkpoint_id.to_string()],
                |row| {
                    Ok((
                        Uuid::parse_str(&row.get::<_, String>(0)?).map_err(|_| {
                            rusqlite::Error::InvalidColumnType(
                                0,
                                "Invalid UUID".to_string(),
                                rusqlite::types::Type::Text,
                            )
                        })?,
                        Uuid::parse_str(&row.get::<_, String>(1)?).map_err(|_| {
                            rusqlite::Error::InvalidColumnType(
                                1,
                                "Invalid UUID".to_string(),
                                rusqlite::types::Type::Text,
                            )
                        })?,
                        row.get::<_, String>(2)?,
                        DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                            .map_err(|_| {
                                rusqlite::Error::InvalidColumnType(
                                    3,
                                    "Invalid datetime".to_string(),
                                    rusqlite::types::Type::Text,
                                )
                            })?
                            .with_timezone(&Utc),
                        row.get::<_, i32>(4)? as usize,
                        row.get::<_, i64>(5)? as u64,
                        serde_json::from_str::<Vec<String>>(&row.get::<_, String>(6)?).map_err(
                            |_| {
                                rusqlite::Error::InvalidColumnType(
                                    6,
                                    "Invalid JSON".to_string(),
                                    rusqlite::types::Type::Text,
                                )
                            },
                        )?,
                        serde_json::from_str::<HashMap<String, String>>(&row.get::<_, String>(7)?)
                            .map_err(|_| {
                                rusqlite::Error::InvalidColumnType(
                                    7,
                                    "Invalid JSON".to_string(),
                                    rusqlite::types::Type::Text,
                                )
                            })?,
                    ))
                },
            )
            .optional()?;

        if let Some((
            id,
            session_id,
            description,
            created_at,
            files_affected,
            size_bytes,
            tags,
            metadata,
        )) = checkpoint_row
        {
            // Get file changes
            let mut stmt = conn.prepare(
                r#"
                SELECT path, change_type, original_content, new_content, size_bytes,
                       content_hash, permissions, modified_at, encoding, compressed
                FROM file_changes WHERE checkpoint_id = ?1
                ORDER BY id
                "#,
            )?;

            let file_changes: Vec<FileChange> = stmt
                .query_map(params![checkpoint_id.to_string()], |row| {
                    let file_change = FileChange {
                        path: row.get::<_, String>(0)?.into(),
                        change_type: serde_json::from_str(&row.get::<_, String>(1)?).map_err(
                            |_| {
                                rusqlite::Error::InvalidColumnType(
                                    1,
                                    "Invalid JSON".to_string(),
                                    rusqlite::types::Type::Text,
                                )
                            },
                        )?,
                        original_content: row.get::<_, Option<String>>(2)?,
                        new_content: row.get::<_, Option<String>>(3)?,
                        size_bytes: row.get::<_, i64>(4)? as u64,
                        content_hash: row.get::<_, String>(5)?,
                        permissions: row.get::<_, Option<i64>>(6)?.map(|p| p as u32),
                        modified_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                            .map_err(|_| {
                                rusqlite::Error::InvalidColumnType(
                                    7,
                                    "Invalid datetime".to_string(),
                                    rusqlite::types::Type::Text,
                                )
                            })?
                            .with_timezone(&Utc),
                        encoding: serde_json::from_str(&row.get::<_, String>(8)?).map_err(
                            |_| {
                                rusqlite::Error::InvalidColumnType(
                                    8,
                                    "Invalid JSON".to_string(),
                                    rusqlite::types::Type::Text,
                                )
                            },
                        )?,
                        compressed: row.get::<_, i32>(9)? != 0,
                    };
                    Ok(file_change)
                })?
                .collect::<std::result::Result<Vec<_>, rusqlite::Error>>()?;

            Ok(Some(Checkpoint {
                id,
                session_id,
                description,
                created_at,
                file_changes,
                file_inventory: Vec::new(), // Populated from checkpoint storage, not DB
                files_affected,
                size_bytes,
                tags,
                metadata,
            }))
        } else {
            Ok(None)
        }
    }

    /// List checkpoints for a session
    pub fn list_checkpoints(
        &self,
        session_id: &SessionId,
        limit: Option<usize>,
    ) -> Result<Vec<Checkpoint>> {
        let conn = self.connection.read();

        let sql = if let Some(limit) = limit {
            format!(
                r#"
                SELECT id, session_id, description, created_at, files_affected, 
                       size_bytes, tags, metadata
                FROM checkpoints 
                WHERE session_id = ?1 
                ORDER BY created_at DESC 
                LIMIT {}
                "#,
                limit
            )
        } else {
            r#"
            SELECT id, session_id, description, created_at, files_affected, 
                   size_bytes, tags, metadata
            FROM checkpoints 
            WHERE session_id = ?1 
            ORDER BY created_at DESC
            "#
            .to_string()
        };

        let mut stmt = conn.prepare(&sql)?;
        let checkpoint_rows = stmt.query_map(params![session_id.to_string()], |row| {
            Ok((
                Uuid::parse_str(&row.get::<_, String>(0)?).map_err(|_| {
                    rusqlite::Error::InvalidColumnType(
                        0,
                        "Invalid UUID".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })?,
                Uuid::parse_str(&row.get::<_, String>(1)?).map_err(|_| {
                    rusqlite::Error::InvalidColumnType(
                        1,
                        "Invalid UUID".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })?,
                row.get::<_, String>(2)?,
                DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .map_err(|_| {
                        rusqlite::Error::InvalidColumnType(
                            3,
                            "Invalid datetime".to_string(),
                            rusqlite::types::Type::Text,
                        )
                    })?
                    .with_timezone(&Utc),
                row.get::<_, i32>(4)? as usize,
                row.get::<_, i64>(5)? as u64,
                serde_json::from_str::<Vec<String>>(&row.get::<_, String>(6)?).map_err(|_| {
                    rusqlite::Error::InvalidColumnType(
                        6,
                        "Invalid JSON".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })?,
                serde_json::from_str::<HashMap<String, String>>(&row.get::<_, String>(7)?)
                    .map_err(|_| {
                        rusqlite::Error::InvalidColumnType(
                            7,
                            "Invalid JSON".to_string(),
                            rusqlite::types::Type::Text,
                        )
                    })?,
            ))
        })?;

        let mut checkpoints = Vec::new();
        for row in checkpoint_rows {
            let (
                id,
                session_id,
                description,
                created_at,
                files_affected,
                size_bytes,
                tags,
                metadata,
            ) = row?;

            // For listing, we don't load file changes to improve performance
            checkpoints.push(Checkpoint {
                id,
                session_id,
                description,
                created_at,
                file_changes: Vec::new(),   // Empty for listing
                file_inventory: Vec::new(), // Empty for listing
                files_affected,
                size_bytes,
                tags,
                metadata,
            });
        }

        Ok(checkpoints)
    }

    /// Delete a checkpoint
    pub fn delete_checkpoint(&self, checkpoint_id: &CheckpointId) -> Result<()> {
        let mut conn = self.connection.write();
        let tx = conn.transaction()?;

        // Get checkpoint info before deletion for audit
        let checkpoint_info = tx
            .query_row(
                "SELECT session_id, size_bytes FROM checkpoints WHERE id = ?1",
                params![checkpoint_id.to_string()],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
            )
            .optional()?;

        if let Some((session_id, size_bytes)) = checkpoint_info {
            // Delete file changes (cascade should handle this, but being explicit)
            tx.execute(
                "DELETE FROM file_changes WHERE checkpoint_id = ?1",
                params![checkpoint_id.to_string()],
            )?;

            // Delete checkpoint
            tx.execute(
                "DELETE FROM checkpoints WHERE id = ?1",
                params![checkpoint_id.to_string()],
            )?;

            // Update session statistics
            tx.execute(
                r#"
                UPDATE sessions 
                SET checkpoint_count = checkpoint_count - 1,
                    total_size_bytes = total_size_bytes - ?1
                WHERE id = ?2
                "#,
                params![size_bytes, session_id],
            )?;

            tx.commit()?;

            // Log audit entry
            self.log_audit_action(
                AuditAction::CheckpointDeleted,
                "system",
                &checkpoint_id.to_string(),
                HashMap::from([("size_bytes".to_string(), size_bytes.to_string())]),
                AuditResult::Success,
            )?;
        }

        Ok(())
    }

    /// Get checkpoint statistics
    pub fn get_stats(&self) -> Result<CheckpointStats> {
        let conn = self.connection.read();

        let (total_checkpoints, total_sessions, total_storage_bytes) = conn.query_row(
            r#"
            SELECT 
                (SELECT COUNT(*) FROM checkpoints) as total_checkpoints,
                (SELECT COUNT(*) FROM sessions) as total_sessions,
                (SELECT COALESCE(SUM(total_size_bytes), 0) FROM sessions) as total_storage
            "#,
            [],
            |row| {
                Ok((
                    row.get::<_, i32>(0)? as usize,
                    row.get::<_, i32>(1)? as usize,
                    row.get::<_, i64>(2)? as u64,
                ))
            },
        )?;

        let avg_checkpoint_size = if total_checkpoints > 0 {
            total_storage_bytes / total_checkpoints as u64
        } else {
            0
        };

        let files_tracked = conn.query_row("SELECT COUNT(*) FROM file_changes", [], |row| {
            row.get::<_, i32>(0).map(|n| n as usize)
        })?;

        let last_cleanup = conn
            .query_row(
                r#"
            SELECT timestamp FROM audit_log 
            WHERE action = 'CleanupPerformed' 
            ORDER BY timestamp DESC 
            LIMIT 1
            "#,
                [],
                |row| {
                    DateTime::parse_from_rfc3339(&row.get::<_, String>(0)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .map_err(|_| {
                            rusqlite::Error::InvalidColumnType(
                                0,
                                "Invalid datetime".to_string(),
                                rusqlite::types::Type::Text,
                            )
                        })
                },
            )
            .optional()?;

        // Calculate performance metrics
        let performance = self.calculate_performance_metrics()?;

        Ok(CheckpointStats {
            total_checkpoints,
            total_sessions,
            total_storage_bytes,
            avg_checkpoint_size,
            files_tracked,
            compression_ratio: 0.7, // TODO: Calculate actual compression ratio
            deduplication_savings: 0, // TODO: Implement deduplication tracking
            last_cleanup,
            performance,
        })
    }

    /// Calculate performance metrics
    fn calculate_performance_metrics(&self) -> Result<PerformanceMetrics> {
        let conn = self.connection.read();

        // Get average creation time from recent operations
        let avg_creation_time_ms = conn
            .query_row(
                r#"
            SELECT AVG(duration_ms) FROM performance_metrics 
            WHERE operation = 'checkpoint_creation' 
            AND timestamp > datetime('now', '-24 hours')
            "#,
                [],
                |row| row.get::<_, Option<f64>>(0),
            )?
            .unwrap_or(0.0);

        let avg_restoration_time_ms = conn
            .query_row(
                r#"
            SELECT AVG(duration_ms) FROM performance_metrics 
            WHERE operation = 'checkpoint_restoration' 
            AND timestamp > datetime('now', '-24 hours')
            "#,
                [],
                |row| row.get::<_, Option<f64>>(0),
            )?
            .unwrap_or(0.0);

        // Estimate other metrics (would need actual monitoring in production)
        Ok(PerformanceMetrics {
            avg_creation_time_ms,
            avg_restoration_time_ms,
            db_queries_per_second: 1000.0, // Placeholder
            file_io_mbps: 100.0,           // Placeholder
            memory_usage_mb: 50.0,         // Placeholder
        })
    }

    /// Log an audit action
    pub fn log_audit_action(
        &self,
        action: AuditAction,
        actor: &str,
        resource: &str,
        details: HashMap<String, String>,
        result: AuditResult,
    ) -> Result<()> {
        let conn = self.connection.write();

        let entry = AuditEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            action,
            actor: actor.to_string(),
            resource: resource.to_string(),
            details,
            result,
        };

        conn.execute(
            r#"
            INSERT INTO audit_log (id, timestamp, action, actor, resource, details, result)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                entry.id.to_string(),
                entry.timestamp.to_rfc3339(),
                serde_json::to_string(&entry.action)?,
                entry.actor,
                entry.resource,
                serde_json::to_string(&entry.details)?,
                serde_json::to_string(&entry.result)?
            ],
        )?;

        Ok(())
    }

    /// Record performance metric
    pub fn record_performance_metric(
        &self,
        operation: &str,
        duration_ms: f64,
        size_bytes: Option<u64>,
        metadata: HashMap<String, String>,
    ) -> Result<()> {
        let conn = self.connection.write();

        conn.execute(
            r#"
            INSERT INTO performance_metrics (timestamp, operation, duration_ms, size_bytes, metadata)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            params![
                Utc::now().to_rfc3339(),
                operation,
                duration_ms,
                size_bytes.unwrap_or(0) as i64,
                serde_json::to_string(&metadata)?
            ],
        )?;

        Ok(())
    }

    /// Clean up old checkpoints based on retention policy
    pub fn cleanup_old_checkpoints(&self, retention_days: i64) -> Result<usize> {
        let mut conn = self.connection.write();
        let tx = conn.transaction()?;

        let cutoff_date = Utc::now() - chrono::Duration::days(retention_days);

        // Get checkpoints to delete
        let checkpoint_ids: Vec<String> = {
            let mut stmt = tx.prepare("SELECT id FROM checkpoints WHERE created_at < ?1")?;

            let result = stmt
                .query_map(params![cutoff_date.to_rfc3339()], |row| {
                    Ok(row.get::<_, String>(0)?)
                })?
                .collect::<std::result::Result<Vec<_>, rusqlite::Error>>()?;
            result
        };

        let count = checkpoint_ids.len();

        // Delete old checkpoints (cascades to file_changes)
        tx.execute(
            "DELETE FROM checkpoints WHERE created_at < ?1",
            params![cutoff_date.to_rfc3339()],
        )?;

        // Update session statistics
        tx.execute(
            r#"
            UPDATE sessions 
            SET checkpoint_count = (
                SELECT COUNT(*) FROM checkpoints WHERE session_id = sessions.id
            ),
            total_size_bytes = (
                SELECT COALESCE(SUM(size_bytes), 0) FROM checkpoints WHERE session_id = sessions.id
            )
            "#,
            [],
        )?;

        tx.commit()?;

        // Log cleanup action
        self.log_audit_action(
            AuditAction::CleanupPerformed,
            "system",
            "checkpoints",
            HashMap::from([
                ("deleted_count".to_string(), count.to_string()),
                ("retention_days".to_string(), retention_days.to_string()),
            ]),
            AuditResult::Success,
        )?;

        Ok(count)
    }

    /// Vacuum database to reclaim space
    pub fn vacuum(&self) -> Result<()> {
        let conn = self.connection.write();
        conn.execute("VACUUM", [])?;
        Ok(())
    }

    /// Get database size in bytes
    pub fn get_database_size(&self) -> Result<u64> {
        let conn = self.connection.read();
        let size = conn.query_row(
            "SELECT page_count * page_size FROM pragma_page_count(), pragma_page_size()",
            [],
            |row| row.get::<_, i64>(0),
        )? as u64;

        Ok(size)
    }
}
