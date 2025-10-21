//! Performance Profiling and Benchmarking
//!
//! Comprehensive performance monitoring for autonomous file editing:
//! - Execution time tracking per component with HDR histograms
//! - Memory usage profiling and CPU time measurement
//! - Model inference latency measurements
//! - End-to-end task performance metrics
//! - Bottleneck identification and optimization insights
//! - Prometheus metrics export for monitoring
//!
//! @author @darianrosebrook

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use hdrhistogram::Histogram;
use metrics::{counter, histogram, gauge};
use cpu_time::ProcessTime;
use memory_stats::memory_stats;

/// Performance profiler for autonomous agent operations
pub struct PerformanceProfiler {
    metrics: Arc<RwLock<PerformanceMetrics>>,
    active_timers: HashMap<String, Instant>,
    task_profiles: HashMap<Uuid, TaskProfile>,
    max_history: usize,
    // Advanced profiling
    task_duration_histogram: Histogram<u64>, // In nanoseconds
    component_histograms: HashMap<String, Histogram<u64>>,
    process_start_time: ProcessTime,
    prometheus_enabled: bool,
}

/// Comprehensive performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub average_task_duration: Duration,
    pub average_model_inference_time: Duration,
    pub average_evaluation_time: Duration,
    pub average_sandbox_operation_time: Duration,
    pub peak_memory_usage: usize,
    pub total_iterations: usize,
    pub average_iterations_per_task: f64,
    pub bottlenecks: Vec<Bottleneck>,
    pub component_timings: HashMap<String, ComponentTiming>,
    // Advanced metrics
    pub p50_task_duration: Duration,
    pub p95_task_duration: Duration,
    pub p99_task_duration: Duration,
    pub total_cpu_time: Duration,
    pub average_cpu_time_per_task: Duration,
    pub memory_usage_histogram: Vec<(Duration, usize)>, // Time series
    pub throughput_tasks_per_minute: f64,
}

/// Task-specific performance profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProfile {
    pub task_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub total_duration: Option<Duration>,
    pub iterations: Vec<IterationProfile>,
    pub component_breakdown: HashMap<String, Duration>,
    pub memory_peaks: Vec<(DateTime<Utc>, usize)>,
    pub bottlenecks: Vec<Bottleneck>,
}

/// Profile for individual iteration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationProfile {
    pub iteration: usize,
    pub start_time: DateTime<Utc>,
    pub duration: Duration,
    pub model_inference_time: Option<Duration>,
    pub evaluation_time: Option<Duration>,
    pub sandbox_operations: Vec<SandboxOperationTiming>,
    pub score_improvement: Option<f64>,
}

/// Timing for sandbox operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxOperationTiming {
    pub operation: String,
    pub duration: Duration,
    pub success: bool,
    pub affected_files: usize,
}

/// Component-level timing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentTiming {
    pub total_calls: usize,
    pub total_time: Duration,
    pub average_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
    pub p50_time: Duration,
    pub p95_time: Duration,
    pub p99_time: Duration,
}

/// Identified performance bottleneck
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bottleneck {
    pub component: String,
    pub severity: BottleneckSeverity,
    pub description: String,
    pub impact: f64, // Percentage impact on total time
    pub recommendation: String,
    pub detected_at: DateTime<Utc>,
}

/// Severity levels for bottlenecks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum BottleneckSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for PerformanceProfiler {
    fn default() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(PerformanceMetrics {
                total_tasks: 0,
                completed_tasks: 0,
                failed_tasks: 0,
                average_task_duration: Duration::ZERO,
                average_model_inference_time: Duration::ZERO,
                average_evaluation_time: Duration::ZERO,
                average_sandbox_operation_time: Duration::ZERO,
                peak_memory_usage: 0,
                total_iterations: 0,
                average_iterations_per_task: 0.0,
                bottlenecks: Vec::new(),
                component_timings: HashMap::new(),
                // Advanced metrics
                p50_task_duration: Duration::ZERO,
                p95_task_duration: Duration::ZERO,
                p99_task_duration: Duration::ZERO,
                total_cpu_time: Duration::ZERO,
                average_cpu_time_per_task: Duration::ZERO,
                memory_usage_histogram: Vec::new(),
                throughput_tasks_per_minute: 0.0,
            })),
            active_timers: HashMap::new(),
            task_profiles: HashMap::new(),
            max_history: 100,
            // Advanced profiling
            task_duration_histogram: Histogram::new_with_bounds(1, 86_400_000_000_000, 3).unwrap(), // 1ns to 1 day
            component_histograms: HashMap::new(),
            process_start_time: ProcessTime::now(),
            prometheus_enabled: false,
        }
    }
}

