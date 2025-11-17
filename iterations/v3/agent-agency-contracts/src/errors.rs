//! Error taxonomy for contracts
//!
//! Flat, serializable error enums for ports and adapters.
//! These errors are designed to be:
//! - Serializable across process boundaries
//! - Flat (no nesting) for simple handling
//! - Domain-specific but minimal
//! - Convertible from internal errors at boundaries
//!
//! @author @darianrosebrook

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Core planning domain errors
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "data")]
pub enum PlanningError {
    /// Invalid task descriptor data
    InvalidTaskDescriptor { field: String, reason: String },
    /// Planning constraints violated
    ConstraintViolation { constraint: String, details: String },
    /// Resource allocation failed
    ResourceAllocationFailed {
        resource: String,
        required: u32,
        available: u32,
    },
    /// Dependency cycle detected
    DependencyCycle { nodes: Vec<String> },
    /// Quality gate failed
    QualityGateFailed {
        gate: String,
        actual: String,
        required: String,
    },
    /// Execution plan generation failed
    PlanGenerationFailed { reason: String },
}

/// Model and inference domain errors
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "data")]
pub enum ModelError {
    /// Model not found or unavailable
    ModelNotFound { model_id: String },
    /// Model inference failed
    InferenceFailed { model_id: String, reason: String },
    /// Invalid input format for model
    InvalidInput {
        expected_format: String,
        provided_format: String,
    },
    /// Model resource exhausted (memory, compute)
    ResourceExhausted { resource: String, limit: String },
    /// Model initialization failed
    InitializationFailed { model_id: String, reason: String },
}

/// Database and persistence domain errors
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "data")]
pub enum DatabaseError {
    /// Connection to database failed
    ConnectionFailed { host: String, reason: String },
    /// Query execution failed
    QueryFailed {
        table: String,
        operation: String,
        reason: String,
    },
    /// Record not found
    NotFound { resource: String, id: String },
    /// Data integrity constraint violated
    IntegrityViolation { constraint: String, details: String },
    /// Transaction failed
    TransactionFailed { reason: String },
}

/// Security and authentication domain errors
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "data")]
pub enum SecurityError {
    /// Authentication failed
    AuthenticationFailed { reason: String },
    /// Authorization denied
    AuthorizationDenied { resource: String, action: String },
    /// Invalid credentials provided
    InvalidCredentials { field: String },
    /// Token expired or invalid
    TokenInvalid { token_type: String },
    /// Rate limit exceeded
    RateLimitExceeded {
        limit: String,
        reset_in_seconds: u32,
    },
}

/// Configuration domain errors
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "data")]
pub enum ConfigError {
    /// Required configuration missing
    MissingConfig { key: String },
    /// Configuration value invalid
    InvalidConfig {
        key: String,
        value: String,
        expected: String,
    },
    /// Configuration file not found
    ConfigFileNotFound { path: String },
    /// Environment variable not set
    EnvVarMissing { var_name: String },
}

/// External service integration errors
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "data")]
pub enum ServiceError {
    /// External service unavailable
    ServiceUnavailable { service: String, reason: String },
    /// Service request timeout
    Timeout {
        service: String,
        timeout_seconds: u32,
    },
    /// Invalid response from service
    InvalidResponse {
        service: String,
        status_code: u16,
        reason: String,
    },
    /// Service rate limit exceeded
    ServiceRateLimited {
        service: String,
        retry_after_seconds: u32,
    },
}

/// Memory system domain errors
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "data")]
pub enum MemoryError {
    /// Memory operation failed
    OperationFailed { operation: String, reason: String },
    /// Memory entry not found
    NotFound { memory_id: String },
    /// Invalid memory data format
    InvalidData { reason: String },
    /// Memory storage capacity exceeded
    CapacityExceeded { limit: usize, attempted: usize },
    /// Memory system initialization failed
    InitializationFailed { reason: String },
}

/// Validation domain errors
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "data")]
pub enum ValidationError {
    /// Required field missing
    RequiredFieldMissing { field: String },
    /// Field value invalid
    InvalidFieldValue {
        field: String,
        value: String,
        reason: String,
    },
    /// Data format invalid
    InvalidFormat { expected: String, provided: String },
    /// Size limit exceeded
    SizeLimitExceeded {
        field: String,
        max_size: usize,
        actual_size: usize,
    },
}

