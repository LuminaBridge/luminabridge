//! Middleware module for LuminaBridge
//!
//! Provides HTTP middleware for authentication, logging, and other cross-cutting concerns.
//! 为认证、日志记录和其他横切关注点提供 HTTP 中间件。

pub mod auth;
pub mod rate_limit;
pub mod api_key_auth;

pub use auth::{
    require_auth,
    optional_auth,
    extract_token_from_header,
    get_auth_extension,
    get_claims,
    AuthExtension,
};

pub use rate_limit::{
    rate_limit_middleware,
    create_rate_limit_layer,
    RateLimiterState,
    RateLimitResult,
};

pub use api_key_auth::{
    api_key_auth,
    validate_api_token,
    check_model_permission,
    extract_api_key_from_header,
    get_api_key_auth,
    get_token,
    ApiKeyAuthExtension,
};
