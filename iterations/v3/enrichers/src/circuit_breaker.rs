//! @darianrosebrook
//! Circuit breaker pattern for enricher resilience

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,    // Normal operation
    Open,      // Too many failures, reject calls
    HalfOpen,  // Testing if service recovered
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
pub struct CircuitBreaker {
    state: Arc<Mutex<CircuitState>>,
    failure_count: Arc<Mutex<usize>>,
    success_count: Arc<Mutex<usize>>,
    last_failure_time: Arc<Mutex<Option<Instant>>>,
    config: CircuitBreakerConfig,
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

    /// Check if circuit is available (Closed or HalfOpen)
    pub fn is_available(&self) -> bool {
        let state = *self.state.lock().unwrap();

        match state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if timeout has passed
                if let Some(last_failure) = *self.last_failure_time.lock().unwrap() {
                    if last_failure.elapsed() >= self.config.timeout {
                        // Transition to HalfOpen for testing
                        *self.state.lock().unwrap() = CircuitState::HalfOpen;
                        *self.success_count.lock().unwrap() = 0;
                        tracing::debug!("Circuit breaker transitioning to HalfOpen");
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Record a success
    pub fn record_success(&self) {
        let state = *self.state.lock().unwrap();

        match state {
            CircuitState::Closed => {
                // Reset failure count on success
                *self.failure_count.lock().unwrap() = 0;
            }
            CircuitState::HalfOpen => {
                let mut success_count = self.success_count.lock().unwrap();
                *success_count += 1;

                if *success_count >= self.config.success_threshold {
                    // Close circuit after successful recovery
                    *self.state.lock().unwrap() = CircuitState::Closed;
                    *self.failure_count.lock().unwrap() = 0;
                    *self.success_count.lock().unwrap() = 0;
                    tracing::info!("Circuit breaker closed - service recovered");
                }
            }
            CircuitState::Open => {
                // Success while open doesn't do anything (waiting for timeout)
            }
        }
    }

    /// Record a failure
    pub fn record_failure(&self) {
        let state = *self.state.lock().unwrap();

        match state {
            CircuitState::Closed => {
                let mut failure_count = self.failure_count.lock().unwrap();
                *failure_count += 1;

                if *failure_count >= self.config.failure_threshold {
                    // Open circuit after too many failures
                    *self.state.lock().unwrap() = CircuitState::Open;
                    *self.last_failure_time.lock().unwrap() = Some(Instant::now());
                    tracing::warn!("Circuit breaker opened after {} failures", failure_count);
                }
            }
            CircuitState::HalfOpen => {
                // Failure in HalfOpen goes back to Open
                *self.state.lock().unwrap() = CircuitState::Open;
                *self.last_failure_time.lock().unwrap() = Some(Instant::now());
                *self.success_count.lock().unwrap() = 0;
                tracing::warn!("Circuit breaker re-opened during HalfOpen test");
            }
            CircuitState::Open => {
                // Failure while open, just update last failure time
                *self.last_failure_time.lock().unwrap() = Some(Instant::now());
            }
        }
    }

    /// Get current state
    pub fn state(&self) -> CircuitState {
        *self.state.lock().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_basic() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 1,
            timeout: Duration::from_secs(1),
        });

        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.is_available());

        // Record failures
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.is_available());
    }

    #[test]
    fn test_circuit_breaker_recovery() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 1,
            timeout: Duration::from_millis(10),
        });

        // Fail once
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        // Wait for timeout
        std::thread::sleep(Duration::from_millis(50));

        // Should be available for testing
        assert!(cb.is_available());
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        // Success should close circuit
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }
}
