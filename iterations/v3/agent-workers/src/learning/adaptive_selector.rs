//! Adaptive worker selector

use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::collections::HashMap;
use anyhow::Result;
use tracing::error;

use crate::worker_types::TaskId;
use crate::{WorkerId, WorkerSpecialty, SubTask};
use crate::learning::types::*;
use crate::learning::PatternAnalyzer;
use crate::worker_types::{ExecutionOutcome, LearningMode};

/// Strategy for worker selection

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum WorkerSelectionStrategy {
    /// Select based on performance history
    PerformanceBased,
    /// Select based on capability matching
    CapabilityBased,
    /// Select based on fairness
    FairnessBased,
    /// Select based on load balancing
    LoadBalanced,
    /// Select randomly
    Random,
}

/// Adaptive worker selector that learns from execution history
pub struct AdaptiveWorkerSelector {
    strategy: WorkerSelectionStrategy,
    worker_profiles: Arc<tokio::sync::RwLock<HashMap<WorkerId, WorkerPerformanceProfile>>>,
    fairness_monitor: Arc<dyn FairnessMonitor>,
    pattern_analyzer: Arc<PatternAnalyzer>,
}

impl AdaptiveWorkerSelector {
    pub fn new(
        strategy: WorkerSelectionStrategy,
        fairness_monitor: Arc<dyn FairnessMonitor>,
        pattern_analyzer: Arc<PatternAnalyzer>,
    ) -> Self {
        Self {
            strategy,
            worker_profiles: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            fairness_monitor,
            pattern_analyzer,
        }
    }

    /// Select the best worker for a subtask
    pub async fn select_worker(
        &self,
        subtask: &SubTask,
        available_workers: &[WorkerId],
    ) -> Result<Option<WorkerId>, Box<dyn std::error::Error + Send + Sync>> {
        if available_workers.is_empty() {
            return Ok(None);
        }

        let selection = match self.strategy {
            WorkerSelectionStrategy::PerformanceBased => {
                self.select_by_performance(subtask, available_workers).await?
            }
            WorkerSelectionStrategy::CapabilityBased => {
                self.select_by_capability(subtask, available_workers).await?
            }
            WorkerSelectionStrategy::FairnessBased => {
                self.select_by_fairness(subtask, available_workers).await?
            }
            WorkerSelectionStrategy::LoadBalanced => {
                self.select_by_load_balance(subtask, available_workers).await?
            }
            WorkerSelectionStrategy::Random => {
                self.select_randomly(available_workers)
            }
        };

        Ok(selection)
    }

    /// Select worker based on performance history
    async fn select_by_performance(
        &self,
        subtask: &SubTask,
        available_workers: &[WorkerId],
    ) -> Result<Option<WorkerId>, Box<dyn std::error::Error + Send + Sync>> {
        let profiles = self.worker_profiles.read().await;
        
        let mut best_worker = None;
        let mut best_score = 0.0;

        for worker_id in available_workers {
            if let Some(profile) = profiles.get(worker_id) {
                let score = self.calculate_performance_score(profile, subtask);
                if score > best_score {
                    best_score = score;
                    best_worker = Some(*worker_id);
                }
            }
        }

        Ok(best_worker)
    }

    /// Select worker based on capability matching
    async fn select_by_capability(
        &self,
        subtask: &SubTask,
        available_workers: &[WorkerId],
    ) -> Result<Option<WorkerId>, Box<dyn std::error::Error + Send + Sync>> {
        let profiles = self.worker_profiles.read().await;
        
        let mut best_worker = None;
        let mut best_match_score = 0.0;

        for worker_id in available_workers {
            if let Some(profile) = profiles.get(worker_id) {
                let match_score = self.calculate_capability_match(profile, subtask);
                if match_score > best_match_score {
                    best_match_score = match_score;
                    best_worker = Some(*worker_id);
                }
            }
        }

        Ok(best_worker)
    }

    /// Select worker based on fairness
    async fn select_by_fairness(
        &self,
        subtask: &SubTask,
        available_workers: &[WorkerId],
    ) -> Result<Option<WorkerId>, Box<dyn std::error::Error + Send + Sync>> {
        let fairness_metrics = self.fairness_monitor.get_fairness_metrics().await?;
        
        let mut best_worker = None;
        let mut lowest_utilization = 1.0;

        for worker_id in available_workers {
            if let Some(utilization) = fairness_metrics.worker_utilization.get(worker_id) {
                if *utilization < lowest_utilization {
                    lowest_utilization = *utilization;
                    best_worker = Some(*worker_id);
                }
            }
        }

        Ok(best_worker)
    }

    /// Select worker based on load balancing
    async fn select_by_load_balance(
        &self,
        subtask: &SubTask,
        available_workers: &[WorkerId],
    ) -> Result<Option<WorkerId>, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement real load balancing selection
        // - [ ] Query worker load metrics (CPU, memory, active tasks)
        // - [ ] Calculate load scores for each available worker
        // - [ ] Select worker with lowest load score
        // - [ ] Handle workers with similar load scores
        // - [ ] Add unit tests with mock worker load data
        // - [ ] Add integration tests with real worker load balancing
        // For now, use fairness-based selection as a proxy for load balancing
        self.select_by_fairness(subtask, available_workers).await
    }

    /// Select worker randomly
    fn select_randomly(&self, available_workers: &[WorkerId]) -> Option<WorkerId> {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        available_workers.choose(&mut rng).copied()
    }

    /// Calculate performance score for a worker
    fn calculate_performance_score(&self, profile: &WorkerPerformanceProfile, subtask: &SubTask) -> f64 {
        let success_rate = if profile.total_executions > 0 {
            profile.successful_executions as f64 / profile.total_executions as f64
        } else {
            0.5 // Default score for new workers
        };

        let quality_score = profile.average_quality_score;
        let speed_score = if profile.average_execution_time_ms > 0.0 {
            1.0 / (profile.average_execution_time_ms / 1000.0) // Convert to seconds and invert
        } else {
            0.5 // Default score
        };

        // Weighted combination of metrics
        0.4 * success_rate + 0.4 * quality_score + 0.2 * speed_score
    }

    /// Calculate capability match score
    fn calculate_capability_match(&self, profile: &WorkerPerformanceProfile, subtask: &SubTask) -> f64 {
        // Check if worker has the required specialty capability
        let specialty_str = format!("{:?}", subtask.specialty);

        if let Some(score) = profile.capability_scores.get(&specialty_str) {
            *score
        } else {
            0.5 // Default score for workers without this specialty
        }
    }

    /// Update worker profiles
    pub async fn update_worker_profiles(&self, profiles: HashMap<WorkerId, WorkerPerformanceProfile>) -> Result<()> {
        let mut current_profiles = self.worker_profiles.write().await;
        for (worker_id, profile) in profiles {
            current_profiles.insert(worker_id, profile);
        }
        Ok(())
    }

    /// Get selection strategy
    pub fn get_strategy(&self) -> &WorkerSelectionStrategy {
        &self.strategy
    }

    /// Set selection strategy
    pub fn set_strategy(&mut self, strategy: WorkerSelectionStrategy) {
        self.strategy = strategy;
    }
}

/// Trait for fairness monitoring
#[async_trait::async_trait]
pub trait FairnessMonitor: Send + Sync {
    async fn get_fairness_metrics(&self) -> Result<FairnessMetrics, Box<dyn std::error::Error + Send + Sync>>;
    async fn record_task_assignment(&self, worker_id: WorkerId, task_id: crate::worker_types::TaskId) -> Result<()>;
}

// RealFairnessMonitor implementation moved to fairness_monitor.rs
