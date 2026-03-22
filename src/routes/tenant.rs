//! Tenant management routes for LuminaBridge API
//!
//! Handles tenant configuration and settings.
//! 处理租户配置和设置。

use axum::{
    routing::{get, put},
    Router,
    extract::{State, Json},
    response::Json as ResponseJson,
    Extension,
};
use serde::{Deserialize, Serialize};
use tracing::info;
use chrono::{DateTime, Utc};

use crate::server::AppState;
use crate::error::{Error, Result};
use crate::types::SuccessResponse;

/// Create tenant routes
/// 创建租户路由
pub fn tenant_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(get_tenant))
        .route("/", put(update_tenant))
        .route("/usage", get(get_tenant_usage))
        .route("/members", get(get_tenant_members))
        .with_state(state)
}

/// Tenant DTO
/// 租户数据传输对象
#[derive(Debug, Serialize, Clone)]
pub struct TenantDTO {
    /// Tenant ID
    /// 租户 ID
    pub id: i64,
    
    /// Tenant name
    /// 租户名称
    pub name: String,
    
    /// Tenant slug
    /// 租户标识
    pub slug: String,
    
    /// Status
    /// 状态
    pub status: String,
    
    /// Quota limit
    /// 配额限制
    pub quota_limit: Option<i64>,
    
    /// Quota used
    /// 已用配额
    pub quota_used: i64,
    
    /// Settings
    /// 设置
    pub settings: Option<serde_json::Value>,
    
    /// Created at
    /// 创建时间
    pub created_at: DateTime<Utc>,
    
    /// Updated at
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// Update tenant request
/// 更新租户请求
#[derive(Debug, Deserialize)]
pub struct UpdateTenantRequest {
    /// Tenant name
    /// 租户名称
    pub name: Option<String>,
    
    /// Settings
    /// 设置
    pub settings: Option<serde_json::Value>,
}

/// Tenant usage stats
/// 租户用量统计
#[derive(Debug, Serialize)]
pub struct TenantUsageStats {
    /// Current period start
    /// 当前周期开始
    pub period_start: DateTime<Utc>,
    
    /// Current period end
    /// 当前周期结束
    pub period_end: DateTime<Utc>,
    
    /// Quota limit
    /// 配额限制
    pub quota_limit: Option<i64>,
    
    /// Quota used
    /// 已用配额
    pub quota_used: i64,
    
    /// Quota remaining
    /// 剩余配额
    pub quota_remaining: Option<i64>,
    
    /// Usage percentage
    /// 使用百分比
    pub usage_percentage: f64,
    
    /// Total requests
    /// 总请求数
    pub total_requests: i64,
    
    /// Total tokens
    /// 总 token 数
    pub total_tokens: i64,
    
    /// Total cost
    /// 总费用
    pub total_cost: f64,
}

/// Tenant member
/// 租户成员
#[derive(Debug, Serialize)]
pub struct TenantMember {
    /// User ID
    /// 用户 ID
    pub user_id: i64,
    
    /// Email
    /// 电子邮件
    pub email: String,
    
    /// Display name
    /// 显示名称
    pub display_name: Option<String>,
    
    /// Role
    /// 角色
    pub role: String,
    
    /// Status
    /// 状态
    pub status: String,
    
    /// Joined at
    /// 加入时间
    pub joined_at: DateTime<Utc>,
}

/// Get tenant handler
/// 获取租户处理器
///
/// GET /api/v1/tenant
async fn get_tenant(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::auth::TokenClaims>,
) -> Result<ResponseJson<SuccessResponse<TenantDTO>>> {
    info!("Getting tenant {} for user {}", claims.tenant.tenant_id, claims.user_id);
    
    let tenant = state.db.find_tenant(claims.tenant.tenant_id).await?
        .ok_or_else(|| Error::Validation("Tenant not found".to_string()))?;
    
    Ok(ResponseJson(SuccessResponse::new(TenantDTO::from(&tenant))))
}

/// Update tenant handler
/// 更新租户处理器
///
/// PUT /api/v1/tenant
async fn update_tenant(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::auth::TokenClaims>,
    Json(payload): Json<UpdateTenantRequest>,
) -> Result<ResponseJson<SuccessResponse<TenantDTO>>> {
    info!("Updating tenant {} for user {}", claims.tenant.tenant_id, claims.user_id);
    
    let tenant = state.db.find_tenant(claims.tenant.tenant_id).await?
        .ok_or_else(|| Error::Validation("Tenant not found".to_string()))?;
    
    let updated = state.db.update_tenant(claims.tenant.tenant_id, &payload).await?;
    
    Ok(ResponseJson(SuccessResponse::new(TenantDTO::from(&updated))
        .with_message("租户配置更新成功")))
}

/// Get tenant usage handler
/// 获取租户用量处理器
///
/// GET /api/v1/tenant/usage
async fn get_tenant_usage(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::auth::TokenClaims>,
) -> Result<ResponseJson<SuccessResponse<TenantUsageStats>>> {
    info!("Getting tenant usage for tenant {}", claims.tenant.tenant_id);
    
    let tenant_id = claims.tenant.tenant_id;
    
    let stats = state.db.get_tenant_usage_stats(tenant_id).await?;
    
    Ok(ResponseJson(SuccessResponse::new(stats)))
}

/// Get tenant members handler
/// 获取租户成员处理器
///
/// GET /api/v1/tenant/members
async fn get_tenant_members(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::auth::TokenClaims>,
) -> Result<ResponseJson<SuccessResponse<Vec<TenantMember>>>> {
    info!("Getting tenant members for tenant {}", claims.tenant.tenant_id);
    
    let tenant_id = claims.tenant.tenant_id;
    
    let members = state.db.find_tenant_members(tenant_id).await?;
    
    Ok(ResponseJson(SuccessResponse::new(members)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_dto_serialization() {
        let tenant = TenantDTO {
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
        assert!(json.contains("\"name\":\"Test Tenant\""));
        assert!(json.contains("\"slug\":\"test-tenant\""));
    }
}
