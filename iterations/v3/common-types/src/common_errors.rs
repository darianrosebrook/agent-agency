//! Common error types and traits for consistent error handling across the system

use serde::{Deserialize, Serialize};
use std::fmt;

/// Common error categories used across the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
    /// Configuration errors (missing/invalid config)
    Configuration,
    /// Validation errors (invalid input/data)
    Validation,
    /// Network/communication errors
    Network,
    /// Database/storage errors
    Database,
    /// Authentication/authorization errors
    Authentication,
    /// Resource limit/quota errors
    ResourceLimit,
    /// Internal system errors
    Internal,
    /// External service errors
    ExternalService,
    /// Timeout errors
    Timeout,
    /// Concurrency/race condition errors
    Concurrency,
}

/// Common error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorSeverity {
    /// Debug-level errors (usually logged only)
    Debug,
    /// Info-level errors (minor issues)
    Info,
    /// Warning-level errors (potential issues)
    Warning,
    /// Error-level errors (functional failures)
    Error,
    /// Critical errors (system stability at risk)
    Critical,
    /// Fatal errors (immediate shutdown required)
    Fatal,
}

/// Common error codes used across the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    // Configuration errors
    ConfigMissing,
    ConfigInvalid,
    ConfigInconsistent,

    // Validation errors
    ValidationFailed,
    InvalidInput,
    SchemaViolation,

    // Network errors
    ConnectionFailed,
    Timeout,
    ServiceUnavailable,

    // Database errors
    DatabaseConnectionFailed,
    QueryFailed,
    DataIntegrityViolation,

    // Authentication errors
    AuthenticationFailed,
    AuthorizationDenied,
    TokenExpired,

    // Resource errors
    ResourceExhausted,
    RateLimitExceeded,
    QuotaExceeded,

    // Internal errors
    InternalError,
    UnexpectedState,
    OperationFailed,

    // External service errors
    ExternalServiceError,
    DependencyFailure,
    ApiError,
}

/// Standardized error structure used across the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonError {
    /// Unique error code
    pub code: ErrorCode,
    /// Error category for grouping
    pub category: ErrorCategory,
    /// Error severity level
    pub severity: ErrorSeverity,
    /// Human-readable error message
    pub message: String,
    /// Additional context/details
    pub details: Option<String>,
    /// Source component/module where error occurred
    pub source: String,
    /// Optional operation ID for tracing
    pub operation_id: Option<String>,
    /// Optional user-friendly suggestion
    pub suggestion: Option<String>,
}

impl CommonError {
    /// Create a new common error
    pub fn new(
        code: ErrorCode,
        message: String,
        source: String,
    ) -> Self {
        let (category, severity) = Self::categorize_error(code);
        Self {
            code,
            category,
            severity,
            message,
            details: None,
            source,
            operation_id: None,
            suggestion: None,
        }
    }

    /// Create a new error with additional details
    pub fn with_details(
        code: ErrorCode,
        message: String,
        source: String,
        details: String,
    ) -> Self {
        let mut error = Self::new(code, message, source);
        error.details = Some(details);
        error
    }

    /// Create a new error with operation ID
    pub fn with_operation_id(
        code: ErrorCode,
        message: String,
        source: String,
        operation_id: String,
    ) -> Self {
        let mut error = Self::new(code, message, source);
        error.operation_id = Some(operation_id);
        error
    }

    /// Add a suggestion to the error
    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestion = Some(suggestion);
        self
    }

    /// Determine category and severity based on error code
    fn categorize_error(code: ErrorCode) -> (ErrorCategory, ErrorSeverity) {
        match code {
            // Configuration errors
            ErrorCode::ConfigMissing | ErrorCode::ConfigInvalid | ErrorCode::ConfigInconsistent => {
                (ErrorCategory::Configuration, ErrorSeverity::Error)
            }

            // Validation errors
            ErrorCode::ValidationFailed | ErrorCode::InvalidInput | ErrorCode::SchemaViolation => {
                (ErrorCategory::Validation, ErrorSeverity::Warning)
            }

            // Network errors
            ErrorCode::ConnectionFailed | ErrorCode::Timeout | ErrorCode::ServiceUnavailable => {
                (ErrorCategory::Network, ErrorSeverity::Error)
            }

            // Database errors
            ErrorCode::DatabaseConnectionFailed | ErrorCode::QueryFailed | ErrorCode::DataIntegrityViolation => {
                (ErrorCategory::Database, ErrorSeverity::Error)
            }

            // Authentication errors
            ErrorCode::AuthenticationFailed | ErrorCode::AuthorizationDenied | ErrorCode::TokenExpired => {
                (ErrorCategory::Authentication, ErrorSeverity::Warning)
            }

            // Resource errors
            ErrorCode::ResourceExhausted | ErrorCode::RateLimitExceeded | ErrorCode::QuotaExceeded => {
                (ErrorCategory::ResourceLimit, ErrorSeverity::Warning)
            }

            // Internal errors
            ErrorCode::InternalError | ErrorCode::UnexpectedState | ErrorCode::OperationFailed => {
                (ErrorCategory::Internal, ErrorSeverity::Error)
            }

            // External service errors
            ErrorCode::ExternalServiceError | ErrorCode::DependencyFailure | ErrorCode::ApiError => {
                (ErrorCategory::ExternalService, ErrorSeverity::Warning)
            }
        }
    }
}

impl fmt::Display for CommonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.source, self.code as u32, self.message)
    }
}

impl std::error::Error for CommonError {}

/// Result type alias using CommonError
pub type CommonResult<T> = Result<T, CommonError>;

/// Helper functions for creating common errors from standard error types

impl CommonError {
    /// Create a CommonError from an IO error
    pub fn from_io_error(error: std::io::Error, source: String) -> Self {
        Self::new(
            ErrorCode::InternalError,
            format!("IO error: {}", error),
            source,
        )
    }

    /// Create a CommonError from a JSON parsing error
    pub fn from_json_error(error: serde_json::Error, source: String) -> Self {
        Self::new(
            ErrorCode::ValidationFailed,
            format!("JSON parsing error: {}", error),
            source,
        )
    }

    /// Create a CommonError from a generic string error
    pub fn from_string(error: String, source: String) -> Self {
        Self::new(
            ErrorCode::InternalError,
            error,
            source,
        )
    }
}
