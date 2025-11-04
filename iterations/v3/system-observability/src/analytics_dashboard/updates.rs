//! Real-time update structures for analytics dashboard

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::data::{
    AnalyticsInsight, AnomalyDetectionResult, OptimizationRecommendation,
    PredictiveModelResult, TrendAnalysis,
};

/// Analytics real-time update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsRealTimeUpdate {
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    pub trend_updates: Option<TrendUpdates>,
    pub anomaly_updates: Option<AnomalyUpdates>,
    pub prediction_updates: Option<PredictionUpdates>,
    pub optimization_updates: Option<OptimizationUpdates>,
}

/// Trend updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendUpdates {
    pub new_trends: Vec<TrendAnalysis>,
    pub updated_trends: Vec<TrendAnalysis>,
    pub trend_alerts: Vec<AnalyticsInsight>,
}

/// Anomaly updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyUpdates {
    pub new_anomalies: Vec<AnomalyDetectionResult>,
    pub resolved_anomalies: Vec<AnomalyDetectionResult>,
    pub anomaly_alerts: Vec<AnalyticsInsight>,
}

/// Prediction updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionUpdates {
    pub new_predictions: Vec<PredictiveModelResult>,
    pub updated_predictions: Vec<PredictiveModelResult>,
    pub prediction_alerts: Vec<AnalyticsInsight>,
}

/// Optimization updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationUpdates {
    pub new_recommendations: Vec<OptimizationRecommendation>,
    pub updated_recommendations: Vec<OptimizationRecommendation>,
    pub optimization_alerts: Vec<AnalyticsInsight>,
}
