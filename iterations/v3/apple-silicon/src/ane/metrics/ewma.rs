//! Exponential Weighted Moving Average (EWMA) metrics
//!
//! This module provides EWMA calculation utilities for performance monitoring
//! and adaptive metrics tracking in ANE operations.

/// EWMA calculation utilities
pub struct Ewma;

impl Ewma {
    /// Update EWMA with a new sample
    /// 
    /// # Arguments
    /// * `prev` - Previous EWMA value
    /// * `sample` - New sample value
    /// * `alpha` - Smoothing factor (0.0-1.0, higher = more responsive)
    /// 
    /// # Returns
    /// * Updated EWMA value
    /// 
    /// # Formula
    /// `new_ewma = alpha * sample + (1 - alpha) * prev`
    pub fn update(prev: f64, sample: f64, alpha: f64) -> f64 {
        alpha * sample + (1.0 - alpha) * prev
    }
    
    /// Calculate EWMA with bounds checking
    /// 
    /// # Arguments
    /// * `prev` - Previous EWMA value
    /// * `sample` - New sample value
    /// * `alpha` - Smoothing factor (0.0-1.0)
    /// * `min_value` - Minimum allowed value
    /// * `max_value` - Maximum allowed value
    /// 
    /// # Returns
    /// * Updated EWMA value clamped to bounds
    pub fn update_bounded(
        prev: f64,
        sample: f64,
        alpha: f64,
        min_value: f64,
        max_value: f64,
    ) -> f64 {
        let new_value = Self::update(prev, sample, alpha);
        new_value.clamp(min_value, max_value)
    }
    
    /// Calculate EWMA with adaptive alpha based on sample variance
    /// 
    /// # Arguments
    /// * `prev` - Previous EWMA value
    /// * `sample` - New sample value
    /// * `base_alpha` - Base smoothing factor
    /// * `variance` - Sample variance (higher = more adaptive)
    /// 
    /// # Returns
    /// * Updated EWMA value with adaptive alpha
    pub fn update_adaptive(
        prev: f64,
        sample: f64,
        base_alpha: f64,
        variance: f64,
    ) -> f64 {
        // Adaptive alpha: higher variance = higher alpha (more responsive)
        let adaptive_alpha = base_alpha * (1.0 + variance).min(2.0);
        Self::update(prev, sample, adaptive_alpha)
    }
    
    /// Calculate EWMA with outlier detection
    /// 
    /// # Arguments
    /// * `prev` - Previous EWMA value
    /// * `sample` - New sample value
    /// * `alpha` - Smoothing factor
    /// * `outlier_threshold` - Threshold for outlier detection (standard deviations)
    /// 
    /// # Returns
    /// * Updated EWMA value (outliers are weighted less)
    pub fn update_with_outlier_detection(
        prev: f64,
        sample: f64,
        alpha: f64,
        outlier_threshold: f64,
    ) -> f64 {
        // Simple outlier detection based on deviation from previous value
        let deviation = (sample - prev).abs();
        let threshold = prev * outlier_threshold;
        
        if deviation > threshold {
            // Outlier detected, use reduced alpha
            let reduced_alpha = alpha * 0.5;
            Self::update(prev, sample, reduced_alpha)
        } else {
            // Normal sample, use full alpha
            Self::update(prev, sample, alpha)
        }
    }
    
