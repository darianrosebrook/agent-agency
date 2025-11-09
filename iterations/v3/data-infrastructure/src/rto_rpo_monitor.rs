//! RTO/RPO Monitoring and Compliance System
//!
//! Provides real-time monitoring and alerting for Recovery Time Objectives (RTO)
//! and Recovery Point Objectives (RPO) to ensure disaster recovery compliance.

use schemars::JsonSchema;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, mpsc};
use tokio::time;
use tracing::{info, warn, error};
use uuid::Uuid;

use crate::api_alerts::{
    ReliabilityMonitor, ComplianceStatus, ComplianceViolation, RecoveryMetrics, ComplianceAlert,
    ServiceComplianceStatus, ViolationType, ViolationSeverity, AlertType, AlertSeverity, MonthlyStats
};
use crate::service_failover::{ServiceFailoverManager, ServiceType, ServiceStatus};

/// RTO/RPO objectives configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RecoveryObjectives {
    /// Recovery Time Objective (seconds) - max time to restore service
    pub rto_seconds: u64,
    /// Recovery Point Objective (seconds) - max data loss acceptable
    pub rpo_seconds: u64,
    /// Service-specific objectives
    pub service_objectives: HashMap<ServiceType, ServiceRecoveryObjectives>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ServiceRecoveryObjectives {
    pub rto_seconds: u64,
    pub rpo_seconds: u64,
    pub critical_service: bool, // Requires immediate attention if violated
}

// Using ComplianceStatus and ServiceComplianceStatus from api_alerts module

// Using ComplianceViolation from api_alerts module

// Using ViolationType and ViolationSeverity from api_alerts module

// Using RecoveryMetrics and MonthlyStats from api_alerts module

/// Alert configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlertConfig {
    pub rto_violation_threshold_seconds: u64,
    pub rpo_violation_threshold_seconds: u64,
    pub max_violations_before_alert: u32,
    pub alert_cooldown_minutes: u64,
    pub email_alerts_enabled: bool,
    pub slack_alerts_enabled: bool,
    pub pager_duty_integration: bool,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            rto_violation_threshold_seconds: 3600, // 1 hour
            rpo_violation_threshold_seconds: 1800, // 30 minutes
            max_violations_before_alert: 3,
            alert_cooldown_minutes: 15,
            email_alerts_enabled: true,
            slack_alerts_enabled: false,
            pager_duty_integration: false,
        }
    }
}

/// RTO/RPO monitoring system
pub struct RtoRpoMonitor {
    objectives: RecoveryObjectives,
    alert_config: AlertConfig,
    compliance_status: Arc<RwLock<ComplianceStatus>>,
    violations: Arc<RwLock<Vec<ComplianceViolation>>>,
    recovery_metrics: Arc<RwLock<RecoveryMetrics>>,
    service_failover_manager: Arc<ServiceFailoverManager>,
    alert_sender: mpsc::UnboundedSender<ComplianceAlert>,
    alert_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<ComplianceAlert>>>>,
}

// Using ComplianceAlert and AlertType from api_alerts module

