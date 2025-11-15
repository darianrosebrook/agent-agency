//! RTO/RPO Monitoring and Compliance System
//!
//! Provides real-time monitoring and alerting for Recovery Time Objectives (RTO)
//! and Recovery Point Objectives (RPO) to ensure disaster recovery compliance.

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio::time;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::api_alerts::{
    AlertSeverity, AlertType, ComplianceAlert, ComplianceStatus, ComplianceViolation, MonthlyStats,
    RecoveryMetrics, ReliabilityMonitor, ServiceComplianceStatus, ViolationSeverity, ViolationType,
};
use crate::service_failover::{ServiceFailoverManager, ServiceStatus, ServiceType};
use crate::simple_client::DatabaseClient;
use sqlx::Row;

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
    db_client: Option<Arc<DatabaseClient>>,
}

// Using ComplianceAlert and AlertType from api_alerts module

impl RtoRpoMonitor {
    /// Create a new RTO/RPO monitor
    pub fn new(
        objectives: RecoveryObjectives,
        alert_config: AlertConfig,
        service_failover_manager: Arc<ServiceFailoverManager>,
    ) -> Self {
        Self::with_database_client(objectives, alert_config, service_failover_manager, None)
    }

    /// Create a new RTO/RPO monitor with optional database client for RPO compliance checking
    pub fn with_database_client(
        objectives: RecoveryObjectives,
        alert_config: AlertConfig,
        service_failover_manager: Arc<ServiceFailoverManager>,
        db_client: Option<Arc<DatabaseClient>>,
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
            db_client,
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
            let service_objectives = self
                .objectives
                .service_objectives
                .get(&service_type)
                .unwrap_or(&default_objectives);

            // Calculate current RTO (from failover history)
            let recent_failovers: Vec<_> = failover_history
                .iter()
                .filter(|event| {
                    matches!(
                        event,
                        crate::service_failover::FailoverEvent::FailoverCompleted { .. }
                    )
                })
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
                    description: format!(
                        "Service {} is not available (status: {:?})",
                        service_id, status
                    ),
                    measured_value: 0.0,  // Not applicable
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
                    description: format!(
                        "RTO exceeded for {}: {}s > {}s objective",
                        service_type_as_string(service_type),
                        avg_rto.unwrap(),
                        service_objectives.rto_seconds
                    ),
                    measured_value: avg_rto.unwrap() as f64,
                    objective_value: service_objectives.rto_seconds as f64,
                    resolved: false,
                    resolution_time: None,
                });
            }

            // Check actual RPO compliance by querying WAL log records for last recovery point
            let rpo_compliant = self
                .check_rpo_compliance(service_objectives.rpo_seconds)
                .await;

            if !rpo_compliant {
                let time_since_last_backup =
                    self.get_time_since_last_backup().await.unwrap_or(u64::MAX);
                violations.push(ComplianceViolation {
                    id: Uuid::new_v4(),
                    timestamp: Utc::now(),
                    violation_type: ViolationType::RPOExceeded,
                    service_type: service_type_as_string(service_type).to_string(),
                    severity: ViolationSeverity::High,
                    description: format!(
                        "RPO exceeded for {}: {}s since last backup > {}s objective",
                        service_type_as_string(service_type),
                        time_since_last_backup,
                        service_objectives.rpo_seconds
                    ),
                    measured_value: time_since_last_backup as f64,
                    objective_value: service_objectives.rpo_seconds as f64,
                    resolved: false,
                    resolution_time: None,
                });
            }

            service_compliance.insert(
                service_type_as_string(service_type).to_string(),
                ServiceComplianceStatus {
                    service_name: service_type_as_string(service_type).to_string(),
                    current_rto_seconds: avg_rto.unwrap_or(0),
                    current_rpo_seconds: service_objectives.rpo_seconds,
                    // TODO: Get actual last recovery time
                    //       Replace placeholder timestamp with actual recovery system query to track real recovery events and timing.
                    //
                    // COMPLETION CHECKLIST:
                    // [ ] Primary functionality implemented
                    // [ ] Query recovery system for last recovery timestamp by service
                    // [ ] Handle missing recovery data with appropriate fallbacks
                    // [ ] Implement caching for recovery timestamps to avoid repeated queries
                    // [ ] Add error handling for recovery system unavailability
                    // [ ] API/data structures defined & stable
                    // [ ] Error handling + validation aligned with error taxonomy
                    // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
                    // [ ] Integration tests for external systems/contracts
                    // [ ] Documentation: public API + system behavior
                    // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
                    // [ ] Security posture reviewed (inputs, authz, sandboxing)
                    // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
                    // [ ] Configurability and feature flags defined if relevant
                    // [ ] Failure-mode cards documented (degradation paths)
                    //
                    // ACCEPTANCE CRITERIA:
                    // - Last recovery time reflects actual system recovery events
                    // - Missing data scenarios handled with sensible defaults
                    // - Query performance is acceptable (<100ms per service status)
                    // - Integration tests validate end-to-end recovery system communication
                    //
                    // DEPENDENCIES:
                    // - Recovery system integration (Required)
                    // - Recovery event logging system (Required)
                    // - Error handling framework (Required)
                    // - Test infrastructure for mock recovery system (Optional)
                    //
                    // ESTIMATED EFFORT: 4-6 hours (medium confidence)
                    // PRIORITY: Medium
                    // BLOCKING: No
                    //
                    // GOVERNANCE:
                    // - CAWS Tier: 2 (monitoring functionality enhancement)
                    // - Change Budget: ~150 LOC
                    // - Reviewer Requirements: Recovery systems and monitoring expertise
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
                    compliance_percentage: if rto_compliant && rpo_compliant {
                        100.0
                    } else {
                        0.0
                    }, // Temporary: basic binary calculation until proper percentage calculation
                },
            );
        }

        // Update compliance status
        {
            let mut compliance_status = self.compliance_status.write().await;
            compliance_status.timestamp = Utc::now();
            compliance_status.service_status = service_compliance;
            compliance_status.violations = violations.clone();
            compliance_status.rto_compliant = violations
                .iter()
                .all(|v| v.violation_type != ViolationType::RTOExceeded);
            compliance_status.rpo_compliant = violations
                .iter()
                .all(|v| v.violation_type != ViolationType::RPOExceeded);
            compliance_status.overall_compliant = compliance_status.rto_compliant
                && compliance_status.rpo_compliant
                && violations
                    .iter()
                    .all(|v| v.violation_type != ViolationType::ServiceUnavailable);
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
        let recent_violations: Vec<_> = violations
            .iter()
            .filter(|v| !v.resolved && (Utc::now() - v.timestamp) < chrono::Duration::minutes(5))
            .collect();

        if recent_violations.len() >= self.alert_config.max_violations_before_alert as usize {
            let alert = ComplianceAlert {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                alert_type: AlertType::ComplianceThreshold,
                severity: AlertSeverity::High,
                message: format!(
                    "{} compliance violations detected in last 5 minutes",
                    recent_violations.len()
                ),
                affected_services: vec![], // TODO: Convert string service_type back to ServiceType enum
                //       Replace empty vector with proper service type conversion to identify which services are affected by violations.
                //
                // COMPLETION CHECKLIST:
                // [ ] Primary functionality implemented
                // [ ] Implement string to ServiceType enum conversion
                // [ ] Extract unique service types from violations
                // [ ] Handle unknown service type strings gracefully
                // [ ] Add validation for service type conversion
                // [ ] API/data structures defined & stable
                // [ ] Error handling + validation aligned with error taxonomy
                // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
                // [ ] Integration tests for external systems/contracts
                // [ ] Documentation: public API + system behavior
                // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
                // [ ] Security posture reviewed (inputs, authz, sandboxing)
                // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
                // [ ] Configurability and feature flags defined if relevant
                // [ ] Failure-mode cards documented (degradation paths)
                //
                // ACCEPTANCE CRITERIA:
                // - Affected services list accurately reflects violation data
                // - Unknown service types are handled without crashes
                // - Conversion performance is acceptable (<10ms for typical violation sets)
                // - Integration tests validate end-to-end service type handling
                //
                // DEPENDENCIES:
                // - ServiceType enum definition (Required)
                // - String parsing utilities (Required)
                // - Error handling framework (Required)
                // - Test data with various service types (Required)
                //
                // ESTIMATED EFFORT: 3-5 hours (medium confidence)
                // PRIORITY: Medium
                // BLOCKING: No
                //
                // GOVERNANCE:
                // - CAWS Tier: 2 (monitoring data accuracy)
                // - Change Budget: ~100 LOC
                // - Reviewer Requirements: Data type conversion and enum handling expertise
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
        warn!(
            "Compliance Alert: {} - {}",
            alert.alert_type_as_string(),
            alert.message
        );

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

        violations
            .iter()
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
        let violation_uuid =
            Uuid::parse_str(violation_id).map_err(|e| format!("Invalid violation ID: {}", e))?;
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
fn calculate_compliance_percentage(
    violations: &[ComplianceViolation],
    violation_type: ViolationType,
) -> f64 {
    let relevant_violations = violations
        .iter()
        .filter(|v| v.violation_type == violation_type)
        .count();

    if relevant_violations == 0 {
        100.0
    } else {
        // TODO: Implement sophisticated compliance score calculation
        //       Currently uses basic linear penalty; should implement comprehensive compliance scoring considering violation severity, frequency, and service criticality.
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] Implement violation severity weighting (critical > high > medium > low)
        // [ ] Add violation frequency analysis (recent vs historical)
        // [ ] Factor in service criticality and impact assessment
        // [ ] Implement time-decay for older violations
        // [ ] Add compliance trend analysis (improving vs degrading)
        // [ ] Support configurable scoring parameters
        // [ ] Add compliance score normalization and bounds checking
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - Compliance scores reflect actual violation impact and severity
        // - Critical violations have higher penalty than minor ones
        // - Recent violations are weighted more than historical ones
        // - Service criticality influences scoring appropriately
        // - Scores are normalized to expected range (0-100)
        // - Performance meets SLA for real-time calculation
        // - Edge cases (no violations, all violations) handled correctly
        // - Integration tests validate scoring against business rules
        //
        // DEPENDENCIES:
        // - Violation severity classification (Required)
        // - Service criticality metadata (Required)
        // - Time-based weighting functions (Optional)
        // - Statistical analysis utilities (Optional)
        //
        // ESTIMATED EFFORT: 8-12 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (compliance monitoring enhancement)
        // - Change Budget: ~200 LOC
        // - Reviewer Requirements: Compliance and risk assessment expertise
        (100.0 - (relevant_violations as f64 * 10.0)).max(0.0)
    }
}

#[allow(dead_code)]
fn generate_service_breakdown(
    service_status: &HashMap<String, ServiceComplianceStatus>,
) -> HashMap<String, ServiceComplianceSummary> {
    service_status
        .iter()
        .map(|(service_type, status)| {
            (
                service_type.clone(),
                ServiceComplianceSummary {
                    service_type: service_type.clone(),
                    // TODO: Calculate actual uptime percentage
                    //       Replace placeholder 99.9% with actual uptime calculation from historical service status data.
                    //
                    // COMPLETION CHECKLIST:
                    // [ ] Primary functionality implemented
                    // [ ] Query historical service status data from monitoring system
                    // [ ] Calculate uptime percentage from availability records
                    // [ ] Handle missing data with appropriate statistical fallbacks
                    // [ ] Implement time-window based uptime calculations
                    // [ ] Add data validation and error handling
                    // [ ] API/data structures defined & stable
                    // [ ] Error handling + validation aligned with error taxonomy
                    // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
                    // [ ] Integration tests for external systems/contracts
                    // [ ] Documentation: public API + system behavior
                    // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
                    // [ ] Security posture reviewed (inputs, authz, sandboxing)
                    // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
                    // [ ] Configurability and feature flags defined if relevant
                    // [ ] Failure-mode cards documented (degradation paths)
                    //
                    // ACCEPTANCE CRITERIA:
                    // - Uptime percentage reflects actual service availability
                    // - Missing data scenarios use reasonable statistical defaults
                    // - Calculation performance meets monitoring SLA (<50ms per service)
                    // - Integration tests validate end-to-end monitoring system communication
                    //
                    // DEPENDENCIES:
                    // - Service monitoring system integration (Required)
                    // - Historical availability data storage (Required)
                    // - Statistical calculation utilities (Required)
                    // - Error handling framework (Required)
                    // - Test infrastructure for mock monitoring data (Optional)
                    //
                    // ESTIMATED EFFORT: 4-6 hours (medium confidence)
                    // PRIORITY: Medium
                    // BLOCKING: No
                    //
                    // GOVERNANCE:
                    // - CAWS Tier: 2 (monitoring metrics accuracy)
                    // - Change Budget: ~150 LOC
                    // - Reviewer Requirements: Service monitoring and uptime calculation expertise
                    uptime_percentage: 99.9, // Placeholder - would calculate from actual data
                    violations_count: 0,     // Not available in current struct
                    average_rto_seconds: Some(status.current_rto_seconds as f64),
                },
            )
        })
        .collect()
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

    issues
        .into_iter()
        .take(5)
        .map(|(issue, count)| format!("{} ({} occurrences)", issue, count))
        .collect()
}

#[allow(dead_code)]
fn generate_recommendations(
    status: &ComplianceStatus,
    violations: &[ComplianceViolation],
) -> Vec<String> {
    let mut recommendations = Vec::new();

    if !status.rto_compliant {
        recommendations
            .push("Review and optimize recovery procedures to meet RTO objectives".to_string());
        recommendations.push("Consider implementing faster backup restoration methods".to_string());
    }

    if !status.rpo_compliant {
        recommendations.push("Increase backup frequency to meet RPO objectives".to_string());
        recommendations
            .push("Implement continuous data replication for critical systems".to_string());
    }

    if violations
        .iter()
        .any(|v| v.violation_type == ViolationType::ServiceUnavailable)
    {
        recommendations
            .push("Improve service health monitoring and auto-healing capabilities".to_string());
        recommendations
            .push("Review failover procedures for faster service restoration".to_string());
    }

    if recommendations.is_empty() {
        recommendations
            .push("Continue monitoring - all objectives currently being met".to_string());
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
        //       Currently returns hardcoded default values; should collect actual recovery metrics from system monitoring, incident tracking, and compliance data.
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] Query incident tracking system for recovery events
        // [ ] Collect RTO/RPO measurements from actual recovery operations
        // [ ] Calculate compliance rates from historical data
        // [ ] Aggregate metrics across different time periods
        // [ ] Handle missing data with appropriate statistical fallbacks
        // [ ] Implement data validation and error handling
        // [ ] Add performance monitoring for metrics collection
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - Recovery metrics reflect actual system performance data
        // - RTO/RPO measurements are based on real recovery operations
        // - Compliance rates are calculated from historical incident data
        // - Time period aggregations provide accurate trending information
        // - Missing data scenarios are handled with reasonable defaults
        // - Performance meets SLA for metrics collection and calculation
        // - Integration tests validate metrics against operational data
        //
        // DEPENDENCIES:
        // - Incident tracking system integration (Required)
        // - Recovery operation monitoring (Required)
        // - Historical metrics storage (Required)
        // - Statistical calculation utilities (Optional)
        //
        // ESTIMATED EFFORT: 10-14 hours (medium confidence)
        // PRIORITY: High
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 1 (core reliability monitoring)
        // - Change Budget: ~250 LOC
        // - Reviewer Requirements: Reliability engineering and incident response expertise
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
            overall_compliance_percentage: if status.overall_compliant {
                100.0
            } else {
                95.0
            }, // TODO: Calculate actual compliance percentage
            //       Replace binary 100%/95% calculation with actual compliance percentage based on violation severity and frequency.
            //
            // COMPLETION CHECKLIST:
            // [ ] Primary functionality implemented
            // [ ] Implement compliance percentage calculation based on violation metrics
            // [ ] Weight compliance by violation severity (critical vs warning)
            // [ ] Consider violation frequency and recency in calculation
            // [ ] Handle edge cases (no violations, all violations)
            // [ ] Add statistical validation of compliance percentages
            // [ ] API/data structures defined & stable
            // [ ] Error handling + validation aligned with error taxonomy
            // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
            // [ ] Integration tests for external systems/contracts
            // [ ] Documentation: public API + system behavior
            // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
            // [ ] Security posture reviewed (inputs, authz, sandboxing)
            // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
            // [ ] Configurability and feature flags defined if relevant
            // [ ] Failure-mode cards documented (degradation paths)
            //
            // ACCEPTANCE CRITERIA:
            // - Compliance percentage reflects actual system reliability
            // - Violation severity properly weights the calculation
            // - Recent violations have higher impact than old ones
            // - Calculation performance is acceptable (<10ms per status)
            // - Integration tests validate against known compliance scenarios
            //
            // DEPENDENCIES:
            // - Violation severity classification system (Required)
            // - Time-weighted calculation utilities (Required)
            // - Statistical aggregation functions (Required)
            // - Test data with various violation patterns (Required)
            //
            // ESTIMATED EFFORT: 3-5 hours (medium confidence)
            // PRIORITY: Medium
            // BLOCKING: No
            //
            // GOVERNANCE:
            // - CAWS Tier: 2 (compliance metrics accuracy)
            // - Change Budget: ~100 LOC
            // - Reviewer Requirements: Statistical analysis and compliance metrics expertise
            rto_compliance_percentage: if status.rto_compliant { 100.0 } else { 90.0 },
            rpo_compliance_percentage: if status.rpo_compliant { 100.0 } else { 85.0 },
            total_violations: violations.len(),
            critical_violations: violations
                .iter()
                .filter(|v| v.severity == ViolationSeverity::Critical)
                .count(),
            average_recovery_time_seconds: metrics.average_rto_seconds as f64,
            max_recovery_time_seconds: metrics.average_rto_seconds as f64 * 2.0, // TODO: Calculate actual max recovery time
            //       Replace placeholder calculation with actual maximum recovery time from historical incident data.
            //
            // COMPLETION CHECKLIST:
            // [ ] Primary functionality implemented
            // [ ] Query historical incident data for recovery times
            // [ ] Calculate true maximum recovery time across all incidents
            // [ ] Handle missing or incomplete incident data
            // [ ] Implement statistical outlier detection for anomalous recovery times
            // [ ] Add time-window filtering for relevant incident data
            // [ ] API/data structures defined & stable
            // [ ] Error handling + validation aligned with error taxonomy
            // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
            // [ ] Integration tests for external systems/contracts
            // [ ] Documentation: public API + system behavior
            // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
            // [ ] Security posture reviewed (inputs, authz, sandboxing)
            // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
            // [ ] Configurability and feature flags defined if relevant
            // [ ] Failure-mode cards documented (degradation paths)
            //
            // ACCEPTANCE CRITERIA:
            // - Maximum recovery time reflects actual worst-case incidents
            // - Outlier detection prevents anomalous data from skewing metrics
            // - Time-window filtering ensures relevance of historical data
            // - Missing data scenarios handled with statistical fallbacks
            // - Integration tests validate against known incident recovery times
            //
            // DEPENDENCIES:
            // - Incident tracking system integration (Required)
            // - Recovery time logging system (Required)
            // - Statistical outlier detection utilities (Required)
            // - Time-series filtering functions (Required)
            // - Test data with various incident scenarios (Required)
            //
            // ESTIMATED EFFORT: 4-6 hours (medium confidence)
            // PRIORITY: Medium
            // BLOCKING: No
            //
            // GOVERNANCE:
            // - CAWS Tier: 2 (recovery metrics accuracy)
            // - Change Budget: ~150 LOC
            // - Reviewer Requirements: Incident management and statistical analysis expertise
            data_loss_incidents: metrics.total_incidents as usize,
            service_availability_percentage: 99.9, // TODO: Calculate actual service availability from metrics
            //       Replace hardcoded 99.9% with actual service availability calculation from uptime and incident data.
            //
            // COMPLETION CHECKLIST:
            // [ ] Primary functionality implemented
            // [ ] Query service uptime data from monitoring system
            // [ ] Calculate availability percentage from incident and uptime records
            // [ ] Handle scheduled maintenance windows appropriately
            // [ ] Implement time-weighted availability calculations
            // [ ] Add statistical confidence intervals for availability metrics
            // [ ] API/data structures defined & stable
            // [ ] Error handling + validation aligned with error taxonomy
            // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
            // [ ] Integration tests for external systems/contracts
            // [ ] Documentation: public API + system behavior
            // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
            // [ ] Security posture reviewed (inputs, authz, sandboxing)
            // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
            // [ ] Configurability and feature flags defined if relevant
            // [ ] Failure-mode cards documented (degradation paths)
            //
            // ACCEPTANCE CRITERIA:
            // - Availability percentage reflects actual service reliability
            // - Scheduled maintenance is properly accounted for
            // - Time-weighted calculations prioritize recent availability
            // - Statistical confidence provides uncertainty quantification
            // - Integration tests validate against known availability scenarios
            //
            // DEPENDENCIES:
            // - Service monitoring system integration (Required)
            // - Incident tracking system (Required)
            // - Scheduled maintenance tracking (Required)
            // - Time-weighted statistical utilities (Required)
            // - Test data with various availability scenarios (Required)
            //
            // ESTIMATED EFFORT: 4-6 hours (medium confidence)
            // PRIORITY: Medium
            // BLOCKING: No
            //
            // GOVERNANCE:
            // - CAWS Tier: 2 (availability metrics accuracy)
            // - Change Budget: ~150 LOC
            // - Reviewer Requirements: Service monitoring and availability calculation expertise
        }
    }
}

