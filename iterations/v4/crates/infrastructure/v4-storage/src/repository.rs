//! Repository Pattern
//!
//! High-level data access combining event store and projections.

use std::sync::{Arc, RwLock};

use crate::event_store::EventStore;
use crate::events::{AgentEvent, SequenceNumber, StoredEvent};
use crate::projections::{
    CouncilVerdictProjection, ProjectionCounts, ProjectionStore, TaskProjection, TaskStatus,
    WorkerProjection,
};
use crate::StorageError;

/// Repository for agent data
///
/// Combines event sourcing with projections for efficient read/write access.
pub struct Repository {
    event_store: Arc<EventStore>,
    projections: Arc<RwLock<ProjectionStore>>,
}

impl Repository {
    /// Create a new repository with an in-memory event store
    pub fn in_memory() -> Result<Self, StorageError> {
        let event_store = Arc::new(EventStore::in_memory()?);
        let projections = Arc::new(RwLock::new(ProjectionStore::new()));

        Ok(Self {
            event_store,
            projections,
        })
    }

    /// Create a new repository with a file-based event store
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self, StorageError> {
        let event_store = Arc::new(EventStore::open(path)?);
        let mut projections = ProjectionStore::new();

        // Rebuild projections from event store
        let events = event_store.get_all()?;
        projections.apply_all(&events);

        Ok(Self {
            event_store,
            projections: Arc::new(RwLock::new(projections)),
        })
    }

    /// Record an event and update projections
    pub fn record(&self, event: AgentEvent) -> Result<StoredEvent, StorageError> {
        let stored = self.event_store.append(event)?;

        let mut projections = self.projections.write().unwrap();
        projections.apply(&stored);

        Ok(stored)
    }

    /// Record an event with correlation ID
    pub fn record_with_correlation(
        &self,
        event: AgentEvent,
        correlation_id: impl Into<String>,
    ) -> Result<StoredEvent, StorageError> {
        let stored = self
            .event_store
            .append_with_correlation(event, Some(correlation_id.into()))?;

        let mut projections = self.projections.write().unwrap();
        projections.apply(&stored);

        Ok(stored)
    }

    // Task operations

    /// Create a new task
    pub fn create_task(
        &self,
        task_id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        risk_tier: u8,
    ) -> Result<StoredEvent, StorageError> {
        self.record(AgentEvent::TaskCreated {
            task_id: task_id.into(),
            title: title.into(),
            description: description.into(),
            risk_tier,
        })
    }

    /// Start a task with a worker
    pub fn start_task(
        &self,
        task_id: impl Into<String>,
        worker_id: impl Into<String>,
    ) -> Result<StoredEvent, StorageError> {
        self.record(AgentEvent::TaskStarted {
            task_id: task_id.into(),
            worker_id: worker_id.into(),
        })
    }

    /// Complete a task
    pub fn complete_task(
        &self,
        task_id: impl Into<String>,
        result_hash: impl Into<String>,
    ) -> Result<StoredEvent, StorageError> {
        self.record(AgentEvent::TaskCompleted {
            task_id: task_id.into(),
            result_hash: result_hash.into(),
        })
    }

    /// Fail a task
    pub fn fail_task(
        &self,
        task_id: impl Into<String>,
        error: impl Into<String>,
    ) -> Result<StoredEvent, StorageError> {
        self.record(AgentEvent::TaskFailed {
            task_id: task_id.into(),
            error: error.into(),
        })
    }

    /// Get a task by ID
    pub fn get_task(&self, task_id: &str) -> Option<TaskProjection> {
        let projections = self.projections.read().unwrap();
        projections.get_task(task_id).cloned()
    }

    /// Get all tasks
    pub fn all_tasks(&self) -> Vec<TaskProjection> {
        let projections = self.projections.read().unwrap();
        projections.all_tasks().into_iter().cloned().collect()
    }

    /// Get tasks by status
    pub fn tasks_by_status(&self, status: TaskStatus) -> Vec<TaskProjection> {
        let projections = self.projections.read().unwrap();
        projections
            .tasks_by_status(status)
            .into_iter()
            .cloned()
            .collect()
    }

    // Worker operations

    /// Spawn a worker
    pub fn spawn_worker(
        &self,
        worker_id: impl Into<String>,
        task_id: impl Into<String>,
        worker_type: impl Into<String>,
    ) -> Result<StoredEvent, StorageError> {
        self.record(AgentEvent::WorkerSpawned {
            worker_id: worker_id.into(),
            task_id: task_id.into(),
            worker_type: worker_type.into(),
        })
    }

