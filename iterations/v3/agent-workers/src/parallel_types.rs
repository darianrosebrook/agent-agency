//! Parallel execution types
//!
//! This module defines the core types used for parallel task execution,
//! including ComplexTask, TaskResult, and related structures.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Re-export TaskStatus so parallel modules can depend on a single definition.
pub type TaskStatus = crate::worker_types::TaskStatus;

/// A complex task that can be decomposed into parallel subtasks
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ComplexTask {
    pub id: TaskId,
    pub title: String,
    pub description: String,
    pub complexity_score: f64,
    pub priority: Priority,
    pub scope: TaskScope,
    pub quality_requirements: QualityRequirements,
    #[schemars(with = "String")]

    pub created_at: DateTime<Utc>,
    pub deadline: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Task identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct TaskId (pub Uuid);

impl TaskId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

/// Sub-task identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct SubTaskId (pub Uuid);

impl SubTaskId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for SubTaskId {
    fn default() -> Self {
        Self::new()
    }
}

/// Task scope definition
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskScope {
    pub domains: Vec<String>,
    pub files_affected: Vec<String>,
    pub files: Vec<String>,
    pub directories: Vec<String>,
    pub patterns: Vec<String>,
    pub max_files: Option<usize>,
    pub max_loc: Option<usize>,
}

/// Quality requirements for task execution
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualityRequirements {
    pub min_coverage: Option<f64>,
    pub max_complexity: Option<f64>,
    pub required_tests: bool,
    pub documentation_required: bool,
}

impl Default for QualityRequirements {
    fn default() -> Self {
        Self {
            min_coverage: Some(0.8),
            max_complexity: Some(10.0),
            required_tests: true,
            documentation_required: false,
        }
    }
}

/// Task priority levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
pub enum Priority {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

impl Default for Priority {
    fn default() -> Self {
        Self::Medium
    }
}

/// Result of task execution
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskResult {
    pub task_id: TaskId,
    pub success: bool,
    pub subtasks_completed: usize,
    pub total_subtasks: usize,
    pub execution_time: std::time::Duration,
    pub execution_time_ms: u64,
    pub summary: String,
    pub worker_breakdown: Vec<WorkerBreakdown>,
    pub quality_scores: HashMap<String, f64>,
    pub errors: Vec<String>,
    pub error_message: Option<String>,
    pub tool_used: Option<String>,
    pub status: TaskStatus,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Worker breakdown information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkerBreakdown {
    pub worker_id: WorkerId,
    pub subtasks_assigned: usize,
    pub subtasks_completed: usize,
    pub execution_time: std::time::Duration,
    pub quality_score: f64,
    pub errors: Vec<String>,
}

/// Worker identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct WorkerId (pub Uuid);

impl WorkerId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for WorkerId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for WorkerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A subtask that can be executed by a worker
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SubTask {
    pub id: SubTaskId,
    pub parent_task_id: TaskId,
    pub parent_id: TaskId,
    pub title: String,
    pub description: String,
    pub complexity: f64,
    pub dependencies: Vec<SubTaskId>,
    pub assigned_worker: Option<WorkerId>,
    pub status: SubTaskStatus,
    pub priority: Priority,
    pub estimated_duration: std::time::Duration,
    pub scope: TaskScope,
    pub specialty: WorkerSpecialty,
    pub estimated_effort: f64,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Status of a subtask
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum SubTaskStatus {
    Pending,
    Assigned,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Metrics for worker execution
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkerMetrics {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub files_modified: usize,
    pub lines_changed: usize,
}

impl Default for WorkerMetrics {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            start_time: now,
            end_time: now,
            files_modified: 0,
            lines_changed: 0,
        }
    }
}

/// Result from a worker execution
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkerResult {
    pub task_id: TaskId,
    pub subtask_id: SubTaskId,
    pub worker_id: WorkerId,
    pub success: bool,
    pub output: String,
    pub execution_time: std::time::Duration,
    pub quality_score: f64,
    pub errors: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub metrics: WorkerMetrics,
    pub artifacts: Vec<crate::worker_types::Artifact>,
}

/// Analysis of a task for decomposition
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskAnalysis {
    pub task_id: TaskId,
    pub complexity_score: f64,
    pub should_parallelize: bool,
    pub patterns: Vec<TaskPattern>,
    pub recommended_workers: usize,
    pub subtask_scores: SubtaskScores,
    pub dependencies: Vec<Dependency>,
}

/// Scores for subtask analysis
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SubtaskScores {
    pub parallelization_score: f64,
    pub complexity_scores: Vec<f64>,
    pub estimated_durations: Vec<std::time::Duration>,
}

/// Pattern identified in a task
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum TaskPattern {
    CompilationErrors {
        error_groups: Vec<ErrorGroup>,
    },
    RefactoringOperations {
        operations: Vec<RefactoringOperation>,
    },
    TestingGaps {
        missing_tests: Vec<String>,
    },
    DocumentationNeeds {
        files_needing_docs: Vec<String>,
    },
}

/// Error group for compilation errors
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ErrorGroup {
    pub file_path: String,
    pub error_count: usize,
    pub severity: ErrorSeverity,
    pub error_code: String,
    pub count: usize,
    pub affected_files: Vec<String>,
}

/// Error severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
pub enum ErrorSeverity {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Refactoring operation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RefactoringOperation {
    pub operation_type: String,
    pub file_path: String,
    pub complexity: f64,
    pub description: String,
}

/// Dependency between subtasks
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Dependency {
    pub from: SubTaskId,
    pub to: SubTaskId,
    pub dependency_type: DependencyType,
}

/// Type of dependency
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum DependencyType {
    Sequential,
    Data,
    Resource,
}

/// Worker specialty for task assignment
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum WorkerSpecialty {
    Compilation,
    CompilationErrors {
        error_codes: Vec<String>,
    },
    Refactoring {
        strategies: Vec<String>,
    },
    Testing {
        frameworks: Vec<String>,
    },
    Documentation {
        formats: Vec<String>,
    },
    General,
}

/// Dependency between tasks (used in parallel execution)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskDependency {
    pub dependent_task: SubTaskId,
    pub dependency_task: SubTaskId,
    pub dependency_type: DependencyType,
}

/// Parallel execution plan
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ParallelExecutionPlan {
    pub main_task: crate::worker_types::TaskDefinition,
    pub subtasks: Vec<SubTask>,
    pub dependencies: Vec<TaskDependency>,
    pub coordination_strategy: CoordinationStrategy,
}

/// Coordination strategy for parallel execution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum CoordinationStrategy {
    FullyParallel,
    SequentialDependencies,
    Adaptive,
}

/// Decomposition strategy for breaking down tasks
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum DecompositionStrategy {
    Sequential,
    Parallel,
    Hierarchical,
    Adaptive,
}

/// Result type for parallel operations
pub type ParallelResult<T> = Result<T, ParallelError>;

/// Errors that can occur during parallel execution

#[derive(Debug, Serialize, JsonSchema, thiserror::Error)]
enum ParallelError {
    #[error("Decomposition error: {message}")]
    Decomposition { message: String, source: Option<Box<dyn std::error::Error + Send + Sync>> },
    
    #[error("Worker error: {0}")]
    Worker(String),
    
    #[error("Coordination error: {message}")]
    Coordination { message: String },
    
    #[error("Validation error: {message}")]
    Validation { message: String },
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Timeout error: {0}")]
    Timeout(String),
    
    #[error("Resource error: {0}")]
    Resource(String),
}

impl From<crate::worker_errors::DecompositionError> for ParallelError {
    fn from(error: crate::worker_errors::DecompositionError) -> Self {
        ParallelError::Decomposition {
            message: error.to_string(),
            source: Some(Box::new(error)),
        }
    }
}
