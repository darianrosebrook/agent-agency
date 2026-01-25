//! Metrics Collection
//!
//! Lightweight metrics for tracking system performance and behavior.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Metric type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
}

/// A metric value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(HistogramValue),
}

/// Histogram value with buckets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramValue {
    pub count: u64,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
    pub buckets: Vec<(f64, u64)>, // (upper_bound, count)
}

impl Default for HistogramValue {
    fn default() -> Self {
        Self {
            count: 0,
            sum: 0.0,
            min: f64::MAX,
            max: f64::MIN,
            buckets: vec![
                (0.001, 0),  // 1ms
                (0.005, 0),  // 5ms
                (0.01, 0),   // 10ms
                (0.025, 0),  // 25ms
                (0.05, 0),   // 50ms
                (0.1, 0),    // 100ms
                (0.25, 0),   // 250ms
                (0.5, 0),    // 500ms
                (1.0, 0),    // 1s
                (2.5, 0),    // 2.5s
                (5.0, 0),    // 5s
                (10.0, 0),   // 10s
                (f64::MAX, 0), // +Inf
            ],
        }
    }
}

impl HistogramValue {
    pub fn observe(&mut self, value: f64) {
        self.count += 1;
        self.sum += value;
        self.min = self.min.min(value);
        self.max = self.max.max(value);

        for (upper_bound, count) in &mut self.buckets {
            if value <= *upper_bound {
                *count += 1;
            }
        }
    }

    pub fn mean(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum / self.count as f64
        }
    }
}

/// Metric with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub description: String,
    pub metric_type: MetricType,
    pub labels: HashMap<String, String>,
    pub value: MetricValue,
    pub updated_at: DateTime<Utc>,
}

/// Thread-safe counter
pub struct Counter {
    value: AtomicU64,
}

impl Counter {
    pub fn new() -> Self {
        Self {
            value: AtomicU64::new(0),
        }
    }

    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add(&self, n: u64) {
        self.value.fetch_add(n, Ordering::Relaxed);
    }

    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
}

impl Default for Counter {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe gauge
pub struct Gauge {
    value: Arc<RwLock<f64>>,
}

impl Gauge {
    pub fn new() -> Self {
        Self {
            value: Arc::new(RwLock::new(0.0)),
        }
    }

    pub fn set(&self, value: f64) {
        *self.value.write().unwrap() = value;
    }

    pub fn inc(&self) {
        *self.value.write().unwrap() += 1.0;
    }

    pub fn dec(&self) {
        *self.value.write().unwrap() -= 1.0;
    }

    pub fn add(&self, n: f64) {
        *self.value.write().unwrap() += n;
    }

    pub fn get(&self) -> f64 {
        *self.value.read().unwrap()
    }
}

impl Default for Gauge {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe histogram
pub struct Histogram {
    value: Arc<RwLock<HistogramValue>>,
}

impl Histogram {
    pub fn new() -> Self {
        Self {
            value: Arc::new(RwLock::new(HistogramValue::default())),
        }
    }

    pub fn observe(&self, value: f64) {
        self.value.write().unwrap().observe(value);
    }

    pub fn get(&self) -> HistogramValue {
        self.value.read().unwrap().clone()
    }
}

impl Default for Histogram {
    fn default() -> Self {
        Self::new()
    }
}

/// Metrics registry
pub struct MetricsRegistry {
    counters: RwLock<HashMap<String, Arc<Counter>>>,
    gauges: RwLock<HashMap<String, Arc<Gauge>>>,
    histograms: RwLock<HashMap<String, Arc<Histogram>>>,
    descriptions: RwLock<HashMap<String, String>>,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self {
            counters: RwLock::new(HashMap::new()),
            gauges: RwLock::new(HashMap::new()),
            histograms: RwLock::new(HashMap::new()),
            descriptions: RwLock::new(HashMap::new()),
        }
    }

    /// Register a counter
    pub fn counter(&self, name: &str, description: &str) -> Arc<Counter> {
        let mut counters = self.counters.write().unwrap();
        if let Some(counter) = counters.get(name) {
            return counter.clone();
        }

        let counter = Arc::new(Counter::new());
        counters.insert(name.to_string(), counter.clone());
        self.descriptions
            .write()
            .unwrap()
            .insert(name.to_string(), description.to_string());
        counter
    }

