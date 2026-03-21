//! GitHub OAuth Provider
//!
//! Implements OAuth 2.0 authentication with GitHub.
//! 实现与 GitHub 的 OAuth 2.0 认证。

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::config::OAuthProviderConfig;
use crate::error::{Error, Result};
use super::provider::OAuthProvider;
use super::{TokenResponse, UserInfo};

/// GitHub OAuth Provider implementation
/// GitHub OAuth 提供商实现
pub struct GitHubProvider {
    /// OAuth configuration
    /// OAuth 配置
    config: OAuthProviderConfig,
    
    /// HTTP client
    /// HTTP 客户端
    client: Client,
    
    /// GitHub OAuth endpoints
    /// GitHub OAuth 端点
    auth_url: &'static str,
    token_url: &'static str,
    api_url: &'static str,
}

impl GitHubProvider {
    /// Create a new GitHub OAuth provider
    /// 创建新的 GitHub OAuth 提供商
    ///
    /// # Arguments
    ///
    /// * `config` - OAuth provider configuration
    ///
    /// # Returns
    ///
    /// * `Self` - New GitHub provider instance
    pub fn new(config: OAuthProviderConfig) -> Self {
        GitHubProvider {
            config,
            client: Client::new(),
            auth_url: "https://github.com/login/oauth/authorize",
            token_url: "https://github.com/login/oauth/access_token",
            api_url: "https://api.github.com",
        }
    }
    
    /// Build query parameters for authorization URL
    /// 构建授权 URL 的查询参数
    fn build_auth_params(&self, state: &str) -> Vec<(&str, &str)> {
        let mut params = vec![
            ("client_id", self.config.client_id.as_str()),
            ("redirect_uri", self.config.redirect_url.as_str()),
            ("state", state),
        ];
        
        if !self.config.scopes.is_empty() {
            let scope = self.config.scopes.join(" ");
            params.push(("scope", scope.as_str()));
        }
        
        params
    }
}

#[async_trait]
impl OAuthProvider for GitHubProvider {
    fn name(&self) -> &str {
        "github"
    }
    
    fn get_authorization_url(&self, state: &str) -> String {
        let params = self.build_auth_params(state);
        let query = urlencoding::encode(&params.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&"));
        
        format!("{}?{}", self.auth_url, query)
    }
    
    async fn exchange_code(&self, code: &str) -> Result<TokenResponse> {
        info!("Exchanging GitHub authorization code for token");
        
        let params = [
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
            ("code", code),
            ("redirect_uri", &self.config.redirect_url),
        ];
        
        let response = self.client
            .post(self.token_url)
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .await
            .map_err(|e| Error::OAuth(format!("GitHub token exchange failed: {}", e)))?;
        
        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(Error::OAuth(format!("GitHub returned error: {}", error)));
        }
        
        let token_response: GitHubTokenResponse = response.json().await
            .map_err(|e| Error::OAuth(format!("Failed to parse GitHub response: {}", e)))?;
        
        // Check for OAuth errors in response
        if let Some(error) = token_response.error {
            return Err(Error::OAuth(format!("GitHub OAuth error: {}", error)));
        }
        
        Ok(TokenResponse {
            access_token: token_response.access_token
                .ok_or_else(|| Error::OAuth("No access token in response".to_string()))?,
            token_type: token_response.token_type.unwrap_or_else(|| "Bearer".to_string()),
            expires_in: None, // GitHub tokens don't expire
            refresh_token: None,
            scope: token_response.scope,
        })
    }
    
    async fn get_user_info(&self, access_token: &str) -> Result<UserInfo> {
        info!("Fetching GitHub user info");
        
        let response = self.client
            .get(format!("{}/user", self.api_url))
            .header("Authorization", format!("token {}", access_token))
            .header("User-Agent", "LuminaBridge")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .map_err(|e| Error::OAuth(format!("Failed to fetch GitHub user: {}", e)))?;
        
        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(Error::OAuth(format!("GitHub API error: {}", error)));
        }
        
