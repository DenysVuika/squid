use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

/// Represents a file attachment in a chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAttachment {
    pub filename: String,
    pub content: String,
}

/// Represents a message in the chat history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String, // "user" or "assistant"
    pub content: String,
    pub sources: Vec<Source>,
    pub timestamp: i64,
}

/// Represents a source (file attachment) to be displayed with a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub title: String,
    pub content: String,
}

/// Represents a chat session with history and context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub messages: Vec<ChatMessage>,
    pub created_at: i64,
    pub updated_at: i64,
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
        }
    }

    /// Add a message to the session
    pub fn add_message(&mut self, role: String, content: String, sources: Vec<Source>) {
        let now = chrono::Utc::now().timestamp();
        self.messages.push(ChatMessage {
            role,
            content,
            sources,
            timestamp: now,
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
}

impl Default for ChatSession {
    fn default() -> Self {
        Self::new()
    }
}

/// Session manager to handle multiple chat sessions
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, ChatSession>>>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new session and return its ID
    pub fn create_session(&self) -> String {
        let session = ChatSession::new();
        let session_id = session.id.clone();

        let mut sessions = self.sessions.write().unwrap();
        sessions.insert(session_id.clone(), session);

        session_id
    }

    /// Get a session by ID
    pub fn get_session(&self, session_id: &str) -> Option<ChatSession> {
        let sessions = self.sessions.read().unwrap();
        sessions.get(session_id).cloned()
    }

    /// Update a session
    pub fn update_session(&self, session: ChatSession) {
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
        let mut sessions = self.sessions.write().unwrap();
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| "Session not found".to_string())?;

        // Convert file attachments to sources
        let sources: Vec<Source> = files
            .iter()
            .map(|file| Source {
                title: file.filename.clone(),
                content: file.content.clone(),
            })
            .collect();

        session.add_message("user".to_string(), content, sources.clone());

        Ok(sources)
    }

    /// Add an assistant message to a session
    pub fn add_assistant_message(
        &self,
        session_id: &str,
        content: String,
        sources: Vec<Source>,
    ) -> Result<(), String> {
        let mut sessions = self.sessions.write().unwrap();
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| "Session not found".to_string())?;

        session.add_message("assistant".to_string(), content, sources);

        Ok(())
    }

    /// Delete a session
    pub fn delete_session(&self, session_id: &str) -> bool {
        let mut sessions = self.sessions.write().unwrap();
        sessions.remove(session_id).is_some()
    }

    /// Get all session IDs
    pub fn list_sessions(&self) -> Vec<String> {
        let sessions = self.sessions.read().unwrap();
        sessions.keys().cloned().collect()
    }

    /// Clean up old sessions (older than specified seconds)
    pub fn cleanup_old_sessions(&self, max_age_seconds: i64) {
        let now = chrono::Utc::now().timestamp();
        let mut sessions = self.sessions.write().unwrap();
        sessions.retain(|_, session| (now - session.updated_at) < max_age_seconds);
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_session() {
        let manager = SessionManager::new();
        let session_id = manager.create_session();
        assert!(!session_id.is_empty());

        let session = manager.get_session(&session_id);
        assert!(session.is_some());
    }

    #[test]
    fn test_add_messages() {
        let manager = SessionManager::new();
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
            .add_assistant_message(&session_id, "Hi there!".to_string(), sources)
            .unwrap();

        let session = manager.get_session(&session_id).unwrap();
        assert_eq!(session.messages.len(), 2);
    }

    #[test]
    fn test_delete_session() {
        let manager = SessionManager::new();
        let session_id = manager.create_session();

        assert!(manager.delete_session(&session_id));
        assert!(manager.get_session(&session_id).is_none());
    }
}
