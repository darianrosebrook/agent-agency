//! REST API Interface for Autonomous Task Execution
//!
//! Provides HTTP endpoints for submitting tasks, monitoring execution,
//! and retrieving results in a tool-agnostic manner.

use std::collections::HashMap;
use std::sync::Arc;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::orchestration::orchestrate::Orchestrator;
use crate::orchestration::planning::types::{WorkingSpec, ExecutionArtifacts};
use crate::orchestration::tracking::{ProgressTracker, ExecutionProgress};
use crate::orchestration::quality::QualityReport;
use crate::self_prompting_agent::loop_controller::{SelfPromptingLoop, SelfPromptingEvent, ExecutionMode};
use agent_agency_database::DatabaseClient;

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Server host
    pub host: String,
    /// Server port
    pub port: u16,
    /// Enable CORS
    pub enable_cors: bool,
    /// API key authentication required
    pub require_api_key: bool,
    /// API keys (if authentication enabled)
    pub api_keys: Vec<String>,
    /// Rate limiting enabled
    pub enable_rate_limiting: bool,
    /// Rate limit requests per minute
    pub rate_limit_per_minute: u32,
}

/// Task submission request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSubmissionRequest {
    /// Natural language task description
    pub description: String,
    /// Execution mode (strict/auto/dry-run)
    pub execution_mode: Option<String>,
    /// Risk tier override (optional)
    pub risk_tier: Option<String>,
    /// Additional context or requirements
    pub context: Option<String>,
    /// Priority level
    pub priority: Option<String>,
    /// Deadline (optional)
    pub deadline: Option<DateTime<Utc>>,
}

/// Task submission response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSubmissionResponse {
    pub task_id: Uuid,
    pub status: String,
    pub message: String,
    pub estimated_completion: Option<DateTime<Utc>>,
}

/// Task status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatusResponse {
    pub task_id: Uuid,
    pub status: String,
    pub progress_percentage: f32,
    pub current_phase: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub quality_score: Option<f64>,
}

/// Task result response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResultResponse {
    pub task_id: Uuid,
    pub status: String,
    pub working_spec: Option<WorkingSpec>,
    pub artifacts: Option<ExecutionArtifacts>,
    pub quality_report: Option<QualityReport>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SaveQueryRequest {
    pub name: String,
    pub query_text: String,
}

#[derive(Debug, Serialize)]
pub struct SavedQueryResponse {
    pub id: Uuid,
    pub name: String,
    pub query_text: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Dashboard iteration summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardIterationSummary {
    pub iteration: usize,
    pub score: f64,
    pub stop_reason: String,
    pub file_changes: usize,
    pub timestamp: DateTime<Utc>,
    pub model_used: String,
}

/// Dashboard task summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardTaskSummary {
    pub task_id: Uuid,
    pub description: String,
    pub status: String,
    pub current_iteration: usize,
    pub total_iterations: usize,
    pub score: Option<f64>,
    pub execution_mode: String,
    pub start_time: DateTime<Utc>,
    pub last_update: DateTime<Utc>,
    pub iterations: Vec<DashboardIterationSummary>,
}

/// Diff summary for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardDiffSummary {
    pub iteration: usize,
    pub file_path: String,
    pub change_type: String,
    pub lines_added: usize,
    pub lines_removed: usize,
    pub diff_preview: String,
}

/// Waiver request for creating new waivers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaiverRequest {
    pub title: String,
    pub reason: String,
    pub description: String,
    pub gates: Vec<String>,
    pub approved_by: String,
    pub impact_level: String,
    pub mitigation_plan: String,
    pub expires_at: DateTime<Utc>,
}

/// Waiver response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaiverResponse {
    pub id: Uuid,
    pub title: String,
    pub reason: String,
    pub description: String,
    pub gates: Vec<String>,
    pub approved_by: String,
    pub impact_level: String,
    pub mitigation_plan: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: String,
    pub metadata: serde_json::Value,
}

