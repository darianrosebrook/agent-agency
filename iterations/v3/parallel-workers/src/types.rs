//! Shared types and enums for the parallel worker system

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for tasks
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub String);

impl TaskId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for subtasks
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SubTaskId(pub String);

impl SubTaskId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl std::fmt::Display for SubTaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for workers
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkerId(pub String);

impl WorkerId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl std::fmt::Display for WorkerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Complex task that may benefit from parallel decomposition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexTask {
    pub id: TaskId,
    pub description: String,
    pub context: TaskContext,
    pub complexity_score: f32,
    pub estimated_subtasks: Option<usize>,
}

/// Task execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub working_directory: PathBuf,
    pub environment_variables: HashMap<String, String>,
    pub timeout: Option<Duration>,
}

/// Subtask created from decomposition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTask {
    pub id: SubTaskId,
    pub parent_id: TaskId,
    pub title: String,
    pub description: String,
    pub scope: TaskScope,
    pub specialty: WorkerSpecialty,
    pub dependencies: Vec<SubTaskId>,
    pub estimated_effort: Duration,
    pub priority: Priority,
}

/// Task priority levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// Worker specialty domains
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerSpecialty {
    CompilationErrors { error_codes: Vec<String> },
    TypeSystem { domains: Vec<TypeDomain> },
    AsyncPatterns { patterns: Vec<String> },
    Refactoring { strategies: Vec<String> },
    Testing { frameworks: Vec<String> },
    Documentation { formats: Vec<String> },
    Custom { domain: String, capabilities: Vec<String> },
}

/// Type system domains for specialization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeDomain {
    StructFields,
    TraitBounds,
    GenericParameters,
    LifetimeParameters,
    AssociatedTypes,
}

/// Task scope boundaries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskScope {
    pub included_files: Vec<PathBuf>,
    pub excluded_files: Vec<PathBuf>,
    pub included_patterns: Vec<String>,
    pub excluded_patterns: Vec<String>,
    pub time_budget: Duration,
    pub quality_requirements: QualityRequirements,
}

/// Quality requirements for task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRequirements {
    pub min_test_coverage: Option<f32>,
    pub linting_required: bool,
    pub compilation_required: bool,
    pub documentation_required: bool,
}

/// Worker execution context
#[derive(Debug, Clone)]
pub struct WorkerContext {
    pub subtask: SubTask,
    pub workspace_root: PathBuf,
    pub isolated_env: HashMap<String, String>,
    pub communication_channel: tokio::sync::mpsc::UnboundedSender<WorkerMessage>,
}

/// Worker communication messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkerMessage {
    Started {
        worker_id: WorkerId,
        subtask_id: SubTaskId,
        timestamp: DateTime<Utc>,
    },
    Progress {
        worker_id: WorkerId,
        subtask_id: SubTaskId,
        completed: u32,
        total: u32,
        status: String,
        timestamp: DateTime<Utc>,
    },
    Blocked {
        worker_id: WorkerId,
        subtask_id: SubTaskId,
        reason: BlockageReason,
        context: String,
        timestamp: DateTime<Utc>,
    },
    Completed {
        worker_id: WorkerId,
        subtask_id: SubTaskId,
        result: WorkerResult,
        timestamp: DateTime<Utc>,
    },
    Failed {
        worker_id: WorkerId,
        subtask_id: SubTaskId,
        error: WorkerError,
        recoverable: bool,
        timestamp: DateTime<Utc>,
    },
}

/// Blockage reasons requiring coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockageReason {
    DependencyWait {
        required_worker: WorkerId,
        resource: String,
    },
    ExternalDependency {
        system: String,
        issue: String,
    },
    ComplexityExceeded {
        estimated_additional_time: Duration,
    },
    ScopeClarification {
        question: String,
    },
    ResourceExhausted {
        resource_type: String,
        available: u64,
        required: u64,
    },
}

/// Worker execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerResult {
    pub subtask_id: SubTaskId,
    pub success: bool,
    pub output: String,
    pub error_message: Option<String>,
    pub metrics: ExecutionMetrics,
    pub artifacts: Vec<Artifact>,
}

/// Execution metrics for performance tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub cpu_usage_percent: Option<f32>,
    pub memory_usage_mb: Option<f32>,
    pub files_modified: usize,
    pub lines_changed: usize,
}

/// Artifacts produced by worker execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub name: String,
    pub path: PathBuf,
    pub artifact_type: ArtifactType,
    pub size_bytes: u64,
}

/// Types of artifacts workers can produce
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactType {
    SourceCode,
    TestFile,
    Documentation,
    Configuration,
    Log,
    Binary,
}

/// Worker execution error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerError {
    pub error_type: WorkerErrorType,
    pub message: String,
    pub details: Option<String>,
    pub suggestions: Vec<String>,
}

/// Types of worker errors
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerErrorType {
    Compilation,
    Runtime,
    Timeout,
    ResourceExhaustion,
    DependencyFailure,
    ValidationFailure,
    Unknown,
}

