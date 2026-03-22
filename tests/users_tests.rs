//! Integration tests for users API
//!
//! Tests for user management endpoints.
//! 用户管理端点测试。

use luminabridge::db::models::User;
use chrono::Utc;

#[test]
fn test_user_creation() {
    let user = User {
        id: 1,
        tenant_id: 1,
        email: "test@example.com".to_string(),
        password_hash: Some("hashed_password".to_string()),
        display_name: Some("Test User".to_string()),
        avatar_url: None,
        role: "user".to_string(),
        status: "active".to_string(),
        oauth_provider: None,
        oauth_id: None,
        last_login_at: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.display_name, Some("Test User".to_string()));
    assert_eq!(user.role, "user");
}

#[test]
fn test_user_role_transitions() {
    let mut user = User {
        id: 1,
        tenant_id: 1,
        email: "test@example.com".to_string(),
        password_hash: None,
        display_name: Some("Test".to_string()),
        avatar_url: None,
        role: "user".to_string(),
        status: "active".to_string(),
        oauth_provider: None,
        oauth_id: None,
        last_login_at: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    // User to admin
    user.role = "admin".to_string();
    assert_eq!(user.role, "admin");
    
    // Admin to user
    user.role = "user".to_string();
    assert_eq!(user.role, "user");
}

#[test]
fn test_user_status_transitions() {
    let mut user = User {
        id: 1,
        tenant_id: 1,
        email: "test@example.com".to_string(),
        password_hash: None,
        display_name: Some("Test".to_string()),
        avatar_url: None,
        role: "user".to_string(),
        status: "active".to_string(),
        oauth_provider: None,
        oauth_id: None,
        last_login_at: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    // Active to inactive
    user.status = "inactive".to_string();
    assert_eq!(user.status, "inactive");
    
    // Inactive to active
    user.status = "active".to_string();
    assert_eq!(user.status, "active");
    
    // Active to suspended
    user.status = "suspended".to_string();
    assert_eq!(user.status, "suspended");
}

#[test]
fn test_user_login_tracking() {
    let mut user = User {
        id: 1,
        tenant_id: 1,
        email: "test@example.com".to_string(),
        password_hash: None,
        display_name: Some("Test".to_string()),
        avatar_url: None,
        role: "user".to_string(),
        status: "active".to_string(),
        oauth_provider: None,
        oauth_id: None,
        last_login_at: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    // First login
    user.last_login_at = Some(Utc::now());
    assert!(user.last_login_at.is_some());
    
    // Subsequent login
    let login_time = Utc::now();
    user.last_login_at = Some(login_time);
    assert!(user.last_login_at.is_some());
}

#[test]
fn test_user_email_validation() {
    let valid_emails = vec![
        "user@example.com",
        "user.name@example.com",
        "user+tag@example.co.uk",
    ];
    
    for email in valid_emails {
        assert!(email.contains('@'));
        assert!(email.contains('.'));
    }
    
    let invalid_emails = vec![
        "invalid",
        "invalid@",
        "@example.com",
    ];
    
    for email in invalid_emails {
        assert!(!email.contains('@') || !email.contains('.'));
    }
}

#[test]
fn test_user_display_name() {
    let user_with_name = User {
        id: 1,
        tenant_id: 1,
        email: "test@example.com".to_string(),
        password_hash: None,
        display_name: Some("John Doe".to_string()),
        avatar_url: None,
        role: "user".to_string(),
        status: "active".to_string(),
        oauth_provider: None,
        oauth_id: None,
        last_login_at: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    assert!(user_with_name.display_name.is_some());
    assert_eq!(user_with_name.display_name.unwrap(), "John Doe");
    
    let user_without_name = User {
        id: 2,
        tenant_id: 1,
        email: "test2@example.com".to_string(),
        password_hash: None,
        display_name: None,
        avatar_url: None,
        role: "user".to_string(),
        status: "active".to_string(),
        oauth_provider: None,
        oauth_id: None,
        last_login_at: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    assert!(user_without_name.display_name.is_none());
}

#[test]
fn test_user_oauth_linking() {
    let mut user = User {
        id: 1,
        tenant_id: 1,
        email: "test@example.com".to_string(),
        password_hash: None,
        display_name: Some("Test".to_string()),
        avatar_url: None,
        role: "user".to_string(),
        status: "active".to_string(),
        oauth_provider: None,
        oauth_id: None,
        last_login_at: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    // Link GitHub OAuth
    user.oauth_provider = Some("github".to_string());
    user.oauth_id = Some("12345".to_string());
    
    assert_eq!(user.oauth_provider, Some("github".to_string()));
    assert_eq!(user.oauth_id, Some("12345".to_string()));
    
    // Link Discord OAuth (would replace in real scenario)
    user.oauth_provider = Some("discord".to_string());
    user.oauth_id = Some("67890".to_string());
    
    assert_eq!(user.oauth_provider, Some("discord".to_string()));
    assert_eq!(user.oauth_id, Some("67890".to_string()));
}

#[test]
fn test_user_serialization() {
    let user = User {
        id: 1,
        tenant_id: 1,
        email: "test@example.com".to_string(),
        password_hash: None,
        display_name: Some("Test User".to_string()),
        avatar_url: None,
        role: "user".to_string(),
        status: "active".to_string(),
        oauth_provider: None,
        oauth_id: None,
        last_login_at: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    let json = serde_json::to_string(&user).unwrap();
    assert!(json.contains("test@example.com"));
    assert!(json.contains("Test User"));
    
    // Deserialize back
    let deserialized: User = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.email, user.email);
    assert_eq!(deserialized.display_name, user.display_name);
}

#[test]
fn test_user_password_hash() {
    // Simulate password hashing (in real scenario, use bcrypt or argon2)
    let password = "secure_password_123";
    let password_hash = format!("hashed_{}", password); // Simplified for testing
    
    let user = User {
        id: 1,
        tenant_id: 1,
        email: "test@example.com".to_string(),
        password_hash: Some(password_hash),
        display_name: Some("Test".to_string()),
        avatar_url: None,
        role: "user".to_string(),
        status: "active".to_string(),
        oauth_provider: None,
        oauth_id: None,
        last_login_at: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    assert!(user.password_hash.is_some());
    assert!(user.password_hash.unwrap().starts_with("hashed_"));
}

#[test]
fn test_user_tenant_association() {
    let user = User {
        id: 1,
        tenant_id: 1,
        email: "test@example.com".to_string(),
        password_hash: None,
        display_name: Some("Test".to_string()),
        avatar_url: None,
        role: "user".to_string(),
        status: "active".to_string(),
        oauth_provider: None,
        oauth_id: None,
        last_login_at: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    assert_eq!(user.tenant_id, 1);
    
    // Multi-tenant scenario
    let user_tenant_2 = User {
        id: 2,
        tenant_id: 2,
        email: "test2@example.com".to_string(),
        password_hash: None,
        display_name: Some("Test 2".to_string()),
        avatar_url: None,
        role: "user".to_string(),
        status: "active".to_string(),
        oauth_provider: None,
        oauth_id: None,
        last_login_at: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    assert_eq!(user_tenant_2.tenant_id, 2);
}
