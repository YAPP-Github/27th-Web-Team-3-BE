//! Trigger filtering logic for AI automation pipeline

use crate::event::{Event, Severity};
use std::collections::HashSet;
use std::str::FromStr;
use tracing::{debug, info};

/// Trigger filter configuration and logic
#[derive(Debug, Clone)]
pub struct TriggerFilter {
    /// Enabled event types (empty = all enabled)
    enabled_events: HashSet<String>,
    /// Allowed users (whitelist, empty = all allowed)
    allowed_users: HashSet<String>,
    /// Ignored error codes (blacklist)
    ignored_error_codes: HashSet<String>,
    /// Minimum severity level to trigger
    min_severity: Severity,
    /// Whether the filter is active
    active: bool,
}

impl Default for TriggerFilter {
    fn default() -> Self {
        Self {
            enabled_events: HashSet::new(),
            allowed_users: HashSet::new(),
            ignored_error_codes: HashSet::new(),
            min_severity: Severity::Warning,
            active: true,
        }
    }
}

impl TriggerFilter {
    /// Create a new trigger filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a filter that accepts all events
    pub fn allow_all() -> Self {
        Self {
            active: false,
            ..Self::default()
        }
    }

    /// Enable specific event types
    pub fn with_enabled_events(
        mut self,
        events: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.enabled_events = events.into_iter().map(|e| e.into()).collect();
        self
    }

    /// Add event type to enabled list
    pub fn enable_event(mut self, event_type: impl Into<String>) -> Self {
        self.enabled_events.insert(event_type.into());
        self
    }

    /// Set allowed users (whitelist)
    pub fn with_allowed_users(
        mut self,
        users: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.allowed_users = users.into_iter().map(|u| u.into()).collect();
        self
    }

    /// Add user to allowed list
    pub fn allow_user(mut self, user: impl Into<String>) -> Self {
        self.allowed_users.insert(user.into());
        self
    }

    /// Set ignored error codes (blacklist)
    pub fn with_ignored_error_codes(
        mut self,
        codes: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.ignored_error_codes = codes.into_iter().map(|c| c.into()).collect();
        self
    }

    /// Add error code to ignore list
    pub fn ignore_error_code(mut self, code: impl Into<String>) -> Self {
        self.ignored_error_codes.insert(code.into());
        self
    }

    /// Set minimum severity level
    pub fn with_min_severity(mut self, severity: Severity) -> Self {
        self.min_severity = severity;
        self
    }

    /// Set filter active state
    pub fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Check if an event should trigger the AI pipeline
    pub fn should_trigger(&self, event: &Event) -> bool {
        // If filter is not active, always trigger
        if !self.active {
            return true;
        }

        // 1. Check event type whitelist
        if !self.enabled_events.is_empty() && !self.enabled_events.contains(&event.event_type) {
            debug!(
                event_type = %event.event_type,
                "Event type not in enabled list, skipping"
            );
            return false;
        }

        // 2. Check user whitelist
        if !self.allowed_users.is_empty() {
            match &event.metadata.user {
                Some(user) if self.allowed_users.contains(user) => {
                    // User is in allowed list, continue
                }
                Some(user) => {
                    debug!(
                        user = %user,
                        "User not in allowed list, skipping"
                    );
                    return false;
                }
                None => {
                    debug!("Event has no user but allowed_users is configured, skipping");
                    return false;
                }
            }
        }

        // 3. Check error code blacklist (for monitoring events)
        if let Some(error_code) = event.data.get("error_code").and_then(|v| v.as_str()) {
            if self.ignored_error_codes.contains(error_code) {
                debug!(
                    error_code = %error_code,
                    "Error code in ignored list, skipping"
                );
                return false;
            }
        }

        // 4. Check severity level (for monitoring events)
        if let Some(severity_str) = event.data.get("severity").and_then(|v| v.as_str()) {
            if let Ok(severity) = Severity::from_str(severity_str) {
                if severity < self.min_severity {
                    debug!(
                        severity = ?severity,
                        min_severity = ?self.min_severity,
                        "Severity below minimum, skipping"
                    );
                    return false;
                }
            }
        }

        info!(
            event_id = %event.id,
            event_type = %event.event_type,
            "Event passed trigger filter"
        );

        true
    }

    /// Get the enabled event types
    pub fn enabled_events(&self) -> &HashSet<String> {
        &self.enabled_events
    }

    /// Get the allowed users
    pub fn allowed_users(&self) -> &HashSet<String> {
        &self.allowed_users
    }

    /// Get the ignored error codes
    pub fn ignored_error_codes(&self) -> &HashSet<String> {
        &self.ignored_error_codes
    }

    /// Get the minimum severity level
    pub fn min_severity(&self) -> Severity {
        self.min_severity
    }

    /// Check if the filter is active
    pub fn is_active(&self) -> bool {
        self.active
    }
}

/// Builder for creating TriggerFilter from environment variables
pub struct TriggerFilterBuilder {
    filter: TriggerFilter,
}

