//! REST API Server Implementation
//!
//! Contains the main RestApi struct and core business logic methods
//! for task management, execution, and API operations.

use schemars::JsonSchema;
use chrono::{DateTime, Utc, TimeDelta};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use axum::{Router, routing::get};
use axum::response::IntoResponse;
use sqlx::Row;

use super::{ApiError, Result};
use super::types::{
    ApiConfig, TaskSubmissionRequest, TaskSubmissionResponse,
    TaskStatusResponse, TaskResultResponse, SavedQueryResponse, SaveQueryRequest,
    DashboardTaskSummary, DashboardDiffSummary
};
use agent_agency_contracts::{ExecutionArtifacts, WorkingSpec, QualityReport};
use crate::DatabaseClient;

// Stub types for compilation

#[derive(Debug, Clone, JsonSchema)]
pub struct ExecutionProgress {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub status: String,
    pub progress_percentage: f64,
    pub current_phase: String,
    #[schemars(with = "String")]

    pub started_at: DateTime<Utc>,
    #[schemars(with = "String")]

    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ProgressTracker {
    db_client: Arc<DatabaseClient>,
}

impl ProgressTracker {
    pub fn new(db_client: Arc<DatabaseClient>) -> Self {
        Self { db_client }
    }

    pub async fn start_execution(&self, _task_id: Uuid, _mode: String) -> Result<()> {
        // Execution tracking is handled by task_executions table
        // This method can be used for logging or additional tracking if needed
        Ok(())
    }

    pub async fn get_progress(&self, task_id: Uuid) -> Result<ExecutionProgress> {
        // Query task_executions table for execution status and timing
        let execution_row = sqlx::query(
            r#"
            SELECT status, execution_started_at, execution_completed_at, execution_time_ms
            FROM task_executions
            WHERE task_id = $1
            ORDER BY execution_started_at DESC
            LIMIT 1
            "#
        )
        .bind(task_id)
        .fetch_optional(self.db_client.pool())
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to query task execution progress: {}", e)))?;

        // Query task_execution_states for current state
        let state_row = sqlx::query(
            r#"
            SELECT status, last_updated, state_data
            FROM task_execution_states
            WHERE task_id = $1
            "#
        )
        .bind(task_id)
        .fetch_optional(self.db_client.pool())
        .await
        .map_err(|e| ApiError::InternalError(format!("Failed to query task execution state: {}", e)))?;

        // Determine status and progress from execution and state
        let (status, progress_percentage, current_phase, started_at, updated_at) = match (execution_row, state_row) {
            (Some(exec_row), Some(state_row)) => {
                let exec_status: String = exec_row.try_get("status")
                    .unwrap_or_else(|_| "unknown".to_string());
                let state_status: String = state_row.try_get("status")
                    .unwrap_or_else(|_| "unknown".to_string());
                
                let started: DateTime<Utc> = exec_row.try_get("execution_started_at")
                    .unwrap_or_else(|_| Utc::now());
                let updated: DateTime<Utc> = state_row.try_get("last_updated")
                    .unwrap_or_else(|_| Utc::now());
                
                // Use state status if available, otherwise execution status
                let final_status = if state_status != "unknown" { state_status } else { exec_status };
                
                // Calculate progress based on status
                let progress = match final_status.as_str() {
                    "pending" => 0.0,
                    "running" => {
                        // Estimate progress based on execution time if available
                        if let Ok(Some(_exec_time_ms)) = exec_row.try_get::<Option<i32>, &str>("execution_time_ms") {
                            // If we have execution time, estimate based on average task duration
                            // Assume average task takes 5 minutes, cap at 90% while running
                            let elapsed: TimeDelta = Utc::now() - started;
                            let elapsed_secs = elapsed.num_seconds();
                            (elapsed_secs as f64 / 300.0 * 90.0).min(90.0)
                        } else {
                            50.0 // Default midpoint for running tasks
                        }
                    }
                    "completed" => 100.0,
                    "failed" | "cancelled" => 0.0,
                    "paused" => {
                        // Estimate based on state_data if available
                        if let Ok(Some(state_data)) = state_row.try_get::<Option<serde_json::Value>, &str>("state_data") {
                            // Try to extract progress from state_data
                            state_data.get("progress")
                                .and_then(|p| p.as_f64())
                                .unwrap_or(50.0)
                        } else {
                            50.0
                        }
                    }
                    _ => 0.0,
                };
                
                let phase = match final_status.as_str() {
                    "pending" => "pending",
                    "running" => "executing",
                    "completed" => "completed",
                    "failed" => "failed",
                    "cancelled" => "cancelled",
                    "paused" => "paused",
                    _ => "unknown",
                }.to_string();
                
                (final_status, progress, phase, started, updated)
            }
            (Some(exec_row), None) => {
                let status: String = exec_row.try_get("status")
                    .unwrap_or_else(|_| "unknown".to_string());
                let started: DateTime<Utc> = exec_row.try_get("execution_started_at")
                    .unwrap_or_else(|_| Utc::now());
                let progress = match status.as_str() {
                    "running" => 50.0,
                    "completed" => 100.0,
                    "failed" | "cancelled" => 0.0,
                    _ => 0.0,
                };
                (status, progress, "executing".to_string(), started, Utc::now())
            }
            (None, Some(state_row)) => {
                let status: String = state_row.try_get("status")
                    .unwrap_or_else(|_| "unknown".to_string());
                let updated: DateTime<Utc> = state_row.try_get("last_updated")
                    .unwrap_or_else(|_| Utc::now());
                let progress = match status.as_str() {
                    "pending" => 0.0,
                    "running" => 50.0,
                    "completed" => 100.0,
                    "failed" | "cancelled" => 0.0,
                    "paused" => 50.0,
                    _ => 0.0,
                };
                (status.clone(), progress, status.clone(), updated, updated)
            }
            (None, None) => {
                // No execution or state found - check if task exists
                let task_row = sqlx::query(
                    r#"
                    SELECT status, created_at, updated_at
                    FROM tasks
                    WHERE id = $1
                    "#
                )
                .bind(task_id)
                .fetch_optional(self.db_client.pool())
                .await
                .map_err(|e| ApiError::InternalError(format!("Failed to query task: {}", e)))?;
                
                match task_row {
                    Some(row) => {
                        let status: String = row.try_get("status")
                            .unwrap_or_else(|_| "unknown".to_string());
                        let created: DateTime<Utc> = row.try_get("created_at")
                            .unwrap_or_else(|_| Utc::now());
                        let updated: DateTime<Utc> = row.try_get("updated_at")
                            .unwrap_or_else(|_| Utc::now());
                        let progress = match status.as_str() {
                            "pending" => 0.0,
                            "in_progress" => 50.0,
                            "completed" => 100.0,
                            "failed" => 0.0,
                            _ => 0.0,
                        };
                        (status, progress, "pending".to_string(), created, updated)
                    }
                    None => {
                        return Err(ApiError::TaskNotFound(task_id.to_string()));
                    }
                }
            }
        };

        Ok(ExecutionProgress {
            task_id,
            status,
            progress_percentage,
            current_phase,
            started_at,
            updated_at,
        })
    }

