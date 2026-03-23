//! Discord OAuth Provider
//!
//! Implements OAuth 2.0 authentication with Discord.
//! 实现与 Discord 的 OAuth 2.0 认证。

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::config::OAuthProviderConfig;
use crate::error::{Error, Result};
use super::provider::OAuthProvider;
use super::{TokenResponse, UserInfo};

/// Discord OAuth Provider implementation
/// Discord OAuth 提供商实现
pub struct DiscordProvider {
    /// OAuth configuration
    /// OAuth 配置
    config: OAuthProviderConfig,
    
    /// HTTP client
    /// HTTP 客户端
    client: Client,
    
    /// Discord OAuth endpoints
    /// Discord OAuth 端点
    auth_url: &'static str,
    token_url: &'static str,
    api_url: &'static str,
}

impl DiscordProvider {
    /// Create a new Discord OAuth provider
    /// 创建新的 Discord OAuth 提供商
    ///
    /// # Arguments
    ///
    /// * `config` - OAuth provider configuration
    ///
    /// # Returns
    ///
    /// * `Self` - New Discord provider instance
    pub fn new(config: OAuthProviderConfig) -> Self {
        DiscordProvider {
            config,
            client: Client::new(),
            auth_url: "https://discord.com/api/oauth2/authorize",
            token_url: "https://discord.com/api/oauth2/token",
            api_url: "https://discord.com/api",
        }
    }
    
    /// Build query parameters for authorization URL
    /// 构建授权 URL 的查询参数
    fn build_auth_params(&self, state: &str) -> Vec<(String, String)> {
        let mut params = vec![
            ("client_id".to_string(), self.config.client_id.clone()),
            ("redirect_uri".to_string(), self.config.redirect_url.clone()),
            ("response_type".to_string(), "code".to_string()),
            ("state".to_string(), state.to_string()),
        ];
        
        if !self.config.scopes.is_empty() {
            let scope = self.config.scopes.join("+");
            params.push(("scope".to_string(), scope));
        }
        
        params
    }
}

#[async_trait]
impl OAuthProvider for DiscordProvider {
    fn name(&self) -> &str {
        "discord"
    }
    
