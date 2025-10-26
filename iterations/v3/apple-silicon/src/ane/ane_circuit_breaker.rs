//! Circuit breaker pattern for ANE model resilience
//!
//! This module provides circuit breaker functionality for protecting
//! against cascading failures in Core ML model inference operations.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,   // Normal operation
    Open,     // Too many failures, reject calls
    HalfOpen, // Testing if service recovered
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: usize,
    pub success_threshold: usize,
    pub timeout: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 3,
            success_threshold: 2,
            timeout: Duration::from_secs(60),
        }
    }
}

/// Circuit breaker for protecting against cascading failures
#[derive(Debug)]
pub struct CircuitBreaker {
    state: Arc<Mutex<CircuitState>>,
    failure_count: Arc<Mutex<usize>>,
    success_count: Arc<Mutex<usize>>,
    last_failure_time: Arc<Mutex<Option<Instant>>>,
    config: CircuitBreakerConfig,
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitState::Closed)),
            failure_count: Arc::new(Mutex::new(0)),
            success_count: Arc::new(Mutex::new(0)),
            last_failure_time: Arc::new(Mutex::new(None)),
            config,
        }
    }

    /// Execute an operation with circuit breaker protection
    pub async fn execute<F, Fut, T>(&self, operation: F) -> Result<T, CircuitBreakerError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>>,
    {
        // Check if circuit is open
        if self.is_open() {
            return Err(CircuitBreakerError::CircuitOpen);
        }

        // Execute operation
        match operation().await {
            Ok(result) => {
                self.record_success();
                Ok(result)
            }
            Err(e) => {
                self.record_failure();
                Err(CircuitBreakerError::OperationFailed(e))
            }
        }
    }

    /// Check if circuit is open
    pub fn is_open(&self) -> bool {
        let state = *self.state.lock().unwrap();
        match state {
            CircuitState::Open => {
                // Check if timeout has elapsed
                if let Some(last_failure) = *self.last_failure_time.lock().unwrap() {
                    if last_failure.elapsed() >= self.config.timeout {
                        self.transition_to_half_open();
                        false
                    } else {
                        true
                    }
                } else {
                    true
                }
            }
            CircuitState::HalfOpen => false,
            CircuitState::Closed => false,
        }
    }

    /// Record a successful operation
    pub fn record_success(&self) {
        let state = self.state.lock().unwrap();
        let mut success_count = self.success_count.lock().unwrap();

        match *state {
            CircuitState::HalfOpen => {
                *success_count += 1;
                if *success_count >= self.config.success_threshold {
                    self.transition_to_closed();
                }
            }
            CircuitState::Closed => {
                // Reset success count
                *success_count = 0;
            }
            CircuitState::Open => {
                // Should not happen, but ignore
            }
        }
    }

    /// Record a failed operation
    pub fn record_failure(&self) {
        let mut failure_count = self.failure_count.lock().unwrap();
        *failure_count += 1;

        if *failure_count >= self.config.failure_threshold {
            self.transition_to_open();
        }
    }

    fn transition_to_open(&self) {
        let mut state = self.state.lock().unwrap();
        let mut last_failure_time = self.last_failure_time.lock().unwrap();
        *state = CircuitState::Open;
        *last_failure_time = Some(Instant::now());
    }

    fn transition_to_half_open(&self) {
        let mut state = self.state.lock().unwrap();
        *state = CircuitState::HalfOpen;
    }

    fn transition_to_closed(&self) {
        let mut state = self.state.lock().unwrap();
        let mut failure_count = self.failure_count.lock().unwrap();
        let mut success_count = self.success_count.lock().unwrap();

        *state = CircuitState::Closed;
        *failure_count = 0;
        *success_count = 0;
    }

    /// Get current circuit state
    pub fn state(&self) -> CircuitState {
        *self.state.lock().unwrap()
    }

    /// Get failure count
    pub fn failure_count(&self) -> usize {
        *self.failure_count.lock().unwrap()
    }

    /// Get success count
    pub fn success_count(&self) -> usize {
        *self.success_count.lock().unwrap()
    }

    /// Acquire a permit for execution (async version)
    pub async fn acquire(&self) -> Result<CircuitBreakerPermit, CircuitBreakerError> {
        if self.is_open() {
            return Err(CircuitBreakerError::CircuitOpen);
        }
        Ok(CircuitBreakerPermit { circuit_breaker: self })
    }
}

/// Permit for circuit breaker execution
pub struct CircuitBreakerPermit<'a> {
    circuit_breaker: &'a CircuitBreaker,
}

impl<'a> Drop for CircuitBreakerPermit<'a> {
    fn drop(&mut self) {
        // Permit is automatically released when dropped
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CircuitBreakerError {
    #[error("Circuit breaker is open")]
    CircuitOpen,

    #[error("Operation failed: {0}")]
    OperationFailed(#[source] Box<dyn std::error::Error + Send + Sync>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_circuit_breaker_creation() {
        let config = CircuitBreakerConfig::default();
        let cb = CircuitBreaker::new(config);

        assert_eq!(cb.state(), CircuitState::Closed);
        assert_eq!(cb.failure_count(), 0);
        assert_eq!(cb.success_count(), 0);
    }

    #[test]
    fn test_circuit_breaker_failure_threshold() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 1,
            timeout: Duration::from_secs(1),
        };
        let cb = CircuitBreaker::new(config);

        // Record failures
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn test_circuit_breaker_success_recovery() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 1,
            timeout: Duration::from_millis(100),
        };
        let cb = CircuitBreaker::new(config);

        // Cause failure
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        // Wait for timeout
        std::thread::sleep(Duration::from_millis(150));
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        // Record success
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }
}
