use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use log::{debug, info};
use rusqlite::{Connection, Result as SqliteResult, params};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::session::{ChatMessage, ChatSession, Source};

/// Row type returned by `list_rag_documents`: (id, filename, file_size, created_at, updated_at)
pub type RagDocumentRow = (i64, String, i64, i64, i64);

/// Database manager for SQLite operations
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Create a new database connection
    /// If the database file doesn't exist, it will be created
    pub fn new<P: AsRef<Path>>(path: P) -> SqliteResult<Self> {
        // Register sqlite-vec extension once at startup
        Self::register_vec_extension();

        let conn = Connection::open(path)?;

        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON", [])?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        // Run migrations
        db.migrate()?;

        Ok(db)
    }

    /// Register the sqlite-vec extension using sqlite3_auto_extension
    /// This only needs to be called once, and all future connections will have it
    fn register_vec_extension() {
        use rusqlite::ffi::sqlite3_auto_extension;
        use sqlite_vec::sqlite3_vec_init;

        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
        }
        info!("Registered sqlite-vec extension");
    }

    /// Run database migrations
    fn migrate(&self) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();

        // Create migrations tracking table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at INTEGER NOT NULL
            )",
            [],
        )?;

        // Helper function to check if migration was applied
        let migration_applied = |version: i32| -> SqliteResult<bool> {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM schema_migrations WHERE version = ?1",
                [version],
                |row| row.get(0),
            )?;
            Ok(count > 0)
        };

        // Helper function to mark migration as applied
        let mark_migration_applied = |version: i32| -> SqliteResult<()> {
            conn.execute(
                "INSERT OR IGNORE INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
                [version, chrono::Utc::now().timestamp() as i32],
            )?;
            Ok(())
        };

        // Helper function to run migration with error handling for duplicate columns
        let run_migration = |version: i32, name: &str, sql: &str| -> SqliteResult<()> {
            if !migration_applied(version)? {
                debug!("Running migration {}: {}", version, name);
                match conn.execute_batch(sql) {
                    Ok(_) => {
                        mark_migration_applied(version)?;
                        Ok(())
                    }
                    Err(e) => {
                        // If error is about duplicate column, mark as applied (already exists)
                        let err_msg = e.to_string();
                        if err_msg.contains("duplicate column name") {
                            debug!(
                                "Migration {} already partially applied (duplicate column), marking as complete",
                                version
                            );
                            mark_migration_applied(version)?;
                            Ok(())
                        } else {
                            Err(e)
                        }
                    }
                }
            } else {
                debug!("Skipping migration {} (already applied)", version);
                Ok(())
            }
        };

        // Migration 001: Initial schema
        run_migration(
            1,
            "Initial schema",
            include_str!("../migrations/001_initial_schema.sql"),
        )?;

        // Migration 002: Logs table
        run_migration(
            2,
            "Logs table",
            include_str!("../migrations/002_logs_table.sql"),
        )?;

        // Migration 003: Session titles
        run_migration(
            3,
            "Session titles",
            include_str!("../migrations/003_session_titles.sql"),
        )?;

        // Migration 004: Token tracking
        run_migration(
            4,
            "Token tracking",
            include_str!("../migrations/004_token_tracking.sql"),
        )?;

        // Migration 005: Context window
        run_migration(
            5,
            "Context window",
            include_str!("../migrations/005_context_window.sql"),
        )?;

        // Migration 006: Deduplicate sources
        run_migration(
            6,
            "Deduplicate sources",
            include_str!("../migrations/006_deduplicate_sources.sql"),
        )?;

        // Migration 007: Reasoning column
        run_migration(
            7,
            "Reasoning column",
            include_str!("../migrations/007_reasoning_column.sql"),
        )?;

        // Migration 008: Tool invocations
        run_migration(
            8,
            "Tool invocations",
            include_str!("../migrations/008_tool_invocations.sql"),
        )?;

        // Migration 009: Thinking steps
        run_migration(
            9,
            "Thinking steps",
            include_str!("../migrations/009_thinking_steps.sql"),
        )?;

        // Migration 010: Content split markers
        run_migration(
            10,
            "Content split markers",
            include_str!("../migrations/010_content_split_markers.sql"),
        )?;

        // Migration 011: RAG vectors
        run_migration(
            11,
            "RAG vectors",
            include_str!("../migrations/011_rag_vectors.sql"),
        )?;

        // Migration 012: Rename model_id to agent_id
        run_migration(
            12,
            "Rename model_id to agent_id",
            include_str!("../migrations/012_rename_model_to_agent.sql"),
        )?;

        // Migration 013: Agent token stats
        run_migration(
            13,
            "Agent token stats",
            include_str!("../migrations/013_agent_token_stats.sql"),
        )?;

        // Migration 014: Background jobs
        run_migration(
            14,
            "Background jobs",
            include_str!("../migrations/014_background_jobs.sql"),
        )?;

        // Migration 015: Add job timeout
        run_migration(
            15,
            "Job timeout support",
            include_str!("../migrations/015_job_timeout.sql"),
        )?;

        // Migration 016: Job execution history
        run_migration(
            16,
            "Job execution history",
            include_str!("../migrations/016_job_executions.sql"),
        )?;

        info!("Database migrations completed successfully");
        Ok(())
    }

    /// Save a session to the database
    pub fn save_session(&self, session: &ChatSession) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();

        // Try to update existing session first
        let updated = conn.execute(
            "UPDATE sessions SET created_at = ?2, updated_at = ?3, metadata = ?4, title = ?5, agent_id = ?6, total_tokens = ?7, input_tokens = ?8, output_tokens = ?9, reasoning_tokens = ?10, cache_tokens = ?11, cost_usd = ?12, context_window = ?13 WHERE id = ?1",
            params![
                session.id,
                session.created_at,
                session.updated_at,
                Option::<String>::None,
                session.title.as_ref(),
                session.agent_id.as_ref(),
                session.token_usage.total_tokens,
                session.token_usage.input_tokens,
                session.token_usage.output_tokens,
                session.token_usage.reasoning_tokens,
                session.token_usage.cache_tokens,
                session.cost_usd,
                session.token_usage.context_window,
            ],
        )?;

        // If no rows were updated, insert new session
        if updated == 0 {
            conn.execute(
                "INSERT INTO sessions (id, created_at, updated_at, metadata, title, agent_id, total_tokens, input_tokens, output_tokens, reasoning_tokens, cache_tokens, cost_usd, context_window) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![
                    session.id,
                    session.created_at,
                    session.updated_at,
                    Option::<String>::None,
                    session.title.as_ref(),
                    session.agent_id.as_ref(),
                    session.token_usage.total_tokens,
                    session.token_usage.input_tokens,
                    session.token_usage.output_tokens,
                    session.token_usage.reasoning_tokens,
                    session.token_usage.cache_tokens,
                    session.cost_usd,
                    session.token_usage.context_window,
                ],
            )?;
        }

        Ok(())
    }

    /// Load a session from the database
    pub fn load_session(&self, session_id: &str) -> SqliteResult<Option<ChatSession>> {
        let conn = self.conn.lock().unwrap();

        // Load session metadata
        let mut stmt = conn.prepare("SELECT id, created_at, updated_at, title, agent_id, total_tokens, input_tokens, output_tokens, reasoning_tokens, cache_tokens, cost_usd, context_window FROM sessions WHERE id = ?1")?;
        let session_result = stmt.query_row(params![session_id], |row| {
            Ok(ChatSession {
                id: row.get(0)?,
                messages: Vec::new(), // Will be populated below
                created_at: row.get(1)?,
                updated_at: row.get(2)?,
                title: row.get(3)?,
                agent_id: row.get(4)?,
                token_usage: crate::session::TokenUsage {
                    total_tokens: row.get(5)?,
                    input_tokens: row.get(6)?,
                    output_tokens: row.get(7)?,
                    reasoning_tokens: row.get(8)?,
                    cache_tokens: row.get(9)?,
                    context_window: row.get(11)?,
                    context_utilization: 0.0, // Will be calculated
                },
                cost_usd: row.get(10)?,
            })
        });

        let mut session = match session_result {
            Ok(s) => s,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(e) => return Err(e),
        };

        // Load messages
        let mut msg_stmt = conn.prepare(
            "SELECT id, role, content, timestamp FROM messages WHERE session_id = ?1 ORDER BY timestamp ASC"
        )?;

        let messages = msg_stmt
            .query_map(params![session_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            })?
            .collect::<SqliteResult<Vec<(i64, String, String, i64)>>>()?;

        // Convert to ChatMessages and load sources for each
        let messages: Vec<ChatMessage> = messages.into_iter().map(|(message_id, role, content, timestamp)| {
            // Load sources for this message (support both old and new schema)
            let mut source_stmt = conn.prepare(
                "SELECT s.title, s.content, s.content_id, fc.content_compressed
                 FROM sources s
                 LEFT JOIN file_contents fc ON s.content_id = fc.id
                 WHERE s.message_id = ?1"
            )?;

            let sources = source_stmt.query_map(params![message_id], |row| {
                let title: String = row.get(0)?;

                // Try to get content from new schema first (compressed)
                let content = if let Ok(Some(compressed_data)) = row.get::<_, Option<Vec<u8>>>(3) {
                    // Decompress content
                    let mut decoder = GzDecoder::new(&compressed_data[..]);
                    let mut decompressed = String::new();
                    decoder.read_to_string(&mut decompressed).map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            3,
                            rusqlite::types::Type::Blob,
                            Box::new(e)
                        )
                    })?;
                    decompressed
                } else if let Ok(Some(old_content)) = row.get::<_, Option<String>>(1) {
                    // Fall back to old schema (uncompressed, might be NULL)
                    old_content
                } else {
                    // Should not happen, but handle gracefully
                    String::new()
                };

                Ok(Source {
                    title,
                    content,
                })
            })?.collect::<SqliteResult<Vec<Source>>>()?;

            // Load thinking steps for this message
            let mut steps_stmt = conn.prepare(
                "SELECT step_order, step_type, content, tool_name, tool_arguments, tool_result, tool_error, content_before_tool
                 FROM thinking_steps
                 WHERE message_id = ?1
                 ORDER BY step_order ASC"
            )?;

            let thinking_steps = steps_stmt.query_map(params![message_id], |row| {
                let tool_args_json: Option<String> = row.get(4)?;
                let tool_arguments = tool_args_json.and_then(|json| serde_json::from_str(&json).ok());

                Ok(crate::session::ThinkingStep {
                    step_order: row.get(0)?,
                    step_type: row.get(1)?,
                    content: row.get(2)?,
                    tool_name: row.get(3)?,
                    tool_arguments,
                    tool_result: row.get(5)?,
                    tool_error: row.get(6)?,
                    content_before_tool: row.get(7)?,
                })
            })?.collect::<SqliteResult<Vec<crate::session::ThinkingStep>>>()?;

            // Filter out thinking steps with no meaningful content
            // (empty reasoning steps, tool steps without tool_name, etc.)
            let thinking_steps: Vec<_> = thinking_steps.into_iter().filter(|step| {
                // Reasoning steps must have non-empty content
                if step.step_type == "reasoning" {
                    step.content.as_ref().map_or(false, |c| !c.trim().is_empty())
                } else if step.step_type == "tool" {
                    // Tool steps must have a tool_name
                    step.tool_name.is_some()
                } else {
                    // Keep other step types
                    true
                }
            }).collect();

            let thinking_steps = if thinking_steps.is_empty() {
                None
            } else {
                Some(thinking_steps)
            };

            Ok(ChatMessage {
                role,
                content,
                sources,
                timestamp,
                thinking_steps,
            })
        }).collect::<SqliteResult<Vec<ChatMessage>>>()?;

        session.messages = messages;

        Ok(Some(session))
    }

    /// Save a message to the database
    pub fn save_message(&self, session_id: &str, message: &ChatMessage) -> SqliteResult<i64> {
        let conn = self.conn.lock().unwrap();

        // Insert message
        conn.execute(
            "INSERT INTO messages (session_id, role, content, timestamp) VALUES (?1, ?2, ?3, ?4)",
            params![session_id, message.role, message.content, message.timestamp],
        )?;

        let message_id = conn.last_insert_rowid();

        // Insert sources with deduplication and compression
        for source in &message.sources {
            // Check file size limit (10MB)
            const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;
            if source.content.len() > MAX_FILE_SIZE {
                continue;
            }

            // Calculate hash of content
            let mut hasher = Sha256::new();
            hasher.update(source.content.as_bytes());

            let digest = hasher.finalize();
            let hash: String = digest.iter().map(|b| format!("{:02x}", b)).collect();

            // Check if content already exists
            let content_id: Option<i64> = conn
                .query_row(
                    "SELECT id FROM file_contents WHERE content_hash = ?1",
                    params![hash],
                    |row| row.get(0),
                )
                .ok();

            let content_id = if let Some(id) = content_id {
                // Content already exists, reuse it
                id
            } else {
                // Compress content
                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder
                    .write_all(source.content.as_bytes())
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
                let compressed = encoder
                    .finish()
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

                let original_size = source.content.len() as i64;
                let compressed_size = compressed.len() as i64;

                // Insert new content
                conn.execute(
                    "INSERT INTO file_contents (content_hash, content_compressed, original_size, compressed_size, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![hash, compressed, original_size, compressed_size, chrono::Utc::now().timestamp()],
                )?;

                conn.last_insert_rowid()
            };

            // Insert source reference
            conn.execute(
                "INSERT INTO sources (message_id, title, content_id) VALUES (?1, ?2, ?3)",
                params![message_id, source.title, content_id],
            )?;
        }

        // Update session's updated_at timestamp
        conn.execute(
            "UPDATE sessions SET updated_at = ?1 WHERE id = ?2",
            params![message.timestamp, session_id],
        )?;

        // Save thinking steps if present
        if let Some(thinking_steps) = &message.thinking_steps {
            for step in thinking_steps {
                let tool_args_json = step
                    .tool_arguments
                    .as_ref()
                    .map(|args| serde_json::to_string(args).unwrap_or_default());

                conn.execute(
                    "INSERT INTO thinking_steps (message_id, step_order, step_type, content, tool_name, tool_arguments, tool_result, tool_error, content_before_tool, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    params![
                        message_id,
                        step.step_order,
                        step.step_type,
                        step.content,
                        step.tool_name,
                        tool_args_json,
                        step.tool_result,
                        step.tool_error,
                        step.content_before_tool,
                        chrono::Utc::now().timestamp(),
                    ],
                )?;
            }
        }

        Ok(message_id)
    }

    /// Delete a session and all its messages
    pub fn delete_session(&self, session_id: &str) -> SqliteResult<bool> {
        let conn = self.conn.lock().unwrap();

        let deleted = conn.execute("DELETE FROM sessions WHERE id = ?1", params![session_id])?;

        Ok(deleted > 0)
    }

    /// List all session IDs, ordered by updated_at (most recent first)
    pub fn list_sessions(&self) -> SqliteResult<Vec<String>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare("SELECT id FROM sessions ORDER BY updated_at DESC")?;

        let sessions = stmt
            .query_map([], |row| row.get(0))?
            .collect::<SqliteResult<Vec<String>>>()?;

        Ok(sessions)
    }

    /// Update session title
    pub fn update_session_title(&self, session_id: &str, title: &str) -> SqliteResult<bool> {
        let conn = self.conn.lock().unwrap();

        let updated = conn.execute(
            "UPDATE sessions SET title = ?1 WHERE id = ?2",
            params![title, session_id],
        )?;

        Ok(updated > 0)
    }

    /// Delete sessions older than the specified number of seconds
    pub fn cleanup_old_sessions(&self, max_age_seconds: i64) -> SqliteResult<usize> {
        let conn = self.conn.lock().unwrap();

        let cutoff_time = chrono::Utc::now().timestamp() - max_age_seconds;

        let deleted = conn.execute(
            "DELETE FROM sessions WHERE updated_at < ?1",
            params![cutoff_time],
        )?;

        if deleted > 0 {
            info!("Cleaned up {} old session(s)", deleted);
        }

        Ok(deleted)
    }

    /// Get the number of messages in a session
    pub fn get_message_count(&self, session_id: &str) -> SqliteResult<usize> {
        let conn = self.conn.lock().unwrap();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM messages WHERE session_id = ?1",
            params![session_id],
            |row| row.get(0),
        )?;

        Ok(count as usize)
    }

    /// Check if a session exists
    pub fn session_exists(&self, session_id: &str) -> SqliteResult<bool> {
        let conn = self.conn.lock().unwrap();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sessions WHERE id = ?1",
            params![session_id],
            |row| row.get(0),
        )?;

        Ok(count > 0)
    }

    // RAG (Retrieval-Augmented Generation) helper methods

    /// Insert or update a RAG document
    pub fn upsert_rag_document(
        &self,
        filename: &str,
        content: &str,
        content_hash: &str,
        file_size: i64,
    ) -> SqliteResult<i64> {
        let conn = self.conn.lock().unwrap();

        let now = chrono::Utc::now().timestamp();

        // Try to update existing document first
        let updated = conn.execute(
            "UPDATE rag_documents SET content = ?1, content_hash = ?2, file_size = ?3, updated_at = ?4 WHERE filename = ?5",
            params![content, content_hash, file_size, now, filename],
        )?;

        if updated == 0 {
            // Insert new document
            conn.execute(
                "INSERT INTO rag_documents (filename, content, content_hash, file_size, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![filename, content, content_hash, file_size, now, now],
            )?;
            Ok(conn.last_insert_rowid())
        } else {
            // Get existing document ID
            let doc_id: i64 = conn.query_row(
                "SELECT id FROM rag_documents WHERE filename = ?1",
                params![filename],
                |row| row.get(0),
            )?;
            Ok(doc_id)
        }
    }

    /// Insert a document chunk
    pub fn insert_rag_chunk(
        &self,
        document_id: i64,
        chunk_index: i32,
        chunk_text: &str,
        chunk_tokens: i32,
    ) -> SqliteResult<i64> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT INTO rag_chunks (document_id, chunk_index, chunk_text, chunk_tokens)
             VALUES (?1, ?2, ?3, ?4)",
            params![document_id, chunk_index, chunk_text, chunk_tokens],
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// Insert a vector embedding for a chunk
    /// Note: Uses raw SQL as vec0 virtual table has specific syntax
    pub fn insert_rag_embedding(&self, chunk_id: i64, embedding: &[f32]) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();

        // Convert embedding to format expected by vec0
        let embedding_json = serde_json::to_string(embedding)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        conn.execute(
            "INSERT INTO rag_embeddings (chunk_id, embedding) VALUES (?1, ?2)",
            params![chunk_id, embedding_json],
        )?;

        Ok(())
    }

    /// Delete all chunks and embeddings for a document
    pub fn delete_rag_document_chunks(&self, document_id: i64) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();

        // Get all chunk IDs for this document
        let chunk_ids: Vec<i64> = {
            let mut stmt = conn.prepare("SELECT id FROM rag_chunks WHERE document_id = ?1")?;
            stmt.query_map(params![document_id], |row| row.get(0))?
                .collect::<SqliteResult<Vec<i64>>>()?
        };

        // Delete embeddings for these chunks
        for chunk_id in chunk_ids {
            conn.execute(
                "DELETE FROM rag_embeddings WHERE chunk_id = ?1",
                params![chunk_id],
            )?;
        }

        // Delete chunks
        conn.execute(
            "DELETE FROM rag_chunks WHERE document_id = ?1",
            params![document_id],
        )?;

        Ok(())
    }

    /// Delete a RAG document and all its chunks/embeddings
    pub fn delete_rag_document(&self, document_id: i64) -> SqliteResult<bool> {
        let conn = self.conn.lock().unwrap();

        // Chunks will be deleted by CASCADE, but embeddings need manual deletion
        // Get all chunk IDs first
        let chunk_ids: Vec<i64> = {
            let mut stmt = conn.prepare("SELECT id FROM rag_chunks WHERE document_id = ?1")?;
            stmt.query_map(params![document_id], |row| row.get(0))?
                .collect::<SqliteResult<Vec<i64>>>()?
        };

        // Delete embeddings
        for chunk_id in chunk_ids {
            conn.execute(
                "DELETE FROM rag_embeddings WHERE chunk_id = ?1",
                params![chunk_id],
            )?;
        }

        // Delete document (will CASCADE delete chunks)
        let deleted = conn.execute(
            "DELETE FROM rag_documents WHERE id = ?1",
            params![document_id],
        )?;

        Ok(deleted > 0)
    }

    /// Get RAG document by filename
    pub fn get_rag_document_by_filename(
        &self,
        filename: &str,
    ) -> SqliteResult<Option<(i64, String, i64)>> {
        let conn = self.conn.lock().unwrap();

        match conn.query_row(
            "SELECT id, content_hash, updated_at FROM rag_documents WHERE filename = ?1",
            params![filename],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ) {
            Ok(result) => Ok(Some(result)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// List all RAG documents
    pub fn list_rag_documents(&self) -> SqliteResult<Vec<RagDocumentRow>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, filename, file_size, created_at, updated_at FROM rag_documents ORDER BY filename"
        )?;

        let docs = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            })?
            .collect::<SqliteResult<Vec<_>>>()?;

        Ok(docs)
    }

    /// Get RAG statistics
    pub fn get_rag_stats(&self) -> SqliteResult<(i64, i64, i64)> {
        let conn = self.conn.lock().unwrap();

        let doc_count: i64 =
            conn.query_row("SELECT COUNT(*) FROM rag_documents", [], |row| row.get(0))?;
        let chunk_count: i64 =
            conn.query_row("SELECT COUNT(*) FROM rag_chunks", [], |row| row.get(0))?;
        let embedding_count: i64 =
            conn.query_row("SELECT COUNT(*) FROM rag_embeddings", [], |row| row.get(0))?;

        Ok((doc_count, chunk_count, embedding_count))
    }

    /// Query similar chunks using vector similarity
    /// Returns (chunk_id, chunk_text, filename, distance)
    pub fn query_similar_chunks(
        &self,
        query_embedding: &[f32],
        limit: i32,
    ) -> SqliteResult<Vec<(i64, String, String, f32)>> {
        let conn = self.conn.lock().unwrap();

        // Convert embedding to JSON format
        let embedding_json = serde_json::to_string(query_embedding)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        // Query using vec0 distance function
        let mut stmt = conn.prepare(
            "SELECT c.id, c.chunk_text, d.filename, vec_distance_L2(e.embedding, ?1) as distance
             FROM rag_embeddings e
             JOIN rag_chunks c ON e.chunk_id = c.id
             JOIN rag_documents d ON c.document_id = d.id
             ORDER BY distance
             LIMIT ?2",
        )?;

        let results = stmt
            .query_map(params![embedding_json, limit], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })?
            .collect::<SqliteResult<Vec<_>>>()?;

        Ok(results)
    }

    // Agent token stats helper methods

    /// Update or insert agent token stats by accumulating values
    pub fn update_agent_token_stats(
        &self,
        agent_id: &str,
        input_tokens: i64,
        output_tokens: i64,
        reasoning_tokens: i64,
        cache_tokens: i64,
        cost_usd: f64,
    ) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();

        // Try to update existing stats
        let updated = conn.execute(
            "UPDATE agent_token_stats 
             SET total_sessions = total_sessions + 1,
                 total_tokens = total_tokens + ?1 + ?2 + ?3 + ?4,
                 input_tokens = input_tokens + ?1,
                 output_tokens = output_tokens + ?2,
                 reasoning_tokens = reasoning_tokens + ?3,
                 cache_tokens = cache_tokens + ?4,
                 total_cost_usd = total_cost_usd + ?5,
                 last_used_at = ?6
             WHERE agent_id = ?7",
            params![
                input_tokens,
                output_tokens,
                reasoning_tokens,
                cache_tokens,
                cost_usd,
                now,
                agent_id
            ],
        )?;

        // If no rows updated, insert new stats
        if updated == 0 {
            let total_tokens = input_tokens + output_tokens + reasoning_tokens + cache_tokens;
            conn.execute(
                "INSERT INTO agent_token_stats 
                 (agent_id, total_sessions, total_tokens, input_tokens, output_tokens, 
                  reasoning_tokens, cache_tokens, total_cost_usd, first_used_at, last_used_at)
                 VALUES (?1, 1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    agent_id,
                    total_tokens,
                    input_tokens,
                    output_tokens,
                    reasoning_tokens,
                    cache_tokens,
                    cost_usd,
                    now,
                    now
                ],
            )?;
        }

        Ok(())
    }

    /// Get token stats for a specific agent
    pub fn get_agent_token_stats(
        &self,
        agent_id: &str,
    ) -> SqliteResult<Option<AgentTokenStatsRow>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT agent_id, total_sessions, total_tokens, input_tokens, output_tokens, 
                    reasoning_tokens, cache_tokens, total_cost_usd, first_used_at, last_used_at
             FROM agent_token_stats 
             WHERE agent_id = ?1",
        )?;

        let result = stmt.query_row(params![agent_id], |row| {
            Ok(AgentTokenStatsRow {
                agent_id: row.get(0)?,
                total_sessions: row.get(1)?,
                total_tokens: row.get(2)?,
                input_tokens: row.get(3)?,
                output_tokens: row.get(4)?,
                reasoning_tokens: row.get(5)?,
                cache_tokens: row.get(6)?,
                total_cost_usd: row.get(7)?,
                first_used_at: row.get(8)?,
                last_used_at: row.get(9)?,
            })
        });

        match result {
            Ok(stats) => Ok(Some(stats)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Get token stats for all agents
    pub fn get_all_agent_token_stats(&self) -> SqliteResult<Vec<AgentTokenStatsRow>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT agent_id, total_sessions, total_tokens, input_tokens, output_tokens, 
                    reasoning_tokens, cache_tokens, total_cost_usd, first_used_at, last_used_at
             FROM agent_token_stats 
             ORDER BY total_tokens DESC",
        )?;

        let stats = stmt
            .query_map([], |row| {
                Ok(AgentTokenStatsRow {
                    agent_id: row.get(0)?,
                    total_sessions: row.get(1)?,
                    total_tokens: row.get(2)?,
                    input_tokens: row.get(3)?,
                    output_tokens: row.get(4)?,
                    reasoning_tokens: row.get(5)?,
                    cache_tokens: row.get(6)?,
                    total_cost_usd: row.get(7)?,
                    first_used_at: row.get(8)?,
                    last_used_at: row.get(9)?,
                })
            })?
            .collect::<SqliteResult<Vec<AgentTokenStatsRow>>>()?;

        Ok(stats)
    }
}

