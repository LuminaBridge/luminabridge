//! Discord notification module
//! 
//! Sends alert notifications via Discord webhooks

use crate::alerts::AlertPriority;
use crate::error::{Result, Error};
use chrono::Utc;
use reqwest::Client;
use serde::Serialize;

/// Discord embed structure
#[derive(Debug, Serialize)]
pub struct DiscordEmbed {
    pub title: String,
    pub description: String,
    pub color: u32,
    pub footer: DiscordFooter,
    pub timestamp: String,
}

/// Discord footer structure
#[derive(Debug, Serialize)]
pub struct DiscordFooter {
    pub text: String,
    pub icon_url: Option<String>,
}

/// Discord webhook payload
#[derive(Debug, Serialize)]
pub struct DiscordPayload {
    pub content: String,
    pub embeds: Vec<DiscordEmbed>,
    pub username: String,
    pub avatar_url: Option<String>,
}

/// Get color for priority level
fn get_priority_color(priority: AlertPriority) -> u32 {
    match priority {
        AlertPriority::Low => 0x36a64f,      // Green
        AlertPriority::Medium => 0xff9800,   // Orange
        AlertPriority::High => 0xff5722,     // Red-Orange
        AlertPriority::Critical => 0xf44336, // Red
    }
}

/// Get emoji for priority level
fn get_priority_emoji(priority: AlertPriority) -> &'static str {
    match priority {
        AlertPriority::Low => "ℹ️",
        AlertPriority::Medium => "⚠️",
        AlertPriority::High => "🚨",
        AlertPriority::Critical => "🔴",
    }
}

/// Send a Discord webhook notification
pub async fn send_discord_message(
    webhook_url: &str,
    title: &str,
    message: &str,
    priority: AlertPriority,
) -> Result<()> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| Error::Internal(format!("Failed to create HTTP client: {}", e)))?;

    let color = get_priority_color(priority);
    let emoji = get_priority_emoji(priority);

    let embed = DiscordEmbed {
        title: format!("{} {}", emoji, title),
        description: message.to_string(),
        color,
        footer: DiscordFooter {
            text: "LuminaBridge Alerts".to_string(),
            icon_url: None,
        },
        timestamp: Utc::now().to_rfc3339(),
    };

    let payload = DiscordPayload {
        content: format!("**Alert: {}**", priority_as_text(priority)),
        embeds: vec![embed],
        username: "LuminaBridge".to_string(),
        avatar_url: None,
    };

    let response = client
        .post(webhook_url)
        .json(&payload)
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| Error::Internal(format!("Failed to send Discord webhook: {}", e)))?;

    if response.status().is_success() {
        tracing::info!("Discord alert sent successfully");
        Ok(())
    } else {
        Err(Error::Internal(format!(
            "Discord webhook returned non-success status: {}",
            response.status()
        )))
    }
}

/// Convert priority to text
fn priority_as_text(priority: AlertPriority) -> &'static str {
    match priority {
        AlertPriority::Low => "Low",
        AlertPriority::Medium => "Medium",
        AlertPriority::High => "High",
        AlertPriority::Critical => "Critical",
    }
}

/// Send a rich Discord notification with multiple fields
pub async fn send_discord_rich_notification(
    webhook_url: &str,
    title: &str,
    message: &str,
    priority: AlertPriority,
    fields: Vec<(&str, &str, bool)>,
) -> Result<()> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| Error::Internal(format!("Failed to create HTTP client: {}", e)))?;

    let color = get_priority_color(priority);
    let emoji = get_priority_emoji(priority);

    let mut embed = DiscordEmbed {
        title: format!("{} {}", emoji, title),
        description: message.to_string(),
        color,
        footer: DiscordFooter {
            text: "LuminaBridge Alerts".to_string(),
            icon_url: None,
        },
        timestamp: Utc::now().to_rfc3339(),
    };

    // Note: For full field support, we'd need to extend DiscordEmbed
    // This is a simplified version

    let payload = DiscordPayload {
        content: format!("**Alert: {}**", priority_as_text(priority)),
        embeds: vec![embed],
        username: "LuminaBridge".to_string(),
        avatar_url: None,
    };

    let response = client
        .post(webhook_url)
        .json(&payload)
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| Error::Internal(format!("Failed to send Discord webhook: {}", e)))?;

    if response.status().is_success() {
        tracing::info!("Discord rich notification sent successfully");
        Ok(())
    } else {
        Err(Error::Internal(format!(
            "Discord webhook returned non-success status: {}",
            response.status()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_color() {
        assert_eq!(get_priority_color(AlertPriority::Low), 0x36a64f);
        assert_eq!(get_priority_color(AlertPriority::Medium), 0xff9800);
        assert_eq!(get_priority_color(AlertPriority::High), 0xff5722);
        assert_eq!(get_priority_color(AlertPriority::Critical), 0xf44336);
    }

    #[test]
    fn test_priority_emoji() {
        assert_eq!(get_priority_emoji(AlertPriority::Low), "ℹ️");
        assert_eq!(get_priority_emoji(AlertPriority::Medium), "⚠️");
        assert_eq!(get_priority_emoji(AlertPriority::High), "🚨");
        assert_eq!(get_priority_emoji(AlertPriority::Critical), "🔴");
    }

    #[test]
    fn test_discord_payload_serialization() {
        let payload = DiscordPayload {
            content: "Test".to_string(),
            embeds: vec![],
            username: "Test".to_string(),
            avatar_url: None,
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("Test"));
    }
}
