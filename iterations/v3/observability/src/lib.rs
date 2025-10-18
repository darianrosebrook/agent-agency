//! Observability crate for comprehensive monitoring and logging
//!
//! This crate provides:
//! - Structured logging with tracing
//! - SLO tracking and alerting
//! - Basic metrics collection
//! - Agent-specific telemetry and performance tracking
//! - Real-time system monitoring and business intelligence

pub mod agent_telemetry;
pub mod alerts;
pub mod analytics;
pub mod analytics_dashboard;
pub mod dashboard;
pub mod logging;
pub mod metrics;
pub mod slo;

// Re-export specific types to avoid conflicts
pub use agent_telemetry::{
    AgentPerformanceMetrics, AgentPerformanceTracker, AgentTelemetryCollector, AgentType,
    BusinessMetrics, CoordinationMetrics,
};
pub use alerts::{
    Alert, AlertCondition, AlertManager, AlertRule, AlertSeverity as AlertSeverityType,
    AlertStatus as AlertStatusType, AlertType,
};
pub use analytics::{
    AgentPerformanceSnapshot, AnalyticsConfig, AnalyticsEngine, AnomalyDetectionResult,
    AnomalySeverity, AnomalyType, BusinessMetricsSnapshot, CoordinationMetricsSnapshot,
    EffortLevel, HistoricalData, OptimizationRecommendation, OptimizationType, PredictionType,
    PredictiveModelResult, PriorityLevel, SystemHealthSnapshot, TrendAnalysis, TrendDirection,
};
pub use analytics_dashboard::{
    AnalyticsAlertPreferences, AnalyticsDashboard, AnalyticsDashboardConfig,
    AnalyticsDashboardData, AnalyticsInsight, AnalyticsPreferences, AnalyticsRealTimeUpdate,
    AnalyticsSession, AnalyticsSubscriptionType, AnalyticsSystemOverview, AnomalyDetectionSummary,
    AnomalyUpdates, ChartConfig, ChartType, DataPoint, InsightSeverity, InsightType,
    OptimizationUpdates, PredictionUpdates, PredictiveInsightsSummary, TrendAnalysisSummary,
    TrendUpdates, VisualData,
};
pub use dashboard::*;
pub use logging::*;
pub use metrics::*;
pub use slo::*;
