//! Relay routes for OpenAI-compatible API
//!
//! Implements chat completions, completions, and models endpoints.
//! 实现聊天完成、完成和模型端点。

use axum::{
    extract::{State, Path, Extension},
    response::{Json, Response},
    http::{StatusCode, header},
    routing::{get, post},
    Router,
};
use serde_json::json;
use tokio_stream::StreamExt;
use tracing::{info, warn, error};

use crate::server::AppState;
use crate::error::{Error, Result};
use crate::relay::RelayService;
use crate::relay::types::{
    ChatCompletionRequest, CompletionRequest,
    ChatCompletionResponse, CompletionResponse,
};
use crate::middleware::api_key_auth::{ApiKeyAuthExtension, check_model_permission};

/// Create relay routes
/// 创建中继路由
pub fn relay_routes(state: AppState) -> Router<AppState> {
    Router::new()
        // Chat completions
        .route("/chat/completions", post(chat_completions_handler))
        
        // Text completions (legacy)
        .route("/completions", post(completions_handler))
        
        // Models
        .route("/models", get(list_models_handler))
        .route("/models/:id", get(get_model_handler))
        
        .with_state(state)
}

/// Chat completions endpoint
/// 聊天完成端点
/// 
/// POST /v1/chat/completions
#[axum::debug_handler]
pub async fn chat_completions_handler(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuthExtension>,
    body: Json<ChatCompletionRequest>,
) -> Result<Response> {
    info!("Received chat completions request for model: {}", body.model);
    
    let token = &auth.token;
    let tenant_id = token.tenant_id;
    let api_key = &token.key;
    
    // Check model permission
    if !check_model_permission(token, &body.model)? {
        return Err(Error::ModelNotPermitted);
    }
    
    // Get relay service
    let relay = RelayService::new(state.config.clone(), state.db.clone());
    
    // Select channel based on model
    let channel = relay.select_channel(&body.model, tenant_id).await?;
    
    let request = body.into_inner();
    
    // Check if streaming is requested
    if request.stream.unwrap_or(false) {
        // Handle streaming response
        handle_streaming_completion(relay, request, &channel, &api_key, token.id).await
    } else {
        // Handle regular response
        handle_regular_completion(relay, request, &channel, &api_key, tenant_id, token.id).await
    }
}

/// Handle regular (non-streaming) chat completion
/// 处理常规（非流式）聊天完成
async fn handle_regular_completion(
    relay: RelayService,
    request: ChatCompletionRequest,
    channel: &crate::db::models::Channel,
    api_key: &str,
    tenant_id: i64,
    token_id: i64,
) -> Result<Response> {
    let start_time = std::time::Instant::now();
    
    // Relay the request
    let response = relay.relay_chat_completion(request, channel, api_key).await?;
    
    let latency_ms = start_time.elapsed().as_millis() as i32;
    
    // Record usage statistics and update token quota
    if let Some(usage) = &response.usage {
        let total_tokens = usage.total_tokens as i64;
        
        // Update token usage
        let _ = relay.update_token_usage(token_id, total_tokens).await;
        
        // Record usage statistics
        let _ = relay.record_usage(
            tenant_id,
            None, // user_id would come from auth context
            channel.id,
            &response.model,
            usage,
            latency_ms,
        ).await;
    }
    
    // Convert to axum response
    let response_json = serde_json::to_value(&response)
        .map_err(|e| Error::Internal(format!("Failed to serialize response: {}", e)))?;
    
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(response_json),
    ).into_response())
}

/// Handle streaming chat completion
/// 处理流式聊天完成
/// 
/// Note: Token usage tracking for streaming responses is not yet implemented.
/// The token quota is checked before the stream starts, but usage is not updated
/// after completion. This should be implemented by accumulating token counts from
/// stream chunks and calling `relay.update_token_usage()` after the stream completes.
/// 
/// 注意：流式响应的令牌用量跟踪尚未实现。
/// 令牌配额在流开始前检查，但完成后不更新用量。
/// 应该通过累积流块中的令牌计数并在流完成后调用 `relay.update_token_usage()` 来实现。
async fn handle_streaming_completion(
    relay: RelayService,
    request: ChatCompletionRequest,
    channel: &crate::db::models::Channel,
    api_key: &str,
    token_id: i64,
) -> Result<Response> {
    use axum::body::Body;
    use tokio_stream::wrappers::ReceiverStream;
    use futures_util::stream::Stream;
    use std::convert::Infallible;
    
    info!("Handling streaming completion (token usage tracking not implemented for streaming)");
    
    // TODO: Implement token usage tracking for streaming
    // This would require:
    // 1. Accumulating token counts from stream chunks
    // 2. Calling relay.update_token_usage() after stream completes
    // 3. Handling stream errors gracefully
    
    // Create the stream
    let stream = relay.stream_chat_completion(request, channel, api_key).await?;
    
    // Convert to SSE stream
    let sse_stream = stream.map(|chunk_result| {
        match chunk_result {
            Ok(chunk) => {
                let data = serde_json::to_string(&chunk).unwrap_or_default();
                Ok::<_, Infallible>(format!("data: {}\n\n", data))
            }
            Err(e) => {
                warn!("Stream error: {}", e);
                Ok(format!("data: {{\"error\": \"{}\"}}\n\n", e))
            }
        }
    });
    
    // Add final [DONE] marker
    let done_stream = tokio_stream::once(async { Ok::<_, Infallible>("data: [DONE]\n\n".to_string()) });
    
    let combined_stream = sse_stream.chain(done_stream);
    
    let body = Body::from_stream(combined_stream);
    
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/event-stream"),
            (header::CACHE_CONTROL, "no-cache"),
            (header::CONNECTION, "keep-alive"),
        ],
        body,
    ).into_response())
}

