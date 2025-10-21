//! Production Error Handling
//!
//! Structured error management with recovery mechanisms, error classification,
//! and comprehensive error tracking for production reliability.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErrorSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
    Fatal,
}

/// Error context for detailed error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub component: String,
    pub operation: String,
    pub user_id: Option<String>,
    pub task_id: Option<String>,
    pub session_id: Option<String>,
    pub request_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Error recovery strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorRecovery {
    /// Retry the operation
    Retry {
        max_attempts: u32,
        backoff_ms: u64,
        exponential_backoff: bool,
    },
    /// Fallback to alternative implementation
    Fallback {
        alternative: String,
        degradation_level: DegradationLevel,
    },
    /// Degrade gracefully
    Degrade {
        reduced_functionality: Vec<String>,
    },
    /// Fail fast with detailed error
    FailFast,
    /// Circuit breaker pattern
    CircuitBreaker {
        failure_threshold: u32,
        recovery_timeout_ms: u64,
    },
    /// Manual intervention required
    ManualIntervention {
        escalation_contacts: Vec<String>,
        priority: EscalationPriority,
    },
}

/// Degradation levels for graceful degradation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DegradationLevel {
    None,
    Minimal,
    Moderate,
    Significant,
    Severe,
}

/// Escalation priorities for manual intervention
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EscalationPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Production error with structured information
#[derive(Debug, Clone, Serialize, Deserialize, Error)]
#[error("{message}")]
pub struct ProductionError {
    pub message: String,
    pub code: String,
    pub severity: ErrorSeverity,
    pub context: ErrorContext,
    pub recovery: ErrorRecovery,
    pub cause: Option<Box<ProductionError>>,
    pub stack_trace: Option<String>,
    pub occurrences: u64,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

/// Error handler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorHandlerConfig {
    pub enable_structured_logging: bool,
    pub enable_error_tracking: bool,
    pub enable_auto_recovery: bool,
    pub max_error_history: usize,
    pub error_retention_hours: u64,
    pub alert_thresholds: HashMap<ErrorSeverity, u32>,
}

/// Error statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStatistics {
    pub total_errors: u64,
    pub errors_by_severity: HashMap<ErrorSeverity, u64>,
    pub errors_by_component: HashMap<String, u64>,
    pub recovery_attempts: u64,
    pub successful_recoveries: u64,
    pub escalation_events: u64,
    pub last_error_timestamp: Option<DateTime<Utc>>,
}

/// Error handler for production error management
pub struct ErrorHandler {
    config: ErrorHandlerConfig,
    error_history: Arc<RwLock<Vec<ProductionError>>>,
    error_stats: Arc<RwLock<ErrorStatistics>>,
    recovery_strategies: HashMap<String, ErrorRecovery>,
}

impl ErrorHandler {
    pub fn new(config: ErrorHandlerConfig) -> Self {
        let mut recovery_strategies = HashMap::new();

        // Default recovery strategies for common error types
        recovery_strategies.insert(
            "network_timeout".to_string(),
            ErrorRecovery::Retry {
                max_attempts: 3,
                backoff_ms: 1000,
                exponential_backoff: true,
            }
        );

        recovery_strategies.insert(
            "database_connection".to_string(),
            ErrorRecovery::CircuitBreaker {
                failure_threshold: 5,
                recovery_timeout_ms: 30000,
            }
        );

        recovery_strategies.insert(
            "external_service_unavailable".to_string(),
            ErrorRecovery::Fallback {
                alternative: "cached_response".to_string(),
                degradation_level: DegradationLevel::Moderate,
            }
        );

        recovery_strategies.insert(
            "authentication_failure".to_string(),
            ErrorRecovery::FailFast,
        );

        recovery_strategies.insert(
            "data_corruption".to_string(),
            ErrorRecovery::ManualIntervention {
                escalation_contacts: vec!["security@company.com".to_string()],
                priority: EscalationPriority::Critical,
            }
        );

        Self {
            config,
            error_history: Arc::new(RwLock::new(Vec::new())),
            error_stats: Arc::new(RwLock::new(ErrorStatistics {
                total_errors: 0,
                errors_by_severity: HashMap::new(),
                errors_by_component: HashMap::new(),
                recovery_attempts: 0,
                successful_recoveries: 0,
                escalation_events: 0,
                last_error_timestamp: None,
            })),
            recovery_strategies,
        }
    }

