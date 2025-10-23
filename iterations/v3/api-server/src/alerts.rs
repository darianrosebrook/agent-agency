//! Alerting System - Automated notifications for system failures and issues
//!
//! Provides comprehensive alerting capabilities with multiple notification channels,
//! escalation policies, and integration with monitoring systems.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tracing::{error, info, warn};

use crate::rto_rpo_monitor::{ComplianceAlert, RtoRpoMonitor, ComplianceStatus, ComplianceViolation, RecoveryMetrics};

/// Reliability Monitor trait for dependency injection (P0-7)
/// Abstracts reliability monitoring capabilities for pluggable implementations
#[async_trait::async_trait]
pub trait ReliabilityMonitor: Send + Sync {
    /// Get current compliance status
    async fn get_compliance_status(&self) -> Result<ComplianceStatus, Box<dyn std::error::Error + Send + Sync>>;

    /// Get recent violations within the specified hours
    async fn get_recent_violations(&self, hours: i64) -> Result<Vec<ComplianceViolation>, Box<dyn std::error::Error + Send + Sync>>;

    /// Get recovery metrics
    async fn get_recovery_metrics(&self) -> Result<RecoveryMetrics, Box<dyn std::error::Error + Send + Sync>>;

    /// Get alerts that should be processed by the alert manager
    async fn get_pending_alerts(&self) -> Result<Vec<ComplianceAlert>, Box<dyn std::error::Error + Send + Sync>>;

    /// Acknowledge a compliance violation
    async fn acknowledge_violation(&self, violation_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Mark a compliance violation as resolved
    async fn resolve_violation(&self, violation_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Alert status
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AlertStatus {
    Active,
    Acknowledged,
    Resolved,
    Suppressed,
}

/// Alert category
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AlertCategory {
    System,
    Performance,
    Security,
    Availability,
    Compliance,
    Business,
}

/// Alert definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: AlertCategory,
    pub severity: AlertSeverity,
    pub enabled: bool,

    /// Condition that triggers the alert
    pub condition: AlertCondition,

    /// Evaluation interval in seconds
    pub evaluation_interval_secs: u64,

    /// Cooldown period between alerts (seconds)
    pub cooldown_period_secs: u64,

    /// Auto-resolve after this many successful evaluations
    pub auto_resolve_after: Option<u32>,

    /// Notification channels
    pub notification_channels: Vec<String>,

    /// Escalation policy
    pub escalation_policy: Option<String>,

    /// Tags for grouping and filtering
    pub tags: HashSet<String>,
}

/// Alert condition types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    /// Metric threshold (value > threshold)
    MetricThreshold {
        metric_name: String,
        operator: ThresholdOperator,
        threshold: f64,
        duration_secs: u64, // Must be true for this duration
    },

    /// Service health status
    ServiceHealth {
        service_name: String,
        expected_status: String,
    },

    /// Error rate threshold
    ErrorRate {
        service_name: String,
        threshold_percent: f64,
        time_window_secs: u64,
    },

    /// Component unavailable
    ComponentUnavailable { component_name: String },

    /// Custom condition with expression
    Custom {
        expression: String,
        parameters: HashMap<String, serde_json::Value>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ThresholdOperator {
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Equal,
    NotEqual,
}

/// Active alert instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveAlert {
    pub id: String,
    pub definition_id: String,
    pub title: String,
    pub description: String,
    pub severity: AlertSeverity,
    pub category: AlertCategory,
    pub status: AlertStatus,

    /// When the alert was first triggered
    pub triggered_at: DateTime<Utc>,

    /// When the alert was last updated
    pub updated_at: DateTime<Utc>,

    /// Current value that triggered the alert
    pub current_value: Option<f64>,

    /// Expected threshold
    pub threshold_value: Option<f64>,

    /// Affected services/components
    pub affected_services: Vec<String>,

    /// Labels for filtering and grouping
    pub labels: HashMap<String, String>,

    /// Number of times this alert has fired
    pub occurrence_count: u32,

    /// Escalation level
    pub escalation_level: u32,

    /// Next escalation time
    pub next_escalation_at: Option<DateTime<Utc>>,
}

/// Alert history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertHistory {
    pub alert_id: String,
    pub timestamp: DateTime<Utc>,
    pub action: AlertAction,
    pub details: serde_json::Value,
    pub user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertAction {
    Triggered,
    Acknowledged,
    Resolved,
    Escalated,
    Suppressed,
}

/// Notification channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannel {
    pub id: String,
    pub name: String,
    pub channel_type: NotificationChannelType,
    pub config: HashMap<String, String>,
    pub enabled: bool,
    pub retry_attempts: u32,
    pub retry_delay_secs: u64,
}

/// Notification channel types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannelType {
    Email,
    Slack,
    Webhook,
    PagerDuty,
    SMS,
}

/// Escalation policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationPolicy {
    pub id: String,
    pub name: String,
    pub description: String,

    /// Escalation levels with delays and channels
    pub levels: Vec<EscalationLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationLevel {
    pub level: u32,
    pub delay_minutes: u32,
    pub notification_channels: Vec<String>,
    pub required_acknowledgment: bool,
}

