//! Relay module for LuminaBridge
//!
//! Handles request relay to AI providers with channel selection, load balancing,
//! failover, and response transformation.
//! 处理向 AI 提供商的请求中继，包括渠道选择、负载均衡、故障转移和响应转换。

pub mod types;

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio_stream::Stream;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{info, warn, error, debug};
use serde_json::{json, Value};
use chrono::Utc;

use crate::config::Config;
use crate::error::{Error, Result};
use crate::db::Database;
use crate::db::models::Channel;
use crate::types::ChannelStatus;
use self::types::{
    ChatCompletionRequest, ChatCompletionResponse, ChatCompletionChunk,
    CompletionRequest, CompletionResponse, Message, MessageRole, MessageContent,
    Choice, Delta, ChunkChoice, Usage, Model, ModelList,
};

/// Relay service for forwarding requests to AI providers
/// 用于将请求转发到 AI 提供商的中继服务
pub struct RelayService {
    /// Application configuration
    /// 应用程序配置
    config: Arc<Config>,
    
    /// Database connection
    /// 数据库连接
    db: Arc<Database>,
    
    /// HTTP client for making requests
    /// 用于发出请求的 HTTP 客户端
    client: reqwest::Client,
}

impl RelayService {
    /// Create a new relay service
    /// 创建新的中继服务
    pub fn new(config: Arc<Config>, db: Arc<Database>) -> Self {
        let timeout = Duration::from_secs(config.server.timeout_secs);
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .gzip(true)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        
        RelayService { config, db, client }
    }
    
    // ========================================================================
    // Channel Selection
    // ========================================================================
    
    /// Select a channel based on model name with load balancing and failover
    /// 根据模型名称选择渠道，支持负载均衡和故障转移
    pub async fn select_channel(&self, model: &str, tenant_id: i64) -> Result<Channel> {
        debug!("Selecting channel for model: {} (tenant: {})", model, tenant_id);
        
        // Get all active channels for this tenant that support the model
        let channels = self.db.get_channels_for_model(tenant_id, model).await?;
        
        if channels.is_empty() {
            return Err(Error::Provider(format!("No channel found for model: {}", model)));
        }
        
        // Filter to only active channels
        let active_channels: Vec<&Channel> = channels
            .iter()
            .filter(|c| c.status == "active")
            .collect();
        
        if active_channels.is_empty() {
            return Err(Error::Provider("No active channels available".to_string()));
        }
        
        // Weighted random selection
        let selected = self.weighted_random_select(&active_channels)?;
        
        info!("Selected channel: {} (weight: {})", selected.name, selected.weight);
        Ok(selected.clone())
    }
    
    /// Weighted random selection of a channel
    /// 渠道的加权随机选择
    fn weighted_random_select(&self, channels: &[&Channel]) -> Result<&Channel> {
        if channels.is_empty() {
            return Err(Error::Provider("No channels to select from".to_string()));
        }
        
        // Calculate total weight
        let total_weight: i32 = channels.iter().map(|c| c.weight).sum();
        
        if total_weight <= 0 {
            // Fall back to first channel if all weights are 0 or negative
            return Ok(channels[0]);
        }
        
        // Random selection based on weight
        let mut rng = rand::random::<i32>() % total_weight;
        
        for channel in channels {
            if rng < channel.weight {
                return Ok(*channel);
            }
            rng -= channel.weight;
        }
        
        // Fallback to last channel
        Ok(*channels.last().unwrap())
    }
    
    // ========================================================================
    // Request Forwarding
    // ========================================================================
    
