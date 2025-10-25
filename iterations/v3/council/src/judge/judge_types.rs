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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub overall_risk: RiskLevel,
    pub risk_factors: Vec<RiskFactor>,
    pub mitigation_strategies: Vec<String>,
    pub confidence_score: f64,
}

/// Risk levels for assessments
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RiskFactorType {
    Security,
    Performance,
    Reliability,
    Compliance,
    Complexity,
    ResourceUsage,
}

/// Severity levels for risk factors
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    pub category: ChangeCategory,
    pub impact: ChangeImpact,
}

/// Types of changes that can be required
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
pub trait Judge: Send + Sync + std::fmt::Debug {
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
    pub max_complexity: ComplexityLevel,
    pub supported_languages: Vec<String>,
    pub specialization_score: f64,
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
}

impl std::fmt::Display for JudgeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            JudgeType::Constitutional => "constitutional",
            JudgeType::QualityAssurance => "quality_assurance",
            JudgeType::Security => "security",
            JudgeType::Performance => "performance",
            JudgeType::Architecture => "architecture",
            JudgeType::Testing => "testing",
            JudgeType::Compliance => "compliance",
            JudgeType::DomainExpert => "domain_expert",
            JudgeType::Ethics => "ethics",
            JudgeType::Technical => "technical",
            JudgeType::Quality => "quality",
        };
        write!(f, "{}", name)
    }
}

/// Judge configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChangeImpact {
    Minor,
    Moderate,
    Major,
    Breaking,
}

/// Change priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChangePriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Effort complexity levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EffortComplexity {
    Trivial,
    Simple,
    Moderate,
    Complex,
    VeryComplex,
}

/// Effort estimate for changes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffortEstimate {
    pub person_hours: f64,
    pub developer_hours: f64,
    pub complexity: ComplexityLevel,
    pub effort_complexity: EffortComplexity,
    pub skills_required: Vec<String>,
    pub dependencies: Vec<String>,
}

/// Complexity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComplexityLevel {
    Simple,
    Moderate,
    Complex,
    VeryComplex,
}

/// Computational complexity levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComputationalComplexity {
    Constant,
    Logarithmic,
    Linear,
    LogLinear,
    Polynomial,
    Exponential,
    Factorial,
}

/// Change category for required changes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

/// Technical risk assessment
#[derive(Debug, Clone)]
pub struct TechnicalRiskAssessment {
    pub feasibility_score: f64,
    pub complexity_assessment: ComplexityLevel,
    pub resource_risks: Vec<String>,
    pub technology_maturity: f64,
    pub integration_complexity: f64,
    pub performance_risks: Vec<PerformanceRisk>,
}

/// Ethical risk assessment
#[derive(Debug, Clone)]
pub struct EthicalRiskAssessment {
    pub ethical_score: f64,
    pub concern_categories: Vec<EthicalConcernCategory>,
    pub stakeholder_impacts: Vec<StakeholderImpact>,
    pub regulatory_risks: Vec<RegulatoryRisk>,
    pub societal_impacts: Vec<SocietalImpact>,
    pub uncertainty_factors: Vec<String>,
    pub privacy_risks: Vec<String>,
    pub bias_risks: Vec<String>,
    pub fairness_concerns: Vec<String>,
    pub transparency_issues: Vec<String>,
    pub accountability_gaps: Vec<String>,
}

/// Operational risk assessment
#[derive(Debug, Clone)]
pub struct OperationalRiskAssessment {
    pub feasibility_score: f64,
    pub deployment_complexity: DeploymentComplexity,
    pub maintenance_requirements: MaintenanceRequirements,
    pub monitoring_requirements: MonitoringRequirements,
    pub scalability_concerns: Vec<ScalabilityConcern>,
    pub incident_response: IncidentResponseAssessment,
}

/// Business risk assessment
#[derive(Debug, Clone)]
pub struct BusinessRiskAssessment {
    pub viability_score: f64,
    pub market_impact: MarketImpact,
    pub financial_risks: Vec<FinancialRisk>,
    pub stakeholder_complexity: StakeholderComplexity,
    pub competitive_positioning: CompetitivePositioning,
    pub exit_strategy: ExitStrategy,
    pub market_risks: Vec<String>,
    pub regulatory_risks: Vec<String>,
    pub financial_impacts: Vec<String>,
    pub stakeholder_impacts: Vec<String>,
    pub competitive_threats: Vec<String>,
}

/// Multi-dimensional risk assessment combining all dimensions
#[derive(Debug, Clone)]
pub struct MultiDimensionalRiskAssessment {
    pub overall_risk_score: f64,
    pub technical_risk: TechnicalRiskAssessment,
    pub ethical_risk: EthicalRiskAssessment,
    pub operational_risk: OperationalRiskAssessment,
    pub business_risk: BusinessRiskAssessment,
    pub risk_interactions: Vec<RiskInteraction>,
    pub mitigation_priorities: Vec<MitigationPriority>,
    pub risk_projections: RiskProjections,
    pub assessment_confidence: f64,
}

