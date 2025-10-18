//! Database health monitoring and diagnostics
//!
//! Provides comprehensive health checking, performance monitoring,
//! and diagnostic capabilities for production database operations.

use crate::DatabaseClient;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Connection tracker for statistics collection
#[derive(Debug)]
pub struct ConnectionTracker {
    /// Connection creation timestamps (most recent first)
    connection_times: Arc<RwLock<VecDeque<Instant>>>,
    /// Maximum number of connection records to keep
    max_records: usize,
}

impl ConnectionTracker {
    pub fn new(max_records: usize) -> Self {
        Self {
            connection_times: Arc::new(RwLock::new(VecDeque::new())),
            max_records,
        }
    }

    /// Record a new connection creation
    pub async fn record_connection(&self) {
        let mut times = self.connection_times.write().await;
        times.push_front(Instant::now());

        // Keep only the most recent records
        while times.len() > self.max_records {
            times.pop_back();
        }
    }

    /// Calculate connection creation rate per minute
    pub async fn calculate_creation_rate_per_minute(&self) -> f64 {
        let times = self.connection_times.read().await;

        if times.len() < 2 {
            return 0.0;
        }

        // Look at connections in the last 5 minutes for rate calculation
        let five_minutes_ago = Instant::now() - Duration::from_secs(300);
        let recent_connections: Vec<_> = times
            .iter()
            .take_while(|&&time| time > five_minutes_ago)
            .collect();

        if recent_connections.len() < 2 {
            return 0.0;
        }

        // Calculate rate based on time span
        let oldest_time = *recent_connections.last().unwrap();
        let newest_time = *recent_connections.first().unwrap();
        let time_span_minutes = (newest_time.duration_since(*oldest_time).as_secs_f64()) / 60.0;

        if time_span_minutes > 0.0 {
            (recent_connections.len() - 1) as f64 / time_span_minutes
        } else {
            0.0
        }
    }

    /// Calculate average connection lifetime
    pub async fn calculate_average_lifetime(&self) -> f64 {
        let times = self.connection_times.read().await;
        let now = Instant::now();

        if times.is_empty() {
            return 0.0;
        }

        let total_lifetime: f64 = times
            .iter()
            .map(|&time| now.duration_since(time).as_secs_f64())
            .sum();

        total_lifetime / times.len() as f64
    }

    /// Get total connections tracked
    pub async fn total_connections(&self) -> usize {
        self.connection_times.read().await.len()
    }
}

/// Database health checker
pub struct DatabaseHealthChecker {
    /// Database client
    client: DatabaseClient,
    /// Health check configuration
    config: HealthCheckConfig,
    /// Connection tracking for statistics
    connection_tracker: ConnectionTracker,
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
    /// Execute a query that returns rows
    async fn execute_query_rows(&self, query: &str) -> Result<Vec<sqlx::postgres::PgRow>> {
        let query = query.to_string();
        let pool = self.client.pool().clone();

        self.client
            .execute_query(|| {
                Box::pin(async move {
                    sqlx::query(&query)
                        .fetch_all(&pool)
                        .await
                        .context("Query execution failed")
                })
            })
            .await
    }

