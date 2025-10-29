//! Simple database client wrapper
//!
//! This provides a simple interface to the complex DatabaseClient
//! for backwards compatibility with existing code.

use crate::client::orchestrator::DatabaseClient as ComplexDatabaseClient;
use crate::database_config::DatabaseConfig;
use crate::database_operations::DatabaseOperations;
use anyhow::Result;
use sqlx::postgres::PgPool;
use sqlx::Row;
use std::sync::Arc;
use uuid::Uuid;

/// Simple database client that wraps the complex DatabaseClient
#[derive(Clone, Debug)]
pub struct DatabaseClient {
    inner: Arc<ComplexDatabaseClient>,
}

impl DatabaseClient {
    /// Create a new database client with the given configuration
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        let inner = ComplexDatabaseClient::new(config).await?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &PgPool {
        self.inner.pool()
    }

    /// Get a reference to the health monitor (if available)
    pub fn health_monitor(&self) -> Option<&crate::health::DatabaseHealthMonitor> {
        self.inner.health_monitor.as_ref().map(|arc| arc.as_ref())
    }

    /// Execute a parameterized query
    pub async fn execute(
        &self,
        query: &str,
        params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync)],
    ) -> Result<sqlx::postgres::PgQueryResult> {
        self.inner.execute(query, params).await
    }

    /// Execute a query and return rows
    pub async fn query(
        &self,
        query: &str,
        params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync)],
    ) -> Result<Vec<sqlx::postgres::PgRow>> {
        self.inner.query_with_params(query, params).await
    }

    /// Execute a query and return a single row (if any)
    pub async fn query_one(
        &self,
        query: &str,
        params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync)],
    ) -> Result<Option<sqlx::postgres::PgRow>> {
        self.inner.query_one_with_params(query, params).await
    }

    /// Execute a parameterized query and return rows (alias for query)
    pub async fn query_with_params(
        &self,
        query: &str,
        params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync)],
    ) -> Result<Vec<sqlx::postgres::PgRow>> {
        self.inner.query_with_params(query, params).await
    }

    /// Execute a parameterized query and return a single row (if any)
    pub async fn query_one_with_params(
        &self,
        query: &str,
        params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync)],
    ) -> Result<Option<sqlx::postgres::PgRow>> {
        self.inner.query_one_with_params(query, params).await
    }

    /// Execute a safe query (alias for execute with empty params)
    pub async fn execute_safe_query(&self, query: &str) -> Result<sqlx::postgres::PgQueryResult> {
        self.inner.execute_safe_query(query).await
    }

    /// Execute a parameterized query (alias for execute)
    pub async fn execute_parameterized_query(
        &self,
        query: &str,
        params: Vec<&(dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync)>,
    ) -> Result<sqlx::postgres::PgQueryResult> {
        self.inner.execute(query, &params).await
    }

    /// List all waivers
    pub async fn list_waivers(&self) -> Result<Vec<crate::models::Waiver>> {
        let rows = self.query(
            "SELECT id, title, reason, description, gates, approved_by, impact_level, mitigation_plan, expires_at, created_at, updated_at, status, metadata FROM waivers ORDER BY created_at DESC",
            &[]
        ).await?;

        let mut waivers = Vec::new();
        for row in rows {
            let gates_json: serde_json::Value = row.try_get("gates")?;
            let gates: Vec<String> = gates_json.as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect();

            let waiver = crate::models::Waiver {
                id: row.try_get("id")?,
                title: row.try_get("title")?,
                reason: row.try_get("reason")?,
                description: row.try_get("description")?,
                gates,
                approved_by: row.try_get("approved_by")?,
                impact_level: row.try_get("impact_level")?,
                mitigation_plan: row.try_get("mitigation_plan")?,
                expires_at: row.try_get("expires_at")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
                status: row.try_get("status")?,
                metadata: row.try_get("metadata")?,
            };
            waivers.push(waiver);
        }

        Ok(waivers)
    }

    /// Create a new waiver
    pub async fn create_waiver(&self, waiver: &crate::models::Waiver) -> Result<Uuid> {
        let gates_json = serde_json::to_value(&waiver.gates)?;
        
        self.execute(
            r#"
            INSERT INTO waivers (
                id, title, reason, description, gates, approved_by, 
                impact_level, mitigation_plan, expires_at, created_at, 
                updated_at, status, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
            &[
                &waiver.id,
                &waiver.title,
                &waiver.reason,
                &waiver.description,
                &gates_json,
                &waiver.approved_by,
                &waiver.impact_level,
                &waiver.mitigation_plan,
                &waiver.expires_at,
                &waiver.created_at,
                &waiver.updated_at,
                &waiver.status,
                &waiver.metadata,
            ]
        ).await?;

        Ok(waiver.id)
    }

    /// Get a waiver by ID
    pub async fn get_waiver(&self, waiver_id: &Uuid) -> Result<Option<crate::models::Waiver>> {
        let row = self.query_one(
            "SELECT id, title, reason, description, gates, approved_by, impact_level, mitigation_plan, expires_at, created_at, updated_at, status, metadata FROM waivers WHERE id = $1",
            &[waiver_id]
        ).await?;

        match row {
            Some(row) => {
                let gates_json: serde_json::Value = row.try_get("gates")?;
                let gates: Vec<String> = gates_json.as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect();

                let waiver = crate::models::Waiver {
                    id: row.try_get("id")?,
                    title: row.try_get("title")?,
                    reason: row.try_get("reason")?,
                    description: row.try_get("description")?,
                    gates,
                    approved_by: row.try_get("approved_by")?,
                    impact_level: row.try_get("impact_level")?,
                    mitigation_plan: row.try_get("mitigation_plan")?,
                    expires_at: row.try_get("expires_at")?,
                    created_at: row.try_get("created_at")?,
                    updated_at: row.try_get("updated_at")?,
                    status: row.try_get("status")?,
                    metadata: row.try_get("metadata")?,
                };
                Ok(Some(waiver))
            }
            None => Ok(None)
        }
    }

    /// Approve a waiver
    pub async fn approve_waiver(&self, waiver_id: &Uuid) -> Result<()> {
        let now = chrono::Utc::now();
        
        self.execute(
            "UPDATE waivers SET status = $1, updated_at = $2 WHERE id = $3",
            &[&"approved".to_string(), &now, waiver_id]
        ).await?;

        Ok(())
    }

    /// Delete a waiver
    pub async fn delete_waiver(&self, waiver_id: &Uuid) -> Result<()> {
        self.execute(
            "DELETE FROM waivers WHERE id = $1",
            &[waiver_id]
        ).await?;

        Ok(())
    }

    /// Get task provenance
    pub async fn get_task_provenance(&self, task_id: &Uuid) -> Result<Vec<crate::models::ProvenanceEntry>> {
        let rows = self.query(
            "SELECT id, task_id, action, actor, resource_id, resource_type, change_summary, timestamp, created_at, metadata FROM provenance_entries WHERE task_id = $1 ORDER BY created_at DESC",
            &[task_id]
        ).await?;

        let mut provenance_entries = Vec::new();
        for row in rows {
            let entry = crate::models::ProvenanceEntry {
                id: row.try_get("id")?,
                task_id: row.try_get("task_id")?,
                action: row.try_get("action")?,
                actor: row.try_get("actor")?,
                resource_id: row.try_get("resource_id")?,
                resource_type: row.try_get("resource_type")?,
                change_summary: row.try_get("change_summary")?,
                timestamp: row.try_get("timestamp")?,
                created_at: row.try_get("created_at")?,
                metadata: row.try_get("metadata")?,
            };
            provenance_entries.push(entry);
        }

        Ok(provenance_entries)
    }

    /// Create a task
    pub async fn create_task(&self, task: &crate::models::Task) -> Result<Uuid> {
        // Convert models::Task to database_operations::CreateTask
        let create_task = crate::database_operations::CreateTask {
            title: task.title.clone(),
            description: task.description.clone(),
            risk_tier: task.risk_tier.clone(),
            scope: task.scope.clone(),
            acceptance_criteria: task.acceptance_criteria.clone(),
            context: task.context.clone(),
            caws_spec: task.caws_spec.clone(),
            status: task.status.clone(),
            assigned_worker_id: task.assigned_worker_id,
            priority: task.priority,
            deadline: task.deadline,
            metadata: task.metadata.clone(),
        };
        
        let created_task = self.inner.create_task(create_task).await?;
        Ok(created_task.id)
    }

    /// Get a task by ID
    pub async fn get_task(&self, task_id: &Uuid) -> Result<Option<crate::models::Task>> {
        let task = self.inner.get_task(*task_id).await?;
        Ok(task.map(|t| crate::models::Task {
            id: t.id,
            title: t.title,
            description: t.description,
            risk_tier: t.risk_tier,
            scope: t.scope,
            acceptance_criteria: t.acceptance_criteria,
            context: t.context,
            caws_spec: t.caws_spec,
            status: t.status,
            assigned_worker_id: t.assigned_worker_id,
            priority: t.priority,
            deadline: t.deadline,
            metadata: t.metadata,
            created_at: t.created_at,
            updated_at: t.updated_at,
            completed_at: t.completed_at,
        }))
    }

    /// Revoke a waiver
    pub async fn revoke_waiver(&self, waiver_id: &Uuid, revoked_by: &str, revocation_reason: &str) -> Result<()> {
        let now = chrono::Utc::now();
        
        self.execute(
            "UPDATE waivers SET status = $1, updated_at = $2, metadata = jsonb_set(COALESCE(metadata, '{}'), '{revocation}', $3) WHERE id = $4",
            &[&"revoked".to_string(), &now, &serde_json::json!({
                "revoked_by": revoked_by,
                "revocation_reason": revocation_reason,
                "revoked_at": now
            }), waiver_id]
        ).await?;

        Ok(())
    }

    /// Get waiver audit trail
    pub async fn get_waiver_audit_trail(&self, waiver_id: &Uuid) -> Result<Vec<serde_json::Value>> {
        let rows = self.query(
            "SELECT action, actor, timestamp, metadata FROM waiver_audit_log WHERE waiver_id = $1 ORDER BY timestamp DESC",
            &[waiver_id]
        ).await?;

        let mut audit_entries = Vec::new();
        for row in rows {
            let entry = serde_json::json!({
                "action": row.try_get::<String, _>("action")?,
                "actor": row.try_get::<String, _>("actor")?,
                "timestamp": row.try_get::<chrono::DateTime<chrono::Utc>, _>("timestamp")?,
                "metadata": row.try_get::<serde_json::Value, _>("metadata")?
            });
            audit_entries.push(entry);
        }

        Ok(audit_entries)
    }

    /// Acknowledge SLO alert
    pub async fn acknowledge_slo_alert(&self, alert_id: &Uuid, acknowledged_by: &str, acknowledgment_notes: &str) -> Result<()> {
        let now = chrono::Utc::now();
        
        self.execute(
            "UPDATE slo_alerts SET status = $1, acknowledged_by = $2, acknowledged_at = $3, acknowledgment_notes = $4 WHERE id = $5",
            &[&"acknowledged".to_string(), &acknowledged_by, &now, &acknowledgment_notes, alert_id]
        ).await?;

        Ok(())
    }

    /// List SLOs
    pub async fn list_slos(&self) -> Result<Vec<serde_json::Value>> {
        let rows = self.query(
            "SELECT id, name, description, target_value, current_value, status, created_at, updated_at FROM slos ORDER BY created_at DESC",
            &[]
        ).await?;

        let mut slos = Vec::new();
        for row in rows {
            let slo = serde_json::json!({
                "id": row.try_get::<Uuid, _>("id")?,
                "name": row.try_get::<String, _>("name")?,
                "description": row.try_get::<String, _>("description")?,
                "target_value": row.try_get::<f64, _>("target_value")?,
                "current_value": row.try_get::<f64, _>("current_value")?,
                "status": row.try_get::<String, _>("status")?,
                "created_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at")?,
                "updated_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("updated_at")?
            });
            slos.push(slo);
        }

        Ok(slos)
    }

    /// Get SLO status
    pub async fn get_slo_status(&self, slo_id: &Uuid) -> Result<Option<serde_json::Value>> {
        let row = self.query_one(
            "SELECT id, name, description, target_value, current_value, status, created_at, updated_at FROM slos WHERE id = $1",
            &[slo_id]
        ).await?;

        match row {
            Some(row) => {
                let slo = serde_json::json!({
                    "id": row.try_get::<Uuid, _>("id")?,
                    "name": row.try_get::<String, _>("name")?,
                    "description": row.try_get::<String, _>("description")?,
                    "target_value": row.try_get::<f64, _>("target_value")?,
                    "current_value": row.try_get::<f64, _>("current_value")?,
                    "status": row.try_get::<String, _>("status")?,
                    "created_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at")?,
                    "updated_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("updated_at")?
                });
                Ok(Some(slo))
            }
            None => Ok(None)
        }
    }

    /// Get SLO measurements
    pub async fn get_slo_measurements(&self, slo_id: &Uuid) -> Result<Vec<serde_json::Value>> {
        let rows = self.query(
            "SELECT id, slo_id, value, timestamp, metadata FROM slo_measurements WHERE slo_id = $1 ORDER BY timestamp DESC LIMIT 100",
            &[slo_id]
        ).await?;

        let mut measurements = Vec::new();
        for row in rows {
            let measurement = serde_json::json!({
                "id": row.try_get::<Uuid, _>("id")?,
                "slo_id": row.try_get::<Uuid, _>("slo_id")?,
                "value": row.try_get::<f64, _>("value")?,
                "timestamp": row.try_get::<chrono::DateTime<chrono::Utc>, _>("timestamp")?,
                "metadata": row.try_get::<serde_json::Value, _>("metadata")?
            });
            measurements.push(measurement);
        }

        Ok(measurements)
    }

    /// List SLO alerts
    pub async fn list_slo_alerts(&self) -> Result<Vec<serde_json::Value>> {
        let rows = self.query(
            "SELECT id, slo_id, alert_type, severity, message, status, created_at, acknowledged_at FROM slo_alerts ORDER BY created_at DESC",
            &[]
        ).await?;

        let mut alerts = Vec::new();
        for row in rows {
            let alert = serde_json::json!({
                "id": row.try_get::<Uuid, _>("id")?,
                "slo_id": row.try_get::<Uuid, _>("slo_id")?,
                "alert_type": row.try_get::<String, _>("alert_type")?,
                "severity": row.try_get::<String, _>("severity")?,
                "message": row.try_get::<String, _>("message")?,
                "status": row.try_get::<String, _>("status")?,
                "created_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at")?,
                "acknowledged_at": row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("acknowledged_at")?
            });
            alerts.push(alert);
        }

        Ok(alerts)
    }

    /// List provenance records
    pub async fn list_provenance_records(&self) -> Result<Vec<crate::models::ProvenanceEntry>> {
        let rows = self.query(
            "SELECT id, task_id, action, actor, resource_id, resource_type, change_summary, timestamp, created_at, metadata FROM provenance_entries ORDER BY created_at DESC LIMIT 100",
            &[]
        ).await?;

        let mut provenance_entries = Vec::new();
        for row in rows {
            let entry = crate::models::ProvenanceEntry {
                id: row.try_get("id")?,
                task_id: row.try_get("task_id")?,
                action: row.try_get("action")?,
                actor: row.try_get("actor")?,
                resource_id: row.try_get("resource_id")?,
                resource_type: row.try_get("resource_type")?,
                change_summary: row.try_get("change_summary")?,
                timestamp: row.try_get("timestamp")?,
                created_at: row.try_get("created_at")?,
                metadata: row.try_get("metadata")?,
            };
            provenance_entries.push(entry);
        }

        Ok(provenance_entries)
    }

    /// Link provenance to commit
    pub async fn link_provenance_to_commit(&self, provenance_id: &Uuid, commit_hash: &str) -> Result<()> {
        self.execute(
            "UPDATE provenance_entries SET metadata = jsonb_set(COALESCE(metadata, '{}'), '{commit_hash}', $1) WHERE id = $2",
            &[&serde_json::Value::String(commit_hash.to_string()), provenance_id]
        ).await?;

        Ok(())
    }

    /// Verify provenance trailer
    pub async fn verify_provenance_trailer(&self, commit_hash: &str) -> Result<serde_json::Value> {
        let rows = self.query(
            "SELECT id, task_id, action, actor, resource_id, resource_type, change_summary, timestamp, created_at, metadata FROM provenance_entries WHERE metadata->>'commit_hash' = $1 ORDER BY timestamp DESC",
            &[&commit_hash]
        ).await?;

        let mut entries = Vec::new();
        for row in rows {
            let entry = serde_json::json!({
                "id": row.try_get::<Uuid, _>("id")?,
                "task_id": row.try_get::<Uuid, _>("task_id")?,
                "action": row.try_get::<String, _>("action")?,
                "actor": row.try_get::<String, _>("actor")?,
                "resource_id": row.try_get::<Option<Uuid>, _>("resource_id")?,
                "resource_type": row.try_get::<Option<String>, _>("resource_type")?,
                "change_summary": row.try_get::<String, _>("change_summary")?,
                "timestamp": row.try_get::<chrono::DateTime<chrono::Utc>, _>("timestamp")?,
                "created_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at")?,
                "metadata": row.try_get::<serde_json::Value, _>("metadata")?
            });
            entries.push(entry);
        }

        Ok(serde_json::json!({
            "commit_hash": commit_hash,
            "entries": entries,
            "verified": !entries.is_empty()
        }))
    }

    /// Get provenance by commit
    pub async fn get_provenance_by_commit(&self, commit_hash: &str) -> Result<Vec<crate::models::ProvenanceEntry>> {
        let rows = self.query(
            "SELECT id, task_id, action, actor, resource_id, resource_type, change_summary, timestamp, created_at, metadata FROM provenance_entries WHERE metadata->>'commit_hash' = $1 ORDER BY timestamp DESC",
            &[&commit_hash]
        ).await?;

        let mut provenance_entries = Vec::new();
        for row in rows {
            let entry = crate::models::ProvenanceEntry {
                id: row.try_get("id")?,
                task_id: row.try_get("task_id")?,
                action: row.try_get("action")?,
                actor: row.try_get("actor")?,
                resource_id: row.try_get("resource_id")?,
                resource_type: row.try_get("resource_type")?,
                change_summary: row.try_get("change_summary")?,
                timestamp: row.try_get("timestamp")?,
                created_at: row.try_get("created_at")?,
                metadata: row.try_get("metadata")?,
            };
            provenance_entries.push(entry);
        }

        Ok(provenance_entries)
    }

    /// Get system metrics
    pub async fn get_system_metrics(&self) -> Result<serde_json::Value> {
        // Get basic system metrics
        let task_count = self.query_one(
            "SELECT COUNT(*) as count FROM tasks",
            &[]
        ).await?;

        let active_task_count = self.query_one(
            "SELECT COUNT(*) as count FROM tasks WHERE status = 'running'",
            &[]
        ).await?;

        let waiver_count = self.query_one(
            "SELECT COUNT(*) as count FROM waivers",
            &[]
        ).await?;

        Ok(serde_json::json!({
            "total_tasks": task_count.map(|r| r.try_get::<i64, _>("count").unwrap_or(0)).unwrap_or(0),
            "active_tasks": active_task_count.map(|r| r.try_get::<i64, _>("count").unwrap_or(0)).unwrap_or(0),
            "total_waivers": waiver_count.map(|r| r.try_get::<i64, _>("count").unwrap_or(0)).unwrap_or(0),
            "timestamp": chrono::Utc::now()
        }))
    }

    /// Get dashboard data
    pub async fn get_dashboard_data(&self) -> Result<serde_json::Value> {
        let metrics = self.get_system_metrics().await?;
        
        let recent_tasks = self.query(
            "SELECT id, title, status, created_at FROM tasks ORDER BY created_at DESC LIMIT 10",
            &[]
        ).await?;

        let recent_waivers = self.query(
            "SELECT id, title, status, created_at FROM waivers ORDER BY created_at DESC LIMIT 10",
            &[]
        ).await?;

        let mut task_list = Vec::new();
        for row in recent_tasks {
            task_list.push(serde_json::json!({
                "id": row.try_get::<Uuid, _>("id")?,
                "title": row.try_get::<String, _>("title")?,
                "status": row.try_get::<String, _>("status")?,
                "created_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at")?
            }));
        }

        let mut waiver_list = Vec::new();
        for row in recent_waivers {
            waiver_list.push(serde_json::json!({
                "id": row.try_get::<Uuid, _>("id")?,
                "title": row.try_get::<String, _>("title")?,
                "status": row.try_get::<String, _>("status")?,
                "created_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at")?
            }));
        }

        Ok(serde_json::json!({
            "metrics": metrics,
            "recent_tasks": task_list,
            "recent_waivers": waiver_list,
            "timestamp": chrono::Utc::now()
        }))
    }

    /// List tasks
    pub async fn list_tasks(&self) -> Result<Vec<crate::models::Task>> {
        let rows = self.query(
            "SELECT id, title, description, risk_tier, scope, acceptance_criteria, context, caws_spec, status, assigned_worker_id, priority, deadline, metadata, created_at, updated_at, completed_at FROM tasks ORDER BY created_at DESC",
            &[]
        ).await?;

        let mut tasks = Vec::new();
        for row in rows {
            let task = crate::models::Task {
                id: row.try_get("id")?,
                title: row.try_get("title")?,
                description: row.try_get("description")?,
                risk_tier: row.try_get("risk_tier")?,
                scope: row.try_get("scope")?,
                acceptance_criteria: row.try_get("acceptance_criteria")?,
                context: row.try_get("context")?,
                caws_spec: row.try_get("caws_spec")?,
                status: row.try_get("status")?,
                assigned_worker_id: row.try_get("assigned_worker_id")?,
                priority: row.try_get("priority")?,
                deadline: row.try_get("deadline")?,
                metadata: row.try_get("metadata")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
                completed_at: row.try_get("completed_at")?,
            };
            tasks.push(task);
        }

        Ok(tasks)
    }
}
