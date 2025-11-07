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
        // PLACEHOLDER: Store execution plan in database
        // TODO: Implement execution_plans table insert
        // For now, return a model without persisting
        warn!("create_execution_plan() not yet implemented - plan not persisted");
        Ok(models::ExecutionPlan {
            id: plan.id,
            title: plan.title,
            overview: plan.overview,
        })
    }

    async fn get_execution_plan(&self, id: Uuid) -> Result<Option<models::ExecutionPlan>> {
        // PLACEHOLDER: Query execution plan from database
        // TODO: Implement execution_plans table query
        warn!("get_execution_plan() not yet implemented - returning None");
        Ok(None)
    }

    async fn get_execution_plans(&self) -> Result<Vec<models::ExecutionPlan>> {
        // PLACEHOLDER: Query all execution plans from database
        // TODO: Implement execution_plans table query
        warn!("get_execution_plans() not yet implemented - returning empty list");
        Ok(vec![])
    }

    async fn update_execution_plan(&self, id: Uuid, update: UpdateExecutionPlan) -> Result<models::ExecutionPlan> {
        // PLACEHOLDER: Update execution plan in database
        // TODO: Implement execution_plans table update
        warn!("update_execution_plan() not yet implemented");
        Err(anyhow!("update_execution_plan not yet implemented"))
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
        // PLACEHOLDER: Store planning session in database
        // TODO: Implement planning_sessions table insert
        warn!("create_planning_session() not yet implemented - session not persisted");
        Ok(models::PlanningSession {
            id: Uuid::new_v4(),
            plan_id: session.plan_id,
            status: "active".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: session.metadata,
        })
    }

    async fn get_planning_session(&self, id: Uuid) -> Result<Option<models::PlanningSession>> {
        // PLACEHOLDER: Query planning session from database
        // TODO: Implement planning_sessions table query
        warn!("get_planning_session() not yet implemented - returning None");
        Ok(None)
    }

    async fn update_planning_session(&self, id: Uuid, session: UpdatePlanningSession) -> Result<()> {
        // PLACEHOLDER: Update planning session in database
        // TODO: Implement planning_sessions table update
        warn!("update_planning_session() not yet implemented");
        Ok(())
    }

    async fn create_planning_telemetry(&self, telemetry: CreatePlanningTelemetry) -> Result<models::PlanningTelemetry> {
        // PLACEHOLDER: Store planning telemetry in database
        // TODO: Implement planning_telemetry table insert
        warn!("create_planning_telemetry() not yet implemented - telemetry not persisted");
        Ok(models::PlanningTelemetry {
            id: Uuid::new_v4(),
            session_id: telemetry.session_id,
            metric_name: telemetry.metric_name,
            metric_value: telemetry.metric_value,
            timestamp: Utc::now(),
            metadata: telemetry.metadata,
        })
    }

    async fn get_planning_telemetry(&self, plan_id: Uuid, metric_type: Option<String>) -> Result<Vec<models::PlanningTelemetry>> {
        // PLACEHOLDER: Query planning telemetry from database
        // TODO: Implement planning_telemetry table query filtered by plan_id and metric_type
        warn!("get_planning_telemetry() not yet implemented - returning empty list");
        Ok(vec![])
    }

    async fn create_planning_audit_event(&self, event: CreatePlanningAuditEvent) -> Result<()> {
        // PLACEHOLDER: Store planning audit event in database
        // TODO: Implement planning_audit_events table insert
        warn!("create_planning_audit_event() not yet implemented - event not persisted");
        Ok(())
    }

    async fn get_planning_audit_events(&self, plan_id: Uuid) -> Result<Vec<models::PlanningAuditEvent>> {
        // PLACEHOLDER: Query planning audit events from database
        // TODO: Implement planning_audit_events table query filtered by plan_id
        warn!("get_planning_audit_events() not yet implemented - returning empty list");
        Ok(vec![])
    }

    async fn delete_execution_plan(&self, id: Uuid) -> Result<()> {
        // PLACEHOLDER: Delete execution plan from database
        // TODO: Implement execution_plans table delete
        warn!("delete_execution_plan() not yet implemented");
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

