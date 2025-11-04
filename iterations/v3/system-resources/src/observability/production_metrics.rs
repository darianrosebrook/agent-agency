//! Metrics collection and management

use schemars::JsonSchema;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

use super::core::{ObservabilityConfig, LogEntry};
use super::quantiles::QuantileEstimator;

/// Metric types supported by the observability system
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum MetricType {
    /// Monotonically increasing counter
    Counter,
    /// Gauge that can go up and down
    Gauge,
    /// Histogram with buckets
    Histogram,
    /// Summary with quantiles
    Summary,
}

impl std::fmt::Display for MetricType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricType::Counter => write!(f, "counter"),
            MetricType::Gauge => write!(f, "gauge"),
            MetricType::Histogram => write!(f, "histogram"),
            MetricType::Summary => write!(f, "summary"),
        }
    }
}

/// Metric value container
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum MetricValue {
    /// Counter value
    Counter(u64),
    /// Gauge value
    Gauge(f64),
    /// Histogram with count, sum, and bucket data
    Histogram {
        count: u64,
        sum: f64,
        buckets: Vec<(f64, u64)>
    },
    /// Summary with count, sum, and quantile data
    Summary {
        count: u64,
        sum: f64,
        quantiles: Vec<(f64, f64)>
    },
}

impl MetricValue {
    /// Get the numeric value for simple metrics
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            MetricValue::Counter(v) => Some(*v as f64),
            MetricValue::Gauge(v) => Some(*v),
            _ => None,
        }
    }

    /// Get the count for aggregate metrics
    pub fn count(&self) -> u64 {
        match self {
            MetricValue::Counter(v) => *v,
            MetricValue::Gauge(_) => 1,
            MetricValue::Histogram { count, .. } => *count,
            MetricValue::Summary { count, .. } => *count,
        }
    }
}

/// Individual metric data point
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MetricDataPoint {
    /// Metric name
    pub name: String,
    /// Metric value
    pub value: MetricValue,
    /// Label dimensions
    pub labels: HashMap<String, String>,
    /// Timestamp when the metric was recorded
    ##[schemars(with = "String")]

    pub timestamp: DateTime<Utc>,
}

impl MetricDataPoint {
    /// Create a new counter metric
    pub fn counter(name: impl Into<String>, value: u64) -> Self {
        Self {
            name: name.into(),
            value: MetricValue::Counter(value),
            labels: HashMap::new(),
            timestamp: Utc::now(),
        }
    }

    /// Create a new gauge metric
    pub fn gauge(name: impl Into<String>, value: f64) -> Self {
        Self {
            name: name.into(),
            value: MetricValue::Gauge(value),
            labels: HashMap::new(),
            timestamp: Utc::now(),
        }
    }

    /// Add a label to the metric
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    /// Set a custom timestamp
    pub fn with_timestamp(mut self, #[schemars(with = "String")]
    timestamp: DateTime<Utc>) -> Self {
        self.timestamp = timestamp;
        self
    }
}

/// Observability error types
#[derive(Debug, Clone, thiserror::Error, JsonSchema)]
pub enum ObservabilityError {
    #[error("Metrics collection error: {message}")]
    CollectionError { message: String },

    #[error("Storage error: {message}")]
    StorageError { message: String },

    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    #[error("Serialization error: {message}")]
    SerializationError { message: String },
}

/// Metrics collector trait for pluggable metric backends
#[async_trait]
pub trait MetricsCollector: Send + Sync {
    /// Record a counter metric
    async fn record_counter(&self, name: &str, value: u64) -> Result<(), ObservabilityError>;

    /// Record a gauge metric
    async fn record_gauge(&self, name: &str, value: f64) -> Result<(), ObservabilityError>;

    /// Record a histogram observation
    async fn record_histogram(&self, name: &str, value: f64) -> Result<(), ObservabilityError>;

    /// Record a summary observation
    async fn record_summary(&self, name: &str, value: f64) -> Result<(), ObservabilityError>;

