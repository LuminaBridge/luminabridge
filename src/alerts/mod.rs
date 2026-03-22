//! Alert notification module
//! 
//! This module provides alert notification functionality with support for multiple channels:
//! - Email notifications (SMTP)
//! - Webhook notifications (generic)
//! - Discord notifications

pub mod email;
pub mod webhook;
pub mod discord;

use crate::config::Config;
use crate::error::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Alert priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Alert types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    ChannelError,
    ChannelInactive,
    HighErrorRate,
    HighLatency,
    LowBalance,
    QuotaExceeded,
    SystemError,
    Custom(String),
}

/// Alert notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    pub email_enabled: bool,
    pub email_smtp_host: String,
    pub email_smtp_port: u16,
    pub email_username: String,
    pub email_password: String,
    pub email_recipients: Vec<String>,
    pub webhook_enabled: bool,
    pub webhook_url: String,
    pub discord_enabled: bool,
    pub discord_webhook_url: String,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            email_enabled: false,
            email_smtp_host: String::new(),
            email_smtp_port: 587,
            email_username: String::new(),
            email_password: String::new(),
            email_recipients: Vec::new(),
            webhook_enabled: false,
            webhook_url: String::new(),
            discord_enabled: false,
            discord_webhook_url: String::new(),
        }
    }
}

/// Alert record for history tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRecord {
    pub id: String,
    pub alert_type: AlertType,
    pub priority: AlertPriority,
    pub title: String,
    pub message: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub sent_channels: Vec<String>,
    pub status: AlertStatus,
}

/// Alert status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertStatus {
    Pending,
    Sent,
    Failed,
    Acknowledged,
}

/// Alert notification manager
pub struct AlertManager {
    config: AlertConfig,
    email_sender: Option<email::EmailSender>,
    history: Arc<RwLock<Vec<AlertRecord>>>,
}