/// Row type returned by agent token stats queries
pub struct AgentTokenStatsRow {
    pub agent_id: String,
    pub total_sessions: i64,
    pub total_tokens: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_tokens: i64,
    pub cache_tokens: i64,
    pub total_cost_usd: f64,
    pub first_used_at: i64,
    pub last_used_at: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::Source;

    #[test]
    fn test_database_creation() {
        let db = Database::new(":memory:").unwrap();
        assert!(db.conn.lock().is_ok());
    }

    #[test]
    fn test_session_lifecycle() {
        let db = Database::new(":memory:").unwrap();

        let session = ChatSession::new();
        let session_id = session.id.clone();

        // Save session
        db.save_session(&session).unwrap();

        // Load session
        let loaded = db.load_session(&session_id).unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().id, session_id);

        // Delete session
        let deleted = db.delete_session(&session_id).unwrap();
        assert!(deleted);

        // Verify deletion
        let loaded = db.load_session(&session_id).unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn test_message_persistence() {
        let db = Database::new(":memory:").unwrap();

        let mut session = ChatSession::new();
        let session_id = session.id.clone();

        db.save_session(&session).unwrap();

        // Add message
        let sources = vec![Source {
            title: "test.txt".to_string(),
            content: "test content".to_string(),
        }];

        session.add_message("user".to_string(), "Hello".to_string(), sources.clone());

        let message = session.messages.last().unwrap();
        db.save_message(&session_id, message).unwrap();

        // Load and verify
        let loaded = db.load_session(&session_id).unwrap().unwrap();
        assert_eq!(loaded.messages.len(), 1);
        assert_eq!(loaded.messages[0].content, "Hello");
        assert_eq!(loaded.messages[0].sources.len(), 1);
        assert_eq!(loaded.messages[0].sources[0].title, "test.txt");
    }

