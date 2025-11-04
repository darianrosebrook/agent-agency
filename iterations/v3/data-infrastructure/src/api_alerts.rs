//! Reliability Monitor Trait
//!
//! Defines the interface for monitoring system reliability metrics,
//! compliance status, and recovery operations.

use schemars::JsonSchema;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Compliance status for RTO/RPO monitoring
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ComplianceStatus {
    #[schemars(with = "String")]

    pub timestamp: DateTime<Utc>,
    pub overall_compliant: bool,
    pub rto_compliant: bool,
    pub rpo_compliant: bool,
    pub service_status: std::collections::HashMap<String, ServiceComplianceStatus>,
    pub violations: Vec<ComplianceViolation>,
    pub last_incident_response_time: Option<DateTime<Utc>>,
}

/// Service compliance status
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ServiceComplianceStatus {
    pub service_name: String,
    pub current_rto_seconds: u64,
    pub current_rpo_seconds: u64,
    pub last_recovery_time: Option<DateTime<Utc>>,
    pub compliance_percentage: f64,
}

/// Compliance violation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ComplianceViolation {
    #[schemars(with = "String")]
    pub id: Uuid,
    #[schemars(with = "String")]

    pub timestamp: DateTime<Utc>,
    pub violation_type: ViolationType,
    pub severity: ViolationSeverity,
    pub description: String,
    pub service_type: String,
    pub measured_value: f64,
    pub objective_value: f64,
    pub resolved: bool,
    pub resolution_time: Option<DateTime<Utc>>,
}

/// Violation types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum ViolationType {
    RTOExceeded,
    RPOExceeded,
    ServiceUnavailable,
}

/// Violation severity levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Recovery metrics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RecoveryMetrics {
    pub total_incidents: u64,
    pub average_rto_seconds: u64,
    pub average_rpo_seconds: u64,
    pub compliance_rate_percent: f64,
    pub last_month_stats: MonthlyStats,
}

/// Monthly statistics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MonthlyStats {
    #[schemars(with = "String")]

    pub period_start: DateTime<Utc>,
    pub incidents: u64,
    pub violations: u64,
    pub average_recovery_time_seconds: u64,
    pub compliance_percentage: f64,
}

/// Compliance alert
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ComplianceAlert {
    #[schemars(with = "String")]
    pub id: Uuid,
    #[schemars(with = "String")]

    pub timestamp: DateTime<Utc>,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub affected_services: Vec<ServiceType>,
    pub recommended_actions: Vec<String>,
}

/// Alert types
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum AlertType {
    RTOViolation,
    RPOViolation,
    ServiceDegradation,
    SystemFailure,
    ServiceUnavailable,
    ComplianceThreshold,
    RecoveryFailure,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Severity levels (alias for AlertSeverity)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Service types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, JsonSchema)]
pub enum ServiceType {
    Database,
    API,
    Worker,
    Storage,
    Network,
    ApiServer,
    WorkerPool,
    MessageQueue,
    Cache,
    FileStorage,
    ExternalApi,
}

/// Alert manager for handling compliance alerts
#[derive(Debug, Clone, JsonSchema)]
pub struct AlertManager {
    pub alerts: std::collections::HashMap<Uuid, ComplianceAlert>,
}

impl AlertManager {
    pub fn new() -> Self {
        Self {
            alerts: std::collections::HashMap::new(),
        }
    }
    
    pub async fn add_alert(&mut self, alert: ComplianceAlert) {
        self.alerts.insert(alert.id, alert);
    }
    
    pub async fn get_alerts(&self) -> Vec<ComplianceAlert> {
        self.alerts.values().cloned().collect()
    }
    
    pub async fn get_active_alerts(&self) -> Result<Vec<ComplianceAlert>, Box<dyn std::error::Error>> {
        Ok(self.alerts.values().cloned().collect())
    }
    
    pub async fn get_alert_history(&self) -> Result<Vec<ComplianceAlert>, Box<dyn std::error::Error>> {
        Ok(self.alerts.values().cloned().collect())
    }
    
    pub async fn get_alert_statistics(&self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        Ok(serde_json::json!({
            "total": self.alerts.len(),
            "by_severity": {
                "critical": 0,
                "high": 0,
                "medium": 0,
                "low": 0
            }
        }))
    }
    
    pub async fn acknowledge_alert(&mut self, _alert_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    
    pub async fn resolve_alert(&mut self, _alert_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

/// Reliability monitoring trait
#[async_trait]
pub trait ReliabilityMonitor: Send + Sync {
    /// Get current compliance status
    async fn get_compliance_status(&self) -> Result<ComplianceStatus, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Get recent violations
    async fn get_recent_violations(&self, hours: i64) -> Result<Vec<ComplianceViolation>, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Get recovery metrics
    async fn get_recovery_metrics(&self) -> Result<RecoveryMetrics, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Get pending alerts
    async fn get_pending_alerts(&self) -> Result<Vec<ComplianceAlert>, Box<dyn std::error::Error + Send + Sync>>;
}
