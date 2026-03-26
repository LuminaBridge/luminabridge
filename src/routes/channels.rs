//! Channel management routes for LuminaBridge API
//!
//! Handles channel CRUD operations, testing, and batch operations.
//! 处理渠道 CRUD 操作、测试和批量操作。

use axum::{
    routing::{get, post, put, delete},
    Router,
    extract::{State, Json, Path, Query},
    response::Json as ResponseJson,
    Extension,
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use chrono::{DateTime, Utc};

use crate::server::AppState;
use crate::error::{Error, Result};
use crate::types::{SuccessResponse, ErrorResponse, ErrorCode, PaginationParams, ChannelStatus, ChannelType, CreateChannelRequest, UpdateChannelRequest};
use crate::auth::TokenClaims;
use crate::db;

/// Create channel routes
/// 创建渠道路由
pub fn channel_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_channels))
        .route("/", post(create_channel))
        .route("/:id", get(get_channel))
        .route("/:id", put(update_channel))
        .route("/:id", delete(delete_channel))
        .route("/:id/test", post(test_channel))
        .route("/:id/enable", post(enable_channel))
        .route("/:id/disable", post(disable_channel))
        .route("/batch", post(batch_operation))
}

/// Channel list query parameters
/// 渠道列表查询参数
#[derive(Debug, Deserialize)]
pub struct ChannelListParams {
    #[serde(flatten)]
    pub pagination: PaginationParams,
    
    /// Filter by group
    /// 按分组筛选
    pub group: Option<String>,
    
    /// Filter by status
    /// 按状态筛选
    pub status: Option<String>,
    
    /// Filter by channel type
    /// 按渠道类型筛选
    pub channel_type: Option<String>,
}

/// Channel DTO
/// 渠道数据传输对象
#[derive(Debug, Serialize, Clone)]
pub struct ChannelDTO {
    /// Channel ID
    /// 渠道 ID
    pub id: i64,
    
    /// Tenant ID
    /// 租户 ID
    pub tenant_id: i64,
    
    /// Channel name
    /// 渠道名称
    pub name: String,
    
    /// Channel type
    /// 渠道类型
    pub channel_type: String,
    
    /// Base URL
    /// 基础 URL
    pub base_url: Option<String>,
    
    /// Supported models
    /// 支持的模型
    pub models: Vec<String>,
    
    /// Weight for load balancing
    /// 负载均衡权重
    pub weight: i32,
    
    /// Channel status
    /// 渠道状态
    pub status: String,
    
    /// Priority
    /// 优先级
    pub priority: i32,
    
    /// Timeout in milliseconds
    /// 超时时间（毫秒）
    pub timeout_ms: i32,
    
    /// Retry count
    /// 重试次数
    pub retry_count: i32,
    
    /// Current balance
    /// 当前余额
    pub balance: Option<f64>,
    
    /// Last test timestamp
    /// 最后测试时间
    pub last_test_at: Option<DateTime<Utc>>,
    
    /// Last test status
    /// 最后测试状态
    pub last_test_status: Option<String>,
    
    /// Created at
    /// 创建时间
    pub created_at: DateTime<Utc>,
    
    /// Updated at
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// Test channel response
/// 测试渠道响应
#[derive(Debug, Serialize)]
pub struct TestChannelResponse {
    /// Test success
    /// 测试成功
    pub success: bool,
    
    /// Latency in milliseconds
    /// 延迟（毫秒）
    pub latency_ms: i64,
    
    /// Message
    /// 消息
    pub message: String,
}

/// Batch operation request
/// 批量操作请求
#[derive(Debug, Deserialize)]
pub struct BatchOperationRequest {
    /// Operation type: enable, disable, delete
    /// 操作类型：enable, disable, delete
    pub action: String,
    
    /// Channel IDs
    /// 渠道 ID 列表
    pub ids: Vec<i64>,
}

/// Batch operation response
/// 批量操作响应
#[derive(Debug, Serialize)]
pub struct BatchOperationResponse {
    /// Success count
    /// 成功数量
    pub success_count: i64,
    
    /// Failed count
    /// 失败数量
    pub failed_count: i64,
    