    /// Complete a worker
    pub fn complete_worker(
        &self,
        worker_id: impl Into<String>,
        status: impl Into<String>,
    ) -> Result<StoredEvent, StorageError> {
        self.record(AgentEvent::WorkerCompleted {
            worker_id: worker_id.into(),
            status: status.into(),
        })
    }

    /// Get a worker by ID
    pub fn get_worker(&self, worker_id: &str) -> Option<WorkerProjection> {
        let projections = self.projections.read().unwrap();
        projections.get_worker(worker_id).cloned()
    }

    /// Get all workers
    pub fn all_workers(&self) -> Vec<WorkerProjection> {
        let projections = self.projections.read().unwrap();
        projections.all_workers().into_iter().cloned().collect()
    }

    // Governance operations

    /// Record a council verdict
    pub fn record_council_verdict(
        &self,
        task_id: impl Into<String>,
        constitutional_score: f64,
        technical_score: f64,
        quality_score: f64,
        aggregate_score: f64,
        passed: bool,
    ) -> Result<StoredEvent, StorageError> {
        self.record(AgentEvent::CouncilConvened {
            task_id: task_id.into(),
            constitutional_score,
            technical_score,
            quality_score,
            aggregate_score,
            passed,
        })
    }

    /// Record an invariant check
    pub fn record_invariant_check(
        &self,
        invariant_id: impl Into<String>,
        task_id: impl Into<String>,
        passed: bool,
        details: Option<String>,
    ) -> Result<StoredEvent, StorageError> {
        self.record(AgentEvent::InvariantChecked {
            invariant_id: invariant_id.into(),
            task_id: task_id.into(),
            passed,
            details,
        })
    }

    /// Record a gate result
    pub fn record_gate_result(
        &self,
        gate_id: impl Into<String>,
        task_id: impl Into<String>,
        score: f64,
        threshold: f64,
    ) -> Result<StoredEvent, StorageError> {
        let gate_id = gate_id.into();
        let task_id = task_id.into();

        if score >= threshold {
            self.record(AgentEvent::GatePassed {
                gate_id,
                task_id,
                score,
            })
        } else {
            self.record(AgentEvent::GateFailed {
                gate_id,
                task_id,
                score,
                threshold,
            })
        }
    }

    /// Get verdicts for a task
    pub fn verdicts_for_task(&self, task_id: &str) -> Vec<CouncilVerdictProjection> {
        let projections = self.projections.read().unwrap();
        projections
            .verdicts_for_task(task_id)
            .into_iter()
            .cloned()
            .collect()
    }

    // Event queries

    /// Get all events
    pub fn all_events(&self) -> Result<Vec<StoredEvent>, StorageError> {
        self.event_store.get_all()
    }

    /// Get events for a task
    pub fn events_for_task(&self, task_id: &str) -> Result<Vec<StoredEvent>, StorageError> {
        self.event_store.get_by_aggregate("task", task_id)
    }

    /// Get events after a sequence
    pub fn events_after(&self, sequence: SequenceNumber) -> Result<Vec<StoredEvent>, StorageError> {
        self.event_store.get_after(sequence)
    }

    /// Get events by correlation ID
    pub fn events_by_correlation(
        &self,
        correlation_id: &str,
    ) -> Result<Vec<StoredEvent>, StorageError> {
        self.event_store.get_by_correlation(correlation_id)
    }

    // Stats

    /// Get projection counts
    pub fn counts(&self) -> ProjectionCounts {
        let projections = self.projections.read().unwrap();
        projections.counts()
    }

    /// Get total event count
    pub fn event_count(&self) -> Result<u64, StorageError> {
        self.event_store.count()
    }

    /// Get current sequence number
    pub fn current_sequence(&self) -> SequenceNumber {
        self.event_store.current_sequence()
    }

