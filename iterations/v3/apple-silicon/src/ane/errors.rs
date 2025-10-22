//! ANE-specific error types and result aliases
//!
//! This module provides comprehensive error handling for Apple Neural Engine operations,
//! including model lifecycle, inference, capability detection, and resource management.

use thiserror::Error;

/// Comprehensive error types for ANE operations
#[derive(Debug, Error)]
pub enum ANEError {
    /// ANE is not available on this target platform
    #[error("ANE unavailable on this target")]
    Unavailable,
    
    /// Core ML framework error
    #[error("CoreML error: {0}")]
    CoreMLError(String),
    
    /// Model not found or invalid
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    
    /// Model already loaded (duplicate load attempt)
    #[error("Model already loaded: {0}")]
    ModelAlreadyLoaded(String),
    
    /// Invalid input/output tensor shape
    #[error("Invalid IO shape: {0}")]
    InvalidShape(String),
    
    /// Unsupported precision or data type
    #[error("Unsupported precision: {0}")]
    UnsupportedPrecision(String),
    
    /// Resource limit exceeded (memory, concurrency, etc.)
    #[error("Resource limit exceeded: {0}")]
    ResourceLimit(String),
    
    /// Operation timed out
    #[error("Timeout after {0} ms")]
    Timeout(u64),
    
    /// Internal system error
    #[error("Internal: {0}")]
    Internal(&'static str),
    
    /// Invalid model format or corrupted file
    #[error("Invalid model format: {0}")]
    InvalidModelFormat(String),
    
    /// Model compilation failed
    #[error("Model compilation failed: {0}")]
    CompilationFailed(String),
    
    /// Inference execution failed
    #[error("Inference execution failed: {0}")]
    InferenceFailed(String),
    
    /// Memory allocation failed
    #[error("Memory allocation failed: {0}")]
    MemoryAllocationFailed(String),
    
    /// Device capability mismatch
    #[error("Device capability mismatch: {0}")]
    CapabilityMismatch(String),
    
    /// Configuration validation failed
    #[error("Configuration validation failed: {0}")]
    ConfigurationError(String),
}

/// Result type alias for ANE operations
pub type Result<T> = std::result::Result<T, ANEError>;

/// Convert from std::io::Error to ANEError
impl From<std::io::Error> for ANEError {
    fn from(_err: std::io::Error) -> Self {
        ANEError::Internal("IO error")
    }
}

/// Convert from anyhow::Error to ANEError
impl From<anyhow::Error> for ANEError {
    fn from(_err: anyhow::Error) -> Self {
        ANEError::Internal("Anyhow error")
    }
}

/// Convert from serde_json::Error to ANEError
impl From<serde_json::Error> for ANEError {
    fn from(_err: serde_json::Error) -> Self {
        ANEError::Internal("JSON error")
    }
}

/// Helper trait for converting errors to ANEError
pub trait IntoANEError {
    fn into_ane_error(self) -> ANEError;
}

impl IntoANEError for String {
    fn into_ane_error(self) -> ANEError {
        ANEError::Internal("String error")
    }
}

impl IntoANEError for &str {
    fn into_ane_error(self) -> ANEError {
        ANEError::Internal("String error")
    }
}

/// Error severity levels for logging and monitoring
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    /// Low severity - informational
    Low,
    /// Medium severity - warning
    Medium,
    /// High severity - error
    High,
    /// Critical severity - system failure
    Critical,
}

impl ANEError {
    /// Get the severity level of this error
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            ANEError::Unavailable => ErrorSeverity::High,
            ANEError::CoreMLError(_) => ErrorSeverity::High,
            ANEError::ModelNotFound(_) => ErrorSeverity::Medium,
            ANEError::ModelAlreadyLoaded(_) => ErrorSeverity::Low,
            ANEError::InvalidShape(_) => ErrorSeverity::Medium,
            ANEError::UnsupportedPrecision(_) => ErrorSeverity::Medium,
            ANEError::ResourceLimit(_) => ErrorSeverity::High,
            ANEError::Timeout(_) => ErrorSeverity::Medium,
            ANEError::Internal(_) => ErrorSeverity::High,
            ANEError::InvalidModelFormat(_) => ErrorSeverity::High,
            ANEError::CompilationFailed(_) => ErrorSeverity::High,
            ANEError::InferenceFailed(_) => ErrorSeverity::High,
            ANEError::MemoryAllocationFailed(_) => ErrorSeverity::Critical,
            ANEError::CapabilityMismatch(_) => ErrorSeverity::Medium,
            ANEError::ConfigurationError(_) => ErrorSeverity::Medium,
        }
    }
    
    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            ANEError::Timeout(_) => true,
            ANEError::ResourceLimit(_) => true,
            ANEError::ModelAlreadyLoaded(_) => true,
            _ => false,
        }
    }
    
    /// Get a user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            ANEError::Unavailable => "Apple Neural Engine is not available on this system".to_string(),
            ANEError::CoreMLError(msg) => format!("Core ML error: {}", msg),
            ANEError::ModelNotFound(path) => format!("Model not found at: {}", path),
            ANEError::ModelAlreadyLoaded(path) => format!("Model already loaded: {}", path),
            ANEError::InvalidShape(msg) => format!("Invalid tensor shape: {}", msg),
            ANEError::UnsupportedPrecision(precision) => format!("Unsupported precision: {}", precision),
            ANEError::ResourceLimit(resource) => format!("Resource limit exceeded: {}", resource),
            ANEError::Timeout(ms) => format!("Operation timed out after {} ms", ms),
            ANEError::Internal(msg) => format!("Internal error: {}", msg),
            ANEError::InvalidModelFormat(msg) => format!("Invalid model format: {}", msg),
            ANEError::CompilationFailed(msg) => format!("Model compilation failed: {}", msg),
            ANEError::InferenceFailed(msg) => format!("Inference execution failed: {}", msg),
            ANEError::MemoryAllocationFailed(msg) => format!("Memory allocation failed: {}", msg),
            ANEError::CapabilityMismatch(msg) => format!("Device capability mismatch: {}", msg),
            ANEError::ConfigurationError(msg) => format!("Configuration error: {}", msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_severity() {
        assert_eq!(ANEError::Unavailable.severity(), ErrorSeverity::High);
        assert_eq!(ANEError::ModelAlreadyLoaded("test".to_string()).severity(), ErrorSeverity::Low);
        assert_eq!(ANEError::MemoryAllocationFailed("test".to_string()).severity(), ErrorSeverity::Critical);
    }

    #[test]
    fn test_error_recoverability() {
        assert!(ANEError::Timeout(1000).is_recoverable());
        assert!(ANEError::ResourceLimit("memory".to_string()).is_recoverable());
        assert!(!ANEError::Unavailable.is_recoverable());
        assert!(!ANEError::MemoryAllocationFailed("test".to_string()).is_recoverable());
    }

    #[test]
    fn test_user_messages() {
        let msg = ANEError::Unavailable.user_message();
        assert!(msg.contains("Apple Neural Engine"));
        
        let msg = ANEError::Timeout(5000).user_message();
        assert!(msg.contains("5000"));
    }

    #[test]
    fn test_error_conversions() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let ane_err: ANEError = io_err.into();
        assert!(matches!(ane_err, ANEError::Internal(_)));
    }
}
