//! Event system module for AI automation pipeline
//!
//! This module provides event handling infrastructure for the AI automation system:
//! - Event definition and types
//! - Event queue abstraction
//! - File-based queue implementation (MVP)
//! - Trigger filtering logic

#[allow(dead_code)]
pub mod file_queue;
#[allow(dead_code)]
pub mod queue;
#[allow(dead_code)]
pub mod trigger;

// Allow module_inception for event module naming
#[allow(clippy::module_inception)]
mod event_types;

#[allow(unused_imports)]
pub use event_types::{Event, EventMetadata, EventStatus, Priority, Severity};
#[allow(unused_imports)]
pub use file_queue::FileEventQueue;
#[allow(unused_imports)]
pub use queue::EventQueue;
#[allow(unused_imports)]
pub use trigger::TriggerFilter;
