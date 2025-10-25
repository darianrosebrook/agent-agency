//! Configuration structures for analytics dashboard

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Analytics dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsDashboardConfig {
    /// Dashboard refresh interval in seconds
    pub refresh_interval_seconds: u64,
    /// Maximum number of concurrent sessions
    pub max_sessions: usize,
    /// Enable real-time analytics updates
    pub enable_real_time_updates: bool,
    /// Analytics data retention in hours
    pub data_retention_hours: u64,
    /// Enable trend analysis
    pub enable_trend_analysis: bool,
    /// Enable anomaly detection
    pub enable_anomaly_detection: bool,
    /// Enable predictive analytics
    pub enable_predictive_analytics: bool,
}

impl Default for AnalyticsDashboardConfig {
    fn default() -> Self {
        Self {
            refresh_interval_seconds: 30,
            max_sessions: 50,
            enable_real_time_updates: true,
            data_retention_hours: 168, // 1 week
            enable_trend_analysis: true,
            enable_anomaly_detection: true,
            enable_predictive_analytics: true,
        }
    }
}

/// Analytics session
#[derive(Debug, Clone)]
pub struct AnalyticsSession {
    /// Session ID
    pub session_id: String,
    /// User ID
    pub user_id: Option<String>,
    /// Session start time
    pub start_time: DateTime<Utc>,
    /// Last activity time
    pub last_activity: DateTime<Utc>,
    /// Session preferences
    pub preferences: AnalyticsPreferences,
    /// Active subscriptions
    pub subscriptions: Vec<AnalyticsSubscriptionType>,
}

/// Analytics preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsPreferences {
    /// Time range for analysis
    pub time_range_hours: u64,
    /// Enable trend analysis
    pub show_trends: bool,
    /// Enable anomaly detection
    pub show_anomalies: bool,
    /// Enable predictions
    pub show_predictions: bool,
    /// Enable optimization recommendations
    pub show_optimizations: bool,
    /// Alert preferences
    pub alert_preferences: AnalyticsAlertPreferences,
}

/// Analytics alert preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsAlertPreferences {
    /// Enable trend alerts
    pub enable_trend_alerts: bool,
    /// Enable anomaly alerts
    pub enable_anomaly_alerts: bool,
    /// Enable prediction alerts
    pub enable_prediction_alerts: bool,
    /// Alert sensitivity
    pub alert_sensitivity: f64,
}

impl Default for AnalyticsPreferences {
    fn default() -> Self {
        Self {
            time_range_hours: 24,
            show_trends: true,
            show_anomalies: true,
            show_predictions: true,
            show_optimizations: true,
            alert_preferences: AnalyticsAlertPreferences {
                enable_trend_alerts: true,
                enable_anomaly_alerts: true,
                enable_prediction_alerts: true,
                alert_sensitivity: 0.7,
            },
        }
    }
}

/// Analytics subscription types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalyticsSubscriptionType {
    /// Trend analysis updates
    TrendAnalysis,
    /// Anomaly detection updates
    AnomalyDetection,
    /// Predictive analytics updates
    PredictiveAnalytics,
    /// Optimization recommendations
    OptimizationRecommendations,
    /// All analytics updates
    All,
}
