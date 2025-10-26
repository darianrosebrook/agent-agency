//! Analytics dashboard module
//!
//! This module provides comprehensive analytics visualization, trend analysis,
//! anomaly detection, and predictive insights for the Agent Agency V3 system.

pub mod redis_client;
pub mod config;
pub mod data;
pub mod updates;
pub mod metrics;
pub mod ml;
pub mod dashboard;

// Re-exports for external users
pub use redis_client::{RedisClient, RedisConfig, ProductionRedisClient};
pub use config::{AnalyticsDashboardConfig, AnalyticsSession, AnalyticsPreferences, AnalyticsAlertPreferences, AnalyticsSubscriptionType};
pub use data::{
    AnalyticsInsight, AnalyticsDashboardData, AnalyticsSystemOverview,
    TrendAnalysisSummary, AnomalyDetectionSummary, PredictiveInsightsSummary,
    TrendAnalysis, AnomalyDetectionResult, PredictiveModelResult,
    OptimizationRecommendation, CachedInsights, CacheMetadata,
    CpuStatistics, CpuMeasurement, AnalyticsError,
    InsightType, InsightSeverity, TrendDirection, AnomalySeverity,
    PredictionType, ConfidenceInterval, OptimizationPriority, ImplementationEffort,
};
pub use updates::{AnalyticsRealTimeUpdate, TrendUpdates, AnomalyUpdates, PredictionUpdates, OptimizationUpdates};
pub use metrics::{SystemMetrics, AgentMetrics, TaskMetrics, ProcessedSystemMetrics, ValidatedPredictions, CachePerformanceMetrics};
pub use ml::{MLModel, ModelPrediction, OnnxModelInfo, InferenceResult};
pub use dashboard::AnalyticsDashboard;
