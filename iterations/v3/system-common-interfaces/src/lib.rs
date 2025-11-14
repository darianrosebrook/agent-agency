//! System Common Interfaces
//!
//! This crate provides common interfaces and types that are shared across multiple
//! system crates without creating circular dependencies. All interfaces are designed
//! to be dependency-injection friendly, allowing concrete implementations to be
//! provided at runtime.
//!
//! ## Architecture
//!
//! This crate breaks circular dependencies by providing:
//!
//! - **Trait-based interfaces**: Allow dependency injection of implementations
//! - **Common data types**: Shared without implementation details
//! - **Abstracted services**: Database, observability, health checks
//! - **Configuration types**: Shared configuration structures
//!
//! ## Usage Pattern
//!
//! ```rust
//! use system_common_interfaces::{DatabaseInterface, ObservabilityInterface};
//!
//! struct MyService<D: DatabaseInterface, O: ObservabilityInterface> {
//!     database: D,
//!     observability: O,
//! }
//! ```
//!
//! @author @darianrosebrook

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod common;
pub mod config;
pub mod database;
pub mod file_operations;
pub mod health;
pub mod learning;
pub mod memory;
pub mod model_orchestration;
pub mod observability;
pub mod types;

pub use common::*;
pub use config::*;
pub use database::*;
pub use file_operations::*;
pub use health::{
    DatabaseHealthCheck, DependencyHealth, DependencyHealthCheck, HealthCheck, HealthCheckExecutor,
    HealthCheckInfo, HealthCheckRegistry, HealthCheckResult, HealthCheckScheduler, HealthReport,
    HealthSummary, HttpHealthCheck, ScheduledCheckStatus,
};
pub use learning::{
    AlgorithmConfig, AlgorithmStatistics, Difficulty, Experience, Improvement, ImprovementType,
    LearningContext, LearningError, LearningInsights, LearningStatistics, OptimizationGoal,
    OptimizationRecommendation, Pattern, PatternType, Priority, QTable, RecommendationType,
    TaskPerformance,
};
pub use memory::*;
pub use model_orchestration::{
    InferenceRequest as OrchestratorInferenceRequest, InferenceResponse, ModelCapabilities,
    ModelInstance, ModelOrchestrator, ModelStatistics, OrchestrationError, OrchestrationResult,
    OrchestrationStatistics, PerformanceCharacteristics,
    PerformanceRequirements as OrchestratorPerformanceRequirements,
    Priority as OrchestratorPriority, QualityRequirements as OrchestratorQualityRequirements,
    RoutingDecision, RoutingStrategy,
};
pub use observability::{
    ComponentHealth, HealthMonitoringInterface, HealthReport as ObsHealthReport, LoggingInterface,
    MetricType, MetricValue, ObsValue, ObservabilityInterface, PerformanceMetric,
    PerformanceMetrics as ObsPerformanceMetrics, PerformanceMonitoringInterface, SpanHandle,
    SpanStatus, SystemPerformanceStats, TracingInterface,
};
pub use types::{
    ApiResponse, AuditLogEntry, CircuitBreakerState, CircuitBreakerStats, ErrorDetails,
    ErrorResponse, EventMetadata, EventSeverity, Filter, FilterOperator, HealthCheckResponse,
    HealthCheckStatus, MessageEnvelope, MetricDataPoint, MetricsSnapshot, Pagination,
    PerformanceMetrics, QueryParams, RateLimitInfo, ResponseMetadata, RetryConfig, ServiceInfo,
    SortDirection, Sorting, SystemEvent, TaskId, TaskPriority, TaskResult, TaskStatus,
    TimeoutConfig, VersionInfo,
};

/// Common result type
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Service health status
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Service lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ServiceState {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
}

/// Common service metadata
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ServiceMetadata {
    pub service_name: String,
    pub service_version: String,
    #[schemars(with = "String")]
    pub instance_id: Uuid,
    #[schemars(with = "String")]
    pub started_at: DateTime<Utc>,
    pub environment: String,
}

/// Common error types that can be shared across services
#[derive(thiserror::Error, Debug)]
pub enum SystemError {
    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Common pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub order_by: Option<String>,
    pub order_direction: Option<OrderDirection>,
}

/// Sort direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderDirection {
    Asc,
    Desc,
}
