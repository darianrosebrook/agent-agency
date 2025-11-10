//! Query Performance Monitoring
//!
//! Tracks query execution times, identifies slow queries, and provides performance insights.
//! Integrates with existing DatabasePerformanceMonitor and provides API endpoints.
//!
//! @author @darianrosebrook

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{warn, error, info};

/// Query performance metrics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QueryMetrics {
    /// Hash of the query (for deduplication)
    pub query_hash: String,
    
    /// Original query text
    pub query_text: String,
    
    /// Number of times this query has been executed
    pub execution_count: u64,
    
    /// Total execution time in milliseconds
    pub total_execution_time_ms: u64,
    
    /// Average execution time in milliseconds
    pub average_execution_time_ms: f64,
    
    /// Minimum execution time in milliseconds
    pub min_execution_time_ms: u64,
    
    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: u64,
    
    /// Timestamp of last execution
    #[schemars(with = "String")]
    pub last_executed: DateTime<Utc>,
    
    /// Number of slow executions (above threshold)
    pub slow_execution_count: u64,
    
    /// Percentage of executions that were slow
    pub slow_execution_rate: f64,
}

/// Slow query alert
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SlowQueryAlert {
    /// Query hash
    pub query_hash: String,
    
    /// Query text
    pub query_text: String,
    
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    
    /// Threshold that was exceeded
    pub threshold_ms: u64,
    
    /// Timestamp when alert was generated
    #[schemars(with = "String")]
    pub alerted_at: DateTime<Utc>,
}

/// Query performance monitor configuration
#[derive(Debug, Clone)]
pub struct QueryPerformanceConfig {
    /// Threshold for slow queries (milliseconds)
    pub slow_query_threshold_ms: u64,
    
    /// Threshold for critical slow queries (milliseconds)
    pub critical_slow_query_threshold_ms: u64,
    
    /// Maximum number of metrics to keep in memory
    pub max_metrics: usize,
    
    /// Maximum number of slow queries to keep
    pub max_slow_queries: usize,
    
    /// Enable automatic logging of slow queries
    pub enable_slow_query_logging: bool,
    
    /// Enable alerts for critical slow queries
    pub enable_critical_alerts: bool,
}

impl Default for QueryPerformanceConfig {
    fn default() -> Self {
        Self {
            slow_query_threshold_ms: 1000, // 1 second
            critical_slow_query_threshold_ms: 5000, // 5 seconds
            max_metrics: 10000,
            max_slow_queries: 1000,
            enable_slow_query_logging: true,
            enable_critical_alerts: true,
        }
    }
}

/// Query performance monitor
///
/// Tracks query execution times and identifies slow queries.
/// Provides metrics aggregation and alerting capabilities.
#[derive(Debug, Clone)]
pub struct QueryPerformanceMonitor {
    config: QueryPerformanceConfig,
    metrics: Arc<RwLock<HashMap<String, QueryMetrics>>>,
    slow_queries: Arc<RwLock<Vec<SlowQueryAlert>>>,
}

