//! Refinement decision contract for council-directed improvements.
//!
//! Defines the council's decision on current artifacts with targeted
//! refinement directives, risk assessments, and specific improvement actions.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Council-directed refinement decision with targeted improvement focus
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RefinementDecision {
    /// Contract version for compatibility
    pub version: String,

    /// Task identifier
    pub task_id: Uuid,

    /// Working spec identifier
    pub working_spec_id: String,

    /// Current iteration number
    pub iteration: u32,

    /// Council's decision on current artifacts
    pub decision: CouncilDecision,

    /// Council's confidence in the decision (0.0-1.0)
    pub confidence: f64,

    /// Detailed reasoning for the decision
    pub rationale: String,

    /// Evidence supporting the decision
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub evidence: Vec<EvidenceItem>,

    /// Detailed refinement instructions (only for 'refine' decisions)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refinement_directive: Option<RefinementDirective>,

    /// Detailed council voting and consensus information
    pub council_verdict: CouncilVerdict,

    /// Risk assessment and mitigation recommendations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_assessment: Option<RiskAssessment>,

    /// Decision metadata and processing information
    pub metadata: DecisionMetadata,
}

/// Council's decision on current artifacts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CouncilDecision {
    /// Accept current artifacts as meeting requirements
    Accept,

    /// Apply targeted refinements before re-evaluation
    Refine,

    /// Reject artifacts due to fundamental issues
    Reject,

    /// Escalate to human intervention required
    Escalate,
}

/// Evidence supporting the council's decision
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EvidenceItem {
    /// Type of evidence
    pub r#type: EvidenceType,

    /// Evidence description
    pub description: String,

    /// Evidence severity
    pub severity: EvidenceSeverity,

    /// Supporting data and measurements
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supporting_data: Option<serde_json::Value>,
}

/// Type of evidence supporting decision
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    QualityReport,
    TestResults,
    CodeReview,
    PerformanceMetrics,
    SecurityScan,
    AccessibilityAudit,
}

/// Evidence severity levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Detailed refinement instructions for improvement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RefinementDirective {
    /// Areas requiring focused improvement
    pub focus_areas: Vec<FocusArea>,

    /// Priority order for addressing focus areas
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub priority_order: Vec<String>,

    /// Maximum additional iterations allowed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_iterations: Option<u32>,

    /// Specific quality targets to achieve
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality_targets: Option<QualityTargets>,

    /// Suggested constraint adjustments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints_adjustments: Option<ConstraintAdjustments>,

    /// Specific actionable improvements
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub specific_actions: Vec<SpecificAction>,

    /// General guidance for the refinement approach
    pub guidance: String,
}

/// Areas requiring focused improvement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FocusArea {
    UnitTests,
    IntegrationTests,
    E2eTests,
    CodeCoverage,
    MutationTesting,
    Linting,
    TypeChecking,
    Security,
    Performance,
    Accessibility,
    ErrorHandling,
    Documentation,
    CodeStructure,
    ApiDesign,
    DataValidation,
}

/// Specific quality targets to achieve
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct QualityTargets {
    /// Minimum overall quality score required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_overall_score: Option<f64>,

    /// Specific gates that must pass
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub required_gates: Vec<String>,

    /// Issues that must be resolved
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub blocking_issues: Vec<String>,
}

/// Suggested constraint adjustments
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ConstraintAdjustments {
    /// Increase change budget if needed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub increase_budget: Option<BudgetIncrease>,

    /// Extend maximum duration in minutes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extend_timeline: Option<u32>,

    /// Adjust scope restrictions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adjust_scope: Option<ScopeAdjustment>,
}

/// Increase change budget
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BudgetIncrease {
    /// Additional files that can be modified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_files: Option<u32>,

    /// Additional lines of code that can be added/changed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_loc: Option<u32>,
}

/// Adjust scope restrictions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ScopeAdjustment {
    /// Add allowed paths
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub add_allowed_paths: Vec<String>,

    /// Remove blocked paths
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub remove_blocked_paths: Vec<String>,
}

/// Specific actionable improvement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SpecificAction {
    /// Type of action to perform
    pub action_type: ActionType,

    /// Target for the action (file, component, etc.)
    pub target: String,

    /// Description of what to do
    pub description: String,

    /// Action priority
    pub priority: ActionPriority,

    /// Estimated effort to implement
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_effort: Option<EffortLevel>,
}

/// Type of specific action
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    AddTests,
    FixLinting,
    ImproveCoverage,
    AddressSecurity,
    EnhancePerformance,
    AddDocumentation,
    RefactorCode,
    UpdateTypes,
    AddErrorHandling,
    ValidateData,
}

/// Action priority levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionPriority {
    Critical,
    High,
    Medium,
    Low,
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

/// Detailed council voting and consensus information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CouncilVerdict {
    /// Whether quorum was achieved for decision
    pub quorum_achieved: bool,

    /// Total number of judges that participated
    pub total_judges: u32,

    /// Number of judges voting for the final decision
    pub votes_for_decision: u32,

    /// Dissenting opinions from judges
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub dissenting_opinions: Vec<DissentingOpinion>,

    /// Contributions from each judge
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub judge_contributions: Vec<JudgeContribution>,
}

/// Dissenting opinion from a judge
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DissentingOpinion {
    /// Judge identifier
    pub judge_id: String,

    /// Alternative decision proposed
    pub alternative_decision: CouncilDecision,

    /// Rationale for the alternative decision
    pub rationale: String,
}

/// Individual judge's contribution to the decision
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct JudgeContribution {
    /// Judge identifier
    pub judge_id: String,

    /// Judge specialization type
    pub judge_type: JudgeType,

    /// Judge's decision
    pub decision: CouncilDecision,

    /// Judge's confidence in their decision (0.0-1.0)
    pub confidence: f64,

    /// Recommended focus areas for improvement
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub focus_recommendations: Vec<String>,
}

/// Type of judge specialization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JudgeType {
    Quality,
    Security,
    Performance,
    Usability,
    Compliance,
    Architecture,
}

/// Risk assessment and mitigation recommendations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RiskAssessment {
    /// Current risk level assessment
    pub current_risk_level: RiskLevel,

    /// Specific risk factors identified
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub risk_factors: Vec<RiskFactor>,

    /// Whether mitigation is required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mitigation_required: Option<bool>,

    /// Whether escalation is recommended
    #[serde(skip_serializing_if = "Option::is_none")]
    pub escalation_recommended: Option<bool>,

    /// Reason for escalation recommendation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub escalation_reason: Option<String>,
}

/// Risk level assessment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Individual risk factor
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RiskFactor {
    /// Risk factor description
    pub factor: String,

    /// Risk severity
    pub severity: RiskLevel,

    /// Detailed description of the risk
    pub description: String,
}

/// Decision metadata and processing information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DecisionMetadata {
    /// When the decision was made
    pub decided_at: chrono::DateTime<chrono::Utc>,

    /// How long the decision process took
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision_duration_ms: Option<u64>,

    /// Version of the council system used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub council_version: Option<String>,

    /// Versions of models used by judges
    pub model_versions: std::collections::HashMap<String, String>,

    /// Additional processing metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_metadata: Option<serde_json::Value>,
}
