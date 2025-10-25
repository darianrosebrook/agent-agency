//! Storage interfaces and implementations for learning signals
//!
//! This module provides storage abstractions and implementations
//! for persisting and retrieving learning signals and historical data.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;
use async_trait::async_trait;

use crate::types::{JudgeId, TaskId, VerdictId, ResourceTrend, ResourceUsageMetrics, ResourcePrediction};
use super::types::*;

/// Learning signal storage and retrieval
#[async_trait::async_trait]
pub trait LearningSignalStorage: Send + Sync + std::fmt::Debug {
    /// Store a learning signal
    async fn store_signal(&self, signal: LearningSignal) -> Result<()>;

    /// Get learning signals for a task
    async fn get_signals_for_task(&self, task_id: TaskId) -> Result<Vec<LearningSignal>>;

    /// Get learning signals for a judge
    async fn get_signals_for_judge(&self, judge_id: &JudgeId) -> Result<Vec<LearningSignal>>;

    /// Get learning signals within time range
    async fn get_signals_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<LearningSignal>>;

    /// Get aggregated performance metrics
    async fn get_performance_metrics(
        &self,
        entity_type: PerformanceEntityType,
        entity_id: String,
        time_window: TimeWindow,
    ) -> Result<AggregatedMetrics>;

    /// Get learning recommendations
    async fn get_learning_recommendations(&self) -> Result<Vec<LearningRecommendation>>;

    /// Query database for historical resource data
    async fn query_database_for_historical_resource_data(&self, task_spec: &crate::types::TaskSpec) -> Result<HistoricalResourceData>;

    /// Get cached historical resource data
    async fn get_cached_historical_resource_data(&self, task_spec: &crate::types::TaskSpec) -> Result<Option<HistoricalResourceData>>;

    /// Aggregate historical resource data
    async fn aggregate_historical_resource_data(&self, db_data: &HistoricalResourceData, cached_data: Option<&HistoricalResourceData>) -> Result<HistoricalResourceData>;

    /// Perform comprehensive historical resource lookup
    async fn perform_comprehensive_historical_resource_lookup(&self, task_spec: &crate::types::TaskSpec) -> Result<HistoricalResourceData>;

    /// Monitor resource data performance
    async fn monitor_resource_data_performance(&self, query_time: Duration, result_count: usize, cache_hit: bool) -> Result<()>;

    /// Analyze resource usage trends
    async fn analyze_resource_usage_trends(&self, data: &HistoricalResourceData) -> Result<Vec<ResourceTrend>>;

    /// Generate resource usage predictions
    async fn generate_resource_usage_predictions(&self, data: &HistoricalResourceData, trends: &[ResourceTrend]) -> Result<Vec<ResourcePrediction>>;

    /// Estimate task complexity
    fn estimate_task_complexity(&self, task_spec: &crate::types::TaskSpec) -> TaskComplexity;
}

/// Historical resource usage data for trend analysis
#[derive(Debug, Clone)]
pub struct HistoricalResourceData {
    pub entries: Vec<HistoricalResourceEntry>,
    pub total_entries: usize,
    pub date_range: (DateTime<Utc>, DateTime<Utc>),
    pub query_timestamp: DateTime<Utc>,
    pub data_source: String,
}

/// Individual historical resource usage entry
#[derive(Debug, Clone)]
pub struct HistoricalResourceEntry {
    pub task_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub cpu_percent: f32,
    pub memory_mb: u32,
    pub io_bytes_per_sec: u64,
    pub duration_ms: u64,
    pub task_complexity: TaskComplexity,
    pub success: bool,
    pub resource_usage: ResourceUsageMetrics,
}

/// Resource usage pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsagePatterns {
    pub cpu_pattern: ResourcePattern,
    pub memory_pattern: ResourcePattern,
    pub io_pattern: ResourcePattern,
    pub seasonal_patterns: Vec<SeasonalPattern>,
    pub anomaly_patterns: Vec<ResourceAnomaly>,
}

/// Individual resource usage pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePattern {
    pub average: f32,
    pub peak: f32,
    pub trend: String,
    pub confidence: f32,
}

/// Seasonal resource usage pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonalPattern {
    pub pattern_type: String,
    pub description: String,
    pub impact: String,
    pub confidence: f32,
}

/// Resource usage anomaly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAnomaly {
    pub timestamp: DateTime<Utc>,
    pub resource_type: String,
    pub deviation: f32,
    pub description: String,
    pub severity: String,
}