    pub async fn complete_execution(&self, _task_id: Uuid, _success: bool) -> Result<()> {
        // Execution completion is tracked in task_executions table
        // This method can be used for additional tracking if needed
        Ok(())
    }

    pub async fn pause_execution(&self, _task_id: Uuid) -> Result<()> {
        // Execution pausing is tracked in task_execution_states table
        // This method can be used for additional tracking if needed
        Ok(())
    }

    pub async fn resume_execution(&self, _task_id: Uuid) -> Result<()> {
        // Execution resuming is tracked in task_execution_states table
        // This method can be used for additional tracking if needed
        Ok(())
    }

    pub async fn cancel_execution(&self, _task_id: Uuid) -> Result<()> {
        // Execution cancellation is tracked in task_executions table
        // This method can be used for additional tracking if needed
        Ok(())
    }
}

// Orchestrator wrapper for RestApi compatibility
// Uses OrchestratorService internally
#[derive(Clone)]
pub struct Orchestrator {
    service: Arc<crate::OrchestratorService>,
}

impl Orchestrator {
    pub fn new(service: Arc<crate::OrchestratorService>) -> Self {
        Self { service }
    }

    pub async fn orchestrate_task(&self, description: &str, execution_mode: String) -> Result<ExecutionArtifacts> {
        // Execute task using orchestrator service
        let task_id = self.service.execute_task(
            description.to_string(),
            Some(execution_mode),
            None,
        ).await.map_err(|e| ApiError::ExecutionError(format!("Task execution failed: {}", e)))?;

        // Wait a moment for task to start (in real implementation, would poll or use events)
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Get task status to construct artifacts
        let task_state = self.service.get_task_status(task_id).await
            .map_err(|e| ApiError::InternalError(format!("Failed to get task status: {}", e)))?
            .ok_or_else(|| ApiError::TaskNotFound(task_id.to_string()))?;

        // Construct ExecutionArtifacts from task state
        // Note: ExecutionArtifacts structure is different - we return what we have
        if let Some(artifacts) = task_state.artifacts {
            Ok(artifacts)
        } else {
            // Return minimal artifacts if execution hasn't completed yet
            Ok(ExecutionArtifacts {
                version: "1.0.0".to_string(),
                task_id,
                working_spec_id: task_state.working_spec.as_ref()
                    .map(|ws| ws.id.clone())
                    .unwrap_or_else(|| "unknown".to_string()),
                iteration: 0,
                code_changes: agent_agency_contracts::execution_artifacts::CodeChanges::default(),
                tests: agent_agency_contracts::execution_artifacts::TestArtifacts::default(),
                coverage: agent_agency_contracts::execution_artifacts::CoverageResults::default(),
                linting: agent_agency_contracts::execution_artifacts::LintingResults::default(),
                provenance: agent_agency_contracts::execution_artifacts::Provenance::default(),
                metadata: None,
            })
        }
    }
}
// use super::middleware;

/// REST API server
#[cfg(feature = "orchestration")]
#[derive(Clone)]
pub struct RestApi {
    __config: ApiConfig,
    orchestrator: Arc<Orchestrator>,
    progress_tracker: Arc<ProgressTracker>,
    active_tasks: Arc<RwLock<HashMap<Uuid, TaskState>>>,
    pub db_client: Arc<DatabaseClient>,
}

#[derive(Debug, Clone)]
struct TaskState {
    description: String,
    status: TaskStatus,
    result: Option<serde_json::Value>,
    working_spec: Option<WorkingSpec>,
    artifacts: Option<ExecutionArtifacts>,
    quality_report: Option<QualityReport>,
    started_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
enum TaskStatus {
    Pending,
    Planning,
    Executing,
    QualityCheck,
    Refining,
    Paused,
    Running,
    Completed,
    Failed,
}

#[cfg(feature = "orchestration")]
impl RestApi {
    pub fn new(
        config: ApiConfig,
        orchestrator: Arc<Orchestrator>,
        progress_tracker: Arc<ProgressTracker>,
        db_client: Arc<DatabaseClient>,
    ) -> Self {
        Self {
            __config: config,
            orchestrator,
            progress_tracker,
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            db_client,
        }
    }

