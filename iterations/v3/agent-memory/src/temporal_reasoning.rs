//! Temporal Reasoning Engine - Time-based analysis and causality detection

use crate::types::*;
use crate::MemoryResult;
use agent_agency_database::{DatabaseClient, DatabaseConfig, Row};
use std::sync::Arc;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use tracing::{info, debug};

/// Temporal reasoning engine for time-based memory analysis
#[derive(Debug)]
pub struct TemporalReasoningEngine {
    db_client: Arc<DatabaseClient>,
    config: TemporalConfig,
}

impl TemporalReasoningEngine {
    /// Create a new temporal reasoning engine
    pub async fn new(config: &TemporalConfig) -> MemoryResult<Self> {
        let db_config = agent_agency_database::DatabaseConfig::default();
        let db_client = Arc::new(DatabaseClient::new(db_config).await?);

        Ok(Self {
            db_client,
            config: config.clone(),
        })
    }

    /// Analyze performance patterns for an agent over time
    pub async fn analyze_agent_performance(&self, agent_id: &str, time_range: &TimeRange) -> MemoryResult<TemporalAnalysis> {
        // Get performance metrics over time
        let performance_data = sqlx::query(
            r#"
            SELECT
                DATE_TRUNC('day', timestamp) as day,
                AVG((outcome->>'performance_score')::float) as avg_performance,
                COUNT(*) as experience_count,
                AVG((outcome->>'execution_time_ms')::float) as avg_execution_time,
                COUNT(CASE WHEN outcome->>'success' = 'true' THEN 1 END) * 100.0 / COUNT(*) as success_rate
            FROM agent_experiences
            WHERE agent_id = $1
              AND timestamp BETWEEN $2 AND $3
            GROUP BY DATE_TRUNC('day', timestamp)
            ORDER BY day
            "#,
        )
        .bind(agent_id)
        .bind(time_range.start)
        .bind(time_range.end)
        .fetch_all(self.db_client.pool())
        .await?;

        let mut trends = Vec::new();
        let mut performance_values = Vec::new();

        for row in &performance_data {
            let day: DateTime<Utc> = row.try_get("day")?;
            let avg_performance: Option<f64> = row.try_get("avg_performance")?;
            let success_rate: Option<f64> = row.try_get("success_rate")?;

            if let (Some(perf), Some(success)) = (avg_performance, success_rate) {
                performance_values.push((day, perf as f32, success as f32));
            }
        }

        // Analyze trends
        if performance_values.len() >= 3 {
            let recent_performance: Vec<f32> = performance_values.iter()
                .rev()
                .take(7)  // Last 7 days
                .map(|(_, perf, _)| *perf)
                .collect();

            let trend = self.calculate_trend(&recent_performance);
            trends.push(TemporalTrend {
                metric: "performance_score".to_string(),
                direction: trend,
                magnitude: self.calculate_trend_magnitude(&recent_performance),
                confidence: 0.8,
                time_range: time_range.clone(),
            });
        }

        // Detect change points
        let change_points = self.detect_performance_change_points(&performance_values).await?;

        // Find causality links
        let causality_links = self.detect_performance_causality(agent_id, time_range).await?;

        // Calculate performance summary
        let summary = self.calculate_performance_summary(&performance_values);

        Ok(TemporalAnalysis {
            time_range: time_range.clone(),
            trends,
            change_points,
            causality_links,
            performance_summary: summary,
        })
    }

    /// Calculate trend direction from a series of values
    fn calculate_trend(&self, values: &[f32]) -> TrendDirection {
        if values.len() < 2 {
            return TrendDirection::Stable;
        }

        let first_half: f32 = values.iter().take(values.len() / 2).sum::<f32>() / (values.len() / 2) as f32;
        let second_half: f32 = values.iter().rev().take(values.len() / 2).sum::<f32>() / (values.len() / 2) as f32;

        let change = second_half - first_half;
        let threshold = first_half * 0.1; // 10% change threshold

        if change > threshold {
            TrendDirection::Improving
        } else if change < -threshold {
            TrendDirection::Declining
        } else {
            TrendDirection::Stable
        }
    }

