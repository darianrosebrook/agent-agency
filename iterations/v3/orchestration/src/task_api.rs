//! Task API Endpoints
//!
//! Provides REST API for task management, including task listing, detail retrieval,
//! and lifecycle operations. Endpoints use CQRS commands and queries for proper
//! separation of concerns and business logic encapsulation.

use agent_agency_database::PgPool;
#[cfg(feature = "api-server")]
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;
use std::fmt;
use std::sync::Arc;

// CQRS imports
use crate::cqrs::{
    CqrsBus,
    commands::*,
    queries::*,
};

// Re-export CQRS API functions for use in routing
pub use execute_task_cqrs as execute_task;
pub use cancel_task_with_cqrs as cancel_task;
pub use update_task_progress_cqrs as update_task_progress;
pub use register_worker_cqrs as register_worker;
pub use update_worker_health_cqrs as update_worker_health;
pub use get_task_status_cqrs as get_task_status;
pub use get_system_health_cqrs as get_system_health;
pub use get_active_tasks_cqrs as get_active_tasks;

/// Task API error types
#[derive(Debug)]
pub enum TaskApiError {
    DatabaseError(String),
    NotFound,
    InvalidInput(String),
}

impl fmt::Display for TaskApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::DatabaseError(e) => write!(f, "Database error: {}", e),
            Self::NotFound => write!(f, "Task not found"),
            Self::InvalidInput(e) => write!(f, "Invalid input: {}", e),
        }
    }
}

#[cfg(feature = "api-server")]
impl IntoResponse for TaskApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::DatabaseError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e),
            Self::NotFound => (StatusCode::NOT_FOUND, "Task not found".to_string()),
            Self::InvalidInput(e) => (StatusCode::BAD_REQUEST, e),
        };

        (status, Json(serde_json::json!({"error": message}))).into_response()
    }
}

/// Task response model for API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResponse {
    pub id: Uuid,
    pub state: String,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
}

/// Detailed task information including spec and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDetail {
    pub id: Uuid,
    pub spec: serde_json::Value,
    pub state: String,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
    pub metadata: serde_json::Value,
}

/// Task event entry - matches task_audit_logs table structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEvent {
    pub id: Uuid,
    pub task_id: Uuid,
    pub ts: String,
    pub category: String,
    pub actor: String,
    pub action: String,
    pub payload: serde_json::Value,
    pub idx: i64,
}

