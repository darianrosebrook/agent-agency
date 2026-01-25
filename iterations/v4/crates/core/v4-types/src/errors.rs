//! Error types for V4
//!
//! Centralized error definitions to avoid duplication across crates.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Contract validation error
#[derive(Debug, Error)]
pub enum ContractError {
    /// JSON Schema validation failed
    #[error("Schema validation failed: {0}")]
    SchemaValidation(String),

    /// Required field missing
    #[error("Required field missing: {0}")]
    MissingField(String),

    /// Invalid field value
    #[error("Invalid value for field {field}: {message}")]
    InvalidValue { field: String, message: String },

    /// Type mismatch
    #[error("Type mismatch for field {field}: expected {expected}, got {actual}")]
    TypeMismatch {
        field: String,
        expected: String,
        actual: String,
    },
}

/// Validation error with location context
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidationError {
    /// Field or path that failed validation
    pub path: String,
    /// Error message
    pub message: String,
    /// Severity of the error
    pub severity: ValidationSeverity,
}

/// Severity levels for validation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ValidationSeverity {
    /// Informational - not blocking
    Info,
    /// Warning - should be addressed
    Warning,
    /// Error - must be fixed
    Error,
    /// Critical - blocks all progress
    Critical,
}

impl ValidationError {
    /// Create a new validation error
    pub fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
            severity: ValidationSeverity::Error,
        }
    }

    /// Create a critical validation error
    pub fn critical(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
            severity: ValidationSeverity::Critical,
        }
    }

    /// Create a warning
    pub fn warning(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
            severity: ValidationSeverity::Warning,
        }
    }

    /// Check if this error is blocking
    pub fn is_blocking(&self) -> bool {
        matches!(
            self.severity,
            ValidationSeverity::Error | ValidationSeverity::Critical
        )
    }
}

/// Result type alias for operations that can fail with validation errors
pub type ValidationResult<T> = Result<T, Vec<ValidationError>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error_creation() {
        let err = ValidationError::new("field.path", "must be non-empty");
        assert_eq!(err.path, "field.path");
        assert_eq!(err.severity, ValidationSeverity::Error);
        assert!(err.is_blocking());
    }

    #[test]
    fn test_warning_not_blocking() {
        let warning = ValidationError::warning("field", "consider changing");
        assert!(!warning.is_blocking());
    }

    #[test]
    fn test_critical_is_blocking() {
        let critical = ValidationError::critical("field", "catastrophic");
        assert!(critical.is_blocking());
    }
}
