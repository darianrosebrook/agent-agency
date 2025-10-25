//! Core types for learning signals and adaptive routing

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::types::{JudgeId, TaskId, VerdictId, SpecializationScore, TaskType, HistoricalJudgeData, ResourceTrend, TrendType, ResourceUsageMetrics, ResourcePrediction};

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

    // Additional fields
    pub signal_type: String,
    pub confidence: f32,
    pub data: serde_json::Value,
    pub source: String,
}

/// Task outcome classification for learning
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskOutcome {
    Success,
    Failure,
    Timeout,
    ResourceExhaustion,
    QualityIssue,
    ConsensusFailure,
    EthicalViolation,
}

/// Judge dissent information for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeDissent {
    pub judge_id: JudgeId,
    pub dissenting_reason: String,
    pub confidence_in_dissent: f32,
    pub alternative_verdict: Option<String>,
}

/// Task complexity assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskComplexity {
    pub algorithmic_complexity: f32,
    pub data_complexity: f32,
    pub coordination_complexity: f32,
    pub overall_complexity: f32,
}

/// Worker performance metrics for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerPerformanceMetrics {
    pub worker_id: String,
    pub task_completion_rate: f32,
    pub average_latency_ms: u64,
    pub error_rate: f32,
    pub resource_efficiency: f32,
}

/// Routing recommendation for task assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRecommendation {
    pub recommended_judges: Vec<JudgeRecommendation>,
    pub resource_allocation: ResourceAllocation,
    pub estimated_complexity: f32,
    pub confidence: f32,
    pub rationale: String,
}

/// Individual judge recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeRecommendation {
    pub judge_id: JudgeId,
    pub confidence_score: f32,
    pub specialization_match: f32,
    pub performance_history: JudgePerformanceHistory,
    pub rationale: String,
}

/// Judge performance history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgePerformanceHistory {
    pub tasks_evaluated: u32,
    pub success_rate: f32,
    pub average_latency_ms: u64,
    pub quality_score_average: f32,
    pub recent_performance_trend: PerformanceTrend,
}

/// Performance trend indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceTrend {
    Improving,
    Stable,
    Declining,
    Volatile,
}

/// Resource allocation recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub judge_id: String,
    pub cpu_cores: u32,
    pub memory_gb: u32,
    pub memory_mb: u32,
    pub estimated_duration_ms: u64,
    pub preferred_accelerator: AcceleratorPreference,
    pub thermal_budget: f32,
}

/// Preferred accelerator type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AcceleratorPreference {
    CPU,
    GPU,
    ANE,
    NeuralEngine,
    Auto,
}

/// Task features for similarity analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFeatures {
    pub task_type: TaskType,
    pub complexity_score: f32,
    pub resource_requirements: ResourceRequirements,
    pub domain_keywords: Vec<String>,
    pub technical_stack: Vec<String>,
    pub estimated_duration: Option<u64>,
}

/// Resource requirements for task analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: u32,
    pub memory_gb: u32,
    pub storage_gb: u32,
    pub network_bandwidth_mbps: u32,
    pub special_hardware: Vec<String>,
}

/// Judge performance analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgePerformanceAnalysis {
    pub judge_id: JudgeId,
    pub overall_score: f32,
    pub task_type_performance: HashMap<TaskType, f32>,
    pub resource_efficiency: f32,
    pub quality_consistency: f32,
    pub recommended_judges: Vec<JudgeRecommendation>,
}

/// Resource requirements analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAnalysis {
    pub optimal_allocation: Option<ResourceAllocation>,
    pub alternative_allocations: Vec<ResourceAllocation>,
    pub estimated_complexity: f32,
    pub resource_utilization_prediction: f32,
    pub scaling_recommendations: Vec<String>,
}

/// Learning signal storage trait
#[async_trait::async_trait]
pub trait LearningSignalStorage: Send + Sync {
    async fn store_signal(&self, signal: LearningSignal) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn get_signals(&self, task_id: Option<TaskId>, limit: usize) -> Result<Vec<LearningSignal>, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_similar_signals(&self, features: &TaskFeatures, limit: usize) -> Result<Vec<LearningSignal>, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_judge_performance(&self, judge_id: &JudgeId, task_type: Option<TaskType>) -> Result<Vec<LearningSignal>, Box<dyn std::error::Error + Send + Sync>>;
    async fn cleanup_old_signals(&self, max_age_days: u32) -> Result<usize, Box<dyn std::error::Error + Send + Sync>>;
}

/// Learning signal analyzer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningSignalAnalyzerConfig {
    pub max_similar_tasks: usize,
    pub performance_history_window_days: u32,
    pub confidence_threshold: f32,
    pub enable_adaptive_routing: bool,
    pub enable_resource_optimization: bool,
    pub signal_retention_days: u32,
}

impl Default for LearningSignalAnalyzerConfig {
    fn default() -> Self {
        Self {
            max_similar_tasks: 50,
            performance_history_window_days: 30,
            confidence_threshold: 0.7,
            enable_adaptive_routing: true,
            enable_resource_optimization: true,
            signal_retention_days: 90,
        }
    }
}

/// Learning signal analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningSignalAnalysis {
    pub routing_recommendation: RoutingRecommendation,
    pub performance_insights: Vec<String>,
    pub improvement_suggestions: Vec<String>,
    pub confidence_score: f32,
    pub analysis_timestamp: DateTime<Utc>,
}

/// Performance metrics for learning analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub average_latency_ms: u64,
    pub success_rate: f32,
    pub resource_utilization: f32,
    pub quality_score: f32,
    pub throughput_signals_per_minute: f32,
}

/// Learning system health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningSystemHealth {
    pub signal_count: u64,
    pub average_processing_time_ms: u64,
    pub storage_health: StorageHealth,
    pub analysis_health: AnalysisHealth,
    pub last_updated: DateTime<Utc>,
}

/// Storage health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageHealth {
    pub total_signals: u64,
    pub storage_size_bytes: u64,
    pub cleanup_operations: u64,
    pub last_cleanup: Option<DateTime<Utc>>,
}

/// Analysis health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisHealth {
    pub average_analysis_time_ms: u64,
    pub error_rate: f32,
    pub cache_hit_rate: f32,
    pub last_analysis: Option<DateTime<Utc>>,
}

impl Default for TaskOutcome {
    fn default() -> Self {
        TaskOutcome::Success
    }
}

impl Default for TaskComplexity {
    fn default() -> Self {
        Self {
            algorithmic_complexity: 1.0,
            data_complexity: 1.0,
            coordination_complexity: 1.0,
            overall_complexity: 1.0,
        }
    }
}