    #[test]
    fn test_list_sessions() {
        let db = Database::new(":memory:").unwrap();

        let session1 = ChatSession::new();
        let session2 = ChatSession::new();

        db.save_session(&session1).unwrap();
        db.save_session(&session2).unwrap();

        let sessions = db.list_sessions().unwrap();
        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn test_cleanup_old_sessions() {
        let db = Database::new(":memory:").unwrap();

        let session = ChatSession::new();
        db.save_session(&session).unwrap();

        // Clean up sessions older than very large number (should delete nothing since session is new)
        let deleted = db.cleanup_old_sessions(999999999).unwrap();
        assert_eq!(deleted, 0);

        // Verify session still exists
        let loaded = db.load_session(&session.id).unwrap();
        assert!(loaded.is_some());

        // Wait 1 second to ensure timestamp difference (timestamps are in seconds)
        std::thread::sleep(std::time::Duration::from_secs(1));

        // Clean up sessions older than 0 seconds (should delete the session now)
        let deleted = db.cleanup_old_sessions(0).unwrap();
        assert_eq!(deleted, 1);

        // Verify session is deleted
        let loaded = db.load_session(&session.id).unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn test_messages_persist_after_session_update() {
        // Regression test for CASCADE DELETE bug where updating a session
        // would delete all its messages due to INSERT OR REPLACE triggering
        // ON DELETE CASCADE on the foreign key
        let db = Database::new(":memory:").unwrap();

        let mut session = ChatSession::new();
        let session_id = session.id.clone();

        // Save initial session
        db.save_session(&session).unwrap();

        // Add first user message
        session.add_message("user".to_string(), "First message".to_string(), vec![]);
        let message1 = session.messages.last().unwrap();
        db.save_message(&session_id, message1).unwrap();

        // Update session (this used to trigger CASCADE DELETE)
        session.title = Some("Test Session".to_string());
        db.save_session(&session).unwrap();

        // Verify first message still exists
        let loaded = db.load_session(&session_id).unwrap().unwrap();
        assert_eq!(loaded.messages.len(), 1);
        assert_eq!(loaded.messages[0].content, "First message");
        assert_eq!(loaded.messages[0].role, "user");

        // Add second assistant message
        session.add_message("assistant".to_string(), "Response".to_string(), vec![]);
        let message2 = session.messages.last().unwrap();
        db.save_message(&session_id, message2).unwrap();

        // Update session again with token usage
        session.token_usage.total_tokens = 100;
        session.token_usage.input_tokens = 50;
        session.token_usage.output_tokens = 50;
        db.save_session(&session).unwrap();

        // Verify both messages still exist
        let loaded = db.load_session(&session_id).unwrap().unwrap();
        assert_eq!(loaded.messages.len(), 2);
        assert_eq!(loaded.messages[0].content, "First message");
        assert_eq!(loaded.messages[0].role, "user");
        assert_eq!(loaded.messages[1].content, "Response");
        assert_eq!(loaded.messages[1].role, "assistant");

        // Verify session metadata was updated
        assert_eq!(loaded.title, Some("Test Session".to_string()));
        assert_eq!(loaded.token_usage.total_tokens, 100);
        assert_eq!(loaded.token_usage.input_tokens, 50);
        assert_eq!(loaded.token_usage.output_tokens, 50);
    }

    #[test]
    fn test_multiple_message_rounds_persist() {
        // Test that multiple user-assistant message pairs persist correctly
        // when session is updated between each pair
        let db = Database::new(":memory:").unwrap();

        let mut session = ChatSession::new();
        let session_id = session.id.clone();

        // Save initial session
        db.save_session(&session).unwrap();

        // Simulate 3 rounds of conversation
        for i in 1..=3 {
            // Add user message
            session.add_message("user".to_string(), format!("User message {}", i), vec![]);
            let user_msg = session.messages.last().unwrap();
            db.save_message(&session_id, user_msg).unwrap();

            // Update session
            db.save_session(&session).unwrap();

            // Add assistant message
            session.add_message(
                "assistant".to_string(),
                format!("Assistant response {}", i),
                vec![],
            );
            let assistant_msg = session.messages.last().unwrap();
            db.save_message(&session_id, assistant_msg).unwrap();

            // Update session with new token counts
            session.token_usage.total_tokens += 10;
            db.save_session(&session).unwrap();
        }

        // Verify all 6 messages (3 user + 3 assistant) persisted
        let loaded = db.load_session(&session_id).unwrap().unwrap();
        assert_eq!(loaded.messages.len(), 6);

        // Verify message order and content
        for i in 0..3 {
            let user_idx = i * 2;
            let assistant_idx = user_idx + 1;

            assert_eq!(loaded.messages[user_idx].role, "user");
            assert_eq!(
                loaded.messages[user_idx].content,
                format!("User message {}", i + 1)
            );

            assert_eq!(loaded.messages[assistant_idx].role, "assistant");
            assert_eq!(
                loaded.messages[assistant_idx].content,
                format!("Assistant response {}", i + 1)
            );
        }

        // Verify final token count
        assert_eq!(loaded.token_usage.total_tokens, 30);
    }

    #[test]
    fn test_tool_invocations_persist() {
        use serde_json::json;

        // Test that tool invocations are correctly saved and loaded
        let db = Database::new(":memory:").unwrap();
        let mut session = ChatSession::new();
        let session_id = session.id.clone();

        // Save initial session
        db.save_session(&session).unwrap();

        // Add user message
        session.add_message("user".to_string(), "Execute demo tool".to_string(), vec![]);
        let user_msg = session.messages.last().unwrap();
        db.save_message(&session_id, user_msg).unwrap();

        // Add assistant message with thinking steps (tool invocations)
        let thinking_steps = vec![
            crate::session::ThinkingStep {
                step_type: "tool".to_string(),
                step_order: 1,
                content: None,
                tool_name: Some("demo_tool".to_string()),
                tool_arguments: Some(json!({"message": "Hello World"})),
                tool_result: Some(r#"{"success": true, "echo": "Hello World"}"#.to_string()),
                tool_error: None,
                content_before_tool: None,
            },
            crate::session::ThinkingStep {
                step_type: "tool".to_string(),
                step_order: 2,
                content: None,
                tool_name: Some("read_file".to_string()),
                tool_arguments: Some(json!({"path": "/tmp/test.txt"})),
                tool_result: None,
                tool_error: Some("File not found".to_string()),
                content_before_tool: None,
            },
        ];

        session.add_message(
            "assistant".to_string(),
            "Tools executed".to_string(),
            vec![],
        );

        // Set thinking steps on the message
        if let Some(message) = session.messages.last_mut() {
            message.thinking_steps = Some(thinking_steps.clone());
        }

        let assistant_msg = session.messages.last().unwrap();
        db.save_message(&session_id, assistant_msg).unwrap();

        // Load session and verify thinking steps were persisted
        let loaded = db.load_session(&session_id).unwrap().unwrap();
        assert_eq!(loaded.messages.len(), 2);

        // Verify user message has no thinking steps
        assert_eq!(loaded.messages[0].role, "user");
        assert!(loaded.messages[0].thinking_steps.is_none());

        // Verify assistant message has thinking steps
        assert_eq!(loaded.messages[1].role, "assistant");
        let loaded_steps = loaded.messages[1].thinking_steps.as_ref().unwrap();
        assert_eq!(loaded_steps.len(), 2);

        // Verify first tool step (successful execution)
        assert_eq!(loaded_steps[0].step_type, "tool");
        assert_eq!(loaded_steps[0].tool_name.as_ref().unwrap(), "demo_tool");
        assert_eq!(
            loaded_steps[0].tool_arguments.as_ref().unwrap()["message"],
            "Hello World"
        );
        assert!(loaded_steps[0].tool_result.is_some());
        assert!(loaded_steps[0].tool_error.is_none());
        assert!(
            loaded_steps[0]
                .tool_result
                .as_ref()
                .unwrap()
                .contains("Hello World")
        );

        // Verify second tool step (error case)
        assert_eq!(loaded_steps[1].step_type, "tool");
        assert_eq!(loaded_steps[1].tool_name.as_ref().unwrap(), "read_file");
        assert_eq!(
            loaded_steps[1].tool_arguments.as_ref().unwrap()["path"],
            "/tmp/test.txt"
        );
        assert!(loaded_steps[1].tool_result.is_none());
        assert!(loaded_steps[1].tool_error.is_some());
        assert_eq!(
            loaded_steps[1].tool_error.as_ref().unwrap(),
            "File not found"
        );
    }

    #[test]
    fn test_empty_reasoning_steps_filtered() {
        // Test that empty reasoning steps are filtered out when loading sessions
        let db = Database::new(":memory:").unwrap();
        let mut session = ChatSession::new();
        let session_id = session.id.clone();

        db.save_session(&session).unwrap();

        // Add user message
        session.add_message("user".to_string(), "Hello".to_string(), vec![]);
        let user_msg = session.messages.last().unwrap();
        db.save_message(&session_id, user_msg).unwrap();

        // Add assistant message with empty reasoning step
        let thinking_steps = vec![crate::session::ThinkingStep {
            step_type: "reasoning".to_string(),
            step_order: 0,
            content: Some("".to_string()), // Empty content
            tool_name: None,
            tool_arguments: None,
            tool_result: None,
            tool_error: None,
            content_before_tool: None,
        }];

        session.add_message("assistant".to_string(), "Response".to_string(), vec![]);
        if let Some(message) = session.messages.last_mut() {
            message.thinking_steps = Some(thinking_steps);
        }

        let assistant_msg = session.messages.last().unwrap();
        db.save_message(&session_id, assistant_msg).unwrap();

        // Load session - empty reasoning should be filtered out
        let loaded = db.load_session(&session_id).unwrap().unwrap();
        assert_eq!(loaded.messages.len(), 2);

        // Verify assistant message has NO thinking steps (empty one was filtered)
        assert_eq!(loaded.messages[1].role, "assistant");
        assert!(loaded.messages[1].thinking_steps.is_none());
    }

    #[test]
    fn test_whitespace_reasoning_steps_filtered() {
        // Test that reasoning steps with only whitespace are filtered out
        let db = Database::new(":memory:").unwrap();
        let mut session = ChatSession::new();
        let session_id = session.id.clone();

        db.save_session(&session).unwrap();

        session.add_message("user".to_string(), "Hello".to_string(), vec![]);
        let user_msg = session.messages.last().unwrap();
        db.save_message(&session_id, user_msg).unwrap();

        // Add assistant message with whitespace-only reasoning step
        let thinking_steps = vec![crate::session::ThinkingStep {
            step_type: "reasoning".to_string(),
            step_order: 0,
            content: Some("   \n\t  ".to_string()), // Only whitespace
            tool_name: None,
            tool_arguments: None,
            tool_result: None,
            tool_error: None,
            content_before_tool: None,
        }];

        session.add_message("assistant".to_string(), "Response".to_string(), vec![]);
        if let Some(message) = session.messages.last_mut() {
            message.thinking_steps = Some(thinking_steps);
        }

        let assistant_msg = session.messages.last().unwrap();
        db.save_message(&session_id, assistant_msg).unwrap();

        // Load session - whitespace-only reasoning should be filtered out
        let loaded = db.load_session(&session_id).unwrap().unwrap();

        // Verify assistant message has NO thinking steps (whitespace was filtered)
        assert!(loaded.messages[1].thinking_steps.is_none());
    }

    #[test]
    fn test_valid_reasoning_preserved() {
        // Test that valid reasoning steps are preserved
        let db = Database::new(":memory:").unwrap();
        let mut session = ChatSession::new();
        let session_id = session.id.clone();

        db.save_session(&session).unwrap();

        session.add_message("user".to_string(), "Hello".to_string(), vec![]);
        let user_msg = session.messages.last().unwrap();
        db.save_message(&session_id, user_msg).unwrap();

        // Add assistant message with valid reasoning step
        let thinking_steps = vec![crate::session::ThinkingStep {
            step_type: "reasoning".to_string(),
            step_order: 0,
            content: Some("This is valid reasoning content".to_string()),
            tool_name: None,
            tool_arguments: None,
            tool_result: None,
            tool_error: None,
            content_before_tool: None,
        }];

        session.add_message("assistant".to_string(), "Response".to_string(), vec![]);
        if let Some(message) = session.messages.last_mut() {
            message.thinking_steps = Some(thinking_steps);
        }

        let assistant_msg = session.messages.last().unwrap();
        db.save_message(&session_id, assistant_msg).unwrap();

        // Load session - valid reasoning should be preserved
        let loaded = db.load_session(&session_id).unwrap().unwrap();

        // Verify assistant message has thinking steps with valid content
        assert!(loaded.messages[1].thinking_steps.is_some());
        let steps = loaded.messages[1].thinking_steps.as_ref().unwrap();
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].step_type, "reasoning");
        assert_eq!(
            steps[0].content.as_ref().unwrap(),
            "This is valid reasoning content"
        );
    }

    #[test]
    fn test_background_job_crud_lifecycle() {
        let db = Database::new(":memory:").unwrap();

        // Create a one-off job
        let job = BackgroundJob {
            id: None,
            name: "Test Job".to_string(),
            schedule_type: "once".to_string(),
            cron_expression: None,
            priority: 5,
            max_cpu_percent: 50,
            status: "pending".to_string(),
            last_run: None,
            next_run: None,
            retries: 0,
            max_retries: 3,
            payload: r#"{"agent_id":"test","message":"hello"}"#.to_string(),
            result: None,
            error_message: None,
            is_active: true,
            timeout_seconds: 3600,
        };

        // Create
        let job_id = db.create_job(&job).unwrap();
        assert!(job_id > 0);

        // Read
        let fetched = db.get_job_by_id(job_id).unwrap();
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.name, "Test Job");
        assert_eq!(fetched.status, "pending");
        assert!(fetched.is_active);

        // Update status to running
        db.update_job_status(job_id, "running").unwrap();
        let fetched = db.get_job_by_id(job_id).unwrap().unwrap();
        assert_eq!(fetched.status, "running");

        // Update result
        db.update_job_result(job_id, "completed", Some("done"), None)
            .unwrap();
        let fetched = db.get_job_by_id(job_id).unwrap().unwrap();
        assert_eq!(fetched.status, "completed");
        assert_eq!(fetched.result, Some("done".to_string()));

        // Get all jobs
        let all_jobs = db.get_all_jobs().unwrap();
        assert_eq!(all_jobs.len(), 1);
    }

