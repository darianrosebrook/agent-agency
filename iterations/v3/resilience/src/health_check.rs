//! Health Check Implementation
//!
//! Provides comprehensive health checking capabilities for services,
//! including dependency health, resource monitoring, and health aggregation.
//!
//! Ported from V2 health check patterns with Rust optimizations.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio::time::{interval, sleep};
use tracing::{debug, error, info, warn};

/// Health check status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Service is healthy
    Healthy,
    /// Service is degraded but functional
    Degraded,
    /// Service is unhealthy
    Unhealthy,
    /// Health status is unknown
    Unknown,
}

impl HealthStatus {
    /// Get the priority of this status (higher = more critical)
    pub fn priority(&self) -> u8 {
        match self {
            HealthStatus::Healthy => 0,
            HealthStatus::Degraded => 1,
            HealthStatus::Unhealthy => 2,
            HealthStatus::Unknown => 3,
        }
    }

    /// Check if this status indicates a problem
    pub fn is_problematic(&self) -> bool {
        matches!(self, HealthStatus::Degraded | HealthStatus::Unhealthy | HealthStatus::Unknown)
    }
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub status: HealthStatus,
    pub message: String,
    pub timestamp: SystemTime,
    pub duration_ms: u64,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Health check name
    pub name: String,
    /// Check interval (seconds)
    pub interval_seconds: u64,
    /// Timeout for individual checks (seconds)
    pub timeout_seconds: u64,
    /// Number of consecutive failures before marking as unhealthy
    pub failure_threshold: u32,
    /// Number of consecutive successes before marking as healthy
    pub success_threshold: u32,
    /// Whether to enable this health check
    pub enabled: bool,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            interval_seconds: 30,
            timeout_seconds: 10,
            failure_threshold: 3,
            success_threshold: 2,
            enabled: true,
        }
    }
}

/// Health check trait
#[async_trait::async_trait]
pub trait HealthCheck: Send + Sync {
    /// Perform the health check
    async fn check(&self) -> Result<HealthCheckResult>;
    
    /// Get the health check name
    fn name(&self) -> &str;
    
    /// Get the health check configuration
    fn config(&self) -> &HealthCheckConfig;
}

/// Simple health check that always returns healthy
pub struct SimpleHealthCheck {
    config: HealthCheckConfig,
}