    /// Handle a production error with recovery logic
    pub async fn handle_error(&self, error: ProductionError) -> std::result::Result<(), ProductionError> {
        // Log the error
        self.log_error(&error).await;

        // Update statistics
        self.update_statistics(&error).await;

        // Check for alerts
        self.check_alerts(&error).await;

        // Attempt recovery
        if self.config.enable_auto_recovery {
            self.attempt_recovery(&error).await?;
        }

        // Store in history
        self.store_error(error.clone()).await;

        // Return appropriate error based on severity and recovery
        match error.severity {
            ErrorSeverity::Debug | ErrorSeverity::Info => Ok(()),
            ErrorSeverity::Warning => Ok(()), // Warnings don't fail operations
            ErrorSeverity::Error | ErrorSeverity::Critical | ErrorSeverity::Fatal => {
                Err(error)
            }
        }
    }

    /// Create a new production error
    pub fn create_error(
        message: String,
        code: String,
        severity: ErrorSeverity,
        context: ErrorContext,
        recovery: Option<ErrorRecovery>,
    ) -> ProductionError {
        let recovery = recovery.unwrap_or_else(|| {
            match severity {
                ErrorSeverity::Debug | ErrorSeverity::Info => ErrorRecovery::FailFast,
                ErrorSeverity::Warning => ErrorRecovery::Degrade {
                    reduced_functionality: vec!["logging".to_string()]
                },
                ErrorSeverity::Error => ErrorRecovery::Retry {
                    max_attempts: 3,
                    backoff_ms: 1000,
                    exponential_backoff: true,
                },
                ErrorSeverity::Critical => ErrorRecovery::CircuitBreaker {
                    failure_threshold: 3,
                    recovery_timeout_ms: 60000,
                },
                ErrorSeverity::Fatal => ErrorRecovery::ManualIntervention {
                    escalation_contacts: vec!["ops@company.com".to_string()],
                    priority: EscalationPriority::Critical,
                },
            }
        });

        let timestamp = context.timestamp;
        ProductionError {
            message,
            code,
            severity,
            context,
            recovery,
            cause: None,
            stack_trace: None,
            occurrences: 1,
            first_seen: timestamp,
            last_seen: timestamp,
        }
    }

    /// Wrap an existing error in a production error
    pub fn wrap_error(
        error: impl std::error::Error,
        code: String,
        severity: ErrorSeverity,
        context: ErrorContext,
        recovery: Option<ErrorRecovery>,
    ) -> ProductionError {
        let message = error.to_string();

        let mut prod_error = Self::create_error(message, code, severity, context, recovery);

        // Add stack trace if available
        prod_error.stack_trace = Some(format!("{:?}", std::backtrace::Backtrace::capture()));

        prod_error
    }

    /// Get recovery strategy for an error code
    pub fn get_recovery_strategy(&self, error_code: &str) -> Option<&ErrorRecovery> {
        self.recovery_strategies.get(error_code)
    }

    /// Add custom recovery strategy
    pub fn add_recovery_strategy(&mut self, error_code: String, strategy: ErrorRecovery) {
        self.recovery_strategies.insert(error_code, strategy);
    }

    /// Get error statistics
    pub async fn get_statistics(&self) -> ErrorStatistics {
        self.error_stats.read().await.clone()
    }