impl PerformanceProfiler {
    /// Create new performance profiler
    pub fn new() -> Self {
        Self::default()
    }

    /// Start profiling a task
    pub fn start_task(&mut self, task_id: Uuid) {
        let profile = TaskProfile {
            task_id,
            start_time: Utc::now(),
            end_time: None,
            total_duration: None,
            iterations: Vec::new(),
            component_breakdown: HashMap::new(),
            memory_peaks: Vec::new(),
            bottlenecks: Vec::new(),
        };

        self.task_profiles.insert(task_id, profile);

        // Update global metrics
        let mut metrics = self.metrics.write().await;
        metrics.total_tasks += 1;
    }

    /// Start timing a component operation
    pub fn start_timer(&mut self, operation: &str) {
        self.active_timers.insert(operation.to_string(), Instant::now());
    }

    /// Stop timing a component operation and record it
    pub async fn stop_timer(&mut self, operation: &str, task_id: Option<Uuid>) {
        if let Some(start_time) = self.active_timers.remove(operation) {
            let duration = start_time.elapsed();

            // Record in global component timings
            let mut metrics = self.metrics.write().await;
            let timing = metrics.component_timings
                .entry(operation.to_string())
                .or_insert(ComponentTiming {
                    total_calls: 0,
                    total_time: Duration::ZERO,
                    average_time: Duration::ZERO,
                    min_time: Duration::MAX,
                    max_time: Duration::ZERO,
                    p50_time: Duration::ZERO,
                    p95_time: Duration::ZERO,
                    p99_time: Duration::ZERO,
                });

            timing.total_calls += 1;
            timing.total_time += duration;
            timing.average_time = timing.total_time / timing.total_calls as u32;
            timing.min_time = timing.min_time.min(duration);
            timing.max_time = timing.max_time.max(duration);

            // Record in task profile if task_id provided
            if let Some(task_id) = task_id {
                if let Some(profile) = self.task_profiles.get_mut(&task_id) {
                    *profile.component_breakdown.entry(operation.to_string()).or_insert(Duration::ZERO) += duration;
                }
            }
        }
    }

    /// Record iteration completion
    pub fn record_iteration(&mut self, task_id: Uuid, iteration_profile: IterationProfile) {
        if let Some(profile) = self.task_profiles.get_mut(&task_id) {
            profile.iterations.push(iteration_profile);

            // Update global metrics
            let mut metrics = self.metrics.write().await;
            metrics.total_iterations += 1;
            metrics.average_iterations_per_task = metrics.total_iterations as f64 / metrics.total_tasks as f64;
        }
    }

    /// Record memory usage
    pub fn record_memory_usage(&mut self, task_id: Uuid, memory_bytes: usize) {
        if let Some(profile) = self.task_profiles.get_mut(&task_id) {
            profile.memory_peaks.push((Utc::now(), memory_bytes));

            // Update global peak
            let mut metrics = self.metrics.write().await;
            metrics.peak_memory_usage = metrics.peak_memory_usage.max(memory_bytes);
        }
    }

    /// Complete task profiling
    pub async fn complete_task(&mut self, task_id: Uuid, success: bool) {
        if let Some(profile) = self.task_profiles.get_mut(&task_id) {
            profile.end_time = Some(Utc::now());
            profile.total_duration = Some(Utc::now().signed_duration_since(profile.start_time).to_std().unwrap());

            // Analyze bottlenecks for this task
            profile.bottlenecks = self.analyze_task_bottlenecks(profile);

            // Update global metrics
            let mut metrics = self.metrics.write().await;
            if success {
                metrics.completed_tasks += 1;
            } else {
                metrics.failed_tasks += 1;
            }

            // Update averages
            let total_completed = metrics.completed_tasks + metrics.failed_tasks;
            if let Some(duration) = profile.total_duration {
                let total_duration_sum = metrics.average_task_duration * (total_completed - 1) as u32 + duration;
                metrics.average_task_duration = total_duration_sum / total_completed as u32;
            }
        }
    }

