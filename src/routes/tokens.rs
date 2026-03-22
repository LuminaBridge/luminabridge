//! Token management routes for LuminaBridge API
//!
//! Handles API token CRUD operations and quota management.
//! 处理 API 令牌 CRUD 操作和配额管理。

use axum::{
    routing::{get, post, delete, patch},
    Router,
    extract::{State, Json, Path, Query},
    response::Json as ResponseJson,
    Extension,
};
use serde::{Deserialize, Serialize};
use tracing::info;
use chrono::{DateTime, Utc};

use crate::server::AppState;
use crate::error::{Error, Result};
use crate::types::{SuccessResponse, PaginationParams};

/// Create token routes
/// 创建令牌路由
pub fn token_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list_tokens))
        .route("/", post(create_token))
        .route("/:id", get(get_token))
        .route("/:id", delete(delete_token))
        .route("/:id/quota", patch(update_quota))
        .route("/:id/regenerate", post(regenerate_token))
        .with_state(state)
}

/// Token list query parameters
/// 令牌列表查询参数
#[derive(Debug, Deserialize)]
pub struct TokenListParams {
    #[serde(flatten)]
    pub pagination: PaginationParams,
    
    /// Filter by status
    /// 按状态筛选
    pub status: Option<String>,
}

/// Token DTO
/// 令牌数据传输对象
#[derive(Debug, Serialize, Clone)]
pub struct TokenDTO {
    /// Token ID
    /// 令牌 ID
    pub id: i64,
    
    /// Tenant ID
    /// 租户 ID
    pub tenant_id: i64,
    
    /// User ID (optional)
    /// 用户 ID（可选）
    pub user_id: Option<i64>,
    
    /// Token key (masked for security)
    /// 令牌密钥（掩码显示）
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
    
    /// Expire timestamp
    /// 过期时间
    pub expire_at: Option<DateTime<Utc>>,
    
    /// Status
    /// 状态
    pub status: String,
    
    /// Allowed IPs
    /// 允许的 IP
    pub allowed_ips: Option<Vec<String>>,
    
    /// Allowed models
    /// 允许的模型
    pub allowed_models: Option<Vec<String>>,
    
    /// Last used timestamp
    /// 最后使用时间
    pub last_used_at: Option<DateTime<Utc>>,
    
    /// Created at
    /// 创建时间
    pub created_at: DateTime<Utc>,
    
    /// Updated at
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// Create token request
/// 创建令牌请求
#[derive(Debug, Deserialize)]
pub struct CreateTokenRequest {
    /// Token name
    /// 令牌名称
    pub name: Option<String>,
    
    /// Quota limit (0 = unlimited)
    /// 配额限制（0 = 无限制）
    #[serde(default)]
    pub quota_limit: i64,
    
    /// Expire timestamp (Unix timestamp)
    /// 过期时间（Unix 时间戳）
    pub expire_at: Option<i64>,
    
    /// Allowed models
    /// 允许的模型
    pub allowed_models: Option<Vec<String>>,
    
    /// Allowed IPs
    /// 允许的 IP
    pub allowed_ips: Option<Vec<String>>,
}

/// Update quota request
/// 更新配额请求
#[derive(Debug, Deserialize)]
pub struct UpdateQuotaRequest {
    /// New quota limit
    /// 新配额限制
    pub quota_limit: i64,
}

/// Regenerate token response
/// 重新生成令牌响应
#[derive(Debug, Serialize)]
pub struct RegenerateTokenResponse {
    /// Token ID
    /// 令牌 ID
    pub id: i64,
    
    /// New token key
    /// 新令牌密钥
    pub key: String,
    
