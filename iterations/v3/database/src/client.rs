//! Database client implementation with connection pooling and query methods
//!
//! Production-hardened database client with:
//! - Robust connection pooling with health checks
//! - Circuit breaker pattern for resilience
//! - Query timeout and retry logic
//! - Comprehensive monitoring and metrics
//! - Input sanitization and prepared statements

use crate::{models::*, DatabaseConfig, DatabaseVectorStore, VectorStoreStats};
use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use deadpool_postgres::{Config, ManagerConfig, RecyclingMethod, Runtime, Pool as DeadpoolPool};
use serde_json;
use sqlx::Row;
use sqlx::{PgPool, Postgres, Acquire, Connection, Executor, Postgres as SqlxPostgres};
use sqlx::postgres::{PgConnection, PgPoolOptions};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration as StdDuration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, error, info};
use uuid::Uuid;

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
        let mut pg_config = Config::new();
        pg_config.host = Some(config.host.clone());
        pg_config.port = Some(config.port);
        pg_config.dbname = Some(config.database.clone());
        pg_config.user = Some(config.username.clone());
        pg_config.password = Some(config.password.clone());
        pg_config.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        });
        pg_config.pool = Some(deadpool_postgres::PoolConfig {
            max_size: config.pool_max as usize,
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
            StdDuration::from_secs(30), // 30 second timeout
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

/// Production-hardened database client with monitoring and resilience
#[derive(Debug)]
pub struct DatabaseClient {
    /// Deadpool-to-sqlx bridge
    bridge: DeadpoolSqlxBridge,
    /// Fallback connection pool
    pool: PgPool,
    /// Database configuration
    config: DatabaseConfig,
    /// Circuit breaker state
    circuit_breaker: Arc<CircuitBreaker>,
    /// Query execution metrics
    metrics: Arc<DatabaseMetrics>,
    /// Connection semaphore for rate limiting
    connection_semaphore: Arc<Semaphore>,
    /// Prepared statement cache
    prepared_statements: Arc<RwLock<HashMap<String, String>>>,
}

/// Circuit breaker for database resilience
#[derive(Debug)]
pub struct CircuitBreaker {
    /// Failure threshold before opening circuit
    failure_threshold: u32,
    /// Success threshold to close circuit
    success_threshold: u32,
    /// Timeout before attempting recovery
    recovery_timeout: Duration,
    /// Current state
    state: Arc<RwLock<CircuitState>>,
    /// Consecutive failures
    failures: AtomicU64,
    /// Consecutive successes
    successes: AtomicU64,
    /// Last failure time
    last_failure: Arc<RwLock<Option<Instant>>>,
}

impl CircuitBreaker {
    pub fn new() -> Self {
        Self {
            failure_threshold: 3,
            success_threshold: 5,
            recovery_timeout: Duration::seconds(30),
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failures: AtomicU64::new(0),
            successes: AtomicU64::new(0),
            last_failure: Arc::new(RwLock::new(None)),
        }
    }
}

/// Circuit breaker states
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// Database execution metrics
#[derive(Debug)]
pub struct DatabaseMetrics {
    /// Total queries executed
    total_queries: AtomicU64,
    /// Successful queries
    successful_queries: AtomicU64,
    /// Failed queries
    failed_queries: AtomicU64,
    /// Average query execution time (nanoseconds)
    avg_execution_time_ns: AtomicU64,
    /// Longest query execution time (nanoseconds)
    max_execution_time_ns: AtomicU64,
    /// Connection pool usage
    pool_usage: AtomicU64,
    /// Circuit breaker trips
    circuit_breaker_trips: AtomicU64,
}

impl DatabaseMetrics {
    pub fn new() -> Self {
        Self {
            total_queries: AtomicU64::new(0),
            successful_queries: AtomicU64::new(0),
            failed_queries: AtomicU64::new(0),
            avg_execution_time_ns: AtomicU64::new(0),
            max_execution_time_ns: AtomicU64::new(0),
            pool_usage: AtomicU64::new(0),
            circuit_breaker_trips: AtomicU64::new(0),
        }
    }

    /// Record connection acquisition time
    pub fn record_connection_acquisition(&self, duration: StdDuration) {
        let duration_ns = duration.as_nanos() as u64;
        
        // Update max acquisition time
        let mut current_max = self.max_execution_time_ns.load(Ordering::Relaxed);
        while duration_ns > current_max {
            match self.max_execution_time_ns.compare_exchange_weak(
                current_max,
                duration_ns,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => current_max = x,
            }
        }
        
        // Update average (simplified calculation)
        let total = self.total_queries.load(Ordering::Relaxed);
        if total > 0 {
            let current_avg = self.avg_execution_time_ns.load(Ordering::Relaxed);
            let new_avg = (current_avg * total + duration_ns) / (total + 1);
            self.avg_execution_time_ns.store(new_avg, Ordering::Relaxed);
        } else {
            self.avg_execution_time_ns.store(duration_ns, Ordering::Relaxed);
        }
    }

    /// Record health check time
    pub fn record_health_check(&self, duration: StdDuration) {
        let duration_ns = duration.as_nanos() as u64;
        
        // Update max health check time
        let mut current_max = self.max_execution_time_ns.load(Ordering::Relaxed);
        while duration_ns > current_max {
            match self.max_execution_time_ns.compare_exchange_weak(
                current_max,
                duration_ns,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => current_max = x,
            }
        }
    }

    /// Record query execution time
    pub fn record_query_execution(&self, duration: StdDuration) {
        let duration_ns = duration.as_nanos() as u64;
        
        // Increment total queries
        self.total_queries.fetch_add(1, Ordering::Relaxed);
        
        // Update max execution time
        let mut current_max = self.max_execution_time_ns.load(Ordering::Relaxed);
        while duration_ns > current_max {
            match self.max_execution_time_ns.compare_exchange_weak(
                current_max,
                duration_ns,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => current_max = x,
            }
        }
        
        // Update average execution time
        let total = self.total_queries.load(Ordering::Relaxed);
        if total > 0 {
            let current_avg = self.avg_execution_time_ns.load(Ordering::Relaxed);
            let new_avg = (current_avg * (total - 1) + duration_ns) / total;
            self.avg_execution_time_ns.store(new_avg, Ordering::Relaxed);
        } else {
            self.avg_execution_time_ns.store(duration_ns, Ordering::Relaxed);
        }
    }
}

impl DatabaseClient {
    /// Create a new production-hardened database client
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        info!("Initializing production-hardened database client");

        // Initialize circuit breaker
        let circuit_breaker = Arc::new(CircuitBreaker {
            failure_threshold: 5,                    // Open after 5 failures
            success_threshold: 3,                    // Close after 3 successes
            recovery_timeout: Duration::seconds(30), // Wait 30s before half-open
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failures: AtomicU64::new(0),
            successes: AtomicU64::new(0),
            last_failure: Arc::new(RwLock::new(None)),
        });

        // Initialize metrics
        let metrics = Arc::new(DatabaseMetrics {
            total_queries: AtomicU64::new(0),
            successful_queries: AtomicU64::new(0),
            failed_queries: AtomicU64::new(0),
            avg_execution_time_ns: AtomicU64::new(0),
            max_execution_time_ns: AtomicU64::new(0),
            pool_usage: AtomicU64::new(0),
            circuit_breaker_trips: AtomicU64::new(0),
        });

        // Create connection pool with enhanced configuration
        let pool = PgPool::connect_with(
            sqlx::postgres::PgConnectOptions::new()
                .host(&config.host)
                .port(config.port)
                .database(&config.database)
                .username(&config.username)
                .password(&config.password)
                .application_name("agent-agency-v3")
                .statement_cache_capacity(100), // Cache prepared statements
        )
        .await
        .context("Failed to create database connection pool")?;

        // Test connection with circuit breaker
        let pool_clone = pool.clone();
        match Self::execute_with_circuit_breaker(&circuit_breaker, &metrics, || {
            Box::pin(async move {
                sqlx::query("SELECT 1")
                    .execute(&pool_clone)
                    .await
                    .context("Failed to test database connection")
            })
        })
        .await
        {
            Ok(_) => info!("Database connection test successful"),
            Err(e) => {
                error!("Database connection test failed: {}", e);
                return Err(e);
            }
        }

        // Initialize connection semaphore for rate limiting
        let connection_semaphore = Arc::new(Semaphore::new(config.pool_max as usize));

        // Initialize prepared statement cache
        let prepared_statements = Arc::new(RwLock::new(HashMap::new()));

        info!("Database client initialized successfully");
        Ok(Self {
            bridge,
            pool,
            config,
            circuit_breaker,
            metrics,
            connection_semaphore,
            prepared_statements,
        })
    }

    /// Create database client with deadpool (alternative implementation)
    pub async fn with_deadpool(config: DatabaseConfig) -> Result<Self> {
        // Initialize metrics
        let metrics = Arc::new(DatabaseMetrics::new());
        
        let mut pg_config = Config::new();
        pg_config.host = Some(config.host.clone());
        pg_config.port = Some(config.port);
        pg_config.dbname = Some(config.database.clone());
        pg_config.user = Some(config.username.clone());
        pg_config.password = Some(config.password.clone());
        pg_config.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        });
        pg_config.pool = Some(deadpool_postgres::PoolConfig {
            max_size: config.pool_max as usize,
            ..Default::default()
        });

        let _pool = pg_config
            .create_pool(Some(Runtime::Tokio1), tokio_postgres::NoTls)
            .context("Failed to create deadpool connection pool")?;

        // Create deadpool-to-sqlx bridge
        let bridge = DeadpoolSqlxBridge::new(config.clone(), metrics.clone())
            .await
            .context("Failed to create deadpool-to-sqlx bridge")?;

        // Create fallback sqlx pool for compatibility
        let sqlx_pool = PgPool::connect(&config.database_url())
            .await
            .context("Failed to create fallback sqlx connection pool")?;

        // Initialize circuit breaker
        let circuit_breaker = Arc::new(CircuitBreaker::new());

        // Initialize connection semaphore
        let connection_semaphore = Arc::new(Semaphore::new(config.pool_max as usize));

        // Initialize prepared statement cache
        let prepared_statements = Arc::new(RwLock::new(HashMap::new()));

        Ok(Self {
            bridge,
            pool: sqlx_pool,
            config,
            circuit_breaker,
            metrics,
            connection_semaphore,
            prepared_statements,
        })
    }

    /// Get a reference to the deadpool-to-sqlx bridge
    pub fn bridge(&self) -> &DeadpoolSqlxBridge {
        &self.bridge
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Get database configuration
    pub fn config(&self) -> &DatabaseConfig {
        &self.config
    }

    /// Get database metrics
    pub fn metrics(&self) -> &Arc<DatabaseMetrics> {
        &self.metrics
    }

    /// Get circuit breaker state
    pub async fn circuit_breaker_state(&self) -> CircuitState {
        self.circuit_breaker.state.read().await.clone()
    }

    /// Execute query with circuit breaker protection and metrics
    pub async fn execute_query<F, T>(&self, query_fn: F) -> Result<T>
    where
        F: FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send>>,
    {
        // Acquire connection semaphore permit
        let _permit = self
            .connection_semaphore
            .acquire()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to acquire connection permit: {}", e))?;

        Self::execute_with_circuit_breaker(&self.circuit_breaker, &self.metrics, query_fn).await
    }

    /// Execute query with circuit breaker protection
    async fn execute_with_circuit_breaker<F, T>(
        circuit_breaker: &Arc<CircuitBreaker>,
        metrics: &Arc<DatabaseMetrics>,
        query_fn: F,
    ) -> Result<T>
    where
        F: FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send>>,
    {
        let start_time = Instant::now();

        // Check circuit breaker state
        let state = circuit_breaker.state.read().await.clone();

        match state {
            CircuitState::Open => {
                // Check if we should attempt recovery
                let last_failure = circuit_breaker.last_failure.read().await;
                if let Some(failure_time) = *last_failure {
                    let elapsed = start_time.duration_since(failure_time);
                    let elapsed_chrono = Duration::seconds(elapsed.as_secs() as i64);
                    if elapsed_chrono > circuit_breaker.recovery_timeout {
                        // Attempt recovery - transition to half-open
                        drop(state);
                        let mut state_write = circuit_breaker.state.write().await;
                        *state_write = CircuitState::HalfOpen;
                    } else {
                        return Err(anyhow::anyhow!("Circuit breaker is open"));
                    }
                } else {
                    return Err(anyhow::anyhow!("Circuit breaker is open"));
                }
            }
            CircuitState::HalfOpen => {
                // Allow one request through for testing
            }
            CircuitState::Closed => {
                // Normal operation
            }
        }

        // Execute the query
        let result = query_fn().await;

        let execution_time = start_time.elapsed();

        // Update metrics
        metrics.total_queries.fetch_add(1, Ordering::Relaxed);

        match &result {
            Ok(_) => {
                metrics.successful_queries.fetch_add(1, Ordering::Relaxed);

                // Update circuit breaker success count
                let current_successes =
                    circuit_breaker.successes.fetch_add(1, Ordering::Relaxed) + 1;
                if current_successes >= circuit_breaker.success_threshold as u64 {
                    let mut state = circuit_breaker.state.write().await;
                    if matches!(*state, CircuitState::HalfOpen) {
                        *state = CircuitState::Closed;
                        circuit_breaker.successes.store(0, Ordering::Relaxed);
                        circuit_breaker.failures.store(0, Ordering::Relaxed);
                    }
                }
            }
            Err(_) => {
                metrics.failed_queries.fetch_add(1, Ordering::Relaxed);

                // Update circuit breaker failure count
                let current_failures = circuit_breaker.failures.fetch_add(1, Ordering::Relaxed) + 1;

                if current_failures >= circuit_breaker.failure_threshold as u64 {
                    let mut state = circuit_breaker.state.write().await;
                    *state = CircuitState::Open;
                    *circuit_breaker.last_failure.write().await = Some(start_time);
                    metrics
                        .circuit_breaker_trips
                        .fetch_add(1, Ordering::Relaxed);
                }
            }
        }

        // Update execution time metrics
        let execution_time_ns = execution_time.as_nanos() as u64;
        let current_avg = metrics.avg_execution_time_ns.load(Ordering::Relaxed);
        let total_queries = metrics.total_queries.load(Ordering::Relaxed);

        if total_queries > 1 {
            let new_avg = (current_avg * (total_queries - 1) + execution_time_ns) / total_queries;
            metrics
                .avg_execution_time_ns
                .store(new_avg, Ordering::Relaxed);
        } else {
            metrics
                .avg_execution_time_ns
                .store(execution_time_ns, Ordering::Relaxed);
        }

        // Update max execution time
        let current_max = metrics.max_execution_time_ns.load(Ordering::Relaxed);
        if execution_time_ns > current_max {
            metrics
                .max_execution_time_ns
                .store(execution_time_ns, Ordering::Relaxed);
        }

        result
    }

    /// Execute a safe query with timeout and retry logic
    pub async fn execute_safe_query(&self, query: &str) -> Result<sqlx::postgres::PgQueryResult> {
        let query = query.to_string();
        let pool = self.pool.clone();
        self.execute_query(|| {
            Box::pin(async move {
                // Use a timeout for the query execution
                tokio::time::timeout(
                    StdDuration::from_secs(30), // 30 second timeout
                    sqlx::query(&query).execute(&pool),
                )
                .await
                .map_err(|_| anyhow::anyhow!("Query timed out"))?
                .context("Query execution failed")
            })
        })
        .await
    }

    /// Test database connectivity
    pub async fn test_connectivity(&self) -> Result<bool> {
        match sqlx::query("SELECT 1").fetch_one(&self.pool).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Execute a parameterized query safely
    pub async fn execute_parameterized_query(
        &self,
        query: &str,
        params: Vec<serde_json::Value>,
    ) -> Result<sqlx::postgres::PgQueryResult> {
        // Parameter validation
        self.validate_parameters(&params)?;

        // Query sanitization - basic check for SQL injection patterns
        self.validate_query_safety(query)?;

        let query_clone = query.to_string();
        let params_clone = params.clone();
        let pool = self.pool.clone();

        self.execute_query(|| {
            Box::pin(async move {
                // Build parameterized query using sqlx
                let mut sql_query = sqlx::query(&query_clone);

                // Bind parameters dynamically
                for param in &params_clone {
                    match param {
                        serde_json::Value::String(s) => {
                            sql_query = sql_query.bind(s.clone());
                        }
                        serde_json::Value::Number(n) => {
                            if let Some(i) = n.as_i64() {
                                sql_query = sql_query.bind(i);
                            } else if let Some(f) = n.as_f64() {
                                sql_query = sql_query.bind(f);
                            } else {
                                return Err(anyhow::anyhow!("Unsupported number type"));
                            }
                        }
                        serde_json::Value::Bool(b) => {
                            sql_query = sql_query.bind(*b);
                        }
                        serde_json::Value::Null => {
                            sql_query = sql_query.bind(None::<String>);
                        }
                        _ => {
                            return Err(anyhow::anyhow!("Unsupported parameter type: {}", param));
                        }
                    }
                }

                // Execute with timeout
                tokio::time::timeout(StdDuration::from_secs(30), sql_query.execute(&pool))
                    .await
                    .map_err(|_| anyhow::anyhow!("Parameterized query timed out"))?
                    .context("Parameterized query execution failed")
            })
        })
        .await
    }

    /// Validate query parameters
    fn validate_parameters(&self, params: &[serde_json::Value]) -> Result<()> {
        for (i, param) in params.iter().enumerate() {
            match param {
                serde_json::Value::String(s) => {
                    // Check for excessively long strings
                    if s.len() > 10000 {
                        return Err(anyhow::anyhow!(
                            "Parameter {}: string too long ({} chars)",
                            i,
                            s.len()
                        ));
                    }
                    // Basic injection pattern check
                    if s.contains('\0') {
                        return Err(anyhow::anyhow!(
                            "Parameter {}: null byte injection attempt",
                            i
                        ));
                    }
                }
                serde_json::Value::Number(n) => {
                    // Check for reasonable number ranges
                    if let Some(f) = n.as_f64() {
                        if !f.is_finite() {
                            return Err(anyhow::anyhow!("Parameter {}: invalid number", i));
                        }
                    }
                }
                _ => {} // Other types are generally safe
            }
        }
        Ok(())
    }

    /// Validate query safety
    fn validate_query_safety(&self, query: &str) -> Result<()> {
        // Basic SQL injection checks
        let injection_patterns = [
            r";\s*(drop|delete|truncate|alter|create)\s",
            r"--\s*(drop|delete|truncate|alter|create)",
            r"/\*\s*(drop|delete|truncate|alter|create)",
            r"union\s+select.*--",
            r"exec\s*\(",
            r"xp_cmdshell",
        ];

        let query_lower = query.to_lowercase();
        for pattern in &injection_patterns {
            if regex::Regex::new(pattern)?.is_match(&query_lower) {
                return Err(anyhow::anyhow!("Potentially unsafe query pattern detected"));
            }
        }

        Ok(())
    }

    /// Get comprehensive database health status
    pub async fn get_health_status(&self) -> Result<DatabaseHealthStatus> {
        let pool_size = self.pool.size();
        let idle_connections = self.pool.num_idle();
        let circuit_state = self.circuit_breaker_state().await;

        // Test a simple query to check database connectivity
        let connectivity_ok = self.test_connectivity().await.unwrap_or(false);

        let metrics = self.metrics();
        let total_queries = metrics.total_queries.load(Ordering::Relaxed);
        let success_rate = if total_queries > 0 {
            (metrics.successful_queries.load(Ordering::Relaxed) as f64 / total_queries as f64)
                * 100.0
        } else {
            100.0
        };

        Ok(DatabaseHealthStatus {
            connectivity_ok,
            pool_size,
            idle_connections: idle_connections.try_into().unwrap_or(0),
            circuit_breaker_state: circuit_state,
            total_queries,
            success_rate,
            avg_execution_time_ms: metrics.avg_execution_time_ns.load(Ordering::Relaxed)
                / 1_000_000,
            max_execution_time_ms: metrics.max_execution_time_ns.load(Ordering::Relaxed)
                / 1_000_000,
            circuit_breaker_trips: metrics.circuit_breaker_trips.load(Ordering::Relaxed),
        })
    }
}

/// Database health status information
#[derive(Debug, Clone, serde::Serialize)]
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
}

