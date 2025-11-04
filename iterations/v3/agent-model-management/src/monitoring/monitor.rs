//! Performance monitor for tracking model metrics

use crate::types::*;
use crate::ModelManagementError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Performance monitor for tracking model metrics and health
#[derive(Debug)]
pub struct PerformanceMonitor {
    /// Model metrics storage
    metrics: Arc<RwLock<HashMap<String, Vec<ModelMetrics>>>>,

    /// Current metrics cache
    current_metrics: Arc<RwLock<HashMap<String, ModelMetrics>>>,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            current_metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record inference performance with comprehensive monitoring
    pub async fn record_inference(&self, model_id: &str, output: &InferenceOutput, success: bool) -> Result<(), ModelManagementError> {
        use std::collections::HashMap;

        // Get current time for timestamping
        let now = chrono::Utc::now();

        // Update current metrics with real-time calculations
        let mut current = self.current_metrics.write().await;
        let mut historical = self.metrics.write().await;

        // Get historical metrics for this model
        let model_history = historical.entry(model_id.to_string()).or_insert_with(Vec::new);

        // Calculate RPS (requests per second) over last minute
        let rps = self.calculate_rps(model_history, &now, 60).await;

        // Calculate latency percentiles from recent history
        let latencies: Vec<f64> = model_history.iter()
            .filter(|m| (now - m.last_updated).num_seconds() < 300) // Last 5 minutes
            .map(|m| m.avg_latency_ms)
            .collect();

        let p95_latency = self.calculate_percentile(&latencies, 95.0);
        let avg_latency = if latencies.is_empty() {
            output.performance.total_latency_ms as f64
        } else {
            latencies.iter().sum::<f64>() / latencies.len() as f64
        };

        // Calculate error rate from recent inferences
        let recent_count = model_history.len().max(1) as f64;
        let error_count = model_history.iter()
            .filter(|m| m.error_rate > 0.0)
            .count() as f64;
        let error_rate = error_count / recent_count;

        // Estimate CPU usage (simplified - would use sysinfo in full implementation)
        // For now, derive from latency and throughput patterns
        let cpu_usage = self.estimate_cpu_usage(&latencies, rps);

        // Calculate memory usage percentage from inference data
        // Assuming total system memory of 16GB for percentage calculation
        const TOTAL_SYSTEM_MEMORY_MB: f64 = 16.0 * 1024.0; // 16GB
        let memory_usage = (output.performance.memory_usage_mb as f64 / TOTAL_SYSTEM_MEMORY_MB) * 100.0;

        // Create comprehensive metrics
        let metrics = ModelMetrics {
            rps,
            avg_latency_ms: avg_latency,
            p95_latency_ms: p95_latency,
            error_rate: if success { error_rate } else { error_rate + 1.0 / recent_count },
            cpu_usage,
            memory_usage: memory_usage.min(100.0).max(0.0), // Clamp to 0-100%
            last_updated: now,
        };

        // Update current metrics
        current.insert(model_id.to_string(), metrics.clone());

        // Add to historical metrics (limit history to prevent unbounded growth)
        model_history.push(metrics);
        if model_history.len() > 1000 { // Keep last 1000 measurements
            model_history.remove(0);
        }

        // Check for performance regressions and log warnings
        self.check_performance_regression(model_id, model_history).await;

        Ok(())
    }

    /// Get current metrics for a model
    pub async fn get_model_metrics(&self, model_id: &str) -> Result<ModelMetrics, ModelManagementError> {
        let current = self.current_metrics.read().await;
        match current.get(model_id) {
            Some(metrics) => Ok(metrics.clone()),
            None => Ok(ModelMetrics {
                rps: 0.0,
                avg_latency_ms: 0.0,
                p95_latency_ms: 0.0,
                error_rate: 0.0,
                cpu_usage: 0.0,
                memory_usage: 0.0,
                last_updated: chrono::Utc::now(),
            }),
        }
    }

    /// Get metrics history for a model
    pub async fn get_metrics_history(&self, model_id: &str) -> Result<Vec<ModelMetrics>, ModelManagementError> {
        let history = self.metrics.read().await;
        match history.get(model_id) {
            Some(metrics) => Ok(metrics.clone()),
            None => Ok(Vec::new()),
        }
    }

