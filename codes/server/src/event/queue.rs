//! Event queue abstraction for AI automation pipeline

use crate::event::Event;
use crate::utils::AppError;
use async_trait::async_trait;
use uuid::Uuid;

/// Result type for queue operations
pub type QueueResult<T> = Result<T, AppError>;

/// Event queue trait defining the interface for queue implementations
#[async_trait]
pub trait EventQueue: Send + Sync {
    /// Push an event to the queue
    async fn push(&self, event: Event) -> QueueResult<()>;

    /// Pop the highest priority event from the queue
    /// Returns None if the queue is empty
    async fn pop(&self) -> QueueResult<Option<Event>>;

    /// Mark an event as completed and remove from processing
    async fn complete(&self, event_id: Uuid) -> QueueResult<()>;

    /// Mark an event as failed
    /// If retry count exceeds limit, move to dead letter queue
    async fn fail(&self, event: Event) -> QueueResult<()>;

    /// Get the number of pending events
    async fn pending_count(&self) -> QueueResult<usize>;

    /// Get the number of events currently being processed
    async fn processing_count(&self) -> QueueResult<usize>;

    /// Check if an event with the given fingerprint exists
    /// Used for deduplication
    async fn contains_fingerprint(&self, fingerprint: &str) -> QueueResult<bool>;
}

/// Configuration for event queue
#[derive(Debug, Clone)]
pub struct QueueConfig {
    /// Maximum retry attempts before moving to DLQ
    pub max_retries: u32,
    /// Deduplication window in seconds
    pub dedup_window_secs: u64,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            dedup_window_secs: 300, // 5 minutes
        }
    }
}

// Note: async_trait is used for trait with async methods
// This requires the async-trait crate in Cargo.toml
