//! OAuth Provider trait
//!
//! Defines the standard interface for OAuth authentication providers.
//! 定义 OAuth 认证提供商的标准接口。

use async_trait::async_trait;
use crate::error::Result;
use super::{TokenResponse, UserInfo};

/// OAuth Provider trait defines the interface for OAuth authentication
/// OAuth Provider 特征定义了 OAuth 认证的标准接口
///
/// This trait should be implemented by each OAuth provider (GitHub, Discord, etc.)
/// 每个 OAuth 提供商（GitHub、Discord 等）都应实现此特征
#[async_trait]
pub trait OAuthProvider: Send + Sync {
    /// Returns the provider name
    /// 返回提供商名称
    ///
    /// # Returns
    ///
    /// * `&str` - Provider identifier (e.g., "github", "discord")
    fn name(&self) -> &str;
    
    /// Get the authorization URL for user login
    /// 获取用户登录的授权 URL
    ///
    /// # Arguments
    ///
    /// * `state` - CSRF protection state parameter
    ///
    /// # Returns
    ///
    /// * `String` - Full authorization URL
    ///
    /// # Example
    ///
    /// ```rust
    /// let auth_url = provider.get_authorization_url("random_state");
    /// ```
    fn get_authorization_url(&self, state: &str) -> String;
    
    /// Exchange authorization code for access token
    /// 用授权代码交换访问令牌
    ///
    /// # Arguments
    ///
    /// * `code` - Authorization code from callback
    ///
    /// # Returns
    ///
    /// * `Result<TokenResponse>` - Token response or error
    ///
    /// # Example
    ///
    /// ```rust
    /// let token = provider.exchange_code(auth_code).await?;
    /// ```
    async fn exchange_code(&self, code: &str) -> Result<TokenResponse>;
    
    /// Get user information using access token
    /// 使用访问令牌获取用户信息
    ///
    /// # Arguments
    ///
    /// * `access_token` - OAuth access token
    ///
    /// # Returns
    ///
    /// * `Result<UserInfo>` - User information or error
    ///
    /// # Example
    ///
    /// ```rust
    /// let user = provider.get_user_info(token).await?;
    /// ```
    async fn get_user_info(&self, access_token: &str) -> Result<UserInfo>;
    
    /// Refresh access token using refresh token
    /// 使用刷新令牌刷新访问令牌
    ///
    /// # Arguments
    ///
    /// * `refresh_token` - Refresh token from previous token response
    ///
    /// # Returns
    ///
    /// * `Result<TokenResponse>` - New token response or error
    ///
    /// # Note
    ///
    /// Not all providers support token refresh
    /// 并非所有提供商都支持令牌刷新
    async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse> {
        // Default implementation returns error - override if provider supports refresh
        Err(crate::error::Error::OAuth(
            "Token refresh not supported by this provider".to_string()
        ))
    }
    
    /// Validate access token
    /// 验证访问令牌
    ///
    /// # Arguments
    ///
    /// * `access_token` - Access token to validate
    ///
    /// # Returns
    ///
    /// * `Result<bool>` - True if valid
    async fn validate_token(&self, access_token: &str) -> Result<bool> {
        // Default implementation tries to get user info
        match self.get_user_info(access_token).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    /// Revoke access token
    /// 撤销访问令牌
    ///
    /// # Arguments
    ///
    /// * `access_token` - Access token to revoke
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok on success
    ///
    /// # Note
    ///
    /// Not all providers support token revocation
    /// 并非所有提供商都支持令牌撤销
    async fn revoke_token(&self, _access_token: &str) -> Result<()> {
        // Default implementation returns error - override if provider supports revocation
        Err(crate::error::Error::OAuth(
            "Token revocation not supported by this provider".to_string()
        ))
    }
}
