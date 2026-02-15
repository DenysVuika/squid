use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

use crate::db::Database;

/// Represents a file attachment in a chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAttachment {
    pub filename: String,
    pub content: String,
}

/// Represents a tool invocation (execution) result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInvocation {
    pub name: String,
    pub arguments: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Represents a message in the chat history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String, // "user" or "assistant"
    pub content: String,
    pub sources: Vec<Source>,
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolInvocation>>,
}

/// Represents a source (file attachment) to be displayed with a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub title: String,
    pub content: String,
}

/// Token usage tracking for a session
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    pub total_tokens: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_tokens: i64,
    pub cache_tokens: i64,
    pub context_window: u32,
    pub context_utilization: f64,
}

impl TokenUsage {
    /// Calculate context utilization percentage (0.0 to 1.0)
    pub fn update_utilization(&mut self) {
        if self.context_window > 0 {
            self.context_utilization = (self.total_tokens as f64) / (self.context_window as f64);
        } else {
            self.context_utilization = 0.0;
        }
    }

    /// Check if context usage is approaching the limit (>80%)
    pub fn is_approaching_limit(&self) -> bool {
        self.context_utilization > 0.8
    }

    /// Check if context usage has exceeded the limit
    pub fn is_over_limit(&self) -> bool {
        self.context_utilization > 1.0
    }
}

/// Represents a chat session with history and context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub messages: Vec<ChatMessage>,
    pub created_at: i64,
    pub updated_at: i64,
    pub title: Option<String>,
    pub model_id: Option<String>,
    pub token_usage: TokenUsage,
    pub cost_usd: f64,
}

impl ChatSession {
    /// Create a new chat session
    pub fn new() -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            messages: Vec::new(),
            created_at: now,
            updated_at: now,
            title: None,
            model_id: None,
            token_usage: TokenUsage::default(),
            cost_usd: 0.0,
        }
    }

    /// Update token usage for this session
    pub fn add_tokens(&mut self, input: i64, output: i64, reasoning: i64, cache: i64) {
        self.token_usage.input_tokens += input;
        self.token_usage.output_tokens += output;
        self.token_usage.reasoning_tokens += reasoning;
        self.token_usage.cache_tokens += cache;
        self.token_usage.total_tokens =
            self.token_usage.input_tokens +
            self.token_usage.output_tokens +
            self.token_usage.reasoning_tokens +
            self.token_usage.cache_tokens;

        // Update context utilization
        self.token_usage.update_utilization();
    }

    /// Set the model used for this session
    pub fn set_model(&mut self, model_id: String) {
        if self.model_id.is_none() {
            self.model_id = Some(model_id);
        }
    }

    /// Set the context window size for this session
    pub fn set_context_window(&mut self, context_window: u32) {
        self.token_usage.context_window = context_window;
        self.token_usage.update_utilization();
    }

    /// Add a message to the session
    pub fn add_message(&mut self, role: String, content: String, sources: Vec<Source>, reasoning: Option<String>, tools: Option<Vec<ToolInvocation>>) {
        let now = chrono::Utc::now().timestamp();
        self.messages.push(ChatMessage {
            role,
            content,
            sources,
            timestamp: now,
            reasoning,
            tools,
        });
        self.updated_at = now;
    }

    /// Get the full conversation history for the LLM
    pub fn get_conversation_context(&self) -> Vec<(String, String)> {
        self.messages
            .iter()
            .map(|msg| (msg.role.clone(), msg.content.clone()))
            .collect()
    }

    /// Get sources from the last user message
    pub fn get_last_user_sources(&self) -> Vec<Source> {
        self.messages
            .iter()
            .rev()
            .find(|msg| msg.role == "user")
            .map(|msg| msg.sources.clone())
            .unwrap_or_default()
    }

    /// Generate a title from the first user message
    /// Returns a truncated version (max 100 chars) of the first user message
    fn generate_title(&self) -> Option<String> {
        self.messages
            .iter()
            .find(|msg| msg.role == "user")
            .map(|msg| {
                let content = msg.content.trim();
                if content.len() > 100 {
                    format!("{}...", &content[..97])
                } else {
                    content.to_string()
                }
            })
    }

    /// Update the session title if it hasn't been set yet
    pub fn update_title_if_needed(&mut self) {
        if self.title.is_none() {
            self.title = self.generate_title();
        }
    }
}

