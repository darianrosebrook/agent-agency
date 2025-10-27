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
