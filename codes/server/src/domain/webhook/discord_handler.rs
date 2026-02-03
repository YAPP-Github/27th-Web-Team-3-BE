//! Discord webhook handler for AI automation pipeline
//!
//! Handles Discord interactions and commands:
//! - Ping/Pong verification
//! - AI commands (@AI 분석해줘, @AI 수정해줘, etc.)

use crate::domain::webhook::dto::{DiscordCommand, DiscordWebhookPayload, DiscordWebhookResponse};
use crate::event::{Event, Priority};
use crate::utils::AppError;
use axum::{body::Bytes, http::HeaderMap, Json};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
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

/// Maximum allowed time difference for timestamp validation (5 minutes in seconds)
const MAX_TIMESTAMP_DIFF_SECS: i64 = 300;

/// Verify Discord signature using Ed25519
///
/// Discord uses Ed25519 signatures to verify request integrity.
/// Reference: https://discord.com/developers/docs/interactions/receiving-and-responding#security-and-authorization
///
/// # Arguments
/// * `public_key` - Discord application's public key (hex-encoded)
/// * `signature` - Request signature from X-Signature-Ed25519 header (hex-encoded)
/// * `timestamp` - Request timestamp from X-Signature-Timestamp header
/// * `body` - Raw request body bytes
///
/// # Returns
/// * `Ok(())` if signature is valid
/// * `Err(AppError)` if signature is invalid or verification fails
fn verify_discord_signature(
    public_key: &str,
    signature: &str,
    timestamp: &str,
    body: &[u8],
) -> Result<(), AppError> {
    // For development, skip verification if DISCORD_SKIP_VERIFICATION=true
    if std::env::var("DISCORD_SKIP_VERIFICATION").unwrap_or_default() == "true" {
        warn!("Discord signature verification skipped (development mode)");
        return Ok(());
    }

    // 1. Validate timestamp (must be within 5 minutes)
    validate_timestamp(timestamp)?;

    // 2. Parse public key (hex -> bytes -> VerifyingKey)
    let public_key_bytes = hex::decode(public_key).map_err(|e| {
        error!(error = %e, "Invalid public key hex format");
        AppError::InternalError("Invalid public key format".to_string())
    })?;

    let public_key_array: [u8; 32] = public_key_bytes.try_into().map_err(|_| {
        error!("Public key must be 32 bytes");
        AppError::InternalError("Invalid public key length".to_string())
    })?;

    let verifying_key = VerifyingKey::from_bytes(&public_key_array).map_err(|e| {
        error!(error = %e, "Failed to create verifying key");
        AppError::InternalError("Invalid public key".to_string())
    })?;

    // 3. Parse signature (hex -> bytes -> Signature)
    let signature_bytes = hex::decode(signature).map_err(|e| {
        warn!(error = %e, "Invalid signature hex format");
        AppError::Unauthorized("Invalid signature format".to_string())
    })?;

    let signature_array: [u8; 64] = signature_bytes.try_into().map_err(|_| {
        warn!("Signature must be 64 bytes");
        AppError::Unauthorized("Invalid signature length".to_string())
    })?;

    let ed25519_signature = Signature::from_bytes(&signature_array);

    // 4. Construct message: timestamp + body
    let mut message = timestamp.as_bytes().to_vec();
    message.extend_from_slice(body);

    // 5. Verify signature
    verifying_key
        .verify(&message, &ed25519_signature)
        .map_err(|e| {
            warn!(error = %e, "Signature verification failed");
            AppError::Unauthorized("Signature verification failed".to_string())
        })?;

    debug!("Discord signature verified successfully");
    Ok(())
}

