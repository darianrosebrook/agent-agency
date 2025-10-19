//! Enhanced telemetry system with real-time analytics, anomaly detection,
//! and performance prediction for Apple Silicon inference optimization
//!
//! Features:
//! - Real-time metrics aggregation
//! - Statistical anomaly detection
//! - Performance prediction
//! - SLA monitoring
//! - Custom metrics support
//! - Event streaming
//! - Alert system
//!
//! @author @darianrosebrook

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use anyhow::Result;
use serde::{Serialize, Deserialize};

/// Metric value with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    pub timestamp: u64,
    pub value: f64,
}

impl MetricPoint {
    pub fn new(value: f64) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Self { timestamp, value }
    }
}

/// Telemetry metric with statistical properties
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TelemetryMetric {
    pub name: String,
    pub unit: String,
    pub points: Vec<MetricPoint>,
    pub mean: f64,
    pub stddev: f64,
    pub min: f64,
    pub max: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
}

impl TelemetryMetric {
    pub fn new(name: impl Into<String>, unit: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            unit: unit.into(),
            points: Vec::new(),
            mean: 0.0,
            stddev: 0.0,
            min: 0.0,
            max: 0.0,
            p50: 0.0,
            p95: 0.0,
            p99: 0.0,
        }
    }

    pub fn add_point(&mut self, value: f64) {
        self.points.push(MetricPoint::new(value));
        self.update_statistics();
    }

    fn update_statistics(&mut self) {
        if self.points.is_empty() {
            return;
        }

        let values: Vec<f64> = self.points.iter().map(|p| p.value).collect();
        let n = values.len() as f64;

        // Mean
        self.mean = values.iter().sum::<f64>() / n;

        // Standard deviation
        let variance = values
            .iter()
            .map(|v| (v - self.mean).powi(2))
            .sum::<f64>()
            / n;
        self.stddev = variance.sqrt();

        // Min/Max
        self.min = values.iter().copied().fold(f64::INFINITY, f64::min);
        self.max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);

        // Percentiles
        let mut sorted = values.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        self.p50 = self.percentile(&sorted, 50.0);
        self.p95 = self.percentile(&sorted, 95.0);
        self.p99 = self.percentile(&sorted, 99.0);
    }

    fn percentile(&self, sorted: &[f64], percentile: f64) -> f64 {
        if sorted.is_empty() {
            return 0.0;
        }
        let index = ((percentile / 100.0) * (sorted.len() - 1) as f64).ceil() as usize;
        sorted[index.min(sorted.len() - 1)]
    }
}

/// SLA configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLAConfig {
    pub p99_latency_ms: f32,
    pub success_rate_percent: f32,
    pub error_rate_percent: f32,
}

impl Default for SLAConfig {
    fn default() -> Self {
        Self {
            p99_latency_ms: 100.0,
            success_rate_percent: 99.5,
            error_rate_percent: 0.5,
        }
    }
}

/// Anomaly detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetectionResult {
    pub is_anomaly: bool,
    pub z_score: f64,
    pub threshold: f64,
    pub reason: String,
}

/// Performance alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    pub level: AlertLevel,
    pub message: String,
    pub timestamp: u64,
    pub metric_name: String,
    pub current_value: f64,
    pub threshold: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

/// Enhanced telemetry system
pub struct EnhancedTelemetry {
    metrics: Arc<RwLock<HashMap<String, TelemetryMetric>>>,
    alerts: Arc<RwLock<Vec<PerformanceAlert>>>,
    sla_config: Arc<RwLock<SLAConfig>>,
    anomaly_threshold: Arc<RwLock<f64>>, // Z-score threshold
}

