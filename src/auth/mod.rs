//! Authentication module for LuminaBridge
//!
//! Handles user authentication and authorization.
//! 处理用户认证和授权。

pub mod oauth;

use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use chrono::{Utc, Duration};

use crate::config::OAuthConfig;
use crate::error::{Error, Result};
use crate::db::User;

pub use oauth::{OAuthProvider, OAuthFlow, GitHubProvider, DiscordProvider};
pub use self::{TokenClaims, TenantClaims};

/// Authentication service
/// 认证服务
pub struct AuthService {
    /// OAuth configuration
    /// OAuth 配置
    config: OAuthConfig,
}

impl AuthService {
    /// Create a new authentication service
    /// 创建新的认证服务
    ///
    /// # Arguments
    ///
    /// * `config` - OAuth configuration
    ///
    /// # Returns
    ///
    /// * `Self` - New auth service instance
    pub fn new(config: OAuthConfig) -> Self {
        AuthService { config }
    }
    
    /// Generate JWT token for a user
    /// 为用户生成 JWT 令牌
    ///
    /// # Arguments
    ///
    /// * `user` - User to generate token for
    ///
    /// # Returns
    ///
    /// * `Result<String>` - JWT token
    ///
    /// # Example
    ///
    /// ```rust
    /// let token = auth.generate_token(&user)?;
    /// ```
    pub fn generate_token(&self, user: &User) -> Result<String> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::seconds(self.config.token_expiration_secs as i64))
            .ok_or_else(|| Error::Auth("Failed to calculate expiration".to_string()))?;
        
        let claims = TokenClaims {
            sub: user.id.to_string(),
            user_id: user.id,
            email: user.email.clone(),
            display_name: user.display_name.clone(),
            role: user.role.clone(),
            tenant: TenantClaims {
                tenant_id: user.tenant_id,
                tenant_name: "Default".to_string(), // TODO: Load from database
                tenant_slug: "default".to_string(),
            },
            exp: expiration.timestamp() as usize,
            iat: Utc::now().timestamp() as usize,
        };
        
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_bytes()),
        )
        .map_err(|e| Error::Jwt(e))?;
        
        Ok(token)
    }
    
    /// Validate and decode JWT token
    /// 验证和解码 JWT 令牌
    ///
    /// # Arguments
    ///
    /// * `token` - JWT token to validate
    ///
    /// # Returns
    ///
    /// * `Result<TokenClaims>` - Decoded claims
    pub fn validate_token(&self, token: &str) -> Result<TokenClaims> {
        let token_data = decode::<TokenClaims>(
            token,
            &DecodingKey::from_secret(self.config.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| Error::Jwt(e))?;
        
        Ok(token_data.claims)
    }
    
    /// Get OAuth provider by name
    /// 按名称获取 OAuth 提供商
    ///
    /// # Arguments
    ///
    /// * `name` - Provider name (github, discord)
    ///
    /// # Returns
    ///
    /// * `Option<Arc<dyn OAuthProvider>>` - OAuth provider if configured
    pub fn get_provider(&self, name: &str) -> Option<Arc<dyn OAuthProvider>> {
        match name.to_lowercase().as_str() {
            "github" => {
                self.config.github.as_ref().map(|config| {
                    Arc::new(GitHubProvider::new(config.clone())) as Arc<dyn OAuthProvider>
                })
            }
            "discord" => {
                self.config.discord.as_ref().map(|config| {
                    Arc::new(DiscordProvider::new(config.clone())) as Arc<dyn OAuthProvider>
                })
            }
            _ => None,
        }
    }
}

/// Tenant claims in JWT token
/// JWT 令牌中的租户声明
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantClaims {
    /// Tenant ID
    /// 租户 ID
    pub tenant_id: i64,
    
    /// Tenant name
    /// 租户名称
    pub tenant_name: String,
    
    /// Tenant slug
    /// 租户标识
    pub tenant_slug: String,
}

/// JWT token claims
/// JWT 令牌声明
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    /// Subject (user ID as string)
    /// 主题（用户 ID 字符串）
    pub sub: String,
    
    /// User ID
    /// 用户 ID
    pub user_id: i64,
    
    /// User email
    /// 用户电子邮件
    pub email: String,
    
    /// User display name
    /// 用户显示名称
    pub display_name: Option<String>,
    
    /// User role
    /// 用户角色
    pub role: String,
    
    /// Tenant information
    /// 租户信息
    pub tenant: TenantClaims,
    
    /// Expiration time (Unix timestamp)
    /// 过期时间（Unix 时间戳）
    pub exp: usize,
    
    /// Issued at (Unix timestamp)
    /// 签发时间（Unix 时间戳）
    pub iat: usize,
}

/// Session information
/// 会话信息
#[derive(Debug, Clone)]
pub struct Session {
    /// Session ID
    /// 会话 ID
    pub id: String,
    
    /// User ID
    /// 用户 ID
    pub user_id: i64,
    
    /// Session token
    /// 会话令牌
    pub token: String,
    
    /// Expiration timestamp
    /// 过期时间戳
    pub expires_at: chrono::DateTime<chrono::Utc>,
    
    /// IP address
    /// IP 地址
    pub ip_address: Option<String>,
    
    /// User agent
    /// 用户代理
    pub user_agent: Option<String>,
}

/// Permission levels
/// 权限级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    /// Read-only access
    /// 只读访问
    Read,
    
    /// Read and write access
    /// 读写访问
    Write,
    
    /// Administrative access
    /// 管理访问
    Admin,
}

impl Permission {
    /// Check if this permission includes another
    /// 检查此权限是否包含另一个权限
    pub fn includes(&self, other: &Permission) -> bool {
        match (self, other) {
            (Permission::Admin, _) => true,
            (Permission::Write, Permission::Read) => true,
            (Permission::Read, Permission::Read) => true,
            _ => false,
        }
    }
}

/// API key authentication
/// API 密钥认证
pub struct ApiKeyAuth {
    /// API key hash
    /// API 密钥哈希
    key_hash: String,
    
    /// Permissions
    /// 权限
    permissions: Vec<Permission>,
    
    /// User ID
    /// 用户 ID
    user_id: i64,
}

impl ApiKeyAuth {
    /// Create new API key auth
    /// 创建新的 API 密钥认证
    pub fn new(key_hash: String, permissions: Vec<Permission>, user_id: i64) -> Self {
        ApiKeyAuth {
            key_hash,
            permissions,
            user_id,
        }
    }
    
    /// Check if has permission
    /// 检查是否有权限
    pub fn has_permission(&self, required: Permission) -> bool {
        self.permissions.iter().any(|p| p.includes(&required))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_includes() {
        assert!(Permission::Admin.includes(&Permission::Read));
        assert!(Permission::Admin.includes(&Permission::Write));
        assert!(Permission::Write.includes(&Permission::Read));
        assert!(!Permission::Read.includes(&Permission::Write));
    }

    #[test]
    fn test_auth_service_creation() {
        let config = OAuthConfig {
            github: None,
            discord: None,
            jwt_secret: "test-secret-key-at-least-32-chars-long".to_string(),
            token_expiration_secs: 3600,
        };
        
        let auth = AuthService::new(config);
        assert!(auth.get_provider("github").is_none());
    }
}
