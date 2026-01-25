//! V4 Storage
//!
//! Event sourcing and persistence layer for Agent Agency V4.
//!
//! ## Overview
//!
//! This crate provides:
//! - **Event Store**: Append-only storage for domain events with content hashing
//! - **Projections**: Read models built from event streams
//! - **Repository**: High-level data access combining events and projections
//!
//! ## Architecture
//!
//! All state changes are captured as events:
//!
//! ```text
//! Write Path:                      Read Path:
//!
//! Command                          Query
//!    │                                │
//!    ▼                                ▼
//! Repository                    ProjectionStore
//!    │                                ▲
//!    ▼                                │
//! EventStore ─────────────────► Projections
//!    │                          (in-memory)
//!    ▼
//! SQLite
//! (append-only)
//! ```
//!
//! ## Event Sourcing Benefits
//!
//! 1. **Audit Trail**: Every change is recorded with SHA-256 content hash
//! 2. **Replay**: State can be rebuilt from events
//! 3. **Temporal Queries**: Query state at any point in time
//! 4. **Debugging**: Full history of what happened
//!
//! ## Example
//!
//! ```rust,ignore
//! use v4_storage::{Repository, TaskStatus};
//!
//! let repo = Repository::in_memory()?;
//!
//! // Create a task
//! repo.create_task("task-1", "Implement feature", "Description", 2)?;
//!
//! // Start the task
//! repo.start_task("task-1", "worker-1")?;
//!
//! // Get current state
//! let task = repo.get_task("task-1").unwrap();
//! assert_eq!(task.status, TaskStatus::InProgress);
//!
//! // Complete the task
//! repo.complete_task("task-1", "result-hash")?;
//!
//! // Get event history
//! let events = repo.events_for_task("task-1")?;
//! assert_eq!(events.len(), 3);
//! ```
//!
//! ## Content Hashing
//!
//! Every event has a SHA-256 content hash for integrity verification:
//!
//! ```rust,ignore
//! let events = repo.all_events()?;
//! for event in events {
//!     assert!(event.verify()); // Verify hash integrity
//! }
//! ```

pub mod event_store;
pub mod events;
pub mod projections;
pub mod repository;
mod schema;

// Re-export main types
pub use event_store::EventStore;
pub use events::{AgentEvent, EventId, EventStream, SequenceNumber, StoredEvent};
pub use projections::{
    CouncilVerdictProjection, ProjectionCounts, ProjectionStore, TaskProjection, TaskStatus,
    WorkerProjection, WorkerStatus,
};
pub use repository::Repository;

/// Storage errors
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Event not found: {0}")]
    EventNotFound(String),

    #[error("Invalid event: {0}")]
    InvalidEvent(String),

    #[error("Integrity error: {0}")]
    IntegrityError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_workflow() {
        let repo = Repository::in_memory().unwrap();

        // Create and process a task
        repo.create_task("t-1", "Implement feature", "Add new functionality", 2)
            .unwrap();

        // Record invariant check
        repo.record_invariant_check("INV-CORE-04", "t-1", true, None)
            .unwrap();

        // Record council verdict
        repo.record_council_verdict("t-1", 0.95, 0.90, 0.88, 0.91, true)
            .unwrap();

        // Record gate result
        repo.record_gate_result("integration-f1", "t-1", 0.92, 0.90)
            .unwrap();

        // Spawn worker and start task
        repo.spawn_worker("w-1", "t-1", "local").unwrap();
        repo.start_task("t-1", "w-1").unwrap();

        // Complete task
        repo.complete_task("t-1", "sha256:abc123").unwrap();
        repo.complete_worker("w-1", "completed").unwrap();

        // Verify final state
        let task = repo.get_task("t-1").unwrap();
        assert_eq!(task.status, TaskStatus::Completed);

        // Verify event count (Created, Started, Completed have task aggregate)
        let events = repo.events_for_task("t-1").unwrap();
        assert_eq!(events.len(), 3);

        // Verify integrity
        assert!(repo.verify_all().unwrap());

        // Check counts
        let counts = repo.counts();
        assert_eq!(counts.total_tasks, 1);
        assert_eq!(counts.completed_tasks, 1);
        assert_eq!(counts.total_workers, 1);
        assert_eq!(counts.total_verdicts, 1);
    }

    #[test]
    fn test_event_types() {
        let repo = Repository::in_memory().unwrap();

        // Record various event types
        repo.create_task("t-1", "Task", "Desc", 1).unwrap();

        repo.record(AgentEvent::ToolInvoked {
            tool_name: "read_file".to_string(),
            args_hash: "hash1".to_string(),
            task_id: "t-1".to_string(),
        })
        .unwrap();

        repo.record(AgentEvent::ToolCompleted {
            tool_name: "read_file".to_string(),
            result_hash: "hash2".to_string(),
            success: true,
        })
        .unwrap();

        repo.record(AgentEvent::MemoryStored {
            key: "context-1".to_string(),
            content_hash: "hash3".to_string(),
            memory_type: "working".to_string(),
        })
        .unwrap();

        assert_eq!(repo.event_count().unwrap(), 4);
    }

    #[test]
    fn test_sequence_ordering() {
        let repo = Repository::in_memory().unwrap();

        for i in 1..=10 {
            repo.create_task(&format!("t-{}", i), &format!("Task {}", i), "Desc", 1)
                .unwrap();
        }

        let events = repo.all_events().unwrap();
        for (i, event) in events.iter().enumerate() {
            assert_eq!(event.sequence, SequenceNumber((i + 1) as u64));
        }
    }

    #[test]
    fn test_events_after() {
        let repo = Repository::in_memory().unwrap();

        for i in 1..=5 {
            repo.create_task(&format!("t-{}", i), &format!("Task {}", i), "Desc", 1)
                .unwrap();
        }

        let events = repo.events_after(SequenceNumber(3)).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].sequence, SequenceNumber(4));
        assert_eq!(events[1].sequence, SequenceNumber(5));
    }
}
