//! API Error Handling and Response Types
//!
//! Defines the API error enum and Axum response conversions for consistent
//! error handling across all API endpoints.

use axum::http::StatusCode;
use axum::response::Json;
use serde::Serialize;
use uuid::Uuid;

/// API result type alias for convenience
pub type Result<T> = std::result::Result<T, ApiError>;

/// API error types with detailed error information
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Task not found: {0}")]
    TaskNotFound(Uuid),

    #[error("Task execution failed: {0}")]
    ExecutionError(String),

    #[error("Internal server error: {0}")]
    InternalError(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Database operation failed: {0}")]
    DatabaseError(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Authentication required")]
    Unauthorized,

    #[error("Insufficient permissions: {0}")]
    Forbidden(String),
}

// Axum error response conversion
impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            ApiError::TaskNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::ExecutionError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ApiError::InternalError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
            ApiError::InvalidRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ApiError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()),
            ApiError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::InvalidOperation(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "API key required".to_string()),
            ApiError::Forbidden(_) => (StatusCode::FORBIDDEN, self.to_string()),
        };

        let body = serde_json::json!({
            "error": message,
            "code": format!("{:?}", self).split('(').next().unwrap_or("Unknown")
        });

        (status, Json(body)).into_response()
    }
}