    /// Calculate trend magnitude
    fn calculate_trend_magnitude(&self, values: &[f32]) -> f32 {
        if values.len() < 2 {
            return 0.0;
        }

        let first = values[0];
        let last = values[values.len() - 1];
        let change = (last - first) / first;

        change.abs()
    }

    /// Detect significant change points in performance data
    async fn detect_performance_change_points(&self, performance_data: &[(DateTime<Utc>, f32, f32)]) -> MemoryResult<Vec<ChangePoint>> {
        let mut change_points = Vec::new();

        if performance_data.len() < 5 {
            return Ok(change_points);
        }

        // Simple change point detection using moving averages
        let window_size = 3;
        for i in window_size..performance_data.len().saturating_sub(window_size) {
            let before_window: Vec<f32> = performance_data[i-window_size..i].iter().map(|(_, perf, _)| *perf).collect();
            let after_window: Vec<f32> = performance_data[i..i+window_size].iter().map(|(_, perf, _)| *perf).collect();

            let before_avg = before_window.iter().sum::<f32>() / before_window.len() as f32;
            let after_avg = after_window.iter().sum::<f32>() / after_window.len() as f32;

            let change_magnitude = (after_avg - before_avg).abs() / before_avg.max(after_avg);

            if change_magnitude > self.config.change_point_sensitivity {
                let (timestamp, performance, _) = performance_data[i];
                change_points.push(ChangePoint {
                    timestamp,
                    metric: "performance_score".to_string(),
                    change_magnitude,
                    confidence: 0.7,
                    description: format!("Performance changed by {:.1}% around {}", change_magnitude * 100.0, timestamp),
                });
            }
        }

        Ok(change_points)
    }

    /// Detect causality relationships in performance data
    async fn detect_performance_causality(&self, agent_id: &str, time_range: &TimeRange) -> MemoryResult<Vec<CausalityLink>> {
        let mut causality_links = Vec::new();

        // Look for correlations between task types and outcomes
        let correlations = sqlx::query(
            r#"
            SELECT
                context->>'task_type' as task_type,
                AVG((outcome->>'performance_score')::float) as avg_performance,
                COUNT(*) as experience_count,
                STDDEV((outcome->>'performance_score')::float) as performance_stddev
            FROM agent_experiences
            WHERE agent_id = $1
              AND timestamp BETWEEN $2 AND $3
              AND outcome->>'performance_score' IS NOT NULL
            GROUP BY context->>'task_type'
            HAVING COUNT(*) >= 3
            ORDER BY avg_performance DESC
            "#,
        )
        .bind(agent_id)
        .bind(time_range.start)
        .bind(time_range.end)
        .fetch_all(self.db_client.pool())
        .await?;

        for row in correlations {
            let task_type: String = row.try_get("task_type")?;
            let avg_performance: Option<f64> = row.try_get("avg_performance")?;
            let count: i64 = row.try_get("experience_count")?;
            let stddev: Option<f64> = row.try_get("performance_stddev")?;

            if let (Some(avg_perf), Some(stddev_val)) = (avg_performance, stddev) {
                // Look for capability learning patterns
                let capability_growth = sqlx::query(
                    r#"
                    SELECT COUNT(*) as learned_count
                    FROM agent_experiences
                    WHERE agent_id = $1
                      AND context->>'task_type' = $2
                      AND jsonb_array_length(outcome->'learned_capabilities') > 0
                    "#,
                )
                .bind(agent_id)
                .bind(&task_type)
                .fetch_one(self.db_client.pool())
                .await?;

                let learned_count: i64 = capability_growth.try_get("learned_count")?;

                if learned_count > 0 && avg_perf > 0.7 && stddev_val < 0.2 {
                    // Strong correlation between task type and consistent high performance with learning
                    causality_links.push(CausalityLink {
                        cause_event: format!("performing_{}", task_type.to_lowercase().replace(" ", "_")),
                        effect_event: "high_performance_with_learning".to_string(),
                        confidence: (avg_perf as f32).min(0.9),
                        time_delay_ms: None,
                        supporting_evidence: vec![
                            format!("{} experiences", count),
                            format!("{:.2} avg performance", avg_perf),
                            format!("{} learning events", learned_count),
                        ],
                    });
                }
            }
        }

        Ok(causality_links)
    }

