//! REST API Server Implementation
//!
//! Contains the main RestApi struct and core business logic methods
//! for task management, execution, and API operations.

use std::collections::HashMap;
use std::sync::Arc;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::orchestration::orchestrate::Orchestrator;
use crate::orchestration::planning::types::{WorkingSpec, ExecutionArtifacts};
use crate::orchestration::quality::QualityReport;
use crate::orchestration::tracking::{ProgressTracker, ExecutionProgress};
use crate::self_prompting_agent::loop_controller::{SelfPromptingLoop, SelfPromptingEvent, ExecutionMode};
use agent_agency_database::DatabaseClient;

use super::types::*;
use super::errors::{ApiError, Result};
use super::middleware;

/// REST API server
#[derive(Clone)]
pub struct RestApi {
    config: ApiConfig,
    orchestrator: Arc<Orchestrator>,
    progress_tracker: Arc<ProgressTracker>,
    active_tasks: Arc<RwLock<HashMap<Uuid, TaskState>>>,
    db_client: Arc<DatabaseClient>,
}

#[derive(Debug, Clone)]
struct TaskState {
    status: TaskStatus,
    working_spec: Option<WorkingSpec>,
    artifacts: Option<ExecutionArtifacts>,
    quality_report: Option<QualityReport>,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum TaskStatus {
    Pending,
    Planning,
    Executing,
    QualityCheck,
    Refining,
    Paused,
    Completed,
    Failed,
}

impl RestApi {
    pub fn new(
        config: ApiConfig,
        orchestrator: Arc<Orchestrator>,
        progress_tracker: Arc<ProgressTracker>,
        db_client: Arc<DatabaseClient>,
    ) -> Self {
        Self {
            config,
            orchestrator,
            progress_tracker,
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            db_client,
        }
    }

    /// Create the Axum router with all endpoints
    pub fn create_router(&self) -> Router {
        let state = ApiState {
            api: Arc::new(self.clone()),
        };

        let mut router = Router::new()
            .route("/health", get(health_check))
            .route("/tasks", post(submit_task))
            .route("/tasks/:task_id", get(get_task_status))
            .route("/tasks/:task_id/result", get(get_task_result))
            .route("/tasks/:task_id/cancel", post(cancel_task))
            .route("/tasks/:task_id/pause", post(pause_task))
            .route("/tasks/:task_id/resume", post(resume_task))
            .route("/queries", get(list_saved_queries))
            .route("/queries", post(save_query))
            .route("/queries/:query_id", delete(delete_saved_query))
            .route("/waivers", get(list_waivers))
            .route("/waivers", post(create_waiver))
            .route("/waivers/:waiver_id/approve", post(approve_waiver))
            .route("/tasks/:task_id/provenance", get(get_task_provenance))
            .route("/provenance", get(list_provenance_records))
            .route("/provenance/link", post(link_provenance_to_commit))
            .route("/provenance/verify/:commit_hash", get(verify_provenance_trailer))
            .route("/provenance/commit/:commit_hash", get(get_provenance_by_commit))
            .route("/slos", get(list_slos))
            .route("/slos/:slo_name/status", get(get_slo_status))
            .route("/slos/:slo_name/measurements", get(get_slo_measurements))
            .route("/slo-alerts", get(list_slo_alerts))
            .route("/slo-alerts/:alert_id/acknowledge", post(acknowledge_slo_alert))
            .route("/tasks", get(list_tasks))
            .route("/metrics", get(get_metrics))
            .route("/dashboard/tasks/:task_id", get(get_dashboard_data))
            .route("/dashboard/tasks/:task_id/diffs/:iteration", get(get_diff_summary))
            .with_state(state);

        // Add API key authentication middleware if required
        if self.config.require_api_key {
            let api_keys = self.config.api_keys.clone();
            router = router.layer(axum::middleware::from_fn(move |headers: axum::http::HeaderMap, request: axum::http::Request<_>, next: axum::middleware::Next<_>| async move {
                match middleware::api_key_auth(headers, api_keys.clone()).await {
                    Ok(_) => Ok(next.run(request).await),
                    Err(status) => Err(status),
                }
            }));
        }

        router
    }

