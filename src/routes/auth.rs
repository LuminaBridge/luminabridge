//! Authentication routes for LuminaBridge API
//!
//! Handles user authentication, login, logout, and OAuth flows.
//! 处理用户认证、登录、登出和 OAuth 流程。

use axum::{
    routing::{get, post},
    Router,
    extract::{State, Json, Query},
    response::{Json as ResponseJson, IntoResponse},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::server::AppState;
use crate::error::{Error, Result};
use crate::types::{SuccessResponse, ErrorResponse, ErrorCode};
use crate::auth::{AuthService, TokenClaims};
use crate::db::{Database, UserRepository, User};

/// Create authentication routes
/// 创建认证路由
pub fn auth_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/refresh", post(refresh_token))
        .route("/register", post(register))
        .route("/oauth/github", get(github_oauth))
        .route("/oauth/github/callback", get(github_callback))
        .route("/oauth/discord", get(discord_oauth))
        .route("/oauth/discord/callback", get(discord_callback))
        .with_state(state)
}

/// Login request
/// 登录请求
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    /// User email
    /// 用户电子邮件
    pub email: String,
    
    /// User password
    /// 用户密码
    pub password: String,
    
    /// Remember me flag
    /// 记住我标志
    #[serde(default)]
    pub remember_me: bool,
}

/// Login response
/// 登录响应
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    /// JWT access token
    /// JWT 访问令牌
    pub token: String,
    
    /// Refresh token
    /// 刷新令牌
    pub refresh_token: String,
    
    /// User information
    /// 用户信息
    pub user: UserDTO,
}

/// User DTO (Data Transfer Object)
/// 用户数据传输对象
#[derive(Debug, Serialize, Clone)]
pub struct UserDTO {
    /// User ID
    /// 用户 ID
    pub id: i64,
    
    /// User email
    /// 用户电子邮件
    pub email: String,
    
    /// Display name
    /// 显示名称
    pub display_name: Option<String>,
    
    /// Avatar URL
    /// 头像 URL
    pub avatar_url: Option<String>,
    
    /// User role
    /// 用户角色
    pub role: String,
}

impl From<&User> for UserDTO {
    fn from(user: &User) -> Self {
        UserDTO {
            id: user.id,
            email: user.email.clone(),
            display_name: user.display_name.clone(),
            avatar_url: user.avatar_url.clone(),
            role: user.role.clone(),
        }
    }
}

/// Login handler
/// 登录处理器
///
/// POST /api/v1/auth/login
async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<ResponseJson<SuccessResponse<LoginResponse>>> {
    info!("Login attempt for email: {}", payload.email);
    
    // Validate input
    if payload.email.is_empty() || payload.password.is_empty() {
        return Err(Error::Validation("Email and password are required".to_string()));
    }
    
    // Find user by email
    let user_repo = UserRepository::new(&state.db);
    let user = user_repo.find_by_email(&payload.email).await?
        .ok_or_else(|| Error::Auth("Invalid credentials".to_string()))?;
    
    // Verify password
    // Note: In production, use argon2 or bcrypt for password verification
    if !verify_password(&payload.password, &user.password_hash)? {
        return Err(Error::Auth("Invalid credentials".to_string()));
    }
    
    // Check user status
    if user.status != "active" {
        return Err(Error::Auth("User account is not active".to_string()));
    }
    
    // Create auth service
    let auth_service = AuthService::new(state.config.oauth.clone());
    
    // Generate JWT token
    let token = auth_service.generate_token(&user)?;
    
    // Generate refresh token
    let refresh_token = generate_refresh_token(user.id)?;
    
    info!("User {} logged in successfully", user.email);
    
    Ok(ResponseJson(SuccessResponse::new(LoginResponse {
        token,
        refresh_token,
        user: UserDTO::from(&user),
    }).with_message("登录成功")))
}

