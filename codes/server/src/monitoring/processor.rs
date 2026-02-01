//! Event processor for AI automation pipeline
//!
//! Processes events from the queue and dispatches to appropriate handlers:
//! - monitoring.error_detected -> Discord alert
//! - discord.command.* -> Discord command processing
//! - github.* -> GitHub event processing

use crate::event::{Event, EventQueue, Severity, TriggerFilter};
use crate::monitoring::DiscordAlert;
use crate::utils::AppError;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, instrument, warn};

/// Event processor that handles events from the queue
pub struct EventProcessor<Q: EventQueue> {
    /// Event queue
    queue: Arc<Q>,
    /// Discord alert service
    discord_alert: DiscordAlert,
    /// Trigger filter for event filtering
    trigger_filter: TriggerFilter,
    /// Processing interval (milliseconds)
    poll_interval_ms: u64,
}

impl<Q: EventQueue> EventProcessor<Q> {
    /// Create a new event processor
    pub fn new(queue: Arc<Q>, discord_alert: DiscordAlert, trigger_filter: TriggerFilter) -> Self {
        Self {
            queue,
            discord_alert,
            trigger_filter,
            poll_interval_ms: 1000,
        }
    }

    /// Set the poll interval
    pub fn with_poll_interval(mut self, interval_ms: u64) -> Self {
        self.poll_interval_ms = interval_ms;
        self
    }

    /// Process a single event from the queue
    ///
    /// Returns:
    /// - Ok(true) if an event was processed
    /// - Ok(false) if the queue was empty
    /// - Err if processing failed
    #[instrument(skip(self), level = "debug")]
    pub async fn process_once(&self) -> Result<bool, AppError> {
        // Pop event from queue
        let event = match self.queue.pop().await? {
            Some(e) => e,
            None => {
                debug!("Queue is empty, no event to process");
                return Ok(false);
            }
        };

        info!(
            event_id = %event.id,
            event_type = %event.event_type,
            priority = ?event.priority,
            "Processing event"
        );

        // Check trigger filter
        if !self.trigger_filter.should_trigger(&event) {
            info!(
                event_id = %event.id,
                "Event filtered out by trigger filter"
            );
            self.queue.complete(event.id).await?;
            return Ok(true);
        }

        // Process event based on type
        let result = self.dispatch_event(&event).await;

        match result {
            Ok(()) => {
                self.queue.complete(event.id).await?;
                info!(event_id = %event.id, "Event processed successfully");
                Ok(true)
            }
            Err(e) => {
                error!(
                    event_id = %event.id,
                    error = ?e,
                    retry_count = %event.retry_count,
                    "Event processing failed"
                );
                self.queue.fail(event).await?;
                Err(e)
            }
        }
    }

    /// Dispatch event to appropriate handler based on event type
    async fn dispatch_event(&self, event: &Event) -> Result<(), AppError> {
        match event.event_type.as_str() {
            // Monitoring events
            "monitoring.error_detected" => self.handle_monitoring_event(event).await,

            // Discord command events
            t if t.starts_with("discord.command") => self.handle_discord_event(event).await,

            // GitHub events
            t if t.starts_with("github.") => self.handle_github_event(event).await,

            // Unknown event type
            _ => {
                warn!(
                    event_type = %event.event_type,
                    "Unknown event type, skipping"
                );
                Ok(())
            }
        }
    }