impl RtoRpoMonitor {
    /// Create a new RTO/RPO monitor
    pub fn new(
        objectives: RecoveryObjectives,
        alert_config: AlertConfig,
        service_failover_manager: Arc<ServiceFailoverManager>,
    ) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        Self {
            objectives,
            alert_config,
            compliance_status: Arc::new(RwLock::new(ComplianceStatus {
                timestamp: Utc::now(),
                overall_compliant: true,
                rto_compliant: true,
                rpo_compliant: true,
                service_status: HashMap::new(),
                violations: Vec::new(),
                last_incident_response_time: None,
            })),
            violations: Arc::new(RwLock::new(Vec::new())),
            recovery_metrics: Arc::new(RwLock::new(RecoveryMetrics {
                total_incidents: 0,
                average_rto_seconds: 0,
                average_rpo_seconds: 0,
                compliance_rate_percent: 100.0,
                last_month_stats: MonthlyStats {
                    period_start: Utc::now() - chrono::Duration::days(30),
                    incidents: 0,
                    violations: 0,
                    average_recovery_time_seconds: 0,
                    compliance_percentage: 100.0,
                },
            })),
            service_failover_manager,
            alert_sender: tx,
            alert_receiver: Arc::new(RwLock::new(Some(rx))),
        }
    }

    /// Start monitoring loop
    pub async fn start_monitoring(self) -> Result<(), String> {
        info!("Starting RTO/RPO monitoring");

        // Create Arc to share between tasks
        let monitor = Arc::new(self);

        // Start compliance check loop
        let compliance_monitor = Arc::clone(&monitor);
        tokio::spawn(async move {
            compliance_monitor.compliance_check_loop().await;
        });

        // Start alert processing loop
        let alert_monitor = Arc::clone(&monitor);
        tokio::spawn(async move {
            alert_monitor.alert_processing_loop().await;
        });

        Ok(())
    }

    /// Compliance check loop
    async fn compliance_check_loop(&self) {
        let interval = Duration::from_secs(60); // Check every minute

        loop {
            time::sleep(interval).await;

            if let Err(e) = self.perform_compliance_check().await {
                error!("Compliance check failed: {}", e);
            }
        }
    }

    /// Perform comprehensive compliance check
    async fn perform_compliance_check(&self) -> Result<(), String> {
        let service_status = self.service_failover_manager.get_service_status().await;
        let failover_history = self.service_failover_manager.get_failover_history(50).await;

        let mut service_compliance: HashMap<String, ServiceComplianceStatus> = HashMap::new();
        let mut violations = Vec::new();

        for (service_id, status) in service_status {
            // TODO: Implement robust service type parsing from service ID and metadata
            //       Currently uses basic string matching; should use service registry or metadata for accurate type identification.
            //
            // COMPLETION CHECKLIST:
            // [ ] Query service registry for service type information
            // [ ] Use service metadata for type identification
            // [ ] Handle service type aliases and variations
            // [ ] Support custom service types
            // [ ] Add fallback for unknown service types
            // [ ] Add unit tests for service type parsing
            // [ ] Add integration tests with real service registry
            // [ ] Verify service type parsing accuracy
            //
            // ACCEPTANCE CRITERIA:
            // - Service types are identified accurately from registry or metadata
            // - Service type aliases and variations are handled
            // - Unknown service types are handled gracefully
            // - Service type parsing is reliable and maintainable
            //
            // DEPENDENCIES:
            // - Service registry API (Required)
            // - Service metadata structure (Required)
            // - Service type definitions (Required)
            //
            // ESTIMATED EFFORT: 3-4 hours (medium confidence)
            // PRIORITY: Low
            // BLOCKING: No
            //
            // GOVERNANCE:
            // - CAWS Tier: 3 (low risk enhancement)
            // - Change Budget: ~60 LOC
            // - Reviewer Requirements: Service management expertise
            let service_type = if service_id.contains("api") {
                ServiceType::ApiServer
            } else if service_id.contains("db") {
                ServiceType::Database
            } else if service_id.contains("worker") {
                ServiceType::WorkerPool
            } else if service_id.contains("queue") {
                ServiceType::MessageQueue
            } else if service_id.contains("cache") {
                ServiceType::Cache
            } else if service_id.contains("storage") {
                ServiceType::FileStorage
            } else {
                ServiceType::ExternalApi
            };

            let default_objectives = ServiceRecoveryObjectives {
                rto_seconds: self.objectives.rto_seconds,
                rpo_seconds: self.objectives.rpo_seconds,
                critical_service: false,
            };
            let service_objectives = self.objectives.service_objectives.get(&service_type)
                .unwrap_or(&default_objectives);

            // Calculate current RTO (from failover history)
            let recent_failovers: Vec<_> = failover_history.iter()
                .filter(|event| matches!(event, crate::service_failover::FailoverEvent::FailoverCompleted { .. }))
                .take(5)
                .collect();

            let avg_rto = if !recent_failovers.is_empty() {
                // TODO: Track actual recovery times from failover history
                //       Currently uses basic estimate; should track actual recovery times from failover events.
                //
                // COMPLETION CHECKLIST:
                // [ ] Extract actual recovery times from failover events
                // [ ] Calculate average recovery time from historical data
                // [ ] Handle missing recovery time data gracefully
                // [ ] Support time-weighted averages for recent events
                // [ ] Add unit tests for recovery time calculation
                // [ ] Add integration tests with real failover data
                // [ ] Verify recovery time accuracy
                //
                // ACCEPTANCE CRITERIA:
                // - Recovery times are calculated from actual failover events
                // - Average recovery time reflects historical performance
                // - Missing data is handled gracefully
                // - Time-weighted averages prioritize recent events
                //
                // DEPENDENCIES:
                // - Failover history tracking (Required)
                // - Recovery time extraction utilities (Required)
                // - Statistical calculation utilities (Required)
                //
                // ESTIMATED EFFORT: 2-3 hours (medium confidence)
                // PRIORITY: Medium
                // BLOCKING: No
                //
                // GOVERNANCE:
                // - CAWS Tier: 2 (monitoring feature)
                // - Change Budget: ~50 LOC
                // - Reviewer Requirements: Monitoring and metrics expertise
                Some(service_objectives.rto_seconds / 2) // Temporary: basic estimate until actual recovery time tracking
            } else {
                None
            };

            // Check for service unavailability
            let service_available = status == ServiceStatus::Healthy;
            if !service_available {
                violations.push(ComplianceViolation {
                    id: Uuid::new_v4(),
                    timestamp: Utc::now(),
                    violation_type: ViolationType::ServiceUnavailable,
                    service_type: format!("{:?}", service_type),
                    severity: if service_objectives.critical_service {
                        ViolationSeverity::Critical
                    } else {
                        ViolationSeverity::High
                    },
                    description: format!("Service {} is not available (status: {:?})", service_id, status),
                    measured_value: 0.0, // Not applicable
                    objective_value: 0.0, // Not applicable
                    resolved: false,
                    resolution_time: None,
                });
            }

            // Check RTO compliance
            let rto_compliant = if let Some(current_rto) = avg_rto {
                current_rto <= service_objectives.rto_seconds
            } else {
                true // No recent failures to measure
            };

            if !rto_compliant && avg_rto.is_some() {
                violations.push(ComplianceViolation {
                    id: Uuid::new_v4(),
                    timestamp: Utc::now(),
                    violation_type: ViolationType::RTOExceeded,
                    service_type: service_type_as_string(service_type).to_string(),
                    severity: ViolationSeverity::High,
                    description: format!("RTO exceeded for {}: {}s > {}s objective",
                        service_type_as_string(service_type), avg_rto.unwrap(), service_objectives.rto_seconds),
                    measured_value: avg_rto.unwrap() as f64,
                    objective_value: service_objectives.rto_seconds as f64,
                    resolved: false,
                    resolution_time: None,
                });
            }

            // TODO: Check actual RPO compliance
            // - [ ] Query backup system for last backup timestamp
            // - [ ] Calculate time since last backup
            // - [ ] Compare against service RPO objectives
            // - [ ] Handle missing backup data gracefully
            // - [ ] Add unit tests with mock backup system
            // - [ ] Add integration tests with real backup system
            // TODO: Integrate with backup system to check actual RPO compliance
            //       Currently uses placeholder; should query backup system for actual backup age and compare against RPO objectives.
            //
            // COMPLETION CHECKLIST:
            // [ ] Query backup system for last backup timestamp
            // [ ] Calculate time since last backup
            // [ ] Compare against service RPO objectives
            // [ ] Handle missing backup data gracefully
            // [ ] Support multiple backup sources
            // [ ] Add unit tests with mock backup system
            // [ ] Add integration tests with real backup system
            // [ ] Verify RPO compliance accuracy
            //
            // ACCEPTANCE CRITERIA:
            // - Backup age is queried from backup system
            // - RPO compliance is calculated accurately
            // - Missing backup data is handled gracefully
            // - Multiple backup sources are supported
            //
            // DEPENDENCIES:
            // - Backup system API (Required)
            // - Backup metadata structure (Required)
            // - RPO calculation utilities (Required)
            //
            // ESTIMATED EFFORT: 3-4 hours (medium confidence)
            // PRIORITY: Medium
            // BLOCKING: No
            //
            // GOVERNANCE:
            // - CAWS Tier: 2 (monitoring feature)
            // - Change Budget: ~70 LOC
            // - Reviewer Requirements: Backup and monitoring expertise
            let rpo_compliant = true; // Temporary: placeholder until backup system integration

            service_compliance.insert(service_type_as_string(service_type).to_string(), ServiceComplianceStatus {
                service_name: service_type_as_string(service_type).to_string(),
                current_rto_seconds: avg_rto.unwrap_or(0),
                current_rpo_seconds: service_objectives.rpo_seconds,
                // TODO: Get actual last recovery time
                // - [ ] Query recovery system for last recovery timestamp
                // - [ ] Handle missing recovery data
                // - [ ] Add unit tests with mock recovery system
                // - [ ] Add integration tests with real recovery system
                last_recovery_time: Some(Utc::now() - chrono::Duration::hours(1)), // Placeholder
                // TODO: Calculate actual compliance percentage based on RTO/RPO metrics
                //       Currently uses basic binary calculation; should calculate percentage based on actual metrics and objectives.
                //
                // COMPLETION CHECKLIST:
                // [ ] Calculate compliance percentage from RTO/RPO metrics
                // [ ] Weight RTO and RPO compliance appropriately
                // [ ] Handle partial compliance scenarios
                // [ ] Support time-weighted compliance calculations
                // [ ] Add unit tests for compliance calculation
                // [ ] Add integration tests with various metrics
                // [ ] Verify compliance percentage accuracy
                //
                // ACCEPTANCE CRITERIA:
                // - Compliance percentage reflects actual RTO/RPO performance
                // - Partial compliance is calculated correctly
                // - Time-weighted calculations prioritize recent metrics
                // - Compliance percentage is accurate and meaningful
                //
                // DEPENDENCIES:
                // - RTO/RPO metrics (Required)
                // - Compliance calculation utilities (Required)
                // - Statistical calculation utilities (Required)
                //
                // ESTIMATED EFFORT: 2-3 hours (medium confidence)
                // PRIORITY: Low
                // BLOCKING: No
                //
                // GOVERNANCE:
                // - CAWS Tier: 3 (monitoring enhancement)
                // - Change Budget: ~40 LOC
                // - Reviewer Requirements: Metrics and monitoring expertise
                compliance_percentage: if rto_compliant && rpo_compliant { 100.0 } else { 0.0 }, // Temporary: basic binary calculation until proper percentage calculation
            });
        }

        // Update compliance status
        {
            let mut compliance_status = self.compliance_status.write().await;
            compliance_status.timestamp = Utc::now();
            compliance_status.service_status = service_compliance;
            compliance_status.violations = violations.clone();
            compliance_status.rto_compliant = violations.iter()
                .all(|v| v.violation_type != ViolationType::RTOExceeded);
            compliance_status.rpo_compliant = violations.iter()
                .all(|v| v.violation_type != ViolationType::RPOExceeded);
            compliance_status.overall_compliant =
                compliance_status.rto_compliant && compliance_status.rpo_compliant &&
                violations.iter().all(|v| v.violation_type != ViolationType::ServiceUnavailable);
        }

        // Store violations
        {
            let mut stored_violations = self.violations.write().await;
            stored_violations.extend(violations);
        }

        // Send alerts for violations
        self.send_violation_alerts().await?;

        Ok(())
    }

    /// Send alerts for compliance violations
    async fn send_violation_alerts(&self) -> Result<(), String> {
        let violations = self.violations.read().await;
        let recent_violations: Vec<_> = violations.iter()
            .filter(|v| !v.resolved && (Utc::now() - v.timestamp) < chrono::Duration::minutes(5))
            .collect();

        if recent_violations.len() >= self.alert_config.max_violations_before_alert as usize {
            let alert = ComplianceAlert {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                alert_type: AlertType::ComplianceThreshold,
                severity: AlertSeverity::High,
                message: format!("{} compliance violations detected in last 5 minutes", recent_violations.len()),
                affected_services: vec![], // TODO: Convert string service_type back to ServiceType enum
                recommended_actions: vec![
                    "Review system health dashboard".to_string(),
                    "Check recent failover events".to_string(),
                    "Verify backup procedures are running".to_string(),
                ],
            };

            let _ = self.alert_sender.send(alert);
        }

        Ok(())
    }

    /// Alert processing loop
    async fn alert_processing_loop(&self) {
        let mut receiver = {
            let mut rx = self.alert_receiver.write().await;
            rx.take().unwrap()
        };

        while let Some(alert) = receiver.recv().await {
            self.process_alert(&alert).await;
        }
    }

    /// Process compliance alert
    async fn process_alert(&self, alert: &ComplianceAlert) {
        warn!("Compliance Alert: {} - {}", alert.alert_type_as_string(), alert.message);

        // In a real system, this would:
        // - Send email notifications
        // - Post to Slack
        // - Create PagerDuty incident
        // - Update monitoring dashboards

        match alert.alert_type {
            AlertType::RTOViolation => {
                error!(" CRITICAL: RTO violation detected - immediate attention required");
            }
            AlertType::RPOViolation => {
                error!(" CRITICAL: RPO violation detected - data loss risk");
            }
            AlertType::ServiceUnavailable => {
                error!(" CRITICAL: Critical service unavailable");
            }
            AlertType::ComplianceThreshold => {
                warn!("⚠️  WARNING: Multiple compliance violations detected");
            }
            AlertType::RecoveryFailure => {
                error!(" CRITICAL: Recovery operation failed");
            }
            AlertType::ServiceDegradation => {
                warn!("⚠️  WARNING: Service performance degraded");
            }
            AlertType::SystemFailure => {
                error!(" CRITICAL: System failure detected");
            }
        }

        for action in &alert.recommended_actions {
            info!("Recommended action: {}", action);
        }
    }

    /// Get current compliance status
    pub async fn get_compliance_status(&self) -> ComplianceStatus {
        self.compliance_status.read().await.clone()
    }

    /// Get recent violations
    pub async fn get_recent_violations(&self, hours: i64) -> Vec<ComplianceViolation> {
        let cutoff = Utc::now() - chrono::Duration::hours(hours);
        let violations = self.violations.read().await;

        violations.iter()
            .filter(|v| v.timestamp > cutoff)
            .cloned()
            .collect()
    }

    /// Get recovery metrics
    pub async fn get_recovery_metrics(&self) -> RecoveryMetrics {
        self.recovery_metrics.read().await.clone()
    }

    /// Record incident response time
    pub async fn record_incident_response(&self, _response_time_seconds: u64) {
        let mut status = self.compliance_status.write().await;
        status.last_incident_response_time = Some(Utc::now());
    }

    /// Mark violation as resolved
    pub async fn resolve_violation(&self, violation_id: &str) -> Result<(), String> {
        let violation_uuid = Uuid::parse_str(violation_id).map_err(|e| format!("Invalid violation ID: {}", e))?;
        let mut violations = self.violations.write().await;

        if let Some(violation) = violations.iter_mut().find(|v| v.id == violation_uuid) {
            violation.resolved = true;
            violation.resolution_time = Some(Utc::now());
            info!("Violation {} marked as resolved", violation_id);
            Ok(())
        } else {
            Err(format!("Violation not found: {}", violation_id))
        }
    }

}