/// Logout handler
/// 登出处理器
///
/// POST /api/v1/auth/logout
async fn logout(
    State(_state): State<AppState>,
) -> Result<ResponseJson<SuccessResponse<serde_json::Value>>> {
    // In a stateless JWT system, logout is handled client-side by discarding the token
    // For production, you might want to maintain a token blacklist in Redis
    
    Ok(ResponseJson(SuccessResponse::new(serde_json::json!({
        "logged_out": true
    })).with_message("登出成功")))
}

/// Refresh token request
/// 刷新令牌请求
#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    /// Refresh token
    /// 刷新令牌
    pub refresh_token: String,
}

/// Refresh token handler
/// 刷新令牌处理器
///
/// POST /api/v1/auth/refresh
async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<ResponseJson<SuccessResponse<LoginResponse>>> {
    info!("Token refresh requested");
    
    // Validate refresh token
    // In production, verify the refresh token signature and check expiration
    if payload.refresh_token.is_empty() {
        return Err(Error::Auth("Refresh token is required".to_string()));
    }
    
    // For now, extract user ID from refresh token (simplified)
    // In production, you would decode and validate the refresh token
    let user_id = extract_user_id_from_refresh_token(&payload.refresh_token)?;
    
    // Get user from database
    let user_repo = UserRepository::new(&state.db);
    let user = user_repo.find_by_id(user_id).await?
        .ok_or_else(|| Error::Auth("User not found".to_string()))?;
    
    // Create auth service
    let auth_service = AuthService::new(state.config.oauth.clone());
    
    // Generate new JWT token
    let token = auth_service.generate_token(&user)?;
    
    // Generate new refresh token
    let refresh_token = generate_refresh_token(user.id)?;
    
    Ok(ResponseJson(SuccessResponse::new(LoginResponse {
        token,
        refresh_token,
        user: UserDTO::from(&user),
    }).with_message("Token refreshed successfully")))
}

/// Register request
/// 注册请求
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    /// User email
    /// 用户电子邮件
    pub email: String,
    
    /// User password
    /// 用户密码
    pub password: String,
    
    /// Display name
    /// 显示名称
    pub display_name: Option<String>,
}

/// Register handler
/// 注册处理器
///
/// POST /api/v1/auth/register
async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<ResponseJson<SuccessResponse<LoginResponse>>> {
    info!("Registration attempt for email: {}", payload.email);
    
    // Validate input
    if payload.email.is_empty() || payload.password.is_empty() {
        return Err(Error::Validation("Email and password are required".to_string()));
    }
    
    // Check if user already exists
    let user_repo = UserRepository::new(&state.db);
    if user_repo.find_by_email(&payload.email).await?.is_some() {
        return Err(Error::Auth("User already exists".to_string()));
    }
    
    // Hash password
    let password_hash = hash_password(&payload.password)?;
    
    // Create user
    let user = user_repo.create_with_password(
        &payload.email,
        &payload.display_name.unwrap_or_else(|| payload.email.clone()),
        &password_hash,
    ).await?;
    
    // Create auth service
    let auth_service = AuthService::new(state.config.oauth.clone());
    
    // Generate JWT token
    let token = auth_service.generate_token(&user)?;
    
    // Generate refresh token
    let refresh_token = generate_refresh_token(user.id)?;
    
    info!("User {} registered successfully", user.email);
    
    Ok(ResponseJson(SuccessResponse::new(LoginResponse {
        token,
        refresh_token,
        user: UserDTO::from(&user),
    }).with_message("注册成功")))
}

/// GitHub OAuth handler
/// GitHub OAuth 处理器
///
/// GET /api/v1/auth/oauth/github
async fn github_oauth(
    State(state): State<AppState>,
) -> Result<impl IntoResponse> {
    let auth_service = AuthService::new(state.config.oauth.clone());
    
    match auth_service.get_provider("github") {
        Some(provider) => {
            let state_param = generate_oauth_state();
            let auth_url = provider.get_authorization_url(&state_param);
            
            // Store state in Redis or session for callback validation
            // For now, just redirect
            
            Ok(axum::response::Redirect::temporary(&auth_url))
        }
        None => Err(Error::Auth("GitHub OAuth is not configured".to_string())),
    }
}

