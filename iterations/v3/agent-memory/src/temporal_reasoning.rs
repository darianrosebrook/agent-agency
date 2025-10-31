#![cfg(feature = "database")]
//! Temporal Reasoning Engine - Time-based analysis and causality detection

use crate::memory_types::*;
use crate::MemoryResult;
use sqlx::{PgPool, Row};
use sqlx::postgres::PgRow;
use std::sync::Arc;
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use tracing::{info, debug, warn, error};
use reqwest::Client;
use anyhow::{Context, Result};

/// Real HTTP-based temporal analysis service
#[derive(Debug)]
pub struct HttpTemporalAnalysisService {
    client: Client,
    base_url: String,
    timeout_ms: u64,
}

impl HttpTemporalAnalysisService {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            timeout_ms: 30000,
        }
    }

    /// Analyze temporal patterns via HTTP call to external service
    pub async fn analyze_patterns(&self, data: &TemporalAnalysisRequest) -> Result<TemporalAnalysisResponse> {
        let url = format!("{}/api/v1/temporal/analyze", self.base_url);
        
        let payload = serde_json::json!({
            "agent_id": data.agent_id,
            "time_range": {
                "start": data.time_range.start,
                "end": data.time_range.end
            },
            "metrics": data.metrics,
            "analysis_type": data.analysis_type
        });

        debug!("Analyzing temporal patterns for agent: {}", data.agent_id);

        let response = self.client
            .post(&url)
            .json(&payload)
            .timeout(std::time::Duration::from_millis(self.timeout_ms))
            .send()
            .await
            .context("Failed to send temporal analysis request")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow::anyhow!("Temporal analysis service error {}: {}", status, error_text));
        }

        let result: TemporalAnalysisResponse = response.json().await
            .context("Failed to parse temporal analysis response")?;

        debug!("Temporal analysis completed with {} patterns found", result.patterns.len());
        Ok(result)
    }

    /// Detect causality relationships via HTTP call
    pub async fn detect_causality(&self, events: &[TemporalEvent]) -> Result<Vec<CausalityRelationship>> {
        let url = format!("{}/api/v1/temporal/causality", self.base_url);
        
        let payload = serde_json::json!({
            "events": events
        });

        debug!("Detecting causality relationships for {} events", events.len());

        let response = self.client
            .post(&url)
            .json(&payload)
            .timeout(std::time::Duration::from_millis(self.timeout_ms))
            .send()
            .await
            .context("Failed to send causality detection request")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow::anyhow!("Causality detection service error {}: {}", status, error_text));
        }

        let result: serde_json::Value = response.json().await
            .context("Failed to parse causality detection response")?;

        // Extract causality relationships from response
        let relationships = result["relationships"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid causality detection response format"))?
            .iter()
            .map(|rel| CausalityRelationship {
                cause_event: rel["cause_event"].as_str().unwrap_or("").to_string(),
                effect_event: rel["effect_event"].as_str().unwrap_or("").to_string(),
                confidence: rel["confidence"].as_f64().unwrap_or(0.0) as f32,
                time_lag: rel["time_lag"].as_u64().unwrap_or(0) as i64,
                relationship_type: rel["relationship_type"].as_str().unwrap_or("CAUSES").to_string(),
            })
            .collect::<Vec<CausalityRelationship>>();

        debug!("Detected {} causality relationships", relationships.len());
        Ok(relationships)
    }

    /// Health check for temporal analysis service
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);

        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

/// Request for temporal analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalAnalysisRequest {
    pub agent_id: String,
    pub time_range: TimeRange,
    pub metrics: Vec<String>,
    pub analysis_type: String,
}

/// Response from temporal analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalAnalysisResponse {
    pub patterns: Vec<TemporalPattern>,
    pub trends: Vec<TrendAnalysis>,
    pub change_points: Vec<ChangePoint>,
    pub predictions: Vec<Prediction>,
}

/// Temporal pattern detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalPattern {
    pub pattern_type: String,
    pub confidence: f32,
    pub description: String,
    pub time_range: TimeRange,
}

/// Trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub metric: String,
    pub trend_direction: String,
    pub slope: f32,
    pub r_squared: f32,
}

/// Change point in time series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePoint {
    pub timestamp: DateTime<Utc>,
    pub confidence: f32,
    pub change_type: String,
    pub magnitude: f32,
}

/// Prediction for future values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    pub metric: String,
    pub predicted_value: f32,
    pub confidence: f32,
    pub prediction_time: DateTime<Utc>,
}

/// Temporal event for causality analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalEvent {
    pub event_id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Causality relationship between events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalityRelationship {
    pub cause_event: String,
    pub effect_event: String,
    pub confidence: f32,
    pub time_lag: i64,
    pub relationship_type: String,
}

/// Temporal reasoning engine for time-based memory analysis
#[derive(Debug)]
pub struct TemporalReasoningEngine {
    db_pool: Arc<PgPool>,
    config: TemporalConfig,
    temporal_service: Arc<HttpTemporalAnalysisService>,
}

