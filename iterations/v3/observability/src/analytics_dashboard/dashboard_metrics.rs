//! Metrics structures for analytics dashboard

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::data::{AnalyticsInsight, PredictiveModelResult};

/// System metrics collected from monitoring sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_throughput: f64,
    pub response_time_ms: f64,
    pub error_rate: f64,
    pub uptime_seconds: u64,
    pub timestamp: DateTime<Utc>,
}

/// Agent metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub active_agents: usize,
    pub idle_agents: usize,
    pub busy_agents: usize,
    pub failed_agents: usize,
    pub average_response_time: f64,
    pub total_requests_processed: u64,
    pub success_rate: f64,
    pub timestamp: DateTime<Utc>,
}

/// Task metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetrics {
    pub total_tasks: u32,
    pub completed_tasks: u32,
    pub failed_tasks: u32,
    pub pending_tasks: u32,
    pub average_completion_time: f64,
    pub throughput_tasks_per_hour: f64,
    pub timestamp: DateTime<Utc>,
}

/// Processed system metrics for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedSystemMetrics {
    pub active_agents: usize,
    pub total_tasks: u32,
    pub system_load: f64,
    pub task_success_rate: f64,
    pub agent_utilization: f64,
    pub system_stability: f64,
    pub timestamp: DateTime<Utc>,
}

/// Validated predictions from multiple models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedPredictions {
    pub capacity_predictions: Vec<PredictiveModelResult>,
    pub performance_forecasts: Vec<PredictiveModelResult>,
    pub quality_predictions: Vec<PredictiveModelResult>,
    pub cost_projections: Vec<PredictiveModelResult>,
    pub validation_timestamp: DateTime<Utc>,
}

/// Cached analytics insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedInsights {
    /// Cached insights
    pub insights: Vec<AnalyticsInsight>,
    /// Cache timestamp
    pub cached_at: DateTime<Utc>,
    /// Cache metadata
    pub metadata: CacheMetadata,
}

/// Cache metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    /// Cache key
    pub cache_key: String,
    /// Cache size in bytes
    pub cache_size_bytes: usize,
    /// Number of insights
    pub insights_count: usize,
    /// Cache generation time
    pub generation_time_ms: u64,
    /// System state hash
    pub system_state_hash: String,
}

/// Cache performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachePerformanceMetrics {
    /// Cache hit rate
    pub hit_rate: f64,
    /// Cache miss rate
    pub miss_rate: f64,
    /// Average cache access time
    pub avg_access_time_ms: f64,
    /// Cache size
    pub cache_size_bytes: usize,
    /// Number of cache operations
    pub operations_count: u64,
    /// Last cache update
    pub last_update: DateTime<Utc>,
}