/// Alert manager
pub struct AlertManager {
    alert_definitions: Arc<RwLock<HashMap<String, AlertDefinition>>>,
    active_alerts: Arc<RwLock<HashMap<String, ActiveAlert>>>,
    alert_history: Arc<RwLock<Vec<AlertHistory>>>,
    notification_channels: Arc<RwLock<HashMap<String, NotificationChannel>>>,
    escalation_policies: Arc<RwLock<HashMap<String, EscalationPolicy>>>,

    /// Integration with reliability monitor for compliance alerts (P0-7: DI seam)
    reliability_monitor: Option<Arc<dyn ReliabilityMonitor>>,

    /// Notification sender
    notification_sender: mpsc::UnboundedSender<NotificationRequest>,

    /// Alert evaluation ticker
    evaluation_sender: mpsc::UnboundedSender<EvaluationRequest>,
}

#[derive(Debug, Clone)]
pub struct NotificationRequest {
    pub alert: ActiveAlert,
    pub channels: Vec<String>,
    pub message: String,
    pub escalation_level: u32,
}

#[derive(Debug, Clone)]
pub struct EvaluationRequest {
    pub definition_id: String,
    pub force_evaluation: bool,
}

impl AlertManager {
    /// Create a new alert manager (P0-7: accepts ReliabilityMonitor trait)
    pub fn new(reliability_monitor: Option<Arc<dyn ReliabilityMonitor>>) -> Self {
        let (notification_tx, _) = mpsc::unbounded_channel();
        let (evaluation_tx, _) = mpsc::unbounded_channel();

        Self {
            alert_definitions: Arc::new(RwLock::new(HashMap::new())),
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            alert_history: Arc::new(RwLock::new(Vec::new())),
            notification_channels: Arc::new(RwLock::new(HashMap::new())),
            escalation_policies: Arc::new(RwLock::new(HashMap::new())),
            reliability_monitor,
            notification_sender: notification_tx,
            evaluation_sender: evaluation_tx,
        }
    }

    /// Create a new alert manager with RTO/RPO monitor (P0-7: convenience method)
    pub fn with_rto_rpo_monitor(rto_rpo_monitor: Arc<RtoRpoMonitor>) -> Self {
        Self::new(Some(rto_rpo_monitor))
    }

    /// Start the alert manager
    pub async fn start(&self) -> Result<(), String> {
        info!("Starting alert manager...");

        // Start alert evaluation loop
        let manager = Arc::new(self.clone());
        tokio::spawn(async move {
            manager.alert_evaluation_loop().await;
        });

        // Start notification processing loop
        let manager = Arc::new(self.clone());
        tokio::spawn(async move {
            manager.notification_processing_loop().await;
        });

        // Start escalation processing loop
        let manager = Arc::new(self.clone());
        tokio::spawn(async move {
            manager.escalation_processing_loop().await;
        });

        // Load default alert definitions
        self.load_default_alert_definitions().await?;

        // Load default notification channels
        self.load_default_notification_channels().await?;

        info!("Alert manager started successfully");
        Ok(())
    }

