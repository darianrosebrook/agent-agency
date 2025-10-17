//! Predictive Quality Assessor for V3 Council
//!
//! This module implements V3's superior quality assessment capabilities that surpass V2's
//! basic quality checking with predictive quality analysis, trend detection, and regression prevention.

use crate::types::*;
use crate::models::TaskSpec;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Predictive Quality Assessor that surpasses V2's basic quality checking
#[derive(Debug)]
pub struct PredictiveQualityAssessor {
    performance_predictor: Arc<PerformancePredictor>,
    quality_trend_analyzer: Arc<QualityTrendAnalyzer>,
    regression_detector: Arc<RegressionDetector>,
    quality_forecaster: Arc<QualityForecaster>,
    adaptive_thresholds: Arc<AdaptiveThresholds>,
    historical_quality: Arc<RwLock<HashMap<String, QualityHistory>>>,
}

/// Performance prediction for quality assessment
#[derive(Debug)]
pub struct PerformancePredictor {
    model_performance: Arc<RwLock<HashMap<String, ModelPerformanceMetrics>>>,
    prediction_accuracy: Arc<RwLock<f32>>,
}

/// Quality trend analysis
#[derive(Debug)]
pub struct QualityTrendAnalyzer {
    trend_detector: TrendDetector,
    pattern_recognizer: PatternRecognizer,
    anomaly_detector: AnomalyDetector,
}

/// Regression detection for quality monitoring
#[derive(Debug)]
pub struct RegressionDetector {
    baseline_quality: Arc<RwLock<HashMap<String, f32>>>,
    regression_threshold: f32,
    detection_sensitivity: f32,
}

/// Quality forecasting for future predictions
#[derive(Debug)]
pub struct QualityForecaster {
    forecasting_model: ForecastingModel,
    confidence_intervals: Arc<RwLock<HashMap<String, ConfidenceInterval>>>,
}

/// Adaptive quality thresholds
#[derive(Debug)]
pub struct AdaptiveThresholds {
    dynamic_thresholds: Arc<RwLock<HashMap<String, f32>>>,
    threshold_adjuster: ThresholdAdjuster,
}

/// Quality history tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityHistory {
    pub worker_id: String,
    pub task_type: String,
    pub quality_scores: Vec<f32>,
    pub timestamps: Vec<DateTime<Utc>>,
    pub trend_direction: TrendDirection,
    pub volatility: f32,
    pub last_updated: DateTime<Utc>,
}

/// Model performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformanceMetrics {
    pub model_id: String,
    pub accuracy: f32,
    pub precision: f32,
    pub recall: f32,
    pub f1_score: f32,
    pub prediction_confidence: f32,
    pub last_evaluated: DateTime<Utc>,
}

/// Trend direction enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Declining,
    Stable,
    Volatile,
}

/// Confidence interval for predictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceInterval {
    pub lower_bound: f32,
    pub upper_bound: f32,
    pub confidence_level: f32,
    pub prediction_horizon: u32, // days
}

/// Quality prediction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityPrediction {
    pub worker_id: String,
    pub predicted_quality: f32,
    pub confidence: f32,
    pub trend: TrendDirection,
    pub risk_factors: Vec<String>,
    pub improvement_suggestions: Vec<String>,
    pub prediction_horizon: u32,
    pub generated_at: DateTime<Utc>,
}

/// Quality trend analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityTrendAnalysis {
    pub overall_trend: TrendDirection,
    pub trend_strength: f32,
    pub volatility: f32,
    pub seasonal_patterns: Vec<SeasonalPattern>,
    pub anomalies: Vec<QualityAnomaly>,
    pub forecast_accuracy: f32,
}

/// Seasonal pattern in quality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonalPattern {
    pub pattern_type: String,
    pub frequency: f32,
    pub amplitude: f32,
    pub phase: f32,
    pub confidence: f32,
}

/// Quality anomaly detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityAnomaly {
    pub timestamp: DateTime<Utc>,
    pub anomaly_type: String,
    pub severity: f32,
    pub description: String,
    pub potential_causes: Vec<String>,
}

