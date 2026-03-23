//! User management routes for LuminaBridge API
//!
//! Handles user CRUD operations and user-specific data.
//! 处理用户 CRUD 操作和用户特定数据。

use axum::{
    routing::{get, post, put, delete},
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
use crate::types::{SuccessResponse, UpdateUserRequest};
use crate::routes::auth::UserDTO;
use crate::auth::TokenClaims;
pub use crate::db::UserListParams;

/// Create user routes
/// 创建用户路由
pub fn user_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list_users))
        .route("/:id", get(get_user))
        .route("/:id", put(update_user))
        .route("/:id", delete(delete_user))
        .route("/invite", post(invite_user))
        .route("/:id/usage", get(get_user_usage))
        .route("/me", get(get_current_user))
        .with_state(state)
}

/// User detail DTO
/// 用户详情 DTO
#[derive(Debug, Serialize, Clone)]
pub struct UserDetailDTO {
    /// User ID
    /// 用户 ID
    pub id: i64,
    
    /// Email
    /// 电子邮件
    pub email: String,
    
    /// Display name
    /// 显示名称
    pub display_name: Option<String>,
    
    /// Avatar URL
    /// 头像 URL
    pub avatar_url: Option<String>,
    
    /// Role
    /// 角色
    pub role: String,
    
    /// Status
    /// 状态
    pub status: String,
    
    /// Tenant ID
    /// 租户 ID
    pub tenant_id: i64,
    
    /// OAuth provider
    /// OAuth 提供商
    pub oauth_provider: Option<String>,
    
    /// Created at
    /// 创建时间
    pub created_at: DateTime<Utc>,
    
    /// Updated at
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    
    /// Last login at
    /// 最后登录时间
    pub last_login_at: Option<DateTime<Utc>>,
}

impl From<&crate::db::User> for UserDetailDTO {
    fn from(user: &crate::db::User) -> Self {
        UserDetailDTO {
            id: user.id,
            email: user.email.clone(),
            display_name: user.display_name.clone(),
            avatar_url: user.avatar_url.clone(),
            role: user.role.clone(),
            status: user.status.clone(),
            tenant_id: user.tenant_id,
            oauth_provider: user.oauth_provider.clone(),
            created_at: user.created_at,
            updated_at: user.updated_at,
            last_login_at: user.last_login_at,
        }
    }
}

/// Invite user request
/// 邀请用户请求
#[derive(Debug, Deserialize)]
pub struct InviteUserRequest {
    /// Email to invite
    /// 要邀请的电子邮件
    pub email: String,
    
    /// Role
    /// 角色
    pub role: Option<String>,
    
    /// Message
    /// 消息
    pub message: Option<String>,
}

/// User usage stats
/// 用户用量统计
#[derive(Debug, Serialize)]
pub struct UserUsageStats {
    /// Total requests
    /// 总请求数
    pub total_requests: i64,
    
    /// Total tokens
    /// 总 token 数
    pub total_tokens: i64,
    
    /// Total cost
    /// 总费用
    pub total_cost: f64,
    
    /// Requests by model
    /// 按模型划分的请求
    pub requests_by_model: Vec<ModelUsage>,
}

#[derive(Debug, Serialize)]
pub struct ModelUsage {
    /// Model name
    /// 模型名称
    pub model: String,
    
    /// Request count
    /// 请求数
    pub count: i64,
    
    /// Token count
    /// Token 数
    pub tokens: i64,
}

/// List users handler
/// 列出用户处理器
///
/// GET /api/v1/users
async fn list_users(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::auth::TokenClaims>,
    Query(params): Query<UserListParams>,
) -> Result<ResponseJson<SuccessResponse<Vec<UserDetailDTO>>>> {
    info!("Listing users for tenant {}", claims.tenant.tenant_id);
    
    let tenant_id = claims.tenant.tenant_id;
    
    let users = state.db.find_users_by_tenant(tenant_id, &params).await?;
    let total = state.db.count_users(tenant_id, &params).await?;
    
    let user_dtos: Vec<UserDetailDTO> = users.iter().map(UserDetailDTO::from).collect();
    
    Ok(ResponseJson(SuccessResponse::new(user_dtos)
        .with_meta(crate::types::ResponseMeta::for_pagination(
            params.pagination.page,
            params.pagination.page_size,
            total,
        ))))
}

/// Get user handler
/// 获取用户处理器
///
/// GET /api/v1/users/:id
async fn get_user(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::auth::TokenClaims>,
    Path(user_id): Path<i64>,
) -> Result<ResponseJson<SuccessResponse<UserDetailDTO>>> {
    info!("Getting user {} for tenant {}", user_id, claims.tenant.tenant_id);
    
    let user = state.db.find_user(user_id).await?
        .ok_or_else(|| Error::Validation("User not found".to_string()))?;
    
    if user.tenant_id != claims.tenant.tenant_id {
        return Err(Error::Validation("User not found".to_string()));
    }
    
    Ok(ResponseJson(SuccessResponse::new(UserDetailDTO::from(&user))))
}