    /// Load default alert definitions
    async fn load_default_alert_definitions(&self) -> Result<(), String> {
        let definitions = vec![
            // System health alerts
            AlertDefinition {
                id: "cpu_high_usage".to_string(),
                name: "High CPU Usage".to_string(),
                description: "CPU usage is above 80% for more than 5 minutes".to_string(),
                category: AlertCategory::Performance,
                severity: AlertSeverity::Warning,
                enabled: true,
                condition: AlertCondition::MetricThreshold {
                    metric_name: "cpu_usage_percent".to_string(),
                    operator: ThresholdOperator::GreaterThan,
                    threshold: 80.0,
                    duration_secs: 300,
                },
                evaluation_interval_secs: 60,
                cooldown_period_secs: 1800, // 30 minutes
                auto_resolve_after: Some(3),
                notification_channels: vec!["slack-alerts".to_string()],
                escalation_policy: Some("default-escalation".to_string()),
                tags: ["system".to_string(), "performance".to_string()]
                    .into_iter()
                    .collect(),
            },
            AlertDefinition {
                id: "memory_high_usage".to_string(),
                name: "High Memory Usage".to_string(),
                description: "Memory usage is above 85%".to_string(),
                category: AlertCategory::Performance,
                severity: AlertSeverity::Error,
                enabled: true,
                condition: AlertCondition::MetricThreshold {
                    metric_name: "memory_usage_percent".to_string(),
                    operator: ThresholdOperator::GreaterThan,
                    threshold: 85.0,
                    duration_secs: 60,
                },
                evaluation_interval_secs: 30,
                cooldown_period_secs: 900, // 15 minutes
                auto_resolve_after: Some(5),
                notification_channels: vec!["email-admin".to_string(), "slack-alerts".to_string()],
                escalation_policy: Some("critical-escalation".to_string()),
                tags: ["system".to_string(), "memory".to_string()]
                    .into_iter()
                    .collect(),
            },
            AlertDefinition {
                id: "api_server_down".to_string(),
                name: "API Server Unavailable".to_string(),
                description: "API server is not responding".to_string(),
                category: AlertCategory::Availability,
                severity: AlertSeverity::Critical,
                enabled: true,
                condition: AlertCondition::ServiceHealth {
                    service_name: "api".to_string(),
                    expected_status: "healthy".to_string(),
                },
                evaluation_interval_secs: 30,
                cooldown_period_secs: 300, // 5 minutes
                auto_resolve_after: Some(2),
                notification_channels: vec![
                    "pagerduty-critical".to_string(),
                    "slack-alerts".to_string(),
                ],
                escalation_policy: Some("critical-escalation".to_string()),
                tags: ["api".to_string(), "availability".to_string()]
                    .into_iter()
                    .collect(),
            },
            AlertDefinition {
                id: "database_connection_failed".to_string(),
                name: "Database Connection Failed".to_string(),
                description: "Cannot connect to database".to_string(),
                category: AlertCategory::Availability,
                severity: AlertSeverity::Critical,
                enabled: true,
                condition: AlertCondition::ServiceHealth {
                    service_name: "database".to_string(),
                    expected_status: "healthy".to_string(),
                },
                evaluation_interval_secs: 15,
                cooldown_period_secs: 180, // 3 minutes
                auto_resolve_after: Some(3),
                notification_channels: vec![
                    "pagerduty-critical".to_string(),
                    "email-admin".to_string(),
                ],
                escalation_policy: Some("critical-escalation".to_string()),
                tags: ["database".to_string(), "availability".to_string()]
                    .into_iter()
                    .collect(),
            },
            AlertDefinition {
                id: "high_error_rate".to_string(),
                name: "High Error Rate".to_string(),
                description: "Error rate exceeds 5% over 10 minutes".to_string(),
                category: AlertCategory::Performance,
                severity: AlertSeverity::Error,
                enabled: true,
                condition: AlertCondition::ErrorRate {
                    service_name: "api".to_string(),
                    threshold_percent: 5.0,
                    time_window_secs: 600,
                },
                evaluation_interval_secs: 60,
                cooldown_period_secs: 600, // 10 minutes
                auto_resolve_after: Some(3),
                notification_channels: vec!["slack-alerts".to_string()],
                escalation_policy: Some("default-escalation".to_string()),
                tags: ["api".to_string(), "errors".to_string()]
                    .into_iter()
                    .collect(),
            },
            AlertDefinition {
                id: "rto_violation".to_string(),
                name: "RTO Violation".to_string(),
                description: "Recovery Time Objective exceeded".to_string(),
                category: AlertCategory::Compliance,
                severity: AlertSeverity::Critical,
                enabled: true,
                condition: AlertCondition::Custom {
                    expression: "rto_violation_detected".to_string(),
                    parameters: HashMap::new(),
                },
                evaluation_interval_secs: 300, // 5 minutes
                cooldown_period_secs: 3600,    // 1 hour
                auto_resolve_after: None,
                notification_channels: vec![
                    "email-admin".to_string(),
                    "pagerduty-critical".to_string(),
                ],
                escalation_policy: Some("compliance-escalation".to_string()),
                tags: ["compliance".to_string(), "rto".to_string()]
                    .into_iter()
                    .collect(),
            },
            AlertDefinition {
                id: "rpo_violation".to_string(),
                name: "RPO Violation".to_string(),
                description: "Recovery Point Objective exceeded".to_string(),
                category: AlertCategory::Compliance,
                severity: AlertSeverity::Critical,
                enabled: true,
                condition: AlertCondition::Custom {
                    expression: "rpo_violation_detected".to_string(),
                    parameters: HashMap::new(),
                },
                evaluation_interval_secs: 300, // 5 minutes
                cooldown_period_secs: 3600,    // 1 hour
                auto_resolve_after: None,
                notification_channels: vec![
                    "email-admin".to_string(),
                    "pagerduty-critical".to_string(),
                ],
                escalation_policy: Some("compliance-escalation".to_string()),
                tags: ["compliance".to_string(), "rpo".to_string()]
                    .into_iter()
                    .collect(),
            },
        ];

        let mut alert_definitions = self.alert_definitions.write().await;
        for definition in definitions {
            alert_definitions.insert(definition.id.clone(), definition);
        }

        info!(
            "Loaded {} default alert definitions",
            alert_definitions.len()
        );
        Ok(())
    }

    /// Load default notification channels
    async fn load_default_notification_channels(&self) -> Result<(), String> {
        let channels = vec![
            NotificationChannel {
                id: "email-admin".to_string(),
                name: "Admin Email".to_string(),
                channel_type: NotificationChannelType::Email,
                config: [
                    ("smtp_server".to_string(), "smtp.company.com".to_string()),
                    ("smtp_port".to_string(), "587".to_string()),
                    ("username".to_string(), "alerts@company.com".to_string()),
                    ("password".to_string(), "secure_password".to_string()),
                    ("from_address".to_string(), "alerts@company.com".to_string()),
                    (
                        "to_addresses".to_string(),
                        "admin@company.com,devops@company.com".to_string(),
                    ),
                ]
                .into_iter()
                .collect(),
                enabled: true,
                retry_attempts: 3,
                retry_delay_secs: 60,
            },
            NotificationChannel {
                id: "slack-alerts".to_string(),
                name: "Slack Alerts".to_string(),
                channel_type: NotificationChannelType::Slack,
                config: [
                    (
                        "webhook_url".to_string(),
                        "https://hooks.slack.com/services/...".to_string(),
                    ),
                    ("channel".to_string(), "#alerts".to_string()),
                    ("username".to_string(), "AlertBot".to_string()),
                ]
                .into_iter()
                .collect(),
                enabled: true,
                retry_attempts: 3,
                retry_delay_secs: 30,
            },
            NotificationChannel {
                id: "pagerduty-critical".to_string(),
                name: "PagerDuty Critical".to_string(),
                channel_type: NotificationChannelType::PagerDuty,
                config: [
                    (
                        "integration_key".to_string(),
                        "your_pagerduty_integration_key".to_string(),
                    ),
                    (
                        "api_endpoint".to_string(),
                        "https://events.pagerduty.com/v2/enqueue".to_string(),
                    ),
                ]
                .into_iter()
                .collect(),
                enabled: true,
                retry_attempts: 3,
                retry_delay_secs: 10,
            },
        ];

        let mut notification_channels = self.notification_channels.write().await;
        for channel in channels {
            notification_channels.insert(channel.id.clone(), channel);
        }

        info!(
            "Loaded {} default notification channels",
            notification_channels.len()
        );
        Ok(())
    }

