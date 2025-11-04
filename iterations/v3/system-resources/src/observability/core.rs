//! Core observability types and configuration

use schemars::JsonSchema;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Observability configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ObservabilityConfig {
    /// Enable metrics collection
    pub enable_metrics: bool,
    /// Enable logging
    pub enable_logging: bool,
    /// Enable health checks
    pub enable_health_checks: bool,
    /// Metrics retention period in hours
    pub metrics_retention_hours: u64,
    /// Log retention period in hours
    pub log_retention_hours: u64,
    /// Health check interval in seconds
    pub health_check_interval_seconds: u64,
    /// Alert thresholds for various metrics
    pub alert_thresholds: HashMap<String, f64>,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            enable_metrics: true,
            enable_logging: true,
            enable_health_checks: true,
            metrics_retention_hours: 24,
            log_retention_hours: 168, // 7 days
            health_check_interval_seconds: 30,
            alert_thresholds: HashMap::new(),
        }
    }
}

/// Health check status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum HealthStatus {
    /// Component is healthy
    Healthy,
    /// Component is degraded but functional
    Degraded,
    /// Component is unhealthy
    Unhealthy,
    /// Health status is unknown
    Unknown,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "Healthy"),
            HealthStatus::Degraded => write!(f, "Degraded"),
            HealthStatus::Unhealthy => write!(f, "Unhealthy"),
            HealthStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Component name
    pub component: String,
    /// Health status
    pub status: HealthStatus,
    /// Timestamp of the check
    ##[schemars(with = "String")]

    pub timestamp: DateTime<Utc>,
    /// Optional error message
    pub error_message: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Check duration in milliseconds
    pub duration_ms: u64,
}

impl HealthCheckResult {
    /// Create a healthy result
    pub fn healthy(component: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            status: HealthStatus::Healthy,
            timestamp: Utc::now(),
            error_message: None,
            metadata: HashMap::new(),
            duration_ms: 0,
        }
    }

    /// Create an unhealthy result
    pub fn unhealthy(component: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            status: HealthStatus::Unhealthy,
            timestamp: Utc::now(),
            error_message: Some(error.into()),
            metadata: HashMap::new(),
            duration_ms: 0,
        }
    }
}

/// Log entry levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
pub enum LogLevel {
    /// Debug information
    Debug = 1,
    /// General information
    Info = 2,
    /// Warning messages
    Warn = 3,
    /// Error messages
    Error = 4,
    /// Critical errors
    Critical = 5,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Log entry
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LogEntry {
    /// Log level
    pub level: LogLevel,
    /// Log message
    pub message: String,
    /// Component that generated the log
    pub component: String,
    /// Timestamp
    ##[schemars(with = "String")]

    pub timestamp: DateTime<Utc>,
    /// Additional fields
    pub fields: HashMap<String, serde_json::Value>,
    /// Optional error information
    pub error: Option<String>,
    /// Request ID for tracing
    pub request_id: Option<String>,
}

impl LogEntry {
    /// Create a new log entry
    pub fn new(level: LogLevel, message: impl Into<String>, component: impl Into<String>) -> Self {
        Self {
            level,
            message: message.into(),
            component: component.into(),
            timestamp: Utc::now(),
            fields: HashMap::new(),
            error: None,
            request_id: None,
        }
    }

    /// Add a field to the log entry
    pub fn with_field(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.fields.insert(key.into(), value.into());
        self
    }

    /// Add error information
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    /// Add request ID for tracing
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }
}
