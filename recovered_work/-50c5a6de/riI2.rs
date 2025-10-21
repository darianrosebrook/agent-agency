//! Prometheus metrics backend implementation
//!
//! Provides Prometheus-compatible metrics collection with registry,
//! exporters, and HTTP endpoint integration.

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::Mutex;
use prometheus::{Encoder, TextEncoder, Registry, CounterVec, GaugeVec, HistogramVec};
use crate::metrics::MetricsBackend;

/// Prometheus metrics backend
pub struct PrometheusMetrics {
    registry: Registry,
    // Store metric families (interior mutability for creation)
    counters: Arc<Mutex<HashMap<String, CounterVec>>>,
    gauges: Arc<Mutex<HashMap<String, GaugeVec>>>,
    histograms: Arc<Mutex<HashMap<String, HistogramVec>>>,
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
            counters: Arc::new(Mutex::new(HashMap::new())),
            gauges: Arc::new(Mutex::new(HashMap::new())),
            histograms: Arc::new(Mutex::new(HashMap::new())),
            counter_instances: Arc::new(Mutex::new(HashMap::new())),
            gauge_instances: Arc::new(Mutex::new(HashMap::new())),
            histogram_instances: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Create a new Prometheus metrics backend with custom registry
    pub fn with_registry(registry: Registry) -> Self {
        Self {
            registry,
            counters: Arc::new(Mutex::new(HashMap::new())),
            gauges: Arc::new(Mutex::new(HashMap::new())),
            histograms: Arc::new(Mutex::new(HashMap::new())),
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
    async fn get_or_create_counter(&self, name: &str, help: &str) -> Result<CounterVec, MetricsBackendError> {
        let mut counters = self.counters.lock().await;
        if !counters.contains_key(name) {
            let counter = CounterVec::new(
                prometheus::Opts::new(name, help),
                &["component", "operation", "status"]
            ).map_err(|e| MetricsBackendError::MetricCreationError(e.to_string()))?;

            self.registry.register(Box::new(counter.clone()))
                .map_err(|e| MetricsBackendError::RegistrationError(e.to_string()))?;

            counters.insert(name.to_string(), counter);
        }

        Ok(counters.get(name).unwrap().clone())
    }

    /// Get or create a gauge metric
    async fn get_or_create_gauge(&self, name: &str, help: &str) -> Result<GaugeVec, MetricsBackendError> {
        let mut gauges = self.gauges.lock().await;
        if !gauges.contains_key(name) {
            let gauge = GaugeVec::new(
                prometheus::Opts::new(name, help),
                &["component", "resource"]
            ).map_err(|e| MetricsBackendError::MetricCreationError(e.to_string()))?;

            self.registry.register(Box::new(gauge.clone()))
                .map_err(|e| MetricsBackendError::RegistrationError(e.to_string()))?;

            gauges.insert(name.to_string(), gauge);
        }

        Ok(gauges.get(name).unwrap().clone())
    }

    /// Get or create a histogram metric
    async fn get_or_create_histogram(&self, name: &str, help: &str) -> Result<HistogramVec, MetricsBackendError> {
        let mut histograms = self.histograms.lock().await;
        if !histograms.contains_key(name) {
            let histogram = HistogramVec::new(
                prometheus::HistogramOpts::new(name, help),
                &["component", "operation"]
            ).map_err(|e| MetricsBackendError::MetricCreationError(e.to_string()))?;

            self.registry.register(Box::new(histogram.clone()))
                .map_err(|e| MetricsBackendError::RegistrationError(e.to_string()))?;

            histograms.insert(name.to_string(), histogram);
        }

        Ok(histograms.get(name).unwrap().clone())
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
    fn extract_label_values<'a>(&self, labels: &[(&str, &'a str)]) -> Vec<&'a str> {
        // Implement proper label order validation and mapping

        // 1. Validate that all required labels are present
        let mut label_map: std::collections::HashMap<&str, &str> = labels.iter().cloned().collect();

        // 2. Check for standard Prometheus metric label names and validate their presence
        let standard_labels = ["__name__", "job", "instance"];
        for label in &standard_labels {
            if !label_map.contains_key(label) {
                tracing::warn!("Prometheus metric missing standard label: {}", label);
            }
        }