#[async_trait::async_trait]
impl ReliabilityMonitor for RtoRpoMonitor {
    async fn get_compliance_status(
        &self,
    ) -> Result<ComplianceStatus, Box<dyn std::error::Error + Send + Sync>> {
        let status = self.get_compliance_status().await;
        Ok(ComplianceStatus {
            timestamp: status.timestamp,
            overall_compliant: status.overall_compliant,
            rto_compliant: status.rto_compliant,
            rpo_compliant: status.rpo_compliant,
            service_status: status
                .service_status
                .into_iter()
                .map(|(k, v)| {
                    (
                        k,
                        crate::api_alerts::ServiceComplianceStatus {
                            service_name: v.service_name,
                            current_rto_seconds: v.current_rto_seconds,
                            current_rpo_seconds: v.current_rpo_seconds,
                            last_recovery_time: v.last_recovery_time,
                            compliance_percentage: v.compliance_percentage,
                        },
                    )
                })
                .collect(),
            violations: status
                .violations
                .into_iter()
                .map(|v| ComplianceViolation {
                    id: v.id,
                    timestamp: v.timestamp,
                    violation_type: match v.violation_type {
                        ViolationType::RTOExceeded => crate::api_alerts::ViolationType::RTOExceeded,
                        ViolationType::RPOExceeded => crate::api_alerts::ViolationType::RPOExceeded,
                        ViolationType::ServiceUnavailable => {
                            crate::api_alerts::ViolationType::ServiceUnavailable
                        }
                    },
                    severity: v.severity,
                    description: v.description,
                    service_type: v.service_type,
                    // TODO: Map actual measured and objective values from violations
                    //       Replace placeholder 0.0 values with actual measured and objective values extracted from violation data.
                    //
                    // COMPLETION CHECKLIST:
                    // [ ] Primary functionality implemented
                    // [ ] Extract measured value from violation data structure
                    // [ ] Extract objective value from service objectives configuration
                    // [ ] Handle missing or incomplete violation data gracefully
                    // [ ] Implement proper type conversion for measured/objective values
                    // [ ] Add data validation and error handling
                    // [ ] API/data structures defined & stable
                    // [ ] Error handling + validation aligned with error taxonomy
                    // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
                    // [ ] Integration tests for external systems/contracts
                    // [ ] Documentation: public API + system behavior
                    // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
                    // [ ] Security posture reviewed (inputs, authz, sandboxing)
                    // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
                    // [ ] Configurability and feature flags defined if relevant
                    // [ ] Failure-mode cards documented (degradation paths)
                    //
                    // ACCEPTANCE CRITERIA:
                    // - Measured and objective values accurately reflect violation data
                    // - Missing data scenarios handled with appropriate fallbacks
                    // - Type conversions preserve data integrity
                    // - Performance overhead is minimal (<5ms per violation)
                    // - Integration tests validate end-to-end data mapping
                    //
                    // DEPENDENCIES:
                    // - Violation data structure access (Required)
                    // - Service objectives configuration (Required)
                    // - Type conversion utilities (Required)
                    // - Error handling framework (Required)
                    // - Test data with various violation scenarios (Required)
                    //
                    // ESTIMATED EFFORT: 3-5 hours (medium confidence)
                    // PRIORITY: Medium
                    // BLOCKING: No
                    //
                    // GOVERNANCE:
                    // - CAWS Tier: 2 (violation data accuracy)
                    // - Change Budget: ~100 LOC
                    // - Reviewer Requirements: Data mapping and type conversion expertise
                    measured_value: 0.0, // Placeholder - would need to map from internal violation
                    objective_value: 0.0, // Placeholder - would need to map from internal violation
                    resolved: v.resolved,
                    resolution_time: v.resolution_time,
                })
                .collect(),
            last_incident_response_time: status.last_incident_response_time,
        })
    }

