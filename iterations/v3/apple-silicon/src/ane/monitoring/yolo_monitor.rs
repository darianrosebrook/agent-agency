//! YOLO Performance Monitoring and Optimization
//!
//! Provides comprehensive monitoring for YOLO inference performance,
//! ANE utilization, and optimization recommendations.

use crate::telemetry::TelemetryCollector;
use crate::ane::infer::execute::InferenceMetrics;
use crate::ane::errors::{ANEError, Result};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{info, warn, error};

/// YOLO-specific performance metrics
#[derive(Debug, Clone)]
pub struct YOLOPerformanceMetrics {
    /// Total inference time in milliseconds
    pub total_inference_time_ms: f64,
    /// ANE compute time (if available)
    pub ane_compute_time_ms: Option<f64>,
    /// CPU fallback time (if used)
    pub cpu_fallback_time_ms: Option<f64>,
    /// Memory usage in MB
    pub memory_usage_mb: f64,
    /// ANE utilization percentage (0-100)
    pub ane_utilization_percent: Option<f32>,
    /// Number of objects detected
    pub objects_detected: usize,
    /// Preprocessing time in milliseconds
    pub preprocessing_time_ms: f64,
    /// Postprocessing time in milliseconds
    pub postprocessing_time_ms: f64,
    /// Model loading time (first inference only)
    pub model_load_time_ms: Option<f64>,
}

/// YOLO performance thresholds and alerts
#[derive(Debug, Clone)]
pub struct YOLOPerformanceThresholds {
    /// Maximum acceptable inference time in milliseconds
    pub max_inference_time_ms: f64,
    /// Maximum acceptable memory usage in MB
    pub max_memory_usage_mb: f64,
    /// Minimum ANE utilization percentage
    pub min_ane_utilization_percent: f32,
    /// Alert when CPU fallback occurs
    pub alert_on_cpu_fallback: bool,
    /// Alert when inference time exceeds threshold
    pub alert_on_slow_inference: bool,
}

/// YOLO performance monitor
pub struct YOLOPerformanceMonitor {
    telemetry: TelemetryCollector,
    thresholds: YOLOPerformanceThresholds,
    historical_metrics: Vec<YOLOPerformanceMetrics>,
    max_history_size: usize,
    alerts_enabled: bool,
}

impl YOLOPerformanceMonitor {
    /// Create a new YOLO performance monitor
    pub fn new(
        telemetry: TelemetryCollector,
        thresholds: YOLOPerformanceThresholds,
    ) -> Self {
        Self {
            telemetry,
            thresholds,
            historical_metrics: Vec::new(),
            max_history_size: 1000, // Keep last 1000 inferences
            alerts_enabled: true,
        }
    }

    /// Record YOLO inference performance metrics
    pub fn record_inference(&mut self, metrics: YOLOPerformanceMetrics) -> Result<()> {
        // Record in telemetry
        self.telemetry.record_inference(
            metrics.total_inference_time_ms as u64,
            true // Assume success for now
        );

        // Store in history
        self.historical_metrics.push(metrics.clone());
        if self.historical_metrics.len() > self.max_history_size {
            self.historical_metrics.remove(0);
        }

        // Check for alerts
        if self.alerts_enabled {
            self.check_alerts(&metrics)?;
        }

        // Log performance info
        info!(
            "YOLO inference completed: {:.1}ms total, {} objects detected",
            metrics.total_inference_time_ms,
            metrics.objects_detected
        );

        if let Some(ane_time) = metrics.ane_compute_time_ms {
            info!("ANE compute time: {:.1}ms", ane_time);
        }

        if let Some(ane_util) = metrics.ane_utilization_percent {
            info!("ANE utilization: {:.1}%", ane_util);
        }

        Ok(())
    }

    /// Check for performance alerts
    fn check_alerts(&self, metrics: &YOLOPerformanceMetrics) -> Result<()> {
        // Check inference time
        if self.thresholds.alert_on_slow_inference &&
           metrics.total_inference_time_ms > self.thresholds.max_inference_time_ms {
            warn!(
                "YOLO inference too slow: {:.1}ms > {:.1}ms threshold",
                metrics.total_inference_time_ms,
                self.thresholds.max_inference_time_ms
            );
        }

        // Check memory usage
        if metrics.memory_usage_mb > self.thresholds.max_memory_usage_mb {
            warn!(
                "YOLO memory usage high: {:.1}MB > {:.1}MB threshold",
                metrics.memory_usage_mb,
                self.thresholds.max_memory_usage_mb
            );
        }

        // Check ANE utilization
        if let Some(ane_util) = metrics.ane_utilization_percent {
            if ane_util < self.thresholds.min_ane_utilization_percent {
                warn!(
                    "YOLO ANE utilization low: {:.1}% < {:.1}% threshold",
                    ane_util,
                    self.thresholds.min_ane_utilization_percent
                );
            }
        }

        // Check CPU fallback
        if self.thresholds.alert_on_cpu_fallback &&
           metrics.cpu_fallback_time_ms.is_some() {
            warn!(
                "YOLO fell back to CPU: {:.1}ms CPU time",
                metrics.cpu_fallback_time_ms.unwrap()
            );
        }

        Ok(())
    }