/// Validate that the timestamp is within acceptable range (5 minutes)
///
/// # Arguments
/// * `timestamp` - Unix timestamp as string
///
/// # Returns
/// * `Ok(())` if timestamp is valid
/// * `Err(AppError::Unauthorized)` if timestamp is too old or invalid
fn validate_timestamp(timestamp: &str) -> Result<(), AppError> {
    let request_timestamp: i64 = timestamp.parse().map_err(|e| {
        warn!(error = %e, timestamp = %timestamp, "Invalid timestamp format");
        AppError::Unauthorized("Invalid timestamp format".to_string())
    })?;

    let now = chrono::Utc::now().timestamp();
    let diff = (now - request_timestamp).abs();

    if diff > MAX_TIMESTAMP_DIFF_SECS {
        warn!(
            request_timestamp = request_timestamp,
            current_timestamp = now,
            diff_seconds = diff,
            "Timestamp too old or too far in the future"
        );
        return Err(AppError::Unauthorized(
            "Request timestamp is too old or invalid".to_string(),
        ));
    }

    Ok(())
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
    // 1. Check if we should skip verification BEFORE requiring headers
    let skip_verification =
        std::env::var("DISCORD_SKIP_VERIFICATION").unwrap_or_default() == "true";

    if !skip_verification {
        // Extract and verify signature only when not skipping
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

        let public_key = std::env::var("DISCORD_PUBLIC_KEY").map_err(|_| {
            error!("DISCORD_PUBLIC_KEY not configured");
            AppError::InternalError("DISCORD_PUBLIC_KEY not configured".to_string())
        })?;

        verify_discord_signature(&public_key, signature, timestamp, &body)?;
    } else {
        warn!("Discord signature verification skipped (development mode)");
    }

    // 2. Parse payload
    let payload: DiscordWebhookPayload = serde_json::from_slice(&body).map_err(|e| {
        error!(error = %e, "Failed to parse Discord payload");
        AppError::BadRequest(format!("Invalid payload: {}", e))
    })?;

    debug!(
        interaction_type = payload.interaction_type,
        channel_id = %payload.channel_id_or_unknown(),
        "Received Discord interaction"
    );

    // 3. Handle ping interaction (type 1) - no data required
    if payload.interaction_type == interaction_type::PING {
        info!("Responding to Discord ping");
        return Ok(Json(DiscordWebhookResponse::pong()));
    }

    // 4. Process application command interactions
    if let Some(data) = &payload.data {
        let content = data.get_content_for_parsing();
        let command = DiscordCommand::parse(&content);
        let username = payload.get_username().unwrap_or("unknown");

        info!(
            command_type = command.command_type(),
            channel_id = %payload.channel_id_or_unknown(),
            user = %username,
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

    // No data in payload - may be valid for some interaction types
    warn!("Discord interaction without data");
    Ok(Json(DiscordWebhookResponse::message(
        "요청을 처리할 수 없습니다.",
    )))
}

/// Create an event from Discord command
fn create_discord_event(payload: &DiscordWebhookPayload, command: &DiscordCommand) -> Event {
    let channel_id = payload.channel_id.as_deref();
    let guild_id = payload.guild_id.as_deref();

    let (event_type, data) = match command {
        DiscordCommand::Analyze { args } => (
            "discord.command.analyze",
            serde_json::json!({
                "command": "analyze",
                "args": args,
                "channel_id": channel_id,
                "guild_id": guild_id,
            }),
        ),
        DiscordCommand::Fix { args } => (
            "discord.command.fix",
            serde_json::json!({
                "command": "fix",
                "args": args,
                "channel_id": channel_id,
                "guild_id": guild_id,
            }),
        ),
        DiscordCommand::Review { args } => (
            "discord.command.review",
            serde_json::json!({
                "command": "review",
                "args": args,
                "channel_id": channel_id,
                "guild_id": guild_id,
            }),
        ),
        DiscordCommand::Status => (
            "discord.command.status",
            serde_json::json!({
                "command": "status",
                "channel_id": channel_id,
                "guild_id": guild_id,
            }),
        ),
        DiscordCommand::Unknown { content } => (
            "discord.command.unknown",
            serde_json::json!({
                "command": "unknown",
                "content": content,
                "channel_id": channel_id,
                "guild_id": guild_id,
            }),
        ),
    };

    let mut event = Event::new(event_type, "discord", Priority::P1, data);

    // Add user information to metadata
    if let Some(username) = payload.get_username() {
        event = event.with_user(username);
    }

    event
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::webhook::dto::{DiscordInteractionData, DiscordMember, DiscordUser};
    use ed25519_dalek::SigningKey;

    fn create_test_payload(content: &str) -> DiscordWebhookPayload {
        DiscordWebhookPayload {
            interaction_type: 2,
            token: "test_token".to_string(),
            channel_id: Some("123456".to_string()),
            guild_id: Some("789012".to_string()),
            data: Some(DiscordInteractionData {
                name: None,
                options: vec![],
                custom_id: None,
                content: Some(content.to_string()),
            }),
            member: Some(DiscordMember {
                user: Some(DiscordUser {
                    id: "user123".to_string(),
                    username: "testuser".to_string(),
                }),
                nick: None,
            }),
            user: None,
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

    // ============================================
    // Ed25519 Signature Verification Tests
    // ============================================

    /// Helper function to create a valid Ed25519 signature for testing
    fn create_test_signature(timestamp: &str, body: &[u8]) -> (String, String) {
        use ed25519_dalek::Signer;

        // Generate a random signing key for testing
        let signing_key = SigningKey::generate(&mut rand::thread_rng());
        let verifying_key = signing_key.verifying_key();

        // Create message: timestamp + body
        let mut message = timestamp.as_bytes().to_vec();
        message.extend_from_slice(body);

        // Sign the message
        let signature = signing_key.sign(&message);

        // Return hex-encoded public key and signature
        let public_key_hex = hex::encode(verifying_key.as_bytes());
        let signature_hex = hex::encode(signature.to_bytes());

        (public_key_hex, signature_hex)
    }

    #[test]
    fn should_verify_valid_signature() {
        // Arrange
        let timestamp = chrono::Utc::now().timestamp().to_string();
        let body = b"test body content";
        let (public_key, signature) = create_test_signature(&timestamp, body);

        // Act
        let result = verify_discord_signature(&public_key, &signature, &timestamp, body);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn should_reject_invalid_signature() {
        // Arrange
        let timestamp = chrono::Utc::now().timestamp().to_string();
        let body = b"test body content";
        let (public_key, _) = create_test_signature(&timestamp, body);

        // Create a different signature (for different body)
        let (_, wrong_signature) = create_test_signature(&timestamp, b"different body");

        // Act
        let result = verify_discord_signature(&public_key, &wrong_signature, &timestamp, body);

        // Assert
        assert!(result.is_err());
        if let Err(AppError::Unauthorized(msg)) = result {
            assert!(msg.contains("Signature verification failed"));
        } else {
            panic!("Expected Unauthorized error");
        }
    }

    #[test]
    fn should_reject_tampered_body() {
        // Arrange
        let timestamp = chrono::Utc::now().timestamp().to_string();
        let original_body = b"original body";
        let tampered_body = b"tampered body";
        let (public_key, signature) = create_test_signature(&timestamp, original_body);

        // Act - verify with tampered body
        let result = verify_discord_signature(&public_key, &signature, &timestamp, tampered_body);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn should_reject_tampered_timestamp() {
        // Arrange
        let original_timestamp = chrono::Utc::now().timestamp().to_string();
        let tampered_timestamp = (chrono::Utc::now().timestamp() + 1).to_string();
        let body = b"test body";
        let (public_key, signature) = create_test_signature(&original_timestamp, body);

        // Act - verify with tampered timestamp
        let result = verify_discord_signature(&public_key, &signature, &tampered_timestamp, body);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn should_reject_expired_timestamp() {
        // Arrange - timestamp from 10 minutes ago
        let old_timestamp = (chrono::Utc::now().timestamp() - 600).to_string();
        let body = b"test body";
        let (public_key, signature) = create_test_signature(&old_timestamp, body);

        // Act
        let result = verify_discord_signature(&public_key, &signature, &old_timestamp, body);

        // Assert
        assert!(result.is_err());
        if let Err(AppError::Unauthorized(msg)) = result {
            assert!(msg.contains("timestamp"));
        } else {
            panic!("Expected Unauthorized error about timestamp");
        }
    }

    #[test]
    fn should_reject_future_timestamp() {
        // Arrange - timestamp 10 minutes in the future
        let future_timestamp = (chrono::Utc::now().timestamp() + 600).to_string();
        let body = b"test body";
        let (public_key, signature) = create_test_signature(&future_timestamp, body);

        // Act
        let result = verify_discord_signature(&public_key, &signature, &future_timestamp, body);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn should_accept_timestamp_within_5_minutes() {
        // Arrange - timestamp from 4 minutes ago (within 5 minute window)
        let recent_timestamp = (chrono::Utc::now().timestamp() - 240).to_string();
        let body = b"test body";
        let (public_key, signature) = create_test_signature(&recent_timestamp, body);

        // Act
        let result = verify_discord_signature(&public_key, &signature, &recent_timestamp, body);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn should_reject_invalid_public_key_hex() {
        // Arrange
        let timestamp = chrono::Utc::now().timestamp().to_string();
        let body = b"test body";
        let invalid_public_key = "not_valid_hex!@#$";
        let (_, signature) = create_test_signature(&timestamp, body);

        // Act
        let result = verify_discord_signature(invalid_public_key, &signature, &timestamp, body);

        // Assert
        assert!(result.is_err());
        if let Err(AppError::InternalError(msg)) = result {
            assert!(msg.contains("public key"));
        } else {
            panic!("Expected InternalError about public key");
        }
    }

    #[test]
    fn should_reject_invalid_signature_hex() {
        // Arrange
        let timestamp = chrono::Utc::now().timestamp().to_string();
        let body = b"test body";
        let (public_key, _) = create_test_signature(&timestamp, body);
        let invalid_signature = "not_valid_hex!@#$";

        // Act
        let result = verify_discord_signature(&public_key, invalid_signature, &timestamp, body);

        // Assert
        assert!(result.is_err());
        if let Err(AppError::Unauthorized(msg)) = result {
            assert!(msg.contains("signature"));
        } else {
            panic!("Expected Unauthorized error about signature");
        }
    }

    #[test]
    fn should_reject_invalid_timestamp_format() {
        // Arrange
        let invalid_timestamp = "not_a_number";
        let body = b"test body";
        let (public_key, signature) = create_test_signature("12345", body);

        // Act
        let result = verify_discord_signature(&public_key, &signature, invalid_timestamp, body);

        // Assert
        assert!(result.is_err());
        if let Err(AppError::Unauthorized(msg)) = result {
            assert!(msg.contains("timestamp"));
        } else {
            panic!("Expected Unauthorized error about timestamp");
        }
    }

    #[test]
    fn should_reject_wrong_length_public_key() {
        // Arrange
        let timestamp = chrono::Utc::now().timestamp().to_string();
        let body = b"test body";
        let short_public_key = hex::encode([0u8; 16]); // 16 bytes instead of 32
        let (_, signature) = create_test_signature(&timestamp, body);

        // Act
        let result = verify_discord_signature(&short_public_key, &signature, &timestamp, body);

        // Assert
        assert!(result.is_err());
        if let Err(AppError::InternalError(msg)) = result {
            assert!(msg.contains("public key"));
        } else {
            panic!("Expected InternalError about public key");
        }
    }

    #[test]
    fn should_reject_wrong_length_signature() {
        // Arrange
        let timestamp = chrono::Utc::now().timestamp().to_string();
        let body = b"test body";
        let (public_key, _) = create_test_signature(&timestamp, body);
        let short_signature = hex::encode([0u8; 32]); // 32 bytes instead of 64

        // Act
        let result = verify_discord_signature(&public_key, &short_signature, &timestamp, body);

        // Assert
        assert!(result.is_err());
        if let Err(AppError::Unauthorized(msg)) = result {
            assert!(msg.contains("signature"));
        } else {
            panic!("Expected Unauthorized error about signature");
        }
    }

    // ============================================
    // Timestamp Validation Tests
    // ============================================

    #[test]
    fn should_validate_current_timestamp() {
        // Arrange
        let timestamp = chrono::Utc::now().timestamp().to_string();

        // Act
        let result = validate_timestamp(&timestamp);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn should_reject_timestamp_older_than_5_minutes() {
        // Arrange - 6 minutes ago
        let old_timestamp = (chrono::Utc::now().timestamp() - 360).to_string();

        // Act
        let result = validate_timestamp(&old_timestamp);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn should_reject_timestamp_more_than_5_minutes_in_future() {
        // Arrange - 6 minutes in the future
        let future_timestamp = (chrono::Utc::now().timestamp() + 360).to_string();

        // Act
        let result = validate_timestamp(&future_timestamp);

        // Assert
        assert!(result.is_err());
    }
}
