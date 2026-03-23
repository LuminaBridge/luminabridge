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
use sqlx::FromRow;
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
        .route("/dashboard", get(get_dashboard_stats))
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
#[derive(Debug, FromRow, Serialize)]
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

/// Usage trend entry (for request trend)
/// 用量趋势条目（用于请求趋势）
#[derive(Debug, FromRow, Serialize)]
pub struct UsageTrendEntry {
    /// Date (YYYY-MM-DD)
    /// 日期（YYYY-MM-DD）
    pub date: String,
    
    /// Requests count
    /// 请求数
    pub requests: i64,
    
    /// Tokens count
    /// Token 数
    pub tokens: i64,
    
    /// Cost
    /// 成本
    pub cost: f64,
}

/// Channel statistics
/// 渠道统计
#[derive(Debug, FromRow, Serialize)]
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
#[derive(Debug, FromRow, Serialize)]
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

/// Dashboard statistics
/// 仪表盘统计
#[derive(Debug, Serialize)]
pub struct DashboardStats {
    /// Total requests
    /// 总请求数
    pub total_requests: i64,
    
    /// Total tokens
    /// 总 token 数
    pub total_tokens: i64,
    
    /// Active channels count
    /// 活跃渠道数
    pub active_channels: i64,
    
    /// Today's revenue
    /// 今日收入
    pub today_revenue: f64,
    
    /// Request trend (last 7 days)
    /// 请求趋势（最近 7 天）
    pub request_trend: Vec<UsageStatEntry>,
    
    /// Channel status list
    /// 渠道状态列表
    pub channel_status: Vec<ChannelStatusItem>,
    
    /// Recent alerts
    /// 最近告警
    pub alerts: Vec<AlertItem>,
}

/// Channel status item
/// 渠道状态项
#[derive(Debug, Serialize)]
pub struct ChannelStatusItem {
    /// Channel ID
    /// 渠道 ID
    pub id: i64,
    
    /// Channel name
    /// 渠道名称
    pub name: String,
    
    /// Channel status
    /// 渠道状态
    pub status: String,
}

/// Alert item
/// 告警项
#[derive(Debug, Serialize)]
pub struct AlertItem {
    /// Alert ID
    /// 告警 ID
    pub id: i64,
    
    /// Alert level
    /// 告警级别
    pub level: String,
    
    /// Alert message
    /// 告警消息
    pub message: String,
    
    /// Created at
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// Get dashboard stats handler
/// 获取仪表盘统计处理器
///
/// GET /api/v1/stats/dashboard
pub async fn get_dashboard_stats(
    Extension(claims): Extension<crate::auth::TokenClaims>,
    State(state): State<AppState>,
) -> Result<ResponseJson<SuccessResponse<DashboardStats>>> {
    info!("Getting dashboard stats for tenant {}", claims.tenant.tenant_id);
    
    let tenant_id = claims.tenant.tenant_id;
    
    // Get total requests and tokens
    let total_requests = state.db.get_total_requests(tenant_id).await?;
    let total_tokens = state.db.get_total_tokens(tenant_id).await?;
    
    // Get active channels count
    let active_channels = state.db.get_active_channels_count(tenant_id).await?;
    
    // Get today's revenue
    let today_revenue = state.db.get_today_revenue(tenant_id).await?;
    
    // Get request trend (last 7 days)
    let request_trend = state.db.get_usage_stats(
        tenant_id,
        None,
        None,
        "day",
    ).await?;
    
    // Get channel status list
    let channels = state.db.list_channels(tenant_id).await?;
    let channel_status: Vec<ChannelStatusItem> = channels
        .into_iter()
        .map(|c| ChannelStatusItem {
            id: c.id,
            name: c.name,
            status: c.status,
        })
        .collect();
    
    // Generate alerts based on current metrics
    let _ = state.db.generate_channel_alerts(tenant_id).await;
    let _ = state.db.generate_token_alerts(tenant_id).await;
    
    // Get recent active alerts
    let db_alerts = state.db.get_active_alerts(tenant_id, 10).await?;
    let alerts: Vec<AlertItem> = db_alerts
        .into_iter()
        .map(|a| AlertItem {
            id: a.id,
            level: a.level,
            message: a.message,
            created_at: a.created_at,
        })
        .collect();
    
    Ok(ResponseJson(SuccessResponse::new(DashboardStats {
        total_requests,
        total_tokens,
        active_channels,
        today_revenue,
        request_trend,
        channel_status,
        alerts,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_group_by() {
        assert_eq!(default_group_by(), "day");
    }
}
