//! A2A Error Types

use thiserror::Error;

/// A2A protocol errors.
#[derive(Debug, Error)]
pub enum A2AError {
    /// Task not found.
    #[error("Task not found: {0}")]
    TaskNotFound(String),

    /// Task is in a terminal state and cannot be modified.
    #[error("Task not cancelable: {0}")]
    TaskNotCancelable(String),

    /// Push notifications are not supported by this agent.
    #[error("Push notifications not supported")]
    PushNotSupported,

    /// Requested operation is not supported.
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    /// Content type not supported.
    #[error("Content type not supported: {0}")]
    ContentTypeNotSupported(String),

    /// Missing required parameter.
    #[error("Missing parameter: {0}")]
    MissingParameter(String),

    /// Invalid parameter value.
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Agent handler returned an error during task execution.
    #[error("Handler error: {0}")]
    HandlerError(String),

    /// Governance violation (CAWS).
    #[error("Governance violation: {0}")]
    GovernanceViolation(String),

    /// JSON serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Internal server error.
    #[error("Internal error: {0}")]
    Internal(String),

    /// HTTP client error (for A2A client operations).
    #[error("HTTP error: {0}")]
    Http(String),
}

impl A2AError {
    /// Map to A2A JSON-RPC error code.
    pub fn error_code(&self) -> i32 {
        match self {
            Self::TaskNotFound(_) => -32001,
            Self::TaskNotCancelable(_) => -32002,
            Self::PushNotSupported => -32003,
            Self::UnsupportedOperation(_) => -32004,
            Self::ContentTypeNotSupported(_) => -32005,
            Self::MissingParameter(_) => -32602,
            Self::InvalidParameter(_) => -32602,
            Self::HandlerError(_) => -32000,
            Self::GovernanceViolation(_) => -32001,
            Self::Serialization(_) => -32700,
            Self::Internal(_) => -32603,
            Self::Http(_) => -32000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = A2AError::TaskNotFound("task-123".to_string());
        assert!(err.to_string().contains("task-123"));
    }

    #[test]
    fn test_a2a_specific_error_codes() {
        assert_eq!(A2AError::TaskNotFound("x".into()).error_code(), -32001);
        assert_eq!(A2AError::TaskNotCancelable("x".into()).error_code(), -32002);
        assert_eq!(A2AError::PushNotSupported.error_code(), -32003);
        assert_eq!(A2AError::UnsupportedOperation("x".into()).error_code(), -32004);
        assert_eq!(A2AError::ContentTypeNotSupported("x".into()).error_code(), -32005);
    }

    #[test]
    fn test_standard_error_codes() {
        assert_eq!(A2AError::MissingParameter("x".into()).error_code(), -32602);
        assert_eq!(A2AError::Internal("x".into()).error_code(), -32603);
    }
}
