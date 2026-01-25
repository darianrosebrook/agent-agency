//! Health Checks
//!
//! System health monitoring and status reporting.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Component is healthy
    Healthy,
    /// Component is degraded but operational
    Degraded,
    /// Component is unhealthy
    Unhealthy,
    /// Health status unknown
    Unknown,
}

impl HealthStatus {
    pub fn is_healthy(&self) -> bool {
        matches!(self, Self::Healthy)
    }

    pub fn is_operational(&self) -> bool {
        matches!(self, Self::Healthy | Self::Degraded)
    }
}

/// Health check result for a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub component: String,
    pub status: HealthStatus,
    pub message: Option<String>,
    pub details: HashMap<String, String>,
    pub checked_at: DateTime<Utc>,
    pub response_time_ms: u64,
}

impl HealthCheck {
    pub fn healthy(component: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            status: HealthStatus::Healthy,
            message: None,
            details: HashMap::new(),
            checked_at: Utc::now(),
            response_time_ms: 0,
        }
    }

    pub fn degraded(component: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            status: HealthStatus::Degraded,
            message: Some(message.into()),
            details: HashMap::new(),
            checked_at: Utc::now(),
            response_time_ms: 0,
        }
    }

    pub fn unhealthy(component: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            status: HealthStatus::Unhealthy,
            message: Some(message.into()),
            details: HashMap::new(),
            checked_at: Utc::now(),
            response_time_ms: 0,
        }
    }

    pub fn with_detail(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.details.insert(key.into(), value.into());
        self
    }

    pub fn with_response_time(mut self, ms: u64) -> Self {
        self.response_time_ms = ms;
        self
    }
}

/// Overall system health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub status: HealthStatus,
    pub checks: Vec<HealthCheck>,
    pub generated_at: DateTime<Utc>,
    pub version: String,
}

impl HealthReport {
    pub fn new(checks: Vec<HealthCheck>, version: impl Into<String>) -> Self {
        let status = Self::aggregate_status(&checks);
        Self {
            status,
            checks,
            generated_at: Utc::now(),
            version: version.into(),
        }
    }

    fn aggregate_status(checks: &[HealthCheck]) -> HealthStatus {
        if checks.is_empty() {
            return HealthStatus::Unknown;
        }

        let has_unhealthy = checks.iter().any(|c| c.status == HealthStatus::Unhealthy);
        let has_degraded = checks.iter().any(|c| c.status == HealthStatus::Degraded);
        let has_unknown = checks.iter().any(|c| c.status == HealthStatus::Unknown);

        if has_unhealthy {
            HealthStatus::Unhealthy
        } else if has_degraded {
            HealthStatus::Degraded
        } else if has_unknown {
            HealthStatus::Unknown
        } else {
            HealthStatus::Healthy
        }
    }

    pub fn is_healthy(&self) -> bool {
        self.status.is_healthy()
    }

    pub fn is_operational(&self) -> bool {
        self.status.is_operational()
    }

    pub fn unhealthy_components(&self) -> Vec<&str> {
        self.checks
            .iter()
            .filter(|c| c.status == HealthStatus::Unhealthy)
            .map(|c| c.component.as_str())
            .collect()
    }
}

/// Health checker trait
pub trait HealthChecker: Send + Sync {
    fn name(&self) -> &str;
    fn check(&self) -> HealthCheck;
}

/// Simple health check based on a condition
pub struct SimpleHealthCheck {
    name: String,
    check_fn: Box<dyn Fn() -> bool + Send + Sync>,
}

impl SimpleHealthCheck {
    pub fn new<F>(name: impl Into<String>, check_fn: F) -> Self
    where
        F: Fn() -> bool + Send + Sync + 'static,
    {
        Self {
            name: name.into(),
            check_fn: Box::new(check_fn),
        }
    }
}

impl HealthChecker for SimpleHealthCheck {
    fn name(&self) -> &str {
        &self.name
    }

    fn check(&self) -> HealthCheck {
        let start = std::time::Instant::now();
        let is_healthy = (self.check_fn)();
        let elapsed = start.elapsed();

        if is_healthy {
            HealthCheck::healthy(&self.name)
                .with_response_time(elapsed.as_millis() as u64)
        } else {
            HealthCheck::unhealthy(&self.name, "Check failed")
                .with_response_time(elapsed.as_millis() as u64)
        }
    }
}

/// Health monitor for managing health checks
pub struct HealthMonitor {
    checkers: RwLock<Vec<Arc<dyn HealthChecker>>>,
    version: String,
    last_report: RwLock<Option<HealthReport>>,
}

