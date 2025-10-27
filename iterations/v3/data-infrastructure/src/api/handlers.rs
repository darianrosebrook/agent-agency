//! API handlers for the data infrastructure service
//!
//! Contains all the HTTP request handlers for tasks, chat, metrics, etc.

use axum::{
    extract::{Path, Query, State, WebSocketUpgrade},
    extract::ws::{Message, WebSocket},
    response::sse::{Event, Sse},
    response::{Json, IntoResponse},
    http::StatusCode,
};
use serde_json::json;
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::{wrappers::IntervalStream, Stream, StreamExt};
use tokio::time;
use uuid::Uuid;
use tracing::{info, warn, error};

use crate::{
    AppState, TaskStoreTrait, DatabaseClient,
};
use crate::api::types::{TaskSubmissionRequest, TaskSubmissionResponse};

// Re-export the health check function from health module
pub use super::health::health_check;

/// List all waivers (stub implementation)
pub async fn list_waivers() -> Json<serde_json::Value> {
    Json(serde_json::json!({"waivers": [], "status": "stub"}))
}

/// Create a new waiver (stub implementation)
pub async fn create_waiver(_waiver_data: Json<serde_json::Value>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"waiver_id": "stub", "status": "created"}))
}

/// Approve a waiver (stub implementation)
pub async fn approve_waiver(Path(_waiver_id): Path<String>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "approved"}))
}

/// Get task provenance (stub implementation)
pub async fn get_task_provenance(Path(_task_id): Path<String>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"provenance": [], "status": "stub"}))
}

