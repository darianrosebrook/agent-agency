//! Inference testing infrastructure and performance measurement
//!
//! This module provides testing utilities and performance measurement tools
//! for Core ML inference operations, including latency tracking, throughput
//! analysis, and ANE utilization monitoring.

use crate::ane::ane_errors::{ANEError, Result};
use std::time::{Duration, Instant};
use super::types::*;

/// Phase 3B inference testing results
#[derive(Debug, Clone)]
pub struct InferenceTestResults {
    /// Total number of iterations
    pub total_iterations: usize,
    /// Number of successful inferences
    pub successful_inferences: usize,
    /// Number of failed inferences
    pub failed_inferences: usize,
    /// Number of inferences that used ANE
    pub ane_inferences: usize,
    /// Total testing time
    pub total_time: Duration,
    /// Latency measurements (in milliseconds)
    pub latencies_ms: Vec<f64>,
    /// P50 latency
    pub p50_latency_ms: f64,
    /// P99 latency
    pub p99_latency_ms: f64,
    /// Average latency
    pub avg_latency_ms: f64,
}

impl InferenceTestResults {
    /// Create a new test results instance
    pub fn new() -> Self {
        Self {
            total_iterations: 0,
            successful_inferences: 0,
            failed_inferences: 0,
            ane_inferences: 0,
            total_time: Duration::ZERO,
            latencies_ms: Vec::new(),
            p50_latency_ms: 0.0,
            p99_latency_ms: 0.0,
            avg_latency_ms: 0.0,
        }
    }

    /// Record a successful inference with its latency
    pub fn record_successful_inference(&mut self, duration: Duration) {
        self.successful_inferences += 1;
        self.latencies_ms.push(duration.as_secs_f64() * 1000.0);
    }

    /// Record a failed inference
    pub fn record_failed_inference(&mut self) {
        self.failed_inferences += 1;
    }

    /// Calculate performance percentiles from latency measurements
    pub fn calculate_percentiles(&mut self) {
        if self.latencies_ms.is_empty() {
            return;
        }

        self.latencies_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let len = self.latencies_ms.len();
        self.p50_latency_ms = self.latencies_ms[len * 50 / 100];
        self.p99_latency_ms = self.latencies_ms[len * 99 / 100];
        self.avg_latency_ms = self.latencies_ms.iter().sum::<f64>() / len as f64;
    }

    /// Get the ANE dispatch rate (percentage of inferences that used ANE)
    pub fn get_ane_dispatch_rate(&self) -> f64 {
        if self.successful_inferences == 0 {
            return 0.0;
        }
        self.ane_inferences as f64 / self.successful_inferences as f64
    }

    /// Get the success rate (percentage of successful inferences)
    pub fn get_success_rate(&self) -> f64 {
        if self.total_iterations == 0 {
            return 0.0;
        }
        self.successful_inferences as f64 / self.total_iterations as f64
    }

    /// Get throughput in inferences per second
    pub fn get_throughput_ips(&self) -> f64 {
        if self.total_time.as_secs_f64() == 0.0 {
            return 0.0;
        }
        self.successful_inferences as f64 / self.total_time.as_secs_f64()
    }
}

/// Performance benchmark configuration
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    /// Number of iterations to run
    pub iterations: usize,
    /// Warm-up iterations before measurement
    pub warm_up_iterations: usize,
    /// Whether to measure ANE utilization
    pub measure_ane_utilization: bool,
    /// Timeout for individual inferences
    pub timeout_ms: Option<u64>,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            iterations: 100,
            warm_up_iterations: 10,
            measure_ane_utilization: true,
            timeout_ms: Some(5000), // 5 second timeout
        }
    }
}

/// Core ML performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Average latency in milliseconds
    pub avg_latency_ms: f64,
    /// P50 latency in milliseconds
    pub p50_latency_ms: f64,
    /// P95 latency in milliseconds
    pub p95_latency_ms: f64,
    /// P99 latency in milliseconds
    pub p99_latency_ms: f64,
    /// Minimum latency in milliseconds
    pub min_latency_ms: f64,
    /// Maximum latency in milliseconds
    pub max_latency_ms: f64,
    /// Throughput in inferences per second
    pub throughput_ips: f64,
    /// ANE utilization rate (0.0 to 1.0)
    pub ane_utilization: Option<f64>,
    /// Memory usage in bytes
    pub memory_usage_bytes: Option<u64>,
}

/// Benchmark runner for Core ML inference
pub struct BenchmarkRunner<F, T>
where
    F: Fn() -> Result<T>,
{
    inference_fn: F,
    config: BenchmarkConfig,
}

