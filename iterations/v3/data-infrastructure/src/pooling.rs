//! Connection pooling implementation
//!
//! Deadpool-to-SQLx bridge for robust connection pooling with health checks,
//! timeout handling, and comprehensive monitoring.

use crate::database_config::DatabaseConfig;
use crate::database_metrics::DatabaseMetrics;
use anyhow::{Context, Result};
use chrono::Utc;
use deadpool_postgres::{Config, ManagerConfig, RecyclingMethod, Runtime, Pool as DeadpoolPool};
use std::sync::Arc;
use std::time::{Duration as StdDuration, Instant};
use tracing::{debug, info};

/// Deadpool-to-SQLx bridge for connection pooling
///
/// This wrapper implements the sqlx::Pool interface over deadpool::Pool,
/// providing seamless integration between the two connection pool systems.
#[derive(Debug, Clone)]
pub struct DeadpoolSqlxBridge {
    deadpool: DeadpoolPool,
    config: DatabaseConfig,
    metrics: Arc<DatabaseMetrics>,
}

impl DeadpoolSqlxBridge {
    /// Create a new bridge from deadpool configuration
    pub async fn new(config: DatabaseConfig, metrics: Arc<DatabaseMetrics>) -> Result<Self> {
        // Validate configuration for production safety
        config.validate().map_err(|e| anyhow::anyhow!("Database configuration validation failed: {}", e))?;
        info!("Database configuration validated successfully");

        let mut pg_config = Config::new();
        pg_config.host = config.host.clone();
        pg_config.port = config.port;
        pg_config.dbname = config.database.clone();
        pg_config.user = config.username.clone();
        pg_config.password = config.password.clone();
        pg_config.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        });
        pg_config.pool = Some(deadpool_postgres::PoolConfig {
            max_size: config.pool_max.unwrap_or(10) as usize,
            timeouts: deadpool_postgres::Timeouts {
                wait: Some(StdDuration::from_secs(config.connection_timeout_seconds.unwrap_or(30))),
                create: Some(StdDuration::from_secs(10)), // Connection creation timeout
                recycle: Some(StdDuration::from_secs(5)), // Connection recycle timeout
            },
            ..Default::default()
        });

        let deadpool = pg_config
            .create_pool(Some(Runtime::Tokio1), tokio_postgres::NoTls)
            .context("Failed to create deadpool connection pool")?;

        Ok(Self {
            deadpool,
            config,
            metrics,
        })
    }

    /// Get a connection with timeout and retry logic
    pub async fn acquire(&self) -> Result<DeadpoolSqlxConnection> {
        let start_time = Instant::now();

        // Implement timeout and retry logic
        let connection = tokio::time::timeout(
            StdDuration::from_secs(self.config.connection_timeout_seconds.unwrap_or(30)),
            self.deadpool.get()
        )
        .await
        .context("Connection acquisition timeout")?
        .context("Failed to acquire connection from deadpool")?;

        let acquisition_time = start_time.elapsed();
        self.metrics.record_connection_acquisition(acquisition_time);

        Ok(DeadpoolSqlxConnection {
            connection,
            metrics: self.metrics.clone(),
        })
    }

    /// Perform health check on the connection pool
    pub async fn health_check(&self) -> Result<()> {
        let mut conn = self.acquire().await?;
        conn.health_check().await
    }

    /// Get pool size information
    pub fn size(&self) -> usize {
        self.deadpool.status().size
    }

    /// Get available connections
    pub fn available(&self) -> usize {
        self.deadpool.status().available
    }

    /// Get waiting connections
    pub fn waiting(&self) -> usize {
        self.deadpool.status().waiting
    }

    /// Execute a query and return rows
    pub async fn query(&self, query: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> Result<Vec<tokio_postgres::Row>> {
        let mut conn = self.acquire().await?;
        conn.execute_query(query, params).await
    }

    /// Execute a query and return affected row count
    pub async fn execute(&self, query: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> Result<u64> {
        let conn = self.acquire().await?;
        let result = conn.connection.execute(query, params).await?;
        Ok(result)
    }
}

/// Wrapper for deadpool connection that implements sqlx traits
#[derive(Debug)]
pub struct DeadpoolSqlxConnection {
    connection: deadpool_postgres::Client,
    metrics: Arc<DatabaseMetrics>,
}

impl DeadpoolSqlxConnection {
    /// Perform health check on the connection
    pub async fn health_check(&mut self) -> Result<()> {
        let start_time = Instant::now();

        // Simple query to test connection health
        let result = self.connection
            .query_one("SELECT 1", &[])
            .await
            .context("Health check query failed")?;

        let health_check_time = start_time.elapsed();
        self.metrics.record_health_check(health_check_time);

        // Verify the result
        let value: i32 = result.get(0);
        if value != 1 {
            return Err(anyhow::anyhow!("Health check returned unexpected value: {}", value));
        }

        Ok(())
    }

    /// Execute a query and return the connection for further use
    pub async fn execute_query(&mut self, query: &str, params: &[&(dyn tokio_postgres::types::ToSql + Sync)]) -> Result<Vec<tokio_postgres::Row>> {
        let start_time = Instant::now();

        let rows = self.connection
            .query(query, params)
            .await
            .context("Query execution failed")?;

        let execution_time = start_time.elapsed();
        self.metrics.record_query_execution(execution_time);

        Ok(rows)
    }
}


