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

/// Tenant information in JWT claims
/// JWT 声明中的租户信息
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

/// User claims in JWT token
/// JWT 令牌中的用户声明
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserClaims {
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