    fn get_authorization_url(&self, state: &str) -> String {
        let params = self.build_auth_params(state);
        let query = params.iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v.as_str())))
            .collect::<Vec<_>>()
            .join("&");
        
        format!("{}?{}", self.auth_url, query)
    }
    
    async fn exchange_code(&self, code: &str) -> Result<TokenResponse> {
        info!("Exchanging Discord authorization code for token");
        
        let params = [
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
            ("grant_type", &"authorization_code".to_string()),
            ("code", &code.to_string()),
            ("redirect_uri", &self.config.redirect_url),
        ];
        
        let response = self.client
            .post(self.token_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await
            .map_err(|e| Error::OAuth(format!("Discord token exchange failed: {}", e)))?;
        
        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(Error::OAuth(format!("Discord returned error: {}", error)));
        }
        
        let token_response: DiscordTokenResponse = response.json().await
            .map_err(|e| Error::OAuth(format!("Failed to parse Discord response: {}", e)))?;
        
        Ok(TokenResponse {
            access_token: token_response.access_token,
            token_type: token_response.token_type,
            expires_in: Some(token_response.expires_in),
            refresh_token: Some(token_response.refresh_token),
            scope: Some(token_response.scope.join(" ")),
        })
    }
    
    async fn get_user_info(&self, access_token: &str) -> Result<UserInfo> {
        info!("Fetching Discord user info");
        
        let response = self.client
            .get(format!("{}/users/@me", self.api_url))
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| Error::OAuth(format!("Failed to fetch Discord user: {}", e)))?;
        
        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(Error::OAuth(format!("Discord API error: {}", error)));
        }
        
        let discord_user: DiscordUser = response.json().await
            .map_err(|e| Error::OAuth(format!("Failed to parse Discord user: {}", e)))?;
        
        // Discord users may not have verified email
        let email = discord_user.email.clone()
            .ok_or_else(|| Error::OAuth("User has no verified email".to_string()))?;
        
        // Build avatar URL if avatar is present
        let user_id = discord_user.id.clone();
        let avatar_url = discord_user.avatar.clone().map(|avatar| {
            format!(
                "https://cdn.discordapp.com/avatars/{}/{}.png",
                user_id, avatar
            )
        });
        
        Ok(UserInfo {
            provider_id: user_id,
            email,
            name: discord_user.username.clone(),
            avatar_url,
            extra: Some(serde_json::to_value(&discord_user).unwrap_or_default()),
        })
    }
    
    async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse> {
        info!("Refreshing Discord access token");
        
        let params = [
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
            ("grant_type", &"refresh_token".to_string()),
            ("refresh_token", &refresh_token.to_string()),
        ];
        
        let response = self.client
            .post(self.token_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await
            .map_err(|e| Error::OAuth(format!("Discord token refresh failed: {}", e)))?;
        
        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(Error::OAuth(format!("Discord refresh error: {}", error)));
        }
        
        let token_response: DiscordTokenResponse = response.json().await
            .map_err(|e| Error::OAuth(format!("Failed to parse Discord response: {}", e)))?;
        
        Ok(TokenResponse {
            access_token: token_response.access_token,
            token_type: token_response.token_type,
            expires_in: Some(token_response.expires_in),
            refresh_token: Some(token_response.refresh_token),
            scope: Some(token_response.scope.join(" ")),
        })
    }
    
    async fn revoke_token(&self, access_token: &str) -> Result<()> {
        info!("Revoking Discord access token");
        
        let params = [
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
            ("token", &access_token.to_string()),
        ];
        
        let response = self.client
            .post(format!("{}/oauth2/token/revoke", self.api_url))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await
            .map_err(|e| Error::OAuth(format!("Discord token revocation failed: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(Error::OAuth("Failed to revoke Discord token".to_string()));
        }
        
        Ok(())
    }
}

/// Discord token response structure
/// Discord 令牌响应结构
#[derive(Debug, Deserialize, Serialize)]
struct DiscordTokenResponse {
    /// Access token
    /// 访问令牌
    access_token: String,
    
    /// Token type
    /// 令牌类型
    token_type: String,
    
    /// Expires in seconds
    /// 过期时间（秒）
    expires_in: u64,
    
    /// Refresh token
    /// 刷新令牌
    refresh_token: String,
    
    /// Scope
    /// 作用域
    scope: Vec<String>,
}

/// Discord user structure
/// Discord 用户结构
#[derive(Debug, Deserialize, Serialize)]
struct DiscordUser {
    /// User ID (snowflake)
    /// 用户 ID（雪花 ID）
    id: String,
    
    /// Username
    /// 用户名
    username: String,
    
    /// Discriminator (legacy)
    /// 区分符（旧版）
    discriminator: Option<String>,
    
    /// Global display name
    /// 全局显示名称
    global_name: Option<String>,
    
    /// Email address (requires email scope)
    /// 电子邮件地址（需要 email 作用域）
    email: Option<String>,
    
    /// Avatar hash
    /// 头像哈希
    avatar: Option<String>,
    
    /// Whether email is verified
    /// 电子邮件是否已验证
    verified: Option<bool>,
    
    /// Premium type (Nitro subscription)
    /// 高级类型（Nitro 订阅）
    premium_type: Option<u8>,
    
    /// Public flags
    /// 公共标志
    public_flags: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discord_provider_creation() {
        let config = OAuthProviderConfig {
            client_id: "test_client_id".to_string(),
            client_secret: "test_secret".to_string(),
            redirect_url: "http://localhost:3000/auth/discord/callback".to_string(),
            scopes: vec!["identify".to_string(), "email".to_string()],
        };
        
        let provider = DiscordProvider::new(config);
        assert_eq!(provider.name(), "discord");
    }

    #[test]
    fn test_authorization_url() {
        let config = OAuthProviderConfig {
            client_id: "test_id".to_string(),
            client_secret: "test_secret".to_string(),
            redirect_url: "http://localhost/callback".to_string(),
            scopes: vec!["identify".to_string(), "email".to_string()],
        };
        
        let provider = DiscordProvider::new(config);
        let url = provider.get_authorization_url("test_state");
        
        assert!(url.contains("discord.com/api/oauth2/authorize"));
        assert!(url.contains("client_id=test_id"));
        assert!(url.contains("state=test_state"));
    }
}
