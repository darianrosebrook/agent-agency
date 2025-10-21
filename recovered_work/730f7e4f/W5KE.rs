//! Task response contract for autonomous task execution.
//!
//! Defines the response contract for task execution status, progress tracking,
//! and real-time updates during autonomous task processing.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Task response with execution status and tracking information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TaskResponse {
    /// Contract version for compatibility
    pub version: String,

    /// Task identifier
    pub task_id: Uuid,

    /// Current execution status
    pub status: TaskStatus,

    /// Generated working specification (when available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_spec: Option<WorkingSpecSummary>,

    /// URL for real-time progress tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_url: Option<String>,

    /// Estimated completion timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_completion: Option<chrono::DateTime<chrono::Utc>>,

    /// Current execution progress
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<TaskProgress>,

    /// Error information (when status is failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<TaskError>,

    /// Execution metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<TaskExecutionMetadata>,
}

/// Current task execution status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// Task accepted and queued for processing
    Accepted,

    /// Generating and validating working specification
    Planning,

    /// Executing the approved working specification
    Executing,

    /// Council reviewing execution artifacts
    Reviewing,

    /// Applying council-directed refinements
    Refining,

    /// Task completed successfully
    Completed,

    /// Task failed with error
    Failed,

    /// Task cancelled by user or system
    Cancelled,
}

/// Summary of working specification (for response size optimization)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WorkingSpecSummary {
    /// Working spec identifier
    pub id: String,

    /// Human-readable title
    pub title: String,

    /// Brief description
    pub description: String,

    /// High-level objectives
    pub goals: Vec<String>,

    /// Risk tier
    pub risk_tier: u32,

    /// Acceptance criteria count
    pub acceptance_criteria_count: usize,
}

/// Current execution progress information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TaskProgress {
    /// Current execution phase
    pub current_phase: String,

    /// Progress within current phase (0.0-1.0)
    pub phase_progress: f64,

    /// Overall progress across all phases (0.0-1.0)
    pub overall_progress: f64,

    /// Current iteration number
    pub current_iteration: u32,

    /// Maximum allowed iterations
    pub max_iterations: u32,

    /// Current quality score (0.0-1.0)
    pub quality_score: f64,

    /// Number of quality gates passed
    pub gates_passed: u32,

    /// Total number of quality gates
    pub gates_total: u32,
}

/// Task execution error information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TaskError {
    /// Error code for programmatic handling
    pub code: String,

    /// Human-readable error message
    pub message: String,

    /// Additional error details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,

    /// Whether this error is retryable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retryable: Option<bool>,
}

/// Execution metadata and statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TaskExecutionMetadata {
    /// When execution was created
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// When execution started (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,

    /// When execution completed (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Total execution duration in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_duration_seconds: Option<f64>,

    /// Worker assigned to this task
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker_assigned: Option<String>,

    /// Number of council reviews performed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub council_reviews: Option<u32>,
}