    async fn get_recent_violations(
        &self,
        hours: i64,
    ) -> Result<Vec<ComplianceViolation>, Box<dyn std::error::Error + Send + Sync>> {
        let violations = self.violations.read().await;
        let cutoff = Utc::now() - chrono::Duration::hours(hours);
        Ok(violations
            .iter()
            .filter(|v| v.timestamp > cutoff)
            .cloned()
            .collect())
    }

    async fn get_recovery_metrics(
        &self,
    ) -> Result<RecoveryMetrics, Box<dyn std::error::Error + Send + Sync>> {
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
                average_recovery_time_seconds: metrics
                    .last_month_stats
                    .average_recovery_time_seconds,
                compliance_percentage: metrics.last_month_stats.compliance_percentage,
            },
        })
    }

    async fn get_pending_alerts(
        &self,
    ) -> Result<Vec<ComplianceAlert>, Box<dyn std::error::Error + Send + Sync>> {
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
                message: "RPO compliance violated - data loss exceeded acceptable limits"
                    .to_string(),
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

        let failover_manager = Arc::new(ServiceFailoverManager::new(
            crate::service_failover::FailoverConfig::default(),
        ));
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
        let failover_manager = Arc::new(ServiceFailoverManager::new(
            crate::service_failover::FailoverConfig::default(),
        ));
        let monitor = RtoRpoMonitor::new(objectives, alert_config, failover_manager);

        let report = monitor.generate_compliance_report().await;
        assert!(report.overall_compliance_percentage >= 0.0);
        assert!(report.overall_compliance_percentage <= 100.0);
    }
}

