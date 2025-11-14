//! Observability Interface
//!
//! Common observability interface for metrics, logging, and tracing that can be
//! implemented by different observability backends without creating dependencies.
//!
//! This allows system-observability to provide concrete implementations while
//! other crates can depend on the interface for observability operations.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::Level;

use crate::{HealthStatus, Result};

/// Metric value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(Vec<f64>),
}

/// Metric definition
#[derive(Debug, Clone)]
pub struct MetricDefinition {
    pub name: String,
    pub description: String,
    pub metric_type: MetricType,
    pub labels: HashMap<String, String>,
}

/// Metric types
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
}

/// Observability interface for metrics collection
#[async_trait]
pub trait ObservabilityInterface: Send + Sync {
    /// Record a counter metric
    async fn counter(&self, name: &str, value: u64, labels: HashMap<String, String>) -> Result<()>;

    /// Increment a counter by 1
    async fn increment_counter(&self, name: &str, labels: HashMap<String, String>) -> Result<()>;

    /// Record a gauge metric
    async fn gauge(&self, name: &str, value: f64, labels: HashMap<String, String>) -> Result<()>;

    /// Record a histogram observation
    async fn histogram(
        &self,
        name: &str,
        value: f64,
        labels: HashMap<String, String>,
    ) -> Result<()>;

    /// Record timing for an operation
    async fn timing(
        &self,
        name: &str,
        duration: Duration,
        labels: HashMap<String, String>,
    ) -> Result<()>;

    /// Time a future and record its duration
    async fn time_future<F, Fut, T>(
        &self,
        name: &str,
        labels: HashMap<String, String>,
        future: F,
    ) -> Result<T>
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T>> + Send,
        T: Send;

    /// Flush pending metrics
    async fn flush(&self) -> Result<()>;
}

/// Simple value type for observability interfaces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObsValue {
    String(String),
    Number(f64),
    Bool(bool),
}

/// Tracing interface for distributed tracing
#[async_trait]
pub trait TracingInterface: Send + Sync {
    /// Start a new span
    async fn start_span(&self, name: &str, level: Level) -> Result<SpanHandle>;

    /// Start a child span
    async fn child_span(&self, parent: &SpanHandle, name: &str) -> Result<SpanHandle>;

    /// Set attributes on a span
    async fn set_attributes(
        &self,
        span: &SpanHandle,
        attributes: HashMap<String, ObsValue>,
    ) -> Result<()>;

    /// Record an event in a span
    async fn record_event(
        &self,
        span: &SpanHandle,
        event: &str,
        attributes: HashMap<String, ObsValue>,
    ) -> Result<()>;

    /// Set span status
    async fn set_status(&self, span: &SpanHandle, status: SpanStatus) -> Result<()>;

    /// End a span
    async fn end_span(&self, span: SpanHandle) -> Result<()>;
}

/// Handle to an active span
#[derive(Debug, Clone)]
pub struct SpanHandle {
    pub id: String,
    pub trace_id: String,
}

/// Span status
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SpanStatus {
    Ok,
    Error,
    Unset,
}

/// Logging interface for structured logging
#[async_trait]
pub trait LoggingInterface: Send + Sync {
    /// Log a message at the specified level
    async fn log(
        &self,
        level: Level,
        message: &str,
        fields: HashMap<String, ObsValue>,
    ) -> Result<()>;

    /// Log with error context
    async fn log_error(
        &self,
        error: &dyn std::error::Error,
        message: &str,
        fields: HashMap<String, ObsValue>,
    ) -> Result<()>;

    /// Create a logger with pre-set context
    fn with_context(&self, context: HashMap<String, ObsValue>) -> Box<dyn LoggingInterface>;
}

/// Health monitoring interface
#[async_trait]
pub trait HealthMonitoringInterface: Send + Sync {
    /// Register a health check
    async fn register_health_check(&self, name: &str, check: Box<dyn HealthCheck>) -> Result<()>;

    /// Perform all registered health checks
    async fn check_health(&self) -> Result<HealthReport>;

    /// Get health status for a specific check
    async fn check_component(&self, name: &str) -> Result<ComponentHealth>;
}

/// Individual health check
#[async_trait]
pub trait HealthCheck: Send + Sync {
    /// Perform the health check
    async fn check(&self) -> Result<HealthStatus>;
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub overall_status: crate::HealthStatus,
    pub components: HashMap<String, ComponentHealth>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Individual component health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: crate::HealthStatus,
    pub message: Option<String>,
    pub details: HashMap<String, serde_json::Value>,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub response_time_ms: Option<u64>,
}

/// Performance monitoring interface
#[async_trait]
pub trait PerformanceMonitoringInterface: Send + Sync {
    /// Start performance profiling
    async fn start_profiling(&self, name: &str) -> Result<ProfileHandle>;

    /// End performance profiling and record metrics
    async fn end_profiling(&self, handle: ProfileHandle) -> Result<PerformanceMetrics>;

    /// Record custom performance metric
    async fn record_metric(&self, metric: PerformanceMetric) -> Result<()>;

    /// Get current performance statistics
    async fn get_stats(&self) -> Result<SystemPerformanceStats>;
}

/// Handle for active performance profiling
#[derive(Debug)]
pub struct ProfileHandle {
    pub id: String,
    pub start_time: Instant,
}

/// Performance metrics from profiling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub operation_name: String,
    pub duration_ms: u64,
    pub cpu_time_ms: Option<u64>,
    pub memory_usage_kb: Option<u64>,
    pub allocations: Option<u64>,
}

/// Individual performance metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub tags: HashMap<String, String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// System-wide performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPerformanceStats {
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: u64,
    pub disk_io_mbps: f64,
    pub network_io_mbps: f64,
    pub active_connections: u32,
    pub request_rate_per_sec: f64,
    pub error_rate_per_sec: f64,
    pub p95_response_time_ms: f64,
    pub p99_response_time_ms: f64,
}
