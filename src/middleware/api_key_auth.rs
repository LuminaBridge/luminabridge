//! API Key Authentication Middleware for LuminaBridge
//!
//! Provides API token validation, quota checking, and access control.
//! 提供 API 令牌验证、配额检查和访问控制。

use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};
use tracing::{warn, info, debug};
use chrono::Utc;

use crate::server::AppState;
use crate::error::{Error, Result};
use crate::db::models::Token;

/// API Key authentication extension for request context
/// 用于请求上下文的 API 密钥认证扩展
#[derive(Clone, Debug)]
pub struct ApiKeyAuthExtension {
    /// Token information from database
    /// 来自数据库的令牌信息
    pub token: Token,
    
    /// Client IP address
    /// 客户端 IP 地址
    pub client_ip: Option<String>,
}

/// Extract API key from Authorization header
/// 从 Authorization 头部提取 API 密钥
///
/// # Arguments
///
/// * `auth_header` - Authorization header value
///
/// # Returns
///
/// * `Result<String>` - Extracted API key
pub fn extract_api_key_from_header(auth_header: &str) -> Result<String> {
    // Expected format: "Bearer sk-xxx"
    if !auth_header.starts_with("Bearer ") {
        return Err(Error::Auth("Invalid Authorization header format. Expected 'Bearer sk-xxx'".to_string()));
    }
    
    let api_key = auth_header[7..].trim();
    if api_key.is_empty() {
        return Err(Error::Auth("Missing API key in Authorization header".to_string()));
    }
    
    Ok(api_key.to_string())
}

/// Get client IP address from request
/// 从请求中获取客户端 IP 地址
///
/// # Arguments
///
/// * `request` - The HTTP request
///
/// # Returns
///
/// * `Option<String>` - Client IP address if available
fn get_client_ip(request: &Request) -> Option<String> {
    // Try X-Forwarded-For header first
    if let Some(xff) = request.headers().get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
        // X-Forwarded-For can contain multiple IPs, take the first one
        return Some(xff.split(',').next().unwrap_or("").trim().to_string());
    }
    
    // Try X-Real-IP header
    if let Some(xri) = request.headers().get("x-real-ip").and_then(|v| v.to_str().ok()) {
        return Some(xri.trim().to_string());
    }
    
    // Fall back to remote address
    if let Some(addr) = request.extensions().get::<axum::extract::ConnectInfo<std::net::SocketAddr>>() {
        return Some(addr.0.ip().to_string());
    }
    
    None
}

/// Middleware for API key authentication
/// API 密钥认证中间件
///
/// This middleware validates API tokens from the Authorization header.
/// It performs the following checks:
/// 1. Extract API key from Authorization: Bearer sk-xxx
/// 2. Validate token exists in database
/// 3. Check token is not expired
/// 4. Check token quota is not exceeded
/// 5. Check client IP is in whitelist (if configured)
/// 6. Check token status is active
///
/// 此中间件验证来自 Authorization 头部的 API 令牌。
/// 它执行以下检查：
/// 1. 从 Authorization: Bearer sk-xxx 提取 API 密钥
/// 2. 验证令牌存在于数据库中
/// 3. 检查令牌未过期
/// 4. 检查令牌配额未超出
/// 5. 检查客户端 IP 在白名单中（如果配置）
/// 6. 检查令牌状态为活跃
pub async fn api_key_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> std::result::Result<Response, (StatusCode, String)> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());
    
    let auth_header = match auth_header {
        Some(header) => header,
        None => return Err((StatusCode::UNAUTHORIZED, "Missing authorization header".to_string())),
    };
    
    // Extract API key
    let api_key = extract_api_key_from_header(auth_header)
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    
    // Get client IP
    let client_ip = get_client_ip(&request);
    
    // Validate token
    let token = validate_api_token(&state, &api_key, &client_ip).await
        .map_err(|e| {
            warn!("API token validation failed: {}", e);
            match e {
                Error::TokenNotFound => (StatusCode::NOT_FOUND, "Invalid API key".to_string()),
                Error::TokenExpired => (StatusCode::UNAUTHORIZED, "API key expired".to_string()),
                Error::TokenQuotaExceeded => (StatusCode::TOO_MANY_REQUESTS, "API key quota exceeded".to_string()),
                Error::ModelNotPermitted => (StatusCode::FORBIDDEN, "Model not permitted for this API key".to_string()),
                Error::IpNotAllowed => (StatusCode::FORBIDDEN, "IP address not allowed".to_string()),
                _ => (StatusCode::UNAUTHORIZED, "Invalid API key".to_string()),
            }
        })?;
    
    info!("API key authenticated: token_id={}, tenant_id={}", token.id, token.tenant_id);
    
    // Inject token info into request extensions
    request.extensions_mut().insert(ApiKeyAuthExtension {
        token,
        client_ip,
    });
    
    Ok(next.run(request).await)
}