/// Generic operational errors
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "data")]
pub enum OperationalError {
    /// Internal system error
    InternalError { component: String, reason: String },
    /// Feature not implemented
    NotImplemented { feature: String },
    /// System resource exhausted
    SystemOverload {
        resource: String,
        current_usage: String,
    },
    /// Maintenance mode active
    MaintenanceMode {
        reason: String,
        estimated_completion: Option<String>,
    },
}

// Council domain errors
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "data")]
pub enum CouncilError {
    /// Session creation or management failed
    SessionError {
        session_id: Option<String>,
        reason: String,
    },
    /// Task review process failed
    ReviewError { session_id: String, reason: String },
    /// Judge selection or coordination failed
    JudgeError {
        judge_id: Option<String>,
        reason: String,
    },
    /// Verdict aggregation failed
    AggregationError { reason: String },
    /// Decision making process failed
    DecisionError { reason: String },
    /// Session timeout occurred
    Timeout {
        session_id: String,
        timeout_seconds: u64,
    },
    /// Invalid session state for requested operation
    InvalidState {
        session_id: String,
        current_state: String,
        required_state: String,
    },
}

impl std::fmt::Display for CouncilError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CouncilError::SessionError { session_id, reason } => {
                write!(
                    f,
                    "Session error{}: {}",
                    session_id
                        .as_ref()
                        .map(|id| format!(" (session {})", id))
                        .unwrap_or_default(),
                    reason
                )
            }
            CouncilError::ReviewError { session_id, reason } => {
                write!(f, "Review error for session {}: {}", session_id, reason)
            }
            CouncilError::JudgeError { judge_id, reason } => {
                write!(
                    f,
                    "Judge error{}: {}",
                    judge_id
                        .as_ref()
                        .map(|id| format!(" (judge {})", id))
                        .unwrap_or_default(),
                    reason
                )
            }
            CouncilError::AggregationError { reason } => {
                write!(f, "Verdict aggregation error: {}", reason)
            }
            CouncilError::DecisionError { reason } => {
                write!(f, "Decision error: {}", reason)
            }
            CouncilError::Timeout {
                session_id,
                timeout_seconds,
            } => {
                write!(
                    f,
                    "Session {} timed out after {} seconds",
                    session_id, timeout_seconds
                )
            }
            CouncilError::InvalidState {
                session_id,
                current_state,
                required_state,
            } => {
                write!(
                    f,
                    "Invalid state for session {}: current={}, required={}",
                    session_id, current_state, required_state
                )
            }
        }
    }
}

// Data processing domain errors
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "data")]
pub enum DataProcessingError {
    /// Unsupported data format
    UnsupportedFormat { format: String },
    /// Data validation failed
    ValidationFailed { reason: String },
    /// Processing operation failed
    ProcessingFailed { operation: String, reason: String },
    /// File system operation failed
    FileOperationFailed {
        operation: String,
        path: String,
        reason: String,
    },
    /// Resource exhausted during processing
    ResourceExhausted { resource: String, limit: String },
    /// External service unavailable
    ServiceUnavailable { service: String },
    /// Invalid processing context
    InvalidContext { field: String, reason: String },
    /// Processing timeout exceeded
    Timeout { operation: String, timeout_ms: u64 },
    /// Data corruption detected
    DataCorruption { reason: String },
}

