//! Port traits (interfaces)
//!
//! Trait definitions for dependency injection and service boundaries.
//! These enable clean separation between layers without circular dependencies.

use async_trait::async_trait;

use crate::council::{CouncilVerdict, JudgeScores, SessionId};
use crate::events::AgentEvent;
use crate::execution::TaskResult;
use crate::task::{TaskRequest, TaskSpec};
use crate::verdict::QualityReport;

/// Database operations port
#[async_trait]
pub trait DatabasePort: Send + Sync {
    /// Store an event
    async fn store_event(&self, event: &AgentEvent) -> Result<(), DatabaseError>;

    /// Get events for a task
    async fn get_events(&self, task_id: &str) -> Result<Vec<AgentEvent>, DatabaseError>;

    /// Store a task spec
    async fn store_task_spec(&self, spec: &TaskSpec) -> Result<(), DatabaseError>;

    /// Get a task spec
    async fn get_task_spec(&self, task_id: &str) -> Result<Option<TaskSpec>, DatabaseError>;

    /// Store a task result
    async fn store_task_result(&self, result: &TaskResult) -> Result<(), DatabaseError>;

    /// Get a task result
    async fn get_task_result(&self, task_id: &str) -> Result<Option<TaskResult>, DatabaseError>;
}

/// File operations port
#[async_trait]
pub trait FileOperationsPort: Send + Sync {
    /// Read a file
    async fn read_file(&self, path: &str) -> Result<String, FileError>;

    /// Write a file
    async fn write_file(&self, path: &str, content: &str) -> Result<(), FileError>;

    /// Check if a file exists
    async fn file_exists(&self, path: &str) -> Result<bool, FileError>;

    /// List files matching a pattern
    async fn list_files(&self, pattern: &str) -> Result<Vec<String>, FileError>;

    /// Delete a file
    async fn delete_file(&self, path: &str) -> Result<(), FileError>;
}

/// Council coordinator port
#[async_trait]
pub trait CouncilPort: Send + Sync {
    /// Request a council review
    async fn request_review(
        &self,
        task_id: &str,
        spec: &TaskSpec,
    ) -> Result<SessionId, CouncilError>;

    /// Get review status
    async fn get_status(&self, session_id: SessionId) -> Result<ReviewStatus, CouncilError>;

    /// Get verdict when complete
    async fn get_verdict(&self, session_id: SessionId) -> Result<CouncilVerdict, CouncilError>;

    /// Get detailed scores
    async fn get_scores(&self, session_id: SessionId) -> Result<JudgeScores, CouncilError>;
}

/// Memory system port
#[async_trait]
pub trait MemoryPort: Send + Sync {
    /// Store a memory item
    async fn store(&self, key: &str, value: &str) -> Result<(), MemoryError>;

    /// Recall a memory item
    async fn recall(&self, key: &str) -> Result<Option<String>, MemoryError>;

    /// Multi-hop recall
    async fn recall_with_hops(
        &self,
        query: &str,
        max_hops: u32,
    ) -> Result<Vec<RecallResult>, MemoryError>;

    /// Apply decay to all memories
    async fn apply_decay(&self) -> Result<u32, MemoryError>;
}

/// Task executor port
#[async_trait]
pub trait TaskExecutorPort: Send + Sync {
    /// Execute a task
    async fn execute(&self, request: TaskRequest) -> Result<TaskResult, ExecutionError>;

    /// Get execution progress
    async fn get_progress(&self, task_id: &str) -> Result<f64, ExecutionError>;

    /// Cancel a task
    async fn cancel(&self, task_id: &str) -> Result<(), ExecutionError>;
}

/// Quality gate port
#[async_trait]
pub trait QualityGatePort: Send + Sync {
    /// Run all quality gates
    async fn run_gates(&self, result: &TaskResult) -> Result<QualityReport, GateError>;

    /// Run a specific gate
    async fn run_gate(
        &self,
        gate_type: crate::verdict::GateType,
        result: &TaskResult,
    ) -> Result<crate::verdict::GateResult, GateError>;
}

// === Error Types ===

/// Database error
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Query failed: {0}")]
    QueryFailed(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// File operation error
#[derive(Debug, thiserror::Error)]
pub enum FileError {
    #[error("File not found: {0}")]
    NotFound(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Path blocked: {0}")]
    PathBlocked(String),
}

/// Council error
#[derive(Debug, thiserror::Error)]
pub enum CouncilError {
    #[error("Session not found: {0}")]
    SessionNotFound(String),
    #[error("Review failed: {0}")]
    ReviewFailed(String),
    #[error("Timeout")]
    Timeout,
}

/// Memory error
#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    #[error("Storage failed: {0}")]
    StorageFailed(String),
    #[error("Recall failed: {0}")]
    RecallFailed(String),
    #[error("Graph error: {0}")]
    GraphError(String),
}

/// Execution error
#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    #[error("Task not found: {0}")]
    TaskNotFound(String),
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Cancelled")]
    Cancelled,
    #[error("Timeout")]
    Timeout,
}

/// Gate error
#[derive(Debug, thiserror::Error)]
pub enum GateError {
    #[error("Gate check failed: {0}")]
    CheckFailed(String),
    #[error("Invalid result: {0}")]
    InvalidResult(String),
}

// === Supporting Types ===

/// Review status
#[derive(Debug, Clone)]
pub enum ReviewStatus {
    /// Review is in progress
    InProgress { progress: f64 },
    /// Review is complete
    Complete { verdict: CouncilVerdict },
    /// Review failed
    Failed { error: String },
}

/// Memory recall result
#[derive(Debug, Clone)]
pub struct RecallResult {
    /// The key
    pub key: String,
    /// The value
    pub value: String,
    /// Relevance score
    pub relevance: f64,
    /// Hop count (0 = direct match)
    pub hop_count: u32,
    /// Provenance chain
    pub provenance: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_error_display() {
        let err = DatabaseError::NotFound("task_123".to_string());
        assert!(err.to_string().contains("task_123"));
    }

    #[test]
    fn test_file_error_display() {
        let err = FileError::PathBlocked("/etc/passwd".to_string());
        assert!(err.to_string().contains("/etc/passwd"));
    }
}
