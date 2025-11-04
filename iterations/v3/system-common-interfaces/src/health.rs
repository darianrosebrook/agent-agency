//! Health Check Interface
//!
//! Common health check interface for service health monitoring that can be
//! implemented by different health check systems without creating dependencies.
//!
//! This allows system-observability to provide health monitoring implementations
//! while other crates can depend on the interface for health checks.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;
use chrono::{DateTime, Utc};

use crate::{HealthStatus, Result};

/// Health check that can be executed
#[async_trait]
pub trait HealthCheck: Send + Sync {
    /// Execute the health check
    async fn check(&self) -> Result<HealthCheckResult>;

    /// Get the name of this health check
    fn name(&self) -> &str;

    /// Get the description of this health check
    fn description(&self) -> &str;

    /// Get the tags associated with this health check
    fn tags(&self) -> &[String];

    /// Get the timeout for this health check
    fn timeout(&self) -> std::time::Duration;
}

/// Result of a health check execution
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct HealthCheckResult {
    pub status: HealthStatus,
    pub message: Option<String>,
    pub details: HashMap<String, serde_json::Value>,
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    pub duration_ms: u64,
}

/// Health check registry interface
#[async_trait]
pub trait HealthCheckRegistry: Send + Sync {
    /// Register a health check
    async fn register(&self, check: Box<dyn HealthCheck>) -> Result<String>;

    /// Unregister a health check by ID
    async fn unregister(&self, check_id: &str) -> Result<bool>;

    /// Get a health check by ID
    async fn get_check(&self, check_id: &str) -> Result<Option<Box<dyn HealthCheck>>>;

    /// List all registered health checks
    async fn list_checks(&self) -> Result<Vec<HealthCheckInfo>>;
}

/// Information about a registered health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub timeout_ms: u64,
    pub last_result: Option<HealthCheckResult>,
}

/// Health check executor interface
#[async_trait]
pub trait HealthCheckExecutor: Send + Sync {
    /// Execute a single health check
    async fn execute_check(&self, check: &dyn HealthCheck) -> Result<HealthCheckResult>;

    /// Execute all registered health checks
    async fn execute_all(&self) -> Result<HealthReport>;

    /// Execute health checks by tags
    async fn execute_by_tags(&self, tags: &[String]) -> Result<HealthReport>;
}

/// Comprehensive health report
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct HealthReport {
    pub overall_status: HealthStatus,
    pub summary: HealthSummary,
    pub results: Vec<HealthCheckResult>,
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    pub duration_ms: u64,
}

/// Health report summary
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct HealthSummary {
    pub total_checks: usize,
    pub healthy_checks: usize,
    pub degraded_checks: usize,
    pub unhealthy_checks: usize,
    pub average_response_time_ms: f64,
    pub slowest_check_ms: u64,
    pub fastest_check_ms: u64,
}

/// Health check scheduler interface
#[async_trait]
pub trait HealthCheckScheduler: Send + Sync {
    /// Schedule a health check to run periodically
    async fn schedule_check(
        &self,
        check: Box<dyn HealthCheck>,
        interval: std::time::Duration,
    ) -> Result<String>;

    /// Cancel a scheduled health check
    async fn cancel_scheduled(&self, schedule_id: &str) -> Result<bool>;

    /// Get the status of all scheduled checks
    async fn scheduled_status(&self) -> Result<HashMap<String, ScheduledCheckStatus>>;
}

/// Status of a scheduled health check
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ScheduledCheckStatus {
    pub schedule_id: String,
    pub check_name: String,
    pub interval_ms: u64,
    #[schemars(with = "String")]
    pub next_run: DateTime<Utc>,
    #[schemars(with = "Option<String>")]
    pub last_run: Option<DateTime<Utc>>,
    pub last_result: Option<HealthCheckResult>,
    pub consecutive_failures: u32,
    pub is_active: bool,
}

