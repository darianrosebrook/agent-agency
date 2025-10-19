//! Quantization laboratory for model compression and optimization
//!
//! Supports multiple quantization strategies:
//! - FP16 (half-precision floating point)
//! - INT8 (8-bit integer quantization)
//! - INT4 (4-bit integer quantization with grouping)
//! - Pruning (weight sparsification)
//! - Mixed precision (per-layer optimization)
//!
//! Tracks compression metrics and accuracy deltas.
//!
//! @author @darianrosebrook

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Quantization data type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuantizationType {
    /// Full precision floating point (32-bit)
    FP32,
    /// Half precision floating point (16-bit)
    FP16,
    /// 8-bit integer quantization
    INT8,
    /// 4-bit integer quantization with grouping
    INT4,
}

impl QuantizationType {
    /// Get bytes per element
    pub fn bytes_per_element(&self) -> usize {
        match self {
            QuantizationType::FP32 => 4,
            QuantizationType::FP16 => 2,
            QuantizationType::INT8 => 1,
            QuantizationType::INT4 => 1, // packed, so 2 values per byte
        }
    }

    /// Get compression ratio vs FP32
    pub fn compression_ratio(&self) -> f32 {
        match self {
            QuantizationType::FP32 => 1.0,
            QuantizationType::FP16 => 0.5,
            QuantizationType::INT8 => 0.25,
            QuantizationType::INT4 => 0.125, // 8:1 compression
        }
    }
}

/// Quantization strategy
#[derive(Debug, Clone)]
pub struct QuantizationStrategy {
    /// Target quantization type
    pub target_type: QuantizationType,
    /// Calibration dataset size (number of samples)
    pub calibration_samples: usize,
    /// Per-channel quantization for better accuracy
    pub per_channel: bool,
    /// Enable mixed precision (INT8 for weights, FP16 for activations)
    pub mixed_precision: bool,
    /// Pruning threshold (0.0 = no pruning, 1.0 = aggressive)
    pub pruning_threshold: f32,
}

impl Default for QuantizationStrategy {
    fn default() -> Self {
        Self {
            target_type: QuantizationType::INT8,
            calibration_samples: 100,
            per_channel: true,
            mixed_precision: false,
            pruning_threshold: 0.0,
        }
    }
}

/// Quantization metrics for a model
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuantizationMetrics {
    /// Original model size in MB
    pub original_size_mb: f32,
    /// Compressed model size in MB
    pub compressed_size_mb: f32,
    /// Actual compression ratio achieved
    pub achieved_compression_ratio: f32,
    /// Baseline accuracy (0.0-1.0)
    pub baseline_accuracy: f32,
    /// Quantized accuracy (0.0-1.0)
    pub quantized_accuracy: f32,
    /// Accuracy loss percentage
    pub accuracy_loss_percent: f32,
    /// Baseline inference latency (ms)
    pub baseline_latency_ms: f32,
    /// Quantized inference latency (ms)
    pub quantized_latency_ms: f32,
    /// Speedup factor (baseline / quantized)
    pub speedup_factor: f32,
    /// Pruning percentage of weights removed (0-100)
    pub pruning_percentage: f32,
    /// Quantization type applied
    pub quantization_type: Option<QuantizationType>,
}

impl QuantizationMetrics {
    /// Calculate compression efficiency (speedup / size_reduction)
    pub fn efficiency_score(&self) -> f32 {
        if self.achieved_compression_ratio <= 0.0 {
            return 0.0;
        }
        self.speedup_factor / self.achieved_compression_ratio
    }

    /// Check if quantization meets acceptable accuracy threshold
    pub fn meets_accuracy_threshold(&self, threshold_percent: f32) -> bool {
        self.accuracy_loss_percent <= threshold_percent
    }

    /// Check if speedup is worthwhile
    pub fn has_meaningful_speedup(&self, min_speedup: f32) -> bool {
        self.speedup_factor >= min_speedup
    }
}

/// Quantization result for a model variant
#[derive(Debug, Clone)]
pub struct QuantizationResult {
    /// Model variant ID
    pub variant_id: String,
    /// Quantization metrics
    pub metrics: QuantizationMetrics,
    /// Strategy used
    pub strategy: QuantizationStrategy,
    /// Whether quantization is recommended for production
    pub production_ready: bool,
    /// Recommendations for further optimization
    pub recommendations: Vec<String>,
}

/// Quantization laboratory for managing compression experiments
pub struct QuantizationLab {
    /// Results of quantization experiments
    results: Arc<RwLock<HashMap<String, QuantizationResult>>>,
    /// Baselines for comparison
    baselines: Arc<RwLock<HashMap<String, QuantizationMetrics>>>,
}