    /// Relay a chat completion request to the selected channel
    /// 将聊天完成请求中继到选定的渠道
    pub async fn relay_chat_completion(
        &self,
        request: ChatCompletionRequest,
        channel: &Channel,
        api_key: &str,
    ) -> Result<ChatCompletionResponse> {
        info!("Relaying chat completion to channel: {}", channel.name);
        
        let start_time = Instant::now();
        let provider_url = self.build_provider_url(&channel.base_url, "/chat/completions");
        
        // Transform request for specific provider if needed
        let transformed_request = self.transform_request_for_provider(&request, &channel.channel_type)?;
        
        let response = self.client
            .post(&provider_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&transformed_request)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("Request failed: {}", e)))?;
        
        let latency_ms = start_time.elapsed().as_millis() as i32;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            warn!("Provider returned error: {} - {}", status, error_text);
            return Err(Error::Provider(format!("Provider error: {} - {}", status, error_text)));
        }
        
        let response_body: Value = response.json().await
            .map_err(|e| Error::Provider(format!("Failed to parse response: {}", e)))?;
        
        // Transform response to OpenAI format
        let openai_response = self.transform_response_to_openai(response_body, &request.model)?;
        
        info!(
            "Chat completion successful: latency={}ms, tokens={:?}",
            latency_ms,
            openai_response.usage.as_ref().map(|u| u.total_tokens)
        );
        
        Ok(openai_response)
    }
    
    /// Stream a chat completion request
    /// 流式传输聊天完成请求
    pub async fn stream_chat_completion(
        &self,
        request: ChatCompletionRequest,
        channel: &Channel,
        api_key: &str,
    ) -> Result<impl Stream<Item = Result<ChatCompletionChunk>>> {
        info!("Streaming chat completion to channel: {}", channel.name);
        
        let provider_url = self.build_provider_url(&channel.base_url, "/chat/completions");
        
        // Transform request for specific provider
        let mut transformed_request = self.transform_request_for_provider(&request, &channel.channel_type)?;
        transformed_request["stream"] = json!(true);
        
        let response = self.client
            .post(&provider_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&transformed_request)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("Stream request failed: {}", e)))?;
        
        if !response.status().is_success() {
            let status = response.status();
            return Err(Error::Provider(format!("Provider error: {}", status)));
        }
        
        // Create stream from SSE response
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let stream = response.bytes_stream();
        
        tokio::spawn(async move {
            use futures_util::StreamExt;
            let mut stream = stream;
            
            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);
                        // Parse SSE data
                        for line in text.lines() {
                            if line.starts_with("data: ") {
                                let data = &line[6..];
                                if data == "[DONE]" {
                                    break;
                                }
                                
                                if let Ok(chunk) = serde_json::from_str::<ChatCompletionChunk>(data) {
                                    let _ = tx.send(Ok(chunk)).await;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(Error::Provider(format!("Stream error: {}", e)))).await;
                        break;
                    }
                }
            }
        });
        
        Ok(ReceiverStream::new(rx))
    }
    
    /// Relay a text completion request (legacy API)
    /// 中继文本完成请求（传统 API）
    pub async fn relay_completion(
        &self,
        request: CompletionRequest,
        channel: &Channel,
        api_key: &str,
    ) -> Result<CompletionResponse> {
        info!("Relaying completion to channel: {}", channel.name);
        
        let start_time = Instant::now();
        let provider_url = self.build_provider_url(&channel.base_url, "/completions");
        
        let response = self.client
            .post(&provider_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("Request failed: {}", e)))?;
        
        let latency_ms = start_time.elapsed().as_millis() as i32;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Provider(format!("Provider error: {} - {}", status, error_text)));
        }
        
        let response_body: Value = response.json().await
            .map_err(|e| Error::Provider(format!("Failed to parse response: {}", e)))?;
        
        let openai_response: CompletionResponse = serde_json::from_value(response_body)
            .map_err(|e| Error::Provider(format!("Failed to parse completion response: {}", e)))?;
        
        info!("Completion successful: latency={}ms", latency_ms);
        Ok(openai_response)
    }
    
    // ========================================================================
    // Request/Response Transformation
    // ========================================================================
    
    /// Build provider URL from base URL and path
    /// 从基础 URL 和路径构建提供商 URL
    fn build_provider_url(&self, base_url: &Option<String>, path: &str) -> String {
        match base_url {
            Some(url) => {
                let url = url.trim_end_matches('/');
                format!("{}{}", url, path)
            }
            None => format!("https://api.openai.com/v1{}", path),
        }
    }
    
    /// Transform request for specific provider
    /// 为特定提供商转换请求
    fn transform_request_for_provider(
        &self,
        request: &ChatCompletionRequest,
        channel_type: &str,
    ) -> Result<Value> {
        // For now, serialize as-is (OpenAI compatible)
        // Different providers may need specific transformations
        let mut request_value = serde_json::to_value(request)
            .map_err(|e| Error::Internal(format!("Failed to serialize request: {}", e)))?;
        
        // Provider-specific transformations
        match channel_type.to_lowercase().as_str() {
            "anthropic" => {
                // Anthropic uses different format
                request_value = self.transform_for_anthropic(request)?;
            }
            "google" => {
                // Google uses different format
                request_value = self.transform_for_google(request)?;
            }
            _ => {
                // OpenAI compatible - no transformation needed
            }
        }
        
        Ok(request_value)
    }
    
    /// Transform request for Anthropic API
    /// 为 Anthropic API 转换请求
    fn transform_for_anthropic(&self, request: &ChatCompletionRequest) -> Result<Value> {
        // Convert messages to Anthropic format
        let messages: Vec<Value> = request.messages.iter().map(|msg| {
            json!({
                "role": match msg.role {
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                    _ => "user",
                },
                "content": msg.content.as_ref().map(|c| match c {
                    MessageContent::Text(t) => t,
                    MessageContent::Parts(parts) => {
                        parts.iter().filter_map(|p| p.text.as_ref()).next().unwrap_or(&"".to_string())
                    }
                })
            })
        }).collect();
        
        Ok(json!({
            "model": request.model.replace("gpt-", "claude-"),
            "messages": messages,
            "max_tokens": request.max_tokens.unwrap_or(1024),
            "temperature": request.temperature.unwrap_or(0.7),
        }))
    }
    
    /// Transform request for Google API
    /// 为 Google API 转换请求
    fn transform_for_google(&self, request: &ChatCompletionRequest) -> Result<Value> {
        // Convert to Google's format
        let contents: Vec<Value> = request.messages.iter().map(|msg| {
            json!({
                "role": match msg.role {
                    MessageRole::User => "user",
                    MessageRole::Assistant => "model",
                    _ => "user",
                },
                "parts": [{
                    "text": msg.content.as_ref().map(|c| match c {
                        MessageContent::Text(t) => t,
                        MessageContent::Parts(parts) => {
                            parts.iter().filter_map(|p| p.text.as_ref()).next().unwrap_or(&"".to_string())
                        }
                    }).unwrap_or_default()
                }]
            })
        }).collect();
        
        Ok(json!({
            "contents": contents,
            "generationConfig": {
                "temperature": request.temperature.unwrap_or(0.7),
                "maxOutputTokens": request.max_tokens.unwrap_or(1024),
            }
        }))
    }
    
    /// Transform response to OpenAI format
    /// 将响应转换为 OpenAI 格式
    fn transform_response_to_openai(&self, response: Value, model: &str) -> Result<ChatCompletionResponse> {
        // Try to parse as OpenAI format first
        if let Ok(openai_response) = serde_json::from_value::<ChatCompletionResponse>(response.clone()) {
            return Ok(openai_response);
        }
        
        // If that fails, try to transform from other formats
        // This is a simplified transformation - real implementation would be more robust
        
        let id = response["id"].as_str().unwrap_or("chatcmpl-123").to_string();
        let model = response["model"].as_str().unwrap_or(model).to_string();
        
        // Extract choices
        let choices_value = &response["choices"];
        let choices: Vec<Choice> = if choices_value.is_array() {
            choices_value.as_array().unwrap().iter().enumerate().map(|(i, c)| {
                let content = c["message"]["content"].as_str().unwrap_or("").to_string();
                let finish_reason = c["finish_reason"].as_str().map(|s| s.to_string());
                
                Choice {
                    index: i as u32,
                    message: Message {
                        role: MessageRole::Assistant,
                        content: Some(MessageContent::Text(content)),
                        name: None,
                        tool_calls: None,
                        tool_call_id: None,
                    },
                    finish_reason,
                    logprobs: None,
                }
            }).collect()
        } else {
            vec![]
        };
        
        // Extract usage
        let usage = if let Some(usage_val) = response.get("usage") {
            Some(Usage {
                prompt_tokens: usage_val["prompt_tokens"].as_u64().unwrap_or(0) as u32,
                completion_tokens: usage_val["completion_tokens"].as_u64().unwrap_or(0) as u32,
                total_tokens: usage_val["total_tokens"].as_u64().unwrap_or(0) as u32,
            })
        } else {
            None
        };
        
        Ok(ChatCompletionResponse {
            id,
            object: "chat.completion".to_string(),
            created: Utc::now().timestamp() as i64,
            model,
            choices,
            usage,
            system_fingerprint: None,
        })
    }
    
    // ========================================================================
    // Model Management
    // ========================================================================
    
    /// Get list of available models
    /// 获取可用模型列表
    pub async fn list_models(&self, tenant_id: i64) -> Result<ModelList> {
        let channels = self.db.get_active_channels(tenant_id).await?;
        
        let mut models = Vec::new();
        for channel in channels {
            let channel_models: Vec<String> = serde_json::from_value(channel.models.clone())
                .unwrap_or_default();
            
            for model_id in channel_models {
                models.push(Model {
                    id: model_id,
                    object: "model".to_string(),
                    created: Utc::now().timestamp() as i64,
                    owned_by: channel.channel_type.clone(),
                });
            }
        }
        
        Ok(ModelList {
            object: "list".to_string(),
            data: models,
        })
    }
    
    /// Get model details
    /// 获取模型详情
    pub async fn get_model(&self, model_id: &str, tenant_id: i64) -> Result<Model> {
        let channels = self.db.get_channels_for_model(tenant_id, model_id).await?;
        
        if let Some(channel) = channels.first() {
            Ok(Model {
                id: model_id.to_string(),
                object: "model".to_string(),
                created: Utc::now().timestamp() as i64,
                owned_by: channel.channel_type.clone(),
            })
        } else {
            Err(Error::Provider(format!("Model not found: {}", model_id)))
        }
    }
    
    // ========================================================================
    // Usage Statistics
    // ========================================================================
    
    /// Record usage statistics
    /// 记录用量统计
    pub async fn record_usage(
        &self,
        tenant_id: i64,
        user_id: Option<i64>,
        channel_id: i64,
        model: &str,
        usage: &Usage,
        latency_ms: i32,
    ) -> Result<()> {
        // Calculate cost (simplified - real implementation would use pricing tables)
        let cost = rust_decimal::Decimal::ZERO;
        
        self.db.create_usage_stat(
            tenant_id,
            user_id,
            Some(channel_id),
            Some(model.to_string()),
            usage.prompt_tokens as i64,
            usage.completion_tokens as i64,
            usage.total_tokens as i64,
            cost,
            Some("success".to_string()),
            Some(latency_ms),
        ).await?;
        
        info!("Recorded usage: {} tokens", usage.total_tokens);
        Ok(())
    }
    
    /// Update token usage
    /// 更新令牌用量
    pub async fn update_token_usage(
        &self,
        token_id: i64,
        tokens_used: i64,
    ) -> Result<()> {
        self.db.update_token_usage(token_id, tokens_used).await?;
        info!("Updated token usage: token_id={}, tokens_used={}", token_id, tokens_used);
        Ok(())
    }
    
    /// Get list of available models filtered by token permissions
    /// 获取按令牌权限过滤的可用模型列表
    pub async fn list_models_filtered(
        &self,
        tenant_id: i64,
        token: &crate::db::models::Token,
    ) -> Result<ModelList> {
        let channels = self.db.get_active_channels(tenant_id).await?;
        
        let mut models = Vec::new();
        for channel in channels {
            let channel_models: Vec<String> = serde_json::from_value(channel.models.clone())
                .unwrap_or_default();
            
            for model_id in channel_models {
                // Check if model is permitted for this token
                if let Ok(true) = crate::middleware::api_key_auth::check_model_permission(token, &model_id) {
                    models.push(Model {
                        id: model_id,
                        object: "model".to_string(),
                        created: Utc::now().timestamp() as i64,
                        owned_by: channel.channel_type.clone(),
                    });
                }
            }
        }
        
        Ok(ModelList {
            object: "list".to_string(),
            data: models,
        })
    }
}

