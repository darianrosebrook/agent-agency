//! Model Lifecycle Manager
//!
//! Monitors worker performance and triggers hot-swaps when performance degrades
//! below configured thresholds. Integrates with WorkerAssignmentStrategy to
//! monitor performance metrics and DeploymentOrchestrator to perform hot-swaps.
//!
//! @author @darianrosebrook

use anyhow::Result;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::planning::worker_assignment::{WorkerAssignmentStrategy, WorkerPerformance};

#[cfg(feature = "model-management")]
use agent_model_management::deployment::DeploymentOrchestrator;
#[cfg(feature = "model-management")]
use agent_model_management::types::{HotSwapResult, HotSwapStrategy};

#[cfg(not(feature = "model-management"))]
// Placeholder types when model-management feature is disabled
#[derive(Debug, Clone)]
pub enum HotSwapStrategy {
    Immediate,
    Gradual { steps: u32, interval_secs: u64 },
}

#[cfg(not(feature = "model-management"))]
#[derive(Debug)]
pub struct HotSwapResult {
    pub success: bool,
    pub message: String,
}

#[cfg(not(feature = "model-management"))]
pub struct DeploymentOrchestrator;

#[cfg(not(feature = "model-management"))]
impl DeploymentOrchestrator {
    /// TODO: Implement hot-swap of model version for a worker
    ///       This is a stub implementation when model-management feature is disabled.
    ///       Currently returns error; should implement model version hot-swapping.
    ///
    /// COMPLETION CHECKLIST:
    /// [ ] Implement model version hot-swap logic
    /// [ ] Support different hot-swap strategies (rolling, canary, instant)
    /// [ ] Handle model loading and unloading
    /// [ ] Track model version changes
    /// [ ] Add unit tests with various swap scenarios
    /// [ ] Add integration tests with real model management
    ///
    /// ACCEPTANCE CRITERIA:
    /// - Model versions can be hot-swapped without downtime
    /// - Different swap strategies work correctly
    /// - Model loading errors are handled gracefully
    ///
    /// DEPENDENCIES:
    /// - Model management feature (Required)
    ///
    /// ESTIMATED EFFORT: 4-6 hours
    /// PRIORITY: Medium
    /// BLOCKING: No (feature-gated)
    ///
    /// GOVERNANCE:
    /// - CAWS Tier: 2 (model lifecycle management)
    /// - Change Budget: ~150 LOC
    pub async fn perform_hot_swap(
        &self,
        _model_id: &str,
        _new_version: &str,
        _strategy: HotSwapStrategy,
    ) -> Result<HotSwapResult> {
        warn!("DeploymentOrchestrator::perform_hot_swap called but model-management feature is disabled");
        Ok(HotSwapResult {
            success: false,
            message: "Hot-swap not available - model-management feature disabled.".to_string(),
        })
    }
}

/// Model lifecycle manager configuration
#[derive(Debug, Clone)]
pub struct ModelLifecycleConfig {
    /// Performance check interval in seconds
    pub check_interval_secs: u64,

    /// Performance degradation threshold (0.0-1.0)
    /// If performance score drops below this, trigger hot-swap
    pub performance_threshold: f64,

    /// Minimum number of tasks before considering swap
    pub min_tasks_for_evaluation: u64,

    /// Minimum time since last swap before allowing another (seconds)
    pub min_swap_cooldown_secs: u64,

    /// Enable automatic hot-swapping
    pub enable_auto_swap: bool,

    /// Hot-swap strategy to use
    pub swap_strategy: HotSwapStrategy,
}

impl Default for ModelLifecycleConfig {
    fn default() -> Self {
        Self {
            check_interval_secs: 60,      // Check every minute
            performance_threshold: 0.5,   // Swap if performance drops below 50%
            min_tasks_for_evaluation: 10, // Need at least 10 tasks to evaluate
            min_swap_cooldown_secs: 300,  // 5 minutes between swaps
            enable_auto_swap: true,
            swap_strategy: HotSwapStrategy::Gradual {
                steps: 3,
                interval_secs: 10,
            },
        }
    }
}

/// Model lifecycle manager
pub struct ModelLifecycleManager {
    /// Worker assignment strategy (for accessing performance cache)
    worker_assignment: Arc<WorkerAssignmentStrategy>,

    /// Deployment orchestrator (for performing hot-swaps)
    deployment_orchestrator: Option<Arc<DeploymentOrchestrator>>,

