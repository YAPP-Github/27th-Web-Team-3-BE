//! Log file watcher for monitoring pipeline (Phase 2 MVP)
//!
//! This module watches server log files and generates events for error detection:
//! - Watches `logs/server.YYYY-MM-DD.log` files
//! - Parses JSON log lines
//! - Deduplicates errors within a configurable time window
//! - Creates events for the automation pipeline

#![allow(dead_code)]

use crate::event::{Event, EventMetadata, Priority, Severity};
use crate::utils::AppError;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use tracing::{debug, error, info, warn};

/// Default deduplication window in seconds (5 minutes)
const DEFAULT_DEDUP_WINDOW_SECS: u64 = 300;

/// Log watcher result type
pub type LogWatcherResult<T> = Result<T, AppError>;

/// Parsed log entry from JSON log file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    /// Log level (ERROR, WARN, INFO, etc.)
    pub level: String,
    /// Error code if present (e.g., "AI5001", "AUTH4001")
    pub error_code: Option<String>,
    /// Log message
    pub message: String,
    /// Log target (e.g., "server::domain::ai::service")
    pub target: String,
    /// Request ID for tracing
    pub request_id: Option<String>,
    /// Log timestamp
    pub timestamp: DateTime<Utc>,
}

impl LogEntry {
    /// Generate fingerprint for deduplication
    /// Format: ERROR_CODE:TARGET or UNKNOWN:TARGET if no error code
    pub fn fingerprint(&self) -> String {
        let error_code = self.error_code.as_deref().unwrap_or("UNKNOWN");
        format!("{}:{}", error_code, self.target)
    }

    /// Determine severity based on error code
    pub fn severity(&self) -> Severity {
        match self.error_code.as_deref() {
            // Critical: AI errors (5xxx)
            Some(code) if code.starts_with("AI5") => Severity::Critical,
            // Warning: Auth errors (4xxx), Retro errors
            Some(code) if code.starts_with("AUTH4") => Severity::Warning,
            Some(code) if code.starts_with("RETRO4") => Severity::Warning,
            // Info: Other errors
            _ => Severity::Info,
        }
    }

    /// Check if this is an error log
    pub fn is_error(&self) -> bool {
        self.level.to_uppercase() == "ERROR"
    }

    /// Check if this is a warning log
    pub fn is_warning(&self) -> bool {
        self.level.to_uppercase() == "WARN" || self.level.to_uppercase() == "WARNING"
    }
}

/// Internal structure for JSON log parsing
#[derive(Debug, Deserialize)]
struct RawLogEntry {
    timestamp: String,
    level: String,
    target: String,
    fields: Option<LogFields>,
    message: String,
}

/// Log fields structure
#[derive(Debug, Deserialize)]
struct LogFields {
    error_code: Option<String>,
    request_id: Option<String>,
}

/// Deduplication entry with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DedupEntry {
    fingerprint: String,
    first_seen: DateTime<Utc>,
    count: u32,
}

/// Log watcher configuration
#[derive(Debug, Clone)]
pub struct LogWatcherConfig {
    /// Directory containing log files
    pub log_dir: PathBuf,
    /// Directory for state files
    pub state_dir: PathBuf,
    /// Deduplication window in seconds
    pub dedup_window_secs: u64,
}

impl Default for LogWatcherConfig {
    fn default() -> Self {
        Self {
            log_dir: PathBuf::from("logs"),
            state_dir: PathBuf::from("logs/.state"),
            dedup_window_secs: DEFAULT_DEDUP_WINDOW_SECS,
        }
    }
}

/// Log file watcher for monitoring pipeline
pub struct LogWatcher {
    /// Directory containing log files
    log_dir: PathBuf,
    /// Directory for state files
    state_dir: PathBuf,
    /// Deduplication window in seconds
    dedup_window_secs: u64,
    /// In-memory deduplication cache
    dedup_cache: HashMap<String, DateTime<Utc>>,
}

impl LogWatcher {
    /// Create a new log watcher with specified directories
    pub fn new(log_dir: PathBuf, state_dir: PathBuf) -> LogWatcherResult<Self> {
        Self::with_config(LogWatcherConfig {
            log_dir,
            state_dir,
            dedup_window_secs: DEFAULT_DEDUP_WINDOW_SECS,
        })
    }

    /// Create a new log watcher with custom configuration
    pub fn with_config(config: LogWatcherConfig) -> LogWatcherResult<Self> {
        // Create state directory if it doesn't exist
        fs::create_dir_all(&config.state_dir).map_err(|e| {
            error!(error = %e, dir = %config.state_dir.display(), "Failed to create state directory");
            AppError::InternalError(format!("Failed to create state directory: {}", e))
        })?;

        info!(
            log_dir = %config.log_dir.display(),
            state_dir = %config.state_dir.display(),
            dedup_window_secs = %config.dedup_window_secs,
            "LogWatcher initialized"
        );

        Ok(Self {
            log_dir: config.log_dir,
            state_dir: config.state_dir,
            dedup_window_secs: config.dedup_window_secs,
            dedup_cache: HashMap::new(),
        })
    }

