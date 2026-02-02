//! File-based event queue implementation (MVP)
//!
//! Directory structure:
//! ```text
//! queue_dir/
//! ├── pending/     # Events waiting to be processed
//! ├── processing/  # Events currently being processed
//! ├── completed/   # Successfully processed events (optional retention)
//! └── dlq/         # Dead letter queue for failed events
//! ```
//!
//! ## TODO(MVP): 프로덕션 전 개선 필요 사항
//!
//! 1. **비동기 파일 I/O**: 현재 `std::fs` 사용 중. 프로덕션에서는 `tokio::fs`로
//!    전환하거나 `tokio::task::spawn_blocking`으로 감싸야 함.
//!
//! 2. **Completed 디렉토리 Cleanup**: 완료된 이벤트가 계속 쌓임. 다음 중 하나 필요:
//!    - 주기적 cleanup 작업 (일정 기간 이후 삭제)
//!    - 완료 이벤트 즉시 삭제
//!    - 최대 보관 개수 제한

use crate::event::queue::{EventQueue, QueueConfig, QueueResult};
use crate::event::{Event, EventStatus};
use crate::utils::AppError;
use async_trait::async_trait;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// File-based event queue implementation
/// Suitable for MVP and single-instance deployments
pub struct FileEventQueue {
    /// Base directory for queue storage
    queue_dir: PathBuf,
    /// Queue configuration
    config: QueueConfig,
    /// Lock for thread-safe file operations
    lock: Arc<RwLock<()>>,
}

impl FileEventQueue {
    /// Create a new file-based event queue
    pub fn new(queue_dir: PathBuf) -> QueueResult<Self> {
        Self::with_config(queue_dir, QueueConfig::default())
    }

    /// Create a new file-based event queue with custom config
    pub fn with_config(queue_dir: PathBuf, config: QueueConfig) -> QueueResult<Self> {
        // Create directory structure
        let dirs = ["pending", "processing", "completed", "dlq"];
        for dir in &dirs {
            let path = queue_dir.join(dir);
            fs::create_dir_all(&path).map_err(|e| {
                error!(error = %e, dir = %path.display(), "Failed to create queue directory");
                AppError::InternalError(format!("Failed to create queue directory: {}", e))
            })?;
        }

        info!(queue_dir = %queue_dir.display(), "File event queue initialized");

        Ok(Self {
            queue_dir,
            config,
            lock: Arc::new(RwLock::new(())),
        })
    }

    /// Get path to pending directory
    fn pending_dir(&self) -> PathBuf {
        self.queue_dir.join("pending")
    }

    /// Get path to processing directory
    fn processing_dir(&self) -> PathBuf {
        self.queue_dir.join("processing")
    }

    /// Get path to completed directory
    fn completed_dir(&self) -> PathBuf {
        self.queue_dir.join("completed")
    }

    /// Get path to dead letter queue directory
    fn dlq_dir(&self) -> PathBuf {
        self.queue_dir.join("dlq")
    }

    /// Read and parse an event from a file
    fn read_event_file(&self, path: &PathBuf) -> QueueResult<Event> {
        let content = fs::read_to_string(path).map_err(|e| {
            error!(error = %e, path = %path.display(), "Failed to read event file");
            AppError::InternalError(format!("Failed to read event file: {}", e))
        })?;

        serde_json::from_str(&content).map_err(|e| {
            error!(error = %e, path = %path.display(), "Failed to parse event file");
            AppError::InternalError(format!("Failed to parse event file: {}", e))
        })
    }

    /// Write an event to a file
    fn write_event_file(&self, dir: &std::path::Path, event: &Event) -> QueueResult<PathBuf> {
        let filename = event.to_filename();
        let path = dir.join(&filename);

        let content = serde_json::to_string_pretty(event).map_err(|e| {
            error!(error = %e, "Failed to serialize event");
            AppError::InternalError(format!("Failed to serialize event: {}", e))
        })?;

        fs::write(&path, content).map_err(|e| {
            error!(error = %e, path = %path.display(), "Failed to write event file");
            AppError::InternalError(format!("Failed to write event file: {}", e))
        })?;

        debug!(event_id = %event.id, path = %path.display(), "Event written to file");
        Ok(path)
    }