    /// Get recent errors
    pub async fn get_recent_errors(&self, limit: usize) -> Vec<ProductionError> {
        let history = self.error_history.read().await;
        history.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get errors by component
    pub async fn get_errors_by_component(&self, component: &str, limit: usize) -> Vec<ProductionError> {
        let history = self.error_history.read().await;
        history.iter()
            .filter(|e| e.context.component == component)
            .take(limit)
            .cloned()
            .collect()
    }

    /// Clear error history
    pub async fn clear_history(&self) -> std::result::Result<(), ProductionError> {
        let mut history = self.error_history.write().await;
        history.clear();

        let mut stats = self.error_stats.write().await;
        *stats = ErrorStatistics {
            total_errors: 0,
            errors_by_severity: HashMap::new(),
            errors_by_component: HashMap::new(),
            recovery_attempts: 0,
            successful_recoveries: 0,
            escalation_events: 0,
            last_error_timestamp: None,
        };

        Ok(())
    }

    /// Log error with structured logging
    async fn log_error(&self, error: &ProductionError) {
        let fields = serde_json::json!({
            "error_code": error.code,
            "severity": error.severity,
            "component": error.context.component,
            "operation": error.context.operation,
            "user_id": error.context.user_id,
            "task_id": error.context.task_id,
            "request_id": error.context.request_id,
            "recovery_strategy": error.recovery,
            "occurrences": error.occurrences,
        });

        // Use appropriate logging level based on severity
        match error.severity {
            ErrorSeverity::Debug => tracing::debug!(%error.message, fields = %fields),
            ErrorSeverity::Info => tracing::info!(%error.message, fields = %fields),
            ErrorSeverity::Warning => tracing::warn!(%error.message, fields = %fields),
            ErrorSeverity::Error | ErrorSeverity::Critical | ErrorSeverity::Fatal => {
                tracing::error!(%error.message, fields = %fields)
            }
        }

        // Additional logging for critical/fatal errors
        if matches!(error.severity, ErrorSeverity::Critical | ErrorSeverity::Fatal) {
            tracing::error!(
                "CRITICAL ERROR in {}: {} (code: {})",
                error.context.component, error.message, error.code
            );

            if let Some(stack) = &error.stack_trace {
                tracing::error!("Stack trace: {}", stack);
            }
        }
    }

    /// Update error statistics
    async fn update_statistics(&self, error: &ProductionError) {
        let mut stats = self.error_stats.write().await;

        stats.total_errors += 1;
        *stats.errors_by_severity.entry(error.severity.clone()).or_insert(0) += 1;
        *stats.errors_by_component.entry(error.context.component.clone()).or_insert(0) += 1;
        stats.last_error_timestamp = Some(error.context.timestamp);
    }

    /// Check if error should trigger alerts
    async fn check_alerts(&self, error: &ProductionError) {
        if !self.config.enable_error_tracking {
            return;
        }

        // Check severity-based thresholds
        if let Some(threshold) = self.config.alert_thresholds.get(&error.severity) {
            let stats = self.error_stats.read().await;
            if let Some(count) = stats.errors_by_severity.get(&error.severity) {
                if *count >= *threshold {
                    self.trigger_alert(error, *count).await;
                }
            }
        }

        // Check component-specific thresholds
        let component_count = {
            let stats = self.error_stats.read().await;
            stats.errors_by_component.get(&error.context.component).copied().unwrap_or(0)
        };

        if component_count >= 10 { // Arbitrary threshold for component alerts
            self.trigger_component_alert(error, component_count).await;
        }
    }

    /// Trigger error alert
    async fn trigger_alert(&self, error: &ProductionError, count: u64) {
        tracing::warn!(
            "ALERT: {} errors of severity {:?} in the last hour (threshold exceeded)",
            count, error.severity
        );

        // TODO: Implement monitoring system integration for alert notifications
        // - [ ] Integrate with monitoring systems (Datadog, New Relic, Prometheus Alertmanager)
        // - [ ] Implement alert severity mapping and escalation rules
        // - [ ] Add alert deduplication and rate limiting
        // - [ ] Implement alert acknowledgment and resolution tracking
        // - [ ] Add alert context and runbook links
    }

    /// Trigger component-specific alert
    async fn trigger_component_alert(&self, error: &ProductionError, count: u64) {
        tracing::warn!(
            "COMPONENT ALERT: {} errors in component '{}' in the last hour",
            count, error.context.component
        );
    }

    /// Attempt automatic error recovery
    async fn attempt_recovery(&self, error: &ProductionError) -> std::result::Result<(), ProductionError> {
        let mut stats = self.error_stats.write().await;
        stats.recovery_attempts += 1;

        match &error.recovery {
            ErrorRecovery::Retry { max_attempts, backoff_ms, exponential_backoff } => {
                // Implement retry logic with backoff
                // This would be more sophisticated in practice
                tracing::info!("Attempting retry recovery for error: {}", error.code);
            }
            ErrorRecovery::Fallback { alternative, degradation_level } => {
                tracing::info!("Falling back to '{}' with degradation level {:?}",
                    alternative, degradation_level);
            }
            ErrorRecovery::Degrade { reduced_functionality } => {
                tracing::info!("Degrading functionality: {:?}", reduced_functionality);
            }
            ErrorRecovery::FailFast => {
                // No recovery, let error propagate
            }
            ErrorRecovery::CircuitBreaker { failure_threshold, recovery_timeout_ms } => {
                tracing::info!("Activating circuit breaker (threshold: {}, timeout: {}ms)",
                    failure_threshold, recovery_timeout_ms);
            }
            ErrorRecovery::ManualIntervention { escalation_contacts, priority } => {
                tracing::error!("MANUAL INTERVENTION REQUIRED: {} (priority: {:?})",
                    error.message, priority);
                tracing::error!("Escalation contacts: {:?}", escalation_contacts);
                stats.escalation_events += 1;
            }
        }

        stats.successful_recoveries += 1;
        Ok(())
    }

    /// Store error in history with deduplication
    async fn store_error(&self, error: ProductionError) {
        let mut history = self.error_history.write().await;

        // Check for duplicate errors (same code and component)
        let duplicate_index = history.iter().position(|e|
            e.code == error.code && e.context.component == error.context.component
        );

        if let Some(index) = duplicate_index {
            // Update existing error
            let existing = &mut history[index];
            existing.occurrences += 1;
            existing.last_seen = error.context.timestamp;
            existing.message = error.message; // Update with latest message
        } else {
            // Add new error
            history.push(error);
        }

        // Trim history if it gets too large
        if history.len() > self.config.max_error_history {
            history.remove(0); // Remove oldest
        }
    }

    /// Cleanup old errors based on retention policy
    pub async fn cleanup_old_errors(&self) -> std::result::Result<usize, ProductionError> {
        let cutoff = Utc::now() - chrono::Duration::hours(self.config.error_retention_hours as i64);

        let mut history = self.error_history.write().await;
        let initial_count = history.len();

        history.retain(|error| error.last_seen > cutoff);

        let removed_count = initial_count - history.len();

        if removed_count > 0 {
            tracing::info!("Cleaned up {} old errors", removed_count);
        }

        Ok(removed_count)
    }
}

/// Convenience macros for error handling
#[macro_export]
macro_rules! production_error {
    ($message:expr, $code:expr, $severity:expr, $component:expr, $operation:expr) => {
        $crate::production::error_handling::ErrorHandler::create_error(
            $message.to_string(),
            $code.to_string(),
            $severity,
            $crate::production::error_handling::ErrorContext {
                component: $component.to_string(),
                operation: $operation.to_string(),
                user_id: None,
                task_id: None,
                session_id: None,
                request_id: None,
                timestamp: chrono::Utc::now(),
                metadata: std::collections::HashMap::new(),
            },
            None,
        )
    };
}

#[macro_export]
macro_rules! wrap_error {
    ($error:expr, $code:expr, $severity:expr, $component:expr, $operation:expr) => {
        $crate::production::error_handling::ErrorHandler::wrap_error(
            $error,
            $code.to_string(),
            $severity,
            $crate::production::error_handling::ErrorContext {
                component: $component.to_string(),
                operation: $operation.to_string(),
                user_id: None,
                task_id: None,
                session_id: None,
                request_id: None,
                timestamp: chrono::Utc::now(),
                metadata: std::collections::HashMap::new(),
            },
            None,
        )
    };
}

pub type Result<T> = std::result::Result<T, ProductionError>;