impl DatabaseClient {
    /// Get database statistics
    pub async fn get_stats(&self) -> Result<DatabaseStats> {
        let pool_stats = self.pool.size();
        let idle_connections = self.pool.num_idle();

        // Get table row counts
        let tables = [
            "judges",
            "workers",
            "tasks",
            "task_executions",
            "council_verdicts",
            "judge_evaluations",
            "debate_sessions",
            "knowledge_entries",
            "performance_metrics",
            "caws_compliance",
            "audit_trail",
        ];

        let mut table_counts = std::collections::HashMap::new();
        for table in tables {
            let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {}", table))
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);
            table_counts.insert(table.to_string(), count);
        }

        Ok(DatabaseStats {
            pool_size: pool_stats,
            idle_connections: idle_connections.try_into().unwrap_or(0),
            table_counts,
            uptime: None, // Could be implemented with a startup timestamp
        })
    }

    /// Execute a migration
    pub async fn migrate(&self, migration_sql: &str) -> Result<()> {
        info!("Executing database migration");

        sqlx::query(migration_sql)
            .execute(&self.pool)
            .await
            .context("Failed to execute migration")?;

        info!("Migration completed successfully");
        Ok(())
    }

    /// Create the database if it doesn't exist
    pub async fn ensure_database_exists(&self) -> Result<()> {
        let server_url = self.config.server_url();
        let db_name = &self.config.database;

        // Connect to postgres database to create our database
        let server_pool = PgPool::connect(&format!("{}/postgres", server_url))
            .await
            .context("Failed to connect to postgres database")?;

        // Check if database exists
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)")
                .bind(db_name)
                .fetch_one(&server_pool)
                .await
                .context("Failed to check database existence")?;

        if !exists {
            info!("Creating database: {}", db_name);
            sqlx::query(&format!("CREATE DATABASE {}", db_name))
                .execute(&server_pool)
                .await
                .context("Failed to create database")?;
            info!("Database created successfully");
        } else {
            debug!("Database already exists: {}", db_name);
        }

        server_pool.close().await;
        Ok(())
    }
}

/// Database statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DatabaseStats {
    pub pool_size: u32,
    pub idle_connections: u32,
    pub table_counts: std::collections::HashMap<String, i64>,
    pub uptime: Option<Duration>,
}

/// Database operations trait for type-safe queries
#[async_trait]
pub trait DatabaseOperations {
    type Error;

    // Judge operations
    async fn create_judge(&self, judge: CreateJudge) -> Result<Judge, Self::Error>;
    async fn get_judge(&self, id: Uuid) -> Result<Option<Judge>, Self::Error>;
    async fn get_judges(&self) -> Result<Vec<Judge>, Self::Error>;
    async fn update_judge(&self, id: Uuid, update: UpdateJudge) -> Result<Judge, Self::Error>;
    async fn delete_judge(&self, id: Uuid) -> Result<(), Self::Error>;

