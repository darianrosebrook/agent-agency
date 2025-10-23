//! ANE Optimization System
//!
//! Provides intelligent optimization for Apple Neural Engine performance,
//! including model compilation, memory management, and inference tuning.

use crate::ane::errors::{ANEError, Result};
use crate::ane::monitoring::yolo_monitor::{YOLOPerformanceMonitor, YOLOPerformanceStats, YOLOPerformanceThresholds};
use crate::telemetry::TelemetryCollector;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{info, warn, debug};

/// ANE optimization strategies
#[derive(Debug, Clone)]
pub enum ANEOptimizationStrategy {
    /// Maximize performance (may use more memory)
    Performance,
    /// Balance performance and memory usage
    Balanced,
    /// Minimize memory usage (may reduce performance)
    MemoryEfficient,
    /// Custom optimization parameters
    Custom(ANEOptimizationParams),
}

/// Custom optimization parameters
#[derive(Debug, Clone)]
pub struct ANEOptimizationParams {
    /// Batch size for inference
    pub batch_size: usize,
    /// Precision mode
    pub precision: PrecisionMode,
    /// Memory allocation strategy
    pub memory_strategy: MemoryStrategy,
    /// Compute unit preference
    pub compute_units: ComputeUnitPreference,
}

/// Precision modes for CoreML
#[derive(Debug, Clone)]
pub enum PrecisionMode {
    /// Full precision (highest accuracy)
    Full,
    /// Half precision (balanced accuracy/performance)
    Half,
    /// Quantized (lowest memory, may reduce accuracy)
    Quantized,
}

/// Memory allocation strategies
#[derive(Debug, Clone)]
pub enum MemoryStrategy {
    /// Pre-allocate all memory
    Preallocate,
    /// Allocate memory as needed
    OnDemand,
    /// Use memory pooling
    Pooled,
}

/// Compute unit preferences
#[derive(Debug, Clone)]
pub enum ComputeUnitPreference {
    /// Prefer ANE (Apple Neural Engine)
    ANE,
    /// Prefer GPU
    GPU,
    /// Prefer CPU
    CPU,
    /// Auto-select based on model and hardware
    Auto,
}

/// ANE performance optimizer
pub struct ANEOptimizer {
    strategy: ANEOptimizationStrategy,
    performance_history: HashMap<String, Vec<f64>>, // model_name -> inference_times
    current_params: HashMap<String, ANEOptimizationParams>, // model_name -> params
    adaptation_enabled: bool,
}

impl ANEOptimizer {
    /// Create a new ANE optimizer with the specified strategy
    pub fn new(strategy: ANEOptimizationStrategy) -> Self {
        Self {
            strategy: strategy.clone(),
            performance_history: HashMap::new(),
            current_params: HashMap::new(),
            adaptation_enabled: true,
        }
    }

    /// Get optimization parameters for a specific model
    pub fn get_optimization_params(&mut self, model_name: &str) -> ANEOptimizationParams {
        if let Some(params) = self.current_params.get(model_name) {
            return params.clone();
        }

        // Generate initial parameters based on strategy
        let params = match &self.strategy {
            ANEOptimizationStrategy::Performance => ANEOptimizationParams {
                batch_size: 1,
                precision: PrecisionMode::Full,
                memory_strategy: MemoryStrategy::Preallocate,
                compute_units: ComputeUnitPreference::ANE,
            },
            ANEOptimizationStrategy::Balanced => ANEOptimizationParams {
                batch_size: 1,
                precision: PrecisionMode::Half,
                memory_strategy: MemoryStrategy::Pooled,
                compute_units: ComputeUnitPreference::Auto,
            },
            ANEOptimizationStrategy::MemoryEfficient => ANEOptimizationParams {
                batch_size: 1,
                precision: PrecisionMode::Quantized,
                memory_strategy: MemoryStrategy::OnDemand,
                compute_units: ComputeUnitPreference::CPU,
            },
            ANEOptimizationStrategy::Custom(params) => params.clone(),
        };

        self.current_params.insert(model_name.to_string(), params.clone());
        params
    }