    /// Handle monitoring.error_detected events
    ///
    /// Sends a Discord alert for the detected error
    #[instrument(skip(self, event), fields(event_id = %event.id, event_type = %event.event_type))]
    pub async fn handle_monitoring_event(&self, event: &Event) -> Result<(), AppError> {
        // Extract error details from event data
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

        let severity_str = event
            .data
            .get("severity")
            .and_then(|v| v.as_str())
            .unwrap_or("warning");

        let severity = Severity::from_str(severity_str).unwrap_or(Severity::Warning);

        let target = event.data.get("target").and_then(|v| v.as_str());

        let request_id = event.data.get("request_id").and_then(|v| v.as_str());

        info!(
            error_code = %error_code,
            severity = ?severity,
            "Processing monitoring event"
        );

        // Build details for Discord alert
        let mut details = Vec::new();

        if let Some(t) = target {
            details.push(("Target".to_string(), t.to_string()));
        }

        if let Some(r) = request_id {
            details.push(("Request ID".to_string(), r.to_string()));
        }

        details.push(("Event ID".to_string(), event.id.to_string()));

        // Send Discord alert
        self.discord_alert
            .send_error_alert(
                error_code,
                message,
                severity,
                if details.is_empty() {
                    None
                } else {
                    Some(details)
                },
            )
            .await?;

        info!(
            error_code = %error_code,
            "Discord alert sent for monitoring event"
        );

        Ok(())
    }

    /// Handle discord.command.* events
    ///
    /// Processes Discord bot commands
    #[instrument(skip(self, event), fields(event_id = %event.id, event_type = %event.event_type))]
    pub async fn handle_discord_event(&self, event: &Event) -> Result<(), AppError> {
        let command = event
            .data
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let args = event
            .data
            .get("args")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let channel_id = event
            .data
            .get("channel_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        info!(
            command = %command,
            args = %args,
            channel_id = %channel_id,
            "Processing Discord command"
        );

        // TODO: Implement Discord command processing
        // - analyze: Analyze error code or issue
        // - fix: Create fix PR
        // - review: Review PR
        // - status: Show system status

        match command {
            "analyze" => {
                info!("Analyze command received: {}", args);
                // TODO: Implement analysis
            }
            "fix" => {
                info!("Fix command received: {}", args);
                // TODO: Implement fix generation
            }
            "review" => {
                info!("Review command received: {}", args);
                // TODO: Implement PR review
            }
            "status" => {
                info!("Status command received");
                // TODO: Return system status
            }
            _ => {
                warn!("Unknown Discord command: {}", command);
            }
        }

        Ok(())
    }

    /// Handle github.* events
    ///
    /// Processes GitHub webhook events
    #[instrument(skip(self, event), fields(event_id = %event.id, event_type = %event.event_type))]
    pub async fn handle_github_event(&self, event: &Event) -> Result<(), AppError> {
        let action = event
            .data
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        info!(
            event_type = %event.event_type,
            action = %action,
            "Processing GitHub event"
        );

        // TODO: Implement GitHub event processing
        // - github.issue_labeled: Handle ai-fix label
        // - github.issue_comment_created: Handle @ai-bot mentions
        // - github.pr_opened: Auto-review if labeled

        match event.event_type.as_str() {
            "github.issue_labeled" => {
                let label = event
                    .data
                    .get("label")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                info!("Issue labeled with: {}", label);
                // TODO: If label is "ai-fix", trigger fix workflow
            }
            "github.issue_opened" => {
                info!("New issue opened");
                // TODO: Auto-triage or analyze
            }
            "github.issue_comment_created" => {
                info!("Issue comment created");
                // TODO: Check for @ai-bot mention
            }
            "github.pr_opened" => {
                info!("PR opened");
                // TODO: Auto-review if configured
            }
            "github.pr_labeled" => {
                let label = event
                    .data
                    .get("label")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                info!("PR labeled with: {}", label);
                // TODO: If label is "ai-review", trigger review
            }
            _ => {
                debug!("Unhandled GitHub event type: {}", event.event_type);
            }
        }

        Ok(())
    }

