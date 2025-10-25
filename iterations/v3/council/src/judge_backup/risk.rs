//! Risk assessment structures and logic
//!
//! Comprehensive risk evaluation framework including technical,
//! ethical, operational, and business risk assessments with
//! multi-dimensional analysis and mitigation strategies.

/// Risk assessment from a judge
#[derive(Debug, Clone, PartialEq)]
pub struct RiskAssessment {
    pub overall_risk: RiskLevel,
    pub risk_factors: Vec<String>,
    pub mitigation_suggestions: Vec<String>,
    pub confidence: f64,
}

/// Risk level classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Multi-dimensional risk assessment
/// Enhanced with comprehensive risk scoring from integration testing insights
#[derive(Debug, Clone)]
pub struct MultiDimensionalRiskAssessment {
    /// Overall risk score (0.0-1.0, higher = more risky)
    pub overall_risk_score: f32,

    /// Technical risk assessment
    pub technical_risk: TechnicalRiskAssessment,

    /// Ethical risk assessment
    pub ethical_risk: EthicalRiskAssessment,

    /// Operational risk assessment
    pub operational_risk: OperationalRiskAssessment,

    /// Business risk assessment
    pub business_risk: BusinessRiskAssessment,

    /// Risk interactions and compounding effects
    pub risk_interactions: Vec<RiskInteraction>,

    /// Mitigation strategies prioritized by impact
    pub mitigation_priorities: Vec<MitigationPriority>,

    /// Risk trends and projections
    pub risk_projections: RiskProjections,

    /// Confidence in risk assessment
    pub assessment_confidence: f32,
}

/// Technical risk assessment
#[derive(Debug, Clone)]
pub struct TechnicalRiskAssessment {
    /// Technical feasibility score (0.0-1.0, lower = higher risk)
    pub feasibility_score: f32,

    /// Complexity assessment
    pub complexity_assessment: ComplexityAssessment,

    /// Resource risk factors
    pub resource_risks: Vec<ResourceRisk>,

    /// Technology maturity assessment
    pub technology_maturity: TechnologyMaturity,

    /// Integration complexity
    pub integration_complexity: IntegrationComplexity,

    /// Performance risk assessment
    pub performance_risks: Vec<PerformanceRisk>,
}

/// Ethical risk assessment
#[derive(Debug, Clone)]
pub struct EthicalRiskAssessment {
    /// Ethical acceptability score (0.0-1.0, lower = higher ethical risk)
    pub ethical_score: f32,

    /// Ethical concern categories and their severity
    pub concern_categories: Vec<EthicalConcernCategory>,

    /// Stakeholder impact assessment
    pub stakeholder_impacts: Vec<StakeholderImpact>,

    /// Regulatory compliance risks
    pub regulatory_risks: Vec<RegulatoryRisk>,

    /// Long-term societal impact assessment
    pub societal_impacts: Vec<SocietalImpact>,

    /// Ethical uncertainty factors
    pub uncertainty_factors: Vec<String>,
}

/// Operational risk assessment
#[derive(Debug, Clone)]
pub struct OperationalRiskAssessment {
    /// Operational feasibility score (0.0-1.0, lower = higher operational risk)
    pub feasibility_score: f32,

    /// Deployment complexity
    pub deployment_complexity: DeploymentComplexity,

    /// Maintenance requirements
    pub maintenance_requirements: MaintenanceRequirements,

    /// Scalability concerns
    pub scalability_concerns: Vec<ScalabilityConcern>,

    /// Monitoring and observability requirements
    pub monitoring_requirements: MonitoringRequirements,

    /// Incident response planning
    pub incident_response: IncidentResponseAssessment,
}

/// Business risk assessment
#[derive(Debug, Clone)]
pub struct BusinessRiskAssessment {
    /// Business viability score (0.0-1.0, lower = higher business risk)
    pub viability_score: f32,

    /// Market impact assessment
    pub market_impact: MarketImpact,

    /// Financial risk factors
    pub financial_risks: Vec<FinancialRisk>,

    /// Stakeholder management complexity
    pub stakeholder_complexity: StakeholderComplexity,

