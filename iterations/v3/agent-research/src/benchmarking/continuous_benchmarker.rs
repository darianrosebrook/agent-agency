//! Continuous benchmarker orchestrating automated benchmark execution
//!
//! Coordinates benchmark scheduling, dataset management, and execution
//! with integration to ModelPerformanceTracker for storing results.
//!
//! @author @darianrosebrook

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration as TokioDuration};
use tracing::{debug, info, warn, error};
use uuid::Uuid;

use crate::benchmark_types::{
    BenchmarkReport, BenchmarkResult, BenchmarkType, ModelSpecification, BenchmarkMetrics,
    PerformanceSummary, PerformanceTrend, RegressionAlert, ModelRecommendation,
};
use crate::benchmark_runner::BenchmarkRunner;
use crate::performance_tracker::PerformanceTracker;
use crate::scoring_system::MultiDimensionalScoringSystem;

use super::benchmark_scheduler::{BenchmarkScheduler, ScheduledBenchmark};
use super::dataset_manager::{DatasetManager, DatasetValidationResult};

/// Continuous benchmarker orchestrating automated benchmarking
pub struct ContinuousBenchmarker {
    scheduler: Arc<BenchmarkScheduler>,
    dataset_manager: Arc<DatasetManager>,
    benchmark_runner: Arc<BenchmarkRunner>,
    performance_tracker: Arc<PerformanceTracker>,
    scoring_system: Arc<MultiDimensionalScoringSystem>,
    /// Whether continuous benchmarking is running
    is_running: Arc<RwLock<bool>>,
}