/// Complexity assessment for technical risk
#[derive(Debug, Clone)]
pub struct ComplexityAssessment {
    pub algorithmic_complexity: ComputationalComplexity,
    pub integration_points: u32,
    pub external_dependencies: u32,
    pub novelty_factor: f64,
    pub team_experience_level: f64,
}

/// Resource risk assessment
#[derive(Debug, Clone)]
pub struct ResourceRisk {
    pub availability_risk: f64,
    pub cost_volatility: f64,
    pub alternative_sources: Vec<String>,
    pub description: String,
}

/// Technology maturity level enum
#[derive(Debug, Clone, PartialEq)]
pub enum TechnologyMaturityLevel {
    Experimental,
    EarlyAdopter,
    Mature,
    Legacy,
}

/// Technology maturity assessment
#[derive(Debug, Clone)]
pub struct TechnologyMaturity {
    pub maturity_level: TechnologyMaturityLevel,
    pub stability_score: f64,
    pub vendor_support: f64,
    pub community_size: f64,
    pub vendor_stability: f64,
    pub community_support: f64,
}

/// Integration complexity assessment
#[derive(Debug, Clone)]
pub struct IntegrationComplexity {
    pub api_integrations: u32,
    pub protocol_diversity: f64,
    pub legacy_system_interfaces: u32,
    pub real_time_requirements: bool,
}

/// Performance risk type enum
#[derive(Debug, Clone, PartialEq)]
pub enum PerformanceRiskType {
    ResponseTime,
    Throughput,
    Scalability,
    ResourceUtilization,
    MemoryLeak,
    CpuOverload,
    LatencyViolation,
    ScalabilityBottleneck,
}

/// Performance risk assessment
#[derive(Debug, Clone)]
pub struct PerformanceRisk {
    pub risk_type: PerformanceRiskType,
    pub severity: f64,
    pub likelihood: f64,
    pub mitigation_complexity: f64,
}

/// Ethical concern category
#[derive(Debug, Clone)]
pub struct EthicalConcernCategory {
    pub category: EthicalCategory,
    pub severity_score: f32,
    pub affected_population_size: PopulationSize,
    pub regulatory_implications: bool,
}

/// Population size for ethical concerns
#[derive(Debug, Clone)]
pub enum PopulationSize {
    Individual,
    SmallGroup,
    LargeGroup,
    Society,
    SocietyWide,
    Global,
}

/// Regulation type enum
#[derive(Debug, Clone, PartialEq)]
pub enum RegulationType {
    GDPR,
    HIPAA,
    SOX,
    PCI,
    DataPrivacy,
    IndustrySpecific,
    Custom,
}

/// Audit frequency enum
#[derive(Debug, Clone, PartialEq)]
pub enum AuditFrequency {
    Continuous,
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Annual,
}

/// Regulatory risk assessment
#[derive(Debug, Clone)]
pub struct RegulatoryRisk {
    pub jurisdiction: String,
    pub regulation_type: RegulationType,
    pub compliance_complexity: f64,
    pub penalty_severity: f64,
    pub audit_frequency: AuditFrequency,
    pub compliance_burden: f64,
    pub legal_risk: f64,
    pub audit_requirements: Vec<String>,
    pub certification_needs: Vec<String>,
}

/// Societal impact assessment
#[derive(Debug, Clone)]
pub struct SocietalImpact {
    pub impact_type: SocietalImpactType,
    pub time_horizon: TimeHorizon,
    pub magnitude: f64,
    pub reversibility: Reversibility,
    pub affected_domains: Vec<String>,
}

/// Societal impact types
#[derive(Debug, Clone)]
pub enum SocietalImpactType {
    Social,
    Economic,
    Environmental,
    Technological,
}

/// Reversibility of impacts
#[derive(Debug, Clone)]
pub enum Reversibility {
    Immediate,
    ShortTerm,
    MediumTerm,
    LongTerm,
    Permanent,
}

/// Deployment complexity assessment
#[derive(Debug, Clone)]
pub struct DeploymentComplexity {
    pub environment_count: u32,
    pub infrastructure_requirements: InfrastructureRequirement,
    pub automation_level: f64,
    pub rollback_complexity: f64,
    pub configuration_complexity: f64,
    pub zero_downtime_requirement: bool,
}

/// Infrastructure requirements
#[derive(Debug, Clone)]
pub enum InfrastructureRequirement {
    Minimal,
    Moderate,
    Standard,
    Extensive,
    Specialized,
}