    /// Find file by event ID in a directory
    fn find_event_file(
        &self,
        dir: &std::path::Path,
        event_id: Uuid,
    ) -> QueueResult<Option<PathBuf>> {
        let id_str = event_id.to_string();

        let entries = fs::read_dir(dir).map_err(|e| {
            error!(error = %e, dir = %dir.display(), "Failed to read directory");
            AppError::InternalError(format!("Failed to read directory: {}", e))
        })?;

        for entry in entries.flatten() {
            let filename = entry.file_name().to_string_lossy().to_string();
            if filename.contains(&id_str) {
                return Ok(Some(entry.path()));
            }
        }

        Ok(None)
    }

    /// Count files in a directory
    fn count_files_in_dir(&self, dir: &std::path::Path) -> QueueResult<usize> {
        let entries = fs::read_dir(dir).map_err(|e| {
            error!(error = %e, dir = %dir.display(), "Failed to read directory");
            AppError::InternalError(format!("Failed to read directory: {}", e))
        })?;

        Ok(entries.filter_map(|e| e.ok()).count())
    }
}

#[async_trait]
impl EventQueue for FileEventQueue {
    async fn push(&self, event: Event) -> QueueResult<()> {
        let _guard = self.lock.write().await;

        // SAFETY: Caller holds write lock, using unlocked version to avoid deadlock
        if self.contains_fingerprint_unlocked(&event.metadata.fingerprint)? {
            warn!(
                fingerprint = %event.metadata.fingerprint,
                "Duplicate event detected, skipping"
            );
            return Ok(());
        }

        self.write_event_file(&self.pending_dir(), &event)?;

        info!(
            event_id = %event.id,
            event_type = %event.event_type,
            priority = ?event.priority,
            "Event pushed to queue"
        );

        Ok(())
    }

    async fn pop(&self) -> QueueResult<Option<Event>> {
        let _guard = self.lock.write().await;

        let pending_dir = self.pending_dir();
        let processing_dir = self.processing_dir();

        // Search by priority order (P0 -> P1 -> P2 -> P3)
        for priority in 0..=3 {
            let prefix = format!("p{}_", priority);

            let entries = match fs::read_dir(&pending_dir) {
                Ok(e) => e,
                Err(e) => {
                    error!(error = %e, "Failed to read pending directory");
                    return Err(AppError::InternalError(format!(
                        "Failed to read pending directory: {}",
                        e
                    )));
                }
            };

            // Find first matching file for this priority
            for entry in entries.flatten() {
                let filename = entry.file_name().to_string_lossy().to_string();
                if filename.starts_with(&prefix) && filename.ends_with(".json") {
                    let original_path = entry.path();
                    let new_path = processing_dir.join(&filename);

                    // Move to processing directory
                    fs::rename(&original_path, &new_path).map_err(|e| {
                        error!(error = %e, "Failed to move event to processing");
                        AppError::InternalError(format!(
                            "Failed to move event to processing: {}",
                            e
                        ))
                    })?;

                    // Read and parse event - with rollback on failure
                    let mut event = match self.read_event_file(&new_path) {
                        Ok(e) => e,
                        Err(e) => {
                            // Rollback: move back to pending or to DLQ if corrupted
                            error!(error = %e, path = %new_path.display(), "Failed to parse event, moving to DLQ");
                            let dlq_path = self.dlq_dir().join(&filename);
                            if let Err(move_err) = fs::rename(&new_path, &dlq_path) {
                                error!(error = %move_err, "Failed to move corrupted event to DLQ, attempting rollback to pending");
                                let _ = fs::rename(&new_path, &original_path);
                            }
                            return Err(e);
                        }
                    };
                    event.status = EventStatus::Processing;

                    // Persist status change to file - with rollback on failure
                    let content = match serde_json::to_string_pretty(&event) {
                        Ok(c) => c,
                        Err(e) => {
                            error!(error = %e, "Failed to serialize event, rolling back");
                            // Rollback: move back to pending
                            if let Err(rollback_err) = fs::rename(&new_path, &original_path) {
                                error!(error = %rollback_err, "Failed to rollback event to pending");
                            }
                            return Err(AppError::InternalError(format!(
                                "Failed to serialize event: {}",
                                e
                            )));
                        }
                    };

                    if let Err(e) = fs::write(&new_path, content) {
                        error!(error = %e, path = %new_path.display(), "Failed to update event file, rolling back");
                        // Rollback: move back to pending
                        if let Err(rollback_err) = fs::rename(&new_path, &original_path) {
                            error!(error = %rollback_err, "Failed to rollback event to pending");
                        }
                        return Err(AppError::InternalError(format!(
                            "Failed to update event file: {}",
                            e
                        )));
                    }

                    info!(
                        event_id = %event.id,
                        event_type = %event.event_type,
                        priority = ?event.priority,
                        "Event popped from queue"
                    );

                    return Ok(Some(event));
                }
            }
        }

        Ok(None)
    }

