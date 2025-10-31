//! Database operations trait and types for consistent CRUD operations
//!
//! This module defines the DatabaseOperations trait and associated input/output types
//! for all database operations across the system. It provides a unified interface
//! for database clients to implement consistent CRUD operations.

use crate::models::*;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Database operations trait for consistent CRUD operations
#[async_trait]
pub trait DatabaseOperations {
    // Judge operations
    async fn create_judge(&self, judge: CreateJudge) -> Result<Judge>;
    async fn get_judge(&self, id: Uuid) -> Result<Option<Judge>>;
    async fn get_judges(&self) -> Result<Vec<Judge>>;
    async fn update_judge(&self, id: Uuid, update: UpdateJudge) -> Result<Judge>;
    async fn delete_judge(&self, id: Uuid) -> Result<()>;

    // Worker operations
    async fn create_worker(&self, worker: CreateWorker) -> Result<Worker>;
    async fn get_worker(&self, id: Uuid) -> Result<Option<Worker>>;
    async fn get_workers(&self) -> Result<Vec<Worker>>;
    async fn update_worker(&self, id: Uuid, update: UpdateWorker) -> Result<Worker>;
    async fn delete_worker(&self, id: Uuid) -> Result<()>;

    // Task operations
    async fn create_task(&self, task: CreateTask) -> Result<Task>;
    async fn get_task(&self, id: Uuid) -> Result<Option<Task>>;
    async fn get_tasks(&self) -> Result<Vec<Task>>;
    async fn update_task(&self, id: Uuid, update: UpdateTask) -> Result<Task>;
    async fn delete_task(&self, id: Uuid) -> Result<()>;

    // Task execution operations
    async fn create_task_execution(&self, execution: CreateTaskExecution) -> Result<TaskExecution>;
    async fn get_task_execution(&self, id: Uuid) -> Result<Option<TaskExecution>>;
    async fn get_task_executions(&self, task_id: Uuid) -> Result<Vec<TaskExecution>>;
    async fn update_task_execution(&self, id: Uuid, update: UpdateTaskExecution) -> Result<TaskExecution>;

    // Audit trail operations
    async fn create_audit_trail_entry(&self, entry: CreateAuditTrailEntry) -> Result<AuditTrailEntry>;
    async fn get_audit_trail_entries(&self, task_id: Uuid) -> Result<Vec<AuditTrailEntry>>;
    async fn get_audit_trail_entry(&self, id: Uuid) -> Result<Option<AuditTrailEntry>>;

    // Council verdict operations
    async fn create_council_verdict(&self, verdict: CreateCouncilVerdict) -> Result<CouncilVerdict>;
    async fn get_council_verdict(&self, id: Uuid) -> Result<Option<CouncilVerdict>>;
    async fn get_council_verdicts(&self, task_id: Uuid) -> Result<Vec<CouncilVerdict>>;

    // Judge evaluation operations
    async fn create_judge_evaluation(&self, evaluation: CreateJudgeEvaluation) -> Result<JudgeEvaluation>;
    async fn get_judge_evaluations(&self, task_id: Uuid) -> Result<Vec<JudgeEvaluation>>;

