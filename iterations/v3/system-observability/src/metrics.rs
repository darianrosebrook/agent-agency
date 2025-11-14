//! Metrics collection and aggregation
//!
//! Performance metrics, counters, histograms, and statistical analysis.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
/// Metrics registry for managing all metrics
pub struct MetricsRegistry {
    counters: Arc<RwLock<HashMap<String, Counter>>>,
    gauges: Arc<RwLock<HashMap<String, Gauge>>>,
    histograms: Arc<RwLock<HashMap<String, Histogram>>>,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self {
            counters: Arc::new(RwLock::new(HashMap::new())),
            gauges: Arc::new(RwLock::new(HashMap::new())),
            histograms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create or get a counter
    pub async fn counter(&self, name: &str, description: &str) -> CounterHandle {
        let mut counters = self.counters.write().await;
        if !counters.contains_key(name) {
            counters.insert(
                name.to_string(),
                Counter::new(name.to_string(), description.to_string()),
            );
        }

        CounterHandle {
            registry: Arc::clone(&self.counters),
            name: name.to_string(),
        }
    }

    /// Create or get a gauge
    pub async fn gauge(&self, name: &str, description: &str) -> GaugeHandle {
        let mut gauges = self.gauges.write().await;
        if !gauges.contains_key(name) {
            gauges.insert(
                name.to_string(),
                Gauge::new(name.to_string(), description.to_string()),
            );
        }

        GaugeHandle {
            registry: Arc::clone(&self.gauges),
            name: name.to_string(),
        }
    }

    /// Create or get a histogram
    pub async fn histogram(
        &self,
        name: &str,
        description: &str,
        buckets: Vec<f64>,
    ) -> HistogramHandle {
        let mut histograms = self.histograms.write().await;
        if !histograms.contains_key(name) {
            histograms.insert(
                name.to_string(),
                Histogram::new(name.to_string(), description.to_string(), buckets),
            );
        }

        HistogramHandle {
            registry: Arc::clone(&self.histograms),
            name: name.to_string(),
        }
    }

    /// Get all metrics as a snapshot
    pub async fn snapshot(&self) -> MetricsSnapshot {
        let counters = self.counters.read().await;
        let gauges = self.gauges.read().await;
        let histograms = self.histograms.read().await;

        MetricsSnapshot {
            timestamp: chrono::Utc::now(),
            counters: counters.clone(),
            gauges: gauges.clone(),
            histograms: histograms.clone(),
        }
    }

    /// Reset all metrics
    pub async fn reset(&self) {
        let mut counters = self.counters.write().await;
        let mut gauges = self.gauges.write().await;
        let mut histograms = self.histograms.write().await;

        counters.clear();
        gauges.clear();
        histograms.clear();
    }
}

/// Counter metric
#[derive(Debug, Clone)]
pub struct Counter {
    _name: String,
    _description: String,
    value: Arc<RwLock<u64>>,
}

impl Counter {
    fn new(name: String, description: String) -> Self {
        Self {
            _name: name,
            _description: description,
            value: Arc::new(RwLock::new(0)),
        }
    }

    async fn increment(&self, value: u64) {
        let mut current = self.value.write().await;
        *current += value;
    }

    async fn get(&self) -> u64 {
        *self.value.read().await
    }
}

/// Counter handle for safe access
pub struct CounterHandle {
    registry: Arc<RwLock<HashMap<String, Counter>>>,
    name: String,
}

impl CounterHandle {
    pub async fn increment(&self, value: u64) {
        if let Some(counter) = self.registry.read().await.get(&self.name) {
            counter.increment(value).await;
        }
    }

    pub async fn get(&self) -> u64 {
        if let Some(counter) = self.registry.read().await.get(&self.name) {
            counter.get().await
        } else {
            0
        }
    }
}

/// Gauge metric
#[derive(Debug, Clone)]
pub struct Gauge {
    _name: String,
    _description: String,
    value: Arc<RwLock<f64>>,
}

impl Gauge {
    fn new(name: String, description: String) -> Self {
        Self {
            _name: name,
            _description: description,
            value: Arc::new(RwLock::new(0.0)),
        }
    }

