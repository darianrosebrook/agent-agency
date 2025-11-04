//! REST API Server Implementation
//!
//! Contains the main RestApi struct and core business logic methods
//! for task management, execution, and API operations.

use schemars::JsonSchema;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "orchestration")]
use agent_orchestration::audited_orchestrator::Orchestrator;
#[cfg(feature = "orchestration")]
use agent_orchestration::progress_tracker::{ProgressTracker, ExecutionProgress};

use super::{ApiError, Result, WorkingSpec, ExecutionArtifacts, QualityReport};
// use super::handlers::{get_metrics, get_dashboard_data, get_diff_summary, list_tasks, acknowledge_slo_alert, list_slos, get_slo_status, get_slo_measurements, list_slo_alerts, create_waiver, approve_waiver, get_task_provenance, list_provenance_records, link_provenance_to_commit, verify_provenance_trailer, get_provenance_by_commit, cancel_task, pause_task, resume_task, list_saved_queries, save_query, delete_saved_query, list_waivers, health_check, submit_task, get_task_status, get_task_result};

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

#[derive(Debug, Clone, JsonSchema)]
pub struct ProgressTracker {
    #[schemars(with = "String")]
    pub task_id: Uuid,
}

impl ProgressTracker {
    pub async fn start_execution(&self, _task_id: Uuid, _mode: String) -> Result<()> {
        Ok(())
    }

    pub async fn get_progress(&self, _task_id: Uuid) -> Result<ExecutionProgress> {
        Ok(ExecutionProgress {
            task_id: self.task_id,
            status: "in_progress".to_string(),
            progress_percentage: 50.0,
            current_phase: "processing".to_string(),
            started_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    pub async fn complete_execution(&self, _task_id: Uuid, _success: bool) -> Result<()> {
        Ok(())
    }

    pub async fn pause_execution(&self, _task_id: Uuid) -> Result<()> {
        Ok(())
    }

    pub async fn resume_execution(&self, _task_id: Uuid) -> Result<()> {
        Ok(())
    }

    pub async fn cancel_execution(&self, _task_id: Uuid) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, JsonSchema)]
pub struct Orchestrator {
    #[schemars(with = "String")]
    pub id: Uuid,
}

impl Orchestrator {
    pub async fn orchestrate_task(&self, description: &str, _execution_mode: String) -> Result<ExecutionArtifacts> {
        // PLACEHOLDER: Real task orchestration not implemented
        // Per session rules: throw error instead of returning stub artifacts
        // Dependency: Requires integration with agent-orchestration crate's Orchestrator
        return Err(ApiError::InternalError(format!(
            "PLACEHOLDER: Orchestrator::orchestrate_task not implemented. Requires: \
            Integration with agent-orchestration crate Orchestrator. \
            Task description: {}, Execution mode: {}",
            description.chars().take(100).collect::<String>(),
            _execution_mode
        )));
    }
}
// use super::middleware;

/// REST API server
#[cfg(feature = "orchestration")]
#[derive(Clone)]
pub struct RestApi {
    __config: ApiConfig,
    orchestrator: Arc<Orchestrator>,
    progress_tracker: Arc<dyn ProgressTracker>,
    active_tasks: Arc<RwLock<HashMap<Uuid, TaskState>>>,
    pub db_client: Arc<DatabaseClient>,
}

#[derive(Debug, Clone, JsonSchema)]
struct TaskState {
    description: String,
    status: TaskStatus,
    result: Option<serde_json::Value>,
    working_spec: Option<WorkingSpec>,
    artifacts: Option<ExecutionArtifacts>,
    quality_report: Option<QualityReport>,
    #[schemars(with = "String")]
    started_at: DateTime<Utc>,
    #[schemars(with = "String")]
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
        progress_tracker: Arc<dyn ProgressTracker>,
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

    /// Create the Axum router with all endpoints
    pub fn create_router(&self) -> Router<()> {
        // Note: This router is not currently used in main.rs
        // The routes are created directly there with AppState
        let _state = ApiState {
            api: Arc::new(self.clone()),
        };

        // This router is not used in main.rs - routes are created directly there
        let mut router = Router::new()
            .route("/health", get(|| async { "OK" }));

        // Add API key authentication middleware when configured
        if self.__config.require_api_key {
            let api_keys = self.__config.api_keys.clone();
            router = router.layer(axum::middleware::from_fn(move |mut request: axum::http::Request<axum::body::Body>, next: axum::middleware::Next| async move {
                let headers = request.headers().clone();
                match crate::api::middleware::api_key_auth(headers, api_keys.clone()).await {
                    Ok(_) => Ok(next.run(request).await),
                    Err(status) => Err(status.into_response()),
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
                progress: Some(progress.progress_percentage),
                progress_percentage: Some(progress.progress_percentage),
                current_phase: Some(progress.current_phase.clone()),
                started_at: Some(progress.started_at),
                created_at: progress.started_at,
                updated_at: progress.updated_at,
                quality_score: task_state.quality_report.as_ref().map(|r| r.score),
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
                description: None, // TODO: Add description field to database
                created_at,
                updated_at,
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
            description: request.description,
            created_at,
            updated_at,
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
                progress: Some(progress.progress_percentage),
                progress_percentage: Some(progress.progress_percentage),
                current_phase: Some(progress.current_phase.clone()),
                started_at: Some(task_state.started_at),
                created_at: task_state.started_at,
                updated_at: progress.updated_at,
                quality_score: task_state.quality_report.as_ref().map(|r| r.score),
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
        // Build iteration summaries (placeholder - would come from actual iteration data)
        let iterations = vec![];

        Ok(DashboardTaskSummary {
            total_tasks: 1,
            active_tasks: 1,
            completed_tasks: 0,
            failed_tasks: 0,
            success_rate: 1.0,
            average_completion_time: None,
            task_id,
            description: task_state.description.clone(),
            status: format!("{:?}", task_state.status).to_lowercase(),
            current_iteration: 1, // Placeholder - would come from actual iteration tracking
            total_iterations: 5, // Placeholder - would come from actual iteration tracking
            score: task_state.quality_report.as_ref().map(|r| r.score),
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
                id: Uuid::new_v4(),
                task_id,
                diff_type: "code_change".to_string(),
                summary: "Code modification".to_string(),
                timestamp: Utc::now(),
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
}