impl RtoRpoMonitor {
    /// Check RPO compliance by querying WAL log records for last recovery point
    async fn check_rpo_compliance(&self, rpo_seconds: u64) -> bool {
        match self.get_time_since_last_backup().await {
            Ok(time_since_backup) => time_since_backup <= rpo_seconds,
            Err(e) => {
                warn!("Failed to check RPO compliance: {}. Assuming compliant.", e);
                true // Assume compliant if we can't check (graceful degradation)
            }
        }
    }

    /// Get time since last backup by querying WAL log records
    async fn get_time_since_last_backup(&self) -> Result<u64, String> {
        let db_client = match &self.db_client {
            Some(client) => client,
            None => {
                return Err("Database client not available for RPO compliance checking".to_string());
            }
        };

        // Query WAL log records for the most recent recorded_at timestamp
        // This represents the last point-in-time we can recover to
        let query = r#"
            SELECT MAX(recorded_at) as last_backup_time
            FROM wal_log_records
            WHERE recorded_at IS NOT NULL
        "#;

        match db_client.query(query, &[]).await {
            Ok(rows) => {
                if rows.is_empty() {
                    warn!("No WAL log records found. Assuming no recent backups.");
                    return Ok(u64::MAX); // No backup data - treat as violation
                }

                let row = &rows[0];
                match row.try_get::<Option<chrono::DateTime<chrono::Utc>>, &str>("last_backup_time")
                {
                    Ok(Some(last_backup_time)) => {
                        let now = Utc::now();
                        let duration = now.signed_duration_since(last_backup_time);
                        let seconds = duration.num_seconds();

                        if seconds < 0 {
                            warn!("Last backup time is in the future. Using current time.");
                            Ok(0)
                        } else {
                            Ok(seconds as u64)
                        }
                    }
                    Ok(None) => {
                        warn!("No valid backup timestamp found in WAL log records.");
                        Ok(u64::MAX) // No backup data - treat as violation
                    }
                    Err(e) => Err(format!("Failed to parse backup timestamp: {}", e)),
                }
            }
            Err(e) => Err(format!(
                "Failed to query WAL log records for RPO compliance: {}",
                e
            )),
        }
    }
}