    async fn set(&self, value: f64) {
        let mut current = self.value.write().await;
        *current = value;
    }

    async fn increment(&self, value: f64) {
        let mut current = self.value.write().await;
        *current += value;
    }

    async fn decrement(&self, value: f64) {
        let mut current = self.value.write().await;
        *current -= value;
    }

    async fn get(&self) -> f64 {
        *self.value.read().await
    }
}

/// Gauge handle for safe access
pub struct GaugeHandle {
    registry: Arc<RwLock<HashMap<String, Gauge>>>,
    name: String,
}

impl GaugeHandle {
    pub async fn set(&self, value: f64) {
        if let Some(gauge) = self.registry.read().await.get(&self.name) {
            gauge.set(value).await;
        }
    }

    pub async fn increment(&self, value: f64) {
        if let Some(gauge) = self.registry.read().await.get(&self.name) {
            gauge.increment(value).await;
        }
    }

    pub async fn decrement(&self, value: f64) {
        if let Some(gauge) = self.registry.read().await.get(&self.name) {
            gauge.decrement(value).await;
        }
    }

    pub async fn get(&self) -> f64 {
        if let Some(gauge) = self.registry.read().await.get(&self.name) {
            gauge.get().await
        } else {
            0.0
        }
    }
}

/// Histogram metric
#[derive(Debug, Clone)]
pub struct Histogram {
    _name: String,
    _description: String,
    buckets: Vec<f64>,
    counts: Arc<RwLock<Vec<u64>>>,
    sum: Arc<RwLock<f64>>,
    count: Arc<RwLock<u64>>,
}

impl Histogram {
    fn new(name: String, description: String, buckets: Vec<f64>) -> Self {
        Self {
            _name: name,
            _description: description,
            buckets: buckets.clone(),
            counts: Arc::new(RwLock::new(vec![0; buckets.len()])),
            sum: Arc::new(RwLock::new(0.0)),
            count: Arc::new(RwLock::new(0)),
        }
    }

    async fn observe(&self, value: f64) {
        let mut counts = self.counts.write().await;
        let mut sum = self.sum.write().await;
        let mut count = self.count.write().await;

        *sum += value;
        *count += 1;

        // Find the appropriate bucket
        for (i, &bucket) in self.buckets.iter().enumerate() {
            if value <= bucket {
                counts[i] += 1;
                break;
            }
        }

        // If value is greater than all buckets, increment the last bucket
        if value > *self.buckets.last().unwrap_or(&f64::INFINITY) {
            if let Some(last) = counts.last_mut() {
                *last += 1;
            }
        }
    }

    async fn get_snapshot(&self) -> HistogramSnapshot {
        let counts = self.counts.read().await;
        let sum = self.sum.read().await;
        let count = self.count.read().await;

        HistogramSnapshot {
            buckets: self.buckets.clone(),
            counts: counts.clone(),
            sum: *sum,
            count: *count,
        }
    }
}

/// Histogram snapshot for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramSnapshot {
    pub buckets: Vec<f64>,
    pub counts: Vec<u64>,
    pub sum: f64,
    pub count: u64,
}

/// Histogram handle for safe access
pub struct HistogramHandle {
    registry: Arc<RwLock<HashMap<String, Histogram>>>,
    name: String,
}

impl HistogramHandle {
    pub async fn observe(&self, value: f64) {
        if let Some(histogram) = self.registry.read().await.get(&self.name) {
            histogram.observe(value).await;
        }
    }

    pub async fn get_snapshot(&self) -> Option<HistogramSnapshot> {
        if let Some(histogram) = self.registry.read().await.get(&self.name) {
            Some(histogram.get_snapshot().await)
        } else {
            None
        }
    }
}

/// Metrics snapshot for export
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub counters: HashMap<String, Counter>,
    pub gauges: HashMap<String, Gauge>,
    pub histograms: HashMap<String, Histogram>,
}