    /// Failed IDs with errors
    /// 失败的 ID 及错误
    pub failures: Vec<BatchFailure>,
}

#[derive(Debug, Serialize)]
pub struct BatchFailure {
    pub id: i64,
    pub error: String,
}

/// List channels handler
/// 列出渠道处理器
///
/// GET /api/v1/channels
async fn list_channels(
    State(state): State<AppState>,
    Extension(claims): Extension<TokenClaims>,
    Query(params): Query<ChannelListParams>,
) -> Result<ResponseJson<SuccessResponse<Vec<ChannelDTO>>>> {
    info!("Listing channels for tenant {}", claims.tenant.tenant_id);
    
    let tenant_id = claims.tenant.tenant_id;
    
    // Get channels from database
    let channels = state.db
        .find_channels_by_tenant(tenant_id, &params)
        .await?;
    
    let total = state.db.count_channels(tenant_id, &params).await?;
    
    let channel_dtos: Vec<ChannelDTO> = channels.into_iter().map(|c| ChannelDTO::from(c)).collect();
    
    Ok(ResponseJson(SuccessResponse::new(channel_dtos)
        .with_meta(crate::types::ResponseMeta::for_pagination(
            params.pagination.page,
            params.pagination.page_size,
            total,
        ))))
}

/// Create channel handler
/// 创建渠道处理器
///
/// POST /api/v1/channels
async fn create_channel(
    State(state): State<AppState>,
    Extension(claims): Extension<TokenClaims>,
    Json(payload): Json<CreateChannelRequest>,
) -> Result<ResponseJson<SuccessResponse<ChannelDTO>>> {
    info!("Creating channel '{}' for tenant {}", payload.name, claims.tenant.tenant_id);
    
    let tenant_id = claims.tenant.tenant_id;
    
    // Check if channel name already exists
    let exists = state.db.channel_exists(tenant_id, &payload.name).await?;
    if exists {
        return Err(Error::Validation("Channel with this name already exists".to_string()));
    }
    
    // Create channel
    let channel = state.db.create_channel(tenant_id, &payload).await?;
    
    Ok(ResponseJson(SuccessResponse::new(ChannelDTO::from(channel))
        .with_message("渠道创建成功")))
}

/// Get channel handler
/// 获取渠道处理器
///
/// GET /api/v1/channels/:id
async fn get_channel(
    State(state): State<AppState>,
    Extension(claims): Extension<TokenClaims>,
    Path(channel_id): Path<i64>,
) -> Result<ResponseJson<SuccessResponse<ChannelDTO>>> {
    info!("Getting channel {} for tenant {}", channel_id, claims.tenant.tenant_id);
    
    let channel = state.db.find_channel(channel_id).await?
        .ok_or_else(|| Error::Validation("Channel not found".to_string()))?;
    
    // Verify channel belongs to tenant
    if channel.tenant_id != claims.tenant.tenant_id {
        return Err(Error::Validation("Channel not found".to_string()));
    }
    
    Ok(ResponseJson(SuccessResponse::new(ChannelDTO::from(channel))))
}

/// Update channel handler
/// 更新渠道处理器
///
/// PUT /api/v1/channels/:id
async fn update_channel(
    State(state): State<AppState>,
    Extension(claims): Extension<TokenClaims>,
    Path(channel_id): Path<i64>,
    Json(payload): Json<UpdateChannelRequest>,
) -> Result<ResponseJson<SuccessResponse<ChannelDTO>>> {
    info!("Updating channel {} for tenant {}", channel_id, claims.tenant.tenant_id);
    
    let tenant_id = claims.tenant.tenant_id;
    
    // Check if channel exists
    let channel = state.db.find_channel(channel_id).await?
        .ok_or_else(|| Error::Validation("Channel not found".to_string()))?;
    
    // Verify channel belongs to tenant
    if channel.tenant_id != tenant_id {
        return Err(Error::Validation("Channel not found".to_string()));
    }
    
    // Update channel
    let updated = state.db.update_channel(channel_id, &payload).await?;
    
    Ok(ResponseJson(SuccessResponse::new(ChannelDTO::from(updated))
        .with_message("渠道更新成功")))
}

/// Delete channel handler
/// 删除渠道处理器
///
/// DELETE /api/v1/channels/:id
async fn delete_channel(
    State(state): State<AppState>,
    Extension(claims): Extension<TokenClaims>,
    Path(channel_id): Path<i64>,
) -> Result<ResponseJson<SuccessResponse<serde_json::Value>>> {
    info!("Deleting channel {} for tenant {}", channel_id, claims.tenant.tenant_id);
    
    let tenant_id = claims.tenant.tenant_id;
    
    // Check if channel exists
    let channel = state.db.find_channel(channel_id).await?
        .ok_or_else(|| Error::Validation("Channel not found".to_string()))?;
    
    // Verify channel belongs to tenant
    if channel.tenant_id != tenant_id {
        return Err(Error::Validation("Channel not found".to_string()));
    }
    
    // Delete channel
    state.db.delete_channel(channel_id).await?;
    
    Ok(ResponseJson(SuccessResponse::new(serde_json::json!({
        "deleted": true,
        "id": channel_id
    })).with_message("渠道删除成功")))
}

/// Test channel handler
/// 测试渠道处理器
///
/// POST /api/v1/channels/:id/test
async fn test_channel(
    State(state): State<AppState>,
    Extension(claims): Extension<TokenClaims>,
    Path(channel_id): Path<i64>,
) -> Result<ResponseJson<SuccessResponse<TestChannelResponse>>> {
    info!("Testing channel {} for tenant {}", channel_id, claims.tenant.tenant_id);
    
    let tenant_id = claims.tenant.tenant_id;
    
    // Get channel
    let channel = state.db.find_channel(channel_id).await?
        .ok_or_else(|| Error::Validation("Channel not found".to_string()))?;
    
    // Verify channel belongs to tenant
    if channel.tenant_id != tenant_id {
        return Err(Error::Validation("Channel not found".to_string()));
    }
    
    // Test channel connection
    let start = std::time::Instant::now();
    let result = test_channel_connection(&state, &channel).await;
    let latency = start.elapsed().as_millis() as i64;
    
    let response = match result {
        Ok(_) => TestChannelResponse {
            success: true,
            latency_ms: latency,
            message: "测试成功".to_string(),
        },
        Err(e) => TestChannelResponse {
            success: false,
            latency_ms: latency,
            message: format!("测试失败：{}", e),
        },
    };
    
    // Update last test info
    state.db.update_channel_test_info(channel_id, &response).await?;
    
    Ok(ResponseJson(SuccessResponse::new(response)))
}

/// Enable channel handler
/// 启用渠道处理器
///
/// POST /api/v1/channels/:id/enable
async fn enable_channel(
    State(state): State<AppState>,
    Extension(claims): Extension<TokenClaims>,
    Path(channel_id): Path<i64>,
) -> Result<ResponseJson<SuccessResponse<ChannelDTO>>> {
    info!("Enabling channel {} for tenant {}", channel_id, claims.tenant.tenant_id);
    
    let tenant_id = claims.tenant.tenant_id;
    
    let channel = state.db.find_channel(channel_id).await?
        .ok_or_else(|| Error::Validation("Channel not found".to_string()))?;
    
    if channel.tenant_id != tenant_id {
        return Err(Error::Validation("Channel not found".to_string()));
    }
    
    let updated = state.db.set_channel_status(channel_id, "active").await?;
    
    Ok(ResponseJson(SuccessResponse::new(ChannelDTO::from(updated))
        .with_message("渠道已启用")))
}

/// Disable channel handler
/// 禁用渠道处理器
///
/// POST /api/v1/channels/:id/disable
async fn disable_channel(
    State(state): State<AppState>,
    Extension(claims): Extension<TokenClaims>,
    Path(channel_id): Path<i64>,
) -> Result<ResponseJson<SuccessResponse<ChannelDTO>>> {
    info!("Disabling channel {} for tenant {}", channel_id, claims.tenant.tenant_id);
    
    let tenant_id = claims.tenant.tenant_id;
    
    let channel = state.db.find_channel(channel_id).await?
        .ok_or_else(|| Error::Validation("Channel not found".to_string()))?;
    
    if channel.tenant_id != tenant_id {
        return Err(Error::Validation("Channel not found".to_string()));
    }
    
    let updated = state.db.set_channel_status(channel_id, "disabled").await?;
    
    Ok(ResponseJson(SuccessResponse::new(ChannelDTO::from(updated))
        .with_message("渠道已禁用")))
}

/// Batch operation handler
/// 批量操作处理器
///
/// POST /api/v1/channels/batch
async fn batch_operation(
    State(state): State<AppState>,
    Extension(claims): Extension<TokenClaims>,
    Json(payload): Json<BatchOperationRequest>,
) -> Result<ResponseJson<SuccessResponse<BatchOperationResponse>>> {
    info!("Batch operation '{}' for tenant {}", payload.action, claims.tenant.tenant_id);
    
    let tenant_id = claims.tenant.tenant_id;
    let mut success_count = 0i64;
    let mut failed_count = 0i64;
    let mut failures = Vec::new();
    
    for channel_id in payload.ids {
        match payload.action.as_str() {
            "enable" => {
                match state.db.set_channel_status(channel_id, "active").await {
                    Ok(_) => success_count += 1,
                    Err(e) => {
                        failed_count += 1;
                        failures.push(BatchFailure {
                            id: channel_id,
                            error: e.to_string(),
                        });
                    }
                }
            }
            "disable" => {
                match state.db.set_channel_status(channel_id, "disabled").await {
                    Ok(_) => success_count += 1,
                    Err(e) => {
                        failed_count += 1;
                        failures.push(BatchFailure {
                            id: channel_id,
                            error: e.to_string(),
                        });
                    }
                }
            }
            "delete" => {
                // Verify ownership before delete
                if let Ok(Some(channel)) = state.db.find_channel(channel_id).await {
                    if channel.tenant_id == tenant_id {
                        match state.db.delete_channel(channel_id).await {
                            Ok(_) => success_count += 1,
                            Err(e) => {
                                failed_count += 1;
                                failures.push(BatchFailure {
                                    id: channel_id,
                                    error: e.to_string(),
                                });
                            }
                        }
                    } else {
                        failed_count += 1;
                        failures.push(BatchFailure {
                            id: channel_id,
                            error: "Channel not found".to_string(),
                        });
                    }
                } else {
                    failed_count += 1;
                    failures.push(BatchFailure {
                        id: channel_id,
                        error: "Channel not found".to_string(),
                    });
                }
            }
            _ => {
                return Err(Error::Validation(format!("Unknown action: {}", payload.action)));
            }
        }
    }
    
    Ok(ResponseJson(SuccessResponse::new(BatchOperationResponse {
        success_count,
        failed_count,
        failures,
    }).with_message(format!("批量操作完成：成功 {}, 失败 {}", success_count, failed_count))))
}

/// Test channel connection
/// 测试渠道连接
async fn test_channel_connection(
    _state: &AppState,
    _channel: &crate::db::Channel,
) -> Result<()> {
    // In production, this would actually test the channel by making a request
    // For now, just return Ok
    Ok(())
}

impl From<db::Channel> for ChannelDTO {
    fn from(channel: db::Channel) -> Self {
        let models: Vec<String> = serde_json::from_value(channel.models.clone())
            .unwrap_or_default();
        let balance: Option<f64> = channel.balance.map(|d| d.to_string().parse().unwrap_or(0.0));
        
        ChannelDTO {
            id: channel.id,
            tenant_id: channel.tenant_id,
            name: channel.name,
            channel_type: channel.channel_type,
            base_url: channel.base_url,
            models,
            weight: channel.weight,
            status: channel.status,
            priority: channel.priority,
            timeout_ms: channel.timeout_ms,
            retry_count: channel.retry_count,
            balance,
            last_test_at: channel.last_test_at,
            last_test_status: channel.last_test_status,
            created_at: channel.created_at,
            updated_at: channel.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_operation_request() {
        let json = r#"{
            "action": "enable",
            "ids": [1, 2, 3]
        }"#;
        
        let req: BatchOperationRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.action, "enable");
        assert_eq!(req.ids, vec![1, 2, 3]);
    }
}
