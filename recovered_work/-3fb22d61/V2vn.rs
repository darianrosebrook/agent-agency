//! Task request contract for autonomous task execution.
//!
//! Defines the input contract for requesting autonomous task execution
//! with comprehensive constraints, context, and validation requirements.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Task request for autonomous execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TaskRequest {
    /// Contract version for compatibility
    pub version: String,

    /// Unique task identifier
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FileChange {
    /// File path
    pub file: String,

    /// Type of change
    pub change_type: ChangeType,

    /// When the change occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Type of file change
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
}

/// Target environment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Environment {
    Development,
    Staging,
    Production,
}

/// Execution constraints and safety limits
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskTier {
    /// Tier 1: Critical - highest scrutiny, manual approval required
    Tier1 = 1,

    /// Tier 2: Standard - balanced validation and automation
    Tier2 = 2,

    /// Tier 3: Low risk - minimal validation, full automation
    Tier3 = 3,
}

/// Change budget constraints
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

/// Task priority levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Urgent,
}
