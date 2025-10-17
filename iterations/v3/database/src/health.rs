//! Database health monitoring and diagnostics
//!
//! Provides comprehensive health checking, performance monitoring,
//! and diagnostic capabilities for production database operations.

use crate::{DatabaseClient, DatabaseConfig};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// Database health checker
pub struct DatabaseHealthChecker {
    /// Database client
    client: DatabaseClient,
    /// Health check configuration
    config: HealthCheckConfig,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Enable comprehensive health checks
    pub enabled: bool,
    /// Health check interval (seconds)
    pub check_interval_seconds: u64,
    /// Query timeout for health checks (seconds)
    pub query_timeout_seconds: u64,
    /// Connection pool health threshold (%)
    pub pool_health_threshold: f64,
    /// Query performance threshold (ms)
    pub performance_threshold_ms: u64,
    /// Enable detailed diagnostics
    pub enable_diagnostics: bool,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Overall health status
    pub healthy: bool,
    /// Connection status
    pub connection_ok: bool,
    /// Pool status
    pub pool_ok: bool,
    /// Performance status
    pub performance_ok: bool,
    /// Last check timestamp
    pub last_check: DateTime<Utc>,
    /// Response time (milliseconds)
    pub response_time_ms: u64,
    /// Error message if unhealthy
    pub error_message: Option<String>,
    /// Detailed diagnostics
    pub diagnostics: Option<DatabaseDiagnostics>,
}

/// Database diagnostics information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseDiagnostics {
    /// Pool statistics
    pub pool_stats: PoolStats,
    /// Query performance metrics
    pub query_metrics: QueryMetrics,
    /// Connection statistics
    pub connection_stats: ConnectionStats,
    /// Index usage statistics
    pub index_stats: Vec<IndexUsage>,
    /// Table size information
    pub table_sizes: Vec<TableSize>,
    /// Slow queries (if available)
    pub slow_queries: Vec<SlowQuery>,
}

/// Pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    /// Active connections
    pub active_connections: u32,
    /// Idle connections
    pub idle_connections: u32,
    /// Maximum pool size
    pub max_size: u32,
    /// Pool utilization percentage
    pub utilization_percent: f64,
}

/// Query performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMetrics {
    /// Average query time (ms)
    pub avg_query_time_ms: f64,
    /// Maximum query time (ms)
    pub max_query_time_ms: f64,
    /// Total queries executed
    pub total_queries: u64,
    /// Query success rate (%)
    pub success_rate: f64,
}

/// Connection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStats {
    /// Total connections created
    pub total_connections: u64,
    /// Connection creation rate (per minute)
    pub creation_rate_per_minute: f64,
    /// Average connection lifetime (seconds)
    pub avg_lifetime_seconds: f64,
}

/// Index usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexUsage {
    /// Index name
    pub index_name: String,
    /// Table name
    pub table_name: String,
    /// Index scans
    pub scans: u64,
    /// Index size (bytes)
    pub size_bytes: u64,
}

/// Table size information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSize {
    /// Table name
    pub table_name: String,
    /// Table size (bytes)
    pub size_bytes: u64,
}

/// Slow query information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlowQuery {
    /// Query text (truncated)
    pub query: String,
    /// Execution count
    pub calls: u64,
    /// Total execution time
    pub total_time: f64,
    /// Average execution time
    pub mean_time: f64,
}

impl DatabaseHealthChecker {
    /// Create a new health checker
    pub fn new(client: DatabaseClient, config: HealthCheckConfig) -> Self {
        Self { client, config }
    }

    /// Perform comprehensive health check
    pub async fn perform_health_check(&self) -> Result<HealthCheckResult> {
        let start_time = Instant::now();

        if !self.config.enabled {
            return Ok(HealthCheckResult {
                healthy: true,
                connection_ok: true,
                pool_ok: true,
                performance_ok: true,
                last_check: Utc::now(),
                response_time_ms: 0,
                error_message: None,
                diagnostics: None,
            });
        }

        // Test basic connectivity
        let connection_ok = self.test_connectivity().await.unwrap_or(false);

        // Check pool health
        let pool_ok = self.check_pool_health().await.unwrap_or(false);

        // Check query performance
        let performance_ok = self.check_query_performance().await.unwrap_or(true);

        // Overall health
        let healthy = connection_ok && pool_ok && performance_ok;

        let response_time = start_time.elapsed();
        let response_time_ms = response_time.as_millis() as u64;

        let error_message = if !healthy {
            Some(self.generate_error_message(connection_ok, pool_ok, performance_ok))
        } else {
            None
        };

        // Collect diagnostics if enabled and healthy
        let diagnostics = if self.config.enable_diagnostics && healthy {
            Some(self.collect_diagnostics().await.unwrap_or_default())
        } else {
            None
        };

        Ok(HealthCheckResult {
            healthy,
            connection_ok,
            pool_ok,
            performance_ok,
            last_check: Utc::now(),
            response_time_ms,
            error_message,
            diagnostics,
        })
    }

