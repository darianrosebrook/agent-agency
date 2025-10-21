//! Circuit Breaker Pattern Implementation
//!
//! Provides resilience against cascading failures by monitoring external service calls
//! and temporarily stopping requests to services that are failing.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    /// Normal operation - requests pass through
    Closed,
    /// Service is failing - requests are blocked
    Open,
    /// Testing if service has recovered - limited requests allowed
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening the circuit
    pub failure_threshold: u64,
    /// Number of successes needed to close circuit from half-open
    pub success_threshold: u64,
    /// How long to wait before trying half-open (seconds)
    pub recovery_timeout_secs: u64,
    /// Timeout for individual requests (seconds)
    pub request_timeout_secs: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            recovery_timeout_secs: 30,
            request_timeout_secs: 10,
        }
    }
}

/// Metrics for circuit breaker monitoring
#[derive(Debug, Clone)]
pub struct CircuitBreakerMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub rejected_requests: u64,
    pub current_state: CircuitState,
    pub last_failure_time: Option<Instant>,
}

/// Circuit breaker for external service calls
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitState>>,
    failures: Arc<AtomicU64>,
    successes: Arc<AtomicU64>,
    total_requests: Arc<AtomicU64>,
    successful_requests: Arc<AtomicU64>,
    rejected_requests: Arc<AtomicU64>,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with default configuration
    pub fn new() -> Self {
        Self::with_config(CircuitBreakerConfig::default())
    }

    /// Create a new circuit breaker with custom configuration
    pub fn with_config(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failures: Arc::new(AtomicU64::new(0)),
            successes: Arc::new(AtomicU64::new(0)),
            total_requests: Arc::new(AtomicU64::new(0)),
            successful_requests: Arc::new(AtomicU64::new(0)),
            rejected_requests: Arc::new(AtomicU64::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
        }
    }

    /// Check if a request should be allowed through
    pub async fn should_allow_request(&self) -> bool {
        let state = *self.state.read().await;

        match state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if recovery timeout has elapsed
                if let Some(last_failure) = *self.last_failure_time.read().await {
                    if last_failure.elapsed() >= Duration::from_secs(self.config.recovery_timeout_secs) {
                        // Transition to half-open
                        *self.state.write().await = CircuitState::HalfOpen;
                        self.successes.store(0, Ordering::SeqCst);
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => {
                // Allow limited requests in half-open state
                self.successes.load(Ordering::SeqCst) < self.config.success_threshold
            }
        }
    }

    /// Record a successful request
    pub async fn record_success(&self) {
        self.successful_requests.fetch_add(1, Ordering::SeqCst);
        self.total_requests.fetch_add(1, Ordering::SeqCst);

        let mut state = self.state.write().await;
        match *state {
            CircuitState::Closed => {
                // Reset failure count on success
                self.failures.store(0, Ordering::SeqCst);
            }
            CircuitState::HalfOpen => {
                let successes = self.successes.fetch_add(1, Ordering::SeqCst) + 1;
                if successes >= self.config.success_threshold {
                    // Service has recovered
                    *state = CircuitState::Closed;
                    self.failures.store(0, Ordering::SeqCst);
                    tracing::info!("Circuit breaker closed - service recovered");
                }
            }
            CircuitState::Open => {} // Should not happen, but ignore
        }
    }

    /// Record a failed request
    pub async fn record_failure(&self) {
        self.total_requests.fetch_add(1, Ordering::SeqCst);
        *self.last_failure_time.write().await = Some(Instant::now());

        let failures = self.failures.fetch_add(1, Ordering::SeqCst) + 1;

        if failures >= self.config.failure_threshold {
            let mut state = self.state.write().await;
            if *state == CircuitState::Closed {
                *state = CircuitState::Open;
                tracing::warn!("Circuit breaker opened - too many failures ({})", failures);
            }
        }
    }

    /// Record a rejected request (circuit was open)
    pub async fn record_rejection(&self) {
        self.rejected_requests.fetch_add(1, Ordering::SeqCst);
        self.total_requests.fetch_add(1, Ordering::SeqCst);
    }

    /// Get current metrics
    pub async fn metrics(&self) -> CircuitBreakerMetrics {
        CircuitBreakerMetrics {
            total_requests: self.total_requests.load(Ordering::SeqCst),
            successful_requests: self.successful_requests.load(Ordering::SeqCst),
            failed_requests: self.total_requests.load(Ordering::SeqCst) - self.successful_requests.load(Ordering::SeqCst) - self.rejected_requests.load(Ordering::SeqCst),
            rejected_requests: self.rejected_requests.load(Ordering::SeqCst),
            current_state: *self.state.read().await,
            last_failure_time: *self.last_failure_time.read().await,
        }
    }

    /// Manually reset the circuit breaker
    pub async fn reset(&self) {
        *self.state.write().await = CircuitState::Closed;
        self.failures.store(0, Ordering::SeqCst);
        self.successes.store(0, Ordering::SeqCst);
        *self.last_failure_time.write().await = None;
        tracing::info!("Circuit breaker manually reset");
    }
}

