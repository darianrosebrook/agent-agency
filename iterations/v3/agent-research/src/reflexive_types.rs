//! Types for reflexive learning system

use schemars::JsonSchema;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize, Deserializer, Serializer};
use std::collections::HashMap;
use uuid::Uuid;

/// Serialization helper for chrono::Duration as i64 (seconds)
mod duration_serde {
    use super::*;
    
    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let seconds = duration.num_seconds();
        serializer.serialize_i64(seconds)
    }
    
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let seconds = i64::deserialize(deserializer)?;
        Ok(Duration::seconds(seconds))
    }
}


/// Types of learning algorithms supported
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum LearningAlgorithmType {
    ReinforcementLearning,
    SupervisedLearning,
    UnsupervisedLearning,
    TransferLearning,
    DeepReinforcementLearning,
    EnsembleLearning,
    MetaLearning,
    OnlineLearning,
}

/// Configuration for learning algorithms
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlgorithmConfig {
    pub learning_rate: f64,
    pub discount_factor: f64,
    pub exploration_rate: f64,
    pub min_exploration_rate: Option<f64>,
    pub exploration_decay: Option<f64>,
    pub max_iterations: usize,
    pub max_episodes: Option<usize>,
    pub convergence_threshold: f64,
}

impl Default for AlgorithmConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.1,
            discount_factor: 0.9,
            exploration_rate: 0.1,
            min_exploration_rate: Some(0.01),
            exploration_decay: Some(0.995),
            max_iterations: 1000,
            max_episodes: Some(1000),
            convergence_threshold: 0.001,
        }
    }
}

/// Q-learning table for reinforcement learning

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QTable {
    q_values: HashMap<String, HashMap<String, f64>>,
}

impl QTable {
    pub fn new() -> Self {
        Self {
            q_values: HashMap::new(),
        }
    }

    pub fn get(&self, state: &str, action: &str) -> f64 {
        self.q_values
            .get(state)
            .and_then(|actions| actions.get(action))
            .copied()
            .unwrap_or(0.0)
    }

    pub fn set(&mut self, state: &str, action: &str, value: f64) {
        self.q_values
            .entry(state.to_string())
            .or_insert_with(HashMap::new)
            .insert(action.to_string(), value);
    }

    pub fn get_best_action(&self, state: &str) -> Option<String> {
        self.q_values
            .get(state)?
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(action, _)| action.clone())
    }

    pub fn get_actions(&self, state: &str) -> Vec<String> {
        self.q_values
            .get(state)
            .map(|actions| actions.keys().cloned().collect())
            .unwrap_or_default()
    }
}

impl Default for QTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for ensemble learning components
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnsembleComponentStatistics {
    pub component_id: String,
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
    pub training_time_ms: u64,
    pub prediction_time_ms: u64,
    #[schemars(with = "String")]

    pub last_updated: DateTime<Utc>,
}

/// Contribution of a component to ensemble predictions
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ComponentContribution {
    pub component_id: String,
    pub weight: f64,
    pub confidence: f64,
    pub prediction: serde_json::Value,
}

/// Analytics for ensemble learning performance
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnsembleAnalytics {
    pub overall_accuracy: f64,
    pub component_contributions: Vec<ComponentContribution>,
    pub diversity_score: f64,
    pub stability_score: f64,
    #[schemars(with = "String")]

    pub generated_at: DateTime<Utc>,
}

/// Characteristics of a learning problem
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProblemCharacteristics {
    pub feature_count: usize,
    pub sample_count: usize,
    pub class_count: Option<usize>,
    pub has_missing_values: bool,
    pub is_regression: bool,
    pub estimated_complexity: f64,
}

/// Performance metrics for learning algorithms
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlgorithmPerformance {
    pub algorithm_type: LearningAlgorithmType,
    pub accuracy: f64,
    pub training_time_ms: u64,
    pub prediction_time_ms: u64,
    pub memory_usage_mb: f64,
    pub convergence_iterations: usize,
    #[schemars(with = "String")]

    pub measured_at: DateTime<Utc>,
}

/// Learning data point
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LearningDataPoint {
    pub input: LearningInput,
    pub expected_output: LearningOutput,
    pub context: LearningContext,
}

/// Learning input
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum LearningInput {
    TaskPrediction {
        task_type: TaskType,
        complexity: TaskComplexity,
        historical_performance: Option<HistoricalPerformance>,
    },
    QualityAssessment {
        code_sample: String,
        requirements: Vec<String>,
    },
    ResourceEstimation {
        task_type: TaskType,
        complexity: TaskComplexity,
        constraints: Vec<String>,
    },
}