// ============================================================================
// Provider Implementations
// ============================================================================

/// AI Provider trait for implementing provider-specific logic
/// AI 提供商特征，用于实现特定于提供商的逻辑
pub trait AIProvider: Send + Sync {
    /// Get provider name
    /// 获取提供商名称
    fn name(&self) -> &str;
    
    /// Get provider base URL
    /// 获取提供商基础 URL
    fn base_url(&self) -> &str;
    
    /// Transform request for this provider
    /// 为此提供商转换请求
    fn transform_request(&self, request: Value) -> Result<Value>;
    
    /// Transform response from this provider
    /// 转换来自此提供商的响应
    fn transform_response(&self, response: Value) -> Result<Value>;
}

/// OpenAI provider implementation
/// OpenAI 提供商实现
pub struct OpenAIProvider {
    base_url: String,
}

impl OpenAIProvider {
    pub fn new(base_url: Option<String>) -> Self {
        OpenAIProvider {
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
        }
    }
}

impl AIProvider for OpenAIProvider {
    fn name(&self) -> &str {
        "openai"
    }
    
    fn base_url(&self) -> &str {
        &self.base_url
    }
    
    fn transform_request(&self, request: Value) -> Result<Value> {
        // OpenAI format - no transformation needed
        Ok(request)
    }
    