    /// Watch for new log entries and generate events
    /// Returns a list of events for new, non-duplicate errors
    pub fn watch(&mut self) -> LogWatcherResult<Vec<Event>> {
        let today = Utc::now().date_naive();
        let log_file = self.get_log_file_path(today);

        if !log_file.exists() {
            debug!(log_file = %log_file.display(), "Log file does not exist yet");
            return Ok(vec![]);
        }

        // Load state and dedup cache for today (best-effort, won't fail on read errors)
        self.load_dedup_cache(today);

        // Get last read position
        let mut last_line = self.get_last_line_number(today)?;

        // Check for log rotation: if current file line count is less than last_line,
        // the file was truncated/rotated, so reset to read from the beginning
        let current_line_count = self.count_file_lines(&log_file)?;
        if current_line_count < last_line {
            info!(
                previous_line = last_line,
                current_lines = current_line_count,
                "Log file appears to have been rotated/truncated, resetting line counter"
            );
            last_line = 0;
            // Also reset the saved state file
            self.save_last_line_number(today, 0)?;
        }

        // Read new lines
        let entries = self.read_new_entries(&log_file, last_line)?;

        if entries.is_empty() {
            return Ok(vec![]);
        }

        // Filter for ERROR level and deduplicate
        let mut events = Vec::new();
        let mut new_line_count = 0;

        for (line_num, entry) in entries {
            new_line_count = line_num;

            // Only process ERROR level logs (case-insensitive)
            if !entry.is_error() {
                continue;
            }

            // Check deduplication
            if !self.should_alert(&entry) {
                debug!(
                    fingerprint = %entry.fingerprint(),
                    "Skipping duplicate error within dedup window"
                );
                continue;
            }

            // Create event
            let event = self.create_event(&entry);
            events.push(event);

            // Update dedup cache
            self.update_dedup_cache(&entry);
        }

        // Save state
        if new_line_count > last_line {
            self.save_last_line_number(today, new_line_count)?;
        }
        self.save_dedup_cache(today)?;

        if !events.is_empty() {
            info!(count = events.len(), "Generated events from log entries");
        }

        Ok(events)
    }

    /// Parse a JSON log line into a LogEntry
    pub fn parse_log_entry(line: &str) -> Option<LogEntry> {
        let raw: RawLogEntry = serde_json::from_str(line).ok()?;

        let timestamp = DateTime::parse_from_rfc3339(&raw.timestamp)
            .ok()?
            .with_timezone(&Utc);

        let (error_code, request_id) = match raw.fields {
            Some(fields) => (fields.error_code, fields.request_id),
            None => (None, None),
        };

        Some(LogEntry {
            level: raw.level,
            error_code,
            message: raw.message,
            target: raw.target,
            request_id,
            timestamp,
        })
    }

    /// Check if an entry should trigger an alert (not a duplicate)
    pub fn should_alert(&self, entry: &LogEntry) -> bool {
        let fingerprint = entry.fingerprint();
        let now = Utc::now();

        match self.dedup_cache.get(&fingerprint) {
            Some(first_seen) => {
                let elapsed = (now - *first_seen).num_seconds() as u64;
                elapsed >= self.dedup_window_secs
            }
            None => true,
        }
    }

    /// Create an Event from a LogEntry
    ///
    /// Event data uses snake_case keys for consistency with Rust conventions
    /// and compatibility with downstream processors (processor.rs, discord_alert.rs, trigger.rs)
    pub fn create_event(&self, entry: &LogEntry) -> Event {
        let severity = entry.severity();
        let priority = match severity {
            Severity::Critical => Priority::P0,
            Severity::Warning => Priority::P1,
            Severity::Info => Priority::P2,
        };

        // Use snake_case keys for consistency with downstream processors
        let data = serde_json::json!({
            "error_code": entry.error_code,
            "severity": format!("{:?}", severity).to_lowercase(),
            "message": entry.message,
            "target": entry.target,
            "request_id": entry.request_id,
            "log_line": serde_json::to_string(entry).unwrap_or_default(),
            "timestamp": entry.timestamp.to_rfc3339(),
        });

        let fingerprint = entry.fingerprint();
        let metadata = EventMetadata::new(&fingerprint);

        Event::new("monitoring.error_detected", "log-watcher", priority, data)
            .with_metadata(metadata)
    }

    /// Get the log file path for a specific date
    fn get_log_file_path(&self, date: NaiveDate) -> PathBuf {
        let filename = format!("server.{}.log", date.format("%Y-%m-%d"));
        self.log_dir.join(filename)
    }

    /// Get the state file path for a specific date
    fn get_state_file_path(&self, date: NaiveDate) -> PathBuf {
        let filename = format!("log-watcher-state-{}", date.format("%Y-%m-%d"));
        self.state_dir.join(filename)
    }

    /// Get the dedup file path for a specific date
    fn get_dedup_file_path(&self, date: NaiveDate) -> PathBuf {
        let filename = format!("log-watcher-dedup-{}", date.format("%Y-%m-%d"));
        self.state_dir.join(filename)
    }

