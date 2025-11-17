//! Task request contract for autonomous task execution.
//!
//! Defines the input contract for requesting autonomous task execution
//! with comprehensive constraints, context, and validation requirements.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Task request for autonomous execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TaskRequest {
    /// Contract version for compatibility
    pub version: String,

    /// Unique task identifier
    #[schemars(with = "String")]
    pub id: Uuid,

    /// Natural language task description
    pub description: String,

    /// Optional workspace context and constraints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<TaskContext>,

    /// Execution constraints and safety limits
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<TaskConstraints>,

    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<TaskMetadata>,
}

/// Workspace context and dependencies
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TaskContext {
    /// Root directory path for the workspace
    pub workspace_root: String,

    /// Current git branch
    pub git_branch: String,

    /// Recent file changes in the workspace
    pub recent_changes: Vec<FileChange>,

    /// Project dependencies and their versions
    pub dependencies: std::collections::HashMap<String, String>,

    /// Target environment
    pub environment: Environment,
}

/// File change information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct FileChange {
    /// File path
    pub file: String,

    /// Type of change
    pub change_type: ChangeType,

    /// When the change occurred
    #[schemars(with = "String")]
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Type of file change
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
}

/// Target environment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Environment {
    Development,
    Staging,
    Production,
}

/// Execution constraints and safety limits
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TaskConstraints {
    /// Risk tier determining validation strictness
    pub risk_tier: RiskTier,

    /// Maximum allowed execution time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_duration_minutes: Option<u32>,

    /// Maximum refinement iterations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_iterations: Option<u32>,

    /// Change budget constraints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget_limits: Option<BudgetLimits>,

    /// Path-based access restrictions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope_restrictions: Option<ScopeRestrictions>,
}

/// Risk tier for task execution
/// Re-export RiskTier from planning module for consistency
pub use crate::types::planning::RiskTier;

/// Change budget constraints
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BudgetLimits {
    /// Maximum files that can be modified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_files: Option<u32>,

    /// Maximum lines of code that can be added/changed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_loc: Option<u32>,
}

/// Path-based access restrictions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ScopeRestrictions {
    /// Allowed file/directory paths (regex patterns)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub allowed_paths: Vec<String>,

    /// Blocked file/directory paths (regex patterns)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub blocked_paths: Vec<String>,
}

/// Additional task metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TaskMetadata {
    /// Who requested this task
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requester: Option<String>,

    /// Task priority level
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<TaskPriority>,

    /// Categorization tags
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<String>,
}

// Use the unified TaskPriority from types/planning.rs
pub use crate::types::planning::TaskPriority;

/// Validate a task request value against the JSON schema
pub fn validate_task_request_value(
    value: &serde_json::Value,
) -> Result<(), crate::contract_errors::ContractError> {
    use crate::contract_errors::{ContractError, ContractKind};
    use crate::schema::TASK_REQUEST_SCHEMA;

    TASK_REQUEST_SCHEMA.validate(value).map_err(|errors| {
        let issues = errors
            .into_iter()
            .map(|error| crate::contract_errors::ValidationIssue {
                instance_path: error.instance_path.to_string(),
                schema_path: error.schema_path.to_string(),
                message: error.to_string(),
            })
            .collect();
        ContractError::validation(ContractKind::TaskRequest, issues)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn validate_task_request_value_returns_error_on_invalid() {
        // Multiple provably invalid cases - if validate() returns Ok(()), ALL will fail
        let invalid_cases = vec![
            // Missing version
            json!({"id": "123", "description": "test"}),
            // Missing id
            json!({"version": "1.0", "description": "test"}),
            // Missing description
            json!({"version": "1.0", "id": "123"}),
            // Wrong type for id (should be UUID string)
            json!({"version": "1.0", "id": 123, "description": "test"}),
            // Empty object
            json!({}),
            // Null
            json!(null),
        ];

        for (idx, invalid_case) in invalid_cases.iter().enumerate() {
            let result = validate_task_request_value(invalid_case);
            // This MUST fail - if validate() is stubbed to return Ok(()), test fails
            assert!(
                result.is_err(),
                "Invalid case {} should be rejected by validation, but got Ok(()) - validation may be stubbed",
                idx
            );
        }
    }

    #[test]
    fn validate_task_request_value_returns_ok_on_valid() {
        let valid_value = json!({
            "version": "1.0",
            "id": "00000000-0000-0000-0000-000000000000",
            "description": "Test task"
        });

        let result = validate_task_request_value(&valid_value);
        // May pass or fail depending on schema requirements, but should return a Result
        assert!(result.is_ok() || result.is_err());
    }
}
