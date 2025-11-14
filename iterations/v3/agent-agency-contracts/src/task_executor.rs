//! Task Executor Interface
//!
//! Shared trait definition for task execution across orchestration and workers.
//! This breaks the circular dependency by providing a common interface that
//! orchestration can depend on without depending on the concrete implementation.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Result of task execution
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskExecutionResult {
    /// Unique execution identifier
    #[schemars(with = "String")]
    pub execution_id: Uuid,
    /// Task identifier
    #[schemars(with = "String")]
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
    #[schemars(with = "String")]
    pub started_at: DateTime<Utc>,
    /// Execution completion time
    #[schemars(with = "String")]
    pub completed_at: DateTime<Utc>,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Worker that executed the task
    #[schemars(with = "Option<String>")]
    pub worker_id: Option<Uuid>,
}

/// Task execution specification
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskSpec {
    /// Unique task identifier
    #[schemars(with = "String")]
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
    /// Task scope definition
    pub scope: Option<TaskScope>,
    /// Risk tier (1=critical, 2=standard, 3=low)
    pub risk_tier: Option<u32>,
    /// Acceptance criteria in Given-When-Then format
    pub acceptance_criteria: Option<Vec<crate::types::execution::AcceptanceCriterion>>,
    /// CAWS specification
    /// TODO: Use proper CAWS spec type instead of generic HashMap:
    /// 1. Type definition: Define proper CAWS spec type
    ///    - Create CAWS spec struct or enum
    ///    - Resolve circular dependency issues
    ///    - Support CAWS spec serialization
    /// 2. Type integration: Integrate CAWS spec type
    ///    - Replace HashMap with proper type
    ///    - Update all CAWS spec usages
    ///    - Handle type conversion appropriately
    /// 3. Dependency resolution: Resolve circular dependencies
    ///    - Refactor to break circular dependencies
    ///    - Use trait objects or type erasure if needed
    ///    - Support proper type relationships
    ///
    /// ACCEPTANCE CRITERIA:
    /// - CAWS spec uses proper type instead of HashMap
    /// - Circular dependencies are resolved
    /// - Type safety is improved
    ///
    /// DEPENDENCIES:
    /// - CAWS spec type definition (Required)
    /// - Dependency refactoring (Required)
    ///
    /// PRIORITY: Medium
    pub caws_spec: Option<HashMap<String, serde_json::Value>>,
    /// Task execution requirements
    pub requirements: Option<TaskRequirements>,
}

// Use the unified TaskPriority from types/planning.rs
pub use crate::types::planning::TaskPriority;

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
    async fn health_check(
        &self,
    ) -> Result<TaskExecutorHealth, Box<dyn std::error::Error + Send + Sync>>;

    /// Get statistics about task execution
    async fn get_execution_stats(
        &self,
    ) -> Result<TaskExecutionStats, Box<dyn std::error::Error + Send + Sync>>;

    /// Cancel a task execution
    async fn cancel_task_execution(
        &self,
        task_id: Uuid,
        worker_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Health status of the task executor
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskExecutorHealth {
    pub status: HealthStatus,
    #[schemars(with = "String")]
    pub last_execution_time: Option<DateTime<Utc>>,
    pub active_tasks: u32,
    pub queued_tasks: u32,
    pub total_executions: u64,
    pub success_rate: f64,
}

/// Health status enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Task execution statistics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskExecutionStats {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub average_execution_time_ms: f64,
    pub median_execution_time_ms: f64,
    pub p95_execution_time_ms: f64,
    pub p99_execution_time_ms: f64,
}

/// Task requirements for execution
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskRequirements {
    pub required_languages: Vec<String>,
    pub required_frameworks: Vec<String>,
    pub required_domains: Vec<String>,
    pub min_quality_score: f32,
    pub min_caws_awareness: f32,
    pub max_execution_time_ms: Option<u64>,
    pub preferred_worker_type: Option<String>,
    pub context_length_estimate: usize,
}

/// Task context for execution
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskContext {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    #[schemars(with = "String")]
    pub worker_id: Uuid,
    #[schemars(with = "String")]
    pub start_time: DateTime<Utc>,
    pub timeout_ms: u64,
    pub retry_count: u32,
    pub max_retries: u32,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Task scope definition
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskScope {
    pub domains: Vec<String>,
    pub files_affected: Vec<String>,
    pub max_loc: Option<u32>,
}

/// Execution status for tasks
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Timeout,
}

/// Task execution progress tracking
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Progress {
    /// Completion percentage (0.0 to 100.0)
    pub percentage: f64,
    /// Number of completed subtasks
    pub completed_subtasks: usize,
    /// Total number of subtasks
    pub total_subtasks: usize,
    /// Number of active workers
    pub active_workers: usize,
    /// Number of blocked workers
    pub blocked_workers: usize,
    /// Number of failed workers
    pub failed_workers: usize,
    /// Last progress update timestamp
    #[schemars(with = "String")]
    pub last_update: chrono::DateTime<chrono::Utc>,
    /// Current status
    pub status: ExecutionStatus,
    /// Additional progress metadata
    pub metadata: HashMap<String, serde_json::Value>,
}