/// GitHub OAuth callback handler
/// GitHub OAuth 回调处理器
///
/// GET /api/v1/auth/oauth/github/callback
async fn github_callback(
    State(state): State<AppState>,
    Query(params): Query<OAuthCallbackParams>,
) -> Result<ResponseJson<SuccessResponse<LoginResponse>>> {
    info!("GitHub OAuth callback received");
    
    let auth_service = AuthService::new(state.config.oauth.clone());
    
    let provider = auth_service.get_provider("github")
        .ok_or_else(|| Error::Auth("GitHub OAuth is not configured".to_string()))?;
    
    // Exchange code for token
    let token_response = provider.exchange_code(&params.code).await?;
    
    // Get user info
    let user_info = provider.get_user_info(&token_response.access_token).await?;
    
    // Find or create user
    let user_repo = UserRepository::new(&state.db);
    let user = match user_repo.find_by_oauth("github", &user_info.provider_id).await? {
        Some(existing_user) => existing_user,
        None => {
            // Check if email already exists
            if let Some(existing) = user_repo.find_by_email(&user_info.email).await? {
                // Link OAuth account to existing user
                user_repo.link_oauth_account(
                    existing.id,
                    "github",
                    &user_info.provider_id,
                    &token_response.access_token,
                ).await?;
                existing
            } else {
                // Create new user
                user_repo.create_with_oauth(
                    &user_info.email,
                    &user_info.name,
                    "github",
                    &user_info.provider_id,
                    &token_response.access_token,
                ).await?
            }
        }
    };
    
    // Generate JWT token
    let token = auth_service.generate_token(&user)?;
    let refresh_token = generate_refresh_token(user.id)?;
    
    Ok(ResponseJson(SuccessResponse::new(LoginResponse {
        token,
        refresh_token,
        user: UserDTO::from(&user),
    }).with_message("GitHub 登录成功")))
}

/// Discord OAuth handler
/// Discord OAuth 处理器
///
/// GET /api/v1/auth/oauth/discord
async fn discord_oauth(
    State(state): State<AppState>,
) -> Result<impl IntoResponse> {
    let auth_service = AuthService::new(state.config.oauth.clone());
    
    match auth_service.get_provider("discord") {
        Some(provider) => {
            let state_param = generate_oauth_state();
            let auth_url = provider.get_authorization_url(&state_param);
            Ok(axum::response::Redirect::temporary(&auth_url))
        }
        None => Err(Error::Auth("Discord OAuth is not configured".to_string())),
    }
}

/// Discord OAuth callback handler
/// Discord OAuth 回调处理器
///
/// GET /api/v1/auth/oauth/discord/callback
async fn discord_callback(
    State(state): State<AppState>,
    Query(params): Query<OAuthCallbackParams>,
) -> Result<ResponseJson<SuccessResponse<LoginResponse>>> {
    info!("Discord OAuth callback received");
    
    let auth_service = AuthService::new(state.config.oauth.clone());
    
    let provider = auth_service.get_provider("discord")
        .ok_or_else(|| Error::Auth("Discord OAuth is not configured".to_string()))?;
    
    // Exchange code for token
    let token_response = provider.exchange_code(&params.code).await?;
    
    // Get user info
    let user_info = provider.get_user_info(&token_response.access_token).await?;
    
    // Find or create user
    let user_repo = UserRepository::new(&state.db);
    let user = match user_repo.find_by_oauth("discord", &user_info.provider_id).await? {
        Some(existing_user) => existing_user,
        None => {
            if let Some(existing) = user_repo.find_by_email(&user_info.email).await? {
                user_repo.link_oauth_account(
                    existing.id,
                    "discord",
                    &user_info.provider_id,
                    &token_response.access_token,
                ).await?;
                existing
            } else {
                user_repo.create_with_oauth(
                    &user_info.email,
                    &user_info.name,
                    "discord",
                    &user_info.provider_id,
                    &token_response.access_token,
                ).await?
            }
        }
    };
    
    // Generate JWT token
    let token = auth_service.generate_token(&user)?;
    let refresh_token = generate_refresh_token(user.id)?;
    
    Ok(ResponseJson(SuccessResponse::new(LoginResponse {
        token,
        refresh_token,
        user: UserDTO::from(&user),
    }).with_message("Discord 登录成功")))
}

