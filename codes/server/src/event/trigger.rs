//! Trigger filtering logic for AI automation pipeline

use crate::event::{Event, Severity};
use std::collections::{HashMap, HashSet, VecDeque};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Rate limit configuration for different action types
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum API calls per minute
    pub api_calls_per_minute: u32,
    /// Maximum branch creations per hour
    pub branch_creations_per_hour: u32,
    /// Maximum PR creations per hour
    pub pr_creations_per_hour: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        // Default values from security-governance.md
        Self {
            api_calls_per_minute: 10,
            branch_creations_per_hour: 20,
            pr_creations_per_hour: 10,
        }
    }
}

impl RateLimitConfig {
    /// Create a new rate limit configuration
    pub fn new(
        api_calls_per_minute: u32,
        branch_creations_per_hour: u32,
        pr_creations_per_hour: u32,
    ) -> Self {
        Self {
            api_calls_per_minute,
            branch_creations_per_hour,
            pr_creations_per_hour,
        }
    }
}

/// Action types that can be rate limited
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RateLimitAction {
    /// API call action (limited per minute)
    ApiCall,
    /// Branch creation action (limited per hour)
    BranchCreation,
    /// PR creation action (limited per hour)
    PrCreation,
}

impl RateLimitAction {
    /// Get the window duration for this action type
    pub fn window_duration(&self) -> Duration {
        match self {
            RateLimitAction::ApiCall => Duration::from_secs(60), // 1 minute
            RateLimitAction::BranchCreation => Duration::from_secs(3600), // 1 hour
            RateLimitAction::PrCreation => Duration::from_secs(3600), // 1 hour
        }
    }
}

/// Sliding window entry for rate limiting
#[derive(Debug, Clone)]
struct WindowEntry {
    timestamp: Instant,
}

/// Rate limiter using sliding window algorithm
///
/// This implementation uses an in-memory HashMap with Mutex for thread safety.
/// Each action type has its own sliding window of timestamps.
#[derive(Debug)]
pub struct RateLimiter {
    /// Configuration for rate limits
    config: RateLimitConfig,
    /// Sliding window entries per action type
    /// Maps action type to a deque of timestamps
    windows: Arc<Mutex<HashMap<RateLimitAction, VecDeque<WindowEntry>>>>,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(RateLimitConfig::default())
    }
}

/// Clone implementation that shares the same rate limit state.
///
/// **Note**: Cloning a `RateLimiter` shares the underlying sliding window data
/// via `Arc`. This means all clones will share the same rate limit counts.
/// This is intentional for use cases where you want a single rate limit
/// across multiple handlers or threads.
///
/// If you need independent rate limiters, create a new instance with `RateLimiter::new()`.
impl Clone for RateLimiter {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            windows: Arc::clone(&self.windows),
        }
    }
}

impl RateLimiter {
    /// Create a new rate limiter with the given configuration
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            windows: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get the limit for a specific action type
    fn get_limit(&self, action: RateLimitAction) -> u32 {
        match action {
            RateLimitAction::ApiCall => self.config.api_calls_per_minute,
            RateLimitAction::BranchCreation => self.config.branch_creations_per_hour,
            RateLimitAction::PrCreation => self.config.pr_creations_per_hour,
        }
    }

