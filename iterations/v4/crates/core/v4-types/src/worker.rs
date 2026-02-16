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

/// Types of workers — execution environment classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum WorkerType {
    /// Local execution with standard permissions
    Local,
    /// Isolated sandbox with restricted permissions
    Sandboxed,
    /// Remote A2A worker at a URL
    Remote { url: String },
    /// GPU-accelerated execution
    GpuAccelerated,
    /// Requires human review before execution
    ManualReview,
}

impl WorkerType {
    /// Get a human-readable description of this worker type
    pub fn description(&self) -> &'static str {
        match self {
            Self::Local => "Local execution with standard permissions",
            Self::Sandboxed => "Isolated sandbox with restricted permissions",
            Self::Remote { .. } => "Remote A2A worker",
            Self::GpuAccelerated => "GPU-accelerated execution",
            Self::ManualReview => "Requires human review before execution",
        }
    }

    /// Check if this is a remote worker
    pub fn is_remote(&self) -> bool {
        matches!(self, Self::Remote { .. })
    }
}

/// Worker capability — specialist classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum WorkerCapability {
    /// General purpose
    General,
    /// Code editing and refactoring
    CodeEditing,
    /// Test running and coverage
    TestRunning,
    /// Research and analysis
    Research,
    /// File system operations
    FileOperations,
    /// Content generation (via LLM)
    ContentGeneration,
}

impl WorkerCapability {
    /// Get string identifier for this capability
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::General => "general",
            Self::CodeEditing => "code-editing",
            Self::TestRunning => "test-running",
            Self::Research => "research",
            Self::FileOperations => "file-operations",
            Self::ContentGeneration => "content-generation",
        }
    }
}

/// Worker profile combining type and capabilities
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct WorkerProfile {
    /// Execution environment type
    pub worker_type: WorkerType,
    /// Specialist capabilities
    pub capabilities: Vec<WorkerCapability>,
    /// Optional description
    pub description: Option<String>,
}

impl WorkerProfile {
    /// Create a new worker profile
    pub fn new(worker_type: WorkerType) -> Self {
        Self {
            worker_type,
            capabilities: Vec::new(),
            description: None,
        }
    }