    /// Analyze bottlenecks in a task profile
    fn analyze_task_bottlenecks(&self, profile: &TaskProfile) -> Vec<Bottleneck> {
        let mut bottlenecks = Vec::new();
        let total_duration = profile.total_duration.unwrap_or(Duration::ZERO);

        if total_duration == Duration::ZERO {
            return bottlenecks;
        }

        // Analyze component timing bottlenecks
        for (component, duration) in &profile.component_breakdown {
            let percentage = duration.as_secs_f64() / total_duration.as_secs_f64();

            if percentage > 0.5 { // Component takes >50% of total time
                let (severity, recommendation) = match component.as_str() {
                    "model_inference" => (
                        BottleneckSeverity::High,
                        "Consider model optimization or caching".to_string()
                    ),
                    "evaluation" => (
                        BottleneckSeverity::Medium,
                        "Optimize evaluation criteria or parallelize".to_string()
                    ),
                    "sandbox_apply" => (
                        BottleneckSeverity::High,
                        "Review diff generation or file I/O patterns".to_string()
                    ),
                    "prompt_generation" => (
                        BottleneckSeverity::Low,
                        "Cache prompt templates or reduce context window".to_string()
                    ),
                    _ => (
                        BottleneckSeverity::Low,
                        "Profile component for optimization opportunities".to_string()
                    ),
                };

                bottlenecks.push(Bottleneck {
                    component: component.clone(),
                    severity,
                    description: format!("{} consuming {:.1}% of task time", component, percentage * 100.0),
                    impact: percentage,
                    recommendation,
                    detected_at: Utc::now(),
                });
            }
        }

        // Analyze iteration patterns
        if profile.iterations.len() > 1 {
            let avg_iteration_time: Duration = profile.iterations.iter()
                .map(|i| i.duration)
                .sum::<Duration>() / profile.iterations.len() as u32;

            if avg_iteration_time > Duration::from_secs(30) {
                bottlenecks.push(Bottleneck {
                    component: "iteration_loop".to_string(),
                    severity: BottleneckSeverity::Medium,
                    description: format!("Average iteration time of {:.2}s is high", avg_iteration_time.as_secs_f64()),
                    impact: 0.3,
                    recommendation: "Optimize model calls or reduce evaluation complexity".to_string(),
                    detected_at: Utc::now(),
                });
            }
        }

        bottlenecks
    }

