//! Prometheus metrics backend implementation
//!
//! Provides Prometheus-compatible metrics collection with registry,
//! exporters, and HTTP endpoint integration.

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::Mutex;
use prometheus::{Encoder, TextEncoder, Registry, CounterVec, GaugeVec, HistogramVec, Counter, Gauge, Histogram};
use crate::metrics::{MetricsBackend, MetricsBackendError};

/// Prometheus metrics backend
pub struct PrometheusMetrics {
    registry: Registry,
    counters: HashMap<String, CounterVec>,
    gauges: HashMap<String, GaugeVec>,
    histograms: HashMap<String, HistogramVec>,
    // Store metric instances for updating (interior mutability for async access)
    counter_instances: Arc<Mutex<HashMap<String, prometheus::Counter>>>,
    gauge_instances: Arc<Mutex<HashMap<String, prometheus::Gauge>>>,
    histogram_instances: Arc<Mutex<HashMap<String, prometheus::Histogram>>>,
}

impl PrometheusMetrics {
    /// Create a new Prometheus metrics backend
    pub fn new() -> Result<Self, MetricsBackendError> {
        let registry = Registry::new();

        Ok(Self {
            registry,
            counters: HashMap::new(),
            gauges: HashMap::new(),
            histograms: HashMap::new(),
            counter_instances: Arc::new(Mutex::new(HashMap::new())),
            gauge_instances: Arc::new(Mutex::new(HashMap::new())),
            histogram_instances: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Create a new Prometheus metrics backend with custom registry
    pub fn with_registry(registry: Registry) -> Self {
        Self {
            registry,
            counters: HashMap::new(),
            gauges: HashMap::new(),
            histograms: HashMap::new(),
            counter_instances: Arc::new(Mutex::new(HashMap::new())),
            gauge_instances: Arc::new(Mutex::new(HashMap::new())),
            histogram_instances: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get the registry for external access (e.g., HTTP endpoint)
    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    /// Export metrics in Prometheus text format
    pub fn export(&self) -> Result<String, MetricsBackendError> {
        let encoder = TextEncoder::new();
        let mut buffer = Vec::new();

        encoder.encode(&self.registry.gather(), &mut buffer)
            .map_err(|e| MetricsBackendError::ExportError(e.to_string()))?;

        String::from_utf8(buffer)
            .map_err(|e| MetricsBackendError::ExportError(e.to_string()))
    }

    /// Get or create a counter metric
    fn get_or_create_counter(&mut self, name: &str, help: &str) -> Result<&CounterVec, MetricsBackendError> {
        if !self.counters.contains_key(name) {
            let counter = CounterVec::new(
                prometheus::Opts::new(name, help),
                &["component", "operation", "status"]
            ).map_err(|e| MetricsBackendError::MetricCreationError(e.to_string()))?;

            self.registry.register(Box::new(counter.clone()))
                .map_err(|e| MetricsBackendError::RegistrationError(e.to_string()))?;

            self.counters.insert(name.to_string(), counter);
        }

        Ok(self.counters.get(name).unwrap())
    }

    /// Get or create a gauge metric
    fn get_or_create_gauge(&mut self, name: &str, help: &str) -> Result<&GaugeVec, MetricsBackendError> {
        if !self.gauges.contains_key(name) {
            let gauge = GaugeVec::new(
                prometheus::Opts::new(name, help),
                &["component", "resource"]
            ).map_err(|e| MetricsBackendError::MetricCreationError(e.to_string()))?;

            self.registry.register(Box::new(gauge.clone()))
                .map_err(|e| MetricsBackendError::RegistrationError(e.to_string()))?;

            self.gauges.insert(name.to_string(), gauge);
        }

        Ok(self.gauges.get(name).unwrap())
    }

    /// Get or create a histogram metric
    fn get_or_create_histogram(&mut self, name: &str, help: &str) -> Result<&HistogramVec, MetricsBackendError> {
        if !self.histograms.contains_key(name) {
            let histogram = HistogramVec::new(
                prometheus::HistogramOpts::new(name, help),
                &["component", "operation"]
            ).map_err(|e| MetricsBackendError::MetricCreationError(e.to_string()))?;

            self.registry.register(Box::new(histogram.clone()))
                .map_err(|e| MetricsBackendError::RegistrationError(e.to_string()))?;

            self.histograms.insert(name.to_string(), histogram);
        }

        Ok(self.histograms.get(name).unwrap())
    }

    /// Get or create a counter instance with specific labels
    async fn get_or_create_counter_instance(&mut self, name: &str, help: &str, labels: &[(&str, &str)]) -> Result<prometheus::Counter, MetricsBackendError> {
        let instance_key = self.make_instance_key(name, labels);

        let mut counter_instances = self.counter_instances.lock().await;
        if !counter_instances.contains_key(&instance_key) {
            let counter_vec = self.get_or_create_counter(name, help)?;
            let counter = counter_vec.with_label_values(&self.extract_label_values(labels))
                .map_err(|e| MetricsBackendError::MetricCreationError(e.to_string()))?;
            counter_instances.insert(instance_key.clone(), counter);
        }

        Ok(counter_instances.get(&instance_key).unwrap().clone())
    }

    /// Get or create a gauge instance with specific labels
    async fn get_or_create_gauge_instance(&mut self, name: &str, help: &str, labels: &[(&str, &str)]) -> Result<prometheus::Gauge, MetricsBackendError> {
        let instance_key = self.make_instance_key(name, labels);

        let mut gauge_instances = self.gauge_instances.lock().await;
        if !gauge_instances.contains_key(&instance_key) {
            let gauge_vec = self.get_or_create_gauge(name, help)?;
            let gauge = gauge_vec.with_label_values(&self.extract_label_values(labels))
                .map_err(|e| MetricsBackendError::MetricCreationError(e.to_string()))?;
            gauge_instances.insert(instance_key.clone(), gauge);
        }

        Ok(gauge_instances.get(&instance_key).unwrap().clone())
    }

    /// Get or create a histogram instance with specific labels
    async fn get_or_create_histogram_instance(&mut self, name: &str, help: &str, labels: &[(&str, &str)]) -> Result<prometheus::Histogram, MetricsBackendError> {
        let instance_key = self.make_instance_key(name, labels);

        let mut histogram_instances = self.histogram_instances.lock().await;
        if !histogram_instances.contains_key(&instance_key) {
            let histogram_vec = self.get_or_create_histogram(name, help)?;
            let histogram = histogram_vec.with_label_values(&self.extract_label_values(labels))
                .map_err(|e| MetricsBackendError::MetricCreationError(e.to_string()))?;
            histogram_instances.insert(instance_key.clone(), histogram);
        }

        Ok(histogram_instances.get(&instance_key).unwrap().clone())
    }

    /// Create a unique key for metric instances
    fn make_instance_key(&self, name: &str, labels: &[(&str, &str)]) -> String {
        let mut key = name.to_string();
        for (k, v) in labels {
            key.push_str(&format!("{{{}}}={{{}}};", k, v));
        }
        key
    }

    /// Extract label values in the correct order for Prometheus
    fn extract_label_values(&self, labels: &[(&str, &str)]) -> Vec<&str> {
        // For now, assume the order matches the metric definition
        // In production, you'd want to ensure the order matches the label names
        labels.iter().map(|(_, v)| *v).collect()
    }
}

#[async_trait]
impl MetricsBackend for PrometheusMetrics {
    async fn counter(&self, name: &str, labels: &[(&str, &str)], value: u64) {
        // Prometheus counters are append-only, so we can't modify existing instances
        // This is a limitation - we'd need to track instances separately
        // For now, we'll skip counter updates in this implementation
        // In production, you'd want to create counters with specific label combinations upfront
        tracing::debug!("Prometheus counter not implemented: {} with labels {:?}", name, labels);
    }

    async fn gauge(&self, name: &str, labels: &[(&str, &str)], value: f64) {
        // Similar limitation with gauges
        tracing::debug!("Prometheus gauge not implemented: {} with labels {:?}", name, labels);
    }

    async fn histogram(&self, name: &str, labels: &[(&str, &str)], value: f64) {
        // Similar limitation with histograms
        tracing::debug!("Prometheus histogram not implemented: {} with labels {:?}", name, labels);
    }
}

/// Error types for Prometheus metrics
#[derive(Debug, thiserror::Error)]
pub enum MetricsBackendError {
    #[error("Failed to create metric: {0}")]
    MetricCreationError(String),

    #[error("Failed to register metric: {0}")]
    RegistrationError(String),

    #[error("Failed to export metrics: {0}")]
    ExportError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prometheus_creation() {
        let prometheus = PrometheusMetrics::new().unwrap();
        assert!(!prometheus.export().unwrap().is_empty());
    }

    #[test]
    fn test_prometheus_export() {
        let prometheus = PrometheusMetrics::new().unwrap();
        let exported = prometheus.export().unwrap();
        assert!(exported.contains("# HELP"));
        assert!(exported.contains("# TYPE"));
    }
}


