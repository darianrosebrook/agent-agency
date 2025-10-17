//! Types for reflexive learning system

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Learning task for the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningTask {
    pub id: Uuid,
    pub task_type: TaskType,
    pub complexity: TaskComplexity,
    pub expected_duration: chrono::Duration,
    pub success_criteria: Vec<SuccessCriterion>,
    pub context: TaskContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    CodeGeneration,
    CodeReview,
    Testing,
    Documentation,
    Refactoring,
    Debugging,
    Research,
    Integration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskComplexity {
    Simple,
    Moderate,
    Complex,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCriterion {
    pub criterion_type: CriterionType,
    pub description: String,
    pub measurable: bool,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CriterionType {
    Functional,
    Performance,
    Quality,
    Security,
    Compliance, // CAWS compliance
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub domain: String,
    pub technology_stack: Vec<String>,
    pub constraints: Vec<Constraint>,
    pub historical_performance: Option<HistoricalPerformance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub constraint_type: ConstraintType,
    pub description: String,
    pub severity: ConstraintSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    Time,
    Resource,
    Quality,
    Security,
    Compliance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintSeverity {
    Soft,
    Hard,
    Critical,
}

/// Learning session tracking progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningSession {
    pub id: Uuid,
    pub task_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub current_turn: u32,
    pub progress: ProgressMetrics,
    pub learning_state: LearningState,
    pub context_preservation: ContextPreservationState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressMetrics {
    pub completion_percentage: f64,
    pub quality_score: f64,
    pub efficiency_score: f64,
    pub error_rate: f64,
    pub learning_velocity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningState {
    pub current_strategy: LearningStrategy,
    pub adaptation_history: Vec<AdaptationEvent>,
    pub performance_trends: PerformanceTrends,
    pub resource_utilization: ResourceUtilization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningStrategy {
    Conservative,
    Balanced,
    Aggressive,
    Adaptive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationEvent {
    pub timestamp: DateTime<Utc>,
    pub adaptation_type: AdaptationType,
    pub trigger: AdaptationTrigger,
    pub impact: AdaptationImpact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdaptationType {
    StrategyChange,
    ResourceReallocation,
    ContextAdjustment,
    LearningRateAdjustment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdaptationTrigger {
    PerformanceDegradation,
    QualityIssue,
    ResourceConstraint,
    CouncilFeedback,
    ErrorPattern,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationImpact {
    pub performance_change: f64,
    pub quality_change: f64,
    pub efficiency_change: f64,
    pub confidence_change: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrends {
    pub short_term: TrendData,
    pub medium_term: TrendData,
    pub long_term: TrendData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendData {
    pub direction: TrendDirection,
    pub magnitude: f64,
    pub confidence: f64,
    pub data_points: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Declining,
    Stable,
    Volatile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub token_usage: f64,
    pub time_usage: f64,
    pub efficiency_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPreservationState {
    pub preserved_contexts: Vec<PreservedContext>,
    pub context_freshness: HashMap<String, DateTime<Utc>>,
    pub context_usage: HashMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreservedContext {
    pub context_id: Uuid,
    pub context_type: ContextType,
    pub content: String,
    pub relevance_score: f64,
    pub last_accessed: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextType {
    CodeContext,
    DocumentationContext,
    TestContext,
    ErrorContext,
    PerformanceContext,
}

/// Credit assignment for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditAssignment {
    pub session_id: Uuid,
    pub turn_credits: Vec<TurnCredit>,
    pub total_credit: f64,
    pub credit_distribution: CreditDistribution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnCredit {
    pub turn_number: u32,
    pub credit_amount: f64,
    pub credit_type: CreditType,
    pub contributing_factors: Vec<ContributingFactor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CreditType {
    Positive,
    Negative,
    Neutral,
    Corrective,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributingFactor {
    pub factor_type: FactorType,
    pub impact: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FactorType {
    Quality,
    Efficiency,
    Innovation,
    Compliance,
    ErrorReduction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditDistribution {
    pub strategy_credit: f64,
    pub resource_credit: f64,
    pub context_credit: f64,
    pub adaptation_credit: f64,
}

/// Learning signals from council
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilLearningSignal {
    pub signal_id: Uuid,
    pub council_judge: CouncilJudge,
    pub signal_type: LearningSignalType,
    pub content: String,
    pub confidence: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CouncilJudge {
    Constitutional,
    Technical,
    Quality,
    Integration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningSignalType {
    PerformanceFeedback,
    QualityAssessment,
    ComplianceViolation,
    ResourceRecommendation,
    StrategySuggestion,
}

/// Learning update from processing signals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningUpdate {
    pub update_id: Uuid,
    pub session_id: Uuid,
    pub update_type: LearningUpdateType,
    pub changes: Vec<LearningChange>,
    pub impact_assessment: ImpactAssessment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningUpdateType {
    StrategyAdjustment,
    ResourceReallocation,
    ContextUpdate,
    PerformanceOptimization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningChange {
    pub change_type: ChangeType,
    pub description: String,
    pub magnitude: f64,
    pub expected_impact: ExpectedImpact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    LearningRate,
    StrategyWeight,
    ResourceAllocation,
    ContextThreshold,
    QualityThreshold,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedImpact {
    pub performance_impact: f64,
    pub quality_impact: f64,
    pub efficiency_impact: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAssessment {
    pub overall_impact: f64,
    pub risk_level: RiskLevel,
    pub implementation_effort: ImplementationEffort,
    pub rollback_plan: Option<RollbackPlan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationEffort {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackPlan {
    pub rollback_steps: Vec<RollbackStep>,
    pub rollback_time_estimate: chrono::Duration,
    pub rollback_risk: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackStep {
    pub step_number: u32,
    pub description: String,
    pub estimated_time: chrono::Duration,
}

/// Historical performance data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalPerformance {
    pub task_type: TaskType,
    pub average_completion_time: chrono::Duration,
    pub average_quality_score: f64,
    pub success_rate: f64,
    pub common_failure_patterns: Vec<FailurePattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailurePattern {
    pub pattern_type: FailureType,
    pub frequency: f64,
    pub impact: f64,
    pub mitigation_strategy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureType {
    QualityFailure,
    PerformanceFailure,
    ComplianceFailure,
    ResourceFailure,
    ContextFailure,
}

/// Errors for the learning system
#[derive(Debug, thiserror::Error)]
pub enum LearningSystemError {
    #[error("Session management failed: {0}")]
    SessionManagementFailed(String),
    
    #[error("Progress tracking failed: {0}")]
    ProgressTrackingFailed(String),
    
    #[error("Credit assignment failed: {0}")]
    CreditAssignmentFailed(String),
    
    #[error("Resource allocation failed: {0}")]
    ResourceAllocationFailed(String),
    
    #[error("Context preservation failed: {0}")]
    ContextPreservationFailed(String),
    
    #[error("Council integration failed: {0}")]
    CouncilIntegrationFailed(String),
    
    #[error("Learning algorithm failed: {0}")]
    LearningAlgorithmFailed(String),
}