    /// Competitive positioning
    pub competitive_positioning: CompetitivePositioning,

    /// Exit strategy feasibility
    pub exit_strategy: ExitStrategy,
}

/// Risk interaction between different dimensions
#[derive(Debug, Clone)]
pub struct RiskInteraction {
    /// Primary risk dimension
    pub primary_dimension: RiskDimension,

    /// Secondary risk dimension
    pub secondary_dimension: RiskDimension,

    /// Interaction type
    pub interaction_type: InteractionType,

    /// Interaction strength (0.0-1.0)
    pub interaction_strength: f32,

    /// Description of how risks interact
    pub description: String,

    /// Compounded risk level
    pub compounded_risk: RiskLevel,
}

/// Risk dimension types
#[derive(Debug, Clone, PartialEq)]
pub enum RiskDimension {
    Technical,
    Ethical,
    Operational,
    Business,
}

/// Type of risk interaction
#[derive(Debug, Clone)]
pub enum InteractionType {
    /// Risks reinforce each other
    Amplifying,
    /// Risks cancel each other out
    Mitigating,
    /// Risks create new compound risks
    Compounding,
    /// Risks are independent
    Independent,
}

/// Mitigation strategy with priority
#[derive(Debug, Clone)]
pub struct MitigationPriority {
    /// Mitigation strategy description
    pub strategy: String,

    /// Risk dimension this addresses
    pub target_dimension: RiskDimension,

    /// Priority level
    pub priority: MitigationPriorityLevel,

    /// Expected risk reduction (0.0-1.0)
    pub expected_reduction: f32,

    /// Implementation complexity
    pub implementation_complexity: ComplexityLevel,

    /// Timeline estimate in weeks
    pub timeline_weeks: u8,
}

/// Mitigation priority levels
#[derive(Debug, Clone)]
pub enum MitigationPriorityLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Complexity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComplexityLevel {
    Simple,
    Moderate,
    Complex,
    VeryComplex,
}

/// Risk trend projections
#[derive(Debug, Clone)]
pub struct RiskProjections {
    /// Short-term risk trend (next 3 months)
    pub short_term_trend: RiskTrend,

    /// Medium-term risk trend (3-12 months)
    pub medium_term_trend: RiskTrend,

    /// Long-term risk trend (1+ years)
    pub long_term_trend: RiskTrend,

    /// Key inflection points
    pub inflection_points: Vec<RiskInflectionPoint>,

    /// Risk stabilization timeline
    pub stabilization_timeline_months: Option<u8>,
}

/// Risk trend direction
#[derive(Debug, Clone)]
pub enum RiskTrend {
    /// Risk decreasing over time
    Decreasing,
    /// Risk increasing over time
    Increasing,
    /// Risk stable over time
    Stable,
    /// Risk fluctuating unpredictably
    Fluctuating,
}

/// Risk inflection point
#[derive(Debug, Clone)]
pub struct RiskInflectionPoint {
    /// Timeline in months
    pub timeline_months: u8,

    /// Type of inflection
    pub inflection_type: InflectionType,

    /// Description of the change
    pub description: String,

    /// Impact magnitude (-1.0 to 1.0, negative = risk reduction)
    pub impact_magnitude: f32,
}

/// Type of risk inflection
#[derive(Debug, Clone)]
pub enum InflectionType {
    /// Risk increases significantly
    RiskSpike,
    /// Risk decreases significantly
    RiskReduction,
    /// Risk stabilizes
    Stabilization,
    /// External event changes risk profile
    ExternalEvent,
}

/// Complexity assessment for technical risk
#[derive(Debug, Clone)]
pub struct ComplexityAssessment {
    pub algorithmic_complexity: crate::risk_scorer::ComputationalComplexity,
    pub integration_points: u32,
    pub external_dependencies: u32,
    pub novelty_factor: f32, // 0.0-1.0, higher = more novel/untested
    pub team_experience_level: f32, // 0.0-1.0, higher = more experienced
}

