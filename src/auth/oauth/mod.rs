//! OAuth authentication module
//!
//! Provides OAuth 2.0 authentication support for various providers.
//! 为各种提供商提供 OAuth 2.0 认证支持。

pub mod provider;
pub mod github;
pub mod discord;

pub use provider::OAuthProvider;
pub use github::GitHubProvider;
pub use discord::DiscordProvider;

use crate::error::Result;

/// OAuth flow state
/// OAuth 流程状态
#[derive(Debug, Clone)]
pub struct OAuthState {
    /// Random state parameter for CSRF protection
    /// 用于 CSRF 保护的随机状态参数
    pub state: String,
    
    /// Redirect URL after authentication
    /// 认证后的重定向 URL
    pub redirect_url: String,
    
    /// Expiration timestamp
    /// 过期时间戳
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

impl OAuthState {
    /// Create a new OAuth state
    /// 创建新的 OAuth 状态
    pub fn new(redirect_url: String) -> Self {
        use uuid::Uuid;
        
        OAuthState {
            state: Uuid::new_v4().to_string(),
            redirect_url,
            expires_at: Utc::now() + chrono::Duration::minutes(10),
        }
    }
    
    /// Check if state is expired
    /// 检查状态是否过期
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

/// OAuth token response
/// OAuth 令牌响应
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct TokenResponse {
    /// Access token
    /// 访问令牌
    pub access_token: String,
    
    /// Token type (usually "Bearer")
    /// 令牌类型（通常为 "Bearer"）
    pub token_type: String,
    
    /// Expires in seconds
    /// 过期时间（秒）
    pub expires_in: Option<u64>,
    
    /// Refresh token
    /// 刷新令牌
    pub refresh_token: Option<String>,
    
    /// Scope granted
    /// 授予的作用域
    pub scope: Option<String>,
}

/// User information from OAuth provider
/// 来自 OAuth 提供商的用户信息
#[derive(Debug, Clone)]
pub struct UserInfo {
    /// Provider's user ID
    /// 提供商的用户 ID
    pub provider_id: String,
    
    /// Email address
    /// 电子邮件地址
    pub email: String,
    
    /// Display name
    /// 显示名称
    pub name: String,
    
    /// Avatar URL
    /// 头像 URL
    pub avatar_url: Option<String>,
    
    /// Additional provider-specific data
    /// 其他提供商特定数据
    pub extra: Option<serde_json::Value>,
}

use chrono::Utc;
