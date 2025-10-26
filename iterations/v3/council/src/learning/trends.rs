//! Trend analysis and performance monitoring
//!
//! This module provides trend analysis, performance monitoring,
//! and predictive analytics capabilities.

use anyhow::Result;
use chrono::{DateTime, Utc, Duration as ChronoDuration};
use std::collections::HashMap;

use super::types::*;
use super::storage::*;

/// Trend analyzer for performance and resource usage patterns
pub struct TrendAnalyzer {
    storage: Box<dyn LearningSignalStorage>,
}

impl TrendAnalyzer {
    /// Create a new trend analyzer
    pub fn new(storage: Box<dyn LearningSignalStorage>) -> Self {
        Self { storage }
    }

    /// Analyze performance trends over time
    pub async fn analyze_performance_trends(
        &self,
        entity_type: PerformanceEntityType,
        entity_id: String,
        time_window: TimeWindow,
    ) -> Result<PerformanceTrends> {
        // Get metrics for the time window
        let metrics = self.storage.get_performance_metrics(
            entity_type.clone(),
            entity_id.clone(),
            time_window,
        ).await?;

        // Analyze trends in the metrics
        let quality_trend = self.analyze_metric_trend(&metrics, "quality")?;
        let latency_trend = self.analyze_metric_trend(&metrics, "latency")?;
        let dissent_trend = self.analyze_metric_trend(&metrics, "dissent")?;
        let resource_efficiency_trend = self.analyze_metric_trend(&metrics, "resource_efficiency")?;

        // Generate recommendations based on trends
        let recommendations = self.generate_trend_based_recommendations(
            quality_trend,
            latency_trend,
            dissent_trend,
            resource_efficiency_trend,
            &entity_type,
            &entity_id,
        )?;

        Ok(PerformanceTrends {
            trends: vec![
                TrendAnalysis {
                    trend_type: crate::council_types::TrendType::Quality,
                    direction: quality_trend,
                    magnitude: 0.1, // Simplified magnitude calculation
                    confidence: 0.8,
                    time_window,
                },
                TrendAnalysis {
                    trend_type: crate::council_types::TrendType::Latency,
                    direction: latency_trend,
                    magnitude: 0.05,
                    confidence: 0.75,
                    time_window,
                },
                TrendAnalysis {
                    trend_type: crate::council_types::TrendType::Dissent,
                    direction: dissent_trend,
                    magnitude: 0.02,
                    confidence: 0.7,
                    time_window,
                },
                TrendAnalysis {
                    trend_type: crate::council_types::TrendType::ResourceEfficiency,
                    direction: resource_efficiency_trend,
                    magnitude: 0.08,
                    confidence: 0.8,
                    time_window,
                },
            ],
            overall_direction: self.calculate_overall_trend_direction(
                quality_trend,
                latency_trend,
                dissent_trend,
                resource_efficiency_trend,
            ),
            recommendations,
        })
    }

    /// Analyze trend for a specific metric
    fn analyze_metric_trend(&self, metrics: &AggregatedMetrics, metric_type: &str) -> Result<TrendDirection> {
        // Simplified trend analysis - in practice, this would use statistical methods
        // For now, we'll use simple heuristics based on recent performance

        match metric_type {
            "quality" => {
                if metrics.average_quality_score > 0.8 {
                    Ok(TrendDirection::Improving)
                } else if metrics.average_quality_score < 0.6 {
                    Ok(TrendDirection::Declining)
                } else {
                    Ok(TrendDirection::Stable)
                }
            },
            "latency" => {
                if metrics.average_latency_ms < 1000.0 {
                    Ok(TrendDirection::Improving)
                } else if metrics.average_latency_ms > 5000.0 {
                    Ok(TrendDirection::Declining)
                } else {
                    Ok(TrendDirection::Stable)
                }
            },
            "dissent" => {
                if metrics.dissent_rate < 0.1 {
                    Ok(TrendDirection::Improving)
                } else if metrics.dissent_rate > 0.3 {
                    Ok(TrendDirection::Declining)
                } else {
                    Ok(TrendDirection::Stable)
                }
            },
            "resource_efficiency" => {
                if metrics.resource_efficiency > 0.8 {
                    Ok(TrendDirection::Improving)
                } else if metrics.resource_efficiency < 0.5 {
                    Ok(TrendDirection::Declining)
                } else {
                    Ok(TrendDirection::Stable)
                }
            },
            _ => Ok(TrendDirection::Stable),
        }
    }

