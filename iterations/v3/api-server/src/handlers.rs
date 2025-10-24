//! HTTP request handlers for the Agent Agency API Server

use axum::{
    extract::{Path, State},
    response::Json,
    http::StatusCode,
};
use serde_json::json;
use uuid::Uuid;
use chrono;
use serde::{Deserialize, Serialize};

// Task submission request/response types
#[derive(Debug, Deserialize)]
pub struct TaskSubmissionRequest {
    pub description: String,
    pub context: String,
    pub priority: String,
}

#[derive(Debug, Serialize)]
pub struct TaskSubmissionResponse {
    pub task_id: String,
    pub status: String,
    pub message: String,
    pub estimated_completion: Option<String>,
}

// API Error types
#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    RateLimitExceeded,
    InternalServerError,
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::RateLimitExceeded => (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded".to_string()),
            ApiError::InternalServerError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
        };

        let body = json!({
            "error": message,
            "status": "error"
        });

        (status, Json(body)).into_response()
    }
}

// Additional imports for handlers
use async_trait::async_trait;
use std::sync::Arc;
use std::net::IpAddr;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersistedTask {
    pub id: String,
    pub spec: String,
    pub state: String,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: Option<String>,
    pub metadata: String,
}

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub task_store: Arc<dyn TaskStoreTrait + Send + Sync>,
    pub health_monitor: Arc<agent_agency_system_health_monitor::SystemHealthMonitor>,
    pub alert_manager: Arc<crate::alerts::AlertManager>,
    pub rate_limiter: Arc<crate::rate_limiter::RateLimiter>,
}

#[async_trait::async_trait]
pub trait TaskStoreTrait: Send + Sync {
    async fn create_task(&self, task: PersistedTask) -> anyhow::Result<()>;
    async fn get_tasks(&self) -> anyhow::Result<Vec<PersistedTask>>;
    async fn get_task(&self, task_id: String) -> anyhow::Result<Option<PersistedTask>>;
    async fn get_task_events(&self, task_id: String) -> anyhow::Result<Vec<serde_json::Value>>;
}

/// Health check endpoint
pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "agent-agency-v3-api",
        "version": "1.0.0",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "uptime_seconds": 0, // TODO: Implement uptime tracking
        "workers": "simulated" // Placeholder - worker pool integration not implemented
    }))
}

