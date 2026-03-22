//! Integration tests for relay API with authentication
//!
//! Tests for OpenAI-compatible relay endpoints with API key authentication.
//! 带 API 密钥认证的 OpenAI 兼容中继端点测试。

use luminabridge::config::Config;
use luminabridge::db::Database;
use luminabridge::relay::{RelayService, types::*};
use luminabridge::relay::types::{Message, MessageRole, MessageContent, ChatCompletionRequest};
use luminabridge::db::models::Token;
use luminabridge::middleware::api_key_auth::{check_model_permission, check_token_quota};
use std::sync::Arc;
use chrono::{Utc, Duration};
use serde_json::json;

#[tokio::test]
async fn test_relay_service_creation() {
    let config = Arc::new(Config::development());
    let db = Arc::new(Database::new(config.clone()).unwrap());
    
    let relay = RelayService::new(config, db);
    
    // Basic test - ensure service can be created
    assert!(relay.config.server.port > 0);
}

#[tokio::test]
async fn test_build_provider_url() {
    let config = Arc::new(Config::development());
    let db = Arc::new(Database::new(config.clone()).unwrap());
    let relay = RelayService::new(config, db);
    
    // Test with custom base URL
    let url = relay.build_provider_url(&Some("https://api.example.com".to_string()), "/chat/completions");
    assert_eq!(url, "https://api.example.com/chat/completions");
    
    // Test with trailing slash
    let url = relay.build_provider_url(&Some("https://api.example.com/".to_string()), "/chat/completions");
    assert_eq!(url, "https://api.example.com/chat/completions");
    
    // Test with default (OpenAI)
    let url = relay.build_provider_url(&None, "/chat/completions");
    assert!(url.contains("openai.com"));
    assert!(url.contains("/chat/completions"));
}

#[test]
fn test_chat_completion_request_serialization() {
    let request = ChatCompletionRequest {
        model: "gpt-3.5-turbo".to_string(),
        messages: vec![
            Message {
                role: MessageRole::System,
                content: Some(MessageContent::Text("You are a helpful assistant.".to_string())),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            },
            Message {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello!".to_string())),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            },
        ],
        temperature: Some(0.7),
        top_p: None,
        n: None,
        stream: None,
        stop: None,
        max_tokens: Some(100),
        presence_penalty: None,
        frequency_penalty: None,
        logit_bias: None,
        user: None,
        response_format: None,
        tools: None,
        tool_choice: None,
    };
    
    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("gpt-3.5-turbo"));
    assert!(json.contains("Hello!"));
    assert!(json.contains("temperature"));
    
    // Deserialize back
    let deserialized: ChatCompletionRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.model, "gpt-3.5-turbo");
    assert_eq!(deserialized.messages.len(), 2);
}

#[test]
fn test_message_role_serialization() {
    // Test all message roles
    let roles = vec![
        (MessageRole::System, "system"),
        (MessageRole::User, "user"),
        (MessageRole::Assistant, "assistant"),
        (MessageRole::Tool, "tool"),
    ];
    
    for (role, expected) in roles {
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, format!("\"{}\"", expected));
    }
}

#[test]
fn test_message_content_variants() {
    // Test text content
    let text_content = MessageContent::Text("Hello".to_string());
    let json = serde_json::to_string(&text_content).unwrap();
    assert_eq!(json, "\"Hello\"");
    
    // Test parts content (multimodal)
    // This would be tested with actual ContentPart instances
}

#[test]
fn test_usage_serialization() {
    use luminabridge::relay::types::Usage;
    
    let usage = Usage {
        prompt_tokens: 10,
        completion_tokens: 20,
        total_tokens: 30,
    };
    
    let json = serde_json::to_string(&usage).unwrap();
    assert!(json.contains("10"));
    assert!(json.contains("20"));
    assert!(json.contains("30"));
    
    // Deserialize back
    let deserialized: Usage = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.prompt_tokens, 10);
    assert_eq!(deserialized.completion_tokens, 20);
    assert_eq!(deserialized.total_tokens, 30);
}

#[test]
fn test_chat_completion_response_serialization() {
    use luminabridge::relay::types::{ChatCompletionResponse, Choice, Usage};
    
    let response = ChatCompletionResponse {
        id: "chatcmpl-123".to_string(),
        object: "chat.completion".to_string(),
        created: 1234567890,
        model: "gpt-3.5-turbo".to_string(),
        choices: vec![
            Choice {
                index: 0,
                message: Message {
                    role: MessageRole::Assistant,
                    content: Some(MessageContent::Text("Hello! How can I help you?".to_string())),
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                },
                finish_reason: Some("stop".to_string()),
                logprobs: None,
            }
        ],
        usage: Some(Usage {
            prompt_tokens: 10,
            completion_tokens: 20,
            total_tokens: 30,
        }),
        system_fingerprint: None,
    };
    
    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("chatcmpl-123"));
    assert!(json.contains("chat.completion"));
    assert!(json.contains("gpt-3.5-turbo"));
    assert!(json.contains("stop"));
}

#[test]
fn test_model_serialization() {
    use luminabridge::relay::types::Model;
    use chrono::Utc;
    
    let model = Model {
        id: "gpt-3.5-turbo".to_string(),
        object: "model".to_string(),
        created: Utc::now().timestamp() as i64,
        owned_by: "openai".to_string(),
    };
    
    let json = serde_json::to_string(&model).unwrap();
    assert!(json.contains("gpt-3.5-turbo"));
    assert!(json.contains("model"));
    assert!(json.contains("openai"));
}

