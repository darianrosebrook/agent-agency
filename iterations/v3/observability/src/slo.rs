//! Service Level Objectives (SLO) tracking and monitoring

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLODefinition {
    pub name: String,
    pub description: String,
    pub service: String,
    pub metric: String,
    pub target: f64,      // Target value (e.g., 0.99 for 99%)
    pub window_days: i64, // Rolling window in days
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLOTarget {
    pub slo_name: String,
    pub target_value: f64,
    pub current_value: f64,
    pub compliance_percentage: f64,
    pub remaining_budget: f64, // Error budget remaining (0.0 to 1.0)
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub status: SLOStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SLOStatus {
    Compliant,
    AtRisk,
    Violated,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLOMeasurement {
    pub slo_name: String,
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub sample_count: u64,
    pub good_count: u64,
    pub bad_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLOAlert {
    pub slo_name: String,
    pub alert_type: SLOAlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub current_value: f64,
    pub target_value: f64,
    pub time_to_violation: Option<Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SLOAlertType {
    ApproachingViolation,
    ViolationImminent,
    Violated,
    Recovered,
    BudgetExhausted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug)]
pub struct SLOTracker {
    db_client: Arc<DatabaseClient>,
    alert_thresholds: SLOAlertThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLOAlertThresholds {
    pub warning_threshold: f64,   // e.g., 0.95 for 95% of target
    pub critical_threshold: f64,  // e.g., 0.90 for 90% of target
    pub violation_threshold: f64, // e.g., 0.99 (actual SLO target)
    pub recovery_threshold: f64,  // e.g., 0.995 for recovery
}

impl Default for SLOAlertThresholds {
    fn default() -> Self {
        Self {
            warning_threshold: 0.95,
            critical_threshold: 0.90,
            violation_threshold: 0.99,
            recovery_threshold: 0.995,
        }
    }
}

impl SLOTracker {
    pub fn new() -> Self {
        Self {
            definitions: Arc::new(RwLock::new(HashMap::new())),
            measurements: Arc::new(RwLock::new(HashMap::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
            alert_thresholds: SLOAlertThresholds::default(),
        }
    }

    pub fn with_alert_thresholds(mut self, thresholds: SLOAlertThresholds) -> Self {
        self.alert_thresholds = thresholds;
        self
    }

    /// Register a new SLO definition
    pub async fn register_slo(
        &self,
        definition: SLODefinition,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut definitions = self.definitions.write().await;
        definitions.insert(definition.name.clone(), definition);
        Ok(())
    }

    /// Record a measurement for an SLO
    pub async fn record_measurement(
        &self,
        slo_name: &str,
        value: f64,
        good_count: u64,
        bad_count: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let measurement = SLOMeasurement {
            slo_name: slo_name.to_string(),
            timestamp: Utc::now(),
            value,
            sample_count: good_count + bad_count,
            good_count,
            bad_count,
        };

        let mut measurements = self.measurements.write().await;
        measurements
            .entry(slo_name.to_string())
            .or_insert_with(Vec::new)
            .push(measurement);

        // Check for alerts
        self.check_slo_alerts(slo_name).await?;

        Ok(())
    }

    /// Get current SLO status
    pub async fn get_slo_status(
        &self,
        slo_name: &str,
    ) -> Result<SLOTarget, Box<dyn std::error::Error + Send + Sync>> {
        let definitions = self.definitions.read().await;
        let measurements = self.measurements.read().await;

        let definition = definitions
            .get(slo_name)
            .ok_or_else(|| format!("SLO '{}' not found", slo_name))?;

        let slo_measurements = measurements
            .get(slo_name)
            .map(|m| m.as_slice())
            .unwrap_or(&[]);

        // Calculate current value over the rolling window
        let window_start = Utc::now() - Duration::days(definition.window_days);
        let window_measurements: Vec<&SLOMeasurement> = slo_measurements
            .iter()
            .filter(|m| m.timestamp >= window_start)
            .collect();

        if window_measurements.is_empty() {
            return Ok(SLOTarget {
                slo_name: slo_name.to_string(),
                target_value: definition.target,
                current_value: 0.0,
                compliance_percentage: 0.0,
                remaining_budget: 1.0,
                period_start: window_start,
                period_end: Utc::now(),
                status: SLOStatus::Unknown,
            });
        }

        // Calculate weighted average based on sample counts
        let total_good: u64 = window_measurements.iter().map(|m| m.good_count).sum();
        let total_samples: u64 = window_measurements.iter().map(|m| m.sample_count).sum();

        let current_value = if total_samples > 0 {
            total_good as f64 / total_samples as f64
        } else {
            0.0
        };

        let compliance_percentage = (current_value / definition.target).min(1.0);
        let remaining_budget = (1.0 - current_value).max(0.0);

        let status = if current_value >= definition.target {
            SLOStatus::Compliant
        } else if current_value >= definition.target * self.alert_thresholds.warning_threshold {
            SLOStatus::AtRisk
        } else {
            SLOStatus::Violated
        };

        Ok(SLOTarget {
            slo_name: slo_name.to_string(),
            target_value: definition.target,
            current_value,
            compliance_percentage,
            remaining_budget,
            period_start: window_start,
            period_end: Utc::now(),
            status,
        })
    }

    /// Get all SLO statuses
    pub async fn get_all_slo_statuses(
        &self,
    ) -> Result<Vec<SLOTarget>, Box<dyn std::error::Error + Send + Sync>> {
        let definitions = self.definitions.read().await;
        let mut statuses = Vec::new();

        for slo_name in definitions.keys() {
            if let Ok(status) = self.get_slo_status(slo_name).await {
                statuses.push(status);
            }
        }

        Ok(statuses)
    }

    /// Get recent alerts
    pub async fn get_recent_alerts(&self, limit: usize) -> Vec<SLOAlert> {
        let alerts = self.alerts.read().await;
        alerts.iter().rev().take(limit).cloned().collect()
    }

    /// Check for SLO alerts and generate them
    async fn check_slo_alerts(
        &self,
        slo_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let status = self.get_slo_status(slo_name).await?;

        let mut alerts = self.alerts.write().await;

        // Check for violation alerts
        if status.status == SLOStatus::Violated {
            alerts.push(SLOAlert {
                slo_name: slo_name.to_string(),
                alert_type: SLOAlertType::Violated,
                severity: AlertSeverity::Critical,
                message: format!(
                    "SLO '{}' is violated: {:.2}% vs target {:.2}%",
                    slo_name,
                    status.current_value * 100.0,
                    status.target_value * 100.0
                ),
                timestamp: Utc::now(),
                current_value: status.current_value,
                target_value: status.target_value,
                time_to_violation: None,
            });
        }
        // Check for at-risk alerts
        else if status.status == SLOStatus::AtRisk {
            // Calculate time to violation if trending downward
            let time_to_violation = self.estimate_time_to_violation(slo_name).await;

            alerts.push(SLOAlert {
                slo_name: slo_name.to_string(),
                alert_type: SLOAlertType::ApproachingViolation,
                severity: AlertSeverity::Warning,
                message: format!(
                    "SLO '{}' is at risk: {:.2}% vs target {:.2}%",
                    slo_name,
                    status.current_value * 100.0,
                    status.target_value * 100.0
                ),
                timestamp: Utc::now(),
                current_value: status.current_value,
                target_value: status.target_value,
                time_to_violation,
            });
        }

        Ok(())
    }

    /// Estimate time to SLO violation based on trend
    async fn estimate_time_to_violation(&self, slo_name: &str) -> Option<Duration> {
        let measurements = self.measurements.read().await;
        let slo_measurements = measurements.get(slo_name)?;

        if slo_measurements.len() < 2 {
            return None;
        }

        // Simple linear trend calculation
        let recent = &slo_measurements[slo_measurements.len().saturating_sub(10)..];
        if recent.len() < 2 {
            return None;
        }

        let first = &recent[0];
        let last = &recent[recent.len() - 1];
        let time_diff = (last.timestamp - first.timestamp).num_seconds() as f64;
        let value_diff = last.value - first.value;

        if time_diff <= 0.0 || value_diff >= 0.0 {
            return None; // Not trending downward
        }

        let slope = value_diff / time_diff;
        let definitions = self.definitions.read().await;
        let target = definitions.get(slo_name)?.target;

        if slope >= 0.0 || last.value >= target {
            return None;
        }

        let time_to_target = (target - last.value) / slope.abs();
        Some(Duration::seconds(time_to_target as i64))
    }

    /// Clean up old measurements
    pub async fn cleanup_old_measurements(
        &self,
        max_age_days: i64,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let cutoff = Utc::now() - Duration::days(max_age_days);
        let mut measurements = self.measurements.write().await;

        let mut removed_count = 0;
        for measurements_list in measurements.values_mut() {
            let original_len = measurements_list.len();
            measurements_list.retain(|m| m.timestamp >= cutoff);
            removed_count += original_len - measurements_list.len();
        }

        Ok(removed_count)
    }

    /// Export SLO data for analysis
    pub async fn export_slo_data(
        &self,
    ) -> Result<SLOExport, Box<dyn std::error::Error + Send + Sync>> {
        let definitions = self.definitions.read().await;
        let measurements = self.measurements.read().await;
        let alerts = self.alerts.read().await;

        Ok(SLOExport {
            definitions: definitions.clone(),
            measurements: measurements.clone(),
            alerts: alerts.clone(),
            export_timestamp: Utc::now(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLOExport {
    pub definitions: HashMap<String, SLODefinition>,
    pub measurements: HashMap<String, Vec<SLOMeasurement>>,
    pub alerts: Vec<SLOAlert>,
    pub export_timestamp: DateTime<Utc>,
}

// Predefined SLOs for common services
pub fn create_default_slos() -> Vec<SLODefinition> {
    vec![
        SLODefinition {
            name: "api_availability".to_string(),
            description: "API endpoint availability".to_string(),
            service: "api".to_string(),
            metric: "uptime_percentage".to_string(),
            target: 0.995, // 99.5% uptime
            window_days: 30,
            labels: HashMap::new(),
        },
        SLODefinition {
            name: "task_completion".to_string(),
            description: "Task completion success rate".to_string(),
            service: "orchestration".to_string(),
            metric: "completion_rate".to_string(),
            target: 0.98, // 98% success rate
            window_days: 7,
            labels: HashMap::new(),
        },
        SLODefinition {
            name: "council_decision_time".to_string(),
            description: "Council decision response time".to_string(),
            service: "council".to_string(),
            metric: "p95_decision_time_ms".to_string(),
            target: 5000.0, // 5 seconds P95
            window_days: 7,
            labels: HashMap::new(),
        },
        SLODefinition {
            name: "worker_execution_time".to_string(),
            description: "Worker execution time".to_string(),
            service: "workers".to_string(),
            metric: "p95_execution_time_ms".to_string(),
            target: 30000.0, // 30 seconds P95
            window_days: 7,
            labels: HashMap::new(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_slo_registration_and_measurement() {
        let tracker = SLOTracker::new();

        let slo = SLODefinition {
            name: "test_slo".to_string(),
            description: "Test SLO".to_string(),
            service: "test".to_string(),
            metric: "test_metric".to_string(),
            target: 0.95,
            window_days: 7,
            labels: HashMap::new(),
        };

        tracker.register_slo(slo).await.unwrap();

        // Record some measurements
        tracker
            .record_measurement("test_slo", 0.98, 98, 2)
            .await
            .unwrap();
        tracker
            .record_measurement("test_slo", 0.96, 96, 4)
            .await
            .unwrap();

        // Check status
        let status = tracker.get_slo_status("test_slo").await.unwrap();
        assert!(status.current_value > 0.95); // Should be compliant
        assert_eq!(status.status, SLOStatus::Compliant);
    }
}
