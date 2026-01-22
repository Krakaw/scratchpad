//! Authentication and authorization tests

use scratchpad::auth::{
    create_token, validate_token, Claims, User, UserRole, SessionManager,
};

#[test]
fn test_create_admin_user() {
    let user = User::new("admin".to_string(), UserRole::Admin);
    assert_eq!(user.username, "admin");
    assert_eq!(user.role, UserRole::Admin);
    assert!(user.active);
    assert!(user.is_admin());
    assert!(user.can_manage_scratches());
    assert!(user.can_view());
}

#[test]
fn test_create_regular_user() {
    let user = User::new("alice".to_string(), UserRole::User);
    assert_eq!(user.username, "alice");
    assert_eq!(user.role, UserRole::User);
    assert!(!user.is_admin());
    assert!(user.can_manage_scratches());
    assert!(user.can_view());
}

#[test]
fn test_create_viewer_user() {
    let user = User::new("bob".to_string(), UserRole::Viewer);
    assert_eq!(user.username, "bob");
    assert_eq!(user.role, UserRole::Viewer);
    assert!(!user.is_admin());
    assert!(!user.can_manage_scratches());
    assert!(user.can_view());
}

#[test]
fn test_inactive_user_cannot_access() {
    let mut user = User::new("test".to_string(), UserRole::Admin);
    user.active = false;
    assert!(!user.is_admin());
    assert!(!user.can_manage_scratches());
    assert!(!user.can_view());
}

#[test]
fn test_jwt_token_creation() {
    let user = User::new("testuser".to_string(), UserRole::Admin);
    let token = create_token(&user).expect("Failed to create token");
    assert!(!token.is_empty());
    assert_eq!(token.split('.').count(), 3); // JWT format: header.payload.signature
}

#[test]
fn test_jwt_token_validation() {
    let user = User::new("alice".to_string(), UserRole::User);
    let token = create_token(&user).expect("Failed to create token");
    let claims = validate_token(&token).expect("Failed to validate token");

    assert_eq!(claims.username, "alice");
    assert_eq!(claims.role, "user");
    assert_eq!(claims.sub, user.id);
}

#[test]
fn test_jwt_token_expiration_check() {
    let user = User::new("test".to_string(), UserRole::Admin);
    let token = create_token(&user).expect("Failed to create token");
    let claims = validate_token(&token).expect("Failed to validate token");

    // Token should not be expired immediately
    assert!(!claims.is_expired());
}

#[test]
fn test_invalid_token_rejection() {
    let result = validate_token("invalid.token.here");
    assert!(result.is_err());
}

#[test]
fn test_malformed_token_rejection() {
    let result = validate_token("not-a-jwt-token");
    assert!(result.is_err());
}

#[test]
fn test_claims_from_user() {
    let user = User::new("testuser".to_string(), UserRole::Admin);
    let claims = Claims::from_user(&user);

    assert_eq!(claims.username, "testuser");
    assert_eq!(claims.role, "admin");
    assert_eq!(claims.sub, user.id);
    assert!(claims.iat > 0);
    assert!(claims.exp > claims.iat);
}

#[test]
fn test_claims_get_role_admin() {
    let claims = Claims {
        sub: "123".to_string(),
        username: "admin".to_string(),
        role: "admin".to_string(),
        iat: 0,
        exp: 9999999999,
    };
    assert_eq!(claims.get_role(), UserRole::Admin);
}

#[test]
fn test_claims_get_role_user() {
    let claims = Claims {
        sub: "123".to_string(),
        username: "user".to_string(),
        role: "user".to_string(),
        iat: 0,
        exp: 9999999999,
    };
    assert_eq!(claims.get_role(), UserRole::User);
}

#[test]
fn test_claims_get_role_viewer() {
    let claims = Claims {
        sub: "123".to_string(),
        username: "viewer".to_string(),
        role: "viewer".to_string(),
        iat: 0,
        exp: 9999999999,
    };
    assert_eq!(claims.get_role(), UserRole::Viewer);
}

