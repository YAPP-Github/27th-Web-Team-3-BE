//! Event structure and related types for AI automation pipeline
//!
//! TODO(MVP): 현재 dead_code 허용은 MVP 단계이므로 적용됨.
//!            Phase 3 완료 후 실제 사용되지 않는 코드 정리 필요.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

/// Event priority levels
/// Lower values indicate higher priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    /// P0: Critical - process immediately
    P0 = 0,
    /// P1: High - process within 5 minutes
    P1 = 1,
    /// P2: Medium - process within 15 minutes
    P2 = 2,
    /// P3: Low - process within 1 hour
    #[default]
    P3 = 3,
}

impl Priority {
    /// Determine priority from event type and data
    pub fn from_event_type(event_type: &str, data: &serde_json::Value) -> Self {
        match event_type {
            "monitoring.error_detected" => match data.get("severity").and_then(|s| s.as_str()) {
                Some("critical") => Priority::P0,
                Some("warning") => Priority::P1,
                _ => Priority::P2,
            },
            // Match discord.command and all sub-types (discord.command.analyze, etc.)
            t if t == "discord.command" || t.starts_with("discord.command.") => Priority::P1,
            "github.issue_labeled" | "github.issue_comment_created" => Priority::P1,
            "github.issue_opened" => Priority::P1,
            "github.pr_opened" | "github.pr_labeled" => Priority::P2,
            _ => Priority::P3,
        }
    }
}

/// Error severity levels for monitoring events
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Info: log only (no alert)
    #[default]
    Info = 0,
    /// Warning: alert only (no AI diagnosis)
    Warning = 1,
    /// Critical: alert + AI diagnosis + GitHub Issue
    Critical = 2,
}

impl FromStr for Severity {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "info" => Ok(Severity::Info),
            "warning" => Ok(Severity::Warning),
            "critical" => Ok(Severity::Critical),
            _ => Err("invalid severity value: expected 'info', 'warning', or 'critical'"),
        }
    }
}

/// Event processing status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum EventStatus {
    /// Waiting in queue
    #[default]
    Pending,
    /// Currently being processed
    Processing,
    /// Successfully completed
    Completed,
    /// Processing failed
    Failed,
    /// Queued for retry
    Retrying,
}

/// Event metadata for tracking and deduplication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventMetadata {
    /// Fingerprint for deduplication
    pub fingerprint: String,
    /// Correlation ID for related events
    pub correlation_id: Option<Uuid>,
    /// User information (username/login)
    pub user: Option<String>,
    /// Additional attributes
    #[serde(default)]
    pub attributes: HashMap<String, String>,
}

impl EventMetadata {
    /// Create new metadata with fingerprint
    pub fn new(fingerprint: impl Into<String>) -> Self {
        Self {
            fingerprint: fingerprint.into(),
            correlation_id: None,
            user: None,
            attributes: HashMap::new(),
        }
    }

    /// Set correlation ID
    pub fn with_correlation_id(mut self, id: Uuid) -> Self {
        self.correlation_id = Some(id);
        self
    }

    /// Set user
    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Add attribute
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }
}

impl Default for EventMetadata {
    fn default() -> Self {
        Self {
            fingerprint: Uuid::new_v4().to_string(),
            correlation_id: None,
            user: None,
            attributes: HashMap::new(),
        }
    }
}

/// Core event structure for AI automation pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    /// Unique event ID
    pub id: Uuid,
    /// Event type (e.g., "monitoring.error_detected", "discord.command", "github.issue_labeled")
    pub event_type: String,
    /// Event source (e.g., "log-watcher", "discord", "github")
    pub source: String,
    /// Event timestamp (UTC)
    pub timestamp: DateTime<Utc>,
    /// Priority level
    pub priority: Priority,
    /// Event data (source-specific)
    /// - monitoring: { error_code, severity, message, target, request_id, log_line }
    /// - discord: { command, args, channel_id }
    /// - github: { action, issue_number, labels, repository }
    pub data: serde_json::Value,
    /// Event metadata
    pub metadata: EventMetadata,
    /// Retry count
    pub retry_count: u32,
    /// Current status
    pub status: EventStatus,
}

