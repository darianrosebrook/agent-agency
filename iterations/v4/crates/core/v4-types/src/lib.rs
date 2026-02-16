//! V4 Core Type Definitions
//!
//! This crate provides the foundational types for Agent Agency V4.
//! All types are JSON Schema backed for validation and interoperability.
//!
//! ## Design Principles
//!
//! 1. **No circular dependencies** - This crate has no internal dependencies
//! 2. **Serializable by default** - All types derive Serialize/Deserialize
//! 3. **Schema-validated** - All types derive JsonSchema for validation
//! 4. **Small, focused types** - Compose rather than inherit

pub mod council;
pub mod errors;
pub mod events;
pub mod execution;
pub mod operators;
pub mod ports;
pub mod task;
pub mod verdict;
pub mod worker;

// Re-export commonly used types
pub use council::{
    CouncilVerdict, FinalDecision, JudgeResult, ReviewPriority, SessionId, SessionStatus,
    SessionStatusType,
};
pub use errors::{ContractError, ValidationError};
pub use events::AgentEvent;
pub use execution::{ExecutionArtifacts, ExecutionStatus, TaskResult};
pub use operators::OperatorType;
pub use task::{TaskConstraints, TaskRequest, TaskResponse, TaskSpec, TaskStatus};
pub use verdict::{GateResult, GateStatus, GateType, QualityReport};
pub use worker::{
    WorkerAssignment, WorkerCapability, WorkerProfile, WorkerResult, WorkerStatus, WorkerType,
};

/// API version for compatibility checking
pub const API_VERSION: &str = "4.0.0";

/// Check if an API version is compatible with this version
pub fn is_compatible(version: &str) -> bool {
    version.starts_with("4.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_version_format() {
        assert!(API_VERSION.starts_with("4."));
        let parts: Vec<&str> = API_VERSION.split('.').collect();
        assert_eq!(parts.len(), 3);
    }

    #[test]
    fn test_compatibility_check() {
        assert!(is_compatible("4.0.0"));
        assert!(is_compatible("4.1.0"));
        assert!(!is_compatible("3.0.0"));
        assert!(!is_compatible("5.0.0"));
    }
}