/// Compliance report
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ComplianceReport {
    #[schemars(with = "String")]

    pub generated_at: DateTime<Utc>,
    #[schemars(with = "String")]

    pub period_start: DateTime<Utc>,
    #[schemars(with = "String")]

    pub period_end: DateTime<Utc>,
    pub overall_compliance_percentage: f64,
    pub rto_compliance_percentage: f64,
    pub rpo_compliance_percentage: f64,
    pub total_violations: usize,
    pub critical_violations: usize,
    pub service_breakdown: HashMap<String, ServiceComplianceSummary>,
    pub top_issues: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ServiceComplianceSummary {
    pub service_type: String,
    pub uptime_percentage: f64,
    pub violations_count: usize,
    pub average_rto_seconds: Option<f64>,
}

impl ComplianceAlert {
    pub fn alert_type_as_string(&self) -> &'static str {
        match self.alert_type {
            AlertType::RTOViolation => "RTO Violation",
            AlertType::RPOViolation => "RPO Violation",
            AlertType::ServiceUnavailable => "Service Unavailable",
            AlertType::ComplianceThreshold => "Compliance Threshold",
            AlertType::RecoveryFailure => "Recovery Failure",
            AlertType::ServiceDegradation => "Service Degradation",
            AlertType::SystemFailure => "System Failure",
        }
    }
}

