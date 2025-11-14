//! API Types and Data Structures
//!
//! Common types used across the API layer for request/response handling,
//! configuration, and data transfer objects.

use chrono::{DateTime, Duration, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use sqlx::Row;
use uuid::Uuid;

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ApiConfig {
    /// Server host
    pub host: String,
    /// Server port
    pub port: u16,
    /// Enable CORS
    pub enable_cors: bool,
    /// Database configuration
    pub database_url: String,
    /// Redis URL (optional)
    pub redis_url: Option<String>,
    /// Log level
    pub log_level: String,
}

/// Task submission request
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TaskSubmissionRequest {
    pub description: String,
    pub context: Option<String>,
    pub priority: Option<String>,
    pub execution_mode: Option<String>,
}

/// Task submission response
#[derive(Debug, Serialize, JsonSchema)]
pub struct TaskSubmissionResponse {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub status: String,
    pub message: String,
    pub estimated_completion: Option<DateTime<Utc>>,
}

/// Link provenance request
#[derive(Debug, Deserialize, JsonSchema)]
pub struct LinkProvenanceRequest {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    #[schemars(with = "String")]
    pub provenance_id: Uuid,
    pub relationship_type: String,
    pub commit_hash: String,
}

/// Working specification (stub)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkingSpec {
    pub id: String,
    pub title: String,
    pub description: String,
    pub risk_tier: u8,
    pub mode: String,
    pub change_budget: ChangeBudget,
    pub blast_radius: BlastRadius,
    pub operational_rollback_slo: String,
    pub scope: Scope,
    pub invariants: Vec<String>,
    pub acceptance: Vec<AcceptanceCriterion>,
    pub non_functional: NonFunctionalRequirements,
    pub contracts: Vec<Contract>,
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
}

/// Change budget
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChangeBudget {
    pub max_files: u32,
    pub max_loc: u32,
}

/// Blast radius
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlastRadius {
    pub modules: Vec<String>,
    pub data_migration: bool,
}

/// Scope
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Scope {
    pub r#in: Vec<String>,
    pub out: Vec<String>,
}

/// Acceptance criterion
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AcceptanceCriterion {
    pub id: String,
    pub given: String,
    pub when: String,
    pub then: String,
}

/// Non-functional requirements
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NonFunctionalRequirements {
    pub a11y: Vec<String>,
    pub perf: PerformanceRequirements,
    pub security: Vec<String>,
}

/// Performance requirements
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PerformanceRequirements {
    pub api_p95_ms: u32,
    pub lcp_ms: u32,
}

/// Contract
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Contract {
    pub r#type: String,
    pub path: String,
}

/// Execution artifacts (stub)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionArtifacts {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub working_spec: Option<WorkingSpec>,
    pub quality_report: Option<QualityReport>,
    pub artifacts: Vec<ArtifactMetadata>,
}

/// Artifact metadata (stub)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ArtifactMetadata {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub name: String,
    pub content_type: String,
    pub size: u64,
}

/// Quality report (stub)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualityReport {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub score: f64,
    pub details: String,
    pub overall_score: f64,
    pub checks_passed: u32,
    pub checks_failed: u32,
}

/// Progress tracker with real database integration
#[derive(Debug, Clone, JsonSchema)]
pub struct ProgressTracker {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub current_step: String,
    pub progress_percentage: u8,
    /// Database client for querying real task execution state
    /// This is optional to maintain backward compatibility with schema generation
    #[serde(skip)]
    #[schemars(skip)]
    pub db_client: Option<Arc<crate::DatabaseClient>>,
}

impl ProgressTracker {
    /// Create a new ProgressTracker with database access
    pub fn with_db(db_client: Arc<crate::DatabaseClient>, task_id: Uuid) -> Self {
        Self {
            task_id,
            current_step: String::new(),
            progress_percentage: 0,
            db_client: Some(db_client),
        }
    }

    /// Get progress for a task by querying actual task execution state from database
    pub async fn get_progress(&self, task_id: Uuid) -> Result<ExecutionProgress, anyhow::Error> {
        // If database client is available, query real execution state
        if let Some(db_client) = &self.db_client {
            return Self::get_progress_from_db(db_client, task_id).await;
        }

        // Fallback to struct fields if no database access (for backward compatibility)
        Ok(ExecutionProgress {
            task_id: self.task_id,
            status: "in_progress".to_string(),
            progress: self.progress_percentage,
            current_step: self.current_step.clone(),
            estimated_completion: None,
        })
    }