/// Predicted resource requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictedResourceRequirements {
    pub cpu_percent: f32,
    pub memory_mb: u32,
    pub io_bytes_per_sec: u64,
    pub estimated_duration_ms: u64,
    pub confidence: f32,
    pub risk_factors: Vec<String>,
}

/// Risk assessment for resource allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub overall_risk: String,
    pub risk_factors: Vec<String>,
    pub mitigation_strategies: Vec<String>,
    pub contingency_plans: Vec<String>,
}

/// Monitoring alert for resource issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringAlert {
    pub alert_type: String,
    pub threshold: f32,
    pub severity: String,
    pub message: String,
}

/// Aggregated performance metrics with extended fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    pub total_signals: u64,
    pub success_rate: f32,
    pub average_quality_score: f32,
    pub average_latency_ms: f64,
    pub dissent_rate: f32,
    pub resource_efficiency: f32,
    pub trends: PerformanceTrends,
    pub time_range_days: u32,
    pub entity_type: String,
    pub entity_id: String,
    pub avg_quality_score: f32,
    pub avg_latency_ms: f64,
}

/// Performance trends data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrends {
    pub quality_trend: TrendDirection,
    pub latency_trend: TrendDirection,
    pub dissent_trend: TrendDirection,
    pub resource_efficiency_trend: TrendDirection,
}

/// In-memory implementation of LearningSignalStorage for development and testing
#[derive(Debug, Default)]
pub struct InMemoryLearningSignalStorage {
    signals: std::sync::RwLock<Vec<LearningSignal>>,
}

#[async_trait::async_trait]
impl LearningSignalStorage for InMemoryLearningSignalStorage {
    async fn store_signal(&self, signal: LearningSignal) -> Result<()> {
        let mut signals = self.signals.write().unwrap();
        signals.push(signal);
        Ok(())
    }

    async fn get_signals_for_task(&self, task_id: TaskId) -> Result<Vec<LearningSignal>> {
        let signals = self.signals.read().unwrap();
        let task_signals: Vec<_> = signals
            .iter()
            .filter(|s| s.task_id == task_id)
            .cloned()
            .collect();
        Ok(task_signals)
    }

    async fn get_signals_for_judge(&self, judge_id: &JudgeId) -> Result<Vec<LearningSignal>> {
        let signals = self.signals.read().unwrap();
        let judge_signals: Vec<_> = signals
            .iter()
            .filter(|s| s.judge_dissent.iter().any(|d| &d.judge_id == judge_id))
            .cloned()
            .collect();
        Ok(judge_signals)
    }

    async fn get_signals_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<LearningSignal>> {
        let signals = self.signals.read().unwrap();
        let time_filtered: Vec<_> = signals
            .iter()
            .filter(|s| s.timestamp >= start && s.timestamp <= end)
            .cloned()
            .collect();
        Ok(time_filtered)
    }

