//! Structured Logging Implementation
//!
//! Provides structured logging capabilities with context propagation,
//! correlation IDs, and performance metrics.
//!
//! Ported from V2 structured logging patterns with Rust optimizations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn, Level};

/// Log level configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<LogLevel> for Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => Level::TRACE,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warn => Level::WARN,
            LogLevel::Error => Level::ERROR,
        }
    }
}

/// Structured log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredLogEntry {
    pub timestamp: SystemTime,
    pub level: LogLevel,
    pub message: String,
    pub correlation_id: Option<String>,
    pub span_id: Option<String>,
    pub trace_id: Option<String>,
    pub service: String,
    pub component: String,
    pub operation: Option<String>,
    pub duration_ms: Option<u64>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub error: Option<ErrorDetails>,
}

/// Error details for structured logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub error_type: String,
    pub error_message: String,
    pub stack_trace: Option<String>,
    pub error_code: Option<String>,
    pub context: HashMap<String, serde_json::Value>,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub service_name: String,
    pub component_name: String,
    pub log_level: LogLevel,
    pub enable_correlation: bool,
    pub enable_performance_logging: bool,
    pub enable_error_logging: bool,
    pub max_metadata_size: usize,
    pub enable_console_output: bool,
    pub enable_file_output: bool,
    pub log_file_path: Option<String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            service_name: "agent-agency".to_string(),
            component_name: "unknown".to_string(),
            log_level: LogLevel::Info,
            enable_correlation: true,
            enable_performance_logging: true,
            enable_error_logging: true,
            max_metadata_size: 1000,
            enable_console_output: true,
            enable_file_output: false,
            log_file_path: None,
        }
    }
}

/// Structured logger
pub struct StructuredLogger {
    config: LoggingConfig,
    correlation_id: Arc<RwLock<Option<String>>>,
    span_id: Arc<RwLock<Option<String>>>,
    trace_id: Arc<RwLock<Option<String>>>,
    metadata: Arc<RwLock<HashMap<String, serde_json::Value>>>,
}

