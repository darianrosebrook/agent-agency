//! YOLO Performance Monitoring Dashboard
//!
//! Provides a comprehensive dashboard for monitoring YOLO inference performance,
//! ANE utilization, and optimization recommendations.

use crate::ane::monitoring::yolo_monitor::{YOLOPerformanceMonitor, YOLOPerformanceStats, YOLOPerformanceThresholds};
use crate::ane::optimization::ane_optimizer::{ANEOptimizer, ANEOptimizationStrategy, ANEMemoryOptimizer, BatchOptimizer};
use crate::telemetry::TelemetryCollector;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{info, warn, debug};

/// Comprehensive YOLO performance dashboard
pub struct YOLOPerformanceDashboard {
    yolo_monitor: YOLOPerformanceMonitor,
    ane_optimizer: ANEOptimizer,
    memory_optimizer: ANEMemoryOptimizer,
    batch_optimizer: BatchOptimizer,
    dashboard_enabled: bool,
    last_report_time: Instant,
    report_interval: Duration,
}

impl YOLOPerformanceDashboard {
    /// Create a new performance dashboard
    pub fn new(telemetry: TelemetryCollector) -> Self {
        let thresholds = YOLOPerformanceThresholds::default();

        Self {
            yolo_monitor: YOLOPerformanceMonitor::new(telemetry, thresholds),
            ane_optimizer: ANEOptimizer::new(ANEOptimizationStrategy::Balanced),
            memory_optimizer: ANEMemoryOptimizer::new(),
            batch_optimizer: BatchOptimizer::new(4), // Max batch size of 4
            dashboard_enabled: true,
            last_report_time: Instant::now(),
            report_interval: Duration::from_secs(60), // Report every minute
        }
    }

    /// Record a YOLO inference and update all monitoring systems
    pub async fn record_inference(&mut self, metrics: crate::ane::monitoring::yolo_monitor::YOLOPerformanceMetrics) -> crate::ane::errors::Result<()> {
        // Record in YOLO monitor
        self.yolo_monitor.record_inference(metrics.clone())?;

        // Record in optimizer for adaptation
        self.ane_optimizer.record_performance("yolo", metrics.total_inference_time_ms);

        // Check memory optimization
        if self.memory_optimizer.should_optimize_memory(metrics.memory_usage_mb) {
            let recommendations = self.memory_optimizer.get_memory_recommendations(metrics.memory_usage_mb);
            for rec in recommendations {
                warn!("Memory optimization needed: {}", rec);
            }
        }

        // Update batch optimizer with throughput estimate
        let throughput = 1000.0 / metrics.total_inference_time_ms; // inferences per second
        let _new_batch_size = self.batch_optimizer.optimize_batch_size(throughput, metrics.total_inference_time_ms);

        // Generate periodic reports
        if self.dashboard_enabled && self.last_report_time.elapsed() >= self.report_interval {
            self.generate_performance_report();
            self.last_report_time = Instant::now();
        }

        Ok(())
    }

    /// Generate a comprehensive performance report
    pub fn generate_performance_report(&mut self) {
        println!("\n YOLO Performance Dashboard Report");
        println!("=====================================\n");

        let stats = self.yolo_monitor.get_statistics();

        // Basic Statistics
        println!(" Performance Statistics:");
        println!("   Total Inferences: {}", stats.total_inferences);
        println!("   Average Inference Time: {:.1}ms", stats.average_inference_time_ms);
        println!("   Min/Max Inference Time: {:.1}ms / {:.1}ms",
                stats.min_inference_time_ms, stats.max_inference_time_ms);
        println!("   Average Memory Usage: {:.1}MB", stats.average_memory_usage_mb);
        println!("   Average Objects Detected: {:.1}", stats.average_objects_detected);

        // ANE Utilization
        if let Some(ane_util) = stats.average_ane_utilization_percent {
            println!("   Average ANE Utilization: {:.1}%", ane_util);

            if ane_util < 70.0 {
                println!("   ⚠️  Low ANE utilization - check model compatibility");
            } else {
                println!("    Good ANE utilization");
            }
        } else {
            println!("   ANE utilization: Not available");
        }

        // CPU Fallback Rate
        println!("   CPU Fallback Rate: {:.1}%", stats.cpu_fallback_rate * 100.0);
        if stats.cpu_fallback_rate > 0.1 {
            println!("   ⚠️  High CPU fallback rate detected");
        }

        // Performance Analysis
        println!("\n Performance Analysis:");
        if stats.average_inference_time_ms < 50.0 {
            println!("    Excellent performance (< 50ms)");
        } else if stats.average_inference_time_ms < 100.0 {
            println!("    Good performance (< 100ms)");
        } else if stats.average_inference_time_ms < 200.0 {
            println!("   ⚠️  Acceptable performance (< 200ms)");
        } else {
            println!("    Poor performance (> 200ms)");
        }

        // Memory Analysis
        if stats.average_memory_usage_mb < 400.0 {
            println!("    Low memory usage");
        } else if stats.average_memory_usage_mb < 600.0 {
            println!("   ⚠️  Moderate memory usage");
        } else {
            println!("    High memory usage");
        }

        // Optimization Recommendations
        let recommendations = self.yolo_monitor.get_optimization_recommendations();
        if !recommendations.is_empty() && !recommendations[0].contains("looks good") {
            println!("\n Optimization Recommendations:");
            for (i, rec) in recommendations.iter().enumerate() {
                println!("   {}. {}", i + 1, rec);
            }
        }

        // Current Optimization Settings
        let opt_params = self.ane_optimizer.get_optimization_params("yolo");
        println!("\n⚙️  Current Optimization Settings:");
        println!("   Batch Size: {}", opt_params.batch_size);
        println!("   Precision: {:?}", opt_params.precision);
        println!("   Memory Strategy: {:?}", opt_params.memory_strategy);
        println!("   Compute Units: {:?}", opt_params.compute_units);

        println!("\n Dashboard updated at {}", chrono::Utc::now().format("%H:%M:%S UTC"));
    }