impl Default for ChatSession {
    fn default() -> Self {
        Self::new()
    }
}

/// Session manager to handle multiple chat sessions
/// Uses write-through cache with SQLite persistence
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, ChatSession>>>,
    db: Arc<Database>,
}

impl SessionManager {
    /// Create a new session manager with database backend
    pub fn new(db: Database) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            db: Arc::new(db),
        }
    }

    /// Create a new session and return its ID
    pub fn create_session(&self) -> String {
        let session = ChatSession::new();
        let session_id = session.id.clone();

        // Save to database
        if let Err(e) = self.db.save_session(&session) {
            log::error!("Failed to save session to database: {}", e);
        }

        // Cache in memory
        let mut sessions = self.sessions.write().unwrap();
        sessions.insert(session_id.clone(), session);

        session_id
    }

    /// Get a session by ID
    /// First checks memory cache, then falls back to database
    pub fn get_session(&self, session_id: &str) -> Option<ChatSession> {
        // Check memory cache first
        {
            let sessions = self.sessions.read().unwrap();
            if let Some(session) = sessions.get(session_id) {
                return Some(session.clone());
            }
        }

        // Fall back to database
        match self.db.load_session(session_id) {
            Ok(Some(session)) => {
                // Cache for future access
                let mut sessions = self.sessions.write().unwrap();
                sessions.insert(session_id.to_string(), session.clone());
                Some(session)
            }
            Ok(None) => None,
            Err(e) => {
                log::error!("Failed to load session from database: {}", e);
                None
            }
        }
    }

    /// Update a session
    pub fn update_session(&self, session: ChatSession) {
        // Save to database
        if let Err(e) = self.db.save_session(&session) {
            log::error!("Failed to update session in database: {}", e);
        }

        // Update cache
        let mut sessions = self.sessions.write().unwrap();
        sessions.insert(session.id.clone(), session);
    }

    /// Add a user message to a session
    pub fn add_user_message(
        &self,
        session_id: &str,
        content: String,
        files: Vec<FileAttachment>,
    ) -> Result<Vec<Source>, String> {
        // Get or load session
        let mut session = self.get_session(session_id)
            .ok_or_else(|| "Session not found".to_string())?;

        // Convert file attachments to sources
        let sources: Vec<Source> = files
            .iter()
            .map(|file| Source {
                title: file.filename.clone(),
                content: file.content.clone(),
            })
            .collect();

        // Add message to session (users don't have reasoning or tools)
        session.add_message("user".to_string(), content, sources.clone(), None, None);

        // Auto-generate title from first user message if needed
        session.update_title_if_needed();

        // Get the last message
        let message = session.messages.last()
            .ok_or_else(|| "Failed to add message".to_string())?;

        // Save message to database
        if let Err(e) = self.db.save_message(session_id, message) {
            log::error!("Failed to save message to database: {}", e);
            return Err(format!("Failed to save message: {}", e));
        }

        // Update session in database (to save the title)
        if let Err(e) = self.db.save_session(&session) {
            log::error!("Failed to update session in database: {}", e);
        }

        // Update session in cache
        let mut sessions = self.sessions.write().unwrap();
        sessions.insert(session_id.to_string(), session);

        Ok(sources)
    }

    /// Update session title
    pub fn update_session_title(&self, session_id: &str, title: String) -> Result<(), String> {
        // Update in database
        if let Err(e) = self.db.update_session_title(session_id, &title) {
            log::error!("Failed to update session title in database: {}", e);
            return Err(format!("Failed to update session title: {}", e));
        }

        // Update cache if session is loaded
        let mut sessions = self.sessions.write().unwrap();
        if let Some(session) = sessions.get_mut(session_id) {
            session.title = Some(title);
        }

        Ok(())
    }

    /// Add an assistant message to a session
    pub fn add_assistant_message(
        &self,
        session_id: &str,
        content: String,
        sources: Vec<Source>,
        reasoning: Option<String>,
        tools: Option<Vec<ToolInvocation>>,
    ) -> Result<(), String> {
        // Get or load session
        let mut session = self.get_session(session_id)
            .ok_or_else(|| "Session not found".to_string())?;

        // Add message to session
        session.add_message("assistant".to_string(), content, sources, reasoning, tools);

        // Get the last message
        let message = session.messages.last()
            .ok_or_else(|| "Failed to add message".to_string())?;

        // Save message to database
        if let Err(e) = self.db.save_message(session_id, message) {
            log::error!("Failed to save message to database: {}", e);
            return Err(format!("Failed to save message: {}", e));
        }

        // Update session in cache
        let mut sessions = self.sessions.write().unwrap();
        sessions.insert(session_id.to_string(), session);

        Ok(())
    }

    /// Delete a session
    pub fn delete_session(&self, session_id: &str) -> bool {
        // Delete from database
        let db_deleted = self.db.delete_session(session_id).unwrap_or(false);

        // Remove from cache
        let mut sessions = self.sessions.write().unwrap();
        let cache_deleted = sessions.remove(session_id).is_some();

        db_deleted || cache_deleted
    }

    /// Get all session IDs from database
    pub fn list_sessions(&self) -> Vec<String> {
        match self.db.list_sessions() {
            Ok(sessions) => sessions,
            Err(e) => {
                log::error!("Failed to list sessions from database: {}", e);
                Vec::new()
            }
        }
    }

    /// Clean up old sessions (older than specified seconds)
    pub fn cleanup_old_sessions(&self, max_age_seconds: i64) {
        // Clean up database
        if let Err(e) = self.db.cleanup_old_sessions(max_age_seconds) {
            log::error!("Failed to cleanup old sessions from database: {}", e);
        }

        // Clean up cache
        let now = chrono::Utc::now().timestamp();
        let mut sessions = self.sessions.write().unwrap();
        sessions.retain(|_, session| (now - session.updated_at) < max_age_seconds);
    }

    /// Update token usage for a session
    pub fn update_token_usage(
        &self,
        session_id: &str,
        model_id: &str,
        input_tokens: i64,
        output_tokens: i64,
        reasoning_tokens: i64,
        cache_tokens: i64,
        context_window: u32,
    ) -> Result<(), String> {
        // Get or load session
        let mut session = self.get_session(session_id)
            .ok_or_else(|| "Session not found".to_string())?;

        // Set model if not already set
        session.set_model(model_id.to_string());

        // Set context window
        session.set_context_window(context_window);

        // Add token usage
        session.add_tokens(input_tokens, output_tokens, reasoning_tokens, cache_tokens);

        // Update session in database
        if let Err(e) = self.db.save_session(&session) {
            log::error!("Failed to update session token usage in database: {}", e);
            return Err(format!("Failed to update token usage: {}", e));
        }

        // Update cache
        let mut sessions = self.sessions.write().unwrap();
        sessions.insert(session_id.to_string(), session);

        Ok(())
    }
}

