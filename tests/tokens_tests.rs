//! Integration tests for tokens API
//!
//! Tests for API token management endpoints.
//! API 令牌管理端点测试。

use luminabridge::db::models::Token;
use serde_json::json;
use chrono::{Utc, Duration};
use rust_decimal::Decimal;

#[test]
fn test_token_creation() {
    let token = Token {
        id: 1,
        tenant_id: 1,
        user_id: Some(1),
        key: "sk-test-12345".to_string(),
        name: Some("Test Token".to_string()),
        quota_limit: Some(10000),
        quota_used: 0,
        expire_at: None,
        status: "active".to_string(),
        allowed_ips: None,
        allowed_models: None,
        last_used_at: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    assert_eq!(token.key, "sk-test-12345");
    assert_eq!(token.quota_limit, Some(10000));
    assert_eq!(token.quota_used, 0);
}

#[test]
fn test_token_quota_management() {
    let mut token = Token {
        id: 1,
        tenant_id: 1,
        user_id: Some(1),
        key: "sk-test".to_string(),
        name: Some("Test".to_string()),
        quota_limit: Some(10000),
        quota_used: 0,
        expire_at: None,
        status: "active".to_string(),
        allowed_ips: None,
        allowed_models: None,
        last_used_at: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    // Simulate usage
    token.quota_used += 1000;
    assert_eq!(token.quota_used, 1000);
    
    // Check remaining quota
    let remaining = token.quota_limit.unwrap() - token.quota_used;
    assert_eq!(remaining, 9000);
    
    // Usage percentage
    let usage_pct = (token.quota_used as f64 / token.quota_limit.unwrap() as f64) * 100.0;
    assert_eq!(usage_pct, 10.0);
}

#[test]
fn test_token_expiration() {
    let future_time = Utc::now() + Duration::hours(24);
    
    let token = Token {
        id: 1,
        tenant_id: 1,
        user_id: Some(1),
        key: "sk-test".to_string(),
        name: Some("Test".to_string()),
        quota_limit: None,
        quota_used: 0,
        expire_at: Some(future_time),
        status: "active".to_string(),
        allowed_ips: None,
        allowed_models: None,
        last_used_at: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    // Check if token is expired
    if let Some(expire_at) = token.expire_at {
        assert!(expire_at > Utc::now());
    }
}

#[test]
fn test_token_status_transitions() {
    let mut token = Token {
        id: 1,
        tenant_id: 1,
        user_id: Some(1),
        key: "sk-test".to_string(),
        name: Some("Test".to_string()),
        quota_limit: None,
        quota_used: 0,
        expire_at: None,
        status: "active".to_string(),
        allowed_ips: None,
        allowed_models: None,
        last_used_at: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    // Active to inactive
    token.status = "inactive".to_string();
    assert_eq!(token.status, "inactive");
    
    // Inactive to active
    token.status = "active".to_string();
    assert_eq!(token.status, "active");
    
    // Active to revoked
    token.status = "revoked".to_string();
    assert_eq!(token.status, "revoked");
}

#[test]
fn test_token_allowed_models() {
    let allowed_models = json!(["gpt-3.5-turbo", "gpt-4"]);
    
    let token = Token {
        id: 1,
        tenant_id: 1,
        user_id: Some(1),
        key: "sk-test".to_string(),
        name: Some("Test".to_string()),
        quota_limit: None,
        quota_used: 0,
        expire_at: None,
        status: "active".to_string(),
        allowed_ips: None,
        allowed_models: Some(allowed_models),
        last_used_at: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    // Verify allowed models
    if let Some(ref models) = token.allowed_models {
        if let Some(models_array) = models.as_array() {
            assert_eq!(models_array.len(), 2);
            assert!(models_array.iter().any(|m| m.as_str() == Some("gpt-4")));
        }
    }
}

#[test]
fn test_token_allowed_ips() {
    let allowed_ips = json!(["192.168.1.1", "10.0.0.1"]);
    
    let token = Token {
        id: 1,
        tenant_id: 1,
        user_id: Some(1),
        key: "sk-test".to_string(),
        name: Some("Test".to_string()),
        quota_limit: None,
        quota_used: 0,
        expire_at: None,
        status: "active".to_string(),
        allowed_ips: Some(allowed_ips),
        allowed_models: None,
        last_used_at: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    // Verify allowed IPs
    if let Some(ref ips) = token.allowed_ips {
        if let Some(ips_array) = ips.as_array() {
            assert_eq!(ips_array.len(), 2);
            assert!(ips_array.iter().any(|ip| ip.as_str() == Some("192.168.1.1")));
        }
    }
}

#[test]
fn test_token_quota_exceeded() {
    let mut token = Token {
        id: 1,
        tenant_id: 1,
        user_id: Some(1),
        key: "sk-test".to_string(),
        name: Some("Test".to_string()),
        quota_limit: Some(1000),
        quota_used: 0,
        expire_at: None,
        status: "active".to_string(),
        allowed_ips: None,
        allowed_models: None,
        last_used_at: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    // Use up to quota
    token.quota_used = 1000;
    assert_eq!(token.quota_used, token.quota_limit.unwrap());
    
    // Try to exceed quota
    let requested_tokens = 100;
    let would_exceed = token.quota_used + requested_tokens > token.quota_limit.unwrap();
    assert!(would_exceed);
}

#[test]
fn test_token_unlimited_quota() {
    let token = Token {
        id: 1,
        tenant_id: 1,
        user_id: Some(1),
        key: "sk-unlimited".to_string(),
        name: Some("Unlimited Token".to_string()),
        quota_limit: None, // No limit
        quota_used: 0,
        expire_at: None,
        status: "active".to_string(),
        allowed_ips: None,
        allowed_models: None,
        last_used_at: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    // Unlimited quota should have None
    assert!(token.quota_limit.is_none());
}

#[test]
fn test_token_last_used_tracking() {
    let mut token = Token {
        id: 1,
        tenant_id: 1,
        user_id: Some(1),
        key: "sk-test".to_string(),
        name: Some("Test".to_string()),
        quota_limit: None,
        quota_used: 0,
        expire_at: None,
        status: "active".to_string(),
        allowed_ips: None,
        allowed_models: None,
        last_used_at: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    // Simulate first use
    token.last_used_at = Some(Utc::now());
    assert!(token.last_used_at.is_some());
    
    // Simulate subsequent use
    let usage_time = Utc::now();
    token.last_used_at = Some(usage_time);
    assert!(token.last_used_at.unwrap() >= usage_time - Duration::seconds(1));
}

#[test]
fn test_token_serialization() {
    let token = Token {
        id: 1,
        tenant_id: 1,
        user_id: Some(1),
        key: "sk-test".to_string(),
        name: Some("Test".to_string()),
        quota_limit: Some(10000),
        quota_used: 5000,
        expire_at: None,
        status: "active".to_string(),
        allowed_ips: None,
        allowed_models: None,
        last_used_at: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    let json = serde_json::to_string(&token).unwrap();
    assert!(json.contains("sk-test"));
    assert!(json.contains("10000"));
    assert!(json.contains("5000"));
    
    // Deserialize back
    let deserialized: Token = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.key, token.key);
    assert_eq!(deserialized.quota_limit, token.quota_limit);
}