    /// Record performance metrics and potentially adapt optimization parameters
    pub fn record_performance(&mut self, model_name: &str, inference_time_ms: f64) {
        let history = self.performance_history
            .entry(model_name.to_string())
            .or_insert_with(Vec::new);

        history.push(inference_time_ms);

        // Keep only last 100 measurements
        if history.len() > 100 {
            history.remove(0);
        }

        // Adapt parameters if enabled and we have enough data
        if self.adaptation_enabled && history.len() >= 10 {
            self.adapt_parameters(model_name);
        }
    }

    /// Adapt optimization parameters based on performance history
    fn adapt_parameters(&mut self, model_name: &str) {
        let history = match self.performance_history.get(model_name) {
            Some(h) => h,
            None => return,
        };

        if history.len() < 10 {
            return;
        }

        let avg_time = history.iter().sum::<f64>() / history.len() as f64;
        let recent_avg = history.iter().rev().take(5).sum::<f64>() / 5.0;

        // If performance is degrading, try different parameters
        if recent_avg > avg_time * 1.1 {
            debug!("Performance degrading for {}, attempting optimization", model_name);
            self.optimize_for_model(model_name, avg_time, recent_avg);
        }
    }

    /// Optimize parameters for a specific model based on performance
    fn optimize_for_model(&mut self, model_name: &str, avg_time: f64, recent_avg: f64) {
        let current_params = match self.current_params.get_mut(model_name) {
            Some(p) => p,
            None => return,
        };

        // If inference is slow, try different strategies
        if avg_time > 50.0 { // More than 50ms average
            match current_params.compute_units {
                ComputeUnitPreference::ANE => {
                    info!("Switching {} from ANE to Auto due to slow performance", model_name);
                    current_params.compute_units = ComputeUnitPreference::Auto;
                }
                ComputeUnitPreference::Auto => {
                    info!("Switching {} from Auto to CPU for memory efficiency", model_name);
                    current_params.compute_units = ComputeUnitPreference::CPU;
                    current_params.precision = PrecisionMode::Half;
                }
                _ => {
                    // Already on CPU, try reducing precision
                    if matches!(current_params.precision, PrecisionMode::Full) {
                        info!("Reducing {} precision from Full to Half", model_name);
                        current_params.precision = PrecisionMode::Half;
                    }
                }
            }
        }
    }

    /// Get performance statistics for a model
    pub fn get_performance_stats(&self, model_name: &str) -> Option<PerformanceStats> {
        let history = self.performance_history.get(model_name)?;

        if history.is_empty() {
            return None;
        }

        let count = history.len();
        let sum: f64 = history.iter().sum();
        let avg = sum / count as f64;

        let variance = history.iter()
            .map(|x| (x - avg).powi(2))
            .sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();

        let min = history.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = history.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        Some(PerformanceStats {
            count,
            average_ms: avg,
            std_deviation_ms: std_dev,
            min_ms: min,
            max_ms: max,
            p95_ms: percentile(history, 95.0),
            p99_ms: percentile(history, 99.0),
        })
    }

    /// Reset performance history for a model
    pub fn reset_performance_history(&mut self, model_name: &str) {
        self.performance_history.remove(model_name);
    }

    /// Enable or disable automatic parameter adaptation
    pub fn set_adaptation_enabled(&mut self, enabled: bool) {
        self.adaptation_enabled = enabled;
    }
}

/// Performance statistics
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub count: usize,
    pub average_ms: f64,
    pub std_deviation_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
}

/// Calculate percentile from a sorted vector
fn percentile(data: &[f64], p: f64) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let index = (p / 100.0 * (sorted.len() - 1) as f64) as usize;
    sorted[index]
}

/// ANE memory optimizer
pub struct ANEMemoryOptimizer {
    memory_pressure_threshold_mb: f64,
    optimization_enabled: bool,
}