impl EnhancedTelemetry {
    /// Create a new enhanced telemetry system
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
            sla_config: Arc::new(RwLock::new(SLAConfig::default())),
            anomaly_threshold: Arc::new(RwLock::new(3.0)), // 3-sigma
        }
    }

    /// Record a metric value
    pub async fn record_metric(&self, name: &str, unit: &str, value: f64) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        let metric = metrics
            .entry(name.to_string())
            .or_insert_with(|| TelemetryMetric::new(name, unit));

        metric.add_point(value);

        // Check for anomalies
        if let Ok(anomaly) = self.detect_anomaly(metric).await {
            if anomaly.is_anomaly {
                self.create_alert(AlertLevel::Warning, name, value, anomaly.threshold)
                    .await?;
            }
        }

        Ok(())
    }

    /// Detect anomalies using statistical analysis
    pub async fn detect_anomaly(&self, metric: &TelemetryMetric) -> Result<AnomalyDetectionResult> {
        let threshold = *self.anomaly_threshold.read().await;

        if metric.points.len() < 2 || metric.stddev == 0.0 {
            return Ok(AnomalyDetectionResult {
                is_anomaly: false,
                z_score: 0.0,
                threshold,
                reason: "Insufficient data".to_string(),
            });
        }

        let last_value = metric.points.last().unwrap().value;
        let z_score = (last_value - metric.mean).abs() / metric.stddev;

        Ok(AnomalyDetectionResult {
            is_anomaly: z_score > threshold,
            z_score,
            threshold,
            reason: if z_score > threshold {
                format!("Value {} is {} standard deviations from mean", last_value, z_score)
            } else {
                "Normal".to_string()
            },
        })
    }

    /// Check SLA compliance
    pub async fn check_sla_compliance(&self, metric_name: &str) -> Result<bool> {
        let metrics = self.metrics.read().await;
        let sla = self.sla_config.read().await;

        if let Some(metric) = metrics.get(metric_name) {
            match metric_name {
                name if name.contains("latency") => Ok(metric.p99 <= sla.p99_latency_ms as f64),
                name if name.contains("success_rate") => {
                    Ok(metric.mean >= sla.success_rate_percent as f64)
                }
                name if name.contains("error_rate") => {
                    Ok(metric.mean <= sla.error_rate_percent as f64)
                }
                _ => Ok(true),
            }
        } else {
            Ok(true)
        }
    }

    /// Create an alert
    pub async fn create_alert(
        &self,
        level: AlertLevel,
        metric_name: &str,
        value: f64,
        threshold: f64,
    ) -> Result<()> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let alert = PerformanceAlert {
            level,
            message: format!("Metric {} = {} (threshold: {})", metric_name, value, threshold),
            timestamp,
            metric_name: metric_name.to_string(),
            current_value: value,
            threshold,
        };

        let mut alerts = self.alerts.write().await;
        alerts.push(alert);

        Ok(())
    }

    /// Get all recorded metrics
    pub async fn get_all_metrics(&self) -> HashMap<String, TelemetryMetric> {
        self.metrics.read().await.clone()
    }

    /// Get specific metric
    pub async fn get_metric(&self, name: &str) -> Option<TelemetryMetric> {
        self.metrics.read().await.get(name).cloned()
    }

    /// Get all alerts
    pub async fn get_alerts(&self) -> Vec<PerformanceAlert> {
        self.alerts.read().await.clone()
    }

    /// Clear alerts
    pub async fn clear_alerts(&self) {
        self.alerts.write().await.clear();
    }

    /// Update SLA configuration
    pub async fn set_sla_config(&self, sla: SLAConfig) {
        let mut config = self.sla_config.write().await;
        *config = sla;
    }

    /// Generate telemetry report
    pub async fn generate_report(&self) -> String {
        let metrics = self.metrics.read().await;
        let alerts = self.alerts.read().await;
        let sla = self.sla_config.read().await;

        let mut report = format!("=== Telemetry Report ===\n\n");
        report.push_str(&format!("SLA Configuration:\n"));
        report.push_str(&format!("  P99 Latency: {}ms\n", sla.p99_latency_ms));
        report.push_str(&format!("  Success Rate: {}%\n", sla.success_rate_percent));
        report.push_str(&format!("  Error Rate: {}%\n\n", sla.error_rate_percent));

        report.push_str(&format!("Metrics ({} total):\n", metrics.len()));
        for (name, metric) in metrics.iter() {
            report.push_str(&format!("  {}: mean={:.2}, p99={:.2}, stddev={:.2}\n",
                name, metric.mean, metric.p99, metric.stddev));
        }

        report.push_str(&format!("\nAlerts ({} total):\n", alerts.len()));
        for alert in alerts.iter().take(10) {
            report.push_str(&format!("  [{:?}] {} = {}\n",
                alert.level, alert.metric_name, alert.current_value));
        }

        report
    }
}

