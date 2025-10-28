//! Parallel execution types
//!
//! This module defines the core types used for parallel task execution,
//! including ComplexTask, TaskResult, and related structures.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// A complex task that can be decomposed into parallel subtasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexTask {
    pub id: TaskId,
    pub title: String,
    pub description: String,
    pub complexity_score: f64,
    pub priority: Priority,
    pub scope: TaskScope,
    pub quality_requirements: QualityRequirements,
    pub created_at: DateTime<Utc>,
    pub deadline: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Task identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub Uuid);

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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SubTaskId(pub Uuid);

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskScope {
    pub domains: Vec<String>,
    pub files_affected: Vec<String>,
    pub max_files: Option<usize>,
    pub max_loc: Option<usize>,
}

/// Quality requirements for task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: TaskId,
    pub success: bool,
    pub subtasks_completed: usize,
    pub total_subtasks: usize,
    pub execution_time: std::time::Duration,
    pub summary: String,
    pub worker_breakdown: Vec<WorkerBreakdown>,
    pub quality_scores: HashMap<String, f64>,
    pub errors: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Worker breakdown information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerBreakdown {
    pub worker_id: WorkerId,
    pub subtasks_assigned: usize,
    pub subtasks_completed: usize,
    pub execution_time: std::time::Duration,
    pub quality_score: f64,
    pub errors: Vec<String>,
}

/// Worker identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkerId(pub Uuid);

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

/// A subtask that can be executed by a worker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTask {
    pub id: SubTaskId,
    pub parent_task_id: TaskId,
    pub title: String,
    pub description: String,
    pub complexity: f64,
    pub dependencies: Vec<SubTaskId>,
    pub assigned_worker: Option<WorkerId>,
    pub status: SubTaskStatus,
    pub priority: Priority,
    pub estimated_duration: std::time::Duration,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Status of a subtask
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubTaskStatus {
    Pending,
    Assigned,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Result from a worker execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerResult {
    pub subtask_id: SubTaskId,
    pub worker_id: WorkerId,
    pub success: bool,
    pub output: String,
    pub execution_time: std::time::Duration,
    pub quality_score: f64,
    pub errors: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Analysis of a task for decomposition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAnalysis {
    pub task_id: TaskId,
    pub complexity_score: f64,
    pub should_parallelize: bool,
    pub patterns: Vec<TaskPattern>,
    pub recommended_workers: usize,
    pub subtask_scores: SubtaskScores,
}

/// Scores for subtask analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtaskScores {
    pub parallelization_score: f64,
    pub complexity_scores: Vec<f64>,
    pub estimated_durations: Vec<std::time::Duration>,
}

/// Pattern identified in a task
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorGroup {
    pub file_path: String,
    pub error_count: usize,
    pub severity: ErrorSeverity,
}

/// Error severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Refactoring operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringOperation {
    pub operation_type: String,
    pub file_path: String,
    pub complexity: f64,
    pub description: String,
}

/// Dependency between subtasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub from: SubTaskId,
    pub to: SubTaskId,
    pub dependency_type: DependencyType,
}

/// Type of dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    Sequential,
    Data,
    Resource,
}

/// Worker specialty for task assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkerSpecialty {
    Compilation,
    Refactoring,
    Testing,
    Documentation,
    General,
}

/// Result type for parallel operations
pub type ParallelResult<T> = Result<T, ParallelError>;

/// Errors that can occur during parallel execution
#[derive(Debug, thiserror::Error)]
pub enum ParallelError {
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
