//! Inference error types

use thiserror::Error;

/// Inference errors
#[derive(Debug, Error)]
pub enum InferenceError {
    /// Model not loaded
    #[error("Model not loaded: {0}")]
    ModelNotLoaded(String),

    /// Model loading failed
    #[error("Failed to load model: {0}")]
    LoadFailed(String),

    /// Inference failed
    #[error("Inference failed: {0}")]
    InferenceFailed(String),

    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Timeout
    #[error("Inference timed out after {0}ms")]
    Timeout(u64),

    /// Provider not available
    #[error("Provider not available: {0}")]
    ProviderNotAvailable(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Resource exhausted
    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = InferenceError::ModelNotLoaded("mistral-7b".to_string());
        assert!(err.to_string().contains("mistral-7b"));
    }

    #[test]
    fn test_timeout_display() {
        let err = InferenceError::Timeout(5000);
        assert!(err.to_string().contains("5000"));
    }
}
