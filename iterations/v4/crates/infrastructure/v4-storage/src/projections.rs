//! Projections
//!
//! Read models built from event streams.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::events::{AgentEvent, SequenceNumber, StoredEvent};

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

/// Task projection - current state of a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProjection {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub risk_tier: u8,
    pub worker_id: Option<String>,
    pub subtask_ids: Vec<String>,
    pub result_hash: Option<String>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: SequenceNumber,
}

impl TaskProjection {
    /// Apply an event to update the projection
    pub fn apply(&mut self, event: &StoredEvent) {
        match &event.event {
            AgentEvent::TaskDecomposed { subtask_ids, .. } => {
                self.subtask_ids = subtask_ids.clone();
                self.updated_at = event.timestamp;
                self.version = event.sequence;
            }
            AgentEvent::TaskStarted { worker_id, .. } => {
                self.status = TaskStatus::InProgress;
                self.worker_id = Some(worker_id.clone());
                self.updated_at = event.timestamp;
                self.version = event.sequence;
            }
            AgentEvent::TaskCompleted { result_hash, .. } => {
                self.status = TaskStatus::Completed;
                self.result_hash = Some(result_hash.clone());
                self.updated_at = event.timestamp;
                self.version = event.sequence;
            }
            AgentEvent::TaskFailed { error, .. } => {
                self.status = TaskStatus::Failed;
                self.error = Some(error.clone());
                self.updated_at = event.timestamp;
                self.version = event.sequence;
            }
            _ => {}
        }
    }

    /// Create from a TaskCreated event
    pub fn from_created(event: &StoredEvent) -> Option<Self> {
        if let AgentEvent::TaskCreated {
            task_id,
            title,
            description,
            risk_tier,
        } = &event.event
        {
            Some(Self {
                id: task_id.clone(),
                title: title.clone(),
                description: description.clone(),
                status: TaskStatus::Pending,
                risk_tier: *risk_tier,
                worker_id: None,
                subtask_ids: Vec::new(),
                result_hash: None,
                error: None,
                created_at: event.timestamp,
                updated_at: event.timestamp,
                version: event.sequence,
            })
        } else {
            None
        }
    }
}

/// Worker status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerStatus {
    Idle,
    Working,
    Completed,
    Failed,
}

impl std::fmt::Display for WorkerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "idle"),
            Self::Working => write!(f, "working"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

/// Worker projection - current state of a worker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerProjection {
    pub id: String,
    pub worker_type: String,
    pub status: WorkerStatus,
    pub current_task_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: SequenceNumber,
}

impl WorkerProjection {
    /// Apply an event to update the projection
    pub fn apply(&mut self, event: &StoredEvent) {
        if let AgentEvent::WorkerCompleted { status, .. } = &event.event {
            self.status = match status.as_str() {
                "completed" => WorkerStatus::Completed,
                "failed" => WorkerStatus::Failed,
                _ => WorkerStatus::Idle,
            };
            self.current_task_id = None;
            self.updated_at = event.timestamp;
            self.version = event.sequence;
        }
    }

    /// Create from a WorkerSpawned event
    pub fn from_spawned(event: &StoredEvent) -> Option<Self> {
        if let AgentEvent::WorkerSpawned {
            worker_id,
            task_id,
            worker_type,
        } = &event.event
        {
            Some(Self {
                id: worker_id.clone(),
                worker_type: worker_type.clone(),
                status: WorkerStatus::Working,
                current_task_id: Some(task_id.clone()),
                created_at: event.timestamp,
                updated_at: event.timestamp,
                version: event.sequence,
            })
        } else {
            None
        }
    }
}

/// Council verdict projection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilVerdictProjection {
    pub task_id: String,
    pub constitutional_score: f64,
    pub technical_score: f64,
    pub quality_score: f64,
    pub aggregate_score: f64,
    pub passed: bool,
    pub timestamp: DateTime<Utc>,
}

impl CouncilVerdictProjection {
    pub fn from_event(event: &StoredEvent) -> Option<Self> {
        if let AgentEvent::CouncilConvened {
            task_id,
            constitutional_score,
            technical_score,
            quality_score,
            aggregate_score,
            passed,
        } = &event.event
        {
            Some(Self {
                task_id: task_id.clone(),
                constitutional_score: *constitutional_score,
                technical_score: *technical_score,
                quality_score: *quality_score,
                aggregate_score: *aggregate_score,
                passed: *passed,
                timestamp: event.timestamp,
            })
        } else {
            None
        }
    }
}

