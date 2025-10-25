//! Circuit breaker pattern for database resilience
//!
//! Implements the circuit breaker pattern to prevent cascading failures
//! and provide graceful degradation during database connectivity issues.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration as StdDuration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Circuit breaker states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// Circuit breaker for database operations
///
/// Implements the circuit breaker pattern to handle database failures gracefully:
/// - Closed: Normal operation, requests pass through
/// - Open: Failure threshold exceeded, requests fail fast
/// - HalfOpen: Testing recovery, limited requests allowed
#[derive(Debug)]
pub struct CircuitBreaker {
    /// Failure threshold before opening circuit
    failure_threshold: u32,
    /// Success threshold to close circuit
    success_threshold: u32,
    /// Timeout before attempting recovery
    recovery_timeout: chrono::Duration,
    /// Current state
    state: Arc<RwLock<CircuitState>>,
    /// Consecutive failures
    failures: AtomicU64,
    /// Consecutive successes
    successes: AtomicU64,
    /// Last failure time
    last_failure: Arc<RwLock<Option<Instant>>>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with default settings
    pub fn new() -> Self {
        Self {
            failure_threshold: 3,
            success_threshold: 5,
            recovery_timeout: chrono::Duration::seconds(30),
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failures: AtomicU64::new(0),
            successes: AtomicU64::new(0),
            last_failure: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a circuit breaker with custom settings
    pub fn with_config(
        failure_threshold: u32,
        success_threshold: u32,
        recovery_timeout_seconds: i64,
    ) -> Self {
        Self {
            failure_threshold,
            success_threshold,
            recovery_timeout: chrono::Duration::seconds(recovery_timeout_seconds),
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failures: AtomicU64::new(0),
            successes: AtomicU64::new(0),
            last_failure: Arc::new(RwLock::new(None)),
        }
    }

    /// Check if the circuit breaker allows the operation
    pub async fn can_execute(&self) -> Result<(), CircuitBreakerError> {
        let state = self.state.read().await;

        match *state {
            CircuitState::Closed => Ok(()),
            CircuitState::Open => {
                // Check if recovery timeout has elapsed
                if let Some(last_failure) = *self.last_failure.read().await {
                    if last_failure.elapsed() > StdDuration::from_secs(self.recovery_timeout.num_seconds() as u64) {
                        drop(state);
                        self.transition_to_half_open().await;
                        Ok(())
                    } else {
                        Err(CircuitBreakerError::CircuitOpen)
                    }
                } else {
                    Err(CircuitBreakerError::CircuitOpen)
                }
            }
            CircuitState::HalfOpen => Ok(()),
        }
    }

    /// Record a successful operation
    pub async fn record_success(&self) {
        let current_successes = self.successes.fetch_add(1, Ordering::SeqCst) + 1;

        let state = self.state.read().await;

        match *state {
            CircuitState::HalfOpen => {
                if current_successes >= self.success_threshold as u64 {
                    drop(state);
                    self.transition_to_closed().await;
                }
            }
            CircuitState::Closed => {
                // Reset success counter periodically in closed state
                if current_successes >= 100 {
                    self.successes.store(0, Ordering::SeqCst);
                }
            }
            CircuitState::Open => {
                // Should not happen, but ignore
            }
        }
    }

    /// Record a failed operation
    pub async fn record_failure(&self) {
        let current_failures = self.failures.fetch_add(1, Ordering::SeqCst) + 1;
        *self.last_failure.write().await = Some(Instant::now());

        // Reset success counter on failure
        self.successes.store(0, Ordering::SeqCst);

        let state = self.state.read().await;

        if matches!(*state, CircuitState::Closed) || matches!(*state, CircuitState::HalfOpen) {
            if current_failures >= self.failure_threshold as u64 {
                drop(state);
                self.transition_to_open().await;
            }
        }
    }

    /// Get current circuit state
    pub async fn state(&self) -> CircuitState {
        (*self.state.read().await).clone()
    }

    /// Get failure count
    pub fn failure_count(&self) -> u64 {
        self.failures.load(Ordering::SeqCst)
    }

    /// Get success count
    pub fn success_count(&self) -> u64 {
        self.successes.load(Ordering::SeqCst)
    }

    /// Reset the circuit breaker to closed state
    pub async fn reset(&self) {
        *self.state.write().await = CircuitState::Closed;
        self.failures.store(0, Ordering::SeqCst);
        self.successes.store(0, Ordering::SeqCst);
        *self.last_failure.write().await = None;
        info!("Circuit breaker reset to closed state");
    }

    async fn transition_to_open(&self) {
        *self.state.write().await = CircuitState::Open;
        warn!("Circuit breaker opened after {} failures", self.failure_threshold);
    }

    async fn transition_to_half_open(&self) {
        *self.state.write().await = CircuitState::HalfOpen;
        self.failures.store(0, Ordering::SeqCst);
        debug!("Circuit breaker transitioning to half-open state for recovery testing");
    }

    async fn transition_to_closed(&self) {
        *self.state.write().await = CircuitState::Closed;
        self.failures.store(0, Ordering::SeqCst);
        self.successes.store(0, Ordering::SeqCst);
        info!("Circuit breaker closed after successful recovery");
    }
}

/// Circuit breaker error types
#[derive(Debug, thiserror::Error)]
pub enum CircuitBreakerError {
    #[error("Circuit breaker is open")]
    CircuitOpen,

    #[error("Circuit breaker operation failed: {0}")]
    OperationFailed(String),
}