    /// Get the last read line number for a specific date
    fn get_last_line_number(&self, date: NaiveDate) -> LogWatcherResult<usize> {
        let state_file = self.get_state_file_path(date);

        if !state_file.exists() {
            return Ok(0);
        }

        let content = fs::read_to_string(&state_file).map_err(|e| {
            error!(error = %e, file = %state_file.display(), "Failed to read state file");
            AppError::InternalError(format!("Failed to read state file: {}", e))
        })?;

        content.trim().parse().map_err(|e| {
            error!(error = %e, content = %content, "Failed to parse line number");
            AppError::InternalError(format!("Failed to parse line number: {}", e))
        })
    }

    /// Save the last read line number for a specific date
    fn save_last_line_number(&self, date: NaiveDate, line_number: usize) -> LogWatcherResult<()> {
        let state_file = self.get_state_file_path(date);

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&state_file)
            .map_err(|e| {
                error!(error = %e, file = %state_file.display(), "Failed to open state file");
                AppError::InternalError(format!("Failed to open state file: {}", e))
            })?;

        write!(file, "{}", line_number).map_err(|e| {
            error!(error = %e, "Failed to write line number");
            AppError::InternalError(format!("Failed to write line number: {}", e))
        })?;

        debug!(line_number = line_number, "Saved last line number");
        Ok(())
    }

    /// Count the number of lines in a file
    fn count_file_lines(&self, log_file: &PathBuf) -> LogWatcherResult<usize> {
        let file = File::open(log_file).map_err(|e| {
            error!(error = %e, file = %log_file.display(), "Failed to open log file for line count");
            AppError::InternalError(format!("Failed to open log file: {}", e))
        })?;

        let reader = BufReader::new(file);
        Ok(reader.lines().count())
    }

    /// Load deduplication cache from file
    /// Uses best-effort approach: if read fails, logs warning and starts with empty cache
    fn load_dedup_cache(&mut self, date: NaiveDate) {
        let dedup_file = self.get_dedup_file_path(date);

        if !dedup_file.exists() {
            self.dedup_cache.clear();
            return;
        }

        let content = match fs::read_to_string(&dedup_file) {
            Ok(c) => c,
            Err(e) => {
                warn!(error = %e, file = %dedup_file.display(), "Failed to read dedup file, starting with fresh cache");
                self.dedup_cache.clear();
                return;
            }
        };

        self.dedup_cache.clear();
        let now = Utc::now();

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            if let Ok(entry) = serde_json::from_str::<DedupEntry>(line) {
                // Only keep entries within the dedup window
                let elapsed = (now - entry.first_seen).num_seconds() as u64;
                if elapsed < self.dedup_window_secs {
                    self.dedup_cache.insert(entry.fingerprint, entry.first_seen);
                }
            }
        }

        debug!(entries = self.dedup_cache.len(), "Loaded dedup cache");
    }

    /// Save deduplication cache to file
    fn save_dedup_cache(&self, date: NaiveDate) -> LogWatcherResult<()> {
        let dedup_file = self.get_dedup_file_path(date);
        let now = Utc::now();

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&dedup_file)
            .map_err(|e| {
                error!(error = %e, file = %dedup_file.display(), "Failed to open dedup file");
                AppError::InternalError(format!("Failed to open dedup file: {}", e))
            })?;

        for (fingerprint, first_seen) in &self.dedup_cache {
            // Only save entries within the dedup window
            let elapsed = (now - *first_seen).num_seconds() as u64;
            if elapsed < self.dedup_window_secs {
                let entry = DedupEntry {
                    fingerprint: fingerprint.clone(),
                    first_seen: *first_seen,
                    count: 1,
                };
                let json = serde_json::to_string(&entry).map_err(|e| {
                    error!(error = %e, "Failed to serialize dedup entry");
                    AppError::InternalError(format!("Failed to serialize dedup entry: {}", e))
                })?;
                writeln!(file, "{}", json).map_err(|e| {
                    error!(error = %e, "Failed to write dedup entry");
                    AppError::InternalError(format!("Failed to write dedup entry: {}", e))
                })?;
            }
        }

        debug!(entries = self.dedup_cache.len(), "Saved dedup cache");
        Ok(())
    }

    /// Update deduplication cache with a new entry
    fn update_dedup_cache(&mut self, entry: &LogEntry) {
        let fingerprint = entry.fingerprint();
        self.dedup_cache.insert(fingerprint, Utc::now());
    }

    /// Read new log entries from file starting after the given line number
    fn read_new_entries(
        &self,
        log_file: &PathBuf,
        after_line: usize,
    ) -> LogWatcherResult<Vec<(usize, LogEntry)>> {
        let file = File::open(log_file).map_err(|e| {
            error!(error = %e, file = %log_file.display(), "Failed to open log file");
            AppError::InternalError(format!("Failed to open log file: {}", e))
        })?;

        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (idx, line_result) in reader.lines().enumerate() {
            let line_num = idx + 1;

            if line_num <= after_line {
                continue;
            }

            let line = line_result.map_err(|e| {
                error!(error = %e, line = line_num, "Failed to read line");
                AppError::InternalError(format!("Failed to read line {}: {}", line_num, e))
            })?;

            if line.trim().is_empty() {
                continue;
            }

            if let Some(entry) = Self::parse_log_entry(&line) {
                entries.push((line_num, entry));
            } else {
                warn!(line = line_num, "Failed to parse log line, skipping");
            }
        }

        debug!(
            new_entries = entries.len(),
            after_line = after_line,
            "Read new log entries"
        );
        Ok(entries)
    }

    /// Clean up old state files (older than specified days)
    pub fn cleanup_old_state_files(&self, days_to_keep: i64) -> LogWatcherResult<()> {
        let cutoff = Utc::now()
            .date_naive()
            .checked_sub_days(chrono::Days::new(days_to_keep as u64))
            .ok_or_else(|| AppError::InternalError("Invalid days calculation".to_string()))?;

        let entries = fs::read_dir(&self.state_dir).map_err(|e| {
            error!(error = %e, "Failed to read state directory");
            AppError::InternalError(format!("Failed to read state directory: {}", e))
        })?;

        let mut removed = 0;
        for entry in entries.flatten() {
            let filename = entry.file_name().to_string_lossy().to_string();

            // Parse date from filename (format: log-watcher-{state|dedup}-YYYY-MM-DD)
            if let Some(date_str) = filename
                .strip_prefix("log-watcher-state-")
                .or_else(|| filename.strip_prefix("log-watcher-dedup-"))
            {
                if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                    if date < cutoff && fs::remove_file(entry.path()).is_ok() {
                        removed += 1;
                    }
                }
            }
        }

        if removed > 0 {
            info!(removed = removed, "Cleaned up old state files");
        }

        Ok(())
    }

    /// Get the log directory path
    pub fn log_dir(&self) -> &PathBuf {
        &self.log_dir
    }

    /// Get the state directory path
    pub fn state_dir(&self) -> &PathBuf {
        &self.state_dir
    }

    /// Get the dedup window in seconds
    pub fn dedup_window_secs(&self) -> u64 {
        self.dedup_window_secs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    use uuid::Uuid;

    fn create_test_watcher() -> LogWatcher {
        let test_dir = temp_dir().join(format!("test_log_watcher_{}", Uuid::new_v4()));
        let log_dir = test_dir.join("logs");
        let state_dir = test_dir.join("state");

        fs::create_dir_all(&log_dir).expect("Failed to create log dir");

        LogWatcher::new(log_dir, state_dir).expect("Failed to create watcher")
    }

    fn create_test_log_file(log_dir: &PathBuf, date: NaiveDate, content: &str) -> PathBuf {
        let filename = format!("server.{}.log", date.format("%Y-%m-%d"));
        let log_file = log_dir.join(filename);
        fs::write(&log_file, content).expect("Failed to write log file");
        log_file
    }

    #[test]
    fn should_parse_valid_json_log_line() {
        // Arrange
        let line = r#"{"timestamp":"2026-01-31T14:50:00Z","level":"ERROR","target":"server::domain::ai::service","fields":{"error_code":"AI5002","request_id":"test-001"},"message":"OpenAI API connection failed"}"#;

        // Act
        let entry = LogWatcher::parse_log_entry(line);

        // Assert
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.level, "ERROR");
        assert_eq!(entry.error_code, Some("AI5002".to_string()));
        assert_eq!(entry.target, "server::domain::ai::service");
        assert_eq!(entry.request_id, Some("test-001".to_string()));
        assert_eq!(entry.message, "OpenAI API connection failed");
    }

    #[test]
    fn should_return_none_for_invalid_json() {
        // Arrange
        let line = "this is not json";

        // Act
        let entry = LogWatcher::parse_log_entry(line);

        // Assert
        assert!(entry.is_none());
    }

    #[test]
    fn should_parse_log_without_fields() {
        // Arrange
        let line = r#"{"timestamp":"2026-01-31T14:50:00Z","level":"INFO","target":"server","message":"Server started"}"#;

        // Act
        let entry = LogWatcher::parse_log_entry(line);

        // Assert
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.level, "INFO");
        assert!(entry.error_code.is_none());
        assert!(entry.request_id.is_none());
    }

    #[test]
    fn should_generate_correct_fingerprint() {
        // Arrange
        let entry = LogEntry {
            level: "ERROR".to_string(),
            error_code: Some("AI5002".to_string()),
            message: "Test message".to_string(),
            target: "server::domain::ai".to_string(),
            request_id: Some("req-123".to_string()),
            timestamp: Utc::now(),
        };

        // Act
        let fingerprint = entry.fingerprint();

        // Assert
        assert_eq!(fingerprint, "AI5002:server::domain::ai");
    }

    #[test]
    fn should_generate_fingerprint_without_error_code() {
        // Arrange
        let entry = LogEntry {
            level: "ERROR".to_string(),
            error_code: None,
            message: "Test message".to_string(),
            target: "server::domain::ai".to_string(),
            request_id: None,
            timestamp: Utc::now(),
        };

        // Act
        let fingerprint = entry.fingerprint();

        // Assert
        assert_eq!(fingerprint, "UNKNOWN:server::domain::ai");
    }

    #[test]
    fn should_determine_critical_severity_for_ai_errors() {
        // Arrange
        let entry = LogEntry {
            level: "ERROR".to_string(),
            error_code: Some("AI5001".to_string()),
            message: "AI analysis failed".to_string(),
            target: "server::domain::ai".to_string(),
            request_id: None,
            timestamp: Utc::now(),
        };

        // Act
        let severity = entry.severity();

        // Assert
        assert_eq!(severity, Severity::Critical);
    }

    #[test]
    fn should_determine_warning_severity_for_auth_errors() {
        // Arrange
        let entry = LogEntry {
            level: "ERROR".to_string(),
            error_code: Some("AUTH4001".to_string()),
            message: "Authentication failed".to_string(),
            target: "server::domain::auth".to_string(),
            request_id: None,
            timestamp: Utc::now(),
        };

        // Act
        let severity = entry.severity();

        // Assert
        assert_eq!(severity, Severity::Warning);
    }

    #[test]
    fn should_determine_warning_severity_for_retro_errors() {
        // Arrange
        let entry = LogEntry {
            level: "ERROR".to_string(),
            error_code: Some("RETRO4041".to_string()),
            message: "Retro not found".to_string(),
            target: "server::domain::retrospect".to_string(),
            request_id: None,
            timestamp: Utc::now(),
        };

        // Act
        let severity = entry.severity();

        // Assert
        assert_eq!(severity, Severity::Warning);
    }

    #[test]
    fn should_determine_info_severity_for_unknown_errors() {
        // Arrange
        let entry = LogEntry {
            level: "ERROR".to_string(),
            error_code: Some("UNKNOWN001".to_string()),
            message: "Unknown error".to_string(),
            target: "server".to_string(),
            request_id: None,
            timestamp: Utc::now(),
        };

        // Act
        let severity = entry.severity();

        // Assert
        assert_eq!(severity, Severity::Info);
    }

    #[test]
    fn should_create_event_with_correct_priority() {
        // Arrange
        let watcher = create_test_watcher();
        let entry = LogEntry {
            level: "ERROR".to_string(),
            error_code: Some("AI5002".to_string()),
            message: "OpenAI API timeout".to_string(),
            target: "server::domain::ai::service".to_string(),
            request_id: Some("req-123".to_string()),
            timestamp: Utc::now(),
        };

        // Act
        let event = watcher.create_event(&entry);

        // Assert
        assert_eq!(event.event_type, "monitoring.error_detected");
        assert_eq!(event.source, "log-watcher");
        assert_eq!(event.priority, Priority::P0);
        assert_eq!(
            event.metadata.fingerprint,
            "AI5002:server::domain::ai::service"
        );
    }

    #[test]
    fn should_create_event_with_snake_case_keys() {
        // Arrange
        let watcher = create_test_watcher();
        let entry = LogEntry {
            level: "ERROR".to_string(),
            error_code: Some("AI5002".to_string()),
            message: "OpenAI API timeout".to_string(),
            target: "server::domain::ai::service".to_string(),
            request_id: Some("req-123".to_string()),
            timestamp: Utc::now(),
        };

        // Act
        let event = watcher.create_event(&entry);

        // Assert - Verify snake_case keys are used
        assert!(event.data.get("error_code").is_some());
        assert!(event.data.get("request_id").is_some());
        assert!(event.data.get("log_line").is_some());
        // Verify camelCase keys are NOT used
        assert!(event.data.get("errorCode").is_none());
        assert!(event.data.get("requestId").is_none());
        assert!(event.data.get("logLine").is_none());
    }

    #[test]
    fn should_create_event_with_p1_priority_for_warning() {
        // Arrange
        let watcher = create_test_watcher();
        let entry = LogEntry {
            level: "ERROR".to_string(),
            error_code: Some("AUTH4001".to_string()),
            message: "Auth failed".to_string(),
            target: "server::domain::auth".to_string(),
            request_id: None,
            timestamp: Utc::now(),
        };

        // Act
        let event = watcher.create_event(&entry);

        // Assert
        assert_eq!(event.priority, Priority::P1);
    }

    #[test]
    fn should_alert_for_new_fingerprint() {
        // Arrange
        let watcher = create_test_watcher();
        let entry = LogEntry {
            level: "ERROR".to_string(),
            error_code: Some("AI5002".to_string()),
            message: "Test error".to_string(),
            target: "server::domain::ai".to_string(),
            request_id: None,
            timestamp: Utc::now(),
        };

        // Act
        let should_alert = watcher.should_alert(&entry);

        // Assert
        assert!(should_alert);
    }

    #[test]
    fn should_not_alert_for_duplicate_within_window() {
        // Arrange
        let mut watcher = create_test_watcher();
        let entry = LogEntry {
            level: "ERROR".to_string(),
            error_code: Some("AI5002".to_string()),
            message: "Test error".to_string(),
            target: "server::domain::ai".to_string(),
            request_id: None,
            timestamp: Utc::now(),
        };

        // First alert
        watcher.update_dedup_cache(&entry);

        // Act
        let should_alert = watcher.should_alert(&entry);

        // Assert
        assert!(!should_alert);
    }

    #[test]
    fn should_read_new_entries_from_log_file() {
        // Arrange
        let test_dir = temp_dir().join(format!("test_read_entries_{}", Uuid::new_v4()));
        let log_dir = test_dir.join("logs");
        let state_dir = test_dir.join("state");
        fs::create_dir_all(&log_dir).expect("Failed to create log dir");

        let watcher =
            LogWatcher::new(log_dir.clone(), state_dir).expect("Failed to create watcher");
        let today = Utc::now().date_naive();

        let content = r#"{"timestamp":"2026-01-31T14:50:00Z","level":"ERROR","target":"server::ai","fields":{"error_code":"AI5002"},"message":"Error 1"}
{"timestamp":"2026-01-31T14:51:00Z","level":"INFO","target":"server","message":"Info message"}
{"timestamp":"2026-01-31T14:52:00Z","level":"ERROR","target":"server::auth","fields":{"error_code":"AUTH4001"},"message":"Error 2"}"#;

        let log_file = create_test_log_file(&log_dir, today, content);

        // Act
        let entries = watcher
            .read_new_entries(&log_file, 0)
            .expect("Failed to read entries");

        // Assert
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].0, 1); // Line 1
        assert_eq!(entries[0].1.level, "ERROR");
        assert_eq!(entries[1].0, 2); // Line 2
        assert_eq!(entries[1].1.level, "INFO");
        assert_eq!(entries[2].0, 3); // Line 3
        assert_eq!(entries[2].1.level, "ERROR");
    }

    #[test]
    fn should_skip_lines_before_after_line() {
        // Arrange
        let test_dir = temp_dir().join(format!("test_skip_lines_{}", Uuid::new_v4()));
        let log_dir = test_dir.join("logs");
        let state_dir = test_dir.join("state");
        fs::create_dir_all(&log_dir).expect("Failed to create log dir");

        let watcher =
            LogWatcher::new(log_dir.clone(), state_dir).expect("Failed to create watcher");
        let today = Utc::now().date_naive();

        let content = r#"{"timestamp":"2026-01-31T14:50:00Z","level":"ERROR","target":"server","message":"Old error"}
{"timestamp":"2026-01-31T14:51:00Z","level":"ERROR","target":"server","message":"New error"}"#;

        let log_file = create_test_log_file(&log_dir, today, content);

        // Act - skip first line
        let entries = watcher
            .read_new_entries(&log_file, 1)
            .expect("Failed to read entries");

        // Assert
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, 2); // Line 2
        assert_eq!(entries[0].1.message, "New error");
    }

    #[test]
    fn should_watch_and_generate_events() {
        // Arrange
        let test_dir = temp_dir().join(format!("test_watch_{}", Uuid::new_v4()));
        let log_dir = test_dir.join("logs");
        let state_dir = test_dir.join("state");
        fs::create_dir_all(&log_dir).expect("Failed to create log dir");

        let mut watcher =
            LogWatcher::new(log_dir.clone(), state_dir).expect("Failed to create watcher");
        let today = Utc::now().date_naive();

        let content = r#"{"timestamp":"2026-01-31T14:50:00Z","level":"ERROR","target":"server::ai","fields":{"error_code":"AI5002"},"message":"Critical error"}
{"timestamp":"2026-01-31T14:51:00Z","level":"INFO","target":"server","message":"Info message"}
{"timestamp":"2026-01-31T14:52:00Z","level":"ERROR","target":"server::auth","fields":{"error_code":"AUTH4001"},"message":"Auth error"}"#;

        create_test_log_file(&log_dir, today, content);

        // Act
        let events = watcher.watch().expect("Failed to watch");

        // Assert - only ERROR level entries generate events
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].priority, Priority::P0); // AI5002 is critical
        assert_eq!(events[1].priority, Priority::P1); // AUTH4001 is warning
    }

    #[test]
    fn should_deduplicate_errors_within_window() {
        // Arrange
        let test_dir = temp_dir().join(format!("test_dedup_{}", Uuid::new_v4()));
        let log_dir = test_dir.join("logs");
        let state_dir = test_dir.join("state");
        fs::create_dir_all(&log_dir).expect("Failed to create log dir");

        let mut watcher =
            LogWatcher::new(log_dir.clone(), state_dir).expect("Failed to create watcher");
        let today = Utc::now().date_naive();

        // Same error code and target = same fingerprint
        let content = r#"{"timestamp":"2026-01-31T14:50:00Z","level":"ERROR","target":"server::ai","fields":{"error_code":"AI5002"},"message":"Error 1"}
{"timestamp":"2026-01-31T14:50:01Z","level":"ERROR","target":"server::ai","fields":{"error_code":"AI5002"},"message":"Error 2"}"#;

        create_test_log_file(&log_dir, today, content);

        // Act
        let events = watcher.watch().expect("Failed to watch");

        // Assert - only first error generates an event
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data["message"], "Error 1");
    }

    #[test]
    fn should_save_and_load_state() {
        // Arrange
        let test_dir = temp_dir().join(format!("test_state_{}", Uuid::new_v4()));
        let log_dir = test_dir.join("logs");
        let state_dir = test_dir.join("state");
        fs::create_dir_all(&log_dir).expect("Failed to create log dir");

        let mut watcher =
            LogWatcher::new(log_dir.clone(), state_dir.clone()).expect("Failed to create watcher");
        let today = Utc::now().date_naive();

        let content = r#"{"timestamp":"2026-01-31T14:50:00Z","level":"ERROR","target":"server::ai","fields":{"error_code":"AI5002"},"message":"Error 1"}"#;
        create_test_log_file(&log_dir, today, content);

        // First watch
        let events1 = watcher.watch().expect("Failed to watch");
        assert_eq!(events1.len(), 1);

        // Create new watcher (simulating restart)
        let mut watcher2 = LogWatcher::new(log_dir, state_dir).expect("Failed to create watcher");

        // Act - second watch should not re-read old lines
        let events2 = watcher2.watch().expect("Failed to watch");

        // Assert
        assert_eq!(events2.len(), 0);
    }

    #[test]
    fn should_return_empty_when_no_log_file() {
        // Arrange
        let test_dir = temp_dir().join(format!("test_no_log_{}", Uuid::new_v4()));
        let log_dir = test_dir.join("logs");
        let state_dir = test_dir.join("state");
        fs::create_dir_all(&log_dir).expect("Failed to create log dir");

        let mut watcher = LogWatcher::new(log_dir, state_dir).expect("Failed to create watcher");

        // Act
        let events = watcher.watch().expect("Failed to watch");

        // Assert
        assert!(events.is_empty());
    }

    #[test]
    fn should_serialize_log_entry_to_camel_case() {
        // Arrange
        let entry = LogEntry {
            level: "ERROR".to_string(),
            error_code: Some("AI5002".to_string()),
            message: "Test error".to_string(),
            target: "server::ai".to_string(),
            request_id: Some("req-123".to_string()),
            timestamp: Utc::now(),
        };

        // Act
        let json = serde_json::to_string(&entry).expect("Failed to serialize");

        // Assert
        assert!(json.contains("errorCode"));
        assert!(json.contains("requestId"));
        assert!(!json.contains("error_code"));
        assert!(!json.contains("request_id"));
    }

    #[test]
    fn should_check_is_error_correctly() {
        // Arrange
        let error_entry = LogEntry {
            level: "ERROR".to_string(),
            error_code: None,
            message: "test".to_string(),
            target: "test".to_string(),
            request_id: None,
            timestamp: Utc::now(),
        };

        let info_entry = LogEntry {
            level: "INFO".to_string(),
            error_code: None,
            message: "test".to_string(),
            target: "test".to_string(),
            request_id: None,
            timestamp: Utc::now(),
        };

        // Act & Assert
        assert!(error_entry.is_error());
        assert!(!info_entry.is_error());
    }

    #[test]
    fn should_check_is_error_case_insensitive() {
        // Arrange
        let lowercase_error = LogEntry {
            level: "error".to_string(),
            error_code: None,
            message: "test".to_string(),
            target: "test".to_string(),
            request_id: None,
            timestamp: Utc::now(),
        };

        let mixed_case_error = LogEntry {
            level: "Error".to_string(),
            error_code: None,
            message: "test".to_string(),
            target: "test".to_string(),
            request_id: None,
            timestamp: Utc::now(),
        };

        // Act & Assert
        assert!(lowercase_error.is_error());
        assert!(mixed_case_error.is_error());
    }

    #[test]
    fn should_check_is_warning_correctly() {
        // Arrange
        let warn_entry = LogEntry {
            level: "WARN".to_string(),
            error_code: None,
            message: "test".to_string(),
            target: "test".to_string(),
            request_id: None,
            timestamp: Utc::now(),
        };

        let warning_entry = LogEntry {
            level: "WARNING".to_string(),
            error_code: None,
            message: "test".to_string(),
            target: "test".to_string(),
            request_id: None,
            timestamp: Utc::now(),
        };

        let error_entry = LogEntry {
            level: "ERROR".to_string(),
            error_code: None,
            message: "test".to_string(),
            target: "test".to_string(),
            request_id: None,
            timestamp: Utc::now(),
        };

        // Act & Assert
        assert!(warn_entry.is_warning());
        assert!(warning_entry.is_warning());
        assert!(!error_entry.is_warning());
    }

    #[test]
    fn should_get_correct_log_file_path() {
        // Arrange
        let watcher = create_test_watcher();
        let date = NaiveDate::from_ymd_opt(2026, 1, 31).unwrap();

        // Act
        let path = watcher.get_log_file_path(date);

        // Assert
        assert!(path.to_string_lossy().contains("server.2026-01-31.log"));
    }

    #[test]
    fn should_get_accessors_correctly() {
        // Arrange
        let test_dir = temp_dir().join(format!("test_accessors_{}", Uuid::new_v4()));
        let log_dir = test_dir.join("logs");
        let state_dir = test_dir.join("state");
        fs::create_dir_all(&log_dir).expect("Failed to create log dir");

        let config = LogWatcherConfig {
            log_dir: log_dir.clone(),
            state_dir: state_dir.clone(),
            dedup_window_secs: 600,
        };
        let watcher = LogWatcher::with_config(config).expect("Failed to create watcher");

        // Act & Assert
        assert_eq!(watcher.log_dir(), &log_dir);
        assert_eq!(watcher.state_dir(), &state_dir);
        assert_eq!(watcher.dedup_window_secs(), 600);
    }

    #[test]
    fn should_handle_empty_log_file() {
        // Arrange
        let test_dir = temp_dir().join(format!("test_empty_log_{}", Uuid::new_v4()));
        let log_dir = test_dir.join("logs");
        let state_dir = test_dir.join("state");
        fs::create_dir_all(&log_dir).expect("Failed to create log dir");

        let mut watcher =
            LogWatcher::new(log_dir.clone(), state_dir).expect("Failed to create watcher");
        let today = Utc::now().date_naive();

        // Create empty log file
        create_test_log_file(&log_dir, today, "");

        // Act
        let events = watcher.watch().expect("Failed to watch");

        // Assert
        assert!(events.is_empty());
    }

    #[test]
    fn should_skip_invalid_json_lines() {
        // Arrange
        let test_dir = temp_dir().join(format!("test_invalid_json_{}", Uuid::new_v4()));
        let log_dir = test_dir.join("logs");
        let state_dir = test_dir.join("state");
        fs::create_dir_all(&log_dir).expect("Failed to create log dir");

        let mut watcher =
            LogWatcher::new(log_dir.clone(), state_dir).expect("Failed to create watcher");
        let today = Utc::now().date_naive();

        let content = r#"{"timestamp":"2026-01-31T14:50:00Z","level":"ERROR","target":"server::ai","fields":{"error_code":"AI5002"},"message":"Valid error"}
this is not valid json
{"timestamp":"2026-01-31T14:52:00Z","level":"ERROR","target":"server::auth","fields":{"error_code":"AUTH4001"},"message":"Another error"}"#;

        create_test_log_file(&log_dir, today, content);

        // Act
        let events = watcher.watch().expect("Failed to watch");

        // Assert - should skip invalid line and process others
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn should_detect_log_rotation_and_reset_line_counter() {
        // Arrange
        let test_dir = temp_dir().join(format!("test_rotation_{}", Uuid::new_v4()));
        let log_dir = test_dir.join("logs");
        let state_dir = test_dir.join("state");
        fs::create_dir_all(&log_dir).expect("Failed to create log dir");

        let mut watcher =
            LogWatcher::new(log_dir.clone(), state_dir.clone()).expect("Failed to create watcher");
        let today = Utc::now().date_naive();

        // First: write 3 lines
        let content = r#"{"timestamp":"2026-01-31T14:50:00Z","level":"ERROR","target":"server::ai","fields":{"error_code":"AI5001"},"message":"Error 1"}
{"timestamp":"2026-01-31T14:51:00Z","level":"ERROR","target":"server::ai","fields":{"error_code":"AI5002"},"message":"Error 2"}
{"timestamp":"2026-01-31T14:52:00Z","level":"ERROR","target":"server::ai","fields":{"error_code":"AI5003"},"message":"Error 3"}"#;
        create_test_log_file(&log_dir, today, content);

        // Watch first time - should get 3 events
        let events1 = watcher.watch().expect("Failed to watch");
        assert_eq!(events1.len(), 3);

        // Simulate log rotation: truncate file and write only 1 new line
        let rotated_content = r#"{"timestamp":"2026-01-31T15:00:00Z","level":"ERROR","target":"server::auth","fields":{"error_code":"AUTH4001"},"message":"New error after rotation"}"#;
        create_test_log_file(&log_dir, today, rotated_content);

        // Create new watcher (simulating restart or continuing)
        let mut watcher2 = LogWatcher::new(log_dir, state_dir).expect("Failed to create watcher");

        // Watch again - should detect rotation and read the new error
        let events2 = watcher2.watch().expect("Failed to watch");

        // Assert - should have detected the rotation and read the new error
        assert_eq!(events2.len(), 1);
        assert_eq!(events2[0].data["message"], "New error after rotation");
    }

    #[test]
    fn should_process_lowercase_error_level() {
        // Arrange
        let test_dir = temp_dir().join(format!("test_lowercase_{}", Uuid::new_v4()));
        let log_dir = test_dir.join("logs");
        let state_dir = test_dir.join("state");
        fs::create_dir_all(&log_dir).expect("Failed to create log dir");

        let mut watcher =
            LogWatcher::new(log_dir.clone(), state_dir).expect("Failed to create watcher");
        let today = Utc::now().date_naive();

        // Use lowercase "error" level
        let content = r#"{"timestamp":"2026-01-31T14:50:00Z","level":"error","target":"server::ai","fields":{"error_code":"AI5002"},"message":"Lowercase error"}"#;
        create_test_log_file(&log_dir, today, content);

        // Act
        let events = watcher.watch().expect("Failed to watch");

        // Assert - should detect lowercase error
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data["message"], "Lowercase error");
    }
}