/// Overall task result synthesized from worker results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: TaskId,
    pub success: bool,
    pub subtasks_completed: usize,
    pub total_subtasks: usize,
    pub execution_time: Duration,
    pub summary: String,
    pub worker_breakdown: Vec<WorkerBreakdown>,
    pub quality_scores: HashMap<String, f32>,
}

/// Breakdown of work by individual worker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerBreakdown {
    pub worker_id: WorkerId,
    pub subtask_id: SubTaskId,
    pub specialty: WorkerSpecialty,
    pub execution_time: Duration,
    pub success: bool,
    pub files_modified: usize,
    pub lines_changed: usize,
}

/// Progress tracking for overall task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    pub task_id: TaskId,
    pub percentage: f32,
    pub completed_subtasks: usize,
    pub total_subtasks: usize,
    pub active_workers: usize,
    pub blocked_workers: usize,
    pub failed_workers: usize,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub last_update: DateTime<Utc>,
}

/// Worker progress state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerProgress {
    pub worker_id: WorkerId,
    pub subtask_id: SubTaskId,
    pub completed: u32,
    pub total: u32,
    pub task_weight: f32,
    pub status: String,
    pub last_update: DateTime<Utc>,
}

/// Analysis result from decomposition engine
#[derive(Debug, Clone)]
pub struct TaskAnalysis {
    pub patterns: Vec<TaskPattern>,
    pub dependencies: Vec<Dependency>,
    pub subtask_scores: SubtaskScores,
    pub recommended_workers: usize,
    pub should_parallelize: bool,
}

/// Identified patterns in the task
#[derive(Debug, Clone)]
pub enum TaskPattern {
    CompilationErrors { error_groups: Vec<ErrorGroup> },
    RefactoringOperations { operations: Vec<RefactoringOp> },
    TestingGaps { missing_tests: Vec<String> },
    DocumentationNeeds { missing_docs: Vec<String> },
}

/// Group of similar errors
#[derive(Debug, Clone)]
pub struct ErrorGroup {
    pub error_code: String,
    pub count: usize,
    pub affected_files: Vec<PathBuf>,
}

/// Refactoring operations identified
#[derive(Debug, Clone)]
pub struct RefactoringOp {
    pub operation_type: String,
    pub affected_files: Vec<PathBuf>,
    pub complexity: f32,
}

/// Dependencies between subtasks
#[derive(Debug, Clone)]
pub struct Dependency {
    pub from_subtask: SubTaskId,
    pub to_subtask: SubTaskId,
    pub dependency_type: DependencyType,
    pub blocking: bool,
}

/// Types of dependencies
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencyType {
    FileAccess,
    CompilationOrder,
    DataDependency,
    ExternalResource,
}

/// Scoring information for subtasks
#[derive(Debug, Clone)]
pub struct SubtaskScores {
    pub parallelization_score: f32,
    pub complexity_scores: Vec<f32>,
    pub estimated_durations: Vec<Duration>,
}

/// Handle to an active worker
#[derive(Debug)]
pub struct WorkerHandle {
    pub id: WorkerId,
    pub subtask_id: SubTaskId,
    pub join_handle: tokio::task::JoinHandle<Result<WorkerResult, WorkerError>>,
    pub start_time: DateTime<Utc>,
}

/// Quality validation result
#[derive(Debug, Clone)]
pub enum ValidationResult {
    Pass {
        score: f32,
        details: String,
    },
    Fail {
        score: f32,
        details: String,
        suggestions: Vec<String>,
    },
    Warning {
        score: f32,
        details: String,
        suggestions: Vec<String>,
    },
}

impl ValidationResult {
    pub fn score(&self) -> f32 {
        match self {
            ValidationResult::Pass { score, .. } => *score,
            ValidationResult::Fail { score, .. } => *score,
            ValidationResult::Warning { score, .. } => *score,
        }
    }

    pub fn passes(&self, threshold: f32) -> bool {
        match self {
            ValidationResult::Pass { score, .. } => *score >= threshold,
            ValidationResult::Fail { .. } => false,
            ValidationResult::Warning { score, .. } => *score >= threshold,
        }
    }
}

// Type alias for convenience
pub type ValidationOutcome = ValidationResult;

/// Context for quality validation
#[derive(Debug, Clone)]
pub struct ValidationContext {
    pub package_name: String,
    pub workspace_root: PathBuf,
    pub results: Vec<WorkerResult>,
    pub execution_time: Duration,
}

/// Default implementations
impl Default for Priority {
    fn default() -> Self {
        Priority::Medium
    }
}

impl Default for QualityRequirements {
    fn default() -> Self {
        Self {
            min_test_coverage: Some(0.8),
            linting_required: true,
            compilation_required: true,
            documentation_required: false,
        }
    }
}

impl Default for TaskScope {
    fn default() -> Self {
        Self {
            included_files: vec![],
            excluded_files: vec![],
            included_patterns: vec![],
            excluded_patterns: vec![],
            time_budget: Duration::from_secs(300), // 5 minutes
            quality_requirements: QualityRequirements::default(),
        }
    }
}