    /// Calculate overall trend direction from individual trends
    fn calculate_overall_trend_direction(
        &self,
        quality_trend: TrendDirection,
        latency_trend: TrendDirection,
        dissent_trend: TrendDirection,
        resource_efficiency_trend: TrendDirection,
    ) -> TrendDirection {
        let trends = vec![quality_trend, latency_trend, dissent_trend, resource_efficiency_trend];

        let improving_count = trends.iter().filter(|t| matches!(t, TrendDirection::Improving)).count();
        let declining_count = trends.iter().filter(|t| matches!(t, TrendDirection::Declining)).count();

        if improving_count > declining_count {
            TrendDirection::Improving
        } else if declining_count > improving_count {
            TrendDirection::Declining
        } else {
            TrendDirection::Stable
        }
    }

    /// Generate recommendations based on trend analysis
    fn generate_trend_based_recommendations(
        &self,
        quality_trend: TrendDirection,
        latency_trend: TrendDirection,
        dissent_trend: TrendDirection,
        resource_efficiency_trend: TrendDirection,
        entity_type: &PerformanceEntityType,
        entity_id: &str,
    ) -> Result<Vec<LearningRecommendation>> {
        let mut recommendations = Vec::new();

        // Quality trend recommendations
        match quality_trend {
            TrendDirection::Declining => {
                recommendations.push(LearningRecommendation {
                    recommendation_type: RecommendationType::JudgeAssignment,
                    priority: RecommendationPriority::High,
                    description: format!("Quality declining for {} - consider retraining or replacement", entity_id),
                    expected_impact: 0.3,
                    implementation_effort: EffortLevel::Moderate,
                });
            },
            TrendDirection::Improving => {
                recommendations.push(LearningRecommendation {
                    recommendation_type: RecommendationType::SystemOptimization,
                    priority: RecommendationPriority::Low,
                    description: format!("Quality improving for {} - continue current practices", entity_id),
                    expected_impact: 0.1,
                    implementation_effort: EffortLevel::Trivial,
                });
            },
            TrendDirection::Stable => {} // No recommendation needed for stable quality
        }

        // Latency trend recommendations
        if matches!(latency_trend, TrendDirection::Declining) {
            recommendations.push(LearningRecommendation {
                recommendation_type: RecommendationType::ResourceAllocation,
                priority: RecommendationPriority::Medium,
                description: format!("Latency increasing for {} - consider resource optimization", entity_id),
                expected_impact: 0.25,
                implementation_effort: EffortLevel::Simple,
            });
        }

        // Dissent trend recommendations
        if matches!(dissent_trend, TrendDirection::Declining) {
            recommendations.push(LearningRecommendation {
                recommendation_type: RecommendationType::TaskPrioritization,
                priority: RecommendationPriority::Medium,
                description: format!("Increasing dissent for {} - review consensus requirements", entity_id),
                expected_impact: 0.2,
                implementation_effort: EffortLevel::Moderate,
            });
        }

        // Resource efficiency recommendations
        if matches!(resource_efficiency_trend, TrendDirection::Declining) {
            recommendations.push(LearningRecommendation {
                recommendation_type: RecommendationType::SystemOptimization,
                priority: RecommendationPriority::Medium,
                description: format!("Resource efficiency declining for {} - optimize resource usage", entity_id),
                expected_impact: 0.15,
                implementation_effort: EffortLevel::Complex,
            });
        }

        Ok(recommendations)
    }

    /// Monitor resource usage trends and detect anomalies
    pub async fn monitor_resource_trends(
        &self,
        task_spec: &crate::council_types::TaskSpec,
    ) -> Result<ResourceTrendAnalysis> {
        // Get historical resource data
        let historical_data = self.storage.perform_comprehensive_historical_resource_lookup(task_spec).await?;

        if historical_data.entries.is_empty() {
            return Ok(ResourceTrendAnalysis {
                trends: vec![],
                anomalies: vec![],
                predictions: vec![],
                recommendations: vec![],
            });
        }

        // Analyze resource usage trends
        let resource_trends = self.storage.analyze_resource_usage_trends(&historical_data).await?;

        // Generate resource predictions
        let predictions = self.storage.generate_resource_usage_predictions(&historical_data, &resource_trends).await?;

        // Detect anomalies (simplified - would use statistical methods in practice)
        let anomalies = self.detect_resource_anomalies(&historical_data)?;

        // Generate recommendations based on trends
        let recommendations = self.generate_resource_recommendations(&resource_trends, &predictions)?;

        Ok(ResourceTrendAnalysis {
            trends: resource_trends,
            anomalies,
            predictions,
            recommendations,
        })
    }