    #[test]
    fn test_background_job_complete_deactivates_one_off() {
        let db = Database::new(":memory:").unwrap();

        let job = BackgroundJob {
            id: None,
            name: "One-off".to_string(),
            schedule_type: "once".to_string(),
            cron_expression: None,
            priority: 5,
            max_cpu_percent: 50,
            status: "pending".to_string(),
            last_run: None,
            next_run: None,
            retries: 0,
            max_retries: 3,
            payload: r#"{}"#.to_string(),
            result: None,
            error_message: None,
            is_active: true,
            timeout_seconds: 3600,
        };

        let job_id = db.create_job(&job).unwrap();

        // Complete the job (should deactivate one-off jobs)
        db.complete_job(job_id, "once", Some("result"), None)
            .unwrap();

        let fetched = db.get_job_by_id(job_id).unwrap().unwrap();
        assert_eq!(fetched.status, "completed");
        assert!(!fetched.is_active); // One-off job deactivated
    }

    #[test]
    fn test_background_job_complete_keeps_cron_active() {
        let db = Database::new(":memory:").unwrap();

        let job = BackgroundJob {
            id: None,
            name: "Cron Job".to_string(),
            schedule_type: "cron".to_string(),
            cron_expression: Some("0 * * * *".to_string()),
            priority: 5,
            max_cpu_percent: 50,
            status: "running".to_string(),
            last_run: None,
            next_run: None,
            retries: 0,
            max_retries: 3,
            payload: r#"{}"#.to_string(),
            result: None,
            error_message: None,
            is_active: true,
            timeout_seconds: 3600,
        };

        let job_id = db.create_job(&job).unwrap();

        // Complete the job (should keep cron jobs active)
        db.complete_job(job_id, "cron", Some("result"), None)
            .unwrap();

        let fetched = db.get_job_by_id(job_id).unwrap().unwrap();
        assert_eq!(fetched.status, "completed");
        assert!(fetched.is_active); // Cron job stays active
    }