    /// Get real-time performance metrics
    pub fn get_real_time_metrics(&mut self) -> DashboardMetrics {
        let stats = self.yolo_monitor.get_statistics();
        let opt_params = self.ane_optimizer.get_optimization_params("yolo");

        DashboardMetrics {
            total_inferences: stats.total_inferences,
            current_avg_inference_time_ms: stats.average_inference_time_ms,
            current_memory_usage_mb: stats.average_memory_usage_mb,
            ane_utilization_percent: stats.average_ane_utilization_percent,
            cpu_fallback_rate: stats.cpu_fallback_rate,
            batch_size: opt_params.batch_size,
            active_optimizations: self.get_active_optimizations(),
        }
    }

    /// Get list of currently active optimizations
    fn get_active_optimizations(&mut self) -> Vec<String> {
        let mut active = Vec::new();

        let opt_params = self.ane_optimizer.get_optimization_params("yolo");

        match opt_params.precision {
            crate::ane::optimization::ane_optimizer::PrecisionMode::Half => {
                active.push("Half precision".to_string());
            }
            crate::ane::optimization::ane_optimizer::PrecisionMode::Quantized => {
                active.push("Quantization".to_string());
            }
            _ => {}
        }

        if matches!(opt_params.compute_units, crate::ane::optimization::ane_optimizer::ComputeUnitPreference::ANE) {
            active.push("ANE acceleration".to_string());
        }

        if matches!(opt_params.memory_strategy, crate::ane::optimization::ane_optimizer::MemoryStrategy::Pooled) {
            active.push("Memory pooling".to_string());
        }

        if opt_params.batch_size > 1 {
            active.push(format!("Batch processing (size {})", opt_params.batch_size));
        }

        active
    }

    /// Enable or disable the dashboard
    pub fn set_dashboard_enabled(&mut self, enabled: bool) {
        self.dashboard_enabled = enabled;
    }

    /// Set report interval
    pub fn set_report_interval(&mut self, interval: Duration) {
        self.report_interval = interval;
    }

    /// Force generate a report
    pub fn force_report(&mut self) {
        self.generate_performance_report();
    }

    /// Reset all monitoring data
    pub fn reset_all(&mut self) {
        self.yolo_monitor.reset_history();
        self.ane_optimizer.reset_performance_history("yolo");
        self.last_report_time = Instant::now();
    }
}

/// Real-time dashboard metrics
#[derive(Debug, Clone)]
pub struct DashboardMetrics {
    pub total_inferences: usize,
    pub current_avg_inference_time_ms: f64,
    pub current_memory_usage_mb: f64,
    pub ane_utilization_percent: Option<f32>,
    pub cpu_fallback_rate: f64,
    pub batch_size: usize,
    pub active_optimizations: Vec<String>,
}

/// Performance alert system
pub struct PerformanceAlerts {
    alerts: Vec<PerformanceAlert>,
    max_alerts: usize,
}

#[derive(Debug, Clone)]
pub struct PerformanceAlert {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: AlertLevel,
    pub message: String,
    pub metric: String,
    pub value: f64,
    pub threshold: f64,
}

#[derive(Debug, Clone)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

impl PerformanceAlerts {
    pub fn new(max_alerts: usize) -> Self {
        Self {
            alerts: Vec::new(),
            max_alerts,
        }
    }

