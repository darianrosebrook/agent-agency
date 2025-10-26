//! Circuit breaker pattern implementation for resilient external service calls

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{info, warn, error};

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation - requests are allowed
    Closed,
    /// Service is failing - requests are blocked, checking if service recovered
    Open,
    /// Testing if service has recovered - limited requests allowed
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Service name/identifier
    pub service_name: String,
    /// Failure threshold before opening circuit
    pub failure_threshold: u32,
    /// Success threshold to close circuit from half-open
    pub success_threshold: u32,
    /// Time to wait before trying half-open state
    pub timeout_duration: Duration,
    /// Maximum time a request can take before timing out
    pub request_timeout: Duration,
    /// Number of requests allowed in half-open state
    pub half_open_max_requests: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            service_name: "unknown".to_string(),
            failure_threshold: 5,
            success_threshold: 3,
            timeout_duration: Duration::from_secs(60),
            request_timeout: Duration::from_secs(30),
            half_open_max_requests: 3,
        }
    }
}

/// Circuit breaker instance
#[derive(Debug)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
    half_open_requests: u32,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            half_open_requests: 0,
        }
    }

    /// Check if a request should be allowed
    pub fn allow_request(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => {
                // Normal operation - allow all requests
                true
            }
            CircuitState::Open => {
                // Check if timeout has passed to try half-open
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() >= self.config.timeout_duration {
                        info!("Circuit breaker for {} transitioning to half-open", self.config.service_name);
                        self.state = CircuitState::HalfOpen;
                        self.half_open_requests = 0;
                        self.success_count = 0;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => {
                // Allow limited requests in half-open state
                if self.half_open_requests < self.config.half_open_max_requests {
                    self.half_open_requests += 1;
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Record a successful request
    pub fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.config.success_threshold {
                    info!("Circuit breaker for {} closed - service recovered", self.config.service_name);
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                    self.success_count = 0;
                    self.half_open_requests = 0;
                }
            }
            CircuitState::Open => {
                // Should not happen, but log it
                warn!("Unexpected success recorded for open circuit breaker {}", self.config.service_name);
            }
        }
    }

    /// Record a failed request
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());

        match self.state {
            CircuitState::Closed => {
                if self.failure_count >= self.config.failure_threshold {
                    warn!("Circuit breaker for {} opened due to {} failures", self.config.service_name, self.failure_count);
                    self.state = CircuitState::Open;
                }
            }
            CircuitState::HalfOpen => {
                error!("Circuit breaker for {} failed in half-open state, returning to open", self.config.service_name);
                self.state = CircuitState::Open;
                self.success_count = 0;
                self.half_open_requests = 0;
            }
            CircuitState::Open => {
                // Already open, just update failure time
            }
        }
    }

    /// Get current state
    pub fn get_state(&self) -> CircuitState {
        self.state.clone()
    }

    /// Get statistics
    pub fn get_stats(&self) -> CircuitBreakerStats {
        CircuitBreakerStats {
            state: self.state.clone(),
            failure_count: self.failure_count,
            success_count: self.success_count,
            last_failure_time: self.last_failure_time,
            half_open_requests: self.half_open_requests,
        }
    }
}

/// Circuit breaker statistics
#[derive(Debug, Clone)]
pub struct CircuitBreakerStats {
    pub state: CircuitState,
    pub failure_count: u32,
    pub success_count: u32,
    pub last_failure_time: Option<Instant>,
    pub half_open_requests: u32,
}

/// Circuit breaker registry for managing multiple services
#[derive(Debug)]
pub struct CircuitBreakerRegistry {
    breakers: Arc<Mutex<HashMap<String, CircuitBreaker>>>,
}

impl CircuitBreakerRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self {
            breakers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a circuit breaker for a service
    pub fn register(&self, service_name: &str, config: CircuitBreakerConfig) {
        let mut breakers = self.breakers.lock().unwrap();
        breakers.insert(service_name.to_string(), CircuitBreaker::new(config));
        info!("Registered circuit breaker for service: {}", service_name);
    }

    /// Get a circuit breaker for a service (creates default if not exists)
    pub fn get_or_create(&self, service_name: &str) -> CircuitBreaker {
        let mut breakers = self.breakers.lock().unwrap();

        if let Some(breaker) = breakers.get(service_name) {
            breaker.clone()
        } else {
            // Create with default config
            let config = CircuitBreakerConfig {
                service_name: service_name.to_string(),
                ..Default::default()
            };
            let breaker = CircuitBreaker::new(config.clone());
            breakers.insert(service_name.to_string(), breaker.clone());
            warn!("Created circuit breaker with default config for unregistered service: {}", service_name);
            breaker
        }
    }

    /// Update a circuit breaker's config
    pub fn update_config(&self, service_name: &str, config: CircuitBreakerConfig) -> bool {
        let mut breakers = self.breakers.lock().unwrap();
        if let Some(breaker) = breakers.get_mut(service_name) {
            // Note: In a real implementation, we'd need to update the breaker's config
            // but since CircuitBreaker doesn't expose mutating config, we'd recreate it
            *breaker = CircuitBreaker::new(config);
            true
        } else {
            false
        }
    }

