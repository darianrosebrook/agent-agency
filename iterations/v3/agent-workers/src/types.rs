//! Core types for MCP-based worker system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for workers
pub type WorkerId = Uuid;

/// Unique identifier for tasks
pub type TaskId = Uuid;

/// Unique identifier for MCP tools
pub type ToolId = String;

/// Worker specialization types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WorkerSpecialty {
    /// React component generation and SCSS
    ReactComponent,
    /// File editing and manipulation
    FileEditing,
    /// Research and information gathering
    Research,
    /// Code generation and refactoring
    CodeGeneration,
    /// Testing and validation
    Testing,
    /// Documentation generation
    Documentation,
    /// General purpose worker
    General,
}

/// Task priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// Worker health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkerHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Offline,
}

/// Task execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// MCP tool metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    pub id: ToolId,
    pub name: String,
    pub description: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub parameters: HashMap<String, ToolParameter>,
}

/// MCP tool parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    pub description: String,
    pub parameter_type: String,
    pub required: bool,
    pub default_value: Option<serde_json::Value>,
}

/// Task definition with MCP tool requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDefinition {
    pub id: TaskId,
    pub name: String,
    pub description: String,
    pub priority: TaskPriority,
    pub required_tools: Vec<ToolId>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub timeout_seconds: Option<u64>,
}

/// Worker capabilities and status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerCapabilities {
    pub specialties: Vec<WorkerSpecialty>,
    pub available_tools: Vec<ToolId>,
    pub max_concurrent_tasks: usize,
    pub health_status: WorkerHealth,
    pub performance_metrics: WorkerPerformance,
}

/// Worker performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerPerformance {
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub average_execution_time_ms: f64,
    pub success_rate: f64,
}

/// Task execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub task_id: TaskId,
    pub worker_id: WorkerId,
    pub tool_id: ToolId,
    pub parameters: HashMap<String, serde_json::Value>,
    pub execution_timeout: std::time::Duration,
}

/// Task execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: TaskId,
    pub status: TaskStatus,
    pub output: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub execution_time_ms: u64,
    pub tool_used: ToolId,
    pub quality_score: Option<f64>,
}

/// Parallel execution plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelExecutionPlan {
    pub main_task: TaskDefinition,
    pub subtasks: Vec<SubTask>,
    pub dependencies: Vec<TaskDependency>,
    pub coordination_strategy: CoordinationStrategy,
}

/// Subtask definition for parallel execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTask {
    pub id: TaskId,
    pub parent_task_id: TaskId,
    pub name: String,
    pub description: String,
    pub tool_id: ToolId,
    pub parameters: HashMap<String, serde_json::Value>,
    pub priority: TaskPriority,
}

/// Task dependency for parallel execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDependency {
    pub dependent_task: TaskId,
    pub dependency_task: TaskId,
    pub dependency_type: DependencyType,
}

/// Types of task dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    /// Must complete before dependent task
    Completion,
    /// Must succeed before dependent task
    Success,
    /// Can run in parallel but results needed
    DataFlow,
}

/// Coordination strategies for parallel execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinationStrategy {
    /// All subtasks run in parallel
    FullyParallel,
    /// Subtasks run in dependency order
    SequentialDependencies,
    /// Dynamic scheduling based on results
    Adaptive,
}

/// Quality validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityValidation {
    pub passed: bool,
    pub score: f64,
    pub violations: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Worker pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerPoolStats {
    pub total_workers: usize,
    pub active_workers: usize,
    pub idle_workers: usize,
    pub unhealthy_workers: usize,
    pub total_tasks_processed: u64,
    pub tasks_in_progress: usize,
    pub average_queue_time_ms: f64,
    pub average_execution_time_ms: f64,
}
