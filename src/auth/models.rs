//! Authentication models

use serde::{Deserialize, Serialize};
use std::fmt;

/// User roles for authorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    /// Administrator - full access
    Admin,
    /// User - can manage scratches but not system config
    User,
    /// Viewer - read-only access
    Viewer,
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserRole::Admin => write!(f, "admin"),
            UserRole::User => write!(f, "user"),
            UserRole::Viewer => write!(f, "viewer"),
        }
    }
}

/// User information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique user identifier
    pub id: String,
    /// Username for login
    pub username: String,
    /// User's role
    pub role: UserRole,
    /// Whether the account is active
    pub active: bool,
    /// When the account was created
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl User {
    /// Create a new user
    pub fn new(username: String, role: UserRole) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            username,
            role,
            active: true,
            created_at: chrono::Utc::now(),
        }
    }

    /// Check if user is admin
    pub fn is_admin(&self) -> bool {
        self.role == UserRole::Admin && self.active
    }

    /// Check if user can manage scratches
    pub fn can_manage_scratches(&self) -> bool {
        matches!(self.role, UserRole::Admin | UserRole::User) && self.active
    }

    /// Check if user can view
    pub fn can_view(&self) -> bool {
        self.active
    }
}

/// Login credentials
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Login response with token
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
}

/// User information in responses
#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub role: String,
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            role: user.role.to_string(),
        }
    }
}
