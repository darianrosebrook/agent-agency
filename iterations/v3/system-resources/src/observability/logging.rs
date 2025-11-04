//! Logging and log management

use schemars::JsonSchema;
use std::collections::{HashMap, VecDeque};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

use super::core::{LogEntry, LogLevel, ObservabilityConfig};

/// Log sink trait for pluggable logging backends
#[async_trait]
pub trait LogSink: Send + Sync {
    /// Write a log entry
    async fn write(&self, entry: &LogEntry) -> Result<(), LoggingError>;

    /// Flush any buffered log entries
    async fn flush(&self) -> Result<(), LoggingError>;
}

/// In-memory log sink for testing and development
pub struct InMemoryLogSink {
    /// Maximum number of entries to keep
    max_entries: usize,
    /// Stored log entries with interior mutability for thread-safe access
    entries: std::sync::Arc<std::sync::Mutex<VecDeque<LogEntry>>>,
}

impl InMemoryLogSink {
    /// Create a new in-memory log sink
    pub fn new(max_entries: usize) -> Self {
        Self {
            max_entries,
            entries: std::sync::Arc::new(std::sync::Mutex::new(VecDeque::with_capacity(max_entries))),
        }
    }

    /// Get all stored entries
    pub fn entries(&self) -> Vec<LogEntry> {
        let entries = self.entries.lock().unwrap();
        entries.iter().cloned().collect()
    }

    /// Get entries by level
    pub fn entries_by_level(&self, level: LogLevel) -> Vec<LogEntry> {
        let entries = self.entries.lock().unwrap();
        entries.iter().filter(|e| e.level == level).cloned().collect()
    }

    /// Get entries by component
    pub fn entries_by_component(&self, component: &str) -> Vec<LogEntry> {
        let entries = self.entries.lock().unwrap();
        entries.iter().filter(|e| e.component == component).cloned().collect()
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        let mut entries = self.entries.lock().unwrap();
        entries.clear();
    }
}

#[async_trait]
impl LogSink for InMemoryLogSink {
    async fn write(&self, entry: &LogEntry) -> Result<(), LoggingError> {
        let mut entries = self.entries.lock().unwrap();

        // Add new entry
        entries.push_back(entry.clone());

        // Implement LRU eviction if over capacity
        while entries.len() > self.max_entries {
            entries.pop_front(); // Remove oldest entries (LRU)
        }

        Ok(())
    }

    async fn flush(&self) -> Result<(), LoggingError> {
        Ok(())
    }
}

/// Logger implementation
pub struct Logger {
    /// Component name for this logger
    component: String,
    /// Minimum log level
    min_level: LogLevel,
    /// Log sinks
    sinks: Vec<Box<dyn LogSink>>,
}

impl Logger {
    /// Create a new logger
    pub fn new(component: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            min_level: LogLevel::Info,
            sinks: Vec::new(),
        }
    }

    /// Create with minimum log level
    pub fn with_min_level(mut self, level: LogLevel) -> Self {
        self.min_level = level;
        self
    }

    /// Add a log sink
    pub fn add_sink(mut self, sink: Box<dyn LogSink>) -> Self {
        self.sinks.push(sink);
        self
    }

    /// Log a debug message
    pub async fn debug(&self, message: impl Into<String>) -> Result<(), LoggingError> {
        self.log(LogLevel::Debug, message).await
    }

    /// Log an info message
    pub async fn info(&self, message: impl Into<String>) -> Result<(), LoggingError> {
        self.log(LogLevel::Info, message).await
    }

    /// Log a warning message
    pub async fn warn(&self, message: impl Into<String>) -> Result<(), LoggingError> {
        self.log(LogLevel::Warn, message).await
    }

    /// Log an error message
    pub async fn error(&self, message: impl Into<String>) -> Result<(), LoggingError> {
        self.log(LogLevel::Error, message).await
    }

    /// Log a critical message
    pub async fn critical(&self, message: impl Into<String>) -> Result<(), LoggingError> {
        self.log(LogLevel::Critical, message).await
    }

    /// Log with custom level
    pub async fn log(&self, level: LogLevel, message: impl Into<String>) -> Result<(), LoggingError> {
        if level < self.min_level {
            return Ok(());
        }

        let entry = LogEntry::new(level, message, &self.component);

        // Write to all sinks
        for sink in &self.sinks {
            sink.write(&entry).await?;
        }

        Ok(())
    }

    /// Log with additional fields
    pub async fn log_with_fields(
        &self,
        level: LogLevel,
        message: impl Into<String>,
        fields: HashMap<String, serde_json::Value>,
    ) -> Result<(), LoggingError> {
        if level < self.min_level {
            return Ok(());
        }

        let mut entry = LogEntry::new(level, message, &self.component);
        entry.fields = fields;

        for sink in &self.sinks {
            sink.write(&entry).await?;
        }

        Ok(())
    }

    /// Flush all sinks
    pub async fn flush(&self) -> Result<(), LoggingError> {
        for sink in &self.sinks {
            sink.flush().await?;
        }
        Ok(())
    }
}

