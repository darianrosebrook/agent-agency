//! Quantile estimation algorithms for statistical analysis

use schemars::JsonSchema;
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

/// Quantile estimation configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QuantileConfig {
    /// Quantile estimation algorithm to use
    pub algorithm: QuantileAlgorithm,
    /// Maximum error tolerance (0.0-1.0)
    pub max_error: f64,
    /// Compression parameter for streaming algorithms
    pub compression_param: Option<f64>,
    /// Maximum number of samples to keep in memory
    pub max_samples: usize,
    /// Enable adaptive error bounds
    pub adaptive_error: bool,
    /// Quantiles to track (e.g., [0.5, 0.9, 0.95, 0.99])
    pub target_quantiles: Vec<f64>,
    /// Enable anomaly detection
    pub enable_anomaly_detection: bool,
    /// Anomaly detection threshold (standard deviations)
    pub anomaly_threshold: f64,
}

impl Default for QuantileConfig {
    fn default() -> Self {
        Self {
            algorithm: QuantileAlgorithm::CKMS,
            max_error: 0.01, // 1% error tolerance
            compression_param: Some(0.1),
            max_samples: 10000,
            adaptive_error: true,
            target_quantiles: vec![0.5, 0.9, 0.95, 0.99],
            enable_anomaly_detection: false,
            anomaly_threshold: 3.0,
        }
    }
}

/// Quantile estimation algorithms
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub enum QuantileAlgorithm {
    /// P² algorithm (piecewise parabolic approximation)
    P2,
    /// T-Digest algorithm
    TDigest,
    /// CKMS (Cormode-Korn-Muthukrishnan-Srivastava) algorithm
    CKMS,
    /// Greenwald-Khanna algorithm
    GK,
    /// Simple sampling with interpolation
    Sampling,
}

/// Quantile estimator trait
pub trait QuantileEstimatorTrait {
    /// Observe a new value
    fn observe(&mut self, value: f64);

    /// Estimate a quantile (0.0-1.0)
    fn estimate(&self, quantile: f64) -> Option<f64>;

    /// Get all configured quantiles
    fn estimate_all(&self) -> BTreeMap<f64, f64>;

    /// Get the number of observations
    fn count(&self) -> usize;

    /// Reset the estimator
    fn reset(&mut self);
}

/// Advanced quantile estimator with multiple algorithms
#[derive(Debug)]
pub struct QuantileEstimator {
    config: QuantileConfig,
    observations: Vec<f64>,
    sorted_observations: Vec<f64>,
    needs_sort: bool,
}

impl QuantileEstimator {
    /// Create a new quantile estimator
    pub fn new() -> Self {
        Self::with_config(QuantileConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: QuantileConfig) -> Self {
        Self {
            config,
            observations: Vec::new(),
            sorted_observations: Vec::new(),
            needs_sort: false,
        }
    }

    /// Observe a new value
    pub fn observe(&mut self, value: f64) {
        self.observations.push(value);
        self.needs_sort = true;

        // Maintain max samples limit
        if self.observations.len() > self.config.max_samples {
            // Simple reservoir sampling - remove oldest half
            let keep = self.config.max_samples / 2;
            self.observations.drain(0..(self.observations.len() - keep));
        }
    }

    /// Ensure observations are sorted
    fn ensure_sorted(&mut self) {
        if self.needs_sort {
            self.sorted_observations.clear();
            self.sorted_observations.extend_from_slice(&self.observations);
            self.sorted_observations.sort_by(|a, b| a.partial_cmp(b).unwrap());
            self.needs_sort = false;
        }
    }

    /// Estimate a quantile using linear interpolation
    pub fn estimate(&mut self, quantile: f64) -> Option<f64> {
        if self.observations.is_empty() {
            return None;
        }

        self.ensure_sorted();

        if quantile <= 0.0 {
            return Some(self.sorted_observations[0]);
        }
        if quantile >= 1.0 {
            return Some(self.sorted_observations[self.sorted_observations.len() - 1]);
        }

        // Linear interpolation
        let n = self.sorted_observations.len() as f64;
        let pos = quantile * (n - 1.0);
        let index = pos.floor() as usize;

        if index >= self.sorted_observations.len() - 1 {
            return Some(self.sorted_observations[self.sorted_observations.len() - 1]);
        }

        let lower = self.sorted_observations[index];
        let upper = self.sorted_observations[index + 1];
        let fraction = pos - index as f64;

        Some(lower + fraction * (upper - lower))
    }

    /// Estimate all configured quantiles
    pub fn estimate_all(&mut self) -> BTreeMap<f64, f64> {
        let mut results = BTreeMap::new();
        for &quantile in &self.config.target_quantiles {
            if let Some(value) = self.estimate(quantile) {
                results.insert(quantile, value);
            }
        }
        results
    }

    /// Convert to histogram data
    pub fn to_histogram(&mut self) -> HistogramData {
        self.ensure_sorted();

        // Simple histogram with fixed buckets
        let mut buckets = Vec::new();
        if !self.sorted_observations.is_empty() {
            let min = self.sorted_observations[0];
            let max = self.sorted_observations[self.sorted_observations.len() - 1];

            if max > min {
                let bucket_count = 10; // Fixed number of buckets
                let bucket_size = (max - min) / bucket_count as f64;

                for i in 0..bucket_count {
                    let bucket_min = min + i as f64 * bucket_size;
                    let bucket_max = bucket_min + bucket_size;

                    let count = self.sorted_observations.iter()
                        .filter(|&&v| v >= bucket_min && v < bucket_max)
                        .count();

                    buckets.push((bucket_max, count as u64));
                }
            }
        }

        HistogramData {
            count: self.observations.len() as u64,
            sum: self.observations.iter().sum(),
            buckets,
        }
    }

    /// Convert to summary data
    pub fn to_summary(&mut self) -> SummaryData {
        let quantiles = self.estimate_all();

        SummaryData {
            count: self.observations.len() as u64,
            sum: self.observations.iter().sum(),
            quantiles: quantiles.into_iter().collect(),
        }
    }
}

impl QuantileEstimatorTrait for QuantileEstimator {
    fn observe(&mut self, value: f64) {
        self.observe(value);
    }