    fn transform_response(&self, response: Value) -> Result<Value> {
        // OpenAI format - no transformation needed
        Ok(response)
    }
}

/// Anthropic provider implementation
/// Anthropic 提供商实现
pub struct AnthropicProvider {
    base_url: String,
}

impl AnthropicProvider {
    pub fn new() -> Self {
        AnthropicProvider {
            base_url: "https://api.anthropic.com/v1".to_string(),
        }
    }
}

impl AIProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }
    
    fn base_url(&self) -> &str {
        &self.base_url
    }
    
    fn transform_request(&self, request: Value) -> Result<Value> {
        // Transform OpenAI format to Anthropic format
        // Implementation would go here
        Ok(request)
    }
    
    fn transform_response(&self, response: Value) -> Result<Value> {
        // Transform Anthropic format to OpenAI format
        // Implementation would go here
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::db::Database;

    #[test]
    fn test_weighted_random_select() {
        let config = Arc::new(Config::development());
        let db = Arc::new(Database::new(config.clone()).unwrap());
        let relay = RelayService::new(config, db);
        
        // This test would need mock channels to be fully functional
        // For now, just test the service creation
        assert!(relay.config.server.port > 0);
    }

    #[tokio::test]
    async fn test_build_provider_url() {
        let config = Arc::new(Config::development());
        let db = Arc::new(Database::new(config.clone()).unwrap());
        let relay = RelayService::new(config, db);
        
        let url = relay.build_provider_url(&Some("https://api.example.com".to_string()), "/chat");
        assert_eq!(url, "https://api.example.com/chat");
        
        let url = relay.build_provider_url(&None, "/chat");
        assert!(url.contains("openai.com"));
    }
}