/// Validate API token with all checks
/// 验证 API 令牌并进行所有检查
///
/// # Arguments
///
/// * `state` - Application state
/// * `api_key` - API key to validate
/// * `client_ip` - Client IP address (optional)
///
/// # Returns
///
/// * `Result<Token>` - Validated token
pub async fn validate_api_token(
    state: &AppState,
    api_key: &str,
    client_ip: &Option<String>,
) -> Result<Token> {
    debug!("Validating API token: {}", api_key);
    
    // Find token by key
    let token = state.db.find_token_by_key(api_key).await?
        .ok_or(Error::TokenNotFound)?;
    
    // Check token status
    if token.status != "active" {
        return Err(Error::TokenExpired);
    }
    
    // Check expiration
    if let Some(expire_at) = token.expire_at {
        if Utc::now() > expire_at {
            return Err(Error::TokenExpired);
        }
    }
    
    // Check quota
    if !check_token_quota(&token).await? {
        return Err(Error::TokenQuotaExceeded);
    }
    
    // Check IP whitelist
    if let Some(allowed_ips) = &token.allowed_ips {
        if let Some(ip) = client_ip {
            if !is_ip_allowed(allowed_ips, ip) {
                return Err(Error::IpNotAllowed);
            }
        }
    }
    
    Ok(token)
}

/// Check if token has sufficient quota
/// 检查令牌是否有足够的配额
///
/// # Arguments
///
/// * `token` - Token to check
///
/// # Returns
///
/// * `Result<bool>` - true if quota is sufficient
pub async fn check_token_quota(token: &Token) -> Result<bool> {
    // If no quota limit is set, allow unlimited
    let quota_limit = match token.quota_limit {
        Some(limit) => limit,
        None => return Ok(true),
    };
    
    // Check if quota_used < quota_limit
    Ok(token.quota_used < quota_limit)
}

/// Check if IP address is in whitelist
/// 检查 IP 地址是否在白名单中
///
/// # Arguments
///
/// * `allowed_ips` - JSON array of allowed IPs
/// * `client_ip` - Client IP to check
///
/// # Returns
///
/// * `bool` - true if IP is allowed
fn is_ip_allowed(allowed_ips: &serde_json::Value, client_ip: &str) -> bool {
    if let Some(ip_array) = allowed_ips.as_array() {
        for ip_value in ip_array {
            if let Some(ip_str) = ip_value.as_str() {
                // Check exact match
                if ip_str == client_ip {
                    return true;
                }
                
                // Check CIDR notation (simplified - just check prefix for now)
                if ip_str.contains('/') {
                    let parts: Vec<&str> = ip_str.split('/').collect();
                    if parts.len() == 2 {
                        let network_prefix = parts[0];
                        // Simple prefix matching (proper CIDR would need more logic)
                        if client_ip.starts_with(network_prefix.rsplit('.').next().unwrap_or("")) {
                            return true;
                        }
                    }
                }
                
                // Check wildcard patterns
                if ip_str.contains('*') {
                    let pattern = ip_str.replace('*', ".*");
                    if let Ok(re) = regex::Regex::new(&format!("^{}$", pattern)) {
                        if re.is_match(client_ip) {
                            return true;
                        }
                    }
                }
            }
        }
    }
    
    // If whitelist is empty or not configured, allow all
    allowed_ips.as_array().map(|a| a.is_empty()).unwrap_or(true)
}