    /// Create RestApi with OrchestratorService
    pub fn with_orchestrator_service(
        config: ApiConfig,
        orchestrator_service: Arc<crate::OrchestratorService>,
        db_client: Arc<DatabaseClient>,
    ) -> Self {
        let orchestrator = Arc::new(Orchestrator::new(orchestrator_service));
        let progress_tracker = Arc::new(ProgressTracker::new(db_client.clone()));
        Self::new(config, orchestrator, progress_tracker, db_client)
    }

    /// Get API configuration
    pub fn config(&self) -> &ApiConfig {
        &self.__config
    }

    /// Create the Axum router with all endpoints
    pub fn create_router(&self) -> Router<()> {
        // Note: This router is not currently used in main.rs
        // The routes are created directly there with AppState
        let _state = ApiState {
            api: Arc::new(self.clone()),
            websocket_manager: Arc::new(crate::websocket::WebSocketManager::new()),
            query_performance_monitor: Arc::new(crate::monitoring::query_performance::QueryPerformanceMonitor::with_defaults()),
            coreml_inference_callback: None,
        };

        // This router is not used in main.rs - routes are created directly there
        let mut router = Router::new()
            .route("/health", get(|| async { "OK" }));

        // Add API key authentication middleware when configured
        if self.__config.require_api_key {
            let api_keys = self.__config.api_keys.clone();
            router = router.layer(axum::middleware::from_fn(move |request: axum::http::Request<axum::body::Body>, next: axum::middleware::Next| {
                let headers = request.headers().clone();
                let api_keys = api_keys.clone();
                async move {
                    match crate::api::middleware::api_key_auth(headers, api_keys).await {
                        Ok(_) => Ok(next.run(request).await),
                        Err(status) => Err(status.into_response()),
                    }
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
            description: request.description.clone(),
            status: TaskStatus::Pending,
            result: None,
            working_spec: None,
            artifacts: None,
            quality_report: None,
            started_at: Utc::now(),
            updated_at: Utc::now(),
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
                active_tasks.clone(),
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
        let execution_mode = request.execution_mode.as_deref().unwrap_or("default").to_string();
        let result = orchestrator.orchestrate_task(&request.description, execution_mode).await
            .map_err(|e| ApiError::ExecutionError(format!("Task orchestration failed: {:?}", e)))?;

        // Update task state with results
        // Note: ExecutionArtifacts contains task execution artifacts
        // working_spec and quality_report should be stored/retrieved separately if needed
        {
            let mut active_tasks = active_tasks.write().await;
            if let Some(task) = active_tasks.get_mut(&task_id) {
                task.status = TaskStatus::Completed;
                // working_spec and quality_report stored separately - not in ExecutionArtifacts
                task.artifacts = Some(result.clone());
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

        let response = if let Some(task_state) = task_state {
            TaskStatusResponse {
                task_id,
                status: format!("{:?}", task_state.status).to_lowercase(),
                progress_percentage: progress.progress_percentage as f32,
                current_phase: Some(progress.current_phase.clone()),
                started_at: Some(progress.started_at),
                updated_at: Some(progress.updated_at),
                quality_score: task_state.quality_report.as_ref().map(|r| r.overall_score),
            }
        } else {
            return Err(ApiError::TaskNotFound(task_id.to_string()));
        };

        Ok(response)
    }

    /// Get task result
    pub async fn get_task_result(&self, task_id: Uuid) -> Result<TaskResultResponse> {
        let active_tasks = self.active_tasks.read().await;
        let task_state = active_tasks.get(&task_id)
            .ok_or_else(|| ApiError::TaskNotFound(task_id.to_string()))?;

        Ok(TaskResultResponse {
            task_id,
            status: format!("{:?}", task_state.status).to_lowercase(),
            result: task_state.result.clone(),
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
                return Err(ApiError::TaskNotFound(task_id.to_string()));
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
                return Err(ApiError::TaskNotFound(task_id.to_string()));
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
                return Err(ApiError::TaskNotFound(task_id.to_string()));
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
                created_at: created_at.to_string(),
                updated_at: updated_at.to_string(),
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
            .query_one_with_params(
                query,
                &[&request.name, &request.query_text],
            )
            .await
            .map_err(|e| ApiError::DatabaseError(format!("Failed to save query: {}", e)))?
            .ok_or_else(|| ApiError::DatabaseError("Query insertion failed".to_string()))?;

        let id: Uuid = row.get("id");
        let created_at: DateTime<Utc> = row.get("created_at");
        let updated_at: DateTime<Utc> = row.get("updated_at");

        Ok(SavedQueryResponse {
            id,
            name: request.name,
            query_text: request.query_text,
            created_at: created_at.to_string(),
            updated_at: updated_at.to_string(),
        })
    }

    /// Delete a saved query
    pub async fn delete_saved_query(&self, query_id: Uuid) -> Result<()> {
        // Delete saved query from database
        let query = r#"
            DELETE FROM saved_queries
            WHERE id = $1
        "#;

        let result = self.db_client
            .execute(query, &[&query_id])
            .await
            .map_err(|e| ApiError::DatabaseError(format!("Failed to delete query: {}", e)))?;

        if result.rows_affected() == 0 {
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
                progress_percentage: progress.progress_percentage as f32,
                current_phase: Some(progress.current_phase.clone()),
                started_at: Some(task_state.started_at),
                updated_at: Some(progress.updated_at),
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
            .ok_or_else(|| ApiError::TaskNotFound(task_id.to_string()))?;

        let _progress: ExecutionProgress = self.progress_tracker.get_progress(task_id).await
            .map_err(|e| ApiError::InternalError(format!("Progress retrieval failed: {:?}", e)))
            .unwrap_or(ExecutionProgress { task_id: task_id, status: "completed".to_string(), progress_percentage: 100.0, current_phase: "completed".to_string(), started_at: Utc::now(), updated_at: Utc::now() });
        // TODO: Build iteration summaries from actual iteration data
        // - [ ] Query iteration tracking system for task iterations
        // - [ ] Build iteration summaries with progress and status
        // - [ ] Include iteration artifacts and results
        // - [ ] Support pagination for large iteration lists
        // - [ ] Add unit tests with mock iteration data
        // - [ ] Add integration tests with real iteration tracking
        // Build iteration summaries (placeholder - would come from actual iteration data)
        let iterations = vec![];

        Ok(DashboardTaskSummary {
            task_id,
            description: task_state.description.clone(),
            status: format!("{:?}", task_state.status).to_lowercase(),
            current_iteration: 1, // Placeholder - would come from actual iteration tracking
            total_iterations: 5, // Placeholder - would come from actual iteration tracking
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
            .ok_or_else(|| ApiError::TaskNotFound(task_id.to_string()))?;

        // Placeholder diff data - would come from actual artifacts
        Ok(vec![
            DashboardDiffSummary {
                iteration: iteration.try_into().unwrap(),
                file_path: "src/main.rs".to_string(),
                change_type: "modified".to_string(),
                lines_added: 15,
                lines_removed: 5,
                diff_preview: "@@ -10,5 +10,15 @@\n- old code\n+ new code".to_string(),
            }
        ])
    }
}

/// API server state
#[cfg(feature = "orchestration")]
#[derive(Clone)]
pub struct ApiState {
    pub api: Arc<RestApi>,
    pub websocket_manager: Arc<crate::websocket::WebSocketManager>,
    pub query_performance_monitor: Arc<crate::monitoring::query_performance::QueryPerformanceMonitor>,
    /// Optional callback for CoreML inference (set by wrapper when UnifiedOrchestrator is available)
    pub coreml_inference_callback: Option<Arc<dyn Fn(String) -> std::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<String, String>> + Send>> + Send + Sync>>,
}