    /// Message
    /// 消息
    pub message: String,
}

/// List tokens handler
/// 列出令牌处理器
///
/// GET /api/v1/tokens
async fn list_tokens(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::auth::TokenClaims>,
    Query(params): Query<TokenListParams>,
) -> Result<ResponseJson<SuccessResponse<Vec<TokenDTO>>>> {
    info!("Listing tokens for tenant {}", claims.tenant.tenant_id);
    
    let tenant_id = claims.tenant.tenant_id;
    
    let tokens = state.db.find_tokens_by_tenant(tenant_id, &params).await?;
    let total = state.db.count_tokens(tenant_id, &params).await?;
    
    let token_dtos: Vec<TokenDTO> = tokens.into_iter().map(TokenDTO::from).collect();
    
    Ok(ResponseJson(SuccessResponse::new(token_dtos)
        .with_meta(crate::types::ResponseMeta::for_pagination(
            params.pagination.page,
            params.pagination.page_size,
            total,
        ))))
}

/// Create token handler
/// 创建令牌处理器
///
/// POST /api/v1/tokens
async fn create_token(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::auth::TokenClaims>,
    Json(payload): Json<CreateTokenRequest>,
) -> Result<ResponseJson<SuccessResponse<TokenDTO>>> {
    info!("Creating token for tenant {}", claims.tenant.tenant_id);
    
    let tenant_id = claims.tenant.tenant_id;
    let user_id = claims.user_id;
    
    // Generate token key
    let token_key = generate_token_key();
    
    // Create token
    let token = state.db.create_token(
        tenant_id,
        Some(user_id),
        &token_key,
        &payload,
    ).await?;
    
    let mut response = TokenDTO::from(&token);
    // Return full key only on creation
    response.key = token_key;
    
    Ok(ResponseJson(SuccessResponse::new(response)
        .with_message("令牌创建成功")))
}

/// Get token handler
/// 获取令牌处理器
///
/// GET /api/v1/tokens/:id
async fn get_token(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::auth::TokenClaims>,
    Path(token_id): Path<i64>,
) -> Result<ResponseJson<SuccessResponse<TokenDTO>>> {
    info!("Getting token {} for tenant {}", token_id, claims.tenant.tenant_id);
    
    let token = state.db.find_token(token_id).await?
        .ok_or_else(|| Error::Validation("Token not found".to_string()))?;
    
    if token.tenant_id != claims.tenant.tenant_id {
        return Err(Error::Validation("Token not found".to_string()));
    }
    
    Ok(ResponseJson(SuccessResponse::new(TokenDTO::from(&token))))
}

/// Delete token handler
/// 删除令牌处理器
///
/// DELETE /api/v1/tokens/:id
async fn delete_token(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::auth::TokenClaims>,
    Path(token_id): Path<i64>,
) -> Result<ResponseJson<SuccessResponse<serde_json::Value>>> {
    info!("Deleting token {} for tenant {}", token_id, claims.tenant.tenant_id);
    
    let token = state.db.find_token(token_id).await?
        .ok_or_else(|| Error::Validation("Token not found".to_string()))?;
    
    if token.tenant_id != claims.tenant.tenant_id {
        return Err(Error::Validation("Token not found".to_string()));
    }
    
    state.db.delete_token(token_id).await?;
    
    Ok(ResponseJson(SuccessResponse::new(serde_json::json!({
        "deleted": true,
        "id": token_id
    })).with_message("令牌删除成功")))
}

/// Update quota handler
/// 更新配额处理器
///
/// PATCH /api/v1/tokens/:id/quota
async fn update_quota(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::auth::TokenClaims>,
    Path(token_id): Path<i64>,
    Json(payload): Json<UpdateQuotaRequest>,
) -> Result<ResponseJson<SuccessResponse<TokenDTO>>> {
    info!("Updating quota for token {} (tenant {})", token_id, claims.tenant.tenant_id);
    
    let token = state.db.find_token(token_id).await?
        .ok_or_else(|| Error::Validation("Token not found".to_string()))?;
    
    if token.tenant_id != claims.tenant.tenant_id {
        return Err(Error::Validation("Token not found".to_string()));
    }
    
    let updated = state.db.update_token_quota(token_id, payload.quota_limit).await?;
    
    Ok(ResponseJson(SuccessResponse::new(TokenDTO::from(&updated))
        .with_message("配额更新成功")))
}

/// Regenerate token handler
/// 重新生成令牌处理器
///
/// POST /api/v1/tokens/:id/regenerate
async fn regenerate_token(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::auth::TokenClaims>,
    Path(token_id): Path<i64>,
) -> Result<ResponseJson<SuccessResponse<RegenerateTokenResponse>>> {
    info!("Regenerating token {} for tenant {}", token_id, claims.tenant.tenant_id);
    
    let token = state.db.find_token(token_id).await?
        .ok_or_else(|| Error::Validation("Token not found".to_string()))?;
    
    if token.tenant_id != claims.tenant.tenant_id {
        return Err(Error::Validation("Token not found".to_string()));
    }
    
    let new_key = generate_token_key();
    state.db.update_token_key(token_id, &new_key).await?;
    
    Ok(ResponseJson(SuccessResponse::new(RegenerateTokenResponse {
        id: token_id,
        key: new_key,
        message: "令牌已重新生成，请妥善保存新密钥".to_string(),
    })))
}

/// Generate a secure token key
/// 生成安全的令牌密钥
fn generate_token_key() -> String {
    use uuid::Uuid;
    format!("sk-{}", Uuid::new_v4().to_string().replace('-', ""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_key_generation() {
        let key = generate_token_key();
        assert!(key.starts_with("sk-"));
        assert_eq!(key.len(), 36); // sk- + 32 hex chars (without dashes)
    }
}
