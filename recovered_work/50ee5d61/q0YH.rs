//! Database performance optimization and monitoring
//!
//! Provides query optimization, index management, read/write splitting,
//! and comprehensive performance monitoring for production databases.

use crate::{DatabaseClient, DatabaseConfig};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Database optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseOptimizationConfig {
    /// Enable query performance monitoring
    pub enable_query_monitoring: bool,
    /// Slow query threshold (milliseconds)
    pub slow_query_threshold_ms: u64,
    /// Enable automatic index suggestions
    pub enable_index_suggestions: bool,
    /// Enable read/write splitting
    pub enable_read_write_splitting: bool,
    /// Read replica connection strings
    pub read_replicas: Vec<String>,
    /// Query cache size (number of cached query plans)
    pub query_cache_size: usize,
    /// Enable query plan analysis
    pub enable_query_plan_analysis: bool,
    /// Performance monitoring interval (seconds)
    pub monitoring_interval_seconds: u64,
}

impl Default for DatabaseOptimizationConfig {
    fn default() -> Self {
        Self {
            enable_query_monitoring: true,
            slow_query_threshold_ms: 1000, // 1 second
            enable_index_suggestions: true,
            enable_read_write_splitting: false,
            read_replicas: Vec::new(),
            query_cache_size: 1000,
            enable_query_plan_analysis: true,
            monitoring_interval_seconds: 300, // 5 minutes
        }
    }
}

/// Query performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMetrics {
    pub query_hash: String,
    pub query_text: String,
    pub execution_count: u64,
    pub total_execution_time_ms: u64,
    pub average_execution_time_ms: f64,
    pub min_execution_time_ms: u64,
    pub max_execution_time_ms: u64,
    pub last_executed: DateTime<Utc>,
    pub slow_execution_count: u64,
}

/// Index recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexRecommendation {
    pub table_name: String,
    pub column_name: String,
    pub index_type: String,
    pub estimated_improvement: f64,
    pub query_patterns: Vec<String>,
    pub priority: IndexPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum IndexPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// Read/write split database client
#[derive(Debug)]
pub struct ReadWriteSplitClient {
    write_client: DatabaseClient,
    read_clients: Vec<DatabaseClient>,
    read_client_index: std::sync::atomic::AtomicUsize,
}

impl ReadWriteSplitClient {
    pub fn new(write_client: DatabaseClient, read_clients: Vec<DatabaseClient>) -> Self {
        Self {
            write_client,
            read_clients,
            read_client_index: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Get a read client using round-robin
    pub fn get_read_client(&self) -> &DatabaseClient {
        if self.read_clients.is_empty() {
            &self.write_client
        } else {
            let index = self.read_client_index.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % self.read_clients.len();
            &self.read_clients[index]
        }
    }

    /// Get the write client
    pub fn get_write_client(&self) -> &DatabaseClient {
        &self.write_client
    }

    /// Execute a read query (SELECT)
    pub async fn execute_read<T, F, Fut>(&self, operation: F) -> Result<T>
    where
        F: FnOnce(&DatabaseClient) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let client = self.get_read_client();
        operation(client).await
    }

    /// Execute a write query (INSERT, UPDATE, DELETE)
    pub async fn execute_write<T, F, Fut>(&self, operation: F) -> Result<T>
    where
        F: FnOnce(&DatabaseClient) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        operation(&self.write_client).await
    }
}

/// Database performance monitor
pub struct DatabasePerformanceMonitor {
    config: DatabaseOptimizationConfig,
    query_metrics: Arc<RwLock<HashMap<String, QueryMetrics>>>,
    slow_queries: Arc<RwLock<Vec<QueryMetrics>>>,
}