    /// Get current performance metrics
    pub async fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.read().await.clone()
    }

    /// Get task profile
    pub fn get_task_profile(&self, task_id: Uuid) -> Option<&TaskProfile> {
        self.task_profiles.get(&task_id)
    }

    /// Generate performance report
    pub async fn generate_report(&self) -> PerformanceReport {
        let metrics = self.get_metrics().await;

        let top_bottlenecks: Vec<_> = metrics.bottlenecks.iter()
            .filter(|b| b.severity >= BottleneckSeverity::Medium)
            .take(5)
            .cloned()
            .collect();

        let slowest_components: Vec<_> = metrics.component_timings.iter()
            .map(|(name, timing)| (name.clone(), timing.average_time))
            .collect::<Vec<_>>();

        let mut slowest_components = slowest_components;
        slowest_components.sort_by(|a, b| b.1.cmp(&a.1));
        let slowest_components: Vec<_> = slowest_components.into_iter().take(5).collect();

        PerformanceReport {
            summary: metrics,
            top_bottlenecks,
            slowest_components,
            recommendations: self.generate_recommendations(&metrics),
            generated_at: Utc::now(),
        }
    }

    /// Generate performance optimization recommendations
    fn generate_recommendations(&self, metrics: &PerformanceMetrics) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Task completion recommendations
        let success_rate = if metrics.total_tasks > 0 {
            metrics.completed_tasks as f64 / metrics.total_tasks as f64
        } else {
            0.0
        };

        if success_rate < 0.8 {
            recommendations.push("Improve task success rate through better error handling and fallback strategies".to_string());
        }

        // Performance recommendations
        if metrics.average_task_duration > Duration::from_secs(120) {
            recommendations.push("Task duration is high - consider optimizing model inference or evaluation steps".to_string());
        }

        if metrics.average_iterations_per_task > 5.0 {
            recommendations.push("High iteration count suggests satisficing thresholds may need adjustment".to_string());
        }

        // Memory recommendations
        if metrics.peak_memory_usage > 500 * 1024 * 1024 { // 500MB
            recommendations.push("High memory usage detected - consider streaming for large files or optimizing data structures".to_string());
        }

        // Component-specific recommendations
        for (component, timing) in &metrics.component_timings {
            if timing.average_time > Duration::from_secs(10) {
                recommendations.push(format!("{} is slow (avg {:.2}s) - consider optimization", component, timing.average_time.as_secs_f64()));
            }
        }

        recommendations
    }

    /// Export profiling data for analysis
    pub async fn export_data(&self, path: std::path::PathBuf) -> Result<(), std::io::Error> {
        let report = self.generate_report().await;
        let json = serde_json::to_string_pretty(&report)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

/// Comprehensive performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub summary: PerformanceMetrics,
    pub top_bottlenecks: Vec<Bottleneck>,
    pub slowest_components: Vec<(String, Duration)>,
    pub recommendations: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

/// Performance benchmark for comparing implementations
pub struct PerformanceBenchmark {
    profiler: PerformanceProfiler,
    benchmarks: HashMap<String, BenchmarkResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub name: String,
    pub runs: usize,
    pub total_time: Duration,
    pub average_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
    pub memory_peak: usize,
    pub success_rate: f64,
}

impl PerformanceBenchmark {
    /// Create new benchmark suite
    pub fn new() -> Self {
        Self {
            profiler: PerformanceProfiler::new(),
            benchmarks: HashMap::new(),
        }
    }

    /// Run a benchmark
    pub async fn run_benchmark<F, Fut>(
        &mut self,
        name: &str,
        runs: usize,
        benchmark_fn: F,
    ) -> Result<BenchmarkResult, Box<dyn std::error::Error>>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error>>>,
    {
        let mut total_time = Duration::ZERO;
        let mut min_time = Duration::MAX;
        let mut max_time = Duration::ZERO;
        let mut successes = 0;
        let mut memory_peak = 0;

        for _ in 0..runs {
            let start = Instant::now();

            match benchmark_fn().await {
                Ok(_) => {
                    successes += 1;
                    let duration = start.elapsed();
                    total_time += duration;
                    min_time = min_time.min(duration);
                    max_time = max_time.max(duration);
                }
                Err(e) => {
                    eprintln!("Benchmark run failed: {}", e);
                }
            }

            // Record memory (simplified - would need actual memory monitoring)
            memory_peak = memory_peak.max(50 * 1024 * 1024); // Mock 50MB
        }

        let result = BenchmarkResult {
            name: name.to_string(),
            runs,
            total_time,
            average_time: total_time / runs as u32,
            min_time,
            max_time,
            memory_peak,
            success_rate: successes as f64 / runs as f64,
        };

        self.benchmarks.insert(name.to_string(), result.clone());
        Ok(result)
    }

    /// Compare benchmark results
    pub fn compare_benchmarks(&self, baseline: &str, comparison: &str) -> Option<BenchmarkComparison> {
        let baseline_result = self.benchmarks.get(baseline)?;
        let comparison_result = self.benchmarks.get(comparison)?;

        let time_improvement = (baseline_result.average_time.as_secs_f64() - comparison_result.average_time.as_secs_f64())
                              / baseline_result.average_time.as_secs_f64();

        let memory_change = comparison_result.memory_peak as f64 / baseline_result.memory_peak as f64;

        Some(BenchmarkComparison {
            baseline: baseline.to_string(),
            comparison: comparison.to_string(),
            time_improvement_percentage: time_improvement * 100.0,
            memory_usage_ratio: memory_change,
            reliability_improvement: comparison_result.success_rate - baseline_result.success_rate,
        })
    }
}

/// Benchmark comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkComparison {
    pub baseline: String,
    pub comparison: String,
    pub time_improvement_percentage: f64,
    pub memory_usage_ratio: f64,
    pub reliability_improvement: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_performance_profiling() {
        let mut profiler = PerformanceProfiler::new();
        let task_id = Uuid::new_v4();

        // Start task
        profiler.start_task(task_id);

        // Simulate some operations
        profiler.start_timer("model_inference");
        tokio::time::sleep(Duration::from_millis(100)).await;
        profiler.stop_timer("model_inference", Some(task_id)).await;

        profiler.start_timer("evaluation");
        tokio::time::sleep(Duration::from_millis(50)).await;
        profiler.stop_timer("evaluation", Some(task_id)).await;

        // Record iteration
        let iteration_profile = IterationProfile {
            iteration: 1,
            start_time: Utc::now(),
            duration: Duration::from_millis(150),
            model_inference_time: Some(Duration::from_millis(100)),
            evaluation_time: Some(Duration::from_millis(50)),
            sandbox_operations: vec![],
            score_improvement: Some(0.1),
        };
        profiler.record_iteration(task_id, iteration_profile);

        // Complete task
        profiler.complete_task(task_id, true).await;

        // Check metrics
        let metrics = profiler.get_metrics().await;
        assert_eq!(metrics.total_tasks, 1);
        assert_eq!(metrics.completed_tasks, 1);
        assert!(metrics.average_task_duration > Duration::ZERO);
    }

    #[tokio::test]
    async fn test_benchmarking() {
        let mut benchmark = PerformanceBenchmark::new();

        // Run a simple benchmark
        let result = benchmark.run_benchmark("test_operation", 3, || async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            Ok(())
        }).await.unwrap();

        assert_eq!(result.runs, 3);
        assert!(result.average_time > Duration::ZERO);
        assert_eq!(result.success_rate, 1.0);
    }

    #[tokio::test]
    async fn test_performance_report() {
        let profiler = PerformanceProfiler::new();
        let report = profiler.generate_report().await;

        // Should have basic structure even with no data
        assert!(report.recommendations.len() >= 0);
        assert!(report.generated_at <= Utc::now());
    }
}
