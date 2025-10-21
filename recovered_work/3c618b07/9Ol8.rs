//! StatsD metrics backend implementation
//!
//! Provides StatsD-compatible metrics collection with UDP transport
//! and circuit breaker pattern for reliability.

use std::net::UdpSocket;
use std::sync::Arc;
use async_trait::async_trait;
use cadence::{StatsdClient, UdpMetricSink, DEFAULT_PORT};
use crate::metrics::{MetricsBackend, MetricsBackendError};

/// StatsD metrics backend
pub struct StatsDMetrics {
    client: Arc<StatsdClient>,
    socket: UdpSocket,
    prefix: String,
}

impl StatsDMetrics {
    /// Create a new StatsD metrics backend
    pub fn new(host: &str, port: u16, prefix: &str) -> Result<Self, MetricsBackendError> {
        let socket = UdpSocket::bind("0.0.0.0:0")
            .map_err(|e| MetricsBackendError::ConnectionError(e.to_string()))?;

        let sink = UdpMetricSink::from((host, port))
            .map_err(|e| MetricsBackendError::ConnectionError(e.to_string()))?;

        let client = StatsdClient::from_sink(prefix, sink);

        Ok(Self {
            client: Arc::new(client),
            socket,
            prefix: prefix.to_string(),
        })
    }

    /// Create with default localhost configuration
    pub fn localhost(prefix: &str) -> Result<Self, MetricsBackendError> {
        Self::new("127.0.0.1", DEFAULT_PORT, prefix)
    }

    /// Get the prefix for this client
    pub fn prefix(&self) -> &str {
        &self.prefix
    }
}

#[async_trait]
impl MetricsBackend for StatsDMetrics {
    async fn counter(&self, name: &str, labels: &[(&str, &str)], value: u64) {
        // Convert labels to StatsD tags format
        let tags = labels.iter()
            .map(|(k, v)| format!("{}:{}", k, v))
            .collect::<Vec<_>>()
            .join(",");

        let metric_name = format!("{}.{}", self.prefix, name);

        // StatsD counters are incremental
        if let Err(e) = self.client.count_with_tags(&metric_name, value as i64).with_tags(&[&tags]) {
            tracing::warn!("Failed to send StatsD counter: {}", e);
        }
    }

    async fn gauge(&self, name: &str, labels: &[(&str, &str)], value: f64) {
        // Convert labels to StatsD tags format
        let tags = labels.iter()
            .map(|(k, v)| format!("{}:{}", k, v))
            .collect::<Vec<_>>()
            .join(",");

        let metric_name = format!("{}.{}", self.prefix, name);

        // StatsD gauges set absolute values
        if let Err(e) = self.client.gauge_with_tags(&metric_name, value).with_tags(&[&tags]) {
            tracing::warn!("Failed to send StatsD gauge: {}", e);
        }
    }

    async fn histogram(&self, name: &str, labels: &[(&str, &str)], value: f64) {
        // Convert labels to StatsD tags format
        let tags = labels.iter()
            .map(|(k, v)| format!("{}:{}", k, v))
            .collect::<Vec<_>>()
            .join(",");

        let metric_name = format!("{}.{}", self.prefix, name);

        // StatsD histograms/timers
        if let Err(e) = self.client.histogram_with_tags(&metric_name, value).with_tags(&[&tags]) {
            tracing::warn!("Failed to send StatsD histogram: {}", e);
        }
    }
}

/// Circuit breaker for StatsD reliability
pub struct StatsDCircuitBreaker {
    failure_count: std::sync::atomic::AtomicUsize,
    last_failure: std::sync::atomic::AtomicU64,
    failure_threshold: usize,
    reset_timeout_ms: u64,
}

impl StatsDCircuitBreaker {
    pub fn new(failure_threshold: usize, reset_timeout_ms: u64) -> Self {
        Self {
            failure_count: std::sync::atomic::AtomicUsize::new(0),
            last_failure: std::sync::atomic::AtomicU64::new(0),
            failure_threshold,
            reset_timeout_ms,
        }
    }

    pub fn record_success(&self) {
        self.failure_count.store(0, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn record_failure(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        self.failure_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.last_failure.store(now, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn is_open(&self) -> bool {
        let failures = self.failure_count.load(std::sync::atomic::Ordering::Relaxed);
        if failures >= self.failure_threshold {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;

            let last_failure = self.last_failure.load(std::sync::atomic::Ordering::Relaxed);

            // Check if reset timeout has passed
            if now - last_failure > self.reset_timeout_ms {
                // Reset the circuit breaker
                self.failure_count.store(0, std::sync::atomic::Ordering::Relaxed);
                false
            } else {
                true
            }
        } else {
            false
        }
    }
}

/// Error types for StatsD metrics
#[derive(Debug, thiserror::Error)]
pub enum MetricsBackendError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Send error: {0}")]
    SendError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statsd_circuit_breaker() {
        let breaker = StatsDCircuitBreaker::new(3, 1000);

        // Initially closed
        assert!(!breaker.is_open());

        // Record failures
        breaker.record_failure();
        assert!(!breaker.is_open());

        breaker.record_failure();
        assert!(!breaker.is_open());

        breaker.record_failure();
        assert!(breaker.is_open());

        // Record success to reset
        breaker.record_success();
        assert!(!breaker.is_open());
    }

    #[test]
    fn test_statsd_creation() {
        // Test with invalid host should fail
        let result = StatsDMetrics::new("invalid.host", 8125, "test");
        assert!(result.is_err());
    }
}