    /// Query actual task execution state from database
    async fn get_progress_from_db(
        db_client: &Arc<crate::DatabaseClient>,
        task_id: Uuid,
    ) -> Result<ExecutionProgress, anyhow::Error> {
        // Query task_executions table for execution status and timing
        let execution_row = sqlx::query(
            r#"
            SELECT status, execution_started_at, execution_completed_at, execution_time_ms
            FROM task_executions
            WHERE task_id = $1
            ORDER BY execution_started_at DESC
            LIMIT 1
            "#,
        )
        .bind(task_id)
        .fetch_optional(db_client.pool())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to query task execution progress: {}", e))?;

        // Query task_execution_states for current state
        let state_row = sqlx::query(
            r#"
            SELECT status, last_updated, state_data
            FROM task_execution_states
            WHERE task_id = $1
            "#,
        )
        .bind(task_id)
        .fetch_optional(db_client.pool())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to query task execution state: {}", e))?;

        // Query execution_plans to get milestone progress if available
        let plan_row = sqlx::query(
            r#"
            SELECT ep.id, ep.milestones, ep.state, ep.completed_at
            FROM execution_plans ep
            INNER JOIN tasks t ON t.project_id = ep.id
            WHERE t.id = $1
            LIMIT 1
            "#,
        )
        .bind(task_id)
        .fetch_optional(db_client.pool())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to query execution plan: {}", e))?;

        // Determine status and progress from execution, state, and milestones
        let (status, progress, current_step, estimated_completion) = match (execution_row, state_row, plan_row) {
            (Some(exec_row), Some(state_row), Some(plan_row)) => {
                // Use state status if available, otherwise execution status
                let exec_status: String = exec_row
                    .try_get("status")
                    .unwrap_or_else(|_| "unknown".to_string());
                let state_status: String = state_row
                    .try_get("status")
                    .unwrap_or_else(|_| "unknown".to_string());
                let plan_state: String = plan_row
                    .try_get("state")
                    .unwrap_or_else(|_| "unknown".to_string());

                let final_status = if state_status != "unknown" {
                    state_status
                } else if exec_status != "unknown" {
                    exec_status
                } else {
                    plan_state
                };

                // Calculate progress from milestones if available
                let progress = if let Ok(Some(milestones_json)) = plan_row.try_get::<Option<serde_json::Value>, &str>("milestones") {
                    if let Some(milestones_array) = milestones_json.as_array() {
                        let total = milestones_array.len();
                        let completed = milestones_array.iter()
                            .filter(|m| {
                                m.get("state")
                                    .and_then(|s| s.as_str())
                                    .map(|s| s == "completed")
                                    .unwrap_or(false)
                            })
                            .count();
                        if total > 0 {
                            ((completed as f64 / total as f64) * 100.0) as u8
                        } else {
                            0
                        }
                    } else {
                        // Fallback to status-based progress
                        match final_status.as_str() {
                            "pending" => 0,
                            "running" | "executing" => 50,
                            "completed" => 100,
                            "failed" | "cancelled" => 0,
                            "paused" => 50,
                            _ => 0,
                        }
                    }
                } else {
                    // Fallback to status-based progress
                    match final_status.as_str() {
                        "pending" => 0,
                        "running" | "executing" => 50,
                        "completed" => 100,
                        "failed" | "cancelled" => 0,
                        "paused" => 50,
                        _ => 0,
                    }
                };

                // Get current step from state_data or milestone
                let current_step = if let Ok(Some(state_data)) = state_row.try_get::<Option<serde_json::Value>, &str>("state_data") {
                    state_data
                        .get("current_step")
                        .or_else(|| state_data.get("current_phase"))
                        .and_then(|s| s.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| final_status.clone())
                } else {
                    final_status.clone()
                };

                // Estimate completion time if task is in progress
                let estimated_completion = if final_status == "running" || final_status == "executing" {
                    if let Ok(Some(started)) = exec_row.try_get::<Option<DateTime<Utc>>, &str>("execution_started_at") {
                        // Estimate completion based on average task duration (5 minutes)
                        Some(started + Duration::minutes(5))
                    } else {
                        None
                    }
                } else if final_status == "completed" {
                    exec_row.try_get::<Option<DateTime<Utc>>, &str>("execution_completed_at")
                        .ok()
                        .flatten()
                } else {
                    None
                };

                (final_status, progress, current_step, estimated_completion)
            }
            (Some(exec_row), None, _) => {
                let status: String = exec_row
                    .try_get("status")
                    .unwrap_or_else(|_| "unknown".to_string());
                let progress = match status.as_str() {
                    "running" | "executing" => 50,
                    "completed" => 100,
                    "failed" | "cancelled" => 0,
                    _ => 0,
                };
                let estimated_completion = if status == "running" || status == "executing" {
                    exec_row.try_get::<Option<DateTime<Utc>>, &str>("execution_started_at")
                        .ok()
                        .flatten()
                        .map(|started| started + Duration::minutes(5))
                } else {
                    exec_row.try_get::<Option<DateTime<Utc>>, &str>("execution_completed_at")
                        .ok()
                        .flatten()
                };
                (status, progress, "executing".to_string(), estimated_completion)
            }
            (Some(exec_row), Some(state_row), None) => {
                // Execution and state available, but no plan
                let exec_status: String = exec_row
                    .try_get("status")
                    .unwrap_or_else(|_| "unknown".to_string());
                let state_status: String = state_row
                    .try_get("status")
                    .unwrap_or_else(|_| "unknown".to_string());
                let final_status = if state_status != "unknown" {
                    state_status
                } else {
                    exec_status
                };
                let progress = match final_status.as_str() {
                    "pending" => 0,
                    "running" | "executing" => 50,
                    "completed" => 100,
                    "failed" | "cancelled" => 0,
                    "paused" => 50,
                    _ => 0,
                };
                let current_step = if let Ok(Some(state_data)) = state_row.try_get::<Option<serde_json::Value>, &str>("state_data") {
                    state_data
                        .get("current_step")
                        .or_else(|| state_data.get("current_phase"))
                        .and_then(|s| s.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| final_status.clone())
                } else {
                    final_status.clone()
                };
                let estimated_completion = if final_status == "running" || final_status == "executing" {
                    exec_row.try_get::<Option<DateTime<Utc>>, &str>("execution_started_at")
                        .ok()
                        .flatten()
                        .map(|started| started + Duration::minutes(5))
                } else if final_status == "completed" {
                    exec_row.try_get::<Option<DateTime<Utc>>, &str>("execution_completed_at")
                        .ok()
                        .flatten()
                } else {
                    None
                };
                (final_status, progress, current_step, estimated_completion)
            }
            (None, Some(state_row), _) => {
                let status: String = state_row
                    .try_get("status")
                    .unwrap_or_else(|_| "unknown".to_string());
                let progress = match status.as_str() {
                    "pending" => 0,
                    "running" | "executing" => 50,
                    "completed" => 100,
                    "failed" | "cancelled" => 0,
                    "paused" => 50,
                    _ => 0,
                };
                let current_step = if let Ok(Some(state_data)) = state_row.try_get::<Option<serde_json::Value>, &str>("state_data") {
                    state_data
                        .get("current_step")
                        .or_else(|| state_data.get("current_phase"))
                        .and_then(|s| s.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| status.clone())
                } else {
                    status.clone()
                };
                (status, progress, current_step, None)
            }
            (None, None, Some(plan_row)) => {
                let status: String = plan_row
                    .try_get("state")
                    .unwrap_or_else(|_| "unknown".to_string());
                let progress = if let Ok(Some(milestones_json)) = plan_row.try_get::<Option<serde_json::Value>, &str>("milestones") {
                    if let Some(milestones_array) = milestones_json.as_array() {
                        let total = milestones_array.len();
                        let completed = milestones_array.iter()
                            .filter(|m| {
                                m.get("state")
                                    .and_then(|s| s.as_str())
                                    .map(|s| s == "completed")
                                    .unwrap_or(false)
                            })
                            .count();
                        if total > 0 {
                            ((completed as f64 / total as f64) * 100.0) as u8
                        } else {
                            0
                        }
                    } else {
                        0
                    }
                } else {
                    0
                };
                let estimated_completion = plan_row.try_get::<Option<DateTime<Utc>>, &str>("completed_at")
                    .ok()
                    .flatten();
                (status, progress, status.clone(), estimated_completion)
            }
            (None, None, None) => {
                // No execution, state, or plan found - check if task exists
                let task_row = sqlx::query(
                    r#"
                    SELECT status, created_at, updated_at
                    FROM tasks
                    WHERE id = $1
                    "#,
                )
                .bind(task_id)
                .fetch_optional(db_client.pool())
                .await
                .map_err(|e| anyhow::anyhow!("Failed to query task: {}", e))?;

                match task_row {
                    Some(row) => {
                        let status: String = row
                            .try_get("status")
                            .unwrap_or_else(|_| "unknown".to_string());
                        let progress = match status.as_str() {
                            "pending" => 0,
                            "in_progress" => 50,
                            "completed" => 100,
                            "failed" => 0,
                            _ => 0,
                        };
                        (status, progress, "pending".to_string(), None)
                    }
                    None => {
                        return Err(anyhow::anyhow!("Task not found: {}", task_id));
                    }
                }
            }
        };

        Ok(ExecutionProgress {
            task_id,
            status,
            progress,
            current_step,
            estimated_completion,
        })
    }
}

