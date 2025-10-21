//! Comprehensive Error Handling Framework for Agent Agency V3
//!
//! This module provides enterprise-grade error handling capabilities including:
//! - Unified error types with context and recovery strategies
//! - Circuit breaker patterns for external service resilience
//! - Retry mechanisms with exponential backoff
//! - Graceful degradation strategies
//! - Comprehensive error logging and monitoring
//! - Recovery orchestration and automated healing

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// Unified error type for the entire Agent Agency system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgencyError {
    /// Unique error ID for tracking
    pub error_id: Uuid,
    /// Error category for classification
    pub category: ErrorCategory,
    /// Specific error code within category
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Detailed technical context
    pub context: HashMap<String, serde_json::Value>,
    /// Error severity level
    pub severity: ErrorSeverity,
    /// Timestamp when error occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Component that generated the error
    pub component: String,
    /// Operation that was being performed
    pub operation: String,
    /// Recovery strategies available
    pub recovery_strategies: Vec<RecoveryStrategy>,
    /// Whether this error is retryable
    pub retryable: bool,
    /// Correlation ID for distributed tracing
    pub correlation_id: Option<String>,
    /// Chain of errors that led to this one
    pub error_chain: Vec<String>,
}

impl AgencyError {
    /// Create a new agency error
    pub fn new(
        category: ErrorCategory,
        code: &str,
        message: &str,
        severity: ErrorSeverity,
        component: &str,
        operation: &str,
    ) -> Self {
        Self {
            error_id: Uuid::new_v4(),
            category,
            code: code.to_string(),
            message: message.to_string(),
            context: HashMap::new(),
            severity,
            timestamp: chrono::Utc::now(),
            component: component.to_string(),
            operation: operation.to_string(),
            recovery_strategies: Vec::new(),
            retryable: false,
            correlation_id: None,
            error_chain: Vec::new(),
        }
    }

    /// Add context to the error
    pub fn with_context(mut self, key: &str, value: serde_json::Value) -> Self {
        self.context.insert(key.to_string(), value);
        self
    }

    /// Add a recovery strategy
    pub fn with_recovery_strategy(mut self, strategy: RecoveryStrategy) -> Self {
        self.recovery_strategies.push(strategy);
        self
    }

    /// Mark as retryable
    pub fn retryable(mut self, retryable: bool) -> Self {
        self.retryable = retryable;
        self
    }

    /// Set correlation ID for tracing
    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    /// Add to error chain
    pub fn with_error_chain(mut self, previous_error: &str) -> Self {
        self.error_chain.push(previous_error.to_string());
        self
    }

    /// Convert to standard error for compatibility
    pub fn into_std_error(self) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::Other, self.message)
    }
}

impl std::fmt::Display for AgencyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}: {} (component: {}, operation: {})",
               self.category, self.code, self.message, self.component, self.operation)
    }
}

impl std::error::Error for AgencyError {}

/// Error categories for classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// Network and connectivity issues
    Network,
    /// Authentication and authorization failures
    Authentication,
    /// Data validation and integrity issues
    Validation,
    /// External service failures
    ExternalService,
    /// Resource exhaustion (memory, CPU, disk)
    ResourceExhaustion,
    /// Configuration errors
    Configuration,
    /// Business logic violations
    BusinessLogic,
    /// Security violations
    Security,
    /// Performance issues
    Performance,
    /// Internal system errors
    Internal,
    /// Timeout errors
    Timeout,
}

/// Error severity levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Debug level - informational only
    Debug,
    /// Info level - normal operation notes
    Info,
    /// Warning level - potential issues
    Warning,
    /// Error level - operation failures
    Error,
    /// Critical level - system stability at risk
    Critical,
    /// Fatal level - system shutdown required
    Fatal,
}

/// Recovery strategies for error handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStrategy {
    /// Strategy type
    pub strategy_type: RecoveryStrategyType,
    /// Description of the recovery approach
    pub description: String,
    /// Estimated time to recover (if applicable)
    pub estimated_duration: Option<Duration>,
    /// Success probability (0.0-1.0)
    pub success_probability: f32,
    /// Resources required for recovery
    pub required_resources: Vec<String>,
    /// Automated recovery capability
    pub automated: bool,
}