    async fn get_performance_metrics(
        &self,
        entity_type: PerformanceEntityType,
        entity_id: String,
        time_window: TimeWindow,
    ) -> Result<AggregatedMetrics> {
        let (start_time, end_time, time_range_days) = match time_window {
            TimeWindow::LastHour => (Utc::now() - chrono::Duration::hours(1), Utc::now(), 0),
            TimeWindow::LastDay => (Utc::now() - chrono::Duration::days(1), Utc::now(), 1),
            TimeWindow::LastWeek => (Utc::now() - chrono::Duration::days(7), Utc::now(), 7),
            TimeWindow::LastMonth => (Utc::now() - chrono::Duration::days(30), Utc::now(), 30),
            _ => (Utc::now() - chrono::Duration::days(1), Utc::now(), 1), // Default to last day
        };

        let signals = self.get_signals_by_time_range(start_time, end_time).await?;

        let filtered_signals: Vec<_> = signals
            .into_iter()
            .filter(|s| match &entity_type {
                PerformanceEntityType::Judge(_) => s.judge_dissent.iter().any(|d| d.judge_id.to_string() == entity_id),
                PerformanceEntityType::TaskType(task_type) => task_type == &entity_id,
                PerformanceEntityType::Worker(_) => s.worker_performance.as_ref().map(|w| w.worker_id.to_string() == entity_id).unwrap_or(false),
                PerformanceEntityType::System => true, // Include all signals for system metrics
            })
            .collect();

        if filtered_signals.is_empty() {
            return Ok(AggregatedMetrics {
                total_signals: 0,
                success_rate: 0.0,
                average_quality_score: 0.0,
                average_latency_ms: 0.0,
                dissent_rate: 0.0,
                resource_efficiency: 0.0,
                trends: PerformanceTrends {
                    quality_trend: TrendDirection::Stable,
                    latency_trend: TrendDirection::Stable,
                    dissent_trend: TrendDirection::Stable,
                    resource_efficiency_trend: TrendDirection::Stable,
                },
                time_range_days,
                entity_type: format!("{:?}", entity_type),
                entity_id,
                avg_quality_score: 0.0,
                avg_latency_ms: 0.0,
            });
        }

        let total_signals = filtered_signals.len() as u64;
        let avg_latency_ms = filtered_signals.iter().map(|s| s.latency_ms as f64).sum::<f64>() / total_signals as f64;
        let avg_quality_score = filtered_signals.iter().map(|s| s.quality_score).sum::<f32>() / total_signals as f32;
        let success_rate = filtered_signals.iter()
            .filter(|s| matches!(s.outcome, TaskOutcome::Success { .. }))
            .count() as f32 / total_signals as f32;

        // Calculate resource efficiency (lower resource usage per quality score = better)
        let avg_resource_usage = filtered_signals.iter()
            .map(|s| (s.resource_usage.cpu_percent + s.resource_usage.memory_mb) / 100.0)
            .sum::<f32>() / total_signals as f32;
        let resource_efficiency = if avg_resource_usage > 0.0 {
            avg_quality_score / avg_resource_usage
        } else {
            1.0
        };

        Ok(AggregatedMetrics {
            total_signals,
            success_rate,
            average_quality_score: avg_quality_score,
            average_latency_ms: avg_latency_ms,
            dissent_rate: 0.1, // Placeholder
            resource_efficiency,
            trends: PerformanceTrends {
                quality_trend: TrendDirection::Stable,
                latency_trend: TrendDirection::Stable,
                dissent_trend: TrendDirection::Stable,
                resource_efficiency_trend: TrendDirection::Stable,
            },
            time_range_days,
            entity_type: format!("{:?}", entity_type),
            entity_id,
            avg_quality_score: avg_quality_score,
            avg_latency_ms,
        })
    }

    async fn get_learning_recommendations(&self) -> Result<Vec<LearningRecommendation>> {
        // Return empty recommendations for in-memory storage
        Ok(vec![])
    }

    async fn query_database_for_historical_resource_data(&self, _task_spec: &crate::types::TaskSpec) -> Result<HistoricalResourceData> {
        // Not implemented for in-memory storage
        Err(anyhow::anyhow!("Database queries not supported in in-memory storage"))
    }

    async fn get_cached_historical_resource_data(&self, _task_spec: &crate::types::TaskSpec) -> Result<Option<HistoricalResourceData>> {
        // No caching in in-memory storage
        Ok(None)
    }

    async fn aggregate_historical_resource_data(&self, db_data: &HistoricalResourceData, _cached_data: Option<&HistoricalResourceData>) -> Result<HistoricalResourceData> {
        // Simple aggregation - just return db_data
        Ok(db_data.clone())
    }

    async fn perform_comprehensive_historical_resource_lookup(&self, task_spec: &crate::types::TaskSpec) -> Result<HistoricalResourceData> {
        // Try database first, then cache
        match self.query_database_for_historical_resource_data(task_spec).await {
            Ok(data) => Ok(data),
            Err(_) => {
                match self.get_cached_historical_resource_data(task_spec).await? {
                    Some(data) => Ok(data),
                    None => Err(anyhow::anyhow!("No historical resource data available")),
                }
            }
        }
    }

    async fn monitor_resource_data_performance(&self, _query_time: Duration, _result_count: usize, _cache_hit: bool) -> Result<()> {
        // No-op for in-memory storage
        Ok(())
    }

    async fn analyze_resource_usage_trends(&self, _data: &HistoricalResourceData) -> Result<Vec<ResourceTrend>> {
        // Return empty trends for in-memory storage
        Ok(vec![])
    }

    async fn generate_resource_usage_predictions(&self, _data: &HistoricalResourceData, _trends: &[ResourceTrend]) -> Result<Vec<ResourcePrediction>> {
        // Return empty predictions for in-memory storage
        Ok(vec![])
    }

    fn estimate_task_complexity(&self, _task_spec: &crate::types::TaskSpec) -> TaskComplexity {
        // Return default complexity for in-memory storage
        TaskComplexity::Moderate
    }
}
