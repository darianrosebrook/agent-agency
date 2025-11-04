//! Core data structures for analytics dashboard

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
/// Analytics insight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsInsight {
    /// Insight ID
    pub insight_id: String,
    /// Insight type
    pub insight_type: InsightType,
    /// Insight title
    pub title: String,
    /// Insight description
    pub description: String,
    /// Insight severity
    pub severity: InsightSeverity,
    /// Confidence score
    pub confidence: f64,
    /// Timestamp
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    /// Related metrics
    pub related_metrics: Vec<String>,
    /// Recommendations
    pub recommendations: Vec<String>,
    /// Visual data
    pub visual_data: Option<VisualData>,
}

/// Insight types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InsightType {
    TrendAnalysis,
    AnomalyDetection,
    PredictiveInsight,
    OptimizationOpportunity,
    PerformanceBottleneck,
    CapacityPlanning,
}

/// Insight severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InsightSeverity {
    Info,
    Warning,
    Critical,
    Opportunity,
}

/// Visual data for charts and graphs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualData {
    /// Chart type
    pub chart_type: ChartType,
    /// Data points
    pub data_points: Vec<DataPoint>,
    /// Chart configuration
    pub config: ChartConfig,
}

/// Chart types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChartType {
    Line,
    Bar,
    Scatter,
    Heatmap,
    Gauge,
}

/// Data point for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    /// X-axis value (usually timestamp)
    pub x: f64,
    /// Y-axis value
    pub y: f64,
    /// Label
    pub label: Option<String>,
    /// Color
    pub color: Option<String>,
}

/// Chart configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartConfig {
    /// Chart title
    pub title: String,
    /// X-axis label
    pub x_label: String,
    /// Y-axis label
    pub y_label: String,
    /// Chart width
    pub width: u32,
    /// Chart height
    pub height: u32,
}

/// Analytics dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsDashboardData {
    /// System overview
    pub system_overview: AnalyticsSystemOverview,
    /// Trend analysis
    pub trend_analysis: TrendAnalysisSummary,
    /// Anomaly detection
    pub anomaly_detection: AnomalyDetectionSummary,
    /// Predictive insights
    pub predictive_insights: PredictiveInsightsSummary,
    /// Optimization recommendations
    pub optimization_recommendations: Vec<OptimizationRecommendation>,
    /// Performance insights
    pub performance_insights: Vec<AnalyticsInsight>,
    /// Last updated
    #[schemars(with = "String")]
    pub last_updated: DateTime<Utc>,
}

/// Analytics system overview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSystemOverview {
    /// Overall system health
    pub system_health: String,
    /// Active agents
    pub active_agents: usize,
    /// Total tasks
    pub total_tasks: u32,
    /// System performance score
    pub performance_score: f64,
    /// Quality score
    pub quality_score: f64,
    /// Efficiency score
    pub efficiency_score: f64,
    /// Key metrics trends
    pub key_metrics_trends: HashMap<String, TrendDirection>,
}

/// Trend direction for performance analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Stable,
    Declining,
    Volatile,
}

/// Trend analysis summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysisSummary {
    /// Total trends analyzed
    pub total_trends: usize,
    /// Positive trends
    pub positive_trends: usize,
    /// Negative trends
    pub negative_trends: usize,
    /// Stable trends
    pub stable_trends: usize,
    /// Volatile trends
    pub volatile_trends: usize,
    /// Top trends
    pub top_trends: Vec<TrendAnalysis>,
    /// Trend insights
    pub trend_insights: Vec<AnalyticsInsight>,
}

/// Anomaly detection summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetectionSummary {
    /// Total anomalies detected
    pub total_anomalies: usize,
    /// Critical anomalies
    pub critical_anomalies: usize,
    /// High severity anomalies
    pub high_anomalies: usize,
    /// Medium severity anomalies
    pub medium_anomalies: usize,
    /// Low severity anomalies
    pub low_anomalies: usize,
    /// Recent anomalies
    pub recent_anomalies: Vec<AnomalyDetectionResult>,
    /// Anomaly insights
    pub anomaly_insights: Vec<AnalyticsInsight>,
}

