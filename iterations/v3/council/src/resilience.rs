//! Production Resilience Patterns (V2 Integration)
//!
//! Implements battle-tested resilience patterns from V2:
//! - Circuit breakers for external API calls
//! - Retry logic with exponential backoff
//! - Health checks and monitoring
//! - Structured logging for production observability

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tokio::time::{sleep, timeout};
use tracing::{debug, error, info, warn};

/// Circuit breaker states (V2 pattern)
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    Closed,   // Normal operation
    Open,     // Failing, reject calls
    HalfOpen, // Testing recovery
}

/// Circuit breaker configuration (V2-style)
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,        // Failures before opening
    pub recovery_timeout_ms: u64,      // Time before trying recovery
    pub success_threshold: u32,        // Successes needed to close
    pub monitoring_window_ms: u64,     // Window for failure tracking
}

/// Circuit breaker implementation (V2 pattern)
#[derive(Debug)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitState>>,
    failures: Arc<Mutex<Vec<Instant>>>,
    successes: Arc<Mutex<u32>>,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failures: Arc::new(Mutex::new(Vec::new())),
            successes: Arc::new(Mutex::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
        }
    }

    /// Execute an operation with circuit breaker protection (V2 pattern)
    pub async fn execute<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let state = self.state.read().await.clone();

        match state {
            CircuitState::Open => {
                // Check if we should try recovery
                if let Some(last_failure) = *self.last_failure_time.read().await {
                    if last_failure.elapsed() > Duration::from_millis(self.config.recovery_timeout_ms) {
                        *self.state.write().await = CircuitState::HalfOpen;
                        info!("Circuit breaker transitioning to HalfOpen for recovery attempt");
                    } else {
                        return Err(anyhow::anyhow!("Circuit breaker is OPEN - rejecting call"));
                    }
                }
            }
            CircuitState::HalfOpen => {
                debug!("Circuit breaker in HalfOpen state - allowing test call");
            }
            CircuitState::Closed => {
                debug!("Circuit breaker closed - normal operation");
            }
        }

        // Execute the operation
        match timeout(Duration::from_secs(30), operation()).await {
            Ok(result) => match result {
                Ok(value) => {
                    self.record_success().await;
                    Ok(value)
                }
                Err(e) => {
                    self.record_failure().await;
                    Err(e)
                }
            },
            Err(_) => {
                self.record_failure().await;
                Err(anyhow::anyhow!("Operation timed out"))
            }
        }
    }

    /// Record a successful operation (V2 pattern)
    async fn record_success(&self) {
        let mut successes = self.successes.lock().await;
        *successes += 1;

        let state = self.state.read().await.clone();
        if state == CircuitState::HalfOpen && *successes >= self.config.success_threshold {
            *self.state.write().await = CircuitState::Closed;
            *successes = 0;
            info!("Circuit breaker CLOSED after {} successes", self.config.success_threshold);
        }
    }

    /// Record a failed operation (V2 pattern)
    async fn record_failure(&self) {
        let mut failures = self.failures.lock().await;
        let now = Instant::now();

        // Clean old failures outside monitoring window
        let window_start = now - Duration::from_millis(self.config.monitoring_window_ms);
        failures.retain(|&time| time > window_start);

        failures.push(now);
        *self.last_failure_time.write().await = Some(now);

        if failures.len() >= self.config.failure_threshold as usize {
            *self.state.write().await = CircuitState::Open;
            warn!("Circuit breaker OPENED after {} failures in window", failures.len());
        }
    }

    /// Get current circuit breaker status (V2 pattern)
    pub async fn status(&self) -> CircuitBreakerStatus {
        let state = self.state.read().await.clone();
        let failures = self.failures.lock().await.len();
        let successes = *self.successes.lock().await;
        let last_failure = *self.last_failure_time.read().await;

        CircuitBreakerStatus {
            state,
            failure_count: failures,
            success_count: successes,
            last_failure_time: last_failure,
        }
    }
}

/// Circuit breaker status for monitoring
#[derive(Debug, Clone)]
pub struct CircuitBreakerStatus {
    pub state: CircuitState,
    pub failure_count: usize,
    pub success_count: u32,
    pub last_failure_time: Option<Instant>,
}

/// Retry configuration with exponential backoff (V2 pattern)
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
    pub jitter_factor: f64,
}

/// Retry executor with exponential backoff (V2 pattern)
#[derive(Debug)]
pub struct RetryExecutor {
    config: RetryConfig,
}

impl RetryExecutor {
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Execute operation with retry logic (V2 pattern)
    pub async fn execute<F, Fut, T>(&self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut attempt = 0;
        let mut delay = self.config.initial_delay_ms;

        loop {
            attempt += 1;

            match operation().await {
                Ok(result) => {
                    if attempt > 1 {
                        info!("Operation succeeded on attempt {}", attempt);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    if attempt >= self.config.max_attempts {
                        error!("Operation failed after {} attempts: {}", attempt, e);
                        return Err(e);
                    }

                    // Calculate delay with exponential backoff and jitter
                    let backoff_delay = delay as f64 * self.config.backoff_multiplier;
                    delay = (backoff_delay as u64).min(self.config.max_delay_ms);

                    // Add jitter to prevent thundering herd
                    let jitter = (delay as f64 * self.config.jitter_factor * rand::random::<f64>()) as u64;
                    let total_delay = delay + jitter;

                    warn!(
                        "Operation failed (attempt {}/{}): {}. Retrying in {}ms",
                        attempt,
                        self.config.max_attempts,
                        e,
                        total_delay
                    );

                    sleep(Duration::from_millis(total_delay)).await;
                }
            }
        }
    }
}

/// Health check system (V2 pattern)
#[derive(Debug)]
pub struct HealthChecker {
    checks: Arc<RwLock<HashMap<String, Box<dyn HealthCheck>>>>,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            checks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a health check (V2 pattern)
    pub async fn register_check(&self, name: String, check: Box<dyn HealthCheck>) {
        self.checks.write().await.insert(name, check);
    }