/// Query parameters for task listing
#[derive(Debug, Deserialize)]
pub struct TaskListQuery {
    pub state: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Get list of tasks with optional filtering
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `query` - Optional query parameters (state filter, pagination)
///
/// # Returns
#[cfg(feature = "api-server")]
/// List of tasks or error
pub async fn get_tasks(
    Extension(pool): Extension<PgPool>,
    Query(query): Query<TaskListQuery>,
) -> Result<Json<Vec<TaskResponse>>, TaskApiError> {
    let limit = query.limit.unwrap_or(50).max(1).min(1000);
    let offset = query.offset.unwrap_or(0).max(0);

    let sql = if let Some(state) = &query.state {
        format!(
            "SELECT id, state, created_at, updated_at, created_by FROM tasks WHERE state = $1 ORDER BY created_at DESC LIMIT {} OFFSET {}",
            limit, offset
        )
    } else {
        format!(
            "SELECT id, state, created_at, updated_at, created_by FROM tasks ORDER BY created_at DESC LIMIT {} OFFSET {}",
            limit, offset
        )
    };

    let rows = if let Some(state) = &query.state {
        sqlx::query(&sql)
            .bind(state)
            .fetch_all(&pool)
            .await
            .map_err(|e| TaskApiError::DatabaseError(e.to_string()))?
    } else {
        sqlx::query(&sql)
            .fetch_all(&pool)
            .await
            .map_err(|e| TaskApiError::DatabaseError(e.to_string()))?
    };

    let tasks = rows
        .into_iter()
        .map(|row| TaskResponse {
            id: row.get::<Uuid, _>("id"),
            state: row.get::<String, _>("state"),
            created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
            updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
            created_by: row.get::<Option<String>, _>("created_by"),
        })
        .collect();

    Ok(Json(tasks))
}

/// Get task detail by ID
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `task_id` - UUID of task to retrieve
///
/// # Returns
/// Task detail with spec, metadata, and audit trail
#[cfg(feature = "api-server")]
pub async fn get_task_detail(
    Extension(pool): Extension<PgPool>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<TaskDetail>, TaskApiError> {
    let row = sqlx::query(
        "SELECT id, spec, state, created_at, updated_at, created_by, metadata FROM tasks WHERE id = $1"
    )
    .bind(&task_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| TaskApiError::DatabaseError(e.to_string()))?
    .ok_or(TaskApiError::NotFound)?;

    let detail = TaskDetail {
        id: row.get::<Uuid, _>("id"),
        spec: row.get::<serde_json::Value, _>("spec"),
        state: row.get::<String, _>("state"),
        created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
        updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
        created_by: row.get::<Option<String>, _>("created_by"),
        metadata: row.get::<serde_json::Value, _>("metadata"),
    };

    Ok(Json(detail))
}

/// Get audit trail for a task
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `task_id` - UUID of task to retrieve events for
///
/// # Returns
/// List of audit events for the task
#[cfg(feature = "api-server")]
pub async fn get_task_events(
    Extension(pool): Extension<PgPool>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<Vec<TaskEvent>>, TaskApiError> {
    // Use the new task audit logs table (P0 requirement: persist audit trail + surface it on tasks)
    let events = sqlx::query(
        r#"
        SELECT id, task_id, created_at, category, actor, action, payload, idx
        FROM task_audit_logs
        WHERE task_id = $1
        ORDER BY idx DESC
        LIMIT 100
        "#,
    )
    .bind(&task_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| TaskApiError::DatabaseError(e.to_string()))?
    .into_iter()
    .map(|row: sqlx::postgres::PgRow| TaskEvent {
        id: row.get("id"),
        task_id: row.get("task_id"),
        ts: row.get("created_at"),
        category: row.get("category"),
        actor: row.get("actor"),
        action: row.get("action"),
        payload: row.get("payload"),
        idx: row.get("idx"),
    })
    .collect::<Vec<_>>();

    Ok(Json(events))
}

/// Pause a task (P0 requirement: wire pause/resume real, not just update local state)
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `task_id` - UUID of task to pause
///
/// # Returns
/// Updated task state
#[cfg(feature = "api-server")]
pub async fn pause_task(
    Extension(pool): Extension<PgPool>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<TaskResponse>, TaskApiError> {
    // P0: Wire pause/resume real - call worker control endpoint
    let worker_endpoint = std::env::var("AGENT_AGENCY_WORKER_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:8081".to_string());

    // Call worker control endpoint to pause
    let control_url = format!("{}/tasks/{}/control", worker_endpoint.trim_end_matches('/'), task_id);
    let client = reqwest::Client::new();

    let control_request = client
        .post(&control_url)
        .json(&serde_json::json!({
            "task_id": task_id,
            "action": "pause",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
        .send();

    // Update task state to paused
    sqlx::query("UPDATE tasks SET state = 'paused', updated_at = NOW() WHERE id = $1 AND state = 'executing'")
        .bind(&task_id)
        .execute(&pool)
        .await
        .map_err(|e| TaskApiError::DatabaseError(e.to_string()))?;

    // P0: Write to task_audit_logs
    sqlx::query(
        r#"
        INSERT INTO task_audit_logs (task_id, category, actor, action, payload)
        VALUES ($1, 'orchestration', 'api', 'paused', $2)
        "#,
    )
    .bind(&task_id)
    .bind(serde_json::json!({
        "reason": "User paused task via API",
        "worker_endpoint": control_url,
        "stage": "pause_initiated"
    }))
    .execute(&pool)
    .await
    .map_err(|e| TaskApiError::DatabaseError(e.to_string()))?;

    // Wait for worker response with timeout
    match tokio::time::timeout(
        std::time::Duration::from_secs(5),
        control_request
    ).await {
        Ok(Ok(response)) => {
            if response.status().is_success() {
                // Audit successful pause
                sqlx::query(
                    r#"
                    INSERT INTO task_audit_logs (task_id, category, actor, action, payload)
                    VALUES ($1, 'orchestration', 'worker', 'pause_confirmed', $2)
                    "#,
                )
                .bind(&task_id)
                .bind(serde_json::json!({
                    "outcome": "success",
                    "worker_response": response.status().as_u16(),
                    "stage": "pause_complete"
                }))
                .execute(&pool)
                .await
                .map_err(|e| TaskApiError::DatabaseError(e.to_string()))?;
            } else {
                // Worker returned error, but task is still marked as paused
                sqlx::query(
                    r#"
                    INSERT INTO task_audit_logs (task_id, category, actor, action, payload)
                    VALUES ($1, 'orchestration', 'worker', 'pause_error', $2)
                    "#,
                )
                .bind(&task_id)
                .bind(serde_json::json!({
                    "outcome": "worker_error",
                    "worker_response": response.status().as_u16(),
                    "stage": "pause_with_error"
                }))
                .execute(&pool)
                .await
                .map_err(|e| TaskApiError::DatabaseError(e.to_string()))?;
            }
        }
        Ok(Err(e)) => {
            // Network error from worker request
            sqlx::query(
                r#"
                INSERT INTO task_audit_logs (task_id, category, actor, action, payload)
                VALUES ($1, 'orchestration', 'system', 'pause_network_error', $2)
                "#,
            )
            .bind(&task_id)
            .bind(serde_json::json!({
                "outcome": "network_error",
                "error": e.to_string(),
                "stage": "pause_with_network_error"
            }))
            .execute(&pool)
            .await
            .map_err(|e| TaskApiError::DatabaseError(e.to_string()))?;
        }
        Err(_) => {
            // Timeout waiting for worker response
            sqlx::query(
                r#"
                INSERT INTO task_audit_logs (task_id, category, actor, action, payload)
                VALUES ($1, 'orchestration', 'system', 'pause_timeout', $2)
                "#,
            )
            .bind(&task_id)
            .bind(serde_json::json!({
                "stage": "pause_timeout",
                "note": "Task marked as paused, worker may still process pause"
            }))
            .execute(&pool)
            .await
            .map_err(|e| TaskApiError::DatabaseError(e.to_string()))?;
        }
    }

    // Fetch updated task
    let row = sqlx::query(
        "SELECT id, state, created_at, updated_at, created_by FROM tasks WHERE id = $1"
    )
    .bind(&task_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| TaskApiError::DatabaseError(e.to_string()))?;

    match row {
        Some(row) => {
            let task = TaskResponse {
                id: row.get("id"),
                state: row.get("state"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                created_by: row.get("created_by"),
            };
            Ok(Json(task))
        }
        None => Err(TaskApiError::NotFound),
    }
}

/// Resume a task (P0 requirement: wire pause/resume real, not just update local state)
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `task_id` - UUID of task to resume
///
/// # Returns
/// Updated task state
#[cfg(feature = "api-server")]
pub async fn resume_task(
    Extension(pool): Extension<PgPool>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<TaskResponse>, TaskApiError> {
    // P0: Wire pause/resume real - call worker control endpoint
    let worker_endpoint = std::env::var("AGENT_AGENCY_WORKER_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:8081".to_string());

    // Call worker control endpoint to resume
    let control_url = format!("{}/tasks/{}/control", worker_endpoint.trim_end_matches('/'), task_id);
    let client = reqwest::Client::new();

    let control_request = client
        .post(&control_url)
        .json(&serde_json::json!({
            "task_id": task_id,
            "action": "resume",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
        .send();

    // Update task state to executing (resumed)
    sqlx::query("UPDATE tasks SET state = 'executing', updated_at = NOW() WHERE id = $1 AND state = 'paused'")
        .bind(&task_id)
        .execute(&pool)
        .await
        .map_err(|e| TaskApiError::DatabaseError(e.to_string()))?;

    // P0: Write to task_audit_logs
    sqlx::query(
        r#"
        INSERT INTO task_audit_logs (task_id, category, actor, action, payload)
        VALUES ($1, 'orchestration', 'api', 'resumed', $2)
        "#,
    )
    .bind(&task_id)
    .bind(serde_json::json!({
        "reason": "User resumed task via API",
        "worker_endpoint": control_url,
        "stage": "resume_initiated"
    }))
    .execute(&pool)
    .await
    .map_err(|e| TaskApiError::DatabaseError(e.to_string()))?;

    // Wait for worker response with timeout
    match tokio::time::timeout(
        std::time::Duration::from_secs(5),
        control_request
    ).await {
        Ok(Ok(response)) => {
            if response.status().is_success() {
                // Audit successful resume
                sqlx::query(
                    r#"
                    INSERT INTO task_audit_logs (task_id, category, actor, action, payload)
                    VALUES ($1, 'orchestration', 'worker', 'resume_confirmed', $2)
                    "#,
                )
                .bind(&task_id)
                .bind(serde_json::json!({
                    "outcome": "success",
                    "worker_response": response.status().as_u16(),
                    "stage": "resume_complete"
                }))
                .execute(&pool)
                .await
                .map_err(|e| TaskApiError::DatabaseError(e.to_string()))?;
            } else {
                // Worker returned error, but task is still marked as executing
                sqlx::query(
                    r#"
                    INSERT INTO task_audit_logs (task_id, category, actor, action, payload)
                    VALUES ($1, 'orchestration', 'worker', 'resume_error', $2)
                    "#,
                )
                .bind(&task_id)
                .bind(serde_json::json!({
                    "outcome": "worker_error",
                    "worker_response": response.status().as_u16(),
                    "stage": "resume_with_error"
                }))
                .execute(&pool)
                .await
                .map_err(|e| TaskApiError::DatabaseError(e.to_string()))?;
            }
        }
        Ok(Err(e)) => {
            // Network error from worker request
            sqlx::query(
                r#"
                INSERT INTO task_audit_logs (task_id, category, actor, action, payload)
                VALUES ($1, 'orchestration', 'system', 'resume_network_error', $2)
                "#,
            )
            .bind(&task_id)
            .bind(serde_json::json!({
                "outcome": "network_error",
                "error": e.to_string(),
                "stage": "resume_with_network_error"
            }))
            .execute(&pool)
            .await
            .map_err(|e| TaskApiError::DatabaseError(e.to_string()))?;
        }
        Err(_) => {
            // Timeout waiting for worker response
            sqlx::query(
                r#"
                INSERT INTO task_audit_logs (task_id, category, actor, action, payload)
                VALUES ($1, 'orchestration', 'system', 'resume_timeout', $2)
                "#,
            )
            .bind(&task_id)
            .bind(serde_json::json!({
                "stage": "resume_timeout",
                "note": "Task marked as executing, worker may still process resume"
            }))
            .execute(&pool)
            .await
            .map_err(|e| TaskApiError::DatabaseError(e.to_string()))?;
        }
    }

    // Fetch updated task
    let row = sqlx::query(
        "SELECT id, state, created_at, updated_at, created_by FROM tasks WHERE id = $1"
    )
    .bind(&task_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| TaskApiError::DatabaseError(e.to_string()))?;

    match row {
        Some(row) => {
            let task = TaskResponse {
                id: row.get("id"),
                state: row.get("state"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                created_by: row.get("created_by"),
            };
            Ok(Json(task))
        }
        None => Err(TaskApiError::NotFound),
    }
}

/// Cancel a task (P0 requirement: implement cancel end-to-end)
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `task_id` - UUID of task to cancel
///
/// # Returns
#[cfg(feature = "api-server")]
/// Updated task state
pub async fn cancel_task_legacy(
    Extension(pool): Extension<PgPool>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<TaskResponse>, TaskApiError> {
    // P0: Implement cancel end-to-end - call worker HTTP endpoint
    // For now, use the configured worker endpoint from environment
    let worker_endpoint = std::env::var("AGENT_AGENCY_WORKER_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:8081".to_string());

    // Call worker cancel endpoint (idempotent)
    let cancel_url = format!("{}/tasks/{}/cancel", worker_endpoint.trim_end_matches('/'), task_id);
    let client = reqwest::Client::new();

    // Fire cancel request but don't wait for completion (async)
    let cancel_request = client
        .post(&cancel_url)
        .json(&serde_json::json!({
            "task_id": task_id,
            "reason": "User canceled task via API",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
        .send();

    // Update task state to canceling (intermediate state)
    sqlx::query("UPDATE tasks SET state = 'canceling', updated_at = NOW() WHERE id = $1")
        .bind(&task_id)
        .execute(&pool)
        .await
        .map_err(|e| TaskApiError::DatabaseError(e.to_string()))?;

    // P0: Write to task_audit_logs instead of general audit_logs
    sqlx::query(
        r#"
        INSERT INTO task_audit_logs (task_id, category, actor, action, payload)
        VALUES ($1, 'orchestration', 'api', 'cancel_requested', $2)
        "#,
    )
    .bind(&task_id)
    .bind(serde_json::json!({
        "reason": "User canceled task via API",
        "worker_endpoint": cancel_url,
        "stage": "cancellation_initiated"
    }))
    .execute(&pool)
    .await
    .map_err(|e| TaskApiError::DatabaseError(e.to_string()))?;

    // Wait for worker cancel response with timeout
    match tokio::time::timeout(
        std::time::Duration::from_secs(5),
        cancel_request
    ).await {
        Ok(Ok(response)) => {
            if response.status().is_success() {
                // Update to fully canceled
                sqlx::query("UPDATE tasks SET state = 'canceled', updated_at = NOW() WHERE id = $1")
                    .bind(&task_id)
                    .execute(&pool)
                    .await
                    .map_err(|e| TaskApiError::DatabaseError(e.to_string()))?;

                // P0: Audit successful cancellation
                sqlx::query(
                    r#"
                    INSERT INTO task_audit_logs (task_id, category, actor, action, payload)
                    VALUES ($1, 'orchestration', 'worker', 'canceled', $2)
                    "#,
                )
                .bind(&task_id)
                .bind(serde_json::json!({
                    "outcome": "success",
                    "worker_response": response.status().as_u16(),
                    "stage": "cancellation_complete"
                }))
                .execute(&pool)
                .await
                .map_err(|e| TaskApiError::DatabaseError(e.to_string()))?;
            } else {
                // Worker returned error, still mark as canceled but log the issue
                sqlx::query("UPDATE tasks SET state = 'canceled', updated_at = NOW() WHERE id = $1")
                    .bind(&task_id)
                    .execute(&pool)
                    .await
                    .map_err(|e| TaskApiError::DatabaseError(e.to_string()))?;

                sqlx::query(
                    r#"
                    INSERT INTO task_audit_logs (task_id, category, actor, action, payload)
                    VALUES ($1, 'orchestration', 'worker', 'canceled', $2)
                    "#,
                )
                .bind(&task_id)
                .bind(serde_json::json!({
                    "outcome": "worker_error",
                    "worker_response": response.status().as_u16(),
                    "stage": "cancellation_with_error"
                }))
                .execute(&pool)
                .await
                .map_err(|e| TaskApiError::DatabaseError(e.to_string()))?;
            }
        }
        Ok(Err(e)) => {
            // Network error, but task is still marked as canceling
            // This is acceptable - the worker might still process the cancel
            sqlx::query(
                r#"
                INSERT INTO task_audit_logs (task_id, category, actor, action, payload)
                VALUES ($1, 'orchestration', 'system', 'cancel_timeout', $2)
                "#,
            )
            .bind(&task_id)
            .bind(serde_json::json!({
                "error": e.to_string(),
                "stage": "cancellation_timeout",
                "note": "Task marked as canceling, worker may still process cancel"
            }))
            .execute(&pool)
            .await
            .map_err(|e| TaskApiError::DatabaseError(e.to_string()))?;
        }
        Err(_) => {
            // Timeout, but task is still marked as canceling
            sqlx::query(
                r#"
                INSERT INTO task_audit_logs (task_id, category, actor, action, payload)
                VALUES ($1, 'orchestration', 'system', 'cancel_timeout', $2)
                "#,
            )
            .bind(&task_id)
            .bind(serde_json::json!({
                "stage": "cancellation_timeout",
                "note": "Task marked as canceling, worker may still process cancel"
            }))
            .execute(&pool)
            .await
            .map_err(|e| TaskApiError::DatabaseError(e.to_string()))?;
        }
    }

    // Fetch updated task
    let row = sqlx::query(
        "SELECT id, state, created_at, updated_at, created_by FROM tasks WHERE id = $1"
    )
    .bind(&task_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| TaskApiError::DatabaseError(e.to_string()))?
    .ok_or(TaskApiError::NotFound)?;

    Ok(Json(TaskResponse {
        id: row.get::<Uuid, _>("id"),
        state: row.get::<String, _>("state"),
        created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
        updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at").to_rfc3339(),
        created_by: row.get::<Option<String>, _>("created_by"),
    }))
}

// ============================================================================
// CQRS-BASED API ENDPOINTS
// ============================================================================

#[cfg(feature = "api-server")]
/// CQRS-based task execution endpoint
#[utoipa::path(
    post,
    path = "/api/tasks/{task_id}/execute",
    params(("task_id" = Uuid, Path, description = "Task ID to execute")),
    request_body = ExecuteTaskRequest,
    responses(
        (status = 200, description = "Task execution started", body = ExecuteTaskResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn execute_task_cqrs(
    Extension(cqrs_bus): Extension<Arc<CqrsBus>>,
    Path(task_id): Path<Uuid>,
    Json(request): Json<ExecuteTaskRequest>,
) -> Result<Json<ExecuteTaskResponse>, TaskApiError> {
    // Create CQRS command
    let command = ExecuteTaskCommand {
        task_descriptor: request.task_descriptor,
        worker_id: request.worker_id,
        requested_at: chrono::Utc::now(),
    };

    // Execute command via CQRS bus
    let execution_id = cqrs_bus.execute_command(command).await
        .map_err(|e| TaskApiError::DatabaseError(format!("Command execution failed: {:?}", e)))?;

    Ok(Json(ExecuteTaskResponse {
        execution_id,
        task_id,
        status: "queued".to_string(),
        message: "Task queued for execution".to_string(),
    }))
}

#[cfg(feature = "api-server")]
/// CQRS-based task cancellation endpoint
#[utoipa::path(
    post,
    path = "/api/tasks/{task_id}/cancel",
    params(("task_id" = Uuid, Path, description = "Task ID to cancel")),
    request_body = CancelTaskRequest,
    responses(
        (status = 200, description = "Task cancellation initiated"),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn cancel_task_with_cqrs(
    Extension(cqrs_bus): Extension<Arc<CqrsBus>>,
    Path(task_id): Path<Uuid>,
    Json(request): Json<CancelTaskRequest>,
) -> Result<Json<CancelTaskResponse>, TaskApiError> {
    // Create CQRS command
    let command = CancelTaskCommand {
        task_id,
        worker_id: request.worker_id,
        reason: request.reason,
    };

    // Execute command via CQRS bus
    cqrs_bus.execute_command(command).await
        .map_err(|e| TaskApiError::DatabaseError(format!("Command execution failed: {:?}", e)))?;

    Ok(Json(CancelTaskResponse {
        task_id,
        status: "cancelled".to_string(),
        message: "Task cancellation initiated".to_string(),
    }))
}

#[cfg(feature = "api-server")]
/// CQRS-based task progress update endpoint
#[utoipa::path(
    post,
    path = "/api/tasks/{task_id}/progress",
    params(("task_id" = Uuid, Path, description = "Task ID to update")),
    request_body = UpdateProgressRequest,
    responses(
        (status = 200, description = "Progress updated successfully"),
        (status = 400, description = "Invalid progress value"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_task_progress_cqrs(
    Extension(cqrs_bus): Extension<Arc<CqrsBus>>,
    Path(task_id): Path<Uuid>,
    Json(request): Json<UpdateProgressRequest>,
) -> Result<Json<UpdateProgressResponse>, TaskApiError> {
    // Create CQRS command
    let command = UpdateTaskProgressCommand {
        task_id,
        progress_percentage: request.progress_percentage,
        status_message: request.status_message,
    };

    // Execute command via CQRS bus
    cqrs_bus.execute_command(command).await
        .map_err(|e| TaskApiError::DatabaseError(format!("Command execution failed: {:?}", e)))?;

    Ok(Json(UpdateProgressResponse {
        task_id,
        progress_percentage: request.progress_percentage,
        status: "updated".to_string(),
        message: "Task progress updated".to_string(),
    }))
}

#[cfg(feature = "api-server")]
/// CQRS-based worker registration endpoint
#[utoipa::path(
    post,
    path = "/api/workers/register",
    request_body = RegisterWorkerRequest,
    responses(
        (status = 200, description = "Worker registered successfully", body = RegisterWorkerResponse),
        (status = 400, description = "Invalid capabilities or request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn register_worker_cqrs(
    Extension(cqrs_bus): Extension<Arc<CqrsBus>>,
    Json(request): Json<RegisterWorkerRequest>,
) -> Result<Json<RegisterWorkerResponse>, TaskApiError> {
    // Create CQRS command
    let command = RegisterWorkerCommand {
        worker_id: request.worker_id,
        capabilities: request.capabilities,
        metadata: request.metadata,
    };

    // Execute command via CQRS bus
    let registration_id = cqrs_bus.execute_command(command).await
        .map_err(|e| TaskApiError::DatabaseError(format!("Command execution failed: {:?}", e)))?;

    Ok(Json(RegisterWorkerResponse {
        registration_id,
        worker_id: request.worker_id,
        status: "registered".to_string(),
        message: "Worker registered successfully".to_string(),
    }))
}

#[cfg(feature = "api-server")]
/// CQRS-based worker health update endpoint
#[utoipa::path(
    post,
    path = "/api/workers/{worker_id}/health",
    params(("worker_id" = Uuid, Path, description = "Worker ID to update")),
    request_body = UpdateHealthRequest,
    responses(
        (status = 200, description = "Worker health updated"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_worker_health_cqrs(
    Extension(cqrs_bus): Extension<Arc<CqrsBus>>,
    Path(worker_id): Path<Uuid>,
    Json(request): Json<UpdateHealthRequest>,
) -> Result<Json<UpdateHealthResponse>, TaskApiError> {
    // Create CQRS command
    let command = UpdateWorkerHealthCommand {
        worker_id,
        is_healthy: request.is_healthy,
        last_seen: request.last_seen,
    };

    // Execute command via CQRS bus
    cqrs_bus.execute_command(command).await
        .map_err(|e| TaskApiError::DatabaseError(format!("Command execution failed: {:?}", e)))?;

    Ok(Json(UpdateHealthResponse {
        worker_id,
        is_healthy: request.is_healthy,
        status: "updated".to_string(),
        message: "Worker health updated".to_string(),
    }))
}

#[cfg(feature = "api-server")]
/// CQRS-based task status query endpoint
#[utoipa::path(
    get,
    path = "/api/tasks/{task_id}/status",
    params(("task_id" = Uuid, Path, description = "Task ID to query")),
    responses(
        (status = 200, description = "Task status retrieved", body = TaskStatusResponse),
        (status = 404, description = "Task not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_task_status_cqrs(
    Extension(cqrs_bus): Extension<Arc<CqrsBus>>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<TaskStatusResponse>, TaskApiError> {
    // Create CQRS query
    let query = GetTaskStatusQuery { task_id };

    // Execute query via CQRS bus
    let task_status = cqrs_bus.execute_query(query).await
        .map_err(|e| TaskApiError::DatabaseError(format!("Query execution failed: {:?}", e)))?;

    match task_status {
        Some(status) => Ok(Json(TaskStatusResponse {
            task_id: status.task_id,
            status: format!("{:?}", status.status).to_lowercase(),
            progress_percentage: status.progress_percentage,
            started_at: status.started_at,
            completed_at: status.completed_at,
            worker_id: status.worker_id,
            error_message: status.error_message,
        })),
        None => Err(TaskApiError::NotFound),
    }
}

#[cfg(feature = "api-server")]
/// CQRS-based system health query endpoint
#[utoipa::path(
    get,
    path = "/api/health",
    responses(
        (status = 200, description = "System health retrieved", body = SystemHealthResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_system_health_cqrs(
    Extension(cqrs_bus): Extension<Arc<CqrsBus>>,
) -> Result<Json<SystemHealthResponse>, TaskApiError> {
    // Create CQRS query
    let query = GetSystemHealthQuery;

    // Execute query via CQRS bus
    let health = cqrs_bus.execute_query(query).await
        .map_err(|e| TaskApiError::DatabaseError(format!("Query execution failed: {:?}", e)))?;

    Ok(Json(SystemHealthResponse {
        total_workers: health.total_workers,
        active_workers: health.active_workers,
        healthy_workers: health.healthy_workers,
        total_tasks: health.total_tasks,
        active_tasks: health.active_tasks,
        completed_tasks: health.completed_tasks,
        failed_tasks: health.failed_tasks,
        average_task_duration_ms: health.average_task_duration_ms,
        uptime_seconds: health.uptime_seconds,
        timestamp: chrono::Utc::now(),
    }))
}

#[cfg(feature = "api-server")]
/// CQRS-based active tasks query endpoint
#[utoipa::path(
    get,
    path = "/api/tasks/active",
    params(("limit" = Option<usize>, Query, description = "Maximum number of tasks to return")),
    params(("offset" = Option<usize>, Query, description = "Number of tasks to skip")),
    responses(
        (status = 200, description = "Active tasks retrieved", body = Vec<TaskStatusResponse>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_active_tasks_cqrs(
    Extension(cqrs_bus): Extension<Arc<CqrsBus>>,
    Query(params): Query<ActiveTasksQueryParams>,
) -> Result<Json<Vec<TaskStatusResponse>>, TaskApiError> {
    // Create CQRS query
    let query = ListActiveTasksQuery {
        limit: params.limit,
        offset: params.offset,
    };

    // Execute query via CQRS bus
    let tasks = cqrs_bus.execute_query(query).await
        .map_err(|e| TaskApiError::DatabaseError(format!("Query execution failed: {:?}", e)))?;

    let responses = tasks.into_iter()
        .map(|task| TaskStatusResponse {
            task_id: task.task_id,
            status: format!("{:?}", task.status).to_lowercase(),
            progress_percentage: task.progress_percentage,
            started_at: task.started_at,
            completed_at: task.completed_at,
            worker_id: task.worker_id,
            error_message: task.error_message,
        })
        .collect();

    Ok(Json(responses))
}

// ============================================================================
// CQRS API REQUEST/RESPONSE MODELS
// ============================================================================

/// Execute task request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteTaskRequest {
    pub task_descriptor: crate::caws_runtime::TaskDescriptor,
    pub worker_id: Uuid,
}

/// Execute task response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteTaskResponse {
    pub execution_id: Uuid,
    pub task_id: Uuid,
    pub status: String,
    pub message: String,
}

/// Cancel task request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelTaskRequest {
    pub worker_id: Uuid,
    pub reason: String,
}

/// Cancel task response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelTaskResponse {
    pub task_id: Uuid,
    pub status: String,
    pub message: String,
}

/// Update progress request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProgressRequest {
    pub progress_percentage: u8,
    pub status_message: Option<String>,
}

/// Update progress response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProgressResponse {
    pub task_id: Uuid,
    pub progress_percentage: u8,
    pub status: String,
    pub message: String,
}

/// Register worker request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterWorkerRequest {
    pub worker_id: Uuid,
    pub capabilities: Vec<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

/// Register worker response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterWorkerResponse {
    pub registration_id: Uuid,
    pub worker_id: Uuid,
    pub status: String,
    pub message: String,
}

/// Update health request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateHealthRequest {
    pub is_healthy: bool,
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

/// Update health response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateHealthResponse {
    pub worker_id: Uuid,
    pub is_healthy: bool,
    pub status: String,
    pub message: String,
}

/// Task status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatusResponse {
    pub task_id: Uuid,
    pub status: String,
    pub progress_percentage: u8,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub worker_id: Option<Uuid>,
    pub error_message: Option<String>,
}

/// System health response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthResponse {
    pub total_workers: u64,
    pub active_workers: u64,
    pub healthy_workers: u64,
    pub total_tasks: u64,
    pub active_tasks: u64,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub average_task_duration_ms: f64,
    pub uptime_seconds: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Active tasks query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveTasksQueryParams {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_response_serialization() {
        let task = TaskResponse {
            id: Uuid::new_v4(),
            state: "pending".to_string(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
            created_by: Some("test@example.com".to_string()),
        };

        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("pending"));
    }

    #[test]
    fn test_task_list_query_defaults() {
        let query = TaskListQuery {
            state: None,
            limit: None,
            offset: None,
        };
        assert_eq!(query.limit, None);
        assert_eq!(query.offset, None);
    }
}
