//! Database Operations Adapter
//!
//! Adapts data-infrastructure DatabaseClient to agent-orchestration DatabaseOperations trait.
//! Maps between agent-orchestration types and data-infrastructure database types.
//!
//! @author @darianrosebrook

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::{anyhow, Result};
use uuid::Uuid;
use chrono::Utc;
use tracing::{warn, info};
use sqlx;

use agent_orchestration::planning::data_infrastructure_types::{
    DatabaseOperations, CreateExecutionPlan, UpdateExecutionPlan,
    CreateAuditTrailEntry, CreatePlanningSession, UpdatePlanningSession,
    CreatePlanningTelemetry, CreatePlanningAuditEvent,
    CreateJudge, CreateJudgeEvaluation, CreateWaiver, UpdateWaiver,
    models,
};
use data_infrastructure::DatabaseClient;

/// Adapter that bridges data-infrastructure DatabaseClient to agent-orchestration DatabaseOperations
pub struct DatabaseOperationsAdapter {
    db_client: Arc<DatabaseClient>,
}

impl DatabaseOperationsAdapter {
    /// Create a new database operations adapter
    pub fn new(db_client: Arc<DatabaseClient>) -> Self {
        Self { db_client }
    }
}

#[async_trait]
impl DatabaseOperations for DatabaseOperationsAdapter {
    async fn get_workers(&self) -> Result<Vec<models::Worker>> {
        // PLACEHOLDER: Query workers from database
        // TODO: Implement worker table query
        // For now, return empty list - workers are managed by MCPWorkerPool
        warn!("get_workers() not yet implemented - returning empty list");
        Ok(vec![])
    }

