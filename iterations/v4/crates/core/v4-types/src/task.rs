//! Task types
//!
//! Core types for task requests, responses, and specifications.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Task request from a client
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskRequest {
    /// Unique task identifier
    pub id: String,
    /// Human-readable title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Task constraints
    pub constraints: TaskConstraints,
    /// Priority level
    pub priority: TaskPriority,
    /// Environment context
    pub environment: Environment,
    /// Metadata
    #[schemars(with = "Option<serde_json::Value>")]
    pub metadata: Option<serde_json::Value>,
}

/// Task constraints
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskConstraints {
    /// Maximum execution time in seconds
    pub max_duration_seconds: Option<u64>,
    /// Maximum number of iterations
    pub max_iterations: Option<u32>,
    /// Maximum files to modify
    pub max_files: Option<u32>,
    /// Maximum lines of code to change
    pub max_loc: Option<u32>,
    /// Allowed file paths (glob patterns)
    pub allowed_paths: Vec<String>,
    /// Blocked file paths (glob patterns)
    pub blocked_paths: Vec<String>,
    /// Whether breaking changes are allowed
    pub allow_breaking_changes: bool,
}

impl Default for TaskConstraints {
    fn default() -> Self {
        Self {
            max_duration_seconds: Some(3600), // 1 hour
            max_iterations: Some(100),
            max_files: Some(50),
            max_loc: Some(5000),
            allowed_paths: vec!["**/*".to_string()],
            blocked_paths: vec![
                "**/node_modules/**".to_string(),
                "**/.git/**".to_string(),
                "**/target/**".to_string(),
            ],
            allow_breaking_changes: false,
        }
    }
}

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum TaskPriority {
    /// Low priority - can be deferred
    Low,
    /// Normal priority
    Normal,
    /// High priority - should be handled soon
    High,
    /// Critical - immediate attention required
    Critical,
}

impl Default for TaskPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Execution environment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum Environment {
    /// Development environment
    Development,
    /// Staging/testing environment
    Staging,
    /// Production environment
    Production,
}

impl Default for Environment {
    fn default() -> Self {
        Self::Development
    }
}

/// Task specification (working spec)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskSpec {
    /// Task ID
    pub id: String,
    /// Version of the spec
    pub version: String,
    /// Title
    pub title: String,
    /// Description
    pub description: String,
    /// Goals to achieve
    pub goals: Vec<String>,
    /// Risk tier (1-5)
    pub risk_tier: u8,
    /// Constraints
    pub constraints: TaskConstraints,
    /// Acceptance criteria
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    /// Created timestamp
    #[schemars(with = "String")]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Acceptance criterion in Given/When/Then format
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AcceptanceCriterion {
    /// Criterion ID
    pub id: String,
    /// Given precondition
    pub given: String,
    /// When action
    pub when: String,
    /// Then expected result
    pub then: String,
}

/// Task response
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskResponse {
    /// Task ID
    pub task_id: String,
    /// Current status
    pub status: TaskStatus,
    /// Progress (0.0 to 1.0)
    pub progress: f64,
    /// Result (if completed)
    pub result: Option<TaskResultSummary>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Timestamp
    #[schemars(with = "String")]
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Task execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum TaskStatus {
    /// Task is queued
    Pending,
    /// Task is being planned
    Planning,
    /// Task is awaiting council review
    AwaitingReview,
    /// Task is executing
    Executing,
    /// Task completed successfully
    Completed,
    /// Task failed
    Failed,
    /// Task was cancelled
    Cancelled,
}

/// Summary of task result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskResultSummary {
    /// Files modified
    pub files_modified: Vec<String>,
    /// Lines added
    pub lines_added: u32,
    /// Lines removed
    pub lines_removed: u32,
    /// Tests passed
    pub tests_passed: Option<u32>,
    /// Tests failed
    pub tests_failed: Option<u32>,
    /// Quality score
    pub quality_score: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_constraints() {
        let constraints = TaskConstraints::default();
        assert_eq!(constraints.max_duration_seconds, Some(3600));
        assert!(!constraints.allow_breaking_changes);
        assert!(!constraints.blocked_paths.is_empty());
    }

    #[test]
    fn test_task_status_transitions() {
        // Valid transitions
        let status = TaskStatus::Pending;
        assert!(matches!(status, TaskStatus::Pending));

        // All statuses are distinct
        assert_ne!(TaskStatus::Pending, TaskStatus::Executing);
        assert_ne!(TaskStatus::Completed, TaskStatus::Failed);
    }

    #[test]
    fn test_priority_default() {
        let priority = TaskPriority::default();
        assert_eq!(priority, TaskPriority::Normal);
    }
}
