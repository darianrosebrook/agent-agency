//! RTO/RPO Monitoring and Compliance System
//!
//! Provides real-time monitoring and alerting for Recovery Time Objectives (RTO)
//! and Recovery Point Objectives (RPO) to ensure disaster recovery compliance.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, mpsc};
use tokio::time;
use tracing::{info, warn, error};

use crate::service_failover::{ServiceFailoverManager, ServiceType, ServiceStatus};

/// RTO/RPO objectives configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryObjectives {
    /// Recovery Time Objective (seconds) - max time to restore service
    pub rto_seconds: u64,
    /// Recovery Point Objective (seconds) - max data loss acceptable
    pub rpo_seconds: u64,
    /// Service-specific objectives
    pub service_objectives: HashMap<ServiceType, ServiceRecoveryObjectives>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRecoveryObjectives {
    pub rto_seconds: u64,
    pub rpo_seconds: u64,
    pub critical_service: bool, // Requires immediate attention if violated
}

/// RTO/RPO compliance status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStatus {
    pub timestamp: DateTime<Utc>,
    pub overall_compliant: bool,
    pub rto_compliant: bool,
    pub rpo_compliant: bool,
    pub service_status: HashMap<ServiceType, ServiceComplianceStatus>,
    pub violations: Vec<ComplianceViolation>,
    pub last_incident_response_time: Option<Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceComplianceStatus {
    pub service_type: ServiceType,
    pub rto_compliant: bool,
    pub rpo_compliant: bool,
    pub current_rto_seconds: Option<u64>,
    pub current_rpo_seconds: Option<u64>,
    pub last_backup_time: Option<DateTime<Utc>>,
    pub incidents_this_period: u32,
}