    async fn create_execution_plan(&self, plan: CreateExecutionPlan) -> Result<models::ExecutionPlan> {
        let pool = self.db_client.pool();
        let now = Utc::now();
        
        // Insert execution plan into database
        sqlx::query(
            r#"
            INSERT INTO execution_plans (
                id, session_id, working_spec_id, title, overview, state,
                milestones, dependency_graph, change_budget, quality_gates,
                evidence_requirements, active_waivers, metadata, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#
        )
        .bind(plan.id)
        .bind(Uuid::new_v4()) // session_id - generate new session
        .bind(format!("PLAN-{}", plan.id)) // working_spec_id - derive from plan id
        .bind(&plan.title)
        .bind(&plan.overview)
        .bind("draft") // state
        .bind(serde_json::json!([])) // milestones - empty array
        .bind(serde_json::json!({})) // dependency_graph - empty object
        .bind(serde_json::json!({})) // change_budget - empty object
        .bind(serde_json::json!({})) // quality_gates - empty object
        .bind(serde_json::json!([])) // evidence_requirements - empty array
        .bind(serde_json::json!([])) // active_waivers - empty array
        .bind(serde_json::json!({})) // metadata - empty object
        .bind(now)
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to persist execution plan: {}", e))?;
        
        info!("Persisted execution plan {} to database", plan.id);
        
        Ok(models::ExecutionPlan {
            id: plan.id,
            title: plan.title,
            overview: plan.overview,
        })
    }

    async fn get_execution_plan(&self, id: Uuid) -> Result<Option<models::ExecutionPlan>> {
        let pool = self.db_client.pool();
        
        // Query execution plan from database
        let row = sqlx::query!(
            r#"
            SELECT id, title, overview, state
            FROM execution_plans
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| anyhow!("Failed to query execution plan: {}", e))?;
        
        Ok(row.map(|r| models::ExecutionPlan {
            id: r.id,
            title: r.title,
            overview: r.overview.unwrap_or_default(),
        }))
    }

    async fn get_execution_plans(&self) -> Result<Vec<models::ExecutionPlan>> {
        let pool = self.db_client.pool();
        
        // Query all execution plans from database
        let rows = sqlx::query!(
            r#"
            SELECT id, title, overview, state
            FROM execution_plans
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow!("Failed to query execution plans: {}", e))?;
        
        Ok(rows.into_iter().map(|r| models::ExecutionPlan {
            id: r.id,
            title: r.title,
            overview: r.overview.unwrap_or_default(),
        }).collect())
    }

    async fn update_execution_plan(&self, id: Uuid, update: UpdateExecutionPlan) -> Result<models::ExecutionPlan> {
        let pool = self.db_client.pool();
        
        // Build update query dynamically based on provided fields
        let mut updates = Vec::new();
        let mut bind_index = 1;
        
        if let Some(ref title) = update.title {
            updates.push(format!("title = ${}", bind_index));
            bind_index += 1;
        }
        if let Some(ref overview) = update.overview {
            updates.push(format!("overview = ${}", bind_index));
            bind_index += 1;
        }
        if let Some(ref status) = update.status {
            updates.push(format!("state = ${}", bind_index));
            bind_index += 1;
        }
        
        if updates.is_empty() {
            // No updates provided, just return existing plan
            return self.get_execution_plan(id).await?
                .ok_or_else(|| anyhow!("Execution plan {} not found", id));
        }
        
        updates.push(format!("updated_at = ${}", bind_index));
        bind_index += 1;
        
        let query = format!(
            "UPDATE execution_plans SET {} WHERE id = ${}",
            updates.join(", "),
            bind_index
        );
        
        // Build query with bindings
        let mut query_builder = sqlx::query(&query);
        if let Some(ref title) = update.title {
            query_builder = query_builder.bind(title);
        }
        if let Some(ref overview) = update.overview {
            query_builder = query_builder.bind(overview);
        }
        if let Some(ref status) = update.status {
            query_builder = query_builder.bind(status);
        }
        query_builder = query_builder.bind(Utc::now());
        query_builder = query_builder.bind(id);
        
        query_builder.execute(pool)
            .await
            .map_err(|e| anyhow!("Failed to update execution plan: {}", e))?;
        
        // Retrieve updated plan
        self.get_execution_plan(id).await?
            .ok_or_else(|| anyhow!("Execution plan {} not found after update", id))
    }

    async fn create_audit_trail_entry(&self, entry: CreateAuditTrailEntry) -> Result<models::AuditTrailEntry> {
        // Convert agent-orchestration CreateAuditTrailEntry to data-infrastructure format
        // Extract task_id from metadata if available, otherwise use a default
        let task_id = entry.metadata.get("task_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or_else(|| Uuid::new_v4());

        // Convert to data-infrastructure format
        let db_entry = data_infrastructure::database_operations::CreateAuditTrailEntry {
            entity_type: "task".to_string(),
            entity_id: task_id,
            action: entry.event_type.clone(),
            details: serde_json::json!({
                "description": entry.description,
                "metadata": entry.metadata,
            }),
            user_id: entry.metadata.get("user_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            ip_address: entry.metadata.get("ip_address")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            timestamp: Some(Utc::now()),
        };

        // Use DatabaseClient's pool to execute SQL directly
        let pool = self.db_client.pool();
        let id = Uuid::new_v4();
        let timestamp = db_entry.timestamp.unwrap_or_else(|| Utc::now());
        
        // Insert audit trail entry
        sqlx::query(
            r#"
            INSERT INTO audit_trail_entries (
                id, entity_type, entity_id, action, details,
                user_id, ip_address, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#
        )
        .bind(id)
        .bind(&db_entry.entity_type)
        .bind(db_entry.entity_id)
        .bind(&db_entry.action)
        .bind(&db_entry.details)
        .bind(&db_entry.user_id)
        .bind(&db_entry.ip_address)
        .bind(timestamp)
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to persist audit trail entry: {}", e))?;
        
        // Retrieve the persisted entry
        let db_result = sqlx::query_as::<_, data_infrastructure::models::AuditTrailEntry>(
            r#"
            SELECT id, entity_type, entity_id, action, details, user_id, ip_address, created_at
            FROM audit_trail_entries
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow!("Failed to retrieve persisted audit trail entry: {}", e))?;

        // Convert back to agent-orchestration format
        Ok(models::AuditTrailEntry {
            id: db_result.id,
            event_type: db_result.action,
            description: db_result.details.get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "".to_string()),
            timestamp: db_result.created_at,
            metadata: db_result.details.get("metadata")
                .and_then(|v| v.as_object())
                .map(|obj| {
                    obj.iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect()
                })
                .unwrap_or_default(),
        })
    }

    async fn get_audit_trail_entries(&self, task_id: Uuid) -> Result<Vec<models::AuditTrailEntry>> {
        // Query audit trail entries directly via SQL
        let pool = self.db_client.pool();
        let db_results = sqlx::query_as::<_, data_infrastructure::models::AuditTrailEntry>(
            r#"
            SELECT id, entity_type, entity_id, action, details, user_id, ip_address, created_at
            FROM audit_trail_entries
            WHERE entity_id = $1 AND entity_type = 'task'
            ORDER BY created_at DESC
            "#
        )
        .bind(task_id)
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow!("Failed to query audit trail entries: {}", e))?;

        // Convert to agent-orchestration format
        Ok(db_results.into_iter().map(|db_entry| {
            models::AuditTrailEntry {
                id: db_entry.id,
                event_type: db_entry.action,
                description: db_entry.details.get("description")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "".to_string()),
                timestamp: db_entry.created_at,
                metadata: db_entry.details.get("metadata")
                    .and_then(|v| v.as_object())
                    .map(|obj| {
                        obj.iter()
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect()
                    })
                    .unwrap_or_default(),
            }
        }).collect())
    }

    async fn get_audit_trail_entry(&self, id: Uuid) -> Result<Option<models::AuditTrailEntry>> {
        // Query audit trail entry directly via SQL
        let pool = self.db_client.pool();
        let db_result = sqlx::query_as::<_, data_infrastructure::models::AuditTrailEntry>(
            r#"
            SELECT id, entity_type, entity_id, action, details, user_id, ip_address, created_at
            FROM audit_trail_entries
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| anyhow!("Failed to query audit trail entry: {}", e))?;

        // Convert to agent-orchestration format
        Ok(db_result.map(|db_entry| {
            models::AuditTrailEntry {
                id: db_entry.id,
                event_type: db_entry.action,
                description: db_entry.details.get("description")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "".to_string()),
                timestamp: db_entry.created_at,
                metadata: db_entry.details.get("metadata")
                    .and_then(|v| v.as_object())
                    .map(|obj| {
                        obj.iter()
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect()
                    })
                    .unwrap_or_default(),
            }
        }))
    }