fn service_type_as_string(service_type: ServiceType) -> &'static str {
    match service_type {
        ServiceType::Database => "Database",
        ServiceType::API => "API",
        ServiceType::Worker => "Worker",
        ServiceType::Storage => "Storage",
        ServiceType::Network => "Network",
        ServiceType::ApiServer => "API Server",
        ServiceType::WorkerPool => "Worker Pool",
        ServiceType::MessageQueue => "Message Queue",
        ServiceType::Cache => "Cache",
        ServiceType::FileStorage => "File Storage",
        ServiceType::ExternalApi => "External API",
    }
}

#[allow(dead_code)]
fn calculate_compliance_percentage(violations: &[ComplianceViolation], violation_type: ViolationType) -> f64 {
    let relevant_violations = violations.iter()
        .filter(|v| v.violation_type == violation_type)
        .count();

    if relevant_violations == 0 {
        100.0
    } else {
        // TODO: Implement sophisticated compliance score calculation
        //       Currently uses basic calculation; should implement comprehensive compliance scoring with proper weighting.
        (100.0 - (relevant_violations as f64 * 10.0)).max(0.0)
    }
}

#[allow(dead_code)]
fn generate_service_breakdown(service_status: &HashMap<String, ServiceComplianceStatus>) -> HashMap<String, ServiceComplianceSummary> {
    service_status.iter().map(|(service_type, status)| {
        (service_type.clone(), ServiceComplianceSummary {
            service_type: service_type.clone(),
            // TODO: Calculate actual uptime percentage
            // - [ ] Query historical service status data
            // - [ ] Calculate uptime from service availability records
            // - [ ] Handle missing data gracefully
            // - [ ] Add unit tests with mock service data
            // - [ ] Add integration tests with real service data
            uptime_percentage: 99.9, // Placeholder - would calculate from actual data
            violations_count: 0, // Not available in current struct
            average_rto_seconds: Some(status.current_rto_seconds as f64),
        })
    }).collect()
}