#[test]
fn test_model_list_serialization() {
    use luminabridge::relay::types::{Model, ModelList};
    
    let model_list = ModelList {
        object: "list".to_string(),
        data: vec![
            Model {
                id: "gpt-3.5-turbo".to_string(),
                object: "model".to_string(),
                created: 1234567890,
                owned_by: "openai".to_string(),
            },
            Model {
                id: "gpt-4".to_string(),
                object: "model".to_string(),
                created: 1234567890,
                owned_by: "openai".to_string(),
            },
        ],
    };
    
    let json = serde_json::to_string(&model_list).unwrap();
    assert!(json.contains("\"list\""));
    assert!(json.contains("gpt-3.5-turbo"));
    assert!(json.contains("gpt-4"));
}

#[tokio::test]
async fn test_channel_selection_no_channels() {
    let config = Arc::new(Config::development());
    let db = Arc::new(Database::new(config.clone()).unwrap());
    let relay = RelayService::new(config, db);
    
    // Test with non-existent tenant
    let result = relay.select_channel("gpt-3.5-turbo", 999).await;
    
    // Should return an error (no channels found)
    assert!(result.is_err());
}

// ============================================================================
// API Key Authentication Tests
// ============================================================================

#[test]
fn test_check_model_permission_no_restrictions() {
    let token = create_test_token(None, None);
    
    // No restrictions should allow all models
    assert!(check_model_permission(&token, "gpt-3.5-turbo").unwrap());
    assert!(check_model_permission(&token, "claude-3").unwrap());
    assert!(check_model_permission(&token, "any-model").unwrap());
}

#[test]
fn test_check_model_permission_with_restrictions() {
    let token = create_test_token(Some(json!(["gpt-3.5-turbo", "gpt-4"])), None);
    
    // Allowed models
    assert!(check_model_permission(&token, "gpt-3.5-turbo").unwrap());
    assert!(check_model_permission(&token, "gpt-4").unwrap());
    
    // Not allowed models
    assert!(!check_model_permission(&token, "claude-3").unwrap());
    assert!(!check_model_permission(&token, "gemini-pro").unwrap());
}

#[test]
fn test_check_model_permission_wildcard() {
    let token = create_test_token(Some(json!(["gpt-*"])), None);
    
    // Wildcard should match
    assert!(check_model_permission(&token, "gpt-3.5-turbo").unwrap());
    assert!(check_model_permission(&token, "gpt-4").unwrap());
    assert!(check_model_permission(&token, "gpt-4-turbo").unwrap());
    
    // Should not match non-gpt models
    assert!(!check_model_permission(&token, "claude-3").unwrap());
}

#[tokio::test]
async fn test_check_token_quota_no_limit() {
    let token = create_test_token(None, None);
    
    // No quota limit should always pass
    assert!(check_token_quota(&token).await.unwrap());
}

#[tokio::test]
async fn test_check_token_quota_under_limit() {
    let mut token = create_test_token(None, Some(10000));
    token.quota_used = 5000;
    
    // Under limit should pass
    assert!(check_token_quota(&token).await.unwrap());
}

#[tokio::test]
async fn test_check_token_quota_exceeded() {
    let mut token = create_test_token(None, Some(10000));
    token.quota_used = 10000;
    
    // At limit should fail
    assert!(!check_token_quota(&token).await.unwrap());
    
    // Over limit should fail
    token.quota_used = 15000;
    assert!(!check_token_quota(&token).await.unwrap());
}

// Helper function to create test tokens
fn create_test_token(
    allowed_models: Option<serde_json::Value>,
    quota_limit: Option<i64>,
) -> Token {
    Token {
        id: 1,
        tenant_id: 1,
        user_id: None,
        key: "sk-test-key".to_string(),
        name: Some("Test Token".to_string()),
        quota_limit,
        quota_used: 0,
        expire_at: None,
        status: "active".to_string(),
        allowed_ips: None,
        allowed_models,
        last_used_at: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

#[test]
fn test_token_expiration_check() {
    // Token not expired
    let token = create_test_token(None, None);
    // Should be valid (no expiration set)
    assert!(token.expire_at.is_none());
    
    // Token expired
    let mut expired_token = create_test_token(None, None);
    expired_token.expire_at = Some(Utc::now() - Duration::hours(1));
    assert!(expired_token.expire_at.is_some());
    assert!(Utc::now() > expired_token.expire_at.unwrap());
}

#[test]
fn test_token_status_check() {
    let mut token = create_test_token(None, None);
    
    // Active token
    assert_eq!(token.status, "active");
    
    // Inactive token
    token.status = "inactive".to_string();
    assert_ne!(token.status, "active");
}

#[test]
fn test_completion_request_serialization() {
    use luminabridge::relay::types::{CompletionRequest, Prompt};
    
    let request = CompletionRequest {
        model: "text-davinci-003".to_string(),
        prompt: Prompt::String("Write a story".to_string()),
        temperature: Some(0.7),
        max_tokens: Some(100),
        stream: None,
        stop: None,
        n: None,
    };
    
    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("text-davinci-003"));
    assert!(json.contains("Write a story"));
}

#[test]
fn test_stop_sequence_variants() {
    use luminabridge::relay::types::StopSequence;
    
    // Test single stop sequence
    let stop = StopSequence::String("\n".to_string());
    let json = serde_json::to_string(&stop).unwrap();
    assert_eq!(json, "\"\\n\"");
    
    // Test multiple stop sequences
    let stop = StopSequence::Array(vec!["\n".to_string(), "END".to_string()]);
    let json = serde_json::to_string(&stop).unwrap();
    assert!(json.contains("\\n"));
    assert!(json.contains("END"));
}