/// Execution progress (stub)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionProgress {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub status: String,
    pub progress: u8,
    pub current_step: String,
    pub estimated_completion: Option<DateTime<Utc>>,
}

/// Orchestrator (stub)
#[derive(Debug, Clone, JsonSchema)]
pub struct Orchestrator {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub name: String,
    pub status: String,
}

impl Orchestrator {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            status: "active".to_string(),
        }
    }
}

/// Provenance response
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProvenanceResponse {
    #[schemars(with = "String")]
    pub id: Uuid,
    #[schemars(with = "String")]
    pub verdict_id: Uuid,
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub decision: Value,
    pub consensus_score: f64,
    pub caws_compliance: Value,
    pub git_commit_hash: Option<String>,
    pub git_trailer: String,
    pub signature: String,
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    pub metadata: Value,
}

/// Dashboard diff summary
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DashboardDiffSummary {
    #[schemars(with = "String")]
    pub id: Uuid,
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub diff_type: String,
    pub summary: String,
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    pub iteration: u32,
    pub file_path: String,
    pub change_type: String,
    pub lines_added: u32,
    pub lines_removed: u32,
    pub diff_preview: String,
}

/// Waiver approval request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WaiverApprovalRequest {
    #[schemars(with = "String")]
    pub waiver_id: Uuid,
    pub approved_by: String,
    pub approval_notes: Option<String>,
}

