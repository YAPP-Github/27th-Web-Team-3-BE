//! Discord webhook handler for AI automation pipeline
//!
//! Handles Discord interactions and commands:
//! - Ping/Pong verification
//! - AI commands (@AI 분석해줘, @AI 수정해줘, etc.)

use crate::domain::webhook::dto::{DiscordCommand, DiscordWebhookPayload, DiscordWebhookResponse};
use crate::event::{Event, Priority};
use crate::utils::AppError;
use axum::{body::Bytes, http::HeaderMap, Json};
use tracing::{debug, error, info, warn};

/// Discord interaction type constants
pub mod interaction_type {
    /// Ping interaction (used for endpoint verification)
    pub const PING: i32 = 1;
    /// Application command interaction
    pub const APPLICATION_COMMAND: i32 = 2;
    /// Message component interaction
    pub const MESSAGE_COMPONENT: i32 = 3;
}

/// Verify Discord signature using Ed25519
/// Note: Requires ed25519-dalek crate for full implementation
/// For MVP, we skip verification in development mode
fn verify_discord_signature(
    _public_key: &str,
    _signature: &str,
    _timestamp: &str,
    _body: &[u8],
) -> Result<(), AppError> {
    // For MVP, skip verification if DISCORD_SKIP_VERIFICATION=true
    if std::env::var("DISCORD_SKIP_VERIFICATION").unwrap_or_default() == "true" {
        warn!("Discord signature verification skipped (development mode)");
        return Ok(());
    }

    // TODO: Implement Ed25519 signature verification with ed25519-dalek
    // For now, reject all requests in production mode to prevent unauthorized access
    error!("Discord signature verification not implemented - rejecting request");
    Err(AppError::Unauthorized(
        "Discord signature verification not implemented. Set DISCORD_SKIP_VERIFICATION=true for development.".to_string()
    ))
}

/// Handle Discord webhook interactions
///
/// Endpoint: POST /api/webhooks/discord
///
/// This handler:
/// 1. Verifies the Discord signature (security)
/// 2. Responds to ping interactions (endpoint verification)
/// 3. Parses commands and creates events for the AI pipeline
pub async fn handle_discord_webhook(
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<DiscordWebhookResponse>, AppError> {
    // 1. Extract signature headers
    let signature = headers
        .get("X-Signature-Ed25519")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            warn!("Missing X-Signature-Ed25519 header");
            AppError::Unauthorized("Missing X-Signature-Ed25519 header".to_string())
        })?;

    let timestamp = headers
        .get("X-Signature-Timestamp")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            warn!("Missing X-Signature-Timestamp header");
            AppError::Unauthorized("Missing X-Signature-Timestamp header".to_string())
        })?;

    // 2. Get public key from environment
    let public_key = std::env::var("DISCORD_PUBLIC_KEY").map_err(|_| {
        error!("DISCORD_PUBLIC_KEY not configured");
        AppError::InternalError("DISCORD_PUBLIC_KEY not configured".to_string())
    })?;

    // 3. Verify signature
    verify_discord_signature(&public_key, signature, timestamp, &body)?;

    // 4. Parse payload
    let payload: DiscordWebhookPayload = serde_json::from_slice(&body).map_err(|e| {
        error!(error = %e, "Failed to parse Discord payload");
        AppError::BadRequest(format!("Invalid payload: {}", e))
    })?;

    debug!(
        interaction_type = payload.interaction_type,
        channel_id = %payload.channel_id,
        "Received Discord interaction"
    );

    // 5. Handle ping interaction (type 1)
    if payload.interaction_type == interaction_type::PING {
        info!("Responding to Discord ping");
        return Ok(Json(DiscordWebhookResponse::pong()));
    }

    // 6. Process message/command interactions
    if let Some(data) = &payload.data {
        let command = DiscordCommand::parse(&data.content);

        info!(
            command_type = command.command_type(),
            channel_id = %payload.channel_id,
            user = %data.author.username,
            "Processing Discord command"
        );

        // Create event for AI pipeline
        let event = create_discord_event(&payload, &command);

        // In a real implementation, we would push this to the event queue
        // For now, just log the event creation
        info!(
            event_id = %event.id,
            event_type = %event.event_type,
            priority = ?event.priority,
            "Created event from Discord command"
        );

        // Return acknowledgment response
        let response_message = match command {
            DiscordCommand::Analyze { .. } => "분석 요청을 접수했습니다. 처리 중...",
            DiscordCommand::Fix { .. } => "수정 요청을 접수했습니다. 처리 중...",
            DiscordCommand::Review { .. } => "리뷰 요청을 접수했습니다. 처리 중...",
            DiscordCommand::Status => "시스템 상태를 확인 중입니다...",
            DiscordCommand::Unknown { .. } => "알 수 없는 명령입니다. @AI 분석해줘, @AI 수정해줘, @AI 리뷰해줘, @AI 상태 중 하나를 사용해주세요.",
        };

        return Ok(Json(DiscordWebhookResponse::message(response_message)));
    }

    // No data in payload
    warn!("Discord interaction without data");
    Ok(Json(DiscordWebhookResponse::message(
        "요청을 처리할 수 없습니다.",
    )))
}

