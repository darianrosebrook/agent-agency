//! Observability crate for comprehensive monitoring and logging
//!
//! This crate provides:
//! - Structured logging with tracing
//! - SLO tracking and alerting
//! - Basic metrics collection
//! - Agent-specific telemetry and performance tracking
//! - Real-time system monitoring and business intelligence
//! - Caching backends for performance optimization

pub mod agent_telemetry;
pub mod alerts;
pub mod analytics;
pub mod analytics_dashboard;
pub mod cache;
pub mod dashboard;
pub mod diff_observability;
pub mod health_monitoring;
pub mod observability_errors;
pub mod logging;
pub mod observability_metrics;
pub mod multimodal_metrics;
pub mod otel_integration;
pub mod slo;
pub mod span_management;
pub mod trace_hierarchy;
pub mod trace_types;
pub mod tracing;

// Re-export specific types to avoid conflicts
pub use agent_telemetry::{
    AgentPerformanceMetrics, AgentPerformanceTracker, AgentTelemetryCollector, AgentType,
    BusinessMetrics, CoordinationMetrics, SystemDashboard, SystemAlert, TelemetryConfig,
};
pub use alerts::{
    Alert, AlertCondition, AlertManager, AlertRule, AlertSeverity as AlertSeverityType,
    AlertStatus as AlertStatusType, AlertType,
};
pub use analytics::{
    AnalyticsInsight, AnalyticsDashboardData, AnalyticsSystemOverview,
    TrendAnalysisSummary, AnomalyDetectionSummary, PredictiveInsightsSummary,
    TrendAnalysis, AnomalyDetectionResult, PredictiveModelResult,
    OptimizationRecommendation, CachedInsights, CacheMetadata,
    CpuStatistics, CpuMeasurement, AnalyticsError,
    InsightType, InsightSeverity, TrendDirection, AnomalySeverity,
    PredictionType, ConfidenceInterval, OptimizationPriority, ImplementationEffort,
    AnalyticsRealTimeUpdate, TrendUpdates, AnomalyUpdates, PredictionUpdates, OptimizationUpdates,
    SystemMetrics, AgentMetrics, TaskMetrics, ProcessedSystemMetrics, ValidatedPredictions, CachePerformanceMetrics,
    MLModel, ModelPrediction, OnnxModelInfo, InferenceResult,
    AnalyticsDashboard,
};
// Note: Analytics functionality now consolidated in analytics module above
pub use cache::{RedisCache, CacheBackend, CacheError};
pub use dashboard::*;
pub use logging::*;
pub use observability_metrics::{
    MetricsBackend, NoOpMetricsBackend, InMemoryMetricsBackend,
    MetricsCollector, MetricValue, MetricsSnapshot,
};
pub use observability_metrics::prometheus::PrometheusMetrics;
pub use observability_metrics::statsd::{StatsDMetrics, StatsDCircuitBreaker};
pub use observability_metrics::redis::{RedisMetrics, RedisCircuitBreaker, RedisMetricsError as RedisError};
pub use multimodal_metrics::{
    MultimodalMetricsCollector, MultimodalProcessingMetrics, VectorSearchMetrics,
    EmbeddingMetrics, CrossModalValidationMetrics, ContextRetrievalMetrics,
    DeduplicationMetrics, MultimodalSystemHealth, PerformanceSummary,
};
pub use slo::*;
pub use diff_observability::{
    DiffGenerator, DiffViewer, UnifiedDiff, DiffHeader, DiffHunk, DiffStats, DiffMetadata,
    SideBySideConfig, SideBySideView, ViolationSummary, FileNavigation, ViolationSeverity,
    DiffGeneratorConfig, FileChange, DiffError,
};
pub use observability_errors::ObservabilityError;