    #[test]
    fn test_background_job_retries() {
        let db = Database::new(":memory:").unwrap();

        let job = BackgroundJob {
            id: None,
            name: "Retry Test".to_string(),
            schedule_type: "once".to_string(),
            cron_expression: None,
            priority: 5,
            max_cpu_percent: 50,
            status: "pending".to_string(),
            last_run: None,
            next_run: None,
            retries: 0,
            max_retries: 3,
            payload: r#"{}"#.to_string(),
            result: None,
            error_message: None,
            is_active: true,
            timeout_seconds: 3600,
        };

        let job_id = db.create_job(&job).unwrap();

        // Increment retries twice
        db.increment_job_retries(job_id).unwrap();
        db.increment_job_retries(job_id).unwrap();

        let fetched = db.get_job_by_id(job_id).unwrap().unwrap();
        assert_eq!(fetched.retries, 2);

        // Update with error
        db.update_job_result(job_id, "failed", None, Some("timeout"))
            .unwrap();

        let fetched = db.get_job_by_id(job_id).unwrap().unwrap();
        assert_eq!(fetched.status, "failed");
        assert_eq!(fetched.error_message, Some("timeout".to_string()));
    }

    #[test]
    fn test_background_job_cancel() {
        let db = Database::new(":memory:").unwrap();

        let job = BackgroundJob {
            id: None,
            name: "Cancel Me".to_string(),
            schedule_type: "once".to_string(),
            cron_expression: None,
            priority: 5,
            max_cpu_percent: 50,
            status: "pending".to_string(),
            last_run: None,
            next_run: None,
            retries: 0,
            max_retries: 3,
            payload: r#"{}"#.to_string(),
            result: None,
            error_message: None,
            is_active: true,
            timeout_seconds: 3600,
        };

        let job_id = db.create_job(&job).unwrap();

        // Cancel the job
        db.cancel_job(job_id).unwrap();

        let fetched = db.get_job_by_id(job_id).unwrap().unwrap();
        assert_eq!(fetched.status, "cancelled");
        assert!(!fetched.is_active);
    }

