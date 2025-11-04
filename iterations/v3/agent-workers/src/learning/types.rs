//! Learning system types and data structures

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::parallel_types::{TaskId, WorkerId, WorkerSpecialty};

/// Execution record for learning analysis
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionRecord {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub task_id: TaskId,
    pub worker_id: WorkerId,
    pub execution_time_ms: u64,
    pub success: bool,
    pub quality_score: f64,
    pub error_message: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    #[schemars(with = "String")]

    pub created_at: DateTime<Utc>,
}

#[cfg(feature = "sqlx")]
impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for ExecutionRecord {
    fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        let id: Uuid = row.try_get("id")?;
        let task_id_uuid: Uuid = row.try_get("task_id")?;
        let worker_id_uuid: Uuid = row.try_get("worker_id")?;
        let execution_time_ms: i64 = row.try_get("execution_time_ms")?;
        let success = row.try_get::<i16, _>("success")? != 0; // Convert from smallint
        let quality_score: f64 = row.try_get("quality_score")?;
        let error_message: Option<String> = row.try_get("error_message")?;
        let metadata: serde_json::Value = row.try_get("metadata")?;
        let metadata_map: HashMap<String, serde_json::Value> = serde_json::from_value(metadata)
            .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
        let created_at: DateTime<Utc> = row.try_get("created_at")?;

        Ok(ExecutionRecord {
            id,
            task_id: TaskId(task_id_uuid),
            worker_id: WorkerId(worker_id_uuid),
            execution_time_ms: execution_time_ms as u64,
            success,
            quality_score,
            error_message,
            metadata: metadata_map,
            created_at,
        })
    }
}

/// Worker performance profile
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkerPerformanceProfile {
    pub worker_id: WorkerId,
    pub specialty: WorkerSpecialty,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub average_execution_time_ms: f64,
    pub average_quality_score: f64,
    #[schemars(with = "String")]

    pub last_updated: DateTime<Utc>,
    pub performance_trend: PerformanceTrend,
    pub capability_scores: HashMap<String, f64>,
    pub task_count: u64,
    pub success_rate: f64,
    pub quality_score: f64,
    pub specialization_score: f64,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Performance trend over time
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum PerformanceTrend {
    Improving,
    Stable,
    Declining,
    Unknown,
}

/// Success pattern identified from execution records
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SuccessPattern {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub pattern_type: PatternType,
    pub conditions: HashMap<String, serde_json::Value>,
    pub success_rate: f64,
    pub average_quality: f64,
    pub frequency: u64,
    #[schemars(with = "String")]

    pub created_at: DateTime<Utc>,
}

/// Failure pattern identified from execution records
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FailurePattern {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub pattern_type: PatternType,
    pub conditions: HashMap<String, serde_json::Value>,
    pub failure_rate: f64,
    pub common_errors: Vec<String>,
    pub frequency: u64,
    #[schemars(with = "String")]

    pub created_at: DateTime<Utc>,
}

/// Type of pattern
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum PatternType {
    TaskComplexity,
    WorkerCapability,
    ResourceConstraint,
    TimeConstraint,
    QualityRequirement,
    DependencyIssue,
}

/// Optimal configuration discovered through learning
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OptimalConfig {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub worker_type: String,
    pub task_type: String,
    pub config: serde_json::Value,
    pub performance_metrics: PerformanceMetrics,
    pub confidence: f64,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
}

/// Type of configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ConfigType {
    WorkerSelection,
    TaskDecomposition,
    ResourceAllocation,
    QualityThresholds,
    TimeoutSettings,
    RetryPolicy,
}

/// Performance metrics for configuration evaluation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PerformanceMetrics {
    pub execution_time_ms: f64,
    pub quality_score: f64,
    pub success_rate: f64,
    pub resource_utilization: f64,
    pub cost_score: f64,
}