/// HTTP client with circuit breaker integration
pub struct ResilientHttpClient {
    client: reqwest::Client,
    circuit_breaker: CircuitBreaker,
}

impl ResilientHttpClient {
    pub fn new() -> Self {
        Self::with_config(CircuitBreakerConfig::default())
    }

    pub fn with_config(config: CircuitBreakerConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.request_timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            circuit_breaker: CircuitBreaker::with_config(config),
        }
    }

    /// Make a request with circuit breaker protection
    pub async fn request(&self, request_builder: reqwest::RequestBuilder) -> Result<reqwest::Response, ResilientHttpError> {
        if !self.circuit_breaker.should_allow_request().await {
            self.circuit_breaker.record_rejection().await;
            return Err(ResilientHttpError::CircuitOpen);
        }

        match request_builder.send().await {
            Ok(response) => {
                if response.status().is_success() {
                    self.circuit_breaker.record_success().await;
                    Ok(response)
                } else {
                    self.circuit_breaker.record_failure().await;
                    Err(ResilientHttpError::Http(response.status()))
                }
            }
            Err(e) => {
                self.circuit_breaker.record_failure().await;
                Err(ResilientHttpError::Network(e))
            }
        }
    }

    /// Get circuit breaker metrics
    pub async fn circuit_metrics(&self) -> CircuitBreakerMetrics {
        self.circuit_breaker.metrics().await
    }

    /// Reset the circuit breaker
    pub async fn reset_circuit(&self) {
        self.circuit_breaker.reset().await;
    }
}

/// Errors from resilient HTTP client
#[derive(Debug, thiserror::Error)]
pub enum ResilientHttpError {
    #[error("Circuit breaker is open")]
    CircuitOpen,

    #[error("HTTP error: {0}")]
    Http(reqwest::StatusCode),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_circuit_breaker_closed_state() {
        let cb = CircuitBreaker::new();
        assert!(cb.should_allow_request().await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_failure_threshold() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            ..Default::default()
        };
        let cb = CircuitBreaker::with_config(config);

        // Should start closed
        assert!(cb.should_allow_request().await);

        // Record failures
        for _ in 0..2 {
            cb.record_failure().await;
            assert!(cb.should_allow_request().await);
        }

        // Third failure should open circuit
        cb.record_failure().await;
        assert!(!cb.should_allow_request().await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_recovery() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            recovery_timeout_secs: 1,
            ..Default::default()
        };
        let cb = CircuitBreaker::with_config(config);

        // Open circuit
        cb.record_failure().await;
        cb.record_failure().await;
        assert!(!cb.should_allow_request().await);

        // Wait for recovery timeout
        sleep(Duration::from_secs(2)).await;

        // Should allow request in half-open state
        assert!(cb.should_allow_request().await);

        // Record successes to close circuit
        cb.record_success().await;
        cb.record_success().await;

        // Circuit should be closed
        assert!(cb.should_allow_request().await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_metrics() {
        let cb = CircuitBreaker::new();

        cb.record_success().await;
        cb.record_failure().await;
        cb.record_rejection().await;

        let metrics = cb.metrics().await;
        assert_eq!(metrics.total_requests, 3);
        assert_eq!(metrics.successful_requests, 1);
        assert_eq!(metrics.failed_requests, 1);
        assert_eq!(metrics.rejected_requests, 1);
    }
}