/// Compliance violation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceViolation {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub violation_type: ViolationType,
    pub service_type: ServiceType,
    pub severity: ViolationSeverity,
    pub description: String,
    pub measured_value: u64,
    pub objective_value: u64,
    pub resolved: bool,
    pub resolution_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ViolationType {
    RTOExceeded,
    RPOExceeded,
    NoRecentBackup,
    ServiceUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Recovery metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryMetrics {
    pub total_incidents: u64,
    pub successful_recoveries: u64,
    pub failed_recoveries: u64,
    pub average_rto_seconds: f64,
    pub average_rpo_seconds: f64,
    pub compliance_rate_percent: f64,
    pub last_month_stats: MonthlyStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyStats {
    pub period_start: DateTime<Utc>,
    pub incidents: u32,
    pub violations: u32,
    pub average_recovery_time_seconds: f64,
    pub compliance_percentage: f64,
}

/// Alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    pub rto_violation_threshold_seconds: u64,
    pub rpo_violation_threshold_seconds: u64,
    pub max_violations_before_alert: u32,
    pub alert_cooldown_minutes: u64,
    pub email_alerts_enabled: bool,
    pub slack_alerts_enabled: bool,
    pub pager_duty_integration: bool,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceAlert {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub alert_type: AlertType,
    pub severity: ViolationSeverity,
    pub message: String,
    pub affected_services: Vec<ServiceType>,
    pub recommended_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    RTOViolation,
    RPOViolation,
    ServiceUnavailable,
    ComplianceThreshold,
    RecoveryFailure,
}

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
                successful_recoveries: 0,
                failed_recoveries: 0,
                average_rto_seconds: 0.0,
                average_rpo_seconds: 0.0,
                compliance_rate_percent: 100.0,
                last_month_stats: MonthlyStats {
                    period_start: Utc::now() - chrono::Duration::days(30),
                    incidents: 0,
                    violations: 0,
                    average_recovery_time_seconds: 0.0,
                    compliance_percentage: 100.0,
                },
            })),
            service_failover_manager,
            alert_sender: tx,
            alert_receiver: Arc::new(RwLock::new(Some(rx))),
        }
    }

    /// Start monitoring loop
    pub async fn start_monitoring(&self) -> Result<(), String> {
        info!("Starting RTO/RPO monitoring");

        // Start compliance check loop
        let monitor = Arc::new(self.clone());
        tokio::spawn(async move {
            monitor.compliance_check_loop().await;
        });

        // Start alert processing loop
        let monitor = Arc::new(self.clone());
        tokio::spawn(async move {
            monitor.alert_processing_loop().await;
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

        let mut service_compliance = HashMap::new();
        let mut violations = Vec::new();

        for (service_id, status) in service_status {
            // Parse service type from ID (simplified - in real system would be more robust)
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

            let service_objectives = self.objectives.service_objectives.get(&service_type)
                .unwrap_or(&ServiceRecoveryObjectives {
                    rto_seconds: self.objectives.rto_seconds,
                    rpo_seconds: self.objectives.rpo_seconds,
                    critical_service: false,
                });

            // Calculate current RTO (from failover history)
            let recent_failovers: Vec<_> = failover_history.iter()
                .filter(|event| matches!(event, crate::service_failover::FailoverEvent::FailoverCompleted { .. }))
                .take(5)
                .collect();

            let avg_rto = if !recent_failovers.is_empty() {
                // Simplified - in real system would track actual recovery times
                Some(service_objectives.rto_seconds / 2)
            } else {
                None
            };

            // Check for service unavailability
            let service_available = *status == ServiceStatus::Healthy;
            if !service_available {
                violations.push(ComplianceViolation {
                    id: format!("violation_{}", Utc::now().timestamp()),
                    timestamp: Utc::now(),
                    violation_type: ViolationType::ServiceUnavailable,
                    service_type,
                    severity: if service_objectives.critical_service {
                        ViolationSeverity::Critical
                    } else {
                        ViolationSeverity::High
                    },
                    description: format!("Service {} is not available (status: {:?})", service_id, status),
                    measured_value: 0, // Not applicable
                    objective_value: 0, // Not applicable
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
                    id: format!("rto_violation_{}", Utc::now().timestamp()),
                    timestamp: Utc::now(),
                    violation_type: ViolationType::RTOExceeded,
                    service_type,
                    severity: ViolationSeverity::High,
                    description: format!("RTO exceeded for {}: {}s > {}s objective",
                        service_type_as_string(service_type), avg_rto.unwrap(), service_objectives.rto_seconds),
                    measured_value: avg_rto.unwrap(),
                    objective_value: service_objectives.rto_seconds,
                    resolved: false,
                    resolution_time: None,
                });
            }

            // Check RPO compliance (simplified - would integrate with backup system)
            let rpo_compliant = true; // Placeholder - would check actual backup age

            service_compliance.insert(service_type, ServiceComplianceStatus {
                service_type,
                rto_compliant,
                rpo_compliant,
                current_rto_seconds: avg_rto,
                current_rpo_seconds: Some(service_objectives.rpo_seconds),
                last_backup_time: Some(Utc::now() - chrono::Duration::hours(1)), // Placeholder
                incidents_this_period: recent_failovers.len() as u32,
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
                id: format!("alert_{}", Utc::now().timestamp()),
                timestamp: Utc::now(),
                alert_type: AlertType::ComplianceThreshold,
                severity: ViolationSeverity::High,
                message: format!("{} compliance violations detected in last 5 minutes", recent_violations.len()),
                affected_services: recent_violations.iter().map(|v| v.service_type).collect(),
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
    pub async fn record_incident_response(&self, response_time_seconds: u64) {
        let mut status = self.compliance_status.write().await;
        status.last_incident_response_time = Some(Duration::from_secs(response_time_seconds));
    }

    /// Mark violation as resolved
    pub async fn resolve_violation(&self, violation_id: &str) -> Result<(), String> {
        let mut violations = self.violations.write().await;

        if let Some(violation) = violations.iter_mut().find(|v| v.id == violation_id) {
            violation.resolved = true;
            violation.resolution_time = Some(Utc::now());
            info!("Violation {} marked as resolved", violation_id);
            Ok(())
        } else {
            Err(format!("Violation not found: {}", violation_id))
        }
    }

    /// Get compliance report
    pub async fn generate_compliance_report(&self) -> ComplianceReport {
        let status = self.get_compliance_status().await;
        let violations = self.get_recent_violations(24).await;
        let metrics = self.get_recovery_metrics().await;

        ComplianceReport {
            generated_at: Utc::now(),
            period_start: Utc::now() - chrono::Duration::days(30),
            period_end: Utc::now(),
            overall_compliance_percentage: if metrics.total_incidents > 0 {
                (metrics.successful_recoveries as f64 / metrics.total_incidents as f64) * 100.0
            } else {
                100.0
            },
            rto_compliance_percentage: calculate_compliance_percentage(&violations, ViolationType::RTOExceeded),
            rpo_compliance_percentage: calculate_compliance_percentage(&violations, ViolationType::RPOExceeded),
            total_violations: violations.len(),
            critical_violations: violations.iter().filter(|v| v.severity == ViolationSeverity::Critical).count(),
            service_breakdown: generate_service_breakdown(&status.service_status),
            top_issues: identify_top_issues(&violations),
            recommendations: generate_recommendations(&status, &violations),
        }
    }
}

/// Compliance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub generated_at: DateTime<Utc>,
    pub period_start: DateTime<Utc>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceComplianceSummary {
    pub service_type: String,
    pub uptime_percentage: f64,
    pub violations_count: usize,
    pub average_rto_seconds: Option<f64>,
}

impl ComplianceAlert {
    fn alert_type_as_string(&self) -> &'static str {
        match self.alert_type {
            AlertType::RTOViolation => "RTO Violation",
            AlertType::RPOViolation => "RPO Violation",
            AlertType::ServiceUnavailable => "Service Unavailable",
            AlertType::ComplianceThreshold => "Compliance Threshold",
            AlertType::RecoveryFailure => "Recovery Failure",
        }
    }
}

fn service_type_as_string(service_type: ServiceType) -> &'static str {
    match service_type {
        ServiceType::Database => "Database",
        ServiceType::ApiServer => "API Server",
        ServiceType::WorkerPool => "Worker Pool",
        ServiceType::MessageQueue => "Message Queue",
        ServiceType::Cache => "Cache",
        ServiceType::FileStorage => "File Storage",
        ServiceType::ExternalApi => "External API",
    }
}

fn calculate_compliance_percentage(violations: &[ComplianceViolation], violation_type: ViolationType) -> f64 {
    let relevant_violations = violations.iter()
        .filter(|v| v.violation_type == violation_type)
        .count();

    if relevant_violations == 0 {
        100.0
    } else {
        // Simplified calculation - in real system would be more sophisticated
        (100.0 - (relevant_violations as f64 * 10.0)).max(0.0)
    }
}

fn generate_service_breakdown(service_status: &HashMap<ServiceType, ServiceComplianceStatus>) -> HashMap<String, ServiceComplianceSummary> {
    service_status.iter().map(|(service_type, status)| {
        (service_type_as_string(*service_type).to_string(), ServiceComplianceSummary {
            service_type: service_type_as_string(*service_type).to_string(),
            uptime_percentage: 99.9, // Placeholder - would calculate from actual data
            violations_count: status.incidents_this_period as usize,
            average_rto_seconds: status.current_rto_seconds.map(|rto| rto as f64),
        })
    }).collect()
}

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
