//! Local type definitions for data infrastructure to avoid circular dependencies

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Audit trail entry
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AuditTrailEntry {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub event_type: String,
    pub description: String,
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Database operations trait
#[async_trait]
pub trait DatabaseOperations: Send + Sync {
    async fn create_execution_plan(
        &self,
        plan: CreateExecutionPlan,
    ) -> Result<models::ExecutionPlan, anyhow::Error>;
    async fn get_execution_plan(
        &self,
        id: Uuid,
    ) -> Result<Option<models::ExecutionPlan>, anyhow::Error>;
    async fn get_execution_plans(&self) -> Result<Vec<models::ExecutionPlan>, anyhow::Error>;
    async fn update_execution_plan(
        &self,
        id: Uuid,
        update: UpdateExecutionPlan,
    ) -> Result<models::ExecutionPlan, anyhow::Error>;
    async fn create_audit_trail_entry(
        &self,
        entry: CreateAuditTrailEntry,
    ) -> Result<models::AuditTrailEntry, anyhow::Error>;
    async fn get_audit_trail_entries(
        &self,
        task_id: Uuid,
    ) -> Result<Vec<models::AuditTrailEntry>, anyhow::Error>;
    async fn get_audit_trail_entry(
        &self,
        id: Uuid,
    ) -> Result<Option<models::AuditTrailEntry>, anyhow::Error>;
    async fn create_planning_session(
        &self,
        session: CreatePlanningSession,
    ) -> Result<models::PlanningSession, anyhow::Error>;
    async fn get_planning_session(
        &self,
        id: Uuid,
    ) -> Result<Option<models::PlanningSession>, anyhow::Error>;
    async fn update_planning_session(
        &self,
        id: Uuid,
        session: UpdatePlanningSession,
    ) -> Result<(), anyhow::Error>;
    async fn create_planning_telemetry(
        &self,
        telemetry: CreatePlanningTelemetry,
    ) -> Result<models::PlanningTelemetry, anyhow::Error>;
    async fn get_planning_telemetry(
        &self,
        plan_id: Uuid,
        metric_type: Option<String>,
    ) -> Result<Vec<models::PlanningTelemetry>, anyhow::Error>;
    async fn create_planning_audit_event(
        &self,
        event: CreatePlanningAuditEvent,
    ) -> Result<(), anyhow::Error>;
    async fn get_planning_audit_events(
        &self,
        plan_id: Uuid,
    ) -> Result<Vec<models::PlanningAuditEvent>, anyhow::Error>;
    async fn delete_execution_plan(&self, id: Uuid) -> Result<(), anyhow::Error>;
    async fn create_judge(&self, judge: CreateJudge) -> Result<models::Judge, anyhow::Error>;
    async fn get_judge(&self, id: Uuid) -> Result<Option<models::Judge>, anyhow::Error>;
    async fn get_judges(&self) -> Result<Vec<models::Judge>, anyhow::Error>;
    async fn create_judge_evaluation(
        &self,
        evaluation: CreateJudgeEvaluation,
    ) -> Result<models::JudgeEvaluation, anyhow::Error>;
    async fn get_judge_evaluations(
        &self,
        task_id: Uuid,
    ) -> Result<Vec<models::JudgeEvaluation>, anyhow::Error>;
    async fn get_workers(&self) -> Result<Vec<models::Worker>, anyhow::Error>;
    async fn get_worker(&self, id: Uuid) -> Result<Option<models::Worker>, anyhow::Error>;
    async fn create_worker(&self, worker: CreateWorker) -> Result<models::Worker, anyhow::Error>;
    async fn update_worker(
        &self,
        id: Uuid,
        update: UpdateWorker,
    ) -> Result<models::Worker, anyhow::Error>;
    async fn get_waivers(
        &self,
        status: Option<String>,
    ) -> Result<Vec<models::Waiver>, anyhow::Error>;
    async fn create_waiver(&self, waiver: CreateWaiver) -> Result<models::Waiver, anyhow::Error>;
    async fn update_waiver(
        &self,
        id: Uuid,
        update: UpdateWaiver,
    ) -> Result<models::Waiver, anyhow::Error>;
    async fn create_execution_result(
        &self,
        result: CreateExecutionResult,
    ) -> Result<models::PlanExecutionResult, anyhow::Error>;
    async fn get_execution_result(
        &self,
        plan_id: Uuid,
    ) -> Result<Option<models::PlanExecutionResult>, anyhow::Error>;
    async fn create_council_session(
        &self,
        session: CreateCouncilSession,
    ) -> Result<models::CouncilSession, anyhow::Error>;
    async fn get_council_session(
        &self,
        session_id: Uuid,
    ) -> Result<Option<models::CouncilSession>, anyhow::Error>;
    async fn get_council_session_by_task(
        &self,
        task_id: Uuid,
    ) -> Result<Option<models::CouncilSession>, anyhow::Error>;
    async fn update_council_session(
        &self,
        session_id: Uuid,
        update: UpdateCouncilSession,
    ) -> Result<models::CouncilSession, anyhow::Error>;
}

/// Create execution plan request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateExecutionPlan {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub title: String,
    pub overview: String,
    /// Working spec ID (e.g., "TASK-<UUID>" for task-based plans, "PLAN-<UUID>" for direct plans)
    pub working_spec_id: Option<String>,
    // Add other fields as needed
}

