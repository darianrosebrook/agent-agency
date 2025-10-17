//! Types for model benchmarking system

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Model specification for benchmarking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSpecification {
    pub id: Uuid,
    pub name: String,
    pub model_type: ModelType,
    pub parameters: ModelParameters,
    pub capabilities: Vec<Capability>,
    pub constraints: Vec<ModelConstraint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelType {
    CodeGeneration,
    CodeReview,
    Testing,
    Documentation,
    Research,
    Analysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelParameters {
    pub size: u64,
    pub context_length: u32,
    pub training_data: String,
    pub architecture: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub capability_type: CapabilityType,
    pub proficiency_level: ProficiencyLevel,
    pub supported_domains: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CapabilityType {
    CodeGeneration,
    CodeReview,
    Testing,
    Documentation,
    Research,
    Analysis,
    Debugging,
    Refactoring,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProficiencyLevel {
    Basic,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConstraint {
    pub constraint_type: ConstraintType,
    pub value: f64,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    MaxTokens,
    MaxTime,
    MaxMemory,
    MaxCpu,
}

/// Benchmarking report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkingReport {
    pub report_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub benchmark_results: Vec<BenchmarkResult>,
    pub performance_summary: PerformanceSummary,
    pub recommendations: Vec<Recommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub model_id: Uuid,
    pub benchmark_type: BenchmarkType,
    pub metrics: BenchmarkMetrics,
    pub score: f64,
    pub ranking: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BenchmarkType {
    MicroBenchmark,
    MacroBenchmark,
    QualityBenchmark,
    PerformanceBenchmark,
    ComplianceBenchmark,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMetrics {
    pub accuracy: f64,
    pub speed: f64,
    pub efficiency: f64,
    pub quality: f64,
    pub compliance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub overall_performance: f64,
    pub performance_trend: PerformanceTrend,
    pub top_performers: Vec<ModelPerformance>,
    pub improvement_areas: Vec<ImprovementArea>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PerformanceTrend {
    Improving,
    Declining,
    Stable,
    Volatile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformance {
    pub model_id: Uuid,
    pub model_name: String,
    pub performance_score: f64,
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementArea {
    pub area: String,
    pub current_score: f64,
    pub target_score: f64,
    pub improvement_potential: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub recommendation_type: RecommendationType,
    pub description: String,
    pub priority: Priority,
    pub expected_impact: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    ModelSelection,
    ResourceAllocation,
    PerformanceOptimization,
    QualityImprovement,
    ComplianceEnhancement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// Model evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEvaluationResult {
    pub evaluation_id: Uuid,
    pub model_spec: ModelSpecification,
    pub evaluation_metrics: EvaluationMetrics,
    pub comparison_results: ComparisonResults,
    pub recommendation: ModelRecommendation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationMetrics {
    pub overall_score: f64,
    pub capability_scores: Vec<CapabilityScore>,
    pub performance_metrics: BenchmarkMetrics,
    pub compliance_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityScore {
    pub capability: CapabilityType,
    pub score: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResults {
    pub baseline_comparison: BaselineComparison,
    pub peer_comparison: PeerComparison,
    pub historical_comparison: HistoricalComparison,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineComparison {
    pub baseline_model: String,
    pub performance_delta: f64,
    pub improvement_areas: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerComparison {
    pub peer_models: Vec<PeerModel>,
    pub ranking: u32,
    pub percentile: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerModel {
    pub model_id: Uuid,
    pub model_name: String,
    pub performance_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalComparison {
    pub historical_average: f64,
    pub trend_direction: PerformanceTrend,
    pub improvement_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRecommendation {
    pub recommendation: RecommendationDecision,
    pub reasoning: String,
    pub confidence: f64,
    pub conditions: Vec<Condition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationDecision {
    Adopt,
    Reject,
    ConditionalAdopt,
    FurtherEvaluation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub condition_type: ConditionType,
    pub description: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionType {
    PerformanceImprovement,
    ResourceConstraint,
    ComplianceRequirement,
    QualityThreshold,
}

/// Task context for routing recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub task_type: TaskType,
    pub complexity: TaskComplexity,
    pub domain: String,
    pub constraints: Vec<TaskConstraint>,
    pub quality_requirements: QualityRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    CodeGeneration,
    CodeReview,
    Testing,
    Documentation,
    Research,
    Analysis,
    Debugging,
    Refactoring,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskComplexity {
    Simple,
    Moderate,
    Complex,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConstraint {
    pub constraint_type: ConstraintType,
    pub value: f64,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRequirements {
    pub minimum_quality: f64,
    pub compliance_required: bool,
    pub performance_target: f64,
    pub error_tolerance: f64,
}

/// Routing recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRecommendation {
    pub recommended_model: Uuid,
    pub confidence: f64,
    pub reasoning: String,
    pub expected_performance: ExpectedPerformance,
    pub resource_requirements: ResourceRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedPerformance {
    pub quality_score: f64,
    pub completion_time: chrono::Duration,
    pub success_probability: f64,
    pub error_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: u32,
    pub memory_mb: u64,
    pub storage_mb: u64,
    pub network_bandwidth: u64,
}

/// Benchmarking errors
#[derive(Debug, thiserror::Error)]
pub enum BenchmarkingError {
    #[error("Benchmark execution failed: {0}")]
    BenchmarkExecutionFailed(String),
    
    #[error("Model evaluation failed: {0}")]
    ModelEvaluationFailed(String),
    
    #[error("Performance tracking failed: {0}")]
    PerformanceTrackingFailed(String),
    
    #[error("Scoring system failed: {0}")]
    ScoringSystemFailed(String),
    
    #[error("Regression detection failed: {0}")]
    RegressionDetectionFailed(String),
    
    #[error("Metrics collection failed: {0}")]
    MetricsCollectionFailed(String),
    
    #[error("General error: {0}")]
    GeneralError(String),
}

impl From<anyhow::Error> for BenchmarkingError {
    fn from(err: anyhow::Error) -> Self {
        BenchmarkingError::GeneralError(err.to_string())
    }
}

/// Alert for performance regressions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionAlert {
    pub model_id: Uuid,
    pub metric_name: String,
    pub current_value: f64,
    pub previous_value: f64,
    pub regression_percentage: f64,
    pub severity: RegressionSeverity,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegressionSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Benchmark report containing results and analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub report_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub benchmark_results: Vec<BenchmarkResult>,
    pub performance_summary: PerformanceSummary,
    pub regression_alerts: Vec<RegressionAlert>,
    pub recommendations: Vec<ModelRecommendation>,
}

/// Result of comparing model performance against baseline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub improvement_percentage: f64,
    pub regression_areas: Vec<String>,
    pub improvement_areas: Vec<String>,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
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
    VeryHigh,
}