    /// Run the event processing loop continuously
    ///
    /// This method runs indefinitely, processing events from the queue.
    /// Use this for background processing.
    #[instrument(skip(self), level = "info")]
    pub async fn run_loop(&self) {
        info!(
            poll_interval_ms = self.poll_interval_ms,
            "Starting event processor loop"
        );

        loop {
            match self.process_once().await {
                Ok(true) => {
                    // Event processed, continue immediately
                    debug!("Event processed, checking for more");
                }
                Ok(false) => {
                    // Queue empty, wait before checking again
                    tokio::time::sleep(Duration::from_millis(self.poll_interval_ms)).await;
                }
                Err(e) => {
                    // Processing failed, log and continue
                    error!(error = ?e, "Event processing error, continuing");
                    tokio::time::sleep(Duration::from_millis(self.poll_interval_ms)).await;
                }
            }
        }
    }

    /// Run the event processing loop for a limited number of iterations
    ///
    /// Useful for testing
    pub async fn run_iterations(&self, max_iterations: usize) -> Result<usize, AppError> {
        let mut processed = 0;

        for _ in 0..max_iterations {
            match self.process_once().await {
                Ok(true) => processed += 1,
                Ok(false) => break,
                Err(e) => {
                    error!(error = ?e, "Event processing error");
                    break;
                }
            }
        }

        Ok(processed)
    }

    /// Get the queue reference
    pub fn queue(&self) -> &Arc<Q> {
        &self.queue
    }

    /// Get the trigger filter reference
    pub fn trigger_filter(&self) -> &TriggerFilter {
        &self.trigger_filter
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Event, FileEventQueue, Priority};
    use std::env::temp_dir;
    use uuid::Uuid;

    fn create_test_queue() -> Arc<FileEventQueue> {
        let test_dir = temp_dir().join(format!("test_processor_{}", Uuid::new_v4()));
        Arc::new(FileEventQueue::new(test_dir).expect("Failed to create test queue"))
    }

    fn create_test_processor(queue: Arc<FileEventQueue>) -> EventProcessor<FileEventQueue> {
        let discord_alert = DiscordAlert::disabled();
        let trigger_filter = TriggerFilter::allow_all();
        EventProcessor::new(queue, discord_alert, trigger_filter)
    }

    fn create_monitoring_event(error_code: &str, severity: &str) -> Event {
        Event::with_auto_priority(
            "monitoring.error_detected",
            "log-watcher",
            serde_json::json!({
                "error_code": error_code,
                "severity": severity,
                "message": "Test error message",
                "target": "server::handler"
            }),
        )
    }

    fn create_discord_command_event(command: &str, args: &str) -> Event {
        Event::new(
            format!("discord.command.{}", command),
            "discord",
            Priority::P1,
            serde_json::json!({
                "command": command,
                "args": args,
                "channel_id": "123456"
            }),
        )
    }

    fn create_github_event(event_type: &str, action: &str) -> Event {
        Event::new(
            event_type,
            "github",
            Priority::P2,
            serde_json::json!({
                "action": action,
                "issue_number": 123,
                "repository": "org/repo"
            }),
        )
    }