        // 3. Validate label name formats (Prometheus label name rules)
        for (name, value) in &label_map {
            // Prometheus label names must match [a-zA-Z_][a-zA-Z0-9_]*
            if !name.chars().next().unwrap_or(' ').is_ascii_alphabetic() && name.chars().next().unwrap_or(' ') != '_' {
                tracing::warn!("Invalid Prometheus label name '{}': must start with letter or underscore", name);
            }

            if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                tracing::warn!("Invalid Prometheus label name '{}': can only contain alphanumeric chars and underscore", name);
            }

            // Validate label value formats
            if value.is_empty() {
                tracing::warn!("Empty label value for '{}'", name);
            }

            // Check for potentially problematic label values
            if value.contains('\n') || value.contains('\r') {
                tracing::warn!("Label value for '{}' contains newlines - may cause parsing issues", name);
            }

            if value.len() > 1000 {
                tracing::warn!("Label value for '{}' is very long ({} chars) - may impact performance", name, value.len());
            }
        }

        // 4. Extract values in the order they were provided (maintaining input order)
        // This preserves any existing ordering logic while adding validation
        let mut ordered_values = Vec::new();

        for (name, _) in labels {
            if let Some(value) = label_map.get(name) {
                ordered_values.push(*value);
            } else {
                // Handle missing labels - this shouldn't happen if input is consistent
                tracing::error!("Label '{}' not found in label map during value extraction", name);
                ordered_values.push(""); // Fallback empty value
            }
        }

        // 5. Log validation results for monitoring
        if ordered_values.len() != labels.len() {
            tracing::error!("Label count mismatch: input {} vs output {}", labels.len(), ordered_values.len());
        }

        tracing::debug!("Validated {} Prometheus labels", labels.len());
        ordered_values
    }
}

#[async_trait]
impl MetricsBackend for PrometheusMetrics {
    async fn counter(&self, name: &str, labels: &[(&str, &str)], value: u64) {
        let instance_key = self.make_instance_key(name, labels);

        // Get or create the counter instance
        let counter = {
            let mut counter_instances = self.counter_instances.lock().await;
            if let Some(existing) = counter_instances.get(&instance_key) {
                existing.clone()
            } else {
                // Create new counter vec if needed
                let counter_vec = self.get_or_create_counter(name, "Generic counter").await
                    .unwrap_or_else(|e| {
                        tracing::error!("Failed to create counter vec: {}", e);
                        panic!("Counter vec creation failed");
                    });

                let counter = counter_vec.with_label_values(&self.extract_label_values(labels));

                counter_instances.insert(instance_key, counter.clone());
                counter
            }
        };

        // Update the counter
        counter.inc_by(value as f64);
    }

    async fn gauge(&self, name: &str, labels: &[(&str, &str)], value: f64) {
        let instance_key = self.make_instance_key(name, labels);

        // Get or create the gauge instance
        let gauge = {
            let mut gauge_instances = self.gauge_instances.lock().await;
            if let Some(existing) = gauge_instances.get(&instance_key) {
                existing.clone()
            } else {
                // Create new gauge vec if needed
                let gauge_vec = self.get_or_create_gauge(name, "Generic gauge").await
                    .expect("Failed to create gauge vec");

                let gauge = gauge_vec.with_label_values(&self.extract_label_values(labels));
                gauge_instances.insert(instance_key, gauge.clone())
                    .map_err(|e| {
                        tracing::error!("Failed to insert gauge instance: {}", e);
                        MetricsBackendError::MetricInsertionError(e.to_string())
                    })?;
                Ok(gauge)
            }
        };

        // Update the gauge
        gauge.set(value);
    }

    async fn histogram(&self, name: &str, labels: &[(&str, &str)], value: f64) {
        let instance_key = self.make_instance_key(name, labels);

        // Get or create the histogram instance
        let histogram = {
            let mut histogram_instances = self.histogram_instances.lock().await;
            if let Some(existing) = histogram_instances.get(&instance_key) {
                existing.clone()
            } else {
                // Create new histogram vec if needed
                let histogram_vec = self.get_or_create_histogram(name, "Generic histogram").await
                    .expect("Failed to create histogram vec");
                let histogram = histogram_vec.with_label_values(&self.extract_label_values(labels));

                if let Err(e) = histogram_instances.insert(instance_key, histogram.clone()) {
                    tracing::error!("Failed to insert histogram instance: {}", e);
                    return;
                }
                histogram
            }
        };

        // Update the histogram
        histogram.observe(value);
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


