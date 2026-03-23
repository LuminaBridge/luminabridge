//! Webhook notification module
//! 
//! Sends alert notifications via HTTP webhooks

use crate::alerts::{AlertPriority, AlertType};
use crate::error::{Result, Error};
use chrono::Utc;
use reqwest::Client;
use serde::Serialize;

/// Webhook payload structure
#[derive(Debug, Serialize)]
pub struct WebhookPayload {
    pub alert_type: String,
    pub priority: String,
    pub title: String,
    pub message: String,
    pub timestamp: String,
    pub source: String,
}

/// Send a webhook notification
pub async fn send_webhook(
    url: &str,
    title: &str,
    message: &str,
    priority: AlertPriority,
) -> Result<()> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| Error::Internal(format!("Failed to create HTTP client: {}", e)))?;

    let payload = WebhookPayload {
        alert_type: "alert".to_string(),
        priority: format!("{:?}", priority),
        title: title.to_string(),
        message: message.to_string(),
        timestamp: Utc::now().to_rfc3339(),
        source: "LuminaBridge".to_string(),
    };

    let response = client
        .post(url)
        .json(&payload)
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| Error::Internal(format!("Failed to send webhook: {}", e)))?;

    if response.status().is_success() {
        tracing::info!("Webhook alert sent successfully to {}", url);
        Ok(())
    } else {
        Err(Error::Internal(format!(
            "Webhook returned non-success status: {}",
            response.status()
        )))
    }
}

/// Send a Slack-compatible webhook notification
pub async fn send_slack_webhook(
    url: &str,
    title: &str,
    message: &str,
    priority: AlertPriority,
) -> Result<()> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| Error::Internal(format!("Failed to create HTTP client: {}", e)))?;

    let color = match priority {
        AlertPriority::Low => "#36a64f",
        AlertPriority::Medium => "#ff9800",
        AlertPriority::High => "#ff5722",
        AlertPriority::Critical => "#f44336",
    };

    let payload = serde_json::json!({
        "attachments": [
            {
                "color": color,
                "title": title,
                "text": message,
                "footer": "LuminaBridge Alerts",
                "ts": Utc::now().timestamp(),
            }
        ]
    });

    let response = client
        .post(url)
        .json(&payload)
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| Error::Internal(format!("Failed to send Slack webhook: {}", e)))?;

    if response.status().is_success() {
        tracing::info!("Slack webhook alert sent successfully");
        Ok(())
    } else {
        Err(Error::Internal(format!(
            "Slack webhook returned non-success status: {}",
            response.status()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_payload_serialization() {
        let payload = WebhookPayload {
            alert_type: "test".to_string(),
            priority: "High".to_string(),
            title: "Test Alert".to_string(),
            message: "Test message".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            source: "Test".to_string(),
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("Test Alert"));
        assert!(json.contains("High"));
    }
}
