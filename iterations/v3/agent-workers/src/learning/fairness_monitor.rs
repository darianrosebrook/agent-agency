//! Fairness monitor for tracking worker utilization fairness

use std::sync::Arc;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use anyhow::Result;

use crate::parallel_types::{WorkerId, TaskId};
use crate::learning::types::*;
use crate::worker_types::{ExecutionOutcome, LearningMode};

/// Monitors fairness in worker utilization and task distribution
pub struct FairnessMonitor {
    task_assignments: Arc<tokio::sync::RwLock<HashMap<WorkerId, Vec<TaskId>>>>,
    last_updated: Arc<tokio::sync::RwLock<DateTime<Utc>>>,
}

impl FairnessMonitor {
    pub fn new() -> Self {
        Self {
            task_assignments: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            last_updated: Arc::new(tokio::sync::RwLock::new(Utc::now())),
        }
    }

    /// Record a task assignment to a worker
    pub async fn record_task_assignment(&self, worker_id: WorkerId, task_id: TaskId) -> anyhow::Result<()> {
        let mut assignments = self.task_assignments.write().await;
        assignments.entry(worker_id).or_default().push(task_id);
        
        let mut last_updated = self.last_updated.write().await;
        *last_updated = Utc::now();
        
        Ok(())
    }

    /// Get fairness metrics
    pub async fn get_fairness_metrics(&self) -> Result<FairnessMetrics, Box<dyn std::error::Error + Send + Sync>> {
        let assignments = self.task_assignments.read().await;
        let last_updated = *self.last_updated.read().await;

        let mut worker_utilization = HashMap::new();
        let mut task_distribution = HashMap::new();
        let mut total_tasks = 0;

        // Calculate utilization for each worker
        for (worker_id, tasks) in assignments.iter() {
            let task_count = tasks.len() as u64;
            task_distribution.insert(*worker_id, task_count);
            total_tasks += task_count;
        }

        // Calculate utilization percentages
        if total_tasks > 0 {
            for (worker_id, task_count) in &task_distribution {
                let utilization = *task_count as f64 / total_tasks as f64;
                worker_utilization.insert(*worker_id, utilization);
            }
        }

        // Calculate Gini coefficient for inequality measurement
        let gini_coefficient = self.calculate_gini_coefficient(&worker_utilization);

        Ok(FairnessMetrics {
            gini_coefficient,
            worker_utilization,
            task_distribution,
            last_updated,
        })
    }

    /// Calculate Gini coefficient for inequality measurement
    fn calculate_gini_coefficient(&self, utilizations: &HashMap<WorkerId, f64>) -> f64 {
        if utilizations.is_empty() {
            return 0.0;
        }

        let mut values: Vec<f64> = utilizations.values().cloned().collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let n = values.len();
        let sum: f64 = values.iter().sum();
        
        if sum == 0.0 {
            return 0.0;
        }

        let mut gini = 0.0;
        for (i, value) in values.iter().enumerate() {
            gini += (2.0 * (i as f64 + 1.0) - n as f64 - 1.0) * value;
        }

        gini / (n as f64 * sum)
    }

    /// Check if worker utilization is fair
    pub async fn is_utilization_fair(&self, threshold: f64) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let metrics = self.get_fairness_metrics().await?;
        Ok(metrics.gini_coefficient <= threshold)
    }

    /// Get worker with lowest utilization
    pub async fn get_least_utilized_worker(&self) -> Result<Option<WorkerId>, Box<dyn std::error::Error + Send + Sync>> {
        let metrics = self.get_fairness_metrics().await?;
        
        let mut least_utilized = None;
        let mut min_utilization = 1.0;

        for (worker_id, utilization) in &metrics.worker_utilization {
            if *utilization < min_utilization {
                min_utilization = *utilization;
                least_utilized = Some(*worker_id);
            }
        }

        Ok(least_utilized)
    }

    /// Get worker with highest utilization
    pub async fn get_most_utilized_worker(&self) -> Result<Option<WorkerId>, Box<dyn std::error::Error + Send + Sync>> {
        let metrics = self.get_fairness_metrics().await?;
        
        let mut most_utilized = None;
        let mut max_utilization = 0.0;

        for (worker_id, utilization) in &metrics.worker_utilization {
            if *utilization > max_utilization {
                max_utilization = *utilization;
                most_utilized = Some(*worker_id);
            }
        }

        Ok(most_utilized)
    }

    /// Reset fairness tracking
    pub async fn reset(&self) -> anyhow::Result<()> {
        let mut assignments = self.task_assignments.write().await;
        assignments.clear();
        
        let mut last_updated = self.last_updated.write().await;
        *last_updated = Utc::now();
        
        Ok(())
    }
}

impl Default for FairnessMonitor {
    fn default() -> Self {
        Self::new()
    }
}
