//! Redis metrics backend implementation
//!
//! Provides Redis-backed metrics collection with TTL support,
//! connection pooling, and circuit breaker pattern for reliability.

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use redis::{Client, Commands, Connection, PipelineCommands};
use tokio::sync::Mutex;
use crate::metrics::{MetricsBackend, MetricsBackendError};

/// Redis metrics backend with connection pooling and TTL
pub struct RedisMetrics {
    client: Client,
    ttl_seconds: usize,
    prefix: String,
    circuit_breaker: Arc<RedisCircuitBreaker>,
}

impl RedisMetrics {
    /// Create a new Redis metrics backend
    pub fn new(redis_url: &str, prefix: &str, ttl_seconds: usize) -> Result<Self, MetricsBackendError> {
        let client = Client::open(redis_url)
            .map_err(|e| MetricsBackendError::ConnectionError(e.to_string()))?;

        Ok(Self {
            client,
            ttl_seconds,
            prefix: prefix.to_string(),
            circuit_breaker: Arc::new(RedisCircuitBreaker::new(5, 30000)), // 5 failures, 30s reset
        })
    }

    /// Create with default localhost configuration
    pub fn localhost(prefix: &str) -> Result<Self, MetricsBackendError> {
        Self::new("redis://127.0.0.1:6379", prefix, 3600) // 1 hour TTL
    }

    /// Get a Redis connection (internal use)
    fn get_connection(&self) -> Result<Connection, MetricsBackendError> {
        if self.circuit_breaker.is_open() {
            return Err(MetricsBackendError::CircuitBreakerOpen);
        }

        match self.client.get_connection() {
            Ok(conn) => {
                self.circuit_breaker.record_success();
                Ok(conn)
            }
            Err(e) => {
                self.circuit_breaker.record_failure();
                Err(MetricsBackendError::ConnectionError(e.to_string()))
            }
        }
    }

    /// Create a Redis key for a metric
    fn make_key(&self, name: &str, labels: &[(&str, &str)]) -> String {
        let mut key = format!("{}:{}", self.prefix, name);
        for (k, v) in labels {
            key.push_str(&format!(":{}:{}", k, v));
        }
        key
    }

    /// Execute Redis command with error handling
    async fn execute_command<T, F>(&self, operation: F) -> Result<T, MetricsBackendError>
    where
        F: FnOnce(Connection) -> redis::RedisResult<T>,
    {
        let conn = self.get_connection()?;
        operation(conn).map_err(|e| MetricsBackendError::CommandError(e.to_string()))
    }
}

#[async_trait]
impl MetricsBackend for RedisMetrics {
    async fn counter(&self, name: &str, labels: &[(&str, &str)], value: u64) {
        if self.circuit_breaker.is_open() {
            tracing::warn!("Redis circuit breaker open, skipping counter increment");
            return;
        }

        let key = self.make_key(name, labels);
        let result: Result<(), MetricsBackendError> = self.execute_command(|mut conn| {
            // Use INCRBY for atomic increment
            let _: () = conn.incr(&key, value)?;
            // Set TTL if not already set
            let _: () = conn.expire(&key, self.ttl_seconds)?;
            Ok(())
        }).await;

        if let Err(e) = result {
            tracing::warn!("Failed to increment Redis counter: {}", e);
        }
    }

    async fn gauge(&self, name: &str, labels: &[(&str, &str)], value: f64) {
        if self.circuit_breaker.is_open() {
            tracing::warn!("Redis circuit breaker open, skipping gauge update");
            return;
        }

        let key = self.make_key(name, labels);
        let result: Result<(), MetricsBackendError> = self.execute_command(|mut conn| {
            // Store as string since Redis doesn't have native float support
            let _: () = conn.set(&key, value.to_string())?;
            // Set TTL
            let _: () = conn.expire(&key, self.ttl_seconds)?;
            Ok(())
        }).await;

        if let Err(e) = result {
            tracing::warn!("Failed to update Redis gauge: {}", e);
        }
    }

    async fn histogram(&self, name: &str, labels: &[(&str, &str)], value: f64) {
        if self.circuit_breaker.is_open() {
            tracing::warn!("Redis circuit breaker open, skipping histogram observation");
            return;
        }

        let key = self.make_key(name, labels);
        let result: Result<(), MetricsBackendError> = self.execute_command(|mut conn| {
            // Use RPUSH to append to list (simulating histogram buckets)
            let _: () = conn.rpush(&key, value.to_string())?;
            // Keep only last 1000 values
            let _: () = conn.ltrim(&key, -1000, -1)?;
            // Set TTL
            let _: () = conn.expire(&key, self.ttl_seconds)?;
            Ok(())
        }).await;

        if let Err(e) = result {
            tracing::warn!("Failed to record Redis histogram: {}", e);
        }
    }
}

/// Circuit breaker for Redis reliability
pub struct RedisCircuitBreaker {
    failure_count: std::sync::atomic::AtomicUsize,
    last_failure: std::sync::atomic::AtomicU64,
    failure_threshold: usize,
    reset_timeout_ms: u64,
}

impl RedisCircuitBreaker {
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

/// Error types for Redis metrics
#[derive(Debug, thiserror::Error)]
pub enum MetricsBackendError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Command error: {0}")]
    CommandError(String),

    #[error("Circuit breaker is open")]
    CircuitBreakerOpen,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redis_circuit_breaker() {
        let breaker = RedisCircuitBreaker::new(3, 1000);

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
    fn test_redis_key_generation() {
        let metrics = RedisMetrics::localhost("test").unwrap();
        let key = metrics.make_key("counter", &[("component", "api"), ("status", "success")]);
        assert_eq!(key, "test:counter:component:api:status:success");
    }
}