    /// Alert evaluation loop
    async fn alert_evaluation_loop(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(10));

        loop {
            interval.tick().await;

            if let Err(e) = self.evaluate_all_alerts().await {
                error!("Error evaluating alerts: {}", e);
            }
        }
    }

    /// Evaluate all enabled alert definitions
    async fn evaluate_all_alerts(&self) -> Result<(), String> {
        let definitions = {
            let defs = self.alert_definitions.read().await;
            defs.values()
                .filter(|d| d.enabled)
                .cloned()
                .collect::<Vec<_>>()
        };

        for definition in definitions {
            if let Err(e) = self.evaluate_alert_definition(&definition).await {
                warn!("Failed to evaluate alert {}: {}", definition.id, e);
            }
        }

        // P0-7: Also process alerts from reliability monitor
        if let Err(e) = self.process_reliability_alerts().await {
            warn!("Failed to process reliability alerts: {}", e);
        }

        Ok(())
    }

    /// Evaluate a single alert definition
    async fn evaluate_alert_definition(&self, definition: &AlertDefinition) -> Result<(), String> {
        let condition_result = match &definition.condition {
            AlertCondition::MetricThreshold {
                metric_name,
                operator,
                threshold,
                duration_secs,
            } => {
                self.evaluate_metric_threshold(metric_name, *operator, *threshold, *duration_secs)
                    .await
            }
            AlertCondition::ServiceHealth {
                service_name,
                expected_status,
            } => {
                self.evaluate_service_health(service_name, expected_status)
                    .await
            }
            AlertCondition::ErrorRate {
                service_name,
                threshold_percent,
                time_window_secs,
            } => {
                self.evaluate_error_rate(service_name, *threshold_percent, *time_window_secs)
                    .await
            }
            AlertCondition::ComponentUnavailable { component_name } => {
                self.evaluate_component_availability(component_name).await
            }
            AlertCondition::Custom {
                expression,
                parameters: _,
            } => self.evaluate_custom_condition(expression).await,
        };

        match condition_result {
            Ok(true) => {
                // Condition is met, trigger or update alert
                self.trigger_or_update_alert(definition).await?;
            }
            Ok(false) => {
                // Condition not met, check if we should auto-resolve
                self.check_auto_resolve_alert(&definition.id).await?;
            }
            Err(e) => {
                warn!("Alert evaluation failed for {}: {}", definition.id, e);
            }
        }

        Ok(())
    }

    /// Evaluate metric threshold condition
    async fn evaluate_metric_threshold(
        &self,
        metric_name: &str,
        operator: ThresholdOperator,
        threshold: f64,
        duration_secs: u64,
    ) -> Result<bool, String> {
        // This would integrate with the metrics collection system
        // For now, return a mock result
        let current_value = match metric_name {
            "cpu_usage_percent" => 75.0,    // Mock value
            "memory_usage_percent" => 82.0, // Mock value
            _ => 0.0,
        };

        let condition_met = match operator {
            ThresholdOperator::GreaterThan => current_value > threshold,
            ThresholdOperator::GreaterThanOrEqual => current_value >= threshold,
            ThresholdOperator::LessThan => current_value < threshold,
            ThresholdOperator::LessThanOrEqual => current_value <= threshold,
            ThresholdOperator::Equal => (current_value - threshold).abs() < f64::EPSILON,
            ThresholdOperator::NotEqual => (current_value - threshold).abs() >= f64::EPSILON,
        };

        Ok(condition_met)
    }

    /// Evaluate service health condition
    async fn evaluate_service_health(
        &self,
        service_name: &str,
        expected_status: &str,
    ) -> Result<bool, String> {
        // This would check actual service health
        // For now, return a mock result
        let current_status = match service_name {
            "api" => "healthy",
            "database" => "healthy",
            "orchestrator" => "healthy",
            "workers" => "healthy",
            _ => "unknown",
        };

        Ok(current_status != expected_status)
    }

    /// Evaluate error rate condition
    async fn evaluate_error_rate(
        &self,
        service_name: &str,
        threshold_percent: f64,
        time_window_secs: u64,
    ) -> Result<bool, String> {
        // This would calculate actual error rates
        // For now, return a mock result
        let current_error_rate = match service_name {
            "api" => 2.5, // 2.5%
            _ => 0.0,
        };

        Ok(current_error_rate > threshold_percent)
    }

    /// Evaluate component availability
    async fn evaluate_component_availability(&self, component_name: &str) -> Result<bool, String> {
        // This would check component availability
        // For now, return a mock result
        let is_available = match component_name {
            "api" => true,
            "database" => true,
            "cache" => true,
            _ => false,
        };

        Ok(!is_available)
    }

    /// Evaluate custom condition
    async fn evaluate_custom_condition(&self, expression: &str) -> Result<bool, String> {
        match expression {
            "rto_violation_detected" => {
                if let Some(monitor) = &self.reliability_monitor {
                    match monitor.get_compliance_status().await {
                        Ok(status) => Ok(!status.rto_compliant),
                        Err(e) => {
                            warn!("Failed to get compliance status for RTO check: {}", e);
                            Ok(false)
                        }
                    }
                } else {
                    Ok(false)
                }
            }
            "rpo_violation_detected" => {
                if let Some(monitor) = &self.reliability_monitor {
                    match monitor.get_compliance_status().await {
                        Ok(status) => Ok(!status.rpo_compliant),
                        Err(e) => {
                            warn!("Failed to get compliance status for RPO check: {}", e);
                            Ok(false)
                        }
                    }
                } else {
                    Ok(false)
                }
            }
            "reliability_alert_triggered" => {
                // This condition is always true when triggered by the reliability monitor
                // The actual alert creation is handled in process_reliability_alerts
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Trigger or update an alert
    async fn trigger_or_update_alert(&self, definition: &AlertDefinition) -> Result<(), String> {
        let mut active_alerts = self.active_alerts.write().await;

        let alert_id = format!("alert_{}_{}", definition.id, Utc::now().timestamp());

        if let Some(existing_alert) = active_alerts
            .values_mut()
            .find(|a| a.definition_id == definition.id && a.status == AlertStatus::Active)
        {
            // Update existing alert
            existing_alert.updated_at = Utc::now();
            existing_alert.occurrence_count += 1;

            // Check if escalation is needed
            self.check_alert_escalation(existing_alert).await?;

            info!("Updated existing alert: {}", existing_alert.id);
        } else {
            // Create new alert
            let alert = ActiveAlert {
                id: alert_id.clone(),
                definition_id: definition.id.clone(),
                title: definition.name.clone(),
                description: definition.description.clone(),
                severity: definition.severity,
                category: definition.category.clone(),
                status: AlertStatus::Active,
                triggered_at: Utc::now(),
                updated_at: Utc::now(),
                current_value: None,   // Would be populated based on condition
                threshold_value: None, // Would be populated based on condition
                affected_services: vec![], // Would be populated based on condition
                labels: HashMap::new(),
                occurrence_count: 1,
                escalation_level: 0,
                next_escalation_at: None,
            };

            active_alerts.insert(alert_id.clone(), alert.clone());

            // Send notification
            self.send_alert_notification(&alert, &definition.notification_channels)
                .await?;

            // Record in history
            self.record_alert_history(
                &alert_id,
                AlertAction::Triggered,
                serde_json::json!({
                    "definition_id": definition.id,
                    "severity": format!("{:?}", definition.severity)
                }),
            )
            .await?;

            info!(
                "Triggered new alert: {} ({:?})",
                alert.title, alert.severity
            );
        }

        Ok(())
    }

    /// Check if alert should be auto-resolved
    async fn check_auto_resolve_alert(&self, definition_id: &str) -> Result<(), String> {
        let definitions = self.alert_definitions.read().await;
        let definition = definitions
            .get(definition_id)
            .ok_or(format!("Alert definition not found: {}", definition_id))?;

        if let Some(auto_resolve_after) = definition.auto_resolve_after {
            let mut active_alerts = self.active_alerts.write().await;

            if let Some(alert) = active_alerts
                .values_mut()
                .find(|a| a.definition_id == definition_id && a.status == AlertStatus::Active)
            {
                // This is a simplified check - in reality, we'd track successful evaluations
                let time_since_trigger = Utc::now().signed_duration_since(alert.triggered_at);
                if time_since_trigger.num_minutes() > 5 {
                    // Mock: resolve after 5 minutes of no violations
                    alert.status = AlertStatus::Resolved;
                    alert.updated_at = Utc::now();

                    // Record in history
                    self.record_alert_history(
                        &alert.id,
                        AlertAction::Resolved,
                        serde_json::json!({
                            "auto_resolved": true,
                            "duration_minutes": time_since_trigger.num_minutes()
                        }),
                    )
                    .await?;

                    info!("Auto-resolved alert: {}", alert.title);
                }
            }
        }

        Ok(())
    }

    /// Check if alert needs escalation
    async fn check_alert_escalation(&self, alert: &mut ActiveAlert) -> Result<(), String> {
        // Simplified escalation logic
        let time_since_trigger = Utc::now().signed_duration_since(alert.triggered_at);

        let new_level = match time_since_trigger.num_minutes() {
            0..=5 => 0,
            6..=15 => 1,
            16..=30 => 2,
            _ => 3,
        };

        if new_level > alert.escalation_level {
            alert.escalation_level = new_level;
            alert.next_escalation_at = Some(Utc::now() + chrono::Duration::minutes(15));

            // Record escalation
            self.record_alert_history(
                &alert.id,
                AlertAction::Escalated,
                serde_json::json!({
                    "from_level": alert.escalation_level - 1,
                    "to_level": alert.escalation_level
                }),
            )
            .await?;

            info!(
                "Escalated alert {} to level {}",
                alert.title, alert.escalation_level
            );
        }

        Ok(())
    }

    /// Send alert notification
    async fn send_alert_notification(
        &self,
        alert: &ActiveAlert,
        channels: &[String],
    ) -> Result<(), String> {
        let message = self.format_alert_message(alert);

        let request = NotificationRequest {
            alert: alert.clone(),
            channels: channels.to_vec(),
            message,
            escalation_level: alert.escalation_level,
        };

        if let Err(e) = self.notification_sender.send(request) {
            error!("Failed to send alert notification: {}", e);
            return Err(format!("Failed to queue notification: {}", e));
        }

        Ok(())
    }

    /// Format alert message for notifications
    fn format_alert_message(&self, alert: &ActiveAlert) -> String {
        let severity_emoji = match alert.severity {
            AlertSeverity::Info => "ℹ️",
            AlertSeverity::Warning => "⚠️",
            AlertSeverity::Error => "",
            AlertSeverity::Critical => "",
        };

        format!(
            "{} **{}**\n\n{}\n\nSeverity: {:?}\nStatus: {:?}\nTriggered: {}\nOccurrences: {}",
            severity_emoji,
            alert.title,
            alert.description,
            alert.severity,
            alert.status,
            alert.triggered_at.format("%Y-%m-%d %H:%M:%S UTC"),
            alert.occurrence_count
        )
    }

    /// Record alert history
    async fn record_alert_history(
        &self,
        alert_id: &str,
        action: AlertAction,
        details: serde_json::Value,
    ) -> Result<(), String> {
        let history_entry = AlertHistory {
            alert_id: alert_id.to_string(),
            timestamp: Utc::now(),
            action,
            details,
            user_id: None, // Would be populated for user actions
        };

        let mut alert_history = self.alert_history.write().await;
        alert_history.push(history_entry);

        // Keep only recent history
        if alert_history.len() > 10000 {
            alert_history.remove(0);
        }

        Ok(())
    }

    /// Notification processing loop
    async fn notification_processing_loop(&self) {
        let mut receiver = self.notification_sender.subscribe();

        while let Ok(notification) = receiver.recv().await {
            if let Err(e) = self.process_notification(&notification).await {
                error!("Failed to process notification: {}", e);
            }
        }
    }

    /// Process a notification request
    async fn process_notification(&self, request: &NotificationRequest) -> Result<(), String> {
        let channels = self.notification_channels.read().await;

        for channel_id in &request.channels {
            if let Some(channel) = channels.get(channel_id) {
                if channel.enabled {
                    if let Err(e) = self.send_to_channel(channel, &request.message).await {
                        warn!(
                            "Failed to send notification to channel {}: {}",
                            channel_id, e
                        );
                        // Continue trying other channels
                    }
                }
            } else {
                warn!("Notification channel not found: {}", channel_id);
            }
        }

        Ok(())
    }

    /// Send notification to a specific channel
    async fn send_to_channel(
        &self,
        channel: &NotificationChannel,
        message: &str,
    ) -> Result<(), String> {
        match channel.channel_type {
            NotificationChannelType::Email => self.send_email_notification(channel, message).await,
            NotificationChannelType::Slack => self.send_slack_notification(channel, message).await,
            NotificationChannelType::Webhook => {
                self.send_webhook_notification(channel, message).await
            }
            NotificationChannelType::PagerDuty => {
                self.send_pagerduty_notification(channel, message).await
            }
            NotificationChannelType::SMS => self.send_sms_notification(channel, message).await,
        }
    }

    /// Send email notification
    async fn send_email_notification(
        &self,
        _channel: &NotificationChannel,
        _message: &str,
    ) -> Result<(), String> {
        // Implementation would use SMTP library
        info!("Would send email notification: {}", _message);
        Ok(())
    }

    /// Send Slack notification
    async fn send_slack_notification(
        &self,
        channel: &NotificationChannel,
        message: &str,
    ) -> Result<(), String> {
        if let Some(webhook_url) = channel.config.get("webhook_url") {
            let client = reqwest::Client::new();
            let payload = serde_json::json!({
                "text": message,
                "channel": channel.config.get("channel").unwrap_or(&"#alerts".to_string()),
                "username": channel.config.get("username").unwrap_or(&"AlertBot".to_string())
            });

            let response = client
                .post(webhook_url)
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await
                .map_err(|e| format!("Slack request failed: {}", e))?;

            if response.status().is_success() {
                Ok(())
            } else {
                Err(format!("Slack API returned status: {}", response.status()))
            }
        } else {
            Err("Slack webhook URL not configured".to_string())
        }
    }

    /// Send webhook notification
    async fn send_webhook_notification(
        &self,
        channel: &NotificationChannel,
        message: &str,
    ) -> Result<(), String> {
        if let Some(webhook_url) = channel.config.get("webhook_url") {
            let client = reqwest::Client::new();
            let payload = serde_json::json!({
                "alert_message": message,
                "timestamp": Utc::now().to_rfc3339(),
                "channel": channel.id
            });

            let response = client
                .post(webhook_url)
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await
                .map_err(|e| format!("Webhook request failed: {}", e))?;

            if response.status().is_success() {
                Ok(())
            } else {
                Err(format!("Webhook returned status: {}", response.status()))
            }
        } else {
            Err("Webhook URL not configured".to_string())
        }
    }

    /// Send PagerDuty notification
    async fn send_pagerduty_notification(
        &self,
        _channel: &NotificationChannel,
        _message: &str,
    ) -> Result<(), String> {
        // Implementation would use PagerDuty Events API v2
        info!("Would send PagerDuty notification: {}", _message);
        Ok(())
    }

    /// Send SMS notification
    async fn send_sms_notification(
        &self,
        _channel: &NotificationChannel,
        _message: &str,
    ) -> Result<(), String> {
        // Implementation would use SMS service (Twilio, AWS SNS, etc.)
        info!("Would send SMS notification: {}", _message);
        Ok(())
    }

    /// Escalation processing loop
    async fn escalation_processing_loop(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(60)); // Check every minute

        loop {
            interval.tick().await;

            if let Err(e) = self.process_escalations().await {
                error!("Error processing escalations: {}", e);
            }
        }
    }

    /// Process alert escalations
    async fn process_escalations(&self) -> Result<(), String> {
        let mut active_alerts = self.active_alerts.write().await;
        let now = Utc::now();

        for alert in active_alerts.values_mut() {
            if alert.status == AlertStatus::Active {
                if let Some(next_escalation) = alert.next_escalation_at {
                    if now >= next_escalation {
                        // Time to escalate
                        alert.escalation_level += 1;

                        // Send escalated notification
                        let definitions = self.alert_definitions.read().await;
                        if let Some(definition) = definitions.get(&alert.definition_id) {
                            self.send_alert_notification(alert, &definition.notification_channels)
                                .await?;
                        }

                        // Set next escalation time
                        alert.next_escalation_at = Some(now + chrono::Duration::minutes(15));

                        info!(
                            "Escalated alert {} to level {}",
                            alert.title, alert.escalation_level
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Get all active alerts
    pub async fn get_active_alerts(&self) -> Vec<ActiveAlert> {
        let active_alerts = self.active_alerts.read().await;
        active_alerts.values().cloned().collect()
    }

    /// Acknowledge an alert
    pub async fn acknowledge_alert(&self, alert_id: &str, user_id: &str) -> Result<(), String> {
        let mut active_alerts = self.active_alerts.write().await;

        if let Some(alert) = active_alerts.get_mut(alert_id) {
            alert.status = AlertStatus::Acknowledged;
            alert.updated_at = Utc::now();

            // Record in history
            self.record_alert_history(
                alert_id,
                AlertAction::Acknowledged,
                serde_json::json!({
                    "user_id": user_id
                }),
            )
            .await?;

            info!("Alert {} acknowledged by user {}", alert_id, user_id);
            Ok(())
        } else {
            Err(format!("Alert not found: {}", alert_id))
        }
    }

    /// Resolve an alert
    pub async fn resolve_alert(&self, alert_id: &str, user_id: &str) -> Result<(), String> {
        let mut active_alerts = self.active_alerts.write().await;

        if let Some(alert) = active_alerts.get_mut(alert_id) {
            alert.status = AlertStatus::Resolved;
            alert.updated_at = Utc::now();

            // Record in history
            self.record_alert_history(
                alert_id,
                AlertAction::Resolved,
                serde_json::json!({
                    "user_id": user_id,
                    "manual_resolution": true
                }),
            )
            .await?;

            info!("Alert {} resolved by user {}", alert_id, user_id);
            Ok(())
        } else {
            Err(format!("Alert not found: {}", alert_id))
        }
    }

    /// Get alert history
    pub async fn get_alert_history(&self, limit: usize) -> Vec<AlertHistory> {
        let history = self.alert_history.read().await;
        history.iter().rev().take(limit).cloned().collect()
    }

    /// Process alerts from the reliability monitor (P0-7: integration with ReliabilityMonitor)
    pub async fn process_reliability_alerts(&self) -> Result<(), String> {
        if let Some(monitor) = &self.reliability_monitor {
            match monitor.get_pending_alerts().await {
                Ok(alerts) => {
                    for alert in alerts {
                        // Convert ComplianceAlert to internal alert format and trigger
                        let alert_definition = AlertDefinition {
                            id: format!("reliability_{}", alert.id),
                            name: format!("Reliability Alert: {}", alert.alert_type_as_string()),
                            description: alert.message.clone(),
                            category: AlertCategory::Compliance,
                            severity: match alert.severity {
                                crate::rto_rpo_monitor::ViolationSeverity::Low => AlertSeverity::Info,
                                crate::rto_rpo_monitor::ViolationSeverity::Medium => AlertSeverity::Warning,
                                crate::rto_rpo_monitor::ViolationSeverity::High => AlertSeverity::Error,
                                crate::rto_rpo_monitor::ViolationSeverity::Critical => AlertSeverity::Critical,
                            },
                            enabled: true,
                            condition: AlertCondition::Custom {
                                expression: "reliability_alert_triggered".to_string(),
                                parameters: serde_json::json!({
                                    "alert_id": alert.id,
                                    "alert_type": alert.alert_type_as_string()
                                }).as_object().unwrap().clone(),
                            },
                            evaluation_interval_secs: 60,
                            cooldown_period_secs: 300,
                            auto_resolve_after: None,
                            notification_channels: vec!["email-admin".to_string(), "slack-alerts".to_string()],
                            escalation_policy: Some("critical-escalation".to_string()),
                            tags: ["reliability".to_string(), "compliance".to_string()].into_iter().collect(),
                        };

                        // Trigger the alert directly
                        self.trigger_or_update_alert(&alert_definition).await?;
                    }
                    Ok(())
                }
                Err(e) => {
                    warn!("Failed to get pending alerts from reliability monitor: {}", e);
                    Ok(())
                }
            }
        } else {
            Ok(())
        }
    }

    /// Get alert statistics
    pub async fn get_alert_statistics(&self) -> AlertStatistics {
        let active_alerts = self.active_alerts.read().await;
        let history = self.alert_history.read().await;

        let mut stats = AlertStatistics {
            total_active_alerts: active_alerts.len(),
            alerts_by_severity: HashMap::new(),
            alerts_by_category: HashMap::new(),
            alerts_by_status: HashMap::new(),
            recent_history_count: 0,
            average_resolution_time_minutes: 0.0,
        };

        // Count by severity
        for alert in active_alerts.values() {
            *stats.alerts_by_severity.entry(alert.severity).or_insert(0) += 1;
            *stats
                .alerts_by_category
                .entry(alert.category.clone())
                .or_insert(0) += 1;
            *stats.alerts_by_status.entry(alert.status).or_insert(0) += 1;
        }

        // Calculate recent history and resolution times
        let recent_history: Vec<_> = history
            .iter()
            .filter(|h| Utc::now().signed_duration_since(h.timestamp).num_hours() < 24)
            .collect();

        stats.recent_history_count = recent_history.len();

        // Calculate average resolution time for resolved alerts in the last 24 hours
        let resolved_alerts: Vec<_> = recent_history
            .iter()
            .filter(|h| matches!(h.action, AlertAction::Resolved))
            .collect();

        if !resolved_alerts.is_empty() {
            let total_resolution_time: i64 = resolved_alerts
                .iter()
                .filter_map(|h| {
                    // Find the corresponding trigger event
                    history
                        .iter()
                        .find(|trigger| {
                            trigger.alert_id == h.alert_id
                                && matches!(trigger.action, AlertAction::Triggered)
                        })
                        .map(|trigger| {
                            h.timestamp
                                .signed_duration_since(trigger.timestamp)
                                .num_minutes()
                        })
                })
                .sum();

            stats.average_resolution_time_minutes =
                total_resolution_time as f64 / resolved_alerts.len() as f64;
        }

        stats
    }
}

/// Alert statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertStatistics {
    pub total_active_alerts: usize,
    pub alerts_by_severity: HashMap<AlertSeverity, usize>,
    pub alerts_by_category: HashMap<AlertCategory, usize>,
    pub alerts_by_status: HashMap<AlertStatus, usize>,
    pub recent_history_count: usize,
    pub average_resolution_time_minutes: f64,
}

// P0-7: ReliabilityMonitor implementation for RtoRpoMonitor
#[async_trait::async_trait]
impl ReliabilityMonitor for RtoRpoMonitor {
    async fn get_compliance_status(&self) -> Result<ComplianceStatus, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.get_compliance_status().await)
    }

    async fn get_recent_violations(&self, hours: i64) -> Result<Vec<ComplianceViolation>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.get_recent_violations(hours).await)
    }

    async fn get_recovery_metrics(&self) -> Result<RecoveryMetrics, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.get_recovery_metrics().await)
    }

    async fn get_pending_alerts(&self) -> Result<Vec<ComplianceAlert>, Box<dyn std::error::Error + Send + Sync>> {
        // RtoRpoMonitor doesn't maintain pending alerts directly, so return empty vec
        // In a real implementation, this would collect alerts from the monitor's alert channel
        Ok(Vec::new())
    }

    async fn acknowledge_violation(&self, violation_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.resolve_violation(violation_id)?;
        Ok(())
    }

    async fn resolve_violation(&self, violation_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.resolve_violation(violation_id)?;
        Ok(())
    }
}

