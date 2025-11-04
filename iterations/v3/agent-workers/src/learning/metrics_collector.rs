//! Parallel worker metrics collector

use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::parallel_types::{TaskId, WorkerId};
use crate::learning::types::*;
use crate::worker_types::{ExecutionOutcome, LearningMode};

/// Collects and analyzes metrics from parallel worker execution
pub struct ParallelWorkerMetricsCollector {
    reward_weights: RewardWeights,
    baseline: Baseline,
    execution_records: Arc<tokio::sync::RwLock<Vec<ExecutionRecord>>>,
    worker_profiles: Arc<tokio::sync::RwLock<HashMap<WorkerId, WorkerPerformanceProfile>>>,
}

impl ParallelWorkerMetricsCollector {
    pub fn new(reward_weights: RewardWeights, baseline: Baseline) -> Self {
        Self {
            reward_weights,
            baseline,
            execution_records: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            worker_profiles: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Record a task execution
    pub async fn record_execution(
        &self,
        task_id: TaskId,
        worker_id: WorkerId,
        execution_time_ms: u64,
        success: bool,
        quality_score: f64,
        error_message: Option<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let record = ExecutionRecord {
            id: Uuid::new_v4(),
            task_id,
            worker_id,
            execution_time_ms,
            success,
            quality_score,
            error_message,
            metadata,
            created_at: Utc::now(),
        };

        // Store the execution record
        {
            let mut records = self.execution_records.write().await;
            records.push(record.clone());
        }

        // Update worker profile
        self.update_worker_profile(worker_id, &record).await?;

        Ok(())
    }

    /// Update worker performance profile
    async fn update_worker_profile(
        &self,
        worker_id: WorkerId,
        record: &ExecutionRecord,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut profiles = self.worker_profiles.write().await;
        
        let profile = profiles.entry(worker_id).or_insert_with(|| WorkerPerformanceProfile {
            worker_id,
            specialty: crate::parallel_types::WorkerSpecialty::General, // Default specialty
            total_executions: 0,
            successful_executions: 0,
            average_execution_time_ms: 0.0,
            average_quality_score: 0.0,
            last_updated: Utc::now(),
            performance_trend: PerformanceTrend::Unknown,
            capability_scores: HashMap::new(),
        });

        // Update profile statistics
        profile.total_executions += 1;
        if record.success {
            profile.successful_executions += 1;
        }

        // Update running averages
        let total = profile.total_executions as f64;
        profile.average_execution_time_ms = 
            (profile.average_execution_time_ms * (total - 1.0) + record.execution_time_ms as f64) / total;
        profile.average_quality_score = 
            (profile.average_quality_score * (total - 1.0) + record.quality_score) / total;

        profile.last_updated = Utc::now();

        Ok(())
    }

    /// Calculate reward score for an execution
    pub fn calculate_reward(&self, record: &ExecutionRecord) -> f64 {
        let quality_reward = record.quality_score * self.reward_weights.quality;
        let latency_reward = if record.execution_time_ms <= self.baseline.p50_ms as u64 {
            self.reward_weights.latency
        } else {
            self.reward_weights.latency * (self.baseline.p50_ms / record.execution_time_ms as f64)
        };
        let rework_reward = if record.success {
            self.reward_weights.rework
        } else {
            0.0
        };
        let cost_reward = self.reward_weights.cost; // Simplified cost calculation

        quality_reward + latency_reward + rework_reward + cost_reward
    }

    /// Get performance statistics
    pub async fn get_performance_stats(&self) -> Result<PerformanceStats, Box<dyn std::error::Error + Send + Sync>> {
        let records = self.execution_records.read().await;
        let profiles = self.worker_profiles.read().await;

        let total_executions = records.len();
        let successful_executions = records.iter().filter(|r| r.success).count();
        let success_rate = if total_executions > 0 {
            successful_executions as f64 / total_executions as f64
        } else {
            0.0
        };

        let avg_execution_time = if total_executions > 0 {
            records.iter().map(|r| r.execution_time_ms).sum::<u64>() as f64 / total_executions as f64
        } else {
            0.0
        };

        let avg_quality_score = if total_executions > 0 {
            records.iter().map(|r| r.quality_score).sum::<f64>() / total_executions as f64
        } else {
            0.0
        };

        Ok(PerformanceStats {
            total_executions,
            successful_executions,
            success_rate,
            average_execution_time_ms: avg_execution_time,
            average_quality_score: avg_quality_score,
            active_workers: profiles.len(),
            last_updated: Utc::now(),
        })
    }

    /// Get worker performance profiles
    pub async fn get_worker_profiles(&self) -> Result<HashMap<WorkerId, WorkerPerformanceProfile>, Box<dyn std::error::Error + Send + Sync>> {
        let profiles = self.worker_profiles.read().await;
        Ok(profiles.clone())
    }

    /// Get execution records
    pub async fn get_execution_records(&self, limit: Option<usize>) -> Result<Vec<ExecutionRecord>, Box<dyn std::error::Error + Send + Sync>> {
        let records = self.execution_records.read().await;
        let mut result = records.clone();
        
        if let Some(limit) = limit {
            result.truncate(limit);
        }
        
        Ok(result)
    }
}

/// Performance statistics

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct PerformanceStats {
    pub total_executions: usize,
    pub successful_executions: usize,
    pub success_rate: f64,
    pub average_execution_time_ms: f64,
    pub average_quality_score: f64,
    pub active_workers: usize,
    #[schemars(with = "String")]

    pub last_updated: DateTime<Utc>,
}