#[allow(dead_code)]
fn identify_top_issues(violations: &[ComplianceViolation]) -> Vec<String> {
    let mut issue_counts = HashMap::new();

    for violation in violations {
        let issue_key = format!("{:?}: {}", violation.violation_type, violation.description);
        *issue_counts.entry(issue_key).or_insert(0) += 1;
    }

    let mut issues: Vec<_> = issue_counts.into_iter().collect();
    issues.sort_by(|a, b| b.1.cmp(&a.1));

    issues.into_iter().take(5).map(|(issue, count)| format!("{} ({} occurrences)", issue, count)).collect()
}

#[allow(dead_code)]
fn generate_recommendations(status: &ComplianceStatus, violations: &[ComplianceViolation]) -> Vec<String> {
    let mut recommendations = Vec::new();

    if !status.rto_compliant {
        recommendations.push("Review and optimize recovery procedures to meet RTO objectives".to_string());
        recommendations.push("Consider implementing faster backup restoration methods".to_string());
    }

    if !status.rpo_compliant {
        recommendations.push("Increase backup frequency to meet RPO objectives".to_string());
        recommendations.push("Implement continuous data replication for critical systems".to_string());
    }

    if violations.iter().any(|v| v.violation_type == ViolationType::ServiceUnavailable) {
        recommendations.push("Improve service health monitoring and auto-healing capabilities".to_string());
        recommendations.push("Review failover procedures for faster service restoration".to_string());
    }

    if recommendations.is_empty() {
        recommendations.push("Continue monitoring - all objectives currently being met".to_string());
    }

    recommendations
}

