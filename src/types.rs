//! Unified response types for LuminaBridge API
//!
//! Provides consistent response formats for all API endpoints.
//! 为所有 API 端点提供一致的响应格式。

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Success response wrapper
/// 成功响应包装器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessResponse<T> {
    /// Success flag
    /// 成功标志
    pub success: bool,
    
    /// Response data
    /// 响应数据
    pub data: T,
    
    /// Optional message
    /// 可选消息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    
    /// Optional metadata (pagination, etc.)
    /// 可选元数据（分页等）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ResponseMeta>,
}

impl<T> SuccessResponse<T> {
    /// Create a new success response
    /// 创建新的成功响应
    pub fn new(data: T) -> Self {
        SuccessResponse {
            success: true,
            data,
            message: None,
            meta: None,
        }
    }
    
    /// Set message
    /// 设置消息
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }
    
    /// Set metadata
    /// 设置元数据
    pub fn with_meta(mut self, meta: ResponseMeta) -> Self {
        self.meta = Some(meta);
        self
    }
}

/// Error response
/// 错误响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Success flag (always false)
    /// 成功标志（始终为 false）
    pub success: bool,
    
    /// Error details
    /// 错误详情
    pub error: ErrorDetail,
}

impl ErrorResponse {
    /// Create a new error response
    /// 创建新的错误响应
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        ErrorResponse {
            success: false,
            error: ErrorDetail {
                code: code.into(),
                message: message.into(),
                details: None,
            },
        }
    }
    
    /// Add error details
    /// 添加错误详情
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.error.details = Some(details);
        self
    }
}

/// Error detail structure
/// 错误详情结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    /// Error code
    /// 错误代码
    pub code: String,
    
    /// Error message
    /// 错误消息
    pub message: String,
    
    /// Additional error details
    /// 额外错误详情
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Response metadata (pagination, etc.)
/// 响应元数据（分页等）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMeta {
    /// Current page number
    /// 当前页码
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<i64>,
    
    /// Page size
    /// 每页大小
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<i64>,
    
    /// Total count
    /// 总数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
    
    /// Total pages
    /// 总页数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_pages: Option<i64>,
}

impl ResponseMeta {
    /// Create pagination metadata
    /// 创建分页元数据
    pub fn for_pagination(page: i64, page_size: i64, total: i64) -> Self {
        let total_pages = (total as f64 / page_size as f64).ceil() as i64;
        ResponseMeta {
            page: Some(page),
            page_size: Some(page_size),
            total: Some(total),
            total_pages: Some(total_pages),
        }
    }
}

/// API Error codes
/// API 错误代码
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    // Common errors | 通用错误
    InternalError,
    InvalidRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    
    // Authentication | 认证相关
    InvalidCredentials,
    TokenExpired,
    TokenInvalid,
    OAuthFailed,
    
    // Channel | 渠道相关
    ChannelNotFound,
    ChannelAlreadyExists,
    ChannelTestFailed,
    
    // Token | 令牌相关
    TokenNotFound,
    TokenQuotaExceeded,
    
    // Tenant | 租户相关
    TenantNotFound,
    TenantQuotaExceeded,
    
    // User | 用户相关
    UserNotFound,
    UserAlreadyExists,
    UserInactive,
}