// Type aliases for common error patterns
pub type PlanningResult<T> = Result<T, PlanningError>;
pub type ModelResult<T> = Result<T, ModelError>;
pub type DatabaseResult<T> = Result<T, DatabaseError>;
pub type MemoryResult<T> = Result<T, MemoryError>;
pub type CouncilResult<T> = Result<T, CouncilError>;
pub type SecurityResult<T> = Result<T, SecurityError>;
pub type ConfigResult<T> = Result<T, ConfigError>;
pub type ServiceResult<T> = Result<T, ServiceError>;
pub type ValidationResult<T> = Result<T, ValidationError>;
pub type OperationalResult<T> = Result<T, OperationalError>;
pub type ResearchResult<T> = Result<T, PlanningError>;
pub type ToolChainResult<T> = Result<T, PlanningError>;
pub type DataProcessingResult<T> = Result<T, DataProcessingError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_serialization() {
        let error = PlanningError::InvalidTaskDescriptor {
            field: "priority".to_string(),
            reason: "must be valid TaskPriority enum value".to_string(),
        };

        let serialized = serde_json::to_string(&error).unwrap();
        let deserialized: PlanningError = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            PlanningError::InvalidTaskDescriptor { field, reason } => {
                assert_eq!(field, "priority");
                assert_eq!(reason, "must be valid TaskPriority enum value");
            }
            _ => panic!("Wrong error variant"),
        }
    }

    #[test]
    fn test_council_error_serialization() {
        let error = CouncilError::SessionError {
            session_id: Some("session-123".to_string()),
            reason: "judge selection failed".to_string(),
        };

        let serialized = serde_json::to_string(&error).unwrap();
        let deserialized: CouncilError = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            CouncilError::SessionError { session_id, reason } => {
                assert_eq!(session_id, Some("session-123".to_string()));
                assert_eq!(reason, "judge selection failed");
            }
            _ => panic!("Wrong error variant"),
        }
    }

    #[test]
    fn test_schema_generation() {
        let schema = schemars::schema_for!(PlanningError);
        let json_schema = serde_json::to_string_pretty(&schema).unwrap();
        assert!(json_schema.contains("InvalidTaskDescriptor"));
    }

    #[test]
    fn council_error_display_session_error_with_id() {
        let error = CouncilError::SessionError {
            session_id: Some("session-123".to_string()),
            reason: "judge selection failed".to_string(),
        };
        let formatted = error.to_string();
        assert!(formatted.contains("Session error"));
        assert!(formatted.contains("session-123"));
        assert!(formatted.contains("judge selection failed"));
    }

    #[test]
    fn council_error_display_session_error_without_id() {
        let error = CouncilError::SessionError {
            session_id: None,
            reason: "connection failed".to_string(),
        };
        let formatted = error.to_string();
        assert!(formatted.contains("Session error"));
        assert!(formatted.contains("connection failed"));
        assert!(!formatted.contains("session"));
    }

    #[test]
    fn council_error_display_review_error() {
        let error = CouncilError::ReviewError {
            session_id: "session-456".to_string(),
            reason: "invalid verdict format".to_string(),
        };
        let formatted = error.to_string();
        assert!(formatted.contains("Review error for session"));
        assert!(formatted.contains("session-456"));
        assert!(formatted.contains("invalid verdict format"));
    }

    #[test]
    fn council_error_display_judge_error_with_id() {
        let error = CouncilError::JudgeError {
            judge_id: Some("judge-tech".to_string()),
            reason: "timeout occurred".to_string(),
        };
        let formatted = error.to_string();
        assert!(formatted.contains("Judge error"));
        assert!(formatted.contains("judge-tech"));
        assert!(formatted.contains("timeout occurred"));
    }

    #[test]
    fn council_error_display_judge_error_without_id() {
        let error = CouncilError::JudgeError {
            judge_id: None,
            reason: "processing failed".to_string(),
        };
        let formatted = error.to_string();
        assert!(formatted.contains("Judge error"));
        assert!(formatted.contains("processing failed"));
        assert!(!formatted.contains("judge"));
    }

    #[test]
    fn council_error_display_aggregation_error() {
        let error = CouncilError::AggregationError {
            reason: "insufficient votes".to_string(),
        };
        let formatted = error.to_string();
        assert!(formatted.contains("Verdict aggregation error"));
        assert!(formatted.contains("insufficient votes"));
    }

    #[test]
    fn council_error_display_decision_error() {
        let error = CouncilError::DecisionError {
            reason: "conflict detected".to_string(),
        };
        let formatted = error.to_string();
        assert!(formatted.contains("Decision error"));
        assert!(formatted.contains("conflict detected"));
    }

    #[test]
    fn council_error_display_timeout() {
        let error = CouncilError::Timeout {
            session_id: "session-789".to_string(),
            timeout_seconds: 30,
        };
        let formatted = error.to_string();
        assert!(formatted.contains("Session"));
        assert!(formatted.contains("session-789"));
        assert!(formatted.contains("timed out"));
        assert!(formatted.contains("30"));
        assert!(formatted.contains("seconds"));
    }

    #[test]
    fn council_error_display_invalid_state() {
        let error = CouncilError::InvalidState {
            session_id: "session-999".to_string(),
            current_state: "pending".to_string(),
            required_state: "active".to_string(),
        };
        let formatted = error.to_string();
        assert!(formatted.contains("Invalid state for session"));
        assert!(formatted.contains("session-999"));
        assert!(formatted.contains("current=pending"));
        assert!(formatted.contains("required=active"));
    }
}
