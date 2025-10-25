//! Quality metrics and evaluation for Apple Silicon inference results

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Quality metrics for inference results and model performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Overall quality score (0.0-1.0, higher is better)
    pub overall_score: f32,
    /// Accuracy score (0.0-1.0, higher is better)
    pub accuracy_score: f32,
    /// Precision score (0.0-1.0, higher is better)
    pub precision_score: f32,
    /// Recall score (0.0-1.0, higher is better)
    pub recall_score: f32,
    /// F1 score (harmonic mean of precision and recall)
    pub f1_score: f32,
    /// Confidence score (0.0-1.0, higher is better)
    pub confidence_score: f32,
    /// Latency in milliseconds (lower is better)
    pub latency_ms: u64,
    /// Throughput in inferences per second (higher is better)
    pub throughput_inferences_per_sec: f32,
    /// Memory efficiency score (0.0-1.0, higher is better)
    pub memory_efficiency: f32,
    /// Power efficiency score (0.0-1.0, higher is better)
    pub power_efficiency: f32,
    /// Model-specific metrics
    pub model_metrics: HashMap<String, f32>,
    /// Additional quality indicators
    pub custom_metrics: HashMap<String, serde_json::Value>,
}

impl QualityMetrics {
    /// Create quality metrics with default values
    pub fn new() -> Self {
        Self {
            overall_score: 0.0,
            accuracy_score: 0.0,
            precision_score: 0.0,
            recall_score: 0.0,
            f1_score: 0.0,
            confidence_score: 0.0,
            latency_ms: 0,
            throughput_inferences_per_sec: 0.0,
            memory_efficiency: 0.0,
            power_efficiency: 0.0,
            model_metrics: HashMap::new(),
            custom_metrics: HashMap::new(),
        }
    }

    /// Calculate F1 score from precision and recall
    pub fn calculate_f1_score(&mut self) {
        if self.precision_score + self.recall_score > 0.0 {
            self.f1_score = 2.0 * self.precision_score * self.recall_score /
                           (self.precision_score + self.recall_score);
        } else {
            self.f1_score = 0.0;
        }
    }

    /// Calculate overall quality score as weighted average
    pub fn calculate_overall_score(&mut self) {
        let weights = [
            (self.accuracy_score, 0.3),
            (self.f1_score, 0.3),
            (self.confidence_score, 0.2),
            (self.memory_efficiency, 0.1),
            (self.power_efficiency, 0.1),
        ];

        let weighted_sum: f32 = weights.iter()
            .map(|(score, weight)| score * weight)
            .sum();

        self.overall_score = weighted_sum;
    }

    /// Check if quality metrics meet minimum thresholds
    pub fn meets_thresholds(&self, thresholds: &QualityThresholds) -> bool {
        self.overall_score >= thresholds.min_overall_score &&
        self.accuracy_score >= thresholds.min_accuracy_score &&
        self.f1_score >= thresholds.min_f1_score &&
        self.latency_ms <= thresholds.max_latency_ms
    }

    /// Add a custom metric
    pub fn add_custom_metric(&mut self, name: String, value: serde_json::Value) {
        self.custom_metrics.insert(name, value);
    }

    /// Get a custom metric as float
    pub fn get_custom_metric_as_float(&self, name: &str) -> Option<f32> {
        self.custom_metrics.get(name)
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
    }
}

/// Quality thresholds for evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityThresholds {
    /// Minimum overall quality score
    pub min_overall_score: f32,
    /// Minimum accuracy score
    pub min_accuracy_score: f32,
    /// Minimum F1 score
    pub min_f1_score: f32,
    /// Maximum acceptable latency in milliseconds
    pub max_latency_ms: u64,
    /// Minimum throughput requirement
    pub min_throughput_inferences_per_sec: f32,
    /// Minimum memory efficiency
    pub min_memory_efficiency: f32,
}

impl Default for QualityThresholds {
    fn default() -> Self {
        Self {
            min_overall_score: 0.8,
            min_accuracy_score: 0.85,
            min_f1_score: 0.8,
            max_latency_ms: 100,
            min_throughput_inferences_per_sec: 10.0,
            min_memory_efficiency: 0.7,
        }
    }
}

/// Benchmark results for model performance evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Model name being benchmarked
    pub model_name: String,
    /// Hardware target used
    pub hardware_target: String,
    /// Quality metrics from the benchmark
    pub quality_metrics: QualityMetrics,
    /// Resource usage during benchmark
    pub resource_usage: super::resources::ResourceUsage,
    /// Benchmark configuration
    pub config: BenchmarkConfig,
    /// Timestamp when benchmark was run
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Benchmark configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    /// Number of iterations to run
    pub iterations: u32,
    /// Warmup iterations before measurement
    pub warmup_iterations: u32,
    /// Input batch size
    pub batch_size: usize,
    /// Whether to enable detailed profiling
    pub enable_profiling: bool,
    /// Custom benchmark parameters
    pub custom_params: HashMap<String, serde_json::Value>,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            iterations: 100,
            warmup_iterations: 10,
            batch_size: 1,
            enable_profiling: false,
            custom_params: HashMap::new(),
        }
    }
}

/// Performance regression analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRegression {
    /// Metric that regressed
    pub metric_name: String,
    /// Previous value
    pub previous_value: f64,
    /// Current value
    pub current_value: f64,
    /// Percentage change (negative = regression)
    pub percentage_change: f64,
    /// Statistical significance (p-value)
    pub significance: f64,
    /// Whether this is considered a significant regression
    pub is_significant: bool,
    /// Timestamp of regression detection
    pub detected_at: chrono::DateTime<chrono::Utc>,
}

impl PerformanceRegression {
    /// Check if the change represents a regression (negative change)
    pub fn is_regression(&self) -> bool {
        self.percentage_change < 0.0
    }

    /// Check if the change represents an improvement (positive change)
    pub fn is_improvement(&self) -> bool {
        self.percentage_change > 0.0
    }

    /// Get severity level of the regression (0-1, higher = more severe)
    pub fn severity(&self) -> f32 {
        if self.is_regression() {
            (-self.percentage_change / 100.0).min(1.0) as f32
        } else {
            0.0
        }
    }
}