    // Planning operations
    async fn create_planning_telemetry(&self, telemetry: CreatePlanningTelemetry) -> Result<PlanningTelemetry>;
    async fn get_planning_telemetry(&self, plan_id: Uuid, metric_type: Option<String>) -> Result<Vec<PlanningTelemetry>>;
    async fn create_milestone(&self, milestone: CreateMilestone) -> Result<Milestone>;
    async fn get_milestone(&self, plan_id: Uuid, milestone_id: String) -> Result<Option<Milestone>>;
    async fn get_milestones(&self, plan_id: Uuid) -> Result<Vec<Milestone>>;
    async fn update_milestone(&self, plan_id: Uuid, milestone_id: String, update: UpdateMilestone) -> Result<Milestone>;
    async fn delete_milestone(&self, plan_id: Uuid, milestone_id: String) -> Result<()>;
    async fn create_planning_session(&self, session: CreatePlanningSession) -> Result<PlanningSession>;
    async fn get_planning_session(&self, id: Uuid) -> Result<Option<PlanningSession>>;
    async fn get_planning_sessions(&self, plan_id: Uuid) -> Result<Vec<PlanningSession>>;
    async fn update_planning_session(&self, id: Uuid, update: UpdatePlanningSession) -> Result<PlanningSession>;
    async fn create_evidence_artifact(&self, artifact: CreateEvidenceArtifact) -> Result<EvidenceArtifact>;
    async fn get_evidence_artifacts(&self, plan_id: Uuid) -> Result<Vec<EvidenceArtifact>>;
    async fn get_evidence_artifacts_for_milestone(&self, plan_id: Uuid, milestone_id: String) -> Result<Vec<EvidenceArtifact>>;
    async fn update_evidence_artifact(&self, id: Uuid, update: UpdateEvidenceArtifact) -> Result<EvidenceArtifact>;
    async fn create_planning_audit_event(&self, event: CreatePlanningAuditEvent) -> Result<PlanningAuditEvent>;
    async fn get_planning_audit_events(&self, plan_id: Uuid) -> Result<Vec<PlanningAuditEvent>>;
    async fn create_execution_plan(&self, plan: CreateExecutionPlan) -> Result<ExecutionPlan>;
    async fn get_execution_plan(&self, id: Uuid) -> Result<Option<ExecutionPlan>>;
    async fn get_execution_plans(&self) -> Result<Vec<ExecutionPlan>>;
    async fn update_execution_plan(&self, id: Uuid, update: UpdateExecutionPlan) -> Result<ExecutionPlan>;
    async fn delete_execution_plan(&self, id: Uuid) -> Result<()>;

    // Waiver operations
    async fn get_waivers(&self, status: Option<String>) -> Result<Vec<Waiver>>;
    async fn create_waiver(&self, waiver: CreateWaiver) -> Result<Waiver>;
    async fn update_waiver(&self, id: Uuid, update: UpdateWaiver) -> Result<Waiver>;
}