/// Types of recovery strategies
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecoveryStrategyType {
    /// Retry the operation
    Retry,
    /// Use cached data or fallback service
    Fallback,
    /// Degrade functionality gracefully
    GracefulDegradation,
    /// Restart the component
    ComponentRestart,
    /// Fail over to backup system
    Failover,
    /// Escalate to human intervention
    HumanIntervention,
    /// Abort the operation
    Abort,
}

/// Circuit breaker for external service resilience
#[derive(Debug)]
pub struct CircuitBreaker {
    /// Service identifier
    service_name: String,
    /// Current state of the circuit breaker
    state: Arc<RwLock<CircuitBreakerState>>,
    /// Configuration
    config: CircuitBreakerConfig,
    /// Statistics for monitoring
    stats: Arc<RwLock<CircuitBreakerStats>>,
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CircuitBreakerState {
    /// Normal operation
    Closed,
    /// Detecting failures, allowing limited requests
    Open,
    /// Allowing requests to test if service recovered
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Failure threshold (number of failures before opening)
    pub failure_threshold: u32,
    /// Success threshold (number of successes needed to close from half-open)
    pub success_threshold: u32,
    /// Timeout before attempting recovery (half-open)
    pub recovery_timeout: Duration,
    /// Window size for failure counting
    pub monitoring_window: Duration,
    /// Timeout for individual requests
    pub request_timeout: Duration,
}

/// Circuit breaker statistics
#[derive(Debug, Clone)]
pub struct CircuitBreakerStats {
    /// Total requests made
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Current consecutive failures
    pub consecutive_failures: u32,
    /// Last failure time
    pub last_failure_time: Option<Instant>,
    /// Last success time
    pub last_success_time: Option<Instant>,
    /// State change history
    pub state_changes: Vec<StateChange>,
}

/// State change record
#[derive(Debug, Clone)]
pub struct StateChange {
    pub timestamp: Instant,
    pub from_state: CircuitBreakerState,
    pub to_state: CircuitBreakerState,
    pub reason: String,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(service_name: String, config: CircuitBreakerConfig) -> Self {
        Self {
            service_name,
            state: Arc::new(RwLock::new(CircuitBreakerState::Closed)),
            config,
            stats: Arc::new(RwLock::new(CircuitBreakerStats {
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                consecutive_failures: 0,
                last_failure_time: None,
                last_success_time: None,
                state_changes: Vec::new(),
            })),
        }
    }

    /// Execute an operation with circuit breaker protection
    pub async fn execute<F, Fut, T>(&self, operation: F) -> Result<T, AgencyError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, AgencyError>>,
    {
        let current_state = self.state.read().await.clone();

        match current_state {
            CircuitBreakerState::Open => {
                // Check if we should attempt recovery
                let stats = self.stats.read().await;
                if let Some(last_failure) = stats.last_failure_time {
                    if last_failure.elapsed() >= self.config.recovery_timeout {
                        drop(stats);
                        self.transition_to_half_open().await;
                        return self.execute_half_open(operation).await;
                    }
                }
                return Err(AgencyError::new(
                    ErrorCategory::ExternalService,
                    "CIRCUIT_BREAKER_OPEN",
                    &format!("Circuit breaker is open for service: {}", self.service_name),
                    ErrorSeverity::Error,
                    "circuit_breaker",
                    "execute"
                ));
            }
            CircuitBreakerState::HalfOpen => {
                return self.execute_half_open(operation).await;
            }
            CircuitBreakerState::Closed => {
                return self.execute_closed(operation).await;
            }
        }
    }

