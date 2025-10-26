use chrono::Utc;
use std::collections::HashMap;

/// Core evaluation metrics
#[derive(Debug, Clone)]
pub struct EvaluationMetrics {
    pub total: u64,
    pub successful: u64,
    pub failed: u64,
    pub success_rate: f64,
}

/// SLA compliance metrics
#[derive(Debug, Clone)]
pub struct SLAMetrics {
    pub violations: u64,
    pub violation_rate: f64,
    pub threshold_ms: u64,
}

/// Judge performance snapshot
#[derive(Debug, Clone)]
pub struct JudgePerformanceSnapshot {
    pub judge_stats: HashMap<String, JudgePerformanceStats>,
    pub total_judges: u64,
    pub average_confidence: f32,
}

/// Health indicators for system monitoring
#[derive(Debug, Clone)]
pub struct HealthIndicators {
    pub active_evaluations: u64,
    pub queue_depth: u64,
    pub error_rate: f64,
}

/// Detailed timing metrics for SLA verification and testing
#[derive(Debug, Clone)]
pub struct TimingMetrics {
    pub total_evaluations: u64,
    pub successful_evaluations: u64,
    pub failed_evaluations: u64,
    pub total_evaluation_time_ms: u64,
    pub total_enrichment_time_ms: u64,
    pub total_judge_inference_time_ms: u64,
    pub total_debate_time_ms: u64,
    pub sla_violations: u64,
    pub average_evaluation_time_ms: u64,
    pub average_enrichment_time_ms: u64,
    pub average_judge_inference_time_ms: u64,
    pub average_debate_time_ms: u64,
}

/// Performance record for participant analysis
#[derive(Debug, Clone, Default)]
pub struct JudgePerformanceStats {
    pub total_evaluations: u64,
    pub successful_evaluations: u64,
    pub average_confidence: f32,
    pub total_time_ms: u64,
}

/// Coordinator metrics snapshot
#[derive(Debug, Clone)]
pub struct CoordinatorMetricsSnapshot {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub uptime_seconds: u64,
    pub evaluations: EvaluationMetrics,
    pub timing: TimingMetrics,
    pub sla: SLAMetrics,
    pub judge_performance: JudgePerformanceSnapshot,
    pub health: HealthIndicators,
}