impl ContinuousBenchmarker {
    /// Create a new continuous benchmarker
    pub fn new(
        scheduler: Arc<BenchmarkScheduler>,
        dataset_manager: Arc<DatasetManager>,
        benchmark_runner: Arc<BenchmarkRunner>,
        performance_tracker: Arc<PerformanceTracker>,
        scoring_system: Arc<MultiDimensionalScoringSystem>,
    ) -> Self {
        Self {
            scheduler,
            dataset_manager,
            benchmark_runner,
            performance_tracker,
            scoring_system,
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start continuous benchmarking loop
    pub async fn start(&self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            warn!("Continuous benchmarking already running");
            return Ok(());
        }
        *is_running = true;
        drop(is_running);

        info!("Starting continuous benchmarking system");

        // Start background task for checking and executing benchmarks
        let scheduler = Arc::clone(&self.scheduler);
        let dataset_manager = Arc::clone(&self.dataset_manager);
        let benchmark_runner = Arc::clone(&self.benchmark_runner);
        let performance_tracker = Arc::clone(&self.performance_tracker);
        let scoring_system = Arc::clone(&self.scoring_system);
        let is_running_flag = Arc::clone(&self.is_running);

        tokio::spawn(async move {
            let mut check_interval = interval(TokioDuration::from_secs(300)); // Check every 5 minutes

            loop {
                check_interval.tick().await;

                // Check if still running
                {
                    let running = is_running_flag.read().await;
                    if !*running {
                        break;
                    }
                }

                // Check for due benchmarks and queue them
                match scheduler.check_and_queue_due_benchmarks().await {
                    Ok(count) if count > 0 => {
                        info!("Found {} benchmarks due for execution", count);
                    }
                    Ok(_) => {
                        debug!("No benchmarks due for execution");
                    }
                    Err(e) => {
                        error!("Error checking for due benchmarks: {}", e);
                    }
                }

                // Process queue
                while let Some(scheduled) = scheduler.pop_next_benchmark().await {
                    let scheduler_clone = Arc::clone(&scheduler);
                    let dataset_manager_clone = Arc::clone(&dataset_manager);
                    let benchmark_runner_clone = Arc::clone(&benchmark_runner);
                    let performance_tracker_clone = Arc::clone(&performance_tracker);
                    let scoring_system_clone = Arc::clone(&scoring_system);

                    // Execute benchmark in background
                    tokio::spawn(async move {
                        if let Err(e) = Self::execute_scheduled_benchmark(
                            &scheduled,
                            &dataset_manager_clone,
                            &benchmark_runner_clone,
                            &performance_tracker_clone,
                            &scoring_system_clone,
                        )
                        .await
                        {
                            error!("Failed to execute benchmark {}: {}", scheduled.id, e);
                        } else {
                            // Mark as executed
                            if let Err(e) = scheduler_clone.mark_executed(scheduled.id).await {
                                error!("Failed to mark benchmark {} as executed: {}", scheduled.id, e);
                            }
                        }
                    });
                }
            }
        });

        Ok(())
    }

    /// Stop continuous benchmarking
    pub async fn stop(&self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        info!("Stopped continuous benchmarking system");
        Ok(())
    }

    /// Execute a scheduled benchmark
    async fn execute_scheduled_benchmark(
        scheduled: &ScheduledBenchmark,
        dataset_manager: &DatasetManager,
        benchmark_runner: &BenchmarkRunner,
        performance_tracker: &PerformanceTracker,
        scoring_system: &MultiDimensionalScoringSystem,
    ) -> Result<()> {
        info!(
            "Executing scheduled benchmark {} (type: {:?})",
            scheduled.id,
            scheduled.benchmark_type
        );

        // Get dataset tasks
        let dataset_id = Uuid::new_v4(); // In real implementation, this would come from scheduled.dataset_version
        let tasks = dataset_manager
            .get_dataset_tasks(dataset_id, scheduled.dataset_version.as_deref())
            .await
            .unwrap_or_else(|_| Vec::new());

        if tasks.is_empty() {
            warn!("No tasks available for benchmark {}", scheduled.id);
            return Ok(());
        }

        // Execute benchmarks for each model
        let mut benchmark_results = Vec::new();

        for model in &scheduled.models {
            info!("Running benchmark for model {}", model.name);

            // TODO: Integrate actual benchmark runner API
            //       Currently uses placeholder integration; should use actual benchmark runner API for comprehensive benchmarking.
            let result = BenchmarkResult {
                model_id: model.id,
                benchmark_type: scheduled.benchmark_type.clone(),
                metrics: BenchmarkMetrics {
                    accuracy: 0.0,
                    speed: 0.0,
                    efficiency: 0.0,
                    quality: 0.0,
                    compliance: 0.0,
                },
                score: 0.0, // Would be calculated by scoring system
                ranking: 0,
                timestamp: Utc::now(),
                sla_validation: None,
            };

            // Calculate performance score using multi-dimensional scoring
            // Simple weighted average: accuracy (30%), speed (20%), efficiency (20%), quality (20%), compliance (10%)
            let performance_score = (result.metrics.accuracy * 0.3)
                + (result.metrics.speed * 0.2)
                + (result.metrics.efficiency * 0.2)
                + (result.metrics.quality * 0.2)
                + (result.metrics.compliance * 0.1);

            let mut result = result;
            result.score = performance_score;

            benchmark_results.push(result);
        }

        // Calculate performance summary using scoring system
        let performance_summary = scoring_system
            .calculate_performance_summary(&benchmark_results)
            .await
            .unwrap_or_else(|_| PerformanceSummary {
                overall_performance: if !benchmark_results.is_empty() {
                    benchmark_results.iter()
                        .map(|r| r.score)
                        .sum::<f64>() / benchmark_results.len() as f64
                } else {
                    0.0
                },
                performance_trend: PerformanceTrend::Stable,
                top_performers: Vec::new(),
                improvement_areas: Vec::new(),
            });

        // Create benchmark report
        let report = BenchmarkReport {
            report_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            benchmark_results: benchmark_results.clone(),
            performance_summary,
            regression_alerts: Vec::new(),
            recommendations: Vec::new(),
        };

        // Store benchmark report in performance tracker
        performance_tracker.store_benchmark_report(&report).await?;

        info!(
            "Completed benchmark {} with {} results",
            scheduled.id,
            benchmark_results.len()
        );

        Ok(())
    }

    /// Manually trigger a benchmark execution
    pub async fn trigger_benchmark(
        &self,
        benchmark_type: BenchmarkType,
        models: Vec<ModelSpecification>,
        dataset_version: Option<String>,
    ) -> Result<Uuid> {
        info!("Manually triggering {} benchmark", format!("{:?}", benchmark_type));

        // Create a one-time scheduled benchmark
        let scheduled_id = self
            .scheduler
            .schedule_benchmark(
                benchmark_type,
                super::benchmark_scheduler::BenchmarkCadence::Daily, // Doesn't matter for one-time
                models,
                dataset_version,
            )
            .await?;

        // Execute immediately
        let scheduled = self
            .scheduler
            .get_scheduled_benchmarks()
            .await
            .into_iter()
            .find(|b| b.id == scheduled_id)
            .ok_or_else(|| anyhow::anyhow!("Scheduled benchmark not found"))?;

        Self::execute_scheduled_benchmark(
            &scheduled,
            &self.dataset_manager,
            &self.benchmark_runner,
            &self.performance_tracker,
            &self.scoring_system,
        )
        .await?;

        self.scheduler.mark_executed(scheduled_id).await?;

        Ok(scheduled_id)
    }

    /// Get benchmarking status
    pub async fn get_status(&self) -> BenchmarkingStatus {
        let queue_size = self.scheduler.queue_size().await;
        let scheduled = self.scheduler.get_scheduled_benchmarks().await;
        let is_running = *self.is_running.read().await;

        BenchmarkingStatus {
            is_running,
            queue_size,
            total_scheduled: scheduled.len(),
            active_scheduled: scheduled.iter().filter(|b| b.active).count(),
        }
    }
}

/// Benchmarking system status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkingStatus {
    pub is_running: bool,
    pub queue_size: usize,
    pub total_scheduled: usize,
    pub active_scheduled: usize,
}