/// Input types for database operations

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJudge {
    pub name: String,
    pub model_name: String,
    pub endpoint: String,
    pub weight: f32,
    pub timeout_ms: i32,
    pub optimization_target: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateJudge {
    pub name: Option<String>,
    pub model_name: Option<String>,
    pub endpoint: Option<String>,
    pub weight: Option<f32>,
    pub timeout_ms: Option<i32>,
    pub optimization_target: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorker {
    pub name: String,
    pub worker_type: String,
    pub specialty: Option<String>,
    pub model_name: String,
    pub endpoint: String,
    pub capabilities: serde_json::Value,
    pub performance_history: serde_json::Value,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWorker {
    pub name: Option<String>,
    pub worker_type: Option<String>,
    pub specialty: Option<String>,
    pub model_name: Option<String>,
    pub endpoint: Option<String>,
    pub capabilities: Option<serde_json::Value>,
    pub performance_history: Option<serde_json::Value>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTask {
    pub title: String,
    pub description: String,
    pub risk_tier: String,
    pub scope: serde_json::Value,
    pub acceptance_criteria: serde_json::Value,
    pub context: serde_json::Value,
    pub caws_spec: Option<serde_json::Value>,
    pub status: String,
    pub assigned_worker_id: Option<Uuid>,
    pub priority: Option<i32>,
    pub deadline: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTask {
    pub title: Option<String>,
    pub description: Option<String>,
    pub risk_tier: Option<String>,
    pub scope: Option<serde_json::Value>,
    pub acceptance_criteria: Option<serde_json::Value>,
    pub context: Option<serde_json::Value>,
    pub caws_spec: Option<serde_json::Value>,
    pub status: Option<String>,
    pub assigned_worker_id: Option<Uuid>,
    pub priority: Option<i32>,
    pub deadline: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskExecution {
    pub task_id: Uuid,
    pub worker_id: Uuid,
    pub execution_started_at: DateTime<Utc>,
    pub status: String,
    pub worker_output: serde_json::Value,
    pub self_assessment: serde_json::Value,
    pub metadata: serde_json::Value,
    pub error_message: Option<String>,
    pub tokens_used: Option<i32>,
    pub execution_metadata: Option<serde_json::Value>,
    pub result_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskExecution {
    pub execution_completed_at: Option<DateTime<Utc>>,
    pub execution_time_ms: Option<i32>,
    pub status: Option<String>,
    pub worker_output: Option<serde_json::Value>,
    pub self_assessment: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub tokens_used: Option<i32>,
    pub execution_metadata: Option<serde_json::Value>,
    pub result_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAuditTrailEntry {
    pub entity_type: String,
    pub entity_id: Uuid,
    pub action: String,
    pub details: serde_json::Value,
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCouncilVerdict {
    pub task_id: Uuid,
    pub verdict_id: Uuid,
    pub consensus_score: f32,
    pub final_verdict: serde_json::Value,
    pub individual_verdicts: serde_json::Value,
    pub debate_rounds: i32,
    pub evaluation_time_ms: i32,
    pub contract: serde_json::Value,
    pub verdict_details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJudgeEvaluation {
    pub task_id: Uuid,
    pub judge_id: Uuid,
    pub evaluation_score: f32,
    pub evaluation_reasoning: String,
    pub evaluation_metadata: serde_json::Value,
    pub evaluation_time_ms: i32,
    pub evaluation_timestamp: DateTime<Utc>,
}

/// Planning telemetry input types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePlanningTelemetry {
    pub plan_id: Uuid,
    pub metric_type: String,
    pub metric_value: serde_json::Value,
    pub metadata: Option<serde_json::Value>,
    pub collected_at: Option<DateTime<Utc>>,
}

/// Milestone input types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMilestone {
    pub id: String,
    pub plan_id: Uuid,
    pub objective: String,
    pub scope: Option<serde_json::Value>,
    pub interfaces: Option<serde_json::Value>,
    pub tests: Option<serde_json::Value>,
    pub evidence_gate: Option<serde_json::Value>,
    pub rollback_plan: Option<String>,
    pub dependencies: Option<serde_json::Value>,
    pub state: Option<String>,
    pub assigned_worker_id: Option<Uuid>,
    pub estimated_effort: Option<f64>,
    pub priority: Option<String>,
    pub risk_tier: Option<i32>,
    pub is_blocking: Option<bool>,
    pub blocking_reason: Option<String>,
    pub metrics: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMilestone {
    pub objective: Option<String>,
    pub scope: Option<serde_json::Value>,
    pub interfaces: Option<serde_json::Value>,
    pub tests: Option<serde_json::Value>,
    pub evidence_gate: Option<serde_json::Value>,
    pub rollback_plan: Option<String>,
    pub dependencies: Option<serde_json::Value>,
    pub state: Option<String>,
    pub assigned_worker_id: Option<Uuid>,
    pub estimated_effort: Option<f64>,
    pub priority: Option<String>,
    pub risk_tier: Option<i32>,
    pub is_blocking: Option<bool>,
    pub blocking_reason: Option<String>,
    pub metrics: Option<serde_json::Value>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Planning session input types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePlanningSession {
    pub plan_id: Uuid,
    pub orchestrator_id: String,
    pub worker_pool_id: String,
    pub council_session_id: Option<Uuid>,
    pub audit_correlation_id: Uuid,
    pub status: Option<String>,
    pub execution_state: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePlanningSession {
    pub status: Option<String>,
    pub execution_state: Option<serde_json::Value>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Evidence artifact input types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEvidenceArtifact {
    pub milestone_id: String,
    pub plan_id: Uuid,
    pub artifact_type: String,
    pub artifact_data: serde_json::Value,
    pub verified: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEvidenceArtifact {
    pub artifact_type: Option<String>,
    pub artifact_data: Option<serde_json::Value>,
    pub verified: Option<bool>,
    pub verified_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
}

/// Planning audit event input types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePlanningAuditEvent {
    pub plan_id: Uuid,
    pub milestone_id: Option<String>,
    pub worker_id: Option<Uuid>,
    pub event_type: String,
    pub description: String,
    pub metadata: Option<serde_json::Value>,
}

/// Execution plan input types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateExecutionPlan {
    pub id: Uuid,
    pub session_id: Uuid,
    pub working_spec_id: String,
    pub title: String,
    pub overview: Option<String>,
    pub state: Option<String>,
    pub milestones: Option<serde_json::Value>,
    pub dependency_graph: Option<serde_json::Value>,
    pub change_budget: Option<serde_json::Value>,
    pub quality_gates: Option<serde_json::Value>,
    pub evidence_requirements: Option<serde_json::Value>,
    pub active_waivers: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateExecutionPlan {
    pub title: Option<String>,
    pub overview: Option<String>,
    pub state: Option<String>,
    pub milestones: Option<serde_json::Value>,
    pub dependency_graph: Option<serde_json::Value>,
    pub change_budget: Option<serde_json::Value>,
    pub quality_gates: Option<serde_json::Value>,
    pub evidence_requirements: Option<serde_json::Value>,
    pub active_waivers: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub approved_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Waiver input types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWaiver {
    pub title: String,
    pub reason: String,
    pub description: String,
    pub gates: Vec<String>,
    pub approved_by: String,
    pub impact_level: String,
    pub mitigation_plan: String,
    pub expires_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWaiver {
    pub title: Option<String>,
    pub description: Option<String>,
    pub mitigation_plan: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub status: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Factory function to create a database operations instance
/// 
/// This function creates a DatabaseClient and returns it as an Arc<dyn DatabaseOperations>
/// for dependency injection. The client uses the provided database configuration.
/// 
/// # Example
/// 
/// ```rust,ignore
/// use data_infrastructure::{create_database_operations, DatabaseConfig};
/// 
/// let config = DatabaseConfig {
///     database_url: "postgresql://localhost/test".to_string(),
///     ..Default::default()
/// };
/// 
/// let db_ops = create_database_operations(config).await?;
/// ```
pub async fn create_database_operations(
    config: crate::database_config::DatabaseConfig,
) -> Result<Arc<dyn DatabaseOperations + Send + Sync>> {
    use crate::client::orchestrator::DatabaseClient;
    
    let client = DatabaseClient::new(config).await?;
    Ok(Arc::new(client))
}

/// Adapter to implement DatabaseAuditOperations for DatabaseOperations
/// 
/// This allows DatabaseOperations implementations to be used where DatabaseAuditOperations
/// is required, breaking circular dependencies by using the interface from system-common-interfaces.
pub struct DatabaseAuditOperationsAdapter {
    db_ops: Arc<dyn DatabaseOperations + Send + Sync>,
}

impl DatabaseAuditOperationsAdapter {
    /// Create a new adapter wrapping a DatabaseOperations implementation
    pub fn new(db_ops: Arc<dyn DatabaseOperations + Send + Sync>) -> Self {
        Self { db_ops }
    }
}

#[async_trait]
impl system_common_interfaces::DatabaseAuditOperations for DatabaseAuditOperationsAdapter {
    async fn create_audit_entry(&self, entry: system_common_interfaces::CreateAuditEntry) -> system_common_interfaces::Result<()> {
        // Convert from system-common-interfaces type to data-infrastructure type
        let audit_entry = CreateAuditTrailEntry {
            entity_type: entry.entity_type,
            entity_id: entry.entity_id,
            action: entry.action,
            details: entry.details,
            user_id: entry.user_id,
            ip_address: entry.ip_address,
            timestamp: entry.timestamp,
        };
        
        // Call the underlying DatabaseOperations implementation
        self.db_ops.create_audit_trail_entry(audit_entry).await
            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())) as Box<dyn std::error::Error + Send + Sync>)?;
        
        Ok(())
    }
}

/// Factory function to create a DatabaseAuditOperations adapter
/// 
/// This wraps a DatabaseOperations implementation in an adapter that implements
/// DatabaseAuditOperations, allowing it to be injected into components that need
/// only audit functionality without creating circular dependencies.
/// 
/// # Example
/// 
/// ```rust,ignore
/// use data_infrastructure::{create_database_audit_operations, DatabaseConfig};
/// 
/// let config = DatabaseConfig {
///     database_url: "postgresql://localhost/test".to_string(),
///     ..Default::default()
/// };
/// 
/// let db_audit_ops = create_database_audit_operations(config).await?;
/// ```
pub async fn create_database_audit_operations(
    config: crate::database_config::DatabaseConfig,
) -> Result<Arc<dyn system_common_interfaces::DatabaseAuditOperations + Send + Sync>> {
    let db_ops = create_database_operations(config).await?;
    Ok(Arc::new(DatabaseAuditOperationsAdapter::new(db_ops)))
}