impl ANEMemoryOptimizer {
    pub fn new() -> Self {
        Self {
            memory_pressure_threshold_mb: 500.0, // 500MB threshold
            optimization_enabled: true,
        }
    }

    /// Check if memory optimization is needed
    pub fn should_optimize_memory(&self, current_memory_mb: f64) -> bool {
        self.optimization_enabled && current_memory_mb > self.memory_pressure_threshold_mb
    }

    /// Get memory optimization recommendations
    pub fn get_memory_recommendations(&self, current_memory_mb: f64) -> Vec<String> {
        let mut recommendations = Vec::new();

        if current_memory_mb > self.memory_pressure_threshold_mb * 1.5 {
            recommendations.push("Critical: Memory usage very high. Consider model quantization.".to_string());
        } else if current_memory_mb > self.memory_pressure_threshold_mb {
            recommendations.push("High memory usage. Consider reducing batch size or using memory pooling.".to_string());
        }

        if current_memory_mb > 1000.0 {
            recommendations.push("Memory usage over 1GB. Consider model offloading or smaller variants.".to_string());
        }

        recommendations
    }
}

/// Batch size optimizer for concurrent inference
pub struct BatchOptimizer {
    max_batch_size: usize,
    current_batch_size: usize,
    adaptation_enabled: bool,
}

impl BatchOptimizer {
    pub fn new(max_batch_size: usize) -> Self {
        Self {
            max_batch_size,
            current_batch_size: 1,
            adaptation_enabled: true,
        }
    }

    /// Optimize batch size based on performance metrics
    pub fn optimize_batch_size(&mut self, throughput: f64, latency: f64) -> usize {
        if !self.adaptation_enabled {
            return self.current_batch_size;
        }

        // Simple heuristic: if latency is acceptable (< 100ms) and throughput is good,
        // try increasing batch size
        if latency < 100.0 && throughput > 10.0 && self.current_batch_size < self.max_batch_size {
            self.current_batch_size = (self.current_batch_size * 2).min(self.max_batch_size);
            info!("Increased batch size to {}", self.current_batch_size);
        } else if latency > 200.0 && self.current_batch_size > 1 {
            // If latency is too high, reduce batch size
            self.current_batch_size = (self.current_batch_size / 2).max(1);
            info!("Reduced batch size to {}", self.current_batch_size);
        }

        self.current_batch_size
    }

    /// Get current batch size
    pub fn current_batch_size(&self) -> usize {
        self.current_batch_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimizer_creation() {
        let mut optimizer = ANEOptimizer::new(ANEOptimizationStrategy::Balanced);
        let params = optimizer.get_optimization_params("test_model");

        assert_eq!(params.batch_size, 1);
        assert!(matches!(params.precision, PrecisionMode::Half));
        assert!(matches!(params.memory_strategy, MemoryStrategy::Pooled));
    }

    #[test]
    fn test_performance_recording() {
        let mut optimizer = ANEOptimizer::new(ANEOptimizationStrategy::Balanced);

        optimizer.record_performance("test_model", 50.0);
        optimizer.record_performance("test_model", 45.0);
        optimizer.record_performance("test_model", 55.0);

        let stats = optimizer.get_performance_stats("test_model").unwrap();
        assert_eq!(stats.count, 3);
        assert!((stats.average_ms - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_memory_optimizer() {
        let mem_optimizer = ANEMemoryOptimizer::new();

        assert!(mem_optimizer.should_optimize_memory(600.0));
        assert!(!mem_optimizer.should_optimize_memory(400.0));

        let recommendations = mem_optimizer.get_memory_recommendations(600.0);
        assert!(!recommendations.is_empty());
    }

    #[test]
    fn test_batch_optimizer() {
        let mut batch_optimizer = BatchOptimizer::new(8);

        // Should increase batch size with good performance
        let new_size = batch_optimizer.optimize_batch_size(15.0, 80.0);
        assert_eq!(new_size, 2);

        // Should decrease batch size with poor latency
        let new_size = batch_optimizer.optimize_batch_size(5.0, 250.0);
        assert_eq!(new_size, 1);
    }
}