    /// Configuration
    config: ModelLifecycleConfig,

    /// Last swap time per worker (worker_id -> timestamp)
    last_swap_times: Arc<RwLock<HashMap<Uuid, chrono::DateTime<Utc>>>>,

    /// Monitoring task handle
    monitoring_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl ModelLifecycleManager {
    /// Create a new model lifecycle manager
    pub fn new(
        worker_assignment: Arc<WorkerAssignmentStrategy>,
        deployment_orchestrator: Option<Arc<DeploymentOrchestrator>>,
        config: ModelLifecycleConfig,
    ) -> Self {
        Self {
            worker_assignment,
            deployment_orchestrator,
            config,
            last_swap_times: Arc::new(RwLock::new(HashMap::new())),
            monitoring_handle: Arc::new(RwLock::new(None)),
        }
    }

    /// Start performance monitoring loop
    pub async fn start_monitoring(&self) -> Result<()> {
        if !self.config.enable_auto_swap {
            info!("Model lifecycle manager: auto-swap disabled, monitoring not started");
            return Ok(());
        }

        if self.deployment_orchestrator.is_none() {
            warn!("Model lifecycle manager: no deployment orchestrator, monitoring not started");
            return Ok(());
        }

        let worker_assignment = self.worker_assignment.clone();
        let deployment_orchestrator = self.deployment_orchestrator.as_ref().unwrap().clone();
        let config = self.config.clone();
        let last_swap_times = self.last_swap_times.clone();

        let handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(config.check_interval_secs));

            loop {
                interval.tick().await;

                if let Err(e) = Self::check_and_swap_models(
                    &worker_assignment,
                    &deployment_orchestrator,
                    &config,
                    &last_swap_times,
                )
                .await
                {
                    error!("Error in model lifecycle monitoring: {}", e);
                }
            }
        });

        *self.monitoring_handle.write().await = Some(handle);
        info!(
            "Model lifecycle manager: monitoring started (interval: {}s)",
            self.config.check_interval_secs
        );

        Ok(())
    }

    /// Stop performance monitoring loop
    pub async fn stop_monitoring(&self) {
        let mut handle_guard = self.monitoring_handle.write().await;
        if let Some(handle) = handle_guard.take() {
            handle.abort();
            info!("Model lifecycle manager: monitoring stopped");
        }
    }

    /// Check worker performance and trigger swaps if needed
    async fn check_and_swap_models(
        worker_assignment: &WorkerAssignmentStrategy,
        deployment_orchestrator: &DeploymentOrchestrator,
        config: &ModelLifecycleConfig,
        last_swap_times: &Arc<RwLock<HashMap<Uuid, chrono::DateTime<Utc>>>>,
    ) -> Result<()> {
        // Get performance cache
        let performance_cache = worker_assignment.get_performance_cache().await?;

        // Check each worker's performance
        for (worker_id, performance) in performance_cache.iter() {
            // Skip if not enough tasks for evaluation
            let total_tasks = performance.tasks_completed + performance.tasks_failed;
            if total_tasks < config.min_tasks_for_evaluation {
                debug!(
                    "Worker {}: insufficient tasks ({}) for evaluation",
                    worker_id, total_tasks
                );
                continue;
            }

            // Check if performance is below threshold
            if performance.performance_score < config.performance_threshold {
                warn!(
                    "Worker {}: performance degraded (score: {:.2}, threshold: {:.2})",
                    worker_id, performance.performance_score, config.performance_threshold
                );

                // Check cooldown period
                let last_swap_times_guard = last_swap_times.read().await;
                if let Some(last_swap) = last_swap_times_guard.get(worker_id) {
                    let time_since_swap = Utc::now().signed_duration_since(*last_swap);
                    if time_since_swap.num_seconds() < config.min_swap_cooldown_secs as i64 {
                        debug!(
                            "Worker {}: still in cooldown period ({}s remaining)",
                            worker_id,
                            config.min_swap_cooldown_secs as i64 - time_since_swap.num_seconds()
                        );
                        continue;
                    }
                }
                drop(last_swap_times_guard);

                // Trigger hot-swap
                info!(
                    "Worker {}: triggering hot-swap due to performance degradation",
                    worker_id
                );

                // Get worker model information from database
                let worker = match worker_assignment.get_worker_by_id(*worker_id).await {
                    Ok(Some(w)) => w,
                    Ok(None) => {
                        warn!(
                            "Worker {}: not found in database, skipping hot-swap",
                            worker_id
                        );
                        continue;
                    }
                    Err(e) => {
                        error!(
                            "Worker {}: failed to retrieve worker from database: {}",
                            worker_id, e
                        );
                        continue;
                    }
                };

                // Use worker's model_name as the model_id for hot-swap
                let current_model_id = worker.model_name.clone();

                // Query for optimal new version/model using PerformanceTracker if available
                let (model_id, new_version) = if let Some(tracker) =
                    worker_assignment.get_performance_tracker()
                {
                    // Get all model performance data sorted by performance score (highest first)
                    match tracker.get_model_performance().await {
                        Ok(performances) => {
                            // Find a better performing model with similar capabilities
                            // For now, use the top-performing model if it's better than current
                            // In the future, this could match by capabilities or model family
                            if let Some(better_model) = performances.first() {
                                if better_model.performance_score > performance.performance_score {
                                    debug!(
                                        "Worker {}: found better performing model {} (score: {:.2} vs {:.2}), swapping to new model",
                                        worker_id,
                                        better_model.model_name,
                                        better_model.performance_score,
                                        performance.performance_score
                                    );
                                    // Swap to the better model entirely (use its name as model_id, "latest" as version)
                                    (better_model.model_name.clone(), "latest".to_string())
                                } else {
                                    // No better model found, use "latest" version of current model
                                    debug!("Worker {}: no better model found, upgrading to latest version", worker_id);
                                    (current_model_id, "latest".to_string())
                                }
                            } else {
                                // No performance data available, use "latest" version of current model
                                debug!("Worker {}: no performance data available, using latest version", worker_id);
                                (current_model_id, "latest".to_string())
                            }
                        }
                        Err(e) => {
                            warn!("Worker {}: failed to query performance tracker: {}, using 'latest'", worker_id, e);
                            (current_model_id, "latest".to_string())
                        }
                    }
                } else {
                    // PerformanceTracker not available, use "latest" version of current model
                    (current_model_id, "latest".to_string())
                };

                debug!(
                    "Worker {}: hot-swapping model {} to version {}",
                    worker_id, model_id, new_version
                );

                match deployment_orchestrator
                    .perform_hot_swap(&model_id, &new_version, config.swap_strategy.clone())
                    .await
                {
                    Ok(result) => {
                        if result.success {
                            info!("Worker {}: hot-swap completed successfully", worker_id);

                            // Update last swap time
                            let mut last_swap_times_guard = last_swap_times.write().await;
                            last_swap_times_guard.insert(*worker_id, Utc::now());
                        } else {
                            warn!(
                                "Worker {}: hot-swap completed but reported failure",
                                worker_id
                            );
                        }
                    }
                    Err(e) => {
                        error!("Worker {}: hot-swap failed: {}", worker_id, e);
                    }
                }
            } else {
                debug!(
                    "Worker {}: performance acceptable (score: {:.2})",
                    worker_id, performance.performance_score
                );
            }
        }

        Ok(())
    }

    /// Manually trigger a hot-swap for a specific worker
    pub async fn trigger_swap(
        &self,
        worker_id: Uuid,
        new_model_version: &str,
    ) -> Result<HotSwapResult> {
        let deployment_orchestrator = self
            .deployment_orchestrator
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Deployment orchestrator not available"))?;

        let model_id = format!("worker-{}", worker_id);

        info!(
            "Manually triggering hot-swap for worker {} to version {}",
            worker_id, new_model_version
        );

        let result = deployment_orchestrator
            .perform_hot_swap(
                &model_id,
                new_model_version,
                self.config.swap_strategy.clone(),
            )
            .await?;

        if result.success {
            // Update last swap time
            let mut last_swap_times = self.last_swap_times.write().await;
            last_swap_times.insert(worker_id, Utc::now());
        }

        Ok(result)
    }

    /// Get performance summary for all workers
    pub async fn get_performance_summary(&self) -> Result<HashMap<Uuid, WorkerPerformance>> {
        self.worker_assignment.get_performance_cache().await
    }
}

impl Drop for ModelLifecycleManager {
    fn drop(&mut self) {
        // Stop monitoring on drop
        let rt = tokio::runtime::Runtime::new().ok();
        if let Some(rt) = rt {
            rt.block_on(self.stop_monitoring());
        }
    }
}