/// Projection store - in-memory cache of projections
pub struct ProjectionStore {
    tasks: HashMap<String, TaskProjection>,
    workers: HashMap<String, WorkerProjection>,
    verdicts: Vec<CouncilVerdictProjection>,
    last_sequence: SequenceNumber,
}

impl ProjectionStore {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            workers: HashMap::new(),
            verdicts: Vec::new(),
            last_sequence: SequenceNumber(0),
        }
    }

    /// Apply an event to update projections
    pub fn apply(&mut self, event: &StoredEvent) {
        match &event.event {
            AgentEvent::TaskCreated { task_id, .. } => {
                if let Some(projection) = TaskProjection::from_created(event) {
                    self.tasks.insert(task_id.clone(), projection);
                }
            }
            AgentEvent::TaskDecomposed { task_id, .. }
            | AgentEvent::TaskStarted { task_id, .. }
            | AgentEvent::TaskCompleted { task_id, .. }
            | AgentEvent::TaskFailed { task_id, .. } => {
                if let Some(projection) = self.tasks.get_mut(task_id) {
                    projection.apply(event);
                }
            }
            AgentEvent::WorkerSpawned { worker_id, .. } => {
                if let Some(projection) = WorkerProjection::from_spawned(event) {
                    self.workers.insert(worker_id.clone(), projection);
                }
            }
            AgentEvent::WorkerCompleted { worker_id, .. } => {
                if let Some(projection) = self.workers.get_mut(worker_id) {
                    projection.apply(event);
                }
            }
            AgentEvent::CouncilConvened { .. } => {
                if let Some(verdict) = CouncilVerdictProjection::from_event(event) {
                    self.verdicts.push(verdict);
                }
            }
            _ => {}
        }

        self.last_sequence = event.sequence;
    }

    /// Apply multiple events
    pub fn apply_all(&mut self, events: &[StoredEvent]) {
        for event in events {
            self.apply(event);
        }
    }

    /// Get a task by ID
    pub fn get_task(&self, id: &str) -> Option<&TaskProjection> {
        self.tasks.get(id)
    }

    /// Get all tasks
    pub fn all_tasks(&self) -> Vec<&TaskProjection> {
        self.tasks.values().collect()
    }

    /// Get tasks by status
    pub fn tasks_by_status(&self, status: TaskStatus) -> Vec<&TaskProjection> {
        self.tasks.values().filter(|t| t.status == status).collect()
    }

    /// Get a worker by ID
    pub fn get_worker(&self, id: &str) -> Option<&WorkerProjection> {
        self.workers.get(id)
    }

    /// Get all workers
    pub fn all_workers(&self) -> Vec<&WorkerProjection> {
        self.workers.values().collect()
    }

    /// Get verdicts for a task
    pub fn verdicts_for_task(&self, task_id: &str) -> Vec<&CouncilVerdictProjection> {
        self.verdicts.iter().filter(|v| v.task_id == task_id).collect()
    }

    /// Get the last processed sequence number
    pub fn last_sequence(&self) -> SequenceNumber {
        self.last_sequence
    }

    /// Get counts for dashboard
    pub fn counts(&self) -> ProjectionCounts {
        ProjectionCounts {
            total_tasks: self.tasks.len(),
            pending_tasks: self.tasks_by_status(TaskStatus::Pending).len(),
            in_progress_tasks: self.tasks_by_status(TaskStatus::InProgress).len(),
            completed_tasks: self.tasks_by_status(TaskStatus::Completed).len(),
            failed_tasks: self.tasks_by_status(TaskStatus::Failed).len(),
            total_workers: self.workers.len(),
            total_verdicts: self.verdicts.len(),
        }
    }
}

impl Default for ProjectionStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary counts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionCounts {
    pub total_tasks: usize,
    pub pending_tasks: usize,
    pub in_progress_tasks: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub total_workers: usize,
    pub total_verdicts: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::StoredEvent;