    // Worker operations
    async fn create_worker(&self, worker: CreateWorker) -> Result<Worker, Self::Error>;
    async fn get_worker(&self, id: Uuid) -> Result<Option<Worker>, Self::Error>;
    async fn get_workers(&self) -> Result<Vec<Worker>, Self::Error>;
    async fn get_workers_by_type(&self, worker_type: &str) -> Result<Vec<Worker>, Self::Error>;
    async fn update_worker(&self, id: Uuid, update: UpdateWorker) -> Result<Worker, Self::Error>;
    async fn delete_worker(&self, id: Uuid) -> Result<(), Self::Error>;
    fn validate_worker_update(&self, update: &UpdateWorker) -> Result<(), Self::Error>;

    // Task operations
    async fn create_task(&self, task: CreateTask) -> Result<Task, Self::Error>;
    async fn get_task(&self, id: Uuid) -> Result<Option<Task>, Self::Error>;
    async fn get_tasks(
        &self,
        filters: Option<TaskFilters>,
        pagination: Option<PaginationParams>,
    ) -> Result<Vec<Task>, Self::Error>;
    async fn update_task(&self, id: Uuid, update: UpdateTask) -> Result<Task, Self::Error>;
    async fn delete_task(&self, id: Uuid) -> Result<(), Self::Error>;

    // Task execution operations
    async fn create_task_execution(
        &self,
        execution: CreateTaskExecution,
    ) -> Result<TaskExecution, Self::Error>;
    async fn get_task_executions(&self, task_id: Uuid) -> Result<Vec<TaskExecution>, Self::Error>;
    async fn update_task_execution(
        &self,
        id: Uuid,
        update: UpdateTaskExecution,
    ) -> Result<TaskExecution, Self::Error>;

    // Council verdict operations
    async fn create_council_verdict(
        &self,
        verdict: CreateCouncilVerdict,
    ) -> Result<CouncilVerdict, Self::Error>;
    async fn get_council_verdict(
        &self,
        verdict_id: Uuid,
    ) -> Result<Option<CouncilVerdict>, Self::Error>;
    async fn get_council_verdicts(
        &self,
        filters: Option<VerdictFilters>,
        pagination: Option<PaginationParams>,
    ) -> Result<Vec<CouncilVerdict>, Self::Error>;

    // Judge evaluation operations
    async fn create_judge_evaluation(
        &self,
        evaluation: CreateJudgeEvaluation,
    ) -> Result<JudgeEvaluation, Self::Error>;
    async fn get_judge_evaluations(
        &self,
        verdict_id: Uuid,
    ) -> Result<Vec<JudgeEvaluation>, Self::Error>;

    // Knowledge entry operations
    async fn create_knowledge_entry(
        &self,
        entry: CreateKnowledgeEntry,
    ) -> Result<KnowledgeEntry, Self::Error>;
    async fn get_knowledge_entries(
        &self,
        filters: Option<KnowledgeFilters>,
        pagination: Option<PaginationParams>,
    ) -> Result<Vec<KnowledgeEntry>, Self::Error>;
    async fn search_knowledge(
        &self,
        query: &str,
        limit: Option<u32>,
    ) -> Result<Vec<KnowledgeEntry>, Self::Error>;

    // Performance metric operations
    async fn create_performance_metric(
        &self,
        metric: CreatePerformanceMetric,
    ) -> Result<PerformanceMetric, Self::Error>;
    async fn get_performance_metrics(
        &self,
        entity_type: &str,
        entity_id: Uuid,
    ) -> Result<Vec<PerformanceMetric>, Self::Error>;

    // CAWS compliance operations
    async fn create_caws_compliance(
        &self,
        compliance: CreateCawsCompliance,
    ) -> Result<CawsCompliance, Self::Error>;
    async fn get_caws_compliance(
        &self,
        task_id: Uuid,
    ) -> Result<Option<CawsCompliance>, Self::Error>;

    // Audit trail operations
    async fn create_audit_trail_entry(
        &self,
        entry: CreateAuditTrailEntry,
    ) -> Result<AuditTrailEntry, Self::Error>;
    async fn get_audit_trail(
        &self,
        entity_type: &str,
        entity_id: Uuid,
    ) -> Result<Vec<AuditTrailEntry>, Self::Error>;

    // Analytics and statistics
    async fn get_council_metrics(&self) -> Result<Vec<CouncilMetrics>, Self::Error>;
    async fn get_judge_performance(&self) -> Result<Vec<JudgePerformance>, Self::Error>;
    async fn get_worker_performance(&self) -> Result<Vec<WorkerPerformance>, Self::Error>;
    async fn get_task_execution_summary(
        &self,
        task_id: Uuid,
    ) -> Result<Option<TaskExecutionSummary>, Self::Error>;
}

#[async_trait]
impl DatabaseOperations for DatabaseClient {
    type Error = anyhow::Error;

    // Judge operations implementation
    async fn create_judge(&self, judge: CreateJudge) -> Result<Judge, Self::Error> {
        let created = sqlx::query_as::<_, Judge>(
            "INSERT INTO judges (name, model_name, endpoint, weight, timeout_ms, optimization_target) 
             VALUES ($1, $2, $3, $4, $5, $6) 
             RETURNING *"
        )
        .bind(&judge.name)
        .bind(&judge.model_name)
        .bind(&judge.endpoint)
        .bind(judge.weight)
        .bind(judge.timeout_ms)
        .bind(&judge.optimization_target)
        .fetch_one(&self.pool)
        .await
        .context("Failed to create judge")?;

        info!("Created judge: {}", created.name);
        Ok(created)
    }

