//! Core tracing types and data structures
//!
//! Defines the fundamental types used throughout the tracing system
//! for spans, traces, events, and status information.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
/// Trace context information for distributed tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceContext {
    /// Unique trace identifier
    pub trace_id: String,
    /// Current span identifier
    pub span_id: String,
    /// Parent span identifier if nested
    pub parent_span_id: Option<String>,
    /// Service name generating the trace
    pub service_name: String,
    /// Operation being traced
    pub operation: String,
    /// Additional metadata tags
    pub tags: HashMap<String, String>,
}

/// Information about a single span in a trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanInfo {
    /// Span name/operation
    pub name: String,
    /// When the span started
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// When the span ended (if completed)
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    /// Duration in milliseconds (calculated)
    pub duration_ms: Option<u64>,
    /// Span attributes/metadata
    pub attributes: HashMap<String, serde_json::Value>,
    /// Events that occurred during the span
    pub events: Vec<SpanEvent>,
    /// Span completion status
    pub status: SpanStatus,
}

/// Event that occurred within a span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
    /// Event name
    pub name: String,
    /// When the event occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Event attributes
    pub attributes: HashMap<String, serde_json::Value>,
}

/// Status of a span or trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpanStatus {
    /// Span completed successfully
    Ok,
    /// Span completed with error
    Error,
    /// Span status not set
    Unset,
}

/// Complete trace information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceInfo {
    /// Unique trace identifier
    pub trace_id: String,
    /// Root span of the trace
    pub root_span: SpanInfo,
    /// All child spans in the trace
    pub child_spans: Vec<SpanInfo>,
    /// Total trace duration in milliseconds
    pub duration_ms: u64,
    /// Overall trace status
    pub status: TraceStatus,
}

/// Overall status of a complete trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraceStatus {
    /// All spans completed successfully
    Success,
    /// One or more spans failed
    PartialFailure,
    /// Trace timed out
    Timeout,
    /// Trace was cancelled
    Cancelled,
}

/// Detailed error information from span analysis
#[derive(Debug, Clone)]
pub struct SpanErrorInfo {
    /// Whether this span represents an error
    pub is_error: bool,
    /// Error message if applicable
    pub error_message: Option<String>,
    /// Error type/category
    pub error_type: Option<String>,
    /// Stack trace if available
    pub stack_trace: Option<String>,
    /// HTTP status code if applicable
    pub http_status: Option<u16>,
}

/// Configuration for the tracing system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceConfig {
    /// Whether tracing is enabled
    pub enabled: bool,
    /// Service name for this instance
    pub service_name: String,
    /// Sampling rate (0.0-1.0)
    pub sample_rate: f64,
    /// Maximum number of spans per trace
    pub max_spans_per_trace: usize,
    /// Maximum span duration before timeout
    pub max_span_duration_ms: u64,
    /// Whether to export traces via OTLP
    pub enable_otlp: bool,
    /// OTLP endpoint URL
    pub otlp_endpoint: Option<String>,
    /// Whether to include detailed span attributes
    pub detailed_attributes: bool,
    /// Buffer size for pending spans
    pub buffer_size: usize,
}

impl Default for TraceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            service_name: "agent-agency".to_string(),
            sample_rate: 0.1, // 10% sampling
            max_spans_per_trace: 1000,
            max_span_duration_ms: 300000, // 5 minutes
            enable_otlp: false,
            otlp_endpoint: None,
            detailed_attributes: true,
            buffer_size: 10000,
        }
    }
}

/// Result of a health check operation
#[derive(Debug, Clone, Serialize, Deserialize, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Component being checked
    pub component: String,
    /// Whether the check passed
    pub healthy: bool,
    /// Check timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Additional health metrics
    pub metrics: HashMap<String, serde_json::Value>,
    /// Error message if unhealthy
    pub error_message: Option<String>,
}

/// State of a circuit breaker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerState {
    /// Component protected by circuit breaker
    pub component: String,
    /// Current state
    pub state: CircuitBreakerStatus,
    /// Failure count
    pub failure_count: u32,
    /// Success count
    pub success_count: u32,
    /// Last failure time
    pub last_failure_time: Option<chrono::DateTime<chrono::Utc>>,
    /// Last success time
    pub last_success_time: Option<chrono::DateTime<chrono::Utc>>,
}

/// Status of a circuit breaker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CircuitBreakerStatus {
    /// Circuit is closed, requests flow normally
    Closed,
    /// Circuit is open, requests are blocked
    Open,
    /// Circuit is testing if service recovered
    HalfOpen,
}

/// Snapshot of system health across all components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthSnapshot {
    /// Overall system health
    pub overall_healthy: bool,
    /// Individual component health results
    pub component_health: HashMap<String, HealthCheckResult>,
    /// Circuit breaker states
    pub circuit_breakers: HashMap<String, CircuitBreakerState>,
    /// System metrics
    pub metrics: HashMap<String, serde_json::Value>,
    /// Snapshot timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
