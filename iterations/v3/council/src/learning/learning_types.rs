//! Core types for the learning signal system
//!
//! This module contains the fundamental types and data structures
//! used throughout the learning signal infrastructure.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::council_types::{JudgeId, TaskId, VerdictId, SpecializationScore, TaskType, HistoricalJudgeData, ResourceTrend, TrendType, ResourceUsageMetrics, ResourcePrediction};

/// Learning signal capturing task outcomes and judge performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningSignal {
    pub id: Uuid,
    pub task_id: TaskId,
    pub verdict_id: VerdictId,
    pub outcome: TaskOutcome,
    pub judge_dissent: Vec<JudgeDissent>,
    pub latency_ms: u64,
    pub quality_score: f32,
    pub timestamp: DateTime<Utc>,

    // Performance metrics
    pub resource_usage: ResourceUsageMetrics,
    pub caws_compliance_score: f32,
    pub claim_verification_score: Option<f32>,

    // Context for learning
    pub task_complexity: TaskComplexity,
    pub worker_performance: Option<WorkerPerformanceMetrics>,

    // Additional fields expected by the code
    pub signal_type: String,
    pub confidence: f32,
    pub data: serde_json::Value,
    pub source: String,
}

/// Task outcome classification for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Quality indicators for successful tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityIndicator {
    HighConfidence,
    ComprehensiveEvidence,
    MinimalDissent,
    EfficientExecution,
    StrongCAWSCompliance,
    CompleteClaimVerification,
}

/// Failure categories for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureCategory {
    ConsensusFailure,
    ResourceExhaustion,
    CAWSViolation,
    ClaimVerificationFailure,
    JudgeTimeout,
    SystemError,
}

/// Partial results from timed-out tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialResults {
    pub completed_judges: Vec<JudgeId>,
    pub partial_consensus: f32,
    pub estimated_completion: f32,
}

/// Judge dissent tracking for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeDissent {
    pub judge_id: JudgeId,
    pub dissent_severity: DissentSeverity,
    pub rationale: String,
    pub confidence: f32,
    pub evidence_quality: f32,
}

/// Dissent severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DissentSeverity {
    Minor,    // Minor disagreement, easily resolved
    Moderate, // Significant disagreement, requires discussion
    Major,    // Fundamental disagreement, blocks consensus
    Critical, // Complete disagreement, system failure
}

/// Thermal status for resource optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThermalStatus {
    Normal,
    Warning,
    Throttling,
    Critical,
}

/// Effort levels for task complexity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffortLevel {
    Trivial,     // < 5 minutes
    Simple,      // 5-30 minutes
    Moderate,    // 30 minutes - 2 hours
    Complex,     // 2-8 hours
    VeryComplex, // > 8 hours
}

/// Risk factors affecting task complexity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskFactor {
    HighRiskTier,
    MultipleDomains,
    BreakingChanges,
    ExternalDependencies,
    PerformanceCritical,
    SecuritySensitive,
    DataMigration,
}

/// Worker performance metrics for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerPerformanceMetrics {
    pub worker_id: Uuid,
    pub execution_time_ms: u64,
    pub quality_score: f32,
    pub caws_compliance: f32,
    pub claim_accuracy: Option<f32>,
    pub resource_efficiency: f32,
}

/// Task complexity assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskComplexity {
    Simple,
    Moderate,
    Complex,
    VeryComplex,
}

/// Performance entity types for metrics tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceEntityType {
    Judge,
    Task,
    Worker,
}

/// Time windows for performance analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeWindow {
    Hour,
    Day,
    Week,
    Month,
}

/// Trend analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub trend_type: TrendType,
    pub direction: TrendDirection,
    pub magnitude: f32,
    pub confidence: f32,
    pub time_window: TimeWindow,
}

/// Judge ranking for performance optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeRanking {
    pub judge_id: JudgeId,
    pub specialization_score: SpecializationScore,
    pub performance_score: f32,
    pub reliability_score: f32,
    pub resource_efficiency: f32,
}

/// Aggregated performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    pub entity_id: String,
    pub entity_type: PerformanceEntityType,
    pub time_window: TimeWindow,
    pub success_rate: f32,
    pub average_latency_ms: f64,
    pub average_quality_score: f32,
    pub dissent_rate: f32,
    pub resource_efficiency: f32,
}

/// Performance trends over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrends {
    pub trends: Vec<TrendAnalysis>,
    pub overall_direction: TrendDirection,
    pub recommendations: Vec<LearningRecommendation>,
}

/// Trend direction indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Stable,
    Declining,
}

/// Learning recommendations for system improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningRecommendation {
    pub recommendation_type: RecommendationType,
    pub priority: RecommendationPriority,
    pub description: String,
    pub expected_impact: f32,
    pub implementation_effort: EffortLevel,
}

/// Types of learning recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    JudgeAssignment,
    ResourceAllocation,
    TaskPrioritization,
    SystemOptimization,
}

/// Recommendation priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}