/// Learning output
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum LearningOutput {
    TaskPrediction {
        success_probability: f64,
        estimated_quality: f64,
        recommended_strategy: LearningStrategy,
    },
    QualityScore {
        overall_score: f64,
        component_scores: HashMap<String, f64>,
        recommendations: Vec<String>,
    },
    ResourceEstimate {
        cpu_hours: f64,
        memory_mb: u64,
        time_estimate_minutes: u64,
        confidence: f64,
    },
}

/// Learning context
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LearningContext {
    pub domain: String,
    pub technology_stack: Vec<String>,
    pub time_pressure: bool,
    pub quality_requirements: Vec<String>,
    pub system_metrics: Option<HashMap<String, serde_json::Value>>,
}

/// Learning feedback for algorithm improvement
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LearningFeedback {
    pub input: LearningInput,
    pub predicted_output: LearningOutput,
    pub actual_outcome: TaskOutcome,
    pub performance_delta: f64,
    pub lessons_learned: Vec<String>,
}

/// Learning system health monitor

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LearningSystemHealth {
    pub algorithm_count: usize,
    pub total_training_sessions: u64,
    pub average_performance: f64,
    pub system_uptime_seconds: u64,
    pub memory_usage_mb: f64,
    #[schemars(with = "String")]

    pub last_health_check: DateTime<Utc>,
}

impl LearningSystemHealth {
    pub fn new() -> Self {
        Self {
            algorithm_count: 0,
            total_training_sessions: 0,
            average_performance: 0.0,
            system_uptime_seconds: 0,
            memory_usage_mb: 0.0,
            last_health_check: chrono::Utc::now(),
        }
    }

    /// Check if the system is healthy
    pub fn is_healthy(&self) -> bool {
        self.algorithm_count > 0 &&
        self.average_performance > 0.5 &&
        self.memory_usage_mb < 1000.0 // Less than 1GB
    }

    /// Get health score (0.0 to 1.0)
    pub fn health_score(&self) -> f64 {
        let mut score: f64 = 0.0;

        // Algorithm availability (30%)
        if self.algorithm_count > 0 {
            score += 0.3 * (self.algorithm_count as f64 / 5.0).min(1.0);
        }

        // Performance (40%)
        score += 0.4 * self.average_performance;

        // Memory usage (20%) - lower is better
        let memory_score = if self.memory_usage_mb < 500.0 {
            1.0
        } else if self.memory_usage_mb < 1000.0 {
            0.5
        } else {
            0.0
        };
        score += 0.2 * memory_score;

        // Training activity (10%)
        let training_score = (self.total_training_sessions as f64 / 100.0).min(1.0);
        score += 0.1 * training_score;

        score.min(1.0)
    }

    /// Update health metrics
    pub fn update_metrics(&mut self, algorithm_count: usize, performance_tracker: &AlgorithmPerformanceTracker) {
        self.algorithm_count = algorithm_count;
        self.total_training_sessions += 1;
        self.last_health_check = chrono::Utc::now();

        // Calculate average performance across all algorithms
        let mut total_performance = 0.0;
        let mut count = 0;

        for algorithm_type in [
            LearningAlgorithmType::ReinforcementLearning,
            LearningAlgorithmType::SupervisedLearning,
            LearningAlgorithmType::UnsupervisedLearning,
            LearningAlgorithmType::EnsembleLearning,
        ].iter() {
            if let Some(performance) = performance_tracker.get_average_performance(algorithm_type) {
                total_performance += performance.accuracy;
                count += 1;
            }
        }

        if count > 0 {
            self.average_performance = total_performance / count as f64;
        }
    }
}

/// Algorithm performance tracker

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AlgorithmPerformanceTracker {
    performance_history: HashMap<LearningAlgorithmType, Vec<AlgorithmPerformance>>,
}

impl AlgorithmPerformanceTracker {
    pub fn new() -> Self {
        Self {
            performance_history: HashMap::new(),
        }
    }

    /// Record algorithm performance
    pub fn record_performance(&mut self, performance: AlgorithmPerformance) {
        self.performance_history
            .entry(performance.algorithm_type)
            .or_insert_with(Vec::new)
            .push(performance);
    }

    /// Get recent performance for algorithm type
    pub fn get_recent_performance(&self, algorithm_type: &LearningAlgorithmType, count: usize) -> Vec<&AlgorithmPerformance> {
        if let Some(history) = self.performance_history.get(algorithm_type) {
            history.iter().rev().take(count).collect()
        } else {
            Vec::new()
        }
    }