impl ErrorCode {
    /// Get error code as string
    /// 获取错误代码字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::InternalError => "INTERNAL_ERROR",
            ErrorCode::InvalidRequest => "INVALID_REQUEST",
            ErrorCode::Unauthorized => "UNAUTHORIZED",
            ErrorCode::Forbidden => "FORBIDDEN",
            ErrorCode::NotFound => "NOT_FOUND",
            ErrorCode::InvalidCredentials => "INVALID_CREDENTIALS",
            ErrorCode::TokenExpired => "TOKEN_EXPIRED",
            ErrorCode::TokenInvalid => "TOKEN_INVALID",
            ErrorCode::OAuthFailed => "OAUTH_FAILED",
            ErrorCode::ChannelNotFound => "CHANNEL_NOT_FOUND",
            ErrorCode::ChannelAlreadyExists => "CHANNEL_ALREADY_EXISTS",
            ErrorCode::ChannelTestFailed => "CHANNEL_TEST_FAILED",
            ErrorCode::TokenNotFound => "TOKEN_NOT_FOUND",
            ErrorCode::TokenQuotaExceeded => "TOKEN_QUOTA_EXCEEDED",
            ErrorCode::TenantNotFound => "TENANT_NOT_FOUND",
            ErrorCode::TenantQuotaExceeded => "TENANT_QUOTA_EXCEEDED",
            ErrorCode::UserNotFound => "USER_NOT_FOUND",
            ErrorCode::UserAlreadyExists => "USER_ALREADY_EXISTS",
            ErrorCode::UserInactive => "USER_INACTIVE",
        }
    }
    
    /// Get HTTP status code for this error
    /// 获取此错误的 HTTP 状态码
    pub fn status_code(&self) -> u16 {
        match self {
            ErrorCode::InternalError => 500,
            ErrorCode::InvalidRequest => 400,
            ErrorCode::Unauthorized | ErrorCode::InvalidCredentials | ErrorCode::TokenExpired | ErrorCode::TokenInvalid => 401,
            ErrorCode::Forbidden => 403,
            ErrorCode::NotFound | ErrorCode::ChannelNotFound | ErrorCode::TokenNotFound | ErrorCode::UserNotFound | ErrorCode::TenantNotFound => 404,
            ErrorCode::OAuthFailed | ErrorCode::ChannelTestFailed => 400,
            ErrorCode::ChannelAlreadyExists | ErrorCode::UserAlreadyExists => 409,
            ErrorCode::TokenQuotaExceeded | ErrorCode::TenantQuotaExceeded => 429,
            ErrorCode::UserInactive => 403,
        }
    }
}

/// Pagination parameters
/// 分页参数
#[derive(Debug, Clone, Deserialize)]
pub struct PaginationParams {
    /// Page number (default: 1)
    /// 页码（默认：1）
    #[serde(default = "default_page")]
    pub page: i64,
    
    /// Page size (default: 20, max: 100)
    /// 每页大小（默认：20，最大：100）
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

fn default_page() -> i64 { 1 }
fn default_page_size() -> i64 { 20 }

impl PaginationParams {
    /// Get offset for database query
    /// 获取数据库查询的偏移量
    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.page_size
    }
    
    /// Get limit for database query
    /// 获取数据库查询的限制
    pub fn limit(&self) -> i64 {
        self.page_size.min(100)
    }
}

/// Channel status enum
/// 渠道状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelStatus {
    /// Active and healthy
    /// 活跃且健康
    Active,
    /// Warning state (high latency, etc.)
    /// 警告状态（高延迟等）
    Warning,
    /// Error state
    /// 错误状态
    Error,
    /// Disabled
    /// 已禁用
    Disabled,
}

impl Default for ChannelStatus {
    fn default() -> Self {
        ChannelStatus::Active
    }
}

/// Channel type enum
/// 渠道类型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChannelType {
    /// OpenAI compatible API
    OpenAI,
    /// Anthropic Claude
    Anthropic,
    /// Google Gemini
    Google,
    /// Azure OpenAI
    Azure,
    /// Custom provider
    Custom,
}

/// Real-time statistics
/// 实时统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeStats {
    /// Requests per second
    /// 每秒请求数
    pub tps: i64,
    
    /// Requests per minute
    /// 每分钟请求数
    pub rpm: i64,
    
    /// Average latency in milliseconds
    /// 平均延迟（毫秒）
    pub latency_ms: f64,
    
    /// Error rate (0.0 - 1.0)
    /// 错误率（0.0 - 1.0）
    pub error_rate: f64,
    
    /// Active channels count
    /// 活跃渠道数
    pub active_channels: i64,
    
    /// Timestamp
    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_response() {
        let response = SuccessResponse::new("test data")
            .with_message("Success!")
            .with_meta(ResponseMeta::for_pagination(1, 20, 100));
        
        assert!(response.success);
        assert_eq!(response.data, "test data");
        assert_eq!(response.message, Some("Success!".to_string()));
    }

    #[test]
    fn test_error_response() {
        let response = ErrorResponse::new("TEST_ERROR", "Test error message");
        
        assert!(!response.success);
        assert_eq!(response.error.code, "TEST_ERROR");
        assert_eq!(response.error.message, "Test error message");
    }

    #[test]
    fn test_pagination_params() {
        let params = PaginationParams { page: 3, page_size: 50 };
        assert_eq!(params.offset(), 100);
        assert_eq!(params.limit(), 50);
    }

    #[test]
    fn test_error_code_status() {
        assert_eq!(ErrorCode::Unauthorized.status_code(), 401);
        assert_eq!(ErrorCode::NotFound.status_code(), 404);
        assert_eq!(ErrorCode::InternalError.status_code(), 500);
    }
}