    /// Run all health checks (V2 pattern)
    pub async fn check_health(&self) -> HealthStatus {
        let checks = self.checks.read().await;
        let mut results = Vec::new();
        let mut overall_healthy = true;

        for (name, check) in checks.iter() {
            let start = Instant::now();
            let result = check.check_health().await;
            let duration = start.elapsed();

            let healthy = matches!(result, HealthCheckResult::Healthy);
            overall_healthy &= healthy;

            results.push(HealthCheckReport {
                name: name.clone(),
                status: result,
                duration_ms: duration.as_millis() as u64,
                timestamp: chrono::Utc::now(),
            });
        }

        HealthStatus {
            overall_healthy,
            checks: results,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Health check trait (V2 pattern)
#[async_trait::async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check_health(&self) -> HealthCheckResult;
}

/// Health check result types (V2 pattern)
#[derive(Debug, Clone)]
pub enum HealthCheckResult {
    Healthy,
    Degraded(String),
    Unhealthy(String),
}

/// Comprehensive health status (V2 pattern)
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub overall_healthy: bool,
    pub checks: Vec<HealthCheckReport>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Individual health check report (V2 pattern)
#[derive(Debug, Clone)]
pub struct HealthCheckReport {
    pub name: String,
    pub status: HealthCheckResult,
    pub duration_ms: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Resilience manager that combines all patterns (V2 integration)
#[derive(Debug)]
pub struct ResilienceManager {
    circuit_breakers: Arc<RwLock<HashMap<String, Arc<CircuitBreaker>>>>,
    retry_executor: RetryExecutor,
    health_checker: HealthChecker,
}

impl ResilienceManager {
    /// Create new resilience manager with V2 defaults
    pub fn new() -> Self {
        let retry_config = RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
        };

        Self {
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
            retry_executor: RetryExecutor::new(retry_config),
            health_checker: HealthChecker::new(),
        }
    }

    /// Get or create a circuit breaker for a service (V2 pattern)
    pub async fn get_circuit_breaker(&self, service_name: &str) -> Arc<CircuitBreaker> {
        let breakers = self.circuit_breakers.read().await;
        if let Some(breaker) = breakers.get(service_name) {
            return breaker.clone();
        }
        drop(breakers);

        // Create new circuit breaker with V2 defaults
        let config = CircuitBreakerConfig {
            failure_threshold: 5,
            recovery_timeout_ms: 30000, // 30 seconds
            success_threshold: 3,
            monitoring_window_ms: 60000, // 1 minute
        };

        let breaker = Arc::new(CircuitBreaker::new(config));
        self.circuit_breakers.write().await.insert(service_name.to_string(), breaker.clone());
        breaker
    }

    /// Execute operation with full resilience (circuit breaker + retry) (V2 pattern)
    pub async fn execute_resilient<F, Fut, T>(&self, service_name: &str, operation: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let breaker = self.get_circuit_breaker(service_name).await;

        breaker
            .execute(|| self.retry_executor.execute(operation))
            .await
    }

    /// Register a health check (V2 pattern)
    pub async fn register_health_check(&self, name: String, check: Box<dyn HealthCheck>) {
        self.health_checker.register_check(name, check).await;
    }

    /// Get comprehensive health status (V2 pattern)
    pub async fn health_status(&self) -> HealthStatus {
        self.health_checker.check_health().await
    }

    /// Get circuit breaker statuses for monitoring (V2 pattern)
    pub async fn circuit_breaker_statuses(&self) -> HashMap<String, CircuitBreakerStatus> {
        let breakers = self.circuit_breakers.read().await;
        let mut statuses = HashMap::new();

        for (name, breaker) in breakers.iter() {
            statuses.insert(name.clone(), breaker.status().await);
        }

        statuses
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_circuit_breaker_closed_state() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            recovery_timeout_ms: 1000,
            success_threshold: 2,
            monitoring_window_ms: 5000,
        };

        let breaker = CircuitBreaker::new(config);

        // Should start closed
        let status = breaker.status().await;
        assert_eq!(status.state, CircuitState::Closed);

        // Successful operations should keep it closed
        for _ in 0..5 {
            let result = breaker.execute(|| async { Ok(42) }).await;
            assert!(result.is_ok());
        }

        let status = breaker.status().await;
        assert_eq!(status.state, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_on_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            recovery_timeout_ms: 1000,
            success_threshold: 1,
            monitoring_window_ms: 5000,
        };

        let breaker = CircuitBreaker::new(config);

        // Fail operations to open circuit
        for _ in 0..2 {
            let result: Result<i32> = breaker.execute(|| async { Err(anyhow::anyhow!("test error")) }).await;
            assert!(result.is_err());
        }

        let status = breaker.status().await;
        assert_eq!(status.state, CircuitState::Open);
    }

    #[tokio::test]
    async fn test_retry_executor_success() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            backoff_multiplier: 2.0,
            jitter_factor: 0.0,
        };

        let executor = RetryExecutor::new(config);
        let counter = Arc::new(AtomicU32::new(0));

        let result = executor
            .execute(|| {
                let counter = counter.clone();
                async move {
                    let attempts = counter.fetch_add(1, Ordering::SeqCst);
                    if attempts < 2 {
                        Err(anyhow::anyhow!("temporary failure"))
                    } else {
                        Ok(42)
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 3); // Should have tried 3 times
    }
}


