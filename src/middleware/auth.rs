//! Authentication middleware for LuminaBridge
//!
//! Provides JWT token extraction, validation, and user context injection.
//! 提供 JWT 令牌提取、验证和用户上下文注入。

use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};
use tracing::{warn, info};

use crate::server::AppState;
use crate::error::{Error, Result};
use crate::auth::{AuthService, TokenClaims};

/// Authentication extension for request context
/// 用于请求上下文的认证扩展
#[derive(Clone, Debug)]
pub struct AuthExtension {
    /// User claims from JWT token
    /// JWT 令牌中的用户声明
    pub claims: TokenClaims,
}

/// Extract JWT token from Authorization header
/// 从 Authorization 头部提取 JWT 令牌
///
/// # Arguments
///
/// * `auth_header` - Authorization header value
///
/// # Returns
///
/// * `Result<String>` - Extracted token
pub fn extract_token_from_header(auth_header: &str) -> Result<String> {
    // Expected format: "Bearer <token>"
    if !auth_header.starts_with("Bearer ") {
        return Err(Error::TokenInvalid);
    }
    
    let token = auth_header[7..].trim();
    if token.is_empty() {
        return Err(Error::TokenInvalid);
    }
    
    Ok(token.to_string())
}

/// Middleware for required authentication
/// 强制认证中间件
///
/// This middleware requires a valid JWT token. Requests without a valid token
/// will be rejected with 401 Unauthorized.
///
/// 此中间件需要有效的 JWT 令牌。没有有效令牌的请求将被拒绝，返回 401 Unauthorized。
pub async fn require_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> std::result::Result<Response, (StatusCode, String)> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());
    
    let token = match auth_header {
        Some(header) => extract_token_from_header(header)
            .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?,
        None => return Err((StatusCode::UNAUTHORIZED, "Missing authorization header".to_string())),
    };
    
    // Validate token
    let auth_service = AuthService::new(state.config.oauth.clone());
    let claims = auth_service.validate_token(&token)
        .map_err(|e| {
            warn!("Token validation failed: {}", e);
            (StatusCode::UNAUTHORIZED, "Invalid or expired token".to_string())
        })?;
    
    // Inject claims into request extensions
    request.extensions_mut().insert(AuthExtension { claims });
    
    info!("Authenticated request");
    Ok(next.run(request).await)
}

/// Middleware for optional authentication
/// 可选认证中间件
///
/// This middleware attempts to authenticate the request, but doesn't require it.
/// If a valid token is present, claims are injected into the request context.
/// If no token or invalid token, the request proceeds without authentication.
///
/// 此中间件尝试认证请求，但不强制要求。如果存在有效令牌，则将声明注入请求上下文。
/// 如果没有令牌或令牌无效，请求将继续进行而无需认证。
pub async fn optional_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());
    
    if let Some(header) = auth_header {
        if let Ok(token) = extract_token_from_header(header) {
            let auth_service = AuthService::new(state.config.oauth.clone());
            if let Ok(claims) = auth_service.validate_token(&token) {
                request.extensions_mut().insert(AuthExtension { claims });
                info!("Optional auth: authenticated");
                return next.run(request).await;
            } else {
                warn!("Optional auth: invalid token, proceeding without auth");
            }
        }
    }
    
    info!("Optional auth: no token, proceeding without auth");
    next.run(request).await
}

/// Extract authentication extension from request
/// 从请求中提取认证扩展
///
/// # Arguments
///
/// * `request` - The request to extract from
///
/// # Returns
///
/// * `Option<&AuthExtension>` - Auth extension if present
pub fn get_auth_extension(request: &Request) -> Option<&AuthExtension> {
    request.extensions().get::<AuthExtension>()
}

/// Extract claims from request
/// 从请求中提取声明
///
/// # Arguments
///
/// * `request` - The request to extract from
///
/// # Returns
///
/// * `Option<&TokenClaims>` - Token claims if authenticated
pub fn get_claims(request: &Request) -> Option<&TokenClaims> {
    get_auth_extension(request).map(|ext| &ext.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_token_from_header() {
        // Valid Bearer token
        let token = extract_token_from_header("Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...").unwrap();
        assert_eq!(token, "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...");
        
        // Missing Bearer prefix
        let result = extract_token_from_header("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...");
        assert!(result.is_err());
        
        // Empty token
        let result = extract_token_from_header("Bearer ");
        assert!(result.is_err());
        
        // Extra whitespace
        let token = extract_token_from_header("Bearer   eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...  ").unwrap();
        assert_eq!(token, "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...");
    }

    #[test]
    fn test_auth_extension() {
        use crate::auth::TenantClaims;
        
        let claims = TokenClaims {
            sub: "123".to_string(),
            user_id: 123,
            email: "test@example.com".to_string(),
            display_name: Some("Test User".to_string()),
            role: "user".to_string(),
            tenant: TenantClaims {
                tenant_id: 1,
                tenant_name: "Default".to_string(),
                tenant_slug: "default".to_string(),
            },
            exp: 9999999999,
            iat: 1000000000,
        };
        
        let ext = AuthExtension { claims: claims.clone() };
        assert_eq!(ext.claims.user_id, 123);
        assert_eq!(ext.claims.email, "test@example.com");
    }
}
