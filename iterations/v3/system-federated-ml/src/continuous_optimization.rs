//! Continuous Optimization Service - Runtime parameter optimization with performance monitoring
//!
//! Provides continuous parameter optimization with periodic triggers, performance baseline
//! tracking, and quality-preservation mechanisms. Enables automatic tuning of system
//! parameters while maintaining CAWS compliance and performance SLAs.

use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, warn, debug};
use chrono::{DateTime, Utc, Duration};

use crate::bayesian_optimizer::{BayesianOptimizer, OptimizationResult};
use crate::performance_monitor::PerformanceMetrics;

/// Configuration for continuous optimization
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContinuousOptimizationConfig {
    /// Enable continuous optimization
    pub enabled: bool,

    /// Optimization interval (seconds)
    pub optimization_interval_secs: u64,

    /// Minimum decisions before triggering optimization
    pub min_decisions_before_optimization: usize,

    /// Performance degradation threshold (percentage)
    pub performance_degradation_threshold: f64,

    /// Quality preservation threshold (0.0-1.0)
    pub quality_preservation_threshold: f64,

    /// Maximum concurrent optimizations
    pub max_concurrent_optimizations: usize,

    /// Baseline tracking window (seconds)
    pub baseline_tracking_window_secs: u64,
}

impl Default for ContinuousOptimizationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            optimization_interval_secs: 300, // 5 minutes
            min_decisions_before_optimization: 100,
            performance_degradation_threshold: 0.1, // 10% degradation
            quality_preservation_threshold: 0.8, // 80% quality preservation
            max_concurrent_optimizations: 1,
            baseline_tracking_window_secs: 3600, // 1 hour
        }
    }
}

/// Performance baseline for comparison
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PerformanceBaseline {
    /// Baseline timestamp
    pub timestamp: DateTime<Utc>,

    /// Baseline metrics
    pub metrics: PerformanceMetrics,

    /// Number of samples used for baseline
    pub sample_count: usize,

    /// Baseline confidence (0.0-1.0)
    pub confidence: f64,
}

/// Continuous optimization service
#[derive(Debug)]
pub struct ContinuousOptimizationService {
    /// Configuration
    config: ContinuousOptimizationConfig,

    /// Bayesian optimizer
    optimizer: Arc<RwLock<BayesianOptimizer>>,

    /// Current performance baseline
    baseline: Arc<RwLock<Option<PerformanceBaseline>>>,

    /// Performance history for baseline calculation
    performance_history: Arc<RwLock<Vec<(DateTime<Utc>, PerformanceMetrics)>>>,

    /// Active optimization tasks
    active_optimizations: Arc<RwLock<HashMap<String, OptimizationTask>>>,

    /// Optimization command channel
    command_sender: mpsc::UnboundedSender<OptimizationCommand>,

    /// Service running state
    running: Arc<RwLock<bool>>,
}

/// Optimization task state
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OptimizationTask {
    /// Task ID
    pub id: String,

    /// Start timestamp
    pub started_at: DateTime<Utc>,

    /// Current status
    pub status: OptimizationStatus,

    /// Baseline metrics before optimization
    pub baseline_metrics: PerformanceMetrics,

    /// Optimization result (when completed)
    pub result: Option<OptimizationResult>,
}

/// Optimization status
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum OptimizationStatus {
    /// Task is pending
    Pending,

    /// Task is running
    Running,

    /// Task completed successfully
    Completed,

    /// Task failed
    Failed(String),

    /// Task was cancelled
    Cancelled,
}

/// Optimization commands
#[derive(Debug, Clone)]
pub enum OptimizationCommand {
    /// Start optimization cycle
    StartOptimization {
        baseline_metrics: PerformanceMetrics,
    },

    /// Cancel optimization
    CancelOptimization {
        task_id: String,
    },

    /// Get optimization status
    GetStatus {
        response_sender: mpsc::UnboundedSender<ContinuousOptimizationStatus>,
    },
}

/// Service status
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContinuousOptimizationStatus {
    /// Service is running
    pub running: bool,

    /// Current baseline
    pub baseline: Option<PerformanceBaseline>,

    /// Active optimization tasks
    pub active_tasks: Vec<OptimizationTask>,

    /// Total optimizations completed
    pub total_optimizations: usize,

    /// Last optimization timestamp
    pub last_optimization: Option<DateTime<Utc>>,
}

