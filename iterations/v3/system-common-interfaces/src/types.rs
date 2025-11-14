//! Common Data Types
//!
//! Shared data types and structures that are used across multiple crates
//! without creating circular dependencies. These types define the common
//! language for communication between system components.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for tasks and operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub Uuid);

impl TaskId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Re-export TaskPriority from contracts for backward compatibility
pub use agent_agency_contracts::types::planning::TaskPriority;

/// Task execution status
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Paused,
}

/// Task scope for execution control
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskScope {
    Local,
    Distributed,
    Remote(String), // Remote execution endpoint
}

/// Task execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: TaskId,
    pub status: TaskStatus,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Resource usage information
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ResourceUsage {
    pub cpu_percent: f64,
    pub memory_mb: u64,
    pub disk_mb: u64,
    pub network_mbps: f64,
}

/// Performance metrics for operations
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PerformanceMetrics {
    pub operation_name: String,
    #[schemars(with = "String")]
    pub start_time: DateTime<Utc>,
    #[schemars(with = "String")]
    pub end_time: DateTime<Utc>,
    pub duration_ms: u64,
    pub resource_usage: ResourceUsage,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Audit log entry for security and compliance
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AuditLogEntry {
    #[schemars(with = "String")]
    pub id: Uuid,
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<String>,
    pub action: String,
    pub resource: String,
    pub resource_id: Option<String>,
    pub details: serde_json::Value,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub success: bool,
}

/// Message envelope for inter-service communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope<T> {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub sender: String,
    pub recipient: String,
    pub message_type: String,
    pub correlation_id: Option<Uuid>,
    pub payload: T,
    pub headers: HashMap<String, String>,
}

/// Event types for the event system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemEvent {
    TaskStarted {
        task_id: TaskId,
        task_type: String,
    },
    TaskCompleted {
        task_id: TaskId,
        result: TaskResult,
    },
    TaskFailed {
        task_id: TaskId,
        error: String,
    },
    ServiceStarted {
        service_name: String,
        version: String,
    },
    ServiceStopped {
        service_name: String,
    },
    HealthCheckFailed {
        component: String,
        error: String,
    },
    ResourceThresholdExceeded {
        resource: String,
        current: f64,
        threshold: f64,
    },
    ConfigurationChanged {
        key: String,
        old_value: Option<String>,
        new_value: Option<String>,
    },
}

/// Event metadata
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct EventMetadata {
    #[schemars(with = "String")]
    pub event_id: Uuid,
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub severity: EventSeverity,
    pub tags: Vec<String>,
}

/// Event severity levels
#[derive(
    Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, schemars::JsonSchema,
)]
pub enum EventSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

/// Pagination parameters for list operations
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
    pub total_items: Option<u64>,
    pub total_pages: Option<u32>,
}

/// Sorting parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sorting {
    pub field: String,
    pub direction: SortDirection,
}

/// Sort direction
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SortDirection {
    Ascending,
    Descending,
}

/// Filter parameters for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    pub field: String,
    pub operator: FilterOperator,
    pub value: serde_json::Value,
}

/// Filter operators
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FilterOperator {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Contains,
    StartsWith,
    EndsWith,
    In,
    NotIn,
}

/// Query parameters combining pagination, sorting, and filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryParams {
    pub pagination: Option<Pagination>,
    pub sorting: Option<Sorting>,
    pub filters: Vec<Filter>,
}

/// API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
    pub metadata: ResponseMetadata,
}

/// Response metadata
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ResponseMetadata {
    #[schemars(with = "String")]
    pub request_id: Uuid,
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    pub processing_time_ms: u64,
    pub api_version: String,
    pub pagination: Option<Pagination>,
}

/// Rate limit information
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RateLimitInfo {
    pub limit: u32,
    pub remaining: u32,
    #[schemars(with = "String")]
    pub reset_time: DateTime<Utc>,
    pub retry_after_seconds: Option<u64>,
}

/// Error response structure
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ErrorResponse {
    pub error: ErrorDetails,
    #[schemars(with = "String")]
    pub request_id: Uuid,
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
}

/// Error details
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub field_errors: Option<HashMap<String, String>>,
}

/// Version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub version: String,
    pub git_commit: String,
    pub build_date: String,
    pub rust_version: String,
    pub dependencies: HashMap<String, String>,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub version: VersionInfo,
    pub checks: HashMap<String, HealthCheckStatus>,
    pub uptime_seconds: u64,
}

/// Individual health check status
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct HealthCheckStatus {
    pub status: String,
    pub message: Option<String>,
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    pub response_time_ms: u64,
}

/// Metrics data point
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct MetricDataPoint {
    pub name: String,
    pub value: f64,
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    pub tags: HashMap<String, String>,
}

/// Metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct MetricsSnapshot {
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    pub metrics: Vec<MetricDataPoint>,
    pub interval_seconds: u64,
}

/// Service discovery information
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ServiceInfo {
    pub name: String,
    pub address: String,
    pub port: u16,
    pub health_endpoint: Option<String>,
    pub metadata: HashMap<String, String>,
    #[schemars(with = "String")]
    pub last_seen: DateTime<Utc>,
}

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

/// Circuit breaker statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerStats {
    pub state: CircuitBreakerState,
    pub failure_count: u64,
    pub success_count: u64,
    pub consecutive_failures: u64,
    pub consecutive_successes: u64,
    pub last_failure_time: Option<DateTime<Utc>>,
    pub last_success_time: Option<DateTime<Utc>>,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
    pub retryable_errors: Vec<String>,
}

/// Timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    pub connect_timeout_ms: u64,
    pub read_timeout_ms: u64,
    pub write_timeout_ms: u64,
    pub overall_timeout_ms: u64,
}
