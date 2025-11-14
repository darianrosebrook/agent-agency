//! Standardized metrics collection for pipelines
//!
//! This module provides common metrics structures and collection patterns
//! that can be used across all pipeline implementations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Core metrics collector for pipelines
#[derive(Debug, Clone)]
pub struct PipelineMetrics {
    /// Metrics data
    data: Arc<RwLock<MetricsData>>,
    /// Prometheus registry (optional)
    #[cfg(feature = "prometheus")]
    registry: prometheus::Registry,
}

impl PipelineMetrics {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(MetricsData::default())),
            #[cfg(feature = "prometheus")]
            registry: prometheus::Registry::new(),
        }
    }

    /// Record a pipeline execution
    pub async fn record_execution(&self, duration_ms: u64, success: bool) {
        let mut data = self.data.write().await;
        data.total_executions += 1;
        data.total_execution_time_ms += duration_ms;

        if success {
            data.successful_executions += 1;
        } else {
            data.failed_executions += 1;
        }

        data.avg_execution_time_ms =
            data.total_execution_time_ms as f64 / data.total_executions as f64;
        data.last_updated = chrono::Utc::now();
    }

    /// Record a stage execution
    pub async fn record_stage_execution(&self, stage_name: &str, duration_ms: u64, success: bool) {
        let mut data = self.data.write().await;

        let stage_metric = data
            .stage_metrics
            .entry(stage_name.to_string())
            .or_insert_with(StageMetrics::default);

        stage_metric.total_executions += 1;
        stage_metric.total_execution_time_ms += duration_ms;

        if success {
            stage_metric.successful_executions += 1;
        } else {
            stage_metric.failed_executions += 1;
        }

        stage_metric.avg_execution_time_ms =
            stage_metric.total_execution_time_ms as f64 / stage_metric.total_executions as f64;
    }

    /// Record an error
    pub async fn record_error(&self, error_type: &str) {
        let mut data = self.data.write().await;
        *data.error_counts.entry(error_type.to_string()).or_insert(0) += 1;
        data.last_updated = chrono::Utc::now();
    }

    /// Get current metrics as JSON
    pub async fn to_json(&self) -> serde_json::Result<serde_json::Value> {
        let data = self.data.read().await.clone();
        serde_json::to_value(data)
    }

    /// Get metrics snapshot
    pub async fn snapshot(&self) -> MetricsData {
        self.data.read().await.clone()
    }

    /// Reset all metrics
    pub async fn reset(&self) {
        let mut data = self.data.write().await;
        *data = MetricsData::default();
    }

    /// Record buffer depth measurement
    pub async fn record_buffer_depth(&self, depth: usize) {
        let mut data = self.data.write().await;
        data.buffer_depth_metrics.total_samples += 1;
        data.buffer_depth_metrics.current_depth = depth;
        data.buffer_depth_metrics.max_depth = data.buffer_depth_metrics.max_depth.max(depth);
        data.buffer_depth_metrics.min_depth = data.buffer_depth_metrics.min_depth.min(depth);

        // Calculate moving average
        let alpha = 0.1; // Smoothing factor
        data.buffer_depth_metrics.average_depth =
            alpha * depth as f64 + (1.0 - alpha) * data.buffer_depth_metrics.average_depth;

        // Track depth distribution
        let depth_bucket = (depth / 10) * 10; // 10-unit buckets
        *data
            .buffer_depth_metrics
            .depth_distribution
            .entry(depth_bucket)
            .or_insert(0) += 1;

        data.last_updated = chrono::Utc::now();
    }

    /// Record buffer overflow event
    pub async fn record_buffer_overflow(&self) {
        let mut data = self.data.write().await;
        data.buffer_depth_metrics.overflow_count += 1;
        data.buffer_depth_metrics.last_overflow = Some(chrono::Utc::now());
        data.last_updated = chrono::Utc::now();
    }
}

impl Default for PipelineMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Core metrics data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsData {
    /// Total executions
    pub total_executions: u64,
    /// Successful executions
    pub successful_executions: u64,
    /// Failed executions
    pub failed_executions: u64,
    /// Total execution time (ms)
    pub total_execution_time_ms: u64,
    /// Average execution time (ms)
    pub avg_execution_time_ms: f64,
    /// Stage-specific metrics
    pub stage_metrics: HashMap<String, StageMetrics>,
    /// Error counts by type
    pub error_counts: HashMap<String, u64>,
    /// Buffer depth metrics
    pub buffer_depth_metrics: BufferDepthMetrics,
    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl Default for MetricsData {
    fn default() -> Self {
        Self {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            total_execution_time_ms: 0,
            avg_execution_time_ms: 0.0,
            stage_metrics: HashMap::new(),
            error_counts: HashMap::new(),
            buffer_depth_metrics: BufferDepthMetrics::default(),
            last_updated: chrono::Utc::now(),
        }
    }
}

/// Metrics for individual pipeline stages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageMetrics {
    /// Total executions for this stage
    pub total_executions: u64,
    /// Successful executions
    pub successful_executions: u64,
    /// Failed executions
    pub failed_executions: u64,
    /// Total execution time (ms)
    pub total_execution_time_ms: u64,
    /// Average execution time (ms)
    pub avg_execution_time_ms: f64,
}

