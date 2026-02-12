use log::{debug, info};
use rusqlite::{params, Connection, Result as SqliteResult};
use std::path::Path;
use std::sync::{Arc, Mutex};

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

        // Read and execute migrations in order
        let schema_001 = include_str!("../migrations/001_initial_schema.sql");
        conn.execute_batch(schema_001)?;

        let schema_002 = include_str!("../migrations/002_logs_table.sql");
        conn.execute_batch(schema_002)?;

        info!("Database migrations completed successfully");
        Ok(())
    }

    /// Save a session to the database
    pub fn save_session(&self, session: &ChatSession) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();

        debug!("Saving session: {}", session.id);

        // Insert or update session
        conn.execute(
            "INSERT OR REPLACE INTO sessions (id, created_at, updated_at, metadata) VALUES (?1, ?2, ?3, ?4)",
            params![session.id, session.created_at, session.updated_at, Option::<String>::None],
        )?;

        Ok(())
    }

    /// Load a session from the database
    pub fn load_session(&self, session_id: &str) -> SqliteResult<Option<ChatSession>> {
        let conn = self.conn.lock().unwrap();

        debug!("Loading session: {}", session_id);

        // Load session metadata
        let mut stmt = conn.prepare("SELECT id, created_at, updated_at FROM sessions WHERE id = ?1")?;
        let session_result = stmt.query_row(params![session_id], |row| {
            Ok(ChatSession {
                id: row.get(0)?,
                messages: Vec::new(), // Will be populated below
                created_at: row.get(1)?,
                updated_at: row.get(2)?,
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
            let message_id: i64 = row.get(0)?;
            let role: String = row.get(1)?;
            let content: String = row.get(2)?;
            let timestamp: i64 = row.get(3)?;

            // Load sources for this message
            let mut source_stmt = conn.prepare(
                "SELECT title, content FROM sources WHERE message_id = ?1"
            )?;

            let sources = source_stmt.query_map(params![message_id], |row| {
                Ok(Source {
                    title: row.get(0)?,
                    content: row.get(1)?,
                })
            })?
            .collect::<SqliteResult<Vec<Source>>>()?;

            Ok(ChatMessage {
                role,
                content,
                sources,
                timestamp,
            })
        })?
        .collect::<SqliteResult<Vec<ChatMessage>>>()?;

        session.messages = messages;

        Ok(Some(session))
    }

    /// Save a message to the database
    pub fn save_message(&self, session_id: &str, message: &ChatMessage) -> SqliteResult<i64> {
        let conn = self.conn.lock().unwrap();

        debug!("Saving message for session: {}", session_id);

        // Insert message
        conn.execute(
            "INSERT INTO messages (session_id, role, content, timestamp) VALUES (?1, ?2, ?3, ?4)",
            params![session_id, message.role, message.content, message.timestamp],
        )?;

        let message_id = conn.last_insert_rowid();

        // Insert sources
        for source in &message.sources {
            conn.execute(
                "INSERT INTO sources (message_id, title, content) VALUES (?1, ?2, ?3)",
                params![message_id, source.title, source.content],
            )?;
        }

        // Update session's updated_at timestamp
        conn.execute(
            "UPDATE sessions SET updated_at = ?1 WHERE id = ?2",
            params![message.timestamp, session_id],
        )?;

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
}