/// Proxy handler for backend requests
pub async fn proxy_handler(
    State(state): State<crate::AppState>,
    Path(path): Path<String>,
) -> Result<impl IntoResponse, axum::http::StatusCode> {
    let backend_url = format!("{}/{}", state.backend_host.trim_end_matches('/'), path);

    match state.http_client.get(&backend_url).send().await {
        Ok(response) => {
            let status_code = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            Ok((axum::http::StatusCode::from_u16(status_code).unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR), body))
        }
        Err(_) => {
            // Return a stub response if backend is not available
            Ok((axum::http::StatusCode::OK, r#"{"status": "stub", "message": "Backend not available"}"#.to_string()))
        }
    }
}

/// Get system metrics (stub implementation)
pub async fn get_metrics() -> Json<serde_json::Value> {
    Json(json!({"message": "Metrics not implemented yet"}))
}

/// Get dashboard data (stub implementation)
pub async fn get_dashboard_data() -> Json<serde_json::Value> {
    Json(json!({"message": "Dashboard data not implemented yet"}))
}

/// Get diff summary (stub implementation)
pub async fn get_diff_summary() -> Json<serde_json::Value> {
    Json(json!({"message": "Diff summary not implemented yet"}))
}

/// Acknowledge SLO alert (stub implementation)
pub async fn acknowledge_slo_alert(Path(_alert_id): Path<String>) -> Json<serde_json::Value> {
    Json(json!({"message": "SLO alert acknowledgment not implemented yet"}))
}

/// List SLOs (stub implementation)
pub async fn list_slos() -> Json<serde_json::Value> {
    Json(json!({"slos": [], "message": "SLO system not implemented yet"}))
}

/// Get SLO status (stub implementation)
pub async fn get_slo_status(Path(_slo_id): Path<String>) -> Json<serde_json::Value> {
    Json(json!({"message": "SLO status not implemented yet"}))
}

/// Get SLO measurements (stub implementation)
pub async fn get_slo_measurements(Path(_slo_id): Path<String>) -> Json<serde_json::Value> {
    Json(json!({"measurements": [], "message": "SLO measurements not implemented yet"}))
}

/// List SLO alerts (stub implementation)
pub async fn list_slo_alerts() -> Json<serde_json::Value> {
    Json(json!({"alerts": [], "message": "SLO alerts not implemented yet"}))
}

/// List provenance records (stub implementation)
pub async fn list_provenance_records() -> Json<serde_json::Value> {
    Json(json!({"records": [], "message": "Provenance records not implemented yet"}))
}

/// Link provenance to commit (stub implementation)
pub async fn link_provenance_to_commit(_link_data: Json<serde_json::Value>) -> Json<serde_json::Value> {
    Json(json!({"message": "Provenance linking not implemented yet"}))
}

/// Verify provenance trailer (stub implementation)
pub async fn verify_provenance_trailer(Query(_params): Query<std::collections::HashMap<String, String>>) -> Json<serde_json::Value> {
    Json(json!({"message": "Provenance verification not implemented yet"}))
}

/// Get provenance by commit (stub implementation)
pub async fn get_provenance_by_commit(Path(_commit_hash): Path<String>) -> Json<serde_json::Value> {
    Json(json!({"message": "Provenance by commit not implemented yet"}))
}

/// Cancel task (stub implementation)
pub async fn cancel_task(Path(_task_id): Path<String>) -> Json<serde_json::Value> {
    Json(json!({"message": "Task cancellation not implemented yet"}))
}

/// Pause task (stub implementation)
pub async fn pause_task(Path(_task_id): Path<String>) -> Json<serde_json::Value> {
    Json(json!({"message": "Task pausing not implemented yet"}))
}

/// Resume task (stub implementation)
pub async fn resume_task(Path(_task_id): Path<String>) -> Json<serde_json::Value> {
    Json(json!({"message": "Task resuming not implemented yet"}))
}

/// List saved queries (stub implementation)
pub async fn list_saved_queries() -> Json<serde_json::Value> {
    Json(json!({"queries": [], "message": "Saved queries not implemented yet"}))
}

/// Save query (stub implementation)
pub async fn save_query(_query_data: Json<serde_json::Value>) -> Json<serde_json::Value> {
    Json(json!({"message": "Query saving not implemented yet"}))
}

/// Delete saved query (stub implementation)
pub async fn delete_saved_query(Path(_query_id): Path<String>) -> Json<serde_json::Value> {
    Json(json!({"message": "Query deletion not implemented yet"}))
}

/// Submit task (stub implementation)
pub async fn submit_task(_task_data: Json<serde_json::Value>) -> Json<serde_json::Value> {
    Json(json!({"task_id": "stub-task-id", "message": "Task submission not implemented yet"}))
}

/// Get task status (stub implementation)
pub async fn get_task_status(Path(_task_id): Path<String>) -> Json<serde_json::Value> {
    Json(json!({"status": "unknown", "message": "Task status not implemented yet"}))
}

/// Get task result (stub implementation)
pub async fn get_task_result(Path(_task_id): Path<String>) -> Json<serde_json::Value> {
    Json(json!({"result": null, "message": "Task result not implemented yet"}))
}

/// List all tasks
pub async fn list_tasks(
    State(state): State<crate::AppState>,
) -> Json<serde_json::Value> {
    match state.task_store.get_tasks().await {
        Ok(tasks) => {
            let task_summaries: Vec<serde_json::Value> = tasks
                .into_iter()
                .map(|task| {
                    let spec: serde_json::Value = serde_json::from_str(&task.spec).unwrap_or(json!({}));
                    let empty_map = serde_json::Map::new();
                    let spec = spec.as_object().unwrap_or(&empty_map);
                    let title = spec.get("description")
                        .and_then(|d| d.as_str())
                        .unwrap_or("Untitled Task");

                    json!({
                        "id": task.id,
                        "title": title,
                        "status": task.state,
                        "priority": spec.get("priority").and_then(|p| p.as_str()).unwrap_or("medium"),
                        "createdAt": task.created_at,
                        "updatedAt": task.updated_at
                    })
                })
                .collect();

            Json(json!({
                "tasks": task_summaries,
                "total": task_summaries.len(),
                "page": 1,
                "limit": 50,
                "status": "success"
            }))
        }
        Err(e) => {
            println!("⚠️  Failed to list tasks: {}", e);
            Json(json!({
                "error": "Failed to retrieve tasks",
                "status": "error"
            }))
        }
    }
}