impl DatabasePerformanceMonitor {
    pub fn new(config: DatabaseOptimizationConfig) -> Self {
        Self {
            config,
            query_metrics: Arc::new(RwLock::new(HashMap::new())),
            slow_queries: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Record query execution metrics
    pub async fn record_query_execution(
        &self,
        query_text: &str,
        execution_time_ms: u64,
    ) {
        let query_hash = self.hash_query(query_text);

        let mut metrics = self.query_metrics.write().await;
        let entry = metrics.entry(query_hash.clone()).or_insert_with(|| QueryMetrics {
            query_hash: query_hash.clone(),
            query_text: query_text.to_string(),
            execution_count: 0,
            total_execution_time_ms: 0,
            average_execution_time_ms: 0.0,
            min_execution_time_ms: u64::MAX,
            max_execution_time_ms: 0,
            last_executed: Utc::now(),
            slow_execution_count: 0,
        });

        entry.execution_count += 1;
        entry.total_execution_time_ms += execution_time_ms;
        entry.average_execution_time_ms = entry.total_execution_time_ms as f64 / entry.execution_count as f64;
        entry.min_execution_time_ms = entry.min_execution_time_ms.min(execution_time_ms);
        entry.max_execution_time_ms = entry.max_execution_time_ms.max(execution_time_ms);
        entry.last_executed = Utc::now();

        if execution_time_ms >= self.config.slow_query_threshold_ms {
            entry.slow_execution_count += 1;

            // Record in slow queries log
            let mut slow_queries = self.slow_queries.write().await;
            slow_queries.push(entry.clone());

            // Keep only recent slow queries (last 1000)
            if slow_queries.len() > 1000 {
                slow_queries.remove(0);
            }

            warn!("Slow query detected: {}ms - {}", execution_time_ms, query_text);
        }
    }

    /// Get performance metrics for all queries
    pub async fn get_query_metrics(&self) -> HashMap<String, QueryMetrics> {
        self.query_metrics.read().await.clone()
    }

    /// Get slow queries
    pub async fn get_slow_queries(&self, limit: usize) -> Vec<QueryMetrics> {
        let slow_queries = self.slow_queries.read().await;
        slow_queries.iter().rev().take(limit).cloned().collect()
    }

    /// Generate index recommendations
    pub async fn generate_index_recommendations(&self) -> Vec<IndexRecommendation> {
        let metrics = self.query_metrics.read().await;
        let mut recommendations = Vec::new();

        for (_hash, metric) in metrics.iter() {
            if metric.execution_count > 10 && metric.average_execution_time_ms > 100.0 {
                // Analyze query for potential indexes
                let recs = self.analyze_query_for_indexes(&metric.query_text);
                recommendations.extend(recs);
            }
        }

        recommendations.sort_by(|a, b| b.priority.cmp(&a.priority));
        recommendations
    }

    /// Analyze query text for index opportunities
    fn analyze_query_for_indexes(&self, query_text: &str) -> Vec<IndexRecommendation> {
        let mut recommendations = Vec::new();
        let query_lower = query_text.to_lowercase();

        // Look for WHERE clauses that might benefit from indexes
        if let Some(where_clause) = self.extract_where_clause(&query_lower) {
            // Check for common patterns
            if where_clause.contains("status =") {
                recommendations.push(IndexRecommendation {
                    table_name: self.extract_table_name(&query_lower).unwrap_or_else(|| "unknown".to_string()),
                    column_name: "status".to_string(),
                    index_type: "btree".to_string(),
                    estimated_improvement: 0.8,
                    query_patterns: vec!["status filtering".to_string()],
                    priority: IndexPriority::High,
                });
            }

            if where_clause.contains("created_at >") || where_clause.contains("created_at <") {
                recommendations.push(IndexRecommendation {
                    table_name: self.extract_table_name(&query_lower).unwrap_or_else(|| "unknown".to_string()),
                    column_name: "created_at".to_string(),
                    index_type: "btree".to_string(),
                    estimated_improvement: 0.7,
                    query_patterns: vec!["time range queries".to_string()],
                    priority: IndexPriority::High,
                });
            }

            if where_clause.contains("id =") {
                recommendations.push(IndexRecommendation {
                    table_name: self.extract_table_name(&query_lower).unwrap_or_else(|| "unknown".to_string()),
                    column_name: "id".to_string(),
                    index_type: "btree".to_string(),
                    estimated_improvement: 0.9,
                    query_patterns: vec!["primary key lookups".to_string()],
                    priority: IndexPriority::Critical,
                });
            }
        }

        recommendations
    }

    /// Extract WHERE clause from query
    fn extract_where_clause(&self, query: &str) -> Option<String> {
        if let Some(where_pos) = query.find("where") {
            let after_where = &query[where_pos + 5..];
            if let Some(end_pos) = after_where.find("order by").or_else(|| after_where.find("group by")).or_else(|| after_where.find("limit")) {
                Some(after_where[..end_pos].trim().to_string())
            } else {
                Some(after_where.trim().to_string())
            }
        } else {
            None
        }
    }

    /// Extract table name from query
    fn extract_table_name(&self, query: &str) -> Option<String> {
        if let Some(from_pos) = query.find("from") {
            let after_from = &query[from_pos + 4..];
            if let Some(end_pos) = after_from.find("where").or_else(|| after_from.find("order")).or_else(|| after_from.find("limit")) {
                let table_part = after_from[..end_pos].trim();
                // Extract first word (table name)
                table_part.split_whitespace().next().map(|s| s.to_string())
            } else {
                after_from.split_whitespace().next().map(|s| s.to_string())
            }
        } else {
            None
        }
    }

    /// Hash query for metrics tracking
    fn hash_query(&self, query: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        query.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

/// Database index manager
pub struct DatabaseIndexManager {
    client: DatabaseClient,
}

impl DatabaseIndexManager {
    pub fn new(client: DatabaseClient) -> Self {
        Self { client }
    }

    /// Create recommended indexes
    pub async fn create_recommended_indexes(&self, recommendations: &[IndexRecommendation]) -> Result<Vec<String>> {
        let mut created_indexes = Vec::new();

        for rec in recommendations {
            if rec.priority >= IndexPriority::High {
                let index_name = format!("idx_{}_{}", rec.table_name, rec.column_name);
                let create_sql = match rec.index_type.as_str() {
                    "btree" => format!("CREATE INDEX CONCURRENTLY {} ON {} ({})", index_name, rec.table_name, rec.column_name),
                    "hash" => format!("CREATE INDEX CONCURRENTLY {} ON {} USING hash ({})", index_name, rec.table_name, rec.column_name),
                    _ => continue,
                };

                match self.client.execute_parameterized_query(&create_sql, vec![]).await {
                    Ok(_) => {
                        info!("Created index: {} (estimated improvement: {:.1}%)", index_name, rec.estimated_improvement * 100.0);
                        created_indexes.push(index_name);
                    }
                    Err(e) => {
                        warn!("Failed to create index {}: {}", index_name, e);
                    }
                }
            }
        }

        Ok(created_indexes)
    }

    /// Analyze existing indexes and provide optimization suggestions
    pub async fn analyze_indexes(&self) -> Result<Vec<String>> {
        let query = r#"
            SELECT
                schemaname,
                tablename,
                indexname,
                idx_scan,
                idx_tup_read,
                idx_tup_fetch
            FROM pg_stat_user_indexes
            ORDER BY idx_scan DESC
        "#;

        let rows = sqlx::query(query).fetch_all(&*self.client.pool()).await?;
        let mut suggestions = Vec::new();

        for row in rows {
            let index_name: String = row.get("indexname");
            let scan_count: Option<i64> = row.get("idx_scan");

            if scan_count.unwrap_or(0) == 0 {
                suggestions.push(format!("Index '{}' is unused and could be removed", index_name));
            }
        }

        Ok(suggestions)
    }

    /// Get table statistics for optimization
    pub async fn get_table_statistics(&self) -> Result<HashMap<String, TableStats>> {
        let query = r#"
            SELECT
                schemaname,
                tablename,
                n_tup_ins,
                n_tup_upd,
                n_tup_del,
                n_live_tup,
                n_dead_tup
            FROM pg_stat_user_tables
        "#;

        let rows = sqlx::query(query).fetch_all(&*self.client.pool()).await?;
        let mut stats = HashMap::new();

        for row in rows {
            let table_name: String = row.get("tablename");
            let table_stats = TableStats {
                inserts: row.get::<Option<i64>, _>("n_tup_ins").unwrap_or(0),
                updates: row.get::<Option<i64>, _>("n_tup_upd").unwrap_or(0),
                deletes: row.get::<Option<i64>, _>("n_tup_del").unwrap_or(0),
                live_rows: row.get::<Option<i64>, _>("n_live_tup").unwrap_or(0),
                dead_rows: row.get::<Option<i64>, _>("n_dead_tup").unwrap_or(0),
            };
            stats.insert(table_name, table_stats);
        }

        Ok(stats)
    }
}

/// Table statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableStats {
    pub inserts: i64,
    pub updates: i64,
    pub deletes: i64,
    pub live_rows: i64,
    pub dead_rows: i64,
}

/// Database optimization manager
pub struct DatabaseOptimizationManager {
    client: DatabaseClient,
    monitor: Arc<DatabasePerformanceMonitor>,
    index_manager: DatabaseIndexManager,
    config: DatabaseOptimizationConfig,
}

impl DatabaseOptimizationManager {
    pub fn new(
        client: DatabaseClient,
        config: DatabaseOptimizationConfig,
    ) -> Self {
        let monitor = Arc::new(DatabasePerformanceMonitor::new(config.clone()));
        let index_manager = DatabaseIndexManager::new(client.clone());

        Self {
            client,
            monitor,
            index_manager,
            config,
        }
    }

    /// Initialize database optimizations
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing database optimizations...");

        // Create performance monitoring tables if they don't exist
        self.create_monitoring_tables().await?;

        // Start background monitoring
        self.start_monitoring().await?;

        // Run initial analysis
        self.run_initial_analysis().await?;

        Ok(())
    }

    /// Create monitoring tables
    async fn create_monitoring_tables(&self) -> Result<()> {
        let create_query_metrics = r#"
            CREATE TABLE IF NOT EXISTS query_metrics (
                query_hash VARCHAR(64) PRIMARY KEY,
                query_text TEXT NOT NULL,
                execution_count BIGINT DEFAULT 0,
                total_execution_time_ms BIGINT DEFAULT 0,
                average_execution_time_ms DOUBLE PRECISION DEFAULT 0,
                min_execution_time_ms BIGINT DEFAULT 0,
                max_execution_time_ms BIGINT DEFAULT 0,
                last_executed TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                slow_execution_count BIGINT DEFAULT 0,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
        "#;

        let create_index_recommendations = r#"
            CREATE TABLE IF NOT EXISTS index_recommendations (
                id SERIAL PRIMARY KEY,
                table_name VARCHAR(255) NOT NULL,
                column_name VARCHAR(255) NOT NULL,
                index_type VARCHAR(50) NOT NULL,
                estimated_improvement DOUBLE PRECISION,
                query_patterns JSONB,
                priority VARCHAR(20) NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                applied BOOLEAN DEFAULT FALSE
            )
        "#;

        self.client.execute_parameterized_query(create_query_metrics, vec![]).await?;
        self.client.execute_parameterized_query(create_index_recommendations, vec![]).await?;

        Ok(())
    }

    /// Start background monitoring
    async fn start_monitoring(&self) -> Result<()> {
        let monitor = self.monitor.clone();
        let interval = self.config.monitoring_interval_seconds;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(Duration::from_secs(interval));

            loop {
                interval_timer.tick().await;

                // Generate and log recommendations
                let recommendations = monitor.generate_index_recommendations().await;
                if !recommendations.is_empty() {
                    info!("Database optimization recommendations available: {}", recommendations.len());
                    for rec in recommendations.iter().take(3) {
                        info!("Consider adding {} index on {}.{} (priority: {:?})",
                            rec.index_type, rec.table_name, rec.column_name, rec.priority);
                    }
                }

                // Log slow queries
                let slow_queries = monitor.get_slow_queries(5).await;
                if !slow_queries.is_empty() {
                    warn!("Recent slow queries detected: {}", slow_queries.len());
                }
            }
        });

        Ok(())
    }

    /// Run initial database analysis
    async fn run_initial_analysis(&self) -> Result<()> {
        info!("Running initial database analysis...");

        // Analyze table statistics
        if let Ok(table_stats) = self.index_manager.get_table_statistics().await {
            for (table_name, stats) in table_stats.iter() {
                let bloat_ratio = if stats.live_rows > 0 {
                    stats.dead_rows as f64 / stats.live_rows as f64
                } else {
                    0.0
                };

                if bloat_ratio > 0.2 {
                    warn!("Table '{}' has high bloat ratio: {:.1}% - consider VACUUM", table_name, bloat_ratio * 100.0);
                }

                debug!("Table '{}' stats: {} live rows, {} dead rows", table_name, stats.live_rows, stats.dead_rows);
            }
        }

        // Analyze existing indexes
        if let Ok(index_suggestions) = self.index_manager.analyze_indexes().await {
            for suggestion in index_suggestions {
                info!("Index optimization: {}", suggestion);
            }
        }

        Ok(())
    }

    /// Get performance metrics
    pub async fn get_performance_metrics(&self) -> HashMap<String, QueryMetrics> {
        self.monitor.get_query_metrics().await
    }

    /// Get slow queries
    pub async fn get_slow_queries(&self, limit: usize) -> Vec<QueryMetrics> {
        self.monitor.get_slow_queries(limit).await
    }

    /// Get index recommendations
    pub async fn get_index_recommendations(&self) -> Vec<IndexRecommendation> {
        self.monitor.generate_index_recommendations().await
    }

    /// Apply index recommendations
    pub async fn apply_index_recommendations(&self, recommendations: &[IndexRecommendation]) -> Result<Vec<String>> {
        self.index_manager.create_recommended_indexes(recommendations).await
    }

    /// Get table statistics
    pub async fn get_table_statistics(&self) -> HashMap<String, TableStats> {
        self.index_manager.get_table_statistics().await.unwrap_or_default()
    }

    /// Execute query with performance monitoring
    pub async fn execute_query_monitored<'a, T>(
        &self,
        query: &str,
        params: &[T],
    ) -> Result<Vec<sqlx::postgres::PgRow>>
    where
        T: sqlx::Encode<'a, sqlx::Postgres> + sqlx::Type<sqlx::Postgres> + Send + Sync,
    {
        let start_time = Instant::now();

        // For now, execute without parameters - this needs proper parameter binding
        let result = sqlx::query(query).execute(&*self.client.pool()).await.map(|_| vec![]).map_err(anyhow::Error::from);

        let execution_time = start_time.elapsed().as_millis() as u64;

        // Record metrics asynchronously
        let monitor = self.monitor.clone();
        let query_text = query.to_string();
        tokio::spawn(async move {
            monitor.record_query_execution(&query_text, execution_time).await;
        });

        result
    }