    /// Get performance statistics
    pub fn get_statistics(&self) -> YOLOPerformanceStats {
        if self.historical_metrics.is_empty() {
            return YOLOPerformanceStats::default();
        }

        let count = self.historical_metrics.len();
        let mut total_inference_time = 0.0;
        let mut total_memory = 0.0;
        let mut total_objects = 0;
        let mut ane_utilizations = Vec::new();
        let mut cpu_fallbacks = 0;

        for metrics in &self.historical_metrics {
            total_inference_time += metrics.total_inference_time_ms;
            total_memory += metrics.memory_usage_mb;
            total_objects += metrics.objects_detected;

            if let Some(util) = metrics.ane_utilization_percent {
                ane_utilizations.push(util);
            }

            if metrics.cpu_fallback_time_ms.is_some() {
                cpu_fallbacks += 1;
            }
        }

        let avg_inference_time = total_inference_time / count as f64;
        let avg_memory = total_memory / count as f64;
        let avg_objects = total_objects as f64 / count as f64;

        let avg_ane_utilization = if ane_utilizations.is_empty() {
            None
        } else {
            Some(ane_utilizations.iter().sum::<f32>() / ane_utilizations.len() as f32)
        };

        let cpu_fallback_rate = cpu_fallbacks as f64 / count as f64;

        YOLOPerformanceStats {
            total_inferences: count,
            average_inference_time_ms: avg_inference_time,
            average_memory_usage_mb: avg_memory,
            average_objects_detected: avg_objects,
            average_ane_utilization_percent: avg_ane_utilization,
            cpu_fallback_rate,
            min_inference_time_ms: self.historical_metrics.iter()
                .map(|m| m.total_inference_time_ms)
                .fold(f64::INFINITY, f64::min),
            max_inference_time_ms: self.historical_metrics.iter()
                .map(|m| m.total_inference_time_ms)
                .fold(0.0, f64::max),
        }
    }

    /// Generate optimization recommendations
    pub fn get_optimization_recommendations(&self) -> Vec<String> {
        let stats = self.get_statistics();
        let mut recommendations = Vec::new();

        // Inference time recommendations
        if stats.average_inference_time_ms > 100.0 {
            recommendations.push(
                format!("High average inference time ({:.1}ms). Consider model optimization or batching.",
                       stats.average_inference_time_ms)
            );
        }

        // Memory recommendations
        if stats.average_memory_usage_mb > 500.0 {
            recommendations.push(
                format!("High memory usage ({:.1}MB). Consider quantized model or memory optimization.",
                       stats.average_memory_usage_mb)
            );
        }

        // ANE utilization recommendations
        if let Some(ane_util) = stats.average_ane_utilization_percent {
            if ane_util < 70.0 {
                recommendations.push(
                    format!("Low ANE utilization ({:.1}%). Check model compatibility or ANE drivers.",
                           ane_util)
                );
            }
        }

        // CPU fallback recommendations
        if stats.cpu_fallback_rate > 0.1 {
            recommendations.push(
                format!("High CPU fallback rate ({:.1}%). ANE may not be available or model incompatible.",
                       stats.cpu_fallback_rate * 100.0)
            );
        }

        // Object detection recommendations
        if stats.average_objects_detected < 1.0 {
            recommendations.push(
                "Low object detection rate. Consider adjusting confidence threshold or checking input images.".to_string()
            );
        }

        if recommendations.is_empty() {
            recommendations.push("Performance looks good! No optimization recommendations.".to_string());
        }

        recommendations
    }

    /// Reset performance history
    pub fn reset_history(&mut self) {
        self.historical_metrics.clear();
    }

    /// Enable or disable alerts
    pub fn set_alerts_enabled(&mut self, enabled: bool) {
        self.alerts_enabled = enabled;
    }
}