impl StructuredLogger {
    /// Create a new structured logger
    pub fn new(config: LoggingConfig) -> Self {
        Self {
            config,
            correlation_id: Arc::new(RwLock::new(None)),
            span_id: Arc::new(RwLock::new(None)),
            trace_id: Arc::new(RwLock::new(None)),
            metadata: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set correlation ID
    pub async fn set_correlation_id(&self, correlation_id: String) {
        *self.correlation_id.write().await = Some(correlation_id);
    }

    /// Set span ID
    pub async fn set_span_id(&self, span_id: String) {
        *self.span_id.write().await = Some(span_id);
    }

    /// Set trace ID
    pub async fn set_trace_id(&self, trace_id: String) {
        *self.trace_id.write().await = Some(trace_id);
    }

    /// Add metadata
    pub async fn add_metadata(&self, key: String, value: serde_json::Value) {
        let mut metadata = self.metadata.write().await;
        if metadata.len() < self.config.max_metadata_size {
            metadata.insert(key, value);
        }
    }

    /// Clear metadata
    pub async fn clear_metadata(&self) {
        self.metadata.write().await.clear();
    }

    /// Log an info message
    pub async fn info(&self, message: &str, metadata: Option<HashMap<String, serde_json::Value>>) {
        self.log(LogLevel::Info, message, None, None, metadata)
            .await;
    }

    /// Log a warning message
    pub async fn warn(&self, message: &str, metadata: Option<HashMap<String, serde_json::Value>>) {
        self.log(LogLevel::Warn, message, None, None, metadata)
            .await;
    }

    /// Log an error message
    pub async fn error(
        &self,
        message: &str,
        error: Option<ErrorDetails>,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) {
        self.log(LogLevel::Error, message, error, None, metadata)
            .await;
    }

    /// Log a debug message
    pub async fn debug(&self, message: &str, metadata: Option<HashMap<String, serde_json::Value>>) {
        self.log(LogLevel::Debug, message, None, None, metadata)
            .await;
    }

    /// Log a trace message
    pub async fn trace(&self, message: &str, metadata: Option<HashMap<String, serde_json::Value>>) {
        self.log(LogLevel::Trace, message, None, None, metadata)
            .await;
    }

    /// Log performance metrics
    pub async fn log_performance(
        &self,
        operation: &str,
        duration_ms: u64,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) {
        if self.config.enable_performance_logging {
            self.log(
                LogLevel::Info,
                &format!("Operation '{}' completed in {}ms", operation, duration_ms),
                None,
                Some(operation.to_string()),
                metadata,
            )
            .await;
        }
    }

    /// Log with custom level and details
    pub async fn log(
        &self,
        level: LogLevel,
        message: &str,
        error: Option<ErrorDetails>,
        operation: Option<String>,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) {
        let entry = StructuredLogEntry {
            timestamp: SystemTime::now(),
            level: level.clone(),
            message: message.to_string(),
            correlation_id: if self.config.enable_correlation {
                self.correlation_id.read().await.clone()
            } else {
                None
            },
            span_id: if self.config.enable_correlation {
                self.span_id.read().await.clone()
            } else {
                None
            },
            trace_id: if self.config.enable_correlation {
                self.trace_id.read().await.clone()
            } else {
                None
            },
            service: self.config.service_name.clone(),
            component: self.config.component_name.clone(),
            operation,
            duration_ms: None,
            metadata: {
                let mut all_metadata = self.metadata.read().await.clone();
                if let Some(additional_metadata) = metadata {
                    all_metadata.extend(additional_metadata);
                }
                all_metadata
            },
            error,
        };

        // Log using tracing
        match level {
            LogLevel::Trace => debug!("{}", serde_json::to_string(&entry).unwrap_or_default()),
            LogLevel::Debug => debug!("{}", serde_json::to_string(&entry).unwrap_or_default()),
            LogLevel::Info => info!("{}", serde_json::to_string(&entry).unwrap_or_default()),
            LogLevel::Warn => warn!("{}", serde_json::to_string(&entry).unwrap_or_default()),
            LogLevel::Error => error!("{}", serde_json::to_string(&entry).unwrap_or_default()),
        }
    }

    /// Create a new logger with a specific component name
    pub fn with_component(&self, component_name: String) -> StructuredLogger {
        let mut config = self.config.clone();
        config.component_name = component_name;
        StructuredLogger::new(config)
    }

    /// Create a new logger with a specific correlation ID
    pub async fn with_correlation_id(&self, correlation_id: String) -> StructuredLogger {
        let logger = StructuredLogger::new(self.config.clone());
        logger.set_correlation_id(correlation_id).await;
        logger
    }

    /// Get current correlation ID
    pub async fn get_correlation_id(&self) -> Option<String> {
        self.correlation_id.read().await.clone()
    }

    /// Get current span ID
    pub async fn get_span_id(&self) -> Option<String> {
        self.span_id.read().await.clone()
    }

    /// Get current trace ID
    pub async fn get_trace_id(&self) -> Option<String> {
        self.trace_id.read().await.clone()
    }

    /// Get current metadata
    pub async fn get_metadata(&self) -> HashMap<String, serde_json::Value> {
        self.metadata.read().await.clone()
    }
}

/// Performance timer for measuring operation duration
pub struct PerformanceTimer {
    logger: Arc<StructuredLogger>,
    operation: String,
    start_time: SystemTime,
    metadata: Option<HashMap<String, serde_json::Value>>,
}

impl PerformanceTimer {
    /// Create a new performance timer
    pub fn new(
        logger: Arc<StructuredLogger>,
        operation: String,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Self {
        Self {
            logger,
            operation,
            start_time: SystemTime::now(),
            metadata,
        }
    }

    /// Finish timing and log the result
    pub async fn finish(self) {
        let duration = self.start_time.elapsed().unwrap_or_default().as_millis() as u64;
        self.logger
            .log_performance(&self.operation, duration, self.metadata)
            .await;
    }
}

/// Convenience function to create a performance timer
pub fn start_timer(
    logger: Arc<StructuredLogger>,
    operation: String,
    metadata: Option<HashMap<String, serde_json::Value>>,
) -> PerformanceTimer {
    PerformanceTimer::new(logger, operation, metadata)
}

/// Error logging utilities
pub struct ErrorLogger;

impl ErrorLogger {
    /// Create error details from an error
    pub fn create_error_details(error: &dyn std::error::Error) -> ErrorDetails {
        ErrorDetails {
            error_type: std::any::type_name_of_val(error).to_string(),
            error_message: error.to_string(),
            stack_trace: None, // Stack traces are not easily available in Rust
            error_code: None,
            context: HashMap::new(),
        }
    }

    /// Create error details with context
    pub fn create_error_details_with_context(
        error: &dyn std::error::Error,
        context: HashMap<String, serde_json::Value>,
    ) -> ErrorDetails {
        ErrorDetails {
            error_type: std::any::type_name_of_val(error).to_string(),
            error_message: error.to_string(),
            stack_trace: None,
            error_code: None,
            context,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_structured_logger_creation() {
        let config = LoggingConfig::default();
        let logger = StructuredLogger::new(config);

        assert_eq!(logger.get_correlation_id().await, None);
        assert_eq!(logger.get_span_id().await, None);
        assert_eq!(logger.get_trace_id().await, None);
    }

    #[tokio::test]
    async fn test_structured_logger_correlation() {
        let config = LoggingConfig::default();
        let logger = StructuredLogger::new(config);

        let correlation_id = "test-correlation-id".to_string();
        logger.set_correlation_id(correlation_id.clone()).await;

        assert_eq!(logger.get_correlation_id().await, Some(correlation_id));
    }

    #[tokio::test]
    async fn test_structured_logger_metadata() {
        let config = LoggingConfig::default();
        let logger = StructuredLogger::new(config);

        logger
            .add_metadata("key1".to_string(), "value1".into())
            .await;
        logger.add_metadata("key2".to_string(), 42.into()).await;

        let metadata = logger.get_metadata().await;
        assert_eq!(metadata.len(), 2);
        assert_eq!(metadata.get("key1"), Some(&"value1".into()));
        assert_eq!(metadata.get("key2"), Some(&42.into()));
    }

    #[tokio::test]
    async fn test_structured_logger_with_component() {
        let config = LoggingConfig::default();
        let logger = StructuredLogger::new(config);

        let component_logger = logger.with_component("test-component".to_string());
        assert_eq!(component_logger.config.component_name, "test-component");
    }

    #[tokio::test]
    async fn test_structured_logger_with_correlation_id() {
        let config = LoggingConfig::default();
        let logger = StructuredLogger::new(config);

        let correlation_id = "test-correlation-id".to_string();
        let correlated_logger = logger.with_correlation_id(correlation_id.clone()).await;

        assert_eq!(
            correlated_logger.get_correlation_id().await,
            Some(correlation_id)
        );
    }

    #[tokio::test]
    async fn test_performance_timer() {
        let config = LoggingConfig::default();
        let logger = Arc::new(StructuredLogger::new(config));

        let timer = start_timer(logger.clone(), "test-operation".to_string(), None);
        tokio::time::sleep(Duration::from_millis(10)).await;
        timer.finish().await;

        // The timer should have logged the performance
        // In a real test, you might want to capture the log output
    }

    #[tokio::test]
    async fn test_error_logger() {
        let error = std::io::Error::new(std::io::ErrorKind::Other, "test error");
        let error_details = ErrorLogger::create_error_details(&error);

        assert_eq!(error_details.error_message, "test error");
        assert!(!error_details.error_type.is_empty());
    }

    #[tokio::test]
    async fn test_error_logger_with_context() {
        let error = std::io::Error::new(std::io::ErrorKind::Other, "test error");
        let mut context = HashMap::new();
        context.insert("key".to_string(), "value".into());

        let error_details = ErrorLogger::create_error_details_with_context(&error, context);

        assert_eq!(error_details.error_message, "test error");
        assert_eq!(error_details.context.len(), 1);
        assert_eq!(error_details.context.get("key"), Some(&"value".into()));
    }
}