#[test]
fn test_user_role_display() {
    assert_eq!(UserRole::Admin.to_string(), "admin");
    assert_eq!(UserRole::User.to_string(), "user");
    assert_eq!(UserRole::Viewer.to_string(), "viewer");
}

#[test]
fn test_user_info_conversion() {
    let user = User::new("alice".to_string(), UserRole::User);
    let info = scratchpad::auth::models::UserInfo::from(user.clone());

    assert_eq!(info.id, user.id);
    assert_eq!(info.username, "alice");
    assert_eq!(info.role, "user");
}

#[tokio::test]
async fn test_session_manager_create_session() {
    let manager = SessionManager::new();
    let user = User::new("testuser".to_string(), UserRole::Admin);
    let session_id = manager.create_session(user.clone()).await;

    assert!(!session_id.is_empty());
    let session = manager.get_session(&session_id).await;
    assert!(session.is_some());
    assert_eq!(session.unwrap().user.username, "testuser");
}

#[tokio::test]
async fn test_session_manager_delete_session() {
    let manager = SessionManager::new();
    let user = User::new("testuser".to_string(), UserRole::Admin);
    let session_id = manager.create_session(user).await;

    manager.delete_session(&session_id).await;
    assert!(manager.get_session(&session_id).await.is_none());
}

#[tokio::test]
async fn test_session_manager_non_existent_session() {
    let manager = SessionManager::new();
    let session = manager.get_session("non-existent-id").await;
    assert!(session.is_none());
}

#[tokio::test]
async fn test_session_manager_cleanup() {
    let manager = SessionManager::new();
    let user1 = User::new("user1".to_string(), UserRole::Admin);
    let user2 = User::new("user2".to_string(), UserRole::User);

    let id1 = manager.create_session(user1).await;
    let id2 = manager.create_session(user2).await;

    let count_before = manager.session_count().await;
    assert_eq!(count_before, 2);

    manager.cleanup_expired().await;
    let count_after = manager.session_count().await;
    // Sessions shouldn't be expired immediately
    assert_eq!(count_after, 2);
}

#[tokio::test]
async fn test_session_clone() {
    let manager1 = SessionManager::new();
    let manager2 = manager1.clone();

    let user = User::new("testuser".to_string(), UserRole::Admin);
    let session_id = manager1.create_session(user).await;

    // Both managers should see the same session
    let session1 = manager1.get_session(&session_id).await;
    let session2 = manager2.get_session(&session_id).await;

    assert!(session1.is_some());
    assert!(session2.is_some());
}

#[test]
fn test_multiple_token_generation() {
    let user1 = User::new("alice".to_string(), UserRole::Admin);
    let user2 = User::new("bob".to_string(), UserRole::User);

    let token1 = create_token(&user1).expect("Failed to create token1");
    let token2 = create_token(&user2).expect("Failed to create token2");

    // Tokens should be different
    assert_ne!(token1, token2);

    // Each token should decode correctly
    let claims1 = validate_token(&token1).expect("Failed to validate token1");
    let claims2 = validate_token(&token2).expect("Failed to validate token2");

    assert_eq!(claims1.username, "alice");
    assert_eq!(claims2.username, "bob");
}

#[test]
fn test_user_id_uniqueness() {
    let user1 = User::new("alice".to_string(), UserRole::Admin);
    let user2 = User::new("alice".to_string(), UserRole::Admin);

    // Each user should have a unique ID even with same username
    assert_ne!(user1.id, user2.id);
}

#[test]
fn test_user_creation_timestamp() {
    let user = User::new("testuser".to_string(), UserRole::Admin);
    let now = chrono::Utc::now();

    // Timestamp should be recent (within 1 second)
    let diff = now.signed_duration_since(user.created_at);
    assert!(diff.num_seconds() <= 1);
}