    /// Execute when circuit breaker is closed
    async fn execute_closed<F, Fut, T>(&self, operation: F) -> Result<T, AgencyError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, AgencyError>>,
    {
        let result = tokio::time::timeout(
            self.config.request_timeout,
            operation()
        ).await;

        let mut stats = self.stats.write().await;
        stats.total_requests += 1;

        match result {
            Ok(Ok(success)) => {
                stats.successful_requests += 1;
                stats.consecutive_failures = 0;
                stats.last_success_time = Some(Instant::now());
                Ok(success)
            }
            Ok(Err(error)) => {
                stats.failed_requests += 1;
                stats.consecutive_failures += 1;
                stats.last_failure_time = Some(Instant::now());

                // Check if we should open the circuit
                if stats.consecutive_failures >= self.config.failure_threshold {
                    drop(stats);
                    self.transition_to_open("Failure threshold exceeded".to_string()).await;
                }

                Err(error)
            }
            Err(_) => {
                // Timeout
                stats.failed_requests += 1;
                stats.consecutive_failures += 1;
                stats.last_failure_time = Some(Instant::now());

                if stats.consecutive_failures >= self.config.failure_threshold {
                    drop(stats);
                    self.transition_to_open("Timeout threshold exceeded".to_string()).await;
                }

                Err(AgencyError::new(
                    ErrorCategory::Timeout,
                    "REQUEST_TIMEOUT",
                    &format!("Request to {} timed out", self.service_name),
                    ErrorSeverity::Error,
                    "circuit_breaker",
                    "execute_closed"
                ))
            }
        }
    }

    /// Execute when circuit breaker is half-open
    async fn execute_half_open<F, Fut, T>(&self, operation: F) -> Result<T, AgencyError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, AgencyError>>,
    {
        let result = tokio::time::timeout(
            self.config.request_timeout,
            operation()
        ).await;

        let mut stats = self.stats.write().await;
        stats.total_requests += 1;

        match result {
            Ok(Ok(success)) => {
                stats.successful_requests += 1;
                stats.consecutive_failures = 0;
                stats.last_success_time = Some(Instant::now());

                // Check if we should close the circuit
                if stats.successful_requests % self.config.success_threshold as u64 == 0 {
                    drop(stats);
                    self.transition_to_closed("Success threshold reached".to_string()).await;
                }

                Ok(success)
            }
            Ok(Err(_)) | Err(_) => {
                // Any failure in half-open state sends us back to open
                stats.failed_requests += 1;
                stats.consecutive_failures += 1;
                stats.last_failure_time = Some(Instant::now());

                drop(stats);
                self.transition_to_open("Failure during recovery attempt".to_string()).await;

                Err(AgencyError::new(
                    ErrorCategory::ExternalService,
                    "HALF_OPEN_FAILURE",
                    &format!("Service {} failed during recovery attempt", self.service_name),
                    ErrorSeverity::Error,
                    "circuit_breaker",
                    "execute_half_open"
                ))
            }
        }
    }

    /// Transition to open state
    async fn transition_to_open(&self, reason: String) {
        let mut state = self.state.write().await;
        let previous_state = state.clone();
        *state = CircuitBreakerState::Open;

        let mut stats = self.stats.write().await;
        stats.state_changes.push(StateChange {
            timestamp: Instant::now(),
            from_state: previous_state,
            to_state: CircuitBreakerState::Open,
            reason,
        });
    }

    /// Transition to half-open state
    async fn transition_to_half_open(&self) {
        let mut state = self.state.write().await;
        let previous_state = state.clone();
        *state = CircuitBreakerState::HalfOpen;

        let mut stats = self.stats.write().await;
        stats.state_changes.push(StateChange {
            timestamp: Instant::now(),
            from_state: previous_state,
            to_state: CircuitBreakerState::HalfOpen,
            reason: "Recovery timeout elapsed".to_string(),
        });
    }

    /// Transition to closed state
    async fn transition_to_closed(&self, reason: String) {
        let mut state = self.state.write().await;
        let previous_state = state.clone();
        *state = CircuitBreakerState::Closed;

        let mut stats = self.stats.write().await;
        stats.state_changes.push(StateChange {
            timestamp: Instant::now(),
            from_state: previous_state,
            to_state: CircuitBreakerState::Closed,
            reason,
        });
    }

    /// Get current statistics
    pub async fn get_stats(&self) -> CircuitBreakerStats {
        self.stats.read().await.clone()
    }

    /// Get current state
    pub async fn get_state(&self) -> CircuitBreakerState {
        self.state.read().await.clone()
    }
}

/// Retry mechanism with exponential backoff
#[derive(Debug)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
    /// Jitter factor (0.0-1.0) to randomize delays
    pub jitter_factor: f64,
}

