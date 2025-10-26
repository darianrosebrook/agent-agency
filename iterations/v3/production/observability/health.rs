//! Health monitoring and checking

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

use super::core::{HealthStatus, HealthCheckResult};

/// Health check trait for components that can be monitored
#[async_trait]
pub trait ProductionHealthCheck: Send + Sync {
    /// Perform a health check
    async fn check_health(&self) -> HealthCheckResult;

    /// Get the component name
    fn component_name(&self) -> &str;
}

/// Health monitor that manages multiple health checks
pub struct HealthMonitor {
    /// Registered health checks
    checks: HashMap<String, Box<dyn HealthCheck>>,
    /// Health check results cache
    results_cache: HashMap<String, (HealthCheckResult, DateTime<Utc>)>,
    /// Cache TTL in seconds
    cache_ttl_seconds: u64,
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new() -> Self {
        Self {
            checks: HashMap::new(),
            results_cache: HashMap::new(),
            cache_ttl_seconds: 30, // 30 second cache
        }
    }

    /// Create a health monitor with custom cache TTL
    pub fn with_cache_ttl(cache_ttl_seconds: u64) -> Self {
        Self {
            checks: HashMap::new(),
            results_cache: HashMap::new(),
            cache_ttl_seconds,
        }
    }

    /// Register a health check
    pub fn register_check(&mut self, check: Box<dyn HealthCheck>) {
        let name = check.component_name().to_string();
        self.checks.insert(name, check);
    }

    /// Register a health check by name
    pub fn register_check_named(&mut self, name: impl Into<String>, check: Box<dyn HealthCheck>) {
        self.checks.insert(name.into(), check);
    }

    /// Run all health checks
    pub async fn run_all_checks(&mut self) -> Vec<HealthCheckResult> {
        let mut results = Vec::new();

        for (name, check) in &self.checks {
            let result = check.check_health().await;
            self.results_cache.insert(name.clone(), (result.clone(), Utc::now()));
            results.push(result);
        }

        results
    }

    /// Run a specific health check
    pub async fn run_check(&mut self, name: &str) -> Option<HealthCheckResult> {
        if let Some(check) = self.checks.get(name) {
            let result = check.check_health().await;
            self.results_cache.insert(name.to_string(), (result.clone(), Utc::now()));
            Some(result)
        } else {
            None
        }
    }

    /// Get cached health check result
    pub fn get_cached_result(&self, name: &str) -> Option<&HealthCheckResult> {
        if let Some((result, timestamp)) = self.results_cache.get(name) {
            // Check if cache is still valid
            if Utc::now().signed_duration_since(*timestamp).num_seconds() < self.cache_ttl_seconds as i64 {
                Some(result)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get overall system health status
    pub fn get_overall_health(&self) -> HealthStatus {
        let mut has_degraded = false;
        let mut has_unhealthy = false;

        for (_, (result, _)) in &self.results_cache {
            match result.status {
                HealthStatus::Unhealthy => has_unhealthy = true,
                HealthStatus::Degraded => has_degraded = true,
                _ => {}
            }
        }

        if has_unhealthy {
            HealthStatus::Unhealthy
        } else if has_degraded {
            HealthStatus::Degraded
        } else if !self.results_cache.is_empty() {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        }
    }

    /// Get all registered check names
    pub fn get_check_names(&self) -> Vec<&str> {
        self.checks.keys().map(|s| s.as_str()).collect()
    }

    /// Clear the results cache
    pub fn clear_cache(&mut self) {
        self.results_cache.clear();
    }

    /// Remove a health check
    pub fn remove_check(&mut self, name: &str) -> bool {
        let removed = self.checks.remove(name).is_some();
        if removed {
            self.results_cache.remove(name);
        }
        removed
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple health check implementation
pub struct SimpleHealthCheck<F> {
    name: String,
    check_fn: F,
}

impl<F, Fut> SimpleHealthCheck<F>
where
    F: Fn() -> Fut + Send + Sync,
    Fut: std::future::Future<Output = HealthCheckResult> + Send,
{
    /// Create a new simple health check
    pub fn new(name: impl Into<String>, check_fn: F) -> Self {
        Self {
            name: name.into(),
            check_fn,
        }
    }
}

#[async_trait]
impl<F, Fut> HealthCheck for SimpleHealthCheck<F>
where
    F: Fn() -> Fut + Send + Sync,
    Fut: std::future::Future<Output = HealthCheckResult> + Send,
{
    async fn check_health(&self) -> HealthCheckResult {
        (self.check_fn)().await
    }

    fn component_name(&self) -> &str {
        &self.name
    }
}

/// Database health check implementation
pub struct DatabaseHealthCheck {
    name: String,
    connection_string: String,
}

impl DatabaseHealthCheck {
    /// Create a new database health check
    pub fn new(name: impl Into<String>, connection_string: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            connection_string: connection_string.into(),
        }
    }
}

#[async_trait]
impl HealthCheck for DatabaseHealthCheck {
    async fn check_health(&self) -> HealthCheckResult {
        let start = std::time::Instant::now();

        // Simulate database connection check
        // In real implementation, this would actually test the database connection
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let duration = start.elapsed().as_millis() as u64;

        HealthCheckResult {
            component: self.name.clone(),
            status: HealthStatus::Healthy,
            timestamp: Utc::now(),
            error_message: None,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("connection_string".to_string(),
                    serde_json::Value::String("***masked***".to_string()));
                meta.insert("check_duration_ms".to_string(),
                    serde_json::json!(duration));
                meta
            },
            duration_ms: duration,
        }
    }

    fn component_name(&self) -> &str {
        &self.name
    }
}

/// HTTP endpoint health check implementation
pub struct HttpHealthCheck {
    name: String,
    url: String,
    timeout_ms: u64,
}

impl HttpHealthCheck {
    /// Create a new HTTP health check
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            url: url.into(),
            timeout_ms: 5000, // 5 second timeout
        }
    }

    /// Create with custom timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
}

#[async_trait]
impl HealthCheck for HttpHealthCheck {
    async fn check_health(&self) -> HealthCheckResult {
        let start = std::time::Instant::now();

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(self.timeout_ms))
            .build();

        let result = match client {
            Ok(client) => {
                match client.get(&self.url).send().await {
                    Ok(response) if response.status().is_success() => HealthStatus::Healthy,
                    Ok(_) => HealthStatus::Degraded,
                    Err(e) => {
                        if e.is_timeout() {
                            HealthStatus::Unhealthy
                        } else {
                            HealthStatus::Degraded
                        }
                    }
                }
            }
            Err(_) => HealthStatus::Unhealthy,
        };

        let duration = start.elapsed().as_millis() as u64;

        let (status, error_msg) = match result {
            HealthStatus::Healthy => (HealthStatus::Healthy, None),
            HealthStatus::Degraded => (HealthStatus::Degraded, Some("HTTP endpoint returned non-2xx status".to_string())),
            HealthStatus::Unhealthy => (HealthStatus::Unhealthy, Some("HTTP endpoint unreachable".to_string())),
            _ => (HealthStatus::Unknown, Some("Unknown health check error".to_string())),
        };

        HealthCheckResult {
            component: self.name.clone(),
            status,
            timestamp: Utc::now(),
            error_message: error_msg,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("url".to_string(), serde_json::json!(self.url));
                meta.insert("timeout_ms".to_string(), serde_json::json!(self.timeout_ms));
                meta.insert("check_duration_ms".to_string(), serde_json::json!(duration));
                meta
            },
            duration_ms: duration,
        }
    }

    fn component_name(&self) -> &str {
        &self.name
    }
}
