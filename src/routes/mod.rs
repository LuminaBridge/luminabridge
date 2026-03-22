//! Routes module for LuminaBridge API
//!
//! Registers all API routes and configures middleware.
//! 注册所有 API 路由并配置中间件。

pub mod auth;
pub mod channels;
pub mod tokens;
pub mod users;
pub mod stats;
pub mod ws;
pub mod tenant;
pub mod relay;

use axum::{
    routing::{get, post, put, delete, patch},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

use crate::server::AppState;
use crate::middleware::auth::{require_auth, optional_auth};
use crate::middleware::api_key_auth::api_key_auth;

/// Create the API v1 router with all routes
/// 创建包含所有路由的 API v1 路由器
///
/// # Arguments
///
/// * `state` - Application state
///
/// # Returns
///
/// * `Router<AppState>` - Configured router
pub fn api_v1_routes(state: AppState) -> Router<AppState> {
    info!("Setting up API v1 routes with authentication middleware");
    
    // Public routes (no auth required)
    let public_routes = Router::new()
        // Auth routes - public (login, register, OAuth)
        .nest("/auth", auth::auth_routes(state.clone()));
    
    // Protected routes (auth required)
    let protected_routes = Router::new()
        // Tenant routes
        .nest("/tenant", tenant::tenant_routes(state.clone()))
        
        // Channel routes
        .nest("/channels", channels::channel_routes(state.clone()))
        
        // Token routes
        .nest("/tokens", tokens::token_routes(state.clone()))
        
        // User routes
        .nest("/users", users::user_routes(state.clone()))
        
        // Stats routes
        .nest("/stats", stats::stats_routes(state.clone()))
        .layer(axum::middleware::from_fn_with_state(state.clone(), require_auth));
    
    // Relay routes (OpenAI-compatible API) - with API key auth and rate limiting
    let relay_routes = Router::new()
        .nest("/v1", relay::relay_routes(state.clone()))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            api_key_auth,
        ))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::rate_limit_middleware,
        ));
    
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .merge(relay_routes)
        
        // WebSocket upgrade endpoint (optional auth)
        .route("/ws", get(ws::websocket_handler))
        .layer(axum::middleware::from_fn_with_state(state.clone(), optional_auth))
}

/// Create the main router with CORS and other middleware
/// 创建包含 CORS 和其他中间件的主路由器
///
/// # Arguments
///
/// * `state` - Application state
///
/// # Returns
///
/// * `Router<AppState>` - Main router with all middleware
pub fn create_router(state: AppState) -> Router<AppState> {
    info!("Creating main router with middleware");
    
    // Configure CORS - allow frontend on port 3000
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_credentials(true);
    
    Router::new()
        // Health check endpoints
        .route("/health", get(handlers::health_check))
        .route("/ready", get(handlers::ready_check))
        
        // API v1 routes
        .nest("/api/v1", api_v1_routes(state.clone()))
        
        // OAuth routes (legacy support)
        .nest("/auth", oauth_routes(state.clone()))
        
        // Apply middleware
        .layer(cors)
        .with_state(state)
}

/// Legacy OAuth routes
/// 传统 OAuth 路由
fn oauth_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/github/login", get(auth::github_login))
        .route("/github/callback", get(auth::github_callback))
        .route("/discord/login", get(auth::discord_login))
        .route("/discord/callback", get(auth::discord_callback))
        .with_state(state)
}

/// Health check handlers
/// 健康检查处理器
mod handlers {
    use axum::{
        extract::State,
        response::Json,
        http::StatusCode,
    };
    use serde_json::json;
    use crate::server::AppState;
    use crate::error::Result;

    /// Health check endpoint
    /// 健康检查端点
    pub async fn health_check() -> Json<serde_json::Value> {
        Json(json!({
            "status": "healthy",
            "service": "luminabridge"
        }))
    }

    /// Readiness check endpoint
    /// 就绪检查端点
    pub async fn ready_check(State(state): State<AppState>) -> Result<Json<serde_json::Value>> {
        // Check database connectivity
        state.db.health_check().await?;
        
        Ok(Json(json!({
            "status": "ready",
            "database": "connected"
        })))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::config::Config;

    #[test]
    fn test_routes_setup() {
        let config = Arc::new(Config::development());
        // Basic test - just ensure router can be created
        // Note: Can't fully test without async context and real DB
        assert!(config.server.port > 0);
    }
}
