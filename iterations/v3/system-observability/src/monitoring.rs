//! Monitoring functionality
//!
//! Real-time monitoring, alerting, and health checks.

use crate::{TelemetryCollector, TelemetryData, TelemetryDataType, TelemetryError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Health monitor for system components
pub struct HealthMonitor {
    name: String,
    collection_interval: std::time::Duration,
    component_checks: Arc<RwLock<HashMap<String, ComponentHealth>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub component_name: String,
    pub status: HealthStatus,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub response_time_ms: Option<u64>,
    pub error_message: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl HealthMonitor {
    pub fn new(name: String, collection_interval: std::time::Duration) -> Self {
        Self {
            name,
            collection_interval,
            component_checks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a component for health monitoring
    pub async fn register_component(&self, component_name: String) {
        let health = ComponentHealth {
            component_name: component_name.clone(),
            status: HealthStatus::Unknown,
            last_check: chrono::Utc::now(),
            response_time_ms: None,
            error_message: None,
            metadata: serde_json::json!({}),
        };

        let mut checks = self.component_checks.write().await;
        checks.insert(component_name, health);
    }

    /// Update component health status
    pub async fn update_health(
        &self,
        component_name: String,
        status: HealthStatus,
        response_time_ms: Option<u64>,
        error_message: Option<String>,
        metadata: serde_json::Value,
    ) {
        let health = ComponentHealth {
            component_name: component_name.clone(),
            status,
            last_check: chrono::Utc::now(),
            response_time_ms,
            error_message,
            metadata,
        };

        let mut checks = self.component_checks.write().await;
        checks.insert(component_name, health);
    }

    /// Perform health check on a component
    pub async fn check_component_health(
        &self,
        component_name: &str,
    ) -> Result<HealthStatus, TelemetryError> {
        // Mock health check - in real implementation, this would actually check the component
        let status = match component_name {
            "database" => HealthStatus::Healthy,
            "api-server" => HealthStatus::Healthy,
            "inference-engine" => HealthStatus::Degraded,
            _ => HealthStatus::Unknown,
        };

        self.update_health(
            component_name.to_string(),
            status,
            Some(150), // Mock response time
            None,
            serde_json::json!({"mock": true}),
        )
        .await;

        Ok(status)
    }

    /// Get overall system health
    pub async fn get_system_health(&self) -> SystemHealth {
        let checks = self.component_checks.read().await;

        let mut healthy_count = 0;
        let mut degraded_count = 0;
        let mut unhealthy_count = 0;
        let mut unknown_count = 0;
        let mut total_response_time = 0u64;
        let mut response_time_count = 0usize;

        for health in checks.values() {
            match health.status {
                HealthStatus::Healthy => healthy_count += 1,
                HealthStatus::Degraded => degraded_count += 1,
                HealthStatus::Unhealthy => unhealthy_count += 1,
                HealthStatus::Unknown => unknown_count += 1,
            }

            if let Some(response_time) = health.response_time_ms {
                total_response_time += response_time;
                response_time_count += 1;
            }
        }

        let total_components = checks.len();
        let overall_status = if unhealthy_count > 0 {
            HealthStatus::Unhealthy
        } else if degraded_count > 0 {
            HealthStatus::Degraded
        } else if healthy_count > 0 {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        };

        let avg_response_time = if response_time_count > 0 {
            Some(total_response_time / response_time_count as u64)
        } else {
            None
        };

        SystemHealth {
            overall_status,
            total_components,
            healthy_components: healthy_count,
            degraded_components: degraded_count,
            unhealthy_components: unhealthy_count,
            unknown_components: unknown_count,
            average_response_time_ms: avg_response_time,
            last_updated: chrono::Utc::now(),
        }
    }
}

#[async_trait]
impl TelemetryCollector for HealthMonitor {
    async fn collect(&self) -> Result<TelemetryData, TelemetryError> {
        let system_health = self.get_system_health().await;

        let data = TelemetryData {
            timestamp: chrono::Utc::now(),
            source: self.name.clone(),
            data_type: TelemetryDataType::Metric,
            payload: serde_json::json!({
                "overall_status": system_health.overall_status,
                "total_components": system_health.total_components,
                "healthy_components": system_health.healthy_components,
                "degraded_components": system_health.degraded_components,
                "unhealthy_components": system_health.unhealthy_components,
                "average_response_time_ms": system_health.average_response_time_ms
            }),
            tags: {
                let mut tags = HashMap::new();
                tags.insert("category".to_string(), "health".to_string());
                tags.insert("monitor".to_string(), "system".to_string());
                tags
            },
        };

        Ok(data)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn interval(&self) -> std::time::Duration {
        self.collection_interval
    }
}

impl std::fmt::Debug for HealthMonitor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HealthMonitor")
            .field("name", &self.name)
            .field("collection_interval", &self.collection_interval)
            .finish()
    }
}

/// System health summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub overall_status: HealthStatus,
    pub total_components: usize,
    pub healthy_components: usize,
    pub degraded_components: usize,
    pub unhealthy_components: usize,
    pub unknown_components: usize,
    pub average_response_time_ms: Option<u64>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Alert manager for telemetry events
pub struct AlertManager {
    name: String,
    alerts: Arc<RwLock<Vec<Alert>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub severity: AlertSeverity,
    pub title: String,
    pub description: String,
    pub source: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub acknowledged: bool,
    pub resolved: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl AlertManager {
    pub fn new(name: String) -> Self {
        Self {
            name,
            alerts: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get the name of this alert manager
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Create a new alert
    pub async fn create_alert(
        &self,
        severity: AlertSeverity,
        title: String,
        description: String,
        source: String,
    ) {
        let alert = Alert {
            id: format!("alert_{}", chrono::Utc::now().timestamp_millis()),
            severity,
            title,
            description,
            source,
            timestamp: chrono::Utc::now(),
            acknowledged: false,
            resolved: false,
        };

        let mut alerts = self.alerts.write().await;
        alerts.push(alert);
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Vec<Alert> {
        let alerts = self.alerts.read().await;
        alerts
            .iter()
            .filter(|alert| !alert.resolved)
            .cloned()
            .collect()
    }

    /// Acknowledge an alert
    pub async fn acknowledge_alert(&self, alert_id: &str) -> Result<(), TelemetryError> {
        let mut alerts = self.alerts.write().await;
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.acknowledged = true;
            Ok(())
        } else {
            Err(TelemetryError::ProcessingFailed {
                message: format!("Alert not found: {}", alert_id),
            })
        }
    }

    /// Resolve an alert
    pub async fn resolve_alert(&self, alert_id: &str) -> Result<(), TelemetryError> {
        let mut alerts = self.alerts.write().await;
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.resolved = true;
            Ok(())
        } else {
            Err(TelemetryError::ProcessingFailed {
                message: format!("Alert not found: {}", alert_id),
            })
        }
    }
}