    /// Check if an action is allowed without recording it
    pub fn check(&self, action: RateLimitAction) -> bool {
        let limit = self.get_limit(action);
        let window_duration = action.window_duration();
        let now = Instant::now();

        let windows = match self.windows.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Rate limiter mutex poisoned, recovering");
                poisoned.into_inner()
            }
        };

        let count = windows
            .get(&action)
            .map(|entries| {
                entries
                    .iter()
                    .filter(|e| now.duration_since(e.timestamp) < window_duration)
                    .count()
            })
            .unwrap_or(0);

        count < limit as usize
    }

    /// Try to acquire permission for an action
    ///
    /// Returns true if the action is allowed and records it.
    /// Returns false if the rate limit would be exceeded.
    pub fn try_acquire(&self, action: RateLimitAction) -> bool {
        let limit = self.get_limit(action);
        let window_duration = action.window_duration();
        let now = Instant::now();

        let mut windows = match self.windows.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("Rate limiter mutex poisoned, recovering");
                poisoned.into_inner()
            }
        };

        // Get or create the window for this action
        let entries = windows.entry(action).or_insert_with(VecDeque::new);

        // Remove expired entries (sliding window cleanup)
        while let Some(front) = entries.front() {
            if now.duration_since(front.timestamp) >= window_duration {
                entries.pop_front();
            } else {
                break;
            }
        }

        // Check if we're within the limit
        if entries.len() < limit as usize {
            entries.push_back(WindowEntry { timestamp: now });
            debug!(
                action = ?action,
                current_count = entries.len(),
                limit = limit,
                "Rate limit check passed"
            );
            true
        } else {
            warn!(
                action = ?action,
                current_count = entries.len(),
                limit = limit,
                "Rate limit exceeded"
            );
            false
        }
    }

    /// Record an action (for external tracking)
    pub fn record(&self, action: RateLimitAction) {
        let _ = self.try_acquire(action);
    }

    /// Get the current count for an action type
    pub fn current_count(&self, action: RateLimitAction) -> usize {
        let window_duration = action.window_duration();
        let now = Instant::now();

        let windows = match self.windows.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        windows
            .get(&action)
            .map(|entries| {
                entries
                    .iter()
                    .filter(|e| now.duration_since(e.timestamp) < window_duration)
                    .count()
            })
            .unwrap_or(0)
    }

    /// Get the remaining capacity for an action type
    pub fn remaining(&self, action: RateLimitAction) -> u32 {
        let limit = self.get_limit(action);
        let current = self.current_count(action) as u32;
        limit.saturating_sub(current)
    }

    /// Reset all rate limit counters (useful for testing)
    pub fn reset(&self) {
        let mut windows = match self.windows.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        windows.clear();
    }

    /// Get the configuration
    pub fn config(&self) -> &RateLimitConfig {
        &self.config
    }
}

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
    /// Rate limiter for action throttling
    rate_limiter: Option<RateLimiter>,
}

