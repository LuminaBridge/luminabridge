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

    /// Cache errors
    /// 缓存错误
    #[error("Cache error: {0}")]
    Cache(String),
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
            Error::RateLimit(_) => 429,
            Error::Provider(_) => 502,
            Error::Internal(_) => 500,
            Error::Validation(_) => 400,
            Error::Cache(_) => 500,
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
            Error::RateLimit(_) => "RATE_LIMIT_EXCEEDED",
            Error::Provider(_) => "PROVIDER_ERROR",
            Error::Internal(_) => "INTERNAL_ERROR",
            Error::Validation(_) => "VALIDATION_ERROR",
            Error::Cache(_) => "CACHE_ERROR",
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
