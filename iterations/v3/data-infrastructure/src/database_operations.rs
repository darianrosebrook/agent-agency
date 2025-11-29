//! Database operations trait and types for consistent CRUD operations
//!
//! This module defines the DatabaseOperations trait and associated input/output types
//! for all database operations across the system. It provides a unified interface
//! for database clients to implement consistent CRUD operations.

use crate::models::*;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
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
    async fn get_tasks_by_project(&self, project_id: Uuid) -> Result<Vec<Task>>;
    async fn get_project_task_stats(&self, project_id: Uuid) -> Result<serde_json::Value>;
    async fn update_task(&self, id: Uuid, update: UpdateTask) -> Result<Task>;
    async fn delete_task(&self, id: Uuid) -> Result<()>;

    // Task execution operations
    async fn create_task_execution(&self, execution: CreateTaskExecution) -> Result<TaskExecution>;
    async fn get_task_execution(&self, id: Uuid) -> Result<Option<TaskExecution>>;
    async fn get_task_executions(&self, task_id: Uuid) -> Result<Vec<TaskExecution>>;
    async fn update_task_execution(
        &self,
        id: Uuid,
        update: UpdateTaskExecution,
    ) -> Result<TaskExecution>;

    // Audit trail operations
    async fn create_audit_trail_entry(
        &self,
        entry: CreateAuditTrailEntry,
    ) -> Result<AuditTrailEntry>;
    async fn get_audit_trail_entries(&self, task_id: Uuid) -> Result<Vec<AuditTrailEntry>>;
    async fn get_audit_trail_entry(&self, id: Uuid) -> Result<Option<AuditTrailEntry>>;

    // Council verdict operations
    async fn create_council_verdict(&self, verdict: CreateCouncilVerdict)
        -> Result<CouncilVerdict>;
    async fn get_council_verdict(&self, id: Uuid) -> Result<Option<CouncilVerdict>>;
    async fn get_council_verdicts(&self, task_id: Uuid) -> Result<Vec<CouncilVerdict>>;

    // Council session operations
    async fn create_council_session(&self, session: CreateCouncilSession)
        -> Result<CouncilSession>;
    async fn get_council_session(&self, session_id: Uuid) -> Result<Option<CouncilSession>>;
    async fn get_council_session_by_task(&self, task_id: Uuid) -> Result<Option<CouncilSession>>;
    async fn update_council_session(
        &self,
        session_id: Uuid,
        update: UpdateCouncilSession,
    ) -> Result<CouncilSession>;

    // Judge evaluation operations
    async fn create_judge_evaluation(
        &self,
        evaluation: CreateJudgeEvaluation,
    ) -> Result<JudgeEvaluation>;
    async fn get_judge_evaluations(&self, task_id: Uuid) -> Result<Vec<JudgeEvaluation>>;

    // Planning operations
    async fn create_planning_telemetry(
        &self,
        telemetry: CreatePlanningTelemetry,
    ) -> Result<PlanningTelemetry>;
    async fn get_planning_telemetry(
        &self,
        plan_id: Uuid,
        metric_type: Option<String>,
    ) -> Result<Vec<PlanningTelemetry>>;
    async fn create_milestone(&self, milestone: CreateMilestone) -> Result<Milestone>;
    async fn get_milestone(&self, plan_id: Uuid, milestone_id: String)
        -> Result<Option<Milestone>>;
    async fn get_milestones(&self, plan_id: Uuid) -> Result<Vec<Milestone>>;
    async fn update_milestone(
        &self,
        plan_id: Uuid,
        milestone_id: String,
        update: UpdateMilestone,
    ) -> Result<Milestone>;
    async fn delete_milestone(&self, plan_id: Uuid, milestone_id: String) -> Result<()>;
    async fn create_planning_session(
        &self,
        session: CreatePlanningSession,
    ) -> Result<PlanningSession>;
    async fn get_planning_session(&self, id: Uuid) -> Result<Option<PlanningSession>>;
    async fn get_planning_sessions(&self, plan_id: Uuid) -> Result<Vec<PlanningSession>>;
    async fn update_planning_session(
        &self,
        id: Uuid,
        update: UpdatePlanningSession,
    ) -> Result<PlanningSession>;
    async fn create_evidence_artifact(
        &self,
        artifact: CreateEvidenceArtifact,
    ) -> Result<EvidenceArtifact>;
    async fn get_evidence_artifacts(&self, plan_id: Uuid) -> Result<Vec<EvidenceArtifact>>;
    async fn get_evidence_artifacts_for_milestone(
        &self,
        plan_id: Uuid,
        milestone_id: String,
    ) -> Result<Vec<EvidenceArtifact>>;
    async fn update_evidence_artifact(
        &self,
        id: Uuid,
        update: UpdateEvidenceArtifact,
    ) -> Result<EvidenceArtifact>;
    async fn create_planning_audit_event(
        &self,
        event: CreatePlanningAuditEvent,
    ) -> Result<PlanningAuditEvent>;
    async fn get_planning_audit_events(&self, plan_id: Uuid) -> Result<Vec<PlanningAuditEvent>>;
    async fn create_execution_plan(&self, plan: CreateExecutionPlan) -> Result<ExecutionPlan>;
    async fn get_execution_plan(&self, id: Uuid) -> Result<Option<ExecutionPlan>>;
    async fn get_execution_plans(&self) -> Result<Vec<ExecutionPlan>>;
    async fn update_execution_plan(
        &self,
        id: Uuid,
        update: UpdateExecutionPlan,
    ) -> Result<ExecutionPlan>;
    async fn delete_execution_plan(&self, id: Uuid) -> Result<()>;

    // Waiver operations
    async fn get_waivers(&self, status: Option<String>) -> Result<Vec<Waiver>>;
    async fn create_waiver(&self, waiver: CreateWaiver) -> Result<Waiver>;
    async fn update_waiver(&self, id: Uuid, update: UpdateWaiver) -> Result<Waiver>;

    // User operations
    async fn create_user(&self, user: CreateUser) -> Result<User>;
    async fn get_user(&self, id: Uuid) -> Result<Option<User>>;
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>>;
    async fn update_user(&self, id: Uuid, update: UpdateUser) -> Result<User>;
    async fn delete_user(&self, id: Uuid) -> Result<()>;

    // Session operations
    async fn create_session(&self, session: CreateSession) -> Result<Session>;
    async fn get_session(&self, id: Uuid) -> Result<Option<Session>>;
    async fn get_session_by_token_hash(&self, token_hash: &str) -> Result<Option<Session>>;
    async fn get_session_by_refresh_token_hash(&self, refresh_token_hash: &str) -> Result<Option<Session>>;
    async fn get_user_sessions(&self, user_id: Uuid) -> Result<Vec<Session>>;
    async fn update_session(&self, id: Uuid, update: UpdateSession) -> Result<Session>;
    async fn delete_session(&self, id: Uuid) -> Result<()>;
    async fn delete_user_sessions(&self, user_id: Uuid) -> Result<()>;
    async fn cleanup_expired_sessions(&self) -> Result<usize>;

    // Password reset token operations
    async fn create_password_reset_token(
        &self,
        token: CreatePasswordResetToken,
    ) -> Result<PasswordResetToken>;
    async fn get_password_reset_token(
        &self,
        token_hash: &str,
    ) -> Result<Option<PasswordResetToken>>;
    async fn mark_password_reset_token_used(&self, id: Uuid) -> Result<()>;
    async fn cleanup_expired_password_reset_tokens(&self) -> Result<usize>;

    // User settings operations
    async fn create_user_setting(&self, setting: CreateUserSetting) -> Result<UserSetting>;
    async fn get_user_setting(
        &self,
        user_id: Uuid,
        setting_key: &str,
    ) -> Result<Option<UserSetting>>;
    async fn get_user_settings(
        &self,
        user_id: Uuid,
        setting_type: Option<&str>,
    ) -> Result<Vec<UserSetting>>;
    async fn update_user_setting(
        &self,
        user_id: Uuid,
        setting_key: &str,
        update: UpdateUserSetting,
    ) -> Result<UserSetting>;
    async fn delete_user_setting(&self, user_id: Uuid, setting_key: &str) -> Result<()>;

    // App settings operations
    async fn create_app_setting(&self, setting: CreateAppSetting) -> Result<AppSetting>;
    async fn get_app_setting(&self, setting_key: &str) -> Result<Option<AppSetting>>;
    async fn get_app_settings(
        &self,
        setting_type: Option<&str>,
        is_public: Option<bool>,
    ) -> Result<Vec<AppSetting>>;
    async fn update_app_setting(
        &self,
        setting_key: &str,
        update: UpdateAppSetting,
    ) -> Result<AppSetting>;
    async fn delete_app_setting(&self, setting_key: &str) -> Result<()>;

    // Integration operations
    async fn create_integration(&self, integration: CreateIntegration) -> Result<Integration>;
    async fn get_integration(&self, id: Uuid) -> Result<Option<Integration>>;
    async fn get_integrations(
        &self,
        provider: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<Vec<Integration>>;
    async fn update_integration(&self, id: Uuid, update: UpdateIntegration) -> Result<Integration>;
    async fn delete_integration(&self, id: Uuid) -> Result<()>;

    // API key operations
    async fn create_api_key(&self, api_key: CreateApiKey) -> Result<ApiKey>;
    async fn get_api_key(&self, id: Uuid) -> Result<Option<ApiKey>>;
    async fn get_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>>;
    async fn get_user_api_keys(
        &self,
        user_id: Uuid,
        is_active: Option<bool>,
    ) -> Result<Vec<ApiKey>>;
    async fn update_api_key(&self, id: Uuid, update: UpdateApiKey) -> Result<ApiKey>;
    async fn revoke_api_key(&self, id: Uuid, reason: Option<String>) -> Result<()>;
    async fn delete_api_key(&self, id: Uuid) -> Result<()>;

    // Two-factor authentication operations
    async fn create_two_factor_auth(&self, two_fa: CreateTwoFactorAuth) -> Result<TwoFactorAuth>;
    async fn get_two_factor_auth(
        &self,
        user_id: Uuid,
        method: Option<&str>,
    ) -> Result<Option<TwoFactorAuth>>;
    async fn update_two_factor_auth(
        &self,
        user_id: Uuid,
        method: &str,
        update: UpdateTwoFactorAuth,
    ) -> Result<TwoFactorAuth>;
    async fn delete_two_factor_auth(&self, user_id: Uuid, method: &str) -> Result<()>;

    // CAWS Rules operations
    async fn create_caws_rule(&self, rule: CreateCawsRule) -> Result<CawsRule>;
    async fn get_caws_rule(&self, id: &str) -> Result<Option<CawsRule>>;
    async fn get_caws_rules(
        &self,
        rule_type: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<Vec<CawsRule>>;
    async fn update_caws_rule(&self, id: &str, update: UpdateCawsRule) -> Result<CawsRule>;
    async fn delete_caws_rule(&self, id: &str) -> Result<()>;

    // CAWS Violations operations
    async fn create_caws_violation(&self, violation: CreateCawsViolation) -> Result<CawsViolation>;
    async fn get_caws_violation(&self, id: Uuid) -> Result<Option<CawsViolation>>;
    async fn get_caws_violations(
        &self,
        task_id: Option<Uuid>,
        rule_id: Option<&str>,
        status: Option<&str>,
    ) -> Result<Vec<CawsViolation>>;
    async fn update_caws_violation(
        &self,
        id: Uuid,
        update: UpdateCawsViolation,
    ) -> Result<CawsViolation>;
    async fn resolve_caws_violation(&self, id: Uuid) -> Result<()>;

    // CAWS Specifications operations
    async fn create_caws_specification(
        &self,
        spec: CreateCawsSpecification,
    ) -> Result<CawsSpecification>;
    async fn get_caws_specification(&self, id: Uuid) -> Result<Option<CawsSpecification>>;
    async fn get_caws_specifications(
        &self,
        name: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<Vec<CawsSpecification>>;
    async fn update_caws_specification(
        &self,
        id: Uuid,
        update: UpdateCawsSpecification,
    ) -> Result<CawsSpecification>;
    async fn delete_caws_specification(&self, id: Uuid) -> Result<()>;

    // Rule templates operations
    async fn get_rule_templates(&self, rule_type: Option<&str>) -> Result<Vec<RuleTemplate>>;
    async fn create_rule_template(&self, template: CreateRuleTemplate) -> Result<RuleTemplate>;

    // Rule enforcement status operations
    async fn get_rule_enforcement_status(
        &self,
        rule_id: Option<&str>,
        task_id: Option<Uuid>,
    ) -> Result<Vec<RuleEnforcementStatus>>;
    async fn update_rule_enforcement_status(
        &self,
        rule_id: &str,
        task_id: Option<Uuid>,
        status: UpdateRuleEnforcementStatus,
    ) -> Result<RuleEnforcementStatus>;

    // Rule history operations
    async fn get_rule_history(&self, rule_id: &str, limit: Option<u32>)
        -> Result<Vec<RuleHistory>>;
}

/// Input types for database operations

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateJudge {
    pub name: String,
    pub model_name: String,
    pub endpoint: String,
    pub weight: f32,
    pub timeout_ms: i32,
    pub optimization_target: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateJudge {
    pub name: Option<String>,
    pub model_name: Option<String>,
    pub endpoint: Option<String>,
    pub weight: Option<f32>,
    pub timeout_ms: Option<i32>,
    pub optimization_target: Option<String>,
    pub is_active: Option<bool>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
    pub project_id: Option<Uuid>,
    pub priority: Option<i32>,
    pub deadline: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
    pub project_id: Option<Uuid>,
    pub priority: Option<i32>,
    pub deadline: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateTaskExecution {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    #[schemars(with = "String")]
    pub worker_id: Uuid,
    #[schemars(with = "String")]
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateAuditTrailEntry {
    pub entity_type: String,
    #[schemars(with = "String")]
    pub entity_id: Uuid,
    pub action: String,
    pub details: serde_json::Value,
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateCouncilVerdict {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    #[schemars(with = "String")]
    pub verdict_id: Uuid,
    pub consensus_score: f32,
    pub final_verdict: serde_json::Value,
    pub individual_verdicts: serde_json::Value,
    pub debate_rounds: i32,
    pub evaluation_time_ms: i32,
    pub contract: serde_json::Value,
    pub verdict_details: Option<serde_json::Value>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateJudgeEvaluation {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    #[schemars(with = "String")]
    pub judge_id: Uuid,
    pub evaluation_score: f32,
    pub evaluation_reasoning: String,
    pub evaluation_metadata: serde_json::Value,
    pub evaluation_time_ms: i32,
    #[schemars(with = "String")]
    pub evaluation_timestamp: DateTime<Utc>,
}

/// Planning telemetry input types
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreatePlanningTelemetry {
    #[schemars(with = "String")]
    pub plan_id: Uuid,
    pub metric_type: String,
    pub metric_value: serde_json::Value,
    pub metadata: Option<serde_json::Value>,
    pub collected_at: Option<DateTime<Utc>>,
}

/// Milestone input types
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateMilestone {
    pub id: String,
    #[schemars(with = "String")]
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreatePlanningSession {
    #[schemars(with = "String")]
    pub plan_id: Uuid,
    pub orchestrator_id: String,
    pub worker_pool_id: String,
    pub council_session_id: Option<Uuid>,
    #[schemars(with = "String")]
    pub audit_correlation_id: Uuid,
    pub status: Option<String>,
    pub execution_state: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdatePlanningSession {
    pub status: Option<String>,
    pub execution_state: Option<serde_json::Value>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Evidence artifact input types
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateEvidenceArtifact {
    pub milestone_id: String,
    #[schemars(with = "String")]
    pub plan_id: Uuid,
    pub artifact_type: String,
    pub artifact_data: serde_json::Value,
    pub verified: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateEvidenceArtifact {
    pub artifact_type: Option<String>,
    pub artifact_data: Option<serde_json::Value>,
    pub verified: Option<bool>,
    pub verified_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
}

/// Planning audit event input types
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreatePlanningAuditEvent {
    #[schemars(with = "String")]
    pub plan_id: Uuid,
    pub milestone_id: Option<String>,
    pub worker_id: Option<Uuid>,
    pub event_type: String,
    pub description: String,
    pub metadata: Option<serde_json::Value>,
}

/// Execution plan input types
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateExecutionPlan {
    #[schemars(with = "String")]
    pub id: Uuid,
    #[schemars(with = "String")]
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateWaiver {
    pub title: String,
    pub reason: String,
    pub description: String,
    pub gates: Vec<String>,
    pub approved_by: String,
    pub impact_level: String,
    pub mitigation_plan: String,
    #[schemars(with = "String")]
    pub expires_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateWaiver {
    pub title: Option<String>,
    pub description: Option<String>,
    pub mitigation_plan: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub status: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateUser {
    pub username: String,
    pub password_hash: String,
    pub name: Option<String>,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub password_hash: Option<String>,
    pub name: Option<String>,
    pub roles: Option<Vec<String>>,
    pub is_active: Option<bool>,
    pub failed_attempts: Option<i32>,
    pub locked_until: Option<DateTime<Utc>>,
    pub last_login: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateSession {
    #[schemars(with = "String")]
    pub user_id: Uuid,
    pub token_hash: String,
    pub refresh_token_hash: Option<String>,
    #[schemars(with = "String")]
    pub expires_at: DateTime<Utc>,
    pub refresh_expires_at: Option<DateTime<Utc>>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateSession {
    pub token_hash: Option<String>,
    pub refresh_token_hash: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub refresh_expires_at: Option<DateTime<Utc>>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreatePasswordResetToken {
    #[schemars(with = "String")]
    pub user_id: Uuid,
    pub token_hash: String,
    #[schemars(with = "String")]
    pub expires_at: DateTime<Utc>,
    pub ip_address: Option<String>,
}

// Settings management input types

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateUserSetting {
    #[schemars(with = "String")]
    pub user_id: Uuid,
    pub setting_key: String,
    pub setting_value: serde_json::Value,
    pub setting_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateUserSetting {
    pub setting_value: Option<serde_json::Value>,
    pub setting_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateAppSetting {
    pub setting_key: String,
    pub setting_value: serde_json::Value,
    pub setting_type: String,
    pub description: Option<String>,
    pub is_public: bool,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateAppSetting {
    pub setting_value: Option<serde_json::Value>,
    pub setting_type: Option<String>,
    pub description: Option<String>,
    pub is_public: Option<bool>,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateIntegration {
    pub name: String,
    pub integration_type: String,
    pub provider: String,
    pub configuration: serde_json::Value,
    pub credentials: serde_json::Value,
    pub is_active: bool,
    pub is_enabled: bool,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateIntegration {
    pub name: Option<String>,
    pub configuration: Option<serde_json::Value>,
    pub credentials: Option<serde_json::Value>,
    pub is_active: Option<bool>,
    pub is_enabled: Option<bool>,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateApiKey {
    #[schemars(with = "String")]
    pub user_id: Uuid,
    pub key_name: String,
    pub key_hash: String,
    pub key_prefix: String,
    pub scopes: Vec<String>,
    pub rate_limit_per_minute: Option<i32>,
    pub rate_limit_per_hour: Option<i32>,
    pub rate_limit_per_day: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateApiKey {
    pub key_name: Option<String>,
    pub scopes: Option<Vec<String>>,
    pub rate_limit_per_minute: Option<i32>,
    pub rate_limit_per_hour: Option<i32>,
    pub rate_limit_per_day: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateTwoFactorAuth {
    #[schemars(with = "String")]
    pub user_id: Uuid,
    pub method: String,
    pub secret_encrypted: String,
    pub backup_codes: Vec<String>,
    pub is_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateTwoFactorAuth {
    pub secret_encrypted: Option<String>,
    pub backup_codes: Option<Vec<String>>,
    pub is_enabled: Option<bool>,
}

// ============================================================================
// CAWS Rules Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateCawsRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rule_type: String,
    pub severity: String,
    pub file_patterns: serde_json::Value,
    pub config: serde_json::Value,
    pub constitutional_reference: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateCawsRule {
    pub name: Option<String>,
    pub description: Option<String>,
    pub rule_type: Option<String>,
    pub severity: Option<String>,
    pub file_patterns: Option<serde_json::Value>,
    pub config: Option<serde_json::Value>,
    pub constitutional_reference: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateCawsViolation {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub violation_code: String,
    pub severity: String,
    pub description: String,
    pub file_path: Option<String>,
    pub line_number: Option<i32>,
    pub column_number: Option<i32>,
    pub rule_id: String,
    pub constitutional_reference: Option<String>,
    pub status: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateCawsViolation {
    pub status: Option<String>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateCawsSpecification {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub rules: serde_json::Value,
    pub config: serde_json::Value,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateCawsSpecification {
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub rules: Option<serde_json::Value>,
    pub config: Option<serde_json::Value>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, JsonSchema)]
pub struct RuleTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rule_type: String,
    pub template_config: serde_json::Value,
    pub example_config: Option<serde_json::Value>,
    pub is_system: bool,
    pub created_by: String,
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
    #[schemars(with = "String")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateRuleTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rule_type: String,
    pub template_config: serde_json::Value,
    pub example_config: Option<serde_json::Value>,
    pub is_system: bool,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, JsonSchema)]
pub struct RuleEnforcementStatus {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub rule_id: String,
    #[schemars(with = "String")]
    pub task_id: Option<Uuid>,
    pub enforcement_state: String,
    pub paused_until: Option<DateTime<Utc>>,
    pub paused_reason: Option<String>,
    pub override_reason: Option<String>,
    pub metadata: serde_json::Value,
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
    #[schemars(with = "String")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateRuleEnforcementStatus {
    pub enforcement_state: Option<String>,
    pub paused_until: Option<DateTime<Utc>>,
    pub paused_reason: Option<String>,
    pub override_reason: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, JsonSchema)]
pub struct RuleHistory {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub rule_id: String,
    pub action: String,
    pub changed_by: String,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
    pub change_reason: Option<String>,
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
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
    async fn create_audit_entry(
        &self,
        entry: system_common_interfaces::CreateAuditEntry,
    ) -> system_common_interfaces::Result<()> {
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
        self.db_ops
            .create_audit_trail_entry(audit_entry)
            .await
            .map_err(|e| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                )) as Box<dyn std::error::Error + Send + Sync>
            })?;

        Ok(())
    }
}

/// Milestone completion input types
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateMilestoneCompletion {
    #[schemars(with = "String")]
    pub agent_id: Uuid,
    pub milestone_id: String,
    pub domain: String,
    pub required_level: String,
    pub target_level: String,
    pub complexity: String,
    pub success: bool,
    pub quality_score: f64,
    pub completion_time_ms: Option<i32>,
    pub attempts: i32,
    pub prerequisites_met: bool,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateLearningRecord {
    #[schemars(with = "String")]
    pub agent_id: Uuid,
    #[schemars(with = "String")]
    pub task_id: Option<Uuid>,
    pub domain: String,
    pub complexity: String,
    pub adjusted_complexity: Option<String>,
    pub skill_level_before: String,
    pub skill_level_after: Option<String>,
    pub success: bool,
    pub quality_score: f64,
    pub execution_metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateCurriculumProfile {
    #[schemars(with = "String")]
    pub agent_id: Uuid,
    pub overall_level: String,
    pub skills: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MilestoneCompletionResult {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub milestone_id: String,
    pub success: bool,
    pub completed_at: DateTime<Utc>,
}

/// Curriculum learning database operations
impl crate::client::orchestrator::DatabaseClient {
    /// Record milestone completion
    pub async fn record_milestone_completion(
        &self,
        completion: CreateMilestoneCompletion,
    ) -> Result<MilestoneCompletionResult> {
        let result = sqlx::query!(
            r#"
            SELECT record_milestone_completion(
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12
            ) as completion_id
            "#,
            completion.agent_id,
            completion.milestone_id,
            completion.domain,
            completion.required_level,
            completion.target_level,
            completion.complexity,
            completion.success,
            completion.quality_score,
            completion.completion_time_ms,
            completion.attempts,
            completion.prerequisites_met,
            completion.metadata.unwrap_or(serde_json::Value::Null)
        )
        .fetch_one(&self.pool)
        .await?;

        // Get the created completion details
        let completion_details = sqlx::query!(
            r#"
            SELECT id, agent_id, milestone_id, success, completed_at
            FROM milestone_completions
            WHERE id = $1
            "#,
            result.completion_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(MilestoneCompletionResult {
            id: completion_details.id,
            agent_id: completion_details.agent_id,
            milestone_id: completion_details.milestone_id,
            success: completion_details.success,
            completed_at: completion_details.completed_at,
        })
    }

    /// Record learning outcome
    pub async fn record_learning_outcome(
        &self,
        record: CreateLearningRecord,
    ) -> Result<Uuid> {
        let result = sqlx::query!(
            r#"
            INSERT INTO learning_history (
                agent_id, task_id, domain, complexity, adjusted_complexity,
                skill_level_before, skill_level_after, success, quality_score,
                execution_metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id
            "#,
            record.agent_id,
            record.task_id,
            record.domain,
            record.complexity,
            record.adjusted_complexity,
            record.skill_level_before,
            record.skill_level_after,
            record.success,
            record.quality_score,
            record.execution_metadata.unwrap_or(serde_json::Value::Null)
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.id)
    }

    /// Check if milestone prerequisites are met
    pub async fn check_milestone_prerequisites(
        &self,
        agent_id: Uuid,
        prerequisite_ids: Vec<String>,
    ) -> Result<bool> {
        let prerequisites_json = serde_json::Value::Array(
            prerequisite_ids.into_iter().map(serde_json::Value::String).collect()
        );

        let result = sqlx::query!(
            r#"
            SELECT check_milestone_prerequisites($1, $2) as prerequisites_met
            "#,
            agent_id,
            prerequisites_json
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.prerequisites_met.unwrap_or(false))
    }

    /// Get agent skill level for domain
    pub async fn get_agent_skill_level(
        &self,
        agent_id: Uuid,
        domain: &str,
    ) -> Result<String> {
        let result = sqlx::query!(
            r#"
            SELECT get_agent_skill_level($1, $2) as skill_level
            "#,
            agent_id,
            domain
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.skill_level.unwrap_or_else(|| "beginner".to_string()))
    }

    /// Create or update curriculum profile
    pub async fn upsert_curriculum_profile(
        &self,
        profile: CreateCurriculumProfile,
    ) -> Result<Uuid> {
        let result = sqlx::query!(
            r#"
            INSERT INTO curriculum_profiles (
                agent_id, overall_level, skills, total_tasks_completed,
                total_tasks_succeeded, last_updated
            )
            VALUES ($1, $2, $3, 0, 0, NOW())
            ON CONFLICT (agent_id) DO UPDATE SET
                overall_level = EXCLUDED.overall_level,
                skills = EXCLUDED.skills,
                last_updated = NOW()
            RETURNING id
            "#,
            profile.agent_id,
            profile.overall_level,
            profile.skills
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.id)
    }

    /// Get curriculum profile for agent
    pub async fn get_curriculum_profile(
        &self,
        agent_id: Uuid,
    ) -> Result<Option<CreateCurriculumProfile>> {
        let result = sqlx::query!(
            r#"
            SELECT agent_id, overall_level, skills
            FROM curriculum_profiles
            WHERE agent_id = $1
            "#,
            agent_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|row| CreateCurriculumProfile {
            agent_id: row.agent_id,
            overall_level: row.overall_level,
            skills: row.skills,
        }))
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