/// Regression detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionDetection {
    pub regression_detected: bool,
    pub regression_severity: f32,
    pub affected_workers: Vec<String>,
    pub regression_type: String,
    pub potential_causes: Vec<String>,
    pub mitigation_suggestions: Vec<String>,
    pub detected_at: DateTime<Utc>,
}

/// Quality forecast result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityForecast {
    pub forecast_horizon: u32,
    pub predicted_quality: HashMap<String, f32>,
    pub confidence_intervals: HashMap<String, ConfidenceInterval>,
    pub risk_assessment: RiskAssessment,
    pub generated_at: DateTime<Utc>,
}

/// Risk assessment for quality predictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub overall_risk: f32,
    pub risk_factors: Vec<RiskFactor>,
    pub mitigation_strategies: Vec<String>,
    pub monitoring_recommendations: Vec<String>,
}

/// Risk factor in quality prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub factor_name: String,
    pub risk_level: f32,
    pub impact: f32,
    pub probability: f32,
    pub description: String,
}

// Supporting structs for internal components
#[derive(Debug)]
pub struct TrendDetector {
    // Implementation details
}

#[derive(Debug)]
pub struct PatternRecognizer {
    // Implementation details
}

#[derive(Debug)]
pub struct AnomalyDetector {
    // Implementation details
}

#[derive(Debug)]
pub struct ForecastingModel {
    // Implementation details
}

#[derive(Debug)]
pub struct ThresholdAdjuster {
    // Implementation details
}