/// Predictive insights summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictiveInsightsSummary {
    /// Total predictions
    pub total_predictions: usize,
    /// Capacity planning predictions
    pub capacity_predictions: Vec<PredictiveModelResult>,
    /// Performance forecasts
    pub performance_forecasts: Vec<PredictiveModelResult>,
    /// Quality predictions
    pub quality_predictions: Vec<PredictiveModelResult>,
    /// Cost projections
    pub cost_projections: Vec<PredictiveModelResult>,
    /// Predictive insights
    pub predictive_insights: Vec<AnalyticsInsight>,
}

/// Trend analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    /// Metric name
    pub metric: String,
    /// Trend direction
    pub direction: TrendDirection,
    /// Trend strength
    pub strength: f64,
    /// Change percentage
    pub change_percent: f64,
    /// Time period analyzed
    pub time_period_hours: u64,
}

/// Anomaly detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetectionResult {
    /// Anomaly ID
    pub anomaly_id: String,
    /// Metric affected
    pub metric: String,
    /// Anomaly severity
    pub severity: AnomalySeverity,
    /// Deviation from normal
    pub deviation: f64,
    /// Confidence in detection
    pub confidence: f64,
    /// Timestamp of anomaly
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    /// Description
    pub description: String,
}

/// Anomaly severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Predictive model result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictiveModelResult {
    /// Model name
    pub model_name: String,
    /// Prediction type
    pub prediction_type: PredictionType,
    /// Predicted value
    pub predicted_value: f64,
    /// Confidence interval
    pub confidence_interval: ConfidenceInterval,
    /// Model accuracy
    pub model_accuracy: f64,
    /// Prediction horizon in hours
    pub prediction_horizon_hours: u64,
    /// Timestamp
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    /// Recommendations
    pub recommendations: Vec<String>,
}

/// Prediction types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PredictionType {
    Performance,
    CapacityPlanning,
    Quality,
    Cost,
    ResourceUtilization,
}

/// Confidence interval for predictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceInterval {
    /// Lower bound
    pub lower: f64,
    /// Upper bound
    pub upper: f64,
    /// Confidence level (0.0-1.0)
    pub confidence_level: f64,
}

/// Optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    /// Recommendation ID
    pub recommendation_id: String,
    /// Title
    pub title: String,
    /// Description
    pub description: String,
    /// Priority
    pub priority: OptimizationPriority,
    /// Expected impact
    pub expected_impact: f64,
    /// Implementation effort
    pub implementation_effort: ImplementationEffort,
    /// Related metrics
    pub related_metrics: Vec<String>,
}

/// Optimization priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Implementation effort levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationEffort {
    Low,
    Medium,
    High,
}

/// Cached insights for performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedInsights {
    /// Cached insights
    pub insights: Vec<AnalyticsInsight>,
    /// Cache timestamp
    #[schemars(with = "String")]
    pub cached_at: DateTime<Utc>,
    /// Cache metadata
    pub metadata: CacheMetadata,
}

/// Cache metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    /// Cache size in bytes
    pub size_bytes: u64,
    /// Number of entries
    pub entries_count: usize,
    /// Hit rate
    pub hit_rate: f64,
    /// Last access time
    #[schemars(with = "String")]
    pub last_access: DateTime<Utc>,
}

/// CPU statistics for system monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuStatistics {
    pub average_usage: f64,
    pub peak_usage: f64,
    pub min_usage: f64,
    pub trend_slope: f64,
    pub volatility: f64,
}

/// CPU measurement data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuMeasurement {
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    pub usage: f64,
    pub core_id: Option<u32>,
}

/// Analytics-specific errors
#[derive(Debug, thiserror::Error)]
pub enum AnalyticsError {
    #[error("Model loading error: {0}")]
    ModelLoadError(String),
    #[error("Inference error: {0}")]
    InferenceError(String),
}