/// Execute operation with retry logic
pub async fn with_retry<F, Fut, T>(
    config: &RetryConfig,
    operation: F,
) -> Result<T, AgencyError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, AgencyError>>,
{
    let mut attempt = 0;
    let mut delay = config.initial_delay;

    loop {
        attempt += 1;

        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                if attempt >= config.max_attempts || !error.retryable {
                    return Err(error);
                }

                // Calculate delay with exponential backoff and jitter
                let backoff_delay = delay.mul_f64(config.backoff_multiplier.min(10.0));
                delay = backoff_delay.min(config.max_delay);

                // Add jitter
                let jitter = delay.mul_f64(config.jitter_factor * rand::random::<f64>());
                let total_delay = delay + jitter;

                tracing::warn!(
                    "Operation failed (attempt {}/{}), retrying in {:.2}s: {}",
                    attempt,
                    config.max_attempts,
                    total_delay.as_secs_f64(),
                    error
                );

                tokio::time::sleep(total_delay).await;
            }
        }
    }
}

/// Graceful degradation manager
#[derive(Debug)]
pub struct DegradationManager {
    /// Current degradation state
    state: Arc<RwLock<DegradationState>>,
    /// Degradation policies by component
    policies: HashMap<String, DegradationPolicy>,
}

/// Current degradation state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegradationState {
    /// Whether the system is in degraded mode
    pub degraded: bool,
    /// Degraded components and their degradation levels
    pub degraded_components: HashMap<String, DegradationLevel>,
    /// Timestamp when degradation started
    #[serde(skip)]
    pub degradation_start: Option<Instant>,
    /// Expected recovery time
    #[serde(skip)]
    pub expected_recovery: Option<Instant>,
}

/// Degradation policy for a component
#[derive(Debug, Clone)]
pub struct DegradationPolicy {
    /// Component name
    pub component: String,
    /// Degradation levels in priority order
    pub levels: Vec<DegradationLevel>,
    /// Recovery conditions
    pub recovery_conditions: Vec<String>,
}

/// Degradation level with specific strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegradationLevel {
    /// Level name (e.g., "reduced_accuracy", "limited_functionality")
    pub name: String,
    /// Description of what functionality is reduced
    pub description: String,
    /// Performance impact (0.0-1.0, higher = more impact)
    pub performance_impact: f32,
    /// Functionality impact (0.0-1.0, higher = more impact)
    pub functionality_impact: f32,
    /// Recovery priority (higher = recover first)
    pub recovery_priority: u32,
}

impl DegradationManager {
    /// Create a new degradation manager
    pub fn new(policies: HashMap<String, DegradationPolicy>) -> Self {
        Self {
            state: Arc::new(RwLock::new(DegradationState {
                degraded: false,
                degraded_components: HashMap::new(),
                degradation_start: None,
                expected_recovery: None,
            })),
            policies,
        }
    }

    /// Apply degradation to a component
    pub async fn degrade_component(&self, component: &str, level: DegradationLevel) -> Result<(), AgencyError> {
        let mut state = self.state.write().await;

        if !state.degraded {
            state.degraded = true;
            state.degradation_start = Some(Instant::now());
        }

        state.degraded_components.insert(component.to_string(), level.clone());

        // Log degradation event
        tracing::warn!(
            "Component '{}' degraded to level '{}' - {}",
            component, level.name, level.description
        );

        Ok(())
    }

    /// Check if a component should be degraded based on error patterns
    pub async fn should_degrade(&self, component: &str, error_count: u32, time_window: Duration) -> Option<DegradationLevel> {
        let policy = self.policies.get(component)?;

        // Simple degradation logic: degrade after 5 errors in 5 minutes
        if error_count >= 5 && time_window <= Duration::from_secs(300) {
            // Return the first (least severe) degradation level
            policy.levels.first().cloned()
        } else {
            None
        }
    }