impl Default for StageMetrics {
    fn default() -> Self {
        Self {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            total_execution_time_ms: 0,
            avg_execution_time_ms: 0.0,
        }
    }
}

/// Performance metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    /// Pipeline throughput (operations/sec)
    pub throughput_ops_per_sec: f64,
    /// Average latency (ms)
    pub avg_latency_ms: f64,
    /// P95 latency (ms)
    pub p95_latency_ms: f64,
    /// P99 latency (ms)
    pub p99_latency_ms: f64,
    /// Error rate (percentage)
    pub error_rate_percent: f64,
    /// Resource utilization (percentage)
    pub resource_utilization_percent: f64,
    /// Memory usage (bytes)
    pub memory_usage_bytes: u64,
    /// CPU usage (percentage)
    pub cpu_usage_percent: f64,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for PerformanceSnapshot {
    fn default() -> Self {
        Self {
            throughput_ops_per_sec: 0.0,
            avg_latency_ms: 0.0,
            p95_latency_ms: 0.0,
            p99_latency_ms: 0.0,
            error_rate_percent: 0.0,
            resource_utilization_percent: 0.0,
            memory_usage_bytes: 0,
            cpu_usage_percent: 0.0,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Health metrics for pipeline monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    /// Overall health status
    pub status: HealthStatus,
    /// Health score (0.0-1.0, 1.0 = perfectly healthy)
    pub health_score: f64,
    /// Number of healthy components
    pub healthy_components: usize,
    /// Number of unhealthy components
    pub unhealthy_components: usize,
    /// Last health check timestamp
    pub last_check: chrono::DateTime<chrono::Utc>,
    /// Health check duration (ms)
    pub check_duration_ms: u64,
}

impl Default for HealthMetrics {
    fn default() -> Self {
        Self {
            status: HealthStatus::Unknown,
            health_score: 0.0,
            healthy_components: 0,
            unhealthy_components: 0,
            last_check: chrono::Utc::now(),
            check_duration_ms: 0,
        }
    }
}

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    /// Health status unknown
    Unknown,
    /// System is healthy
    Healthy,
    /// System has minor issues
    Degraded,
    /// System is unhealthy but operational
    Unhealthy,
    /// System is not operational
    Down,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Unknown => write!(f, "unknown"),
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded => write!(f, "degraded"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
            HealthStatus::Down => write!(f, "down"),
        }
    }
}

/// Buffer depth metrics for streaming pipelines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferDepthMetrics {
    /// Total number of depth samples
    pub total_samples: u64,
    /// Current buffer depth
    pub current_depth: usize,
    /// Maximum observed depth
    pub max_depth: usize,
    /// Minimum observed depth
    pub min_depth: usize,
    /// Moving average depth
    pub average_depth: f64,
    /// Number of overflow events
    pub overflow_count: u64,
    /// Last overflow timestamp
    pub last_overflow: Option<chrono::DateTime<chrono::Utc>>,
    /// Depth distribution histogram (bucket -> count)
    pub depth_distribution: HashMap<usize, u64>,
}

impl Default for BufferDepthMetrics {
    fn default() -> Self {
        Self {
            total_samples: 0,
            current_depth: 0,
            max_depth: 0,
            min_depth: usize::MAX,
            average_depth: 0.0,
            overflow_count: 0,
            last_overflow: None,
            depth_distribution: HashMap::new(),
        }
    }
}

/// Resource usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    /// Memory usage (bytes)
    pub memory_bytes: u64,
    /// CPU usage (percentage)
    pub cpu_percent: f32,
    /// Disk I/O (bytes/sec)
    pub disk_io_bytes_per_sec: u64,
    /// Network I/O (bytes/sec)
    pub network_io_bytes_per_sec: u64,
    /// Active connections
    pub active_connections: usize,
    /// Thread count
    pub thread_count: usize,
    /// File descriptor count
    pub fd_count: usize,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for ResourceMetrics {
    fn default() -> Self {
        Self {
            memory_bytes: 0,
            cpu_percent: 0.0,
            disk_io_bytes_per_sec: 0,
            network_io_bytes_per_sec: 0,
            active_connections: 0,
            thread_count: 0,
            fd_count: 0,
            timestamp: chrono::Utc::now(),
        }
    }
}