impl<F, T> BenchmarkRunner<F, T>
where
    F: Fn() -> Result<T>,
{
    /// Create a new benchmark runner
    pub fn new(inference_fn: F, config: BenchmarkConfig) -> Self {
        Self {
            inference_fn,
            config,
        }
    }

    /// Run the benchmark and return performance metrics
    pub fn run(&self) -> Result<PerformanceMetrics> {
        // Warm-up phase
        for _ in 0..self.config.warm_up_iterations {
            let _ = (self.inference_fn)();
        }

        // Measurement phase
        let mut latencies = Vec::with_capacity(self.config.iterations);
        let start_time = Instant::now();

        for _ in 0..self.config.iterations {
            let inference_start = Instant::now();

            match (self.inference_fn)() {
                Ok(_) => {
                    let latency = inference_start.elapsed();
                    latencies.push(latency.as_secs_f64() * 1000.0);
                }
                Err(e) => {
                    return Err(ANEError::Internal(format!("Inference failed during benchmark: {}", e)));
                }
            }
        }

        let total_time = start_time.elapsed();

        // Calculate metrics
        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let len = latencies.len();
        let avg_latency = latencies.iter().sum::<f64>() / len as f64;
        let min_latency = latencies[0];
        let max_latency = latencies[len - 1];
        let p50_latency = latencies[len * 50 / 100];
        let p95_latency = latencies[len * 95 / 100];
        let p99_latency = latencies[len * 99 / 100];
        let throughput = len as f64 / total_time.as_secs_f64();

        Ok(PerformanceMetrics {
            avg_latency_ms: avg_latency,
            p50_latency_ms: p50_latency,
            p95_latency_ms: p95_latency,
            p99_latency_ms: p99_latency,
            min_latency_ms: min_latency,
            max_latency_ms: max_latency,
            throughput_ips: throughput,
            ane_utilization: None, // Would need platform-specific measurement
            memory_usage_bytes: None, // Would need platform-specific measurement
        })
    }
}

/// Test data generation utilities
pub mod test_data {
    use crate::ane::compat::types::{MLMultiArray, MLFeatureProvider, MLDictionaryFeatureProvider};
    use std::collections::HashMap;

    /// Generate random test data for benchmarking
    pub fn generate_random_data(shape: &[i32], seed: u64) -> Vec<f32> {
        let total_elements: usize = shape.iter().map(|&x| x as usize).product();
        let mut data = Vec::with_capacity(total_elements);

        // Simple LCG random number generator for deterministic test data
        let mut state = seed;
        for _ in 0..total_elements {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            let random_value = (state % 1000) as f32 / 1000.0; // 0.0 to 1.0
            data.push(random_value);
        }

        data
    }

    /// Create a test MLMultiArray with random data
    pub fn create_test_array(shape: &[i32], seed: u64) -> crate::ane::ane_errors::Result<MLMultiArray> {
        let data = generate_random_data(shape, seed);
        Ok(MLMultiArray::from_slice(&data, shape)?)
    }

    /// Create a test feature provider for inference
    pub fn create_test_provider(shape: &[i32], seed: u64) -> crate::ane::ane_errors::Result<MLDictionaryFeatureProvider> {
        let array = create_test_array(shape, seed)?;
        let mut features = HashMap::new();
        features.insert("input".to_string(), super::super::types::MLFeatureValue::MultiArray(array));

        Ok(MLDictionaryFeatureProvider::from_dictionary(&features)?)
    }
}

/// Performance comparison results
#[derive(Debug, Clone)]
pub struct PerformanceComparison {
    /// Latency improvement in milliseconds (positive = faster)
    pub latency_improvement_ms: f64,
    /// Throughput improvement in IPS (positive = better)
    pub throughput_improvement_ips: f64,
    /// Latency regression as percentage (positive = slower)
    pub latency_regression_percent: f64,
    /// Throughput regression as percentage (positive = worse)
    pub throughput_regression_percent: f64,
}

impl PerformanceComparison {
    /// Check if performance meets requirements
    pub fn meets_requirements(&self, max_regression_percent: f64) -> bool {
        self.latency_regression_percent <= max_regression_percent &&
        self.throughput_regression_percent <= max_regression_percent
    }
}

/// Validation utilities for test results
pub mod validation {
    use super::{PerformanceMetrics, PerformanceComparison};

    /// Validate that performance metrics are within acceptable ranges
    pub fn validate_metrics(metrics: &PerformanceMetrics, max_latency_ms: f64, min_throughput: f64) -> Result<(), crate::ane::ane_errors::ANEError> {
        if metrics.p99_latency_ms > max_latency_ms {
            return Err(crate::ane::ane_errors::ANEError::Internal(
                format!("P99 latency {:.2}ms exceeds maximum allowed {:.2}ms", metrics.p99_latency_ms, max_latency_ms)
            ));
        }

        if metrics.throughput_ips < min_throughput {
            return Err(crate::ane::ane_errors::ANEError::Internal(
                format!("Throughput {:.2} IPS below minimum required {:.2} IPS", metrics.throughput_ips, min_throughput)
            ));
        }

        Ok(())
    }

    /// Compare two sets of performance metrics
    pub fn compare_metrics(baseline: &PerformanceMetrics, current: &PerformanceMetrics) -> PerformanceComparison {
        PerformanceComparison {
            latency_improvement_ms: baseline.avg_latency_ms - current.avg_latency_ms,
            throughput_improvement_ips: current.throughput_ips - baseline.throughput_ips,
            latency_regression_percent: if baseline.avg_latency_ms > 0.0 {
                ((current.avg_latency_ms - baseline.avg_latency_ms) / baseline.avg_latency_ms) * 100.0
            } else {
                0.0
            },
            throughput_regression_percent: if baseline.throughput_ips > 0.0 {
                ((baseline.throughput_ips - current.throughput_ips) / baseline.throughput_ips) * 100.0
            } else {
                0.0
            },
        }
    }
}