/// Compliance report for monitoring dashboards
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MonitoringComplianceReport {
    #[schemars(with = "String")]

    pub timestamp: DateTime<Utc>,
    pub overall_compliance_percentage: f64,
    pub rto_compliance_percentage: f64,
    pub rpo_compliance_percentage: f64,
    pub total_violations: usize,
    pub critical_violations: usize,
    pub average_recovery_time_seconds: f64,
    pub max_recovery_time_seconds: f64,
    pub data_loss_incidents: usize,
    pub service_availability_percentage: f64,
}

impl RtoRpoMonitor {
    /// Internal method to get recovery metrics
    async fn internal_get_recovery_metrics(&self) -> RecoveryMetrics {
        // TODO: Implement actual recovery metrics collection
        //       Currently returns default metrics; should collect actual recovery metrics from system state.
        RecoveryMetrics {
            total_incidents: 0,
            average_rto_seconds: 0,
            average_rpo_seconds: 0,
            compliance_rate_percent: 100.0,
            last_month_stats: MonthlyStats {
                period_start: Utc::now() - chrono::Duration::days(30),
                incidents: 0,
                violations: 0,
                average_recovery_time_seconds: 0,
                compliance_percentage: 100.0,
            },
        }
    }

    /// Generate compliance report for monitoring
    pub async fn generate_compliance_report(&self) -> MonitoringComplianceReport {
        let status = self.get_compliance_status().await;
        let violations = self.get_recent_violations(24).await;
        let metrics = self.get_recovery_metrics().await;

        MonitoringComplianceReport {
            timestamp: Utc::now(),
            overall_compliance_percentage: if status.overall_compliant { 100.0 } else { 95.0 }, // TODO: Calculate actual compliance percentage
            rto_compliance_percentage: if status.rto_compliant { 100.0 } else { 90.0 },
            rpo_compliance_percentage: if status.rpo_compliant { 100.0 } else { 85.0 },
            total_violations: violations.len(),
            critical_violations: violations.iter().filter(|v| v.severity == ViolationSeverity::Critical).count(),
            average_recovery_time_seconds: metrics.average_rto_seconds as f64,
            max_recovery_time_seconds: metrics.average_rto_seconds as f64 * 2.0, // TODO: Calculate actual max recovery time
            data_loss_incidents: metrics.total_incidents as usize,
            service_availability_percentage: 99.9, // TODO: Calculate actual service availability from metrics
        }
    }
}