    async fn create_planning_session(&self, session: CreatePlanningSession) -> Result<models::PlanningSession> {
        let pool = self.db_client.pool();
        let session_id = Uuid::new_v4();
        let now = Utc::now();
        
        // Insert planning session into database
        sqlx::query(
            r#"
            INSERT INTO planning_sessions (
                id, plan_id, orchestrator_id, worker_pool_id, council_session_id,
                audit_correlation_id, status, execution_state, started_at, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#
        )
        .bind(session_id)
        .bind(session.plan_id)
        .bind("unified_orchestrator") // orchestrator_id
        .bind("mcp_worker_pool") // worker_pool_id
        .bind::<Option<Uuid>>(None) // council_session_id - optional
        .bind(Uuid::new_v4()) // audit_correlation_id
        .bind("active") // status
        .bind(serde_json::json!({})) // execution_state - empty object
        .bind(now) // started_at
        .bind(now) // created_at
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to persist planning session: {}", e))?;
        
        info!("Persisted planning session {} to database", session_id);
        
        Ok(models::PlanningSession {
            id: session_id,
            plan_id: session.plan_id,
            status: "active".to_string(),
            created_at: now,
            updated_at: now,
            metadata: session.metadata,
        })
    }

    async fn get_planning_session(&self, id: Uuid) -> Result<Option<models::PlanningSession>> {
        let pool = self.db_client.pool();
        
        // Query planning session from database
        let row = sqlx::query!(
            r#"
            SELECT id, plan_id, status, started_at, created_at
            FROM planning_sessions
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| anyhow!("Failed to query planning session: {}", e))?;
        
        Ok(row.map(|r| models::PlanningSession {
            id: r.id,
            plan_id: r.plan_id,
            status: r.status,
            created_at: r.created_at,
            updated_at: r.started_at, // Use started_at as updated_at fallback
            metadata: std::collections::HashMap::new(), // Metadata not stored in this query
        }))
    }

    async fn update_planning_session(&self, id: Uuid, session: UpdatePlanningSession) -> Result<()> {
        let pool = self.db_client.pool();
        
        // Build update query dynamically
        let mut updates = Vec::new();
        let mut bind_index = 1;
        
        if let Some(ref status) = session.status {
            updates.push(format!("status = ${}", bind_index));
            bind_index += 1;
        }
        if let Some(ref metadata) = session.metadata {
            updates.push(format!("execution_state = ${}", bind_index));
            bind_index += 1;
        }
        
        if updates.is_empty() {
            // No updates provided
            return Ok(());
        }
        
        let query = format!(
            "UPDATE planning_sessions SET {} WHERE id = ${}",
            updates.join(", "),
            bind_index
        );
        
        // Build query with bindings
        let mut query_builder = sqlx::query(&query);
        if let Some(ref status) = session.status {
            query_builder = query_builder.bind(status);
        }
        if let Some(ref metadata) = session.metadata {
            // Store metadata in execution_state JSONB field
            query_builder = query_builder.bind(serde_json::json!(metadata));
        }
        query_builder = query_builder.bind(id);
        
        query_builder.execute(pool)
            .await
            .map_err(|e| anyhow!("Failed to update planning session: {}", e))?;
        
        Ok(())
    }

    async fn create_planning_telemetry(&self, telemetry: CreatePlanningTelemetry) -> Result<models::PlanningTelemetry> {
        let pool = self.db_client.pool();
        let telemetry_id = Uuid::new_v4();
        let now = Utc::now();
        
        // Extract plan_id from session_id (assuming session_id maps to plan_id)
        // For now, use session_id as plan_id
        let plan_id = telemetry.session_id;
        
        // Insert planning telemetry into database
        sqlx::query(
            r#"
            INSERT INTO planning_telemetry (
                id, plan_id, metric_type, metric_value, collected_at, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6)
            "#
        )
        .bind(telemetry_id)
        .bind(plan_id)
        .bind(&telemetry.metric_name)
        .bind(serde_json::json!(telemetry.metric_value)) // Store as JSONB
        .bind(now)
        .bind(&telemetry.metadata)
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to persist planning telemetry: {}", e))?;
        
        info!("Persisted planning telemetry {} to database", telemetry_id);
        
        Ok(models::PlanningTelemetry {
            id: telemetry_id,
            session_id: telemetry.session_id,
            metric_name: telemetry.metric_name,
            metric_value: telemetry.metric_value,
            timestamp: now,
            metadata: telemetry.metadata,
        })
    }

    async fn get_planning_telemetry(&self, plan_id: Uuid, metric_type: Option<String>) -> Result<Vec<models::PlanningTelemetry>> {
        let pool = self.db_client.pool();
        
        // Query planning telemetry from database
        let rows = if let Some(ref metric_type) = metric_type {
            sqlx::query!(
                r#"
                SELECT id, plan_id, metric_type, metric_value, collected_at, metadata
                FROM planning_telemetry
                WHERE plan_id = $1 AND metric_type = $2
                ORDER BY collected_at DESC
                "#,
                plan_id,
                metric_type
            )
            .fetch_all(pool)
            .await
        } else {
            sqlx::query!(
                r#"
                SELECT id, plan_id, metric_type, metric_value, collected_at, metadata
                FROM planning_telemetry
                WHERE plan_id = $1
                ORDER BY collected_at DESC
                "#,
                plan_id
            )
            .fetch_all(pool)
            .await
        }
        .map_err(|e| anyhow!("Failed to query planning telemetry: {}", e))?;
        
        Ok(rows.into_iter().map(|r| {
            // Extract metric_value from JSONB
            let metric_value = r.metric_value
                .as_f64()
                .or_else(|| r.metric_value.as_i64().map(|v| v as f64))
                .unwrap_or(0.0);
            
            models::PlanningTelemetry {
                id: r.id,
                session_id: r.plan_id, // Use plan_id as session_id
                metric_name: r.metric_type,
                metric_value,
                timestamp: r.collected_at,
                metadata: r.metadata.as_object()
                    .map(|obj| {
                        obj.iter()
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect()
                    })
                    .unwrap_or_default(),
            }
        }).collect())
    }

    async fn create_planning_audit_event(&self, event: CreatePlanningAuditEvent) -> Result<()> {
        let pool = self.db_client.pool();
        let event_id = Uuid::new_v4();
        let now = Utc::now();
        
        // Extract optional fields from metadata
        let milestone_id = event.metadata.get("milestone_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let worker_id = event.metadata.get("worker_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok());
        
        // Insert planning audit event into database
        sqlx::query(
            r#"
            INSERT INTO planning_audit_events (
                id, plan_id, milestone_id, worker_id, event_type, description, metadata, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#
        )
        .bind(event_id)
        .bind(event.plan_id)
        .bind(milestone_id.as_deref())
        .bind(worker_id)
        .bind(&event.event_type)
        .bind(&event.description)
        .bind(&event.metadata)
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to persist planning audit event: {}", e))?;
        
        info!("Persisted planning audit event {} to database", event_id);
        
        Ok(())
    }

    async fn get_planning_audit_events(&self, plan_id: Uuid) -> Result<Vec<models::PlanningAuditEvent>> {
        let pool = self.db_client.pool();
        
        // Query planning audit events from database
        let rows = sqlx::query!(
            r#"
            SELECT id, plan_id, milestone_id, worker_id, event_type, description, metadata, created_at
            FROM planning_audit_events
            WHERE plan_id = $1
            ORDER BY created_at DESC
            "#,
            plan_id
        )
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow!("Failed to query planning audit events: {}", e))?;
        
        Ok(rows.into_iter().map(|r| models::PlanningAuditEvent {
            id: r.id,
            session_id: r.plan_id, // Use plan_id as session_id
            event_type: r.event_type,
            description: r.description,
            timestamp: r.created_at,
            metadata: r.metadata.as_object()
                .map(|obj| {
                    obj.iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect()
                })
                .unwrap_or_default(),
        }).collect())
    }

    async fn delete_execution_plan(&self, id: Uuid) -> Result<()> {
        let pool = self.db_client.pool();
        
        // Delete execution plan from database (cascade will delete related records)
        let rows_affected = sqlx::query(
            r#"
            DELETE FROM execution_plans
            WHERE id = $1
            "#
        )
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to delete execution plan: {}", e))?
        .rows_affected();
        
        if rows_affected > 0 {
            info!("Deleted execution plan {} from database", id);
        } else {
            warn!("Execution plan {} not found for deletion", id);
        }
        
        Ok(())
    }

    async fn get_judges(&self) -> Result<Vec<models::Judge>> {
        // PLACEHOLDER: Query judges from database
        // TODO: Implement judges table query
        // For now, return empty list - judges are configured in code
        warn!("get_judges() not yet implemented - returning empty list");
        Ok(vec![])
    }

    async fn create_judge(&self, judge: CreateJudge) -> Result<models::Judge> {
        // PLACEHOLDER: Store judge in database
        // TODO: Implement judges table insert
        warn!("create_judge() not yet implemented");
        Err(anyhow!("create_judge not yet implemented"))
    }

    async fn get_judge(&self, id: Uuid) -> Result<Option<models::Judge>> {
        // PLACEHOLDER: Query judge from database
        // TODO: Implement judges table query
        warn!("get_judge() not yet implemented - returning None");
        Ok(None)
    }

    async fn create_judge_evaluation(&self, evaluation: CreateJudgeEvaluation) -> Result<models::JudgeEvaluation> {
        // PLACEHOLDER: Store judge evaluation in database
        // TODO: Implement judge_evaluations table insert
        warn!("create_judge_evaluation() not yet implemented");
        Err(anyhow!("create_judge_evaluation not yet implemented"))
    }

    async fn get_judge_evaluations(&self, task_id: Uuid) -> Result<Vec<models::JudgeEvaluation>> {
        // PLACEHOLDER: Query judge evaluations from database
        // TODO: Implement judge_evaluations table query filtered by task_id
        warn!("get_judge_evaluations() not yet implemented - returning empty list");
        Ok(vec![])
    }

    async fn get_waivers(&self, status: Option<String>) -> Result<Vec<models::Waiver>> {
        // PLACEHOLDER: Query waivers from database
        // TODO: Implement waivers table query filtered by status
        warn!("get_waivers() not yet implemented - returning empty list");
        Ok(vec![])
    }

    async fn create_waiver(&self, waiver: CreateWaiver) -> Result<models::Waiver> {
        // PLACEHOLDER: Store waiver in database
        // TODO: Implement waivers table insert
        warn!("create_waiver() not yet implemented");
        Err(anyhow!("create_waiver not yet implemented"))
    }

    async fn update_waiver(&self, id: Uuid, update: UpdateWaiver) -> Result<models::Waiver> {
        // PLACEHOLDER: Update waiver in database
        // TODO: Implement waivers table update
        warn!("update_waiver() not yet implemented");
        Err(anyhow!("update_waiver not yet implemented"))
    }
}