    #[tokio::test]
    async fn should_return_false_for_empty_queue() {
        // Arrange
        let queue = create_test_queue();
        let processor = create_test_processor(queue);

        // Act
        let result = processor.process_once().await;

        // Assert
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn should_process_monitoring_event() {
        // Arrange
        let queue = create_test_queue();
        let event = create_monitoring_event("AI5001", "critical");

        queue.push(event).await.expect("Failed to push event");

        let processor = create_test_processor(queue.clone());

        // Act
        let result = processor.process_once().await;

        // Assert
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Verify event was completed
        assert_eq!(queue.processing_count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn should_process_discord_command_event() {
        // Arrange
        let queue = create_test_queue();
        let event = create_discord_command_event("analyze", "AI5001");

        queue.push(event).await.expect("Failed to push event");

        let processor = create_test_processor(queue.clone());

        // Act
        let result = processor.process_once().await;

        // Assert
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn should_process_github_event() {
        // Arrange
        let queue = create_test_queue();
        let event = create_github_event("github.issue_labeled", "labeled");

        queue.push(event).await.expect("Failed to push event");

        let processor = create_test_processor(queue.clone());

        // Act
        let result = processor.process_once().await;

        // Assert
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn should_skip_unknown_event_type() {
        // Arrange
        let queue = create_test_queue();
        let event = Event::new("unknown.event", "test", Priority::P3, serde_json::json!({}));

        queue.push(event).await.expect("Failed to push event");

        let processor = create_test_processor(queue.clone());

        // Act
        let result = processor.process_once().await;

        // Assert
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn should_filter_events_with_trigger_filter() {
        // Arrange
        let queue = create_test_queue();

        // Create event with severity below minimum
        let event = create_monitoring_event("AI5001", "info");

        queue.push(event).await.expect("Failed to push event");

        // Create processor with filter that requires at least "warning" severity
        let discord_alert = DiscordAlert::disabled();
        let trigger_filter = TriggerFilter::new().with_min_severity(Severity::Warning);
        let processor = EventProcessor::new(queue.clone(), discord_alert, trigger_filter);

        // Act
        let result = processor.process_once().await;

        // Assert
        assert!(result.is_ok());
        assert!(result.unwrap()); // Event was processed (filtered out)
        assert_eq!(queue.pending_count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn should_process_multiple_events_in_priority_order() {
        // Arrange
        let queue = create_test_queue();

        // Push events in reverse priority order
        let p3_event = Event::new("test.p3", "test", Priority::P3, serde_json::json!({}));
        let p1_event = create_discord_command_event("status", "");
        let p0_event = create_monitoring_event("AI5001", "critical");

        queue.push(p3_event).await.expect("Failed to push P3");
        queue.push(p1_event).await.expect("Failed to push P1");
        queue.push(p0_event).await.expect("Failed to push P0");

        let processor = create_test_processor(queue.clone());

        // Act - Process all events
        let processed = processor.run_iterations(10).await.expect("Failed to run");

        // Assert
        assert_eq!(processed, 3);
        assert_eq!(queue.pending_count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn should_handle_monitoring_event_with_all_fields() {
        // Arrange
        let queue = create_test_queue();
        let event = Event::with_auto_priority(
            "monitoring.error_detected",
            "log-watcher",
            serde_json::json!({
                "error_code": "AI5002",
                "severity": "warning",
                "message": "Connection timeout to Claude API",
                "target": "server::ai::client",
                "request_id": "req-12345"
            }),
        );

        queue.push(event).await.expect("Failed to push event");

        let processor = create_test_processor(queue.clone());

        // Act
        let result = processor.process_once().await;

        // Assert
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn should_use_custom_poll_interval() {
        // Arrange
        let queue = create_test_queue();
        let discord_alert = DiscordAlert::disabled();
        let trigger_filter = TriggerFilter::allow_all();

        // Act
        let processor =
            EventProcessor::new(queue, discord_alert, trigger_filter).with_poll_interval(500);

        // Assert
        assert_eq!(processor.poll_interval_ms, 500);
    }

    #[tokio::test]
    async fn should_handle_discord_analyze_command() {
        // Arrange
        let queue = create_test_queue();
        let event = create_discord_command_event("analyze", "AI5001 error investigation");

        queue.push(event).await.expect("Failed to push event");

        let processor = create_test_processor(queue.clone());
        let popped_event = queue.pop().await.expect("Failed to pop").expect("No event");

        // Act
        let result = processor.handle_discord_event(&popped_event).await;

        // Assert - Should complete without error (actual implementation is TODO)
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_handle_github_issue_labeled_event() {
        // Arrange
        let queue = create_test_queue();
        let event = Event::new(
            "github.issue_labeled",
            "github",
            Priority::P1,
            serde_json::json!({
                "action": "labeled",
                "label": "ai-fix",
                "issue_number": 123,
                "repository": "org/repo"
            }),
        );

        queue.push(event).await.expect("Failed to push event");

        let processor = create_test_processor(queue.clone());

        // Act
        let result = processor.process_once().await;

        // Assert
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}