/// Configuration recommendations
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConfigurationRecommendations {
    pub worker_selection: Option<WorkerSelectionRecommendation>,
    pub task_decomposition: Option<TaskDecompositionRecommendation>,
    pub resource_allocation: Option<ResourceAllocationRecommendation>,
    pub quality_thresholds: Option<QualityThresholdRecommendation>,
    pub confidence: f64,
    pub reasoning: String,
    pub recommended_config: Option<HashMap<String, serde_json::Value>>,
    pub confidence_score: f64,
    pub expected_improvement: Option<f64>,
    pub optimization_reason: Option<String>,
}

/// Worker selection recommendation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkerSelectionRecommendation {
    pub preferred_workers: Vec<WorkerId>,
    pub worker_weights: HashMap<WorkerId, f64>,
    pub reasoning: String,
}

/// Task decomposition recommendation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskDecompositionRecommendation {
    pub suggested_subtasks: u32,
    pub decomposition_strategy: String,
    pub reasoning: String,
}

/// Resource allocation recommendation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResourceAllocationRecommendation {
    pub cpu_allocation: f64,
    pub memory_allocation: f64,
    pub timeout_ms: u64,
    pub reasoning: String,
}

/// Quality threshold recommendation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualityThresholdRecommendation {
    pub min_quality_score: f64,
    pub max_rework_rate: f64,
    pub reasoning: String,
}

/// Optimization event
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OptimizationEvent {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub event_type: OptimizationEventType,
    #[schemars(with = "String")]
    pub config_id: Uuid,
    pub performance_delta: PerformanceMetrics,
    #[schemars(with = "String")]

    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub config_before: Option<HashMap<String, serde_json::Value>>,
    pub config_after: Option<HashMap<String, serde_json::Value>>,
    pub performance_improvement: Option<f64>,
    pub optimization_type: Option<String>,
}

/// Type of optimization event
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum OptimizationEventType {
    ConfigApplied,
    PerformanceImproved,
    PerformanceDegraded,
    ConfigRejected,
    LearningTriggered,
}

/// Task pattern for analysis
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskPattern {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub pattern_type: PatternType,
    pub characteristics: HashMap<String, serde_json::Value>,
    pub frequency: u64,
    #[schemars(with = "String")]

    pub last_seen: DateTime<Utc>,
    pub required_capabilities: Vec<String>,
}

/// Pattern match result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PatternMatch {
    #[schemars(with = "String")]
    pub pattern_id: Uuid,
    pub match_score: f64,
    pub matched_characteristics: Vec<String>,
    pub confidence: f64,
}

/// Reward weights for learning
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RewardWeights {
    pub quality: f64,
    pub latency: f64,
    pub rework: f64,
    pub cost: f64,
}

/// Baseline performance metrics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Baseline {
    pub p50_ms: f64,
    pub p50_quality: f64,
    pub p50_tokens: f64,
}

/// Fairness metrics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FairnessMetrics {
    pub gini_coefficient: f64,
    pub worker_utilization: HashMap<WorkerId, f64>,
    pub task_distribution: HashMap<WorkerId, u64>,
    #[schemars(with = "String")]

    pub last_updated: DateTime<Utc>,
}

/// Queue health metrics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QueueHealthMetrics {
    pub queue_depth: u64,
    pub average_wait_time_ms: f64,
    pub processing_rate: f64,
    pub error_rate: f64,
    #[schemars(with = "String")]

    pub last_updated: DateTime<Utc>,
}

/// Failure category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum FailureCategory {
    WorkerFailure,
    TaskFailure,
    ResourceExhaustion,
    Timeout,
    QualityViolation,
    DependencyFailure,
    ConfigurationError,
    Unknown,
}

/// Failure analysis result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FailureAnalysis {
    pub category: FailureCategory,
    pub root_cause: String,
    pub contributing_factors: Vec<String>,
    pub prevention_suggestions: Vec<String>,
    pub confidence: f64,
}

// Re-export ExecutionOutcome and LearningMode from worker_types
pub use crate::worker_types::{ExecutionOutcome, LearningMode};
