//! Judge verdict types and core data structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::types::{RiskTier, WorkingSpec, WorkingSpecScope};

/// Judge verdict on a working specification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "verdict_type")]
pub enum JudgeVerdict {
    /// Approve the working specification for execution
    Approve {
        confidence: f64,
        reasoning: String,
        quality_score: f64,
        risk_assessment: RiskAssessment,
    },

    /// Request refinements before approval
    Refine {
        confidence: f64,
        reasoning: String,
        required_changes: Vec<RequiredChange>,
        priority: ChangePriority,
        estimated_effort: EffortEstimate,
    },

    /// Reject the working specification
    Reject {
        confidence: f64,
        reasoning: String,
        critical_issues: Vec<String>,
        compliance_violations: Vec<String>,
    },
}

/// Risk assessment for working specifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub overall_risk: RiskLevel,
    pub risk_factors: Vec<RiskFactor>,
    pub mitigation_strategies: Vec<String>,
    pub confidence_score: f64,
}

/// Risk levels for assessments
#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Individual risk factors
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskFactor {
    pub factor_type: RiskFactorType,
    pub severity: RiskSeverity,
    pub description: String,
    pub probability: f64,
    pub impact: f64,
}

/// Types of risk factors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskFactorType {
    Security,
    Performance,
    Reliability,
    Compliance,
    Complexity,
    ResourceUsage,
}

/// Severity levels for risk factors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Required changes for refinement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequiredChange {
    pub change_type: ChangeType,
    pub description: String,
    pub affected_components: Vec<String>,
    pub breaking_change: bool,
    pub test_required: bool,
}

/// Types of changes that can be required
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    SecurityFix,
    PerformanceOptimization,
    CodeQuality,
    Documentation,
    Testing,
    Architecture,
    Configuration,
}



/// Judge trait for different types of judges
#[async_trait::async_trait]
pub trait Judge: Send + Sync {
    /// Get the judge's unique identifier
    fn id(&self) -> Uuid;

    /// Get the judge's specialization type
    fn judge_type(&self) -> JudgeType;

    /// Get the judge's configuration
    fn config(&self) -> &JudgeConfig;

    /// Check if the judge is available for evaluation
    fn is_available(&self) -> bool {
        true // Default implementation
    }

    /// Get health metrics for the judge
    fn health_metrics(&self) -> HealthMetrics {
        HealthMetrics {
            response_time_p95_ms: 100.0, // Default
            success_rate: 0.95,
            error_rate: 0.05,
            last_health_check: chrono::Utc::now(),
        }
    }

    /// Calculate specialization score for a review context
    fn specialization_score(&self, context: &ReviewContext) -> f64 {
        // Default implementation based on judge type matching risk tier
        match (self.judge_type(), context.risk_tier) {
            (JudgeType::QualityAssurance, RiskTier::Tier1) => 0.9,
            (JudgeType::Security, RiskTier::Tier1) => 0.9,
            (JudgeType::QualityAssurance, RiskTier::Tier2) => 0.8,
            (JudgeType::Technical, RiskTier::Tier2) => 0.8,
            _ => 0.7,
        }
    }

    /// Evaluate a working specification
    async fn evaluate(
        &self,
        spec_id: Uuid,
        title: &str,
        description: &str,
        acceptance_criteria: &[String],
    ) -> Result<JudgeVerdict, Box<dyn std::error::Error + Send + Sync>>;

    /// Get judge capabilities and constraints
    fn capabilities(&self) -> JudgeCapabilities;

    /// Check if judge is healthy and ready
    async fn health_check(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Health metrics for judges
#[derive(Debug, Clone)]
pub struct HealthMetrics {
    pub response_time_p95_ms: f64,
    pub success_rate: f64,
    pub error_rate: f64,
    pub last_health_check: chrono::DateTime<chrono::Utc>,
}

/// Judge capabilities and constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeCapabilities {
    pub supported_domains: Vec<String>,
    pub max_spec_length: usize,
    pub requires_network: bool,
    pub processing_timeout_seconds: u64,
    pub confidence_threshold: f64,
}

/// Configuration for judge panels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgePanelConfig {
    pub judges: Vec<JudgeConfig>,
    pub consensus_threshold: f64,
    pub timeout_seconds: u64,
    pub max_retries: u32,
}


impl Default for JudgePanelConfig {
    fn default() -> Self {
        Self {
            judges: vec![],
            consensus_threshold: 0.7,
            timeout_seconds: 300,
            max_retries: 3,
        }
    }
}