/// Check if model is permitted for this token
/// 检查此令牌是否允许访问该模型
///
/// # Arguments
///
/// * `token` - Token to check
/// * `model` - Model name to check
///
/// # Returns
///
/// * `Result<bool>` - true if model is permitted
pub fn check_model_permission(token: &Token, model: &str) -> Result<bool> {
    // If no model restrictions, allow all
    let allowed_models = match &token.allowed_models {
        Some(models) => models,
        None => return Ok(true),
    };
    
    if let Some(model_array) = allowed_models.as_array() {
        // Check if model is in allowed list
        for model_value in model_array {
            if let Some(allowed_model) = model_value.as_str() {
                // Exact match
                if allowed_model == model {
                    return Ok(true);
                }
                
                // Wildcard match (e.g., "gpt-*" matches "gpt-3.5-turbo")
                if allowed_model.ends_with('*') {
                    let prefix = &allowed_model[..allowed_model.len() - 1];
                    if model.starts_with(prefix) {
                        return Ok(true);
                    }
                }
            }
        }
        return Ok(false);
    }
    
    // If allowed_models is null or empty, allow all
    Ok(true)
}

/// Extract API key auth extension from request
/// 从请求中提取 API 密钥认证扩展
///
/// # Arguments
///
/// * `request` - The request to extract from
///
/// # Returns
///
/// * `Option<&ApiKeyAuthExtension>` - Auth extension if present
pub fn get_api_key_auth(request: &Request) -> Option<&ApiKeyAuthExtension> {
    request.extensions().get::<ApiKeyAuthExtension>()
}

/// Extract token from request
/// 从请求中提取令牌
///
/// # Arguments
///
/// * `request` - The request to extract from
///
/// # Returns
///
/// * `Option<&Token>` - Token if authenticated
pub fn get_token(request: &Request) -> Option<&Token> {
    get_api_key_auth(request).map(|ext| &ext.token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_api_key_from_header() {
        // Valid Bearer token
        let key = extract_api_key_from_header("Bearer sk-test123").unwrap();
        assert_eq!(key, "sk-test123");
        
        // Missing Bearer prefix
        let result = extract_api_key_from_header("sk-test123");
        assert!(result.is_err());
        
        // Empty token
        let result = extract_api_key_from_header("Bearer ");
        assert!(result.is_err());
        
        // Extra whitespace
        let key = extract_api_key_from_header("Bearer   sk-test123  ").unwrap();
        assert_eq!(key, "sk-test123");
    }

    #[test]
    fn test_check_token_quota() {
        use crate::db::models::Token;
        use chrono::Utc;
        
        // Token with no quota limit
        let token = Token {
            id: 1,
            tenant_id: 1,
            user_id: None,
            key: "sk-test".to_string(),
            name: Some("Test".to_string()),
            quota_limit: None,
            quota_used: 1000,
            expire_at: None,
            status: "active".to_string(),
            allowed_ips: None,
            allowed_models: None,
            last_used_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        // Should allow unlimited
        assert!(tokio_test::block_on(check_token_quota(&token)).unwrap());
        
        // Token with quota limit and usage under limit
        let mut token_with_limit = token.clone();
        token_with_limit.quota_limit = Some(2000);
        assert!(tokio_test::block_on(check_token_quota(&token_with_limit)).unwrap());
        
        // Token with quota exceeded
        token_with_limit.quota_used = 2500;
        assert!(!tokio_test::block_on(check_token_quota(&token_with_limit)).unwrap());
    }

    #[test]
    fn test_check_model_permission() {
        use crate::db::models::Token;
        use chrono::Utc;
        
        let token = Token {
            id: 1,
            tenant_id: 1,
            user_id: None,
            key: "sk-test".to_string(),
            name: Some("Test".to_string()),
            quota_limit: None,
            quota_used: 0,
            expire_at: None,
            status: "active".to_string(),
            allowed_ips: None,
            allowed_models: Some(json!(["gpt-3.5-turbo", "gpt-4"])),
            last_used_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        // Allowed model
        assert!(check_model_permission(&token, "gpt-3.5-turbo").unwrap());
        assert!(check_model_permission(&token, "gpt-4").unwrap());
        
        // Not allowed model
        assert!(!check_model_permission(&token, "claude-3").unwrap());
    }

    #[test]
    fn test_is_ip_allowed() {
        use serde_json::json;
        
        // Exact match
        let allowed_ips = json!(["192.168.1.1", "10.0.0.1"]);
        assert!(is_ip_allowed(&allowed_ips, "192.168.1.1"));
        assert!(!is_ip_allowed(&allowed_ips, "192.168.1.2"));
        
        // Empty list allows all
        let allowed_ips = json!([]);
        assert!(is_ip_allowed(&allowed_ips, "192.168.1.1"));
        
        // Null allows all
        assert!(is_ip_allowed(&json!(null), "192.168.1.1"));
    }
}
