//! @darianrosebrook
//! Shared worker type definitions for interoperability contracts.
//!
//! These types are used across multiple crates (workers, orchestration, council)
//! and are defined here to avoid circular dependencies.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::router_decision::WorkerType;

/// Worker specialty types for task routing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerSpecialty {
    /// Compilation error fixing
    CompilationErrors { error_codes: Vec<String> },
    /// Code refactoring and restructuring
    Refactoring { strategies: Vec<String> },
    /// Testing framework expertise
    Testing { frameworks: Vec<String> },
    /// Documentation generation and maintenance
    Documentation { formats: Vec<String> },
    /// Type system and generics expertise
    TypeSystem { domains: Vec<String> },
    /// Async patterns and concurrency
    AsyncPatterns { patterns: Vec<String> },
    /// Custom domain expertise
    Custom { domain: String, capabilities: Vec<String> },
}

/// Trait for specialized workers
#[async_trait::async_trait]
pub trait SpecializedWorker: Send + Sync + std::fmt::Debug {
    /// Check if this worker has the specified specialty
    fn has_specialty(&self, specialty: &WorkerSpecialty) -> bool;

    /// Execute a subtask within this worker's specialty
    async fn execute_subtask(&self, context: WorkerContext) -> Result<WorkerResult, Box<dyn std::error::Error + Send + Sync>>;
}


/// Context for worker task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerContext {
    /// Unique task identifier
    pub task_id: Uuid,
    /// Task description
    pub description: String,
    /// Required capabilities
    pub required_capabilities: Vec<String>,
    /// Task priority
    pub priority: TaskPriority,
    /// Working specification ID
    pub working_spec_id: String,
    /// Task-specific metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Result from worker execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerResult {
    /// Success status
    pub success: bool,
    /// Output content
    pub content: String,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Worker health status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerHealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Worker health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerHealthMetrics {
    pub response_time_ms: u64,
    pub cpu_usage_percent: f32,
    pub memory_usage_percent: f32,
    pub active_tasks: u32,
    pub queue_depth: u32,
    pub last_seen: DateTime<Utc>,
    pub consecutive_failures: u32,
}

/// Worker pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerPoolStats {
    pub total_workers: u32,
    pub available_workers: u32,
    pub busy_workers: u32,
    pub unhealthy_workers: u32,
    pub average_response_time_ms: u64,
    pub total_tasks_processed: u64,
    pub tasks_per_second: f32,
}

/// Worker assignment with reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerAssignment {
    pub worker_id: Uuid,
    pub priority: TaskPriority,
    pub estimated_completion_time: DateTime<Utc>,
    pub confidence_score: f32,
}

/// Worker registration request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerRegistration {
    pub name: String,
    pub worker_type: WorkerType,
    pub model_name: String,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub max_concurrent_tasks: u32,
    pub health_check_endpoint: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Worker update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerUpdate {
    pub capabilities: Option<Vec<String>>,
    pub max_concurrent_tasks: Option<u32>,
    pub status: Option<String>,
}

/// Worker pool events for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkerPoolEvent {
    WorkerRegistered {
        worker_id: Uuid,
        capabilities: Vec<String>,
    },
    WorkerHealthChecked {
        worker_id: Uuid,
        is_healthy: bool,
        response_time_ms: u64,
        checked_at: DateTime<Utc>,
    },
    WorkerAssigned {
        task_id: Uuid,
        worker_id: Uuid,
        estimated_completion_time: DateTime<Utc>,
    },
    WorkerTaskCompleted {
        task_id: Uuid,
        worker_id: Uuid,
        success: bool,
        execution_time_ms: u64,
    },
    WorkerTaskFailed {
        task_id: Uuid,
        worker_id: Uuid,
        error: String,
        retry_count: u32,
    },
}

/// Worker event types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerEventType {
    Registration,
    HealthCheck,
    Assignment,
    Completion,
    Failure,
    Decommission,
}

/// Task priority levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}