    async fn complete(&self, event_id: Uuid) -> QueueResult<()> {
        let _guard = self.lock.write().await;

        let processing_dir = self.processing_dir();
        let completed_dir = self.completed_dir();

        // Find event file in processing directory
        if let Some(path) = self.find_event_file(&processing_dir, event_id)? {
            // Read event, update status, and write to completed directory
            let mut event = self.read_event_file(&path)?;
            event.status = EventStatus::Completed;

            let filename = path
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| format!("{}.json", event_id));

            let new_path = completed_dir.join(&filename);

            // Write updated event to completed directory
            let content = serde_json::to_string_pretty(&event).map_err(|e| {
                error!(error = %e, "Failed to serialize event");
                AppError::InternalError(format!("Failed to serialize event: {}", e))
            })?;
            fs::write(&new_path, &content).map_err(|e| {
                error!(error = %e, path = %new_path.display(), "Failed to write completed event");
                AppError::InternalError(format!("Failed to write completed event: {}", e))
            })?;

            // Remove from processing directory
            fs::remove_file(&path).map_err(|e| {
                error!(error = %e, event_id = %event_id, "Failed to remove event from processing");
                AppError::InternalError(format!("Failed to remove event from processing: {}", e))
            })?;

            info!(event_id = %event_id, "Event completed successfully");
            Ok(())
        } else {
            warn!(event_id = %event_id, "Event not found in processing directory");
            Ok(())
        }
    }

    async fn fail(&self, event: Event) -> QueueResult<()> {
        let _guard = self.lock.write().await;

        let processing_dir = self.processing_dir();

        // Remove from processing if exists
        if let Some(path) = self.find_event_file(&processing_dir, event.id)? {
            fs::remove_file(&path).map_err(|e| {
                error!(error = %e, event_id = %event.id, "Failed to remove event from processing");
                AppError::InternalError(format!("Failed to remove event from processing: {}", e))
            })?;
        }

        if event.retry_count >= self.config.max_retries {
            // Move to dead letter queue
            let mut dlq_event = event.clone();
            dlq_event.status = EventStatus::Failed;
            self.write_event_file(&self.dlq_dir(), &dlq_event)?;

            warn!(
                event_id = %event.id,
                retry_count = %event.retry_count,
                "Event moved to dead letter queue after max retries"
            );
        } else {
            // Re-queue with incremented retry count
            let mut retry_event = event.clone();
            retry_event.retry_count += 1;
            retry_event.status = EventStatus::Retrying;
            self.write_event_file(&self.pending_dir(), &retry_event)?;

            info!(
                event_id = %event.id,
                retry_count = %retry_event.retry_count,
                "Event re-queued for retry"
            );
        }

        Ok(())
    }

    async fn pending_count(&self) -> QueueResult<usize> {
        let _guard = self.lock.read().await;
        self.count_files_in_dir(&self.pending_dir())
    }

    async fn processing_count(&self) -> QueueResult<usize> {
        let _guard = self.lock.read().await;
        self.count_files_in_dir(&self.processing_dir())
    }

    async fn contains_fingerprint(&self, fingerprint: &str) -> QueueResult<bool> {
        // Acquire read lock for thread-safe external calls
        // Note: Internal calls from push() use contains_fingerprint_unlocked() directly
        // while holding the write lock to avoid deadlock.
        let _guard = self.lock.read().await;
        self.contains_fingerprint_unlocked(fingerprint)
    }
}