/// Dependency health check for external services
#[async_trait]
pub trait DependencyHealthCheck: Send + Sync {
    /// Check health of a specific dependency
    async fn check_dependency(&self, name: &str) -> Result<DependencyHealth>;

    /// Check health of all dependencies
    async fn check_all_dependencies(&self) -> Result<HashMap<String, DependencyHealth>>;
}

/// Health status of a service dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyHealth {
    pub name: String,
    pub status: HealthStatus,
    pub message: Option<String>,
    pub response_time_ms: Option<u64>,
    pub last_successful_check: Option<DateTime<Utc>>,
    pub error_count: u32,
    pub details: HashMap<String, serde_json::Value>,
}

/// Pre-built health checks for common scenarios

/// Database connectivity health check
pub struct DatabaseHealthCheck<D: crate::database::DatabaseConnection> {
    pub name: String,
    pub description: String,
    pub connection: D,
}

#[async_trait]
impl<D: crate::database::DatabaseConnection> HealthCheck for DatabaseHealthCheck<D> {
    async fn check(&self) -> Result<HealthCheckResult> {
        let start = std::time::Instant::now();

        match self.connection.health_check().await {
            Ok(()) => Ok(HealthCheckResult {
                status: HealthStatus::Healthy,
                message: Some("Database connection is healthy".to_string()),
                details: HashMap::new(),
                timestamp: Utc::now(),
                duration_ms: start.elapsed().as_millis() as u64,
            }),
            Err(e) => Ok(HealthCheckResult {
                status: HealthStatus::Unhealthy,
                message: Some(format!("Database health check failed: {}", e)),
                details: HashMap::new(),
                timestamp: Utc::now(),
                duration_ms: start.elapsed().as_millis() as u64,
            }),
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn tags(&self) -> &[String] {
        static TAGS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| {
            vec!["database".to_string(), "connectivity".to_string()]
        });
        TAGS.as_slice()
    }

    fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(5)
    }
}

/// HTTP endpoint health check
pub struct HttpHealthCheck {
    pub name: String,
    pub description: String,
    pub url: String,
    pub expected_status: Option<u16>,
    pub timeout: std::time::Duration,
}

#[async_trait]
impl HealthCheck for HttpHealthCheck {
    async fn check(&self) -> Result<HealthCheckResult> {
        let start = std::time::Instant::now();

        match reqwest::get(&self.url).await {
            Ok(response) => {
                let status = response.status().as_u16();
                let expected_status = self.expected_status.unwrap_or(200);

                let (status_health, message) = if status == expected_status {
                    (HealthStatus::Healthy, format!("HTTP {} response", status))
                } else {
                    (HealthStatus::Unhealthy, format!("Unexpected HTTP status: {} (expected {})", status, expected_status))
                };

                Ok(HealthCheckResult {
                    status: status_health,
                    message: Some(message),
                    details: {
                        let mut details = HashMap::new();
                        details.insert("status_code".to_string(), status.into());
                        details.insert("url".to_string(), self.url.clone().into());
                        details
                    },
                    timestamp: Utc::now(),
                    duration_ms: start.elapsed().as_millis() as u64,
                })
            }
            Err(e) => Ok(HealthCheckResult {
                status: HealthStatus::Unhealthy,
                message: Some(format!("HTTP request failed: {}", e)),
                details: {
                    let mut details = HashMap::new();
                    details.insert("url".to_string(), self.url.clone().into());
                    details.insert("error".to_string(), e.to_string().into());
                    details
                },
                timestamp: Utc::now(),
                duration_ms: start.elapsed().as_millis() as u64,
            }),
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn tags(&self) -> &[String] {
        static TAGS: LazyLock<Vec<String>> = LazyLock::new(|| {
            vec!["http".to_string(), "external".to_string()]
        });
        TAGS.as_slice()
    }

    fn timeout(&self) -> std::time::Duration {
        self.timeout
    }
}
