//! Common types used across the system interfaces
//!
//! This module contains shared types that are used by multiple interface modules
//! to avoid duplication and ensure consistency.
//!
//! @author @darianrosebrook

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Service health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Service is healthy and fully operational
    Healthy,
    /// Service is experiencing issues but still operational
    Degraded,
    /// Service is unhealthy and not operational
    Unhealthy,
}

/// Task scope levels for complexity assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskScope {
    /// Single file modifications
    File,
    /// Module-level changes
    Module,
    /// Package-level changes
    Package,
    /// System-wide changes
    System,
}

/// Resource usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU usage percentage (0.0-100.0)
    pub cpu_percent: f64,
    /// Memory usage in MB
    pub memory_mb: u64,
    /// Disk I/O in bytes per second
    pub disk_io_bytes: u64,
    /// Network I/O in bytes per second
    pub network_io_bytes: u64,
}

/// Performance requirements specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRequirements {
    /// Maximum response time in milliseconds (P95)
    pub max_response_time_ms: Option<u64>,
    /// Maximum throughput in requests per second
    pub max_throughput_rps: Option<f64>,
    /// Maximum CPU usage percentage
    pub max_cpu_percent: Option<f64>,
    /// Maximum memory usage in MB
    pub max_memory_mb: Option<u64>,
}

/// Quality requirements specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRequirements {
    /// Minimum test coverage percentage (0-100)
    pub min_test_coverage: Option<f64>,
    /// Required code quality metrics
    pub code_quality_metrics: HashMap<String, f64>,
    /// Security requirements
    pub security_requirements: Vec<String>,
    /// Compliance requirements
    pub compliance_requirements: Vec<String>,
}

/// System metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// Current CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Current memory usage in MB
    pub memory_usage_mb: u64,
    /// Current disk usage percentage
    pub disk_usage_percent: f64,
    /// Network I/O in bytes per second
    pub network_io_bps: u64,
    /// Number of active connections
    pub active_connections: u32,
    /// Current request rate per second
    pub request_rate_per_sec: f64,
    /// Current error rate per second
    pub error_rate_per_sec: f64,
    /// P95 response time in milliseconds
    pub p95_response_time_ms: f64,
    /// P99 response time in milliseconds
    pub p99_response_time_ms: f64,
}

/// Inference request for model operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    /// Model identifier
    pub model_id: String,
    /// Input prompt or data
    pub prompt: String,
    /// Additional context data
    pub context: HashMap<String, serde_json::Value>,
    /// Model-specific parameters
    pub parameters: HashMap<String, serde_json::Value>,
}