impl ContinuousOptimizationService {
    /// Create new continuous optimization service
    pub fn new(
        config: ContinuousOptimizationConfig,
        optimizer: Arc<RwLock<BayesianOptimizer>>,
    ) -> Self {
        let (command_sender, command_receiver) = mpsc::unbounded_channel();

        let service = Self {
            config,
            optimizer,
            baseline: Arc::new(RwLock::new(None)),
            performance_history: Arc::new(RwLock::new(Vec::new())),
            active_optimizations: Arc::new(RwLock::new(HashMap::new())),
            command_sender,
            running: Arc::new(RwLock::new(false)),
        };

        // Start the service loop
        tokio::spawn(Self::service_loop(
            Arc::clone(&service.running),
            Arc::clone(&service.baseline),
            Arc::clone(&service.performance_history),
            Arc::clone(&service.active_optimizations),
            Arc::clone(&service.optimizer),
            command_receiver,
            service.config.clone(),
        ));

        service
    }

    /// Start the service
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(()); // Already running
        }
        *running = true;
        info!("Continuous optimization service started");
        Ok(())
    }

    /// Stop the service
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;
        info!("Continuous optimization service stopped");
        Ok(())
    }

    /// Update performance metrics
    pub async fn update_performance(&self, metrics: PerformanceMetrics) -> Result<()> {
        if !*self.running.read().await {
            return Ok(()); // Service not running
        }

        let timestamp = Utc::now();

        // Add to performance history
        {
            let mut history = self.performance_history.write().await;
            history.push((timestamp, metrics.clone()));

            // Trim history to baseline window
            let cutoff = timestamp - Duration::seconds(self.config.baseline_tracking_window_secs as i64);
            history.retain(|(ts, _)| *ts > cutoff);
        }

        // Update baseline if needed
        self.update_baseline_if_needed().await;

        // Check if optimization should be triggered
        self.check_optimization_triggers(metrics).await?;

        Ok(())
    }

    /// Get current status
    pub async fn get_status(&self) -> Result<ContinuousOptimizationStatus> {
        let (response_sender, mut response_receiver) = mpsc::unbounded_channel();

        self.command_sender.send(OptimizationCommand::GetStatus { response_sender })?;

        match response_receiver.recv().await {
            Some(status) => Ok(status),
            None => Err(anyhow::anyhow!("Failed to get status response")),
        }
    }

    /// Force optimization cycle
    pub async fn force_optimization(&self) -> Result<String> {
        let baseline_metrics = {
            let baseline = self.baseline.read().await;
            match baseline.as_ref() {
                Some(b) => b.metrics.clone(),
                None => return Err(anyhow::anyhow!("No performance baseline available")),
            }
        };

        let task_id = format!("forced-{}", Utc::now().timestamp());
        self.command_sender.send(OptimizationCommand::StartOptimization { baseline_metrics })?;

        Ok(task_id)
    }

    /// Update baseline if needed
    async fn update_baseline_if_needed(&self) {
        let history = self.performance_history.read().await;
        if history.len() < 10 { // Need minimum samples for stable baseline
            return;
        }

        let mut baseline = self.baseline.write().await;
        let should_update = match baseline.as_ref() {
            Some(b) => {
                // Update baseline if it's older than half the tracking window
                let age = Utc::now().signed_duration_since(b.timestamp);
                age.num_seconds() > (self.config.baseline_tracking_window_secs / 2) as i64
            }
            None => true, // No baseline yet
        };

        if should_update {
            // Calculate new baseline from recent history
            let recent_history: Vec<_> = history.iter()
                .filter(|(ts, _)| {
                    let age = Utc::now().signed_duration_since(*ts);
                    age.num_seconds() < 300 // Last 5 minutes
                })
                .collect();

            if recent_history.len() >= 5 {
                let avg_metrics = Self::calculate_average_metrics(&recent_history.iter().map(|(ts, m)| (*ts, m.clone())).collect::<Vec<_>>());
                *baseline = Some(PerformanceBaseline {
                    timestamp: Utc::now(),
                    metrics: avg_metrics,
                    sample_count: recent_history.len(),
                    confidence: (recent_history.len() as f64 / 10.0).min(1.0), // Simple confidence based on sample count
                });

                debug!("Updated performance baseline with {} samples", recent_history.len());
            }
        }
    }

    /// Check if optimization should be triggered
    async fn check_optimization_triggers(&self, current_metrics: PerformanceMetrics) -> Result<()> {
        let baseline = self.baseline.read().await;
        let Some(baseline) = baseline.as_ref() else {
            return Ok(()); // No baseline to compare against
        };

        // Check performance degradation
        let degradation = Self::calculate_performance_degradation(&baseline.metrics, &current_metrics);
        if degradation > self.config.performance_degradation_threshold {
            info!("Performance degradation detected: {:.2}%, triggering optimization", degradation * 100.0);
            self.command_sender.send(OptimizationCommand::StartOptimization {
                baseline_metrics: baseline.metrics.clone(),
            })?;
        }

        Ok(())
    }

    /// Calculate average metrics from history
    fn calculate_average_metrics(history: &[(DateTime<Utc>, PerformanceMetrics)]) -> PerformanceMetrics {
        let mut total_latency = 0.0;
        let mut total_throughput = 0.0;
        let mut total_p95_latency = 0.0;
        let mut total_error_rate = 0.0;
        let mut count = 0;

        for (_, metrics) in history {
            total_latency += metrics.avg_latency_ms;
            total_throughput += metrics.throughput;
            total_p95_latency += metrics.p95_latency_ms;
            total_error_rate += metrics.error_rate;
            count += 1;
        }

        if count == 0 {
            return PerformanceMetrics::default();
        }

        PerformanceMetrics {
            throughput: total_throughput / count as f64,
            avg_latency_ms: total_latency / count as f64,
            p95_latency_ms: total_p95_latency / count as f64,
            p99_latency_ms: 0.0, // Not averaging this for simplicity
            error_rate: total_error_rate / count as f64,
            cpu_usage_percent: 0.0, // Not tracked in history
            memory_usage_percent: 0.0, // Not tracked in history
            active_connections: 0, // Not tracked in history
            queue_depth: 0, // Not tracked in history
            timestamp: Utc::now(),
        }
    }

    /// Calculate performance degradation
    fn calculate_performance_degradation(baseline: &PerformanceMetrics, current: &PerformanceMetrics) -> f64 {
        // Calculate weighted degradation across multiple metrics
        let latency_degradation = (current.avg_latency_ms - baseline.avg_latency_ms) / baseline.avg_latency_ms;
        let throughput_degradation = (baseline.throughput - current.throughput) / baseline.throughput;
        let p95_latency_degradation = (current.p95_latency_ms - baseline.p95_latency_ms) / baseline.p95_latency_ms;
        let error_rate_increase = current.error_rate - baseline.error_rate;

        // Weighted average (latency and throughput are more important for performance)
        (latency_degradation * 0.3 + throughput_degradation * 0.3 + p95_latency_degradation * 0.3 + error_rate_increase * 0.1).max(0.0)
    }

    /// Service loop handling optimization commands
    async fn service_loop(
        running: Arc<RwLock<bool>>,
        baseline: Arc<RwLock<Option<PerformanceBaseline>>>,
        performance_history: Arc<RwLock<Vec<(DateTime<Utc>, PerformanceMetrics)>>>,
        active_optimizations: Arc<RwLock<HashMap<String, OptimizationTask>>>,
        optimizer: Arc<RwLock<BayesianOptimizer>>,
        mut command_receiver: mpsc::UnboundedReceiver<OptimizationCommand>,
        config: ContinuousOptimizationConfig,
    ) {
        info!("Continuous optimization service loop started");

        while *running.read().await {
            tokio::select! {
                Some(command) = command_receiver.recv() => {
                    Self::handle_command(
                        &baseline,
                        &active_optimizations,
                        &optimizer,
                        command,
                        &config,
                    ).await;
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(60)) => {
                    // Periodic cleanup and maintenance
                    Self::periodic_maintenance(&active_optimizations).await;
                }
            }
        }

        info!("Continuous optimization service loop stopped");
    }

    /// Handle optimization commands
    async fn handle_command(
        baseline: &Arc<RwLock<Option<PerformanceBaseline>>>,
        active_optimizations: &Arc<RwLock<HashMap<String, OptimizationTask>>>,
        optimizer: &Arc<RwLock<BayesianOptimizer>>,
        command: OptimizationCommand,
        config: &ContinuousOptimizationConfig,
    ) {
        match command {
            OptimizationCommand::StartOptimization { baseline_metrics } => {
                Self::start_optimization_task(
                    active_optimizations,
                    optimizer,
                    baseline_metrics,
                    config,
                ).await;
            }
            OptimizationCommand::CancelOptimization { task_id } => {
                Self::cancel_optimization_task(active_optimizations, &task_id).await;
            }
            OptimizationCommand::GetStatus { response_sender } => {
                let status = Self::build_status(
                    baseline,
                    active_optimizations,
                    config,
                ).await;

                let _ = response_sender.send(status);
            }
        }
    }

    /// Start optimization task
    async fn start_optimization_task(
        active_optimizations: &Arc<RwLock<HashMap<String, OptimizationTask>>>,
        optimizer: &Arc<RwLock<BayesianOptimizer>>,
        baseline_metrics: PerformanceMetrics,
        config: &ContinuousOptimizationConfig,
    ) {
        let active_count = active_optimizations.read().await.len();
        if active_count >= config.max_concurrent_optimizations {
            warn!("Maximum concurrent optimizations reached ({}), skipping", active_count);
            return;
        }

        let task_id = format!("opt-{}", Utc::now().timestamp_nanos_opt().unwrap_or(0));

        let task = OptimizationTask {
            id: task_id.clone(),
            started_at: Utc::now(),
            status: OptimizationStatus::Pending,
            baseline_metrics: baseline_metrics.clone(),
            result: None,
        };

        active_optimizations.write().await.insert(task_id.clone(), task);

        // Spawn optimization task
        let active_optimizations_clone = Arc::clone(active_optimizations);
        let optimizer_clone = Arc::clone(optimizer);
        let baseline_metrics_clone = baseline_metrics.clone();

        tokio::spawn(async move {
            Self::run_optimization_task(
                task_id,
                baseline_metrics_clone,
                active_optimizations_clone,
                optimizer_clone,
            ).await;
        });
    }

    /// Run optimization task
    async fn run_optimization_task(
        task_id: String,
        baseline_metrics: PerformanceMetrics,
        active_optimizations: Arc<RwLock<HashMap<String, OptimizationTask>>>,
        optimizer: Arc<RwLock<BayesianOptimizer>>,
    ) {
        // Update task status to running
        {
            let mut tasks = active_optimizations.write().await;
            if let Some(task) = tasks.get_mut(&task_id) {
                task.status = OptimizationStatus::Running;
            }
        }

        info!("Starting optimization task {}", task_id);

        // Run optimization
        let result = {
            let mut opt = optimizer.write().await;
            match opt.optimize_parameters(&baseline_metrics).await {
                Ok(result) => {
                    info!("Optimization task {} completed: improvement={:.2}%, confidence={:.2}%",
                         task_id, result.expected_improvement * 100.0, result.confidence * 100.0);
                    Ok(result)
                }
                Err(e) => {
                    warn!("Optimization task {} failed: {}", task_id, e);
                    Err(e)
                }
            }
        };

        // Update task with result
        let mut tasks = active_optimizations.write().await;
        if let Some(task) = tasks.get_mut(&task_id) {
            match &result {
                Ok(optimization_result) => {
                    task.status = OptimizationStatus::Completed;
                    task.result = Some(optimization_result.clone());
                }
                Err(e) => {
                    task.status = OptimizationStatus::Failed(e.to_string());
                }
            }
        }

        // Update task status
        let mut tasks = active_optimizations.write().await;
        if let Some(task) = tasks.get_mut(&task_id) {
            match result {
                Ok(optimization_result) => {
                    task.status = OptimizationStatus::Completed;
                    task.result = Some(optimization_result);
                }
                Err(e) => {
                    task.status = OptimizationStatus::Failed(e.to_string());
                }
            }
        }
    }

    /// Cancel optimization task
    async fn cancel_optimization_task(
        active_optimizations: &Arc<RwLock<HashMap<String, OptimizationTask>>>,
        task_id: &str,
    ) {
        let mut tasks = active_optimizations.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
            task.status = OptimizationStatus::Cancelled;
            info!("Cancelled optimization task {}", task_id);
        }
    }

    /// Build service status
    async fn build_status(
        baseline: &Arc<RwLock<Option<PerformanceBaseline>>>,
        active_optimizations: &Arc<RwLock<HashMap<String, OptimizationTask>>>,
        config: &ContinuousOptimizationConfig,
    ) -> ContinuousOptimizationStatus {
        let baseline_data = baseline.read().await.clone();
        let active_tasks: Vec<_> = active_optimizations.read().await.values().cloned().collect();

        // Count total completed optimizations
        let total_optimizations = active_tasks.iter()
            .filter(|t| matches!(t.status, OptimizationStatus::Completed))
            .count();

        // Find last optimization time
        let last_optimization = active_tasks.iter()
            .filter(|t| matches!(t.status, OptimizationStatus::Completed))
            .map(|t| t.started_at)
            .max();

        ContinuousOptimizationStatus {
            running: config.enabled,
            baseline: baseline_data,
            active_tasks,
            total_optimizations,
            last_optimization,
        }
    }

    /// Periodic maintenance
    async fn periodic_maintenance(active_optimizations: &Arc<RwLock<HashMap<String, OptimizationTask>>>) {
        let mut tasks = active_optimizations.write().await;

        // Remove old completed tasks (keep only recent ones)
        let cutoff = Utc::now() - Duration::hours(1);
        tasks.retain(|_, task| {
            match &task.status {
                OptimizationStatus::Completed | OptimizationStatus::Failed(_) => {
                    task.started_at > cutoff
                }
                _ => true, // Keep active and pending tasks
            }
        });

        debug!("Completed periodic maintenance, {} active tasks remaining", tasks.len());
    }
}

impl Default for ContinuousOptimizationService {
    fn default() -> Self {
        let config = ContinuousOptimizationConfig::default();
        let optimizer = Arc::new(RwLock::new(BayesianOptimizer::new(Default::default()).unwrap()));

        Self::new(config, optimizer)
    }
}
