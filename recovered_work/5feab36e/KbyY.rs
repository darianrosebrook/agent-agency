//! Planning Agent error types and results

use std::fmt;

/// Result type for planning operations
pub type PlanningResult<T> = Result<T, PlanningError>;

/// Errors that can occur during planning operations
#[derive(Debug, thiserror::Error)]
pub enum PlanningError {
    #[error("Task request validation failed: {0}")]
    InvalidTaskRequest(String),

    #[error("Working spec generation failed: {0}")]
    WorkingSpecGeneration(String),

    #[error("CAWS validation failed: {0}")]
    CawsValidation(String),

    #[error("Validation pipeline failed at stage {stage}: {error}")]
    ValidationPipeline { stage: String, error: String },

    #[error("Refinement failed: {0}")]
    Refinement(String),

    #[error("Database operation failed: {0}")]
    Database(#[from] anyhow::Error),

    #[error("Task constraints violated: {0}")]
    ConstraintViolation(String),

    #[error("Planning timeout exceeded: {0}")]
    Timeout(String),

    #[error("Serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Risk tier escalation required: {reason}")]
    RiskEscalation { reason: String },
}

impl PlanningError {
    /// Check if this error requires human intervention
    pub fn requires_human_intervention(&self) -> bool {
        matches!(self, PlanningError::RiskEscalation { .. })
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        !matches!(self, PlanningError::RiskEscalation { .. })
    }

    /// Get the error category for logging/metrics
    pub fn category(&self) -> &'static str {
        match self {
            PlanningError::InvalidTaskRequest(_) => "validation",
            PlanningError::WorkingSpecGeneration(_) => "generation",
            PlanningError::CawsValidation(_) => "caws",
            PlanningError::ValidationPipeline { .. } => "pipeline",
            PlanningError::Refinement(_) => "refinement",
            PlanningError::Database(_) => "database",
            PlanningError::ConstraintViolation(_) => "constraints",
            PlanningError::Timeout(_) => "timeout",
            PlanningError::RiskEscalation { .. } => "escalation",
        }
    }
}