impl TemporalReasoningEngine {
    /// Create a new temporal reasoning engine
    pub async fn new(config: &TemporalConfig) -> MemoryResult<Self> {
        // Get database URL from environment
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/agent_agency_v3".to_string());

        // Create direct database connection pool
        let db_pool = Arc::new(
            PgPool::connect(&database_url)
                .await
                .context("Failed to connect to database for temporal reasoning")?
        );

        // Get temporal analysis service URL from environment or use default
        let temporal_url = std::env::var("TEMPORAL_ANALYSIS_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:9000".to_string());

        info!("Initializing HTTP temporal analysis service at: {}", temporal_url);

        // Create HTTP-based temporal analysis service
        let temporal_service = Arc::new(HttpTemporalAnalysisService::new(temporal_url));
        
        // Test connection
        if let Err(e) = temporal_service.health_check().await {
            warn!("Temporal analysis service health check failed: {}", e);
        } else {
            info!("Temporal analysis service health check passed");
        }

        Ok(Self {
            db_pool,
            config: config.clone(),
            temporal_service,
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
        .fetch_all(&*self.db_pool)
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
                time_range: (time_range.start, time_range.end),
            });
        }

        // Detect change points
        let change_points = self.detect_performance_change_points(&performance_values).await?;

        // Find causality links
        let causality_links = self.detect_performance_causality(agent_id, time_range).await?;

        // Calculate performance summary
        let summary = self.calculate_performance_summary(&performance_values);

        // Convert complex structs to expected simple types
        let trend_directions: Vec<TrendDirection> = trends.into_iter().map(|t| t.direction).collect();
        let change_point_times: Vec<chrono::DateTime<chrono::Utc>> = change_points.into_iter().map(|cp| cp.timestamp).collect();
        let causality_simple: Vec<(String, String, f32)> = causality_links.into_iter()
            .map(|cl| (cl.cause, cl.effect, cl.confidence)).collect();

        Ok(TemporalAnalysis {
            time_range: (time_range.start, time_range.end),
            trends: trend_directions,
            change_points: change_point_times,
            causality_links: causality_simple,
            performance_summary: format!("Overall score: {:.2}, Best: {:.2}, Worst: {:.2}", 
                summary.overall_score,
                summary.metric_scores.get("best").unwrap_or(&0.0),
                summary.metric_scores.get("worst").unwrap_or(&0.0)),
            patterns: vec![],
            performance_metrics: HashMap::new(),
            recommendations: vec![],
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
                    confidence: 0.7,
                    change_type: if change_magnitude > 0.0 { "Spike".to_string() } else { "Drop".to_string() },
                    magnitude: change_magnitude.abs(),
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
        .fetch_all(&*self.db_pool)
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
                .fetch_one(&*self.db_pool)
                .await?;

                let learned_count: i64 = capability_growth.try_get("learned_count")?;

                if learned_count > 0 && avg_perf > 0.7 && stddev_val < 0.2 {
                    // Strong correlation between task type and consistent high performance with learning
                    causality_links.push(CausalityLink {
                        cause: format!("performing_{}", task_type.to_lowercase().replace(" ", "_")),
                        effect: "high_performance_with_learning".to_string(),
                        confidence: (avg_perf as f32).min(0.9),
                        time_lag_seconds: 3600, // 1 hour lag
                        evidence: vec![
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
                overall_score: 0.0,
                metric_scores: HashMap::new(),
                trends: vec![],
                recommendations: vec![],
                analyzed_at: Utc::now(),
            };
        }

        let scores: Vec<f32> = performance_data.iter().map(|(_, score, _)| *score).collect();
        let avg_score = scores.iter().sum::<f32>() / scores.len() as f32;
        
        // Create metric scores map
        let mut metric_scores = HashMap::new();
        metric_scores.insert("average".to_string(), avg_score);
        metric_scores.insert("best".to_string(), scores.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b)));
        metric_scores.insert("worst".to_string(), scores.iter().fold(f32::INFINITY, |a, &b| a.min(b)));

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

        metric_scores.insert("improvement_rate".to_string(), improvement_rate);
        metric_scores.insert("consistency".to_string(), consistency_score);

        PerformanceSummary {
            overall_score: avg_score,
            metric_scores,
            trends: vec![],
            recommendations: vec![],
            analyzed_at: Utc::now(),
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
        .fetch_all(&*self.db_pool)
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

                // Create evolution points from timeline
                let evolution_points: Vec<EvolutionPoint> = timeline.iter()
                    .map(|(timestamp, events, performance)| {
                        let mut metrics = HashMap::new();
                        metrics.insert("learning_events".to_string(), *events as f32);
                        metrics.insert("performance".to_string(), *performance as f32);
                        
                        EvolutionPoint {
                            timestamp: *timestamp,
                            level: (*performance as f32).min(1.0).max(0.0),
                            context: format!("{} learning events", events),
                            metrics,
                        }
                    })
                    .collect();

                capability_evolution.push(CapabilityEvolution {
                    capability: capability,
                    timeline: evolution_points,
                    current_level: latest_performance.min(1.0).max(0.0),
                    predicted_level: (latest_performance + avg_learning_rate as f32).min(1.0).max(0.0),
                    learning_rate: avg_learning_rate as f32,
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
        .fetch_all(&*self.db_pool)
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