    /// Calculate EWMA with trend detection
    /// 
    /// # Arguments
    /// * `prev` - Previous EWMA value
    /// * `sample` - New sample value
    /// * `alpha` - Smoothing factor
    /// * `trend_samples` - Number of recent samples for trend calculation
    /// 
    /// # Returns
    /// * Updated EWMA value with trend adjustment
    pub fn update_with_trend(
        prev: f64,
        sample: f64,
        alpha: f64,
        trend_samples: &[f64],
    ) -> f64 {
        if trend_samples.len() < 2 {
            return Self::update(prev, sample, alpha);
        }
        
        // Calculate trend (simple linear regression slope)
        let n = trend_samples.len() as f64;
        let sum_x = (0..trend_samples.len()).map(|i| i as f64).sum::<f64>();
        let sum_y = trend_samples.iter().sum::<f64>();
        let sum_xy = trend_samples.iter().enumerate()
            .map(|(i, &y)| i as f64 * y)
            .sum::<f64>();
        let sum_x2 = (0..trend_samples.len()).map(|i| (i as f64).powi(2)).sum::<f64>();
        
        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        
        // Adjust alpha based on trend strength
        let trend_factor = 1.0 + slope.abs() * 0.1; // 10% adjustment per unit slope
        let adjusted_alpha = alpha * trend_factor.min(2.0);
        
        Self::update(prev, sample, adjusted_alpha)
    }
    
    /// Calculate EWMA with seasonal adjustment
    /// 
    /// # Arguments
    /// * `prev` - Previous EWMA value
    /// * `sample` - New sample value
    /// * `alpha` - Smoothing factor
    /// * `seasonal_factor` - Seasonal adjustment factor (0.5-2.0)
    /// 
    /// # Returns
    /// * Updated EWMA value with seasonal adjustment
    pub fn update_seasonal(
        prev: f64,
        sample: f64,
        alpha: f64,
        seasonal_factor: f64,
    ) -> f64 {
        let adjusted_sample = sample * seasonal_factor;
        Self::update(prev, adjusted_sample, alpha)
    }
    
    /// Calculate EWMA with confidence weighting
    /// 
    /// # Arguments
    /// * `prev` - Previous EWMA value
    /// * `sample` - New sample value
    /// * `alpha` - Smoothing factor
    /// * `confidence` - Confidence in the sample (0.0-1.0)
    /// 
    /// # Returns
    /// * Updated EWMA value with confidence weighting
    pub fn update_with_confidence(
        prev: f64,
        sample: f64,
        alpha: f64,
        confidence: f64,
    ) -> f64 {
        let weighted_alpha = alpha * confidence;
        Self::update(prev, sample, weighted_alpha)
    }
    
    /// Calculate EWMA with multiple samples (batch update)
    /// 
    /// # Arguments
    /// * `prev` - Previous EWMA value
    /// * `samples` - New sample values
    /// * `alpha` - Smoothing factor
    /// 
    /// # Returns
    /// * Updated EWMA value
    pub fn update_batch(prev: f64, samples: &[f64], alpha: f64) -> f64 {
        if samples.is_empty() {
            return prev;
        }
        
        // Calculate average of samples
        let avg_sample = samples.iter().sum::<f64>() / samples.len() as f64;
        Self::update(prev, avg_sample, alpha)
    }
    
    /// Calculate EWMA with weighted samples
    /// 
    /// # Arguments
    /// * `prev` - Previous EWMA value
    /// * `samples` - New sample values
    /// * `weights` - Weights for each sample
    /// * `alpha` - Smoothing factor
    /// 
    /// # Returns
    /// * Updated EWMA value
    pub fn update_weighted(
        prev: f64,
        samples: &[f64],
        weights: &[f64],
        alpha: f64,
    ) -> f64 {
        if samples.is_empty() || samples.len() != weights.len() {
            return prev;
        }
        
        // Calculate weighted average
        let total_weight = weights.iter().sum::<f64>();
        if total_weight == 0.0 {
            return prev;
        }
        
        let weighted_sum = samples.iter().zip(weights.iter())
            .map(|(s, w)| s * w)
            .sum::<f64>();
        let weighted_avg = weighted_sum / total_weight;
        
        Self::update(prev, weighted_avg, alpha)
    }
}