impl SimpleHealthCheck {
    pub fn new(config: HealthCheckConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl HealthCheck for SimpleHealthCheck {
    async fn check(&self) -> Result<HealthCheckResult> {
        Ok(HealthCheckResult {
            status: HealthStatus::Healthy,
            message: "Simple health check passed".to_string(),
            timestamp: SystemTime::now(),
            duration_ms: 0,
            metadata: HashMap::new(),
        })
    }

    fn name(&self) -> &str {
        &self.config.name
    }

    fn config(&self) -> &HealthCheckConfig {
        &self.config
    }
}

/// HTTP health check
pub struct HttpHealthCheck {
    config: HealthCheckConfig,
    url: String,
    client: reqwest::Client,
}

impl HttpHealthCheck {
    pub fn new(config: HealthCheckConfig, url: String) -> Self {
        Self {
            config,
            url,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl HealthCheck for HttpHealthCheck {
    async fn check(&self) -> Result<HealthCheckResult> {
        let start_time = SystemTime::now();
        
        let timeout = Duration::from_secs(self.config.timeout_seconds);
        let response = tokio::time::timeout(timeout, self.client.get(&self.url).send()).await;
        
        let duration = start_time.elapsed().unwrap_or_default().as_millis() as u64;
        
        match response {
            Ok(Ok(resp)) => {
                let status = if resp.status().is_success() {
                    HealthStatus::Healthy
                } else {
                    HealthStatus::Degraded
                };
                
                Ok(HealthCheckResult {
                    status,
                    message: format!("HTTP check returned status {}", resp.status()),
                    timestamp: SystemTime::now(),
                    duration_ms: duration,
                    metadata: {
                        let mut metadata = HashMap::new();
                        metadata.insert("status_code".to_string(), resp.status().as_u16().into());
                        metadata.insert("url".to_string(), self.url.clone().into());
                        metadata
                    },
                })
            }
            Ok(Err(e)) => {
                Ok(HealthCheckResult {
                    status: HealthStatus::Unhealthy,
                    message: format!("HTTP check failed: {}", e),
                    timestamp: SystemTime::now(),
                    duration_ms: duration,
                    metadata: {
                        let mut metadata = HashMap::new();
                        metadata.insert("url".to_string(), self.url.clone().into());
                        metadata.insert("error".to_string(), e.to_string().into());
                        metadata
                    },
                })
            }
            Err(_) => {
                Ok(HealthCheckResult {
                    status: HealthStatus::Unhealthy,
                    message: "HTTP check timed out".to_string(),
                    timestamp: SystemTime::now(),
                    duration_ms: duration,
                    metadata: {
                        let mut metadata = HashMap::new();
                        metadata.insert("url".to_string(), self.url.clone().into());
                        metadata.insert("timeout_seconds".to_string(), self.config.timeout_seconds.into());
                        metadata
                    },
                })
            }
        }
    }

    fn name(&self) -> &str {
        &self.config.name
    }

    fn config(&self) -> &HealthCheckConfig {
        &self.config
    }
}

/// Health check manager
pub struct HealthCheckManager {
    checks: Arc<RwLock<HashMap<String, Arc<dyn HealthCheck>>>>,
    results: Arc<RwLock<HashMap<String, HealthCheckResult>>>,
    failure_counts: Arc<RwLock<HashMap<String, u32>>>,
    success_counts: Arc<RwLock<HashMap<String, u32>>>,
}

impl HealthCheckManager {
    /// Create a new health check manager
    pub fn new() -> Self {
        Self {
            checks: Arc::new(RwLock::new(HashMap::new())),
            results: Arc::new(RwLock::new(HashMap::new())),
            failure_counts: Arc::new(RwLock::new(HashMap::new())),
            success_counts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a health check
    pub async fn add_check(&self, check: Arc<dyn HealthCheck>) {
        let name = check.name().to_string();
        self.checks.write().await.insert(name.clone(), check);
        self.results.write().await.insert(name.clone(), HealthCheckResult {
            status: HealthStatus::Unknown,
            message: "Health check not yet run".to_string(),
            timestamp: SystemTime::now(),
            duration_ms: 0,
            metadata: HashMap::new(),
        });
        self.failure_counts.write().await.insert(name.clone(), 0);
        self.success_counts.write().await.insert(name, 0);
    }

    /// Remove a health check
    pub async fn remove_check(&self, name: &str) {
        self.checks.write().await.remove(name);
        self.results.write().await.remove(name);
        self.failure_counts.write().await.remove(name);
        self.success_counts.write().await.remove(name);
    }

    /// Run a specific health check
    pub async fn run_check(&self, name: &str) -> Result<HealthCheckResult> {
        let check = {
            let checks = self.checks.read().await;
            checks.get(name).cloned()
        };

        match check {
            Some(check) => {
                let result = check.check().await?;
                self.update_check_result(name, &result).await;
                Ok(result)
            }
            None => Err(anyhow::anyhow!("Health check '{}' not found", name))
        }
    }

    /// Run all health checks
    pub async fn run_all_checks(&self) -> HashMap<String, HealthCheckResult> {
        let checks = {
            let checks = self.checks.read().await;
            checks.clone()
        };

        let mut results = HashMap::new();
        
        for (name, check) in checks {
            if check.config().enabled {
                match check.check().await {
                    Ok(result) => {
                        self.update_check_result(&name, &result).await;
                        results.insert(name, result);
                    }
                    Err(e) => {
                        error!("Health check '{}' failed: {}", name, e);
                        let error_result = HealthCheckResult {
                            status: HealthStatus::Unhealthy,
                            message: format!("Health check failed: {}", e),
                            timestamp: SystemTime::now(),
                            duration_ms: 0,
                            metadata: HashMap::new(),
                        };
                        self.update_check_result(&name, &error_result).await;
                        results.insert(name, error_result);
                    }
                }
            }
        }

        results
    }

    /// Get the overall health status
    pub async fn get_overall_health(&self) -> HealthStatus {
        let results = self.results.read().await;
        
        if results.is_empty() {
            return HealthStatus::Unknown;
        }

        let mut worst_status = HealthStatus::Healthy;
        
        for result in results.values() {
            if result.status.priority() > worst_status.priority() {
                worst_status = result.status.clone();
            }
        }

        worst_status
    }

    /// Get all health check results
    pub async fn get_all_results(&self) -> HashMap<String, HealthCheckResult> {
        self.results.read().await.clone()
    }

    /// Start the health check manager (runs checks periodically)
    pub async fn start(&self) {
        let checks = Arc::clone(&self.checks);
        let results = Arc::clone(&self.results);
        let failure_counts = Arc::clone(&self.failure_counts);
        let success_counts = Arc::clone(&self.success_counts);

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30)); // Default interval
            
            loop {
                interval.tick().await;
                
                let checks_to_run = {
                    let checks = checks.read().await;
                    checks.clone()
                };

                for (name, check) in checks_to_run {
                    if check.config().enabled {
                        let interval_duration = Duration::from_secs(check.config().interval_seconds);
                        sleep(interval_duration).await;
                        
                        match check.check().await {
                            Ok(result) => {
                                // Update failure/success counts
                                {
                                    let mut failure_counts = failure_counts.write().await;
                                    let mut success_counts = success_counts.write().await;
                                    
                                    if result.status.is_problematic() {
                                        *failure_counts.entry(name.clone()).or_insert(0) += 1;
                                        *success_counts.entry(name.clone()).or_insert(0) = 0;
                                    } else {
                                        *success_counts.entry(name.clone()).or_insert(0) += 1;
                                        *failure_counts.entry(name.clone()).or_insert(0) = 0;
                                    }
                                }
                                
                                // Update results
                                {
                                    let mut results = results.write().await;
                                    results.insert(name, result);
                                }
                            }
                            Err(e) => {
                                error!("Health check '{}' failed: {}", name, e);
                            }
                        }
                    }
                }
            }
        });
    }

    /// Update check result and apply thresholds
    async fn update_check_result(&self, name: &str, result: &HealthCheckResult) {
        let mut results = self.results.write().await;
        let mut failure_counts = self.failure_counts.write().await;
        let mut success_counts = self.success_counts.write().await;
        
        let failure_count = failure_counts.entry(name.to_string()).or_insert(0);
        let success_count = success_counts.entry(name.to_string()).or_insert(0);
        
        let mut final_result = result.clone();
        
        // Apply failure threshold
        if result.status.is_problematic() {
            *failure_count += 1;
            *success_count = 0;
            
            if *failure_count >= 3 { // Default threshold
                final_result.status = HealthStatus::Unhealthy;
                final_result.message = format!("{} ({} consecutive failures)", result.message, failure_count);
            }
        } else {
            *success_count += 1;
            *failure_count = 0;
            
            if *success_count >= 2 { // Default threshold
                final_result.status = HealthStatus::Healthy;
                final_result.message = format!("{} ({} consecutive successes)", result.message, success_count);
            }
        }
        
        results.insert(name.to_string(), final_result);
    }
}

impl Default for HealthCheckManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_health_status_priority() {
        assert_eq!(HealthStatus::Healthy.priority(), 0);
        assert_eq!(HealthStatus::Degraded.priority(), 1);
        assert_eq!(HealthStatus::Unhealthy.priority(), 2);
        assert_eq!(HealthStatus::Unknown.priority(), 3);
    }

    #[tokio::test]
    async fn test_health_status_is_problematic() {
        assert!(!HealthStatus::Healthy.is_problematic());
        assert!(HealthStatus::Degraded.is_problematic());
        assert!(HealthStatus::Unhealthy.is_problematic());
        assert!(HealthStatus::Unknown.is_problematic());
    }

    #[tokio::test]
    async fn test_simple_health_check() {
        let config = HealthCheckConfig::default();
        let check = SimpleHealthCheck::new(config);
        
        let result = check.check().await.unwrap();
        assert_eq!(result.status, HealthStatus::Healthy);
        assert_eq!(check.name(), "default");
    }

    #[tokio::test]
    async fn test_health_check_manager() {
        let manager = HealthCheckManager::new();
        
        let config = HealthCheckConfig {
            name: "test".to_string(),
            ..Default::default()
        };
        let check = Arc::new(SimpleHealthCheck::new(config));
        
        manager.add_check(check).await;
        
        let result = manager.run_check("test").await.unwrap();
        assert_eq!(result.status, HealthStatus::Healthy);
        
        let overall_health = manager.get_overall_health().await;
        assert_eq!(overall_health, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_health_check_manager_remove() {
        let manager = HealthCheckManager::new();
        
        let config = HealthCheckConfig {
            name: "test".to_string(),
            ..Default::default()
        };
        let check = Arc::new(SimpleHealthCheck::new(config));
        
        manager.add_check(check).await;
        assert!(manager.run_check("test").await.is_ok());
        
        manager.remove_check("test").await;
        assert!(manager.run_check("test").await.is_err());
    }
}
