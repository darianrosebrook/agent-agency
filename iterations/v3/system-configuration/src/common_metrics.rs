//! Common metrics abstractions for performance, resource usage, and monitoring

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Common trait for all metrics that have a timestamp
pub trait TimedMetric {
    fn timestamp(&self) -> DateTime<Utc>;
}

/// Common trait for metrics that can be aggregated
pub trait AggregatableMetric {
    fn aggregate(&self, other: &Self) -> Self;
}

/// Common trait for metrics that have resource usage information
pub trait ResourceMetric {
    fn cpu_usage_percent(&self) -> Option<f64>;
    fn memory_usage_mb(&self) -> Option<u64>;
    fn disk_usage_mb(&self) -> Option<u64>;
    fn network_usage_mb(&self) -> Option<u64>;
}

/// Common trait for performance metrics
pub trait PerformanceMetric {
    fn throughput(&self) -> Option<f64>; // items/second
    fn avg_latency_ms(&self) -> Option<f64>;
    fn p95_latency_ms(&self) -> Option<f64>;
    fn p99_latency_ms(&self) -> Option<f64>;
    fn error_rate(&self) -> Option<f64>; // 0.0-1.0
}

/// Generic resource usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonResourceUsage {
    pub cpu_usage_percent: Option<f64>,
    pub memory_usage_mb: Option<u64>,
    pub disk_usage_mb: Option<u64>,
    pub network_usage_mb: Option<u64>,
    pub active_connections: Option<u64>,
    pub queue_depth: Option<u64>,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl TimedMetric for CommonResourceUsage {
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

impl ResourceMetric for CommonResourceUsage {
    fn cpu_usage_percent(&self) -> Option<f64> {
        self.cpu_usage_percent
    }

    fn memory_usage_mb(&self) -> Option<u64> {
        self.memory_usage_mb
    }

    fn disk_usage_mb(&self) -> Option<u64> {
        self.disk_usage_mb
    }

    fn network_usage_mb(&self) -> Option<u64> {
        self.network_usage_mb
    }
}

impl AggregatableMetric for CommonResourceUsage {
    fn aggregate(&self, other: &Self) -> Self {
        Self {
            cpu_usage_percent: self.cpu_usage_percent.zip(other.cpu_usage_percent)
                .map(|(a, b)| (a + b) / 2.0),
            memory_usage_mb: self.memory_usage_mb.zip(other.memory_usage_mb)
                .map(|(a, b)| (a + b) / 2),
            disk_usage_mb: self.disk_usage_mb.zip(other.disk_usage_mb)
                .map(|(a, b)| (a + b) / 2),
            network_usage_mb: self.network_usage_mb.zip(other.network_usage_mb)
                .map(|(a, b)| (a + b) / 2),
            active_connections: self.active_connections.zip(other.active_connections)
                .map(|(a, b)| a.max(b)), // Take max for connections
            queue_depth: self.queue_depth.zip(other.queue_depth)
                .map(|(a, b)| a.max(b)), // Take max for queue depth
            timestamp: self.timestamp.max(other.timestamp),
            metadata: HashMap::new(), // Clear metadata on aggregation
        }
    }
}

/// Generic performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonPerformanceMetrics {
    pub throughput: Option<f64>, // requests/second
    pub avg_latency_ms: Option<f64>,
    pub p95_latency_ms: Option<f64>,
    pub p99_latency_ms: Option<f64>,
    pub error_rate: Option<f64>, // 0.0-1.0
    pub success_rate: Option<f64>, // 0.0-1.0
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl TimedMetric for CommonPerformanceMetrics {
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

impl PerformanceMetric for CommonPerformanceMetrics {
    fn throughput(&self) -> Option<f64> {
        self.throughput
    }

    fn avg_latency_ms(&self) -> Option<f64> {
        self.avg_latency_ms
    }

    fn p95_latency_ms(&self) -> Option<f64> {
        self.p95_latency_ms
    }

    fn p99_latency_ms(&self) -> Option<f64> {
        self.p99_latency_ms
    }

    fn error_rate(&self) -> Option<f64> {
        self.error_rate
    }
}

impl AggregatableMetric for CommonPerformanceMetrics {
    fn aggregate(&self, other: &Self) -> Self {
        Self {
            throughput: self.throughput.zip(other.throughput)
                .map(|(a, b)| (a + b) / 2.0),
            avg_latency_ms: self.avg_latency_ms.zip(other.avg_latency_ms)
                .map(|(a, b)| (a + b) / 2.0),
            p95_latency_ms: self.p95_latency_ms.zip(other.p95_latency_ms)
                .map(|(a, b)| a.max(b)), // Take worst case
            p99_latency_ms: self.p99_latency_ms.zip(other.p99_latency_ms)
                .map(|(a, b)| a.max(b)), // Take worst case
            error_rate: self.error_rate.zip(other.error_rate)
                .map(|(a, b)| (a + b) / 2.0),
            success_rate: self.success_rate.zip(other.success_rate)
                .map(|(a, b)| (a + b) / 2.0),
            timestamp: self.timestamp.max(other.timestamp),
            metadata: HashMap::new(), // Clear metadata on aggregation
        }
    }
}

/// Quality metrics abstraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonQualityMetrics {
    pub coverage_percent: Option<f64>,
    pub mutation_score: Option<f64>,
    pub cyclomatic_complexity: Option<f64>,
    pub maintainability_index: Option<f64>,
    pub reliability_score: Option<f64>,
    pub security_score: Option<f64>,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl TimedMetric for CommonQualityMetrics {
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}