    /// Submit a task for autonomous execution
    pub async fn submit_task(&self, request: TaskSubmissionRequest) -> Result<TaskSubmissionResponse> {
        let task_id = Uuid::new_v4();

        // Initialize task state
        let task_state = TaskState {
            status: TaskStatus::Pending,
            working_spec: None,
            artifacts: None,
            quality_report: None,
            started_at: Utc::now(),
            completed_at: None,
            error_message: None,
        };

        {
            let mut active_tasks = self.active_tasks.write().await;
            active_tasks.insert(task_id, task_state);
        }

        // Start task execution in background
        let orchestrator = Arc::clone(&self.orchestrator);
        let active_tasks = Arc::clone(&self.active_tasks);
        let progress_tracker = Arc::clone(&self.progress_tracker);

        tokio::spawn(async move {
            if let Err(e) = Self::execute_task(
                task_id,
                request,
                orchestrator,
                active_tasks,
                progress_tracker,
            ).await {
                tracing::error!("Task execution failed for {}: {:?}", task_id, e);
                // Update task state with error
                let mut active_tasks = active_tasks.write().await;
                if let Some(task) = active_tasks.get_mut(&task_id) {
                    task.status = TaskStatus::Failed;
                    task.error_message = Some(format!("{:?}", e));
                    task.completed_at = Some(Utc::now());
                }
            }
        });

        Ok(TaskSubmissionResponse {
            task_id,
            status: "accepted".to_string(),
            message: "Task submitted for autonomous execution".to_string(),
            estimated_completion: Some(Utc::now() + chrono::Duration::minutes(30)), // Rough estimate
        })
    }

    /// Execute a task asynchronously
    async fn execute_task(
        task_id: Uuid,
        request: TaskSubmissionRequest,
        orchestrator: Arc<Orchestrator>,
        active_tasks: Arc<RwLock<HashMap<Uuid, TaskState>>>,
        progress_tracker: Arc<ProgressTracker>,
    ) -> Result<()> {
        // Update status to planning
        {
            let mut active_tasks = active_tasks.write().await;
            if let Some(task) = active_tasks.get_mut(&task_id) {
                task.status = TaskStatus::Planning;
            }
        }

        // Start progress tracking
        progress_tracker.start_execution(task_id, "user-submitted".to_string()).await
            .map_err(|e| ApiError::InternalError(format!("Progress tracking failed: {:?}", e)))?;

        // Execute the task with execution mode
        let execution_mode = request.execution_mode.as_deref();
        let result = orchestrator.orchestrate_task(&request.description, execution_mode).await
            .map_err(|e| ApiError::ExecutionError(format!("Task orchestration failed: {:?}", e)))?;

        // Update task state with results
        {
            let mut active_tasks = active_tasks.write().await;
            if let Some(task) = active_tasks.get_mut(&task_id) {
                task.status = TaskStatus::Completed;
                task.working_spec = Some(result.working_spec);
                task.artifacts = Some(result.artifacts);
                task.quality_report = result.quality_report;
                task.completed_at = Some(Utc::now());
            }
        }

        // Complete progress tracking
        progress_tracker.complete_execution(task_id, true).await
            .map_err(|e| ApiError::InternalError(format!("Progress completion failed: {:?}", e)))?;

        Ok(())
    }

    /// Get task status
    pub async fn get_task_status(&self, task_id: Uuid) -> Result<TaskStatusResponse> {
        let progress = self.progress_tracker.get_progress(task_id).await
            .map_err(|e| ApiError::InternalError(format!("Progress retrieval failed: {:?}", e)))?;

        let active_tasks = self.active_tasks.read().await;
        let task_state = active_tasks.get(&task_id);

        let response = if let Some(progress) = progress {
            TaskStatusResponse {
                task_id,
                status: format!("{:?}", progress.status).to_lowercase(),
                progress_percentage: progress.completion_percentage,
                current_phase: progress.current_phase,
                started_at: Some(progress.start_time),
                updated_at: Some(progress.last_update),
                quality_score: None, // Would come from quality report
            }
        } else if let Some(task_state) = task_state {
            TaskStatusResponse {
                task_id,
                status: format!("{:?}", task_state.status).to_lowercase(),
                progress_percentage: if matches!(task_state.status, TaskStatus::Completed) { 100.0 } else { 0.0 },
                current_phase: None,
                started_at: Some(task_state.started_at),
                updated_at: task_state.completed_at,
                quality_score: task_state.quality_report.as_ref().map(|r| r.overall_score),
            }
        } else {
            return Err(ApiError::TaskNotFound(task_id));
        };

        Ok(response)
    }

    /// Get task result
    pub async fn get_task_result(&self, task_id: Uuid) -> Result<TaskResultResponse> {
        let active_tasks = self.active_tasks.read().await;
        let task_state = active_tasks.get(&task_id)
            .ok_or_else(|| ApiError::TaskNotFound(task_id))?;

        Ok(TaskResultResponse {
            task_id,
            status: format!("{:?}", task_state.status).to_lowercase(),
            working_spec: task_state.working_spec.clone(),
            artifacts: task_state.artifacts.clone(),
            quality_report: task_state.quality_report.clone(),
            completed_at: task_state.completed_at,
            error_message: task_state.error_message.clone(),
        })
    }