    pub fn add_alert(&mut self, level: AlertLevel, message: String, metric: String, value: f64, threshold: f64) {
        let alert = PerformanceAlert {
            timestamp: chrono::Utc::now(),
            level: level.clone(),
            message: message.clone(),
            metric,
            value,
            threshold,
        };

        self.alerts.push(alert);

        // Keep only the most recent alerts
        if self.alerts.len() > self.max_alerts {
            self.alerts.remove(0);
        }

        // Log the alert
        match level {
            AlertLevel::Critical => tracing::error!(" {}", message),
            AlertLevel::Warning => tracing::warn!("⚠️  {}", message),
            AlertLevel::Info => tracing::info!("ℹ️  {}", message),
        }
    }

    pub fn get_recent_alerts(&self, count: usize) -> &[PerformanceAlert] {
        let start = if self.alerts.len() > count {
            self.alerts.len() - count
        } else {
            0
        };
        &self.alerts[start..]
    }

    pub fn clear_alerts(&mut self) {
        self.alerts.clear();
    }
}

/// Performance prediction system
pub struct PerformancePredictor {
    historical_data: Vec<(chrono::DateTime<chrono::Utc>, f64)>, // timestamp, inference_time
    prediction_window_hours: i64,
}

impl PerformancePredictor {
    pub fn new(prediction_window_hours: i64) -> Self {
        Self {
            historical_data: Vec::new(),
            prediction_window_hours,
        }
    }

    pub fn record_inference_time(&mut self, inference_time_ms: f64) {
        self.historical_data.push((chrono::Utc::now(), inference_time_ms));

        // Keep only data within the prediction window
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(self.prediction_window_hours);
        self.historical_data.retain(|(timestamp, _)| *timestamp > cutoff);
    }

    pub fn predict_future_performance(&self) -> Option<PerformancePrediction> {
        if self.historical_data.len() < 10 {
            return None; // Need minimum data for prediction
        }

        let recent_data: Vec<f64> = self.historical_data.iter()
            .rev()
            .take(20)
            .map(|(_, time)| *time)
            .collect();

        let avg_recent = recent_data.iter().sum::<f64>() / recent_data.len() as f64;

        // Simple trend analysis
        let trend = if recent_data.len() >= 2 {
            let first_half: Vec<f64> = recent_data.iter().take(recent_data.len() / 2).cloned().collect();
            let second_half: Vec<f64> = recent_data.iter().skip(recent_data.len() / 2).cloned().collect();

            let avg_first = first_half.iter().sum::<f64>() / first_half.len() as f64;
            let avg_second = second_half.iter().sum::<f64>() / second_half.len() as f64;

            if avg_second > avg_first * 1.05 {
                PerformanceTrend::Degrading
            } else if avg_second < avg_first * 0.95 {
                PerformanceTrend::Improving
            } else {
                PerformanceTrend::Stable
            }
        } else {
            PerformanceTrend::Stable
        };

        Some(PerformancePrediction {
            predicted_avg_inference_time_ms: avg_recent,
            confidence_level: 0.8, // Simplified confidence
            trend,
            data_points: recent_data.len(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct PerformancePrediction {
    pub predicted_avg_inference_time_ms: f64,
    pub confidence_level: f64,
    pub trend: PerformanceTrend,
    pub data_points: usize,
}

#[derive(Debug, Clone)]
pub enum PerformanceTrend {
    Improving,
    Stable,
    Degrading,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::TelemetryCollector;

    #[tokio::test]
    async fn test_dashboard_creation() {
        let telemetry = TelemetryCollector::new();
        let mut dashboard = YOLOPerformanceDashboard::new(telemetry);

        let metrics = dashboard.get_real_time_metrics();
        assert_eq!(metrics.total_inferences, 0);
        assert_eq!(metrics.batch_size, 1);
    }

    #[test]
    fn test_performance_alerts() {
        let mut alerts = PerformanceAlerts::new(10);

        alerts.add_alert(
            AlertLevel::Warning,
            "High inference time detected".to_string(),
            "inference_time_ms".to_string(),
            150.0,
            100.0,
        );

        let recent = alerts.get_recent_alerts(5);
        assert_eq!(recent.len(), 1);
        assert!(matches!(recent[0].level, AlertLevel::Warning));
    }

    #[test]
    fn test_performance_predictor() {
        let mut predictor = PerformancePredictor::new(24); // 24 hour window

        // Add some test data
        for i in 0..15 {
            predictor.record_inference_time(50.0 + (i as f64 * 2.0));
        }

        let prediction = predictor.predict_future_performance().unwrap();
        assert!(prediction.predicted_avg_inference_time_ms > 0.0);
        assert!(matches!(prediction.trend, PerformanceTrend::Degrading)); // Data is increasing
    }
}
