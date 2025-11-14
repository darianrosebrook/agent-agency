//! Benchmark scheduler for automated continuous benchmarking
//!
//! Manages scheduling of micro and macro benchmarks with configurable cadence
//! (daily, weekly, monthly) and task queue system.
//!
//! @author @darianrosebrook

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::benchmark_types::{BenchmarkType, ModelSpecification};

/// Benchmark cadence for scheduling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BenchmarkCadence {
    /// Daily benchmarks (micro-benchmarks)
    Daily,
    /// Weekly benchmarks (macro-benchmarks)
    Weekly,
    /// Monthly benchmarks (comprehensive evaluation)
    Monthly,
}

impl BenchmarkCadence {
    /// Get the duration for this cadence
    pub fn duration(&self) -> Duration {
        match self {
            BenchmarkCadence::Daily => Duration::days(1),
            BenchmarkCadence::Weekly => Duration::days(7),
            BenchmarkCadence::Monthly => Duration::days(30),
        }
    }

    /// Get next scheduled time from a given time
    pub fn next_scheduled_time(&self, from: DateTime<Utc>) -> DateTime<Utc> {
        match self {
            BenchmarkCadence::Daily => from + Duration::days(1),
            BenchmarkCadence::Weekly => from + Duration::days(7),
            BenchmarkCadence::Monthly => from + Duration::days(30),
        }
    }
}

/// Scheduled benchmark task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledBenchmark {
    /// Unique identifier for this scheduled benchmark
    pub id: Uuid,
    /// Benchmark type (micro or macro)
    pub benchmark_type: BenchmarkType,
    /// Cadence for this benchmark
    pub cadence: BenchmarkCadence,
    /// Models to benchmark
    pub models: Vec<ModelSpecification>,
    /// Scheduled execution time
    pub scheduled_at: DateTime<Utc>,
    /// Last execution time (if any)
    pub last_executed_at: Option<DateTime<Utc>>,
    /// Whether this benchmark is active
    pub active: bool,
    /// Dataset version to use
    pub dataset_version: Option<String>,
}

/// Benchmark scheduler managing automated benchmark execution
pub struct BenchmarkScheduler {
    /// Queue of scheduled benchmarks ready to execute
    task_queue: Arc<RwLock<VecDeque<ScheduledBenchmark>>>,
    /// All scheduled benchmarks (active and inactive)
    scheduled_benchmarks: Arc<RwLock<Vec<ScheduledBenchmark>>>,
    /// Last check time for scheduling
    last_check_time: Arc<RwLock<DateTime<Utc>>>,
}

impl BenchmarkScheduler {
    /// Create a new benchmark scheduler
    pub fn new() -> Self {
        Self {
            task_queue: Arc::new(RwLock::new(VecDeque::new())),
            scheduled_benchmarks: Arc::new(RwLock::new(Vec::new())),
            last_check_time: Arc::new(RwLock::new(Utc::now())),
        }
    }

    /// Schedule a benchmark with the given cadence
    pub async fn schedule_benchmark(
        &self,
        benchmark_type: BenchmarkType,
        cadence: BenchmarkCadence,
        models: Vec<ModelSpecification>,
        dataset_version: Option<String>,
    ) -> Result<Uuid> {
        let scheduled_at = cadence.next_scheduled_time(Utc::now());

        let scheduled = ScheduledBenchmark {
            id: Uuid::new_v4(),
            benchmark_type: benchmark_type.clone(),
            cadence,
            models,
            scheduled_at,
            last_executed_at: None,
            active: true,
            dataset_version,
        };

        let mut benchmarks = self.scheduled_benchmarks.write().await;
        benchmarks.push(scheduled.clone());

        // If scheduled time is in the past or very soon, add to queue
        if scheduled_at <= Utc::now() + chrono::Duration::minutes(5) {
            let mut queue = self.task_queue.write().await;
            queue.push_back(scheduled.clone());
        }

        info!(
            "Scheduled {} benchmark with {:?} cadence, ID: {}",
            format!("{:?}", &benchmark_type),
            cadence,
            scheduled.id
        );

        Ok(scheduled.id)
    }