impl Event {
    /// Create a new event
    pub fn new(
        event_type: impl Into<String>,
        source: impl Into<String>,
        priority: Priority,
        data: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type: event_type.into(),
            source: source.into(),
            timestamp: Utc::now(),
            priority,
            data,
            metadata: EventMetadata::default(),
            retry_count: 0,
            status: EventStatus::Pending,
        }
    }

    /// Create event with automatic priority detection
    pub fn with_auto_priority(
        event_type: impl Into<String>,
        source: impl Into<String>,
        data: serde_json::Value,
    ) -> Self {
        let event_type_str: String = event_type.into();
        let priority = Priority::from_event_type(&event_type_str, &data);
        Self::new(event_type_str, source, priority, data)
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: EventMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Set fingerprint for deduplication
    pub fn with_fingerprint(mut self, fingerprint: impl Into<String>) -> Self {
        self.metadata.fingerprint = fingerprint.into();
        self
    }

    /// Set user
    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.metadata.user = Some(user.into());
        self
    }

    /// Set correlation ID
    pub fn with_correlation_id(mut self, id: Uuid) -> Self {
        self.metadata.correlation_id = Some(id);
        self
    }

    /// Check if this event is a duplicate of another based on fingerprint
    pub fn is_duplicate_of(&self, other: &Event) -> bool {
        self.metadata.fingerprint == other.metadata.fingerprint
    }

    /// Generate filename for file-based queue
    pub fn to_filename(&self) -> String {
        format!("p{}_{}.json", self.priority as u8, self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_event_with_auto_priority_p0_for_critical_monitoring() {
        // Arrange
        let data = serde_json::json!({
            "error_code": "AI5001",
            "severity": "critical",
            "message": "Claude API timeout"
        });

        // Act
        let event = Event::with_auto_priority("monitoring.error_detected", "log-watcher", data);

        // Assert
        assert_eq!(event.priority, Priority::P0);
        assert_eq!(event.event_type, "monitoring.error_detected");
        assert_eq!(event.source, "log-watcher");
        assert_eq!(event.status, EventStatus::Pending);
    }

    #[test]
    fn should_create_event_with_auto_priority_p1_for_warning_monitoring() {
        // Arrange
        let data = serde_json::json!({
            "error_code": "AUTH4001",
            "severity": "warning",
            "message": "Authentication failed"
        });

        // Act
        let event = Event::with_auto_priority("monitoring.error_detected", "log-watcher", data);

        // Assert
        assert_eq!(event.priority, Priority::P1);
    }

    #[test]
    fn should_create_event_with_auto_priority_p1_for_discord_command() {
        // Arrange
        let data = serde_json::json!({
            "command": "analyze",
            "args": "AI5001"
        });

        // Act
        let event = Event::with_auto_priority("discord.command", "discord", data);

        // Assert
        assert_eq!(event.priority, Priority::P1);
    }

    #[test]
    fn should_create_event_with_auto_priority_p2_for_github_pr() {
        // Arrange
        let data = serde_json::json!({
            "action": "opened",
            "pr_number": 123
        });

        // Act
        let event = Event::with_auto_priority("github.pr_opened", "github", data);

        // Assert
        assert_eq!(event.priority, Priority::P2);
    }

    #[test]
    fn should_parse_severity_from_string() {
        // Arrange & Act & Assert
        assert_eq!(Severity::from_str("critical"), Ok(Severity::Critical));
        assert_eq!(Severity::from_str("CRITICAL"), Ok(Severity::Critical));
        assert_eq!(Severity::from_str("warning"), Ok(Severity::Warning));
        assert_eq!(Severity::from_str("info"), Ok(Severity::Info));

        // Error case should return descriptive message
        let err = Severity::from_str("unknown");
        assert!(err.is_err());
        assert_eq!(
            err.unwrap_err(),
            "invalid severity value: expected 'info', 'warning', or 'critical'"
        );
    }

    #[test]
    fn should_detect_duplicate_events_by_fingerprint() {
        // Arrange
        let fingerprint = "error_ai5001_2025-01-31";
        let event1 = Event::new(
            "monitoring.error_detected",
            "log-watcher",
            Priority::P0,
            serde_json::json!({}),
        )
        .with_fingerprint(fingerprint);

        let event2 = Event::new(
            "monitoring.error_detected",
            "log-watcher",
            Priority::P0,
            serde_json::json!({}),
        )
        .with_fingerprint(fingerprint);

        let event3 = Event::new(
            "monitoring.error_detected",
            "log-watcher",
            Priority::P0,
            serde_json::json!({}),
        )
        .with_fingerprint("different_fingerprint");

        // Act & Assert
        assert!(event1.is_duplicate_of(&event2));
        assert!(!event1.is_duplicate_of(&event3));
    }

    #[test]
    fn should_generate_correct_filename() {
        // Arrange
        let event = Event::new(
            "monitoring.error_detected",
            "log-watcher",
            Priority::P0,
            serde_json::json!({}),
        );

        // Act
        let filename = event.to_filename();

        // Assert
        assert!(filename.starts_with("p0_"));
        assert!(filename.ends_with(".json"));
        assert!(filename.contains(&event.id.to_string()));
    }

    #[test]
    fn should_serialize_event_to_camel_case_json() {
        // Arrange
        let event = Event::new(
            "test.event",
            "test",
            Priority::P1,
            serde_json::json!({"testField": "value"}),
        );

        // Act
        let json = serde_json::to_string(&event).expect("Failed to serialize");

        // Assert
        assert!(json.contains("eventType"));
        assert!(json.contains("retryCount"));
        assert!(!json.contains("event_type"));
        assert!(!json.contains("retry_count"));
    }

    #[test]
    fn should_deserialize_event_from_camel_case_json() {
        // Arrange
        let json = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "eventType": "test.event",
            "source": "test",
            "timestamp": "2025-01-31T14:23:45Z",
            "priority": "p1",
            "data": {"testField": "value"},
            "metadata": {
                "fingerprint": "test_fp",
                "correlationId": null,
                "user": "testuser",
                "attributes": {}
            },
            "retryCount": 0,
            "status": "pending"
        }"#;

        // Act
        let event: Event = serde_json::from_str(json).expect("Failed to deserialize");

        // Assert
        assert_eq!(event.event_type, "test.event");
        assert_eq!(event.priority, Priority::P1);
        assert_eq!(event.metadata.user, Some("testuser".to_string()));
    }
}
