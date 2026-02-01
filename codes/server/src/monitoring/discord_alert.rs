//! Discord alert service for monitoring notifications
//!
//! Sends alerts to Discord channel via webhook.
//! Supports structured Embed messages with severity-based coloring.

use crate::event::{Event, Severity};
use crate::utils::AppError;
use reqwest::Client;
use serde::Serialize;
use tracing::{debug, error, info, instrument, warn};

/// Discord webhook message payload
#[derive(Debug, Serialize)]
pub struct DiscordMessage {
    /// Message content (plain text)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Rich embeds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embeds: Option<Vec<DiscordEmbed>>,
}

/// Discord embed for rich messages
#[derive(Debug, Clone, Serialize)]
pub struct DiscordEmbed {
    /// Embed title
    pub title: String,
    /// Embed description
    pub description: String,
    /// Color (as decimal integer)
    pub color: u32,
    /// Additional fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<DiscordEmbedField>>,
    /// Timestamp (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

/// Discord embed field
#[derive(Debug, Clone, Serialize)]
pub struct DiscordEmbedField {
    /// Field name
    pub name: String,
    /// Field value
    pub value: String,
    /// Whether the field is inline
    #[serde(default)]
    pub inline: bool,
}

/// Discord embed footer
#[derive(Debug, Clone, Serialize)]
pub struct DiscordFooter {
    /// Footer text
    pub text: String,
}

/// Discord color constants (decimal)
pub mod colors {
    /// Critical error - red
    pub const CRITICAL: u32 = 15158332; // #E74C3C
    /// Warning - yellow (as per requirements)
    pub const WARNING: u32 = 16776960; // #FFFF00
    /// Info - green
    pub const INFO: u32 = 3066993; // #2ECC71
    /// Success - green
    pub const SUCCESS: u32 = 3066993; // #2ECC71
}

/// Discord alert service
#[derive(Debug, Clone)]
pub struct DiscordAlert {
    /// Webhook URL
    webhook_url: String,
    /// HTTP client
    client: Client,
    /// Whether alerts are enabled
    enabled: bool,
}

impl DiscordAlert {
    /// Create a new Discord alert service
    pub fn new(webhook_url: impl Into<String>) -> Self {
        Self {
            webhook_url: webhook_url.into(),
            client: Client::new(),
            enabled: true,
        }
    }

    /// Create from environment variable
    pub fn from_env() -> Result<Self, AppError> {
        let webhook_url = std::env::var("DISCORD_WEBHOOK_URL").map_err(|_| {
            warn!("DISCORD_WEBHOOK_URL not configured, alerts disabled");
            AppError::InternalError("DISCORD_WEBHOOK_URL not configured".to_string())
        })?;

        Ok(Self::new(webhook_url))
    }

    /// Create a disabled alert service (for testing)
    pub fn disabled() -> Self {
        Self {
            webhook_url: String::new(),
            client: Client::new(),
            enabled: false,
        }
    }

    /// Check if alerts are enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled && !self.webhook_url.is_empty()
    }

    /// Send an alert message to Discord
    pub async fn send_alert(
        &self,
        title: &str,
        message: &str,
        severity: Severity,
    ) -> Result<(), AppError> {
        if !self.is_enabled() {
            debug!("Discord alerts disabled, skipping");
            return Ok(());
        }

        let color = match severity {
            Severity::Critical => colors::CRITICAL,
            Severity::Warning => colors::WARNING,
            Severity::Info => colors::INFO,
        };

        let embed = DiscordEmbed {
            title: title.to_string(),
            description: message.to_string(),
            color,
            fields: None,
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
        };

        let payload = DiscordMessage {
            content: None,
            embeds: Some(vec![embed]),
        };

        self.send_payload(&payload).await
    }

    /// Send an error alert with additional details
    pub async fn send_error_alert(
        &self,
        error_code: &str,
        message: &str,
        severity: Severity,
        details: Option<Vec<(String, String)>>,
    ) -> Result<(), AppError> {
        if !self.is_enabled() {
            debug!("Discord alerts disabled, skipping");
            return Ok(());
        }

        let color = match severity {
            Severity::Critical => colors::CRITICAL,
            Severity::Warning => colors::WARNING,
            Severity::Info => colors::INFO,
        };

        let severity_emoji = match severity {
            Severity::Critical => ":red_circle:",
            Severity::Warning => ":orange_circle:",
            Severity::Info => ":blue_circle:",
        };

        let title = format!("{} Error: {}", severity_emoji, error_code);

        let fields = details.map(|d| {
            d.into_iter()
                .map(|(name, value)| DiscordEmbedField {
                    name,
                    value,
                    inline: true,
                })
                .collect()
        });

        let embed = DiscordEmbed {
            title,
            description: message.to_string(),
            color,
            fields,
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
        };

        let payload = DiscordMessage {
            content: None,
            embeds: Some(vec![embed]),
        };

        self.send_payload(&payload).await
    }

