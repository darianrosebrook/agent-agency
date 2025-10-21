//! Task API Endpoints
//! 
//! Provides REST API for task management, including task listing, detail retrieval,
//! and lifecycle operations. All endpoints integrate with the persistent database layer.

use agent_agency_database::PgPool;
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
pub async fn get_task_events(
    Extension(pool): Extension<PgPool>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<Vec<TaskEvent>>, TaskApiError> {
    // Use the new task audit logs table (P0 requirement: persist audit trail + surface it on tasks)
    let events = sqlx::query_as::<_, TaskEvent>(
        r#"
        SELECT id, task_id, ts, category, actor, action, payload, idx
        FROM task_audit_logs
        WHERE task_id = $1
        ORDER BY idx DESC
        LIMIT 100
        "#,
    )
    .bind(&task_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| TaskApiError::DatabaseError(e.to_string()))?;

    Ok(Json(events))
}

/// Cancel a task
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `task_id` - UUID of task to cancel
///
/// # Returns
/// Updated task state
pub async fn cancel_task(
    Extension(pool): Extension<PgPool>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<TaskResponse>, TaskApiError> {
    // Update task state
    sqlx::query("UPDATE tasks SET state = 'canceled', updated_at = NOW() WHERE id = $1")
        .bind(&task_id)
        .execute(&pool)
        .await
        .map_err(|e| TaskApiError::DatabaseError(e.to_string()))?;

    // Log audit event
    let _ = sqlx::query(
        "INSERT INTO audit_logs (action, actor, resource_id, resource_type, change_summary) VALUES ('task_canceled', 'api', $1, 'task', jsonb_build_object('reason', 'User canceled task via API'))"
    )
    .bind(&task_id)
    .execute(&pool)
    .await;

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
