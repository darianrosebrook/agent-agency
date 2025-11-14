//! Task Management Endpoints
//!
//! REST API endpoints for task management and monitoring.

use crate::{ApiRequest, ApiResponse, InterfaceError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Task status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    /// Task is pending
    Pending,

    /// Task is running
    Running,

    /// Task completed successfully
    Completed,

    /// Task failed
    Failed,

    /// Task was cancelled
    Cancelled,
}

/// Task information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    /// Task ID
    pub id: Uuid,

    /// Task name
    pub name: String,

    /// Task description
    pub description: Option<String>,

    /// Task status
    pub status: TaskStatus,

    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last update timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,

    /// Progress percentage (0-100)
    pub progress_percent: Option<u8>,

    /// Error message if failed
    pub error_message: Option<String>,
}

/// Task list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskListResponse {
    /// List of tasks
    pub tasks: Vec<TaskInfo>,

    /// Total number of tasks
    pub total: usize,

    /// Current page
    pub page: usize,

    /// Page size
    pub page_size: usize,
}

/// Task handler for managing tasks via REST API
pub struct TaskHandler {
    /// In-memory task storage (in production, this would be a database)
    tasks: std::sync::RwLock<HashMap<Uuid, TaskInfo>>,
}

impl TaskHandler {
    /// Create a new task handler
    pub fn new() -> Self {
        Self {
            tasks: std::sync::RwLock::new(HashMap::new()),
        }
    }

    /// List tasks
    pub async fn list_tasks(
        &self,
        page: Option<usize>,
        page_size: Option<usize>,
    ) -> Result<TaskListResponse, InterfaceError> {
        let tasks = self
            .tasks
            .read()
            .map_err(|e| InterfaceError::ApiError(format!("Failed to read tasks: {}", e)))?;

        let page = page.unwrap_or(0);
        let page_size = page_size.unwrap_or(50);
        let total = tasks.len();

        let tasks_vec: Vec<_> = tasks.values().cloned().collect();
        let start = page * page_size;
        let end = (start + page_size).min(tasks_vec.len());

        let paginated_tasks = if start < tasks_vec.len() {
            tasks_vec[start..end].to_vec()
        } else {
            Vec::new()
        };

        Ok(TaskListResponse {
            tasks: paginated_tasks,
            total,
            page,
            page_size,
        })
    }

    /// Get task by ID
    pub async fn get_task(&self, task_id: Uuid) -> Result<TaskInfo, InterfaceError> {
        let tasks = self
            .tasks
            .read()
            .map_err(|e| InterfaceError::ApiError(format!("Failed to read tasks: {}", e)))?;

        tasks
            .get(&task_id)
            .cloned()
            .ok_or_else(|| InterfaceError::ApiError(format!("Task {} not found", task_id)))
    }

    /// Create a new task
    pub async fn create_task(
        &self,
        name: String,
        description: Option<String>,
    ) -> Result<TaskInfo, InterfaceError> {
        let task_id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let task = TaskInfo {
            id: task_id,
            name,
            description,
            status: TaskStatus::Pending,
            created_at: now,
            updated_at: now,
            progress_percent: None,
            error_message: None,
        };

        let mut tasks = self
            .tasks
            .write()
            .map_err(|e| InterfaceError::ApiError(format!("Failed to write tasks: {}", e)))?;

        tasks.insert(task_id, task.clone());
        Ok(task)
    }

    /// Handle task API request
    pub async fn handle_task_request(
        &self,
        request: ApiRequest,
    ) -> Result<ApiResponse, InterfaceError> {
        match request.path.as_str() {
            "/api/tasks" => {
                if request.method == "GET" {
                    // TODO: Implement comprehensive query parameter handling for task listing
                    //       Currently ignores query params and returns all tasks; should implement comprehensive query parameter parsing and filtering for pagination, filtering, and sorting of task results.
                    //
                    // COMPLETION CHECKLIST:
                    // [ ] Primary functionality implemented
                    // [ ] API/data structures defined & stable
                    // [ ] Error handling + validation aligned with error taxonomy
                    // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
                    // [ ] Integration tests for external systems/contracts
                    // [ ] Documentation: public API + system behavior
                    // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
                    // [ ] Security posture reviewed (inputs, authz, sandboxing)
                    // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
                    // [ ] Configurability and feature flags defined if relevant
                    // [ ] Failure-mode cards documented (degradation paths)
                    //
                    // ACCEPTANCE CRITERIA:
                    // - Query parameters are parsed correctly
                    // - Pagination parameters are supported
                    // - Filtering parameters work correctly
                    // - Sorting parameters are implemented
                    //
                    // DEPENDENCIES:
                    // - Query parameter parsing utilities (Required)
                    // - Task filtering logic (Required)
                    // - Pagination utilities (Required)
                    //
                    // ESTIMATED EFFORT: 6-8 hours (medium confidence)
                    // PRIORITY: Medium
                    // BLOCKING: No
                    //
                    // GOVERNANCE:
                    // - CAWS Tier: 2 (API endpoint enhancement)
                    // - Change Budget: ~150 LOC
                    // - Reviewer Requirements: API design and query parameter handling expertise
                    let response = self.list_tasks(None, None).await?;
                    Ok(ApiResponse {
                        status_code: 200,
                        headers: std::collections::HashMap::new(),
                        body: serde_json::to_string(&response).map_err(|e| {
                            InterfaceError::ApiError(format!("Failed to serialize response: {}", e))
                        })?,
                    })
                } else if request.method == "POST" {
                    let body_str = request.body.as_ref().ok_or_else(|| {
                        InterfaceError::ApiError("Missing request body".to_string())
                    })?;

                    let body: serde_json::Value = serde_json::from_str(body_str)
                        .map_err(|e| InterfaceError::ApiError(format!("Invalid JSON: {}", e)))?;

                    let name = body
                        .get("name")
                        .and_then(|n| n.as_str())
                        .ok_or_else(|| {
                            InterfaceError::ApiError("Missing 'name' field".to_string())
                        })?
                        .to_string();

                    let description = body
                        .get("description")
                        .and_then(|d| d.as_str())
                        .map(|s| s.to_string());

                    let task = self.create_task(name, description).await?;
                    Ok(ApiResponse {
                        status_code: 201,
                        headers: std::collections::HashMap::new(),
                        body: serde_json::to_string(&task).map_err(|e| {
                            InterfaceError::ApiError(format!("Failed to serialize task: {}", e))
                        })?,
                    })
                } else {
                    Err(InterfaceError::ApiError(format!(
                        "Method {} not allowed for /api/tasks",
                        request.method
                    )))
                }
            }
            path if path.starts_with("/api/tasks/") => {
                let task_id_str = path.strip_prefix("/api/tasks/").unwrap_or("");
                let task_id = Uuid::parse_str(task_id_str)
                    .map_err(|e| InterfaceError::ApiError(format!("Invalid task ID: {}", e)))?;

                if request.method == "GET" {
                    let task = self.get_task(task_id).await?;
                    Ok(ApiResponse {
                        status_code: 200,
                        headers: std::collections::HashMap::new(),
                        body: serde_json::to_string(&task).map_err(|e| {
                            InterfaceError::ApiError(format!("Failed to serialize task: {}", e))
                        })?,
                    })
                } else {
                    Err(InterfaceError::ApiError(format!(
                        "Method {} not allowed for task endpoint",
                        request.method
                    )))
                }
            }
            _ => Err(InterfaceError::ApiError(
                "Unknown task endpoint".to_string(),
            )),
        }
    }
}

impl Default for TaskHandler {
    fn default() -> Self {
        Self::new()
    }
}