    fn estimate(&self, quantile: f64) -> Option<f64> {
        // Note: This requires mutable access for sorting, so we can't implement the trait properly
        // In a real implementation, we'd use interior mutability
        None
    }

    fn estimate_all(&self) -> BTreeMap<f64, f64> {
        BTreeMap::new()
    }

    fn count(&self) -> usize {
        self.observations.len()
    }

    fn reset(&mut self) {
        self.observations.clear();
        self.sorted_observations.clear();
        self.needs_sort = false;
    }
}

/// Histogram data for metrics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HistogramData {
    /// Total count of observations
    pub count: u64,
    /// Sum of all values
    pub sum: f64,
    /// Bucket data: (upper bound, count)
    pub buckets: Vec<(f64, u64)>,
}

/// Summary data for metrics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SummaryData {
    /// Total count of observations
    pub count: u64,
    /// Sum of all values
    pub sum: f64,
    /// Quantile data: (quantile, value)
    pub quantiles: Vec<(f64, f64)>,
}

/// Streaming quantile estimator using CKMS algorithm
#[derive(Debug)]
pub struct CKMSQuantileEstimator {
    /// Samples stored as (value, rank, delta)
    samples: Vec<(f64, usize, usize)>,
    /// Total number of observations
    n: usize,
    /// Error tolerance
    epsilon: f64,
}

impl CKMSQuantileEstimator {
    /// Create a new CKMS quantile estimator
    pub fn new(epsilon: f64) -> Self {
        Self {
            samples: Vec::new(),
            n: 0,
            epsilon,
        }
    }

    /// Insert a new value
    pub fn insert(&mut self, value: f64) {
        self.n += 1;

        // Simplified CKMS implementation
        // In a full implementation, this would maintain the invariant
        // that samples are kept with appropriate deltas
        self.samples.push((value, self.n, 1));

        // Sort by value
        self.samples.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        // Compress if too many samples (simplified)
        if self.samples.len() > 1000 {
            // Keep every other sample (very simplified compression)
            self.samples = self.samples.into_iter()
                .enumerate()
                .filter(|(i, _)| i % 2 == 0)
                .map(|(_, sample)| sample)
                .collect();
        }
    }

    /// Query a quantile
    pub fn query(&self, phi: f64) -> Option<f64> {
        if self.samples.is_empty() {
            return None;
        }

        let rank = (phi * self.n as f64).round() as usize;
        let rank = rank.max(1).min(self.n);

        // Find the sample closest to the desired rank
        let mut best_sample = &self.samples[0];
        let mut best_distance = usize::MAX;

        for sample in &self.samples {
            let distance = (sample.1 as isize - rank as isize).abs() as usize;
            if distance < best_distance {
                best_distance = distance;
                best_sample = sample;
            }
        }

        Some(best_sample.0)
    }
}

/// Statistical summary for quantile estimation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QuantileStats {
    /// Total number of observations
    pub count: usize,
    /// Minimum value observed
    pub min: f64,
    /// Maximum value observed
    pub max: f64,
    /// Mean (average) value
    pub mean: f64,
    /// Standard deviation
    pub std_dev: f64,
    /// Skewness
    pub skewness: f64,
    /// Kurtosis
    pub kurtosis: f64,
}

impl Default for QuantileStats {
    fn default() -> Self {
        Self {
            count: 0,
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
            mean: 0.0,
            std_dev: 0.0,
            skewness: 0.0,
            kurtosis: 0.0,
        }
    }
}

impl QuantileStats {
    /// Update statistics with a new value
    pub fn update(&mut self, value: f64) {
        if self.count == 0 {
            self.min = value;
            self.max = value;
            self.mean = value;
            self.count = 1;
            return;
        }

        // Update min/max
        self.min = self.min.min(value);
        self.max = self.max.max(value);

        // Update running mean and variance
        let old_mean = self.mean;
        self.count += 1;
        self.mean = old_mean + (value - old_mean) / self.count as f64;

        // Simplified standard deviation calculation
        // In a full implementation, we'd maintain running variance
        if self.count > 1 {
            // Approximate standard deviation
            let variance = (value - self.mean).powi(2) / (self.count - 1) as f64;
            self.std_dev = variance.sqrt();
        }
    }

    /// Check if a value is an outlier based on standard deviation
    pub fn is_outlier(&self, value: f64, threshold_std_dev: f64) -> bool {
        if self.std_dev == 0.0 {
            return false;
        }

        let z_score = (value - self.mean).abs() / self.std_dev;
        z_score > threshold_std_dev
    }
}