/// Monitoring intensity levels
#[derive(Debug, Clone, PartialEq)]
pub enum MonitoringIntensity {
    Minimal,
    Basic,
    Moderate,
    Comprehensive,
    Intensive,
    Critical,
    Continuous,
}

/// Maintenance requirements
#[derive(Debug, Clone)]
pub struct MaintenanceRequirements {
    pub update_frequency: UpdateFrequency,
    pub monitoring_complexity: f64,
    pub monitoring_intensity: MonitoringIntensity,
    pub support_staffing: f64,
    pub emergency_response_time: std::time::Duration,
    pub cost_per_month: f64,
    pub backup_requirements: Vec<String>,
    pub disaster_recovery: bool,
}

/// Update frequency
#[derive(Debug, Clone)]
pub enum UpdateFrequency {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
}

/// Scalability concern
#[derive(Debug, Clone)]
pub struct ScalabilityConcern {
    pub concern_type: ScalabilityConcernType,
    pub current_limitations: String,
    pub growth_projection: GrowthProjection,
    pub mitigation_complexity: f64,
    pub mitigation_strategies: Vec<String>,
}

/// Scalability concern types
#[derive(Debug, Clone)]
pub enum ScalabilityConcernType {
    UserLoad,
    DataVolume,
    ProcessingPower,
    NetworkBandwidth,
}

/// Growth projection
#[derive(Debug, Clone)]
pub struct GrowthProjection {
    pub expected_growth_rate: f64,
    pub time_to_limit: std::time::Duration,
    pub growth_pattern: GrowthPattern,
}

/// Growth patterns
#[derive(Debug, Clone)]
pub enum GrowthPattern {
    Linear,
    Exponential,
    Seasonal,
}

/// Dashboard complexity levels
#[derive(Debug, Clone, PartialEq)]
pub enum DashboardComplexity {
    Simple,
    Moderate,
    Complex,
    Advanced,
}

/// Log volume levels
#[derive(Debug, Clone, PartialEq)]
pub enum LogVolume {
    Low,
    Moderate,
    High,
    Extreme,
}

/// Escalation complexity levels
#[derive(Debug, Clone, PartialEq)]
pub enum EscalationComplexity {
    Simple,
    Moderate,
    MultiLevel,
    Enterprise,
}

/// Monitoring requirements
#[derive(Debug, Clone)]
pub struct MonitoringRequirements {
    pub metrics_count: u32,
    pub alert_count: u32,
    pub dashboard_complexity: DashboardComplexity,
    pub log_volume: LogVolume,
    pub real_time_requirements: bool,
    pub metrics_collection: Vec<String>,
    pub alerting_thresholds: Vec<String>,
    pub log_aggregation: bool,
    pub performance_monitoring: bool,
}

/// Incident response assessment
#[derive(Debug, Clone)]
pub struct IncidentResponseAssessment {
    pub severity_classification: IncidentSeverityLevels,
    pub response_time_sla: std::time::Duration,
    pub response_team_requirements: Vec<String>,
    pub escalation_procedures: Vec<String>,
    pub recovery_time_objectives: RecoveryObjectives,
}

/// Incident severity levels
#[derive(Debug, Clone)]
pub struct IncidentSeverityLevels {
    pub critical_threshold: f64,
    pub high_threshold: f64,
    pub medium_threshold: f64,
    pub low_threshold: f64,
    pub critical_incidents: u32,
    pub high_incidents: u32,
    pub medium_incidents: u32,
    pub low_incidents: u32,
}

/// Recovery objectives
#[derive(Debug, Clone)]
pub struct RecoveryObjectives {
    pub rto_minutes: u32,
    pub rpo_minutes: u32,
    pub recovery_automation: f64,
    pub backup_frequency: String,
}

/// Industry transformation levels
#[derive(Debug, Clone, PartialEq)]
pub enum IndustryTransformation {
    Incremental,
    Moderate,
    Significant,
    Disruptive,
    Revolutionary,
}

/// Market impact assessment
#[derive(Debug, Clone)]
pub struct MarketImpact {
    pub market_size: f64,
    pub competitive_pressure: f64,
    pub market_share_impact: f64,
    pub entry_barrier_changes: Vec<String>,
    pub market_disruption: f64,
    pub competitive_advantage: f64,
    pub market_share_potential: f64,
    pub industry_transformation: IndustryTransformation,
}

/// Financial risk types
#[derive(Debug, Clone, PartialEq)]
pub enum FinancialRiskType {
    CostOverrun,
    RevenueLoss,
    CashFlow,
    Investment,
    MarketRisk,
    DevelopmentCostOverrun,
    MarketPenetrationFailure,
}