/// Performance statistics summary
#[derive(Debug, Clone)]
pub struct YOLOPerformanceStats {
    pub total_inferences: usize,
    pub average_inference_time_ms: f64,
    pub average_memory_usage_mb: f64,
    pub average_objects_detected: f64,
    pub average_ane_utilization_percent: Option<f32>,
    pub cpu_fallback_rate: f64,
    pub min_inference_time_ms: f64,
    pub max_inference_time_ms: f64,
}

impl Default for YOLOPerformanceStats {
    fn default() -> Self {
        Self {
            total_inferences: 0,
            average_inference_time_ms: 0.0,
            average_memory_usage_mb: 0.0,
            average_objects_detected: 0.0,
            average_ane_utilization_percent: None,
            cpu_fallback_rate: 0.0,
            min_inference_time_ms: 0.0,
            max_inference_time_ms: 0.0,
        }
    }
}

impl Default for YOLOPerformanceThresholds {
    fn default() -> Self {
        Self {
            max_inference_time_ms: 100.0,      // 100ms target
            max_memory_usage_mb: 600.0,        // 600MB limit
            min_ane_utilization_percent: 70.0, // 70% minimum ANE usage
            alert_on_cpu_fallback: true,
            alert_on_slow_inference: true,
        }
    }
}

/// Helper function to create YOLO performance metrics from inference data
pub fn create_yolo_metrics(
    total_time_ms: f64,
    preprocessing_time_ms: f64,
    postprocessing_time_ms: f64,
    objects_detected: usize,
    memory_usage_mb: f64,
    ane_utilization: Option<f32>,
    cpu_fallback_time: Option<f64>,
) -> YOLOPerformanceMetrics {
    YOLOPerformanceMetrics {
        total_inference_time_ms: total_time_ms,
        ane_compute_time_ms: Some(total_time_ms - preprocessing_time_ms - postprocessing_time_ms),
        cpu_fallback_time_ms: cpu_fallback_time,
        memory_usage_mb,
        ane_utilization_percent: ane_utilization,
        objects_detected,
        preprocessing_time_ms,
        postprocessing_time_ms,
        model_load_time_ms: None, // Set on first inference if needed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_monitor_creation() {
        let telemetry = TelemetryCollector::new();
        let thresholds = YOLOPerformanceThresholds::default();
        let monitor = YOLOPerformanceMonitor::new(telemetry, thresholds);

        assert_eq!(monitor.historical_metrics.len(), 0);
        assert!(monitor.alerts_enabled);
    }

    #[test]
    fn test_metrics_recording() {
        let telemetry = TelemetryCollector::new();
        let thresholds = YOLOPerformanceThresholds::default();
        let mut monitor = YOLOPerformanceMonitor::new(telemetry, thresholds);

        let metrics = YOLOPerformanceMetrics {
            total_inference_time_ms: 45.2,
            ane_compute_time_ms: Some(35.0),
            cpu_fallback_time_ms: None,
            memory_usage_mb: 350.0,
            ane_utilization_percent: Some(85.0),
            objects_detected: 3,
            preprocessing_time_ms: 5.0,
            postprocessing_time_ms: 5.2,
            model_load_time_ms: None,
        };

        monitor.record_inference(metrics.clone()).unwrap();
        assert_eq!(monitor.historical_metrics.len(), 1);

        let stats = monitor.get_statistics();
        assert_eq!(stats.total_inferences, 1);
        assert_eq!(stats.average_inference_time_ms, 45.2);
        assert_eq!(stats.average_objects_detected, 3.0);
    }

    #[test]
    fn test_optimization_recommendations() {
        let telemetry = TelemetryCollector::new();
        let thresholds = YOLOPerformanceThresholds::default();
        let mut monitor = YOLOPerformanceMonitor::new(telemetry, thresholds);

        // Add a slow inference
        let slow_metrics = YOLOPerformanceMetrics {
            total_inference_time_ms: 150.0, // Above 100ms threshold
            ane_compute_time_ms: Some(140.0),
            cpu_fallback_time_ms: None,
            memory_usage_mb: 350.0,
            ane_utilization_percent: Some(60.0), // Below 70% threshold
            objects_detected: 1,
            preprocessing_time_ms: 5.0,
            postprocessing_time_ms: 5.0,
            model_load_time_ms: None,
        };

        monitor.record_inference(slow_metrics).unwrap();

        let recommendations = monitor.get_optimization_recommendations();
        assert!(recommendations.len() > 1); // Should have multiple recommendations
        assert!(recommendations.iter().any(|r| r.contains("inference time")));
        assert!(recommendations.iter().any(|r| r.contains("ANE utilization")));
    }
}