/// Text completions endpoint (legacy OpenAI API)
/// 文本完成端点（传统 OpenAI API）
/// 
/// POST /v1/completions
#[axum::debug_handler]
pub async fn completions_handler(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuthExtension>,
    body: Json<CompletionRequest>,
) -> Result<Json<CompletionResponse>> {
    info!("Received completions request for model: {}", body.model);
    
    let token = &auth.token;
    let tenant_id = token.tenant_id;
    let api_key = &token.key;
    
    // Check model permission
    if !check_model_permission(token, &body.model)? {
        return Err(Error::ModelNotPermitted);
    }
    
    // Get relay service
    let relay = RelayService::new(state.config.clone(), state.db.clone());
    
    // Select channel
    let channel = relay.select_channel(&body.model, tenant_id).await?;
    
    let request = body.into_inner();
    
    // Relay the request
    let response = relay.relay_completion(request, &channel, &api_key).await?;
    
    // Update token usage
    if let Some(usage) = &response.usage {
        let _ = relay.update_token_usage(token.id, usage.total_tokens as i64).await;
    }
    
    Ok(Json(response))
}

/// List models endpoint
/// 列出模型端点
/// 
/// GET /v1/models
#[axum::debug_handler]
pub async fn list_models_handler(
    State(state): State<AppState>,
    Extension(auth): Extension<ApiKeyAuthExtension>,
) -> Result<Json<serde_json::Value>> {
    info!("Received list models request");
    
    let token = &auth.token;
    let tenant_id = token.tenant_id;
    
    // Get relay service
    let relay = RelayService::new(state.config.clone(), state.db.clone());
    
    // Get model list (filtered by token permissions)
    let model_list = relay.list_models_filtered(tenant_id, token).await?;
    
    Ok(Json(serde_json::to_value(model_list)?))
}

/// Get model details endpoint
/// 获取模型详情端点
/// 
/// GET /v1/models/:id
#[axum::debug_handler]
pub async fn get_model_handler(
    State(state): State<AppState>,
    Path(model_id): Path<String>,
    Extension(auth): Extension<ApiKeyAuthExtension>,
) -> Result<Json<serde_json::Value>> {
    info!("Received get model request for: {}", model_id);
    
    let token = &auth.token;
    let tenant_id = token.tenant_id;
    
    // Check model permission
    if !check_model_permission(token, &model_id)? {
        return Err(Error::ModelNotPermitted);
    }
    
    // Get relay service
    let relay = RelayService::new(state.config.clone(), state.db.clone());
    
    // Get model details
    let model = relay.get_model(&model_id, tenant_id).await?;
    
    Ok(Json(serde_json::to_value(model)?))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Extract API key from Authorization header
/// 从 Authorization 头提取 API 密钥
fn extract_api_key(headers: &axum::http::HeaderMap) -> Result<String> {
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| Error::Auth("Missing Authorization header".to_string()))?;
    
    if !auth_header.starts_with("Bearer ") {
        return Err(Error::Auth("Invalid Authorization header format".to_string()));
    }
    
    Ok(auth_header[7..].to_string())
}

/// Extract tenant ID from auth context
/// 从认证上下文提取租户 ID
/// 
/// This function is now deprecated - tenant ID should come from the API key auth middleware
/// 此函数现已弃用 - 租户 ID 应来自 API 密钥认证中间件
#[deprecated(since = "0.2.0", note = "Use Extension<ApiKeyAuthExtension> instead")]
async fn extract_tenant_id(
    state: &AppState,
    headers: &axum::http::HeaderMap,
) -> Result<i64> {
    // Simplified implementation - real implementation would:
    // 1. Validate the API key against the database
    // 2. Extract tenant ID from the token/key
    // 3. Return the tenant ID
    
    // For now, return a default tenant ID
    // This should be replaced with proper auth middleware
    Ok(1)
}

// ============================================================================
// Error Response Helpers
// ============================================================================

/// Create error response
/// 创建错误响应
fn error_response(status: StatusCode, code: &str, message: &str) -> Json<serde_json::Value> {
    Json(json!({
        "error": {
            "code": code,
            "message": message,
            "type": "luminabridge_error"
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::config::Config;
    use crate::db::Database;

    #[test]
    fn test_extract_api_key() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            "Bearer sk-test123".parse().unwrap(),
        );
        
        let api_key = extract_api_key(&headers).unwrap();
        assert_eq!(api_key, "sk-test123");
    }

    #[test]
    fn test_extract_api_key_missing() {
        let headers = axum::http::HeaderMap::new();
        let result = extract_api_key(&headers);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_response() {
        let response = error_response(
            StatusCode::BAD_REQUEST,
            "INVALID_REQUEST",
            "Test error"
        );
        
        let value = response.0;
        assert!(value["error"]["code"].as_str().is_some());
        assert!(value["error"]["message"].as_str().is_some());
    }
}
