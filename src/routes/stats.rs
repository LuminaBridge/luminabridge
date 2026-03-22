//! Statistics routes for LuminaBridge API
//!
//! Handles real-time and historical statistics.
//! 处理实时和历史统计。

use axum::{
    routing::get,
    Router,
    extract::{State, Query},
    response::Json as ResponseJson,
    Extension,
};
use serde::{Deserialize, Serialize};
use tracing::info;
use chrono::{DateTime, Utc};

use crate::server::AppState;
use crate::error::{Error, Result};
use crate::types::{SuccessResponse, RealtimeStats};
use crate::auth::TokenClaims;

/// Create stats routes
/// 创建统计路由
pub fn stats_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/realtime", get(get_realtime_stats))
        .route("/usage", get(get_usage_stats))
        .route("/channels", get(get_channel_stats))
        .route("/models", get(get_model_stats))
        .route("/billing", get(get_billing_stats))
        .with_state(state)
}

/// Usage stats query parameters
/// 用量统计查询参数
#[derive(Debug, Deserialize)]
pub struct UsageStatsParams {
    /// Start date (YYYY-MM-DD)
    /// 开始日期
    pub start: Option<String>,
    
    /// End date (YYYY-MM-DD)
    /// 结束日期
    pub end: Option<String>,
    
    /// Group by (day, hour, minute)
    /// 分组方式
    #[serde(default = "default_group_by")]
    pub group_by: String,
}

fn default_group_by() -> String { "day".to_string() }

/// Usage statistics entry
/// 用量统计条目
#[derive(Debug, Serialize)]
pub struct UsageStatEntry {
    /// Timestamp
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    
    /// Requests count
    /// 请求数
    pub requests: i64,
    
    /// Total tokens
    /// 总 token 数
    pub total_tokens: i64,
    
    /// Prompt tokens
    /// 输入 token 数
    pub prompt_tokens: i64,
    
    /// Completion tokens
    /// 输出 token 数
    pub completion_tokens: i64,
    
    /// Cost
    /// 费用
    pub cost: f64,
}

/// Channel statistics
/// 渠道统计
#[derive(Debug, Serialize)]
pub struct ChannelStats {
    /// Channel ID
    /// 渠道 ID
    pub channel_id: i64,
    
    /// Channel name
    /// 渠道名称
    pub channel_name: String,
    
    /// Requests count
    /// 请求数
    pub requests: i64,
    
    /// Success count
    /// 成功数
    pub success_count: i64,
    
    /// Error count
    /// 错误数
    pub error_count: i64,
    
    /// Average latency (ms)
    /// 平均延迟（毫秒）
    pub avg_latency_ms: f64,
    
    /// Total tokens
    /// 总 token 数
    pub total_tokens: i64,
    
    /// Cost
    /// 费用
    pub cost: f64,
}

/// Model statistics
/// 模型统计
#[derive(Debug, Serialize)]
pub struct ModelStats {
    /// Model name
    /// 模型名称
    pub model: String,
    
    /// Requests count
    /// 请求数
    pub requests: i64,
    
    /// Total tokens
    /// 总 token 数
    pub total_tokens: i64,
    
    /// Prompt tokens
    /// 输入 token 数
    pub prompt_tokens: i64,
    
    /// Completion tokens
    /// 输出 token 数
    pub completion_tokens: i64,
    
    /// Cost
    /// 费用
    pub cost: f64,
}

/// Billing statistics
/// 计费统计
#[derive(Debug, Serialize)]
pub struct BillingStats {
    /// Total cost
    /// 总费用
    pub total_cost: f64,
    
    /// Current period start
    /// 当前周期开始
    pub period_start: DateTime<Utc>,
    
    /// Current period end
    /// 当前周期结束
    pub period_end: DateTime<Utc>,
    
    /// Previous period cost
    /// 上一周期费用
    pub previous_period_cost: f64,
    
    /// Cost by category
    /// 按类别划分的费用
    pub cost_by_category: Vec<CategoryCost>,
}

#[derive(Debug, Serialize)]
pub struct CategoryCost {
    /// Category name
    /// 类别名称
    pub category: String,
    
    /// Cost
    /// 费用
    pub cost: f64,
    
    /// Percentage
    /// 百分比
    pub percentage: f64,
}

/// Get realtime stats handler
/// 获取实时统计处理器
///
/// GET /api/v1/stats/realtime
async fn get_realtime_stats(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::auth::TokenClaims>,
) -> Result<ResponseJson<SuccessResponse<RealtimeStats>>> {
    info!("Getting realtime stats for tenant {}", claims.tenant.tenant_id);
    
    let tenant_id = claims.tenant.tenant_id;
    
    // Get realtime stats from Redis or calculate from recent data
    let stats = state.db.get_realtime_stats(tenant_id).await?;
    
    Ok(ResponseJson(SuccessResponse::new(stats)))
}

/// Get usage stats handler
/// 获取用量统计处理器
///
/// GET /api/v1/stats/usage
async fn get_usage_stats(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::auth::TokenClaims>,
    Query(params): Query<UsageStatsParams>,
) -> Result<ResponseJson<SuccessResponse<Vec<UsageStatEntry>>>> {
    info!("Getting usage stats for tenant {}", claims.tenant.tenant_id);
    
    let tenant_id = claims.tenant.tenant_id;
    
    let stats = state.db.get_usage_stats(
        tenant_id,
        params.start.as_deref(),
        params.end.as_deref(),
        &params.group_by,
    ).await?;
    
    Ok(ResponseJson(SuccessResponse::new(stats)))
}

/// Get channel stats handler
/// 获取渠道统计处理器
///
/// GET /api/v1/stats/channels
async fn get_channel_stats(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::auth::TokenClaims>,
    Query(params): Query<UsageStatsParams>,
) -> Result<ResponseJson<SuccessResponse<Vec<ChannelStats>>>> {
    info!("Getting channel stats for tenant {}", claims.tenant.tenant_id);
    
    let tenant_id = claims.tenant.tenant_id;
    
    let stats = state.db.get_channel_stats(
        tenant_id,
        params.start.as_deref(),
        params.end.as_deref(),
    ).await?;
    
    Ok(ResponseJson(SuccessResponse::new(stats)))
}

/// Get model stats handler
/// 获取模型统计处理器
///
/// GET /api/v1/stats/models
async fn get_model_stats(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::auth::TokenClaims>,
    Query(params): Query<UsageStatsParams>,
) -> Result<ResponseJson<SuccessResponse<Vec<ModelStats>>>> {
    info!("Getting model stats for tenant {}", claims.tenant.tenant_id);
    
    let tenant_id = claims.tenant.tenant_id;
    
    let stats = state.db.get_model_stats(
        tenant_id,
        params.start.as_deref(),
        params.end.as_deref(),
    ).await?;
    
    Ok(ResponseJson(SuccessResponse::new(stats)))
}

/// Get billing stats handler
/// 获取计费统计处理器
///
/// GET /api/v1/stats/billing
async fn get_billing_stats(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::auth::TokenClaims>,
) -> Result<ResponseJson<SuccessResponse<BillingStats>>> {
    info!("Getting billing stats for tenant {}", claims.tenant.tenant_id);
    
    let tenant_id = claims.tenant.tenant_id;
    
    let stats = state.db.get_billing_stats(tenant_id).await?;
    
    Ok(ResponseJson(SuccessResponse::new(stats)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_group_by() {
        assert_eq!(default_group_by(), "day");
    }
}
