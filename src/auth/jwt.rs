//! JWT token handling

use crate::auth::models::{User, UserRole};
use crate::error::{Error, Result};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

const JWT_SECRET: &[u8] = b"scratchpad-secret-key-change-in-production";

/// JWT claims
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Username
    pub username: String,
    /// User role
    pub role: String,
    /// Issued at
    pub iat: i64,
    /// Expiration time (1 hour)
    pub exp: i64,
}

impl Claims {
    /// Create claims from user
    pub fn from_user(user: &User) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            sub: user.id.clone(),
            username: user.username.clone(),
            role: user.role.to_string(),
            iat: now,
            exp: now + 3600, // 1 hour expiration
        }
    }

    /// Get user role
    pub fn get_role(&self) -> UserRole {
        match self.role.as_str() {
            "admin" => UserRole::Admin,
            "user" => UserRole::User,
            _ => UserRole::Viewer,
        }
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp() > self.exp
    }
}

/// Create a JWT token
pub fn create_token(user: &User) -> Result<String> {
    let claims = Claims::from_user(user);
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
    .map_err(|e| Error::Config(format!("Failed to create token: {}", e)))
}

/// Validate and decode a JWT token
pub fn validate_token(token: &str) -> Result<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| Error::Config(format!("Invalid token: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_validate_token() {
        let user = User::new("testuser".to_string(), UserRole::Admin);
        let token = create_token(&user).expect("Failed to create token");
        let claims = validate_token(&token).expect("Failed to validate token");

        assert_eq!(claims.username, "testuser");
        assert_eq!(claims.role, "admin");
        assert!(!claims.is_expired());
    }

    #[test]
    fn test_invalid_token() {
        let result = validate_token("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_token_contains_user_info() {
        let user = User::new("alice".to_string(), UserRole::User);
        let token = create_token(&user).expect("Failed to create token");
        let claims = validate_token(&token).expect("Failed to validate token");

        assert_eq!(claims.sub, user.id);
        assert_eq!(claims.username, "alice");
        assert_eq!(claims.get_role(), UserRole::User);
    }
}
