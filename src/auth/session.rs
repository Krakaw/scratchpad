//! Session management

use crate::auth::models::User;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Session information
#[derive(Debug, Clone)]
pub struct Session {
    /// Session ID
    pub id: String,
    /// User associated with this session
    pub user: User,
    /// When the session was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// When the session last accessed
    pub last_accessed: chrono::DateTime<chrono::Utc>,
}

impl Session {
    /// Create a new session
    pub fn new(user: User) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            user,
            created_at: now,
            last_accessed: now,
        }
    }

    /// Check if session is expired (30 minutes)
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now();
        now.signed_duration_since(self.last_accessed).num_minutes() > 30
    }

    /// Update last accessed time
    pub fn touch(&mut self) {
        self.last_accessed = chrono::Utc::now();
    }
}

/// Session manager for in-memory session storage
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new session
    pub async fn create_session(&self, user: User) -> String {
        let session = Session::new(user);
        let session_id = session.id.clone();
        self.sessions
            .write()
            .await
            .insert(session_id.clone(), session);
        session_id
    }

    /// Get a session by ID
    pub async fn get_session(&self, session_id: &str) -> Option<Session> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            if session.is_expired() {
                sessions.remove(session_id);
                return None;
            }
            session.touch();
            return Some(session.clone());
        }
        None
    }

    /// Delete a session
    pub async fn delete_session(&self, session_id: &str) {
        self.sessions.write().await.remove(session_id);
    }

    /// Cleanup expired sessions
    pub async fn cleanup_expired(&self) {
        let mut sessions = self.sessions.write().await;
        sessions.retain(|_, session| !session.is_expired());
    }

    /// Get session count
    pub async fn session_count(&self) -> usize {
        self.sessions.read().await.len()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for SessionManager {
    fn clone(&self) -> Self {
        Self {
            sessions: Arc::clone(&self.sessions),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::models::UserRole;

    #[tokio::test]
    async fn test_create_and_get_session() {
        let manager = SessionManager::new();
        let user = User::new("testuser".to_string(), UserRole::Admin);
        let session_id = manager.create_session(user.clone()).await;

        let retrieved = manager.get_session(&session_id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().user.username, "testuser");
    }

    #[tokio::test]
    async fn test_delete_session() {
        let manager = SessionManager::new();
        let user = User::new("testuser".to_string(), UserRole::Admin);
        let session_id = manager.create_session(user).await;

        manager.delete_session(&session_id).await;
        let retrieved = manager.get_session(&session_id).await;
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_session_expiration() {
        let manager = SessionManager::new();
        let user = User::new("testuser".to_string(), UserRole::Admin);
        let session_id = manager.create_session(user).await;

        // Manually expire the session for testing
        {
            let mut sessions = manager.sessions.write().await;
            if let Some(session) = sessions.get_mut(&session_id) {
                session.last_accessed = chrono::Utc::now() - chrono::Duration::minutes(31);
            }
        }

        let retrieved = manager.get_session(&session_id).await;
        assert!(retrieved.is_none());
    }
}