impl QueryPerformanceMonitor {
    /// Create a new query performance monitor
    pub fn new(config: QueryPerformanceConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(RwLock::new(HashMap::new())),
            slow_queries: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Create monitor with default configuration
    pub fn with_defaults() -> Self {
        Self::new(QueryPerformanceConfig::default())
    }
    
    /// Hash query text for deduplication
    fn hash_query(&self, query_text: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(query_text.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    /// Record query execution
    ///
    /// Call this after executing a query to track its performance.
    /// Automatically logs slow queries and generates alerts for critical queries.
    pub async fn record_query_execution(
        &self,
        query_text: &str,
        execution_time_ms: u64,
    ) {
        let query_hash = self.hash_query(query_text);
        
        // Update metrics
        let mut metrics_map = self.metrics.write().await;
        let metric = metrics_map.entry(query_hash.clone()).or_insert_with(|| {
            QueryMetrics {
                query_hash: query_hash.clone(),
                query_text: query_text.to_string(),
                execution_count: 0,
                total_execution_time_ms: 0,
                average_execution_time_ms: 0.0,
                min_execution_time_ms: u64::MAX,
                max_execution_time_ms: 0,
                last_executed: Utc::now(),
                slow_execution_count: 0,
                slow_execution_rate: 0.0,
            }
        });
        
        // Update metrics
        metric.execution_count += 1;
        metric.total_execution_time_ms += execution_time_ms;
        metric.average_execution_time_ms = metric.total_execution_time_ms as f64 / metric.execution_count as f64;
        metric.min_execution_time_ms = metric.min_execution_time_ms.min(execution_time_ms);
        metric.max_execution_time_ms = metric.max_execution_time_ms.max(execution_time_ms);
        metric.last_executed = Utc::now();
        
        // Check if query is slow
        let is_slow = execution_time_ms >= self.config.slow_query_threshold_ms;
        let is_critical = execution_time_ms >= self.config.critical_slow_query_threshold_ms;
        
        if is_slow {
            metric.slow_execution_count += 1;
            metric.slow_execution_rate = metric.slow_execution_count as f64 / metric.execution_count as f64;
            
            // Log slow query
            if self.config.enable_slow_query_logging {
                warn!(
                    "Slow query detected: {}ms (threshold: {}ms) - {}",
                    execution_time_ms,
                    self.config.slow_query_threshold_ms,
                    query_text.chars().take(200).collect::<String>() // Truncate for logging
                );
            }
            
            // Record slow query alert
            let mut slow_queries = self.slow_queries.write().await;
            slow_queries.push(SlowQueryAlert {
                query_hash: query_hash.clone(),
                query_text: query_text.to_string(),
                execution_time_ms,
                threshold_ms: self.config.slow_query_threshold_ms,
                alerted_at: Utc::now(),
            });
            
            // Keep only recent slow queries
            if slow_queries.len() > self.config.max_slow_queries {
                slow_queries.remove(0);
            }
            
            // Critical alert
            if is_critical && self.config.enable_critical_alerts {
                error!(
                    "CRITICAL: Very slow query detected: {}ms (threshold: {}ms) - {}",
                    execution_time_ms,
                    self.config.critical_slow_query_threshold_ms,
                    query_text.chars().take(200).collect::<String>()
                );
            }
        }
        
        // Cleanup old metrics if we exceed max
        if metrics_map.len() > self.config.max_metrics {
            // Remove oldest metrics (by last_executed)
            // Collect hashes and timestamps first, then remove
            let mut entries: Vec<(String, DateTime<Utc>)> = metrics_map
                .iter()
                .map(|(hash, m)| (hash.clone(), m.last_executed))
                .collect();
            entries.sort_by_key(|(_, timestamp)| *timestamp);
            let to_remove = entries.len() - self.config.max_metrics;
            let hashes_to_remove: Vec<String> = entries
                .iter()
                .take(to_remove)
                .map(|(hash, _)| hash.clone())
                .collect();
            for hash in hashes_to_remove {
                metrics_map.remove(&hash);
            }
        }
    }
    
    /// Record query execution with timing
    ///
    /// Convenience method that takes a start time and calculates duration.
    pub async fn record_query_execution_timed(
        &self,
        query_text: &str,
        start_time: Instant,
    ) {
        let duration = start_time.elapsed();
        let execution_time_ms = duration.as_millis() as u64;
        self.record_query_execution(query_text, execution_time_ms).await;
    }
    
    /// Get all query metrics
    pub async fn get_all_metrics(&self) -> Vec<QueryMetrics> {
        let metrics_map = self.metrics.read().await;
        metrics_map.values().cloned().collect()
    }
    
    /// Get metrics for a specific query
    pub async fn get_query_metrics(&self, query_hash: &str) -> Option<QueryMetrics> {
        let metrics_map = self.metrics.read().await;
        metrics_map.get(query_hash).cloned()
    }
    
    /// Get slow queries
    pub async fn get_slow_queries(&self, limit: Option<usize>) -> Vec<SlowQueryAlert> {
        let slow_queries = self.slow_queries.read().await;
        let limit = limit.unwrap_or(slow_queries.len());
        slow_queries.iter().rev().take(limit).cloned().collect()
    }
    
    /// Get top slow queries by average execution time
    pub async fn get_top_slow_queries(&self, limit: usize) -> Vec<QueryMetrics> {
        let metrics_map = self.metrics.read().await;
        let mut metrics: Vec<QueryMetrics> = metrics_map.values().cloned().collect();
        
        // Sort by average execution time (descending)
        metrics.sort_by(|a, b| {
            b.average_execution_time_ms.partial_cmp(&a.average_execution_time_ms)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        metrics.into_iter().take(limit).collect()
    }
    
    /// Get queries with highest slow execution rate
    pub async fn get_queries_with_high_slow_rate(&self, limit: usize, min_rate: f64) -> Vec<QueryMetrics> {
        let metrics_map = self.metrics.read().await;
        let mut metrics: Vec<QueryMetrics> = metrics_map
            .values()
            .filter(|m| m.slow_execution_rate >= min_rate)
            .cloned()
            .collect();
        
        // Sort by slow execution rate (descending)
        metrics.sort_by(|a, b| {
            b.slow_execution_rate.partial_cmp(&a.slow_execution_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        metrics.into_iter().take(limit).collect()
    }
    
    /// Get performance summary
    pub async fn get_performance_summary(&self) -> PerformanceSummary {
        let metrics_map = self.metrics.read().await;
        let slow_queries = self.slow_queries.read().await;
        
        let total_queries: u64 = metrics_map.values().map(|m| m.execution_count).sum();
        let total_slow_queries: u64 = metrics_map.values().map(|m| m.slow_execution_count).sum();
        let avg_execution_time: f64 = if total_queries > 0 {
            let total_time: u64 = metrics_map.values().map(|m| m.total_execution_time_ms).sum();
            total_time as f64 / total_queries as f64
        } else {
            0.0
        };
        
        PerformanceSummary {
            total_queries,
            total_slow_queries,
            slow_query_rate: if total_queries > 0 {
                total_slow_queries as f64 / total_queries as f64
            } else {
                0.0
            },
            average_execution_time_ms: avg_execution_time,
            unique_query_count: metrics_map.len() as u64,
            recent_slow_query_count: slow_queries.len() as u64,
            slow_query_threshold_ms: self.config.slow_query_threshold_ms,
            critical_slow_query_threshold_ms: self.config.critical_slow_query_threshold_ms,
        }
    }
    
    /// Clear all metrics (useful for testing or reset)
    pub async fn clear_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.clear();
        
        let mut slow_queries = self.slow_queries.write().await;
        slow_queries.clear();
        
        info!("Query performance metrics cleared");
    }
}

/// Performance summary statistics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PerformanceSummary {
    /// Total number of queries executed
    pub total_queries: u64,
    
    /// Total number of slow queries
    pub total_slow_queries: u64,
    
    /// Rate of slow queries (0.0 to 1.0)
    pub slow_query_rate: f64,
    
    /// Average execution time across all queries (milliseconds)
    pub average_execution_time_ms: f64,
    
    /// Number of unique queries tracked
    pub unique_query_count: u64,
    
    /// Number of recent slow queries in alert log
    pub recent_slow_query_count: u64,
    
    /// Slow query threshold (milliseconds)
    pub slow_query_threshold_ms: u64,
    
    /// Critical slow query threshold (milliseconds)
    pub critical_slow_query_threshold_ms: u64,
}

/// Helper macro for timing query execution
///
/// Usage:
/// ```rust
/// let monitor = QueryPerformanceMonitor::with_defaults();
/// let result = time_query!(monitor, "SELECT * FROM users", {
///     // Query execution code
///     db.query("SELECT * FROM users").await?
/// });
/// ```
#[macro_export]
macro_rules! time_query {
    ($monitor:expr, $query_text:expr, $query:block) => {{
        let start_time = std::time::Instant::now();
        let result = $query;
        $monitor.record_query_execution_timed($query_text, start_time).await;
        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_record_query_execution() {
        let monitor = QueryPerformanceMonitor::with_defaults();
        
        monitor.record_query_execution("SELECT * FROM users", 50).await;
        monitor.record_query_execution("SELECT * FROM users", 75).await;
        monitor.record_query_execution("SELECT * FROM users", 100).await;
        
        let metrics = monitor.get_all_metrics().await;
        assert_eq!(metrics.len(), 1);
        
        let metric = &metrics[0];
        assert_eq!(metric.execution_count, 3);
        assert_eq!(metric.min_execution_time_ms, 50);
        assert_eq!(metric.max_execution_time_ms, 100);
        assert!((metric.average_execution_time_ms - 75.0).abs() < 0.1);
    }
    
    #[tokio::test]
    async fn test_slow_query_detection() {
        let config = QueryPerformanceConfig {
            slow_query_threshold_ms: 100,
            ..Default::default()
        };
        let monitor = QueryPerformanceMonitor::new(config);
        
        monitor.record_query_execution("SELECT * FROM users", 50).await;
        monitor.record_query_execution("SELECT * FROM users", 150).await; // Slow
        monitor.record_query_execution("SELECT * FROM users", 200).await; // Slow
        
        let metrics = monitor.get_all_metrics().await;
        let metric = &metrics[0];
        assert_eq!(metric.slow_execution_count, 2);
        assert!((metric.slow_execution_rate - 0.666).abs() < 0.01); // 2/3
        
        let slow_queries = monitor.get_slow_queries(None).await;
        assert_eq!(slow_queries.len(), 2);
    }
    
    #[tokio::test]
    async fn test_performance_summary() {
        let monitor = QueryPerformanceMonitor::with_defaults();
        
        monitor.record_query_execution("SELECT * FROM users", 50).await;
        monitor.record_query_execution("SELECT * FROM tasks", 1500).await; // Slow
        
        let summary = monitor.get_performance_summary().await;
        assert_eq!(summary.total_queries, 2);
        assert_eq!(summary.total_slow_queries, 1);
        assert_eq!(summary.unique_query_count, 2);
    }
}