/// List all tasks
pub async fn list_tasks(
    State(state): State<AppState>,
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

/// Get a specific task by ID
#[axum::debug_handler]
pub async fn get_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Json<serde_json::Value> {
    // Get task and events in parallel for better performance
    let task_id_clone = task_id.clone();
    let (task_result, events_result) = tokio::join!(
        state.task_store.get_task(task_id.clone()),
        state.task_store.get_task_events(task_id_clone)
    );

    match task_result {
        Ok(Some(task)) => {
            let spec: serde_json::Value = serde_json::from_str(&task.spec).unwrap_or(json!({}));
            let empty_map = serde_json::Map::new();
            let spec = spec.as_object().unwrap_or(&empty_map);
            let title = spec.get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("Untitled Task");
            let description = spec.get("context")
                .and_then(|c| c.as_str())
                .unwrap_or("");

            let events = match events_result {
                Ok(events) => events,
                Err(e) => {
                    println!("⚠️  Failed to get events for task {}: {}", task_id.clone(), e);
                    Vec::new()
                }
            };

            Json(json!({
                "task": {
                    "id": task.id,
                    "title": title,
                    "description": description,
                    "status": task.state,
                    "createdAt": task.created_at,
                    "updatedAt": task.updated_at,
                    "createdBy": task.created_by.unwrap_or("unknown".to_string()),
                    "spec": spec
                },
                "events": events,
                "status": "success"
            }))
        }
        Ok(None) => {
            Json(json!({
                "error": "Task not found",
                "status": "error"
            }))
        }
        Err(e) => {
            println!("⚠️  Failed to get task {}: {}", task_id, e);
            Json(json!({
                "error": "Failed to retrieve task",
                "status": "error"
            }))
        }
    }
}

/// Submit a new task
pub async fn submit_task(
    State(state): State<AppState>,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
    Json(request): Json<TaskSubmissionRequest>,
) -> Result<Json<TaskSubmissionResponse>, ApiError> {
    // Rate limiting check
    let client_ip = addr.ip().to_string();
    // Rate limiting check
    let client_ip_addr: IpAddr = addr.ip();
    if let Err(_) = state.rate_limiter.check_rate_limit(client_ip_addr).await {
        return Err(ApiError::RateLimitExceeded);
    }

    // Validate request
    let description = request.description.trim();
    if description.is_empty() {
        return Err(ApiError::BadRequest("Task description cannot be empty".to_string()));
    }

    if description.len() > 10000 {
        return Err(ApiError::BadRequest("Task description too long".to_string()));
    }

    let context = request.context;
    let task_id = Uuid::new_v4();
    let now = chrono::Utc::now().to_rfc3339();

    // Create task specification
    let task_spec = json!({
        "description": description,
        "context": context,
        "priority": request.priority,
        "created_at": now,
        "client_ip": client_ip
    });

    // Create persisted task
    let task = PersistedTask {
        id: task_id.to_string(),
        spec: task_spec.to_string(),
        state: "pending".to_string(),
        created_at: now.clone(),
        updated_at: now,
        created_by: Some("api-client".to_string()),
        metadata: json!({"source": "api", "ip": client_ip}).to_string(),
    };

    // Store task
    if let Err(e) = state.task_store.create_task(task).await {
        println!("⚠️  Failed to create task: {}", e);
        return Err(ApiError::InternalServerError);
    }

    // TODO: Submit to worker pool
    let description_clone = description.to_string();
    tokio::spawn(async move {
        let client = reqwest::Client::new();
        let worker_endpoint = "http://localhost:8081/execute";

        let request_body = serde_json::json!({
            "task_id": task_id.to_string(),
            "prompt": description_clone,
            "context": context,
            "requirements": request.priority,
            "caws_spec": null
        });

        match client.post(worker_endpoint).json(&request_body).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    println!("✅ Task {} submitted to worker", task_id);
                } else {
                    println!("⚠️  Worker returned status: {}", response.status());
                }
            }
            Err(e) => {
                println!("⚠️  Failed to submit task {} to worker: {}", task_id, e);
            }
        }
    });

    Ok(Json(TaskSubmissionResponse {
        task_id: task_id.to_string(),
        status: "accepted".to_string(),
        message: "Task submitted successfully".to_string(),
        estimated_completion: None,
    }))
}

/// Get API metrics
pub async fn get_api_metrics() -> Json<serde_json::Value> {
    Json(json!({
        "metrics": {
            "active_tasks": 1,
            "completed_tasks": 1,
            "failed_tasks": 0,
            "avg_response_time_ms": 150.0,
            "uptime_seconds": 3600,
            "memory_usage_mb": 45.2,
            "cpu_usage_percent": 12.5
        },
        "timestamp": chrono::Utc::now().timestamp(),
        "status": "ok"
    }))
}

/// Create a new chat session
pub async fn create_chat_session() -> Json<serde_json::Value> {
    let session_id = Uuid::new_v4();
    let created_at = chrono::Utc::now().to_rfc3339();

    Json(json!({
        "sessionId": session_id,
        "status": "created",
        "createdAt": created_at,
        "websocketUrl": format!("ws://localhost:8080/api/v1/chat/ws/{}", session_id),
        "protocols": ["agent-agency-v1"],
        "heartbeatInterval": 30000
    }))
}

/// Get WebSocket configuration for a session
pub async fn get_websocket_config(Path(session_id): Path<String>) -> Json<serde_json::Value> {
    // Return WebSocket configuration for the dashboard
    Json(json!({
        "backend_url": format!("ws://localhost:8080/api/v1/chat/ws/{}", session_id),
        "session_id": session_id,
        "heartbeat_interval": 30000,
        "reconnect_attempts": 5,
        "reconnect_delay": 1000,
        "protocols": ["agent-agency-v1"],
        "timestamp": chrono::Utc::now().timestamp()
    }))
}

/// List all waivers
pub async fn list_waivers() -> Json<serde_json::Value> {
    Json(serde_json::json!({"waivers": [], "status": "stub"}))
}

/// Create a new waiver
pub async fn create_waiver(_waiver_data: Json<serde_json::Value>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"waiver_id": "stub", "status": "created"}))
}

/// Approve a waiver
pub async fn approve_waiver(Path(_waiver_id): Path<String>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "approved"}))
}

/// Get task provenance information
pub async fn get_task_provenance(Path(_task_id): Path<String>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"provenance": [], "status": "stub"}))
}