    /// Detect resource usage anomalies
    fn detect_resource_anomalies(&self, historical_data: &HistoricalResourceData) -> Result<Vec<String>> {
        let mut anomalies = Vec::new();

        if historical_data.entries.len() < 5 {
            return Ok(anomalies);
        }

        // Simple anomaly detection - flag unusual resource usage
        let avg_cpu = historical_data.entries.iter()
            .map(|e| e.resource_usage.cpu_percent)
            .sum::<f32>() / historical_data.entries.len() as f32;

        let avg_memory = historical_data.entries.iter()
            .map(|e| e.resource_usage.memory_mb)
            .sum::<f32>() / historical_data.entries.len() as f32;

        // Check recent entries for anomalies
        for entry in historical_data.entries.iter().rev().take(3) {
            if entry.resource_usage.cpu_percent > avg_cpu * 1.5 {
                anomalies.push(format!(
                    "High CPU usage detected: {:.1}% (avg: {:.1}%) at {}",
                    entry.resource_usage.cpu_percent,
                    avg_cpu,
                    entry.timestamp
                ));
            }

            if entry.resource_usage.memory_mb as f32 > avg_memory * 1.5 {
                anomalies.push(format!(
                    "High memory usage detected: {}MB (avg: {:.0}MB) at {}",
                    entry.resource_usage.memory_mb,
                    avg_memory,
                    entry.timestamp
                ));
            }
        }

        Ok(anomalies)
    }

    /// Generate resource recommendations based on trends and predictions
    fn generate_resource_recommendations(
        &self,
        trends: &[crate::council_types::ResourceTrend],
        predictions: &[ResourcePrediction],
    ) -> Result<Vec<String>> {
        let mut recommendations = Vec::new();

        // Analyze trends for recommendations
        for trend in trends {
            match trend.trend_type {
                crate::council_types::TrendType::Increasing => {
                    recommendations.push(format!(
                        "Resource usage increasing - consider scaling resources for {}",
                        trend.resource_type
                    ));
                },
                crate::council_types::TrendType::Decreasing => {
                    recommendations.push(format!(
                        "Resource usage decreasing - potential for optimization in {}",
                        trend.resource_type
                    ));
                },
                crate::council_types::TrendType::Stable => {
                    recommendations.push(format!(
                        "Resource usage stable for {} - maintain current allocation",
                        trend.resource_type
                    ));
                },
            }
        }

        // Add prediction-based recommendations
        for prediction in predictions {
            if prediction.predicted_usage_percent > 80.0 {
                recommendations.push(format!(
                    "High predicted usage for {} ({:.1}%) - prepare additional resources",
                    prediction.resource_type,
                    prediction.predicted_usage_percent
                ));
            }
        }

        Ok(recommendations)
    }

    /// Calculate performance metrics quality score
    pub fn calculate_performance_quality_score(&self, metrics: &AggregatedMetrics) -> f32 {
        let quality_weight = 0.4;
        let latency_weight = 0.3;
        let dissent_weight = 0.2;
        let efficiency_weight = 0.1;

        let quality_score = metrics.average_quality_score;
        let latency_score = if metrics.average_latency_ms < 1000.0 {
            1.0
        } else if metrics.average_latency_ms < 5000.0 {
            0.5
        } else {
            0.0
        };
        let dissent_score = 1.0 - metrics.dissent_rate.min(1.0);
        let efficiency_score = metrics.resource_efficiency;

        quality_score * quality_weight +
        latency_score * latency_weight +
        dissent_score * dissent_weight +
        efficiency_score * efficiency_weight
    }
}

/// Resource trend analysis results
#[derive(Debug, Clone)]
pub struct ResourceTrendAnalysis {
    pub trends: Vec<crate::council_types::ResourceTrend>,
    pub anomalies: Vec<String>,
    pub predictions: Vec<ResourcePrediction>,
    pub recommendations: Vec<String>,
}