impl HealthMonitor {
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            checkers: RwLock::new(Vec::new()),
            version: version.into(),
            last_report: RwLock::new(None),
        }
    }

    /// Register a health checker
    pub fn register(&self, checker: Arc<dyn HealthChecker>) {
        self.checkers.write().unwrap().push(checker);
    }

    /// Register a simple health check
    pub fn register_simple<F>(&self, name: impl Into<String>, check_fn: F)
    where
        F: Fn() -> bool + Send + Sync + 'static,
    {
        self.register(Arc::new(SimpleHealthCheck::new(name, check_fn)));
    }

    /// Run all health checks
    pub fn check(&self) -> HealthReport {
        let checkers = self.checkers.read().unwrap();
        let checks: Vec<HealthCheck> = checkers.iter().map(|c| c.check()).collect();
        let report = HealthReport::new(checks, &self.version);

        *self.last_report.write().unwrap() = Some(report.clone());
        report
    }

    /// Get the last health report without running new checks
    pub fn last_report(&self) -> Option<HealthReport> {
        self.last_report.read().unwrap().clone()
    }

    /// Get checker count
    pub fn checker_count(&self) -> usize {
        self.checkers.read().unwrap().len()
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new("unknown")
    }
}

/// Liveness check - is the service alive?
pub fn liveness_check() -> HealthCheck {
    HealthCheck::healthy("liveness")
}

/// Readiness check - is the service ready to accept requests?
pub struct ReadinessCheck {
    conditions: RwLock<HashMap<String, bool>>,
}

impl ReadinessCheck {
    pub fn new() -> Self {
        Self {
            conditions: RwLock::new(HashMap::new()),
        }
    }

    pub fn set_ready(&self, component: &str, ready: bool) {
        self.conditions
            .write()
            .unwrap()
            .insert(component.to_string(), ready);
    }

    pub fn is_ready(&self) -> bool {
        let conditions = self.conditions.read().unwrap();
        !conditions.is_empty() && conditions.values().all(|&v| v)
    }
}

impl Default for ReadinessCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthChecker for ReadinessCheck {
    fn name(&self) -> &str {
        "readiness"
    }

    fn check(&self) -> HealthCheck {
        if self.is_ready() {
            HealthCheck::healthy("readiness")
        } else {
            let conditions = self.conditions.read().unwrap();
            let not_ready: Vec<_> = conditions
                .iter()
                .filter(|(_, &v)| !v)
                .map(|(k, _)| k.as_str())
                .collect();
            HealthCheck::unhealthy("readiness", format!("Not ready: {:?}", not_ready))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status() {
        assert!(HealthStatus::Healthy.is_healthy());
        assert!(HealthStatus::Healthy.is_operational());
        assert!(!HealthStatus::Degraded.is_healthy());
        assert!(HealthStatus::Degraded.is_operational());
        assert!(!HealthStatus::Unhealthy.is_operational());
    }

    #[test]
    fn test_health_check_creation() {
        let check = HealthCheck::healthy("test")
            .with_detail("version", "1.0")
            .with_response_time(50);

        assert_eq!(check.status, HealthStatus::Healthy);
        assert_eq!(check.details.get("version"), Some(&"1.0".to_string()));
        assert_eq!(check.response_time_ms, 50);
    }

    #[test]
    fn test_health_report_aggregation() {
        let checks = vec![
            HealthCheck::healthy("a"),
            HealthCheck::healthy("b"),
        ];
        let report = HealthReport::new(checks, "1.0");
        assert_eq!(report.status, HealthStatus::Healthy);

        let checks = vec![
            HealthCheck::healthy("a"),
            HealthCheck::degraded("b", "slow"),
        ];
        let report = HealthReport::new(checks, "1.0");
        assert_eq!(report.status, HealthStatus::Degraded);

        let checks = vec![
            HealthCheck::healthy("a"),
            HealthCheck::unhealthy("b", "down"),
        ];
        let report = HealthReport::new(checks, "1.0");
        assert_eq!(report.status, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_simple_health_check() {
        let check = SimpleHealthCheck::new("test", || true);
        let result = check.check();
        assert_eq!(result.status, HealthStatus::Healthy);

        let check = SimpleHealthCheck::new("test", || false);
        let result = check.check();
        assert_eq!(result.status, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_health_monitor() {
        let monitor = HealthMonitor::new("1.0.0");
        monitor.register_simple("always_healthy", || true);
        monitor.register_simple("always_unhealthy", || false);

        let report = monitor.check();
        assert_eq!(report.status, HealthStatus::Unhealthy);
        assert_eq!(report.checks.len(), 2);
    }

    #[test]
    fn test_readiness_check() {
        let readiness = ReadinessCheck::new();
        assert!(!readiness.is_ready()); // No conditions = not ready

        readiness.set_ready("database", true);
        readiness.set_ready("cache", true);
        assert!(readiness.is_ready());

        readiness.set_ready("cache", false);
        assert!(!readiness.is_ready());
    }

    #[test]
    fn test_liveness() {
        let check = liveness_check();
        assert_eq!(check.status, HealthStatus::Healthy);
        assert_eq!(check.component, "liveness");
    }

    #[test]
    fn test_unhealthy_components() {
        let checks = vec![
            HealthCheck::healthy("a"),
            HealthCheck::unhealthy("b", "down"),
            HealthCheck::unhealthy("c", "error"),
        ];
        let report = HealthReport::new(checks, "1.0");

        let unhealthy = report.unhealthy_components();
        assert_eq!(unhealthy.len(), 2);
        assert!(unhealthy.contains(&"b"));
        assert!(unhealthy.contains(&"c"));
    }
}