    fn make_task_created(task_id: &str, title: &str) -> StoredEvent {
        StoredEvent::new(
            AgentEvent::TaskCreated {
                task_id: task_id.to_string(),
                title: title.to_string(),
                description: "Test".to_string(),
                risk_tier: 1,
            },
            SequenceNumber(1),
        )
    }

    #[test]
    fn test_task_projection_creation() {
        let event = make_task_created("t-1", "Test Task");
        let projection = TaskProjection::from_created(&event).unwrap();

        assert_eq!(projection.id, "t-1");
        assert_eq!(projection.title, "Test Task");
        assert_eq!(projection.status, TaskStatus::Pending);
    }

    #[test]
    fn test_task_projection_apply() {
        let created = make_task_created("t-1", "Test Task");
        let mut projection = TaskProjection::from_created(&created).unwrap();

        let started = StoredEvent::new(
            AgentEvent::TaskStarted {
                task_id: "t-1".to_string(),
                worker_id: "w-1".to_string(),
            },
            SequenceNumber(2),
        );

        projection.apply(&started);
        assert_eq!(projection.status, TaskStatus::InProgress);
        assert_eq!(projection.worker_id, Some("w-1".to_string()));

        let completed = StoredEvent::new(
            AgentEvent::TaskCompleted {
                task_id: "t-1".to_string(),
                result_hash: "abc123".to_string(),
            },
            SequenceNumber(3),
        );

        projection.apply(&completed);
        assert_eq!(projection.status, TaskStatus::Completed);
        assert_eq!(projection.result_hash, Some("abc123".to_string()));
    }

    #[test]
    fn test_projection_store() {
        let mut store = ProjectionStore::new();

        let events = vec![
            StoredEvent::new(
                AgentEvent::TaskCreated {
                    task_id: "t-1".to_string(),
                    title: "Task 1".to_string(),
                    description: "Test".to_string(),
                    risk_tier: 1,
                },
                SequenceNumber(1),
            ),
            StoredEvent::new(
                AgentEvent::TaskCreated {
                    task_id: "t-2".to_string(),
                    title: "Task 2".to_string(),
                    description: "Test".to_string(),
                    risk_tier: 2,
                },
                SequenceNumber(2),
            ),
            StoredEvent::new(
                AgentEvent::TaskStarted {
                    task_id: "t-1".to_string(),
                    worker_id: "w-1".to_string(),
                },
                SequenceNumber(3),
            ),
        ];

        store.apply_all(&events);

        assert_eq!(store.all_tasks().len(), 2);
        assert_eq!(store.tasks_by_status(TaskStatus::Pending).len(), 1);
        assert_eq!(store.tasks_by_status(TaskStatus::InProgress).len(), 1);

        let t1 = store.get_task("t-1").unwrap();
        assert_eq!(t1.status, TaskStatus::InProgress);

        let t2 = store.get_task("t-2").unwrap();
        assert_eq!(t2.status, TaskStatus::Pending);
    }

    #[test]
    fn test_projection_counts() {
        let mut store = ProjectionStore::new();

        store.apply(&StoredEvent::new(
            AgentEvent::TaskCreated {
                task_id: "t-1".to_string(),
                title: "Task 1".to_string(),
                description: "Test".to_string(),
                risk_tier: 1,
            },
            SequenceNumber(1),
        ));

        store.apply(&StoredEvent::new(
            AgentEvent::TaskCompleted {
                task_id: "t-1".to_string(),
                result_hash: "abc".to_string(),
            },
            SequenceNumber(2),
        ));

        let counts = store.counts();
        assert_eq!(counts.total_tasks, 1);
        assert_eq!(counts.completed_tasks, 1);
        assert_eq!(counts.pending_tasks, 0);
    }

    #[test]
    fn test_council_verdict_projection() {
        let mut store = ProjectionStore::new();

        store.apply(&StoredEvent::new(
            AgentEvent::CouncilConvened {
                task_id: "t-1".to_string(),
                constitutional_score: 0.95,
                technical_score: 0.90,
                quality_score: 0.85,
                aggregate_score: 0.90,
                passed: true,
            },
            SequenceNumber(1),
        ));

        let verdicts = store.verdicts_for_task("t-1");
        assert_eq!(verdicts.len(), 1);
        assert!(verdicts[0].passed);
        assert_eq!(verdicts[0].aggregate_score, 0.90);
    }
}