    /// Get average performance for algorithm type
    pub fn get_average_performance(&self, algorithm_type: &LearningAlgorithmType) -> Option<AlgorithmPerformance> {
        let performances = self.performance_history.get(algorithm_type)?;
        if performances.is_empty() {
            return None;
        }

        let avg_accuracy = performances.iter().map(|p| p.accuracy).sum::<f64>() / performances.len() as f64;
        let avg_training_time = performances.iter().map(|p| p.training_time_ms).sum::<u64>() / performances.len() as u64;
        let avg_prediction_time = performances.iter().map(|p| p.prediction_time_ms).sum::<u64>() / performances.len() as u64;

        Some(AlgorithmPerformance {
            algorithm_type: algorithm_type.clone(),
            accuracy: avg_accuracy,
            training_time_ms: avg_training_time,
            prediction_time_ms: avg_prediction_time,
            memory_usage_mb: 0.0, // Would need to track this
            convergence_iterations: performances.iter().map(|p| p.convergence_iterations).max().unwrap_or(0),
            measured_at: chrono::Utc::now(),
        })
    }
}

/// Learning task for the system
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LearningTask {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub task_type: TaskType,
    pub complexity: TaskComplexity,
    #[serde(with = "duration_serde")]
    #[schemars(with = "i64")]
    pub expected_duration: chrono::Duration,
    pub success_criteria: Vec<SuccessCriterion>,
    pub context: TaskContext,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, JsonSchema)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq, JsonSchema)]