impl Default for RiskAssessment {
    fn default() -> Self {
        Self {
            overall_risk: RiskLevel::Low,
            risk_factors: vec![],
            mitigation_strategies: vec![],
            confidence_score: 0.8,
        }
    }
}

/// Judge specialization types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JudgeType {
    Constitutional,     // CAWS compliance and constitutional analysis
    QualityAssurance,
    Security,
    Performance,
    Architecture,
    Testing,
    Compliance,
    DomainExpert,
    Ethics, // Advanced ethical reasoning judge
    Technical,
    Quality,
    Integration,
    Unknown,
}

/// Judge configuration
#[derive(Debug, Clone)]
pub struct JudgeConfig {
    pub judge_id: String,
    pub judge_type: JudgeType,
    pub model_name: String,
    pub temperature: f64,
    pub max_tokens: u32,
    pub timeout_seconds: u64,
    pub expertise_areas: Vec<String>,
    pub bias_tendencies: HashMap<String, f64>,
}

/// Judge contribution in a council session
#[derive(Debug, Clone)]
pub struct JudgeContribution {
    pub judge_id: String,
    pub judge_type: JudgeType,
    pub verdict: JudgeVerdict,
    pub processing_time_ms: u64,
    pub model_version: String,
    pub token_usage: Option<TokenUsage>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Token usage statistics
#[derive(Debug, Clone)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Review context provided to judges
#[derive(Debug, Clone)]
pub struct ReviewContext {
    pub working_spec: WorkingSpec,
    pub planning_metadata: Option<PlanningMetadata>,
    pub previous_reviews: Vec<PreviousReview>,
    pub risk_tier: RiskTier,
    pub session_id: String,
    pub judge_instructions: HashMap<String, String>,
}

/// Planning metadata from the planning agent
#[derive(Debug, Clone)]
pub struct PlanningMetadata {
    pub planning_duration: std::time::Duration,
    pub refinement_iterations: u32,
    pub caws_compliance_score: f64,
    pub validation_issues: Vec<String>,
}

/// Previous review in the session
#[derive(Debug, Clone)]
pub struct PreviousReview {
    pub judge_id: String,
    pub judge_type: JudgeType,
    pub verdict_summary: VerdictSummary,
    pub key_insights: Vec<String>,
}

/// Verdict summary for individual judge verdicts
#[derive(Debug, Clone)]
pub struct JudgeVerdictSummary {
    pub judge_id: Uuid,
    pub judge_type: String,
    pub verdict: JudgeVerdict,
    pub processing_time_ms: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Verdict summary for previous reviews
#[derive(Debug, Clone)]
pub enum VerdictSummary {
    Approved { confidence: f64 },
    RequestedRefinement { change_count: usize },
    Rejected { critical_issue_count: usize },
}

/// Change impact on the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChangeImpact {
    Minor,
    Moderate,
    Major,
    Breaking,
}

/// Change priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChangePriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Effort estimate for changes
#[derive(Debug, Clone, PartialEq)]
pub struct EffortEstimate {
    pub person_hours: f64,
    pub complexity: ComplexityLevel,
    pub dependencies: Vec<String>,
}

/// Complexity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComplexityLevel {
    Simple,
    Moderate,
    Complex,
    VeryComplex,
}

/// Change category for required changes
#[derive(Debug, Clone, PartialEq)]
pub enum ChangeCategory {
    Quality,
    Security,
    Performance,
    Maintainability,
    Scalability,
    Requirements,
    Architecture,
    Testing,
    Documentation,
}

/// Critical issue that prevents approval
#[derive(Debug, Clone, PartialEq)]
pub struct CriticalIssue {
    pub severity: IssueSeverity,
    pub category: String,
    pub description: String,
    pub evidence: Vec<String>,
}

/// Issue severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IssueSeverity {
    High,
    Critical,
}

/// Advanced ethical assessment result
#[derive(Debug, Clone)]
pub struct EthicalAssessment {
    /// Overall ethical acceptability (0.0 = highly unethical, 1.0 = highly ethical)
    pub ethical_score: f32,
    /// Specific ethical concerns identified
    pub ethical_concerns: Vec<EthicalConcern>,
    /// Stakeholder impact analysis
    pub stakeholder_impacts: Vec<StakeholderImpact>,
    /// Ethical trade-offs identified
    pub ethical_tradeoffs: Vec<EthicalTradeoff>,
    /// Long-term consequence assessment
    pub long_term_consequences: Vec<ConsequenceAssessment>,
    /// Cultural/contextual ethical considerations
    pub cultural_considerations: Vec<CulturalConsideration>,
    /// Recommended ethical mitigations
    pub ethical_mitigations: Vec<String>,
    /// Ethical uncertainty factors
    pub uncertainty_factors: Vec<String>,
    /// Assessment confidence
    pub assessment_confidence: f32,
}

