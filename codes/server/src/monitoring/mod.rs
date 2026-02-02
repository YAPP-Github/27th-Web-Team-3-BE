//! Monitoring module for AI automation pipeline (Phase 2)
//!
//! This module provides log monitoring and alerting infrastructure:
//! - Log file watching and parsing
//! - Discord webhook notifications
//! - Event processing loop

#![allow(dead_code)]
#![allow(unused_imports)]

pub mod discord_alert;
pub mod log_watcher;
pub mod processor;

pub use discord_alert::DiscordAlert;
pub use log_watcher::{LogEntry, LogWatcher};
pub use processor::EventProcessor;