// Note: Default is not implemented as SessionManager requires a Database instance

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_session() {
        let db = crate::db::Database::new(":memory:").unwrap();
        let manager = SessionManager::new(db);
        let session_id = manager.create_session();
        assert!(!session_id.is_empty());

        let session = manager.get_session(&session_id);
        assert!(session.is_some());
    }

    #[test]
    fn test_add_messages() {
        let db = crate::db::Database::new(":memory:").unwrap();
        let manager = SessionManager::new(db);
        let session_id = manager.create_session();

        let files = vec![FileAttachment {
            filename: "test.txt".to_string(),
            content: "test content".to_string(),
        }];

        let sources = manager
            .add_user_message(&session_id, "Hello".to_string(), files)
            .unwrap();

        assert_eq!(sources.len(), 1);

        manager
            .add_assistant_message(&session_id, "Hi there!".to_string(), sources, None, None)
            .unwrap();

        let session = manager.get_session(&session_id).unwrap();
        assert_eq!(session.messages.len(), 2);
    }

    #[test]
    fn test_delete_session() {
        let db = crate::db::Database::new(":memory:").unwrap();
        let manager = SessionManager::new(db);
        let session_id = manager.create_session();

        assert!(manager.delete_session(&session_id));
        assert!(manager.get_session(&session_id).is_none());
    }

    #[test]
    fn test_messages_persist_after_multiple_updates() {
        // Regression test for CASCADE DELETE bug
        // Ensures user messages persist when session metadata is updated
        let db = crate::db::Database::new(":memory:").unwrap();
        let manager = SessionManager::new(db);
        let session_id = manager.create_session();

        // Add first user message
        manager
            .add_user_message(&session_id, "First question".to_string(), vec![])
            .unwrap();

        // Verify message exists
        let session = manager.get_session(&session_id).unwrap();
        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.messages[0].role, "user");
        assert_eq!(session.messages[0].content, "First question");

        // Add assistant response
        manager
            .add_assistant_message(&session_id, "First answer".to_string(), vec![], None, None)
            .unwrap();

        // Verify both messages exist
        let session = manager.get_session(&session_id).unwrap();
        assert_eq!(session.messages.len(), 2);
        assert_eq!(session.messages[0].role, "user");
        assert_eq!(session.messages[1].role, "assistant");

        // Add second round of messages
        manager
            .add_user_message(&session_id, "Second question".to_string(), vec![])
            .unwrap();

        manager
            .add_assistant_message(&session_id, "Second answer".to_string(), vec![], None, None)
            .unwrap();

        // Verify all 4 messages persist
        let session = manager.get_session(&session_id).unwrap();
        assert_eq!(session.messages.len(), 4);
        assert_eq!(session.messages[0].content, "First question");
        assert_eq!(session.messages[1].content, "First answer");
        assert_eq!(session.messages[2].content, "Second question");
        assert_eq!(session.messages[3].content, "Second answer");
    }

    #[test]
    fn test_user_messages_persist_across_reload() {
        // Test that user messages persist when session is loaded from database
        // This simulates clearing the in-memory cache and reloading from DB
        let db = crate::db::Database::new(":memory:").unwrap();
        let manager = SessionManager::new(db);
        let session_id = manager.create_session();

        // Add multiple user-assistant pairs
        for i in 1..=3 {
            manager
                .add_user_message(&session_id, format!("Question {}", i), vec![])
                .unwrap();

            manager
                .add_assistant_message(&session_id, format!("Answer {}", i), vec![], None, None)
                .unwrap();
        }

        // Clear the in-memory cache to force reload from database
        {
            let mut sessions = manager.sessions.write().unwrap();
            sessions.clear();
        }

        // Load session from database (should be loaded fresh from DB)
        let session = manager.get_session(&session_id).unwrap();

        // Verify all 6 messages (3 user + 3 assistant) were persisted
        assert_eq!(session.messages.len(), 6);

        for i in 0..3 {
            let user_idx = i * 2;
            let assistant_idx = user_idx + 1;

            assert_eq!(session.messages[user_idx].role, "user");
            assert_eq!(
                session.messages[user_idx].content,
                format!("Question {}", i + 1)
            );

            assert_eq!(session.messages[assistant_idx].role, "assistant");
            assert_eq!(
                session.messages[assistant_idx].content,
                format!("Answer {}", i + 1)
            );
        }
    }

    #[test]
    fn test_token_usage_updates_dont_delete_messages() {
        // Ensure updating token usage doesn't trigger message deletion
        let db = crate::db::Database::new(":memory:").unwrap();
        let manager = SessionManager::new(db);
        let session_id = manager.create_session();

        // Add initial messages
        manager
            .add_user_message(&session_id, "Test message".to_string(), vec![])
            .unwrap();

        manager
            .add_assistant_message(&session_id, "Test response".to_string(), vec![], None, None)
            .unwrap();

        // Update token usage multiple times (simulates streaming updates)
        for i in 1..=5 {
            manager.update_token_usage(
                &session_id,
                "test-model",
                i * 5,
                i * 5,
                0,
                0,
                8192,
            ).unwrap();
        }

        // Clear cache and reload from database
        {
            let mut sessions = manager.sessions.write().unwrap();
            sessions.clear();
        }

        // Verify messages still exist after multiple token updates
        let session = manager.get_session(&session_id).unwrap();
        assert_eq!(session.messages.len(), 2);
        assert_eq!(session.messages[0].content, "Test message");
        assert_eq!(session.messages[1].content, "Test response");
        // Token usage accumulates: sum of (1+2+3+4+5) * 5 for each = 75
        assert_eq!(session.token_usage.input_tokens, 75);
        assert_eq!(session.token_usage.output_tokens, 75);
    }
}