/// Financial risk assessment
#[derive(Debug, Clone)]
pub struct FinancialRisk {
    pub risk_type: FinancialRiskType,
    pub amount_at_risk: f64,
    pub probability: f64,
    pub time_horizon_months: u32,
    pub cost_overrun_probability: f64,
    pub revenue_impact: f64,
    pub cash_flow_risk: f64,
    pub investment_recovery: f64,
}

/// Engagement level enum
#[derive(Debug, Clone, PartialEq)]
pub enum EngagementLevel {
    Minimal,
    Basic,
    Moderate,
    Intensive,
    Critical,
    Comprehensive,
}

/// Market position enum
#[derive(Debug, Clone, PartialEq)]
pub enum MarketPosition {
    Dominant,
    Leader,
    MarketLeader,
    Challenger,
    Follower,
    Niche,
    NichePlayer,
    Emerging,
}

/// Stakeholder complexity assessment
#[derive(Debug, Clone)]
pub struct StakeholderComplexity {
    pub stakeholder_count: u32,
    pub communication_complexity: f64,
    pub alignment_difficulty: f64,
    pub influence_distribution: Vec<String>,
    pub stakeholder_diversity: f64,
    pub communication_channels: u32,
    pub conflict_potential: f64,
    pub engagement_required: EngagementLevel,
}

/// Barrier strength enum
#[derive(Debug, Clone, PartialEq)]
pub enum BarrierStrength {
    Low,
    Moderate,
    High,
    Strong,
    VeryHigh,
}

/// Moat strength enum
#[derive(Debug, Clone, PartialEq)]
pub enum MoatStrength {
    Weak,
    Moderate,
    Strong,
    VeryStrong,
}

/// Competitive positioning assessment
#[derive(Debug, Clone)]
pub struct CompetitivePositioning {
    pub market_position: String,
    pub differentiation_factors: Vec<String>,
    pub competitive_advantages: Vec<String>,
    pub vulnerability_assessment: Vec<String>,
    pub barrier_to_entry: BarrierStrength,
    pub sustainability_score: f64,
    pub moat_strength: MoatStrength,
}

/// Exit strategy type enum
#[derive(Debug, Clone, PartialEq)]
pub enum ExitStrategyType {
    IPO,
    Acquisition,
    ManagementBuyout,
    Liquidation,
    Merger,
}

/// Exit strategy assessment
#[derive(Debug, Clone)]
pub struct ExitStrategy {
    pub strategy_type: String,
    pub feasibility_score: f64,
    pub timeline_months: u32,
    pub expected_return: f64,
    pub complexity: f64,
    pub exit_options: Vec<String>,
    pub exit_complexity: f64,
    pub exit_costs: f64,
    pub stakeholder_impact: f64,
}

/// Risk interaction between different risk dimensions
#[derive(Debug, Clone)]
pub struct RiskInteraction {
    pub primary_risk: String,
    pub secondary_risk: String,
    pub interaction_type: RiskInteractionType,
    pub amplification_factor: f64,
    pub mitigation_synergies: Vec<String>,
}

/// Types of risk interactions
#[derive(Debug, Clone)]
pub enum RiskInteractionType {
    Amplifying,
    Compounding,
    Mitigating,
    Neutral,
}

/// Risk dimensions for mitigation targeting
#[derive(Debug, Clone)]
pub enum RiskDimension {
    Technical,
    Ethical,
    Operational,
    Business,
}

/// Mitigation priority levels
#[derive(Debug, Clone)]
pub enum MitigationPriorityLevel {
    Critical,
    High,
    Medium,
    Low,
}

/// Mitigation priority assessment
#[derive(Debug, Clone)]
pub struct MitigationPriority {
    pub strategy: String,
    pub target_dimension: RiskDimension,
    pub priority: MitigationPriorityLevel,
    pub expected_reduction: f64,
    pub implementation_complexity: ComplexityLevel,
    pub timeline_weeks: u32,
}

/// Risk trend over time
#[derive(Debug, Clone)]
pub enum RiskTrend {
    Increasing,
    Decreasing,
    Stable,
}

/// Risk projections over time
#[derive(Debug, Clone)]
pub struct RiskProjections {
    pub short_term_trend: RiskTrend,
    pub medium_term_trend: RiskTrend,
    pub long_term_trend: RiskTrend,
    pub inflection_points: Vec<RiskInflectionPoint>,
    pub stabilization_timeline_months: Option<u32>,
}

/// Types of risk inflection points
#[derive(Debug, Clone)]
pub enum InflectionType {
    RiskReduction,
    RiskSpike,
    ExternalChange,
    InternalChange,
}

/// Risk inflection point in projections
#[derive(Debug, Clone)]
pub struct RiskInflectionPoint {
    pub timeline_months: u32,
    pub inflection_type: InflectionType,
    pub description: String,
    pub impact_magnitude: f64,
}