    #[test]
    fn test_background_job_delete() {
        let db = Database::new(":memory:").unwrap();

        let job = BackgroundJob {
            id: None,
            name: "Delete Me".to_string(),
            schedule_type: "once".to_string(),
            cron_expression: None,
            priority: 5,
            max_cpu_percent: 50,
            status: "pending".to_string(),
            last_run: None,
            next_run: None,
            retries: 0,
            max_retries: 3,
            payload: r#"{}"#.to_string(),
            result: None,
            error_message: None,
            is_active: true,
            timeout_seconds: 3600,
        };

        let job_id = db.create_job(&job).unwrap();

        // Delete the job
        db.delete_job(job_id).unwrap();

        // Verify it's gone
        let fetched = db.get_job_by_id(job_id).unwrap();
        assert!(fetched.is_none());

        // Verify job list is empty
        let all_jobs = db.get_all_jobs().unwrap();
        assert!(all_jobs.is_empty());
    }

    #[test]
    fn test_background_job_payload_parsing() {
        let db = Database::new(":memory:").unwrap();

        let payload = serde_json::json!({
            "agent_id": "shakespeare",
            "message": "Say hello",
            "system_prompt": null,
            "file_path": null,
            "session_id": null
        });

        let job = BackgroundJob {
            id: None,
            name: "Payload Test".to_string(),
            schedule_type: "once".to_string(),
            cron_expression: None,
            priority: 5,
            max_cpu_percent: 50,
            status: "pending".to_string(),
            last_run: None,
            next_run: None,
            retries: 0,
            max_retries: 3,
            payload: payload.to_string(),
            result: None,
            error_message: None,
            is_active: true,
            timeout_seconds: 3600,
        };

        let job_id = db.create_job(&job).unwrap();
        let fetched = db.get_job_by_id(job_id).unwrap().unwrap();

        // Parse the payload back
        let parsed: serde_json::Value = serde_json::from_str(&fetched.payload).unwrap();
        assert_eq!(parsed["agent_id"], "shakespeare");
        assert_eq!(parsed["message"], "Say hello");
    }

    #[test]
    fn test_background_job_priority_ordering() {
        let db = Database::new(":memory:").unwrap();

        // Create jobs with different priorities
        let priorities = vec![3, 8, 1, 10, 5];
        for priority in priorities {
            let job = BackgroundJob {
                id: None,
                name: format!("Priority {}", priority),
                schedule_type: "once".to_string(),
                cron_expression: None,
                priority,
                max_cpu_percent: 50,
                status: "pending".to_string(),
                last_run: None,
                next_run: None,
                retries: 0,
                max_retries: 3,
                payload: r#"{}"#.to_string(),
                result: None,
                error_message: None,
                is_active: true,
                timeout_seconds: 3600,
            };
            db.create_job(&job).unwrap();
        }

        // Get pending jobs (should be ordered by priority DESC)
        let pending = db.get_pending_jobs().unwrap();
        assert_eq!(pending.len(), 5);

        // Verify descending order
        for i in 0..pending.len() - 1 {
            assert!(pending[i].priority >= pending[i + 1].priority);
        }
    }

    // Pause/Resume Tests
    #[test]
    fn test_pause_job_sets_inactive() {
        let db = Database::new(":memory:").unwrap();

        let job = BackgroundJob {
            id: None,
            name: "Pausable Cron".to_string(),
            schedule_type: "cron".to_string(),
            cron_expression: Some("0 0 9 * * *".to_string()),
            priority: 5,
            max_cpu_percent: 50,
            status: "pending".to_string(),
            last_run: None,
            next_run: None,
            retries: 0,
            max_retries: 3,
            payload: r#"{}"#.to_string(),
            result: None,
            error_message: None,
            is_active: true,
            timeout_seconds: 3600,
        };

        let job_id = db.create_job(&job).unwrap();

        // Pause the job
        db.pause_job(job_id).unwrap();

        let fetched = db.get_job_by_id(job_id).unwrap().unwrap();
        assert!(!fetched.is_active);
    }

    #[test]
    fn test_resume_job_sets_active() {
        let db = Database::new(":memory:").unwrap();

        let job = BackgroundJob {
            id: None,
            name: "Resumable Cron".to_string(),
            schedule_type: "cron".to_string(),
            cron_expression: Some("0 0 9 * * *".to_string()),
            priority: 5,
            max_cpu_percent: 50,
            status: "pending".to_string(),
            last_run: None,
            next_run: None,
            retries: 0,
            max_retries: 3,
            payload: r#"{}"#.to_string(),
            result: None,
            error_message: None,
            is_active: true,
            timeout_seconds: 3600,
        };

        let job_id = db.create_job(&job).unwrap();

        // Pause then resume
        db.pause_job(job_id).unwrap();
        assert!(!db.get_job_by_id(job_id).unwrap().unwrap().is_active);

        db.resume_job(job_id).unwrap();
        assert!(db.get_job_by_id(job_id).unwrap().unwrap().is_active);
    }

    #[test]
    fn test_pause_only_affects_cron_jobs() {
        let db = Database::new(":memory:").unwrap();

        // Create a one-off job
        let job = BackgroundJob {
            id: None,
            name: "One-off".to_string(),
            schedule_type: "once".to_string(),
            cron_expression: None,
            priority: 5,
            max_cpu_percent: 50,
            status: "pending".to_string(),
            last_run: None,
            next_run: None,
            retries: 0,
            max_retries: 3,
            payload: r#"{}"#.to_string(),
            result: None,
            error_message: None,
            is_active: true,
            timeout_seconds: 3600,
        };

        let job_id = db.create_job(&job).unwrap();

        // Attempt to pause (should only work on cron jobs)
        db.pause_job(job_id).unwrap();

        // Should still be active because it's not a cron job
        let fetched = db.get_job_by_id(job_id).unwrap().unwrap();
        assert!(fetched.is_active); // Pause SQL query filters by schedule_type = 'cron'
    }

    // Cleanup/Retention Tests
    #[test]
    fn test_cleanup_old_jobs() {
        let db = Database::new(":memory:").unwrap();

        // Create old completed job (simulate by manually setting timestamp)
        let old_job = BackgroundJob {
            id: None,
            name: "Old Completed".to_string(),
            schedule_type: "once".to_string(),
            cron_expression: None,
            priority: 5,
            max_cpu_percent: 50,
            status: "completed".to_string(),
            last_run: None,
            next_run: None,
            retries: 0,
            max_retries: 3,
            payload: r#"{}"#.to_string(),
            result: Some("done".to_string()),
            error_message: None,
            is_active: false,
            timeout_seconds: 3600,
        };

        let old_job_id = db.create_job(&old_job).unwrap();

        // Manually set old timestamp (40 days ago)
        let conn = db.conn.lock().unwrap();
        conn.execute(
            "UPDATE background_jobs SET updated_at = datetime('now', '-40 days') WHERE id = ?1",
            params![old_job_id],
        )
        .unwrap();
        drop(conn);

        // Create recent completed job
        let recent_job = BackgroundJob {
            id: None,
            name: "Recent Completed".to_string(),
            schedule_type: "once".to_string(),
            cron_expression: None,
            priority: 5,
            max_cpu_percent: 50,
            status: "completed".to_string(),
            last_run: None,
            next_run: None,
            retries: 0,
            max_retries: 3,
            payload: r#"{}"#.to_string(),
            result: Some("done".to_string()),
            error_message: None,
            is_active: false,
            timeout_seconds: 3600,
        };

        let recent_job_id = db.create_job(&recent_job).unwrap();

        // Cleanup jobs older than 30 days
        let deleted = db.cleanup_old_jobs(30).unwrap();

        // Should have deleted only the old job
        assert_eq!(deleted, 1);
        assert!(db.get_job_by_id(old_job_id).unwrap().is_none());
        assert!(db.get_job_by_id(recent_job_id).unwrap().is_some());
    }

