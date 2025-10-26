//! Core types for the self-prompting loop controller

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// Execution modes with different safety guardrails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionMode {
    /// Manual approval required for each changeset before application
    Strict,
    /// Automatic execution with promotion only if quality gates pass
    Auto,
    /// Generate all artifacts but never apply changes to filesystem
    DryRun,
}

/// Execution state for task intervention
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExecutionState {
    /// Task is running normally
    Running,
    /// Task is paused, waiting for resume
    Paused,
    /// Task has been aborted
    Aborted,
}

/// Failure types for patch application (addresses 75% of agent failures)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PatchFailureType {
    /// Changeset validation failed
    ValidationError,
    /// File operation failed (permissions, locks, etc.)
    FileOperationError,
    /// Quality gate failure
    QualityGateFailure,
    /// Dependency resolution failed
    DependencyError,
    /// Context window exceeded
    ContextOverflow,
    /// Budget exceeded (files or LOC)
    BudgetExceeded,
}

/// Events emitted during self-prompting loop execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelfPromptingEvent {
    /// Task execution started
    TaskStarted {
        task_id: String,
        description: String,
        timestamp: DateTime<Utc>,
    },
    /// Iteration completed
    IterationCompleted {
        iteration: usize,
        artifacts_generated: usize,
        timestamp: DateTime<Utc>,
    },
    /// Changeset generated
    ChangesetGenerated {
        changeset_id: String,
        files_affected: usize,
        timestamp: DateTime<Utc>,
    },
    /// Quality gate passed
    QualityGatePassed {
        gate_name: String,
        score: f32,
        timestamp: DateTime<Utc>,
    },
    /// Quality gate failed
    QualityGateFailed {
        gate_name: String,
        reason: String,
        timestamp: DateTime<Utc>,
    },
    /// Task paused for user intervention
    TaskPaused {
        reason: String,
        timestamp: DateTime<Utc>,
    },
    /// Task resumed after user intervention
    TaskResumed {
        timestamp: DateTime<Utc>,
    },
    /// Task completed successfully
    TaskCompleted {
        total_iterations: usize,
        total_time_ms: u64,
        final_verdict: String,
        timestamp: DateTime<Utc>,
    },
    /// Task failed
    TaskFailed {
        error: String,
        total_iterations: usize,
        timestamp: DateTime<Utc>,
    },
}

/// Result of a self-prompting loop execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfPromptingResult {
    /// Final task result
    pub task_result: crate::types::TaskResult,
    /// Number of iterations performed
    pub iterations_performed: usize,
    /// Models used during execution
    pub models_used: Vec<String>,
    /// Total execution time in milliseconds
    pub total_time_ms: u64,
    /// Reason why execution stopped
    pub final_stop_reason: crate::types::StopReason,
}

/// Errors that can occur during self-prompting loop execution
#[derive(Debug, Clone, thiserror::Error)]
pub enum SelfPromptingError {
    #[error("Task execution failed: {message}")]
    TaskExecutionFailed { message: String },

    #[error("Model error: {message}")]
    ModelError { message: String },

    #[error("Evaluation error: {message}")]
    EvaluationError { message: String },

    #[error("File operation error: {message}")]
    FileOperationError { message: String },

    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    #[error("Context overflow: {message}")]
    ContextOverflow { message: String },

    #[error("Quality gate failure: {message}")]
    QualityGateFailure { message: String },

    #[error("User intervention required: {message}")]
    UserInterventionRequired { message: String },

    #[error("Timeout exceeded: {timeout_ms}ms")]
    TimeoutExceeded { timeout_ms: u64 },

    #[error("Budget exceeded: {message}")]
    BudgetExceeded { message: String },
}
