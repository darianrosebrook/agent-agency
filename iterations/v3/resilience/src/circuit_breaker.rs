//! Circuit Breaker Pattern Implementation
//!
//! Prevents cascading failures by automatically detecting failures
//! and temporarily stopping requests to failing services.
//!
//! States:
//! - CLOSED: Normal operation
//! - OPEN: Failing, reject all requests
//! - HALF_OPEN: Testing if service has recovered
//!
//! Ported from V2 CircuitBreaker.ts with Rust optimizations.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Error thrown when circuit breaker is open
#[derive(Debug, thiserror::Error)]
#[error("Circuit breaker is open: {message}")]
pub struct CircuitBreakerOpenError {
    pub message: String,
    pub circuit_name: Option<String>,
    pub stats: CircuitBreakerStats,
}

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitState {
    /// Normal operation
    Closed,
    /// Failing, reject requests
    Open,
    /// Testing if recovered
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Optional circuit breaker name
    pub name: Option<String>,
    /// Failures before opening
    pub failure_threshold: u64,
    /// Successes before closing from half-open
    pub success_threshold: u64,
    /// Operation timeout (ms)
    pub timeout_ms: Option<u64>,
    /// Time window for failure counting (ms)
    pub failure_window_ms: Option<u64>,
    /// Time to wait before half-open (ms)
    pub reset_timeout_ms: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            name: None,
            failure_threshold: 5,
            success_threshold: 3,
            timeout_ms: Some(30000), // 30 seconds
            failure_window_ms: Some(60000), // 1 minute
            reset_timeout_ms: 60000, // 1 minute
        }
    }
}

/// Circuit breaker statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerStats {
    pub state: CircuitState,
    pub failure_count: u64,
    pub success_count: u64,
    pub total_requests: u64,
    pub last_failure: Option<SystemTime>,
    pub last_success: Option<SystemTime>,
}

/// Circuit breaker for resilience
///
/// Automatically detects failures and stops calling failing operations.
/// Allows for automatic recovery testing after a timeout period.
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: AtomicU8, // 0=Closed, 1=Open, 2=HalfOpen
    failure_count: AtomicU64,
    success_count: AtomicU64,
    total_requests: AtomicU64,
    next_attempt: Arc<RwLock<Instant>>,
    last_failure: Arc<RwLock<Option<SystemTime>>>,
    last_success: Arc<RwLock<Option<SystemTime>>>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            next_attempt: Arc::new(RwLock::new(Instant::now())),
            last_failure: Arc::new(RwLock::new(None)),
            last_success: Arc::new(RwLock::new(None)),
            state: AtomicU8::new(0), // Closed
            failure_count: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            total_requests: AtomicU64::new(0),
            config,
        }
    }

    /// Execute an operation with circuit breaker protection
    ///
    /// # Arguments
    /// * `operation` - The operation to execute
    /// * `fallback` - Optional fallback if circuit is open
    ///
    /// # Returns
    /// Result of operation or fallback
    pub async fn execute<F, T>(
        &self,
        operation: F,
        fallback: Option<Box<dyn Fn() -> Result<T, Box<dyn std::error::Error + Send + Sync>> + Send + Sync>>,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>> + Send>>,
    {
        self.total_requests.fetch_add(1, Ordering::Relaxed);

        // Check if circuit is open
        if self.get_state() == CircuitState::Open {
            let next_attempt = *self.next_attempt.read().await;
            if Instant::now() < next_attempt {
                // Still in timeout period
                if let Some(fallback_fn) = fallback {
                    return fallback_fn();
                }
                return Err(Box::new(CircuitBreakerOpenError {
                    message: format!(
                        "Circuit breaker is OPEN (next attempt in {}ms)",
                        next_attempt.duration_since(Instant::now()).as_millis()
                    ),
                    circuit_name: self.config.name.clone(),
                    stats: self.get_stats().await,
                }) as Box<dyn std::error::Error + Send + Sync>);
            }
            // Try transitioning to half-open
            self.state.store(2, Ordering::Relaxed); // HalfOpen
            self.success_count.store(0, Ordering::Relaxed);
        }

        let result = if let Some(timeout_ms) = self.config.timeout_ms {
            self.execute_with_timeout(operation, timeout_ms).await
        } else {
            operation().await
        };

        match result {
            Ok(value) => {
                self.on_success().await;
                Ok(value)
            }
            Err(error) => {
                self.on_failure().await;
                if let Some(fallback_fn) = fallback {
                    return fallback_fn();
                }
                Err(error)
            }
        }
    }

    /// Execute operation with timeout
    async fn execute_with_timeout<F, T>(
        &self,
        operation: F,
        timeout_ms: u64,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>> + Send>>,
    {
        let timeout_duration = Duration::from_millis(timeout_ms);
        
        match tokio::time::timeout(timeout_duration, operation()).await {
            Ok(result) => result,
            Err(_) => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Operation timeout",
            )) as Box<dyn std::error::Error + Send + Sync>),
        }
    }

    /// Handle successful operation
    async fn on_success(&self) {
        self.failure_count.store(0, Ordering::Relaxed);
        *self.last_success.write().await = Some(SystemTime::now());

        if self.get_state() == CircuitState::HalfOpen {
            let success_count = self.success_count.fetch_add(1, Ordering::Relaxed) + 1;
            if success_count >= self.config.success_threshold {
                // Enough successes, close circuit
                self.state.store(0, Ordering::Relaxed); // Closed
                self.success_count.store(0, Ordering::Relaxed);
                info!(
                    "Circuit breaker '{}' closed after {} successes",
                    self.config.name.as_deref().unwrap_or("unnamed"),
                    success_count
                );
            }
        }
    }

    /// Handle failed operation
    async fn on_failure(&self) {
        let failure_count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
        *self.last_failure.write().await = Some(SystemTime::now());

        if self.get_state() == CircuitState::HalfOpen || failure_count >= self.config.failure_threshold {
            // Open circuit
            self.state.store(1, Ordering::Relaxed); // Open
            let next_attempt = Instant::now() + Duration::from_millis(self.config.reset_timeout_ms);
            *self.next_attempt.write().await = next_attempt;
            self.success_count.store(0, Ordering::Relaxed);
            
            warn!(
                "Circuit breaker '{}' opened after {} failures",
                self.config.name.as_deref().unwrap_or("unnamed"),
                failure_count
            );
        }
    }

    /// Get current circuit state
    pub fn get_state(&self) -> CircuitState {
        match self.state.load(Ordering::Relaxed) {
            0 => CircuitState::Closed,
            1 => CircuitState::Open,
            2 => CircuitState::HalfOpen,
            _ => CircuitState::Closed, // Default fallback
        }
    }

    /// Get circuit breaker statistics
    pub async fn get_stats(&self) -> CircuitBreakerStats {
        CircuitBreakerStats {
            state: self.get_state(),
            failure_count: self.failure_count.load(Ordering::Relaxed),
            success_count: self.success_count.load(Ordering::Relaxed),
            total_requests: self.total_requests.load(Ordering::Relaxed),
            last_failure: *self.last_failure.read().await,
            last_success: *self.last_success.read().await,
        }
    }

    /// Reset circuit breaker to closed state
    pub async fn reset(&self) {
        self.state.store(0, Ordering::Relaxed); // Closed
        self.failure_count.store(0, Ordering::Relaxed);
        self.success_count.store(0, Ordering::Relaxed);
        *self.last_failure.write().await = None;
        *self.last_success.write().await = None;
        
        info!(
            "Circuit breaker '{}' reset to closed state",
            self.config.name.as_deref().unwrap_or("unnamed")
        );
    }

    /// Force circuit open (for testing or manual intervention)
    pub async fn force_open(&self, timeout_ms: Option<u64>) {
        self.state.store(1, Ordering::Relaxed); // Open
        let timeout = timeout_ms.unwrap_or(self.config.reset_timeout_ms);
        let next_attempt = Instant::now() + Duration::from_millis(timeout);
        *self.next_attempt.write().await = next_attempt;
        
        warn!(
            "Circuit breaker '{}' forced open",
            self.config.name.as_deref().unwrap_or("unnamed")
        );
    }

    /// Force circuit closed (for testing or manual intervention)
    pub async fn force_closed(&self) {
        self.state.store(0, Ordering::Relaxed); // Closed
        self.failure_count.store(0, Ordering::Relaxed);
        self.success_count.store(0, Ordering::Relaxed);
        
        info!(
            "Circuit breaker '{}' forced closed",
            self.config.name.as_deref().unwrap_or("unnamed")
        );
    }
}