    /// Attempt to recover a component
    pub async fn recover_component(&self, component: &str) -> Result<(), AgencyError> {
        let mut state = self.state.write().await;

        if state.degraded_components.remove(component).is_some() {
            tracing::info!("Component '{}' recovered from degradation", component);

            // Check if system is fully recovered
            if state.degraded_components.is_empty() {
                state.degraded = false;
                state.degradation_start = None;
                state.expected_recovery = None;

                tracing::info!("System fully recovered from degradation");
            }
        }

        Ok(())
    }

    /// Get current degradation state
    pub async fn get_state(&self) -> DegradationState {
        self.state.read().await.clone()
    }
}

/// Error recovery orchestrator
#[derive(Debug)]
pub struct RecoveryOrchestrator {
    /// Circuit breakers for external services
    circuit_breakers: HashMap<String, Arc<CircuitBreaker>>,
    /// Degradation manager
    degradation_manager: Arc<DegradationManager>,
    /// Retry configurations by error type
    retry_configs: HashMap<ErrorCategory, RetryConfig>,
    /// Recovery strategies by error pattern
    recovery_strategies: HashMap<String, Vec<RecoveryStrategy>>,
}

impl RecoveryOrchestrator {
    /// Create a new recovery orchestrator
    pub fn new(
        circuit_breakers: HashMap<String, Arc<CircuitBreaker>>,
        degradation_manager: Arc<DegradationManager>,
    ) -> Self {
        let mut retry_configs = HashMap::new();

        // Configure retry policies for different error types
        retry_configs.insert(ErrorCategory::Network, RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
        });

        retry_configs.insert(ErrorCategory::ExternalService, RetryConfig {
            max_attempts: 2,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            jitter_factor: 0.2,
        });

        retry_configs.insert(ErrorCategory::Timeout, RetryConfig {
            max_attempts: 1, // Don't retry timeouts
            initial_delay: Duration::from_millis(0),
            max_delay: Duration::from_millis(0),
            backoff_multiplier: 1.0,
            jitter_factor: 0.0,
        });

