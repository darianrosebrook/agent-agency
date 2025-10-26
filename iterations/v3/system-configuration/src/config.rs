//! Common pipeline configuration patterns
//!
//! This module provides standardized configuration structures that can be used
//! across different pipeline implementations.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Base configuration for all pipelines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    /// Pipeline name for identification
    pub name: String,
    /// Pipeline description
    pub description: Option<String>,
    /// Enable pipeline metrics collection
    pub enable_metrics: bool,
    /// Enable tracing/logging
    pub enable_tracing: bool,
    /// Pipeline execution timeout
    pub timeout: Duration,
    /// Maximum concurrent operations
    pub max_concurrent_operations: usize,
    /// Enable circuit breaker pattern
    pub enable_circuit_breaker: bool,
    /// Circuit breaker failure threshold
    pub circuit_breaker_threshold: u32,
    /// Circuit breaker recovery timeout
    pub circuit_breaker_recovery_timeout: Duration,
    /// Enable health monitoring
    pub enable_health_monitoring: bool,
    /// Health check interval
    pub health_check_interval: Duration,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            name: "default_pipeline".to_string(),
            description: None,
            enable_metrics: true,
            enable_tracing: true,
            timeout: Duration::from_secs(300), // 5 minutes
            max_concurrent_operations: 10,
            enable_circuit_breaker: true,
            circuit_breaker_threshold: 5,
            circuit_breaker_recovery_timeout: Duration::from_secs(60),
            enable_health_monitoring: true,
            health_check_interval: Duration::from_secs(30),
        }
    }
}

/// Configuration for sequential pipelines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequentialPipelineConfig {
    /// Base pipeline configuration
    #[serde(flatten)]
    pub base: PipelineConfig,
    /// Maximum retries for failed stages
    pub max_stage_retries: u32,
    /// Continue on stage failure
    pub continue_on_stage_failure: bool,
    /// Stage execution timeout
    pub stage_timeout: Duration,
    /// Enable stage result caching
    pub enable_stage_caching: bool,
}

impl Default for SequentialPipelineConfig {
    fn default() -> Self {
        Self {
            base: PipelineConfig::default(),
            max_stage_retries: 3,
            continue_on_stage_failure: false,
            stage_timeout: Duration::from_secs(60),
            enable_stage_caching: false,
        }
    }
}

/// Configuration for parallel pipelines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelPipelineConfig {
    /// Base pipeline configuration
    #[serde(flatten)]
    pub base: PipelineConfig,
    /// Maximum parallel stages
    pub max_parallel_stages: usize,
    /// Result aggregation strategy
    pub aggregation_strategy: AggregationStrategy,
    /// Parallel execution timeout
    pub parallel_timeout: Duration,
    /// Enable speculative execution
    pub enable_speculative_execution: bool,
    /// Speculative execution threshold
    pub speculative_threshold: f64,
}

impl Default for ParallelPipelineConfig {
    fn default() -> Self {
        Self {
            base: PipelineConfig::default(),
            max_parallel_stages: 5,
            aggregation_strategy: AggregationStrategy::AllRequired,
            parallel_timeout: Duration::from_secs(120),
            enable_speculative_execution: false,
            speculative_threshold: 0.8,
        }
    }
}

/// Configuration for streaming pipelines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingPipelineConfig {
    /// Base pipeline configuration
    #[serde(flatten)]
    pub base: PipelineConfig,
    /// Channel buffer size
    pub buffer_size: usize,
    /// Maximum active streams
    pub max_active_streams: usize,
    /// Stream processing timeout
    pub stream_timeout: Duration,
    /// Enable backpressure handling
    pub enable_backpressure: bool,
    /// Backpressure threshold
    pub backpressure_threshold: usize,
    /// Enable stream multiplexing
    pub enable_multiplexing: bool,
}

impl Default for StreamingPipelineConfig {
    fn default() -> Self {
        Self {
            base: PipelineConfig::default(),
            buffer_size: 1000,
            max_active_streams: 50,
            stream_timeout: Duration::from_secs(30),
            enable_backpressure: true,
            backpressure_threshold: 100,
            enable_multiplexing: false,
        }
    }
}

/// Configuration for validation pipelines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationPipelineConfig {
    /// Base pipeline configuration
    #[serde(flatten)]
    pub base: PipelineConfig,
    /// Stop on first validation error
    pub stop_on_first_error: bool,
    /// Validation severity threshold
    pub severity_threshold: ValidationSeverity,
    /// Enable validation caching
    pub enable_validation_caching: bool,
    /// Maximum validation time per stage
    pub max_validation_time: Duration,
    /// Collect all errors before failing
    pub collect_all_errors: bool,
}

impl Default for ValidationPipelineConfig {
    fn default() -> Self {
        Self {
            base: PipelineConfig::default(),
            stop_on_first_error: false,
            severity_threshold: ValidationSeverity::Warning,
            enable_validation_caching: true,
            max_validation_time: Duration::from_secs(30),
            collect_all_errors: true,
        }
    }
}

/// Aggregation strategy for parallel pipelines
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AggregationStrategy {
    /// All stages must succeed
    AllRequired,
    /// At least one stage must succeed
    AnyRequired,
    /// Majority of stages must succeed
    MajorityRequired,
    /// Weighted success based on stage priority
    Weighted,
}

// ValidationSeverity is defined in validation.rs and re-exported here for convenience

/// Resource limits for pipeline execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage (bytes)
    pub max_memory_bytes: u64,
    /// Maximum CPU usage (percentage)
    pub max_cpu_percent: f32,
    /// Maximum disk I/O (bytes/sec)
    pub max_disk_io_bytes_per_sec: u64,
    /// Maximum network I/O (bytes/sec)
    pub max_network_io_bytes_per_sec: u64,
    /// Maximum concurrent connections
    pub max_concurrent_connections: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 1_073_741_824, // 1GB
            max_cpu_percent: 80.0,
            max_disk_io_bytes_per_sec: 100_000_000, // 100MB/s
            max_network_io_bytes_per_sec: 50_000_000, // 50MB/s
            max_concurrent_connections: 100,
        }
    }
}

/// Monitoring configuration for pipelines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable Prometheus metrics
    pub enable_prometheus: bool,
    /// Metrics collection interval
    pub metrics_interval: Duration,
    /// Enable health checks
    pub enable_health_checks: bool,
    /// Health check timeout
    pub health_check_timeout: Duration,
    /// Alert thresholds
    pub alert_thresholds: AlertThresholds,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enable_prometheus: true,
            metrics_interval: Duration::from_secs(30),
            enable_health_checks: true,
            health_check_timeout: Duration::from_secs(5),
            alert_thresholds: AlertThresholds::default(),
        }
    }
}

/// Alert thresholds for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// Error rate threshold (percentage)
    pub error_rate_threshold: f32,
    /// Latency threshold (ms)
    pub latency_threshold_ms: u64,
    /// Memory usage threshold (percentage)
    pub memory_threshold_percent: f32,
    /// CPU usage threshold (percentage)
    pub cpu_threshold_percent: f32,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            error_rate_threshold: 5.0, // 5%
            latency_threshold_ms: 1000, // 1 second
            memory_threshold_percent: 90.0, // 90%
            cpu_threshold_percent: 85.0, // 85%
        }
    }
}

// Re-export ValidationSeverity from validation module for backward compatibility
pub use crate::validation::ValidationSeverity;
