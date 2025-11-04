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

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

pub mod database;
pub mod observability;
pub mod health;
pub mod config;
pub mod types;
pub mod file_operations;
pub mod learning;
pub mod model_orchestration;
pub mod memory;
pub mod common;

pub use database::*;
pub use observability::{ObservabilityInterface, TracingInterface, LoggingInterface, HealthMonitoringInterface, PerformanceMonitoringInterface, MetricValue, MetricType, ObsValue, SpanHandle, SpanStatus, PerformanceMetrics as ObsPerformanceMetrics, PerformanceMetric, SystemPerformanceStats, HealthReport as ObsHealthReport, ComponentHealth};
pub use health::{HealthCheck, HealthCheckRegistry, HealthCheckExecutor, HealthCheckResult, HealthCheckInfo, HealthReport, HealthSummary, HealthCheckScheduler, ScheduledCheckStatus, DependencyHealthCheck, DependencyHealth, DatabaseHealthCheck, HttpHealthCheck};
pub use config::*;
pub use types::{TaskId, TaskPriority, TaskStatus, TaskResult, PerformanceMetrics, AuditLogEntry, MessageEnvelope, SystemEvent, EventMetadata, EventSeverity, Pagination, Sorting, SortDirection, Filter, FilterOperator, QueryParams, ApiResponse, ResponseMetadata, RateLimitInfo, ErrorResponse, ErrorDetails, VersionInfo, HealthCheckResponse, HealthCheckStatus, MetricDataPoint, MetricsSnapshot, ServiceInfo, CircuitBreakerState, CircuitBreakerStats, RetryConfig, TimeoutConfig};
pub use file_operations::*;
pub use learning::{LearningError, AlgorithmConfig, QTable, AlgorithmStatistics, Experience, LearningContext, TaskPerformance, OptimizationGoal, LearningInsights, Pattern, PatternType, Improvement, ImprovementType, Difficulty, OptimizationRecommendation, RecommendationType, Priority, LearningStatistics};
pub use model_orchestration::{ModelOrchestrator, InferenceRequest as OrchestratorInferenceRequest, InferenceResponse, RoutingDecision, RoutingStrategy, ModelInstance, ModelCapabilities, PerformanceCharacteristics, ModelStatistics, OrchestrationStatistics, OrchestrationError, OrchestrationResult, Priority as OrchestratorPriority, PerformanceRequirements as OrchestratorPerformanceRequirements, QualityRequirements as OrchestratorQualityRequirements};
pub use memory::*;
pub use common::*;

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

