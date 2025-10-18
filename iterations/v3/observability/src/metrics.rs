//! Basic metrics collection implementation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    pub value: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub counters: HashMap<String, MetricValue>,
    pub gauges: HashMap<String, MetricValue>,
    pub histograms: HashMap<String, Vec<MetricValue>>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
pub struct MetricsCollector {
    counters: Arc<RwLock<HashMap<String, u64>>>,
    gauges: Arc<RwLock<HashMap<String, f64>>>,
    histograms: Arc<RwLock<HashMap<String, Vec<f64>>>>,
    enabled: bool,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            counters: Arc::new(RwLock::new(HashMap::new())),
            gauges: Arc::new(RwLock::new(HashMap::new())),
            histograms: Arc::new(RwLock::new(HashMap::new())),
            enabled: true,
        }
    }

    pub async fn increment_counter(&self, name: &str, labels: &[(&str, &str)]) {
        if !self.enabled {
            return;
        }

        let key = self.make_key(name, labels);
        let mut counters = self.counters.write().await;
        *counters.entry(key).or_insert(0) += 1;
    }

    pub async fn update_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        if !self.enabled {
            return;
        }

        let key = self.make_key(name, labels);
        let mut gauges = self.gauges.write().await;
        gauges.insert(key, value);
    }

    pub async fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        if !self.enabled {
            return;
        }

        let key = self.make_key(name, labels);
        let mut histograms = self.histograms.write().await;
        histograms.entry(key.clone()).or_insert_with(Vec::new).push(value);

        // Keep only last 1000 values per histogram
        if let Some(values) = histograms.get_mut(&key) {
            if values.len() > 1000 {
                values.remove(0);
            }
        }
    }

    // Operation metrics
    pub async fn record_operation_duration(&self, operation: &str, duration_ms: f64, success: bool, component: &str) {
        let status = if success { "success" } else { "failure" };
        self.record_histogram(
            "operation_duration_ms",
            duration_ms,
            &[("operation", operation), ("status", status), ("component", component)],
        ).await;
    }

    pub async fn increment_operation_count(&self, operation: &str, status: &str, component: &str) {
        self.increment_counter(
            "operation_total",
            &[("operation", operation), ("status", status), ("component", component)],
        ).await;
    }

    // Resource metrics
    pub async fn update_cpu_usage(&self, usage_percent: f64, component: &str) {
        self.update_gauge(
            "cpu_usage_percent",
            usage_percent,
            &[("component", component)],
        ).await;
    }

    pub async fn update_memory_usage(&self, usage_mb: f64, component: &str) {
        self.update_gauge(
            "memory_usage_mb",
            usage_mb,
            &[("component", component)],
        ).await;
    }

    // Business metrics
    pub async fn record_task_completion(&self, task_type: &str, duration_ms: f64, success: bool) {
        self.record_operation_duration("task_completion", duration_ms, success, "orchestration").await;
        self.increment_operation_count("task_completed", if success { "success" } else { "failure" }, "orchestration").await;

        self.update_gauge(
            "last_task_duration_ms",
            duration_ms,
            &[("task_type", task_type)],
        ).await;
    }

    pub async fn record_council_decision(&self, decision_type: &str, confidence: f64, duration_ms: f64) {
        self.record_histogram(
            "council_decision_duration_ms",
            duration_ms,
            &[("decision_type", decision_type)],
        ).await;

        self.update_gauge(
            "council_decision_confidence",
            confidence,
            &[("decision_type", decision_type)],
        ).await;
    }

    pub async fn record_worker_execution(&self, worker_type: &str, duration_ms: f64, success: bool) {
        self.record_operation_duration("worker_execution", duration_ms, success, "workers").await;
        self.increment_operation_count("worker_execution", if success { "success" } else { "failure" }, "workers").await;

        if success {
            self.update_gauge(
                "last_worker_execution_duration_ms",
                duration_ms,
                &[("worker_type", worker_type)],
            ).await;
        }
    }

    // Learning metrics
    pub async fn record_learning_progress(&self, algorithm: &str, epoch: u32, loss: f64, accuracy: f64) {
        self.update_gauge(
            "learning_loss",
            loss,
            &[("algorithm", algorithm)],
        ).await;

        self.update_gauge(
            "learning_accuracy",
            accuracy,
            &[("algorithm", algorithm)],
        ).await;

        self.update_gauge(
            "learning_epoch",
            epoch as f64,
            &[("algorithm", algorithm)],
        ).await;
    }

    // Error metrics
    pub async fn record_error(&self, error_type: &str, component: &str, operation: &str) {
        self.increment_counter(
            "errors_total",
            &[("error_type", error_type), ("component", component), ("operation", operation)],
        ).await;
    }

    // Export methods
    pub async fn snapshot(&self) -> MetricsSnapshot {
        let counters = self.counters.read().await;
        let gauges = self.gauges.read().await;
        let histograms = self.histograms.read().await;

        let now = chrono::Utc::now();

        let counter_values = counters
            .iter()
            .map(|(key, &value)| {
                let (name, labels) = self.parse_key(key);
                (name, MetricValue {
                    value: value as f64,
                    timestamp: now,
                    labels,
                })
            })
            .collect();

        let gauge_values = gauges
            .iter()
            .map(|(key, &value)| {
                let (name, labels) = self.parse_key(key);
                (name, MetricValue {
                    value,
                    timestamp: now,
                    labels,
                })
            })
            .collect();

        let histogram_values = histograms
            .iter()
            .map(|(key, values)| {
                let (name, labels) = self.parse_key(key);
                let metric_values: Vec<MetricValue> = values
                    .iter()
                    .enumerate()
                    .map(|(i, &value)| MetricValue {
                        value,
                        timestamp: now - chrono::Duration::milliseconds(i as i64 * 100), // Spread timestamps
                        labels: labels.clone(),
                    })
                    .collect();
                (name, metric_values)
            })
            .collect();

        MetricsSnapshot {
            counters: counter_values,
            gauges: gauge_values,
            histograms: histogram_values,
            timestamp: now,
        }
    }

    fn make_key(&self, name: &str, labels: &[(&str, &str)]) -> String {
        let mut key = name.to_string();
        for (k, v) in labels {
            key.push_str(&format!("{{{}}}={{{}}};", k, v));
        }
        key
    }

    fn parse_key(&self, key: &str) -> (String, HashMap<String, String>) {
        let mut labels = HashMap::new();
        let parts: Vec<&str> = key.split('{').collect();

        let name = parts[0].to_string();

        for part in &parts[1..] {
            if let Some(end) = part.find("}=") {
                let label_key = &part[..end];
                if let Some(value_end) = part[end + 2..].find('}') {
                    let label_value = &part[end + 2..end + 2 + value_end];
                    labels.insert(label_key.to_string(), label_value.to_string());
                }
            }
        }

        (name, labels)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collection() {
        let collector = MetricsCollector::new();

        // Test counter
        collector.increment_counter("test_counter", &[("label", "value")]).await;
        collector.increment_counter("test_counter", &[("label", "value")]).await;

        // Test gauge
        collector.update_gauge("test_gauge", 42.0, &[("label", "value")]).await;

        // Test histogram
        collector.record_histogram("test_histogram", 1.5, &[("label", "value")]).await;
        collector.record_histogram("test_histogram", 2.5, &[("label", "value")]).await;

        // Test snapshot
        let snapshot = collector.snapshot().await;
        assert_eq!(snapshot.counters["test_counter"].value, 2.0);
        assert_eq!(snapshot.gauges["test_gauge"].value, 42.0);
        assert_eq!(snapshot.histograms["test_histogram"].len(), 2);
    }
}