    #[test]
    fn test_cleanup_only_removes_completed_and_failed() {
        let db = Database::new(":memory:").unwrap();

        // Create old jobs with different statuses
        let statuses = vec![
            ("pending", true),    // Should NOT be deleted
            ("running", true),    // Should NOT be deleted
            ("completed", false), // Should be deleted
            ("failed", false),    // Should be deleted
        ];

        let mut job_ids = vec![];
        for (status, should_remain) in &statuses {
            let job = BackgroundJob {
                id: None,
                name: format!("Job {}", status),
                schedule_type: "once".to_string(),
                cron_expression: None,
                priority: 5,
                max_cpu_percent: 50,
                status: status.to_string(),
                last_run: None,
                next_run: None,
                retries: 0,
                max_retries: 3,
                payload: r#"{}"#.to_string(),
                result: None,
                error_message: None,
                is_active: !*should_remain,
                timeout_seconds: 3600,
            };

            let job_id = db.create_job(&job).unwrap();
            job_ids.push((job_id, *should_remain));

            // Set old timestamp (40 days ago)
            let conn = db.conn.lock().unwrap();
            conn.execute(
                "UPDATE background_jobs SET updated_at = datetime('now', '-40 days') WHERE id = ?1",
                params![job_id],
            )
            .unwrap();
            drop(conn);
        }

        // Cleanup jobs older than 30 days
        let deleted = db.cleanup_old_jobs(30).unwrap();

        // Should have deleted only completed and failed
        assert_eq!(deleted, 2);

        // Verify which jobs remain
        for (job_id, should_remain) in job_ids {
            let exists = db.get_job_by_id(job_id).unwrap().is_some();
            assert_eq!(exists, should_remain);
        }
    }

    #[test]
    fn test_cleanup_with_zero_retention_deletes_all_old() {
        let db = Database::new(":memory:").unwrap();

        // Create a just-completed job
        let job = BackgroundJob {
            id: None,
            name: "Just Completed".to_string(),
            schedule_type: "once".to_string(),
            cron_expression: None,
            priority: 5,
            max_cpu_percent: 50,
            status: "completed".to_string(),
            last_run: None,
            next_run: None,
            retries: 0,
            max_retries: 3,
            payload: r#"{}"#.to_string(),
            result: Some("done".to_string()),
            error_message: None,
            is_active: false,
            timeout_seconds: 3600,
        };

        let job_id = db.create_job(&job).unwrap();

        // Set timestamp to 1 day ago
        let conn = db.conn.lock().unwrap();
        conn.execute(
            "UPDATE background_jobs SET updated_at = datetime('now', '-1 days') WHERE id = ?1",
            params![job_id],
        )
        .unwrap();
        drop(conn);

        // Cleanup with 0 retention (delete everything older than now)
        let deleted = db.cleanup_old_jobs(0).unwrap();

        // Should delete the 1-day-old job
        assert_eq!(deleted, 1);
        assert!(db.get_job_by_id(job_id).unwrap().is_none());
    }

    // Retry Logic Tests
    #[test]
    fn test_job_retry_increments_correctly() {
        let db = Database::new(":memory:").unwrap();

        let job = BackgroundJob {
            id: None,
            name: "Retry Test".to_string(),
            schedule_type: "once".to_string(),
            cron_expression: None,
            priority: 5,
            max_cpu_percent: 50,
            status: "pending".to_string(),
            last_run: None,
            next_run: None,
            retries: 0,
            max_retries: 3,
            payload: r#"{}"#.to_string(),
            result: None,
            error_message: None,
            is_active: true,
            timeout_seconds: 3600,
        };

        let job_id = db.create_job(&job).unwrap();

        // First failure
        db.increment_job_retries(job_id).unwrap();
        assert_eq!(db.get_job_by_id(job_id).unwrap().unwrap().retries, 1);

        // Second failure
        db.increment_job_retries(job_id).unwrap();
        assert_eq!(db.get_job_by_id(job_id).unwrap().unwrap().retries, 2);

        // Third failure
        db.increment_job_retries(job_id).unwrap();
        assert_eq!(db.get_job_by_id(job_id).unwrap().unwrap().retries, 3);
    }

    #[test]
    fn test_job_max_retries_check() {
        let db = Database::new(":memory:").unwrap();

        let job = BackgroundJob {
            id: None,
            name: "Max Retries".to_string(),
            schedule_type: "once".to_string(),
            cron_expression: None,
            priority: 5,
            max_cpu_percent: 50,
            status: "pending".to_string(),
            last_run: None,
            next_run: None,
            retries: 0,
            max_retries: 2,
            payload: r#"{}"#.to_string(),
            result: None,
            error_message: None,
            is_active: true,
            timeout_seconds: 3600,
        };

        let job_id = db.create_job(&job).unwrap();

        // Simulate failures up to max
        db.increment_job_retries(job_id).unwrap();
        db.increment_job_retries(job_id).unwrap();

        let job_after = db.get_job_by_id(job_id).unwrap().unwrap();
        assert_eq!(job_after.retries, 2);
        assert!(job_after.retries >= job_after.max_retries); // Should not retry anymore
    }

    #[test]
    fn test_job_retries_reset_on_completion() {
        let db = Database::new(":memory:").unwrap();

        let job = BackgroundJob {
            id: None,
            name: "Retry Reset".to_string(),
            schedule_type: "cron".to_string(),
            cron_expression: Some("0 0 9 * * *".to_string()),
            priority: 5,
            max_cpu_percent: 50,
            status: "running".to_string(),
            last_run: None,
            next_run: None,
            retries: 2, // Had failures before
            max_retries: 3,
            payload: r#"{}"#.to_string(),
            result: None,
            error_message: None,
            is_active: true,
            timeout_seconds: 3600,
        };

        let job_id = db.create_job(&job).unwrap();

        // Complete the job - retries stay as-is for historical record
        db.complete_job(job_id, "cron", Some("success"), None)
            .unwrap();

        let completed = db.get_job_by_id(job_id).unwrap().unwrap();
        assert_eq!(completed.status, "completed");
        // Note: Retries are preserved for historical tracking
        assert_eq!(completed.retries, 2);
    }
}

// ============================================================================
// Background Jobs
// ============================================================================

/// Represents a background job (cron or one-off task)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BackgroundJob {
    pub id: Option<i64>,
    pub name: String,
    pub schedule_type: String, // "cron" or "once"
    pub cron_expression: Option<String>,
    pub priority: i32,
    pub max_cpu_percent: i32,
    pub status: String,
    pub last_run: Option<String>,
    pub next_run: Option<String>,
    pub retries: i32,
    pub max_retries: i32,
    pub payload: String, // JSON
    pub result: Option<String>,
    pub error_message: Option<String>,
    pub is_active: bool,
    pub timeout_seconds: i64, // Job timeout in seconds (0 = no timeout)
}

/// Job Execution Record (tracks individual runs of a job)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JobExecution {
    pub id: Option<i64>,
    pub job_id: i64,
    pub session_id: Option<String>,
    pub status: String,         // "completed", "failed", "cancelled"
    pub result: Option<String>, // JSON
    pub error_message: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub duration_ms: Option<i64>,
    pub tokens_used: Option<i64>,
    pub cost_usd: Option<f64>,
}

/// Payload for a background job
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JobPayload {
    pub agent_id: String,
    pub message: String,
    pub system_prompt: Option<String>,
    pub file_path: Option<String>,
    pub session_id: Option<String>,
}