/// Resource risk factors
#[derive(Debug, Clone)]
pub struct ResourceRisk {
    pub resource_type: String,
    pub availability_risk: f32, // 0.0-1.0
    pub cost_volatility: f32, // 0.0-1.0
    pub alternative_sources: u32,
    pub description: String,
}

/// Technology maturity assessment
#[derive(Debug, Clone)]
pub struct TechnologyMaturity {
    pub maturity_level: TechnologyMaturityLevel,
    pub adoption_rate: f32, // 0.0-1.0
    pub stability_score: f32, // 0.0-1.0
    pub vendor_support: f32, // 0.0-1.0
    pub community_size: String,
}

/// Technology maturity levels
#[derive(Debug, Clone, PartialEq)]
pub enum TechnologyMaturityLevel {
    Experimental,
    EarlyAdopter,
    Mature,
    Legacy,
    Deprecated,
}

/// Integration complexity assessment
#[derive(Debug, Clone)]
pub struct IntegrationComplexity {
    pub api_integrations: u32,
    pub data_format_complexity: f32, // 0.0-1.0
    pub protocol_diversity: u32,
    pub legacy_system_interfaces: u32,
    pub real_time_requirements: bool,
}

/// Performance risk factors
#[derive(Debug, Clone)]
pub struct PerformanceRisk {
    pub risk_type: PerformanceRiskType,
    pub severity: f32, // 0.0-1.0
    pub likelihood: f32, // 0.0-1.0
    pub mitigation_complexity: ComplexityLevel,
}

/// Types of performance risks
#[derive(Debug, Clone, PartialEq)]
pub enum PerformanceRiskType {
    LatencyViolation,
    ThroughputLimitation,
    MemoryLeak,
    ResourceExhaustion,
    ScalabilityBottleneck,
    ConcurrencyIssue,
}

/// Ethical concern category with severity
#[derive(Debug, Clone)]
pub struct EthicalConcernCategory {
    pub category: EthicalCategory,
    pub severity_score: f32, // 0.0-1.0
    pub affected_population_size: PopulationSize,
    pub regulatory_implications: bool,
}

/// Population size affected by ethical concern
#[derive(Debug, Clone, PartialEq)]
pub enum PopulationSize {
    Individual,
    SmallGroup,
    LargeGroup,
    SocietyWide,
    Global,
}

/// Regulatory compliance risks
#[derive(Debug, Clone)]
pub struct RegulatoryRisk {
    pub jurisdiction: String,
    pub regulation_type: RegulationType,
    pub compliance_complexity: f32, // 0.0-1.0
    pub penalty_severity: f32, // 0.0-1.0
    pub audit_frequency: AuditFrequency,
}

/// Types of regulations
#[derive(Debug, Clone, PartialEq)]
pub enum RegulationType {
    DataPrivacy,
    ConsumerProtection,
    LaborLaw,
    Environmental,
    Financial,
    Healthcare,
    Security,
    IntellectualProperty,
}

/// Audit frequency expectations
#[derive(Debug, Clone, PartialEq)]
pub enum AuditFrequency {
    Rare,
    Annual,
    Quarterly,
    Monthly,
    Continuous,
}

/// Societal impact assessment
#[derive(Debug, Clone)]
pub struct SocietalImpact {
    pub impact_type: SocietalImpactType,
    pub time_horizon: TimeHorizon,
    pub magnitude: f32, // -1.0 to 1.0, negative = positive impact
    pub reversibility: Reversibility,
    pub affected_domains: Vec<String>,
}

/// Types of societal impact
#[derive(Debug, Clone, PartialEq)]
pub enum SocietalImpactType {
    Economic,
    Social,
    Environmental,
    Technological,
    Cultural,
    Political,
}

/// Time horizon for impact assessment
#[derive(Debug, Clone, PartialEq)]
pub enum TimeHorizon {
    Immediate,
    ShortTerm,
    MediumTerm,
    LongTerm,
}

/// Reversibility of societal impact
#[derive(Debug, Clone, PartialEq)]
pub enum Reversibility {
    Irreversible,
    LongTerm,
    MediumTerm,
    ShortTerm,
}