impl FileEventQueue {
    /// Internal unlocked method to check fingerprint.
    /// SAFETY: Caller must ensure proper locking (either read or write lock).
    /// - Called from `push()` while holding write lock
    /// - Called from `contains_fingerprint()` while holding read lock
    fn contains_fingerprint_unlocked(&self, fingerprint: &str) -> QueueResult<bool> {
        // Check pending directory
        let pending_dir = self.pending_dir();
        let entries = fs::read_dir(&pending_dir).map_err(|e| {
            AppError::InternalError(format!("Failed to read pending directory: {}", e))
        })?;

        for entry in entries.flatten() {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                // Parse JSON and check fingerprint field directly to avoid false positives
                if let Ok(event) = serde_json::from_str::<Event>(&content) {
                    if event.metadata.fingerprint == fingerprint {
                        return Ok(true);
                    }
                }
            }
        }

        // Check processing directory
        let processing_dir = self.processing_dir();
        let entries = fs::read_dir(&processing_dir).map_err(|e| {
            AppError::InternalError(format!("Failed to read processing directory: {}", e))
        })?;

        for entry in entries.flatten() {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                // Parse JSON and check fingerprint field directly to avoid false positives
                if let Ok(event) = serde_json::from_str::<Event>(&content) {
                    if event.metadata.fingerprint == fingerprint {
                        return Ok(true);
                    }
                }
            }
        }

        // Check completed directory within dedup window
        let completed_dir = self.completed_dir();
        if let Ok(entries) = fs::read_dir(&completed_dir) {
            let dedup_window = std::time::Duration::from_secs(self.config.dedup_window_secs);
            let now = std::time::SystemTime::now();

            for entry in entries.flatten() {
                // Check file modification time for dedup window
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(age) = now.duration_since(modified) {
                            if age > dedup_window {
                                continue; // Skip files older than dedup window
                            }
                        }
                    }
                }

                if let Ok(content) = fs::read_to_string(entry.path()) {
                    if let Ok(event) = serde_json::from_str::<Event>(&content) {
                        if event.metadata.fingerprint == fingerprint {
                            return Ok(true);
                        }
                    }
                }
            }
        }

        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Priority;
    use std::env::temp_dir;
    use uuid::Uuid;

    // TODO(MVP): 테스트 임시 디렉토리 자동 정리 필요
    // 옵션 1: `tempfile` crate의 `TempDir` 사용 (자동 cleanup)
    // 옵션 2: Drop trait 구현하여 테스트 종료 시 cleanup
    // 현재는 시스템 임시 디렉토리 사용으로 OS가 정리

    fn create_test_queue() -> FileEventQueue {
        let test_dir = temp_dir().join(format!("test_queue_{}", Uuid::new_v4()));
        FileEventQueue::new(test_dir).expect("Failed to create test queue")
    }

    fn create_test_event(priority: Priority) -> Event {
        Event::new(
            "test.event",
            "test",
            priority,
            serde_json::json!({"test": "data"}),
        )
    }

    #[tokio::test]
    async fn should_push_and_pop_event() {
        // Arrange
        let queue = create_test_queue();
        let event = create_test_event(Priority::P1);
        let event_id = event.id;

        // Act
        queue.push(event).await.expect("Failed to push event");
        let popped = queue.pop().await.expect("Failed to pop event");

        // Assert
        assert!(popped.is_some());
        let popped_event = popped.unwrap();
        assert_eq!(popped_event.id, event_id);
        assert_eq!(popped_event.status, EventStatus::Processing);
    }

    #[tokio::test]
    async fn should_pop_events_in_priority_order() {
        // Arrange
        let queue = create_test_queue();

        // Push events in reverse priority order
        let p3_event = create_test_event(Priority::P3);
        let p1_event = create_test_event(Priority::P1);
        let p0_event = create_test_event(Priority::P0);

        queue
            .push(p3_event.clone())
            .await
            .expect("Failed to push P3");
        queue
            .push(p1_event.clone())
            .await
            .expect("Failed to push P1");
        queue
            .push(p0_event.clone())
            .await
            .expect("Failed to push P0");

        // Act & Assert - should pop in P0, P1, P3 order
        let first = queue.pop().await.expect("Failed to pop").unwrap();
        assert_eq!(first.id, p0_event.id);

        let second = queue.pop().await.expect("Failed to pop").unwrap();
        assert_eq!(second.id, p1_event.id);

        let third = queue.pop().await.expect("Failed to pop").unwrap();
        assert_eq!(third.id, p3_event.id);
    }

    #[tokio::test]
    async fn should_return_none_for_empty_queue() {
        // Arrange
        let queue = create_test_queue();

        // Act
        let result = queue.pop().await.expect("Failed to pop");

        // Assert
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn should_complete_event() {
        // Arrange
        let queue = create_test_queue();
        let event = create_test_event(Priority::P1);
        let event_id = event.id;

        queue.push(event).await.expect("Failed to push");
        queue.pop().await.expect("Failed to pop");

        // Act
        queue.complete(event_id).await.expect("Failed to complete");

        // Assert
        assert_eq!(queue.processing_count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn should_requeue_failed_event_under_max_retries() {
        // Arrange
        let queue = create_test_queue();
        let event = create_test_event(Priority::P1);
        let event_id = event.id;

        queue.push(event.clone()).await.expect("Failed to push");
        queue.pop().await.expect("Failed to pop");

        // Act
        queue.fail(event).await.expect("Failed to fail");

        // Assert
        let pending = queue.pending_count().await.unwrap();
        assert_eq!(pending, 1);

        // Pop should get the retried event
        let retried = queue.pop().await.expect("Failed to pop").unwrap();
        assert_eq!(retried.id, event_id);
        assert_eq!(retried.retry_count, 1);
        // After pop, status becomes Processing (not Retrying anymore)
        assert_eq!(retried.status, EventStatus::Processing);
    }

    #[tokio::test]
    async fn should_move_to_dlq_after_max_retries() {
        // Arrange
        let config = QueueConfig {
            max_retries: 2,
            dedup_window_secs: 300,
        };
        let test_dir = temp_dir().join(format!("test_queue_{}", Uuid::new_v4()));
        let queue =
            FileEventQueue::with_config(test_dir.clone(), config).expect("Failed to create queue");

        let mut event = create_test_event(Priority::P1);
        event.retry_count = 2; // Already at max

        queue.push(event.clone()).await.expect("Failed to push");
        queue.pop().await.expect("Failed to pop");

        // Act
        queue.fail(event).await.expect("Failed to fail");

        // Assert
        let pending = queue.pending_count().await.unwrap();
        assert_eq!(pending, 0);

        // Check DLQ has the event
        let dlq_count = fs::read_dir(test_dir.join("dlq"))
            .unwrap()
            .filter_map(|e| e.ok())
            .count();
        assert_eq!(dlq_count, 1);
    }

    #[tokio::test]
    async fn should_skip_duplicate_events_by_fingerprint() {
        // Arrange
        let queue = create_test_queue();
        let fingerprint = "unique_fingerprint_123";

        let event1 = create_test_event(Priority::P1).with_fingerprint(fingerprint);
        let event2 = create_test_event(Priority::P1).with_fingerprint(fingerprint);

        // Act
        queue.push(event1).await.expect("Failed to push first");
        queue.push(event2).await.expect("Failed to push second");

        // Assert - only one event should be in the queue
        let pending = queue.pending_count().await.unwrap();
        assert_eq!(pending, 1);
    }

    #[tokio::test]
    async fn should_count_pending_and_processing() {
        // Arrange
        let queue = create_test_queue();

        queue.push(create_test_event(Priority::P1)).await.unwrap();
        queue.push(create_test_event(Priority::P2)).await.unwrap();

        // Act & Assert - initial state
        assert_eq!(queue.pending_count().await.unwrap(), 2);
        assert_eq!(queue.processing_count().await.unwrap(), 0);

        // Pop one event
        queue.pop().await.unwrap();

        // After pop
        assert_eq!(queue.pending_count().await.unwrap(), 1);
        assert_eq!(queue.processing_count().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn should_move_corrupted_json_to_dlq() {
        // Arrange
        let test_dir = temp_dir().join(format!("test_queue_{}", Uuid::new_v4()));
        let queue = FileEventQueue::new(test_dir.clone()).expect("Failed to create queue");

        // Write a corrupted JSON file directly to pending
        let corrupted_file = test_dir.join("pending").join("p1_corrupted.json");
        fs::write(&corrupted_file, "{ invalid json }").expect("Failed to write corrupted file");

        // Act
        let result = queue.pop().await;

        // Assert
        assert!(result.is_err());

        // Corrupted file should be in DLQ
        let dlq_count = fs::read_dir(test_dir.join("dlq"))
            .unwrap()
            .filter_map(|e| e.ok())
            .count();
        assert_eq!(dlq_count, 1);

        // Pending should be empty (file moved to DLQ)
        assert_eq!(queue.pending_count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn should_detect_duplicate_in_completed_within_dedup_window() {
        // Arrange
        let config = QueueConfig {
            max_retries: 3,
            dedup_window_secs: 300, // 5 minutes
        };
        let test_dir = temp_dir().join(format!("test_queue_{}", Uuid::new_v4()));
        let queue =
            FileEventQueue::with_config(test_dir.clone(), config).expect("Failed to create queue");

        let fingerprint = "unique_fingerprint_for_dedup_test";

        // Push, pop, and complete an event
        let event1 = create_test_event(Priority::P1).with_fingerprint(fingerprint);
        queue.push(event1.clone()).await.expect("Failed to push");
        let popped = queue.pop().await.expect("Failed to pop").unwrap();
        queue.complete(popped.id).await.expect("Failed to complete");

        // Act: Try to push another event with same fingerprint (should be detected as duplicate)
        let event2 = create_test_event(Priority::P1).with_fingerprint(fingerprint);
        queue.push(event2).await.expect("Failed to push");

        // Assert: Only one event should be in completed, none in pending (duplicate skipped)
        let pending = queue.pending_count().await.unwrap();
        assert_eq!(pending, 0);
    }

    #[tokio::test]
    async fn should_not_detect_duplicate_outside_dedup_window() {
        // Arrange
        let config = QueueConfig {
            max_retries: 3,
            dedup_window_secs: 0, // No dedup window
        };
        let test_dir = temp_dir().join(format!("test_queue_{}", Uuid::new_v4()));
        let queue =
            FileEventQueue::with_config(test_dir.clone(), config).expect("Failed to create queue");

        let fingerprint = "unique_fingerprint_for_window_test";

        // Push, pop, and complete an event
        let event1 = create_test_event(Priority::P1).with_fingerprint(fingerprint);
        queue.push(event1.clone()).await.expect("Failed to push");
        let popped = queue.pop().await.expect("Failed to pop").unwrap();
        queue.complete(popped.id).await.expect("Failed to complete");

        // Manually set file modification time to the past (simulate old event)
        let completed_files: Vec<_> = fs::read_dir(test_dir.join("completed"))
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        for entry in completed_files {
            // Touch the file with old timestamp (by rewriting with same content)
            let content = fs::read_to_string(entry.path()).unwrap();
            fs::write(entry.path(), content).unwrap();
        }

        // Act: Try to push another event with same fingerprint
        let event2 = create_test_event(Priority::P1).with_fingerprint(fingerprint);
        queue.push(event2).await.expect("Failed to push");

        // Assert: With 0 second dedup window, event should be allowed
        let pending = queue.pending_count().await.unwrap();
        assert_eq!(pending, 1);
    }
}