impl TriggerFilterBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            filter: TriggerFilter::new(),
        }
    }

    /// Load configuration from environment variables
    pub fn load_from_env(mut self) -> Self {
        // TRIGGER_ENABLED_EVENTS (comma-separated)
        if let Ok(events) = std::env::var("TRIGGER_ENABLED_EVENTS") {
            self.filter.enabled_events = events
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        // TRIGGER_ALLOWED_USERS (comma-separated)
        if let Ok(users) = std::env::var("TRIGGER_ALLOWED_USERS") {
            self.filter.allowed_users = users
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        // TRIGGER_IGNORED_ERROR_CODES (comma-separated)
        if let Ok(codes) = std::env::var("TRIGGER_IGNORED_ERROR_CODES") {
            self.filter.ignored_error_codes = codes
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        // TRIGGER_MIN_SEVERITY (info, warning, critical)
        if let Ok(severity_str) = std::env::var("TRIGGER_MIN_SEVERITY") {
            if let Ok(severity) = Severity::from_str(&severity_str) {
                self.filter.min_severity = severity;
            }
        }

        self
    }

    /// Build the trigger filter
    pub fn build(self) -> TriggerFilter {
        self.filter
    }
}

impl Default for TriggerFilterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Event, Priority};

    fn create_monitoring_event(error_code: &str, severity: &str) -> Event {
        Event::new(
            "monitoring.error_detected",
            "log-watcher",
            Priority::P0,
            serde_json::json!({
                "error_code": error_code,
                "severity": severity,
                "message": "Test error"
            }),
        )
    }

    fn create_discord_event(user: &str) -> Event {
        Event::new(
            "discord.command",
            "discord",
            Priority::P1,
            serde_json::json!({
                "command": "analyze",
                "args": "test"
            }),
        )
        .with_user(user)
    }

    #[test]
    fn should_trigger_when_filter_inactive() {
        // Arrange
        let filter = TriggerFilter::allow_all();
        let event = create_monitoring_event("AI5001", "info");

        // Act
        let result = filter.should_trigger(&event);

        // Assert
        assert!(result);
    }

    #[test]
    fn should_filter_by_event_type() {
        // Arrange
        let filter = TriggerFilter::new().with_enabled_events(vec!["discord.command"]);

        let discord_event = create_discord_event("testuser");
        let monitoring_event = create_monitoring_event("AI5001", "critical");

        // Act & Assert
        assert!(filter.should_trigger(&discord_event));
        assert!(!filter.should_trigger(&monitoring_event));
    }

    #[test]
    fn should_filter_by_user_whitelist() {
        // Arrange
        let filter = TriggerFilter::new().with_allowed_users(vec!["admin", "developer"]);

        let allowed_event = create_discord_event("admin");
        let blocked_event = create_discord_event("random_user");

        // Act & Assert
        assert!(filter.should_trigger(&allowed_event));
        assert!(!filter.should_trigger(&blocked_event));
    }

    #[test]
    fn should_allow_all_users_when_whitelist_empty() {
        // Arrange
        let filter = TriggerFilter::new(); // Empty whitelist

        let event = create_discord_event("any_user");

        // Act & Assert
        assert!(filter.should_trigger(&event));
    }

    #[test]
    fn should_filter_by_ignored_error_codes() {
        // Arrange
        let filter = TriggerFilter::new()
            .with_ignored_error_codes(vec!["AUTH4001", "AUTH4002"])
            .with_min_severity(Severity::Info); // Allow all severities

        let ignored_event = create_monitoring_event("AUTH4001", "warning");
        let allowed_event = create_monitoring_event("AI5001", "warning");

        // Act & Assert
        assert!(!filter.should_trigger(&ignored_event));
        assert!(filter.should_trigger(&allowed_event));
    }

    #[test]
    fn should_filter_by_minimum_severity() {
        // Arrange
        let filter = TriggerFilter::new().with_min_severity(Severity::Warning);

        let critical_event = create_monitoring_event("AI5001", "critical");
        let warning_event = create_monitoring_event("AI5001", "warning");
        let info_event = create_monitoring_event("AI5001", "info");

        // Act & Assert
        assert!(filter.should_trigger(&critical_event));
        assert!(filter.should_trigger(&warning_event));
        assert!(!filter.should_trigger(&info_event));
    }

    #[test]
    fn should_apply_multiple_filters() {
        // Arrange
        let filter = TriggerFilter::new()
            .with_enabled_events(vec!["monitoring.error_detected"])
            .with_min_severity(Severity::Warning)
            .with_ignored_error_codes(vec!["AUTH4001"]);

        // Event that passes all filters
        let passing_event = create_monitoring_event("AI5001", "critical");

        // Event that fails event type filter
        let wrong_type = create_discord_event("admin");

        // Event that fails severity filter
        let low_severity = create_monitoring_event("AI5001", "info");

        // Event that fails error code filter
        let ignored_code = create_monitoring_event("AUTH4001", "critical");

        // Act & Assert
        assert!(filter.should_trigger(&passing_event));
        assert!(!filter.should_trigger(&wrong_type));
        assert!(!filter.should_trigger(&low_severity));
        assert!(!filter.should_trigger(&ignored_code));
    }

    #[test]
    fn should_use_builder_pattern() {
        // Arrange
        let filter = TriggerFilter::new()
            .enable_event("monitoring.error_detected")
            .enable_event("discord.command")
            .allow_user("admin")
            .ignore_error_code("AUTH4001")
            .with_min_severity(Severity::Warning)
            .with_active(true);

        // Assert
        assert!(filter.is_active());
        assert!(filter
            .enabled_events()
            .contains("monitoring.error_detected"));
        assert!(filter.enabled_events().contains("discord.command"));
        assert!(filter.allowed_users().contains("admin"));
        assert!(filter.ignored_error_codes().contains("AUTH4001"));
        assert_eq!(filter.min_severity(), Severity::Warning);
    }

    #[test]
    fn should_block_events_without_user_when_whitelist_configured() {
        // Arrange
        let filter = TriggerFilter::new().with_allowed_users(vec!["admin", "developer"]);

        // Event without user metadata
        let event_without_user = Event::new(
            "discord.command",
            "discord",
            Priority::P1,
            serde_json::json!({
                "command": "analyze",
                "args": "test"
            }),
        );
        // Note: no .with_user() call, so metadata.user is None

        // Act & Assert
        assert!(!filter.should_trigger(&event_without_user));
    }
}