/// Ethical category types
#[derive(Debug, Clone, PartialEq)]
pub enum EthicalCategory {
    Harm,
    Privacy,
    Discrimination,
    Transparency,
    Autonomy,
    Justice,
    Sustainability,
    Cultural,
}

/// Stakeholder impact analysis
#[derive(Debug, Clone)]
pub struct StakeholderImpact {
    pub stakeholder_type: StakeholderType,
    pub impact_magnitude: f32, // -1.0 to 1.0, negative = positive impact
    pub impact_description: String,
    pub mitigation_options: Vec<String>,
}

/// Types of stakeholders
#[derive(Debug, Clone, PartialEq)]
pub enum StakeholderType {
    Users,
    Employees,
    Communities,
    Society,
    Environment,
    Regulators,
    Competitors,
    Partners,
}

/// Deployment complexity assessment
#[derive(Debug, Clone)]
pub struct DeploymentComplexity {
    pub environment_count: u32,
    pub manual_steps: u32,
    pub automation_level: f32, // 0.0-1.0
    pub rollback_complexity: ComplexityLevel,
    pub monitoring_setup_complexity: ComplexityLevel,
}

/// Maintenance requirements assessment
#[derive(Debug, Clone)]
pub struct MaintenanceRequirements {
    pub update_frequency: MaintenanceFrequency,
    pub monitoring_intensity: MonitoringIntensity,
    pub support_team_size: u8,
    pub training_requirements: TrainingLevel,
    pub documentation_complexity: DocumentationComplexity,
}

/// Maintenance frequency
#[derive(Debug, Clone, PartialEq)]
pub enum MaintenanceFrequency {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Biannual,
    Annual,
}

/// Monitoring intensity levels
#[derive(Debug, Clone, PartialEq)]
pub enum MonitoringIntensity {
    Low,
    Medium,
    High,
    Critical,
}

/// Training level requirements
#[derive(Debug, Clone, PartialEq)]
pub enum TrainingLevel {
    Basic,
    Intermediate,
    Advanced,
    Specialized,
}

/// Documentation complexity
#[derive(Debug, Clone, PartialEq)]
pub enum DocumentationComplexity {
    Minimal,
    Standard,
    Comprehensive,
    Extensive,
}

/// Scalability concern
#[derive(Debug, Clone)]
pub struct ScalabilityConcern {
    pub concern_type: ScalabilityConcernType,
    pub current_limitations: Vec<String>,
    pub projected_growth: f32, // Growth factor over time
    pub mitigation_complexity: ComplexityLevel,
}

/// Types of scalability concerns
#[derive(Debug, Clone, PartialEq)]
pub enum ScalabilityConcernType {
    UserLoad,
    DataVolume,
    ProcessingCapacity,
    NetworkBandwidth,
    StorageCapacity,
    ConcurrentUsers,
}

/// Monitoring and observability requirements
#[derive(Debug, Clone)]
pub struct MonitoringRequirements {
    pub metrics_count: u32,
    pub alert_count: u32,
    pub dashboard_complexity: DashboardComplexity,
    pub log_volume: LogVolume,
    pub incident_response_time: IncidentResponseTime,
}

/// Dashboard complexity levels
#[derive(Debug, Clone, PartialEq)]
pub enum DashboardComplexity {
    Basic,
    Intermediate,
    Advanced,
    Enterprise,
}

/// Log volume expectations
#[derive(Debug, Clone, PartialEq)]
pub enum LogVolume {
    Low,
    Medium,
    High,
    VeryHigh,
}

/// Incident response time requirements
#[derive(Debug, Clone, PartialEq)]
pub enum IncidentResponseTime {
    Immediate,
    Hours,
    Days,
    Weeks,
}

/// Incident response planning assessment
#[derive(Debug, Clone)]
pub struct IncidentResponseAssessment {
    pub response_team_size: u8,
    pub backup_systems: bool,
    pub recovery_time_objective: RecoveryTime,
    pub communication_plan: CommunicationPlanQuality,
}

/// Recovery time objectives
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryTime {
    Minutes,
    Hours,
    Days,
    Weeks,
}