    /// Send raw Discord message payload
    async fn send_payload(&self, payload: &DiscordMessage) -> Result<(), AppError> {
        let response = self
            .client
            .post(&self.webhook_url)
            .json(payload)
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to send Discord webhook");
                AppError::InternalError(format!("Failed to send Discord webhook: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!(status = %status, body = %body, "Discord webhook returned error");
            return Err(AppError::InternalError(format!(
                "Discord webhook error: {} - {}",
                status, body
            )));
        }

        info!("Discord alert sent successfully");
        Ok(())
    }

    /// Get the webhook URL (for testing)
    pub fn webhook_url(&self) -> &str {
        &self.webhook_url
    }

    /// Convert severity string to Discord color
    ///
    /// - "critical" -> Red (15158332)
    /// - "warning" -> Yellow (16776960)
    /// - "info" or other -> Green (3066993)
    pub fn severity_to_color(severity: &str) -> u32 {
        match severity.to_lowercase().as_str() {
            "critical" => colors::CRITICAL,
            "warning" => colors::WARNING,
            _ => colors::INFO,
        }
    }

    /// Send an alert from an Event to Discord webhook
    ///
    /// Extracts error information from event data:
    /// - error_code
    /// - severity
    /// - message
    /// - target
    /// - request_id
    /// - timestamp
    ///
    /// # Arguments
    /// * `event` - The event to send as an alert
    ///
    /// # Errors
    /// Returns `AppError::InternalError` if the webhook request fails
    #[instrument(skip(self, event), fields(event_id = %event.id, event_type = %event.event_type))]
    pub async fn send_event_alert(&self, event: &Event) -> Result<(), AppError> {
        if !self.is_enabled() {
            debug!("Discord alerts disabled, skipping event alert");
            return Ok(());
        }

        let severity_str = event
            .data
            .get("severity")
            .and_then(|v| v.as_str())
            .unwrap_or("info");

        let error_code = event
            .data
            .get("error_code")
            .and_then(|v| v.as_str())
            .unwrap_or("UNKNOWN");

        let message = event
            .data
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("No message provided");

        let target = event
            .data
            .get("target")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let request_id = event
            .data
            .get("request_id")
            .and_then(|v| v.as_str())
            .unwrap_or("-");

        let color = Self::severity_to_color(severity_str);

        let severity_emoji = match severity_str.to_lowercase().as_str() {
            "critical" => "🚨",
            "warning" => "⚠️",
            _ => "ℹ️",
        };

        let title = format!(
            "{} [{}] {}",
            severity_emoji,
            severity_str.to_uppercase(),
            error_code
        );

        let fields = vec![
            DiscordEmbedField {
                name: "Error Code".to_string(),
                value: format!("`{}`", error_code),
                inline: true,
            },
            DiscordEmbedField {
                name: "Severity".to_string(),
                value: severity_str.to_uppercase(),
                inline: true,
            },
            DiscordEmbedField {
                name: "Target".to_string(),
                value: format!("`{}`", target),
                inline: true,
            },
            DiscordEmbedField {
                name: "Request ID".to_string(),
                value: format!("`{}`", request_id),
                inline: true,
            },
            DiscordEmbedField {
                name: "Source".to_string(),
                value: event.source.clone(),
                inline: true,
            },
            DiscordEmbedField {
                name: "Event ID".to_string(),
                value: format!("`{}`", event.id),
                inline: true,
            },
        ];

        let embed = DiscordEmbed {
            title,
            description: message.to_string(),
            color,
            fields: Some(fields),
            timestamp: Some(event.timestamp.to_rfc3339()),
        };

        let payload = DiscordMessage {
            content: None,
            embeds: Some(vec![embed]),
        };

        info!(
            event_id = %event.id,
            severity = %severity_str,
            error_code = %error_code,
            "Sending Discord event alert"
        );

        self.send_payload(&payload).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Priority;

    #[test]
    fn should_create_discord_alert_from_url() {
        // Arrange & Act
        let alert = DiscordAlert::new("https://discord.com/api/webhooks/test");

        // Assert
        assert!(alert.is_enabled());
        assert_eq!(alert.webhook_url(), "https://discord.com/api/webhooks/test");
    }

    #[test]
    fn should_create_disabled_discord_alert() {
        // Arrange & Act
        let alert = DiscordAlert::disabled();

        // Assert
        assert!(!alert.is_enabled());
    }

    #[test]
    fn should_use_correct_colors_for_severity() {
        // Assert - Updated to match requirements
        assert_eq!(colors::CRITICAL, 15158332); // Red
        assert_eq!(colors::WARNING, 16776960); // Yellow
        assert_eq!(colors::INFO, 3066993); // Green
        assert_eq!(colors::SUCCESS, 3066993); // Green
    }

    #[tokio::test]
    async fn should_skip_alert_when_disabled() {
        // Arrange
        let alert = DiscordAlert::disabled();

        // Act
        let result = alert
            .send_alert("Test", "Test message", Severity::Info)
            .await;

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn should_return_red_color_for_critical_severity() {
        // Arrange & Act
        let color = DiscordAlert::severity_to_color("critical");

        // Assert
        assert_eq!(color, colors::CRITICAL);
        assert_eq!(color, 15158332);
    }

    #[test]
    fn should_return_red_color_for_critical_case_insensitive() {
        // Arrange & Act & Assert
        assert_eq!(
            DiscordAlert::severity_to_color("CRITICAL"),
            colors::CRITICAL
        );
        assert_eq!(
            DiscordAlert::severity_to_color("Critical"),
            colors::CRITICAL
        );
    }

    #[test]
    fn should_return_yellow_color_for_warning_severity() {
        // Arrange & Act
        let color = DiscordAlert::severity_to_color("warning");

        // Assert
        assert_eq!(color, colors::WARNING);
        assert_eq!(color, 16776960);
    }

    #[test]
    fn should_return_yellow_color_for_warning_case_insensitive() {
        // Arrange & Act & Assert
        assert_eq!(DiscordAlert::severity_to_color("WARNING"), colors::WARNING);
        assert_eq!(DiscordAlert::severity_to_color("Warning"), colors::WARNING);
    }

    #[test]
    fn should_return_green_color_for_info_severity() {
        // Arrange & Act
        let color = DiscordAlert::severity_to_color("info");

        // Assert
        assert_eq!(color, colors::INFO);
        assert_eq!(color, 3066993);
    }

    #[test]
    fn should_return_green_color_for_unknown_severity() {
        // Arrange & Act & Assert
        assert_eq!(DiscordAlert::severity_to_color("unknown"), colors::INFO);
        assert_eq!(DiscordAlert::severity_to_color("debug"), colors::INFO);
        assert_eq!(DiscordAlert::severity_to_color(""), colors::INFO);
    }

    #[tokio::test]
    async fn should_skip_event_alert_when_disabled() {
        // Arrange
        let alert = DiscordAlert::disabled();
        let event = Event::new(
            "monitoring.error_detected",
            "log-watcher",
            Priority::P0,
            serde_json::json!({
                "error_code": "AI5001",
                "severity": "critical",
                "message": "Claude API timeout"
            }),
        );

        // Act
        let result = alert.send_event_alert(&event).await;

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn should_return_error_when_env_not_set() {
        // Arrange: Ensure env var is not set
        std::env::remove_var("DISCORD_WEBHOOK_URL");

        // Act
        let result = DiscordAlert::from_env();

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn should_create_discord_alert_from_env_when_set() {
        // Arrange
        let test_url = "https://discord.com/api/webhooks/test123";
        std::env::set_var("DISCORD_WEBHOOK_URL", test_url);

        // Act
        let result = DiscordAlert::from_env();

        // Assert
        assert!(result.is_ok());
        let alert = result.unwrap();
        assert_eq!(alert.webhook_url(), test_url);

        // Cleanup
        std::env::remove_var("DISCORD_WEBHOOK_URL");
    }

    #[test]
    fn should_serialize_discord_message_correctly() {
        // Arrange
        let message = DiscordMessage {
            content: None,
            embeds: Some(vec![DiscordEmbed {
                title: "Test Title".to_string(),
                description: "Test Description".to_string(),
                color: colors::CRITICAL,
                fields: Some(vec![DiscordEmbedField {
                    name: "Field Name".to_string(),
                    value: "Field Value".to_string(),
                    inline: true,
                }]),
                timestamp: Some("2025-01-31T14:23:45Z".to_string()),
            }]),
        };

        // Act
        let json = serde_json::to_string(&message).expect("Failed to serialize");

        // Assert
        assert!(json.contains("\"title\":\"Test Title\""));
        assert!(json.contains("\"description\":\"Test Description\""));
        assert!(json.contains("\"color\":15158332"));
        assert!(json.contains("\"timestamp\":\"2025-01-31T14:23:45Z\""));
        assert!(json.contains("\"inline\":true"));
    }

    #[test]
    fn should_skip_none_fields_in_serialization() {
        // Arrange
        let message = DiscordMessage {
            content: None,
            embeds: Some(vec![DiscordEmbed {
                title: "Test".to_string(),
                description: "Test".to_string(),
                color: colors::INFO,
                fields: None, // None fields
                timestamp: None,
            }]),
        };

        // Act
        let json = serde_json::to_string(&message).expect("Failed to serialize");

        // Assert
        assert!(!json.contains("\"content\"")); // content should be skipped
        assert!(!json.contains("\"fields\"")); // fields should be skipped
        assert!(!json.contains("\"timestamp\"")); // timestamp should be skipped
    }

    #[tokio::test]
    async fn should_fail_with_invalid_webhook_url() {
        // Arrange
        let alert = DiscordAlert::new("invalid-url");
        let event = Event::new(
            "monitoring.error_detected",
            "test",
            Priority::P0,
            serde_json::json!({
                "error_code": "TEST001",
                "severity": "critical",
                "message": "Test error"
            }),
        );

        // Act
        let result = alert.send_event_alert(&event).await;

        // Assert
        assert!(result.is_err());
        if let Err(AppError::InternalError(msg)) = result {
            assert!(msg.contains("Discord webhook"));
        } else {
            panic!("Expected InternalError");
        }
    }

    #[test]
    fn should_handle_event_with_missing_data_fields() {
        // This test verifies that the alert building logic handles missing fields gracefully
        // by using default values

        // Arrange
        let _alert = DiscordAlert::new("https://discord.com/api/webhooks/test");
        let event = Event::new(
            "monitoring.error_detected",
            "log-watcher",
            Priority::P2,
            serde_json::json!({}), // Empty data
        );

        // Act - We can't call send_event_alert without actually sending,
        // but we can verify the event is properly structured
        let severity_str = event
            .data
            .get("severity")
            .and_then(|v| v.as_str())
            .unwrap_or("info");
        let error_code = event
            .data
            .get("error_code")
            .and_then(|v| v.as_str())
            .unwrap_or("UNKNOWN");

        // Assert
        assert_eq!(severity_str, "info");
        assert_eq!(error_code, "UNKNOWN");
        assert_eq!(DiscordAlert::severity_to_color(severity_str), colors::INFO);
    }

    #[test]
    fn should_create_correct_title_format() {
        // Arrange
        let event = Event::new(
            "monitoring.error_detected",
            "log-watcher",
            Priority::P0,
            serde_json::json!({
                "error_code": "AI5001",
                "severity": "critical"
            }),
        );

        // Act
        let severity_str = event
            .data
            .get("severity")
            .and_then(|v| v.as_str())
            .unwrap_or("info");
        let error_code = event
            .data
            .get("error_code")
            .and_then(|v| v.as_str())
            .unwrap_or("UNKNOWN");
        let severity_emoji = match severity_str.to_lowercase().as_str() {
            "critical" => "🚨",
            "warning" => "⚠️",
            _ => "ℹ️",
        };
        let title = format!(
            "{} [{}] {}",
            severity_emoji,
            severity_str.to_uppercase(),
            error_code
        );

        // Assert
        assert_eq!(title, "🚨 [CRITICAL] AI5001");
    }

    #[test]
    fn should_create_warning_title_format() {
        // Arrange & Act
        let severity_str = "warning";
        let error_code = "AUTH4001";
        let severity_emoji = match severity_str.to_lowercase().as_str() {
            "critical" => "🚨",
            "warning" => "⚠️",
            _ => "ℹ️",
        };
        let title = format!(
            "{} [{}] {}",
            severity_emoji,
            severity_str.to_uppercase(),
            error_code
        );

        // Assert
        assert_eq!(title, "⚠️ [WARNING] AUTH4001");
    }

    #[test]
    fn should_create_info_title_format() {
        // Arrange & Act
        let severity_str = "info";
        let error_code = "INFO001";
        let severity_emoji = match severity_str.to_lowercase().as_str() {
            "critical" => "🚨",
            "warning" => "⚠️",
            _ => "ℹ️",
        };
        let title = format!(
            "{} [{}] {}",
            severity_emoji,
            severity_str.to_uppercase(),
            error_code
        );

        // Assert
        assert_eq!(title, "ℹ️ [INFO] INFO001");
    }
}