/// OAuth callback query parameters
/// OAuth 回调查询参数
#[derive(Debug, Deserialize)]
pub struct OAuthCallbackParams {
    /// OAuth authorization code
    /// OAuth 授权码
    pub code: String,
    
    /// OAuth state parameter
    /// OAuth 状态参数
    pub state: Option<String>,
    
    /// Error (if any)
    /// 错误（如果有）
    pub error: Option<String>,
}

/// Helper functions
/// 辅助函数

/// Hash a password using argon2
/// 使用 argon2 哈希密码
fn hash_password(password: &str) -> Result<String> {
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2,
    };
    
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)
        .map_err(|e| Error::Auth(format!("Failed to hash password: {}", e)))?
        .to_string();
    
    Ok(password_hash)
}

/// Verify a password against its hash
/// 验证密码与其哈希值
fn verify_password(password: &str, hash: &str) -> Result<bool> {
    use argon2::{
        password_hash::PasswordVerifier,
        Argon2,
    };
    
    let parsed_hash = argon2::password_hash::PasswordHash::new(hash)
        .map_err(|e| Error::Auth(format!("Invalid password hash: {}", e)))?;
    
    let argon2 = Argon2::default();
    let is_valid = argon2.verify_password(password.as_bytes(), &parsed_hash)
        .is_ok();
    
    Ok(is_valid)
}

/// Generate a refresh token
/// 生成刷新令牌
fn generate_refresh_token(user_id: i64) -> Result<String> {
    use uuid::Uuid;
    
    // In production, this should be a signed JWT or stored in database
    let refresh_token = format!("rt_{}_{}", user_id, Uuid::new_v4());
    Ok(refresh_token)
}

/// Extract user ID from refresh token
/// 从刷新令牌提取用户 ID
fn extract_user_id_from_refresh_token(token: &str) -> Result<i64> {
    // Simplified extraction - in production, properly validate the token
    if !token.starts_with("rt_") {
        return Err(Error::Auth("Invalid refresh token format".to_string()));
    }
    
    let parts: Vec<&str> = token.split('_').collect();
    if parts.len() < 3 {
        return Err(Error::Auth("Invalid refresh token".to_string()));
    }
    
    parts[1].parse()
        .map_err(|_| Error::Auth("Invalid user ID in refresh token".to_string()))
}

/// Generate OAuth state parameter
/// 生成 OAuth 状态参数
fn generate_oauth_state() -> String {
    use uuid::Uuid;
    Uuid::new_v4().to_string()
}

/// Legacy OAuth handlers for backward compatibility
/// 传统 OAuth 处理器用于向后兼容
pub async fn github_login(
    State(state): State<AppState>,
) -> Result<impl IntoResponse> {
    github_oauth(State(state)).await
}

pub async fn github_callback_legacy(
    State(state): State<AppState>,
    Query(params): Query<OAuthCallbackParams>,
) -> Result<ResponseJson<SuccessResponse<LoginResponse>>> {
    github_callback(State(state), Query(params)).await
}

pub async fn discord_login(
    State(state): State<AppState>,
) -> Result<impl IntoResponse> {
    discord_oauth(State(state)).await
}

pub async fn discord_callback_legacy(
    State(state): State<AppState>,
    Query(params): Query<OAuthCallbackParams>,
) -> Result<ResponseJson<SuccessResponse<LoginResponse>>> {
    discord_callback(State(state), Query(params)).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_refresh_token_generation() {
        let token = generate_refresh_token(123).unwrap();
        assert!(token.starts_with("rt_123_"));
    }

    #[test]
    fn test_extract_user_id() {
        let token = "rt_456_abc-def-ghi";
        let user_id = extract_user_id_from_refresh_token(token).unwrap();
        assert_eq!(user_id, 456);
    }
}
