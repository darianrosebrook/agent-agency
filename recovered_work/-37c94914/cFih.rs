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

/// REST API server
pub struct RestApi {
    config: ApiConfig,
    orchestrator: Arc<Orchestrator>,
    progress_tracker: Arc<ProgressTracker>,
    active_tasks: Arc<RwLock<HashMap<Uuid, TaskState>>>,
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
    ) -> Self {
        Self {
            config,
            orchestrator,
            progress_tracker,
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create the Axum router with all endpoints
    pub fn create_router(&self) -> Router {
        let state = ApiState {
            api: Arc::new(self.clone()),
        };

        Router::new()
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
            .route("/tasks", get(list_tasks))
            .route("/metrics", get(get_metrics))
            .route("/dashboard/tasks/:task_id", get(get_dashboard_data))
            .route("/dashboard/tasks/:task_id/diffs/:iteration", get(get_diff_summary))
            .with_state(state)
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

pub type Result<T> = std::result::Result<T, ApiError>;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Task not found: {0}")]
    TaskNotFound(Uuid),

    #[error("Task execution failed: {0}")]
    ExecutionError(String),

    #[error("Internal server error: {0}")]
    InternalError(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}

// Axum error response conversion
impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            ApiError::TaskNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::ExecutionError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ApiError::InternalError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
            ApiError::InvalidRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
        };

        let body = serde_json::json!({
            "error": message,
            "code": format!("{:?}", self).split('(').next().unwrap_or("Unknown")
        });

        (status, Json(body)).into_response()
    }
}