pub enum TaskComplexity {
    Simple,
    Moderate,
    Complex,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SuccessCriterion {
    pub criterion_type: CriterionType,
    pub description: String,
    pub measurable: bool,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum CriterionType {
    Functional,
    Performance,
    Quality,
    Security,
    Compliance, // CAWS compliance
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskContext {
    pub domain: String,
    pub technology_stack: Vec<String>,
    pub constraints: Vec<Constraint>,
    pub historical_performance: Option<HistoricalPerformance>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Constraint {
    pub constraint_type: ConstraintType,
    pub description: String,
    pub severity: ConstraintSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ConstraintType {
    Time,
    Resource,
    Quality,
    Security,
    Compliance,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ConstraintSeverity {
    Soft,
    Hard,
    Critical,
}

/// Quality indicators captured from council evaluations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, JsonSchema)]
pub enum QualityIndicator {
    HighConfidence,
    ComprehensiveEvidence,
    MinimalDissent,
    EfficientExecution,
    StrongCAWSCompliance,
    CompleteClaimVerification,
}

/// Categories for failure analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, JsonSchema)]
pub enum FailureCategory {
    ConsensusFailure,
    ResourceExhaustion,
    CAWSViolation,
    ClaimVerificationFailure,
    JudgeTimeout,
    SystemError,
}

/// Partial results captured when a task times out
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PartialResults {
    pub completed_judges: Vec<Uuid>,
    pub partial_consensus: f32,
    pub estimated_completion: f32,
}

/// Outcome classification for predictive learning
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum TaskOutcome {
    Success {
        confidence: f32,
        quality_indicators: Vec<QualityIndicator>,
    },
    PartialSuccess {
        issues: Vec<String>,
        confidence: f32,
        remediation_applied: bool,
    },
    Failure {
        reason: String,
        failure_category: FailureCategory,
        recoverable: bool,
    },
    Timeout {
        duration_ms: u64,
        partial_results: Option<PartialResults>,
    },
}

/// Learning session tracking progress
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LearningSession {
    #[schemars(with = "String")]
    pub id: Uuid,
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub task_type: TaskType,
    #[schemars(with = "String")]

    pub start_time: DateTime<Utc>,
    pub current_turn: u32,
    pub progress: ProgressMetrics,
    pub learning_state: LearningState,
    pub context_preservation: ContextPreservationState,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProgressMetrics {
    pub completion_percentage: f64,
    pub quality_score: f64,
    pub efficiency_score: f64,
    pub error_rate: f64,
    pub learning_velocity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LearningState {
    pub current_strategy: LearningStrategy,
    pub adaptation_history: Vec<AdaptationEvent>,
    pub performance_trends: PerformanceTrends,
    pub resource_utilization: ResourceUtilization,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub enum LearningStrategy {
    Conservative,
    Balanced,
    Aggressive,
    Adaptive,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AdaptationEvent {
    pub timestamp: DateTime<Utc>,
    pub adaptation_type: AdaptationType,
    pub trigger: AdaptationTrigger,
    pub impact: AdaptationImpact,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum AdaptationType {
    StrategyChange,
    ResourceReallocation,
    ContextAdjustment,
    LearningRateAdjustment,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum AdaptationTrigger {
    PerformanceDegradation,
    QualityIssue,
    ResourceConstraint,
    CouncilFeedback,
    ErrorPattern,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AdaptationImpact {
    pub performance_change: f64,
    pub quality_change: f64,
    pub efficiency_change: f64,
    pub confidence_change: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PerformanceTrends {
    pub short_term: TrendData,
    pub medium_term: TrendData,
    pub long_term: TrendData,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TrendData {
    pub direction: TrendDirection,
    pub magnitude: f64,
    pub confidence: f64,
    pub data_points: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum TrendDirection {
    Improving,
    Declining,
    Stable,
    Volatile,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResourceUtilization {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub token_usage: f64,
    pub time_usage: f64,
    pub efficiency_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContextPreservationState {
    pub preserved_contexts: Vec<PreservedContext>,
    pub context_freshness: HashMap<String, DateTime<Utc>>,
    pub context_usage: HashMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PreservedContext {
    #[schemars(with = "String")]
    pub context_id: Uuid,
    pub context_type: ContextType,
    pub content: String,
    pub relevance_score: f64,
    #[schemars(with = "String")]

    pub last_accessed: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ContextType {
    CodeContext,
    DocumentationContext,
    TestContext,
    ErrorContext,
    PerformanceContext,
}

/// Credit assignment for learning
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreditAssignment {
    #[schemars(with = "String")]
    pub session_id: Uuid,
    pub turn_credits: Vec<TurnCredit>,
    pub total_credit: f64,
    pub credit_distribution: CreditDistribution,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TurnCredit {
    pub turn_number: u32,
    pub credit_amount: f64,
    pub credit_type: CreditType,
    pub contributing_factors: Vec<ContributingFactor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum CreditType {
    Positive,
    Negative,
    Neutral,
    Corrective,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContributingFactor {
    pub factor_type: FactorType,
    pub impact: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum FactorType {
    Quality,
    Efficiency,
    Innovation,
    Compliance,
    ErrorReduction,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreditDistribution {
    pub strategy_credit: f64,
    pub resource_credit: f64,
    pub context_credit: f64,
    pub adaptation_credit: f64,
}

/// Learning signals from council
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CouncilLearningSignal {
    #[schemars(with = "String")]
    pub signal_id: Uuid,
    pub council_judge: CouncilJudge,
    pub signal_type: LearningSignalType,
    pub content: String,
    pub confidence: f64,
    #[schemars(with = "String")]

    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum CouncilJudge {
    Constitutional,
    Technical,
    Quality,
    Integration,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum LearningSignalType {
    PerformanceFeedback,
    QualityAssessment,
    ComplianceViolation,
    ResourceRecommendation,
    StrategySuggestion,
}

/// Learning update from processing signals
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LearningUpdate {
    #[schemars(with = "String")]
    pub update_id: Uuid,
    #[schemars(with = "String")]
    pub session_id: Uuid,
    pub update_type: LearningUpdateType,
    pub changes: Vec<LearningChange>,
    pub impact_assessment: ImpactAssessment,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum LearningUpdateType {
    StrategyAdjustment,
    ResourceReallocation,
    ContextUpdate,
    PerformanceOptimization,
    SelfPromptingOptimization,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LearningChange {
    pub change_type: ChangeType,
    pub description: String,
    pub magnitude: f64,
    pub expected_impact: ExpectedImpact,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ChangeType {
    LearningRate,
    StrategyWeight,
    ResourceAllocation,
    ContextThreshold,
    QualityThreshold,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExpectedImpact {
    pub performance_impact: f64,
    pub quality_impact: f64,
    pub efficiency_impact: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ImpactAssessment {
    pub overall_impact: f64,
    pub risk_level: RiskLevel,
    pub implementation_effort: ImplementationEffort,
    pub rollback_plan: Option<RollbackPlan>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ImplementationEffort {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RollbackPlan {
    pub rollback_steps: Vec<RollbackStep>,
    #[serde(with = "duration_serde")]
    #[schemars(with = "i64")]
    pub rollback_time_estimate: chrono::Duration,
    pub rollback_risk: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RollbackStep {
    pub step_number: u32,
    pub description: String,
    #[serde(with = "duration_serde")]
    #[schemars(with = "i64")]
    pub estimated_time: chrono::Duration,
}

/// Snapshot of the learning context for predictive analytics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskLearningSnapshot {
    pub outcome: TaskOutcome,
    pub progress_metrics: Option<ProgressMetrics>,
    pub historical_performance: Option<HistoricalPerformance>,
    pub recent_resource_usage: Option<ResourceUtilization>,
    pub turn_count: u32,
    pub timestamp: DateTime<Utc>,
}

impl TaskLearningSnapshot {
    pub fn from_outcome(outcome: TaskOutcome) -> Self {
        Self {
            outcome,
            progress_metrics: None,
            historical_performance: None,
            recent_resource_usage: None,
            turn_count: 0,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn with_progress(mut self, metrics: ProgressMetrics) -> Self {
        self.progress_metrics = Some(metrics);
        self
    }

    pub fn with_history(mut self, history: HistoricalPerformance) -> Self {
        self.historical_performance = Some(history);
        self
    }

    pub fn with_resources(mut self, utilization: ResourceUtilization) -> Self {
        self.recent_resource_usage = Some(utilization);
        self
    }

    pub fn with_turn_count(mut self, turn_count: u32) -> Self {
        self.turn_count = turn_count;
        self
    }
}

/// Prediction of future task performance
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PerformancePrediction {
    pub expected_quality_score: f64,
    pub success_probability: f64,
    pub predicted_completion_time_ms: u64,
    pub risk_level: RiskLevel,
    pub confidence: f64,
    pub supporting_factors: Vec<String>,
}

/// Recommendation for strategy adjustments
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StrategyOptimizationPlan {
    pub recommended_strategy: LearningStrategy,
    pub adjustments: Vec<StrategyAdjustmentSuggestion>,
    pub expected_quality_gain: f64,
    pub expected_efficiency_gain: f64,
    pub confidence: f64,
    pub rationale: Vec<String>,
}

/// Suggested adjustment with focus area and magnitude
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StrategyAdjustmentSuggestion {
    pub focus: StrategyAdjustmentFocus,
    pub magnitude: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, JsonSchema)]
pub enum StrategyAdjustmentFocus {
    Quality,
    Efficiency,
    Resource,
    Context,
    Exploration,
}

/// Prediction of future resource requirements
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResourcePrediction {
    pub expected_cpu_usage: f64,
    pub expected_memory_mb: f64,
    pub expected_token_usage: f64,
    pub expected_duration_ms: u64,
    pub pressure_level: ResourcePressureLevel,
    pub confidence: f64,
    pub bottlenecks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub enum ResourcePressureLevel {
    Low,
    Moderate,
    High,
    Critical,
}

/// Aggregated predictive learning insights
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PredictiveLearningInsights {
    pub performance: PerformancePrediction,
    pub strategy: StrategyOptimizationPlan,
    pub resources: ResourcePrediction,
}

/// Historical performance data
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HistoricalPerformance {
    pub task_type: TaskType,
    #[serde(with = "duration_serde")]
    #[schemars(with = "i64")]
    pub average_completion_time: chrono::Duration,
    pub average_quality_score: f64,
    pub success_rate: f64,
    pub common_failure_patterns: Vec<FailurePattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FailurePattern {
    pub pattern_type: FailureType,
    pub frequency: f64,
    pub impact: f64,
    pub mitigation_strategy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub enum FailureType {
    QualityFailure,
    PerformanceFailure,
    ComplianceFailure,
    ResourceFailure,
    ContextFailure,
}

/// Errors for the learning system

#[derive(Debug, Serialize, Deserialize, JsonSchema, thiserror::Error)]
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

    #[error("Initialization failed: {0}")]
    InitializationError(String),

    #[error("Persistence failed: {0}")]
    PersistenceError(String),

    #[error("Validation failed: {0}")]
    ValidationError(String),
}

impl From<String> for LearningSystemError {
    fn from(error: String) -> Self {
        LearningSystemError::InitializationError(error)
    }
}

/// Learning signals from self-prompting agent execution
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum SelfPromptingSignal {
    /// Iteration efficiency patterns
    IterationEfficiency {
        iterations: usize,
        quality: f64,
        time: f64, // milliseconds per iteration
    },

    /// Model performance on specific tasks
    ModelPerformance {
        model_id: String,
        task_type: String,
        score: f64,
    },

    /// Effectiveness of satisficing decisions
    SatisficingEffectiveness {
        stopped_early: bool,
        quality_delta: f64,
        iterations_saved: usize,
    },
}
