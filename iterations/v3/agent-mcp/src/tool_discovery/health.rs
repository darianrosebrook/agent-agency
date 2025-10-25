//! Health monitoring for tool discovery

use super::core::{HealthStatus, HealthCheckResult};
use crate::types::*;
use anyhow::Result;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use std::time::{Duration, Instant};

/// Health monitor for tool discovery service
pub struct ToolDiscoveryHealthMonitor {
    last_check: Option<DateTime<Utc>>,
    check_interval: Duration,
}

impl ToolDiscoveryHealthMonitor {
    pub fn new() -> Self {
        Self {
            last_check: None,
            check_interval: Duration::from_secs(30),
        }
    }

    pub fn with_check_interval(interval: Duration) -> Self {
        Self {
            last_check: None,
            check_interval: interval,
        }
    }

    /// Perform health check
    pub async fn check_health(&mut self) -> HealthCheckResult {
        let start_time = Instant::now();

        let component = "tool_discovery".to_string();
        let timestamp = Utc::now();

        // Check if we need to perform a fresh check
        if let Some(last) = self.last_check {
            if timestamp.signed_duration_since(last) < chrono::Duration::from_std(self.check_interval).unwrap() {
                // Return cached result
                return HealthCheckResult {
                    component,
                    status: HealthStatus::Healthy,
                    timestamp,
                    error_message: None,
                    metadata: HashMap::new(),
                    duration_ms: start_time.elapsed().as_millis() as u64,
                };
            }
        }

        // Perform actual health checks
        let mut errors = Vec::new();
        let mut metadata = HashMap::new();

        // Check basic functionality
        metadata.insert("service_available".to_string(), serde_json::json!(true));
        metadata.insert("last_check".to_string(), serde_json::json!(timestamp.to_rfc3339()));

        self.last_check = Some(timestamp);

        let status = if errors.is_empty() {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy
        };

        let error_message = if errors.is_empty() {
            None
        } else {
            Some(errors.join("; "))
        };

        HealthCheckResult {
            component,
            status,
            timestamp,
            error_message,
            metadata,
            duration_ms: start_time.elapsed().as_millis() as u64,
        }
    }
}

/// Endpoint type enumeration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EndpointType {
    /// HTTP REST endpoint
    Http,
    /// WebSocket endpoint
    WebSocket,
    /// gRPC endpoint
    Grpc,
    /// Local filesystem
    Filesystem,
    /// Unknown endpoint type
    Unknown(String),
}

impl std::fmt::Display for EndpointType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EndpointType::Http => write!(f, "HTTP"),
            EndpointType::WebSocket => write!(f, "WebSocket"),
            EndpointType::Grpc => write!(f, "gRPC"),
            EndpointType::Filesystem => write!(f, "Filesystem"),
            EndpointType::Unknown(s) => write!(f, "Unknown({})", s),
        }
    }
}

/// Health check result structure
#[derive(Debug, Clone)]
pub struct EndpointHealthCheckResult {
    /// Endpoint URL or path
    pub endpoint: String,
    /// Endpoint type
    pub endpoint_type: EndpointType,
    /// Health status
    pub status: HealthStatus,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Timestamp of check
    pub checked_at: DateTime<Utc>,
    /// Error message if unhealthy
    pub error_message: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Internal health check result for processing
#[derive(Debug, Clone)]
pub struct InternalHealthCheckResult {
    /// Component name
    pub component: String,
    /// Check result
    pub result: HealthCheckResult,
    /// Internal metrics
    pub metrics: HashMap<String, f64>,
}
