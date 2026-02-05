//! Event system module for AI automation pipeline
//!
//! This module provides event handling infrastructure for the AI automation system:
//! - Event definition and types
//! - Event queue abstraction
//! - File-based queue implementation (MVP)
//! - Trigger filtering logic
//!
//! TODO(MVP): dead_code/unused_imports 허용은 MVP 단계이므로 적용됨.
//!            Phase 3 완료 후 실제 사용 시점에 제거 필요.

#![allow(dead_code)]
#![allow(unused_imports)]

pub mod file_queue;
pub mod queue;
pub mod trigger;

// Allow module_inception for event module naming
#[allow(clippy::module_inception)]
mod event_types;

pub use event_types::{Event, EventMetadata, EventStatus, Priority, Severity};
pub use file_queue::FileEventQueue;
pub use queue::EventQueue;
pub use trigger::{
    RateLimitAction, RateLimitConfig, RateLimiter, TriggerFilter, TriggerFilterBuilder,
};