/// Waiver request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WaiverRequest {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub title: String,
    pub reason: String,
    pub description: String,
    pub gates: Vec<String>,
    pub approved_by: String,
    pub impact_level: String,
    pub mitigation_plan: String,
    #[schemars(with = "String")]
    pub expires_at: DateTime<Utc>,
    pub metadata: Value,
}

/// Waiver response
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WaiverResponse {
    #[schemars(with = "String")]
    pub id: Uuid,
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub title: String,
    pub reason: String,
    pub description: String,
    pub gates: Vec<String>,
    pub approved_by: String,
    pub impact_level: String,
    pub mitigation_plan: String,
    #[schemars(with = "String")]
    pub expires_at: DateTime<Utc>,
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
    #[schemars(with = "String")]
    pub updated_at: DateTime<Utc>,
    pub status: String,
    pub metadata: Value,
}

/// Task result response
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskResultResponse {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub status: String,
    pub result: Option<Value>,
    pub working_spec: Option<WorkingSpec>,
    pub artifacts: Option<ExecutionArtifacts>,
    pub quality_report: Option<QualityReport>,
    pub error_message: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Saved query response
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SavedQueryResponse {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub name: String,
    pub query_text: String,
    pub description: Option<String>,
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
    #[schemars(with = "String")]
    pub updated_at: DateTime<Utc>,
}

/// Save query request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SaveQueryRequest {
    pub name: String,
    pub query_text: String,
    pub description: Option<String>,
}

/// Task status response
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskStatusResponse {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub status: String,
    pub progress: Option<f64>,
    pub progress_percentage: Option<f64>,
    pub current_phase: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
    #[schemars(with = "String")]
    pub updated_at: DateTime<Utc>,
    pub quality_score: Option<f64>,
}

/// Dashboard iteration summary
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DashboardIterationSummary {
    #[schemars(with = "String")]
    pub iteration_id: Uuid,
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub iteration_number: u32,
    pub iteration: u32,
    pub status: String,
    pub progress: f64,
    pub score: f64,
    pub stop_reason: Option<String>,
    pub file_changes: u32,
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    pub model_used: String,
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Dashboard task summary
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DashboardTaskSummary {
    pub total_tasks: u64,
    pub active_tasks: u64,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub success_rate: f64,
    #[schemars(with = "String")]
    pub average_completion_time: Option<Duration>,
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub description: String,
    pub status: String,
    pub current_iteration: usize,
    pub total_iterations: usize,
    pub score: Option<f64>,
    pub execution_mode: String,
    #[schemars(with = "String")]
    pub start_time: DateTime<Utc>,
    #[schemars(with = "String")]
    pub last_update: DateTime<Utc>,
    pub iterations: Vec<DashboardIterationSummary>,
}