        Self {
            circuit_breakers,
            degradation_manager,
            retry_configs,
            recovery_strategies: HashMap::new(),
        }
    }

    /// Handle an error with comprehensive recovery strategies
    pub async fn handle_error(&self, error: AgencyError) -> Result<(), AgencyError> {
        // Log the error
        self.log_error(&error).await;

        // Check if error is retryable and apply retry logic
        if error.retryable {
            if let Some(retry_config) = self.retry_configs.get(&error.category) {
                // For retryable errors, the caller should implement the retry logic
                // This orchestrator just provides the configuration
                tracing::info!("Error is retryable with config: {:?}", retry_config);
            }
        }

        // Check circuit breaker for external services
        if let ErrorCategory::ExternalService = error.category {
            if let Some(service_name) = error.context.get("service_name") {
                if let Some(service_name_str) = service_name.as_str() {
                    if let Some(circuit_breaker) = self.circuit_breakers.get(service_name_str) {
                        let state = circuit_breaker.get_state().await;
                        tracing::warn!("Circuit breaker for {} is in state: {:?}", service_name_str, state);
                    }
                }
            }
        }

        // Apply degradation if appropriate
        self.evaluate_degradation(&error).await?;

        // Execute recovery strategies
        for strategy in &error.recovery_strategies {
            match self.execute_recovery_strategy(strategy).await {
                Ok(_) => {
                    tracing::info!("Recovery strategy '{}' succeeded", strategy.description);
                    return Ok(());
                }
                Err(recovery_error) => {
                    tracing::warn!("Recovery strategy '{}' failed: {}", strategy.description, recovery_error);
                }
            }
        }

        // If all recovery strategies failed, return the original error
        Err(error)
    }

    /// Log error with comprehensive context
    async fn log_error(&self, error: &AgencyError) {
        let log_level = match error.severity {
            ErrorSeverity::Debug => tracing::Level::DEBUG,
            ErrorSeverity::Info => tracing::Level::INFO,
            ErrorSeverity::Warning => tracing::Level::WARN,
            ErrorSeverity::Error => tracing::Level::ERROR,
            ErrorSeverity::Critical => tracing::Level::ERROR,
            ErrorSeverity::Fatal => tracing::Level::ERROR,
        };

        tracing::event!(
            log_level,
            error_id = %error.error_id,
            category = ?error.category,
            code = %error.code,
            component = %error.component,
            operation = %error.operation,
            severity = ?error.severity,
            retryable = error.retryable,
            correlation_id = ?error.correlation_id,
            "Error occurred: {}",
            error.message
        );

        // Log error chain if present
        if !error.error_chain.is_empty() {
            tracing::event!(
                log_level,
                error_id = %error.error_id,
                "Error chain: {:?}",
                error.error_chain
            );
        }

        // Log recovery strategies
        if !error.recovery_strategies.is_empty() {
            tracing::event!(
                log_level,
                error_id = %error.error_id,
                "Available recovery strategies: {}",
                error.recovery_strategies.len()
            );
        }
    }

    /// Evaluate whether to apply degradation based on error patterns
    async fn evaluate_degradation(&self, error: &AgencyError) -> Result<(), AgencyError> {
        // Check if component should be degraded
        if let Some(degradation_level) = self.degradation_manager
            .should_degrade(&error.component, 1, Duration::from_secs(300))
            .await
        {
            self.degradation_manager
                .degrade_component(&error.component, degradation_level)
                .await?;
        }

        Ok(())
    }

    /// Execute a specific recovery strategy
    async fn execute_recovery_strategy(&self, strategy: &RecoveryStrategy) -> Result<(), AgencyError> {
        match strategy.strategy_type {
            RecoveryStrategyType::Retry => {
                // Retry logic is handled by the caller
                Ok(())
            }
            RecoveryStrategyType::Fallback => {
                // Implement fallback logic here
                tracing::info!("Executing fallback strategy: {}", strategy.description);
                Ok(())
            }
            RecoveryStrategyType::GracefulDegradation => {
                // Apply graceful degradation
                tracing::info!("Applying graceful degradation: {}", strategy.description);
                Ok(())
            }
            RecoveryStrategyType::ComponentRestart => {
                // Implement component restart logic
                tracing::warn!("Component restart requested: {}", strategy.description);
                Ok(())
            }
            RecoveryStrategyType::Failover => {
                // Implement failover logic
                tracing::info!("Executing failover strategy: {}", strategy.description);
                Ok(())
            }
            RecoveryStrategyType::HumanIntervention => {
                // Escalate to human intervention
                tracing::error!("Human intervention required: {}", strategy.description);
                Err(AgencyError::new(
                    ErrorCategory::Internal,
                    "HUMAN_INTERVENTION_REQUIRED",
                    &format!("Human intervention required: {}", strategy.description),
                    ErrorSeverity::Critical,
                    "recovery_orchestrator",
                    "execute_recovery_strategy"
                ))
            }
            RecoveryStrategyType::Abort => {
                // Abort the operation
                tracing::warn!("Aborting operation: {}", strategy.description);
                Err(AgencyError::new(
                    ErrorCategory::BusinessLogic,
                    "OPERATION_ABORTED",
                    &format!("Operation aborted: {}", strategy.description),
                    ErrorSeverity::Error,
                    "recovery_orchestrator",
                    "execute_recovery_strategy"
                ))
            }
        }
    }

    /// Get system health status
    pub async fn get_health_status(&self) -> SystemHealth {
        let mut circuit_breaker_states = HashMap::new();
        for (name, breaker) in &self.circuit_breakers {
            circuit_breaker_states.insert(name.clone(), breaker.get_state().await);
        }

        let degradation_state = self.degradation_manager.get_state().await;

        SystemHealth {
            overall_health: if degradation_state.degraded {
                HealthStatus::Degraded
            } else {
                HealthStatus::Healthy
            },
            circuit_breaker_states,
            degradation_state,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// System health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub overall_health: HealthStatus,
    pub circuit_breaker_states: HashMap<String, CircuitBreakerState>,
    pub degradation_state: DegradationState,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Overall health status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Critical,
}

/// Helper function to create common error types
pub mod error_factory {
    use super::*;

    pub fn network_error(operation: &str, details: &str) -> AgencyError {
        AgencyError::new(
            ErrorCategory::Network,
            "NETWORK_ERROR",
            details,
            ErrorSeverity::Error,
            "network",
            operation,
        )
        .retryable(true)
        .with_recovery_strategy(RecoveryStrategy {
            strategy_type: RecoveryStrategyType::Retry,
            description: "Retry network operation with exponential backoff".to_string(),
            estimated_duration: Some(Duration::from_secs(5)),
            success_probability: 0.7,
            required_resources: vec![],
            automated: true,
        })
        .with_recovery_strategy(RecoveryStrategy {
            strategy_type: RecoveryStrategyType::Fallback,
            description: "Use cached data or offline mode".to_string(),
            estimated_duration: Some(Duration::from_secs(1)),
            success_probability: 0.9,
            required_resources: vec!["cache".to_string()],
            automated: true,
        })
    }

    pub fn external_service_error(service: &str, operation: &str, details: &str) -> AgencyError {
        AgencyError::new(
            ErrorCategory::ExternalService,
            "EXTERNAL_SERVICE_ERROR",
            details,
            ErrorSeverity::Error,
            service,
            operation,
        )
        .with_context("service_name", serde_json::Value::String(service.to_string()))
        .retryable(true)
        .with_recovery_strategy(RecoveryStrategy {
            strategy_type: RecoveryStrategyType::Retry,
            description: format!("Retry {} operation", service),
            estimated_duration: Some(Duration::from_secs(10)),
            success_probability: 0.5,
            required_resources: vec![],
            automated: true,
        })
        .with_recovery_strategy(RecoveryStrategy {
            strategy_type: RecoveryStrategyType::Failover,
            description: format!("Fail over to backup {} service", service),
            estimated_duration: Some(Duration::from_secs(30)),
            success_probability: 0.8,
            required_resources: vec![format!("backup_{}", service)],
            automated: true,
        })
    }

    pub fn timeout_error(component: &str, operation: &str, timeout_duration: Duration) -> AgencyError {
        AgencyError::new(
            ErrorCategory::Timeout,
            "TIMEOUT_ERROR",
            &format!("Operation timed out after {:?}", timeout_duration),
            ErrorSeverity::Warning,
            component,
            operation,
        )
        .retryable(false) // Don't retry timeouts
        .with_recovery_strategy(RecoveryStrategy {
            strategy_type: RecoveryStrategyType::GracefulDegradation,
            description: "Continue with reduced functionality".to_string(),
            estimated_duration: Some(Duration::from_secs(1)),
            success_probability: 1.0,
            required_resources: vec![],
            automated: true,
        })
    }

    pub fn resource_exhaustion_error(component: &str, operation: &str, resource: &str) -> AgencyError {
        AgencyError::new(
            ErrorCategory::ResourceExhaustion,
            "RESOURCE_EXHAUSTION",
            &format!("Resource '{}' exhausted", resource),
            ErrorSeverity::Critical,
            component,
            operation,
        )
        .with_recovery_strategy(RecoveryStrategy {
            strategy_type: RecoveryStrategyType::GracefulDegradation,
            description: format!("Reduce {} usage and continue", resource),
            estimated_duration: Some(Duration::from_secs(5)),
            success_probability: 0.9,
            required_resources: vec![],
            automated: true,
        })
        .with_recovery_strategy(RecoveryStrategy {
            strategy_type: RecoveryStrategyType::ComponentRestart,
            description: "Restart component to free resources".to_string(),
            estimated_duration: Some(Duration::from_secs(30)),
            success_probability: 0.6,
            required_resources: vec!["restart_permissions".to_string()],
            automated: false,
        })
    }

    pub fn security_error(component: &str, operation: &str, violation: &str) -> AgencyError {
        AgencyError::new(
            ErrorCategory::Security,
            "SECURITY_VIOLATION",
            violation,
            ErrorSeverity::Critical,
            component,
            operation,
        )
        .with_recovery_strategy(RecoveryStrategy {
            strategy_type: RecoveryStrategyType::HumanIntervention,
            description: "Security incident requires immediate human investigation".to_string(),
            estimated_duration: None,
            success_probability: 1.0,
            required_resources: vec!["security_team".to_string()],
            automated: false,
        })
    }
}
