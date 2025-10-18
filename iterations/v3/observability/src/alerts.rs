//! Alerting system for observability events

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub title: String,
    pub description: String,
    pub component: String,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
    pub status: AlertStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub value: Option<f64>,
    pub threshold: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertType {
    Performance,
    Availability,
    Resource,
    Security,
    Business,
    SLO,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertStatus {
    Firing,
    Resolved,
    Acknowledged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub name: String,
    pub description: String,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub condition: AlertCondition,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertCondition {
    pub metric: String,
    pub operator: AlertOperator,
    pub threshold: f64,
    pub duration_seconds: u64,
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertOperator {
    GreaterThan,
    LessThan,
    Equal,
    NotEqual,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

#[derive(Debug)]
pub struct AlertManager {
    alerts: Arc<RwLock<HashMap<String, Alert>>>,
    rules: Arc<RwLock<Vec<AlertRule>>>,
    alert_history: Arc<RwLock<Vec<Alert>>>,
    max_history_size: usize,
}

impl AlertManager {
    pub fn new() -> Self {
        Self {
            alerts: Arc::new(RwLock::new(HashMap::new())),
            rules: Arc::new(RwLock::new(Vec::new())),
            alert_history: Arc::new(RwLock::new(Vec::new())),
            max_history_size: 10000,
        }
    }

    /// Register an alert rule
    pub async fn register_rule(&self, rule: AlertRule) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut rules = self.rules.write().await;
        rules.push(rule);
        Ok(())
    }

    /// Evaluate all alert rules against current metrics
    pub async fn evaluate_rules(&self, metrics: &HashMap<String, f64>) -> Result<Vec<Alert>, Box<dyn std::error::Error + Send + Sync>> {
        let rules = self.rules.read().await;
        let mut new_alerts = Vec::new();

        for rule in rules.iter().filter(|r| r.enabled) {
            if self.evaluate_condition(&rule.condition, metrics).await {
                let alert = self.create_alert_from_rule(rule, metrics).await?;
                new_alerts.push(alert);
            }
        }

        Ok(new_alerts)
    }

    /// Create a manual alert
    pub async fn create_alert(
        &self,
        alert_type: AlertType,
        severity: AlertSeverity,
        title: String,
        description: String,
        component: String,
        labels: HashMap<String, String>,
        value: Option<f64>,
        threshold: Option<f64>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        let alert = Alert {
            id: id.clone(),
            alert_type,
            severity,
            title,
            description,
            component,
            labels,
            annotations: HashMap::new(),
            status: AlertStatus::Firing,
            created_at: now,
            updated_at: now,
            resolved_at: None,
            value,
            threshold,
        };

        let mut alerts = self.alerts.write().await;
        alerts.insert(id.clone(), alert);

        Ok(id)
    }

    /// Resolve an alert
    pub async fn resolve_alert(&self, alert_id: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut alerts = self.alerts.write().await;
        let mut history = self.alert_history.write().await;

        if let Some(alert) = alerts.get_mut(alert_id) {
            alert.status = AlertStatus::Resolved;
            alert.updated_at = Utc::now();
            alert.resolved_at = Some(Utc::now());

            // Move to history
            let resolved_alert = alert.clone();
            history.push(resolved_alert);

            // Clean up old alerts
            while history.len() > self.max_history_size {
                history.remove(0);
            }

            alerts.remove(alert_id);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Acknowledge an alert
    pub async fn acknowledge_alert(&self, alert_id: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut alerts = self.alerts.write().await;

        if let Some(alert) = alerts.get_mut(alert_id) {
            alert.status = AlertStatus::Acknowledged;
            alert.updated_at = Utc::now();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get all active alerts
    pub async fn get_active_alerts(&self) -> Vec<Alert> {
        let alerts = self.alerts.read().await;
        alerts.values().cloned().collect()
    }

    /// Get alerts by component
    pub async fn get_alerts_by_component(&self, component: &str) -> Vec<Alert> {
        let alerts = self.alerts.read().await;
        alerts.values()
            .filter(|a| a.component == component)
            .cloned()
            .collect()
    }

    /// Get alerts by severity
    pub async fn get_alerts_by_severity(&self, severity: &AlertSeverity) -> Vec<Alert> {
        let alerts = self.alerts.read().await;
        alerts.values()
            .filter(|a| a.severity == *severity)
            .cloned()
            .collect()
    }

    /// Get alert history
    pub async fn get_alert_history(&self, limit: usize) -> Vec<Alert> {
        let history = self.alert_history.read().await;
        history.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get alert statistics
    pub async fn get_alert_stats(&self) -> AlertStats {
        let alerts = self.alerts.read().await;
        let history = self.alert_history.read().await;

        let mut active_by_severity = HashMap::new();
        let mut resolved_by_severity = HashMap::new();

        for alert in alerts.values() {
            *active_by_severity.entry(alert.severity.clone()).or_insert(0) += 1;
        }

        for alert in history.iter() {
            *resolved_by_severity.entry(alert.severity.clone()).or_insert(0) += 1;
        }

        AlertStats {
            active_alerts: alerts.len(),
            resolved_alerts: history.len(),
            active_by_severity,
            resolved_by_severity,
        }
    }

    async fn evaluate_condition(&self, condition: &AlertCondition, metrics: &HashMap<String, f64>) -> bool {
        let metric_value = match metrics.get(&condition.metric) {
            Some(value) => *value,
            None => return false,
        };

        match condition.operator {
            AlertOperator::GreaterThan => metric_value > condition.threshold,
            AlertOperator::LessThan => metric_value < condition.threshold,
            AlertOperator::Equal => (metric_value - condition.threshold).abs() < f64::EPSILON,
            AlertOperator::NotEqual => (metric_value - condition.threshold).abs() >= f64::EPSILON,
            AlertOperator::GreaterThanOrEqual => metric_value >= condition.threshold,
            AlertOperator::LessThanOrEqual => metric_value <= condition.threshold,
        }
    }

    async fn create_alert_from_rule(&self, rule: &AlertRule, metrics: &HashMap<String, f64>) -> Result<Alert, Box<dyn std::error::Error + Send + Sync>> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        let value = metrics.get(&rule.condition.metric).copied();

        let alert = Alert {
            id,
            alert_type: rule.alert_type.clone(),
            severity: rule.severity.clone(),
            title: rule.name.clone(),
            description: rule.description.clone(),
            component: "observability".to_string(),
            labels: rule.labels.clone(),
            annotations: rule.annotations.clone(),
            status: AlertStatus::Firing,
            created_at: now,
            updated_at: now,
            resolved_at: None,
            value,
            threshold: Some(rule.condition.threshold),
        };

        // Store the alert
        let mut alerts = self.alerts.write().await;
        alerts.insert(alert.id.clone(), alert.clone());

        Ok(alert)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertStats {
    pub active_alerts: usize,
    pub resolved_alerts: usize,
    pub active_by_severity: HashMap<AlertSeverity, usize>,
    pub resolved_by_severity: HashMap<AlertSeverity, usize>,
}

// Predefined alert rules
pub fn create_default_alert_rules() -> Vec<AlertRule> {
    vec![
        AlertRule {
            name: "High CPU Usage".to_string(),
            description: "CPU usage is above 80%".to_string(),
            alert_type: AlertType::Resource,
            severity: AlertSeverity::Warning,
            condition: AlertCondition {
                metric: "cpu_usage_percent".to_string(),
                operator: AlertOperator::GreaterThan,
                threshold: 80.0,
                duration_seconds: 300,
                labels: HashMap::new(),
            },
            labels: HashMap::from([("resource".to_string(), "cpu".to_string())]),
            annotations: HashMap::from([("summary".to_string(), "High CPU usage detected".to_string())]),
            enabled: true,
        },
        AlertRule {
            name: "High Memory Usage".to_string(),
            description: "Memory usage is above 85%".to_string(),
            alert_type: AlertType::Resource,
            severity: AlertSeverity::Warning,
            condition: AlertCondition {
                metric: "memory_usage_percent".to_string(),
                operator: AlertOperator::GreaterThan,
                threshold: 85.0,
                duration_seconds: 300,
                labels: HashMap::new(),
            },
            labels: HashMap::from([("resource".to_string(), "memory".to_string())]),
            annotations: HashMap::from([("summary".to_string(), "High memory usage detected".to_string())]),
            enabled: true,
        },
        AlertRule {
            name: "Task Failure Rate".to_string(),
            description: "Task failure rate is above 10%".to_string(),
            alert_type: AlertType::Business,
            severity: AlertSeverity::Error,
            condition: AlertCondition {
                metric: "task_failure_rate".to_string(),
                operator: AlertOperator::GreaterThan,
                threshold: 0.1,
                duration_seconds: 600,
                labels: HashMap::new(),
            },
            labels: HashMap::from([("service".to_string(), "orchestration".to_string())]),
            annotations: HashMap::from([("summary".to_string(), "High task failure rate".to_string())]),
            enabled: true,
        },
        AlertRule {
            name: "SLO Violation".to_string(),
            description: "Service Level Objective is violated".to_string(),
            alert_type: AlertType::SLO,
            severity: AlertSeverity::Critical,
            condition: AlertCondition {
                metric: "slo_violation_count".to_string(),
                operator: AlertOperator::GreaterThan,
                threshold: 0.0,
                duration_seconds: 60,
                labels: HashMap::new(),
            },
            labels: HashMap::from([("type".to_string(), "slo".to_string())]),
            annotations: HashMap::from([("summary".to_string(), "SLO violation detected".to_string())]),
            enabled: true,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_alert_creation_and_resolution() {
        let manager = AlertManager::new();

        // Create an alert
        let alert_id = manager.create_alert(
            AlertType::Performance,
            AlertSeverity::Warning,
            "Test Alert".to_string(),
            "This is a test alert".to_string(),
            "test".to_string(),
            HashMap::from([("key".to_string(), "value".to_string())]),
            Some(95.0),
            Some(90.0),
        ).await.unwrap();

        // Check active alerts
        let active = manager.get_active_alerts().await;
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, alert_id);

        // Resolve the alert
        let resolved = manager.resolve_alert(&alert_id).await.unwrap();
        assert!(resolved);

        // Check that it's no longer active
        let active_after = manager.get_active_alerts().await;
        assert_eq!(active_after.len(), 0);
    }

    #[tokio::test]
    async fn test_alert_rule_evaluation() {
        let manager = AlertManager::new();

        let rule = AlertRule {
            name: "Test Rule".to_string(),
            description: "Test alert rule".to_string(),
            alert_type: AlertType::Performance,
            severity: AlertSeverity::Warning,
            condition: AlertCondition {
                metric: "test_metric".to_string(),
                operator: AlertOperator::GreaterThan,
                threshold: 50.0,
                duration_seconds: 60,
                labels: HashMap::new(),
            },
            labels: HashMap::new(),
            annotations: HashMap::new(),
            enabled: true,
        };

        manager.register_rule(rule).await.unwrap();

        // Test with value below threshold
        let metrics = HashMap::from([("test_metric".to_string(), 40.0)]);
        let alerts = manager.evaluate_rules(&metrics).await.unwrap();
        assert_eq!(alerts.len(), 0);

        // Test with value above threshold
        let metrics = HashMap::from([("test_metric".to_string(), 60.0)]);
        let alerts = manager.evaluate_rules(&metrics).await.unwrap();
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].severity, AlertSeverity::Warning);
    }
}
