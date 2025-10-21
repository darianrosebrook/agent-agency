//! Enhanced telemetry and monitoring

/// Alert level
#[derive(Debug, Clone)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

/// Anomaly detection result
#[derive(Debug, Clone)]
pub struct AnomalyDetectionResult {
    pub is_anomaly: bool,
    pub confidence: f32,
    pub description: String,
}

/// Enhanced telemetry
#[derive(Debug)]
pub struct EnhancedTelemetry {
    metrics: Vec<TelemetryMetric>,
}

impl EnhancedTelemetry {
    pub fn new() -> Self {
        Self {
            metrics: Vec::new(),
        }
    }

    pub fn record_metric(&mut self, metric: TelemetryMetric) {
        self.metrics.push(metric);
    }
}

/// Performance alert
#[derive(Debug, Clone)]
pub struct PerformanceAlert {
    pub level: AlertLevel,
    pub message: String,
    pub timestamp: u64,
}

/// SLA configuration
#[derive(Debug, Clone)]
pub struct SLAConfig {
    pub max_latency_ms: u64,
    pub min_throughput: f32,
}

/// Telemetry metric
#[derive(Debug, Clone)]
pub struct TelemetryMetric {
    pub name: String,
    pub value: f64,
    pub timestamp: u64,
}

/// Metric point for time series
#[derive(Debug, Clone)]
pub struct MetricPoint {
    pub timestamp: u64,
    pub value: f64,
    pub labels: std::collections::HashMap<String, String>,
}