    /// Execute a query that returns a single row
    async fn execute_query_one(&self, query: &str) -> Result<sqlx::postgres::PgRow> {
        let query = query.to_string();
        let pool = self.client.pool().clone();

        self.client
            .execute_query(|| {
                Box::pin(async move {
                    sqlx::query(&query)
                        .fetch_one(&pool)
                        .await
                        .context("Query execution failed")
                })
            })
            .await
    }
    /// Create a new health checker
    pub fn new(client: DatabaseClient, config: HealthCheckConfig) -> Self {
        Self {
            client,
            config,
            connection_tracker: ConnectionTracker::new(1000), // Track last 1000 connections
        }
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
            self.client.test_connectivity(),
        )
        .await
        {
            Ok(Ok(true)) => {
                debug!(
                    "Database connectivity test passed in {:?}",
                    start_time.elapsed()
                );
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
            let idle_connections_u32 = idle_connections.try_into().unwrap_or(0);
            (pool_size - idle_connections_u32) as f64 / pool_size as f64 * 100.0
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

        let performance_ok =
            health_status.avg_execution_time_ms <= self.config.performance_threshold_ms;

        if !performance_ok {
            warn!(
                "Query performance degraded: avg {}ms > threshold {}ms",
                health_status.avg_execution_time_ms, self.config.performance_threshold_ms
            );
        } else {
            debug!(
                "Query performance OK: avg {}ms",
                health_status.avg_execution_time_ms
            );
        }

        Ok(performance_ok)
    }

    /// Generate error message for unhealthy state
    fn generate_error_message(
        &self,
        connection_ok: bool,
        pool_ok: bool,
        performance_ok: bool,
    ) -> String {
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

    /// Collect index usage statistics from PostgreSQL
    async fn collect_index_statistics(&self) -> Result<Vec<IndexUsage>> {
        // Query PostgreSQL's pg_stat_user_indexes for index usage statistics
        let query = r#"
            SELECT
                schemaname,
                tablename,
                indexname,
                idx_scan,
                idx_tup_read,
                idx_tup_fetch
            FROM pg_stat_user_indexes
            WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
            ORDER BY idx_scan DESC
            LIMIT 100
        "#;

        let rows = self.execute_query_rows(query).await?;

        let mut index_stats = Vec::new();

        for row in rows {
            let schema: String = row.try_get("schemaname")?;
            let table: String = row.try_get("tablename")?;
            let index_name: String = row.try_get("indexname")?;
            let scans: Option<i64> = row.try_get("idx_scan")?;
            let tuples_read: Option<i64> = row.try_get("idx_tup_read")?;
            let tuples_fetched: Option<i64> = row.try_get("idx_tup_fetch")?;

            // Calculate hit rate and usage efficiency
            let scans = scans.unwrap_or(0);
            let tuples_read = tuples_read.unwrap_or(0);
            let tuples_fetched = tuples_fetched.unwrap_or(0);

            let hit_rate = if tuples_read > 0 {
                (tuples_fetched as f64 / tuples_read as f64).min(1.0)
            } else {
                0.0
            };

            let usage_efficiency = if scans > 0 {
                (tuples_fetched as f64 / scans as f64).max(1.0)
            } else {
                0.0
            };

            index_stats.push(IndexUsage {
                index_name: format!("{}.{}", schema, index_name),
                table_name: format!("{}.{}", schema, table),
                scans: scans as u64,
                size_bytes: 0, // Size information not easily available from pg_stat_user_indexes
            });
        }

        debug!(
            "Collected index statistics for {} indexes",
            index_stats.len()
        );
        Ok(index_stats)
    }

    /// Collect table size statistics from PostgreSQL
    async fn collect_table_statistics(&self) -> Result<Vec<TableSize>> {
        // Query PostgreSQL for table sizes and statistics
        let query = r#"
            SELECT
                schemaname,
                tablename,
                pg_total_relation_size(schemaname || '.' || tablename) as total_size,
                pg_table_size(schemaname || '.' || tablename) as table_size,
                pg_indexes_size(schemaname || '.' || tablename) as index_size,
                n_tup_ins,
                n_tup_upd,
                n_tup_del,
                n_live_tup,
                n_dead_tup
            FROM pg_stat_user_tables
            WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
            ORDER BY total_size DESC
            LIMIT 50
        "#;

        let rows = self.execute_query_rows(query).await?;

        let mut table_stats = Vec::new();

        for row in rows {
            let schema: String = row.try_get("schemaname")?;
            let table: String = row.try_get("tablename")?;
            let total_size: Option<i64> = row.try_get("total_size")?;
            let table_size: Option<i64> = row.try_get("table_size")?;
            let index_size: Option<i64> = row.try_get("index_size")?;
            let inserts: Option<i64> = row.try_get("n_tup_ins")?;
            let updates: Option<i64> = row.try_get("n_tup_upd")?;
            let deletes: Option<i64> = row.try_get("n_tup_del")?;
            let live_tuples: Option<i64> = row.try_get("n_live_tup")?;
            let dead_tuples: Option<i64> = row.try_get("n_dead_tup")?;

            let total_size = total_size.unwrap_or(0) as u64;
            let table_size = table_size.unwrap_or(0) as u64;
            let index_size = index_size.unwrap_or(0) as u64;
            let live_tuples = live_tuples.unwrap_or(0) as u64;
            let dead_tuples = dead_tuples.unwrap_or(0) as u64;

            // Calculate bloat ratio (dead tuples / total tuples)
            let total_tuples = live_tuples + dead_tuples;
            let bloat_ratio = if total_tuples > 0 {
                dead_tuples as f64 / total_tuples as f64
            } else {
                0.0
            };

            // Calculate index to table size ratio
            let index_ratio = if table_size > 0 {
                index_size as f64 / table_size as f64
            } else {
                0.0
            };

            table_stats.push(TableSize {
                table_name: format!("{}.{}", schema, table),
                size_bytes: total_size,
            });
        }

        debug!(
            "Collected table statistics for {} tables",
            table_stats.len()
        );
        Ok(table_stats)
    }

    /// Collect slow query statistics from PostgreSQL
    async fn collect_slow_query_statistics(&self) -> Result<Vec<SlowQuery>> {
        // Query PostgreSQL's pg_stat_statements for slow queries
        // Note: pg_stat_statements extension must be enabled
        let query = r#"
            SELECT
                query,
                calls,
                total_time,
                mean_time,
                rows,
                temp_blks_read,
                temp_blks_written,
                blk_read_time,
                blk_write_time
            FROM pg_stat_statements
            WHERE mean_time > 1000  -- Queries taking more than 1 second on average
            ORDER BY mean_time DESC
            LIMIT 20
        "#;

        // Try to execute the query, but gracefully handle if pg_stat_statements is not available
        let rows = match self.execute_query_rows(query).await {
            Ok(rows) => rows,
            Err(e) => {
                debug!(
                    "pg_stat_statements not available for slow query analysis: {}",
                    e
                );
                return Ok(Vec::new());
            }
        };

        let mut slow_queries = Vec::new();

        for row in rows {
            let query_text: String = row.try_get("query")?;
            let calls: Option<i64> = row.try_get("calls")?;
            let total_time: Option<f64> = row.try_get("total_time")?;
            let mean_time: Option<f64> = row.try_get("mean_time")?;
            let rows_affected: Option<i64> = row.try_get("rows")?;
            let temp_read: Option<i64> = row.try_get("temp_blks_read")?;
            let temp_written: Option<i64> = row.try_get("temp_blks_written")?;
            let read_time: Option<f64> = row.try_get("blk_read_time")?;
            let write_time: Option<f64> = row.try_get("blk_write_time")?;

            let calls = calls.unwrap_or(0) as u64;
            let mean_time_ms = mean_time.unwrap_or(0.0);
            let total_time = total_time.unwrap_or(0.0);
            let rows_affected = rows_affected.unwrap_or(0) as u64;
            let temp_blocks = temp_read.unwrap_or(0) as u64 + temp_written.unwrap_or(0) as u64;
            let io_time_ms = read_time.unwrap_or(0.0) + write_time.unwrap_or(0.0);

            // Truncate very long queries for display
            let display_query = if query_text.len() > 200 {
                format!("{}...", &query_text[..200])
            } else {
                query_text.clone()
            };

            slow_queries.push(SlowQuery {
                query: display_query,
                calls,
                total_time,
                mean_time: mean_time_ms,
            });
        }

        debug!(
            "Collected statistics for {} slow queries",
            slow_queries.len()
        );
        Ok(slow_queries)
    }

    /// Collect comprehensive database diagnostics
    async fn collect_diagnostics(&self) -> Result<DatabaseDiagnostics> {
        let pool_size = self.client.pool().size();
        let idle_connections = self.client.pool().num_idle();

        // Pool statistics
        let idle_connections_u32 = idle_connections.try_into().unwrap_or(0);
        let pool_stats = PoolStats {
            active_connections: pool_size - idle_connections_u32,
            idle_connections: idle_connections_u32,
            max_size: self.client.config().pool_max,
            utilization_percent: if pool_size > 0 {
                (pool_size - idle_connections_u32) as f64 / pool_size as f64 * 100.0
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

        // Calculate comprehensive connection statistics
        let creation_rate = self
            .connection_tracker
            .calculate_creation_rate_per_minute()
            .await;
        let avg_lifetime = self.connection_tracker.calculate_average_lifetime().await;
        let total_connections = self.connection_tracker.total_connections().await as u64;

        let connection_stats = ConnectionStats {
            total_connections: total_connections.max(pool_size as u64),
            creation_rate_per_minute: creation_rate,
            avg_lifetime_seconds: avg_lifetime,
        };

        // Collect comprehensive index usage statistics
        let index_stats = self.collect_index_statistics().await.unwrap_or_default();

        // Collect comprehensive table size statistics
        let table_sizes = self.collect_table_statistics().await.unwrap_or_default();

        // Collect comprehensive slow query statistics
        let slow_queries = self
            .collect_slow_query_statistics()
            .await
            .unwrap_or_default();

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
            pool_health_threshold: 80.0,   // 80% utilization threshold
            performance_threshold_ms: 100, // 100ms query threshold
            enable_diagnostics: true,
        }
    }
}