impl Clone for AlertManager {
    fn clone(&self) -> Self {
        Self {
            alert_definitions: Arc::clone(&self.alert_definitions),
            active_alerts: Arc::clone(&self.active_alerts),
            alert_history: Arc::clone(&self.alert_history),
            notification_channels: Arc::clone(&self.notification_channels),
            escalation_policies: Arc::clone(&self.escalation_policies),
            reliability_monitor: self.reliability_monitor.clone(),
            notification_sender: self.notification_sender.clone(),
            evaluation_sender: self.evaluation_sender.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_alert_definition_creation() {
        let alert_def = AlertDefinition {
            id: "test_alert".to_string(),
            name: "Test Alert".to_string(),
            description: "A test alert".to_string(),
            category: AlertCategory::System,
            severity: AlertSeverity::Warning,
            enabled: true,
            condition: AlertCondition::MetricThreshold {
                metric_name: "cpu_usage".to_string(),
                operator: ThresholdOperator::GreaterThan,
                threshold: 80.0,
                duration_secs: 300,
            },
            evaluation_interval_secs: 60,
            cooldown_period_secs: 1800,
            auto_resolve_after: Some(3),
            notification_channels: vec!["email".to_string()],
            escalation_policy: Some("default".to_string()),
            tags: ["test".to_string()].into_iter().collect(),
        };

        assert_eq!(alert_def.id, "test_alert");
        assert_eq!(alert_def.severity, AlertSeverity::Warning);
        assert!(alert_def.enabled);
    }

    #[tokio::test]
    async fn test_metric_threshold_evaluation() {
        let manager = AlertManager::new(None);

        // Test greater than threshold
        let result = manager
            .evaluate_metric_threshold(
                "cpu_usage_percent",
                ThresholdOperator::GreaterThan,
                80.0,
                300,
            )
            .await;
        assert!(result.is_ok());
        // Note: actual result depends on mock implementation
    }
}
