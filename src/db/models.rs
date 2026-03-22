//! Database models for LuminaBridge
//!
//! Defines the data structures for database tables.
//! 定义数据库表的数据结构。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use serde_json::Value as JsonValue;

/// Tenant model
/// 租户模型
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Tenant {
    /// Tenant ID
    /// 租户 ID
    pub id: i64,
    
    /// Tenant name
    /// 租户名称
    pub name: String,
    
    /// Tenant slug (URL-friendly identifier)
    /// 租户标识（URL 友好的标识符）
    pub slug: String,
    
    /// Tenant status
    /// 租户状态
    pub status: String,
    
    /// Quota limit
    /// 配额限制
    pub quota_limit: Option<i64>,
    
    /// Quota used
    /// 已用配额
    pub quota_used: i64,
    
    /// Settings (JSON)
    /// 设置（JSON）
    pub settings: Option<JsonValue>,
    
    /// Created at
    /// 创建时间
    pub created_at: DateTime<Utc>,
    
    /// Updated at
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// User model
/// 用户模型
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    /// User ID
    /// 用户 ID
    pub id: i64,
    
    /// Tenant ID
    /// 租户 ID
    pub tenant_id: i64,
    
    /// User email
    /// 用户电子邮件
    pub email: String,
    
    /// Password hash (for password-based auth)
    /// 密码哈希（用于密码认证）
    pub password_hash: Option<String>,
    
    /// Display name
    /// 显示名称
    pub display_name: Option<String>,
    
    /// Avatar URL
    /// 头像 URL
    pub avatar_url: Option<String>,
    
    /// User role
    /// 用户角色
    pub role: String,
    
    /// User status
    /// 用户状态
    pub status: String,
    
    /// OAuth provider
    /// OAuth 提供商
    pub oauth_provider: Option<String>,
    
    /// OAuth ID
    /// OAuth ID
    pub oauth_id: Option<String>,
    
    /// Last login at
    /// 最后登录时间
    pub last_login_at: Option<DateTime<Utc>>,
    
    /// Created at
    /// 创建时间
    pub created_at: DateTime<Utc>,
    
    /// Updated at
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// Channel model
/// 渠道模型
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Channel {
    /// Channel ID
    /// 渠道 ID
    pub id: i64,
    
    /// Tenant ID
    /// 租户 ID
    pub tenant_id: i64,
    
    /// Channel name
    /// 渠道名称
    pub name: String,
    
    /// Channel type (openai, anthropic, google, etc.)
    /// 渠道类型（openai、anthropic、google 等）
    pub channel_type: String,
    
    /// API key
    /// API 密钥
    pub key: String,
    
    /// Base URL
    /// 基础 URL
    pub base_url: Option<String>,
    
    /// Supported models (JSON array)
    /// 支持的模型（JSON 数组）
    pub models: JsonValue,
    
    /// Weight for load balancing
    /// 负载均衡权重
    pub weight: i32,
    
    /// Channel status
    /// 渠道状态
    pub status: String,
    
    /// Priority
    /// 优先级
    pub priority: i32,
    
    /// Timeout in milliseconds
    /// 超时时间（毫秒）
    pub timeout_ms: i32,
    
    /// Retry count
    /// 重试次数
    pub retry_count: i32,
    
    /// Balance
    /// 余额
    pub balance: Option<rust_decimal::Decimal>,
    
    /// Last test at
    /// 最后测试时间
    pub last_test_at: Option<DateTime<Utc>>,
    
    /// Last test status
    /// 最后测试状态
    pub last_test_status: Option<String>,
    
    /// Created at
    /// 创建时间
    pub created_at: DateTime<Utc>,
    
    /// Updated at
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// Token model
/// 令牌模型
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Token {
    /// Token ID
    /// 令牌 ID
    pub id: i64,
    
    /// Tenant ID
    /// 租户 ID
    pub tenant_id: i64,
    
    /// User ID
    /// 用户 ID
    pub user_id: Option<i64>,
    
    /// Token key
    /// 令牌密钥
    pub key: String,
    
    /// Token name
    /// 令牌名称
    pub name: Option<String>,
    
    /// Quota limit
    /// 配额限制
    pub quota_limit: Option<i64>,
    
    /// Quota used
    /// 已用配额
    pub quota_used: i64,
    
    /// Expire at
    /// 过期时间
    pub expire_at: Option<DateTime<Utc>>,
    
    /// Token status
    /// 令牌状态
    pub status: String,
    
    /// Allowed IPs (JSON array)
    /// 允许的 IP（JSON 数组）
    pub allowed_ips: Option<JsonValue>,
    
    /// Allowed models (JSON array)
    /// 允许的模型（JSON 数组）
    pub allowed_models: Option<JsonValue>,
    
    /// Last used at
    /// 最后使用时间
    pub last_used_at: Option<DateTime<Utc>>,
    
    /// Created at
    /// 创建时间
    pub created_at: DateTime<Utc>,
    
    /// Updated at
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// Usage stat model
/// 用量统计模型
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UsageStat {
    /// Stat ID
    /// 统计 ID
    pub id: i64,
    
    /// Tenant ID
    /// 租户 ID
    pub tenant_id: i64,
    
    /// User ID
    /// 用户 ID
    pub user_id: Option<i64>,
    
    /// Channel ID
    /// 渠道 ID
    pub channel_id: Option<i64>,
    
    /// Model name
    /// 模型名称
    pub model: Option<String>,
    
    /// Prompt tokens
    /// 提示词令牌数
    pub prompt_tokens: i64,
    
    /// Completion tokens
    /// 完成令牌数
    pub completion_tokens: i64,
    
    /// Total tokens
    /// 总令牌数
    pub total_tokens: i64,
    
    /// Cost
    /// 成本
    pub cost: rust_decimal::Decimal,
    
    /// Status
    /// 状态
    pub status: Option<String>,
    
    /// Latency in milliseconds
    /// 延迟（毫秒）
    pub latency_ms: Option<i32>,
    
    /// Created at
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// OAuth account model
/// OAuth 账户模型
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct OAuthAccount {
    /// Account ID
    /// 账户 ID
    pub id: i64,
    
    /// User ID
    /// 用户 ID
    pub user_id: i64,
    
    /// Provider name
    /// 提供商名称
    pub provider: String,
    
    /// Provider user ID
    /// 提供商用户 ID
    pub provider_user_id: String,
    
    /// Access token
    /// 访问令牌
    pub access_token: String,
    
    /// Refresh token
    /// 刷新令牌
    pub refresh_token: Option<String>,
    
    /// Expires at
    /// 过期时间
    pub expires_at: Option<DateTime<Utc>>,
    
    /// Created at
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// Alert model
/// 告警模型
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Alert {
    /// Alert ID
    /// 告警 ID
    pub id: i64,
    
    /// Tenant ID
    /// 租户 ID
    pub tenant_id: i64,
    
    /// Alert level (critical, warning, info)
    /// 告警级别（critical、warning、info）
    pub level: String,
    
    /// Alert type
    /// 告警类型
    pub alert_type: String,
    
    /// Alert message
    /// 告警消息
    pub message: String,
    
    /// Related entity ID (channel_id, token_id, etc.)
    /// 相关实体 ID（渠道 ID、令牌 ID 等）
    pub entity_id: Option<i64>,
    
    /// Entity type (channel, token, user, etc.)
    /// 实体类型（channel、token、user 等）
    pub entity_type: Option<String>,
    
    /// Is resolved
    /// 是否已解决
    pub is_resolved: bool,
    
    /// Created at
    /// 创建时间
    pub created_at: DateTime<Utc>,
    
    /// Resolved at
    /// 解决时间
    pub resolved_at: Option<DateTime<Utc>>,
}

/// Alert level enum
/// 告警级别枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertLevel {
    /// Critical - requires immediate attention
    /// 紧急 - 需要立即处理
    Critical,
    /// Warning - needs attention
    /// 警告 - 需要注意
    Warning,
    /// Info - general notification
    /// 信息 - 一般通知
    Info,
}

impl AlertLevel {
    /// Convert to string
    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertLevel::Critical => "critical",
            AlertLevel::Warning => "warning",
            AlertLevel::Info => "info",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_serialization() {
        let tenant = Tenant {
            id: 1,
            name: "Test Tenant".to_string(),
            slug: "test-tenant".to_string(),
            status: "active".to_string(),
            quota_limit: Some(1000000),
            quota_used: 50000,
            settings: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let json = serde_json::to_string(&tenant).unwrap();
        assert!(json.contains("Test Tenant"));
    }

    #[test]
    fn test_user_from_row() {
        // Just verify the struct can be instantiated
        let user = User {
            id: 1,
            tenant_id: 1,
            email: "test@example.com".to_string(),
            password_hash: None,
            display_name: Some("Test User".to_string()),
            avatar_url: None,
            role: "user".to_string(),
            status: "active".to_string(),
            oauth_provider: None,
            oauth_id: None,
            last_login_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        assert_eq!(user.email, "test@example.com");
    }
}
