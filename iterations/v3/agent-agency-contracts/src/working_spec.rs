//! Working specification contract for CAWS-validated task execution.
//!
//! Defines the comprehensive working specification that guides autonomous
//! task execution with detailed constraints, acceptance criteria, and rollback plans.

use serde::{Deserialize, Serialize};
use crate::task_request::Environment;

/// CAWS-validated working specification for task execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WorkingSpec {
    /// Contract version for compatibility
    pub version: String,

    /// Working spec identifier (e.g., FEAT-001, FIX-042)
    pub id: String,

    /// Human-readable title
    pub title: String,

    /// Detailed task description
    pub description: String,

    /// High-level objectives to achieve
    pub goals: Vec<String>,

    /// Risk tier: 1=critical, 2=standard, 3=low
    pub risk_tier: u32,

    /// Execution constraints and safety limits
    pub constraints: WorkingSpecConstraints,

    /// Acceptance criteria in Given-When-Then format
    pub acceptance_criteria: Vec<AcceptanceCriterion>,

    /// Comprehensive test plan
    pub test_plan: TestPlan,

    /// Rollback and recovery procedures
    pub rollback_plan: RollbackPlan,

    /// Workspace context and dependencies
    pub context: WorkingSpecContext,

    /// Non-functional requirements
    #[serde(skip_serializing_if = "Option::is_none")]
    pub non_functional_requirements: Option<NonFunctionalRequirements>,

    /// CAWS validation results
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_results: Option<ValidationResults>,

    /// Metadata and versioning information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<WorkingSpecMetadata>,
}

/// Execution constraints and safety limits
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WorkingSpecConstraints {
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

/// Acceptance criterion in Given-When-Then format
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AcceptanceCriterion {
    /// Criterion identifier (e.g., A1, A2)
    pub id: String,

    /// Precondition context
    pub given: String,

    /// Action or event that occurs
    pub when: String,

    /// Expected outcome or behavior
    pub then: String,

    /// MoSCoW priority
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<MoSCoWPriority>,
}

/// MoSCoW priority levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MoSCoWPriority {
    Must,
    Should,
    Could,
    #[serde(rename = "won't")]
    Wont,
}

/// Comprehensive test plan
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TestPlan {
    /// Unit test specifications
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub unit_tests: Vec<UnitTestSpec>,

    /// Integration test specifications
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub integration_tests: Vec<IntegrationTestSpec>,

    /// End-to-end test scenarios
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub e2e_scenarios: Vec<E2eScenario>,

    /// Test coverage targets
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coverage_targets: Option<CoverageTargets>,
}

/// Unit test specification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UnitTestSpec {
    /// Test description
    pub description: String,

    /// Target function to test
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_function: Option<String>,

    /// Specific test cases to cover
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub test_cases: Vec<String>,
}

/// Integration test specification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct IntegrationTestSpec {
    /// Test description
    pub description: String,

    /// Components being tested together
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub components: Vec<String>,

    /// Specific test cases
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub test_cases: Vec<String>,
}

/// End-to-end test scenario
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct E2eScenario {
    /// Scenario description
    pub description: String,

    /// User journey description
    pub user_journey: String,

    /// Expected outcomes
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub expected_outcomes: Vec<String>,
}

/// Test coverage targets
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CoverageTargets {
    /// Target line coverage (0.0-1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_coverage: Option<f64>,

    /// Target branch coverage (0.0-1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch_coverage: Option<f64>,

    /// Target mutation score (0.0-1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mutation_score: Option<f64>,
}

/// Rollback and recovery procedures
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RollbackPlan {
    /// Rollback strategy to use
    pub strategy: RollbackStrategy,

    /// Automated rollback steps
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub automated_steps: Vec<String>,

    /// Manual rollback steps required
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub manual_steps: Vec<String>,

    /// Impact on persistent data
    pub data_impact: DataImpact,

    /// Whether downtime is required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downtime_required: Option<bool>,

    /// Time window for successful rollback
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rollback_window_minutes: Option<u32>,
}

/// Rollback strategy options
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RollbackStrategy {
    /// Git revert to previous commit
    GitRevert,

    /// Feature flag rollback
    FeatureFlag,

    /// Manual revert with scripts
    ManualRevert,

    /// Database migration rollback
    DatabaseMigration,
}

/// Impact on persistent data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataImpact {
    /// No persistent data impact
    None,

    /// Changes are reversible
    Reversible,

    /// Data will be lost/destructive
    Destructive,
}

/// Workspace context and dependencies
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WorkingSpecContext {
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

/// Non-functional requirements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct NonFunctionalRequirements {
    /// Performance requirements
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance: Option<PerformanceRequirements>,

    /// Security requirements
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub security: Vec<String>,

    /// Accessibility requirements
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub accessibility: Vec<String>,

    /// Scalability requirements
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scalability: Option<ScalabilityRequirements>,
}

/// Performance requirements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PerformanceRequirements {
    /// Response time requirement in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_time_ms: Option<u32>,

    /// Throughput requirement in requests per second
    #[serde(skip_serializing_if = "Option::is_none")]
    pub throughput_req_per_sec: Option<u32>,

    /// Memory limit in MB
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_limit_mb: Option<u32>,

    /// CPU limit as percentage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_limit_percent: Option<u32>,
}

/// Scalability requirements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ScalabilityRequirements {
    /// Concurrent users to support
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrent_users: Option<u32>,

    /// Data retention period in days
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_retention_days: Option<u32>,
}

/// CAWS validation results
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ValidationResults {
    /// Whether the spec is CAWS compliant
    pub caws_compliant: bool,

    /// CAWS validation violations
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub violations: Vec<String>,

    /// CAWS validation suggestions
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub suggestions: Vec<String>,

    /// Overall quality score (0.0-1.0)
    pub quality_score: f64,

    /// When validation was performed
    pub validated_at: chrono::DateTime<chrono::Utc>,
}

/// Metadata and versioning information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WorkingSpecMetadata {
    /// When the spec was created
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Who created this spec
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,

    /// Last modification timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,

    /// Version number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<u32>,

    /// Categorization tags
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<String>,
}

/// Validate a working spec value against the JSON schema
pub fn validate_working_spec_value(value: &serde_json::Value) -> Result<(), crate::error::ContractError> {
    use crate::error::{ContractError, ContractKind};
    use crate::schema::WORKING_SPEC_SCHEMA;

    WORKING_SPEC_SCHEMA.validate(value).map_err(|errors| {
        let issues = errors
            .into_iter()
            .map(|error| crate::error::ValidationIssue {
                instance_path: error.instance_path.to_string(),
                schema_path: error.schema_path.to_string(),
                message: error.to_string(),
            })
            .collect();
        ContractError::validation(ContractKind::WorkingSpec, issues)
    })
}