/// Create an event from Discord command
fn create_discord_event(payload: &DiscordWebhookPayload, command: &DiscordCommand) -> Event {
    let (event_type, data) = match command {
        DiscordCommand::Analyze { args } => (
            "discord.command.analyze",
            serde_json::json!({
                "command": "analyze",
                "args": args,
                "channel_id": payload.channel_id,
                "guild_id": payload.guild_id,
            }),
        ),
        DiscordCommand::Fix { args } => (
            "discord.command.fix",
            serde_json::json!({
                "command": "fix",
                "args": args,
                "channel_id": payload.channel_id,
                "guild_id": payload.guild_id,
            }),
        ),
        DiscordCommand::Review { args } => (
            "discord.command.review",
            serde_json::json!({
                "command": "review",
                "args": args,
                "channel_id": payload.channel_id,
                "guild_id": payload.guild_id,
            }),
        ),
        DiscordCommand::Status => (
            "discord.command.status",
            serde_json::json!({
                "command": "status",
                "channel_id": payload.channel_id,
                "guild_id": payload.guild_id,
            }),
        ),
        DiscordCommand::Unknown { content } => (
            "discord.command.unknown",
            serde_json::json!({
                "command": "unknown",
                "content": content,
                "channel_id": payload.channel_id,
                "guild_id": payload.guild_id,
            }),
        ),
    };

    let mut event = Event::new(event_type, "discord", Priority::P1, data);

    // Add user information to metadata
    if let Some(data) = &payload.data {
        event = event.with_user(&data.author.username);
    }

    event
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::webhook::dto::{DiscordAuthor, DiscordMessageData};

    fn create_test_payload(content: &str) -> DiscordWebhookPayload {
        DiscordWebhookPayload {
            interaction_type: 2,
            token: "test_token".to_string(),
            channel_id: "123456".to_string(),
            guild_id: Some("789012".to_string()),
            data: Some(DiscordMessageData {
                content: content.to_string(),
                author: DiscordAuthor {
                    id: "user123".to_string(),
                    username: "testuser".to_string(),
                },
                timestamp: "2025-01-31T14:00:00Z".to_string(),
            }),
        }
    }

    #[test]
    fn should_create_analyze_event_from_discord_command() {
        // Arrange
        let payload = create_test_payload("@AI 분석해줘 AI5001 에러");
        let command = DiscordCommand::Analyze {
            args: "AI5001 에러".to_string(),
        };

        // Act
        let event = create_discord_event(&payload, &command);

        // Assert
        assert_eq!(event.event_type, "discord.command.analyze");
        assert_eq!(event.source, "discord");
        assert_eq!(event.priority, Priority::P1);
        assert_eq!(event.metadata.user, Some("testuser".to_string()));
        assert_eq!(
            event.data.get("command").and_then(|v| v.as_str()),
            Some("analyze")
        );
    }

    #[test]
    fn should_create_fix_event_from_discord_command() {
        // Arrange
        let payload = create_test_payload("@AI 수정해줘 #123");
        let command = DiscordCommand::Fix {
            args: "#123".to_string(),
        };

        // Act
        let event = create_discord_event(&payload, &command);

        // Assert
        assert_eq!(event.event_type, "discord.command.fix");
        assert_eq!(
            event.data.get("command").and_then(|v| v.as_str()),
            Some("fix")
        );
    }

    #[test]
    fn should_create_status_event_from_discord_command() {
        // Arrange
        let payload = create_test_payload("@AI 상태");
        let command = DiscordCommand::Status;

        // Act
        let event = create_discord_event(&payload, &command);

        // Assert
        assert_eq!(event.event_type, "discord.command.status");
    }
}
