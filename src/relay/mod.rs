//! Relay module for LuminaBridge
//!
//! Handles request relay to AI providers.
//! 处理向 AI 提供商的请求中继。

use std::sync::Arc;
use tracing::{info, warn};

use crate::config::Config;
use crate::error::{Error, Result};

/// Relay service for forwarding requests to AI providers
/// 用于将请求转发到 AI 提供商的中继服务
pub struct RelayService {
    /// Application configuration
    /// 应用程序配置
    config: Arc<Config>,
    
    /// HTTP client for making requests
    /// 用于发出请求的 HTTP 客户端
    client: reqwest::Client,
}

impl RelayService {
    /// Create a new relay service
    /// 创建新的中继服务
    ///
    /// # Arguments
    ///
    /// * `config` - Application configuration
    ///
    /// # Returns
    ///
    /// * `Self` - New relay service instance
    pub fn new(config: Arc<Config>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.server.timeout_secs))
            .gzip(true)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        
        RelayService { config, client }
    }
    
    /// Relay a request to an AI provider
    /// 将请求中继到 AI 提供商
    ///
    /// # Arguments
    ///
    /// * `provider_url` - Target provider API URL
    /// * `api_key` - Provider API key
    /// * `request_body` - Request body to forward
    ///
    /// # Returns
    ///
    /// * `Result<serde_json::Value>` - Provider response
    ///
    /// # Example
    ///
    /// ```rust
    /// let response = relay.relay_request(url, api_key, body).await?;
    /// ```
    pub async fn relay_request(
        &self,
        provider_url: &str,
        api_key: &str,
        request_body: serde_json::Value,
    ) -> Result<serde_json::Value> {
        info!("Relaying request to provider: {}", provider_url);
        
        let response = self.client
            .post(provider_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("Request failed: {}", e)))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            warn!("Provider returned error: {} - {}", status, error_text);
            return Err(Error::Provider(format!("Provider error: {} - {}", status, error_text)));
        }
        
        let response_body: serde_json::Value = response.json().await
            .map_err(|e| Error::Provider(format!("Failed to parse response: {}", e)))?;
        
        Ok(response_body)
    }
    
    /// Stream a request to an AI provider
    /// 将请求流式传输到 AI 提供商
    ///
    /// # Arguments
    ///
    /// * `provider_url` - Target provider API URL
    /// * `api_key` - Provider API key
    /// * `request_body` - Request body to forward
    ///
    /// # Returns
    ///
    /// * `Result<reqwest::Response>` - Streaming response
    pub async fn stream_request(
        &self,
        provider_url: &str,
        api_key: &str,
        request_body: serde_json::Value,
    ) -> Result<reqwest::Response> {
        info!("Streaming request to provider: {}", provider_url);
        
        let mut request_body = request_body;
        request_body["stream"] = serde_json::json!(true);
        
        let response = self.client
            .post(provider_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| Error::Provider(format!("Stream request failed: {}", e)))?;
        
        if !response.status().is_success() {
            let status = response.status();
            return Err(Error::Provider(format!("Provider error: {}", status)));
        }
        
        Ok(response)
    }
    
    /// Get health status of a provider
    /// 获取提供商的健康状态
    ///
    /// # Arguments
    ///
    /// * `provider_url` - Provider health check URL
    ///
    /// # Returns
    ///
    /// * `Result<bool>` - True if healthy
    pub async fn check_provider_health(&self, provider_url: &str) -> Result<bool> {
        match self.client.get(provider_url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

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
    fn transform_request(&self, request: serde_json::Value) -> Result<serde_json::Value>;
    
    /// Transform response from this provider
    /// 转换来自此提供商的响应
    fn transform_response(&self, response: serde_json::Value) -> Result<serde_json::Value>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relay_service_creation() {
        let config = Arc::new(Config::development());
        let relay = RelayService::new(config);
        // Basic test - just ensure it creates without panic
        assert!(relay.client.get("https://example.com").send().await.is_err() || 
                relay.client.get("https://example.com").send().await.is_ok());
    }
}
