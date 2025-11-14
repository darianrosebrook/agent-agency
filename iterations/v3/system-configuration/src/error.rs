//! Unified error types for pipeline operations
//!
//! This module provides consistent error handling across all pipeline implementations.

use thiserror::Error;

/// Result type for pipeline operations
pub type PipelineResult<T> = Result<T, PipelineError>;

/// Comprehensive error type for pipeline operations
#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("Pipeline configuration error: {0}")]
    Config(String),

    #[error("Pipeline execution error: {0}")]
    Execution(String),

    #[error("Stage processing error: {stage} - {message}")]
    StageError { stage: String, message: String },

    #[error("Pipeline timeout: {0}")]
    Timeout(String),

    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Task join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),

    #[error("Channel send error: {0}")]
    ChannelSendError(String),

    #[error("Channel receive error: {0}")]
    ChannelReceiveError(String),

    #[error("Metrics collection error: {0}")]
    Metrics(String),

    #[error("Health check error: {0}")]
    HealthCheck(String),

    #[error("Circuit breaker error: {0}")]
    CircuitBreaker(String),

    #[error("Rate limit error: {0}")]
    RateLimit(String),

    #[error("Unknown pipeline error: {0}")]
    Other(String),
}

impl PipelineError {
    /// Create a stage error
    pub fn stage_error(stage: impl Into<String>, message: impl Into<String>) -> Self {
        PipelineError::StageError {
            stage: stage.into(),
            message: message.into(),
        }
    }

    /// Create a timeout error
    pub fn timeout(operation: impl Into<String>) -> Self {
        PipelineError::Timeout(operation.into())
    }

    /// Create a resource exhausted error
    pub fn resource_exhausted(resource: impl Into<String>) -> Self {
        PipelineError::ResourceExhausted(resource.into())
    }

    /// Create a validation error
    pub fn validation_error(message: impl Into<String>) -> Self {
        PipelineError::Validation(message.into())
    }

    /// Check if this is a recoverable error
    pub fn is_recoverable(&self) -> bool {
        match self {
            PipelineError::Timeout(_) => true,
            PipelineError::ResourceExhausted(_) => true,
            PipelineError::CircuitBreaker(_) => true,
            PipelineError::RateLimit(_) => true,
            PipelineError::Io(_) => true, // Some I/O errors are recoverable
            _ => false,
        }
    }

    /// Check if this is a configuration error
    pub fn is_config_error(&self) -> bool {
        matches!(self, PipelineError::Config(_))
    }

    /// Check if this is an execution error
    pub fn is_execution_error(&self) -> bool {
        matches!(
            self,
            PipelineError::Execution(_) | PipelineError::StageError { .. }
        )
    }

    /// Get the error category for metrics/logging
    pub fn category(&self) -> &'static str {
        match self {
            PipelineError::Config(_) => "config",
            PipelineError::Execution(_) => "execution",
            PipelineError::StageError { .. } => "stage",
            PipelineError::Timeout(_) => "timeout",
            PipelineError::ResourceExhausted(_) => "resource",
            PipelineError::Validation(_) => "validation",
            PipelineError::Cache(_) => "cache",
            PipelineError::Serialization(_) => "serialization",
            PipelineError::Io(_) => "io",
            PipelineError::Json(_) => "json",
            PipelineError::JoinError(_) => "join",
            PipelineError::ChannelSendError(_) => "channel_send",
            PipelineError::ChannelReceiveError(_) => "channel_receive",
            PipelineError::Metrics(_) => "metrics",
            PipelineError::HealthCheck(_) => "health",
            PipelineError::CircuitBreaker(_) => "circuit_breaker",
            PipelineError::RateLimit(_) => "rate_limit",
            PipelineError::Other(_) => "other",
        }
    }
}

impl From<anyhow::Error> for PipelineError {
    fn from(err: anyhow::Error) -> Self {
        PipelineError::Other(err.to_string())
    }
}