/// Waiver approval request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaiverApprovalRequest {
    pub approver: String,
    pub justification: Option<String>,
}

/// Provenance response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceResponse {
    pub id: Uuid,
    pub verdict_id: Uuid,
    pub task_id: Uuid,
    pub decision: serde_json::Value,
    pub consensus_score: f32,
    pub caws_compliance: serde_json::Value,
    pub git_commit_hash: Option<String>,
    pub git_trailer: String,
    pub signature: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// REST API server
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
                // Extract API key from Authorization header (Bearer token) or X-API-Key header
                let api_key = headers
                    .get("authorization")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|auth| auth.strip_prefix("Bearer "))
                    .or_else(|| {
                        headers
                            .get("x-api-key")
                            .and_then(|h| h.to_str().ok())
                    });

                match api_key {
                    Some(key) => {
                        if api_keys.contains(&key.to_string()) {
                            Ok(next.run(request).await)
                        } else {
                            Err(axum::http::StatusCode::UNAUTHORIZED)
                        }
                    }
                    None => Err(axum::http::StatusCode::UNAUTHORIZED),
                }
            }));
        }

        router
    }

    /// Submit a task for autonomous execution
    pub async fn submit_task(&self, request: TaskSubmissionRequest) -> Result<TaskSubmissionResponse, ApiError> {
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
    ) -> Result<(), ApiError> {
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

        // Execute the task
        let result = orchestrator.orchestrate_task(&request.description).await
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
    pub async fn get_task_status(&self, task_id: Uuid) -> Result<TaskStatusResponse, ApiError> {
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
    pub async fn get_task_result(&self, task_id: Uuid) -> Result<TaskResultResponse, ApiError> {
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
    pub async fn pause_task(&self, task_id: Uuid) -> Result<(), ApiError> {
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

        // TODO: Implement pause in orchestrator when available
        // For now, just update local state

        Ok(())
    }

    /// Resume a paused task
    pub async fn resume_task(&self, task_id: Uuid) -> Result<(), ApiError> {
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

        // TODO: Implement resume in orchestrator when available
        // For now, just update local state

        Ok(())
    }

    /// Cancel a task
    pub async fn cancel_task(&self, task_id: Uuid) -> Result<(), ApiError> {
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
    pub async fn list_saved_queries(&self) -> Result<Vec<SavedQueryResponse>, ApiError> {
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
    pub async fn save_query(&self, request: SaveQueryRequest) -> Result<SavedQueryResponse, ApiError> {
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
    pub async fn delete_saved_query(&self, query_id: Uuid) -> Result<(), ApiError> {
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
    pub async fn list_tasks(&self) -> Result<Vec<TaskStatusResponse>, ApiError> {
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
    pub async fn get_metrics(&self) -> Result<HashMap<String, serde_json::Value>, ApiError> {
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
    pub async fn get_dashboard_data(&self, task_id: Uuid) -> Result<DashboardTaskSummary, ApiError> {
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
            description: task_state.task_description.clone(),
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
    pub async fn get_diff_summary(&self, task_id: Uuid, iteration: usize) -> Result<Vec<DashboardDiffSummary>, ApiError> {
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

// Axum handlers
async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "agent-agency-v3-api",
        "version": "1.0.0"
    }))
}

async fn submit_task(
    State(state): State<ApiState>,
    Json(request): Json<TaskSubmissionRequest>,
) -> Result<Json<TaskSubmissionResponse>, ApiError> {
    let response = state.api.submit_task(request).await?;
    Ok(Json(response))
}

async fn get_task_status(
    State(state): State<ApiState>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<TaskStatusResponse>, ApiError> {
    let response = state.api.get_task_status(task_id).await?;
    Ok(Json(response))
}

async fn get_task_result(
    State(state): State<ApiState>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<TaskResultResponse>, ApiError> {
    let response = state.api.get_task_result(task_id).await?;
    Ok(Json(response))
}

async fn pause_task(
    State(state): State<ApiState>,
    Path(task_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.api.pause_task(task_id).await?;
    Ok(StatusCode::OK)
}

async fn resume_task(
    State(state): State<ApiState>,
    Path(task_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.api.resume_task(task_id).await?;
    Ok(StatusCode::OK)
}

async fn cancel_task(
    State(state): State<ApiState>,
    Path(task_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.api.cancel_task(task_id).await?;
    Ok(StatusCode::OK)
}

async fn list_saved_queries(
    State(state): State<ApiState>,
) -> Result<Json<Vec<SavedQueryResponse>>, ApiError> {
    let queries = state.api.list_saved_queries().await?;
    Ok(Json(queries))
}

async fn save_query(
    State(state): State<ApiState>,
    Json(request): Json<SaveQueryRequest>,
) -> Result<Json<SavedQueryResponse>, ApiError> {
    let response = state.api.save_query(request).await?;
    Ok(Json(response))
}

async fn delete_saved_query(
    State(state): State<ApiState>,
    Path(query_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.api.delete_saved_query(query_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_tasks(
    State(state): State<ApiState>,
) -> Result<Json<Vec<TaskStatusResponse>>, ApiError> {
    let tasks = state.api.list_tasks().await?;
    Ok(Json(tasks))
}

async fn get_metrics(
    State(state): State<ApiState>,
) -> Result<Json<HashMap<String, serde_json::Value>>, ApiError> {
    let metrics = state.api.get_metrics().await?;
    Ok(Json(metrics))
}

async fn get_dashboard_data(
    State(state): State<ApiState>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<DashboardTaskSummary>, ApiError> {
    let dashboard_data = state.api.get_dashboard_data(task_id).await?;
    Ok(Json(dashboard_data))
}

async fn get_diff_summary(
    State(state): State<ApiState>,
    Path((task_id, iteration)): Path<(Uuid, usize)>,
) -> Result<Json<Vec<DashboardDiffSummary>>, ApiError> {
    let diff_summary = state.api.get_diff_summary(task_id, iteration).await?;
    Ok(Json(diff_summary))
}

async fn list_waivers(
    State(state): State<ApiState>,
) -> Result<Json<Vec<WaiverResponse>>, ApiError> {
    // Query waivers from database
    let query = r#"
        SELECT
            id, title, reason, description, gates, approved_by,
            impact_level, mitigation_plan, expires_at, created_at,
            updated_at, status, metadata
        FROM waivers
        ORDER BY created_at DESC
    "#;

    let rows = state.api.db_client
        .query(query, &[])
        .await
        .map_err(|e| ApiError::DatabaseError(format!("Failed to list waivers: {}", e)))?;

    let mut waivers = Vec::new();
    for row in rows {
        let gates: Vec<String> = row.get("gates");

        waivers.push(WaiverResponse {
            id: row.get("id"),
            title: row.get("title"),
            reason: row.get("reason"),
            description: row.get("description"),
            gates,
            approved_by: row.get("approved_by"),
            impact_level: row.get("impact_level"),
            mitigation_plan: row.get("mitigation_plan"),
            expires_at: row.get("expires_at"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            status: row.get("status"),
            metadata: row.get("metadata"),
        });
    }

    Ok(Json(waivers))
}

async fn create_waiver(
    State(state): State<ApiState>,
    Json(request): Json<WaiverRequest>,
) -> Result<Json<WaiverResponse>, ApiError> {
    // Insert waiver into database
    let insert_query = r#"
        INSERT INTO waivers (
            title, reason, description, gates, approved_by,
            impact_level, mitigation_plan, expires_at, metadata
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, created_at, updated_at
    "#;

    let gates_array = request.gates.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
    let metadata = serde_json::json!({
        "created": true,
        "mitigation_plan": request.mitigation_plan
    });

    let row = state.api.db_client
        .query_one(
            insert_query,
            &[
                &request.title,
                &request.reason,
                &request.description,
                &gates_array,
                &request.approved_by,
                &request.impact_level,
                &request.mitigation_plan,
                &request.expires_at,
                &metadata,
            ],
        )
        .await
        .map_err(|e| ApiError::DatabaseError(format!("Failed to create waiver: {}", e)))?;

    let id: Uuid = row.get("id");
    let created_at: DateTime<Utc> = row.get("created_at");
    let updated_at: DateTime<Utc> = row.get("updated_at");

    let waiver = WaiverResponse {
        id,
        title: request.title,
        reason: request.reason,
        description: request.description,
        gates: request.gates,
        approved_by: request.approved_by,
        impact_level: request.impact_level,
        mitigation_plan: request.mitigation_plan,
        expires_at: request.expires_at,
        created_at,
        updated_at,
        status: "active".to_string(),
        metadata,
    };

    Ok(Json(waiver))
}

async fn approve_waiver(
    State(state): State<ApiState>,
    Path(waiver_id): Path<String>,
    Json(request): Json<WaiverApprovalRequest>,
) -> Result<StatusCode, ApiError> {
    // Update waiver status in database
    let update_query = r#"
        UPDATE waivers
        SET status = 'active',
            updated_at = NOW(),
            metadata = metadata || $1::jsonb
        WHERE id = $2::uuid
        RETURNING id, title, gates, expires_at
    "#;

    let metadata = serde_json::json!({
        "approved_at": chrono::Utc::now(),
        "approved_by": request.approver,
        "justification": request.justification
    });

    let waiver_uuid = Uuid::parse_str(&waiver_id)
        .map_err(|_| ApiError::InvalidRequest("Invalid waiver ID format".to_string()))?;

    let row = state.api.db_client
        .query_one(
            update_query,
            &[&metadata, &waiver_uuid],
        )
        .await
        .map_err(|e| ApiError::DatabaseError(format!("Failed to approve waiver: {}", e)))?;

    let title: String = row.get("title");
    let gates: Vec<String> = row.get("gates");

    println!("✅ Waiver '{}' approved by {} for gates: {:?}", title, request.approver, gates);
    Ok(StatusCode::OK)
}

async fn get_task_provenance(
    State(state): State<ApiState>,
    Path(task_id): Path<String>,
) -> Result<Json<ProvenanceResponse>, ApiError> {
    // For now, return a mock response - in a real implementation this would query the provenance service
    let task_uuid = Uuid::parse_str(&task_id)
        .map_err(|_| ApiError::InvalidRequest("Invalid task ID format".to_string()))?;

    let mock_provenance = ProvenanceResponse {
        id: Uuid::new_v4(),
        verdict_id: Uuid::new_v4(),
        task_id: task_uuid,
        decision: serde_json::json!({"type": "accept", "confidence": 0.95, "summary": "Task accepted with high confidence"}),
        consensus_score: 0.95,
        caws_compliance: serde_json::json!({"is_compliant": true, "compliance_score": 0.95, "violations": [], "waivers_used": []}),
        git_commit_hash: Some("abc123".to_string()),
        git_trailer: format!("Provenance: CAWS-VERDICT-{}", Uuid::new_v4()),
        signature: "mock-signature".to_string(),
        timestamp: Utc::now(),
        metadata: serde_json::json!({"working_spec_id": "SPEC-001", "evidence_count": 5, "debate_rounds": 2}),
    };

    Ok(Json(mock_provenance))
}

/// List provenance records
async fn list_provenance_records(State(state): State<ApiState>) -> Result<Json<Vec<serde_json::Value>>, ApiError> {
    // Query provenance records from database
    let query = r#"
        SELECT
            verdict_id, decision_type, consensus_score, git_trailer,
            timestamp, created_at
        FROM provenance_records
        ORDER BY timestamp DESC
        LIMIT 50
    "#;

    let rows = state.api.db_client
        .query(query, &[])
        .await
        .map_err(|e| ApiError::DatabaseError(format!("Failed to list provenance records: {}", e)))?;

    let mut records = Vec::new();
    for row in rows {
        let record = serde_json::json!({
            "verdict_id": row.get::<String, _>("verdict_id"),
            "decision": {
                "decision_type": row.get::<String, _>("decision_type")
            },
            "consensus_score": row.get::<f64, _>("consensus_score"),
            "git_trailer": row.get::<String, _>("git_trailer"),
            "timestamp": row.get::<DateTime<Utc>, _>("timestamp").to_rfc3339(),
            "created_at": row.get::<DateTime<Utc>, _>("created_at").to_rfc3339()
        });
        records.push(record);
    }

    Ok(Json(records))
}

/// Link provenance record to git commit
async fn link_provenance_to_commit(
    State(state): State<ApiState>,
    Json(request): Json<LinkProvenanceRequest>,
) -> Result<StatusCode, ApiError> {
    // Update provenance record with commit hash
    let update_query = r#"
        UPDATE provenance_records
        SET git_commit_hash = $2, updated_at = NOW()
        WHERE verdict_id::text = $1
    "#;

    let rows_affected = state.api.db_client
        .execute(update_query, &[&request.provenance_id, &request.commit_hash])
        .await
        .map_err(|e| ApiError::DatabaseError(format!("Failed to link provenance: {}", e)))?;

    if rows_affected == 0 {
        return Err(ApiError::NotFound(format!("Provenance record {} not found", request.provenance_id)));
    }

    Ok(StatusCode::OK)
}

/// Verify provenance trailer in commit
async fn verify_provenance_trailer(
    State(state): State<ApiState>,
    Path(commit_hash): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Check if commit hash exists in provenance records
    let query = r#"SELECT git_trailer FROM provenance_records WHERE git_commit_hash = $1"#;

    let row = state.api.db_client
        .query_opt(query, &[&commit_hash])
        .await
        .map_err(|e| ApiError::DatabaseError(format!("Failed to verify trailer: {}", e)))?;

    let result = if let Some(row) = row {
        let trailer: String = row.get("git_trailer");
        serde_json::json!({
            "has_trailer": true,
            "trailer": trailer,
            "commit_hash": commit_hash
        })
    } else {
        serde_json::json!({
            "has_trailer": false,
            "commit_hash": commit_hash
        })
    };

    Ok(Json(result))
}

/// Get provenance record by commit hash
async fn get_provenance_by_commit(
    State(state): State<ApiState>,
    Path(commit_hash): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Query full provenance record by commit hash
    let query = r#"
        SELECT
            verdict_id, decision_type, consensus_score, git_trailer,
            timestamp, created_at, updated_at, decision_data, metadata
        FROM provenance_records
        WHERE git_commit_hash = $1
    "#;

    let row = state.api.db_client
        .query_opt(query, &[&commit_hash])
        .await
        .map_err(|e| ApiError::DatabaseError(format!("Failed to get provenance: {}", e)))?
        .ok_or_else(|| ApiError::NotFound(format!("No provenance record found for commit {}", commit_hash)))?;

    let record = serde_json::json!({
        "verdict_id": row.get::<String, _>("verdict_id"),
        "decision": {
            "decision_type": row.get::<String, _>("decision_type"),
            "decision_data": row.get::<serde_json::Value, _>("decision_data")
        },
        "consensus_score": row.get::<f64, _>("consensus_score"),
        "git_trailer": row.get::<String, _>("git_trailer"),
        "timestamp": row.get::<DateTime<Utc>, _>("timestamp").to_rfc3339(),
        "created_at": row.get::<DateTime<Utc>, _>("created_at").to_rfc3339(),
        "updated_at": row.get::<DateTime<Utc>, _>("updated_at").to_rfc3339(),
        "metadata": row.get::<serde_json::Value, _>("metadata")
    });

    Ok(Json(record))
}

/// Link provenance request
#[derive(Debug, Deserialize)]
pub struct LinkProvenanceRequest {
    pub provenance_id: String,
    pub commit_hash: String,
}

/// List all SLOs
async fn list_slos(State(state): State<ApiState>) -> Result<Json<Vec<serde_json::Value>>, ApiError> {
    // For now, return default SLOs - in a real implementation this would query the SLO tracker
    let default_slos = agent_agency_observability::slo::create_default_slos();

    let slos: Vec<serde_json::Value> = default_slos.into_iter()
        .map(|slo| serde_json::json!({
            "name": slo.name,
            "description": slo.description,
            "service": slo.service,
            "metric": slo.metric,
            "target": slo.target,
            "window_days": slo.window_days,
            "labels": slo.labels
        }))
        .collect();

    Ok(Json(slos))
}

/// Get SLO status
async fn get_slo_status(
    State(state): State<ApiState>,
    Path(slo_name): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // For now, return mock SLO status - in a real implementation this would query the SLO tracker
    let mock_status = serde_json::json!({
        "slo_name": slo_name,
        "target_value": 0.99,
        "current_value": 0.985,
        "compliance_percentage": 98.5,
        "remaining_budget": 0.015,
        "period_start": "2024-01-01T00:00:00Z",
        "period_end": "2024-01-31T23:59:59Z",
        "status": "AtRisk",
        "last_updated": chrono::Utc::now().to_rfc3339()
    });

    Ok(Json(mock_status))
}

/// Get SLO measurements
async fn get_slo_measurements(
    State(state): State<ApiState>,
    Path(slo_name): Path<String>,
) -> Result<Json<Vec<serde_json::Value>>, ApiError> {
    // For now, return mock measurements - in a real implementation this would query the database
    let mock_measurements = vec![
        serde_json::json!({
            "slo_name": slo_name,
            "timestamp": "2024-01-15T10:00:00Z",
            "value": 0.995,
            "sample_count": 1000,
            "good_count": 995,
            "bad_count": 5
        }),
        serde_json::json!({
            "slo_name": slo_name,
            "timestamp": "2024-01-15T11:00:00Z",
            "value": 0.985,
            "sample_count": 1000,
            "good_count": 985,
            "bad_count": 15
        }),
    ];

    Ok(Json(mock_measurements))
}

/// List SLO alerts
async fn list_slo_alerts(State(state): State<ApiState>) -> Result<Json<Vec<serde_json::Value>>, ApiError> {
    // For now, return mock SLO alerts - in a real implementation this would query active SLO alerts
    let mock_alerts = vec![
        serde_json::json!({
            "id": "slo-alert-001",
            "slo_name": "api_response_time",
            "title": "API Response Time SLO At Risk",
            "description": "API response time SLO is at 98.5%, below the 99% target",
            "severity": "warning",
            "status": "active",
            "current_value": 0.985,
            "threshold_value": 0.99,
            "triggered_at": "2024-01-15T11:30:00Z",
            "labels": {
                "service": "api",
                "component": "response_time"
            }
        }),
    ];

    Ok(Json(mock_alerts))
}

/// Acknowledge SLO alert
async fn acknowledge_slo_alert(
    State(state): State<ApiState>,
    Path(alert_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    // For now, just log acknowledgment - in a real implementation this would update the alert status
    println!("SLO Alert {} acknowledged", alert_id);
    Ok(StatusCode::OK)
}

pub type Result<T> = std::result::Result<T, ApiError>;

pub enum ApiError {
    #[error("Task not found: {0}")]
    TaskNotFound(Uuid),

    #[error("Task execution failed: {0}")]
    ExecutionError(String),

    #[error("Internal server error: {0}")]
    InternalError(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Authentication required")]
    Unauthorized,
}

// Axum error response conversion
impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            ApiError::TaskNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::ExecutionError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ApiError::InternalError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
            ApiError::InvalidRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "API key required".to_string()),
        };

        let body = serde_json::json!({
            "error": message,
            "code": format!("{:?}", self).split('(').next().unwrap_or("Unknown")
        });

        (status, Json(body)).into_response()
    }
}