    /// Register a gauge
    pub fn gauge(&self, name: &str, description: &str) -> Arc<Gauge> {
        let mut gauges = self.gauges.write().unwrap();
        if let Some(gauge) = gauges.get(name) {
            return gauge.clone();
        }

        let gauge = Arc::new(Gauge::new());
        gauges.insert(name.to_string(), gauge.clone());
        self.descriptions
            .write()
            .unwrap()
            .insert(name.to_string(), description.to_string());
        gauge
    }

    /// Register a histogram
    pub fn histogram(&self, name: &str, description: &str) -> Arc<Histogram> {
        let mut histograms = self.histograms.write().unwrap();
        if let Some(histogram) = histograms.get(name) {
            return histogram.clone();
        }

        let histogram = Arc::new(Histogram::new());
        histograms.insert(name.to_string(), histogram.clone());
        self.descriptions
            .write()
            .unwrap()
            .insert(name.to_string(), description.to_string());
        histogram
    }

    /// Collect all metrics
    pub fn collect(&self) -> Vec<Metric> {
        let mut metrics = Vec::new();
        let now = Utc::now();
        let descriptions = self.descriptions.read().unwrap();

        // Collect counters
        for (name, counter) in self.counters.read().unwrap().iter() {
            metrics.push(Metric {
                name: name.clone(),
                description: descriptions.get(name).cloned().unwrap_or_default(),
                metric_type: MetricType::Counter,
                labels: HashMap::new(),
                value: MetricValue::Counter(counter.get()),
                updated_at: now,
            });
        }

        // Collect gauges
        for (name, gauge) in self.gauges.read().unwrap().iter() {
            metrics.push(Metric {
                name: name.clone(),
                description: descriptions.get(name).cloned().unwrap_or_default(),
                metric_type: MetricType::Gauge,
                labels: HashMap::new(),
                value: MetricValue::Gauge(gauge.get()),
                updated_at: now,
            });
        }

        // Collect histograms
        for (name, histogram) in self.histograms.read().unwrap().iter() {
            metrics.push(Metric {
                name: name.clone(),
                description: descriptions.get(name).cloned().unwrap_or_default(),
                metric_type: MetricType::Histogram,
                labels: HashMap::new(),
                value: MetricValue::Histogram(histogram.get()),
                updated_at: now,
            });
        }

        metrics
    }

    /// Get metrics count
    pub fn metric_count(&self) -> usize {
        self.counters.read().unwrap().len()
            + self.gauges.read().unwrap().len()
            + self.histograms.read().unwrap().len()
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter() {
        let counter = Counter::new();
        assert_eq!(counter.get(), 0);

        counter.inc();
        assert_eq!(counter.get(), 1);

        counter.add(5);
        assert_eq!(counter.get(), 6);
    }

    #[test]
    fn test_gauge() {
        let gauge = Gauge::new();
        assert_eq!(gauge.get(), 0.0);

        gauge.set(10.0);
        assert_eq!(gauge.get(), 10.0);

        gauge.inc();
        assert_eq!(gauge.get(), 11.0);

        gauge.dec();
        assert_eq!(gauge.get(), 10.0);
    }

    #[test]
    fn test_histogram() {
        let histogram = Histogram::new();

        histogram.observe(0.5);
        histogram.observe(1.5);
        histogram.observe(0.25);

        let value = histogram.get();
        assert_eq!(value.count, 3);
        assert_eq!(value.min, 0.25);
        assert_eq!(value.max, 1.5);
    }

    #[test]
    fn test_histogram_mean() {
        let mut hist = HistogramValue::default();
        hist.observe(1.0);
        hist.observe(2.0);
        hist.observe(3.0);

        assert_eq!(hist.mean(), 2.0);
    }

    #[test]
    fn test_registry() {
        let registry = MetricsRegistry::new();

        let counter = registry.counter("requests_total", "Total requests");
        counter.inc();
        counter.inc();

        let gauge = registry.gauge("active_tasks", "Active tasks");
        gauge.set(5.0);

        let histogram = registry.histogram("request_duration", "Request duration");
        histogram.observe(0.1);

        assert_eq!(registry.metric_count(), 3);

        let metrics = registry.collect();
        assert_eq!(metrics.len(), 3);
    }

    #[test]
    fn test_registry_same_metric() {
        let registry = MetricsRegistry::new();

        let counter1 = registry.counter("test", "Test counter");
        let counter2 = registry.counter("test", "Test counter");

        counter1.inc();
        assert_eq!(counter2.get(), 1);
    }
}