    /// Calculate performance summary statistics
    fn calculate_performance_summary(&self, performance_data: &[(DateTime<Utc>, f32, f32)]) -> PerformanceSummary {
        if performance_data.is_empty() {
            return PerformanceSummary {
                average_score: 0.0,
                best_score: 0.0,
                worst_score: 0.0,
                improvement_rate: 0.0,
                consistency_score: 0.0,
                total_samples: 0,
            };
        }

        let scores: Vec<f32> = performance_data.iter().map(|(_, score, _)| *score).collect();
        let avg_score = scores.iter().sum::<f32>() / scores.len() as f32;
        let best_score = scores.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        let worst_score = scores.iter().fold(f32::INFINITY, |a, &b| a.min(b));

        // Calculate improvement rate (linear trend)
        let improvement_rate = if scores.len() > 1 {
            let n = scores.len() as f32;
            let x_sum: f32 = (0..scores.len()).map(|i| i as f32).sum();
            let y_sum: f32 = scores.iter().sum();
            let xy_sum: f32 = scores.iter().enumerate().map(|(i, &y)| i as f32 * y).sum();
            let x_squared_sum: f32 = (0..scores.len()).map(|i| (i as f32).powi(2)).sum();

            let slope = (n * xy_sum - x_sum * y_sum) / (n * x_squared_sum - x_sum.powi(2));
            slope / avg_score // Normalized improvement rate
        } else {
            0.0
        };

        // Calculate consistency (inverse of coefficient of variation)
        let variance = scores.iter().map(|s| (s - avg_score).powi(2)).sum::<f32>() / scores.len() as f32;
        let std_dev = variance.sqrt();
        let consistency_score = if avg_score > 0.0 { 1.0 - (std_dev / avg_score).min(1.0) } else { 0.0 };

        PerformanceSummary {
            average_score: avg_score,
            best_score,
            worst_score,
            improvement_rate,
            consistency_score,
            total_samples: scores.len(),
        }
    }

    /// Analyze capability evolution over time
    pub async fn analyze_capability_evolution(&self, agent_id: &str, time_range: &TimeRange) -> MemoryResult<Vec<CapabilityEvolution>> {
        let capabilities_over_time = sqlx::query(
            r#"
            SELECT
                DATE_TRUNC('week', timestamp) as week,
                jsonb_array_elements_text(outcome->'learned_capabilities') as capability,
                COUNT(*) as learning_events,
                AVG((outcome->>'performance_score')::float) as avg_performance
            FROM agent_experiences
            WHERE agent_id = $1
              AND timestamp BETWEEN $2 AND $3
              AND jsonb_array_length(outcome->'learned_capabilities') > 0
            GROUP BY DATE_TRUNC('week', timestamp), capability
            ORDER BY week, capability
            "#,
        )
        .bind(agent_id)
        .bind(time_range.start)
        .bind(time_range.end)
        .fetch_all(self.db_client.pool())
        .await?;

        let mut capability_evolution = Vec::new();
        let mut capability_timeline: std::collections::HashMap<String, Vec<(DateTime<Utc>, i64, f64)>> = std::collections::HashMap::new();

        // Group by capability
        for row in capabilities_over_time {
            let week: DateTime<Utc> = row.try_get("week")?;
            let capability: String = row.try_get("capability")?;
            let learning_events: i64 = row.try_get("learning_events")?;
            let avg_performance: Option<f64> = row.try_get("avg_performance")?;

            capability_timeline.entry(capability)
                .or_insert_with(Vec::new)
                .push((week, learning_events, avg_performance.unwrap_or(0.0)));
        }

        // Analyze evolution for each capability
        for (capability, timeline) in capability_timeline {
            if timeline.len() >= 2 {
                let learning_rates: Vec<f64> = timeline.windows(2)
                    .map(|window| {
                        let (_, events1, _) = window[0];
                        let (_, events2, _) = window[1];
                        (events2 - events1) as f64
                    })
                    .collect();

                let avg_learning_rate = learning_rates.iter().sum::<f64>() / learning_rates.len() as f64;
                let latest_performance = timeline.last().map(|(_, _, perf)| *perf as f32).unwrap_or(0.0);

                capability_evolution.push(CapabilityEvolution {
                    capability: capability,
                    week: timeline.first().map(|(time, _, _)| *time).unwrap_or_else(|| Utc::now()),
                    learned_count: timeline.iter().map(|(_, events, _)| *events as usize).sum::<usize>() as i64,
                    avg_performance: Some(latest_performance as f64),
                    improvement_rate: avg_learning_rate,
                });
            }
        }

        Ok(capability_evolution)
    }

