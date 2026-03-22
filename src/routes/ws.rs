//! WebSocket routes for LuminaBridge API
//!
//! Handles WebSocket connections for real-time data push.
//! 处理 WebSocket 连接以进行实时数据推送。

use axum::{
    extract::{State, ws::{WebSocketUpgrade, WebSocket, Message, CloseCode}},
    response::IntoResponse,
    Extension,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tracing::{info, warn, error};
use chrono::Utc;

use crate::server::AppState;
use crate::error::Result;
use crate::types::RealtimeStats;

/// Broadcast channel size for stats
/// 统计广播频道大小
const STATS_CHANNEL_SIZE: usize = 1000;

/// WebSocket message types
/// WebSocket 消息类型
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum WsMessage {
    /// Realtime statistics
    /// 实时统计
    #[serde(rename = "stats")]
    Stats { data: RealtimeStats },
    
    /// Channel status change
    /// 渠道状态变化
    #[serde(rename = "channel_status")]
    ChannelStatus { data: ChannelStatusMessage },
    
    /// Alert notification
    /// 告警通知
    #[serde(rename = "alert")]
    Alert { data: AlertMessage },
    
    /// Heartbeat ping
    /// 心跳 ping
    #[serde(rename = "ping")]
    Ping,
    
    /// Heartbeat pong
    /// 心跳 pong
    #[serde(rename = "pong")]
    Pong,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChannelStatusMessage {
    /// Channel ID
    /// 渠道 ID
    pub channel_id: i64,
    
    /// Status
    /// 状态
    pub status: String,
    
    /// Message
    /// 消息
    pub message: Option<String>,
    
    /// Timestamp
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AlertMessage {
    /// Alert level (info, warning, error, critical)
    /// 告警级别
    pub level: String,
    
    /// Alert message
    /// 告警消息
    pub message: String,
    
    /// Alert code
    /// 告警代码
    pub code: Option<String>,
    
    /// Timestamp
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Client message (received from WebSocket)
/// 客户端消息（从 WebSocket 接收）
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    /// Heartbeat ping
    #[serde(rename = "ping")]
    Ping,
    
    /// Subscribe to channel
    #[serde(rename = "subscribe")]
    Subscribe { channel: String },
    
    /// Unsubscribe from channel
    #[serde(rename = "unsubscribe")]
    Unsubscribe { channel: String },
}

/// WebSocket handler
/// WebSocket 处理器
///
/// GET /api/v1/ws
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Extension(claims): Extension<crate::auth::TokenClaims>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, claims, state))
}

/// Handle WebSocket connection
/// 处理 WebSocket 连接
async fn handle_socket(
    socket: WebSocket,
    claims: crate::auth::TokenClaims,
    state: AppState,
) {
    let (mut sender, mut receiver) = socket.split();
    let tenant_id = claims.tenant.tenant_id;
    let user_id = claims.user_id;
    
    info!("WebSocket connection established for user {} (tenant {})", user_id, tenant_id);
    
    // Subscribe to broadcast channel for stats
    let mut stats_rx = state.stats_sender.subscribe();
    
    // Connection metadata
    let connected_at = Utc::now();
    
    // Send task - push data to client
    let send_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
        
        loop {
            tokio::select! {
                // Send heartbeat every 5 seconds
                _ = interval.tick() => {
                    let ping_msg = WsMessage::Ping;
                    if let Ok(json) = serde_json::to_string(&ping_msg) {
                        if sender.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                }
                
                // Receive stats from broadcast channel
                Ok(stats) = stats_rx.recv() => {
                    // Only send stats for this tenant
                    // In production, filter by tenant_id in the stats struct
                    let msg = WsMessage::Stats { data: stats };
                    if let Ok(json) = serde_json::to_string(&msg) {
                        if sender.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    });
    
    // Receive task - handle client messages
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    // Handle client messages
                    if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                        match client_msg {
                            ClientMessage::Ping => {
                                // Respond with pong
                                let pong_msg = WsMessage::Pong;
                                if let Ok(json) = serde_json::to_string(&pong_msg) {
                                    let _ = sender.send(Message::Text(json)).await;
                                }
                            }
                            ClientMessage::Subscribe { channel } => {
                                info!("Client subscribed to channel: {}", channel);
                                // Handle subscription logic
                            }
                            ClientMessage::Unsubscribe { channel } => {
                                info!("Client unsubscribed from channel: {}", channel);
                                // Handle unsubscription logic
                            }
                        }
                    }
                }
                Message::Close(frame) => {
                    info!("WebSocket closing: {:?}", frame);
                    break;
                }
                Message::Pong(_) => {
                    // Heartbeat response received
                }
                _ => {}
            }
        }
    });
    
    // Wait for either task to complete
    tokio::select! {
        _ = send_task => {
            info!("Send task completed for user {}", user_id);
        }
        _ = recv_task => {
            info!("Receive task completed for user {}", user_id);
        }
    }
    
    info!("WebSocket connection closed for user {} (connected for {:?})", 
          user_id, Utc::now().signed_duration_since(connected_at));
}

/// Broadcast stats to all connected WebSocket clients
/// 向所有连接的 WebSocket 客户端广播统计
pub async fn broadcast_stats(state: &AppState, stats: RealtimeStats) {
    if let Err(e) = state.stats_sender.send(stats) {
        warn!("Failed to broadcast stats: {}", e);
    }
}

/// Send alert to specific tenant's WebSocket clients
/// 向特定租户的 WebSocket 客户端发送告警
pub async fn send_alert(
    state: &AppState,
    tenant_id: i64,
    level: &str,
    message: &str,
    code: Option<&str>,
) {
    let alert = AlertMessage {
        level: level.to_string(),
        message: message.to_string(),
        code: code.map(String::from),
        timestamp: Utc::now(),
    };
    
    let msg = WsMessage::Alert { data: alert };
    
    // In production, you would send this to a tenant-specific channel
    // For now, just log it
    info!("Alert for tenant {}: {:?}", tenant_id, msg);
}

/// Send channel status update to WebSocket clients
/// 向 WebSocket 客户端发送渠道状态更新
pub async fn send_channel_status(
    state: &AppState,
    channel_id: i64,
    status: &str,
    message: Option<&str>,
) {
    let channel_msg = ChannelStatusMessage {
        channel_id,
        status: status.to_string(),
        message: message.map(String::from),
        timestamp: Utc::now(),
    };
    
    let msg = WsMessage::ChannelStatus { data: channel_msg };
    
    info!("Channel {} status update: {:?}", channel_id, msg);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_message_serialization() {
        let stats = RealtimeStats {
            tps: 100,
            rpm: 6000,
            latency_ms: 150.5,
            error_rate: 0.001,
            active_channels: 5,
            timestamp: Utc::now(),
        };
        
        let msg = WsMessage::Stats { data: stats.clone() };
        let json = serde_json::to_string(&msg).unwrap();
        
        assert!(json.contains("\"type\":\"stats\""));
        assert!(json.contains("\"tps\":100"));
    }

    #[test]
    fn test_client_message_deserialization() {
        let json = r#"{"type": "ping"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, ClientMessage::Ping));
    }
}
