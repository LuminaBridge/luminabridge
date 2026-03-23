//! Server module for LuminaBridge
//!
//! Handles HTTP server setup and request routing.
//! 处理 HTTP 服务器设置和请求路由。

use axum::{
    routing::{get, post},
    Router,
    extract::State,
    response::Json,
    http::StatusCode,
};
use serde_json::json;
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
    compression::CompressionLayer,
};
use tracing::info;

use crate::config::Config;
use crate::db::Database;
use crate::error::{Error, Result};

/// Application state shared across handlers
/// 在处理器之间共享的应用程序状态
#[derive(Clone)]
pub struct AppState {
    /// Application configuration
    /// 应用程序配置
    pub config: Arc<Config>,
    
    /// Database connection pool
    /// 数据库连接池
    pub db: Arc<Database>,
}

/// HTTP Server for LuminaBridge
/// LuminaBridge 的 HTTP 服务器
pub struct Server {
    /// Server configuration
    /// 服务器配置
    pub config: Arc<Config>,
    
    /// Database connection
    /// 数据库连接
    pub db: Arc<Database>,
}

impl Server {
    /// Create a new server instance
    /// 创建新的服务器实例
    ///
    /// # Arguments
    ///
    /// * `config` - Application configuration
    /// * `db` - Database connection
    ///
    /// # Returns
    ///
    /// * `Server` - New server instance
    pub fn new(config: Arc<Config>, db: Arc<Database>) -> Self {
        Server { config, db }
    }
    
    /// Build the Axum router with all routes
    /// 构建包含所有路由的 Axum 路由器
    ///
    /// # Returns
    ///
    /// * `Router` - Configured router
    fn build_router(&self) -> Router {
        let state = AppState {
            config: self.config.clone(),
            db: self.db.clone(),
        };
        
        // Configure CORS
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);
        
        // Build router with routes
        Router::new()
            // Health check endpoints
            .route("/health", get(health_check))
            .route("/ready", get(ready_check))
            
            // API routes
            .nest("/api/v1", api_routes(state.clone()))
            
            // OAuth routes
            .nest("/auth", oauth_routes(state.clone()))
            
            // Apply middleware
            .layer(cors)
            .layer(TraceLayer::new_for_http())
            .layer(CompressionLayer::new())
            .with_state(state.clone())
    }
    
    /// Run the server
    /// 运行服务器
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok on success, Error on failure
    ///
    /// # Example
    ///
    /// ```rust
    /// server.run().await?;
    /// ```
    pub async fn run(&self) -> Result<()> {
        let router = self.build_router();
        
        let addr = format!("{}:{}", self.config.server.host, self.config.server.port);
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(|e| Error::Server(format!("Failed to bind to {}: {}", addr, e)))?;
        
        info!("🌉 Server listening on http://{}", addr);
        
        axum::serve(listener, router)
            .await
            .map_err(|e| Error::Server(format!("Server error: {}", e)))
    }
}

/// Health check endpoint
/// 健康检查端点
async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "luminabridge"
    }))
}

/// Readiness check endpoint
/// 就绪检查端点
async fn ready_check(State(state): State<AppState>) -> Result<Json<serde_json::Value>> {
    // Check database connectivity
    state.db.health_check().await?;
    
    Ok(Json(json!({
        "status": "ready",
        "database": "connected"
    })))
}

/// API routes
/// API 路由
fn api_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/chat/completions", post(chat_completions))
        .route("/models", get(list_models))
        .with_state(state)
}

/// OAuth routes
/// OAuth 路由
fn oauth_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/github/login", get(github_login))
        .route("/github/callback", get(github_callback))
        .route("/discord/login", get(discord_login))
        .route("/discord/callback", get(discord_callback))
        .with_state(state)
}

/// Chat completions endpoint (OpenAI-compatible)
/// 聊天完成端点（OpenAI 兼容）
async fn chat_completions(
    State(_state): State<AppState>,
    // body: Json<ChatCompletionRequest>,
) -> Result<Json<serde_json::Value>> {
    // TODO: Implement chat completions logic
    Ok(Json(json!({
        "id": "chatcmpl-123",
        "object": "chat.completion",
        "created": 1234567890,
        "model": "gpt-3.5-turbo",
        "choices": []
    })))
}

/// List available models
/// 列出可用模型
async fn list_models(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    Ok(Json(json!({
        "object": "list",
        "data": [
            {
                "id": "gpt-3.5-turbo",
                "object": "model",
                "owned_by": "openai"
            }
        ]
    })))
}

/// GitHub OAuth login
/// GitHub OAuth 登录
async fn github_login(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    // TODO: Implement GitHub OAuth flow
    Ok(Json(json!({
        "authorize_url": "https://github.com/login/oauth/authorize"
    })))
}

/// GitHub OAuth callback
/// GitHub OAuth 回调
async fn github_callback(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    // TODO: Implement GitHub OAuth callback
    Ok(Json(json!({
        "status": "authenticated"
    })))
}

/// Discord OAuth login
/// Discord OAuth 登录
async fn discord_login(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    // TODO: Implement Discord OAuth flow
    Ok(Json(json!({
        "authorize_url": "https://discord.com/api/oauth2/authorize"
    })))
}

/// Discord OAuth callback
/// Discord OAuth 回调
async fn discord_callback(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    // TODO: Implement Discord OAuth callback
    Ok(Json(json!({
        "status": "authenticated"
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let config = Arc::new(Config::development());
        // Note: Can't create Database without async context
        // This is just a basic test
        assert_eq!(config.server.port, 3000);
    }
}