    /// Get all current metrics
    async fn get_metrics(&self) -> Result<Vec<MetricDataPoint>, ObservabilityError>;

    /// Get a specific metric by name
    async fn get_metric(&self, name: &str) -> Result<Option<MetricDataPoint>, ObservabilityError>;
}

/// In-memory metrics collector implementation
pub struct InMemoryMetricsCollector {
    /// Stored metrics
    metrics: Arc<RwLock<HashMap<String, MetricDataPoint>>>,
    /// Configuration
    config: ObservabilityConfig,
    /// Quantile estimators for histogram/summary metrics
    quantile_estimators: Arc<RwLock<HashMap<String, QuantileEstimator>>>,
}

impl InMemoryMetricsCollector {
    /// Create a new in-memory metrics collector
    pub fn new(config: ObservabilityConfig) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            config,
            quantile_estimators: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &ObservabilityConfig {
        &self.config
    }
}

#[async_trait]
impl MetricsCollector for InMemoryMetricsCollector {
    async fn record_counter(&self, name: &str, value: u64) -> Result<(), ObservabilityError> {
        let mut metrics = self.metrics.write().await;
        let key = format!("counter:{}", name);

        // Update existing counter or create new one
        if let Some(existing) = metrics.get_mut(&key) {
            if let MetricValue::Counter(ref mut current) = existing.value {
                *current = (*current).max(value); // Counters should be monotonically increasing
            }
            existing.timestamp = Utc::now();
        } else {
            let metric = MetricDataPoint::counter(name, value);
            metrics.insert(key, metric);
        }

        Ok(())
    }

    async fn record_gauge(&self, name: &str, value: f64) -> Result<(), ObservabilityError> {
        let mut metrics = self.metrics.write().await;
        let key = format!("gauge:{}", name);

        let metric = MetricDataPoint::gauge(name, value);
        metrics.insert(key, metric);

        Ok(())
    }

    async fn record_histogram(&self, name: &str, value: f64) -> Result<(), ObservabilityError> {
        let mut estimators = self.quantile_estimators.write().await;
        let estimator = estimators.entry(name.to_string())
            .or_insert_with(|| QuantileEstimator::new());

        estimator.observe(value);

        // Update the metric with current histogram data
        let mut metrics = self.metrics.write().await;
        let key = format!("histogram:{}", name);

        let histogram_data = estimator.to_histogram();
        let metric = MetricDataPoint {
            name: name.to_string(),
            value: MetricValue::Histogram {
                count: histogram_data.count,
                sum: histogram_data.sum,
                buckets: histogram_data.buckets,
            },
            labels: HashMap::new(),
            timestamp: Utc::now(),
        };
        metrics.insert(key, metric);

        Ok(())
    }

    async fn record_summary(&self, name: &str, value: f64) -> Result<(), ObservabilityError> {
        let mut estimators = self.quantile_estimators.write().await;
        let estimator = estimators.entry(name.to_string())
            .or_insert_with(|| QuantileEstimator::new());

        estimator.observe(value);

        // Update the metric with current summary data
        let mut metrics = self.metrics.write().await;
        let key = format!("summary:{}", name);

        let summary_data = estimator.to_summary();
        let metric = MetricDataPoint {
            name: name.to_string(),
            value: MetricValue::Summary {
                count: summary_data.count,
                sum: summary_data.sum,
                quantiles: summary_data.quantiles,
            },
            labels: HashMap::new(),
            timestamp: Utc::now(),
        };
        metrics.insert(key, metric);

        Ok(())
    }

    async fn get_metrics(&self) -> Result<Vec<MetricDataPoint>, ObservabilityError> {
        let metrics = self.metrics.read().await;
        Ok(metrics.values().cloned().collect())
    }

    async fn get_metric(&self, name: &str) -> Result<Option<MetricDataPoint>, ObservabilityError> {
        let metrics = self.metrics.read().await;
        Ok(metrics.get(name).cloned())
    }
}