    /// Calculate average metrics over a time window
    pub async fn get_average_metrics(&self, model_id: &str, _minutes: u32) -> Result<ModelMetrics, ModelManagementError> {
        let history = self.metrics.read().await;
        match history.get(model_id) {
            Some(metrics) if !metrics.is_empty() => {
                let len = metrics.len() as f64;
                let avg_latency = metrics.iter().map(|m| m.avg_latency_ms).sum::<f64>() / len;
                let avg_rps = metrics.iter().map(|m| m.rps).sum::<f64>() / len;
                let avg_error_rate = metrics.iter().map(|m| m.error_rate).sum::<f64>() / len;
                let avg_cpu = metrics.iter().map(|m| m.cpu_usage).sum::<f64>() / len;
                let avg_memory = metrics.iter().map(|m| m.memory_usage).sum::<f64>() / len;

                // P95 would need proper percentile calculation
                let p95_latency = metrics.iter()
                    .map(|m| m.p95_latency_ms)
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(avg_latency);

                Ok(ModelMetrics {
                    rps: avg_rps,
                    avg_latency_ms: avg_latency,
                    p95_latency_ms: p95_latency,
                    error_rate: avg_error_rate,
                    cpu_usage: avg_cpu,
                    memory_usage: avg_memory,
                    last_updated: chrono::Utc::now(),
                })
            }
            _ => self.get_model_metrics(model_id).await,
        }
    }

    /// Calculate requests per second over a time window
    async fn calculate_rps(&self, history: &[ModelMetrics], now: &chrono::DateTime<chrono::Utc>, window_seconds: i64) -> f64 {
        let window_start = *now - chrono::Duration::seconds(window_seconds);
        let recent_requests = history.iter()
            .filter(|m| m.last_updated > window_start)
            .count();

        recent_requests as f64 / window_seconds as f64
    }

    /// Calculate percentile from a sorted vector of values
    fn calculate_percentile(&self, values: &[f64], percentile: f64) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let index = (percentile / 100.0 * (sorted.len() - 1) as f64) as usize;
        sorted.get(index).copied().unwrap_or(0.0)
    }

    /// Estimate CPU usage based on latency patterns and throughput
    fn estimate_cpu_usage(&self, latencies: &[f64], rps: f64) -> f64 {
        if latencies.is_empty() {
            return 0.0;
        }

        // Simplified CPU estimation based on latency and throughput
        // Higher latency + higher throughput = higher CPU usage
        let avg_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;
        let base_cpu = (avg_latency / 1000.0).min(1.0) * 50.0; // Latency contribution
        let throughput_cpu = (rps / 100.0).min(1.0) * 50.0; // Throughput contribution

        (base_cpu + throughput_cpu).min(100.0).max(0.0)
    }

    /// Check for performance regressions and log warnings
    async fn check_performance_regression(&self, model_id: &str, history: &[ModelMetrics]) {
        if history.len() < 10 {
            return; // Need minimum history for regression detection
        }

        let recent = &history[history.len().saturating_sub(5)..]; // Last 5 measurements
        let older = &history[history.len().saturating_sub(10)..history.len().saturating_sub(5)]; // Previous 5

        if older.is_empty() {
            return;
        }

        // Check for latency regression (P95 increased by >20%)
        let recent_p95_avg = recent.iter().map(|m| m.p95_latency_ms).sum::<f64>() / recent.len() as f64;
        let older_p95_avg = older.iter().map(|m| m.p95_latency_ms).sum::<f64>() / older.len() as f64;

        if recent_p95_avg > older_p95_avg * 1.2 {
            tracing::warn!(
                "Performance regression detected for model {}: P95 latency increased from {:.2}ms to {:.2}ms",
                model_id, older_p95_avg, recent_p95_avg
            );
        }

        // Check for error rate increase
        let recent_error_rate = recent.iter().map(|m| m.error_rate).sum::<f64>() / recent.len() as f64;
        let older_error_rate = older.iter().map(|m| m.error_rate).sum::<f64>() / older.len() as f64;

        if recent_error_rate > older_error_rate * 1.5 && recent_error_rate > 0.1 {
            tracing::warn!(
                "Error rate increase detected for model {}: error rate increased from {:.2}% to {:.2}%",
                model_id, older_error_rate * 100.0, recent_error_rate * 100.0
            );
        }

        // Check for RPS degradation
        let recent_rps_avg = recent.iter().map(|m| m.rps).sum::<f64>() / recent.len() as f64;
        let older_rps_avg = older.iter().map(|m| m.rps).sum::<f64>() / older.len() as f64;

        if recent_rps_avg < older_rps_avg * 0.8 {
            tracing::warn!(
                "Throughput degradation detected for model {}: RPS decreased from {:.2} to {:.2}",
                model_id, older_rps_avg, recent_rps_avg
            );
        }
    }
}