    /// Pause a task
    pub async fn pause_task(&self, task_id: Uuid) -> Result<()> {
        // Update task state
        {
            let mut active_tasks = self.active_tasks.write().await;
            if let Some(task) = active_tasks.get_mut(&task_id) {
                if task.status != TaskStatus::Running {
                    return Err(ApiError::InvalidOperation("Can only pause running tasks".to_string()));
                }
                task.status = TaskStatus::Paused;
                task.updated_at = Utc::now();
            } else {
                return Err(ApiError::TaskNotFound(task_id));
            }
        }

        // Pause in progress tracker
        self.progress_tracker.pause_execution(task_id).await
            .map_err(|e| ApiError::InternalError(format!("Pause failed: {:?}", e)))?;

        Ok(())
    }

    /// Resume a paused task
    pub async fn resume_task(&self, task_id: Uuid) -> Result<()> {
        // Update task state
        {
            let mut active_tasks = self.active_tasks.write().await;
            if let Some(task) = active_tasks.get_mut(&task_id) {
                if task.status != TaskStatus::Paused {
                    return Err(ApiError::InvalidOperation("Can only resume paused tasks".to_string()));
                }
                task.status = TaskStatus::Running;
                task.updated_at = Utc::now();
            } else {
                return Err(ApiError::TaskNotFound(task_id));
            }
        }

        // Resume in progress tracker
        self.progress_tracker.resume_execution(task_id).await
            .map_err(|e| ApiError::InternalError(format!("Resume failed: {:?}", e)))?;

        Ok(())
    }

    /// Cancel a task
    pub async fn cancel_task(&self, task_id: Uuid) -> Result<()> {
        // Update task state
        {
            let mut active_tasks = self.active_tasks.write().await;
            if let Some(task) = active_tasks.get_mut(&task_id) {
                task.status = TaskStatus::Failed;
                task.error_message = Some("Task cancelled by user".to_string());
                task.completed_at = Some(Utc::now());
            } else {
                return Err(ApiError::TaskNotFound(task_id));
            }
        }

        // Cancel in progress tracker
        self.progress_tracker.cancel_execution(task_id).await
            .map_err(|e| ApiError::InternalError(format!("Cancellation failed: {:?}", e)))?;

        Ok(())
    }

    /// List saved queries
    pub async fn list_saved_queries(&self) -> Result<Vec<SavedQueryResponse>> {
        // Query saved queries from database
        let query = r#"
            SELECT id, name, query_text, created_at, updated_at
            FROM saved_queries
            ORDER BY created_at DESC
        "#;

        let rows = self.db_client
            .query(query, &[])
            .await
            .map_err(|e| ApiError::DatabaseError(format!("Failed to list queries: {}", e)))?;

        let mut queries = Vec::new();
        for row in rows {
            let id: Uuid = row.get("id");
            let name: String = row.get("name");
            let query_text: String = row.get("query_text");
            let created_at: DateTime<Utc> = row.get("created_at");
            let updated_at: DateTime<Utc> = row.get("updated_at");

            queries.push(SavedQueryResponse {
                id,
                name,
                query_text,
                created_at: created_at.to_rfc3339(),
                updated_at: updated_at.to_rfc3339(),
            });
        }

        Ok(queries)
    }

    /// Save a query
    pub async fn save_query(&self, request: SaveQueryRequest) -> Result<SavedQueryResponse> {
        // Insert saved query into database
        let query = r#"
            INSERT INTO saved_queries (name, query_text, created_at, updated_at)
            VALUES ($1, $2, NOW(), NOW())
            RETURNING id, created_at, updated_at
        "#;

        let row = self.db_client
            .query_one(
                query,
                &[&request.name, &request.query_text],
            )
            .await
            .map_err(|e| ApiError::DatabaseError(format!("Failed to save query: {}", e)))?;

        let id: Uuid = row.get("id");
        let created_at: DateTime<Utc> = row.get("created_at");
        let updated_at: DateTime<Utc> = row.get("updated_at");

        Ok(SavedQueryResponse {
            id,
            name: request.name,
            query_text: request.query_text,
            created_at: created_at.to_rfc3339(),
            updated_at: updated_at.to_rfc3339(),
        })
    }

    /// Delete a saved query
    pub async fn delete_saved_query(&self, query_id: Uuid) -> Result<()> {
        // Delete saved query from database
        let query = r#"
            DELETE FROM saved_queries
            WHERE id = $1
        "#;

        let rows_affected = self.db_client
            .execute(query, &[&query_id])
            .await
            .map_err(|e| ApiError::DatabaseError(format!("Failed to delete query: {}", e)))?;

        if rows_affected == 0 {
            return Err(ApiError::NotFound(format!("Query with ID {} not found", query_id)));
        }

        Ok(())
    }

