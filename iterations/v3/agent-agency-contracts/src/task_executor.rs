//! Task Executor Interface
//!
//! Shared trait definition for task execution across orchestration and workers.
//! This breaks the circular dependency by providing a common interface that
//! orchestration can depend on without depending on the concrete implementation.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Result of task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutionResult {
    /// Unique execution identifier
    pub execution_id: Uuid,
    /// Task identifier
    pub task_id: Uuid,
    /// Whether execution was successful
    pub success: bool,
    /// Execution output/content
    pub output: String,
    /// Execution errors (if any)
    pub errors: Vec<String>,
    /// Execution metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Execution start time
    pub started_at: DateTime<Utc>,
    /// Execution completion time
    pub completed_at: DateTime<Utc>,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Worker that executed the task
    pub worker_id: Option<Uuid>,
}

/// Task execution specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSpec {
    /// Unique task identifier
    pub id: Uuid,
    /// Task description/title
    pub title: String,
    /// Task content/description
    pub description: String,
    /// Task priority
    pub priority: TaskPriority,
    /// Required capabilities
    pub required_capabilities: Vec<String>,
    /// Task context information
    pub context: HashMap<String, serde_json::Value>,
    /// Working specification ID
    pub working_spec_id: Option<String>,
    /// Execution timeout in seconds
    pub timeout_seconds: Option<u64>,
}

/// Task priority levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Task Executor trait
/// Provides the interface for executing tasks without depending on concrete implementations
#[async_trait]
pub trait TaskExecutor: Send + Sync + std::fmt::Debug {
    /// Execute a task with the given specification
    async fn execute_task(
        &self,
        task_spec: TaskSpec,
        worker_id: Uuid,
    ) -> Result<TaskExecutionResult, Box<dyn std::error::Error + Send + Sync>>;

    /// Execute a task with circuit breaker support
    async fn execute_task_with_circuit_breaker(
        &self,
        task_spec: TaskSpec,
        worker_id: Uuid,
        circuit_breaker_enabled: bool,
    ) -> Result<TaskExecutionResult, Box<dyn std::error::Error + Send + Sync>>;

    /// Get the health status of the task executor
    async fn health_check(&self) -> Result<TaskExecutorHealth, Box<dyn std::error::Error + Send + Sync>>;

    /// Get statistics about task execution
    async fn get_execution_stats(&self) -> Result<TaskExecutionStats, Box<dyn std::error::Error + Send + Sync>>;

    /// Cancel a task execution
    async fn cancel_task_execution(&self, task_id: Uuid, worker_id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Health status of the task executor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutorHealth {
    pub status: HealthStatus,
    pub last_execution_time: Option<DateTime<Utc>>,
    pub active_tasks: u32,
    pub queued_tasks: u32,
    pub total_executions: u64,
    pub success_rate: f64,
}

/// Health status enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Task execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutionStats {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub average_execution_time_ms: f64,
    pub median_execution_time_ms: f64,
    pub p95_execution_time_ms: f64,
    pub p99_execution_time_ms: f64,
}