/// EWMA-based performance metrics tracker
#[derive(Debug, Clone)]
pub struct PerformanceTracker {
    /// Latency EWMA
    pub latency_ewma: f64,
    /// Throughput EWMA
    pub throughput_ewma: f64,
    /// Memory usage EWMA
    pub memory_ewma: f64,
    /// Error rate EWMA
    pub error_rate_ewma: f64,
    /// Alpha values for each metric
    pub alphas: MetricAlphas,
    /// Sample counts
    pub sample_counts: MetricCounts,
}

/// Alpha values for different metrics
#[derive(Debug, Clone)]
pub struct MetricAlphas {
    pub latency: f64,
    pub throughput: f64,
    pub memory: f64,
    pub error_rate: f64,
}

impl Default for MetricAlphas {
    fn default() -> Self {
        Self {
            latency: 0.1,      // More responsive to latency changes
            throughput: 0.05,  // Less responsive to throughput changes
            memory: 0.2,       // Moderately responsive to memory changes
            error_rate: 0.3,   // Very responsive to error rate changes
        }
    }
}

/// Sample counts for metrics
#[derive(Debug, Clone)]
pub struct MetricCounts {
    pub latency: u64,
    pub throughput: u64,
    pub memory: u64,
    pub error_rate: u64,
}

impl Default for MetricCounts {
    fn default() -> Self {
        Self {
            latency: 0,
            throughput: 0,
            memory: 0,
            error_rate: 0,
        }
    }
}

impl PerformanceTracker {
    /// Create a new performance tracker
    pub fn new() -> Self {
        Self {
            latency_ewma: 0.0,
            throughput_ewma: 0.0,
            memory_ewma: 0.0,
            error_rate_ewma: 0.0,
            alphas: MetricAlphas::default(),
            sample_counts: MetricCounts::default(),
        }
    }
    
    /// Update latency metric
    pub fn update_latency(&mut self, latency_ms: f64) {
        self.latency_ewma = Ewma::update(
            self.latency_ewma,
            latency_ms,
            self.alphas.latency,
        );
        self.sample_counts.latency += 1;
    }
    
    /// Update throughput metric
    pub fn update_throughput(&mut self, throughput_ips: f64) {
        self.throughput_ewma = Ewma::update(
            self.throughput_ewma,
            throughput_ips,
            self.alphas.throughput,
        );
        self.sample_counts.throughput += 1;
    }
    
    /// Update memory usage metric
    pub fn update_memory(&mut self, memory_mb: f64) {
        self.memory_ewma = Ewma::update(
            self.memory_ewma,
            memory_mb,
            self.alphas.memory,
        );
        self.sample_counts.memory += 1;
    }
    
    /// Update error rate metric
    pub fn update_error_rate(&mut self, error_rate: f64) {
        self.error_rate_ewma = Ewma::update(
            self.error_rate_ewma,
            error_rate,
            self.alphas.error_rate,
        );
        self.sample_counts.error_rate += 1;
    }
    
    /// Get current performance summary
    pub fn get_summary(&self) -> PerformanceSummary {
        PerformanceSummary {
            latency_ms: self.latency_ewma,
            throughput_ips: self.throughput_ewma,
            memory_mb: self.memory_ewma,
            error_rate: self.error_rate_ewma,
            sample_counts: self.sample_counts.clone(),
        }
    }
}

/// Performance summary
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub latency_ms: f64,
    pub throughput_ips: f64,
    pub memory_mb: f64,
    pub error_rate: f64,
    pub sample_counts: MetricCounts,
}

impl Default for PerformanceTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ewma_basic() {
        let prev = 10.0;
        let sample = 20.0;
        let alpha = 0.5;
        
