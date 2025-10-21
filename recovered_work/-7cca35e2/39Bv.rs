//! Council error types and results

use std::fmt;

/// Result type for council operations
pub type CouncilResult<T> = Result<T, CouncilError>;

/// Errors that can occur during council operations
#[derive(Debug, thiserror::Error)]
pub enum CouncilError {
    #[error("Judge error: {judge_id} - {message}")]
    JudgeError { judge_id: String, message: String },

    #[error("Consensus not reached: {reason}")]
    ConsensusFailure { reason: String },

    #[error("Workflow transition failed: {from} -> {to} ({reason})")]
    WorkflowTransition { from: String, to: String, reason: String },

    #[error("Verdict aggregation failed: {reason}")]
    AggregationFailure { reason: String },

    #[error("Decision making failed: {algorithm} - {reason}")]
    DecisionFailure { algorithm: String, reason: String },

    #[error("Council configuration invalid: {field} - {reason}")]
    ConfigurationError { field: String, reason: String },

    #[error("Session timeout: {session_id} exceeded {timeout_seconds}s")]
    SessionTimeout { session_id: String, timeout_seconds: u64 },

    #[error("Database operation failed: {0}")]
    Database(#[from] anyhow::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Council quorum not met: {available}/{required} judges")]
    QuorumFailure { available: usize, required: usize },

    #[error("Judge dissent unresolved: {judge_count} dissenting judges")]
    UnresolvedDissent { judge_count: usize },
}

impl CouncilError {
    /// Check if this error requires human intervention
    pub fn requires_human_intervention(&self) -> bool {
        matches!(
            self,
            CouncilError::ConsensusFailure { .. } |
            CouncilError::UnresolvedDissent { .. } |
            CouncilError::QuorumFailure { .. }
        )
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        !matches!(
            self,
            CouncilError::ConfigurationError { .. } |
            CouncilError::ConsensusFailure { .. } |
            CouncilError::UnresolvedDissent { .. }
        )
    }

    /// Get the error category for logging/metrics
    pub fn category(&self) -> &'static str {
        match self {
            CouncilError::JudgeError { .. } => "judge",
            CouncilError::ConsensusFailure { .. } => "consensus",
            CouncilError::WorkflowTransition { .. } => "workflow",
            CouncilError::AggregationFailure { .. } => "aggregation",
            CouncilError::DecisionFailure { .. } => "decision",
            CouncilError::ConfigurationError { .. } => "configuration",
            CouncilError::SessionTimeout { .. } => "timeout",
            CouncilError::Database(_) => "database",
            CouncilError::Serialization(_) => "serialization",
            CouncilError::QuorumFailure { .. } => "quorum",
            CouncilError::UnresolvedDissent { .. } => "dissent",
        }
    }
}