    /// Schedule default benchmarks (daily micro, weekly macro)
    pub async fn schedule_default_benchmarks(
        &self,
        models: Vec<ModelSpecification>,
    ) -> Result<Vec<Uuid>> {
        let mut ids = Vec::new();

        // Schedule daily micro-benchmarks
        let daily_id = self
            .schedule_benchmark(
                BenchmarkType::MicroBenchmark,
                BenchmarkCadence::Daily,
                models.clone(),
                None,
            )
            .await?;
        ids.push(daily_id);

        // Schedule weekly macro-benchmarks
        let weekly_id = self
            .schedule_benchmark(
                BenchmarkType::MacroBenchmark,
                BenchmarkCadence::Weekly,
                models,
                None,
            )
            .await?;
        ids.push(weekly_id);

        info!(
            "Scheduled default benchmarks: {} daily, {} weekly",
            daily_id, weekly_id
        );
        Ok(ids)
    }

    /// Check for benchmarks that are due and add them to the queue
    pub async fn check_and_queue_due_benchmarks(&self) -> Result<usize> {
        let now = Utc::now();
        let mut benchmarks = self.scheduled_benchmarks.write().await;
        let mut queue = self.task_queue.write().await;
        let mut queued_count = 0;

        for benchmark in benchmarks.iter_mut() {
            if !benchmark.active {
                continue;
            }

            // Check if benchmark is due
            let is_due = benchmark.scheduled_at <= now;
            let should_reschedule = benchmark
                .last_executed_at
                .map(|last| {
                    let next = benchmark.cadence.next_scheduled_time(last);
                    next <= now
                })
                .unwrap_or(is_due);

            if should_reschedule {
                // Update scheduled time for next execution
                benchmark.scheduled_at = benchmark.cadence.next_scheduled_time(now);

                // Add to queue
                queue.push_back(benchmark.clone());
                queued_count += 1;

                debug!(
                    "Queued benchmark {} (type: {:?}, cadence: {:?})",
                    benchmark.id, benchmark.benchmark_type, benchmark.cadence
                );
            }
        }

        *self.last_check_time.write().await = now;

        if queued_count > 0 {
            info!("Queued {} benchmarks for execution", queued_count);
        }

        Ok(queued_count)
    }

    /// Get next benchmark from queue
    pub async fn pop_next_benchmark(&self) -> Option<ScheduledBenchmark> {
        let mut queue = self.task_queue.write().await;
        queue.pop_front()
    }

    /// Mark benchmark as executed
    pub async fn mark_executed(&self, benchmark_id: Uuid) -> Result<()> {
        let mut benchmarks = self.scheduled_benchmarks.write().await;

        if let Some(benchmark) = benchmarks.iter_mut().find(|b| b.id == benchmark_id) {
            benchmark.last_executed_at = Some(Utc::now());
            info!("Marked benchmark {} as executed", benchmark_id);
        } else {
            warn!(
                "Benchmark {} not found for marking as executed",
                benchmark_id
            );
        }

        Ok(())
    }

    /// Get all scheduled benchmarks
    pub async fn get_scheduled_benchmarks(&self) -> Vec<ScheduledBenchmark> {
        self.scheduled_benchmarks.read().await.clone()
    }

    /// Get queue size
    pub async fn queue_size(&self) -> usize {
        self.task_queue.read().await.len()
    }

    /// Cancel a scheduled benchmark
    pub async fn cancel_benchmark(&self, benchmark_id: Uuid) -> Result<()> {
        let mut benchmarks = self.scheduled_benchmarks.write().await;

        if let Some(benchmark) = benchmarks.iter_mut().find(|b| b.id == benchmark_id) {
            benchmark.active = false;
            info!("Cancelled benchmark {}", benchmark_id);
        } else {
            return Err(anyhow::anyhow!("Benchmark {} not found", benchmark_id));
        }

        // Remove from queue if present
        let mut queue = self.task_queue.write().await;
        queue.retain(|b| b.id != benchmark_id);

        Ok(())
    }
}

impl Default for BenchmarkScheduler {
    fn default() -> Self {
        Self::new()
    }
}
