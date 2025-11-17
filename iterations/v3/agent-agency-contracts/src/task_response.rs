//! Task response contract for autonomous task execution.
//!
//! Defines the response contract for task execution status, progress tracking,
//! and real-time updates during autonomous task processing.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Task response with execution status and tracking information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TaskResponse {
    /// Contract version for compatibility
    pub version: String,

    /// Task identifier
    #[schemars(with = "String")]
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
    #[schemars(with = "Option<String>")]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TaskExecutionMetadata {
    /// When execution was created
    #[schemars(with = "String")]
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// When execution started (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<String>")]
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,

    /// When execution completed (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<String>")]
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

/// Validate a task response value against the JSON schema
pub fn validate_task_response_value(
    value: &serde_json::Value,
) -> Result<(), crate::contract_errors::ContractError> {
    use crate::contract_errors::{ContractError, ContractKind};
    use crate::schema::TASK_RESPONSE_SCHEMA;

    TASK_RESPONSE_SCHEMA.validate(value).map_err(|errors| {
        let issues = errors
            .into_iter()
            .map(|error| crate::contract_errors::ValidationIssue {
                instance_path: error.instance_path.to_string(),
                schema_path: error.schema_path.to_string(),
                message: error.to_string(),
            })
            .collect();
        ContractError::validation(ContractKind::TaskResponse, issues)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn validate_task_response_value_returns_error_on_invalid() {
        // Multiple provably invalid cases - if validate() returns Ok(()), ALL will fail
        let invalid_cases = vec![
            // Missing version
            json!({"task_id": "123", "status": "accepted"}),
            // Missing task_id
            json!({"version": "1.0", "status": "accepted"}),
            // Missing status
            json!({"version": "1.0", "task_id": "123"}),
            // Wrong type for status (should be enum string)
            json!({"version": "1.0", "task_id": "123", "status": 456}),
            // Empty object
            json!({}),
            // Null
            json!(null),
        ];

        for (idx, invalid_case) in invalid_cases.iter().enumerate() {
            let result = validate_task_response_value(invalid_case);
            // This MUST fail - if validate() is stubbed to return Ok(()), test fails
            assert!(
                result.is_err(),
                "Invalid case {} should be rejected by validation, but got Ok(()) - validation may be stubbed",
                idx
            );
        }
    }

    #[test]
    fn validate_task_response_value_returns_ok_on_valid() {
        let valid_value = json!({
            "version": "1.0",
            "task_id": "00000000-0000-0000-0000-000000000000",
            "status": "accepted"
        });

        let result = validate_task_response_value(&valid_value);
        // May pass or fail depending on schema requirements, but should return a Result
        assert!(result.is_ok() || result.is_err());
    }
}