        let github_user: GitHubUser = response.json().await
            .map_err(|e| Error::OAuth(format!("Failed to parse GitHub user: {}", e)))?;
        
        // Get email if not public
        let email = if let Some(ref user_email) = github_user.email {
            user_email.clone()
        } else {
            // Try to get primary email from emails endpoint
            self.get_primary_email(access_token).await?
        };
        
        Ok(UserInfo {
            provider_id: github_user.id.to_string(),
            email,
            name: github_user.name.unwrap_or_else(|| github_user.login.clone()),
            avatar_url: github_user.avatar_url,
            extra: Some(serde_json::to_value(&github_user).unwrap_or_default()),
        })
    }
    
    async fn revoke_token(&self, _access_token: &str) -> Result<()> {
        // GitHub doesn't support token revocation
        Err(Error::OAuth("GitHub doesn't support token revocation".to_string()))
    }
}

/// GitHub token response structure
/// GitHub 令牌响应结构
#[derive(Debug, Deserialize, Serialize)]
struct GitHubTokenResponse {
    /// Access token
    /// 访问令牌
    access_token: Option<String>,
    
    /// Token type
    /// 令牌类型
    token_type: Option<String>,
    
    /// Scope
    /// 作用域
    scope: Option<String>,
    
    /// Error (if any)
    /// 错误（如果有）
    error: Option<String>,
    
    /// Error description
    /// 错误描述
    error_description: Option<String>,
}

/// GitHub user structure
/// GitHub 用户结构
#[derive(Debug, Deserialize, Serialize)]
struct GitHubUser {
    /// User ID
    /// 用户 ID
    id: u64,
    
    /// Login username
    /// 登录用户名
    login: String,
    
    /// Display name
    /// 显示名称
    name: Option<String>,
    
    /// Email address
    /// 电子邮件地址
    email: Option<String>,
    
    /// Avatar URL
    /// 头像 URL
    avatar_url: String,
    
    /// Profile URL
    /// 个人资料 URL
    html_url: String,
    
    /// Company
    /// 公司
    company: Option<String>,
    
    /// Location
    /// 位置
    location: Option<String>,
}

impl GitHubProvider {
    /// Get primary email from GitHub
    /// 从 GitHub 获取主要电子邮件
    async fn get_primary_email(&self, access_token: &str) -> Result<String> {
        #[derive(Deserialize)]
        struct Email {
            email: String,
            primary: bool,
            verified: bool,
        }
        
        let response = self.client
            .get(format!("{}/user/emails", self.api_url))
            .header("Authorization", format!("token {}", access_token))
            .header("User-Agent", "LuminaBridge")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .map_err(|e| Error::OAuth(format!("Failed to fetch GitHub emails: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(Error::OAuth("Failed to fetch GitHub emails".to_string()));
        }
        
        let emails: Vec<Email> = response.json().await
            .map_err(|e| Error::OAuth(format!("Failed to parse GitHub emails: {}", e)))?;
        
        // Find primary verified email
        emails.into_iter()
            .find(|e| e.primary && e.verified)
            .map(|e| e.email)
            .ok_or_else(|| Error::OAuth("No verified email found".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_provider_creation() {
        let config = OAuthProviderConfig {
            client_id: "test_client_id".to_string(),
            client_secret: "test_secret".to_string(),
            redirect_url: "http://localhost:3000/auth/github/callback".to_string(),
            scopes: vec!["user:email".to_string()],
        };
        
        let provider = GitHubProvider::new(config);
        assert_eq!(provider.name(), "github");
    }

    #[test]
    fn test_authorization_url() {
        let config = OAuthProviderConfig {
            client_id: "test_id".to_string(),
            client_secret: "test_secret".to_string(),
            redirect_url: "http://localhost/callback".to_string(),
            scopes: vec!["user:email".to_string()],
        };
        
        let provider = GitHubProvider::new(config);
        let url = provider.get_authorization_url("test_state");
        
        assert!(url.contains("github.com/login/oauth/authorize"));
        assert!(url.contains("client_id=test_id"));
        assert!(url.contains("state=test_state"));
    }
}
