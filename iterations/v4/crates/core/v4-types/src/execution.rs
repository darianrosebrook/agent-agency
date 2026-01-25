//! Execution types
//!
//! Types for task execution results and artifacts.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Task execution result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskResult {
    /// Task ID
    pub task_id: String,
    /// Execution status
    pub status: ExecutionStatus,
    /// Artifacts produced
    pub artifacts: ExecutionArtifacts,
    /// Execution metrics
    pub metrics: ExecutionMetrics,
    /// Timestamp
    #[schemars(with = "String")]
    pub completed_at: chrono::DateTime<chrono::Utc>,
}

/// Execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ExecutionStatus {
    /// Execution succeeded
    Success,
    /// Execution succeeded with warnings
    SuccessWithWarnings,
    /// Execution failed
    Failed,
    /// Execution was cancelled
    Cancelled,
    /// Execution timed out
    TimedOut,
}

impl ExecutionStatus {
    /// Check if this status represents success
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success | Self::SuccessWithWarnings)
    }

    /// Check if this status represents failure
    pub fn is_failure(&self) -> bool {
        matches!(self, Self::Failed | Self::Cancelled | Self::TimedOut)
    }
}

/// Execution artifacts
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionArtifacts {
    /// Files that were modified
    pub modified_files: Vec<FileChange>,
    /// Test results
    pub test_results: Option<TestResults>,
    /// Git commit info (if committed)
    pub git_info: Option<GitInfo>,
    /// Audit events
    pub audit_events: Vec<AuditEvent>,
}

/// File change record
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileChange {
    /// File path
    pub path: String,
    /// Type of change
    pub change_type: ChangeType,
    /// Lines added
    pub lines_added: u32,
    /// Lines removed
    pub lines_removed: u32,
    /// Content hash after change
    pub content_hash: String,
}

/// Type of file change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ChangeType {
    /// New file created
    Created,
    /// Existing file modified
    Modified,
    /// File deleted
    Deleted,
    /// File renamed
    Renamed,
}

/// Test execution results
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TestResults {
    /// Total tests run
    pub total: u32,
    /// Tests passed
    pub passed: u32,
    /// Tests failed
    pub failed: u32,
    /// Tests skipped
    pub skipped: u32,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Coverage percentage (if available)
    pub coverage_percent: Option<f64>,
}

impl TestResults {
    /// Calculate pass rate (0.0 to 1.0)
    pub fn pass_rate(&self) -> f64 {
        if self.total == 0 {
            1.0 // No tests = 100% pass rate
        } else {
            self.passed as f64 / self.total as f64
        }
    }

    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }
}

/// Git commit information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GitInfo {
    /// Commit hash
    pub commit_hash: String,
    /// Branch name
    pub branch: String,
    /// Commit message
    pub message: String,
    /// Author
    pub author: String,
    /// Timestamp
    #[schemars(with = "String")]
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Audit event for tracking
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AuditEvent {
    /// Event ID
    pub id: String,
    /// Event type
    pub event_type: String,
    /// Event description
    pub description: String,
    /// Actor (who/what caused this)
    pub actor: String,
    /// Timestamp
    #[schemars(with = "String")]
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Content hash for verification
    pub content_hash: String,
}

/// Execution metrics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionMetrics {
    /// Total execution time in milliseconds
    pub total_time_ms: u64,
    /// Planning time in milliseconds
    pub planning_time_ms: u64,
    /// Council review time in milliseconds
    pub review_time_ms: u64,
    /// Actual execution time in milliseconds
    pub execution_time_ms: u64,
    /// Number of iterations
    pub iterations: u32,
    /// Number of tool invocations
    pub tool_invocations: u32,
    /// Number of LLM calls
    pub llm_calls: u32,
    /// Token usage
    pub token_usage: TokenUsage,
}

/// Token usage for LLM calls
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TokenUsage {
    /// Input tokens
    pub input_tokens: u64,
    /// Output tokens
    pub output_tokens: u64,
    /// Total tokens
    pub total_tokens: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_status() {
        assert!(ExecutionStatus::Success.is_success());
        assert!(ExecutionStatus::SuccessWithWarnings.is_success());
        assert!(!ExecutionStatus::Failed.is_success());

        assert!(ExecutionStatus::Failed.is_failure());
        assert!(ExecutionStatus::TimedOut.is_failure());
        assert!(!ExecutionStatus::Success.is_failure());
    }

    #[test]
    fn test_test_results_pass_rate() {
        let results = TestResults {
            total: 100,
            passed: 95,
            failed: 5,
            skipped: 0,
            execution_time_ms: 1000,
            coverage_percent: Some(85.0),
        };
        assert!((results.pass_rate() - 0.95).abs() < 0.001);
        assert!(!results.all_passed());

        let perfect = TestResults {
            total: 50,
            passed: 50,
            failed: 0,
            skipped: 0,
            execution_time_ms: 500,
            coverage_percent: None,
        };
        assert!((perfect.pass_rate() - 1.0).abs() < 0.001);
        assert!(perfect.all_passed());
    }

    #[test]
    fn test_empty_test_results() {
        let empty = TestResults {
            total: 0,
            passed: 0,
            failed: 0,
            skipped: 0,
            execution_time_ms: 0,
            coverage_percent: None,
        };
        assert!((empty.pass_rate() - 1.0).abs() < 0.001);
        assert!(empty.all_passed());
    }
}