/// Communication plan quality
#[derive(Debug, Clone, PartialEq)]
pub enum CommunicationPlanQuality {
    Basic,
    Standard,
    Comprehensive,
    Enterprise,
}

/// Market impact assessment
#[derive(Debug, Clone)]
pub struct MarketImpact {
    pub market_size: f64, // Market size in dollars
    pub market_share: f32, // 0.0-1.0
    pub competitive_advantage: f32, // 0.0-1.0
    pub disruption_potential: f32, // 0.0-1.0
    pub adoption_curve: AdoptionCurve,
}

/// Adoption curve types
#[derive(Debug, Clone, PartialEq)]
pub enum AdoptionCurve {
    Slow,
    Linear,
    Exponential,
    SShaped,
}

/// Financial risk factors
#[derive(Debug, Clone)]
pub struct FinancialRisk {
    pub risk_type: FinancialRiskType,
    pub potential_impact: f64, // Impact in dollars
    pub probability: f32, // 0.0-1.0
    pub time_horizon: TimeHorizon,
}

/// Types of financial risks
#[derive(Debug, Clone, PartialEq)]
pub enum FinancialRiskType {
    RevenueLoss,
    CostOverrun,
    InvestmentLoss,
    RegulatoryFine,
    Lawsuit,
    MarketFluctuation,
}

/// Stakeholder management complexity
#[derive(Debug, Clone)]
pub struct StakeholderComplexity {
    pub stakeholder_count: u32,
    pub relationship_complexity: RelationshipComplexity,
    pub communication_channels: u32,
    pub conflict_resolution_needed: bool,
}

/// Relationship complexity levels
#[derive(Debug, Clone, PartialEq)]
pub enum RelationshipComplexity {
    Simple,
    Moderate,
    Complex,
    HighlyComplex,
}

/// Competitive positioning
#[derive(Debug, Clone)]
pub struct CompetitivePositioning {
    pub market_position: MarketPosition,
    pub differentiation_factors: Vec<String>,
    pub barrier_to_entry: BarrierStrength,
    pub sustainability_score: f32, // 0.0-1.0
}

/// Market position types
#[derive(Debug, Clone, PartialEq)]
pub enum MarketPosition {
    MarketLeader,
    StrongCompetitor,
    NichePlayer,
    EmergingPlayer,
    Challenger,
}

/// Barrier to entry strength
#[derive(Debug, Clone, PartialEq)]
pub enum BarrierStrength {
    Weak,
    Moderate,
    Strong,
    VeryStrong,
}

/// Exit strategy feasibility
#[derive(Debug, Clone)]
pub struct ExitStrategy {
    pub strategy_type: ExitStrategyType,
    pub feasibility_score: f32, // 0.0-1.0
    pub timeline_months: u8,
    pub stakeholder_impact: StakeholderImpact,
}

/// Types of exit strategies
#[derive(Debug, Clone, PartialEq)]
pub enum ExitStrategyType {
    Acquisition,
    IPO,
    Liquidation,
    ManagementBuyout,
    StrategicPartnership,
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

/// Ethical severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum EthicalSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Ethical trade-off analysis
#[derive(Debug, Clone)]
pub struct EthicalTradeoff {
    pub benefit_description: String,
    pub cost_description: String,
    pub affected_values: Vec<String>,
    pub mitigation_options: Vec<String>,
}

/// Long-term consequence assessment
#[derive(Debug, Clone)]
pub struct ConsequenceAssessment {
    pub consequence_type: ConsequenceType,
    pub time_horizon: TimeHorizon,
    pub probability: f32, // 0.0-1.0
    pub impact_magnitude: f32, // -1.0 to 1.0
    pub reversibility: Reversibility,
}

/// Types of consequences
#[derive(Debug, Clone, PartialEq)]
pub enum ConsequenceType {
    Positive,
    Negative,
    Neutral,
    Uncertain,
}

/// Cultural consideration in ethical assessment
#[derive(Debug, Clone)]
pub struct CulturalConsideration {
    pub cultural_context: String,
    pub ethical_norms: Vec<String>,
    pub potential_conflicts: Vec<String>,
    pub adaptation_recommendations: Vec<String>,
}


