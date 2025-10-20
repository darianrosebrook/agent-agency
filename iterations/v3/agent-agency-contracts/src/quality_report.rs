//! Quality report contract for comprehensive quality assessment.
//!
//! Defines the quality assessment results with gate statuses, thresholds,
//! performance metrics, and actionable recommendations for improvement.

use serde::{Deserialize, Serialize};

/// Comprehensive quality assessment with gate results and thresholds
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct QualityReport {
    /// Contract version for compatibility
    pub version: String,

    /// Task identifier
    pub task_id: uuid::Uuid,

    /// Working spec identifier
    pub working_spec_id: String,

    /// Execution iteration number
    pub iteration: u32,

    /// Weighted overall quality score (0.0-1.0)
    pub overall_score: f64,

    /// Overall assessment status
    pub overall_status: OverallStatus,

    /// Results from all quality gates
    pub gates: Vec<GateResult>,

    /// Quality thresholds based on risk tier
    pub thresholds: QualityThresholds,

    /// Quality score deltas and trends
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deltas: Option<QualityDeltas>,

    /// Performance metrics for quality gate execution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance_metrics: Option<GatePerformanceMetrics>,

    /// Actionable recommendations for improvement
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub recommendations: Vec<Recommendation>,

    /// Report generation metadata
    pub metadata: ReportMetadata,
}

/// Overall assessment status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OverallStatus {
    /// All gates passed, meets all thresholds
    Passed,

    /// Some gates failed but overall acceptable
    Failed,

    /// Some warnings but no failures
    Warning,

    /// Partial results available
    Partial,
}

/// Result from individual quality gate execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GateResult {
    /// Gate name/identifier
    pub gate_name: String,

    /// Type of quality gate
    pub gate_type: GateType,

    /// Gate execution status
    pub status: GateStatus,

    /// Quality score for this gate (0.0-1.0)
    pub score: f64,

    /// Required threshold for passing
    pub threshold: f64,

    /// Execution duration in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,

    /// Issues found during gate execution
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub issues: Vec<GateIssue>,

    /// Gate-specific metrics and measurements
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<serde_json::Value>,

    /// When the gate was executed
    pub executed_at: chrono::DateTime<chrono::Utc>,

    /// Command or tool used to execute the gate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_used: Option<String>,
}

/// Type of quality gate
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateType {
    Lint,
    TypeCheck,
    UnitTest,
    IntegrationTest,
    E2eTest,
    Coverage,
    Mutation,
    Security,
    Performance,
    Accessibility,
}

/// Gate execution status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateStatus {
    Passed,
    Failed,
    Warning,
    Skipped,
    Error,
    Timeout,
}

/// Individual issue found by a quality gate
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GateIssue {
    /// Issue severity level
    pub severity: IssueSeverity,

    /// Issue code or identifier
    pub code: String,

    /// Human-readable message
    pub message: String,

    /// Affected file path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,

    /// Line number in file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,

    /// Column number in line
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,

    /// Suggested fix or resolution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

/// Issue severity levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

/// Quality thresholds based on risk tier
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct QualityThresholds {
    /// Risk tier determining threshold stringency
    pub risk_tier: u32,

    /// Gate-specific threshold requirements
    pub gate_thresholds: std::collections::HashMap<String, f64>,

    /// Overall quality score threshold
    pub overall_threshold: f64,

    /// Gate names that must pass (no warnings allowed)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub blocking_gates: Vec<String>,
}

/// Quality score deltas and trends
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct QualityDeltas {
    /// Changes from previous iteration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_previous_iteration: Option<IterationDelta>,

    /// Changes from project baseline
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_baseline: Option<BaselineDelta>,
}

/// Changes from previous iteration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct IterationDelta {
    /// Overall score change
    pub overall_score_delta: f64,

    /// Gates that improved since last iteration
    pub gates_improved: u32,

    /// Gates that regressed since last iteration
    pub gates_regressed: u32,

    /// Gates newly passing
    pub gates_newly_passing: u32,

    /// Gates newly failing
    pub gates_newly_failing: u32,
}

/// Changes from project baseline
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BaselineDelta {
    /// Overall score change from baseline
    pub overall_score_delta: f64,

    /// Gates that improved from baseline
    pub gates_improved: u32,

    /// Gates that regressed from baseline
    pub gates_regressed: u32,
}

/// Performance metrics for quality gate execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GatePerformanceMetrics {
    /// Total execution time for all gates
    pub total_execution_time_ms: u64,

    /// Time spent actually executing gates (vs overhead)
    pub gates_execution_time_ms: u64,

    /// Name of the slowest gate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slowest_gate: Option<String>,

    /// Name of the fastest gate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fastest_gate: Option<String>,

    /// How well gates were parallelized (1.0 = perfect)
    pub parallelization_efficiency: f64,

    /// Resource usage during gate execution
    pub resource_usage: ResourceUsage,
}

/// Resource usage during quality gate execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ResourceUsage {
    /// Peak memory usage in MB
    pub peak_memory_mb: u32,

    /// Total CPU time used in milliseconds
    pub cpu_time_ms: u64,

    /// Number of I/O operations performed
    pub io_operations: u64,
}

/// Actionable recommendation for improvement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Recommendation {
    /// Recommendation priority
    pub priority: RecommendationPriority,

    /// Category of improvement
    pub category: RecommendationCategory,

    /// Specific action to take
    pub action: String,

    /// Rationale for the recommendation
    pub rationale: String,

    /// Estimated effort to implement
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_effort: Option<EffortLevel>,

    /// Whether this can be automated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub automated: Option<bool>,
}

/// Recommendation priority levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// Category of improvement recommendation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationCategory {
    Testing,
    CodeQuality,
    Performance,
    Security,
    Accessibility,
    Maintainability,
}

/// Effort level estimation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffortLevel {
    Trivial,
    Small,
    Medium,
    Large,
}

/// Report generation metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ReportMetadata {
    /// When the report was generated
    pub generated_at: chrono::DateTime<chrono::Utc>,

    /// Version of the quality assessment system
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generator_version: Option<String>,

    /// Target environment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,

    /// Configuration used for assessment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_used: Option<serde_json::Value>,

    /// Whether caching was used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caching_used: Option<bool>,

    /// Whether this was an incremental run
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incremental_run: Option<bool>,
}

/// Validate a quality report value against the JSON schema
pub fn validate_quality_report_value(value: &serde_json::Value) -> Result<(), crate::error::ContractError> {
    use crate::error::{ContractError, ContractKind};
    use crate::schema::QUALITY_REPORT_SCHEMA;

    QUALITY_REPORT_SCHEMA.validate(value).map_err(|errors| {
        let issues = errors
            .into_iter()
            .map(|error| crate::error::ValidationIssue {
                instance_path: error.instance_path.to_string(),
                schema_path: error.schema_path.to_string(),
                message: error.to_string(),
            })
            .collect();
        ContractError::validation(ContractKind::QualityReport, issues)
    })
}