#[async_trait::async_trait]
impl ReliabilityMonitor for RtoRpoMonitor {
    async fn get_compliance_status(&self) -> Result<ComplianceStatus, Box<dyn std::error::Error + Send + Sync>> {
        let status = self.get_compliance_status().await;
        Ok(ComplianceStatus {
            timestamp: status.timestamp,
            overall_compliant: status.overall_compliant,
            rto_compliant: status.rto_compliant,
            rpo_compliant: status.rpo_compliant,
            service_status: status.service_status.into_iter().map(|(k, v)| (k, crate::api_alerts::ServiceComplianceStatus {
                service_name: v.service_name,
                current_rto_seconds: v.current_rto_seconds,
                current_rpo_seconds: v.current_rpo_seconds,
                last_recovery_time: v.last_recovery_time,
                compliance_percentage: v.compliance_percentage,
            })).collect(),
            violations: status.violations.into_iter().map(|v| ComplianceViolation {
                id: v.id,
                timestamp: v.timestamp,
                violation_type: match v.violation_type {
                    ViolationType::RTOExceeded => crate::api_alerts::ViolationType::RTOExceeded,
                    ViolationType::RPOExceeded => crate::api_alerts::ViolationType::RPOExceeded,
                    ViolationType::ServiceUnavailable => crate::api_alerts::ViolationType::ServiceUnavailable,
                },
                severity: v.severity,
                description: v.description,
                service_type: v.service_type,
                // TODO: Map actual measured and objective values from violations
                // - [ ] Extract measured value from violation data structure
                // - [ ] Extract objective value from service objectives
                // - [ ] Handle missing data gracefully
                // - [ ] Add unit tests with mock violation data
                // - [ ] Add integration tests with real violation data
                measured_value: 0.0, // Placeholder - would need to map from internal violation
                objective_value: 0.0, // Placeholder - would need to map from internal violation
                resolved: v.resolved,
                resolution_time: v.resolution_time,
            }).collect(),
            last_incident_response_time: status.last_incident_response_time,
        })
    }