impl AlertManager {
    /// Create a new AlertManager from configuration
    pub fn new(config: &Config) -> Result<Self> {
        let alert_config = AlertConfig {
            email_enabled: config.alerts.email_enabled,
            email_smtp_host: config.alerts.email_smtp_host.clone(),
            email_smtp_port: config.alerts.email_smtp_port,
            email_username: config.alerts.email_username.clone(),
            email_password: config.alerts.email_password.clone(),
            email_recipients: config.alerts.email_recipients.clone(),
            webhook_enabled: config.alerts.webhook_enabled,
            webhook_url: config.alerts.webhook_url.clone(),
            discord_enabled: config.alerts.discord_enabled,
            discord_webhook_url: config.alerts.discord_webhook_url.clone(),
        };

        let email_sender = if alert_config.email_enabled {
            Some(email::EmailSender::new(
                &alert_config.email_smtp_host,
                alert_config.email_smtp_port,
                &alert_config.email_username,
                &alert_config.email_password,
            ))
        } else {
            None
        };

        Ok(Self {
            config: alert_config,
            email_sender,
            history: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Send an alert notification through all configured channels
    pub async fn send_alert(
        &self,
        alert_type: AlertType,
        priority: AlertPriority,
        title: &str,
        message: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<AlertRecord> {
        let id = uuid::Uuid::new_v4().to_string();
        let mut sent_channels = Vec::new();
        let mut has_success = false;

        // Send via email
        if self.config.email_enabled {
            if let Some(ref sender) = self.email_sender {
                match sender.send(&self.config.email_recipients, title, message).await {
                    Ok(_) => {
                        sent_channels.push("email".to_string());
                        has_success = true;
                    }
                    Err(e) => {
                        log::error!("Failed to send email alert: {}", e);
                    }
                }
            }
        }

        // Send via webhook
        if self.config.webhook_enabled && !self.config.webhook_url.is_empty() {
            match webhook::send_webhook(&self.config.webhook_url, title, message, priority).await {
                Ok(_) => {
                    sent_channels.push("webhook".to_string());
                    has_success = true;
                }
                Err(e) => {
                    log::error!("Failed to send webhook alert: {}", e);
                }
            }
        }

        // Send via Discord
        if self.config.discord_enabled && !self.config.discord_webhook_url.is_empty() {
            match discord::send_discord_message(
                &self.config.discord_webhook_url,
                title,
                message,
                priority,
            )
            .await
            {
                Ok(_) => {
                    sent_channels.push("discord".to_string());
                    has_success = true;
                }
                Err(e) => {
                    log::error!("Failed to send Discord alert: {}", e);
                }
            }
        }

        let status = if has_success {
            AlertStatus::Sent
        } else {
            AlertStatus::Failed
        };

        let record = AlertRecord {
            id,
            alert_type,
            priority,
            title: title.to_string(),
            message: message.to_string(),
            metadata: metadata.unwrap_or(serde_json::Value::Null),
            created_at: Utc::now(),
            sent_channels,
            status,
        };

        // Add to history
        {
            let mut history = self.history.write().await;
            history.push(record.clone());
            // Keep only last 1000 alerts
            if history.len() > 1000 {
                history.remove(0);
            }
        }

        Ok(record)
    }

    /// Get alert history
    pub async fn get_history(&self, limit: usize) -> Vec<AlertRecord> {
        let history = self.history.read().await;
        history.iter().rev().take(limit).cloned().collect()
    }

    /// Get alert statistics
    pub async fn get_stats(&self) -> AlertStats {
        let history = self.history.read().await;
        let total = history.len();
        let sent = history.iter().filter(|a| a.status == AlertStatus::Sent).count();
        let failed = history.iter().filter(|a| a.status == AlertStatus::Failed).count();
        let pending = history.iter().filter(|a| a.status == AlertStatus::Pending).count();

        AlertStats {
            total,
            sent,
            failed,
            pending,
        }
    }
}

/// Alert statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertStats {
    pub total: usize,
    pub sent: usize,
    pub failed: usize,
    pub pending: usize,
}

/// Helper function to create an alert from channel error
pub async fn create_channel_error_alert(
    manager: &AlertManager,
    channel_id: &str,
    channel_name: &str,
    error_message: &str,
) -> Result<AlertRecord> {
    manager
        .send_alert(
            AlertType::ChannelError,
            AlertPriority::High,
            &format!("Channel Error: {}", channel_name),
            &format!("Channel '{}' (ID: {}) encountered an error: {}", channel_name, channel_id, error_message),
            Some(serde_json::json!({
                "channel_id": channel_id,
                "channel_name": channel_name,
            })),
        )
        .await
}

/// Helper function to create an alert from high error rate
pub async fn create_high_error_rate_alert(
    manager: &AlertManager,
    channel_id: &str,
    channel_name: &str,
    error_rate: f64,
    threshold: f64,
) -> Result<AlertRecord> {
    manager
        .send_alert(
            AlertType::HighErrorRate,
            AlertPriority::High,
            &format!("High Error Rate: {}", channel_name),
            &format!(
                "Channel '{}' has an error rate of {:.2}%, exceeding the threshold of {:.2}%",
                channel_name,
                error_rate * 100.0,
                threshold * 100.0
            ),
            Some(serde_json::json!({
                "channel_id": channel_id,
                "channel_name": channel_name,
                "error_rate": error_rate,
                "threshold": threshold,
            })),
        )
        .await
}

/// Helper function to create an alert from quota exceeded
pub async fn create_quota_exceeded_alert(
    manager: &AlertManager,
    token_id: &str,
    token_name: &str,
    usage_percent: f64,
) -> Result<AlertRecord> {
    manager
        .send_alert(
            AlertType::QuotaExceeded,
            AlertPriority::Critical,
            &format!("Quota Exceeded: {}", token_name),
            &format!(
                "Token '{}' has used {:.2}% of its quota",
                token_name, usage_percent
            ),
            Some(serde_json::json!({
                "token_id": token_id,
                "token_name": token_name,
                "usage_percent": usage_percent,
            })),
        )
        .await
}