/// Update user handler
/// 更新用户处理器
///
/// PUT /api/v1/users/:id
async fn update_user(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::auth::TokenClaims>,
    Path(user_id): Path<i64>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<ResponseJson<SuccessResponse<UserDetailDTO>>> {
    info!("Updating user {} for tenant {}", user_id, claims.tenant.tenant_id);
    
    let user = state.db.find_user(user_id).await?
        .ok_or_else(|| Error::Validation("User not found".to_string()))?;
    
    if user.tenant_id != claims.tenant.tenant_id {
        return Err(Error::Validation("User not found".to_string()));
    }
    
    let updated = state.db.update_user(user_id, &payload).await?;
    
    Ok(ResponseJson(SuccessResponse::new(UserDetailDTO::from(&updated))
        .with_message("用户更新成功")))
}

/// Delete user handler
/// 删除用户处理器
///
/// DELETE /api/v1/users/:id
async fn delete_user(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::auth::TokenClaims>,
    Path(user_id): Path<i64>,
) -> Result<ResponseJson<SuccessResponse<serde_json::Value>>> {
    info!("Deleting user {} for tenant {}", user_id, claims.tenant.tenant_id);
    
    // Prevent self-deletion
    if user_id == claims.user_id {
        return Err(Error::Validation("Cannot delete yourself".to_string()));
    }
    
    let user = state.db.find_user(user_id).await?
        .ok_or_else(|| Error::Validation("User not found".to_string()))?;
    
    if user.tenant_id != claims.tenant.tenant_id {
        return Err(Error::Validation("User not found".to_string()));
    }
    
    state.db.delete_user(user_id).await?;
    
    Ok(ResponseJson(SuccessResponse::new(serde_json::json!({
        "deleted": true,
        "id": user_id
    })).with_message("用户删除成功")))
}

/// Invite user handler
/// 邀请用户处理器
///
/// POST /api/v1/users/invite
async fn invite_user(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::auth::TokenClaims>,
    Json(payload): Json<InviteUserRequest>,
) -> Result<ResponseJson<SuccessResponse<serde_json::Value>>> {
    info!("Inviting user {} to tenant {}", payload.email, claims.tenant.tenant_id);
    
    // Check if user already exists
    if let Some(_existing) = state.db.find_user_by_email(&payload.email).await? {
        return Err(Error::Validation("User already exists".to_string()));
    }
    
    // Generate invitation
    let invitation_code = generate_invitation_code();
    
    // In production, send email invitation
    // For now, just return the invitation code
    
    Ok(ResponseJson(SuccessResponse::new(serde_json::json!({
        "invitation_code": invitation_code,
        "email": payload.email,
        "expires_in": 86400 // 24 hours
    })).with_message("邀请已生成，请发送给被邀请人")))
}

/// Get user usage handler
/// 获取用户用量处理器
///
/// GET /api/v1/users/:id/usage
async fn get_user_usage(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::auth::TokenClaims>,
    Path(user_id): Path<i64>,
) -> Result<ResponseJson<SuccessResponse<UserUsageStats>>> {
    info!("Getting usage for user {} (tenant {})", user_id, claims.tenant.tenant_id);
    
    let user = state.db.find_user(user_id).await?
        .ok_or_else(|| Error::Validation("User not found".to_string()))?;
    
    if user.tenant_id != claims.tenant.tenant_id {
        return Err(Error::Validation("User not found".to_string()));
    }
    
    let stats = state.db.get_user_usage_stats(user_id).await?;
    
    Ok(ResponseJson(SuccessResponse::new(stats)))
}

/// Get current user handler
/// 获取当前用户处理器
///
/// GET /api/v1/users/me
async fn get_current_user(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::auth::TokenClaims>,
) -> Result<ResponseJson<SuccessResponse<UserDetailDTO>>> {
    let user = state.db.find_user(claims.user_id).await?
        .ok_or_else(|| Error::Validation("User not found".to_string()))?;
    
    Ok(ResponseJson(SuccessResponse::new(UserDetailDTO::from(&user))))
}

/// Generate invitation code
/// 生成邀请码
fn generate_invitation_code() -> String {
    use uuid::Uuid;
    format!("inv-{}", Uuid::new_v4().to_string()[..8].to_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invitation_code_generation() {
        let code = generate_invitation_code();
        assert!(code.starts_with("inv-"));
        assert_eq!(code.len(), 13); // inv- + 8 chars
    }
}
