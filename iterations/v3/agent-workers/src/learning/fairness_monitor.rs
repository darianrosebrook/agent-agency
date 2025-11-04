//! Fairness monitor for tracking worker utilization fairness

use std::sync::Arc;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use anyhow::Result;

use crate::{WorkerId, TaskId};
use crate::learning::types::*;
use crate::worker_types::{ExecutionOutcome, LearningMode};
use data_infrastructure::client::DatabaseClient;

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

/// Real implementation using database tracking
pub struct RealFairnessMonitor {
    db_client: Arc<DatabaseClient>,
}

impl RealFairnessMonitor {
    pub fn new(db_client: Arc<DatabaseClient>) -> Self {
        Self { db_client }
    }
}

#[async_trait::async_trait]
impl crate::learning::adaptive_selector::FairnessMonitor for RealFairnessMonitor {
    async fn get_fairness_metrics(&self) -> Result<FairnessMetrics, Box<dyn std::error::Error + Send + Sync>> {
        // Query worker utilization from database
        let query = r#"
            SELECT
                w.id as worker_id,
                w.name as worker_name,
                COUNT(t.id) as task_count,
                AVG(EXTRACT(EPOCH FROM (t.completed_at - t.started_at))) as avg_duration_seconds
            FROM workers w
            LEFT JOIN task_executions t ON w.id = t.worker_id
                AND t.status = 'completed'
                AND t.completed_at >= NOW() - INTERVAL '24 hours'
            GROUP BY w.id, w.name
            ORDER BY task_count DESC
        "#;

        match self.db_client.query(query, &[]).await {
            Ok(rows) => {
                let mut worker_utilization = HashMap::new();
                let mut task_distribution = HashMap::new();
                let mut total_tasks = 0;

                for row in rows {
                    let worker_id: String = row.get("worker_id");
                    let worker_name: String = row.get("worker_name");
                    let task_count: i64 = row.get("task_count");
                    let avg_duration: Option<f64> = row.get("avg_duration_seconds");

                    worker_utilization.insert(worker_id.clone(), task_count as f64);
                    task_distribution.insert(worker_name, task_count as f64);
                    total_tasks += task_count;
                }

                // Calculate Gini coefficient for fairness
                let mut utilization_values: Vec<f64> = worker_utilization.values().cloned().collect();
                utilization_values.sort_by(|a, b| a.partial_cmp(b).unwrap());

                let gini_coefficient = if utilization_values.is_empty() {
                    0.0
                } else {
                    let n = utilization_values.len() as f64;
                    let sum: f64 = utilization_values.iter().sum();
                    let mut gini = 0.0;

                    for (i, value) in utilization_values.iter().enumerate() {
                        gini += (2.0 * (i as f64 + 1.0) - n - 1.0) * value;
                    }

                    gini / (n * sum)
                };

                Ok(FairnessMetrics {
                    gini_coefficient,
                    worker_utilization,
                    task_distribution,
                    last_updated: chrono::Utc::now(),
                })
            }
            Err(e) => {
                tracing::error!("Failed to get fairness metrics: {}", e);
                // Return default metrics on error
                Ok(FairnessMetrics {
                    gini_coefficient: 0.5, // Medium inequality
                    worker_utilization: HashMap::new(),
                    task_distribution: HashMap::new(),
                    last_updated: chrono::Utc::now(),
                })
            }
        }
    }

    async fn record_task_assignment(&self, worker_id: WorkerId, task_id: crate::worker_types::TaskId) -> Result<()> {
        // Record task assignment in database for fairness tracking
        let query = r#"
            INSERT INTO worker_task_assignments (worker_id, task_id, assigned_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (worker_id, task_id) DO NOTHING
        "#;

        let now = chrono::Utc::now();
        match self.db_client.execute(query, &[&worker_id.0.to_string(), &task_id.0.to_string(), &now]).await {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::error!("Failed to record task assignment: {}", e);
                Err(e.into())
            }
        }
    }
}