    /// Predict future performance based on historical patterns
    pub async fn predict_future_performance(&self, agent_id: &str, days_ahead: i64) -> MemoryResult<PerformancePrediction> {
        let historical_data = sqlx::query(
            r#"
            SELECT
                DATE_TRUNC('day', timestamp) as day,
                AVG((outcome->>'performance_score')::float) as avg_performance,
                COUNT(*) as experience_count
            FROM agent_experiences
            WHERE agent_id = $1
              AND timestamp > NOW() - INTERVAL '30 days'
            GROUP BY DATE_TRUNC('day', timestamp)
            ORDER BY day DESC
            LIMIT 14  -- Last 2 weeks
            "#,
        )
        .bind(agent_id)
        .fetch_all(self.db_client.pool())
        .await?;

        if historical_data.len() < 3 {
            return Ok(PerformancePrediction {
                predicted_score: 0.5,
                confidence: 0.1,
                prediction_date: Utc::now() + Duration::days(days_ahead),
                based_on_days: historical_data.len(),
            });
        }

        // Simple linear regression for prediction
        let mut x_values = Vec::new();
        let mut y_values = Vec::new();

        for (i, row) in historical_data.iter().enumerate() {
            let avg_performance: Option<f64> = row.try_get("avg_performance")?;
            if let Some(perf) = avg_performance {
                x_values.push(i as f64);
                y_values.push(perf);
            }
        }

        let n = x_values.len() as f64;
        let x_sum: f64 = x_values.iter().sum();
        let y_sum: f64 = y_values.iter().sum();
        let xy_sum: f64 = x_values.iter().zip(y_values.iter()).map(|(x, y)| x * y).sum();
        let x_squared_sum: f64 = x_values.iter().map(|x| x * x).sum();

        let slope = (n * xy_sum - x_sum * y_sum) / (n * x_squared_sum - x_sum * x_sum);
        let intercept = (y_sum - slope * x_sum) / n;

        // Predict future value
        let future_x = x_values.len() as f64 + days_ahead as f64;
        let predicted_score = slope * future_x + intercept;

        // Calculate confidence based on data quality
        let confidence = (n / 14.0).min(0.9); // Scale confidence by data amount

        Ok(PerformancePrediction {
            predicted_score: predicted_score.max(0.0).min(1.0) as f32,
            confidence: confidence as f32,
            prediction_date: Utc::now() + Duration::days(days_ahead),
            based_on_days: x_values.len(),
        })
    }
}

/// Performance prediction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformancePrediction {
    pub predicted_score: f32,
    pub confidence: f32,
    pub prediction_date: DateTime<Utc>,
    pub based_on_days: usize,
}