    /// List all tasks
    pub async fn list_tasks(&self) -> Result<Vec<TaskStatusResponse>> {
        let active_tasks = self.active_tasks.read().await;
        let mut responses = Vec::new();

        for (task_id, task_state) in active_tasks.iter() {
            let progress = self.progress_tracker.get_progress(*task_id).await
                .map_err(|e| ApiError::InternalError(format!("Progress retrieval failed: {:?}", e)))?;

            let response = TaskStatusResponse {
                task_id: *task_id,
                status: format!("{:?}", task_state.status).to_lowercase(),
                progress_percentage: progress.as_ref()
                    .map(|p| p.completion_percentage)
                    .unwrap_or(if matches!(task_state.status, TaskStatus::Completed) { 100.0 } else { 0.0 }),
                current_phase: progress.as_ref().and_then(|p| p.current_phase.clone()),
                started_at: Some(task_state.started_at),
                updated_at: progress.as_ref().map(|p| p.last_update).or(task_state.completed_at),
                quality_score: task_state.quality_report.as_ref().map(|r| r.overall_score),
            };

            responses.push(response);
        }

        Ok(responses)
    }

    /// Get system metrics
    pub async fn get_metrics(&self) -> Result<HashMap<String, serde_json::Value>> {
        let active_tasks = self.active_tasks.read().await;
        let active_count = active_tasks.len();
        let completed_count = active_tasks.values()
            .filter(|t| matches!(t.status, TaskStatus::Completed))
            .count();
        let failed_count = active_tasks.values()
            .filter(|t| matches!(t.status, TaskStatus::Failed))
            .count();

        let mut metrics = HashMap::new();
        metrics.insert("active_tasks".to_string(), serde_json::json!(active_count));
        metrics.insert("completed_tasks".to_string(), serde_json::json!(completed_count));
        metrics.insert("failed_tasks".to_string(), serde_json::json!(failed_count));
        metrics.insert("success_rate".to_string(), serde_json::json!(
            if completed_count + failed_count > 0 {
                completed_count as f64 / (completed_count + failed_count) as f64
            } else {
                1.0
            }
        ));

        Ok(metrics)
    }

    /// Get dashboard data for a task
    pub async fn get_dashboard_data(&self, task_id: Uuid) -> Result<DashboardTaskSummary> {
        let active_tasks = self.active_tasks.read().await;
        let task_state = active_tasks.get(&task_id)
            .ok_or_else(|| ApiError::TaskNotFound(task_id))?;

        let progress = self.progress_tracker.get_progress(task_id).await
            .map_err(|e| ApiError::InternalError(format!("Progress retrieval failed: {:?}", e)))?;

        // Build iteration summaries (placeholder - would come from actual iteration data)
        let iterations = vec![
            DashboardIterationSummary {
                iteration: 1,
                score: 85.0,
                stop_reason: "Quality plateau reached".to_string(),
                file_changes: 3,
                timestamp: Utc::now(),
                model_used: "gpt-4-turbo".to_string(),
            }
        ];

        Ok(DashboardTaskSummary {
            task_id,
            description: "Task description".to_string(), // TODO: Add task description to TaskState
            status: format!("{:?}", task_state.status).to_lowercase(),
            current_iteration: progress.current_iteration as usize,
            total_iterations: progress.total_iterations as usize,
            score: task_state.quality_report.as_ref().map(|r| r.overall_score),
            execution_mode: "auto".to_string(), // Placeholder
            start_time: task_state.started_at,
            last_update: task_state.completed_at.unwrap_or_else(|| Utc::now()),
            iterations,
        })
    }

    /// Get diff summary for a task iteration
    pub async fn get_diff_summary(&self, task_id: Uuid, iteration: usize) -> Result<Vec<DashboardDiffSummary>> {
        let active_tasks = self.active_tasks.read().await;
        let _task_state = active_tasks.get(&task_id)
            .ok_or_else(|| ApiError::TaskNotFound(task_id))?;

        // Placeholder diff data - would come from actual artifacts
        Ok(vec![
            DashboardDiffSummary {
                iteration,
                file_path: "src/main.rs".to_string(),
                change_type: "modified".to_string(),
                lines_added: 15,
                lines_removed: 5,
                diff_preview: "@@ -10,5 +10,15 @@\n- old code\n+ new code".to_string(),
            }
        ])
    }
}

#[derive(Clone)]
struct ApiState {
    api: Arc<RestApi>,
}