/// Specific ethical concern identified
#[derive(Debug, Clone)]
pub struct EthicalConcern {
    /// Category of ethical concern
    pub category: EthicalCategory,
    /// Severity level
    pub severity: EthicalSeverity,
    /// Detailed description
    pub description: String,
    /// Evidence supporting the concern
    pub evidence: Vec<String>,
    /// Affected stakeholders
    pub affected_stakeholders: Vec<String>,
}

/// Ethical concern categories
#[derive(Debug, Clone, PartialEq)]
pub enum EthicalCategory {
    /// Harm to individuals or groups
    Harm,
    /// Privacy violations
    Privacy,
    /// Discrimination or bias
    Discrimination,
    /// Autonomy and consent issues
    Autonomy,
    /// Fairness and justice concerns
    Fairness,
    /// Transparency issues
    Transparency,
    /// Accountability problems
    Accountability,
    /// Societal impact concerns
    SocietalImpact,
    /// Environmental concerns
    Environmental,
    /// Long-term future implications
    FutureGenerations,
}

/// Ethical severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum EthicalSeverity {
    /// Minor ethical concern
    Minor,
    /// Moderate ethical issue
    Moderate,
    /// Serious ethical problem
    Serious,
    /// Critical ethical violation
    Critical,
}

/// Stakeholder impact analysis
#[derive(Debug, Clone)]
pub struct StakeholderImpact {
    /// Stakeholder group
    pub stakeholder_group: String,
    /// Type of impact (positive/negative/neutral)
    pub impact_type: ImpactType,
    /// Impact magnitude (-1.0 to 1.0, negative = harm)
    pub impact_magnitude: f32,
    /// Duration of impact
    pub duration: ImpactDuration,
    /// Description of the impact
    pub description: String,
    /// Mitigation strategies for negative impacts
    pub mitigation_strategies: Vec<String>,
}

/// Type of stakeholder impact
#[derive(Debug, Clone)]
pub enum ImpactType {
    Positive,
    Negative,
    Neutral,
    Mixed,
}

/// Duration of impact
#[derive(Debug, Clone)]
pub enum ImpactDuration {
    ShortTerm,    // Days to weeks
    MediumTerm,   // Weeks to months
    LongTerm,     // Months to years
    Permanent,    // Lasting impact
}

/// Ethical trade-off analysis
#[derive(Debug, Clone)]
pub struct EthicalTradeoff {
    /// Conflicting ethical principles
    pub conflicting_principles: Vec<String>,
    /// Description of the trade-off
    pub description: String,
    /// Recommended resolution approach
    pub recommended_resolution: String,
    /// Alternative approaches considered
    pub alternative_approaches: Vec<String>,
}

/// Long-term consequence assessment
#[derive(Debug, Clone)]
pub struct ConsequenceAssessment {
    /// Time horizon for consequences
    pub time_horizon: TimeHorizon,
    /// Likelihood of occurrence (0.0-1.0)
    pub likelihood: f32,
    /// Potential consequence description
    pub consequence: String,
    /// Severity of consequence
    pub severity: ConsequenceSeverity,
    /// Mitigation strategies
    pub mitigation_strategies: Vec<String>,
}

/// Time horizon for consequences
#[derive(Debug, Clone)]
pub enum TimeHorizon {
    Immediate,   // Within hours/days
    ShortTerm,   // Days to weeks
    MediumTerm,  // Weeks to months
    LongTerm,    // Months to years
    Generational, // Multiple generations
}

/// Consequence severity levels
#[derive(Debug, Clone)]
pub enum ConsequenceSeverity {
    Negligible,
    Minor,
    Moderate,
    Major,
    Catastrophic,
}

/// Cultural and contextual considerations
#[derive(Debug, Clone)]
pub struct CulturalConsideration {
    /// Cultural or contextual factor
    pub factor: String,
    /// Relevant ethical frameworks
    pub ethical_frameworks: Vec<String>,
    /// Cultural sensitivity implications
    pub cultural_sensitivity: CulturalSensitivity,
    /// Alternative ethical perspectives
    pub alternative_perspectives: Vec<String>,
}

/// Cultural sensitivity levels
#[derive(Debug, Clone)]
pub enum CulturalSensitivity {
    Low,      // Minimal cultural implications
    Moderate, // Some cultural considerations needed
    High,     // Significant cultural sensitivity required
    Critical, // Culturally sensitive, requires expert consultation
}