    /// Add a capability
    pub fn with_capability(mut self, cap: WorkerCapability) -> Self {
        self.capabilities.push(cap);
        self
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
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
    use chrono::{Duration, Utc};

    #[test]
    fn test_worker_type_descriptions() {
        assert!(!WorkerType::Local.description().is_empty());
        assert!(!WorkerType::Sandboxed.description().is_empty());
        assert!(!WorkerType::Remote { url: "http://localhost".to_string() }.description().is_empty());
        assert!(!WorkerType::GpuAccelerated.description().is_empty());
        assert!(!WorkerType::ManualReview.description().is_empty());
    }

    #[test]
    fn test_worker_type_is_remote() {
        assert!(!WorkerType::Local.is_remote());
        assert!(!WorkerType::Sandboxed.is_remote());
        assert!(WorkerType::Remote { url: "http://localhost:3010".to_string() }.is_remote());
        assert!(!WorkerType::GpuAccelerated.is_remote());
        assert!(!WorkerType::ManualReview.is_remote());
    }

    #[test]
    fn test_worker_capability() {
        assert_eq!(WorkerCapability::General.as_str(), "general");
        assert_eq!(WorkerCapability::ContentGeneration.as_str(), "content-generation");
    }

    #[test]
    fn test_worker_profile() {
        let profile = WorkerProfile::new(WorkerType::Remote { url: "http://localhost:3010".to_string() })
            .with_capability(WorkerCapability::ContentGeneration)
            .with_capability(WorkerCapability::Research)
            .with_description("MiniMax M2.5 worker");

        assert!(profile.worker_type.is_remote());
        assert_eq!(profile.capabilities.len(), 2);
        assert!(profile.description.is_some());
    }

    #[test]
    fn test_worker_status_terminal() {
        assert!(WorkerStatus::Completed.is_terminal());
        assert!(WorkerStatus::Failed.is_terminal());
        assert!(WorkerStatus::Cancelled.is_terminal());
        assert!(!WorkerStatus::Running.is_terminal());
        assert!(!WorkerStatus::Idle.is_terminal());
        assert!(!WorkerStatus::Blocked.is_terminal());
    }

    #[test]
    fn test_worker_status_is_active() {
        // Active states
        assert!(WorkerStatus::Running.is_active());
        assert!(WorkerStatus::Blocked.is_active());

        // Non-active states
        assert!(!WorkerStatus::Idle.is_active());
        assert!(!WorkerStatus::Completed.is_active());
        assert!(!WorkerStatus::Failed.is_active());
        assert!(!WorkerStatus::Cancelled.is_active());
    }

    #[test]
    fn test_worker_health_success_rate() {
        let health = WorkerHealth {
            worker_id: "w1".to_string(),
            status: WorkerStatus::Idle,
            tasks_completed: 90,
            tasks_failed: 10,
            avg_execution_time_ms: 100,
            last_heartbeat: Utc::now(),
            memory_bytes: None,
            cpu_percent: None,
        };
        assert!((health.success_rate() - 0.90).abs() < 0.001);
    }

    #[test]
    fn test_worker_health_success_rate_zero_tasks() {
        let health = WorkerHealth {
            worker_id: "w1".to_string(),
            status: WorkerStatus::Idle,
            tasks_completed: 0,
            tasks_failed: 0,
            avg_execution_time_ms: 0,
            last_heartbeat: Utc::now(),
            memory_bytes: None,
            cpu_percent: None,
        };
        // Should return 1.0 (100%) when no tasks have been run
        assert!((health.success_rate() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_worker_health_is_healthy_all_conditions_pass() {
        // Healthy worker: recent heartbeat, good success rate, not failed
        let health = WorkerHealth {
            worker_id: "w1".to_string(),
            status: WorkerStatus::Idle,
            tasks_completed: 95,
            tasks_failed: 5, // 95% success rate
            avg_execution_time_ms: 100,
            last_heartbeat: Utc::now(), // Recent heartbeat
            memory_bytes: None,
            cpu_percent: None,
        };
        assert!(health.is_healthy());
    }

    #[test]
    fn test_worker_health_is_healthy_old_heartbeat() {
        // Old heartbeat (> 30 seconds ago) - should be unhealthy
        let health = WorkerHealth {
            worker_id: "w1".to_string(),
            status: WorkerStatus::Idle,
            tasks_completed: 100,
            tasks_failed: 0,
            avg_execution_time_ms: 100,
            last_heartbeat: Utc::now() - Duration::seconds(35),
            memory_bytes: None,
            cpu_percent: None,
        };
        assert!(!health.is_healthy());
    }

    #[test]
    fn test_worker_health_is_healthy_heartbeat_boundary() {
        // Heartbeat exactly at 30 seconds should be healthy (< 30, not <=)
        let health_at_29 = WorkerHealth {
            worker_id: "w1".to_string(),
            status: WorkerStatus::Idle,
            tasks_completed: 100,
            tasks_failed: 0,
            avg_execution_time_ms: 100,
            last_heartbeat: Utc::now() - Duration::seconds(29),
            memory_bytes: None,
            cpu_percent: None,
        };
        assert!(health_at_29.is_healthy());

        // At exactly 30 seconds should be unhealthy
        let health_at_30 = WorkerHealth {
            worker_id: "w1".to_string(),
            status: WorkerStatus::Idle,
            tasks_completed: 100,
            tasks_failed: 0,
            avg_execution_time_ms: 100,
            last_heartbeat: Utc::now() - Duration::seconds(30),
            memory_bytes: None,
            cpu_percent: None,
        };
        assert!(!health_at_30.is_healthy());
    }

    #[test]
    fn test_worker_health_is_healthy_low_success_rate() {
        // Success rate below 90% - should be unhealthy
        let health = WorkerHealth {
            worker_id: "w1".to_string(),
            status: WorkerStatus::Idle,
            tasks_completed: 80,
            tasks_failed: 20, // 80% success rate
            avg_execution_time_ms: 100,
            last_heartbeat: Utc::now(),
            memory_bytes: None,
            cpu_percent: None,
        };
        assert!(!health.is_healthy());
    }

    #[test]
    fn test_worker_health_is_healthy_success_rate_boundary() {
        // Exactly 90% success rate should pass
        let health_at_90 = WorkerHealth {
            worker_id: "w1".to_string(),
            status: WorkerStatus::Idle,
            tasks_completed: 90,
            tasks_failed: 10,
            avg_execution_time_ms: 100,
            last_heartbeat: Utc::now(),
            memory_bytes: None,
            cpu_percent: None,
        };
        assert!(health_at_90.is_healthy());

        // Just below 90% should fail
        let health_below_90 = WorkerHealth {
            worker_id: "w1".to_string(),
            status: WorkerStatus::Idle,
            tasks_completed: 89,
            tasks_failed: 11,
            avg_execution_time_ms: 100,
            last_heartbeat: Utc::now(),
            memory_bytes: None,
            cpu_percent: None,
        };
        assert!(!health_below_90.is_healthy());
    }

    #[test]
    fn test_worker_health_is_healthy_failed_status() {
        // Failed status - should be unhealthy even with good metrics
        let health = WorkerHealth {
            worker_id: "w1".to_string(),
            status: WorkerStatus::Failed,
            tasks_completed: 100,
            tasks_failed: 0,
            avg_execution_time_ms: 100,
            last_heartbeat: Utc::now(),
            memory_bytes: None,
            cpu_percent: None,
        };
        assert!(!health.is_healthy());
    }
}
