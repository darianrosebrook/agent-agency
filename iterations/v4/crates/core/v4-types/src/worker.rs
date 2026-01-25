//! Worker types
//!
//! Types for worker management and task execution.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Worker assignment
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkerAssignment {
    /// Worker ID
    pub worker_id: String,
    /// Task ID assigned
    pub task_id: String,
    /// Worker type
    pub worker_type: WorkerType,
    /// Assignment timestamp
    #[schemars(with = "String")]
    pub assigned_at: chrono::DateTime<chrono::Utc>,
}

/// Types of workers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum WorkerType {
    /// General purpose worker
    General,
    /// Code editing specialist
    CodeEditor,
    /// Test runner specialist
    TestRunner,
    /// Research/analysis specialist
    Research,
    /// File operations specialist
    FileOps,
}

impl WorkerType {
    /// Get capabilities for this worker type
    pub fn capabilities(&self) -> Vec<&'static str> {
        match self {
            Self::General => vec!["read", "write", "execute", "search"],
            Self::CodeEditor => vec!["read", "write", "diff", "refactor"],
            Self::TestRunner => vec!["read", "execute", "test", "coverage"],
            Self::Research => vec!["read", "search", "web", "analyze"],
            Self::FileOps => vec!["read", "write", "delete", "rename"],
        }
    }
}

/// Worker execution result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkerResult {
    /// Worker ID
    pub worker_id: String,
    /// Task ID
    pub task_id: String,
    /// Execution status
    pub status: WorkerStatus,
    /// Output data
    #[schemars(with = "Option<serde_json::Value>")]
    pub output: Option<serde_json::Value>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Completion timestamp
    #[schemars(with = "String")]
    pub completed_at: chrono::DateTime<chrono::Utc>,
}

/// Worker status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum WorkerStatus {
    /// Worker is idle
    Idle,
    /// Worker is executing a task
    Running,
    /// Worker completed successfully
    Completed,
    /// Worker failed
    Failed,
    /// Worker was cancelled
    Cancelled,
    /// Worker is blocked waiting
    Blocked,
}

impl WorkerStatus {
    /// Check if worker is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }

    /// Check if worker is active
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Running | Self::Blocked)
    }
}

/// Worker health metrics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkerHealth {
    /// Worker ID
    pub worker_id: String,
    /// Current status
    pub status: WorkerStatus,
    /// Tasks completed
    pub tasks_completed: u64,
    /// Tasks failed
    pub tasks_failed: u64,
    /// Average execution time in milliseconds
    pub avg_execution_time_ms: u64,
    /// Last heartbeat
    #[schemars(with = "String")]
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,
    /// Memory usage in bytes
    pub memory_bytes: Option<u64>,
    /// CPU usage percentage
    pub cpu_percent: Option<f64>,
}

impl WorkerHealth {
    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        let total = self.tasks_completed + self.tasks_failed;
        if total == 0 {
            1.0
        } else {
            self.tasks_completed as f64 / total as f64
        }
    }

    /// Check if worker is healthy
    pub fn is_healthy(&self) -> bool {
        let now = chrono::Utc::now();
        let since_heartbeat = now.signed_duration_since(self.last_heartbeat);

        // Healthy if: heartbeat within 30 seconds, success rate > 90%, not failed
        since_heartbeat.num_seconds() < 30
            && self.success_rate() >= 0.90
            && !matches!(self.status, WorkerStatus::Failed)
    }
}

/// Worker pool statistics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkerPoolStats {
    /// Total workers in pool
    pub total_workers: u32,
    /// Active workers
    pub active_workers: u32,
    /// Idle workers
    pub idle_workers: u32,
    /// Failed workers
    pub failed_workers: u32,
    /// Tasks in queue
    pub queued_tasks: u32,
    /// Average queue wait time in milliseconds
    pub avg_queue_wait_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_type_capabilities() {
        let general = WorkerType::General;
        assert!(general.capabilities().contains(&"read"));
        assert!(general.capabilities().contains(&"write"));

        let test_runner = WorkerType::TestRunner;
        assert!(test_runner.capabilities().contains(&"test"));
        assert!(test_runner.capabilities().contains(&"coverage"));
    }

    #[test]
    fn test_worker_status_terminal() {
        assert!(WorkerStatus::Completed.is_terminal());
        assert!(WorkerStatus::Failed.is_terminal());
        assert!(!WorkerStatus::Running.is_terminal());
        assert!(!WorkerStatus::Idle.is_terminal());
    }

    #[test]
    fn test_worker_health_success_rate() {
        let health = WorkerHealth {
            worker_id: "w1".to_string(),
            status: WorkerStatus::Idle,
            tasks_completed: 90,
            tasks_failed: 10,
            avg_execution_time_ms: 100,
            last_heartbeat: chrono::Utc::now(),
            memory_bytes: None,
            cpu_percent: None,
        };
        assert!((health.success_rate() - 0.90).abs() < 0.001);
    }
}
