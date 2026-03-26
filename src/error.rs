//! Error handling module for LuminaBridge
//!
//! Provides comprehensive error types and handling for the gateway.
//! 为网关提供全面的错误类型和处理。

use thiserror::Error;
use std::fmt;

/// Result type alias for LuminaBridge operations
/// LuminaBridge 操作的 Result 类型别名
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for LuminaBridge
/// LuminaBridge 的主要错误类型
#[derive(Error, Debug)]
pub enum Error {
    /// Configuration errors
    /// 配置错误
    #[error("Configuration error: {0}")]
    Config(String),

    /// Database errors
    /// 数据库错误
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// OAuth authentication errors
    /// OAuth 认证错误
    #[error("OAuth error: {0}")]
    OAuth(String),

    /// JWT token errors
    /// JWT 令牌错误
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    /// HTTP client errors
    /// HTTP 客户端错误
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Server errors
    /// 服务器错误
    #[error("Server error: {0}")]
    Server(String),

    /// Authentication/Authorization errors
    /// 认证/授权错误
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Invalid credentials (email/password)
    /// 无效凭证（邮箱/密码）
    #[error("Invalid credentials")]
    InvalidCredentials,

    /// User already exists
    /// 用户已存在
    #[error("User already exists")]
    UserAlreadyExists,

    /// Token expired
    /// 令牌已过期
    #[error("Token expired")]
    TokenExpired,

    /// Token invalid
    /// 令牌无效
    #[error("Token invalid")]
    TokenInvalid,

    /// OAuth failed
    /// OAuth 失败
    #[error("OAuth failed: {0}")]
    OAuthFailed(String),

    /// Rate limiting errors
    /// 速率限制错误
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    /// Provider errors (AI provider specific)
    /// 提供商错误（AI 提供商特定）
    #[error("Provider error: {0}")]
    Provider(String),

    /// Internal errors
    /// 内部错误
    #[error("Internal error: {0}")]
    Internal(String),

    /// Validation errors
    /// 验证错误
    #[error("Validation error: {0}")]
    Validation(String),

    /// JSON serialization errors
    /// JSON 序列化错误
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Cache errors
    /// 缓存错误
    #[error("Cache error: {0}")]
    Cache(String),

    /// API Token not found
    /// API 令牌未找到
    #[error("Token not found")]
    TokenNotFound,

    /// API Token quota exceeded
    /// API 令牌配额超出
    #[error("Token quota exceeded")]
    TokenQuotaExceeded,

    /// Network errors
    /// 网络错误
    #[error("Network error: {0}")]
    Network(String),

    /// Streaming quota exceeded (tokens used, limit)
    /// 流式配额超出（已用令牌数，限制）
    #[error("Streaming quota exceeded: used {0} tokens, limit {1}")]
    QuotaExceeded(i64, i64),

    /// Model not permitted for this token
    /// 此令牌不允许访问该模型
    #[error("Model not permitted")]
    ModelNotPermitted,

    /// IP not allowed
    /// IP 不允许
    #[error("IP address not allowed")]
    IpNotAllowed,
}

/// HTTP status code mapping for errors
/// 错误的 HTTP 状态码映射
impl Error {
    /// Get the HTTP status code for this error
    /// 获取此错误的 HTTP 状态码
    pub fn status_code(&self) -> u16 {
        match self {
            Error::Config(_) => 500,
            Error::Database(_) => 500,
            Error::OAuth(_) => 401,
            Error::Jwt(_) => 401,
            Error::Http(e) => e.status().map(|s| s.as_u16()).unwrap_or(502),
            Error::Server(_) => 500,
            Error::Auth(_) => 401,
            Error::InvalidCredentials => 401,
            Error::UserAlreadyExists => 409,
            Error::TokenExpired => 401,
            Error::TokenInvalid => 401,
            Error::OAuthFailed(_) => 400,
            Error::RateLimit(_) => 429,
            Error::Provider(_) => 502,
            Error::Internal(_) => 500,
            Error::Validation(_) => 400,
            Error::Json(_) => 500,
            Error::Cache(_) => 500,
            Error::TokenNotFound => 404,
            Error::TokenQuotaExceeded => 429,
            Error::Network(_) => 502,
            Error::QuotaExceeded(_, _) => 429,
            Error::ModelNotPermitted => 403,
            Error::IpNotAllowed => 403,
        }
    }

    /// Check if this error is recoverable
    /// 检查此错误是否可恢复
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Error::Http(_) | Error::Provider(_) | Error::Cache(_)
        )
    }
}

/// Convert to Axum response
/// 转换为 Axum 响应
#[cfg(feature = "full")]
impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        use axum::{
            http::StatusCode,
            response::Json,
        };
        use serde_json::json;

        let status = StatusCode::from_u16(self.status_code())
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        let body = Json(json!({
            "error": {
                "code": self.error_code(),
                "message": self.to_string(),
                "type": "luminabridge_error"
            }
        }));

        (status, body).into_response()
    }
}

impl Error {
    /// Get error code string
    /// 获取错误代码字符串
    fn error_code(&self) -> &'static str {
        match self {
            Error::Config(_) => "CONFIG_ERROR",
            Error::Database(_) => "DATABASE_ERROR",
            Error::OAuth(_) => "OAUTH_ERROR",
            Error::Jwt(_) => "JWT_ERROR",
            Error::Http(_) => "HTTP_ERROR",
            Error::Server(_) => "SERVER_ERROR",
            Error::Auth(_) => "AUTH_ERROR",
            Error::InvalidCredentials => "INVALID_CREDENTIALS",
            Error::UserAlreadyExists => "USER_ALREADY_EXISTS",
            Error::TokenExpired => "TOKEN_EXPIRED",
            Error::TokenInvalid => "TOKEN_INVALID",
            Error::OAuthFailed(_) => "OAUTH_FAILED",
            Error::RateLimit(_) => "RATE_LIMIT_EXCEEDED",
            Error::Provider(_) => "PROVIDER_ERROR",
            Error::Internal(_) => "INTERNAL_ERROR",
            Error::Validation(_) => "VALIDATION_ERROR",
            Error::Json(_) => "JSON_ERROR",
            Error::Cache(_) => "CACHE_ERROR",
            Error::TokenNotFound => "TOKEN_NOT_FOUND",
            Error::TokenQuotaExceeded => "TOKEN_QUOTA_EXCEEDED",
            Error::Network(_) => "NETWORK_ERROR",
            Error::QuotaExceeded(_, _) => "QUOTA_EXCEEDED",
            Error::ModelNotPermitted => "MODEL_NOT_PERMITTED",
            Error::IpNotAllowed => "IP_NOT_ALLOWED",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(Error::Auth("test".to_string()).status_code(), 401);
        assert_eq!(Error::RateLimit("test".to_string()).status_code(), 429);
        assert_eq!(Error::Validation("test".to_string()).status_code(), 400);
        assert_eq!(Error::Internal("test".to_string()).status_code(), 500);
    }

    #[test]
    fn test_error_display() {
        let err = Error::Config("missing field".to_string());
        assert!(err.to_string().contains("Configuration error"));
    }
}
