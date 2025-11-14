//! Service Level Objectives (SLO) tracking and monitoring

use chrono::{DateTime, Duration, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Trait for database operations needed by SLOTracker
/// This allows SLOTracker to work with different database client implementations
/// without creating circular dependencies
#[async_trait::async_trait]
pub trait SloDatabaseClient: Send + Sync {
    /// Execute a query that doesn't return rows
    async fn execute(
        &self,
        query: &str,
        params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync)],
    ) -> Result<sqlx::postgres::PgQueryResult, Box<dyn std::error::Error + Send + Sync>>;

    /// Execute a query and return rows
    async fn query(
        &self,
        query: &str,
        params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync)],
    ) -> Result<Vec<sqlx::postgres::PgRow>, Box<dyn std::error::Error + Send + Sync>>;
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SLODefinition {
    pub name: String,
    pub description: String,
    pub service: String,
    pub metric: String,
    pub target: f64,      // Target value (e.g., 0.99 for 99%)
    pub window_days: i64, // Rolling window in days
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SLOTarget {
    pub slo_name: String,
    pub target_value: f64,
    pub current_value: f64,
    pub compliance_percentage: f64,
    pub remaining_budget: f64, // Error budget remaining (0.0 to 1.0)
    #[schemars(with = "String")]
    pub period_start: DateTime<Utc>,
    #[schemars(with = "String")]
    pub period_end: DateTime<Utc>,
    pub status: SLOStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub enum SLOStatus {
    Compliant,
    AtRisk,
    Violated,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SLOMeasurement {
    pub slo_name: String,
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub sample_count: u64,
    pub good_count: u64,
    pub bad_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SLOAlert {
    pub id: String,
    pub slo_name: String,
    pub alert_type: SLOAlertType,
    pub severity: AlertSeverity,
    pub message: String,
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    pub actual_value: f64,
    pub target_value: f64,
    #[schemars(with = "String")]
    pub triggered_at: DateTime<Utc>,
    #[schemars(with = "Option<String>")]
    pub resolved_at: Option<DateTime<Utc>>,
    #[schemars(with = "Option<String>")]
    pub time_to_violation: Option<Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum SLOAlertType {
    ApproachingViolation,
    ViolationImminent,
    Violated,
    Recovered,
    BudgetExhausted,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SLODataPoint {
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub is_good: bool, // Whether this measurement meets the SLO criteria
}

pub struct SLOTracker {
    db_client: Arc<dyn SloDatabaseClient>,
    alert_thresholds: SLOAlertThresholds,
    measurements: Arc<RwLock<HashMap<String, Vec<SLODataPoint>>>>,
    definitions: Arc<RwLock<HashMap<String, SLODefinition>>>,
    alerts: Arc<RwLock<Vec<SLOAlert>>>,
}

impl std::fmt::Debug for SLOTracker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SLOTracker")
            .field("alert_thresholds", &self.alert_thresholds)
            .field(
                "measurements_count",
                &self.measurements.blocking_read().len(),
            )
            .field("definitions_count", &self.definitions.blocking_read().len())
            .field("alerts_count", &self.alerts.blocking_read().len())
            .finish()
    }
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
    pub fn new(db_client: Arc<dyn SloDatabaseClient>) -> Self {
        Self {
            db_client,
            alert_thresholds: SLOAlertThresholds::default(),
            measurements: Arc::new(RwLock::new(HashMap::new())),
            definitions: Arc::new(RwLock::new(HashMap::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
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
        // Insert SLO definition into database
        self.db_client.execute(
            r#"
            INSERT INTO slo_definitions (name, description, service_name, slo_type, target_value, window_minutes)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (name) DO UPDATE SET
                description = EXCLUDED.description,
                target_value = EXCLUDED.target_value,
                window_minutes = EXCLUDED.window_minutes,
                updated_at = NOW()
            "#,
            &[
                &definition.name,
                &definition.description,
                &definition.service,
                &definition.metric, // slo_type
                &(definition.target as f64),
                // TODO: Implement configurable SLO time windows and measurement periods
                // - Support configurable time windows per SLO definition
                // - Implement rolling time window calculations for different durations
                // - Add time window validation and business rule enforcement
                // - Support multiple time windows per SLO (short-term vs long-term)
                // - Implement time window transitions and historical data management
                // - Add time window performance optimization and caching
                // - Support time window-based SLO alerting and thresholding
                // - Implement time window analytics and trend analysis
            ],
        ).await?;
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
        // Get SLO definition ID
        let slo_name = slo_name.to_string();
        let rows = self
            .db_client
            .query(
                "SELECT id FROM slo_definitions WHERE name = $1 AND is_active = true",
                &[&slo_name],
            )
            .await?;
        let slo_id: uuid::Uuid = rows
            .into_iter()
            .next()
            .ok_or("SLO definition not found")?
            .get::<uuid::Uuid, _>(0);

        // Insert measurement
        let total_samples = good_count + bad_count;
        let is_violation = if total_samples > 0 {
            (bad_count as f64 / total_samples as f64)
                > (1.0 - self.alert_thresholds.violation_threshold)
        } else {
            false
        };

        self.db_client.execute(
            r#"
            INSERT INTO slo_measurements (slo_id, actual_value, target_value, is_violation, sample_count, metadata)
            SELECT $1, $2, target_value, $3, $4, $5::jsonb
            FROM slo_definitions WHERE id = $1
            "#,
            &[
                &slo_id.to_string(),
                &(value as f64),
                &is_violation,
                &(total_samples as i32),
                &serde_json::json!({
                    "good_count": good_count,
                    "bad_count": bad_count
                }).to_string(),
            ],
        ).await?;

        // Update status snapshot
        self.update_slo_status_snapshot(&slo_id).await?;

        // Check for alerts
        self.check_slo_alerts(&slo_name).await?;

        Ok(())
    }

    /// Get current SLO status
    pub async fn get_slo_status(
        &self,
        slo_name: &str,
    ) -> Result<SLOTarget, Box<dyn std::error::Error + Send + Sync>> {
        // Get SLO definition and current status from database
        let rows = self
            .db_client
            .query(
                r#"
            SELECT
                d.name, d.target_value, d.window_minutes,
                s.current_value, s.error_budget_used, s.status
            FROM slo_definitions d
            LEFT JOIN slo_status_snapshots s ON d.id = s.slo_id
                AND s.last_updated = (
                    SELECT MAX(last_updated) FROM slo_status_snapshots
                    WHERE slo_id = d.id
                )
            WHERE d.name = $1 AND d.is_active = true
            "#,
                &[&slo_name],
            )
            .await?;
        let row = rows.into_iter().next().ok_or("SLO not found")?;

        let name: String = row.get::<String, _>(0);
        let target_value: f64 = row.get::<f64, _>(1);
        let window_minutes: i32 = row.get::<i32, _>(2);
        let current_value: Option<f64> = row.get::<Option<f64>, _>(3);
        let error_budget_used: Option<f64> = row.get::<Option<f64>, _>(4);
        let status_str: Option<String> = row.get::<Option<String>, _>(5);

        let current_value = current_value.unwrap_or(0.0);
        let error_budget_used = error_budget_used.unwrap_or(1.0);

        let compliance_percentage = if target_value > 0.0 {
            (current_value / target_value).min(1.0)
        } else {
            0.0
        };

        let remaining_budget = (1.0f64 - error_budget_used).max(0.0f64);

        let period_start = Utc::now() - Duration::minutes(window_minutes as i64);

        let status = match status_str.as_deref() {
            Some("healthy") => SLOStatus::Compliant,
            Some("warning") => SLOStatus::AtRisk,
            Some("critical") => SLOStatus::Violated,
            _ => SLOStatus::Unknown,
        };

        Ok(SLOTarget {
            slo_name: name,
            target_value,
            current_value,
            compliance_percentage,
            remaining_budget,
            period_start,
            period_end: Utc::now(),
            status,
        })
    }

    /// Get all SLO statuses
    pub async fn get_all_slo_statuses(
        &self,
    ) -> Result<Vec<SLOTarget>, Box<dyn std::error::Error + Send + Sync>> {
        let rows = self
            .db_client
            .query(
                r#"
            SELECT
                d.name, d.target_value, d.window_minutes,
                s.current_value, s.error_budget_used, s.status
            FROM slo_definitions d
            LEFT JOIN slo_status_snapshots s ON d.id = s.slo_id
                AND s.last_updated = (
                    SELECT MAX(last_updated) FROM slo_status_snapshots
                    WHERE slo_id = d.id
                )
            WHERE d.is_active = true
            ORDER BY d.name
            "#,
                &[],
            )
            .await?;

        let mut statuses = Vec::new();

        for row in rows {
            let name: String = row.get::<String, _>(0);
            let target_value: f64 = row.get::<f64, _>(1);
            let window_minutes: i32 = row.get::<i32, _>(2);
            let current_value: Option<f64> = row.get::<Option<f64>, _>(3);
            let error_budget_used: Option<f64> = row.get::<Option<f64>, _>(4);
            let status_str: Option<String> = row.get::<Option<String>, _>(5);

            let current_value = current_value.unwrap_or(0.0);
            let error_budget_used = error_budget_used.unwrap_or(1.0);

            let compliance_percentage = if target_value > 0.0 {
                (current_value / target_value).min(1.0)
            } else {
                0.0
            };

            let remaining_budget = (1.0f64 - error_budget_used).max(0.0f64);
            let period_start = Utc::now() - Duration::minutes(window_minutes as i64);

            let status = match status_str.as_deref() {
                Some("healthy") => SLOStatus::Compliant,
                Some("warning") => SLOStatus::AtRisk,
                Some("critical") => SLOStatus::Violated,
                _ => SLOStatus::Unknown,
            };

            statuses.push(SLOTarget {
                slo_name: name,
                target_value,
                current_value,
                compliance_percentage,
                remaining_budget,
                period_start,
                period_end: Utc::now(),
                status,
            });
        }

        Ok(statuses)
    }

    /// Get recent alerts
    pub async fn get_recent_alerts(
        &self,
        limit: usize,
    ) -> Result<Vec<SLOAlert>, Box<dyn std::error::Error + Send + Sync>> {
        let rows = match self
            .db_client
            .query(
                r#"
            SELECT
                a.id, d.name as slo_name, a.alert_type, a.severity, a.message,
                a.actual_value, a.target_value, a.triggered_at, a.resolved_at
            FROM slo_alerts a
            JOIN slo_definitions d ON a.slo_id = d.id
            ORDER BY a.triggered_at DESC
            LIMIT $1
            "#,
                &[&(limit as i32)],
            )
            .await
        {
            Ok(rows) => rows,
            Err(e) => {
                tracing::warn!("Failed to fetch SLO alerts from database: {}", e);
                return Ok(Vec::new());
            }
        };

        let mut alerts = Vec::new();
        for row in rows {
            let id: uuid::Uuid = row.get::<uuid::Uuid, _>(0);
            let slo_name: String = row.get::<String, _>(1);
            let alert_type_str: String = row.get::<String, _>(2);
            let severity_str: String = row.get::<String, _>(3);
            let message: String = row.get::<String, _>(4);
            let actual_value: Option<f64> = row.get::<Option<f64>, _>(5);
            let target_value: Option<f64> = row.get::<Option<f64>, _>(6);
            let triggered_at: DateTime<Utc> = {
                let timestamp_str: String = row.get::<String, _>(7);
                DateTime::parse_from_rfc3339(&timestamp_str)
                    .map_err(|e| format!("Invalid timestamp: {}", e))?
                    .with_timezone(&Utc)
            };
            let resolved_at: Option<DateTime<Utc>> = row
                .get::<Option<String>, _>(8)
                .map(|ts_str| {
                    DateTime::parse_from_rfc3339(&ts_str)
                        .map_err(|e| format!("Invalid timestamp: {}", e))
                        .map(|dt| dt.with_timezone(&Utc))
                })
                .transpose()?;

            let alert_type = match alert_type_str.as_str() {
                "warning" => SLOAlertType::ApproachingViolation,
                "critical" => SLOAlertType::ViolationImminent,
                "violation" => SLOAlertType::Violated,
                "recovery" => SLOAlertType::Recovered,
                _ => SLOAlertType::BudgetExhausted,
            };

            let severity = match severity_str.as_str() {
                "low" => AlertSeverity::Info,
                "medium" => AlertSeverity::Warning,
                "high" => AlertSeverity::Critical,
                _ => AlertSeverity::Critical,
            };

            alerts.push(SLOAlert {
                id: id.to_string(),
                slo_name,
                alert_type,
                severity,
                message,
                timestamp: triggered_at,
                actual_value: actual_value.unwrap_or(0.0),
                target_value: target_value.unwrap_or(0.0),
                triggered_at,
                resolved_at,
                time_to_violation: None, // TODO: Calculate time to violation if needed
            });
        }

        Ok(alerts)
    }

    /// Check for SLO alerts and generate them using database function
    async fn check_slo_alerts(
        &self,
        _slo_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Use the database function to check and generate alerts
        self.db_client
            .execute("SELECT check_slo_alerts()", &[])
            .await?;
        Ok(())
    }

    /// Update SLO status snapshot in database
    async fn update_slo_status_snapshot(
        &self,
        _slo_id: &uuid::Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.db_client
            .execute("SELECT update_slo_status_snapshots()", &[])
            .await?;
        Ok(())
    }

    /// Estimate time to SLO violation based on trend
    #[allow(dead_code)]
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SLOExport {
    pub definitions: HashMap<String, SLODefinition>,
    pub measurements: HashMap<String, Vec<SLODataPoint>>,
    pub alerts: Vec<SLOAlert>,
    #[schemars(with = "String")]
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

// Note: Implementations of SloDatabaseClient are provided by adapters in consuming crates
// This avoids circular dependencies. See:
// - agent-mcp: McpDatabaseClientAdapter
// - data-infrastructure: Can implement directly for DatabaseClient

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    #[ignore] // Requires database setup - run with --ignored flag
    async fn test_slo_registration_and_measurement() {
        // This test requires a real database client adapter
        // For now, skip - integration tests should be in a crate that has database access
        // TODO: Implement real database client adapter for this test
        // let db_client = Arc::new(/* adapter implementing SloDatabaseClient */);
        // let tracker = SLOTracker::new(db_client);

        // Test body commented out until database adapter is implemented
        // let slo = SLODefinition {
        //     name: "test_slo".to_string(),
        //     description: "Test SLO".to_string(),
        //     service: "test".to_string(),
        //     metric: "test_metric".to_string(),
        //     target: 0.95,
        //     window_days: 7,
        //     labels: HashMap::new(),
        // };

        // tracker.register_slo(slo).await.unwrap();

        // // Record some measurements
        // tracker
        //     .record_measurement("test_slo", 0.98, 98, 2)
        //     .await
        //     .unwrap();
        // tracker
        //     .record_measurement("test_slo", 0.96, 96, 4)
        //     .await
        //     .unwrap();

        // // Check status
        // let status = tracker.get_slo_status("test_slo").await.unwrap();
        // assert!(status.current_value > 0.95); // Should be compliant
        // assert_eq!(status.status, SLOStatus::Compliant);
    }
}