    /// Test basic database connectivity
    async fn test_connectivity(&self) -> Result<bool> {
        let start_time = Instant::now();

        match tokio::time::timeout(
            Duration::from_secs(self.config.query_timeout_seconds),
            self.client.health_check(),
        ).await {
            Ok(Ok(true)) => {
                debug!("Database connectivity test passed in {:?}", start_time.elapsed());
                Ok(true)
            }
            Ok(Ok(false)) => {
                warn!("Database connectivity test failed");
                Ok(false)
            }
            Ok(Err(e)) => {
                warn!("Database connectivity test error: {}", e);
                Ok(false)
            }
            Err(_) => {
                warn!("Database connectivity test timed out");
                Ok(false)
            }
        }
    }

    /// Check connection pool health
    async fn check_pool_health(&self) -> Result<bool> {
        let pool_size = self.client.pool().size();
        let idle_connections = self.client.pool().num_idle();
        let utilization = if pool_size > 0 {
            (pool_size - idle_connections) as f64 / pool_size as f64 * 100.0
        } else {
            0.0
        };

        let pool_ok = utilization <= self.config.pool_health_threshold;

        if !pool_ok {
            warn!(
                "Pool utilization {:.2}% exceeds threshold {:.2}%",
                utilization, self.config.pool_health_threshold
            );
        } else {
            debug!("Pool health OK: utilization {:.2}%", utilization);
        }

        Ok(pool_ok)
    }

    /// Check query performance
    async fn check_query_performance(&self) -> Result<bool> {
        let health_status = self.client.get_health_status().await?;

        let performance_ok = health_status.avg_execution_time_ms <= self.config.performance_threshold_ms;

        if !performance_ok {
            warn!(
                "Query performance degraded: avg {}ms > threshold {}ms",
                health_status.avg_execution_time_ms, self.config.performance_threshold_ms
            );
        } else {
            debug!("Query performance OK: avg {}ms", health_status.avg_execution_time_ms);
        }

        Ok(performance_ok)
    }

    /// Generate error message for unhealthy state
    fn generate_error_message(&self, connection_ok: bool, pool_ok: bool, performance_ok: bool) -> String {
        let mut issues = Vec::new();

        if !connection_ok {
            issues.push("database connection failed");
        }
        if !pool_ok {
            issues.push("connection pool unhealthy");
        }
        if !performance_ok {
            issues.push("query performance degraded");
        }

        format!("Database health issues: {}", issues.join(", "))
    }

    /// Collect comprehensive database diagnostics
    async fn collect_diagnostics(&self) -> Result<DatabaseDiagnostics> {
        let pool_size = self.client.pool().size();
        let idle_connections = self.client.pool().num_idle();

        // Pool statistics
        let pool_stats = PoolStats {
            active_connections: pool_size - idle_connections,
            idle_connections,
            max_size: self.client.config().pool_max,
            utilization_percent: if pool_size > 0 {
                (pool_size - idle_connections) as f64 / pool_size as f64 * 100.0
            } else {
                0.0
            },
        };

        // Query metrics from health status
        let health_status = self.client.get_health_status().await?;
        let query_metrics = QueryMetrics {
            avg_query_time_ms: health_status.avg_execution_time_ms as f64,
            max_query_time_ms: health_status.max_execution_time_ms as f64,
            total_queries: health_status.total_queries,
            success_rate: health_status.success_rate,
        };

        // Connection statistics (simplified)
        let connection_stats = ConnectionStats {
            total_connections: pool_size as u64,
            creation_rate_per_minute: 0.0, // Would need more detailed tracking
            avg_lifetime_seconds: 0.0,    // Would need connection tracking
        };

        // Index usage (simplified - would need pg_stat_user_indexes query)
        let index_stats = Vec::new();

        // Table sizes (simplified - would need pg_table_size queries)
        let table_sizes = Vec::new();

        // Slow queries (simplified - would need pg_stat_statements)
        let slow_queries = Vec::new();

        Ok(DatabaseDiagnostics {
            pool_stats,
            query_metrics,
            connection_stats,
            index_stats,
            table_sizes,
            slow_queries,
        })
    }
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval_seconds: 60,
            query_timeout_seconds: 5,
            pool_health_threshold: 80.0, // 80% utilization threshold
            performance_threshold_ms: 100, // 100ms query threshold
            enable_diagnostics: true,
        }
    }
}