/// Update execution plan request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateExecutionPlan {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub title: Option<String>,
    pub overview: Option<String>,
    pub status: Option<String>,
    // Add other fields as needed
}

/// Models namespace
pub mod models {
    use super::*;

    /// Execution plan model
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct ExecutionPlan {
        #[schemars(with = "String")]
        pub id: Uuid,
        #[schemars(with = "String")]
        pub session_id: Uuid,
        pub working_spec_id: String,
        pub title: String,
        pub overview: Option<String>,
        pub state: String,
        pub milestones: serde_json::Value,
        pub dependency_graph: serde_json::Value,
        pub change_budget: serde_json::Value,
        pub quality_gates: serde_json::Value,
        pub evidence_requirements: serde_json::Value,
        pub active_waivers: serde_json::Value,
        pub metadata: serde_json::Value,
        #[schemars(with = "String")]
        pub created_at: DateTime<Utc>,
        #[schemars(with = "String")]
        pub updated_at: DateTime<Utc>,
        pub approved_at: Option<DateTime<Utc>>,
        pub completed_at: Option<DateTime<Utc>>,
    }

    /// Planning session model
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct PlanningSession {
        #[schemars(with = "String")]
        pub id: Uuid,
        #[schemars(with = "String")]
        pub plan_id: Uuid,
        pub status: String,
        #[schemars(with = "String")]
        pub created_at: DateTime<Utc>,
        #[schemars(with = "String")]
        pub updated_at: DateTime<Utc>,
        pub metadata: HashMap<String, serde_json::Value>,
    }

    /// Milestone model
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct Milestone {
        #[schemars(with = "String")]
        pub id: Uuid,
        #[schemars(with = "String")]
        pub plan_id: Uuid,
        pub title: String,
        pub description: String,
        pub status: String,
        #[schemars(with = "String")]
        pub created_at: DateTime<Utc>,
        #[schemars(with = "String")]
        pub updated_at: DateTime<Utc>,
    }

    /// Planning audit event model
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct PlanningAuditEvent {
        #[schemars(with = "String")]
        pub id: Uuid,
        #[schemars(with = "String")]
        pub session_id: Uuid,
        pub event_type: String,
        pub description: String,
        #[schemars(with = "String")]
        pub timestamp: DateTime<Utc>,
        pub metadata: HashMap<String, serde_json::Value>,
    }

    /// Planning telemetry model
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct PlanningTelemetry {
        #[schemars(with = "String")]
        pub id: Uuid,
        #[schemars(with = "String")]
        pub session_id: Uuid,
        pub metric_name: String,
        pub metric_value: f64,
        #[schemars(with = "String")]
        pub timestamp: DateTime<Utc>,
        pub metadata: HashMap<String, serde_json::Value>,
    }

    /// Audit trail entry model
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct AuditTrailEntry {
        #[schemars(with = "String")]
        pub id: Uuid,
        pub event_type: String,
        pub description: String,
        #[schemars(with = "String")]
        pub timestamp: DateTime<Utc>,
        pub metadata: HashMap<String, serde_json::Value>,
    }

    /// Worker model
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct Worker {
        #[schemars(with = "String")]
        pub id: Uuid,
        pub name: String,
        pub worker_type: String,
        pub specialty: Option<String>,
        pub model_name: String,
        pub endpoint: String,
        pub capabilities: serde_json::Value,
        pub performance_history: serde_json::Value,
        pub is_active: bool,
        pub metadata: HashMap<String, serde_json::Value>,
        #[schemars(with = "String")]
        pub created_at: DateTime<Utc>,
        #[schemars(with = "String")]
        pub updated_at: DateTime<Utc>,
    }

    /// Waiver model
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct Waiver {
        #[schemars(with = "String")]
        pub id: Uuid,
        #[schemars(with = "String")]
        pub plan_id: Uuid,
        pub waiver_type: String,
        pub reason: String,
        pub approved_by: String,
        pub status: String,
        /// Waived gates
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        pub gates: Vec<String>,
        /// Impact level (low, medium, high, critical)
        pub impact_level: String,
        /// Mitigation plan (if required)
        #[serde(skip_serializing_if = "Option::is_none")]
        pub mitigation_plan: Option<String>,
        #[schemars(with = "String")]
        pub created_at: DateTime<Utc>,
        #[schemars(with = "Option<String>")]
        pub expires_at: Option<DateTime<Utc>>,
        pub metadata: HashMap<String, serde_json::Value>,
    }

