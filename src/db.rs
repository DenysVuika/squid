use log::{debug, info};
use rusqlite::{params, Connection, Result as SqliteResult};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::io::{Read, Write};
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use sha2::{Sha256, Digest};

use crate::session::{ChatMessage, ChatSession, Source};

/// Database manager for SQLite operations
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Create a new database connection
    /// If the database file doesn't exist, it will be created
    pub fn new<P: AsRef<Path>>(path: P) -> SqliteResult<Self> {
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
                            debug!("Migration {} already partially applied (duplicate column), marking as complete", version);
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
        run_migration(1, "Initial schema", include_str!("../migrations/001_initial_schema.sql"))?;

        // Migration 002: Logs table
        run_migration(2, "Logs table", include_str!("../migrations/002_logs_table.sql"))?;

        // Migration 003: Session titles
        run_migration(3, "Session titles", include_str!("../migrations/003_session_titles.sql"))?;

        // Migration 004: Token tracking
        run_migration(4, "Token tracking", include_str!("../migrations/004_token_tracking.sql"))?;

        // Migration 005: Context window
        run_migration(5, "Context window", include_str!("../migrations/005_context_window.sql"))?;

        // Migration 006: Deduplicate sources
        run_migration(6, "Deduplicate sources", include_str!("../migrations/006_deduplicate_sources.sql"))?;

        // Migration 007: Reasoning column
        run_migration(7, "Reasoning column", include_str!("../migrations/007_reasoning_column.sql"))?;

        // Migration 008: Tool invocations
        run_migration(8, "Tool invocations", include_str!("../migrations/008_tool_invocations.sql"))?;

        // Migration 009: Thinking steps
        run_migration(9, "Thinking steps", include_str!("../migrations/009_thinking_steps.sql"))?;

        // Migration 010: Content split markers
        run_migration(10, "Content split markers", include_str!("../migrations/010_content_split_markers.sql"))?;

        info!("Database migrations completed successfully");
        Ok(())
    }

    /// Save a session to the database
    pub fn save_session(&self, session: &ChatSession) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();

        debug!("Saving session: {}", session.id);

        // Try to update existing session first
        let updated = conn.execute(
            "UPDATE sessions SET created_at = ?2, updated_at = ?3, metadata = ?4, title = ?5, model_id = ?6, total_tokens = ?7, input_tokens = ?8, output_tokens = ?9, reasoning_tokens = ?10, cache_tokens = ?11, cost_usd = ?12, context_window = ?13 WHERE id = ?1",
            params![
                session.id,
                session.created_at,
                session.updated_at,
                Option::<String>::None,
                session.title.as_ref(),
                session.model_id.as_ref(),
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
                "INSERT INTO sessions (id, created_at, updated_at, metadata, title, model_id, total_tokens, input_tokens, output_tokens, reasoning_tokens, cache_tokens, cost_usd, context_window) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![
                    session.id,
                    session.created_at,
                    session.updated_at,
                    Option::<String>::None,
                    session.title.as_ref(),
                    session.model_id.as_ref(),
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

        debug!("Loading session: {}", session_id);

        // Load session metadata
        let mut stmt = conn.prepare("SELECT id, created_at, updated_at, title, model_id, total_tokens, input_tokens, output_tokens, reasoning_tokens, cache_tokens, cost_usd, context_window FROM sessions WHERE id = ?1")?;
        let session_result = stmt.query_row(params![session_id], |row| {
            Ok(ChatSession {
                id: row.get(0)?,
                messages: Vec::new(), // Will be populated below
                created_at: row.get(1)?,
                updated_at: row.get(2)?,
                title: row.get(3)?,
                model_id: row.get(4)?,
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

        let messages = msg_stmt.query_map(params![session_id], |row| {
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
            debug!("Processing message_id {} for session {}", message_id, session_id);
            
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

            debug!("Loading thinking steps for message_id: {}", message_id);
            
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

            debug!("Found {} thinking steps for message_id: {}", thinking_steps.len(), message_id);
            
            let thinking_steps = if thinking_steps.is_empty() {
                None
            } else {
                debug!("Loaded {} thinking steps for message {}", thinking_steps.len(), message_id);
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

        debug!("Saving message for session: {} (role: {})", session_id, message.role);

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
                debug!("Source '{}' exceeds size limit ({} bytes > {} bytes), skipping",
                    source.title, source.content.len(), MAX_FILE_SIZE);
                continue;
            }

            // Calculate hash of content
            let mut hasher = Sha256::new();
            hasher.update(source.content.as_bytes());
            let hash = format!("{:x}", hasher.finalize());

            // Check if content already exists
            let content_id: Option<i64> = conn.query_row(
                "SELECT id FROM file_contents WHERE content_hash = ?1",
                params![hash],
                |row| row.get(0),
            ).ok();

            let content_id = if let Some(id) = content_id {
                // Content already exists, reuse it
                debug!("Reusing existing content (hash: {})", hash);
                id
            } else {
                // Compress content
                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(source.content.as_bytes()).map_err(|e| {
                    rusqlite::Error::ToSqlConversionFailure(Box::new(e))
                })?;
                let compressed = encoder.finish().map_err(|e| {
                    rusqlite::Error::ToSqlConversionFailure(Box::new(e))
                })?;

                let original_size = source.content.len() as i64;
                let compressed_size = compressed.len() as i64;

                debug!("Compressed content from {} to {} bytes ({:.1}% reduction)",
                    original_size, compressed_size,
                    100.0 * (1.0 - compressed_size as f64 / original_size as f64));

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
                let tool_args_json = step.tool_arguments.as_ref()
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
            debug!("Saved {} thinking steps for message {}", thinking_steps.len(), message_id);
        }

        Ok(message_id)
    }

    /// Delete a session and all its messages
    pub fn delete_session(&self, session_id: &str) -> SqliteResult<bool> {
        let conn = self.conn.lock().unwrap();

        debug!("Deleting session: {}", session_id);

        let deleted = conn.execute("DELETE FROM sessions WHERE id = ?1", params![session_id])?;

        Ok(deleted > 0)
    }

    /// List all session IDs, ordered by updated_at (most recent first)
    pub fn list_sessions(&self) -> SqliteResult<Vec<String>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare("SELECT id FROM sessions ORDER BY updated_at DESC")?;

        let sessions = stmt.query_map([], |row| row.get(0))?
            .collect::<SqliteResult<Vec<String>>>()?;

        Ok(sessions)
    }

    /// Update session title
    pub fn update_session_title(&self, session_id: &str, title: &str) -> SqliteResult<bool> {
        let conn = self.conn.lock().unwrap();

        debug!("Updating session title: {} -> {}", session_id, title);

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

        debug!("Cleaning up sessions older than timestamp: {}", cutoff_time);

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
            session.add_message(
                "user".to_string(),
                format!("User message {}", i),
                vec![],
            );
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
        session.add_message(
            "user".to_string(),
            "Execute demo tool".to_string(),
            vec![],
        );
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
            },
            crate::session::ThinkingStep {
                step_type: "tool".to_string(),
                step_order: 2,
                content: None,
                tool_name: Some("read_file".to_string()),
                tool_arguments: Some(json!({"path": "/tmp/test.txt"})),
                tool_result: None,
                tool_error: Some("File not found".to_string()),
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
        assert_eq!(loaded_steps[0].tool_arguments.as_ref().unwrap()["message"], "Hello World");
        assert!(loaded_steps[0].tool_result.is_some());
        assert!(loaded_steps[0].tool_error.is_none());
        assert!(loaded_steps[0].tool_result.as_ref().unwrap().contains("Hello World"));

        // Verify second tool step (error case)
        assert_eq!(loaded_steps[1].step_type, "tool");
        assert_eq!(loaded_steps[1].tool_name.as_ref().unwrap(), "read_file");
        assert_eq!(loaded_steps[1].tool_arguments.as_ref().unwrap()["path"], "/tmp/test.txt");
        assert!(loaded_steps[1].tool_result.is_none());
        assert!(loaded_steps[1].tool_error.is_some());
        assert_eq!(loaded_steps[1].tool_error.as_ref().unwrap(), "File not found");
    }
}