    /// Generate database optimization report
    pub async fn generate_optimization_report(&self) -> DatabaseOptimizationReport {
        let query_metrics = self.get_performance_metrics().await;
        let slow_queries = self.get_slow_queries(10).await;
        let index_recommendations = self.get_index_recommendations().await;
        let table_stats = self.get_table_statistics().await;

        DatabaseOptimizationReport {
            generated_at: Utc::now(),
            total_queries_analyzed: query_metrics.len(),
            slow_queries_count: slow_queries.len(),
            index_recommendations_count: index_recommendations.len(),
            top_slow_queries: slow_queries.into_iter().take(5).collect(),
            critical_index_recommendations: index_recommendations.into_iter()
                .filter(|r| r.priority >= IndexPriority::High)
                .take(5)
                .collect(),
            table_statistics: table_stats,
        }
    }
}

/// Database optimization report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseOptimizationReport {
    pub generated_at: DateTime<Utc>,
    pub total_queries_analyzed: usize,
    pub slow_queries_count: usize,
    pub index_recommendations_count: usize,
    pub top_slow_queries: Vec<QueryMetrics>,
    pub critical_index_recommendations: Vec<IndexRecommendation>,
    pub table_statistics: HashMap<String, TableStats>,
}

/// Query execution wrapper for automatic monitoring
pub struct MonitoredQueryExecutor {
    client: DatabaseClient,
    monitor: Arc<DatabasePerformanceMonitor>,
}

impl MonitoredQueryExecutor {
    pub fn new(client: DatabaseClient, monitor: Arc<DatabasePerformanceMonitor>) -> Self {
        Self { client, monitor }
    }

    /// Execute query with automatic performance monitoring
    pub async fn execute<'a, T>(
        &self,
        query: &str,
        params: &[T],
    ) -> Result<Vec<sqlx::postgres::PgRow>>
    where
        T: sqlx::Encode<'a, sqlx::Postgres> + sqlx::Type<sqlx::Postgres> + Send + Sync,
    {
        let start_time = Instant::now();

        // For now, execute without parameters - this needs proper parameter binding
        let result = sqlx::query(query).execute(&*self.client.pool()).await.map(|_| vec![]).map_err(anyhow::Error::from);

        let execution_time = start_time.elapsed().as_millis() as u64;

        // Record metrics
        self.monitor.record_query_execution(query, execution_time).await;

        result
    }
}
