//! Task Management API handlers
//! 
//! This module contains all API handlers related to task management,
//! including submission, status tracking, and lifecycle operations.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json;
use tracing::{info, error};
use uuid::Uuid;

use crate::api::ApiState;

/// Cancel a task
pub async fn cancel_task(Path(task_id): Path<String>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "task_id": task_id,
        "status": "cancelled",
        "message": "Task cancellation requested"
    }))
}

/// Pause a task
pub async fn pause_task(Path(task_id): Path<String>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "task_id": task_id,
        "status": "paused",
        "message": "Task paused"
    }))
}

/// Resume a paused task
pub async fn resume_task(Path(task_id): Path<String>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "task_id": task_id,
        "status": "resumed",
        "message": "Task resumed"
    }))
}

/// Submit a new task
pub async fn submit_task(
    State(state): State<ApiState>,
    Json(task_data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Extract task data from JSON
    let title = task_data.get("title")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let description = task_data.get("description")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let priority = task_data.get("priority")
        .and_then(|v| v.as_str())
        .unwrap_or("medium");
    
    let task_type = task_data.get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("general");

    // Create task struct
    let task = crate::models::Task {
        id: Uuid::new_v4(),
        title: title.to_string(),
        description: description.to_string(),
        risk_tier: "medium".to_string(), // Default risk tier
        scope: serde_json::json!({}),
        acceptance_criteria: serde_json::json!([]),
        context: serde_json::json!({}),
        caws_spec: None,
        status: "pending".to_string(),
        assigned_worker_id: None,
        priority: Some(priority.parse::<i32>().unwrap_or(5)), // Convert priority to integer
        deadline: None,
        metadata: Some(task_data.clone()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        completed_at: None,
    };

    // Create task in database
    match state.api.db_client.create_task(&task).await {
        Ok(task_id) => {
            info!("Created task: {}", task_id);
            Ok(Json(serde_json::json!({
                "id": task.id,
                "title": task.title,
                "description": task.description,
                "priority": task.priority,
                "type": task_type,
                "status": task.status,
                "created_at": task.created_at,
                "updated_at": task.updated_at,
                "metadata": task.metadata,
                "status": "success"
            })))
        }
        Err(e) => {
            error!("Failed to create task: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get task status
pub async fn get_task_status(
    State(state): State<ApiState>,
    Path(task_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let task_uuid = uuid::Uuid::parse_str(&task_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    match state.api.db_client.get_task(&task_uuid).await {
        Ok(Some(task)) => {
            Ok(Json(serde_json::json!({
                "task_id": task_id,
                "status": task.status,
                "progress": 0, // Default progress
                "started_at": task.created_at,
                "updated_at": task.updated_at,
                "estimated_completion": None::<String>,
                "worker_id": task.assigned_worker_id,
                "metadata": task.metadata,
                "status": "success"
            })))
        }
        Ok(None) => {
            error!("Task not found: {}", task_id);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            error!("Failed to get task status for {}: {}", task_id, e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Get task result
pub async fn get_task_result(
    State(state): State<ApiState>,
    Path(task_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let task_uuid = uuid::Uuid::parse_str(&task_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    match state.api.db_client.get_task(&task_uuid).await {
        Ok(Some(task)) => {
            Ok(Json(serde_json::json!({
                "task_id": task_id,
                "success": task.status == "completed",
                "result": task.metadata.clone().unwrap_or(serde_json::json!({})),
                "artifacts": serde_json::json!([]),
                "errors": if task.status == "failed" { serde_json::json!(["Task failed"]) } else { serde_json::json!([]) },
                "completed_at": task.completed_at,
                "execution_time_ms": None::<u64>,
                "quality_score": None::<f64>,
                "metadata": task.metadata,
                "status": "success"
            })))
        }
        Ok(None) => {
            error!("Task not found: {}", task_id);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            error!("Failed to get task result for {}: {}", task_id, e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// List tasks with filtering
pub async fn list_tasks(
    State(state): State<ApiState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.api.db_client.list_tasks().await {
        Ok(tasks) => {
            let task_list: Vec<serde_json::Value> = tasks.into_iter().map(|task| {
                serde_json::json!({
                    "id": task.id,
                    "title": task.title,
                    "description": task.description,
                    "priority": task.priority,
                    "type": "general", // Default type since Task model doesn't have task_type field
                    "status": task.status,
                    "created_at": task.created_at,
                    "updated_at": task.updated_at,
                    "started_at": task.created_at, // Use created_at as started_at
                    "completed_at": task.completed_at,
                    "worker_id": task.assigned_worker_id,
                    "metadata": task.metadata
                })
            }).collect();

            Ok(Json(serde_json::json!({
                "tasks": task_list,
                "total": task_list.len(),
                "status_counts": {
                    "pending": task_list.iter().filter(|t| t.get("status").and_then(|s| s.as_str()) == Some("pending")).count(),
                    "running": task_list.iter().filter(|t| t.get("status").and_then(|s| s.as_str()) == Some("running")).count(),
                    "completed": task_list.iter().filter(|t| t.get("status").and_then(|s| s.as_str()) == Some("completed")).count(),
                    "failed": task_list.iter().filter(|t| t.get("status").and_then(|s| s.as_str()) == Some("failed")).count(),
                    "cancelled": task_list.iter().filter(|t| t.get("status").and_then(|s| s.as_str()) == Some("cancelled")).count(),
                },
                "status": "success"
            })))
        }
        Err(e) => {
            error!("Failed to list tasks: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
