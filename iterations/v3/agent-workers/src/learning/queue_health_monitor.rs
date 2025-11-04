//! Queue health monitor for tracking queue performance

use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use crate::learning::types::*;
use crate::worker_types::{ExecutionOutcome, LearningMode};

/// Monitors queue health and performance metrics
pub struct QueueHealthMonitor {
    queue_metrics: Arc<tokio::sync::RwLock<QueueHealthMetrics>>,
    historical_data: Arc<tokio::sync::RwLock<Vec<QueueHealthMetrics>>>,
}

impl QueueHealthMonitor {
    pub fn new() -> Self {
        Self {
            queue_metrics: Arc::new(tokio::sync::RwLock::new(QueueHealthMetrics {
                queue_depth: 0,
                average_wait_time_ms: 0.0,
                processing_rate: 0.0,
                error_rate: 0.0,
                last_updated: Utc::now(),
            })),
            historical_data: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    /// Update queue metrics
    pub async fn update_metrics(
        &self,
        queue_depth: u64,
        average_wait_time_ms: f64,
        processing_rate: f64,
        error_rate: f64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let metrics = QueueHealthMetrics {
            queue_depth,
            average_wait_time_ms,
            processing_rate,
            error_rate,
            last_updated: Utc::now(),
        };

        // Update current metrics
        {
            let mut current_metrics = self.queue_metrics.write().await;
            *current_metrics = metrics.clone();
        }

        // Store historical data
        {
            let mut historical = self.historical_data.write().await;
            historical.push(metrics);
            
            // Keep only last 1000 entries to prevent memory growth
            if historical.len() > 1000 {
                historical.drain(0..historical.len() - 1000);
            }
        }

        Ok(())
    }

    /// Get current queue health metrics
    pub async fn get_health_metrics(&self) -> Result<QueueHealthMetrics, Box<dyn std::error::Error + Send + Sync>> {
        let metrics = self.queue_metrics.read().await;
        Ok(metrics.clone())
    }

    /// Check if queue is healthy
    pub async fn is_healthy(&self, thresholds: &QueueHealthThresholds) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let metrics = self.get_health_metrics().await?;
        
        Ok(
            metrics.queue_depth <= thresholds.max_queue_depth &&
            metrics.average_wait_time_ms <= thresholds.max_wait_time_ms &&
            metrics.processing_rate >= thresholds.min_processing_rate &&
            metrics.error_rate <= thresholds.max_error_rate
        )
    }

    /// Get queue health status
    pub async fn get_health_status(&self) -> Result<QueueHealthStatus, Box<dyn std::error::Error + Send + Sync>> {
        let metrics = self.get_health_metrics().await?;
        
        let status = if metrics.queue_depth > 100 {
            QueueHealthStatus::Critical
        } else if metrics.queue_depth > 50 || metrics.average_wait_time_ms > 30000.0 {
            QueueHealthStatus::Warning
        } else if metrics.error_rate > 0.1 {
            QueueHealthStatus::Degraded
        } else {
            QueueHealthStatus::Healthy
        };

        Ok(status)
    }

    /// Get historical trends
    pub async fn get_trends(&self, window_minutes: u64) -> Result<QueueTrends, Box<dyn std::error::Error + Send + Sync>> {
        let historical = self.historical_data.read().await;
        let cutoff_time = Utc::now() - chrono::Duration::minutes(window_minutes as i64);
        
        let recent_data: Vec<&QueueHealthMetrics> = historical
            .iter()
            .filter(|metrics| metrics.last_updated > cutoff_time)
            .collect();

        if recent_data.is_empty() {
            return Ok(QueueTrends {
                queue_depth_trend: TrendDirection::Stable,
                wait_time_trend: TrendDirection::Stable,
                processing_rate_trend: TrendDirection::Stable,
                error_rate_trend: TrendDirection::Stable,
            });
        }

        let queue_depth_trend = self.calculate_trend(recent_data.iter().map(|m| m.queue_depth as f64));
        let wait_time_trend = self.calculate_trend(recent_data.iter().map(|m| m.average_wait_time_ms));
        let processing_rate_trend = self.calculate_trend(recent_data.iter().map(|m| m.processing_rate));
        let error_rate_trend = self.calculate_trend(recent_data.iter().map(|m| m.error_rate));

        Ok(QueueTrends {
            queue_depth_trend,
            wait_time_trend,
            processing_rate_trend,
            error_rate_trend,
        })
    }

    /// Calculate trend direction from a series of values
    fn calculate_trend<I>(&self, values: I) -> TrendDirection
    where
        I: Iterator<Item = f64>,
    {
        let values: Vec<f64> = values.collect();
        if values.len() < 2 {
            return TrendDirection::Stable;
        }

        let first_half = &values[..values.len() / 2];
        let second_half = &values[values.len() / 2..];

        let first_avg = first_half.iter().sum::<f64>() / first_half.len() as f64;
        let second_avg = second_half.iter().sum::<f64>() / second_half.len() as f64;

        let change_percent = if first_avg != 0.0 {
            (second_avg - first_avg) / first_avg.abs()
        } else {
            0.0
        };

        if change_percent > 0.1 {
            TrendDirection::Increasing
        } else if change_percent < -0.1 {
            TrendDirection::Decreasing
        } else {
            TrendDirection::Stable
        }
    }

    /// Get performance recommendations
    pub async fn get_recommendations(&self) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let metrics = self.get_health_metrics().await?;
        let mut recommendations = Vec::new();

        if metrics.queue_depth > 50 {
            recommendations.push("Consider scaling up workers to reduce queue depth".to_string());
        }

        if metrics.average_wait_time_ms > 30000.0 {
            recommendations.push("High wait times detected - optimize task processing or add more workers".to_string());
        }

        if metrics.processing_rate < 1.0 {
            recommendations.push("Low processing rate - check worker performance and resource allocation".to_string());
        }

        if metrics.error_rate > 0.05 {
            recommendations.push("High error rate - investigate and fix underlying issues".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("Queue is performing well - continue current configuration".to_string());
        }

        Ok(recommendations)
    }
}

impl Default for QueueHealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Queue health thresholds

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct QueueHealthThresholds {
    pub max_queue_depth: u64,
    pub max_wait_time_ms: f64,
    pub min_processing_rate: f64,
    pub max_error_rate: f64,
}

impl Default for QueueHealthThresholds {
    fn default() -> Self {
        Self {
            max_queue_depth: 100,
            max_wait_time_ms: 30000.0, // 30 seconds
            min_processing_rate: 1.0,   // 1 task per second
            max_error_rate: 0.05,       // 5% error rate
        }
    }
}

/// Queue health status

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
enum QueueHealthStatus {
    Healthy,
    Degraded,
    Warning,
    Critical,
}

/// Trend direction

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
}

/// Queue trends

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct QueueTrends {
    pub queue_depth_trend: TrendDirection,
    pub wait_time_trend: TrendDirection,
    pub processing_rate_trend: TrendDirection,
    pub error_rate_trend: TrendDirection,
}