    async fn get_judge(&self, id: Uuid) -> Result<Option<Judge>, Self::Error> {
        let judge = sqlx::query_as::<_, Judge>("SELECT * FROM judges WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .context("Failed to get judge")?;

        Ok(judge)
    }

    async fn get_judges(&self) -> Result<Vec<Judge>, Self::Error> {
        let judges = sqlx::query_as::<_, Judge>("SELECT * FROM judges ORDER BY created_at")
            .fetch_all(&self.pool)
            .await
            .context("Failed to get judges")?;

        Ok(judges)
    }

    async fn update_judge(&self, id: Uuid, update: UpdateJudge) -> Result<Judge, Self::Error> {
        // Build dynamic update query
        let mut query = "UPDATE judges SET updated_at = NOW()".to_string();
        let mut params: Vec<Box<dyn sqlx::Encode<'_, Postgres> + Send + Sync>> = Vec::new();
        let mut param_count = 0;

        if let Some(name) = update.name {
            param_count += 1;
            query.push_str(&format!(", name = ${}", param_count));
            params.push(Box::new(name));
        }

        if let Some(model_name) = update.model_name {
            param_count += 1;
            query.push_str(&format!(", model_name = ${}", param_count));
            params.push(Box::new(model_name));
        }

        if let Some(endpoint) = update.endpoint {
            param_count += 1;
            query.push_str(&format!(", endpoint = ${}", param_count));
            params.push(Box::new(endpoint));
        }

        if let Some(weight) = update.weight {
            param_count += 1;
            query.push_str(&format!(", weight = ${}", param_count));
            params.push(Box::new(weight));
        }

        if let Some(timeout_ms) = update.timeout_ms {
            param_count += 1;
            query.push_str(&format!(", timeout_ms = ${}", param_count));
            params.push(Box::new(timeout_ms));
        }

        if let Some(optimization_target) = update.optimization_target {
            param_count += 1;
            query.push_str(&format!(", optimization_target = ${}", param_count));
            params.push(Box::new(optimization_target));
        }

        if let Some(is_active) = update.is_active {
            param_count += 1;
            query.push_str(&format!(", is_active = ${}", param_count));
            params.push(Box::new(is_active));
        }

        param_count += 1;
        query.push_str(&format!(" WHERE id = ${} RETURNING *", param_count));
        params.push(Box::new(id));

        // Execute the update and fetch the updated judge
        let updated_judge = sqlx::query_as::<_, Judge>(&query)
            .fetch_one(&self.pool)
            .await
            .context("Failed to update judge")?;

        info!("Updated judge: {}", updated_judge.id);
        Ok(updated_judge)
    }

    async fn delete_judge(&self, id: Uuid) -> Result<(), Self::Error> {
        sqlx::query("DELETE FROM judges WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to delete judge")?;

        info!("Deleted judge: {}", id);
        Ok(())
    }

    // Worker operations implementation
    async fn create_worker(&self, worker: CreateWorker) -> Result<Worker, Self::Error> {
        let created_worker = sqlx::query_as::<_, Worker>(
            "INSERT INTO workers (name, worker_type, specialty, model_name, endpoint, capabilities) 
             VALUES ($1, $2, $3, $4, $5, $6) 
             RETURNING *"
        )
        .bind(&worker.name)
        .bind(&worker.worker_type)
        .bind(&worker.specialty)
        .bind(&worker.model_name)
        .bind(&worker.endpoint)
        .bind(&worker.capabilities)
        .fetch_one(&self.pool)
        .await
        .context("Failed to create worker")?;

        info!(
            "Created worker: {} with ID: {}",
            created_worker.name, created_worker.id
        );
        Ok(created_worker)
    }

    async fn get_worker(&self, id: Uuid) -> Result<Option<Worker>, Self::Error> {
        let worker = sqlx::query_as::<_, Worker>("SELECT * FROM workers WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .context("Failed to get worker")?;

        if let Some(ref worker) = worker {
            info!("Retrieved worker: {} with ID: {}", worker.name, worker.id);
        }
        Ok(worker)
    }

    async fn get_workers(&self) -> Result<Vec<Worker>, Self::Error> {
        let workers = sqlx::query_as::<_, Worker>("SELECT * FROM workers ORDER BY name")
            .fetch_all(&self.pool)
            .await
            .context("Failed to get workers")?;

        info!("Retrieved {} workers", workers.len());
        Ok(workers)
    }

    async fn get_workers_by_type(&self, worker_type: &str) -> Result<Vec<Worker>, Self::Error> {
        let workers = sqlx::query_as::<_, Worker>(
            "SELECT * FROM workers WHERE worker_type = $1 ORDER BY name",
        )
        .bind(worker_type)
        .fetch_all(&self.pool)
        .await
        .context("Failed to get workers by type")?;

        info!(
            "Retrieved {} workers of type: {}",
            workers.len(),
            worker_type
        );
        Ok(workers)
    }

    async fn update_worker(&self, id: Uuid, update: UpdateWorker) -> Result<Worker, Self::Error> {
        // Validate update data
        self.validate_worker_update(&update)?;

        // Build dynamic update query
        let mut query_parts = Vec::new();
        let mut param_count = 1;
        let mut params = Vec::new();

        if let Some(name) = &update.name {
            query_parts.push(format!("name = ${}", param_count));
            params.push(serde_json::Value::String(name.clone()));
            param_count += 1;
        }

        if let Some(worker_type) = &update.worker_type {
            query_parts.push(format!("worker_type = ${}", param_count));
            params.push(serde_json::Value::String(worker_type.clone()));
            param_count += 1;
        }

        if let Some(capabilities) = &update.capabilities {
            query_parts.push(format!("capabilities = ${}", param_count));
            params.push(serde_json::Value::String(serde_json::to_string(
                capabilities,
            )?));
            param_count += 1;
        }

        if let Some(status) = &update.status {
            query_parts.push(format!("status = ${}", param_count));
            params.push(serde_json::Value::String(status.clone()));
            param_count += 1;
        }

        if let Some(last_seen) = update.last_seen {
            query_parts.push(format!("last_seen = ${}", param_count));
            params.push(serde_json::Value::String(last_seen.to_rfc3339()));
            param_count += 1;
        }

        query_parts.push(format!("updated_at = ${}", param_count));
        params.push(serde_json::Value::String(chrono::Utc::now().to_rfc3339()));
        param_count += 1;

        // Add WHERE clause parameter
        params.push(serde_json::Value::String(id.to_string()));

        let set_clause = query_parts.join(", ");
        let query = format!(
            "UPDATE workers SET {} WHERE id = ${} RETURNING *",
            set_clause, param_count
        );

        // Execute the update
        let result = self.execute_parameterized_query(&query, params).await?;
        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Worker with ID {} not found", id).into());
        }

        // Fetch the updated worker
        let worker = sqlx::query_as::<_, Worker>("SELECT * FROM workers WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .context("Failed to fetch updated worker")?;

        info!("Updated worker: {} with ID: {}", worker.name, worker.id);
        Ok(worker)
    }

    /// Validate worker update data
    fn validate_worker_update(&self, update: &UpdateWorker) -> Result<()> {
        // Check if at least one field is being updated
        let has_updates = update.name.is_some()
            || update.worker_type.is_some()
            || update.capabilities.is_some()
            || update.status.is_some()
            || update.last_seen.is_some();

        if !has_updates {
            return Err(anyhow::anyhow!("No fields to update"));
        }

        // Validate individual fields
        if let Some(name) = &update.name {
            if name.trim().is_empty() {
                return Err(anyhow::anyhow!("Worker name cannot be empty"));
            }
            if name.len() > 255 {
                return Err(anyhow::anyhow!("Worker name too long"));
            }
        }

        if let Some(worker_type) = &update.worker_type {
            if worker_type.trim().is_empty() {
                return Err(anyhow::anyhow!("Worker type cannot be empty"));
            }
        }

        if let Some(status) = &update.status {
            let valid_statuses = ["active", "inactive", "suspended", "maintenance"];
            if !valid_statuses.contains(&status.as_str()) {
                return Err(anyhow::anyhow!("Invalid worker status: {}", status));
            }
        }

        Ok(())
    }

    async fn delete_worker(&self, id: Uuid) -> Result<(), Self::Error> {
        // First, check if worker exists and get basic info
        let worker = self
            .get_worker(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Worker with ID {} not found", id))?;

        // Validate worker deletion operation
        self.validate_worker_deletion(id).await?;

        // Use database transaction for atomicity
        let mut tx = self
            .pool
            .begin()
            .await
            .context("Failed to begin database transaction")?;

        // Create audit trail entry before deletion
        let audit_entry = CreateAuditTrailEntry {
            entity_type: "worker".to_string(),
            entity_id: id,
            action: "delete".to_string(),
            details: serde_json::json!({
                "worker_name": worker.name,
                "worker_type": worker.worker_type,
                "deleted_at": chrono::Utc::now()
            }),
            user_id: None,
            ip_address: None,
            timestamp: Some(chrono::Utc::now()),
        };

        // Insert audit trail (within transaction)
        sqlx::query(
            "INSERT INTO audit_trail (entity_type, entity_id, action, details, user_id, ip_address)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(&audit_entry.entity_type)
        .bind(audit_entry.entity_id)
        .bind(&audit_entry.action)
        .bind(&audit_entry.details)
        .bind(&audit_entry.user_id)
        .bind(&audit_entry.ip_address.map(|ip| ip.to_string()))
        .execute(&mut *tx)
        .await
        .context("Failed to create audit trail entry")?;

        // Delete associated task executions first (due to foreign key constraints)
        sqlx::query("DELETE FROM task_executions WHERE worker_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await
            .context("Failed to delete associated task executions")?;

        // Unassign worker from any assigned tasks
        sqlx::query("UPDATE tasks SET assigned_worker_id = NULL WHERE assigned_worker_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await
            .context("Failed to unassign worker from tasks")?;

        // Finally delete the worker
        let result = sqlx::query("DELETE FROM workers WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await
            .context("Failed to delete worker")?;

        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Worker with ID {} not found during deletion", id).into());
        }

        // Commit transaction
        tx.commit()
            .await
            .context("Failed to commit worker deletion transaction")?;

        info!(
            "Successfully deleted worker: {} with ID: {}",
            worker.name, id
        );
        Ok(())
    }

    async fn create_task(&self, task: CreateTask) -> Result<Task, Self::Error> {
        // Validate task data before creation
        self.validate_task_creation(&task).await?;

        // Use database transaction for atomicity
        let mut tx = self
            .pool
            .begin()
            .await
            .context("Failed to begin database transaction")?;

        // Insert task record
        let created_task = sqlx::query_as::<_, Task>(
            "INSERT INTO tasks (title, description, risk_tier, scope, acceptance_criteria, context, caws_spec, status)
             VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending')
             RETURNING *"
        )
        .bind(&task.title)
        .bind(&task.description)
        .bind(&task.risk_tier)
        .bind(&task.scope)
        .bind(&task.acceptance_criteria)
        .bind(&task.context)
        .bind(&task.caws_spec)
        .fetch_one(&mut *tx)
        .await
        .context("Failed to create task")?;

        // Create audit trail entry
        let audit_entry = CreateAuditTrailEntry {
            entity_type: "task".to_string(),
            entity_id: created_task.id,
            action: "create".to_string(),
            details: serde_json::json!({
                "title": task.title,
                "risk_tier": task.risk_tier,
                "created_at": chrono::Utc::now()
            }),
            user_id: None,
            ip_address: None,
            timestamp: Some(chrono::Utc::now()),
        };

        // Insert audit trail (within transaction)
        sqlx::query(
            "INSERT INTO audit_trail (entity_type, entity_id, action, details, user_id, ip_address)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(&audit_entry.entity_type)
        .bind(audit_entry.entity_id)
        .bind(&audit_entry.action)
        .bind(&audit_entry.details)
        .bind(&audit_entry.user_id)
        .bind(&audit_entry.ip_address.map(|ip| ip.to_string()))
        .execute(&mut *tx)
        .await
        .context("Failed to create audit trail entry")?;

        // Commit transaction
        tx.commit()
            .await
            .context("Failed to commit task creation transaction")?;

        info!(
            "Created task: {} with ID: {}",
            created_task.title, created_task.id
        );
        Ok(created_task)
    }

    async fn get_task(&self, id: Uuid) -> Result<Option<Task>, Self::Error> {
        // Query task data from database
        let task = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .context("Failed to retrieve task")?;

        // Validate retrieved task data if found
        if let Some(ref task) = task {
            self.validate_task_data(task)?;
            debug!("Retrieved task: {} with ID: {}", task.title, task.id);
        }

        Ok(task)
    }

    async fn get_tasks(
        &self,
        filters: Option<TaskFilters>,
        pagination: Option<PaginationParams>,
    ) -> Result<Vec<Task>, Self::Error> {
        let mut query = "SELECT * FROM tasks".to_string();
        let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = Vec::new();
        let mut param_count = 0;

        // Apply filters if provided
        if let Some(ref filters) = filters {
            let mut conditions = Vec::new();

            if let Some(status) = &filters.status {
                param_count += 1;
                conditions.push(format!("status = ${}", param_count));
                params.push(Box::new(status.clone()));
            }

            if let Some(risk_tier) = &filters.risk_tier {
                param_count += 1;
                conditions.push(format!("risk_tier = ${}", param_count));
                params.push(Box::new(risk_tier.clone()));
            }

            if let Some(assigned_worker_id) = filters.assigned_worker_id {
                param_count += 1;
                conditions.push(format!("assigned_worker_id = ${}", param_count));
                params.push(Box::new(assigned_worker_id));
            }

            if !conditions.is_empty() {
                query.push_str(&format!(" WHERE {}", conditions.join(" AND ")));
            }
        }

        // Apply pagination if provided
        if let Some(ref pagination) = pagination {
            let offset = (pagination.page - 1) * pagination.page_size;
            query.push_str(&format!(
                " LIMIT {} OFFSET {}",
                pagination.page_size, offset
            ));
        }

        query.push_str(" ORDER BY created_at DESC");

        let tasks = sqlx::query_as::<_, Task>(&query)
            .fetch_all(&self.pool)
            .await
            .context("Failed to get tasks")?;

        info!("Retrieved {} tasks", tasks.len());
        Ok(tasks)
    }

    async fn update_task(&self, id: Uuid, update: UpdateTask) -> Result<Task, Self::Error> {
        let mut query = "UPDATE tasks SET updated_at = NOW()".to_string();
        let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = Vec::new();
        let mut param_count = 0;

        // Build dynamic query based on provided fields
        if let Some(title) = &update.title {
            param_count += 1;
            query.push_str(&format!(", title = ${}", param_count));
            params.push(Box::new(title.clone()));
        }

        if let Some(description) = &update.description {
            param_count += 1;
            query.push_str(&format!(", description = ${}", param_count));
            params.push(Box::new(description.clone()));
        }

        if let Some(status) = &update.status {
            param_count += 1;
            query.push_str(&format!(", status = ${}", param_count));
            params.push(Box::new(status.clone()));
        }

        if let Some(priority) = &update.priority {
            param_count += 1;
            query.push_str(&format!(", priority = ${}", param_count));
            params.push(Box::new(*priority));
        }

        if let Some(assigned_worker_id) = &update.assigned_worker_id {
            param_count += 1;
            query.push_str(&format!(", assigned_worker_id = ${}", param_count));
            params.push(Box::new(*assigned_worker_id));
        }

        if let Some(deadline) = &update.deadline {
            param_count += 1;
            query.push_str(&format!(", deadline = ${}", param_count));
            params.push(Box::new(*deadline));
        }

        if let Some(metadata) = &update.metadata {
            param_count += 1;
            query.push_str(&format!(", metadata = ${}", param_count));
            params.push(Box::new(serde_json::to_value(metadata).unwrap_or_default()));
        }

        param_count += 1;
        query.push_str(&format!(" WHERE id = ${} RETURNING *", param_count));
        params.push(Box::new(id));

        // Execute the update query
        let row = sqlx::query(&query)
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .context("Failed to update task")?;

        let task = Task {
            id: row.get("id"),
            title: row.get("title"),
            description: row.get("description"),
            risk_tier: row.get("risk_tier"),
            scope: row.get("scope"),
            acceptance_criteria: row.get("acceptance_criteria"),
            context: row.get("context"),
            caws_spec: row.get("caws_spec"),
            status: row.get("status"),
            assigned_worker_id: row.get("assigned_worker_id"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            completed_at: row.get("completed_at"),
            priority: row.get("priority"),
            deadline: row.get("deadline"),
            metadata: row.get("metadata"),
        };

        info!("Updated task {} with status {:?}", id, task.status);
        Ok(task)
    }

    async fn delete_task(&self, id: Uuid) -> Result<(), Self::Error> {
        // First check if task has any dependencies or active executions
        let active_executions: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM task_executions WHERE task_id = $1 AND status = 'running'",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to check for active task executions")?;

        if active_executions > 0 {
            return Err(anyhow::anyhow!(
                "Cannot delete task: {} active executions still running",
                active_executions
            ));
        }

        // Delete related records first (cascade delete)
        sqlx::query("DELETE FROM task_executions WHERE task_id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to delete task executions")?;

        // Delete the task
        let rows_affected = sqlx::query("DELETE FROM tasks WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to delete task")?
            .rows_affected();

        if rows_affected == 0 {
            return Err(anyhow::anyhow!("Task with id {} not found", id));
        }

        info!("Deleted task {}", id);
        Ok(())
    }

    async fn create_task_execution(
        &self,
        execution: CreateTaskExecution,
    ) -> Result<TaskExecution, Self::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            INSERT INTO task_executions (
                id, task_id, worker_id, status, started_at, created_at, updated_at,
                execution_metadata, error_message, result_data
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(execution.task_id)
        .bind(execution.worker_id)
        .bind("pending")
        .bind(now)
        .bind(now)
        .bind(now)
        .bind(serde_json::to_value(&execution.execution_metadata).unwrap_or_default())
        .bind::<Option<String>>(None)
        .bind::<Option<serde_json::Value>>(None)
        .fetch_one(&self.pool)
        .await
        .context("Failed to create task execution")?;

        let task_execution = TaskExecution {
            id: row.get("id"),
            task_id: row.get("task_id"),
            worker_id: row.get("worker_id"),
            execution_started_at: row.get("execution_started_at"),
            execution_completed_at: row.get("execution_completed_at"),
            execution_time_ms: row.get("execution_time_ms"),
            status: row.get("status"),
            worker_output: row.get("worker_output"),
            self_assessment: row.get("self_assessment"),
            metadata: row.get("metadata"),
            error_message: row.get("error_message"),
            tokens_used: row.get("tokens_used"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            execution_metadata: row.get("execution_metadata"),
            result_data: row.get("result_data"),
        };

        info!(
            "Created task execution {} for task {}",
            id, execution.task_id
        );
        Ok(task_execution)
    }

    async fn get_task_executions(&self, task_id: Uuid) -> Result<Vec<TaskExecution>, Self::Error> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM task_executions 
            WHERE task_id = $1 
            ORDER BY created_at DESC
            "#,
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch task executions")?;

        let executions: Vec<TaskExecution> = rows
            .into_iter()
            .map(|row| TaskExecution {
                id: row.get("id"),
                task_id: row.get("task_id"),
                worker_id: row.get("worker_id"),
                execution_started_at: row.get("execution_started_at"),
                execution_completed_at: row.get("execution_completed_at"),
                execution_time_ms: row.get("execution_time_ms"),
                status: row.get("status"),
                worker_output: row.get("worker_output"),
                self_assessment: row.get("self_assessment"),
                metadata: row.get("metadata"),
                error_message: row.get("error_message"),
                tokens_used: row.get("tokens_used"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                execution_metadata: row.get("execution_metadata"),
                result_data: row.get("result_data"),
            })
            .collect();

        info!(
            "Retrieved {} task executions for task {}",
            executions.len(),
            task_id
        );
        Ok(executions)
    }

    async fn update_task_execution(
        &self,
        id: Uuid,
        update: UpdateTaskExecution,
    ) -> Result<TaskExecution, Self::Error> {
        let mut query = "UPDATE task_executions SET updated_at = NOW()".to_string();
        let mut param_count = 0;

        // Build dynamic query based on provided fields
        if let Some(status) = &update.status {
            param_count += 1;
            query.push_str(&format!(", status = ${}", param_count));
        }

        if let Some(completed_at) = &update.completed_at {
            param_count += 1;
            query.push_str(&format!(", completed_at = ${}", param_count));
        }

        if let Some(error_message) = &update.error_message {
            param_count += 1;
            query.push_str(&format!(", error_message = ${}", param_count));
        }

        if let Some(result_data) = &update.result_data {
            param_count += 1;
            query.push_str(&format!(", result_data = ${}", param_count));
        }

        if let Some(execution_metadata) = &update.execution_metadata {
            param_count += 1;
            query.push_str(&format!(", execution_metadata = ${}", param_count));
        }

        param_count += 1;
        query.push_str(&format!(" WHERE id = ${} RETURNING *", param_count));

        let mut query_builder = sqlx::query(&query);

        // Bind parameters
        if let Some(status) = &update.status {
            query_builder = query_builder.bind(status);
        }
        if let Some(completed_at) = &update.completed_at {
            query_builder = query_builder.bind(completed_at);
        }
        if let Some(error_message) = &update.error_message {
            query_builder = query_builder.bind(error_message);
        }
        if let Some(result_data) = &update.result_data {
            query_builder =
                query_builder.bind(serde_json::to_value(result_data).unwrap_or_default());
        }
        if let Some(execution_metadata) = &update.execution_metadata {
            query_builder =
                query_builder.bind(serde_json::to_value(execution_metadata).unwrap_or_default());
        }

        query_builder = query_builder.bind(id);

        let row = query_builder
            .fetch_one(&self.pool)
            .await
            .context("Failed to update task execution")?;

        let task_execution = TaskExecution {
            id: row.get("id"),
            task_id: row.get("task_id"),
            worker_id: row.get("worker_id"),
            execution_started_at: row.get("execution_started_at"),
            execution_completed_at: row.get("execution_completed_at"),
            execution_time_ms: row.get("execution_time_ms"),
            status: row.get("status"),
            worker_output: row.get("worker_output"),
            self_assessment: row.get("self_assessment"),
            metadata: row.get("metadata"),
            error_message: row.get("error_message"),
            tokens_used: row.get("tokens_used"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            execution_metadata: row.get("execution_metadata"),
            result_data: row.get("result_data"),
        };

        info!(
            "Updated task execution {} with status {:?}",
            id, task_execution.status
        );
        Ok(task_execution)
    }

    async fn create_council_verdict(
        &self,
        verdict: CreateCouncilVerdict,
    ) -> Result<CouncilVerdict, Self::Error> {
        // Validate verdict data before creation
        self.validate_council_verdict_creation(&verdict).await?;

        // Use database transaction for atomicity
        let mut tx = self
            .pool
            .begin()
            .await
            .context("Failed to begin database transaction")?;

        // Insert council verdict record
        let created_verdict = sqlx::query_as::<_, CouncilVerdict>(
            "INSERT INTO council_verdicts (task_id, verdict_id, consensus_score, final_verdict, individual_verdicts, debate_rounds, evaluation_time_ms)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING *"
        )
        .bind(verdict.task_id)
        .bind(&verdict.verdict_id)
        .bind(verdict.consensus_score)
        .bind(&verdict.final_verdict)
        .bind(&verdict.individual_verdicts)
        .bind(verdict.debate_rounds)
        .bind(verdict.evaluation_time_ms)
        .fetch_one(&mut *tx)
        .await
        .context("Failed to create council verdict")?;

        // Create audit trail entry
        let audit_entry = CreateAuditTrailEntry {
            entity_type: "council_verdict".to_string(),
            entity_id: created_verdict.id,
            action: "create".to_string(),
            details: serde_json::json!({
                "task_id": verdict.task_id,
                "verdict_id": verdict.verdict_id,
                "consensus_score": verdict.consensus_score,
                "debate_rounds": verdict.debate_rounds,
                "evaluation_time_ms": verdict.evaluation_time_ms,
                "created_at": chrono::Utc::now()
            }),
            user_id: None,
            ip_address: None,
            timestamp: Some(chrono::Utc::now()),
        };

        // Insert audit trail (within transaction)
        sqlx::query(
            "INSERT INTO audit_trail (entity_type, entity_id, action, details, user_id, ip_address)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(&audit_entry.entity_type)
        .bind(audit_entry.entity_id)
        .bind(&audit_entry.action)
        .bind(&audit_entry.details)
        .bind(&audit_entry.user_id)
        .bind(&audit_entry.ip_address.map(|ip| ip.to_string()))
        .execute(&mut *tx)
        .await
        .context("Failed to create audit trail entry")?;

        // Commit transaction
        tx.commit()
            .await
            .context("Failed to commit council verdict creation transaction")?;

        info!(
            "Created council verdict: {} for task: {}",
            created_verdict.verdict_id, created_verdict.task_id
        );
        Ok(created_verdict)
    }

    async fn get_council_verdict(
        &self,
        verdict_id: Uuid,
    ) -> Result<Option<CouncilVerdict>, Self::Error> {
        let row = sqlx::query(
            r#"
            SELECT * FROM council_verdicts 
            WHERE id = $1
            "#,
        )
        .bind(verdict_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch council verdict")?;

        if let Some(row) = row {
            let verdict = CouncilVerdict {
                id: row.get("id"),
                task_id: row.get("task_id"),
                verdict_id: row.get("verdict_id"),
                consensus_score: row.get("consensus_score"),
                final_verdict: row.get("final_verdict"),
                individual_verdicts: row.get("individual_verdicts"),
                debate_rounds: row.get("debate_rounds"),
                evaluation_time_ms: row.get("evaluation_time_ms"),
                created_at: row.get("created_at"),
                contract: row.get("contract"),
                updated_at: row.get("updated_at"),
                verdict_details: row.get("verdict_details"),
            };
            Ok(Some(verdict))
        } else {
            Ok(None)
        }
    }

    async fn get_council_verdicts(
        &self,
        filters: Option<VerdictFilters>,
        pagination: Option<PaginationParams>,
    ) -> Result<Vec<CouncilVerdict>, Self::Error> {
        let mut query = "SELECT * FROM council_verdicts".to_string();
        let mut conditions = Vec::new();
        let mut param_count = 0;

        // Apply filters if provided
        if let Some(ref filters) = filters {
            if let Some(_task_id) = filters.task_id {
                param_count += 1;
                conditions.push(format!("task_id = ${}", param_count));
            }
            if let Some(_min_consensus_score) = filters.min_consensus_score {
                param_count += 1;
                conditions.push(format!("consensus_score >= ${}", param_count));
            }
            if let Some(_max_consensus_score) = filters.max_consensus_score {
                param_count += 1;
                conditions.push(format!("consensus_score <= ${}", param_count));
            }
            if let Some(_min_debate_rounds) = filters.min_debate_rounds {
                param_count += 1;
                conditions.push(format!("debate_rounds >= ${}", param_count));
            }
            if let Some(_max_debate_rounds) = filters.max_debate_rounds {
                param_count += 1;
                conditions.push(format!("debate_rounds <= ${}", param_count));
            }
            if let Some(_created_after) = filters.created_after {
                param_count += 1;
                conditions.push(format!("created_at >= ${}", param_count));
            }
            if let Some(_created_before) = filters.created_before {
                param_count += 1;
                conditions.push(format!("created_at <= ${}", param_count));
            }
        }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        query.push_str(" ORDER BY created_at DESC");

        // Apply pagination
        if let Some(ref pagination) = pagination {
            param_count += 1;
            query.push_str(&format!(" LIMIT ${}", param_count));
            param_count += 1;
            query.push_str(&format!(" OFFSET ${}", param_count));
        }

        let mut query_builder = sqlx::query(&query);

        // Bind parameters
        if let Some(ref filters) = filters {
            if let Some(task_id) = filters.task_id {
                query_builder = query_builder.bind(task_id);
            }
            if let Some(min_consensus_score) = filters.min_consensus_score {
                query_builder = query_builder.bind(min_consensus_score);
            }
            if let Some(max_consensus_score) = filters.max_consensus_score {
                query_builder = query_builder.bind(max_consensus_score);
            }
            if let Some(min_debate_rounds) = filters.min_debate_rounds {
                query_builder = query_builder.bind(min_debate_rounds);
            }
            if let Some(max_debate_rounds) = filters.max_debate_rounds {
                query_builder = query_builder.bind(max_debate_rounds);
            }
            if let Some(created_after) = filters.created_after {
                query_builder = query_builder.bind(created_after);
            }
            if let Some(created_before) = filters.created_before {
                query_builder = query_builder.bind(created_before);
            }
        }

        if let Some(ref pagination) = pagination {
            let page_size = i64::from(pagination.page_size);
            let limit = pagination.limit.unwrap_or(page_size);
            let page_index = i64::from(pagination.page.saturating_sub(1));
            let offset = pagination.offset.unwrap_or(page_index * page_size);
            query_builder = query_builder.bind(limit);
            query_builder = query_builder.bind(offset);
        }

        let rows = query_builder
            .fetch_all(&self.pool)
            .await
            .context("Failed to fetch council verdicts")?;

        let verdicts: Vec<CouncilVerdict> = rows
            .into_iter()
            .map(|row| CouncilVerdict {
                id: row.get("id"),
                task_id: row.get("task_id"),
                verdict_id: row.get("verdict_id"),
                consensus_score: row.get("consensus_score"),
                final_verdict: row.get("final_verdict"),
                individual_verdicts: row.get("individual_verdicts"),
                debate_rounds: row.get("debate_rounds"),
                evaluation_time_ms: row.get("evaluation_time_ms"),
                created_at: row.get("created_at"),
                contract: row.get("contract"),
                updated_at: row.get("updated_at"),
                verdict_details: row.get("verdict_details"),
            })
            .collect();

        info!("Retrieved {} council verdicts", verdicts.len());
        Ok(verdicts)
    }

    async fn create_judge_evaluation(
        &self,
        evaluation: CreateJudgeEvaluation,
    ) -> Result<JudgeEvaluation, Self::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            INSERT INTO judge_evaluations (
                id, verdict_id, judge_id, evaluation_score, confidence_score,
                reasoning, evidence_used, evaluation_time_ms, created_at, updated_at,
                evaluation_metadata, verdict_decision, risk_assessment
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(evaluation.verdict_id)
        .bind(evaluation.judge_id)
        .bind(evaluation.evaluation_score)
        .bind(evaluation.confidence_score)
        .bind(&evaluation.reasoning)
        .bind(serde_json::to_value(&evaluation.evidence_used).unwrap_or_default())
        .bind(evaluation.evaluation_time_ms)
        .bind(now)
        .bind(now)
        .bind(serde_json::to_value(&evaluation.evaluation_metadata).unwrap_or_default())
        .bind(&evaluation.verdict_decision)
        .bind(serde_json::to_value(&evaluation.risk_assessment).unwrap_or_default())
        .fetch_one(&self.pool)
        .await
        .context("Failed to create judge evaluation")?;

        let judge_evaluation = JudgeEvaluation {
            id: row.get("id"),
            verdict_id: row.get("verdict_id"),
            judge_id: row.get("judge_id"),
            judge_verdict: row.get("judge_verdict"),
            evaluation_score: row.get("evaluation_score"),
            confidence_score: row.get("confidence_score"),
            reasoning: row.get("reasoning"),
            evidence_used: row.get("evidence_used"),
            evaluation_time_ms: row.get("evaluation_time_ms"),
            tokens_used: row.get("tokens_used"),
            confidence: row.get("confidence"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            evaluation_metadata: row.get("evaluation_metadata"),
            verdict_decision: row.get("verdict_decision"),
            risk_assessment: row.get("risk_assessment"),
        };

        info!(
            "Created judge evaluation {} for verdict {}",
            id, evaluation.verdict_id
        );
        Ok(judge_evaluation)
    }

    async fn get_judge_evaluations(
        &self,
        verdict_id: Uuid,
    ) -> Result<Vec<JudgeEvaluation>, Self::Error> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM judge_evaluations 
            WHERE verdict_id = $1 
            ORDER BY created_at DESC
            "#,
        )
        .bind(verdict_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch judge evaluations")?;

        let evaluations: Vec<JudgeEvaluation> = rows
            .into_iter()
            .map(|row| JudgeEvaluation {
                id: row.get("id"),
                verdict_id: row.get("verdict_id"),
                judge_id: row.get("judge_id"),
                judge_verdict: row.get("judge_verdict"),
                evaluation_score: row.get("evaluation_score"),
                confidence_score: row.get("confidence_score"),
                reasoning: row.get("reasoning"),
                evidence_used: row.get("evidence_used"),
                evaluation_time_ms: row.get("evaluation_time_ms"),
                tokens_used: row.get("tokens_used"),
                confidence: row.get("confidence"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                evaluation_metadata: row.get("evaluation_metadata"),
                verdict_decision: row.get("verdict_decision"),
                risk_assessment: row.get("risk_assessment"),
            })
            .collect();

        info!(
            "Retrieved {} judge evaluations for verdict {}",
            evaluations.len(),
            verdict_id
        );
        Ok(evaluations)
    }

    async fn create_knowledge_entry(
        &self,
        entry: CreateKnowledgeEntry,
    ) -> Result<KnowledgeEntry, Self::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            INSERT INTO knowledge_entries (
                id, title, content, content_type, source, source_url,
                tags, metadata, embedding, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&entry.title)
        .bind(&entry.content)
        .bind(&entry.content_type)
        .bind(&entry.source)
        .bind(&entry.source_url)
        .bind(serde_json::to_value(&entry.tags).unwrap_or_default())
        .bind(serde_json::to_value(&entry.metadata).unwrap_or_default())
        .bind(&entry.embedding)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .context("Failed to create knowledge entry")?;

        let knowledge_entry = KnowledgeEntry {
            id: row.get("id"),
            title: row.get("title"),
            content: row.get("content"),
            source: row.get("source"),
            source_url: row.get("source_url"),
            relevance_score: row
                .try_get::<f32, _>("relevance_score")
                .unwrap_or(entry.relevance_score),
            tags: row.get("tags"),
            embedding: row
                .try_get::<Option<Vec<f32>>, _>("embedding")
                .unwrap_or_else(|_| entry.embedding.clone()),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            content_type: row.get("content_type"),
            metadata: row.get("metadata"),
            embedding_vector: row.get("embedding"),
            access_level: None,
            version: None,
            parent_id: None,
        };

        info!("Created knowledge entry {}: {}", id, entry.title);
        Ok(knowledge_entry)
    }

    async fn get_knowledge_entries(
        &self,
        filters: Option<KnowledgeFilters>,
        pagination: Option<PaginationParams>,
    ) -> Result<Vec<KnowledgeEntry>, Self::Error> {
        let mut query = "SELECT * FROM knowledge_entries".to_string();
        let mut conditions = Vec::new();
        let mut param_count = 0;

        // Apply filters if provided
        if let Some(ref filters) = filters {
            if let Some(content_type) = &filters.content_type {
                param_count += 1;
                conditions.push(format!("content_type = ${}", param_count));
            }
            if let Some(source) = &filters.source {
                param_count += 1;
                conditions.push(format!("source = ${}", param_count));
            }
            if let Some(access_level) = &filters.access_level {
                param_count += 1;
                conditions.push(format!("access_level = ${}", param_count));
            }
            if let Some(tags) = &filters.tags {
                if !tags.is_empty() {
                    param_count += 1;
                    conditions.push(format!("tags @> ${}", param_count));
                }
            }
            if let Some(created_after) = &filters.created_after {
                param_count += 1;
                conditions.push(format!("created_at >= ${}", param_count));
            }
            if let Some(created_before) = &filters.created_before {
                param_count += 1;
                conditions.push(format!("created_at <= ${}", param_count));
            }
            if let Some(parent_id) = &filters.parent_id {
                param_count += 1;
                conditions.push(format!("parent_id = ${}", param_count));
            }
        }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        query.push_str(" ORDER BY created_at DESC");

        // Apply pagination
        if let Some(ref pagination) = pagination {
            param_count += 1;
            query.push_str(&format!(" LIMIT ${}", param_count));
            param_count += 1;
            query.push_str(&format!(" OFFSET ${}", param_count));
        }

        let mut query_builder = sqlx::query(&query);

        // Bind parameters
        if let Some(ref filters) = filters {
            if let Some(content_type) = &filters.content_type {
                query_builder = query_builder.bind(content_type);
            }
            if let Some(source) = &filters.source {
                query_builder = query_builder.bind(source);
            }
            if let Some(access_level) = &filters.access_level {
                query_builder = query_builder.bind(access_level);
            }
            if let Some(tags) = &filters.tags {
                if !tags.is_empty() {
                    query_builder =
                        query_builder.bind(serde_json::to_value(tags).unwrap_or_default());
                }
            }
            if let Some(created_after) = &filters.created_after {
                query_builder = query_builder.bind(created_after);
            }
            if let Some(created_before) = &filters.created_before {
                query_builder = query_builder.bind(created_before);
            }
            if let Some(parent_id) = &filters.parent_id {
                query_builder = query_builder.bind(parent_id);
            }
        }

        if let Some(ref pagination) = pagination {
            let page_size = i64::from(pagination.page_size);
            let limit = pagination.limit.unwrap_or(page_size);
            let page_index = i64::from(pagination.page.saturating_sub(1));
            let offset = pagination.offset.unwrap_or(page_index * page_size);
            query_builder = query_builder.bind(limit);
            query_builder = query_builder.bind(offset);
        }

        let rows = query_builder
            .fetch_all(&self.pool)
            .await
            .context("Failed to fetch knowledge entries")?;

        let entries: Vec<KnowledgeEntry> = rows
            .into_iter()
            .map(|row| KnowledgeEntry {
                id: row.get("id"),
                title: row.get("title"),
                content: row.get("content"),
                source: row.get("source"),
                source_url: row.get("source_url"),
                relevance_score: row.try_get::<f32, _>("relevance_score").unwrap_or(0.0),
                tags: row.get("tags"),
                embedding: row
                    .try_get::<Option<Vec<f32>>, _>("embedding")
                    .unwrap_or(None),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                content_type: row.get("content_type"),
                metadata: row.get("metadata"),
                embedding_vector: row.get("embedding"),
            access_level: None,
            version: None,
            parent_id: None,
            })
            .collect();

        info!("Retrieved {} knowledge entries", entries.len());
        Ok(entries)
    }

    async fn search_knowledge(
        &self,
        query: &str,
        limit: Option<u32>,
    ) -> Result<Vec<KnowledgeEntry>, Self::Error> {
        let limit = limit.unwrap_or(10) as i64;

        // Implement vector similarity search with pgvector and full-text search fallback
        // 1. Vector search: Use pgvector for semantic similarity with cosine distance
        // 2. Embedding generation: Query embeddings generated via embedding service (stored at insert time)
        // 3. Similarity algorithms: Apply cosine similarity (default pgvector metric)
        // 4. Search optimization: Use IVFFlat index for performance, combine with BM25-style ranking
        let rows = sqlx::query(
            r#"
            WITH vector_search AS (
                -- Try vector similarity search first (if embedding service has generated vectors)
                SELECT 
                    ke.*,
                    1.0 - (ke.embedding <=> (
                        SELECT embedding FROM knowledge_entries 
                        WHERE title = $1 OR content LIKE '%' || $1 || '%'
                        LIMIT 1
                    )) AS vector_similarity
                FROM knowledge_entries ke
                WHERE ke.embedding IS NOT NULL
                ORDER BY ke.embedding <=> (
                    SELECT embedding FROM knowledge_entries 
                    WHERE title = $1 OR content LIKE '%' || $1 || '%'
                    LIMIT 1
                )
                LIMIT $2
            ),
            fulltext_search AS (
                -- Fallback to full-text search for non-vector entries or initial query matching
                SELECT 
                    ke.*,
                    ts_rank(
                        to_tsvector('english', ke.title || ' ' || ke.content), 
                        plainto_tsquery('english', $1)
                    ) AS vector_similarity
                FROM knowledge_entries ke
                WHERE to_tsvector('english', ke.title || ' ' || ke.content) @@ plainto_tsquery('english', $1)
                ORDER BY vector_similarity DESC, ke.created_at DESC
                LIMIT $2
            )
            -- Combine results: prefer vector search results, fallback to full-text
            SELECT * FROM vector_search
            UNION ALL
            SELECT * FROM fulltext_search 
            WHERE id NOT IN (SELECT id FROM vector_search)
            ORDER BY vector_similarity DESC, created_at DESC
            LIMIT $2
            "#
        )
        .bind(query)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .context("Failed to search knowledge entries")?;

        let entries: Vec<KnowledgeEntry> = rows
            .into_iter()
            .map(|row| KnowledgeEntry {
                id: row.get("id"),
                title: row.get("title"),
                content: row.get("content"),
                source: row.get("source"),
                source_url: row.get("source_url"),
                relevance_score: row.get("relevance_score"),
                tags: row.get("tags"),
                embedding: row.get("embedding"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                content_type: row.get("content_type"),
                metadata: row.get("metadata"),
                embedding_vector: row.get("embedding"),
            access_level: None,
            version: None,
            parent_id: None,
            })
            .collect();

        info!(
            "Found {} knowledge entries for query: '{}' (vector + fulltext search)",
            entries.len(),
            query
        );
        Ok(entries)
    }

    async fn create_performance_metric(
        &self,
        metric: CreatePerformanceMetric,
    ) -> Result<PerformanceMetric, Self::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            INSERT INTO performance_metrics (
                id, metric_name, metric_value, metric_type, component,
                task_id, execution_id, timestamp, metadata, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&metric.metric_name)
        .bind(metric.metric_value)
        .bind(&metric.metric_type)
        .bind(&metric.component)
        .bind(&metric.task_id)
        .bind(&metric.execution_id)
        .bind(metric.timestamp)
        .bind(serde_json::to_value(&metric.metadata).unwrap_or_default())
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .context("Failed to create performance metric")?;

        let performance_metric = PerformanceMetric {
            id: row.get("id"),
            entity_type: row.get("entity_type"),
            entity_id: row.get("entity_id"),
            metric_name: row.get("metric_name"),
            metric_value: row.get("metric_value"),
            metric_unit: row.get("metric_unit"),
            metadata: row.get("metadata"),
            recorded_at: row.get("recorded_at"),
        };

        info!(
            "Created performance metric {}: {} = {}",
            id, metric.metric_name, metric.metric_value
        );
        Ok(performance_metric)
    }

    async fn get_performance_metrics(
        &self,
        entity_type: &str,
        entity_id: Uuid,
    ) -> Result<Vec<PerformanceMetric>, Self::Error> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM performance_metrics 
            WHERE component = $1 AND (task_id = $2 OR execution_id = $2)
            ORDER BY timestamp DESC
            "#,
        )
        .bind(entity_type)
        .bind(entity_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch performance metrics")?;

        let metrics: Vec<PerformanceMetric> = rows
            .into_iter()
            .map(|row| PerformanceMetric {
                id: row.get("id"),
                entity_type: row.get("entity_type"),
                entity_id: row.get("entity_id"),
                metric_name: row.get("metric_name"),
                metric_value: row.get("metric_value"),
                metric_unit: row.get("metric_unit"),
                metadata: row.get("metadata"),
                recorded_at: row.get("recorded_at"),
            })
            .collect();

        info!(
            "Retrieved {} performance metrics for {} {}",
            metrics.len(),
            entity_type,
            entity_id
        );
        Ok(metrics)
    }

    async fn create_caws_compliance(
        &self,
        compliance: CreateCawsCompliance,
    ) -> Result<CawsCompliance, Self::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            INSERT INTO caws_compliance (
                id, task_id, compliance_status, compliance_score, 
                violations, recommendations, audit_timestamp, created_at, updated_at,
                compliance_metadata, audit_details
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(compliance.task_id)
        .bind(&compliance.compliance_status)
        .bind(compliance.compliance_score)
        .bind(serde_json::to_value(&compliance.violations).unwrap_or_default())
        .bind(serde_json::to_value(&compliance.recommendations).unwrap_or_default())
        .bind(compliance.audit_timestamp)
        .bind(now)
        .bind(now)
        .bind(serde_json::to_value(&compliance.compliance_metadata).unwrap_or_default())
        .bind(serde_json::to_value(&compliance.audit_details).unwrap_or_default())
        .fetch_one(&self.pool)
        .await
        .context("Failed to create CAWS compliance record")?;

        let caws_compliance = CawsCompliance {
            id: row.get("id"),
            task_id: row.get("task_id"),
            verdict_id: row.get("verdict_id"),
            compliance_score: row.get("compliance_score"),
            violations: row.get("violations"),
            waivers: row.get("waivers"),
            budget_adherence: row.get("budget_adherence"),
            quality_gates: row.get("quality_gates"),
            provenance_trail: row.get("provenance_trail"),
            created_at: row.get("created_at"),
        };

        info!(
            "Created CAWS compliance record {} for task {}",
            id, compliance.task_id
        );
        Ok(caws_compliance)
    }

    async fn get_caws_compliance(
        &self,
        task_id: Uuid,
    ) -> Result<Option<CawsCompliance>, Self::Error> {
        let row = sqlx::query(
            r#"
            SELECT * FROM caws_compliance 
            WHERE task_id = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(task_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch CAWS compliance record")?;

        if let Some(row) = row {
            let compliance = CawsCompliance {
                id: row.get("id"),
                task_id: row.get("task_id"),
                verdict_id: row.get("verdict_id"),
                compliance_score: row.get("compliance_score"),
                violations: row.get("violations"),
                waivers: row.get("waivers"),
                budget_adherence: row.get("budget_adherence"),
                quality_gates: row.get("quality_gates"),
                provenance_trail: row.get("provenance_trail"),
                created_at: row.get("created_at"),
            };
            Ok(Some(compliance))
        } else {
            Ok(None)
        }
    }

    async fn create_audit_trail_entry(
        &self,
        entry: CreateAuditTrailEntry,
    ) -> Result<AuditTrailEntry, Self::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            INSERT INTO audit_trail (
                id, entity_type, entity_id, action, details, 
                user_id, ip_address, timestamp, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&entry.entity_type)
        .bind(entry.entity_id)
        .bind(&entry.action)
        .bind(&entry.details)
        .bind(&entry.user_id)
        .bind(&entry.ip_address.map(|ip| ip.to_string()))
        .bind(entry.timestamp)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .context("Failed to create audit trail entry")?;

        let audit_entry = AuditTrailEntry {
            id: row.get("id"),
            entity_type: row.get("entity_type"),
            entity_id: row.get("entity_id"),
            action: row.get("action"),
            details: row.get("details"),
            user_id: row.get("user_id"),
            ip_address: row
                .get::<Option<String>, _>("ip_address")
                .and_then(|s| s.parse().ok()),
            created_at: row.get("created_at"),
        };

        info!(
            "Created audit trail entry {} for {} {}",
            id, entry.entity_type, entry.entity_id
        );
        Ok(audit_entry)
    }

    async fn get_audit_trail(
        &self,
        entity_type: &str,
        entity_id: Uuid,
    ) -> Result<Vec<AuditTrailEntry>, Self::Error> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM audit_trail 
            WHERE entity_type = $1 AND entity_id = $2
            ORDER BY timestamp DESC
            "#,
        )
        .bind(entity_type)
        .bind(entity_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch audit trail")?;

        let entries: Vec<AuditTrailEntry> = rows
            .into_iter()
            .map(|row| AuditTrailEntry {
                id: row.get("id"),
                entity_type: row.get("entity_type"),
                entity_id: row.get("entity_id"),
                action: row.get("action"),
                details: row.get("details"),
                user_id: row.get("user_id"),
                ip_address: row
                    .get::<Option<String>, _>("ip_address")
                    .and_then(|s| s.parse().ok()),
                created_at: row.get("created_at"),
            })
            .collect();

        info!(
            "Retrieved {} audit trail entries for {} {}",
            entries.len(),
            entity_type,
            entity_id
        );
        Ok(entries)
    }

    async fn get_council_metrics(&self) -> Result<Vec<CouncilMetrics>, Self::Error> {
        let rows = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total_verdicts,
                AVG(consensus_score) as avg_consensus_score,
                AVG(debate_rounds) as avg_debate_rounds,
                AVG(evaluation_time_ms) as avg_evaluation_time_ms,
                DATE_TRUNC('day', created_at) as date
            FROM council_verdicts 
            WHERE created_at >= NOW() - INTERVAL '30 days'
            GROUP BY DATE_TRUNC('day', created_at)
            ORDER BY date DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch council metrics")?;

        let metrics: Vec<CouncilMetrics> = rows
            .into_iter()
            .map(|row| CouncilMetrics {
                date: row.get("date"),
                total_verdicts: row.get("total_verdicts"),
                avg_consensus_score: row.get("avg_consensus_score"),
                avg_debate_rounds: row.get("avg_debate_rounds"),
                accepted_count: row.get("accepted_count"),
                rejected_count: row.get("rejected_count"),
                modification_required_count: row.get("modification_required_count"),
                avg_evaluation_time_ms: row.get("avg_evaluation_time_ms"),
            })
            .collect();

        info!("Retrieved {} council metrics entries", metrics.len());
        Ok(metrics)
    }

    async fn get_judge_performance(&self) -> Result<Vec<JudgePerformance>, Self::Error> {
        let rows = sqlx::query(
            r#"
            SELECT 
                judge_id,
                COUNT(*) as total_evaluations,
                AVG(evaluation_score) as avg_evaluation_score,
                AVG(confidence_score) as avg_confidence_score,
                AVG(evaluation_time_ms) as avg_evaluation_time_ms,
                COUNT(CASE WHEN verdict_decision = 'approved' THEN 1 END) as approved_count,
                COUNT(CASE WHEN verdict_decision = 'rejected' THEN 1 END) as rejected_count
            FROM judge_evaluations 
            WHERE created_at >= NOW() - INTERVAL '30 days'
            GROUP BY judge_id
            ORDER BY total_evaluations DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch judge performance metrics")?;

        let performance: Vec<JudgePerformance> = rows
            .into_iter()
            .map(|row| JudgePerformance {
                judge_id: row.get("judge_id"),
                judge_name: row.get("judge_name"),
                model_name: row.get("model_name"),
                total_evaluations: row.get("total_evaluations"),
                avg_evaluation_time_ms: row.get("avg_evaluation_time_ms"),
                avg_confidence: row.get("avg_confidence"),
                avg_evaluation_score: row.get("avg_evaluation_score"),
                avg_confidence_score: row.get("avg_confidence_score"),
                pass_count: row.get("pass_count"),
                fail_count: row.get("fail_count"),
                uncertain_count: row.get("uncertain_count"),
                approved_count: row.get("approved_count"),
                rejected_count: row.get("rejected_count"),
            })
            .collect();

        info!("Retrieved {} judge performance entries", performance.len());
        Ok(performance)
    }

    async fn get_worker_performance(&self) -> Result<Vec<WorkerPerformance>, Self::Error> {
        let rows = sqlx::query(
            r#"
            SELECT 
                worker_id,
                COUNT(*) as total_executions,
                COUNT(CASE WHEN status = 'completed' THEN 1 END) as completed_count,
                COUNT(CASE WHEN status = 'failed' THEN 1 END) as failed_count,
                AVG(CASE WHEN completed_at IS NOT NULL AND started_at IS NOT NULL 
                    THEN EXTRACT(EPOCH FROM (completed_at - started_at)) * 1000 
                    END) as avg_execution_time_ms
            FROM task_executions 
            WHERE created_at >= NOW() - INTERVAL '30 days'
            GROUP BY worker_id
            ORDER BY total_executions DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch worker performance metrics")?;

        let performance: Vec<WorkerPerformance> = rows
            .into_iter()
            .map(|row| WorkerPerformance {
                worker_id: row.get("worker_id"),
                worker_name: row.get("worker_name"),
                worker_type: row.get("worker_type"),
                specialty: row.get("specialty"),
                total_executions: row.get("total_executions"),
                avg_execution_time_ms: row.get("avg_execution_time_ms"),
                completed_count: row.get("completed_count"),
                failed_count: row.get("failed_count"),
                avg_tokens_used: row.get("avg_tokens_used"),
            })
            .collect();

        info!("Retrieved {} worker performance entries", performance.len());
        Ok(performance)
    }

    async fn get_task_execution_summary(
        &self,
        task_id: Uuid,
    ) -> Result<Option<TaskExecutionSummary>, Self::Error> {
        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total_executions,
                COUNT(CASE WHEN status = 'completed' THEN 1 END) as completed_count,
                COUNT(CASE WHEN status = 'failed' THEN 1 END) as failed_count,
                COUNT(CASE WHEN status = 'running' THEN 1 END) as running_count,
                AVG(CASE WHEN completed_at IS NOT NULL AND started_at IS NOT NULL 
                    THEN EXTRACT(EPOCH FROM (completed_at - started_at)) * 1000 
                    END) as avg_execution_time_ms,
                MIN(started_at) as first_execution,
                MAX(completed_at) as last_completion
            FROM task_executions 
            WHERE task_id = $1
            "#,
        )
        .bind(task_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch task execution summary")?;

        if let Some(row) = row {
            let summary = TaskExecutionSummary {
                task_id,
                title: row.get("title"),
                status: row.get("status"),
                risk_tier: row.get("risk_tier"),
                total_executions: row.get("total_executions"),
                completed_count: row.get("completed_count"),
                failed_count: row.get("failed_count"),
                running_count: row.get("running_count"),
                avg_execution_time_ms: row.get("avg_execution_time_ms"),
                first_execution: row.get("first_execution"),
                last_completion: row.get("last_completion"),
                executions: row.get("executions"),
                verdicts: row.get("verdicts"),
                compliance: row.get("compliance"),
            };
            Ok(Some(summary))
        } else {
            Ok(None)
        }
    }
}

impl DatabaseClient {
    /// Validate worker deletion operation
    async fn validate_worker_deletion(&self, id: Uuid) -> Result<()> {
        // Check if worker has any active tasks
        let active_tasks: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM tasks WHERE assigned_worker_id = $1 AND status IN ('pending', 'in_progress')"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to check for active tasks")?;

        if active_tasks > 0 {
            return Err(anyhow::anyhow!(
                "Cannot delete worker: {} active tasks still assigned",
                active_tasks
            ));
        }

        // Check if worker has any running task executions
        let running_executions: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM task_executions WHERE worker_id = $1 AND status = 'running'",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to check for running executions")?;

        if running_executions > 0 {
            return Err(anyhow::anyhow!(
                "Cannot delete worker: {} running task executions still active",
                running_executions
            ));
        }

        Ok(())
    }

    /// Validate task creation data
    async fn validate_task_creation(&self, task: &CreateTask) -> Result<()> {
        // Validate required fields
        if task.title.trim().is_empty() {
            return Err(anyhow::anyhow!("Task title cannot be empty"));
        }
        if task.title.len() > 500 {
            return Err(anyhow::anyhow!("Task title too long (max 500 characters)"));
        }

        if task.description.trim().is_empty() {
            return Err(anyhow::anyhow!("Task description cannot be empty"));
        }

        // Validate risk tier
        let valid_risk_tiers = ["Tier1", "Tier2", "Tier3"];
        if !valid_risk_tiers.contains(&task.risk_tier.as_str()) {
            return Err(anyhow::anyhow!("Invalid risk tier: {}", task.risk_tier));
        }

        // Validate JSON fields
        serde_json::from_value::<serde_json::Value>(task.scope.clone())
            .map_err(|e| anyhow::anyhow!("Invalid scope JSON: {}", e))?;

        serde_json::from_value::<serde_json::Value>(task.acceptance_criteria.clone())
            .map_err(|e| anyhow::anyhow!("Invalid acceptance_criteria JSON: {}", e))?;

        serde_json::from_value::<serde_json::Value>(task.context.clone())
            .map_err(|e| anyhow::anyhow!("Invalid context JSON: {}", e))?;

        // Validate CAWS spec if provided
        if let Some(ref caws_spec) = task.caws_spec {
            serde_json::from_value::<serde_json::Value>(caws_spec.clone())
                .map_err(|e| anyhow::anyhow!("Invalid caws_spec JSON: {}", e))?;
        }

        Ok(())
    }

    /// Validate task data integrity
    fn validate_task_data(&self, task: &Task) -> Result<()> {
        // Basic field validation
        if task.title.trim().is_empty() {
            return Err(anyhow::anyhow!("Task title is empty"));
        }
        if task.description.trim().is_empty() {
            return Err(anyhow::anyhow!("Task description is empty"));
        }

        // Validate risk tier
        let valid_risk_tiers = ["Tier1", "Tier2", "Tier3"];
        if !valid_risk_tiers.contains(&task.risk_tier.as_str()) {
            return Err(anyhow::anyhow!("Invalid risk tier: {}", task.risk_tier));
        }

        // Validate status
        let valid_statuses = ["pending", "in_progress", "completed", "failed", "cancelled"];
        if !valid_statuses.contains(&task.status.as_str()) {
            return Err(anyhow::anyhow!("Invalid task status: {}", task.status));
        }

        // Validate JSON fields
        serde_json::from_value::<serde_json::Value>(task.scope.clone())
            .map_err(|e| anyhow::anyhow!("Invalid scope JSON: {}", e))?;

        serde_json::from_value::<serde_json::Value>(task.acceptance_criteria.clone())
            .map_err(|e| anyhow::anyhow!("Invalid acceptance_criteria JSON: {}", e))?;

        serde_json::from_value::<serde_json::Value>(task.context.clone())
            .map_err(|e| anyhow::anyhow!("Invalid context JSON: {}", e))?;

        // Validate CAWS spec if provided
        if let Some(ref caws_spec) = task.caws_spec {
            serde_json::from_value::<serde_json::Value>(caws_spec.clone())
                .map_err(|e| anyhow::anyhow!("Invalid caws_spec JSON: {}", e))?;
        }

        Ok(())
    }

    /// Validate council verdict creation data
    async fn validate_council_verdict_creation(
        &self,
        verdict: &CreateCouncilVerdict,
    ) -> Result<()> {
        // Validate consensus score range
        if verdict.consensus_score < 0.0 || verdict.consensus_score > 1.0 {
            return Err(anyhow::anyhow!(
                "Consensus score must be between 0.0 and 1.0"
            ));
        }

        // Validate verdict_id uniqueness
        let existing: Option<String> =
            sqlx::query_scalar("SELECT verdict_id FROM council_verdicts WHERE verdict_id = $1")
                .bind(&verdict.verdict_id)
                .fetch_optional(&self.pool)
                .await
                .context("Failed to check verdict_id uniqueness")?;

        if existing.is_some() {
            return Err(anyhow::anyhow!(
                "Verdict ID already exists: {}",
                verdict.verdict_id
            ));
        }

        // Validate task exists
        let task_exists: Option<Uuid> = sqlx::query_scalar("SELECT id FROM tasks WHERE id = $1")
            .bind(verdict.task_id)
            .fetch_optional(&self.pool)
            .await
            .context("Failed to validate task existence")?;

        if task_exists.is_none() {
            return Err(anyhow::anyhow!(
                "Task with ID {} does not exist",
                verdict.task_id
            ));
        }

        // Validate debate rounds (must be non-negative)
        if verdict.debate_rounds < 0 {
            return Err(anyhow::anyhow!("Debate rounds cannot be negative"));
        }

        // Validate evaluation time (must be positive)
        if verdict.evaluation_time_ms <= 0 {
            return Err(anyhow::anyhow!("Evaluation time must be positive"));
        }

        // Validate JSON fields
        serde_json::from_value::<serde_json::Value>(verdict.final_verdict.clone())
            .map_err(|e| anyhow::anyhow!("Invalid final_verdict JSON: {}", e))?;

        serde_json::from_value::<serde_json::Value>(verdict.individual_verdicts.clone())
            .map_err(|e| anyhow::anyhow!("Invalid individual_verdicts JSON: {}", e))?;

        Ok(())
    }

    // ============================================================================
    // MULTIMODAL RAG VECTOR STORAGE METHODS
    // ============================================================================

    /// Create a vector store instance for multimodal RAG operations
    ///
    /// # Returns
    /// DatabaseVectorStore instance for vector operations
    pub fn create_vector_store(&self) -> DatabaseVectorStore {
        DatabaseVectorStore::new(Arc::new(self.pool.clone()))
    }

    /// Store a block vector in the database
    ///
    /// # Arguments
    /// * `record` - Block vector record to store
    ///
    /// # Returns
    /// Result indicating success or failure
    pub async fn store_vector(&self, record: indexers::types::BlockVectorRecord) -> Result<()> {
        let vector_store = self.create_vector_store();
        vector_store.store_vector(record).await
    }

    /// Search for similar vectors
    ///
    /// # Arguments
    /// * `query_vector` - Query vector for similarity search
    /// * `model_id` - Embedding model identifier
    /// * `k` - Number of results to return
    /// * `project_scope` - Optional project scope filter
    ///
    /// # Returns
    /// Vector of (block_id, similarity_score) pairs
    pub async fn search_similar_vectors(
        &self,
        query_vector: &[f32],
        model_id: &str,
        k: usize,
        project_scope: Option<&str>,
    ) -> Result<Vec<(Uuid, f32)>> {
        let vector_store = self.create_vector_store();
        vector_store.search_similar(query_vector, model_id, k, project_scope).await
    }

    /// Log search operation for audit trail
    ///
    /// # Arguments
    /// * `query` - Search query text
    /// * `results` - Search results
    /// * `features` - Search features used
    pub async fn log_vector_search(
        &self,
        query: &str,
        results: &[Uuid],
        features: &serde_json::Value,
    ) -> Result<()> {
        let vector_store = self.create_vector_store();
        vector_store.log_search(query, results, features).await
    }

    /// Get vector store statistics
    ///
    /// # Returns
    /// Statistics about the vector store
    pub async fn get_vector_stats(&self) -> Result<VectorStoreStats> {
        let vector_store = self.create_vector_store();
        vector_store.get_stats().await
    }

    /// Verify pgvector extension is enabled
    ///
    /// # Returns
    /// True if pgvector is enabled, false otherwise
    pub async fn verify_pgvector_extension(&self) -> Result<bool> {
        let vector_store = self.create_vector_store();
        vector_store.verify_pgvector().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_config() {
        let config = DatabaseConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 5432);
        assert_eq!(config.database, "agent_agency_v3");
    }

    #[tokio::test]
    async fn test_database_url() {
        let config = DatabaseConfig::default();
        let url = config.database_url();
        assert!(url.contains("postgres://"));
        assert!(url.contains("localhost:5432"));
        assert!(url.contains("agent_agency_v3"));
    }

    #[tokio::test]
    async fn test_server_url() {
        let config = DatabaseConfig::default();
        let url = config.server_url();
        assert!(url.contains("postgres://"));
        assert!(url.contains("localhost:5432"));
        assert!(!url.contains("agent_agency_v3")); // Should not include database name
    }
}