// ============================================================================
// Request Types (moved from routes to avoid circular dependency)
// ============================================================================

/// Create channel request
/// 创建渠道请求
#[derive(Debug, Deserialize)]
pub struct CreateChannelRequest {
    /// Channel name
    /// 渠道名称
    pub name: String,
    
    /// Channel type
    /// 渠道类型
    pub channel_type: String,
    
    /// API key
    /// API 密钥
    pub key: String,
    
    /// Base URL (optional)
    /// 基础 URL（可选）
    pub base_url: Option<String>,
    
    /// Supported models
    /// 支持的模型
    pub models: Vec<String>,
    
    /// Weight for load balancing (default: 10)
    /// 负载均衡权重（默认：10）
    #[serde(default = "default_weight")]
    pub weight: i32,
    
    /// Priority (default: 0)
    /// 优先级（默认：0）
    #[serde(default)]
    pub priority: i32,
    
    /// Timeout in milliseconds (default: 30000)
    /// 超时时间（默认：30000）
    #[serde(default = "default_timeout")]
    pub timeout_ms: i32,
    
    /// Retry count (default: 3)
    /// 重试次数（默认：3）
    #[serde(default = "default_retry")]
    pub retry_count: i32,
}

fn default_weight() -> i32 { 10 }
fn default_timeout() -> i32 { 30000 }
fn default_retry() -> i32 { 3 }

/// Update channel request
/// 更新渠道请求
#[derive(Debug, Deserialize)]
pub struct UpdateChannelRequest {
    /// Channel name
    /// 渠道名称
    pub name: Option<String>,
    
    /// API key
    /// API 密钥
    pub key: Option<String>,
    
    /// Base URL
    /// 基础 URL
    pub base_url: Option<String>,
    
    /// Supported models
    /// 支持的模型
    pub models: Option<Vec<String>>,
    
    /// Weight
    /// 权重
    pub weight: Option<i32>,
    
    /// Priority
    /// 优先级
    pub priority: Option<i32>,
    
    /// Timeout in milliseconds
    /// 超时时间（毫秒）
    pub timeout_ms: Option<i32>,
    
    /// Retry count
    /// 重试次数
    pub retry_count: Option<i32>,
    
    /// Status
    /// 状态
    pub status: Option<String>,
}

/// Create token request
/// 创建令牌请求
#[derive(Debug, Deserialize)]
pub struct CreateTokenRequest {
    /// Token name
    /// 令牌名称
    pub name: String,
    
    /// Quota limit
    /// 配额限制
    #[serde(default)]
    pub quota_limit: i64,
    
    /// Expire at (Unix timestamp)
    /// 过期时间（Unix 时间戳）
    pub expire_at: Option<i64>,
    
    /// Allowed models
    /// 允许的模型
    pub allowed_models: Option<Vec<String>>,
    
    /// Allowed IPs
    /// 允许的 IP
    pub allowed_ips: Option<Vec<String>>,
}

/// Update user request
/// 更新用户请求
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    /// Display name
    /// 显示名称
    pub display_name: Option<String>,
    
    /// Avatar URL
    /// 头像 URL
    pub avatar_url: Option<String>,
    
    /// Role
    /// 角色
    pub role: Option<String>,
    
    /// Status
    /// 状态
    pub status: Option<String>,
}

/// Update tenant request
/// 更新租户请求
#[derive(Debug, Deserialize)]
pub struct UpdateTenantRequest {
    /// Tenant name
    /// 租户名称
    pub name: Option<String>,
    
    /// Settings (JSON)
    /// 设置（JSON）
    pub settings: Option<serde_json::Value>,
}