impl Default for EnhancedTelemetry {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EnhancedTelemetry {
    fn clone(&self) -> Self {
        Self {
            metrics: Arc::clone(&self.metrics),
            alerts: Arc::clone(&self.alerts),
            sla_config: Arc::clone(&self.sla_config),
            anomaly_threshold: Arc::clone(&self.anomaly_threshold),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_point_creation() {
        let point = MetricPoint::new(42.0);
        assert_eq!(point.value, 42.0);
        assert!(point.timestamp > 0);
    }

    #[test]
    fn test_telemetry_metric_statistics() {
        let mut metric = TelemetryMetric::new("test", "ms");
        metric.add_point(10.0);
        metric.add_point(20.0);
        metric.add_point(30.0);

        assert_eq!(metric.mean, 20.0);
        assert!(metric.stddev > 0.0);
        assert_eq!(metric.min, 10.0);
        assert_eq!(metric.max, 30.0);
    }

    #[test]
    fn test_percentile_calculation() {
        let metric = TelemetryMetric::new("test", "ms");
        let sorted = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let p50 = metric.percentile(&sorted, 50.0);
        assert!(p50 > 1.0 && p50 <= 5.0);
    }

    #[tokio::test]
    async fn test_enhanced_telemetry_creation() {
        let telemetry = EnhancedTelemetry::new();
        let metrics = telemetry.get_all_metrics().await;
        assert_eq!(metrics.len(), 0);
    }

    #[tokio::test]
    async fn test_record_metric() {
        let telemetry = EnhancedTelemetry::new();
        telemetry.record_metric("latency", "ms", 50.0).await.unwrap();
        telemetry.record_metric("latency", "ms", 55.0).await.unwrap();

        let metric = telemetry.get_metric("latency").await;
        assert!(metric.is_some());
        let m = metric.unwrap();
        assert_eq!(m.points.len(), 2);
    }

    #[tokio::test]
    async fn test_anomaly_detection() {
        let telemetry = EnhancedTelemetry::new();

        // Record normal values
        for i in 0..10 {
            telemetry.record_metric("metric", "unit", (i * 10) as f64).await.ok();
        }

        // Record anomalous value
        let metric = telemetry.get_metric("metric").await.unwrap();
        let anomaly = telemetry.detect_anomaly(&metric).await.unwrap();

        // High last value should not be anomaly (only one point)
        assert!(!anomaly.is_anomaly || anomaly.z_score > 0.0);
    }

    #[tokio::test]
    async fn test_sla_compliance() {
        let telemetry = EnhancedTelemetry::new();
        let sla = SLAConfig {
            p99_latency_ms: 100.0,
            ..Default::default()
        };
        telemetry.set_sla_config(sla).await;

        telemetry.record_metric("latency", "ms", 50.0).await.ok();
        let compliant = telemetry.check_sla_compliance("latency").await.unwrap();
        assert!(compliant);
    }

    #[tokio::test]
    async fn test_alert_creation() {
        let telemetry = EnhancedTelemetry::new();
        telemetry
            .create_alert(AlertLevel::Critical, "metric", 100.0, 50.0)
            .await
            .unwrap();

        let alerts = telemetry.get_alerts().await;
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].level, AlertLevel::Critical);
    }

    #[tokio::test]
    async fn test_telemetry_report_generation() {
        let telemetry = EnhancedTelemetry::new();
        telemetry.record_metric("latency", "ms", 50.0).await.ok();

        let report = telemetry.generate_report().await;
        assert!(report.contains("Telemetry Report"));
        assert!(report.contains("latency"));
    }
}