impl Default for TriggerFilter {
    fn default() -> Self {
        Self {
            enabled_events: HashSet::new(),
            allowed_users: HashSet::new(),
            ignored_error_codes: HashSet::new(),
            min_severity: Severity::Warning,
            active: true,
            rate_limiter: None,
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

    /// Set the rate limiter
    pub fn with_rate_limiter(mut self, rate_limiter: RateLimiter) -> Self {
        self.rate_limiter = Some(rate_limiter);
        self
    }

    /// Get the rate limiter (if configured)
    pub fn rate_limiter(&self) -> Option<&RateLimiter> {
        self.rate_limiter.as_ref()
    }

    /// Check if an action is rate limited
    ///
    /// Returns true if the action is allowed, false if rate limited.
    /// If no rate limiter is configured, always returns true.
    pub fn check_rate_limit(&self, action: RateLimitAction) -> bool {
        match &self.rate_limiter {
            Some(limiter) => limiter.try_acquire(action),
            None => true,
        }
    }

    /// Check if an action would be rate limited (without recording)
    pub fn would_be_rate_limited(&self, action: RateLimitAction) -> bool {
        match &self.rate_limiter {
            Some(limiter) => !limiter.check(action),
            None => false,
        }
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
            match Severity::from_str(severity_str) {
                Ok(severity) => {
                    if severity < self.min_severity {
                        debug!(
                            severity = ?severity,
                            min_severity = ?self.min_severity,
                            "Severity below minimum, skipping"
                        );
                        return false;
                    }
                }
                Err(_) => {
                    // Invalid severity string - treat as lowest severity (Info) for safety
                    debug!(
                        severity_str = %severity_str,
                        min_severity = ?self.min_severity,
                        "Invalid severity value, treating as Info level"
                    );
                    if Severity::Info < self.min_severity {
                        return false;
                    }
                }
            }
        } else if event.event_type.starts_with("monitoring.") {
            // Monitoring events without severity field - apply min_severity check with Info default
            debug!(
                event_type = %event.event_type,
                min_severity = ?self.min_severity,
                "Monitoring event without severity, treating as Info level"
            );
            if Severity::Info < self.min_severity {
                return false;
            }
        }

        // 5. Check rate limit (if configured)
        // Determine the action type based on event type
        if let Some(ref limiter) = self.rate_limiter {
            let action = Self::event_to_rate_limit_action(event);
            if !limiter.try_acquire(action) {
                warn!(
                    event_id = %event.id,
                    event_type = %event.event_type,
                    action = ?action,
                    "Event blocked by rate limiter"
                );
                return false;
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

    /// Map an event type to a rate limit action
    ///
    /// This determines which rate limit bucket an event should be counted against.
    ///
    /// Matching rules:
    /// - Branch events: Contains "branch" (case-insensitive)
    /// - PR events: Contains "pull_request", or has ".pr." / ".PR." segment,
    ///   or ends with ".pr" / ".PR", or starts with "pr." / "PR."
    /// - All other events: API call
    fn event_to_rate_limit_action(event: &Event) -> RateLimitAction {
        let event_type = event.event_type.as_str();
        let event_type_lower = event_type.to_lowercase();

        // Branch creation events (case-insensitive)
        if event_type_lower.contains("branch") {
            return RateLimitAction::BranchCreation;
        }

        // PR creation events
        // Match "pull_request" explicitly (case-insensitive)
        if event_type_lower.contains("pull_request") {
            return RateLimitAction::PrCreation;
        }

        // Match "pr" as a distinct segment (not as part of another word)
        // Valid patterns: ".pr.", ".pr" (at end), "pr." (at start)
        if Self::contains_pr_segment(&event_type_lower) {
            return RateLimitAction::PrCreation;
        }

        // Default to API call for all other events
        RateLimitAction::ApiCall
    }

    /// Check if the event type contains "pr" as a distinct segment
    ///
    /// This avoids false positives like "approve", "prepare", "profile", etc.
    /// Valid patterns:
    /// - Starts with "pr." (e.g., "pr.created")
    /// - Ends with ".pr" (e.g., "github.pr")
    /// - Contains ".pr." (e.g., "github.pr.created")
    /// - Exactly "pr"
    fn contains_pr_segment(event_type: &str) -> bool {
        event_type == "pr"
            || event_type.starts_with("pr.")
            || event_type.ends_with(".pr")
            || event_type.contains(".pr.")
    }
}

/// Builder for creating TriggerFilter from environment variables
#[derive(Default)]
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

    /// Set a custom rate limiter
    pub fn with_rate_limiter(mut self, rate_limiter: RateLimiter) -> Self {
        self.filter.rate_limiter = Some(rate_limiter);
        self
    }

    /// Set rate limiter with custom configuration
    pub fn with_rate_limit_config(mut self, config: RateLimitConfig) -> Self {
        self.filter.rate_limiter = Some(RateLimiter::new(config));
        self
    }

    /// Enable rate limiting with default configuration
    ///
    /// Uses default values from security-governance.md:
    /// - api_calls_per_minute: 10
    /// - branch_creations_per_hour: 20
    /// - pr_creations_per_hour: 10
    pub fn with_default_rate_limiting(mut self) -> Self {
        self.filter.rate_limiter = Some(RateLimiter::default());
        self
    }

    /// Load rate limit configuration from environment variables
    ///
    /// Environment variables:
    /// - RATE_LIMIT_API_CALLS_PER_MINUTE (default: 10)
    /// - RATE_LIMIT_BRANCH_CREATIONS_PER_HOUR (default: 20)
    /// - RATE_LIMIT_PR_CREATIONS_PER_HOUR (default: 10)
    pub fn load_rate_limits_from_env(mut self) -> Self {
        let api_calls = std::env::var("RATE_LIMIT_API_CALLS_PER_MINUTE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);

        let branch_creations = std::env::var("RATE_LIMIT_BRANCH_CREATIONS_PER_HOUR")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(20);

        let pr_creations = std::env::var("RATE_LIMIT_PR_CREATIONS_PER_HOUR")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);

        let config = RateLimitConfig::new(api_calls, branch_creations, pr_creations);
        self.filter.rate_limiter = Some(RateLimiter::new(config));
        self
    }

    /// Build the trigger filter
    pub fn build(self) -> TriggerFilter {
        self.filter
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

    #[test]
    fn should_treat_invalid_severity_as_info() {
        // Arrange
        let filter = TriggerFilter::new().with_min_severity(Severity::Warning);

        // Event with invalid severity value
        let event = Event::new(
            "monitoring.error_detected",
            "log-watcher",
            Priority::P0,
            serde_json::json!({
                "error_code": "AI5001",
                "severity": "invalid_value",
                "message": "Test error"
            }),
        );

        // Act & Assert
        // Invalid severity is treated as Info, which is below Warning
        assert!(!filter.should_trigger(&event));
    }

    #[test]
    fn should_allow_invalid_severity_when_min_is_info() {
        // Arrange
        let filter = TriggerFilter::new().with_min_severity(Severity::Info);

        // Event with invalid severity value
        let event = Event::new(
            "monitoring.error_detected",
            "log-watcher",
            Priority::P0,
            serde_json::json!({
                "error_code": "AI5001",
                "severity": "invalid_value",
                "message": "Test error"
            }),
        );

        // Act & Assert
        // Invalid severity is treated as Info, min is Info, so should pass
        assert!(filter.should_trigger(&event));
    }

    #[test]
    fn should_treat_monitoring_event_without_severity_as_info() {
        // Arrange
        let filter = TriggerFilter::new().with_min_severity(Severity::Warning);

        // Monitoring event without severity field
        let event = Event::new(
            "monitoring.error_detected",
            "log-watcher",
            Priority::P0,
            serde_json::json!({
                "error_code": "AI5001",
                "message": "Test error"
                // Note: no "severity" field
            }),
        );

        // Act & Assert
        // Missing severity is treated as Info, which is below Warning
        assert!(!filter.should_trigger(&event));
    }

    #[test]
    fn should_allow_non_monitoring_event_without_severity() {
        // Arrange
        let filter = TriggerFilter::new().with_min_severity(Severity::Warning);

        // Non-monitoring event without severity field
        let event = Event::new(
            "discord.command",
            "discord",
            Priority::P1,
            serde_json::json!({
                "command": "analyze",
                "args": "test"
                // Note: no "severity" field
            }),
        );

        // Act & Assert
        // Non-monitoring events don't require severity check
        assert!(filter.should_trigger(&event));
    }

    // ============================================
    // Rate Limiter Tests
    // ============================================

    mod rate_limiter_tests {
        use super::*;

        #[test]
        fn should_create_rate_limiter_with_default_config() {
            // Arrange & Act
            let limiter = RateLimiter::default();

            // Assert
            assert_eq!(limiter.config().api_calls_per_minute, 10);
            assert_eq!(limiter.config().branch_creations_per_hour, 20);
            assert_eq!(limiter.config().pr_creations_per_hour, 10);
        }

        #[test]
        fn should_create_rate_limiter_with_custom_config() {
            // Arrange
            let config = RateLimitConfig::new(5, 10, 3);

            // Act
            let limiter = RateLimiter::new(config);

            // Assert
            assert_eq!(limiter.config().api_calls_per_minute, 5);
            assert_eq!(limiter.config().branch_creations_per_hour, 10);
            assert_eq!(limiter.config().pr_creations_per_hour, 3);
        }

        #[test]
        fn should_allow_actions_within_limit() {
            // Arrange
            let config = RateLimitConfig::new(3, 5, 2);
            let limiter = RateLimiter::new(config);

            // Act & Assert - API calls (limit: 3)
            assert!(limiter.try_acquire(RateLimitAction::ApiCall));
            assert!(limiter.try_acquire(RateLimitAction::ApiCall));
            assert!(limiter.try_acquire(RateLimitAction::ApiCall));

            // Verify count
            assert_eq!(limiter.current_count(RateLimitAction::ApiCall), 3);
            assert_eq!(limiter.remaining(RateLimitAction::ApiCall), 0);
        }

        #[test]
        fn should_block_actions_exceeding_limit() {
            // Arrange
            let config = RateLimitConfig::new(2, 5, 2);
            let limiter = RateLimiter::new(config);

            // Act - Exhaust the limit
            assert!(limiter.try_acquire(RateLimitAction::ApiCall));
            assert!(limiter.try_acquire(RateLimitAction::ApiCall));

            // Assert - Third call should be blocked
            assert!(!limiter.try_acquire(RateLimitAction::ApiCall));
            assert_eq!(limiter.current_count(RateLimitAction::ApiCall), 2);
        }

        #[test]
        fn should_track_different_action_types_separately() {
            // Arrange
            let config = RateLimitConfig::new(2, 3, 1);
            let limiter = RateLimiter::new(config);

            // Act - Use different action types
            assert!(limiter.try_acquire(RateLimitAction::ApiCall));
            assert!(limiter.try_acquire(RateLimitAction::ApiCall));
            assert!(limiter.try_acquire(RateLimitAction::BranchCreation));
            assert!(limiter.try_acquire(RateLimitAction::PrCreation));

            // Assert - API calls exhausted, but branch and PR still have room
            assert!(!limiter.try_acquire(RateLimitAction::ApiCall));
            assert!(limiter.try_acquire(RateLimitAction::BranchCreation));
            assert!(!limiter.try_acquire(RateLimitAction::PrCreation));
        }

        #[test]
        fn should_check_without_recording() {
            // Arrange
            let config = RateLimitConfig::new(2, 5, 2);
            let limiter = RateLimiter::new(config);

            // Act - Check without recording
            assert!(limiter.check(RateLimitAction::ApiCall));
            assert!(limiter.check(RateLimitAction::ApiCall));

            // Assert - Count should still be 0
            assert_eq!(limiter.current_count(RateLimitAction::ApiCall), 0);

            // Now actually record
            assert!(limiter.try_acquire(RateLimitAction::ApiCall));
            assert_eq!(limiter.current_count(RateLimitAction::ApiCall), 1);
        }

        #[test]
        fn should_reset_all_counters() {
            // Arrange
            let config = RateLimitConfig::new(2, 5, 2);
            let limiter = RateLimiter::new(config);

            // Act - Record some actions
            limiter.try_acquire(RateLimitAction::ApiCall);
            limiter.try_acquire(RateLimitAction::BranchCreation);
            limiter.try_acquire(RateLimitAction::PrCreation);

            // Reset
            limiter.reset();

            // Assert - All counters should be 0
            assert_eq!(limiter.current_count(RateLimitAction::ApiCall), 0);
            assert_eq!(limiter.current_count(RateLimitAction::BranchCreation), 0);
            assert_eq!(limiter.current_count(RateLimitAction::PrCreation), 0);
        }

        #[test]
        fn should_clone_and_share_state() {
            // Arrange
            let config = RateLimitConfig::new(3, 5, 2);
            let limiter1 = RateLimiter::new(config);

            // Act - Clone and modify through both references
            let limiter2 = limiter1.clone();
            limiter1.try_acquire(RateLimitAction::ApiCall);
            limiter2.try_acquire(RateLimitAction::ApiCall);

            // Assert - Both should see the shared state
            assert_eq!(limiter1.current_count(RateLimitAction::ApiCall), 2);
            assert_eq!(limiter2.current_count(RateLimitAction::ApiCall), 2);
        }

        #[test]
        fn should_return_correct_remaining_capacity() {
            // Arrange
            let config = RateLimitConfig::new(5, 10, 3);
            let limiter = RateLimiter::new(config);

            // Act & Assert
            assert_eq!(limiter.remaining(RateLimitAction::ApiCall), 5);

            limiter.try_acquire(RateLimitAction::ApiCall);
            limiter.try_acquire(RateLimitAction::ApiCall);

            assert_eq!(limiter.remaining(RateLimitAction::ApiCall), 3);

            limiter.try_acquire(RateLimitAction::ApiCall);
            limiter.try_acquire(RateLimitAction::ApiCall);
            limiter.try_acquire(RateLimitAction::ApiCall);

            assert_eq!(limiter.remaining(RateLimitAction::ApiCall), 0);
        }
    }

    // ============================================
    // TriggerFilter with RateLimiter Tests
    // ============================================

    mod trigger_filter_rate_limit_tests {
        use super::*;

        fn create_api_event() -> Event {
            Event::new(
                "api.request",
                "api-gateway",
                Priority::P1,
                serde_json::json!({
                    "endpoint": "/health",
                    "method": "GET"
                }),
            )
        }

        fn create_branch_event() -> Event {
            Event::new(
                "git.branch.create",
                "git-handler",
                Priority::P1,
                serde_json::json!({
                    "branch_name": "fix/test-branch"
                }),
            )
        }

        fn create_pr_event() -> Event {
            Event::new(
                "github.pull_request.create",
                "github-handler",
                Priority::P1,
                serde_json::json!({
                    "title": "Fix: Test PR",
                    "base": "dev"
                }),
            )
        }

        #[test]
        fn should_allow_events_without_rate_limiter() {
            // Arrange
            let filter = TriggerFilter::new().with_min_severity(Severity::Info);
            let event = create_api_event();

            // Act & Assert - Multiple events should all pass
            assert!(filter.should_trigger(&event));
            assert!(filter.should_trigger(&event));
            assert!(filter.should_trigger(&event));
        }

        #[test]
        fn should_block_events_when_rate_limited() {
            // Arrange
            let config = RateLimitConfig::new(2, 5, 2);
            let rate_limiter = RateLimiter::new(config);
            let filter = TriggerFilter::new()
                .with_min_severity(Severity::Info)
                .with_rate_limiter(rate_limiter);

            let event = create_api_event();

            // Act & Assert
            assert!(filter.should_trigger(&event)); // 1st
            assert!(filter.should_trigger(&event)); // 2nd
            assert!(!filter.should_trigger(&event)); // 3rd - blocked
        }

        #[test]
        fn should_map_branch_events_to_branch_creation_limit() {
            // Arrange
            let config = RateLimitConfig::new(10, 2, 10); // Low branch limit
            let rate_limiter = RateLimiter::new(config);
            let filter = TriggerFilter::new()
                .with_min_severity(Severity::Info)
                .with_rate_limiter(rate_limiter);

            let branch_event = create_branch_event();

            // Act & Assert
            assert!(filter.should_trigger(&branch_event)); // 1st
            assert!(filter.should_trigger(&branch_event)); // 2nd
            assert!(!filter.should_trigger(&branch_event)); // 3rd - blocked
        }

        #[test]
        fn should_map_pr_events_to_pr_creation_limit() {
            // Arrange
            let config = RateLimitConfig::new(10, 10, 1); // Low PR limit
            let rate_limiter = RateLimiter::new(config);
            let filter = TriggerFilter::new()
                .with_min_severity(Severity::Info)
                .with_rate_limiter(rate_limiter);

            let pr_event = create_pr_event();

            // Act & Assert
            assert!(filter.should_trigger(&pr_event)); // 1st
            assert!(!filter.should_trigger(&pr_event)); // 2nd - blocked
        }

        #[test]
        fn should_use_check_rate_limit_method() {
            // Arrange
            let config = RateLimitConfig::new(2, 5, 2);
            let rate_limiter = RateLimiter::new(config);
            let filter = TriggerFilter::new()
                .with_min_severity(Severity::Info)
                .with_rate_limiter(rate_limiter);

            // Act & Assert
            assert!(filter.check_rate_limit(RateLimitAction::ApiCall));
            assert!(filter.check_rate_limit(RateLimitAction::ApiCall));
            assert!(!filter.check_rate_limit(RateLimitAction::ApiCall));
        }

        #[test]
        fn should_return_true_for_check_rate_limit_without_limiter() {
            // Arrange
            let filter = TriggerFilter::new();

            // Act & Assert - Should always allow without rate limiter
            assert!(filter.check_rate_limit(RateLimitAction::ApiCall));
            assert!(filter.check_rate_limit(RateLimitAction::ApiCall));
            assert!(filter.check_rate_limit(RateLimitAction::ApiCall));
        }

        #[test]
        fn should_use_would_be_rate_limited_method() {
            // Arrange
            let config = RateLimitConfig::new(2, 5, 2);
            let rate_limiter = RateLimiter::new(config);
            let filter = TriggerFilter::new()
                .with_min_severity(Severity::Info)
                .with_rate_limiter(rate_limiter);

            // Act - Check without recording
            assert!(!filter.would_be_rate_limited(RateLimitAction::ApiCall));

            // Exhaust the limit
            filter.check_rate_limit(RateLimitAction::ApiCall);
            filter.check_rate_limit(RateLimitAction::ApiCall);

            // Assert
            assert!(filter.would_be_rate_limited(RateLimitAction::ApiCall));
        }

        #[test]
        fn should_access_rate_limiter() {
            // Arrange
            let config = RateLimitConfig::new(5, 10, 3);
            let rate_limiter = RateLimiter::new(config);
            let filter = TriggerFilter::new().with_rate_limiter(rate_limiter);

            // Act
            let limiter = filter.rate_limiter();

            // Assert
            assert!(limiter.is_some());
            assert_eq!(limiter.unwrap().config().api_calls_per_minute, 5);
        }

        #[test]
        fn should_return_none_for_rate_limiter_when_not_configured() {
            // Arrange
            let filter = TriggerFilter::new();

            // Act & Assert
            assert!(filter.rate_limiter().is_none());
        }
    }

    // ============================================
    // TriggerFilterBuilder Rate Limit Tests
    // ============================================

    mod trigger_filter_builder_rate_limit_tests {
        use super::*;

        #[test]
        fn should_build_with_default_rate_limiting() {
            // Arrange & Act
            let filter = TriggerFilterBuilder::new()
                .with_default_rate_limiting()
                .build();

            // Assert
            assert!(filter.rate_limiter().is_some());
            let limiter = filter.rate_limiter().unwrap();
            assert_eq!(limiter.config().api_calls_per_minute, 10);
            assert_eq!(limiter.config().branch_creations_per_hour, 20);
            assert_eq!(limiter.config().pr_creations_per_hour, 10);
        }

        #[test]
        fn should_build_with_custom_rate_limit_config() {
            // Arrange
            let config = RateLimitConfig::new(5, 15, 8);

            // Act
            let filter = TriggerFilterBuilder::new()
                .with_rate_limit_config(config)
                .build();

            // Assert
            assert!(filter.rate_limiter().is_some());
            let limiter = filter.rate_limiter().unwrap();
            assert_eq!(limiter.config().api_calls_per_minute, 5);
            assert_eq!(limiter.config().branch_creations_per_hour, 15);
            assert_eq!(limiter.config().pr_creations_per_hour, 8);
        }

        #[test]
        fn should_build_with_custom_rate_limiter() {
            // Arrange
            let config = RateLimitConfig::new(3, 6, 2);
            let rate_limiter = RateLimiter::new(config);

            // Act
            let filter = TriggerFilterBuilder::new()
                .with_rate_limiter(rate_limiter)
                .build();

            // Assert
            assert!(filter.rate_limiter().is_some());
            let limiter = filter.rate_limiter().unwrap();
            assert_eq!(limiter.config().api_calls_per_minute, 3);
        }

        #[test]
        fn should_combine_filter_settings_with_rate_limiting() {
            // Arrange & Act
            let filter = TriggerFilterBuilder::new()
                .load_from_env()
                .with_default_rate_limiting()
                .build();

            // Assert
            assert!(filter.rate_limiter().is_some());
            assert!(filter.is_active());
        }
    }

    // ============================================
    // RateLimitAction Tests
    // ============================================

    mod rate_limit_action_tests {
        use super::*;

        #[test]
        fn should_have_correct_window_duration_for_api_call() {
            // Arrange & Act
            let duration = RateLimitAction::ApiCall.window_duration();

            // Assert
            assert_eq!(duration, Duration::from_secs(60));
        }

        #[test]
        fn should_have_correct_window_duration_for_branch_creation() {
            // Arrange & Act
            let duration = RateLimitAction::BranchCreation.window_duration();

            // Assert
            assert_eq!(duration, Duration::from_secs(3600));
        }

        #[test]
        fn should_have_correct_window_duration_for_pr_creation() {
            // Arrange & Act
            let duration = RateLimitAction::PrCreation.window_duration();

            // Assert
            assert_eq!(duration, Duration::from_secs(3600));
        }
    }

    // ============================================
    // Event to RateLimitAction Mapping Tests
    // ============================================

    mod event_mapping_tests {
        use super::*;

        #[test]
        fn should_map_branch_event_to_branch_creation() {
            // Arrange
            let event = Event::new(
                "git.branch.create",
                "git-handler",
                Priority::P1,
                serde_json::json!({}),
            );

            // Act
            let action = TriggerFilter::event_to_rate_limit_action(&event);

            // Assert
            assert_eq!(action, RateLimitAction::BranchCreation);
        }

        #[test]
        fn should_map_pr_event_to_pr_creation() {
            // Arrange
            let event = Event::new(
                "github.pull_request.create",
                "github-handler",
                Priority::P1,
                serde_json::json!({}),
            );

            // Act
            let action = TriggerFilter::event_to_rate_limit_action(&event);

            // Assert
            assert_eq!(action, RateLimitAction::PrCreation);
        }

        #[test]
        fn should_map_pr_event_variant_to_pr_creation() {
            // Arrange
            let event = Event::new(
                "github.PR.opened",
                "github-handler",
                Priority::P1,
                serde_json::json!({}),
            );

            // Act
            let action = TriggerFilter::event_to_rate_limit_action(&event);

            // Assert
            assert_eq!(action, RateLimitAction::PrCreation);
        }

        #[test]
        fn should_map_generic_event_to_api_call() {
            // Arrange
            let event = Event::new(
                "monitoring.error_detected",
                "log-watcher",
                Priority::P0,
                serde_json::json!({}),
            );

            // Act
            let action = TriggerFilter::event_to_rate_limit_action(&event);

            // Assert
            assert_eq!(action, RateLimitAction::ApiCall);
        }

        #[test]
        fn should_map_discord_event_to_api_call() {
            // Arrange
            let event = Event::new(
                "discord.command",
                "discord",
                Priority::P1,
                serde_json::json!({}),
            );

            // Act
            let action = TriggerFilter::event_to_rate_limit_action(&event);

            // Assert
            assert_eq!(action, RateLimitAction::ApiCall);
        }

        // ============================================
        // Tests for PR segment matching (bug fix)
        // These tests verify that "pr" is only matched as a distinct segment,
        // not as a substring of other words like "approve", "prepare", etc.
        // ============================================

        #[test]
        fn should_not_match_approve_as_pr_event() {
            // Arrange - "approve" contains "pr" as a substring, but should NOT be a PR event
            let event = Event::new(
                "github.approve",
                "github-handler",
                Priority::P1,
                serde_json::json!({}),
            );

            // Act
            let action = TriggerFilter::event_to_rate_limit_action(&event);

            // Assert
            assert_eq!(action, RateLimitAction::ApiCall);
        }

        #[test]
        fn should_not_match_prepare_as_pr_event() {
            // Arrange - "prepare" contains "pr" as a substring, but should NOT be a PR event
            let event = Event::new(
                "workflow.prepare",
                "workflow-handler",
                Priority::P1,
                serde_json::json!({}),
            );

            // Act
            let action = TriggerFilter::event_to_rate_limit_action(&event);

            // Assert
            assert_eq!(action, RateLimitAction::ApiCall);
        }

        #[test]
        fn should_not_match_profile_as_pr_event() {
            // Arrange - "profile" contains "pr" as a substring, but should NOT be a PR event
            let event = Event::new(
                "user.profile.update",
                "user-handler",
                Priority::P1,
                serde_json::json!({}),
            );

            // Act
            let action = TriggerFilter::event_to_rate_limit_action(&event);

            // Assert
            assert_eq!(action, RateLimitAction::ApiCall);
        }

        #[test]
        fn should_not_match_reprocess_as_pr_event() {
            // Arrange - "reprocess" contains "pr" as a substring, but should NOT be a PR event
            let event = Event::new(
                "job.reprocess",
                "job-handler",
                Priority::P1,
                serde_json::json!({}),
            );

            // Act
            let action = TriggerFilter::event_to_rate_limit_action(&event);

            // Assert
            assert_eq!(action, RateLimitAction::ApiCall);
        }

        #[test]
        fn should_not_match_deprecation_as_pr_event() {
            // Arrange - "deprecation" contains "pr" as a substring, but should NOT be a PR event
            let event = Event::new(
                "api.deprecation.warning",
                "api-handler",
                Priority::P1,
                serde_json::json!({}),
            );

            // Act
            let action = TriggerFilter::event_to_rate_limit_action(&event);

            // Assert
            assert_eq!(action, RateLimitAction::ApiCall);
        }

        #[test]
        fn should_not_match_compress_as_pr_event() {
            // Arrange - "compress" contains "pr" as a substring, but should NOT be a PR event
            let event = Event::new(
                "file.compress",
                "file-handler",
                Priority::P1,
                serde_json::json!({}),
            );

            // Act
            let action = TriggerFilter::event_to_rate_limit_action(&event);

            // Assert
            assert_eq!(action, RateLimitAction::ApiCall);
        }

        #[test]
        fn should_match_pr_segment_at_start() {
            // Arrange - "pr.created" starts with "pr."
            let event = Event::new(
                "pr.created",
                "github-handler",
                Priority::P1,
                serde_json::json!({}),
            );

            // Act
            let action = TriggerFilter::event_to_rate_limit_action(&event);

            // Assert
            assert_eq!(action, RateLimitAction::PrCreation);
        }

        #[test]
        fn should_match_pr_segment_at_end() {
            // Arrange - "github.pr" ends with ".pr"
            let event = Event::new(
                "github.pr",
                "github-handler",
                Priority::P1,
                serde_json::json!({}),
            );

            // Act
            let action = TriggerFilter::event_to_rate_limit_action(&event);

            // Assert
            assert_eq!(action, RateLimitAction::PrCreation);
        }

        #[test]
        fn should_match_pr_segment_in_middle() {
            // Arrange - "github.pr.created" contains ".pr."
            let event = Event::new(
                "github.pr.created",
                "github-handler",
                Priority::P1,
                serde_json::json!({}),
            );

            // Act
            let action = TriggerFilter::event_to_rate_limit_action(&event);

            // Assert
            assert_eq!(action, RateLimitAction::PrCreation);
        }

        #[test]
        fn should_match_exact_pr_event_type() {
            // Arrange - exactly "pr"
            let event = Event::new("pr", "github-handler", Priority::P1, serde_json::json!({}));

            // Act
            let action = TriggerFilter::event_to_rate_limit_action(&event);

            // Assert
            assert_eq!(action, RateLimitAction::PrCreation);
        }

        #[test]
        fn should_match_pr_case_insensitive() {
            // Arrange - uppercase "PR" should also match as a segment
            let event = Event::new(
                "github.PR.opened",
                "github-handler",
                Priority::P1,
                serde_json::json!({}),
            );

            // Act
            let action = TriggerFilter::event_to_rate_limit_action(&event);

            // Assert
            assert_eq!(action, RateLimitAction::PrCreation);
        }

        #[test]
        fn should_match_branch_case_insensitive() {
            // Arrange - uppercase "Branch" should match
            let event = Event::new(
                "git.Branch.create",
                "git-handler",
                Priority::P1,
                serde_json::json!({}),
            );

            // Act
            let action = TriggerFilter::event_to_rate_limit_action(&event);

            // Assert
            assert_eq!(action, RateLimitAction::BranchCreation);
        }

        #[test]
        fn should_match_pull_request_case_insensitive() {
            // Arrange - mixed case "Pull_Request" should match
            let event = Event::new(
                "github.Pull_Request.create",
                "github-handler",
                Priority::P1,
                serde_json::json!({}),
            );

            // Act
            let action = TriggerFilter::event_to_rate_limit_action(&event);

            // Assert
            assert_eq!(action, RateLimitAction::PrCreation);
        }
    }
}