/// Logging error types
#[derive(Debug, Clone, thiserror::Error, JsonSchema)]
pub enum LoggingError {
    #[error("Log sink error: {message}")]
    SinkError { message: String },

    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    #[error("Configuration error: {message}")]
    ConfigError { message: String },
}

/// Structured logging macros
#[macro_export]
macro_rules! log_debug {
    ($logger:expr, $($arg:tt)*) => {
        $logger.debug(format!($($arg)*)).await
    };
}

#[macro_export]
macro_rules! log_info {
    ($logger:expr, $($arg:tt)*) => {
        $logger.info(format!($($arg)*)).await
    };
}

#[macro_export]
macro_rules! log_warn {
    ($logger:expr, $($arg:tt)*) => {
        $logger.warn(format!($($arg)*)).await
    };
}

#[macro_export]
macro_rules! log_error {
    ($logger:expr, $($arg:tt)*) => {
        $logger.error(format!($($arg)*)).await
    };
}

#[macro_export]
macro_rules! log_critical {
    ($logger:expr, $($arg:tt)*) => {
        $logger.critical(format!($($arg)*)).await
    };
}

/// Log aggregator for collecting and managing logs from multiple sources
pub struct LogAggregator {
    /// Component name
    component: String,
    /// Child loggers
    children: HashMap<String, Logger>,
    /// Global log level
    global_level: LogLevel,
    /// Shared sinks
    sinks: Vec<Box<dyn LogSink>>,
}

impl LogAggregator {
    /// Create a new log aggregator
    pub fn new(component: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            children: HashMap::new(),
            global_level: LogLevel::Info,
            sinks: Vec::new(),
        }
    }

    /// Add a shared sink
    pub fn add_sink(mut self, sink: Box<dyn LogSink>) -> Self {
        self.sinks.push(sink);
        self
    }

    /// Set global log level
    pub fn set_global_level(&mut self, level: LogLevel) {
        self.global_level = level;
        // Update all children
        for child in self.children.values_mut() {
            child.min_level = level;
        }
    }

    /// Create a child logger
    pub fn create_child(&mut self, name: impl Into<String>) -> &mut Logger {
        let child_name = format!("{}.{}", self.component, name.into());
        let logger = Logger::new(&child_name)
            .with_min_level(self.global_level);

        // Add all shared sinks
        let logger = self.sinks.iter().fold(logger, |logger, sink| {
            logger.add_sink(sink.as_ref().as_ref().clone())
        });

        self.children.insert(child_name.clone(), logger);
        self.children.get_mut(&child_name).unwrap()
    }

    /// Get a child logger
    pub fn get_child(&self, name: &str) -> Option<&Logger> {
        let full_name = format!("{}.{}", self.component, name);
        self.children.get(&full_name)
    }

    /// Get a mutable child logger
    pub fn get_child_mut(&mut self, name: &str) -> Option<&mut Logger> {
        let full_name = format!("{}.{}", self.component, name);
        self.children.get_mut(&full_name)
    }

    /// Flush all loggers
    pub async fn flush_all(&self) -> Result<(), LoggingError> {
        for sink in &self.sinks {
            sink.flush().await?;
        }
        Ok(())
    }
}

/// Performance logging utilities
pub struct PerformanceLogger {
    logger: Logger,
}

impl PerformanceLogger {
    /// Create a new performance logger
    pub fn new(logger: Logger) -> Self {
        Self { logger }
    }

    /// Log operation timing
    pub async fn log_timing(
        &self,
        operation: &str,
        duration_ms: u64,
        success: bool,
    ) -> Result<(), LoggingError> {
        let level = if success { LogLevel::Info } else { LogLevel::Warn };
        let status = if success { "completed" } else { "failed" };

        self.logger.log_with_fields(
            level,
            format!("Operation '{}' {} in {}ms", operation, status, duration_ms),
            {
                let mut fields = HashMap::new();
                fields.insert("operation".to_string(), serde_json::json!(operation));
                fields.insert("duration_ms".to_string(), serde_json::json!(duration_ms));
                fields.insert("success".to_string(), serde_json::json!(success));
                fields
            },
        ).await
    }

    /// Log resource usage
    pub async fn log_resource_usage(
        &self,
        component: &str,
        cpu_percent: f32,
        memory_mb: u64,
    ) -> Result<(), LoggingError> {
        self.logger.log_with_fields(
            LogLevel::Info,
            format!("Resource usage for '{}': CPU {:.1}%, Memory {}MB", component, cpu_percent, memory_mb),
            {
                let mut fields = HashMap::new();
                fields.insert("component".to_string(), serde_json::json!(component));
                fields.insert("cpu_percent".to_string(), serde_json::json!(cpu_percent));
                fields.insert("memory_mb".to_string(), serde_json::json!(memory_mb));
                fields
            },
        ).await
    }
}
