//! Integration tests for channels API
//!
//! Tests for channel management endpoints.
//! 渠道管理端点测试。

use luminabridge::config::Config;
use luminabridge::db::Database;
use luminabridge::db::models::Channel;
use serde_json::json;
use std::sync::Arc;
use chrono::Utc;

#[test]
fn test_channel_model_creation() {
    let channel = Channel {
        id: 1,
        tenant_id: 1,
        name: "Test Channel".to_string(),
        channel_type: "openai".to_string(),
        key: "sk-test-key".to_string(),
        base_url: Some("https://api.openai.com/v1".to_string()),
        models: json!(["gpt-3.5-turbo", "gpt-4"]),
        weight: 10,
        status: "active".to_string(),
        priority: 0,
        timeout_ms: 30000,
        retry_count: 3,
        balance: Some(rust_decimal::Decimal::new(100, 2)),
        last_test_at: None,
        last_test_status: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    assert_eq!(channel.name, "Test Channel");
    assert_eq!(channel.channel_type, "openai");
    assert_eq!(channel.weight, 10);
}

#[test]
fn test_channel_status_transitions() {
    let mut channel = Channel {
        id: 1,
        tenant_id: 1,
        name: "Test".to_string(),
        channel_type: "openai".to_string(),
        key: "test".to_string(),
        base_url: None,
        models: json!([]),
        weight: 10,
        status: "active".to_string(),
        priority: 0,
        timeout_ms: 30000,
        retry_count: 3,
        balance: None,
        last_test_at: None,
        last_test_status: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    // Active to inactive
    channel.status = "inactive".to_string();
    assert_eq!(channel.status, "inactive");
    
    // Inactive to active
    channel.status = "active".to_string();
    assert_eq!(channel.status, "active");
    
    // Active to error
    channel.status = "error".to_string();
    assert_eq!(channel.status, "error");
}

#[test]
fn test_channel_weight_calculation() {
    let channels = vec![
        Channel {
            id: 1,
            tenant_id: 1,
            name: "Channel 1".to_string(),
            channel_type: "openai".to_string(),
            key: "test".to_string(),
            base_url: None,
            models: json!([]),
            weight: 10,
            status: "active".to_string(),
            priority: 0,
            timeout_ms: 30000,
            retry_count: 3,
            balance: None,
            last_test_at: None,
            last_test_status: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        Channel {
            id: 2,
            tenant_id: 1,
            name: "Channel 2".to_string(),
            channel_type: "openai".to_string(),
            key: "test".to_string(),
            base_url: None,
            models: json!([]),
            weight: 20,
            status: "active".to_string(),
            priority: 0,
            timeout_ms: 30000,
            retry_count: 3,
            balance: None,
            last_test_at: None,
            last_test_status: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
    ];
    
    let total_weight: i32 = channels.iter().map(|c| c.weight).sum();
    assert_eq!(total_weight, 30);
    
    // Channel 2 should have 2/3 of traffic
    let channel2_ratio = channels[1].weight as f64 / total_weight as f64;
    assert!((channel2_ratio - 0.667).abs() < 0.01);
}

#[test]
fn test_channel_models_json() {
    let models_list = vec!["gpt-3.5-turbo", "gpt-4", "gpt-4-turbo"];
    let models_json = json!(models_list);
    
    let channel = Channel {
        id: 1,
        tenant_id: 1,
        name: "Test".to_string(),
        channel_type: "openai".to_string(),
        key: "test".to_string(),
        base_url: None,
        models: models_json,
        weight: 10,
        status: "active".to_string(),
        priority: 0,
        timeout_ms: 30000,
        retry_count: 3,
        balance: None,
        last_test_at: None,
        last_test_status: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    // Verify models can be extracted
    if let Some(models_array) = channel.models.as_array() {
        assert_eq!(models_array.len(), 3);
        assert!(models_array.iter().any(|m| m.as_str() == Some("gpt-4")));
    } else {
        panic!("Models should be an array");
    }
}

#[tokio::test]
async fn test_channel_balance_check() {
    let mut channel = Channel {
        id: 1,
        tenant_id: 1,
        name: "Test".to_string(),
        channel_type: "openai".to_string(),
        key: "test".to_string(),
        base_url: None,
        models: json!([]),
        weight: 10,
        status: "active".to_string(),
        priority: 0,
        timeout_ms: 30000,
        retry_count: 3,
        balance: Some(rust_decimal::Decimal::new(500, 2)), // $5.00
        last_test_at: None,
        last_test_status: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    // Check balance is sufficient
    if let Some(balance) = channel.balance {
        assert!(balance > rust_decimal::Decimal::new(0, 2));
        
        // Simulate deduction
        let cost = rust_decimal::Decimal::new(100, 2); // $1.00
        let new_balance = balance - cost;
        assert_eq!(new_balance, rust_decimal::Decimal::new(400, 2)); // $4.00
        
        channel.balance = Some(new_balance);
    }
}

#[test]
fn test_channel_timeout_configuration() {
    let channel = Channel {
        id: 1,
        tenant_id: 1,
        name: "Test".to_string(),
        channel_type: "openai".to_string(),
        key: "test".to_string(),
        base_url: None,
        models: json!([]),
        weight: 10,
        status: "active".to_string(),
        priority: 0,
        timeout_ms: 60000, // 60 seconds
        retry_count: 3,
        balance: None,
        last_test_at: None,
        last_test_status: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    assert_eq!(channel.timeout_ms, 60000);
    assert_eq!(channel.retry_count, 3);
}
