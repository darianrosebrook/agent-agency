//! API Error types and handling
//!
//! Provides standardized error types for API operations with proper
//! HTTP status code mapping and error serialization.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use schemars::JsonSchema;
use serde::Serialize;
use std::fmt;
use uuid::Uuid;

/// Standardized error response format matching open-webui patterns
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ErrorResponse {
    /// Human-readable error message
    pub error: String,
    /// Machine-readable error code
    pub code: String,
    /// HTTP status code
    pub status: u16,
    /// Additional error details (optional)
    pub details: Option<serde_json::Value>,
    /// Request ID for correlation (optional)
    pub request_id: Option<String>,
}

/// Standardized API error types
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub enum ApiError {
    /// Database operation failed
    DatabaseError(String),
    /// Resource not found
    NotFound(String),
    /// Task not found
    TaskNotFound(String),
    /// Invalid operation
    InvalidOperation(String),
    /// Invalid request
    InvalidRequest(String),
    /// Execution error
    ExecutionError(String),
    /// Validation error
    ValidationError(String),
    /// Authentication error
    AuthenticationError(String),
    /// Authorization error
    AuthorizationError(String),
    /// Rate limit exceeded
    RateLimitExceeded(String),
    /// Internal server error
    InternalError(String),
    /// Bad request
    BadRequest(String),
}

impl ApiError {
    /// Get machine-readable error code
    pub fn error_code(&self) -> &'static str {
        match self {
            ApiError::DatabaseError(_) => "DATABASE_ERROR",
            ApiError::NotFound(_) => "NOT_FOUND",
            ApiError::TaskNotFound(_) => "TASK_NOT_FOUND",
            ApiError::InvalidOperation(_) => "INVALID_OPERATION",
            ApiError::InvalidRequest(_) => "INVALID_REQUEST",
            ApiError::ExecutionError(_) => "EXECUTION_ERROR",
            ApiError::ValidationError(_) => "VALIDATION_ERROR",
            ApiError::AuthenticationError(_) => "AUTHENTICATION_ERROR",
            ApiError::AuthorizationError(_) => "AUTHORIZATION_ERROR",
            ApiError::RateLimitExceeded(_) => "RATE_LIMIT_EXCEEDED",
            ApiError::InternalError(_) => "INTERNAL_ERROR",
            ApiError::BadRequest(_) => "BAD_REQUEST",
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            ApiError::DatabaseError(msg) => msg,
            ApiError::NotFound(msg) => msg,
            ApiError::TaskNotFound(msg) => msg,
            ApiError::InvalidOperation(msg) => msg,
            ApiError::InvalidRequest(msg) => msg,
            ApiError::ExecutionError(msg) => msg,
            ApiError::ValidationError(msg) => msg,
            ApiError::AuthenticationError(msg) => msg,
            ApiError::AuthorizationError(msg) => msg,
            ApiError::RateLimitExceeded(msg) => msg,
            ApiError::InternalError(msg) => msg,
            ApiError::BadRequest(msg) => msg,
        };
        write!(f, "{}", message)
    }
}

impl std::error::Error for ApiError {}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        // Get error code before moving self
        let error_code = self.error_code().to_string();

        let (status, error_message) = match self {
            ApiError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::TaskNotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::InvalidOperation(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::InvalidRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::ExecutionError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::AuthenticationError(msg) => (StatusCode::UNAUTHORIZED, msg),
            ApiError::AuthorizationError(msg) => (StatusCode::FORBIDDEN, msg),
            ApiError::RateLimitExceeded(msg) => (StatusCode::TOO_MANY_REQUESTS, msg),
            ApiError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
        };

        // Generate request ID for error correlation
        let request_id = Uuid::new_v4().to_string();

        let error_response = ErrorResponse {
            error: error_message.clone(),
            code: error_code,
            status: status.as_u16(),
            details: None,
            request_id: Some(request_id),
        };

        let body = Json(error_response);

        (status, body).into_response()
    }
}

/// Result type alias for API operations
pub type Result<T> = std::result::Result<T, ApiError>;