    /// Get statistics for all circuit breakers
    pub fn get_all_stats(&self) -> HashMap<String, CircuitBreakerStats> {
        let breakers = self.breakers.lock().unwrap();
        breakers.iter()
            .map(|(name, breaker)| (name.clone(), breaker.get_stats()))
            .collect()
    }

    /// Execute a function with circuit breaker protection
    pub async fn execute_with_circuit_breaker<F, Fut, T, E>(
        &self,
        service_name: &str,
        operation: F,
    ) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        let mut breaker = self.get_or_create(service_name);

        if !breaker.allow_request() {
            return Err(CircuitBreakerError::CircuitOpen(service_name.to_string()));
        }

        // Execute the operation with timeout
        let timeout_duration = breaker.config.request_timeout;
        let result = tokio::time::timeout(timeout_duration, operation()).await;

        match result {
            Ok(Ok(success)) => {
                breaker.record_success();
                Ok(success)
            }
            Ok(Err(error)) => {
                breaker.record_failure();
                Err(CircuitBreakerError::OperationFailed(error))
            }
            Err(_) => {
                breaker.record_failure();
                Err(CircuitBreakerError::Timeout(timeout_duration))
            }
        }
    }
}

/// Circuit breaker error types
#[derive(Debug, thiserror::Error)]
pub enum CircuitBreakerError<E> {
    #[error("Circuit breaker is open for service: {0}")]
    CircuitOpen(String),
    #[error("Operation failed: {0}")]
    OperationFailed(E),
    #[error("Operation timed out after {0:?}")]
    Timeout(Duration),
}

/// Global circuit breaker registry instance
static CIRCUIT_BREAKER_REGISTRY: once_cell::sync::OnceCell<CircuitBreakerRegistry> =
    once_cell::sync::OnceCell::new();

/// Initialize the global circuit breaker registry
pub fn init_circuit_breaker_registry() -> &'static CircuitBreakerRegistry {
    CIRCUIT_BREAKER_REGISTRY.get_or_init(CircuitBreakerRegistry::new)
}

/// Get the global circuit breaker registry
pub fn get_circuit_breaker_registry() -> &'static CircuitBreakerRegistry {
    CIRCUIT_BREAKER_REGISTRY.get().expect("Circuit breaker registry not initialized")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[test]
    fn test_circuit_breaker_closed_state() {
        let config = CircuitBreakerConfig {
            service_name: "test".to_string(),
            failure_threshold: 3,
            ..Default::default()
        };
        let mut breaker = CircuitBreaker::new(config);

        // Should allow requests in closed state
        assert!(breaker.allow_request());
        assert!(breaker.allow_request());

        // Record some failures
        breaker.record_failure();
        breaker.record_failure();

        // Should still be closed and allow requests
        assert_eq!(breaker.get_state(), CircuitState::Closed);
        assert!(breaker.allow_request());
    }

    #[test]
    fn test_circuit_breaker_open_state() {
        let config = CircuitBreakerConfig {
            service_name: "test".to_string(),
            failure_threshold: 2,
            timeout_duration: Duration::from_millis(100),
            ..Default::default()
        };
        let mut breaker = CircuitBreaker::new(config);

        // Record failures to open circuit
        breaker.record_failure();
        breaker.record_failure();

        assert_eq!(breaker.get_state(), CircuitState::Open);
        assert!(!breaker.allow_request());
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_transition() {
        let config = CircuitBreakerConfig {
            service_name: "test".to_string(),
            failure_threshold: 2,
            timeout_duration: Duration::from_millis(50),
            success_threshold: 2,
            half_open_max_requests: 3,
            ..Default::default()
        };
        let mut breaker = CircuitBreaker::new(config);

        // Open the circuit
        breaker.record_failure();
        breaker.record_failure();
        assert_eq!(breaker.get_state(), CircuitState::Open);

        // Wait for timeout and transition to half-open
        sleep(Duration::from_millis(60)).await;
        assert!(breaker.allow_request()); // Should transition to half-open
        assert_eq!(breaker.get_state(), CircuitState::HalfOpen);

        // Record successes to close circuit
        breaker.record_success();
        breaker.record_success();
        assert_eq!(breaker.get_state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_registry() {
        let registry = CircuitBreakerRegistry::new();

        let config = CircuitBreakerConfig {
            service_name: "api".to_string(),
            failure_threshold: 5,
            ..Default::default()
        };

        registry.register("api", config);
        let breaker = registry.get_or_create("api");

        assert!(breaker.allow_request());

        // Test getting unregistered service (should create default)
        let default_breaker = registry.get_or_create("unknown");
        assert!(default_breaker.allow_request());
    }

    #[tokio::test]
    async fn test_execute_with_circuit_breaker() {
        let registry = CircuitBreakerRegistry::new();

        // Test successful operation
        let result = registry.execute_with_circuit_breaker("test-service", || async {
            Ok::<_, String>("success")
        }).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");

        // Test failed operation
        let result = registry.execute_with_circuit_breaker("test-service", || async {
            Err::<String, _>("operation failed")
        }).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            CircuitBreakerError::OperationFailed(msg) => assert_eq!(msg, "operation failed"),
            _ => panic!("Expected OperationFailed error"),
        }
    }
}
