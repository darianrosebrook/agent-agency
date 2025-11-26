//! Simple database client wrapper
//!
//! This provides a simple interface to the complex DatabaseClient
//! for backwards compatibility with existing code.

use crate::client::orchestrator::DatabaseClient as ComplexDatabaseClient;
use crate::database_config::DatabaseConfig;
use crate::database_operations::DatabaseOperations;
use anyhow::Result;
use schemars::JsonSchema;
use sqlx::postgres::PgPool;
use sqlx::Row;
use std::sync::Arc;
use uuid::Uuid;

/// Simple database client that wraps the complex DatabaseClient
#[derive(Clone, Debug, JsonSchema)]
pub struct DatabaseClient {
    #[schemars(skip)]
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
            let gates: Vec<String> = gates_json
                .as_array()
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
            ],
        )
        .await?;

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
                let gates: Vec<String> = gates_json
                    .as_array()
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
            None => Ok(None),
        }
    }

    /// Approve a waiver
    pub async fn approve_waiver(&self, waiver_id: &Uuid) -> Result<()> {
        let now = chrono::Utc::now();

        self.execute(
            "UPDATE waivers SET status = $1, updated_at = $2 WHERE id = $3",
            &[&"approved".to_string(), &now, waiver_id],
        )
        .await?;

        Ok(())
    }

    /// Delete a waiver
    pub async fn delete_waiver(&self, waiver_id: &Uuid) -> Result<()> {
        self.execute("DELETE FROM waivers WHERE id = $1", &[waiver_id])
            .await?;

        Ok(())
    }

    /// Create a provenance entry
    pub async fn create_provenance_entry(
        &self,
        task_id: Uuid,
        action: String,
        actor: String,
        change_summary: String,
        resource_id: Option<Uuid>,
        resource_type: Option<String>,
        metadata: serde_json::Value,
    ) -> Result<crate::models::ProvenanceEntry> {
        let id = Uuid::new_v4();
        let timestamp = chrono::Utc::now();
        let created_at = chrono::Utc::now();

        self.execute(
            "INSERT INTO provenance_entries (id, task_id, action, actor, resource_id, resource_type, change_summary, timestamp, created_at, metadata) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
            &[&id, &task_id, &action, &actor, &resource_id, &resource_type, &change_summary, &timestamp, &created_at, &metadata]
        ).await?;

        Ok(crate::models::ProvenanceEntry {
            id,
            task_id,
            action,
            actor,
            resource_id,
            resource_type,
            change_summary,
            timestamp,
            created_at,
            metadata,
        })
    }

    /// Get task provenance
    pub async fn get_task_provenance(
        &self,
        task_id: &Uuid,
    ) -> Result<Vec<crate::models::ProvenanceEntry>> {
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
            project_id: task.project_id,
            priority: task.priority,
            deadline: task.deadline,
            metadata: task.metadata.clone(),
        };

        let created_task = self.inner.create_task(create_task).await?;
        Ok(created_task.id)
    }

    /// Create a task from CreateTask struct
    pub async fn create_task_from_create(
        &self,
        create_task: crate::database_operations::CreateTask,
    ) -> Result<crate::models::Task> {
        let task = self.inner.create_task(create_task).await?;
        Ok(crate::models::Task {
            id: task.id,
            title: task.title,
            description: task.description,
            risk_tier: task.risk_tier,
            scope: task.scope,
            acceptance_criteria: task.acceptance_criteria,
            context: task.context,
            caws_spec: task.caws_spec,
            status: task.status,
            assigned_worker_id: task.assigned_worker_id,
            project_id: task.project_id,
            priority: task.priority,
            deadline: task.deadline,
            metadata: task.metadata,
            created_at: task.created_at,
            updated_at: task.updated_at,
            completed_at: task.completed_at,
        })
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
            project_id: t.project_id,
            priority: t.priority,
            deadline: t.deadline,
            metadata: t.metadata,
            created_at: t.created_at,
            updated_at: t.updated_at,
            completed_at: t.completed_at,
        }))
    }

    /// Update a task
    pub async fn update_task(
        &self,
        id: Uuid,
        update: crate::database_operations::UpdateTask,
    ) -> Result<crate::models::Task> {
        let task = self.inner.update_task(id, update).await?;
        Ok(crate::models::Task {
            id: task.id,
            title: task.title,
            description: task.description,
            risk_tier: task.risk_tier,
            scope: task.scope,
            acceptance_criteria: task.acceptance_criteria,
            context: task.context,
            caws_spec: task.caws_spec,
            status: task.status,
            assigned_worker_id: task.assigned_worker_id,
            project_id: task.project_id,
            priority: task.priority,
            deadline: task.deadline,
            metadata: task.metadata,
            created_at: task.created_at,
            updated_at: task.updated_at,
            completed_at: task.completed_at,
        })
    }

    /// Delete a task
    pub async fn delete_task(&self, id: Uuid) -> Result<()> {
        self.inner.delete_task(id).await
    }

    /// Get all tasks
    pub async fn get_tasks(&self) -> Result<Vec<crate::models::Task>> {
        let tasks = self.inner.get_tasks().await?;
        Ok(tasks
            .into_iter()
            .map(|t| crate::models::Task {
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
                project_id: t.project_id,
                priority: t.priority,
                deadline: t.deadline,
                metadata: t.metadata,
                created_at: t.created_at,
                updated_at: t.updated_at,
                completed_at: t.completed_at,
            })
            .collect())
    }

    /// Get tasks filtered by project_id
    pub async fn get_tasks_by_project(&self, project_id: Uuid) -> Result<Vec<crate::models::Task>> {
        let tasks = self.inner.get_tasks_by_project(project_id).await?;
        Ok(tasks
            .into_iter()
            .map(|t| crate::models::Task {
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
                project_id: t.project_id,
                priority: t.priority,
                deadline: t.deadline,
                metadata: t.metadata,
                created_at: t.created_at,
                updated_at: t.updated_at,
                completed_at: t.completed_at,
            })
            .collect())
    }

    /// Get task statistics for a specific project
    pub async fn get_project_task_stats(&self, project_id: Uuid) -> Result<serde_json::Value> {
        self.inner.get_project_task_stats(project_id).await
    }

    // =========================================================================
    // TELEMETRY OPERATIONS
    // =========================================================================

    /// Get model contributions (LLM usage statistics)
    pub async fn get_model_contributions(
        &self,
        hours: Option<i32>,
    ) -> Result<Vec<serde_json::Value>> {
        let hours = hours.unwrap_or(24);
        let rows = self
            .query(
                r#"
            SELECT 
                model_name,
                SUM(request_count) as total_requests,
                SUM(total_tokens) as total_tokens,
                SUM(prompt_tokens) as prompt_tokens,
                SUM(completion_tokens) as completion_tokens,
                SUM(success_count) as successful_requests,
                SUM(failure_count) as failed_requests,
                CASE 
                    WHEN SUM(request_count) > 0 
                    THEN SUM(success_count)::DOUBLE PRECISION / SUM(request_count)
                    ELSE 0.0 
                END as success_rate,
                AVG(avg_response_time_ms) as avg_response_time_ms,
                SUM(total_cost_usd) as total_cost_usd
            FROM telemetry_model_contributions
            WHERE recorded_at >= NOW() - ($1 || ' hours')::INTERVAL
            GROUP BY model_name
            ORDER BY total_requests DESC
            "#,
                &[&hours.to_string()],
            )
            .await;

        match rows {
            Ok(rows) => {
                let mut contributions = Vec::new();
                for row in rows {
                    contributions.push(serde_json::json!({
                        "model_name": row.try_get::<String, _>("model_name").unwrap_or_default(),
                        "total_requests": row.try_get::<i64, _>("total_requests").unwrap_or(0),
                        "total_tokens": row.try_get::<i64, _>("total_tokens").unwrap_or(0),
                        "prompt_tokens": row.try_get::<i64, _>("prompt_tokens").unwrap_or(0),
                        "completion_tokens": row.try_get::<i64, _>("completion_tokens").unwrap_or(0),
                        "successful_requests": row.try_get::<i64, _>("successful_requests").unwrap_or(0),
                        "failed_requests": row.try_get::<i64, _>("failed_requests").unwrap_or(0),
                        "success_rate": row.try_get::<f64, _>("success_rate").unwrap_or(0.0),
                        "avg_response_time_ms": row.try_get::<f64, _>("avg_response_time_ms").unwrap_or(0.0),
                        "total_cost_usd": row.try_get::<f64, _>("total_cost_usd").unwrap_or(0.0),
                    }));
                }
                Ok(contributions)
            }
            Err(_) => {
                // Table might not exist yet, return empty array
                Ok(Vec::new())
            }
        }
    }

    /// Get agent activity data
    pub async fn get_agent_activity(&self, hours: Option<i32>) -> Result<Vec<serde_json::Value>> {
        let hours = hours.unwrap_or(24);
        let rows = self
            .query(
                r#"
            SELECT 
                a.agent_id,
                w.name as agent_name,
                a.activity_type,
                SUM(a.activity_count) as total_activities,
                COUNT(*) FILTER (WHERE a.success = TRUE) as successful,
                COUNT(*) FILTER (WHERE a.success = FALSE) as failed,
                AVG(a.duration_ms) as avg_duration_ms,
                MAX(a.recorded_at) as last_activity
            FROM telemetry_agent_activity a
            LEFT JOIN workers w ON a.agent_id = w.id
            WHERE a.recorded_at >= NOW() - ($1 || ' hours')::INTERVAL
            GROUP BY a.agent_id, w.name, a.activity_type
            ORDER BY total_activities DESC
            "#,
                &[&hours.to_string()],
            )
            .await;

        match rows {
            Ok(rows) => {
                let mut activities = Vec::new();
                for row in rows {
                    activities.push(serde_json::json!({
                        "agent_id": row.try_get::<Uuid, _>("agent_id").ok(),
                        "agent_name": row.try_get::<String, _>("agent_name").ok(),
                        "activity_type": row.try_get::<String, _>("activity_type").unwrap_or_default(),
                        "total_activities": row.try_get::<i64, _>("total_activities").unwrap_or(0),
                        "successful": row.try_get::<i64, _>("successful").unwrap_or(0),
                        "failed": row.try_get::<i64, _>("failed").unwrap_or(0),
                        "avg_duration_ms": row.try_get::<f64, _>("avg_duration_ms").ok(),
                        "last_activity": row.try_get::<chrono::DateTime<chrono::Utc>, _>("last_activity").ok(),
                    }));
                }
                Ok(activities)
            }
            Err(_) => {
                // Table might not exist yet, return empty array
                Ok(Vec::new())
            }
        }
    }

    /// Get task stats history
    pub async fn get_task_stats_history(&self, days: Option<i32>) -> Result<Vec<serde_json::Value>> {
        let days = days.unwrap_or(30);
        let rows = self
            .query(
                r#"
            SELECT 
                snapshot_date,
                total,
                completed,
                in_progress,
                pending,
                failed,
                cancelled,
                paused,
                completion_rate,
                success_rate,
                avg_completion_time_ms
            FROM task_stats_history
            WHERE snapshot_date >= CURRENT_DATE - ($1 || ' days')::INTERVAL
            ORDER BY snapshot_date DESC
            "#,
                &[&days.to_string()],
            )
            .await;

        match rows {
            Ok(rows) => {
                let mut history = Vec::new();
                for row in rows {
                    history.push(serde_json::json!({
                        "snapshot_date": row.try_get::<chrono::NaiveDate, _>("snapshot_date").ok(),
                        "total": row.try_get::<i32, _>("total").unwrap_or(0),
                        "completed": row.try_get::<i32, _>("completed").unwrap_or(0),
                        "in_progress": row.try_get::<i32, _>("in_progress").unwrap_or(0),
                        "pending": row.try_get::<i32, _>("pending").unwrap_or(0),
                        "failed": row.try_get::<i32, _>("failed").unwrap_or(0),
                        "cancelled": row.try_get::<i32, _>("cancelled").unwrap_or(0),
                        "paused": row.try_get::<i32, _>("paused").unwrap_or(0),
                        "completion_rate": row.try_get::<f64, _>("completion_rate").unwrap_or(0.0),
                        "success_rate": row.try_get::<f64, _>("success_rate").unwrap_or(0.0),
                        "avg_completion_time_ms": row.try_get::<f64, _>("avg_completion_time_ms").ok(),
                    }));
                }
                Ok(history)
            }
            Err(_) => {
                // Table might not exist yet, return empty array
                Ok(Vec::new())
            }
        }
    }

    /// Record an LLM request for telemetry
    pub async fn record_llm_request(
        &self,
        model_name: &str,
        provider: &str,
        task_id: Option<Uuid>,
        agent_id: Option<Uuid>,
        prompt_tokens: i32,
        completion_tokens: i32,
        response_time_ms: Option<i32>,
        success: bool,
        error_message: Option<&str>,
        cost_usd: Option<f64>,
        metadata: Option<serde_json::Value>,
    ) -> Result<Uuid> {
        let request_id = Uuid::new_v4();
        let total_tokens = prompt_tokens + completion_tokens;

        self.execute(
            r#"
            INSERT INTO telemetry_llm_requests (
                request_id, model_name, provider, task_id, agent_id,
                prompt_tokens, completion_tokens, total_tokens,
                response_time_ms, success, error_message, cost_usd, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
            &[
                &request_id,
                &model_name.to_string(),
                &provider.to_string(),
                &task_id,
                &agent_id,
                &prompt_tokens,
                &completion_tokens,
                &total_tokens,
                &response_time_ms,
                &success,
                &error_message.map(|s| s.to_string()),
                &cost_usd.unwrap_or(0.0),
                &metadata.unwrap_or(serde_json::json!({})),
            ],
        )
        .await?;

        Ok(request_id)
    }

    /// Record agent activity for telemetry
    pub async fn record_agent_activity(
        &self,
        agent_id: Uuid,
        activity_type: &str,
        task_id: Option<Uuid>,
        duration_ms: Option<i32>,
        success: bool,
        error_message: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) -> Result<Uuid> {
        let id = Uuid::new_v4();

        self.execute(
            r#"
            INSERT INTO telemetry_agent_activity (
                id, agent_id, activity_type, task_id, duration_ms,
                success, error_message, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            &[
                &id,
                &agent_id,
                &activity_type.to_string(),
                &task_id,
                &duration_ms,
                &success,
                &error_message.map(|s| s.to_string()),
                &metadata.unwrap_or(serde_json::json!({})),
            ],
        )
        .await?;

        Ok(id)
    }

    /// Trigger a task stats snapshot
    pub async fn snapshot_task_stats(&self) -> Result<()> {
        self.execute("SELECT snapshot_task_stats()", &[]).await?;
        Ok(())
    }

    /// Check if a snapshot has been taken today
    pub async fn has_snapshot_today(&self) -> Result<bool> {
        let row = self
            .query_one(
                "SELECT EXISTS(SELECT 1 FROM task_stats_history WHERE snapshot_date = CURRENT_DATE) as exists",
                &[],
            )
            .await?;

        match row {
            Some(r) => Ok(r.try_get::<bool, _>("exists").unwrap_or(false)),
            None => Ok(false),
        }
    }

    /// Revoke a waiver
    pub async fn revoke_waiver(
        &self,
        waiver_id: &Uuid,
        revoked_by: &str,
        revocation_reason: &str,
    ) -> Result<()> {
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
    pub async fn acknowledge_slo_alert(
        &self,
        alert_id: &Uuid,
        acknowledged_by: &str,
        acknowledgment_notes: &str,
    ) -> Result<()> {
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
            None => Ok(None),
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
    pub async fn link_provenance_to_commit(
        &self,
        provenance_id: &Uuid,
        commit_hash: &str,
    ) -> Result<()> {
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
    pub async fn get_provenance_by_commit(
        &self,
        commit_hash: &str,
    ) -> Result<Vec<crate::models::ProvenanceEntry>> {
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
        let task_count = self
            .query_one("SELECT COUNT(*) as count FROM tasks", &[])
            .await?;

        let active_task_count = self
            .query_one(
                "SELECT COUNT(*) as count FROM tasks WHERE status = 'running'",
                &[],
            )
            .await?;

        let waiver_count = self
            .query_one("SELECT COUNT(*) as count FROM waivers", &[])
            .await?;

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

        let recent_tasks = self
            .query(
                "SELECT id, title, status, created_at FROM tasks ORDER BY created_at DESC LIMIT 10",
                &[],
            )
            .await?;

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
            "SELECT id, title, description, risk_tier, scope, acceptance_criteria, context, caws_spec, status, assigned_worker_id, project_id, priority, deadline, metadata, created_at, updated_at, completed_at FROM tasks ORDER BY created_at DESC",
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
                project_id: row.try_get("project_id")?,
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

    // User operations
    pub async fn get_user(&self, id: Uuid) -> Result<Option<crate::models::User>> {
        self.inner.get_user(id).await
    }

    pub async fn get_user_by_username(
        &self,
        username: &str,
    ) -> Result<Option<crate::models::User>> {
        self.inner.get_user_by_username(username).await
    }

    pub async fn update_user(
        &self,
        id: Uuid,
        update: crate::database_operations::UpdateUser,
    ) -> Result<crate::models::User> {
        self.inner.update_user(id, update).await
    }

    // User settings operations
    pub async fn get_user_settings(
        &self,
        user_id: Uuid,
        setting_type: Option<&str>,
    ) -> Result<Vec<crate::models::UserSetting>> {
        self.inner.get_user_settings(user_id, setting_type).await
    }

    pub async fn get_user_setting(
        &self,
        user_id: Uuid,
        setting_key: &str,
    ) -> Result<Option<crate::models::UserSetting>> {
        self.inner.get_user_setting(user_id, setting_key).await
    }

    pub async fn create_user_setting(
        &self,
        setting: crate::database_operations::CreateUserSetting,
    ) -> Result<crate::models::UserSetting> {
        self.inner.create_user_setting(setting).await
    }

    pub async fn update_user_setting(
        &self,
        user_id: Uuid,
        setting_key: &str,
        update: crate::database_operations::UpdateUserSetting,
    ) -> Result<crate::models::UserSetting> {
        self.inner
            .update_user_setting(user_id, setting_key, update)
            .await
    }

    pub async fn delete_user_setting(&self, user_id: Uuid, setting_key: &str) -> Result<()> {
        self.inner.delete_user_setting(user_id, setting_key).await
    }

    // App settings operations
    pub async fn get_app_settings(
        &self,
        setting_type: Option<&str>,
        is_public: Option<bool>,
    ) -> Result<Vec<crate::models::AppSetting>> {
        self.inner.get_app_settings(setting_type, is_public).await
    }

    pub async fn get_app_setting(
        &self,
        setting_key: &str,
    ) -> Result<Option<crate::models::AppSetting>> {
        self.inner.get_app_setting(setting_key).await
    }

    pub async fn create_app_setting(
        &self,
        setting: crate::database_operations::CreateAppSetting,
    ) -> Result<crate::models::AppSetting> {
        self.inner.create_app_setting(setting).await
    }

    pub async fn update_app_setting(
        &self,
        setting_key: &str,
        update: crate::database_operations::UpdateAppSetting,
    ) -> Result<crate::models::AppSetting> {
        self.inner.update_app_setting(setting_key, update).await
    }

    pub async fn delete_app_setting(&self, setting_key: &str) -> Result<()> {
        self.inner.delete_app_setting(setting_key).await
    }

    // Integration operations
    pub async fn get_integrations(
        &self,
        provider: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<Vec<crate::models::Integration>> {
        self.inner.get_integrations(provider, is_active).await
    }

    pub async fn get_integration(&self, id: Uuid) -> Result<Option<crate::models::Integration>> {
        self.inner.get_integration(id).await
    }

    pub async fn create_integration(
        &self,
        integration: crate::database_operations::CreateIntegration,
    ) -> Result<crate::models::Integration> {
        self.inner.create_integration(integration).await
    }

    pub async fn update_integration(
        &self,
        id: Uuid,
        update: crate::database_operations::UpdateIntegration,
    ) -> Result<crate::models::Integration> {
        self.inner.update_integration(id, update).await
    }

    pub async fn delete_integration(&self, id: Uuid) -> Result<()> {
        self.inner.delete_integration(id).await
    }

    // API key operations
    pub async fn get_user_api_keys(
        &self,
        user_id: Uuid,
        is_active: Option<bool>,
    ) -> Result<Vec<crate::models::ApiKey>> {
        self.inner.get_user_api_keys(user_id, is_active).await
    }

    pub async fn get_api_key(&self, id: Uuid) -> Result<Option<crate::models::ApiKey>> {
        self.inner.get_api_key(id).await
    }

    pub async fn get_api_key_by_hash(
        &self,
        key_hash: &str,
    ) -> Result<Option<crate::models::ApiKey>> {
        self.inner.get_api_key_by_hash(key_hash).await
    }

    pub async fn create_api_key(
        &self,
        api_key: crate::database_operations::CreateApiKey,
    ) -> Result<crate::models::ApiKey> {
        self.inner.create_api_key(api_key).await
    }

    pub async fn update_api_key(
        &self,
        id: Uuid,
        update: crate::database_operations::UpdateApiKey,
    ) -> Result<crate::models::ApiKey> {
        self.inner.update_api_key(id, update).await
    }

    pub async fn revoke_api_key(&self, id: Uuid, reason: Option<String>) -> Result<()> {
        self.inner.revoke_api_key(id, reason).await
    }

    pub async fn delete_api_key(&self, id: Uuid) -> Result<()> {
        self.inner.delete_api_key(id).await
    }

    // CAWS Rules operations
    pub async fn create_caws_rule(
        &self,
        rule: crate::database_operations::CreateCawsRule,
    ) -> Result<crate::models::CawsRule> {
        self.inner.create_caws_rule(rule).await
    }

    pub async fn get_caws_rule(&self, id: &str) -> Result<Option<crate::models::CawsRule>> {
        self.inner.get_caws_rule(id).await
    }

    pub async fn get_caws_rules(
        &self,
        rule_type: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<Vec<crate::models::CawsRule>> {
        self.inner.get_caws_rules(rule_type, is_active).await
    }

    pub async fn update_caws_rule(
        &self,
        id: &str,
        update: crate::database_operations::UpdateCawsRule,
    ) -> Result<crate::models::CawsRule> {
        self.inner.update_caws_rule(id, update).await
    }

    pub async fn delete_caws_rule(&self, id: &str) -> Result<()> {
        self.inner.delete_caws_rule(id).await
    }

    // CAWS Violations operations
    pub async fn create_caws_violation(
        &self,
        violation: crate::database_operations::CreateCawsViolation,
    ) -> Result<crate::models::CawsViolation> {
        self.inner.create_caws_violation(violation).await
    }

    pub async fn get_caws_violation(
        &self,
        id: Uuid,
    ) -> Result<Option<crate::models::CawsViolation>> {
        self.inner.get_caws_violation(id).await
    }

    pub async fn get_caws_violations(
        &self,
        task_id: Option<Uuid>,
        rule_id: Option<&str>,
        status: Option<&str>,
    ) -> Result<Vec<crate::models::CawsViolation>> {
        self.inner
            .get_caws_violations(task_id, rule_id, status)
            .await
    }

    pub async fn update_caws_violation(
        &self,
        id: Uuid,
        update: crate::database_operations::UpdateCawsViolation,
    ) -> Result<crate::models::CawsViolation> {
        self.inner.update_caws_violation(id, update).await
    }

    pub async fn resolve_caws_violation(&self, id: Uuid) -> Result<()> {
        self.inner.resolve_caws_violation(id).await
    }

    // CAWS Specifications operations
    pub async fn create_caws_specification(
        &self,
        spec: crate::database_operations::CreateCawsSpecification,
    ) -> Result<crate::models::CawsSpecification> {
        self.inner.create_caws_specification(spec).await
    }

    pub async fn get_caws_specification(
        &self,
        id: Uuid,
    ) -> Result<Option<crate::models::CawsSpecification>> {
        self.inner.get_caws_specification(id).await
    }

    pub async fn get_caws_specifications(
        &self,
        name: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<Vec<crate::models::CawsSpecification>> {
        self.inner.get_caws_specifications(name, is_active).await
    }

    pub async fn update_caws_specification(
        &self,
        id: Uuid,
        update: crate::database_operations::UpdateCawsSpecification,
    ) -> Result<crate::models::CawsSpecification> {
        self.inner.update_caws_specification(id, update).await
    }

    pub async fn delete_caws_specification(&self, id: Uuid) -> Result<()> {
        self.inner.delete_caws_specification(id).await
    }

    // Rule templates operations
    pub async fn get_rule_templates(
        &self,
        rule_type: Option<&str>,
    ) -> Result<Vec<crate::database_operations::RuleTemplate>> {
        self.inner.get_rule_templates(rule_type).await
    }

    pub async fn create_rule_template(
        &self,
        template: crate::database_operations::CreateRuleTemplate,
    ) -> Result<crate::database_operations::RuleTemplate> {
        self.inner.create_rule_template(template).await
    }

    // Rule enforcement status operations
    pub async fn get_rule_enforcement_status(
        &self,
        rule_id: Option<&str>,
        task_id: Option<Uuid>,
    ) -> Result<Vec<crate::database_operations::RuleEnforcementStatus>> {
        self.inner
            .get_rule_enforcement_status(rule_id, task_id)
            .await
    }

    pub async fn update_rule_enforcement_status(
        &self,
        rule_id: &str,
        task_id: Option<Uuid>,
        status: crate::database_operations::UpdateRuleEnforcementStatus,
    ) -> Result<crate::database_operations::RuleEnforcementStatus> {
        self.inner
            .update_rule_enforcement_status(rule_id, task_id, status)
            .await
    }

    // Rule history operations
    pub async fn get_rule_history(
        &self,
        rule_id: &str,
        limit: Option<u32>,
    ) -> Result<Vec<crate::database_operations::RuleHistory>> {
        self.inner.get_rule_history(rule_id, limit).await
    }

    // Session operations
    pub async fn create_session(
        &self,
        session: crate::database_operations::CreateSession,
    ) -> Result<crate::models::Session> {
        self.inner.create_session(session).await
    }

    pub async fn get_session(&self, id: Uuid) -> Result<Option<crate::models::Session>> {
        self.inner.get_session(id).await
    }

    pub async fn get_session_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<crate::models::Session>> {
        self.inner.get_session_by_token_hash(token_hash).await
    }

    pub async fn get_session_by_refresh_token_hash(
        &self,
        refresh_token_hash: &str,
    ) -> Result<Option<crate::models::Session>> {
        self.inner.get_session_by_refresh_token_hash(refresh_token_hash).await
    }

    pub async fn update_session(
        &self,
        id: Uuid,
        update: crate::database_operations::UpdateSession,
    ) -> Result<crate::models::Session> {
        self.inner.update_session(id, update).await
    }

    // Password reset token operations
    pub async fn create_password_reset_token(
        &self,
        token: crate::database_operations::CreatePasswordResetToken,
    ) -> Result<crate::models::PasswordResetToken> {
        self.inner.create_password_reset_token(token).await
    }

    pub async fn get_password_reset_token(
        &self,
        token_hash: &str,
    ) -> Result<Option<crate::models::PasswordResetToken>> {
        self.inner.get_password_reset_token(token_hash).await
    }

    pub async fn mark_password_reset_token_used(&self, id: Uuid) -> Result<()> {
        self.inner.mark_password_reset_token_used(id).await
    }

    // Execution plan operations
    pub async fn create_execution_plan(
        &self,
        plan: crate::database_operations::CreateExecutionPlan,
    ) -> Result<crate::models::ExecutionPlan> {
        self.inner.create_execution_plan(plan).await
    }

    pub async fn get_execution_plan(
        &self,
        id: Uuid,
    ) -> Result<Option<crate::models::ExecutionPlan>> {
        self.inner.get_execution_plan(id).await
    }

    pub async fn get_execution_plans(&self) -> Result<Vec<crate::models::ExecutionPlan>> {
        self.inner.get_execution_plans().await
    }

    pub async fn update_execution_plan(
        &self,
        id: Uuid,
        update: crate::database_operations::UpdateExecutionPlan,
    ) -> Result<crate::models::ExecutionPlan> {
        self.inner.update_execution_plan(id, update).await
    }

    pub async fn delete_execution_plan(&self, id: Uuid) -> Result<()> {
        self.inner.delete_execution_plan(id).await
    }

    // Milestone operations
    pub async fn create_milestone(
        &self,
        milestone: crate::database_operations::CreateMilestone,
    ) -> Result<crate::models::Milestone> {
        self.inner.create_milestone(milestone).await
    }

    pub async fn get_milestone(
        &self,
        plan_id: Uuid,
        milestone_id: String,
    ) -> Result<Option<crate::models::Milestone>> {
        self.inner.get_milestone(plan_id, milestone_id).await
    }

    pub async fn get_milestones(&self, plan_id: Uuid) -> Result<Vec<crate::models::Milestone>> {
        self.inner.get_milestones(plan_id).await
    }

    pub async fn update_milestone(
        &self,
        plan_id: Uuid,
        milestone_id: String,
        update: crate::database_operations::UpdateMilestone,
    ) -> Result<crate::models::Milestone> {
        self.inner
            .update_milestone(plan_id, milestone_id, update)
            .await
    }

    pub async fn delete_milestone(&self, plan_id: Uuid, milestone_id: String) -> Result<()> {
        self.inner.delete_milestone(plan_id, milestone_id).await
    }

    // Worker operations
    pub async fn create_worker(
        &self,
        worker: crate::database_operations::CreateWorker,
    ) -> Result<crate::models::Worker> {
        self.inner.create_worker(worker).await
    }

    pub async fn get_worker(&self, id: Uuid) -> Result<Option<crate::models::Worker>> {
        self.inner.get_worker(id).await
    }

    pub async fn get_workers(&self) -> Result<Vec<crate::models::Worker>> {
        self.inner.get_workers().await
    }

    pub async fn update_worker(
        &self,
        id: Uuid,
        update: crate::database_operations::UpdateWorker,
    ) -> Result<crate::models::Worker> {
        self.inner.update_worker(id, update).await
    }

    pub async fn delete_worker(&self, id: Uuid) -> Result<()> {
        self.inner.delete_worker(id).await
    }

    /// Get task executions by worker_id
    pub async fn get_task_executions_by_worker(
        &self,
        worker_id: Uuid,
    ) -> Result<Vec<crate::models::TaskExecution>> {
        let rows = self.query(
            "SELECT id, task_id, worker_id, execution_started_at, execution_completed_at, execution_time_ms, status, worker_output, self_assessment, metadata, error_message, tokens_used, created_at, updated_at, execution_metadata, result_data FROM task_executions WHERE worker_id = $1 ORDER BY execution_started_at DESC",
            &[&worker_id]
        ).await?;

        let mut executions = Vec::new();
        for row in rows {
            let execution = crate::models::TaskExecution {
                id: row.try_get("id")?,
                task_id: row.try_get("task_id")?,
                worker_id: row.try_get("worker_id")?,
                execution_started_at: row.try_get("execution_started_at")?,
                execution_completed_at: row.try_get("execution_completed_at")?,
                execution_time_ms: row.try_get("execution_time_ms")?,
                status: row.try_get("status")?,
                worker_output: row.try_get("worker_output")?,
                self_assessment: row.try_get("self_assessment")?,
                metadata: row.try_get("metadata")?,
                error_message: row.try_get("error_message")?,
                tokens_used: row.try_get("tokens_used")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
                execution_metadata: row.try_get("execution_metadata")?,
                result_data: row.try_get("result_data")?,
            };
            executions.push(execution);
        }

        Ok(executions)
    }

    // Judge operations
    pub async fn create_judge(
        &self,
        judge: crate::database_operations::CreateJudge,
    ) -> Result<crate::models::Judge> {
        self.inner.create_judge(judge).await
    }

    pub async fn get_judge(&self, id: Uuid) -> Result<Option<crate::models::Judge>> {
        self.inner.get_judge(id).await
    }

    pub async fn get_judges(&self) -> Result<Vec<crate::models::Judge>> {
        self.inner.get_judges().await
    }

    pub async fn update_judge(
        &self,
        id: Uuid,
        update: crate::database_operations::UpdateJudge,
    ) -> Result<crate::models::Judge> {
        self.inner.update_judge(id, update).await
    }

    pub async fn delete_judge(&self, id: Uuid) -> Result<()> {
        self.inner.delete_judge(id).await
    }

    // Two-factor authentication operations
    pub async fn get_two_factor_auth(
        &self,
        user_id: Uuid,
        method: Option<&str>,
    ) -> Result<Option<crate::models::TwoFactorAuth>> {
        self.inner.get_two_factor_auth(user_id, method).await
    }

    pub async fn create_two_factor_auth(
        &self,
        two_fa: crate::database_operations::CreateTwoFactorAuth,
    ) -> Result<crate::models::TwoFactorAuth> {
        self.inner.create_two_factor_auth(two_fa).await
    }

    pub async fn update_two_factor_auth(
        &self,
        user_id: Uuid,
        method: &str,
        update: crate::database_operations::UpdateTwoFactorAuth,
    ) -> Result<crate::models::TwoFactorAuth> {
        self.inner
            .update_two_factor_auth(user_id, method, update)
            .await
    }

    pub async fn delete_two_factor_auth(&self, user_id: Uuid, method: &str) -> Result<()> {
        self.inner.delete_two_factor_auth(user_id, method).await
    }

    /// Get judge evaluations by judge_id
    pub async fn get_judge_evaluations_by_judge(
        &self,
        judge_id: Uuid,
    ) -> Result<Vec<crate::models::JudgeEvaluation>> {
        let rows = self.query(
            "SELECT id, verdict_id, judge_id, judge_verdict, evaluation_time_ms, tokens_used, confidence, evaluation_score, confidence_score, reasoning, evidence_used, evaluation_metadata, verdict_decision, risk_assessment, created_at, updated_at FROM judge_evaluations WHERE judge_id = $1 ORDER BY created_at DESC",
            &[&judge_id]
        ).await?;

        let mut evaluations = Vec::new();
        for row in rows {
            let evaluation = crate::models::JudgeEvaluation {
                id: row.try_get("id")?,
                verdict_id: row.try_get("verdict_id")?,
                judge_id: row.try_get("judge_id")?,
                judge_verdict: row.try_get("judge_verdict")?,
                evaluation_time_ms: row.try_get("evaluation_time_ms")?,
                tokens_used: row.try_get("tokens_used")?,
                confidence: row.try_get("confidence")?,
                created_at: row.try_get("created_at")?,
                evaluation_score: row.try_get("evaluation_score")?,
                confidence_score: row.try_get("confidence_score")?,
                reasoning: row.try_get("reasoning")?,
                evidence_used: row.try_get("evidence_used")?,
                evaluation_metadata: row.try_get("evaluation_metadata")?,
                verdict_decision: row.try_get("verdict_decision")?,
                risk_assessment: row.try_get("risk_assessment")?,
                updated_at: row.try_get("updated_at")?,
            };
            evaluations.push(evaluation);
        }

        Ok(evaluations)
    }
}

/// Adapter for DatabaseClient to be used with agent-research DatabaseClientTrait
///
/// This adapter wraps DatabaseClient and provides the methods needed for
/// agent-research/src/self_prompting_agent/agent_caws_integration.rs::DatabaseClientTrait.
///
/// Usage in agent-research:
/// ```rust
/// use data_infrastructure::simple_client::ProvenanceClientAdapter;
/// use agent_research::self_prompting_agent::agent_caws_integration::DatabaseClientTrait;
///
/// let client = DatabaseClient::new(config).await?;
/// let adapter = ProvenanceClientAdapter::new(client);
/// // Agent-research can then implement DatabaseClientTrait for ProvenanceClientAdapter
/// ```
#[derive(Clone, Debug, JsonSchema)]
pub struct ProvenanceClientAdapter {
    client: Arc<DatabaseClient>,
}

impl ProvenanceClientAdapter {
    /// Create a new provenance client adapter
    pub fn new(client: DatabaseClient) -> Self {
        Self {
            client: Arc::new(client),
        }
    }

    /// Create a new provenance client adapter from Arc
    pub fn from_arc(client: Arc<DatabaseClient>) -> Self {
        Self { client }
    }

    /// Get a reference to the underlying database client
    pub fn client(&self) -> &DatabaseClient {
        &self.client
    }

    /// Create a provenance entry (for use by agent-research trait implementation)
    ///
    /// This method matches the signature expected by DatabaseClientTrait,
    /// converting anyhow::Result to Box<dyn Error + Send + Sync>.
    ///
    /// Uses the real DatabaseClient.create_provenance_entry implementation,
    /// which performs actual database insertion with proper error handling.
    pub async fn create_provenance_entry(
        &self,
        task_id: Uuid,
        action: String,
        actor: String,
        change_summary: String,
        resource_id: Option<Uuid>,
        resource_type: Option<String>,
        metadata: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Use the real DatabaseClient implementation
        self.client
            .create_provenance_entry(
                task_id,
                action,
                actor,
                change_summary,
                resource_id,
                resource_type,
                metadata,
            )
            .await
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            })?;

        Ok(())
    }
}