impl Database {
    /// Create a new background job
    pub fn create_job(&self, job: &BackgroundJob) -> SqliteResult<i64> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT INTO background_jobs
             (name, schedule_type, cron_expression, priority, max_cpu_percent, status, retries, max_retries, payload, result, error_message, is_active, timeout_seconds)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                job.name,
                job.schedule_type,
                job.cron_expression,
                job.priority,
                job.max_cpu_percent,
                job.status,
                job.retries,
                job.max_retries,
                job.payload,
                job.result,
                job.error_message,
                job.is_active,
                job.timeout_seconds,
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// Get all pending jobs (for restoration on startup)
    pub fn get_pending_jobs(&self) -> SqliteResult<Vec<BackgroundJob>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, name, schedule_type, cron_expression, priority, max_cpu_percent,
                    status, last_run, next_run, retries, max_retries, payload,
                    result, error_message, is_active, timeout_seconds
             FROM background_jobs
             WHERE is_active = 1 AND status = 'pending'
             ORDER BY priority DESC, created_at ASC",
        )?;

        let jobs = stmt
            .query_map([], |row| {
                Ok(BackgroundJob {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    schedule_type: row.get(2)?,
                    cron_expression: row.get(3)?,
                    priority: row.get(4)?,
                    max_cpu_percent: row.get(5)?,
                    status: row.get(6)?,
                    last_run: row.get(7)?,
                    next_run: row.get(8)?,
                    retries: row.get(9)?,
                    max_retries: row.get(10)?,
                    payload: row.get(11)?,
                    result: row.get(12)?,
                    error_message: row.get(13)?,
                    is_active: row.get(14)?,
                    timeout_seconds: row.get(15)?,
                })
            })?
            .collect::<SqliteResult<Vec<BackgroundJob>>>()?;

        Ok(jobs)
    }

    /// Get all active cron jobs (for scheduling)
    pub fn get_active_cron_jobs(&self) -> SqliteResult<Vec<BackgroundJob>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, name, schedule_type, cron_expression, priority, max_cpu_percent,
                    status, last_run, next_run, retries, max_retries, payload,
                    result, error_message, is_active, timeout_seconds
             FROM background_jobs
             WHERE is_active = 1 AND schedule_type = 'cron' AND status != 'cancelled'
             ORDER BY priority DESC",
        )?;

        let jobs = stmt
            .query_map([], |row| {
                Ok(BackgroundJob {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    schedule_type: row.get(2)?,
                    cron_expression: row.get(3)?,
                    priority: row.get(4)?,
                    max_cpu_percent: row.get(5)?,
                    status: row.get(6)?,
                    last_run: row.get(7)?,
                    next_run: row.get(8)?,
                    retries: row.get(9)?,
                    max_retries: row.get(10)?,
                    payload: row.get(11)?,
                    result: row.get(12)?,
                    error_message: row.get(13)?,
                    is_active: row.get(14)?,
                    timeout_seconds: row.get(15)?,
                })
            })?
            .collect::<SqliteResult<Vec<BackgroundJob>>>()?;

        Ok(jobs)
    }

    /// Update job status
    pub fn update_job_status(&self, id: i64, status: &str) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();

        if status == "running" {
            conn.execute(
                "UPDATE background_jobs 
                 SET status = ?1, last_run = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
                 WHERE id = ?2",
                params![status, id],
            )?;
        } else {
            conn.execute(
                "UPDATE background_jobs 
                 SET status = ?1, updated_at = CURRENT_TIMESTAMP
                 WHERE id = ?2",
                params![status, id],
            )?;
        }

        Ok(())
    }

    /// Update job result and error message
    pub fn update_job_result(
        &self,
        id: i64,
        status: &str,
        result: Option<&str>,
        error_message: Option<&str>,
    ) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "UPDATE background_jobs
             SET status = ?1, result = ?2, error_message = ?3, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?4",
            params![status, result, error_message, id],
        )?;

        Ok(())
    }

    /// Complete a job (marks as completed, deactivates one-off jobs so they don't re-run)
    pub fn complete_job(
        &self,
        id: i64,
        schedule_type: &str,
        result: Option<&str>,
        error_message: Option<&str>,
    ) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();

        // One-off jobs should be deactivated so they don't get re-queued on restart
        let is_active = if schedule_type == "cron" { 1 } else { 0 };

        conn.execute(
            "UPDATE background_jobs
             SET status = 'completed', result = ?1, error_message = ?2, is_active = ?3, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?4",
            params![result, error_message, is_active, id],
        )?;

        Ok(())
    }

    /// Increment job retries
    pub fn increment_job_retries(&self, id: i64) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "UPDATE background_jobs 
             SET retries = retries + 1, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?1",
            params![id],
        )?;

        Ok(())
    }

    /// Cancel a job
    pub fn cancel_job(&self, id: i64) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "UPDATE background_jobs
             SET status = 'cancelled', is_active = 0, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?1",
            params![id],
        )?;

        Ok(())
    }

    /// Pause a background job (sets is_active = false)
    pub fn pause_job(&self, id: i64) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "UPDATE background_jobs
             SET is_active = 0, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?1 AND schedule_type = 'cron'",
            params![id],
        )?;

        Ok(())
    }

    /// Resume a background job (sets is_active = true)
    pub fn resume_job(&self, id: i64) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "UPDATE background_jobs
             SET is_active = 1, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?1 AND schedule_type = 'cron'",
            params![id],
        )?;

        Ok(())
    }

    /// Delete a job
    pub fn delete_job(&self, id: i64) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute("DELETE FROM background_jobs WHERE id = ?1", params![id])?;

        Ok(())
    }

    /// Get all jobs
    pub fn get_all_jobs(&self) -> SqliteResult<Vec<BackgroundJob>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, name, schedule_type, cron_expression, priority, max_cpu_percent,
                    status, last_run, next_run, retries, max_retries, payload,
                    result, error_message, is_active, timeout_seconds
             FROM background_jobs
             ORDER BY created_at DESC",
        )?;

        let jobs = stmt
            .query_map([], |row| {
                Ok(BackgroundJob {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    schedule_type: row.get(2)?,
                    cron_expression: row.get(3)?,
                    priority: row.get(4)?,
                    max_cpu_percent: row.get(5)?,
                    status: row.get(6)?,
                    last_run: row.get(7)?,
                    next_run: row.get(8)?,
                    retries: row.get(9)?,
                    max_retries: row.get(10)?,
                    payload: row.get(11)?,
                    result: row.get(12)?,
                    error_message: row.get(13)?,
                    is_active: row.get(14)?,
                    timeout_seconds: row.get(15)?,
                })
            })?
            .collect::<SqliteResult<Vec<BackgroundJob>>>()?;

        Ok(jobs)
    }

    /// Get a single job by ID
    pub fn get_job_by_id(&self, id: i64) -> SqliteResult<Option<BackgroundJob>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, name, schedule_type, cron_expression, priority, max_cpu_percent,
                    status, last_run, next_run, retries, max_retries, payload,
                    result, error_message, is_active, timeout_seconds
             FROM background_jobs
             WHERE id = ?1",
        )?;

        let job = stmt.query_row(params![id], |row| {
            Ok(BackgroundJob {
                id: row.get(0)?,
                name: row.get(1)?,
                schedule_type: row.get(2)?,
                cron_expression: row.get(3)?,
                priority: row.get(4)?,
                max_cpu_percent: row.get(5)?,
                status: row.get(6)?,
                last_run: row.get(7)?,
                next_run: row.get(8)?,
                retries: row.get(9)?,
                max_retries: row.get(10)?,
                payload: row.get(11)?,
                result: row.get(12)?,
                error_message: row.get(13)?,
                is_active: row.get(14)?,
                timeout_seconds: row.get(15)?,
            })
        });

        match job {
            Ok(j) => Ok(Some(j)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Delete old completed/failed jobs (retention policy)
    /// Deletes jobs that are completed or failed and older than the specified days
    pub fn cleanup_old_jobs(&self, retention_days: i64) -> SqliteResult<usize> {
        let conn = self.conn.lock().unwrap();

        let deleted = conn.execute(
            "DELETE FROM background_jobs
             WHERE status IN ('completed', 'failed')
             AND updated_at < datetime('now', '-' || ?1 || ' days')",
            params![retention_days],
        )?;

        Ok(deleted)
    }

    /// Update job's next_run timestamp
    pub fn update_job_next_run(&self, id: i64, next_run: Option<&str>) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "UPDATE background_jobs
             SET next_run = ?1, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?2",
            params![next_run, id],
        )?;

        Ok(())
    }

    // ===== Job Executions (Execution History) =====

    /// Create a new job execution record
    pub fn create_job_execution(
        &self,
        job_id: i64,
        session_id: Option<&str>,
        status: &str,
        result: Option<&str>,
        error_message: Option<&str>,
        started_at: &str,
        completed_at: Option<&str>,
        duration_ms: Option<i64>,
        tokens_used: Option<i64>,
        cost_usd: Option<f64>,
    ) -> SqliteResult<i64> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT INTO job_executions (job_id, session_id, status, result, error_message,
                                         started_at, completed_at, duration_ms, tokens_used, cost_usd)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                job_id,
                session_id,
                status,
                result,
                error_message,
                started_at,
                completed_at,
                duration_ms,
                tokens_used,
                cost_usd
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// Get all executions for a specific job
    pub fn get_job_executions(
        &self,
        job_id: i64,
        limit: Option<i64>,
    ) -> SqliteResult<Vec<JobExecution>> {
        let conn = self.conn.lock().unwrap();

        let query = if let Some(lim) = limit {
            format!(
                "SELECT id, job_id, session_id, status, result, error_message,
                        started_at, completed_at, duration_ms, tokens_used, cost_usd
                 FROM job_executions
                 WHERE job_id = ?1
                 ORDER BY started_at DESC
                 LIMIT {}",
                lim
            )
        } else {
            "SELECT id, job_id, session_id, status, result, error_message,
                    started_at, completed_at, duration_ms, tokens_used, cost_usd
             FROM job_executions
             WHERE job_id = ?1
             ORDER BY started_at DESC"
                .to_string()
        };

        let mut stmt = conn.prepare(&query)?;

        let executions = stmt
            .query_map([job_id], |row| {
                Ok(JobExecution {
                    id: row.get(0)?,
                    job_id: row.get(1)?,
                    session_id: row.get(2)?,
                    status: row.get(3)?,
                    result: row.get(4)?,
                    error_message: row.get(5)?,
                    started_at: row.get(6)?,
                    completed_at: row.get(7)?,
                    duration_ms: row.get(8)?,
                    tokens_used: row.get(9)?,
                    cost_usd: row.get(10)?,
                })
            })?
            .collect::<SqliteResult<Vec<JobExecution>>>()?;

        Ok(executions)
    }

    /// Get a single job execution by ID
    pub fn get_job_execution(&self, id: i64) -> SqliteResult<Option<JobExecution>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, job_id, session_id, status, result, error_message,
                    started_at, completed_at, duration_ms, tokens_used, cost_usd
             FROM job_executions
             WHERE id = ?1",
        )?;

        let mut rows = stmt.query([id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(JobExecution {
                id: row.get(0)?,
                job_id: row.get(1)?,
                session_id: row.get(2)?,
                status: row.get(3)?,
                result: row.get(4)?,
                error_message: row.get(5)?,
                started_at: row.get(6)?,
                completed_at: row.get(7)?,
                duration_ms: row.get(8)?,
                tokens_used: row.get(9)?,
                cost_usd: row.get(10)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Delete old job executions (retention policy)
    pub fn delete_old_job_executions(&self, max_age_days: i64) -> SqliteResult<usize> {
        let conn = self.conn.lock().unwrap();

        let rows_deleted = conn.execute(
            "DELETE FROM job_executions
             WHERE started_at < datetime('now', '-' || ?1 || ' days')",
            params![max_age_days],
        )?;

        Ok(rows_deleted)
    }
}