impl std::fmt::Debug for CircuitBreaker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CircuitBreaker")
            .field("config", &self.config)
            .field("state", &self.get_state())
            .field("failure_count", &self.failure_count.load(Ordering::Relaxed))
            .field("success_count", &self.success_count.load(Ordering::Relaxed))
            .field("total_requests", &self.total_requests.load(Ordering::Relaxed))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_circuit_breaker_creation() {
        let config = CircuitBreakerConfig::default();
        let circuit_breaker = CircuitBreaker::new(config);
        
        assert_eq!(circuit_breaker.get_state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_success() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 1,
            ..Default::default()
        };
        let circuit_breaker = CircuitBreaker::new(config);
        
        let result = circuit_breaker.execute(
            || Box::pin(async { Ok::<i32, Box<dyn std::error::Error + Send + Sync>>(42) }),
            None,
        ).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(circuit_breaker.get_state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_failure() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 1,
            ..Default::default()
        };
        let circuit_breaker = CircuitBreaker::new(config);
        
        // First failure
        let result = circuit_breaker.execute(
            || Box::pin(async { Err::<i32, Box<dyn std::error::Error + Send + Sync>>(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "test error"))) }),
            None,
        ).await;
        
        assert!(result.is_err());
        assert_eq!(circuit_breaker.get_state(), CircuitState::Closed);
        
        // Second failure - should open circuit
        let result = circuit_breaker.execute(
            || Box::pin(async { Err::<i32, Box<dyn std::error::Error + Send + Sync>>(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "test error"))) }),
            None,
        ).await;
        
        assert!(result.is_err());
        assert_eq!(circuit_breaker.get_state(), CircuitState::Open);
    }

    #[tokio::test]
    async fn test_circuit_breaker_fallback() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 1,
            ..Default::default()
        };
        let circuit_breaker = CircuitBreaker::new(config);
        
        // Force circuit open
        circuit_breaker.force_open(None).await;
        
        let fallback = Box::new(|| Ok::<i32, Box<dyn std::error::Error + Send + Sync>>(99));
        let result = circuit_breaker.execute(
            || Box::pin(async { Err::<i32, Box<dyn std::error::Error + Send + Sync>>(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "test error"))) }),
            Some(fallback),
        ).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 99);
    }

    #[tokio::test]
    async fn test_circuit_breaker_reset() {
        let config = CircuitBreakerConfig::default();
        let circuit_breaker = CircuitBreaker::new(config);
        
        // Force circuit open
        circuit_breaker.force_open(None).await;
        assert_eq!(circuit_breaker.get_state(), CircuitState::Open);
        
        // Reset circuit
        circuit_breaker.reset().await;
        assert_eq!(circuit_breaker.get_state(), CircuitState::Closed);
    }
}