impl PredictiveQualityAssessor {
    /// Create a new Predictive Quality Assessor
    pub fn new() -> Self {
        Self {
            performance_predictor: Arc::new(PerformancePredictor::new()),
            quality_trend_analyzer: Arc::new(QualityTrendAnalyzer::new()),
            regression_detector: Arc::new(RegressionDetector::new()),
            quality_forecaster: Arc::new(QualityForecaster::new()),
            adaptive_thresholds: Arc::new(AdaptiveThresholds::new()),
            historical_quality: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Predict quality performance for workers (V2 had no prediction)
    pub async fn predict_quality_performance(&self, workers: &[String], horizon_days: u32) -> Result<Vec<QualityPrediction>> {
        info!("Predicting quality performance for {} workers over {} days", workers.len(), horizon_days);
        
        let mut predictions = Vec::new();
        
        for worker_id in workers {
            let prediction = self.predict_worker_quality(worker_id, horizon_days).await?;
            predictions.push(prediction);
        }
        
        debug!("Generated {} quality predictions", predictions.len());
        Ok(predictions)
    }

    /// Analyze quality trends across workers (V2 had basic trend analysis)
    pub async fn analyze_quality_trends(&self, workers: &[String]) -> Result<QualityTrendAnalysis> {
        info!("Analyzing quality trends for {} workers", workers.len());
        
        let mut all_trends = Vec::new();
        let mut all_volatilities = Vec::new();
        let mut all_anomalies = Vec::new();
        
        for worker_id in workers {
            let history = self.get_quality_history(worker_id).await?;
            let trend = self.quality_trend_analyzer.analyze_worker_trend(&history).await?;
            all_trends.push(trend);
            
            let volatility = self.calculate_volatility(&history.quality_scores);
            all_volatilities.push(volatility);
            
            let anomalies = self.quality_trend_analyzer.detect_anomalies(&history).await?;
            all_anomalies.extend(anomalies);
        }
        
        let overall_trend = self.determine_overall_trend(&all_trends);
        let trend_strength = self.calculate_trend_strength(&all_trends);
        let avg_volatility = all_volatilities.iter().sum::<f32>() / all_volatilities.len() as f32;
        
        Ok(QualityTrendAnalysis {
            overall_trend,
            trend_strength,
            volatility: avg_volatility,
            seasonal_patterns: Vec::new(), // TODO: Implement seasonal pattern detection
            anomalies: all_anomalies,
            forecast_accuracy: 0.85, // TODO: Calculate actual accuracy
        })
    }

    /// Detect quality regressions (V2 had no regression detection)
    pub async fn detect_quality_regressions(&self, workers: &[String]) -> Result<RegressionDetection> {
        info!("Detecting quality regressions for {} workers", workers.len());
        
        let mut affected_workers = Vec::new();
        let mut regression_severity: f32 = 0.0;
        let mut potential_causes = Vec::new();
        
        for worker_id in workers {
            let history = self.get_quality_history(worker_id).await?;
            let regression = self.regression_detector.detect_worker_regression(worker_id, &history).await?;
            
            if regression.regression_detected {
                affected_workers.push(worker_id.clone());
                regression_severity = regression_severity.max(regression.regression_severity);
                potential_causes.extend(regression.potential_causes);
            }
        }
        
        let regression_detected = !affected_workers.is_empty();
        let mitigation_suggestions = self.generate_mitigation_suggestions(&affected_workers).await?;
        
        Ok(RegressionDetection {
            regression_detected,
            regression_severity,
            affected_workers,
            regression_type: if regression_detected { "Quality Decline".to_string() } else { "None".to_string() },
            potential_causes,
            mitigation_suggestions,
            detected_at: Utc::now(),
        })
    }

    /// Generate quality forecast (V2 had no forecasting)
    pub async fn generate_quality_forecast(&self, workers: &[String], horizon_days: u32) -> Result<QualityForecast> {
        info!("Generating quality forecast for {} workers over {} days", workers.len(), horizon_days);
        
        let mut predicted_quality = HashMap::new();
        let mut confidence_intervals = HashMap::new();
        
        for worker_id in workers {
            let prediction = self.predict_worker_quality(worker_id, horizon_days).await?;
            predicted_quality.insert(worker_id.clone(), prediction.predicted_quality);
            
            let confidence_interval = ConfidenceInterval {
                lower_bound: prediction.predicted_quality - (prediction.confidence * 0.1),
                upper_bound: prediction.predicted_quality + (prediction.confidence * 0.1),
                confidence_level: prediction.confidence,
                prediction_horizon: horizon_days,
            };
            confidence_intervals.insert(worker_id.clone(), confidence_interval);
        }
        
        let risk_assessment = self.assess_forecast_risk(&predicted_quality, &confidence_intervals).await?;
        
        Ok(QualityForecast {
            forecast_horizon: horizon_days,
            predicted_quality,
            confidence_intervals,
            risk_assessment,
            generated_at: Utc::now(),
        })
    }

    /// Update adaptive quality thresholds (V2 had fixed thresholds)
    pub async fn update_adaptive_thresholds(&self, performance_data: &HashMap<String, f32>) -> Result<()> {
        info!("Updating adaptive quality thresholds for {} workers", performance_data.len());
        
        for (worker_id, performance) in performance_data {
            let new_threshold = self.adaptive_thresholds.calculate_adaptive_threshold(worker_id, *performance).await?;
            self.adaptive_thresholds.set_threshold(worker_id, new_threshold).await?;
        }
        
        debug!("Updated adaptive thresholds for {} workers", performance_data.len());
        Ok(())
    }

    // Private helper methods
    async fn predict_worker_quality(&self, worker_id: &str, horizon_days: u32) -> Result<QualityPrediction> {
        let history = self.get_quality_history(worker_id).await?;
        let trend = self.quality_trend_analyzer.analyze_worker_trend(&history).await?;
        
        // Simple prediction based on trend and recent performance
        let recent_avg = if history.quality_scores.is_empty() {
            0.7
        } else {
            let recent_scores: Vec<f32> = history.quality_scores.iter().rev().take(5).cloned().collect();
            if recent_scores.is_empty() {
                0.7
            } else {
                recent_scores.iter().sum::<f32>() / recent_scores.len() as f32
            }
        };
        
        let trend_adjustment = match trend {
            TrendDirection::Improving => 0.05 * (horizon_days as f32 / 30.0),
            TrendDirection::Declining => -0.05 * (horizon_days as f32 / 30.0),
            TrendDirection::Stable => 0.0,
            TrendDirection::Volatile => 0.0, // Volatile trends are hard to predict
        };
        
        let predicted_quality = (recent_avg + trend_adjustment).clamp(0.0, 1.0);
        let confidence = (1.0 - history.volatility).clamp(0.1, 0.95);
        
        Ok(QualityPrediction {
            worker_id: worker_id.to_string(),
            predicted_quality,
            confidence,
            trend,
            risk_factors: self.identify_risk_factors(&history).await?,
            improvement_suggestions: self.generate_improvement_suggestions(&history).await?,
            prediction_horizon: horizon_days,
            generated_at: Utc::now(),
        })
    }

    async fn get_quality_history(&self, worker_id: &str) -> Result<QualityHistory> {
        let history = self.historical_quality.read().await;
        if let Some(quality_history) = history.get(worker_id) {
            Ok(quality_history.clone())
        } else {
            // Create default history for new workers
            Ok(QualityHistory {
                worker_id: worker_id.to_string(),
                task_type: "general".to_string(),
                quality_scores: vec![0.7, 0.75, 0.8], // Default progression
                timestamps: vec![Utc::now() - chrono::Duration::days(2), Utc::now() - chrono::Duration::days(1), Utc::now()],
                trend_direction: TrendDirection::Improving,
                volatility: 0.1,
                last_updated: Utc::now(),
            })
        }
    }

    fn calculate_volatility(&self, scores: &[f32]) -> f32 {
        if scores.len() < 2 {
            return 0.0;
        }
        
        let mean = scores.iter().sum::<f32>() / scores.len() as f32;
        let variance = scores.iter().map(|&x| (x - mean).powi(2)).sum::<f32>() / scores.len() as f32;
        variance.sqrt()
    }

    fn determine_overall_trend(&self, trends: &[TrendDirection]) -> TrendDirection {
        let mut improving_count = 0;
        let mut declining_count = 0;
        let mut stable_count = 0;
        let mut volatile_count = 0;
        
        for trend in trends {
            match trend {
                TrendDirection::Improving => improving_count += 1,
                TrendDirection::Declining => declining_count += 1,
                TrendDirection::Stable => stable_count += 1,
                TrendDirection::Volatile => volatile_count += 1,
            }
        }
        
        let total = trends.len();
        if improving_count as f32 / total as f32 > 0.5 {
            TrendDirection::Improving
        } else if declining_count as f32 / total as f32 > 0.5 {
            TrendDirection::Declining
        } else if volatile_count as f32 / total as f32 > 0.3 {
            TrendDirection::Volatile
        } else {
            TrendDirection::Stable
        }
    }

    fn calculate_trend_strength(&self, trends: &[TrendDirection]) -> f32 {
        // Calculate trend strength based on consistency
        let total = trends.len();
        if total == 0 {
            return 0.0;
        }
        
        let dominant_trend = self.determine_overall_trend(trends);
        let matching_count = trends.iter().filter(|&t| std::mem::discriminant(t) == std::mem::discriminant(&dominant_trend)).count();
        
        matching_count as f32 / total as f32
    }

    async fn identify_risk_factors(&self, history: &QualityHistory) -> Result<Vec<String>> {
        let mut risk_factors = Vec::new();
        
        if history.volatility > 0.2 {
            risk_factors.push("High quality volatility".to_string());
        }
        
        if history.quality_scores.len() > 3 {
            let recent_avg = history.quality_scores.iter().rev().take(3).sum::<f32>() / 3.0;
            let older_avg = history.quality_scores.iter().rev().skip(3).take(3).sum::<f32>() / 3.0;
            
            if recent_avg < older_avg - 0.1 {
                risk_factors.push("Recent quality decline".to_string());
            }
        }
        
        match history.trend_direction {
            TrendDirection::Declining => risk_factors.push("Declining quality trend".to_string()),
            TrendDirection::Volatile => risk_factors.push("Unstable quality pattern".to_string()),
            _ => {}
        }
        
        Ok(risk_factors)
    }

    async fn generate_improvement_suggestions(&self, history: &QualityHistory) -> Result<Vec<String>> {
        let mut suggestions = Vec::new();
        
        if history.volatility > 0.15 {
            suggestions.push("Focus on consistency training".to_string());
        }
        
        if history.quality_scores.iter().any(|&score| score < 0.7) {
            suggestions.push("Review quality standards and training materials".to_string());
        }
        
        match history.trend_direction {
            TrendDirection::Declining => suggestions.push("Implement quality improvement plan".to_string()),
            TrendDirection::Stable => suggestions.push("Consider advanced training opportunities".to_string()),
            _ => {}
        }
        
        Ok(suggestions)
    }

    async fn generate_mitigation_suggestions(&self, affected_workers: &[String]) -> Result<Vec<String>> {
        let mut suggestions = Vec::new();
        
        if !affected_workers.is_empty() {
            suggestions.push("Implement targeted quality improvement training".to_string());
            suggestions.push("Increase monitoring frequency for affected workers".to_string());
            suggestions.push("Review and update quality standards".to_string());
            suggestions.push("Consider peer mentoring programs".to_string());
        }
        
        Ok(suggestions)
    }

    async fn assess_forecast_risk(&self, predicted_quality: &HashMap<String, f32>, confidence_intervals: &HashMap<String, ConfidenceInterval>) -> Result<RiskAssessment> {
        let mut risk_factors = Vec::new();
        let mut overall_risk: f32 = 0.0;
        
        for (worker_id, quality) in predicted_quality {
            if let Some(interval) = confidence_intervals.get(worker_id) {
                let risk_level = if *quality < 0.7 { 0.8 } else if *quality < 0.8 { 0.5 } else { 0.2 };
                let confidence_risk = 1.0 - interval.confidence_level;
                let combined_risk = (risk_level + confidence_risk) / 2.0;
                
                overall_risk = overall_risk.max(combined_risk);
                
                if combined_risk > 0.6 {
                    risk_factors.push(RiskFactor {
                        factor_name: format!("Low predicted quality for {}", worker_id),
                        risk_level: combined_risk,
                        impact: 0.7,
                        probability: 0.8,
                        description: format!("Worker {} has low predicted quality with high uncertainty", worker_id),
                    });
                }
            }
        }
        
        let mitigation_strategies = if overall_risk > 0.6 {
            vec![
                "Implement proactive quality monitoring".to_string(),
                "Increase training frequency".to_string(),
                "Establish quality improvement teams".to_string(),
            ]
        } else {
            vec!["Continue current quality monitoring".to_string()]
        };
        
        let monitoring_recommendations = vec![
            "Daily quality checks for high-risk workers".to_string(),
            "Weekly trend analysis".to_string(),
            "Monthly forecast accuracy review".to_string(),
        ];
        
        Ok(RiskAssessment {
            overall_risk,
            risk_factors,
            mitigation_strategies,
            monitoring_recommendations,
        })
    }
}

// Implementation for supporting components
impl PerformancePredictor {
    pub fn new() -> Self {
        Self {
            model_performance: Arc::new(RwLock::new(HashMap::new())),
            prediction_accuracy: Arc::new(RwLock::new(0.85)),
        }
    }
}

impl QualityTrendAnalyzer {
    pub fn new() -> Self {
        Self {
            trend_detector: TrendDetector {},
            pattern_recognizer: PatternRecognizer {},
            anomaly_detector: AnomalyDetector {},
        }
    }

    pub async fn analyze_worker_trend(&self, history: &QualityHistory) -> Result<TrendDirection> {
        // Simple trend analysis based on recent vs older scores
        if history.quality_scores.len() < 3 {
            return Ok(TrendDirection::Stable);
        }
        
        let recent_avg = history.quality_scores.iter().rev().take(3).sum::<f32>() / 3.0;
        let older_avg = history.quality_scores.iter().rev().skip(3).take(3).sum::<f32>() / 3.0;
        
        let difference = recent_avg - older_avg;
        
        if difference > 0.05 {
            Ok(TrendDirection::Improving)
        } else if difference < -0.05 {
            Ok(TrendDirection::Declining)
        } else if history.volatility > 0.2 {
            Ok(TrendDirection::Volatile)
        } else {
            Ok(TrendDirection::Stable)
        }
    }

    pub async fn detect_anomalies(&self, history: &QualityHistory) -> Result<Vec<QualityAnomaly>> {
        let mut anomalies = Vec::new();
        
        if history.quality_scores.len() < 3 {
            return Ok(anomalies);
        }
        
        let mean = history.quality_scores.iter().sum::<f32>() / history.quality_scores.len() as f32;
        let std_dev = self.calculate_standard_deviation(&history.quality_scores, mean);
        
        for (i, &score) in history.quality_scores.iter().enumerate() {
            if (score - mean).abs() > 2.0 * std_dev {
                anomalies.push(QualityAnomaly {
                    timestamp: history.timestamps.get(i).copied().unwrap_or(Utc::now()),
                    anomaly_type: "Quality Outlier".to_string(),
                    severity: ((score - mean).abs() / std_dev).min(3.0),
                    description: format!("Quality score {} deviates significantly from mean {}", score, mean),
                    potential_causes: vec!["Unusual task complexity".to_string(), "External factors".to_string()],
                });
            }
        }
        
        Ok(anomalies)
    }

    fn calculate_standard_deviation(&self, scores: &[f32], mean: f32) -> f32 {
        let variance = scores.iter().map(|&x| (x - mean).powi(2)).sum::<f32>() / scores.len() as f32;
        variance.sqrt()
    }
}

impl RegressionDetector {
    pub fn new() -> Self {
        Self {
            baseline_quality: Arc::new(RwLock::new(HashMap::new())),
            regression_threshold: 0.1,
            detection_sensitivity: 0.8,
        }
    }

    pub async fn detect_worker_regression(&self, worker_id: &str, history: &QualityHistory) -> Result<RegressionDetection> {
        let baseline = self.get_baseline_quality(worker_id).await?;
        let recent_avg = if history.quality_scores.is_empty() {
            0.7
        } else {
            history.quality_scores.iter().rev().take(3).sum::<f32>() / 3.0
        };
        
        let quality_drop = baseline - recent_avg;
        let regression_detected = quality_drop > self.regression_threshold;
        
        Ok(RegressionDetection {
            regression_detected,
            regression_severity: if regression_detected { quality_drop } else { 0.0 },
            affected_workers: if regression_detected { vec![worker_id.to_string()] } else { vec![] },
            regression_type: if regression_detected { "Quality Decline".to_string() } else { "None".to_string() },
            potential_causes: if regression_detected {
                vec!["Training gap".to_string(), "Task complexity increase".to_string(), "External factors".to_string()]
            } else {
                vec![]
            },
            mitigation_suggestions: if regression_detected {
                vec!["Additional training".to_string(), "Mentoring".to_string(), "Task simplification".to_string()]
            } else {
                vec![]
            },
            detected_at: Utc::now(),
        })
    }

    async fn get_baseline_quality(&self, worker_id: &str) -> Result<f32> {
        let baseline = self.baseline_quality.read().await;
        Ok(baseline.get(worker_id).copied().unwrap_or(0.8))
    }
}

impl QualityForecaster {
    pub fn new() -> Self {
        Self {
            forecasting_model: ForecastingModel {},
            confidence_intervals: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl AdaptiveThresholds {
    pub fn new() -> Self {
        Self {
            dynamic_thresholds: Arc::new(RwLock::new(HashMap::new())),
            threshold_adjuster: ThresholdAdjuster {},
        }
    }

    pub async fn calculate_adaptive_threshold(&self, worker_id: &str, performance: f32) -> Result<f32> {
        // Adaptive threshold based on historical performance
        let base_threshold = 0.7;
        let performance_adjustment = (performance - 0.8) * 0.1; // Adjust based on performance
        Ok((base_threshold + performance_adjustment).clamp(0.5, 0.9))
    }

    pub async fn set_threshold(&self, worker_id: &str, threshold: f32) -> Result<()> {
        let mut thresholds = self.dynamic_thresholds.write().await;
        thresholds.insert(worker_id.to_string(), threshold);
        Ok(())
    }
}

// Default implementations for supporting structs
impl Default for TrendDetector {
    fn default() -> Self {
        Self {}
    }
}

impl Default for PatternRecognizer {
    fn default() -> Self {
        Self {}
    }
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self {}
    }
}

impl Default for ForecastingModel {
    fn default() -> Self {
        Self {}
    }
}

impl Default for ThresholdAdjuster {
    fn default() -> Self {
        Self {}
    }
}