    /// Verify all events
    pub fn verify_all(&self) -> Result<bool, StorageError> {
        let results = self.event_store.verify_all()?;
        Ok(results.iter().all(|(_, valid)| *valid))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_in_memory() {
        let repo = Repository::in_memory().unwrap();
        assert_eq!(repo.event_count().unwrap(), 0);
    }

    #[test]
    fn test_task_lifecycle() {
        let repo = Repository::in_memory().unwrap();

        // Create task
        repo.create_task("t-1", "Test Task", "Description", 1)
            .unwrap();

        let task = repo.get_task("t-1").unwrap();
        assert_eq!(task.status, TaskStatus::Pending);

        // Start task
        repo.start_task("t-1", "w-1").unwrap();

        let task = repo.get_task("t-1").unwrap();
        assert_eq!(task.status, TaskStatus::InProgress);
        assert_eq!(task.worker_id, Some("w-1".to_string()));

        // Complete task
        repo.complete_task("t-1", "result-hash").unwrap();

        let task = repo.get_task("t-1").unwrap();
        assert_eq!(task.status, TaskStatus::Completed);
    }

    #[test]
    fn test_task_failure() {
        let repo = Repository::in_memory().unwrap();

        repo.create_task("t-1", "Test Task", "Description", 1)
            .unwrap();
        repo.start_task("t-1", "w-1").unwrap();
        repo.fail_task("t-1", "Something went wrong").unwrap();

        let task = repo.get_task("t-1").unwrap();
        assert_eq!(task.status, TaskStatus::Failed);
        assert_eq!(task.error, Some("Something went wrong".to_string()));
    }

    #[test]
    fn test_tasks_by_status() {
        let repo = Repository::in_memory().unwrap();

        repo.create_task("t-1", "Task 1", "Desc", 1).unwrap();
        repo.create_task("t-2", "Task 2", "Desc", 1).unwrap();
        repo.create_task("t-3", "Task 3", "Desc", 1).unwrap();

        repo.start_task("t-1", "w-1").unwrap();
        repo.complete_task("t-1", "hash").unwrap();

        repo.start_task("t-2", "w-2").unwrap();

        let pending = repo.tasks_by_status(TaskStatus::Pending);
        assert_eq!(pending.len(), 1);

        let in_progress = repo.tasks_by_status(TaskStatus::InProgress);
        assert_eq!(in_progress.len(), 1);

        let completed = repo.tasks_by_status(TaskStatus::Completed);
        assert_eq!(completed.len(), 1);
    }

    #[test]
    fn test_worker_lifecycle() {
        let repo = Repository::in_memory().unwrap();

        repo.spawn_worker("w-1", "t-1", "local").unwrap();

        let worker = repo.get_worker("w-1").unwrap();
        assert_eq!(worker.worker_type, "local");

        repo.complete_worker("w-1", "completed").unwrap();

        let worker = repo.get_worker("w-1").unwrap();
        assert_eq!(worker.current_task_id, None);
    }

    #[test]
    fn test_council_verdict() {
        let repo = Repository::in_memory().unwrap();

        repo.create_task("t-1", "Task 1", "Desc", 1).unwrap();
        repo.record_council_verdict("t-1", 0.95, 0.90, 0.85, 0.90, true)
            .unwrap();

        let verdicts = repo.verdicts_for_task("t-1");
        assert_eq!(verdicts.len(), 1);
        assert!(verdicts[0].passed);
    }

    #[test]
    fn test_gate_results() {
        let repo = Repository::in_memory().unwrap();

        // Passing gate
        repo.record_gate_result("gate-1", "t-1", 0.95, 0.90)
            .unwrap();

        // Failing gate
        repo.record_gate_result("gate-2", "t-1", 0.85, 0.90)
            .unwrap();

        let events = repo.all_events().unwrap();
        assert_eq!(events.len(), 2);

        assert!(matches!(events[0].event, AgentEvent::GatePassed { .. }));
        assert!(matches!(events[1].event, AgentEvent::GateFailed { .. }));
    }

    #[test]
    fn test_correlation_id() {
        let repo = Repository::in_memory().unwrap();

        repo.record_with_correlation(
            AgentEvent::TaskCreated {
                task_id: "t-1".to_string(),
                title: "Test".to_string(),
                description: "Test".to_string(),
                risk_tier: 1,
            },
            "request-123",
        )
        .unwrap();

        repo.record_with_correlation(
            AgentEvent::TaskStarted {
                task_id: "t-1".to_string(),
                worker_id: "w-1".to_string(),
            },
            "request-123",
        )
        .unwrap();

        let correlated = repo.events_by_correlation("request-123").unwrap();
        assert_eq!(correlated.len(), 2);
    }

    #[test]
    fn test_counts() {
        let repo = Repository::in_memory().unwrap();

        repo.create_task("t-1", "Task 1", "Desc", 1).unwrap();
        repo.create_task("t-2", "Task 2", "Desc", 1).unwrap();
        repo.complete_task("t-1", "hash").unwrap();

        let counts = repo.counts();
        assert_eq!(counts.total_tasks, 2);
        assert_eq!(counts.completed_tasks, 1);
        assert_eq!(counts.pending_tasks, 1);
    }

    #[test]
    fn test_verify_all() {
        let repo = Repository::in_memory().unwrap();

        repo.create_task("t-1", "Task 1", "Desc", 1).unwrap();
        repo.create_task("t-2", "Task 2", "Desc", 1).unwrap();

        assert!(repo.verify_all().unwrap());
    }
}