        let result = Ewma::update(prev, sample, alpha);
        assert_eq!(result, 15.0); // 0.5 * 20 + 0.5 * 10 = 15
    }

    #[test]
    fn test_ewma_bounded() {
        let prev = 10.0;
        let sample = 20.0;
        let alpha = 0.5;
        let min_val = 5.0;
        let max_val = 15.0;
        
        let result = Ewma::update_bounded(prev, sample, alpha, min_val, max_val);
        assert_eq!(result, 15.0); // Clamped to max_val
    }

    #[test]
    fn test_ewma_adaptive() {
        let prev = 10.0;
        let sample = 20.0;
        let base_alpha = 0.1;
        let variance = 0.5;
        
        let result = Ewma::update_adaptive(prev, sample, base_alpha, variance);
        assert!(result > 10.0); // Should be higher than prev
        assert!(result < 20.0); // Should be lower than sample
    }

    #[test]
    fn test_ewma_outlier_detection() {
        let prev = 10.0;
        let sample = 100.0; // Outlier
        let alpha = 0.1;
        let threshold = 2.0; // 2 standard deviations
        
        let result = Ewma::update_with_outlier_detection(prev, sample, alpha, threshold);
        // Should be less affected by the outlier
        assert!(result < 20.0);
    }

    #[test]
    fn test_ewma_trend() {
        let prev = 10.0;
        let sample = 20.0;
        let alpha = 0.1;
        let trend_samples = vec![5.0, 8.0, 12.0, 15.0]; // Upward trend
        
        let result = Ewma::update_with_trend(prev, sample, alpha, &trend_samples);
        assert!(result > 10.0);
    }

    #[test]
    fn test_ewma_seasonal() {
        let prev = 10.0;
        let sample = 20.0;
        let alpha = 0.1;
        let seasonal_factor = 1.5;
        
        let result = Ewma::update_seasonal(prev, sample, alpha, seasonal_factor);
        assert!(result > 10.0);
    }

    #[test]
    fn test_ewma_confidence() {
        let prev = 10.0;
        let sample = 20.0;
        let alpha = 0.1;
        let confidence = 0.5;
        
        let result = Ewma::update_with_confidence(prev, sample, alpha, confidence);
        assert!(result > 10.0);
        assert!(result < 20.0);
    }

    #[test]
    fn test_ewma_batch() {
        let prev = 10.0;
        let samples = vec![15.0, 20.0, 25.0];
        let alpha = 0.1;
        
        let result = Ewma::update_batch(prev, &samples, alpha);
        assert!(result > 10.0);
        assert!(result < 25.0);
    }

    #[test]
    fn test_ewma_weighted() {
        let prev = 10.0;
        let samples = vec![15.0, 20.0];
        let weights = vec![0.3, 0.7];
        let alpha = 0.1;
        
        let result = Ewma::update_weighted(prev, &samples, &weights, alpha);
        assert!(result > 10.0);
    }

    #[test]
    fn test_performance_tracker() {
        let mut tracker = PerformanceTracker::new();
        
        tracker.update_latency(50.0);
        tracker.update_throughput(10.0);
        tracker.update_memory(100.0);
        tracker.update_error_rate(0.01);
        
        let summary = tracker.get_summary();
        assert!(summary.latency_ms > 0.0);
        assert!(summary.throughput_ips > 0.0);
        assert!(summary.memory_mb > 0.0);
        assert!(summary.error_rate > 0.0);
        assert_eq!(summary.sample_counts.latency, 1);
        assert_eq!(summary.sample_counts.throughput, 1);
        assert_eq!(summary.sample_counts.memory, 1);
        assert_eq!(summary.sample_counts.error_rate, 1);
    }

    #[test]
    fn test_performance_tracker_multiple_updates() {
        let mut tracker = PerformanceTracker::new();
        
        // Update multiple times
        for i in 0..10 {
            tracker.update_latency(50.0 + i as f64);
            tracker.update_throughput(10.0 + i as f64);
            tracker.update_memory(100.0 + i as f64);
            tracker.update_error_rate(0.01 + i as f64 * 0.001);
        }
        
        let summary = tracker.get_summary();
        assert_eq!(summary.sample_counts.latency, 10);
        assert_eq!(summary.sample_counts.throughput, 10);
        assert_eq!(summary.sample_counts.memory, 10);
        assert_eq!(summary.sample_counts.error_rate, 10);
    }
}
