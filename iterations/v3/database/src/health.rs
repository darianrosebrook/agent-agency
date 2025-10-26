//! Database health monitoring and status
//!
//! Comprehensive health checks, statistics collection, and status monitoring
//! for database connectivity, performance, and operational health.

use super::circuit_breaker::CircuitState;
use super::database_metrics::DatabaseMetrics;
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Database health status summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseHealthStatus {
    pub connectivity_ok: bool,
    pub pool_size: u32,
    pub idle_connections: u32,
    pub circuit_breaker_state: CircuitState,
    pub total_queries: u64,
    pub success_rate: f64,
    pub avg_execution_time_ms: u64,
    pub max_execution_time_ms: u64,
    pub circuit_breaker_trips: u64,
    pub last_health_check: DateTime<Utc>,
    pub overall_health: HealthStatus,
}

/// Overall health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Critical,
}

/// Database statistics with comprehensive metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub pool_size: u32,
    pub idle_connections: u32,
    pub table_counts: HashMap<String, i64>,
    pub uptime: Option<Duration>,
    pub memory_usage_mb: Option<u64>,
    pub active_connections: u32,
    pub total_connections_created: u64,
}

/// Health monitor for database operations
#[derive(Debug)]
pub struct DatabaseHealthMonitor {
    metrics: Arc<DatabaseMetrics>,
    health_check_interval: std::time::Duration,
    last_health_check: std::sync::RwLock<Option<DateTime<Utc>>>,
    consecutive_failures: std::sync::atomic::AtomicU64,
}

impl DatabaseHealthMonitor {
    /// Create a new health monitor
    pub fn new(metrics: Arc<DatabaseMetrics>) -> Self {
        Self {
            metrics,
            health_check_interval: std::time::Duration::from_secs(30), // Check every 30 seconds
            last_health_check: std::sync::RwLock::new(None),
            consecutive_failures: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Perform comprehensive health check
    pub async fn perform_health_check(&self) -> Result<DatabaseHealthStatus> {
        let start_time = std::time::Instant::now();

        // Update last check timestamp
        let now = Utc::now();
        *self.last_health_check.write().unwrap() = Some(now);

        // Basic connectivity check (would be implemented with actual DB connection)
        let connectivity_ok = true; // Placeholder

        // Get metrics snapshot
        let metrics_snapshot = self.metrics.snapshot();

        // Determine overall health
        let overall_health = self.determine_overall_health(&metrics_snapshot);

        let status = DatabaseHealthStatus {
            connectivity_ok,
            pool_size: 10, // Placeholder - would get from actual pool
            idle_connections: 5, // Placeholder
            circuit_breaker_state: CircuitState::Closed, // Placeholder
            total_queries: metrics_snapshot.total_queries,
            success_rate: metrics_snapshot.success_rate,
            avg_execution_time_ms: (metrics_snapshot.avg_execution_time_ns / 1_000_000) as u64,
            max_execution_time_ms: (metrics_snapshot.max_execution_time_ns / 1_000_000) as u64,
            circuit_breaker_trips: metrics_snapshot.circuit_breaker_trips,
            last_health_check: now,
            overall_health,
        };

        let check_duration = start_time.elapsed();
        debug!("Health check completed in {:?}", check_duration);

        Ok(status)
    }

    /// Determine overall health status based on metrics
    fn determine_overall_health(&self, metrics: &super::database_metrics::DatabaseMetricsSnapshot) -> HealthStatus {
        // Critical conditions
        if metrics.success_rate < 0.5 {
            return HealthStatus::Critical;
        }

        // Unhealthy conditions
        if metrics.success_rate < 0.8 || metrics.avg_execution_time_ns > 5_000_000_000 { // 5 seconds
            return HealthStatus::Unhealthy;
        }

        // Degraded conditions
        if metrics.success_rate < 0.95 || metrics.avg_execution_time_ns > 1_000_000_000 { // 1 second
            return HealthStatus::Degraded;
        }

        HealthStatus::Healthy
    }

    /// Get detailed health report
    pub async fn get_health_report(&self) -> Result<HealthReport> {
        let status = self.perform_health_check().await?;

        let report = HealthReport {
            status: status.clone(),
            recommendations: self.generate_recommendations(&status).await,
            performance_trends: self.analyze_performance_trends().await,
            alerts: self.check_for_alerts(&status),
        };

        Ok(report)
    }

    /// Generate health recommendations based on status
    async fn generate_recommendations(&self, status: &DatabaseHealthStatus) -> Vec<String> {
        let mut recommendations = Vec::new();

        if status.success_rate < 0.95 {
            recommendations.push("Consider increasing connection pool size".to_string());
        }

        if status.avg_execution_time_ms > 1000 {
            recommendations.push("Review and optimize slow queries".to_string());
        }

        if status.circuit_breaker_trips > 5 {
            recommendations.push("Investigate frequent connection failures".to_string());
        }

        if matches!(status.overall_health, HealthStatus::Critical | HealthStatus::Unhealthy) {
            recommendations.push("Immediate attention required - database performance degraded".to_string());
        }

        recommendations
    }

    /// Analyze performance trends
    async fn analyze_performance_trends(&self) -> PerformanceTrends {
        // Placeholder - would analyze historical metrics
        PerformanceTrends {
            query_time_trend: Trend::Stable,
            success_rate_trend: Trend::Improving,
            connection_usage_trend: Trend::Stable,
        }
    }

    /// Check for health alerts
    fn check_for_alerts(&self, status: &DatabaseHealthStatus) -> Vec<HealthAlert> {
        let mut alerts = Vec::new();

        if status.success_rate < 0.9 {
            alerts.push(HealthAlert {
                level: AlertLevel::Warning,
                message: format!("Low success rate: {:.1}%", status.success_rate * 100.0),
                timestamp: Utc::now(),
            });
        }

        if status.avg_execution_time_ms > 2000 {
            alerts.push(HealthAlert {
                level: AlertLevel::Critical,
                message: format!("High average execution time: {}ms", status.avg_execution_time_ms),
                timestamp: Utc::now(),
            });
        }

        alerts
    }

    /// Check if health check should be performed
    pub async fn should_perform_health_check(&self) -> bool {
        if let Some(last_check) = *self.last_health_check.read().unwrap() {
            let elapsed = Utc::now().signed_duration_since(last_check);
            elapsed > chrono::Duration::from_std(self.health_check_interval).unwrap()
        } else {
            true // No previous check
        }
    }
}

/// Comprehensive health report
#[derive(Debug, Clone)]
pub struct HealthReport {
    pub status: DatabaseHealthStatus,
    pub recommendations: Vec<String>,
    pub performance_trends: PerformanceTrends,
    pub alerts: Vec<HealthAlert>,
}

/// Performance trend analysis
#[derive(Debug, Clone)]
pub struct PerformanceTrends {
    pub query_time_trend: Trend,
    pub success_rate_trend: Trend,
    pub connection_usage_trend: Trend,
}

/// Trend direction
#[derive(Debug, Clone)]
pub enum Trend {
    Improving,
    Degrading,
    Stable,
}

/// Health alert
#[derive(Debug, Clone)]
pub struct HealthAlert {
    pub level: AlertLevel,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

/// Alert severity level
#[derive(Debug, Clone)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}