    async fn get_recent_violations(&self, hours: i64) -> Result<Vec<ComplianceViolation>, Box<dyn std::error::Error + Send + Sync>> {
        let violations = self.violations.read().await;
        let cutoff = Utc::now() - chrono::Duration::hours(hours);
        Ok(violations.iter().filter(|v| v.timestamp > cutoff).cloned().collect())
    }

    async fn get_recovery_metrics(&self) -> Result<RecoveryMetrics, Box<dyn std::error::Error + Send + Sync>> {
        let metrics = self.internal_get_recovery_metrics().await;
        Ok(RecoveryMetrics {
            total_incidents: metrics.total_incidents,
            average_rto_seconds: metrics.average_rto_seconds,
            average_rpo_seconds: metrics.average_rpo_seconds,
            compliance_rate_percent: metrics.compliance_rate_percent,
            last_month_stats: crate::api_alerts::MonthlyStats {
                period_start: metrics.last_month_stats.period_start,
                incidents: metrics.last_month_stats.incidents,
                violations: metrics.last_month_stats.violations,
                average_recovery_time_seconds: metrics.last_month_stats.average_recovery_time_seconds,
                compliance_percentage: metrics.last_month_stats.compliance_percentage,
            },
        })
    }

    async fn get_pending_alerts(&self) -> Result<Vec<ComplianceAlert>, Box<dyn std::error::Error + Send + Sync>> {
        // Generate alerts based on compliance status
        let status = self.get_compliance_status().await;
        let mut alerts = Vec::new();

        if !status.rto_compliant {
            alerts.push(ComplianceAlert {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                alert_type: crate::api_alerts::AlertType::RTOViolation,
                severity: AlertSeverity::High,
                message: "RTO compliance violated - recovery time exceeded objectives".to_string(),
                affected_services: vec![crate::api_alerts::ServiceType::Database],
                recommended_actions: vec!["Review recovery procedures".to_string()],
            });
        }

        if !status.rpo_compliant {
            alerts.push(ComplianceAlert {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                alert_type: crate::api_alerts::AlertType::RPOViolation,
                severity: AlertSeverity::Critical,
                message: "RPO compliance violated - data loss exceeded acceptable limits".to_string(),
                affected_services: vec![crate::api_alerts::ServiceType::Database],
                recommended_actions: vec!["Increase backup frequency".to_string()],
            });
        }

        Ok(alerts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_rto_rpo_monitor_creation() {
        let objectives = RecoveryObjectives {
            rto_seconds: 300,
            rpo_seconds: 60,
            service_objectives: HashMap::new(),
        };

        let alert_config = AlertConfig {
            rto_violation_threshold_seconds: 300,
            rpo_violation_threshold_seconds: 60,
            max_violations_before_alert: 5,
            alert_cooldown_minutes: 15,
            email_alerts_enabled: true,
            slack_alerts_enabled: true,
            pager_duty_integration: false,
        };

        let failover_manager = Arc::new(ServiceFailoverManager::new(crate::service_failover::FailoverConfig::default()));
        let monitor = RtoRpoMonitor::new(objectives, alert_config, failover_manager);

        let status = monitor.get_compliance_status().await;
        assert!(status.overall_compliant);
    }

    #[tokio::test]
    async fn test_compliance_report_generation() {
        let objectives = RecoveryObjectives {
            rto_seconds: 300,
            rpo_seconds: 60,
            service_objectives: HashMap::new(),
        };

        let alert_config = AlertConfig::default();
        let failover_manager = Arc::new(ServiceFailoverManager::new(crate::service_failover::FailoverConfig::default()));
        let monitor = RtoRpoMonitor::new(objectives, alert_config, failover_manager);

        let report = monitor.generate_compliance_report().await;
        assert!(report.overall_compliance_percentage >= 0.0);
        assert!(report.overall_compliance_percentage <= 100.0);
    }
}