impl QuantizationLab {
    /// Create a new quantization laboratory
    pub fn new() -> Self {
        Self {
            results: Arc::new(RwLock::new(HashMap::new())),
            baselines: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a baseline model
    pub async fn register_baseline(
        &self,
        model_id: &str,
        metrics: QuantizationMetrics,
    ) -> Result<()> {
        let mut baselines = self.baselines.write().await;
        baselines.insert(model_id.to_string(), metrics);
        Ok(())
    }

    /// Run quantization experiment
    pub async fn quantize(
        &self,
        variant_id: &str,
        baseline_metrics: QuantizationMetrics,
        strategy: QuantizationStrategy,
    ) -> Result<QuantizationResult> {
        // Simulate quantization (in production, this would call coremltools)
        let compressed_size =
            baseline_metrics.original_size_mb * strategy.target_type.compression_ratio();
        let compression_ratio = baseline_metrics.original_size_mb / compressed_size.max(0.001);

        // Estimate accuracy loss based on quantization type
        let accuracy_loss = match strategy.target_type {
            QuantizationType::FP32 => 0.0,
            QuantizationType::FP16 => 0.1, // ~0.1% loss
            QuantizationType::INT8 => 0.5, // ~0.5% loss
            QuantizationType::INT4 => 2.0, // ~2% loss
        };

        // With per-channel quantization, reduce accuracy loss
        let final_accuracy_loss = if strategy.per_channel {
            accuracy_loss * 0.7
        } else {
            accuracy_loss
        };

        let quantized_accuracy =
            (baseline_metrics.baseline_accuracy * (1.0 - final_accuracy_loss / 100.0)).max(0.0);

        // Estimate latency improvement
        let speedup = match strategy.target_type {
            QuantizationType::FP32 => 1.0,
            QuantizationType::FP16 => 1.5, // 1.5x faster
            QuantizationType::INT8 => 3.0, // 3x faster
            QuantizationType::INT4 => 4.0, // 4x faster (with grouping)
        };

        let quantized_latency = baseline_metrics.baseline_latency_ms / speedup;

        let metrics = QuantizationMetrics {
            original_size_mb: baseline_metrics.original_size_mb,
            compressed_size_mb: compressed_size,
            achieved_compression_ratio: compression_ratio,
            baseline_accuracy: baseline_metrics.baseline_accuracy,
            quantized_accuracy,
            accuracy_loss_percent: final_accuracy_loss,
            baseline_latency_ms: baseline_metrics.baseline_latency_ms,
            quantized_latency_ms: quantized_latency,
            speedup_factor: speedup,
            pruning_percentage: strategy.pruning_threshold * 30.0, // up to 30% pruning
            quantization_type: Some(strategy.target_type),
        };

        // Generate recommendations
        let mut recommendations = Vec::new();
        if metrics.accuracy_loss_percent > 1.0 {
            recommendations
                .push("Consider per-channel quantization to reduce accuracy loss".to_string());
        }
        if metrics.achieved_compression_ratio < 2.0 {
            recommendations.push("Try INT4 or pruning for better compression".to_string());
        }
        if metrics.speedup_factor < 2.0 {
            recommendations.push("Mixed precision may yield better speedup".to_string());
        }

        let production_ready =
            metrics.meets_accuracy_threshold(1.0) && metrics.has_meaningful_speedup(1.5);

        let result = QuantizationResult {
            variant_id: variant_id.to_string(),
            metrics,
            strategy,
            production_ready,
            recommendations,
        };

        let mut results = self.results.write().await;
        results.insert(variant_id.to_string(), result.clone());

        Ok(result)
    }

    /// Get quantization result
    pub async fn get_result(&self, variant_id: &str) -> Option<QuantizationResult> {
        let results = self.results.read().await;
        results.get(variant_id).cloned()
    }

    /// Compare two quantization variants
    pub async fn compare_variants(&self, variant_a: &str, variant_b: &str) -> Result<String> {
        let results = self.results.read().await;

        let result_a = results
            .get(variant_a)
            .ok_or_else(|| anyhow::anyhow!("Variant {} not found", variant_a))?;
        let result_b = results
            .get(variant_b)
            .ok_or_else(|| anyhow::anyhow!("Variant {} not found", variant_b))?;

        let comparison = format!(
            "Variant Comparison:\n\
             {}: {:.2}x speedup, {:.1}% accuracy loss, {:.2}x compression\n\
             {}: {:.2}x speedup, {:.1}% accuracy loss, {:.2}x compression\n\
             Efficiency (A): {:.2} | Efficiency (B): {:.2}",
            variant_a,
            result_a.metrics.speedup_factor,
            result_a.metrics.accuracy_loss_percent,
            result_a.metrics.achieved_compression_ratio,
            variant_b,
            result_b.metrics.speedup_factor,
            result_b.metrics.accuracy_loss_percent,
            result_b.metrics.achieved_compression_ratio,
            result_a.metrics.efficiency_score(),
            result_b.metrics.efficiency_score(),
        );

        Ok(comparison)
    }

    /// Get all results
    pub async fn get_all_results(&self) -> HashMap<String, QuantizationResult> {
        let results = self.results.read().await;
        results.clone()
    }

    /// Recommend best quantization for target constraints
    pub async fn recommend_quantization(
        &self,
        baseline_metrics: QuantizationMetrics,
        max_accuracy_loss_percent: f32,
        min_speedup: f32,
    ) -> Result<QuantizationStrategy> {
        // FP16: smallest accuracy loss, moderate speedup
        let fp16_strategy = QuantizationStrategy {
            target_type: QuantizationType::FP16,
            calibration_samples: 50,
            per_channel: false,
            mixed_precision: false,
            pruning_threshold: 0.0,
        };

        // INT8: balanced accuracy/speed
        let int8_strategy = QuantizationStrategy {
            target_type: QuantizationType::INT8,
            calibration_samples: 100,
            per_channel: true,
            mixed_precision: false,
            pruning_threshold: 0.1,
        };

        // INT4 with pruning: aggressive compression
        let int4_strategy = QuantizationStrategy {
            target_type: QuantizationType::INT4,
            calibration_samples: 200,
            per_channel: true,
            mixed_precision: true,
            pruning_threshold: 0.3,
        };

        // Recommend based on constraints
        if max_accuracy_loss_percent >= 2.0 && min_speedup >= 4.0 {
            Ok(int4_strategy)
        } else if max_accuracy_loss_percent >= 0.5 && min_speedup >= 3.0 {
            Ok(int8_strategy)
        } else {
            Ok(fp16_strategy)
        }
    }
}

impl Clone for QuantizationLab {
    fn clone(&self) -> Self {
        Self {
            results: Arc::clone(&self.results),
            baselines: Arc::clone(&self.baselines),
        }
    }
}

impl Default for QuantizationLab {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantization_type_compression() {
        assert_eq!(QuantizationType::FP32.compression_ratio(), 1.0);
        assert_eq!(QuantizationType::FP16.compression_ratio(), 0.5);
        assert_eq!(QuantizationType::INT8.compression_ratio(), 0.25);
        assert_eq!(QuantizationType::INT4.compression_ratio(), 0.125);
    }

    #[test]
    fn test_quantization_type_bytes() {
        assert_eq!(QuantizationType::FP32.bytes_per_element(), 4);
        assert_eq!(QuantizationType::FP16.bytes_per_element(), 2);
        assert_eq!(QuantizationType::INT8.bytes_per_element(), 1);
        assert_eq!(QuantizationType::INT4.bytes_per_element(), 1);
    }

    #[test]
    fn test_quantization_metrics_efficiency() {
        let mut metrics = QuantizationMetrics::default();
        metrics.speedup_factor = 2.0;
        metrics.achieved_compression_ratio = 2.0;
        assert_eq!(metrics.efficiency_score(), 1.0);
    }

    #[tokio::test]
    async fn test_quantization_lab_basic() {
        let lab = QuantizationLab::new();

        let baseline = QuantizationMetrics {
            original_size_mb: 100.0,
            baseline_accuracy: 0.95,
            baseline_latency_ms: 50.0,
            ..Default::default()
        };

        let strategy = QuantizationStrategy {
            target_type: QuantizationType::INT8,
            ..Default::default()
        };

        let result = lab
            .quantize("test-variant", baseline, strategy)
            .await
            .unwrap();

        assert_eq!(result.variant_id, "test-variant");
        assert!(result.metrics.compressed_size_mb < 100.0);
        assert!(result.metrics.speedup_factor > 1.0);
    }

    #[tokio::test]
    async fn test_quantization_lab_accuracy_threshold() {
        let baseline = QuantizationMetrics {
            original_size_mb: 100.0,
            baseline_accuracy: 0.95,
            baseline_latency_ms: 50.0,
            speedup_factor: 1.5, // Add speedup for test
            ..Default::default()
        };

        assert!(baseline.meets_accuracy_threshold(10.0));
        assert!(baseline.has_meaningful_speedup(1.0));
    }

    #[tokio::test]
    async fn test_quantization_strategy_default() {
        let strategy = QuantizationStrategy::default();
        assert_eq!(strategy.target_type, QuantizationType::INT8);
        assert_eq!(strategy.calibration_samples, 100);
        assert!(strategy.per_channel);
    }
}
