//! Database layer for checkpoint storage
//!
//! This module provides a SQLite-based database layer for storing checkpoint metadata,
//! file changes, sessions, and audit logs. It's designed for enterprise use with
//! performance, reliability, and scalability in mind.

use crate::error::Result;
use crate::types::*;

use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use uuid::Uuid;

/// Database connection pool for thread-safe access
pub struct CheckpointDatabase {
    connection: Arc<Mutex<Connection>>,
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
            connection: Arc::new(Mutex::new(connection)),
        };

        db.initialize_schema()?;
        Ok(db)
    }

    /// Initialize database schema
    fn initialize_schema(&self) -> Result<()> {
        let conn = self.connection.lock();

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

        // === Phase 8.1: Incremental Checkpointing ===
        // Add incremental fields to checkpoints table (migration-safe)
        let _ = conn.execute(
            "ALTER TABLE checkpoints ADD COLUMN parent_checkpoint_id TEXT",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE checkpoints ADD COLUMN is_full_snapshot INTEGER DEFAULT 1",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE checkpoints ADD COLUMN delta_depth INTEGER DEFAULT 0",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE checkpoints ADD COLUMN branch_id TEXT",
            [],
        );

        // === Phase 8.2: Branches table ===
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS branches (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                session_id TEXT NOT NULL,
                base_checkpoint_id TEXT NOT NULL,
                head_checkpoint_id TEXT,
                created_at TEXT NOT NULL,
                description TEXT DEFAULT '',
                is_default INTEGER DEFAULT 0,
                metadata TEXT DEFAULT '{}',
                FOREIGN KEY (session_id) REFERENCES sessions (id) ON DELETE CASCADE,
                FOREIGN KEY (base_checkpoint_id) REFERENCES checkpoints (id)
            )
            "#,
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_branches_session ON branches (session_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_checkpoints_branch ON checkpoints (branch_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_checkpoints_parent ON checkpoints (parent_checkpoint_id)",
            [],
        )?;

        // === Phase 8.3: Checkpoint analysis results cache ===
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS checkpoint_analysis (
                checkpoint_id TEXT PRIMARY KEY,
                generated_description TEXT NOT NULL,
                risk_level TEXT NOT NULL,
                risk_score REAL NOT NULL,
                risk_factors TEXT DEFAULT '[]',
                recommendations TEXT DEFAULT '[]',
                affected_features TEXT DEFAULT '[]',
                affected_layers TEXT DEFAULT '[]',
                change_scope TEXT NOT NULL,
                transitive_impact TEXT DEFAULT '[]',
                grouping_suggestion TEXT,
                analyzed_at TEXT NOT NULL,
                FOREIGN KEY (checkpoint_id) REFERENCES checkpoints (id) ON DELETE CASCADE
            )
            "#,
            [],
        )?;

        // === Phase 8.4: Collaborative Checkpoints ===
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS shared_bundles (
                id TEXT PRIMARY KEY,
                checkpoint_ids TEXT NOT NULL,
                shared_by_user_id TEXT NOT NULL,
                shared_by_name TEXT NOT NULL,
                shared_by_machine TEXT NOT NULL,
                shared_at TEXT NOT NULL,
                description TEXT DEFAULT '',
                format_version INTEGER DEFAULT 1,
                data_size INTEGER DEFAULT 0
            )
            "#,
            [],
        )?;

        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS compliance_audit (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                user_id TEXT NOT NULL,
                machine_id TEXT NOT NULL,
                action TEXT NOT NULL,
                resource_type TEXT NOT NULL,
                resource_id TEXT NOT NULL,
                details TEXT DEFAULT '{}',
                outcome TEXT NOT NULL
            )
            "#,
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_compliance_audit_timestamp ON compliance_audit (timestamp)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_compliance_audit_action ON compliance_audit (action)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_compliance_audit_resource ON compliance_audit (resource_type, resource_id)",
            [],
        )?;

        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS sync_status (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                status TEXT NOT NULL DEFAULT 'Idle',
                last_sync_at TEXT,
                last_error TEXT
            )
            "#,
            [],
        )?;
        let _ = conn.execute(
            "INSERT OR IGNORE INTO sync_status (id, status) VALUES (1, 'Idle')",
            [],
        );

        // === Phase 8.5: Performance Monitoring ===
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS storage_snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                total_bytes INTEGER NOT NULL,
                checkpoint_data_bytes INTEGER NOT NULL,
                database_bytes INTEGER NOT NULL,
                blob_count INTEGER NOT NULL,
                checkpoint_count INTEGER NOT NULL
            )
            "#,
            [],
        )?;

        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS restoration_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                checkpoint_id TEXT NOT NULL,
                success INTEGER NOT NULL,
                duration_ms REAL NOT NULL,
                files_restored INTEGER NOT NULL,
                files_failed INTEGER NOT NULL,
                error TEXT
            )
            "#,
            [],
        )?;

        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS ai_session_metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                started_at TEXT NOT NULL,
                ended_at TEXT,
                files_changed INTEGER NOT NULL DEFAULT 0,
                lines_added INTEGER NOT NULL DEFAULT 0,
                lines_deleted INTEGER NOT NULL DEFAULT 0,
                checkpoints_created INTEGER NOT NULL DEFAULT 0,
                rollbacks INTEGER NOT NULL DEFAULT 0,
                duration_seconds REAL NOT NULL DEFAULT 0.0
            )
            "#,
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_storage_snapshots_ts ON storage_snapshots (timestamp)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_restoration_events_ts ON restoration_events (timestamp)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_ai_session_metrics_session ON ai_session_metrics (session_id)",
            [],
        )?;

        Ok(())
    }

    /// Create a new session
    pub fn create_session(&self, session: &Session) -> Result<()> {
        {
            let conn = self.connection.lock();

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
                    session.checkpoint_count as i64,
                    session.total_size_bytes as i64,
                    serde_json::to_string(&session.metadata)?
                ],
            )?;
        }

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
        let conn = self.connection.lock();

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
        let conn = self.connection.lock();

        conn.execute(
            "UPDATE sessions SET last_accessed = ?1 WHERE id = ?2",
            params![Utc::now().to_rfc3339(), session_id.to_string()],
        )?;

        Ok(())
    }

    /// Create a new checkpoint
    pub fn create_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()> {
        {
            let mut conn = self.connection.lock();
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
                    checkpoint.files_affected as i64,
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
        }

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
        let conn = self.connection.lock();

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
                parent_checkpoint_id: None,
                is_full_snapshot: true,
                delta_depth: 0,
                branch_id: None,
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
        let conn = self.connection.lock();

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
                parent_checkpoint_id: None,
                is_full_snapshot: true,
                delta_depth: 0,
                branch_id: None,
            });
        }

        Ok(checkpoints)
    }

    /// Delete a checkpoint
    pub fn delete_checkpoint(&self, checkpoint_id: &CheckpointId) -> Result<()> {
        let checkpoint_info = {
            let mut conn = self.connection.lock();
            let tx = conn.transaction()?;

            let checkpoint_info = tx
                .query_row(
                    "SELECT session_id, size_bytes FROM checkpoints WHERE id = ?1",
                    params![checkpoint_id.to_string()],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
                )
                .optional()?;

            if let Some((session_id, size_bytes)) = &checkpoint_info {
                tx.execute(
                    "DELETE FROM file_changes WHERE checkpoint_id = ?1",
                    params![checkpoint_id.to_string()],
                )?;

                tx.execute(
                    "DELETE FROM checkpoints WHERE id = ?1",
                    params![checkpoint_id.to_string()],
                )?;

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
            }

            checkpoint_info
        };

        if let Some((_session_id, size_bytes)) = checkpoint_info {
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
        let conn = self.connection.lock();

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

        // Calculate actual compression ratio from stored content sizes
        let (compression_ratio, deduplication_savings) = self.calculate_storage_metrics()?;

        Ok(CheckpointStats {
            total_checkpoints,
            total_sessions,
            total_storage_bytes,
            avg_checkpoint_size,
            files_tracked,
            compression_ratio,
            deduplication_savings,
            last_cleanup,
            performance,
        })
    }

    /// Calculate actual compression ratio and deduplication savings from the storage metrics table
    fn calculate_storage_metrics(&self) -> Result<(f64, u64)> {
        let conn = self.connection.lock();

        // Try to get the latest storage metrics if they've been recorded
        let result = conn
            .query_row(
                r#"
                SELECT compression_ratio, deduplication_savings 
                FROM storage_metrics 
                ORDER BY recorded_at DESC 
                LIMIT 1
                "#,
                [],
                |row| {
                    Ok((
                        row.get::<_, f64>(0)?,
                        row.get::<_, i64>(1)? as u64,
                    ))
                },
            )
            .optional()?;

        match result {
            Some((ratio, savings)) => Ok((ratio, savings)),
            None => {
                // Fallback: estimate from file_changes table
                let total_original: u64 = conn
                    .query_row(
                        "SELECT COALESCE(SUM(size_bytes), 0) FROM file_changes",
                        [],
                        |row| row.get::<_, i64>(0).map(|v| v as u64),
                    )?;

                // If we have data, estimate ratio from actual storage vs logical size
                if total_original > 0 {
                    let ratio = if total_original > 0 {
                        // Use actual session storage vs logical file sizes
                        let actual_storage = conn.query_row(
                            "SELECT COALESCE(SUM(total_size_bytes), 0) FROM sessions",
                            [],
                            |row| row.get::<_, i64>(0).map(|v| v as u64),
                        )?;
                        if actual_storage > 0 {
                            actual_storage as f64 / total_original as f64
                        } else {
                            1.0
                        }
                    } else {
                        1.0
                    };
                    Ok((ratio.min(1.0), 0))
                } else {
                    Ok((1.0, 0))
                }
            }
        }
    }

    /// Record storage metrics (called by the manager after checkpoint operations)
    pub fn record_storage_metrics(
        &self,
        compression_ratio: f64,
        deduplication_savings: u64,
    ) -> Result<()> {
        let conn = self.connection.lock();

        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS storage_metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                recorded_at TEXT NOT NULL,
                compression_ratio REAL NOT NULL,
                deduplication_savings INTEGER NOT NULL
            )
            "#,
            [],
        )?;

        conn.execute(
            r#"
            INSERT INTO storage_metrics (recorded_at, compression_ratio, deduplication_savings)
            VALUES (?1, ?2, ?3)
            "#,
            params![
                Utc::now().to_rfc3339(),
                compression_ratio,
                deduplication_savings as i64
            ],
        )?;

        // Keep only the last 100 entries
        conn.execute(
            r#"
            DELETE FROM storage_metrics 
            WHERE id NOT IN (
                SELECT id FROM storage_metrics ORDER BY recorded_at DESC LIMIT 100
            )
            "#,
            [],
        )?;

        Ok(())
    }

    /// Calculate performance metrics
    fn calculate_performance_metrics(&self) -> Result<PerformanceMetrics> {
        let conn = self.connection.lock();

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
        let conn = self.connection.lock();

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
        let conn = self.connection.lock();

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
        let count = {
            let mut conn = self.connection.lock();
            let tx = conn.transaction()?;

            let cutoff_date = Utc::now() - chrono::Duration::days(retention_days);

            let checkpoint_ids: Vec<String> = {
                let mut stmt = tx.prepare("SELECT id FROM checkpoints WHERE created_at < ?1")?;

                let result = stmt
                    .query_map(params![cutoff_date.to_rfc3339()], |row| {
                        row.get::<_, String>(0)
                    })?
                    .collect::<std::result::Result<Vec<_>, rusqlite::Error>>()?;
                result
            };

            let count = checkpoint_ids.len();

            tx.execute(
                "DELETE FROM checkpoints WHERE created_at < ?1",
                params![cutoff_date.to_rfc3339()],
            )?;

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
            count
        };

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
        let conn = self.connection.lock();
        conn.execute("VACUUM", [])?;
        Ok(())
    }

    /// Get database size in bytes
    pub fn get_database_size(&self) -> Result<u64> {
        let conn = self.connection.lock();
        let size = conn.query_row(
            "SELECT page_count * page_size FROM pragma_page_count(), pragma_page_size()",
            [],
            |row| row.get::<_, i64>(0),
        )? as u64;

        Ok(size)
    }

    // ========================================
    // Phase 8.1: Incremental Checkpointing
    // ========================================

    /// Create a checkpoint with incremental (delta) metadata
    pub fn create_incremental_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()> {
        // Use the existing create_checkpoint, then update incremental fields
        self.create_checkpoint(checkpoint)?;

        let conn = self.connection.lock();
        conn.execute(
            r#"
            UPDATE checkpoints 
            SET parent_checkpoint_id = ?1, is_full_snapshot = ?2, delta_depth = ?3, branch_id = ?4
            WHERE id = ?5
            "#,
            params![
                checkpoint.parent_checkpoint_id.map(|id| id.to_string()),
                if checkpoint.is_full_snapshot { 1 } else { 0 },
                checkpoint.delta_depth as i64,
                checkpoint.branch_id,
                checkpoint.id.to_string(),
            ],
        )?;

        Ok(())
    }

    /// Get the delta chain for a checkpoint (walk up parent_checkpoint_id links)
    pub fn get_delta_chain(&self, checkpoint_id: &CheckpointId) -> Result<Vec<Checkpoint>> {
        let mut chain = Vec::new();
        let mut current_id = Some(*checkpoint_id);

        while let Some(id) = current_id {
            if let Some(checkpoint) = self.get_checkpoint(&id)? {
                let is_full = self.get_checkpoint_incremental_field(&id, "is_full_snapshot")?;
                let parent = self.get_checkpoint_parent(&id)?;
                chain.push(checkpoint);

                if is_full.unwrap_or(true) {
                    break; // Reached a full snapshot, chain is complete
                }
                current_id = parent;
            } else {
                break;
            }
        }

        // Reverse so the full snapshot is first, deltas follow in order
        chain.reverse();
        Ok(chain)
    }

    /// Get parent checkpoint ID
    fn get_checkpoint_parent(&self, checkpoint_id: &CheckpointId) -> Result<Option<CheckpointId>> {
        let conn = self.connection.lock();
        let parent: Option<String> = conn
            .query_row(
                "SELECT parent_checkpoint_id FROM checkpoints WHERE id = ?1",
                params![checkpoint_id.to_string()],
                |row| row.get(0),
            )
            .optional()?
            .flatten();

        if let Some(parent_str) = parent {
            Ok(Some(Uuid::parse_str(&parent_str).map_err(|_| {
                crate::error::CheckpointError::generic("Invalid parent checkpoint UUID")
            })?))
        } else {
            Ok(None)
        }
    }

    /// Get an incremental field for a checkpoint
    fn get_checkpoint_incremental_field(
        &self,
        checkpoint_id: &CheckpointId,
        field: &str,
    ) -> Result<Option<bool>> {
        let conn = self.connection.lock();
        // Only allow known fields to prevent SQL injection
        let query = match field {
            "is_full_snapshot" => {
                "SELECT is_full_snapshot FROM checkpoints WHERE id = ?1"
            }
            _ => return Ok(None),
        };
        let val: Option<i32> = conn
            .query_row(query, params![checkpoint_id.to_string()], |row| {
                row.get(0)
            })
            .optional()?
            .flatten();
        Ok(val.map(|v| v != 0))
    }

    /// Count checkpoints since last full snapshot on a branch
    pub fn count_since_last_full_snapshot(
        &self,
        session_id: &SessionId,
        branch_id: Option<&str>,
    ) -> Result<u32> {
        let conn = self.connection.lock();
        let count: i64 = if let Some(bid) = branch_id {
            conn.query_row(
                r#"
                SELECT COUNT(*) FROM checkpoints 
                WHERE session_id = ?1 AND branch_id = ?2 AND is_full_snapshot = 0
                AND created_at > COALESCE(
                    (SELECT MAX(created_at) FROM checkpoints 
                     WHERE session_id = ?1 AND branch_id = ?2 AND is_full_snapshot = 1),
                    '1970-01-01T00:00:00Z'
                )
                "#,
                params![session_id.to_string(), bid],
                |row| row.get(0),
            )?
        } else {
            conn.query_row(
                r#"
                SELECT COUNT(*) FROM checkpoints 
                WHERE session_id = ?1 AND branch_id IS NULL AND is_full_snapshot = 0
                AND created_at > COALESCE(
                    (SELECT MAX(created_at) FROM checkpoints 
                     WHERE session_id = ?1 AND branch_id IS NULL AND is_full_snapshot = 1),
                    '1970-01-01T00:00:00Z'
                )
                "#,
                params![session_id.to_string()],
                |row| row.get(0),
            )?
        };
        Ok(count as u32)
    }

    /// Get the latest checkpoint on a branch (or main if no branch)
    pub fn get_latest_checkpoint(
        &self,
        session_id: &SessionId,
        branch_id: Option<&str>,
    ) -> Result<Option<Checkpoint>> {
        let conn = self.connection.lock();
        let id_str: Option<String> = if let Some(bid) = branch_id {
            conn.query_row(
                "SELECT id FROM checkpoints WHERE session_id = ?1 AND branch_id = ?2 ORDER BY created_at DESC LIMIT 1",
                params![session_id.to_string(), bid],
                |row| row.get(0),
            ).optional()?
        } else {
            conn.query_row(
                "SELECT id FROM checkpoints WHERE session_id = ?1 AND branch_id IS NULL ORDER BY created_at DESC LIMIT 1",
                params![session_id.to_string()],
                |row| row.get(0),
            ).optional()?
        };

        if let Some(id_s) = id_str {
            let id = Uuid::parse_str(&id_s)
                .map_err(|_| crate::error::CheckpointError::generic("Invalid UUID"))?;
            self.get_checkpoint(&id)
        } else {
            Ok(None)
        }
    }

    // ========================================
    // Phase 8.2: Branch Management
    // ========================================

    /// Create a new branch
    pub fn create_branch(&self, branch: &Branch) -> Result<()> {
        let conn = self.connection.lock();
        conn.execute(
            r#"
            INSERT INTO branches (
                id, name, session_id, base_checkpoint_id, head_checkpoint_id,
                created_at, description, is_default, metadata
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                branch.id,
                branch.name,
                branch.session_id.to_string(),
                branch.base_checkpoint_id.to_string(),
                branch.head_checkpoint_id.map(|id| id.to_string()),
                branch.created_at.to_rfc3339(),
                branch.description,
                if branch.is_default { 1 } else { 0 },
                serde_json::to_string(&branch.metadata)?,
            ],
        )?;
        Ok(())
    }

    /// Get a branch by ID
    pub fn get_branch(&self, branch_id: &str) -> Result<Option<Branch>> {
        let conn = self.connection.lock();
        conn.query_row(
            r#"
            SELECT id, name, session_id, base_checkpoint_id, head_checkpoint_id,
                   created_at, description, is_default, metadata
            FROM branches WHERE id = ?1
            "#,
            params![branch_id],
            |row| {
                Ok(Branch {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    session_id: Uuid::parse_str(&row.get::<_, String>(2)?)
                        .unwrap_or_default(),
                    base_checkpoint_id: Uuid::parse_str(&row.get::<_, String>(3)?)
                        .unwrap_or_default(),
                    head_checkpoint_id: row
                        .get::<_, Option<String>>(4)?
                        .and_then(|s| Uuid::parse_str(&s).ok()),
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    description: row.get(6)?,
                    is_default: row.get::<_, i32>(7)? != 0,
                    metadata: serde_json::from_str(&row.get::<_, String>(8)?)
                        .unwrap_or_default(),
                })
            },
        )
        .optional()
        .map_err(Into::into)
    }

    /// List all branches for a session
    pub fn list_branches(&self, session_id: &SessionId) -> Result<Vec<Branch>> {
        let conn = self.connection.lock();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, session_id, base_checkpoint_id, head_checkpoint_id,
                   created_at, description, is_default, metadata
            FROM branches WHERE session_id = ?1 ORDER BY created_at ASC
            "#,
        )?;

        let branches = stmt
            .query_map(params![session_id.to_string()], |row| {
                Ok(Branch {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    session_id: Uuid::parse_str(&row.get::<_, String>(2)?)
                        .unwrap_or_default(),
                    base_checkpoint_id: Uuid::parse_str(&row.get::<_, String>(3)?)
                        .unwrap_or_default(),
                    head_checkpoint_id: row
                        .get::<_, Option<String>>(4)?
                        .and_then(|s| Uuid::parse_str(&s).ok()),
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    description: row.get(6)?,
                    is_default: row.get::<_, i32>(7)? != 0,
                    metadata: serde_json::from_str(&row.get::<_, String>(8)?)
                        .unwrap_or_default(),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(branches)
    }

    /// Update branch head checkpoint
    pub fn update_branch_head(
        &self,
        branch_id: &str,
        head_checkpoint_id: &CheckpointId,
    ) -> Result<()> {
        let conn = self.connection.lock();
        conn.execute(
            "UPDATE branches SET head_checkpoint_id = ?1 WHERE id = ?2",
            params![head_checkpoint_id.to_string(), branch_id],
        )?;
        Ok(())
    }

    /// Delete a branch (does not delete its checkpoints)
    pub fn delete_branch(&self, branch_id: &str) -> Result<()> {
        let conn = self.connection.lock();
        conn.execute("DELETE FROM branches WHERE id = ?1", params![branch_id])?;
        Ok(())
    }

    /// List checkpoints on a specific branch
    pub fn list_branch_checkpoints(
        &self,
        branch_id: &str,
        limit: Option<usize>,
    ) -> Result<Vec<Checkpoint>> {
        let conn = self.connection.lock();
        let limit_val = limit.unwrap_or(100) as i64;
        let mut stmt = conn.prepare(
            r#"
            SELECT id FROM checkpoints 
            WHERE branch_id = ?1 
            ORDER BY created_at DESC 
            LIMIT ?2
            "#,
        )?;

        let ids: Vec<String> = stmt
            .query_map(params![branch_id, limit_val], |row| row.get(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let mut checkpoints = Vec::new();
        for id_str in ids {
            if let Ok(id) = Uuid::parse_str(&id_str) {
                if let Some(cp) = self.get_checkpoint(&id)? {
                    checkpoints.push(cp);
                }
            }
        }
        Ok(checkpoints)
    }

    /// Find common ancestor checkpoint between two branches
    pub fn find_common_ancestor(
        &self,
        branch_a_id: &str,
        branch_b_id: &str,
    ) -> Result<Option<CheckpointId>> {
        // Walk up the parent chain of branch_a and collect all IDs
        let branch_a = self.get_branch(branch_a_id)?;
        let branch_b = self.get_branch(branch_b_id)?;

        match (branch_a, branch_b) {
            (Some(a), Some(b)) => {
                // Simple strategy: the base checkpoint of the newer branch
                // is likely the common ancestor
                if a.created_at >= b.created_at {
                    Ok(Some(a.base_checkpoint_id))
                } else {
                    Ok(Some(b.base_checkpoint_id))
                }
            }
            _ => Ok(None),
        }
    }

    // ========================================
    // Phase 8.3: Analysis Cache
    // ========================================

    /// Store checkpoint analysis results
    pub fn store_analysis(&self, checkpoint_id: &CheckpointId, analysis: &CheckpointAnalysis) -> Result<()> {
        let conn = self.connection.lock();
        conn.execute(
            r#"
            INSERT OR REPLACE INTO checkpoint_analysis (
                checkpoint_id, generated_description, risk_level, risk_score,
                risk_factors, recommendations, affected_features, affected_layers,
                change_scope, transitive_impact, grouping_suggestion, analyzed_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            "#,
            params![
                checkpoint_id.to_string(),
                analysis.generated_description,
                serde_json::to_string(&analysis.risk_assessment.level)?,
                analysis.risk_assessment.score,
                serde_json::to_string(&analysis.risk_assessment.factors)?,
                serde_json::to_string(&analysis.risk_assessment.recommendations)?,
                serde_json::to_string(&analysis.impact_analysis.affected_features)?,
                serde_json::to_string(&analysis.impact_analysis.affected_layers)?,
                serde_json::to_string(&analysis.impact_analysis.scope)?,
                serde_json::to_string(&analysis.impact_analysis.transitive_impact)?,
                analysis.grouping_suggestion.as_ref().map(|g| serde_json::to_string(g).unwrap_or_default()),
                Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Get cached analysis for a checkpoint
    pub fn get_analysis(&self, checkpoint_id: &CheckpointId) -> Result<Option<CheckpointAnalysis>> {
        let conn = self.connection.lock();
        conn.query_row(
            r#"
            SELECT generated_description, risk_level, risk_score, risk_factors,
                   recommendations, affected_features, affected_layers, change_scope,
                   transitive_impact, grouping_suggestion
            FROM checkpoint_analysis WHERE checkpoint_id = ?1
            "#,
            params![checkpoint_id.to_string()],
            |row| {
                let risk_level: RiskLevel = serde_json::from_str(&row.get::<_, String>(1)?)
                    .unwrap_or(RiskLevel::Low);
                let risk_factors: Vec<RiskFactor> = serde_json::from_str(&row.get::<_, String>(3)?)
                    .unwrap_or_default();
                let recommendations: Vec<String> = serde_json::from_str(&row.get::<_, String>(4)?)
                    .unwrap_or_default();
                let affected_features: Vec<AffectedFeature> = serde_json::from_str(&row.get::<_, String>(5)?)
                    .unwrap_or_default();
                let affected_layers: Vec<String> = serde_json::from_str(&row.get::<_, String>(6)?)
                    .unwrap_or_default();
                let change_scope: ChangeScope = serde_json::from_str(&row.get::<_, String>(7)?)
                    .unwrap_or(ChangeScope::File);
                let transitive_impact: Vec<PathBuf> = serde_json::from_str(&row.get::<_, String>(8)?)
                    .unwrap_or_default();
                let grouping_suggestion: Option<GroupingSuggestion> = row
                    .get::<_, Option<String>>(9)?
                    .and_then(|s| serde_json::from_str(&s).ok());

                Ok(CheckpointAnalysis {
                    generated_description: row.get(0)?,
                    risk_assessment: RiskAssessment {
                        level: risk_level,
                        score: row.get(2)?,
                        factors: risk_factors,
                        recommendations,
                    },
                    impact_analysis: ImpactAnalysis {
                        affected_features,
                        affected_layers,
                        scope: change_scope,
                        transitive_impact,
                    },
                    grouping_suggestion,
                })
            },
        )
        .optional()
        .map_err(Into::into)
    }

    // ========================================
    // Phase 8.4: Collaborative Checkpoints DB
    // ========================================

    /// Record a share event
    pub fn record_share_event(&self, bundle: &SharedCheckpointBundle) -> Result<()> {
        let conn = self.connection.lock();
        conn.execute(
            r#"
            INSERT INTO shared_bundles (
                id, checkpoint_ids, shared_by_user_id, shared_by_name, shared_by_machine,
                shared_at, description, format_version, data_size
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                bundle.id,
                serde_json::to_string(&bundle.checkpoint_ids)?,
                bundle.shared_by.user_id,
                bundle.shared_by.display_name,
                bundle.shared_by.machine_id,
                bundle.shared_at.to_rfc3339(),
                bundle.description,
                bundle.format_version as i64,
                bundle.data.len() as i64,
            ],
        )?;
        Ok(())
    }

    /// List shared bundles
    pub fn list_shared_bundles(&self) -> Result<Vec<SharedCheckpointBundle>> {
        let conn = self.connection.lock();
        let mut stmt = conn.prepare(
            "SELECT id, checkpoint_ids, shared_by_user_id, shared_by_name, shared_by_machine, \
             shared_at, description, format_version FROM shared_bundles ORDER BY shared_at DESC"
        )?;
        let bundles = stmt.query_map([], |row| {
            let checkpoint_ids: Vec<CheckpointId> =
                serde_json::from_str(&row.get::<_, String>(1)?).unwrap_or_default();
            Ok(SharedCheckpointBundle {
                id: row.get(0)?,
                checkpoint_ids,
                shared_by: CollaboratorInfo {
                    user_id: row.get(2)?,
                    display_name: row.get(3)?,
                    machine_id: row.get(4)?,
                    last_seen: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                        .map(|d| d.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                },
                shared_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                description: row.get(6)?,
                format_version: row.get::<_, i64>(7)? as u32,
                data: Vec::new(), // Don't load data in list view
            })
        })?;
        bundles.collect::<std::result::Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Insert a compliance audit record
    pub fn insert_audit_record(&self, record: &AuditRecord) -> Result<()> {
        let conn = self.connection.lock();
        conn.execute(
            r#"
            INSERT INTO compliance_audit (
                id, timestamp, user_id, machine_id, action, resource_type, resource_id, details, outcome
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                record.id,
                record.timestamp.to_rfc3339(),
                record.user_id,
                record.machine_id,
                record.action,
                record.resource_type,
                record.resource_id,
                serde_json::to_string(&record.details)?,
                serde_json::to_string(&record.outcome)?,
            ],
        )?;
        Ok(())
    }

    /// Get audit trail records
    pub fn get_audit_trail(
        &self,
        limit: usize,
        action_filter: Option<&str>,
    ) -> Result<Vec<AuditRecord>> {
        let conn = self.connection.lock();
        let query = if action_filter.is_some() {
            "SELECT id, timestamp, user_id, machine_id, action, resource_type, resource_id, details, outcome \
             FROM compliance_audit WHERE action = ?1 ORDER BY timestamp DESC LIMIT ?2"
        } else {
            "SELECT id, timestamp, user_id, machine_id, action, resource_type, resource_id, details, outcome \
             FROM compliance_audit ORDER BY timestamp DESC LIMIT ?1"
        };

        let records = if let Some(filter) = action_filter {
            let mut stmt = conn.prepare(query)?;
            let rows = stmt.query_map(params![filter, limit as i64], Self::map_audit_row)?;
            rows.collect::<std::result::Result<Vec<_>, _>>()?
        } else {
            let mut stmt = conn.prepare(query)?;
            let rows = stmt.query_map(params![limit as i64], Self::map_audit_row)?;
            rows.collect::<std::result::Result<Vec<_>, _>>()?
        };

        Ok(records)
    }

    /// Get audit trail for a specific resource
    pub fn get_resource_audit_trail(
        &self,
        resource_type: &str,
        resource_id: &str,
    ) -> Result<Vec<AuditRecord>> {
        let conn = self.connection.lock();
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, user_id, machine_id, action, resource_type, resource_id, details, outcome \
             FROM compliance_audit WHERE resource_type = ?1 AND resource_id = ?2 ORDER BY timestamp DESC"
        )?;
        let records = stmt
            .query_map(params![resource_type, resource_id], Self::map_audit_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(records)
    }

    /// Map a DB row to an AuditRecord
    fn map_audit_row(row: &rusqlite::Row) -> rusqlite::Result<AuditRecord> {
        Ok(AuditRecord {
            id: row.get(0)?,
            timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(1)?)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            user_id: row.get(2)?,
            machine_id: row.get(3)?,
            action: row.get(4)?,
            resource_type: row.get(5)?,
            resource_id: row.get(6)?,
            details: serde_json::from_str(&row.get::<_, String>(7)?).unwrap_or_default(),
            outcome: serde_json::from_str(&row.get::<_, String>(8)?)
                .unwrap_or(AuditOutcome::Success),
        })
    }

    /// Get sync status
    pub fn get_sync_status(&self) -> Result<SyncStatus> {
        let conn = self.connection.lock();
        let result = conn.query_row(
            "SELECT status, last_sync_at, last_error FROM sync_status WHERE id = 1",
            [],
            |row| {
                let status_str: String = row.get(0)?;
                let last_sync_at: Option<String> = row.get(1)?;
                let last_error: Option<String> = row.get(2)?;

                Ok(match status_str.as_str() {
                    "InProgress" => SyncStatus::InProgress,
                    "Succeeded" => SyncStatus::Succeeded {
                        at: last_sync_at
                            .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                            .map(|d| d.with_timezone(&Utc))
                            .unwrap_or_else(Utc::now),
                    },
                    "Failed" => SyncStatus::Failed {
                        at: last_sync_at
                            .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                            .map(|d| d.with_timezone(&Utc))
                            .unwrap_or_else(Utc::now),
                        error: last_error.unwrap_or_default(),
                    },
                    _ => SyncStatus::Idle,
                })
            },
        ).optional()?;

        Ok(result.unwrap_or(SyncStatus::Idle))
    }

    /// Update the persisted sync status.
    pub fn set_sync_status(&self, status: &SyncStatus) -> Result<()> {
        let conn = self.connection.lock();

        let (status_name, last_sync_at, last_error) = match status {
            SyncStatus::Idle => ("Idle", None, None),
            SyncStatus::InProgress => ("InProgress", None, None),
            SyncStatus::Succeeded { at } => ("Succeeded", Some(at.to_rfc3339()), None),
            SyncStatus::Failed { at, error } => {
                ("Failed", Some(at.to_rfc3339()), Some(error.clone()))
            }
        };

        conn.execute(
            "UPDATE sync_status SET status = ?1, last_sync_at = ?2, last_error = ?3 WHERE id = 1",
            params![status_name, last_sync_at, last_error],
        )?;

        Ok(())
    }

    /// List checkpoints created since a given time
    pub fn list_checkpoints_since(&self, since: DateTime<Utc>) -> Result<Vec<Checkpoint>> {
        let conn = self.connection.lock();
        let mut stmt = conn.prepare(
            "SELECT id, session_id, description, created_at, files_affected, size_bytes, tags, metadata \
             FROM checkpoints WHERE created_at >= ?1 ORDER BY created_at ASC"
        )?;
        let checkpoints = stmt.query_map(params![since.to_rfc3339()], |row| {
            let tags: Vec<String> = serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default();
            let metadata: HashMap<String, String> = serde_json::from_str(&row.get::<_, String>(7)?).unwrap_or_default();
            Ok(Checkpoint {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                session_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap_or_default(),
                description: row.get(2)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                file_changes: Vec::new(),
                file_inventory: Vec::new(),
                files_affected: row.get::<_, i64>(4)? as usize,
                size_bytes: row.get::<_, i64>(5)? as u64,
                tags,
                metadata,
                parent_checkpoint_id: None,
                is_full_snapshot: true,
                delta_depth: 0,
                branch_id: None,
            })
        })?;
        checkpoints.collect::<std::result::Result<Vec<_>, _>>().map_err(Into::into)
    }

    // ========================================
    // Phase 8.5: Performance Monitoring DB
    // ========================================

    /// Insert a storage usage snapshot
    pub fn insert_storage_snapshot(&self, snapshot: &StorageUsageSnapshot) -> Result<()> {
        let conn = self.connection.lock();
        conn.execute(
            r#"
            INSERT INTO storage_snapshots (
                timestamp, total_bytes, checkpoint_data_bytes, database_bytes, blob_count, checkpoint_count
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                snapshot.timestamp.to_rfc3339(),
                snapshot.total_bytes as i64,
                snapshot.checkpoint_data_bytes as i64,
                snapshot.database_bytes as i64,
                snapshot.blob_count as i64,
                snapshot.checkpoint_count as i64,
            ],
        )?;
        Ok(())
    }

    /// Get storage snapshots since a given time
    pub fn get_storage_snapshots(&self, since: DateTime<Utc>) -> Result<Vec<StorageUsageSnapshot>> {
        let conn = self.connection.lock();
        let mut stmt = conn.prepare(
            "SELECT timestamp, total_bytes, checkpoint_data_bytes, database_bytes, blob_count, checkpoint_count \
             FROM storage_snapshots WHERE timestamp >= ?1 ORDER BY timestamp ASC"
        )?;
        let snapshots = stmt.query_map(params![since.to_rfc3339()], |row| {
            Ok(StorageUsageSnapshot {
                timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(0)?)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                total_bytes: row.get::<_, i64>(1)? as u64,
                checkpoint_data_bytes: row.get::<_, i64>(2)? as u64,
                database_bytes: row.get::<_, i64>(3)? as u64,
                blob_count: row.get::<_, i64>(4)? as u64,
                checkpoint_count: row.get::<_, i64>(5)? as u64,
            })
        })?;
        snapshots.collect::<std::result::Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Get checkpoint creation frequency grouped by day
    pub fn get_creation_frequency(&self, days: u32) -> Result<Vec<CreationFrequencyPoint>> {
        let conn = self.connection.lock();
        let since = Utc::now() - chrono::Duration::days(days as i64);

        let mut stmt = conn.prepare(
            "SELECT date(created_at) as day, COUNT(*) as cnt, tags \
             FROM checkpoints WHERE created_at >= ?1 \
             GROUP BY date(created_at) ORDER BY day ASC"
        )?;

        let rows = stmt.query_map(params![since.to_rfc3339()], |row| {
            let day_str: String = row.get(0)?;
            let count: i64 = row.get(1)?;
            let _tags: String = row.get(2)?; // Not useful in GROUP BY
            Ok((day_str, count as u64))
        })?;

        let mut points = Vec::new();
        for row in rows {
            let (day_str, count) = row?;
            let bucket = chrono::NaiveDate::parse_from_str(&day_str, "%Y-%m-%d")
                .ok()
                .and_then(|d| d.and_hms_opt(0, 0, 0))
                .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
                .unwrap_or_else(Utc::now);

            points.push(CreationFrequencyPoint {
                bucket,
                count,
                by_type: HashMap::new(),
            });
        }

        Ok(points)
    }

    /// Insert a restoration event
    pub fn insert_restoration_event(&self, event: &RestorationEvent) -> Result<()> {
        let conn = self.connection.lock();
        conn.execute(
            r#"
            INSERT INTO restoration_events (
                timestamp, checkpoint_id, success, duration_ms, files_restored, files_failed, error
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                event.timestamp.to_rfc3339(),
                event.checkpoint_id.to_string(),
                event.success as i64,
                event.duration_ms,
                event.files_restored as i64,
                event.files_failed as i64,
                event.error,
            ],
        )?;
        Ok(())
    }

    /// Get restoration events
    pub fn get_restoration_events(&self, limit: usize) -> Result<Vec<RestorationEvent>> {
        let conn = self.connection.lock();
        let mut stmt = conn.prepare(
            "SELECT timestamp, checkpoint_id, success, duration_ms, files_restored, files_failed, error \
             FROM restoration_events ORDER BY timestamp DESC LIMIT ?1"
        )?;
        let events = stmt.query_map(params![limit as i64], |row| {
            Ok(RestorationEvent {
                timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(0)?)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                checkpoint_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap_or_default(),
                success: row.get::<_, i64>(2)? != 0,
                duration_ms: row.get(3)?,
                files_restored: row.get::<_, i64>(4)? as u64,
                files_failed: row.get::<_, i64>(5)? as u64,
                error: row.get(6)?,
            })
        })?;
        events.collect::<std::result::Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Insert AI session metrics
    pub fn insert_ai_session_metrics(&self, metrics: &AISessionMetrics) -> Result<()> {
        let conn = self.connection.lock();
        conn.execute(
            r#"
            INSERT INTO ai_session_metrics (
                session_id, started_at, ended_at, files_changed, lines_added, lines_deleted,
                checkpoints_created, rollbacks, duration_seconds
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                metrics.session_id.to_string(),
                metrics.started_at.to_rfc3339(),
                metrics.ended_at.map(|d| d.to_rfc3339()),
                metrics.files_changed as i64,
                metrics.lines_added as i64,
                metrics.lines_deleted as i64,
                metrics.checkpoints_created as i64,
                metrics.rollbacks as i64,
                metrics.duration_seconds,
            ],
        )?;
        Ok(())
    }

    /// Get AI session metrics
    pub fn get_ai_session_metrics(&self, limit: usize) -> Result<Vec<AISessionMetrics>> {
        let conn = self.connection.lock();
        let mut stmt = conn.prepare(
            "SELECT session_id, started_at, ended_at, files_changed, lines_added, lines_deleted, \
             checkpoints_created, rollbacks, duration_seconds \
             FROM ai_session_metrics ORDER BY started_at DESC LIMIT ?1"
        )?;
        let metrics = stmt.query_map(params![limit as i64], |row| {
            Ok(AISessionMetrics {
                session_id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                started_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(1)?)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                ended_at: row.get::<_, Option<String>>(2)?
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|d| d.with_timezone(&Utc)),
                files_changed: row.get::<_, i64>(3)? as u64,
                lines_added: row.get::<_, i64>(4)? as u64,
                lines_deleted: row.get::<_, i64>(5)? as u64,
                checkpoints_created: row.get::<_, i64>(6)? as u64,
                rollbacks: row.get::<_, i64>(7)? as u64,
                duration_seconds: row.get(8)?,
            })
        })?;
        metrics.collect::<std::result::Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Get performance metrics for a specific operation type
    pub fn get_performance_metrics(
        &self,
        operation: &str,
        limit: usize,
    ) -> Result<Vec<(String, f64)>> {
        let conn = self.connection.lock();
        let mut stmt = conn.prepare(
            "SELECT timestamp, duration_ms FROM performance_metrics \
             WHERE operation = ?1 ORDER BY timestamp DESC LIMIT ?2"
        )?;
        let metrics = stmt.query_map(params![operation, limit as i64], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
        })?;
        metrics.collect::<std::result::Result<Vec<_>, _>>().map_err(Into::into)
    }
}