    /// Judge model
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct Judge {
        #[schemars(with = "String")]
        pub id: Uuid,
        pub name: String,
        pub judge_type: String,
        pub configuration: serde_json::Value,
        pub is_active: bool,
        pub metadata: HashMap<String, serde_json::Value>,
        #[schemars(with = "String")]
        pub created_at: DateTime<Utc>,
        #[schemars(with = "String")]
        pub updated_at: DateTime<Utc>,
    }

    /// Judge evaluation model
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct JudgeEvaluation {
        #[schemars(with = "String")]
        pub id: Uuid,
        #[schemars(with = "String")]
        pub judge_id: Uuid,
        #[schemars(with = "String")]
        pub task_id: Uuid,
        pub evaluation: serde_json::Value,
        pub score: f64,
        pub metadata: HashMap<String, serde_json::Value>,
        #[schemars(with = "String")]
        pub created_at: DateTime<Utc>,
    }

    /// Plan execution result model
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct PlanExecutionResult {
        #[schemars(with = "String")]
        pub plan_id: Uuid,
        pub success: bool,
        pub milestones_completed: i32,
        pub total_duration_ms: i64,
        pub evidence: serde_json::Value,
        pub metrics: serde_json::Value,
        pub final_state: String,
        pub timeline: serde_json::Value,
        #[schemars(with = "String")]
        pub created_at: DateTime<Utc>,
        #[schemars(with = "String")]
        pub updated_at: DateTime<Utc>,
    }

    /// Council session model
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct CouncilSession {
        #[schemars(with = "String")]
        pub id: Uuid,
        #[schemars(with = "String")]
        pub session_id: Uuid,
        #[schemars(with = "String")]
        pub task_id: Option<Uuid>,
        pub working_spec_id: Option<String>,
        pub review_context: serde_json::Value,
        pub status: String,
        pub selected_judges: serde_json::Value,
        pub contributions: serde_json::Value,
        pub aggregation_result: Option<serde_json::Value>,
        pub final_decision: Option<serde_json::Value>,
        pub progress: f64,
        #[schemars(with = "String")]
        pub started_at: DateTime<Utc>,
        pub completed_at: Option<DateTime<Utc>>,
        #[schemars(with = "String")]
        pub created_at: DateTime<Utc>,
        #[schemars(with = "String")]
        pub updated_at: DateTime<Utc>,
        pub metadata: serde_json::Value,
    }
}

/// Create audit trail entry
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateAuditTrailEntry {
    pub event_type: String,
    pub description: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Create planning audit event
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreatePlanningAuditEvent {
    #[schemars(with = "String")]
    pub plan_id: Uuid,
    pub event_type: String,
    pub description: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Create planning session
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreatePlanningSession {
    #[schemars(with = "String")]
    pub plan_id: Uuid,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Update planning session
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdatePlanningSession {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub status: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Create planning telemetry
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreatePlanningTelemetry {
    #[schemars(with = "String")]
    pub session_id: Uuid,
    pub metric_name: String,
    pub metric_value: f64,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Create waiver
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateWaiver {
    #[schemars(with = "String")]
    pub plan_id: Uuid,
    pub reason: String,
    pub waived_gates: Vec<String>,
}

/// Update waiver
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateWaiver {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub status: String,
    // Add other fields as needed
}

/// Create judge request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateJudge {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub name: String,
    pub judge_type: String,
    pub configuration: serde_json::Value,
}

/// Create judge evaluation request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateJudgeEvaluation {
    #[schemars(with = "String")]
    pub judge_id: Uuid,
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub evaluation: serde_json::Value,
    pub score: f64,
}

/// Update judge request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateJudge {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub name: Option<String>,
    pub judge_type: Option<String>,
    pub configuration: Option<serde_json::Value>,
    pub is_active: Option<bool>,
}

/// Create execution result request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateExecutionResult {
    #[schemars(with = "String")]
    pub plan_id: Uuid,
    pub success: bool,
    pub milestones_completed: usize,
    pub total_duration_ms: u64,
    pub evidence: serde_json::Value,
    pub metrics: serde_json::Value,
    pub final_state: String,
    pub timeline: serde_json::Value,
}

/// Create worker request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

/// Update worker request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

/// Create council session request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateCouncilSession {
    #[schemars(with = "String")]
    pub session_id: Uuid,
    #[schemars(with = "String")]
    pub task_id: Option<Uuid>,
    pub working_spec_id: Option<String>,
    pub review_context: serde_json::Value,
    pub status: Option<String>,
    pub selected_judges: Option<serde_json::Value>,
    pub contributions: Option<serde_json::Value>,
    pub progress: Option<f64>,
    pub metadata: Option<serde_json::Value>,
}

/// Update council session request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateCouncilSession {
    pub status: Option<String>,
    pub selected_judges: Option<serde_json::Value>,
    pub contributions: Option<serde_json::Value>,
    pub aggregation_result: Option<serde_json::Value>,
    pub final_decision: Option<serde_json::Value>,
    pub progress: Option<f64>,
    pub completed_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
}

/// Cost limits
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CostLimits {
    pub max_cost: f64,
    pub currency: String,
}
