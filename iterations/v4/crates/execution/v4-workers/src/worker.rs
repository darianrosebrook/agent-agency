//! Worker Types and State
//!
//! Core worker abstraction and state management.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Worker identifier
pub type WorkerId = String;

/// Worker state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerState {
    /// Worker is idle and ready for tasks
    Idle,
    /// Worker is currently executing a task
    Busy,
    /// Worker is paused
    Paused,
    /// Worker has encountered an error
    Error,
    /// Worker is shutting down
    ShuttingDown,
    /// Worker has stopped
    Stopped,
}

impl WorkerState {
    pub fn is_available(&self) -> bool {
        matches!(self, Self::Idle)
    }

    pub fn is_active(&self) -> bool {
        matches!(self, Self::Idle | Self::Busy | Self::Paused)
    }
}

/// Worker type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerType {
    /// Standard local worker
    Local,
    /// Sandboxed worker with restricted permissions
    Sandboxed,
    /// GPU-accelerated worker
    GpuAccelerated,
}

impl WorkerType {
    pub fn description(&self) -> &'static str {
        match self {
            Self::Local => "Local execution with standard permissions",
            Self::Sandboxed => "Sandboxed execution with restricted permissions",
            Self::GpuAccelerated => "GPU-accelerated execution",
        }
    }
}

/// Worker statistics
#[derive(Debug, Default)]
pub struct WorkerStats {
    /// Total tasks completed
    tasks_completed: AtomicU64,
    /// Total tasks failed
    tasks_failed: AtomicU64,
    /// Total execution time in milliseconds
    total_execution_ms: AtomicU64,
}

impl WorkerStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_success(&self, execution_ms: u64) {
        self.tasks_completed.fetch_add(1, Ordering::Relaxed);
        self.total_execution_ms.fetch_add(execution_ms, Ordering::Relaxed);
    }

    pub fn record_failure(&self) {
        self.tasks_failed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn completed(&self) -> u64 {
        self.tasks_completed.load(Ordering::Relaxed)
    }

    pub fn failed(&self) -> u64 {
        self.tasks_failed.load(Ordering::Relaxed)
    }

    pub fn total_ms(&self) -> u64 {
        self.total_execution_ms.load(Ordering::Relaxed)
    }

    pub fn average_ms(&self) -> f64 {
        let completed = self.completed();
        if completed == 0 {
            0.0
        } else {
            self.total_ms() as f64 / completed as f64
        }
    }
}

/// Worker metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerInfo {
    /// Worker ID
    pub id: WorkerId,
    /// Worker type
    pub worker_type: WorkerType,
    /// Current state
    pub state: WorkerState,
    /// Current task ID (if busy)
    pub current_task: Option<String>,
    /// When the worker was created
    pub created_at: DateTime<Utc>,
    /// When the worker last became idle
    pub last_idle_at: Option<DateTime<Utc>>,
    /// Tasks completed
    pub tasks_completed: u64,
    /// Tasks failed
    pub tasks_failed: u64,
}

/// Worker handle for controlling a worker
#[derive(Clone)]
pub struct WorkerHandle {
    pub id: WorkerId,
    pub worker_type: WorkerType,
    pub state: Arc<std::sync::RwLock<WorkerState>>,
    pub stats: Arc<WorkerStats>,
    pub created_at: DateTime<Utc>,
    current_task: Arc<std::sync::RwLock<Option<String>>>,
    last_idle: Arc<std::sync::RwLock<Option<DateTime<Utc>>>>,
}

impl WorkerHandle {
    /// Create a new worker handle
    pub fn new(id: impl Into<String>, worker_type: WorkerType) -> Self {
        Self {
            id: id.into(),
            worker_type,
            state: Arc::new(std::sync::RwLock::new(WorkerState::Idle)),
            stats: Arc::new(WorkerStats::new()),
            created_at: Utc::now(),
            current_task: Arc::new(std::sync::RwLock::new(None)),
            last_idle: Arc::new(std::sync::RwLock::new(Some(Utc::now()))),
        }
    }

    /// Get current state
    pub fn state(&self) -> WorkerState {
        *self.state.read().unwrap()
    }

    /// Set state
    pub fn set_state(&self, state: WorkerState) {
        *self.state.write().unwrap() = state;
        if state == WorkerState::Idle {
            *self.last_idle.write().unwrap() = Some(Utc::now());
        }
    }

    /// Check if worker is available
    pub fn is_available(&self) -> bool {
        self.state().is_available()
    }

    /// Mark worker as busy with a task
    pub fn start_task(&self, task_id: &str) {
        self.set_state(WorkerState::Busy);
        *self.current_task.write().unwrap() = Some(task_id.to_string());
    }

    /// Mark task as complete
    pub fn complete_task(&self, execution_ms: u64) {
        self.stats.record_success(execution_ms);
        *self.current_task.write().unwrap() = None;
        self.set_state(WorkerState::Idle);
    }

    /// Mark task as failed
    pub fn fail_task(&self) {
        self.stats.record_failure();
        *self.current_task.write().unwrap() = None;
        self.set_state(WorkerState::Idle);
    }

    /// Get current task ID
    pub fn current_task(&self) -> Option<String> {
        self.current_task.read().unwrap().clone()
    }

    /// Get worker info
    pub fn info(&self) -> WorkerInfo {
        WorkerInfo {
            id: self.id.clone(),
            worker_type: self.worker_type,
            state: self.state(),
            current_task: self.current_task(),
            created_at: self.created_at,
            last_idle_at: *self.last_idle.read().unwrap(),
            tasks_completed: self.stats.completed(),
            tasks_failed: self.stats.failed(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_state() {
        assert!(WorkerState::Idle.is_available());
        assert!(!WorkerState::Busy.is_available());
        assert!(WorkerState::Idle.is_active());
        assert!(WorkerState::Busy.is_active());
        assert!(!WorkerState::Stopped.is_active());
    }

    #[test]
    fn test_worker_stats() {
        let stats = WorkerStats::new();

        stats.record_success(100);
        stats.record_success(200);
        stats.record_failure();

        assert_eq!(stats.completed(), 2);
        assert_eq!(stats.failed(), 1);
        assert_eq!(stats.total_ms(), 300);
        assert_eq!(stats.average_ms(), 150.0);
    }

    #[test]
    fn test_worker_handle() {
        let handle = WorkerHandle::new("worker-1", WorkerType::Local);

        assert!(handle.is_available());
        assert_eq!(handle.state(), WorkerState::Idle);

        handle.start_task("task-1");
        assert!(!handle.is_available());
        assert_eq!(handle.current_task(), Some("task-1".to_string()));

        handle.complete_task(100);
        assert!(handle.is_available());
        assert_eq!(handle.current_task(), None);
        assert_eq!(handle.stats.completed(), 1);
    }

    #[test]
    fn test_worker_info() {
        let handle = WorkerHandle::new("worker-2", WorkerType::Sandboxed);
        let info = handle.info();

        assert_eq!(info.id, "worker-2");
        assert_eq!(info.worker_type, WorkerType::Sandboxed);
        assert_eq!(info.state, WorkerState::Idle);
    }
}
