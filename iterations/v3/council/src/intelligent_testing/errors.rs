//! Error handling for intelligent testing

use thiserror::Error;

/// Errors that can occur during intelligent testing operations
#[derive(Error, Debug)]
pub enum IntelligentTestingError {
    #[error("Test generation failed: {0}")]
    TestGenerationFailed(String),

    #[error("Edge case analysis failed: {0}")]
    EdgeCaseAnalysisFailed(String),

    #[error("Test optimization failed: {0}")]
    TestOptimizationFailed(String),

    #[error("Coverage analysis failed: {0}")]
    CoverageAnalysisFailed(String),

    #[error("Invalid test specification: {0}")]
    InvalidTestSpecification(String),

    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),

    #[error("Timeout during operation: {0}")]
    TimeoutError(String),
}

/// Result type alias for intelligent testing operations
pub type Result<T> = std::result::Result<T, IntelligentTestingError>;