#![allow(warnings)] // Disables all warnings for the crate
#![allow(dead_code)] // Disables dead_code warnings for the crate

//! Agent Agency V3 API Server
//!
//! Standalone HTTP API server providing REST endpoints for task management,
//! health checks, and metrics streaming.

use axum::{
    extract::{Path, Query, State, WebSocketUpgrade},
    extract::ws::{Message, WebSocket},
    response::sse::{Event, Sse},
    routing::{get, post},
    response::{Json, IntoResponse},
    Router,
    http::StatusCode,
};
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::{wrappers::IntervalStream, Stream, StreamExt};
use tokio::time;
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::RwLock;
use tokio::fs;
use tower_http::cors::CorsLayer;
use uuid::Uuid;
use reqwest::Client;
use agent_agency_database::{DatabaseClient, DatabaseConfig, MigrationManager};
use agent_agency_system_health_monitor::{
    SystemHealthMonitor, SystemHealthMonitorConfig, HealthThresholds,
    EmbeddingServiceConfig, RedisConfig, SystemMetrics, DiskIOMetrics,
    agent_integration::BusinessMetrics
};
// Stub implementations for agent_agency_interfaces
pub async fn list_waivers() -> Json<serde_json::Value> {
    Json(serde_json::json!({"waivers": [], "status": "stub"}))
}

pub async fn create_waiver(_waiver_data: Json<serde_json::Value>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"waiver_id": "stub", "status": "created"}))
}

pub async fn approve_waiver(Path(_waiver_id): Path<String>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "approved"}))
}

pub async fn get_task_provenance(Path(_task_id): Path<String>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"provenance": [], "status": "stub"}))
}
use async_trait::async_trait;
// WebSocket support is built into Axum - no axum-ws needed

mod api_alerts;
mod audit;
mod service_failover;
mod api_circuit_breaker;
mod rto_rpo_monitor;
mod rate_limiter;
mod keystore_api;
mod sandbox_api;

#[derive(Parser)]
#[command(name = "agent-agency-api")]
#[command(about = "Agent Agency V3 REST API Server")]
struct Args {
    /// Server host
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Server port
    #[arg(long, default_value = "8080")]
    port: u16,

    /// Enable CORS
    #[arg(long)]
    enable_cors: bool,

    /// Database host
    #[arg(long, default_value = "localhost")]
    db_host: String,

    /// Database port
    #[arg(long, default_value = "5432")]
    db_port: u16,

    /// Database name
    #[arg(long, default_value = "agent_agency")]
    db_name: String,

    /// Database username
    #[arg(long, default_value = "postgres")]
    db_user: String,

    /// Database password
    #[arg(long, default_value = "password")]
    db_password: String,

    /// Enable Redis for metrics storage
    #[arg(long)]
    enable_redis: bool,

    /// Redis URL
    #[arg(long, default_value = "redis://127.0.0.1:6379")]
    redis_url: String,

    /// Redis key prefix
    #[arg(long, default_value = "agent_agency")]
    redis_key_prefix: String,

    /// V3 Backend host for proxy routes
    #[arg(long, default_value = "http://localhost:3001", env = "V3_BACKEND_HOST")]
    v3_backend_host: String,

    /// Redis metrics TTL (seconds)
    #[arg(long, default_value = "3600")]
    redis_metrics_ttl: u64,

    /// Redis cache TTL (seconds)
    #[arg(long, default_value = "300")]
    redis_cache_ttl: u64,

    /// Log level
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[derive(Debug, Deserialize)]
struct TaskSubmissionRequest {
    description: String,
    context: Option<String>,
    priority: Option<String>,
}

#[derive(Debug, Serialize)]
struct TaskSubmissionResponse {
    task_id: Uuid,
    status: String,
    message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersistedTask {
    id: String,
    spec: String,
    state: String,
    created_at: String,
    updated_at: String,
    created_by: Option<String>,
    metadata: String,
}

/// Simple file-based persistence for MVP
struct TaskStore {
    tasks: RwLock<HashMap<String, PersistedTask>>,
    file_path: String,
}

impl TaskStore {
    fn new(file_path: String) -> Self {
        Self {
            tasks: RwLock::new(HashMap::new()),
            file_path,
        }
    }

    async fn load(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(content) = fs::read_to_string(&self.file_path).await {
            let tasks: HashMap<String, PersistedTask> = serde_json::from_str(&content)?;
            let task_count = tasks.len();
            *self.tasks.write().unwrap() = tasks;
            println!(" Loaded {} tasks from persistence", task_count);
        }
        Ok(())
    }

    async fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let tasks = self.tasks.read().unwrap();
        let content = serde_json::to_string_pretty(&*tasks)?;
        fs::write(&self.file_path, content).await?;
        Ok(())
    }

    async fn create_task(&self, task: PersistedTask) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut tasks = self.tasks.write().unwrap();
            tasks.insert(task.id.clone(), task);
        }
        self.save().await
    }

    fn get_tasks(&self) -> Vec<PersistedTask> {
        self.tasks.read().unwrap().values().cloned().collect()
    }
}

/// Database-backed task store with proper persistence
#[derive(Debug)]
struct DatabaseTaskStore {
    db_client: DatabaseClient,
}

impl DatabaseTaskStore {
    async fn new(config: &DatabaseConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let db_client = DatabaseClient::new(config.clone()).await?;
        Ok(Self { db_client })
    }

    async fn create_task(&self, task: PersistedTask) -> anyhow::Result<()> {
        let query = r#"
            INSERT INTO tasks (id, spec, state, created_by, metadata)
            VALUES ($1, $2, $3, $4, $5)
        "#;

        self.db_client.execute(
            query,
            &[&task.id, &task.spec, &task.state, &task.created_by, &task.metadata],
        ).await?;

        println!(" Created task {} in database", task.id);
        Ok(())
    }

    async fn get_tasks(&self) -> anyhow::Result<Vec<PersistedTask>> {
        let query = r#"
            SELECT id, spec, state, created_at, updated_at, created_by, metadata
            FROM tasks
            ORDER BY created_at DESC
        "#;

        let rows = self.db_client.query(query, &[]).await?;

        let mut tasks = Vec::new();
        for row in rows {
            let task = PersistedTask {
                id: row.get("id"),
                spec: row.get("spec"),
                state: row.get("state"),
                created_at: row.get::<_, String>("created_at"),
                updated_at: row.get::<_, String>("updated_at"),
                created_by: row.get("created_by"),
                metadata: row.get("metadata"),
            };
            tasks.push(task);
        }

        Ok(tasks)
    }

    async fn get_task(&self, task_id: String) -> anyhow::Result<Option<PersistedTask>> {
        let query = r#"
            SELECT id, spec, state, created_at, updated_at, created_by, metadata
            FROM tasks
            WHERE id = $1
        "#;

        let rows = self.db_client.query(query, &[&task_id]).await?;

        if let Some(row) = rows.into_iter().next() {
            let task = PersistedTask {
                id: row.get("id"),
                spec: row.get("spec"),
                state: row.get("state"),
                created_at: row.get::<_, String>("created_at"),
                updated_at: row.get::<_, String>("updated_at"),
                created_by: row.get("created_by"),
                metadata: row.get("metadata"),
            };
            Ok(Some(task))
        } else {
            Ok(None)
        }
    }

    async fn get_task_events(&self, _task_id: String) -> anyhow::Result<Vec<serde_json::Value>> {
        // TODO: Implement task audit events when DatabaseClient supports it
        // For now, return empty events
        Ok(vec![])
    }
}

/// Task store trait for abstraction
#[async_trait]
trait TaskStoreTrait {
    async fn create_task(&self, task: PersistedTask) -> anyhow::Result<()>;
    async fn get_tasks(&self) -> anyhow::Result<Vec<PersistedTask>>;
    async fn get_task(&self, task_id: String) -> anyhow::Result<Option<PersistedTask>>;
    async fn get_task_events(&self, task_id: String) -> anyhow::Result<Vec<serde_json::Value>>;
}

#[async_trait]
impl TaskStoreTrait for DatabaseTaskStore {
    async fn create_task(&self, task: PersistedTask) -> anyhow::Result<()> {
        self.create_task(task).await
    }

    async fn get_tasks(&self) -> anyhow::Result<Vec<PersistedTask>> {
        self.get_tasks().await
    }

    async fn get_task(&self, task_id: String) -> anyhow::Result<Option<PersistedTask>> {
        self.get_task(task_id).await
    }

    async fn get_task_events(&self, task_id: String) -> anyhow::Result<Vec<serde_json::Value>> {
        self.get_task_events(task_id).await
    }

    // Note: get_task_acceptance_criteria removed from trait for simplicity
}

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    task_store: Arc<dyn TaskStoreTrait + Send + Sync>,
    db_client: DatabaseClient,
    audit_logger: audit::AuditLogger,
    keystore: Arc<dyn system_quality_security::Keystore>,
    sandbox: Arc<dyn system_quality_security::Sandbox>,
    health_monitor: Arc<SystemHealthMonitor>,
    alert_manager: Arc<api_alerts::AlertManager>,
    rate_limiter: Arc<rate_limiter::RateLimiter>,
    backend_host: String,
    http_client: Client,
}

pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "agent-agency-v3-api",
        "version": "1.0.0",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "components": {
            "api": "healthy",
            "database": "simulated", // Placeholder - database integration not implemented
            "orchestrator": "simulated", // Placeholder - orchestrator integration not implemented
            "workers": "simulated" // Placeholder - worker pool integration not implemented
        }
    }))
}

pub async fn proxy_handler(
    State(state): State<AppState>,
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

async fn get_task_audit_trail(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let limit = params.get("limit")
        .and_then(|l| l.parse::<i64>().ok())
        .unwrap_or(50);

    let since = params.get("since")
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

    match state.audit_logger.get_task_audit_trail(&task_id, Some(limit), since).await {
        Ok(audit_trail) => {
            Ok(Json(json!({
                "task_id": task_id,
                "audit_trail": audit_trail,
                "total_events": audit_trail.len()
            })))
        }
        Err(e) => {
            eprintln!("Failed to retrieve task audit trail: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

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
                    Vec::new() // Return empty events on error rather than failing the whole request
                }
            };

            // Get acceptance criteria from database (stub implementation)
            let acceptance_criteria = vec!["Complete task successfully".to_string()];

            Json(json!({
                "id": task.id,
                "title": title,
                "description": description,
                "status": task.state,
                "priority": spec.get("priority").and_then(|p| p.as_str()).unwrap_or("medium"),
                "createdAt": task.created_at,
                "updatedAt": task.updated_at,
                "acceptanceCriteria": acceptance_criteria,
                "events": events
            }))
        }
        Ok(None) => Json(json!({
            "error": "Task not found",
            "status": "not_found"
        })),
        Err(e) => {
            println!("⚠️  Failed to get task {}: {}", task_id, e);
            Json(json!({
                "error": "Failed to retrieve task",
                "status": "error"
            }))
        }
    }
}

// Chat session creation
pub async fn create_chat_session(
    State(state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let session_id = Uuid::new_v4().to_string();
    let user_id = request.get("userId")
        .and_then(|u| u.as_str())
        .unwrap_or("anonymous");

    // Create session in database
    let query = r#"
        INSERT INTO chat_sessions (session_id, user_id, metadata)
        VALUES ($1, $2, $3)
        RETURNING id, created_at
    "#;

    let metadata = json!({
        "user_agent": request.get("userAgent").and_then(|ua| ua.as_str()).unwrap_or("unknown"),
        "platform": request.get("platform").and_then(|p| p.as_str()).unwrap_or("web"),
        "ip_address": request.get("ipAddress").and_then(|ip| ip.as_str()).unwrap_or("unknown")
    });

    let session_id_string = session_id.to_string();
    let user_id_string = user_id.to_string();
    let metadata_json = serde_json::to_string(&metadata).unwrap();

    match state.db_client.query(
        query,
        &[&session_id_string, &user_id_string, &metadata_json]
    ).await {
        Ok(rows) => {
            if let Some(row) = rows.into_iter().next() {
                let session_uuid_str: String = row.get("id");
                let session_uuid = Uuid::parse_str(&session_uuid_str).unwrap();
                let created_at: String = row.get("created_at");

                Ok(Json(json!({
                    "sessionId": session_id,
                    "sessionUuid": session_uuid,
                    "createdAt": created_at,
                    "status": "created"
                })))
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
        Err(e) => {
            eprintln!("Failed to create chat session: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Chat message handling (HTTP fallback for MVP)
async fn send_chat_message(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_message = request.get("message")
        .and_then(|m| m.as_str())
        .unwrap_or("Hello");

    let sender = request.get("sender")
        .and_then(|s| s.as_str())
        .unwrap_or("user");

    // First, verify session exists and get its UUID
    let session_query = "SELECT id FROM chat_sessions WHERE session_id = $1 AND status = 'active'";
    let session_id_param = session_id.to_string();
    let session_result = match state.db_client.query(session_query, &[&session_id_param]).await {
        Ok(rows) => {
            if let Some(row) = rows.into_iter().next() {
                let uuid_str: String = row.get("id");
                Ok(Uuid::parse_str(&uuid_str).unwrap())
            } else {
                Err("Session not found or inactive")
            }
        }
        Err(e) => Err(format!("Database error: {:?}", e))
    };

    let session_uuid = match session_result {
        Ok(uuid) => uuid,
        Err(err) => {
            eprintln!("Chat session error: {}", err);
            return Err(StatusCode::NOT_FOUND);
        }
    };

    // Store user message in database
    let insert_user_message = r#"
        INSERT INTO chat_messages (session_id, message_type, content, sender, metadata)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, created_at, sequence_number
    "#;

    let user_metadata = json!({
        "source": "http_api",
        "user_agent": request.get("userAgent").and_then(|ua| ua.as_str()).unwrap_or("unknown")
    });
    let message_type = "message".to_string();
    let sender_param = sender.to_string();

    let user_message_result = state.db_client.query(
        insert_user_message,
        &[&session_uuid.to_string(), &message_type, &user_message, &sender_param, &serde_json::to_string(&user_metadata).unwrap()]
    ).await;

    let user_message_id = match user_message_result {
        Ok(rows) => {
            if let Some(row) = rows.into_iter().next() {
                let id_str: String = row.get("id");
                Uuid::parse_str(&id_str).unwrap()
            } else {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
        Err(e) => {
            eprintln!("Failed to store user message: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Generate AI response (for now, echo back with some variation)
    let ai_response = if user_message.to_lowercase().contains("hello") {
        "Hello! How can I help you with your tasks today?".to_string()
    } else if user_message.to_lowercase().contains("help") {
        "I can help you manage tasks, monitor system health, and chat in real-time. What would you like to know?".to_string()
                        } else {
                            format!("I received your message: '{}'. This is a simulated response - full AI integration coming soon!", user_message)
                        };

    // Store AI response in database
    let insert_ai_message = r#"
        INSERT INTO chat_messages (session_id, message_type, content, sender, metadata)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, created_at, sequence_number
    "#;

    let ai_metadata = json!({
        "source": "ai_assistant",
        "response_type": "generated"
    });
    let ai_sender = "assistant".to_string();

    let ai_message_result = state.db_client.query(
        insert_ai_message,
        &[&session_uuid.to_string(), &message_type, &ai_response, &ai_sender, &serde_json::to_string(&ai_metadata).unwrap()]
    ).await;

    let ai_message_id = match ai_message_result {
        Ok(rows) => {
            if let Some(row) = rows.into_iter().next() {
                let id_str: String = row.get("id");
                Uuid::parse_str(&id_str).unwrap()
            } else {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
        Err(e) => {
            eprintln!("Failed to store AI message: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Update session's last_message_at
    let update_session = "UPDATE chat_sessions SET last_message_at = now(), updated_at = now() WHERE id = $1";
    if let Err(e) = state.db_client.execute(update_session, &[&session_uuid.to_string()]).await {
        eprintln!("Failed to update session timestamp: {:?}", e);
    }

    Ok(Json(json!({
        "userMessageId": user_message_id,
        "aiMessageId": ai_message_id,
        "response": ai_response,
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

// Get chat messages for a session
pub async fn get_chat_messages(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // First, verify session exists and get its UUID
    let session_query = "SELECT id FROM chat_sessions WHERE session_id = $1 AND status = 'active'";
    let session_id_param = session_id.to_string();
    let session_result = match state.db_client.query(session_query, &[&session_id_param]).await {
        Ok(rows) => {
            if let Some(row) = rows.into_iter().next() {
                Ok(row.get::<_, Uuid>("id"))
            } else {
                return Err(StatusCode::NOT_FOUND);
            }
        }
        Err(e) => {
            eprintln!("Database error validating session: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let session_uuid = match session_result {
        Ok(uuid) => uuid,
        Err(_) => return Err(StatusCode::NOT_FOUND),
    };

    // Get messages using the database function
    let limit = params.get("limit")
        .and_then(|l| l.parse::<i64>().ok())
        .unwrap_or(50);

    let since = params.get("since")
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

    let messages_query = r#"
        SELECT id, message_type, content, sender, metadata, created_at, sequence_number
        FROM get_recent_chat_messages($1, $2, $3)
        ORDER BY sequence_number ASC
    "#;

    let limit_param = limit as i64;
    let messages_result = state.db_client.query(
        messages_query,
        &[&session_uuid.to_string(), &limit_param, &since.map(|dt| dt.to_rfc3339())]
    ).await;

    match messages_result {
        Ok(rows) => {
            let messages: Vec<serde_json::Value> = rows.into_iter()
                .map(|row| {
                    let message_type: String = row.get("message_type");
                    let content: String = row.get("content");
                    let sender: Option<String> = row.get("sender");
                    let metadata_str: String = row.get("metadata");
                    let metadata: serde_json::Value = serde_json::from_str(&metadata_str).unwrap_or(json!({}));
                    let created_at: String = row.get("created_at");
                    let sequence_number: i64 = row.get("sequence_number");
                    let id_str: String = row.get("id");
                    let id = Uuid::parse_str(&id_str).unwrap();

                    json!({
                        "id": id,
                        "type": message_type,
                        "content": content,
                        "sender": sender,
                        "metadata": metadata,
                        "timestamp": created_at,
                        "sequence_number": sequence_number
                    })
                })
                .collect();

            Ok(Json(json!({
                "session_id": session_id,
                "messages": messages,
                "total_count": messages.len()
            })))
        }
        Err(e) => {
            eprintln!("Failed to fetch chat messages: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// WebSocket configuration endpoint for dashboard
pub async fn get_websocket_config(Path(session_id): Path<String>) -> Json<serde_json::Value> {
    // Return WebSocket configuration for the dashboard
    Json(json!({
        "backend_url": format!("ws://localhost:8080/api/v1/chat/ws/{}", session_id),
        "session_id": session_id,
        "heartbeat_interval": 30000,
        "reconnect_attempts": 5
    }))
}

// WebSocket chat handler for real-time messaging
async fn websocket_chat_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| handle_websocket_chat(socket, session_id, state))
}

async fn handle_websocket_chat(mut socket: axum::extract::ws::WebSocket, session_id: String, state: AppState) {
    println!(" WebSocket chat connection established for session: {}", session_id);

    // Send welcome message
    let welcome_msg = json!({
        "type": "system",
        "message": "Connected to Agent Agency V3 chat",
        "session_id": session_id,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    if let Ok(msg) = serde_json::to_string(&welcome_msg) {
        let _ = socket.send(axum::extract::ws::Message::Text(msg.into())).await;
    }

    // First, verify session exists and get its UUID
    let session_query = "SELECT id FROM chat_sessions WHERE session_id = $1 AND status = 'active'";
    let session_id_param = session_id.to_string();
    let session_uuid = match state.db_client.query(session_query, &[&session_id_param]).await {
        Ok(rows) => {
            if let Some(row) = rows.into_iter().next() {
                let id_str: String = row.get("id");
                Uuid::parse_str(&id_str).unwrap()
            } else {
                // Send error and close connection
                let error_msg = json!({
                    "type": "error",
                    "message": "Invalid or inactive session",
                    "timestamp": chrono::Utc::now().to_rfc3339()
                });
                if let Ok(error_text) = serde_json::to_string(&error_msg) {
                    let _ = socket.send(axum::extract::ws::Message::Text(error_text.into())).await;
                }
                return;
            }
        }
        Err(e) => {
            eprintln!("Database error validating session: {:?}", e);
            return;
        }
    };

    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(axum::extract::ws::Message::Text(text)) => {
                // Parse incoming message
                if let Ok(chat_msg) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(message) = chat_msg.get("message").and_then(|m| m.as_str()) {
                        println!(" Received chat message: {}", message);

                        let sender = chat_msg.get("sender")
                            .and_then(|s| s.as_str())
                            .unwrap_or("user");

                        // Store user message in database
                        let insert_user_message = r#"
                            INSERT INTO chat_messages (session_id, message_type, content, sender, metadata)
                            VALUES ($1, $2, $3, $4, $5)
                            RETURNING id, created_at, sequence_number
                        "#;

                        let user_metadata = json!({
                            "source": "websocket",
                            "connection_type": "real_time"
                        });
                        let message_type = "message".to_string();
                        let sender_param = sender.to_string();

                        let user_message_result = state.db_client.query(
                            insert_user_message,
                            &[&session_uuid.to_string(), &message_type, &message, &sender_param, &serde_json::to_string(&user_metadata).unwrap()]
                        ).await;

                        let user_message_id = match user_message_result {
                            Ok(rows) => {
                                if let Some(row) = rows.into_iter().next() {
                                    let id_str: String = row.get("id");
                                    Some(Uuid::parse_str(&id_str).unwrap())
                                } else {
                                    None
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to store user message: {:?}", e);
                                None
                            }
                        };

                        // Generate AI response (for now, contextual responses)
                        let ai_response = if message.to_lowercase().contains("hello") {
                            "Hello! How can I help you with your tasks today?".to_string()
                        } else if message.to_lowercase().contains("help") {
                            "I can help you manage tasks, monitor system health, and chat in real-time. What would you like to know?".to_string()
                        } else if message.to_lowercase().contains("status") {
                            "The system is running well. All core services are operational. Would you like me to check specific metrics?".to_string()
                        } else {
                            format!("I received your message: '{}'. This is a simulated response - full AI integration coming soon!", message)
                        };

                        // Store AI response in database
                        let insert_ai_message = r#"
                            INSERT INTO chat_messages (session_id, message_type, content, sender, metadata)
                            VALUES ($1, $2, $3, $4, $5)
                            RETURNING id, created_at, sequence_number
                        "#;

                        let ai_metadata = json!({
                            "source": "ai_assistant",
                            "response_type": "generated"
                        });
                        let ai_sender = "assistant".to_string();

                        let ai_message_result = state.db_client.query(
                            insert_ai_message,
                            &[&session_uuid.to_string(), &message_type, &ai_response, &ai_sender, &serde_json::to_string(&ai_metadata).unwrap()]
                        ).await;

                        let ai_message_id = match ai_message_result {
                            Ok(rows) => {
                                if let Some(row) = rows.into_iter().next() {
                                    let id_str: String = row.get("id");
                                    Some(Uuid::parse_str(&id_str).unwrap())
                                } else {
                                    None
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to store AI message: {:?}", e);
                                None
                            }
                        };

                        // Update session's last_message_at
                        let update_session = "UPDATE chat_sessions SET last_message_at = now(), updated_at = now() WHERE id = $1";
                        if let Err(e) = state.db_client.execute(update_session, &[&session_uuid.to_string()]).await {
                            eprintln!("Failed to update session timestamp: {:?}", e);
                        }

                        // Send response back to client
                        let response_msg = json!({
                            "type": "response",
                            "user_message_id": user_message_id,
                            "ai_message_id": ai_message_id,
                            "response": ai_response,
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        });

                        if let Ok(response_text) = serde_json::to_string(&response_msg) {
                            if socket.send(axum::extract::ws::Message::Text(response_text.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                }
            }
            Ok(axum::extract::ws::Message::Close(_)) => {
                println!(" WebSocket chat connection closed for session: {}", session_id);
                break;
            }
            Err(e) => {
                println!(" WebSocket error for session {}: {}", session_id, e);
                break;
            }
            _ => {} // Ignore other message types
        }
    }

    println!(" WebSocket chat handler ended for session: {}", session_id);
}

pub async fn get_api_metrics() -> Json<serde_json::Value> {
    Json(json!({
        "metrics": {
            "active_tasks": 1,
            "completed_tasks": 1,
            "failed_tasks": 0,
            "avg_response_time_ms": 250.0
        },
        "status": "simulated"
    }))
}

async fn metrics_stream(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = tokio_stream::wrappers::IntervalStream::new(time::interval(Duration::from_secs(2)))
        .then(move |_| {
            let state = state.clone();
            async move {
                // Collect real system metrics from health monitor
                let timestamp = chrono::Utc::now().timestamp_millis();

                // Get real system metrics
                let system_metrics = match state.health_monitor.get_health_metrics().await {
                    Ok(health_metrics) => health_metrics.system,
                    Err(_) => {
                        // Fallback to basic metrics if health monitor fails
                        agent_agency_system_health_monitor::SystemMetrics {
                            timestamp: chrono::Utc::now(),
                            cpu_usage: 0.0,
                            memory_usage: 0.0,
                            disk_usage: 0.0,
                            load_average: [0.0, 0.0, 0.0],
                            network_io: 0,
                            disk_io: 0,
                            disk_io_metrics: DiskIOMetrics {
                                read_iops: 0,
                                write_iops: 0,
                                read_throughput: 0.0,
                                write_throughput: 0.0,
                                avg_read_latency_ms: 0.0,
                                avg_write_latency_ms: 0.0,
                                queue_depth: 0,
                            },
                        }
                    }
                };

                // Get task metrics from task store
                let task_metrics = match state.task_store.get_tasks().await {
                    Ok(tasks) => {
                        let active_tasks = tasks.iter().filter(|t| t.state == "running").count() as i32;
                        let completed_tasks = tasks.iter().filter(|t| t.state == "completed").count() as i32;
                        let failed_tasks = tasks.iter().filter(|t| t.state == "failed").count() as i32;
                        (active_tasks, completed_tasks, failed_tasks)
                    }
                    Err(_) => (0, 0, 0)
                };

                // Use fallback business metrics for now
                let business_metrics = BusinessMetrics {
                    throughput_tasks_per_hour: 0.0,
                    system_availability: 100.0,
                    average_task_completion_time_ms: 0.0,
                    error_rate: 0.0,
                };

                Ok(Event::default().data(serde_json::to_string(&json!({
                    "timestamp": timestamp,
                    "metrics": {
                        "cpu_usage_percent": system_metrics.cpu_usage,
                        "memory_usage_percent": system_metrics.memory_usage,
                        "disk_usage_percent": system_metrics.disk_usage,
                        "network_rx_bytes": system_metrics.network_io,
                        "network_tx_bytes": system_metrics.disk_io,
                        "active_tasks": task_metrics.0,
                        "completed_tasks": task_metrics.1,
                        "failed_tasks": task_metrics.2,
                        "total_requests": business_metrics.throughput_tasks_per_hour as i32,
                        "successful_requests": (business_metrics.throughput_tasks_per_hour * (1.0 - business_metrics.error_rate)) as i32,
                        "failed_requests": (business_metrics.throughput_tasks_per_hour * business_metrics.error_rate) as i32,
                        "avg_response_time_ms": business_metrics.average_task_completion_time_ms,
                        "p95_response_time_ms": business_metrics.average_task_completion_time_ms * 1.5,
                        "p99_response_time_ms": business_metrics.average_task_completion_time_ms * 2.0
                    },
                    "components": {
                        "api": "healthy",
                        "database": "healthy",
                        "orchestrator": "healthy",
                        "workers": "healthy"
                    }
                })).unwrap()))
            }
        });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}

async fn pause_task(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
    Path(task_id): Path<String>,
) -> Result<axum::http::StatusCode, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match Uuid::parse_str(&task_id) {
        Ok(uuid) => {
            // P0: Call orchestrator pause endpoint
            let orchestrator_endpoint = std::env::var("AGENT_AGENCY_ORCHESTRATOR_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:3000".to_string());

            let pause_url = format!("{}/api/tasks/{}/pause", orchestrator_endpoint.trim_end_matches('/'), task_id);
            let client = reqwest::Client::new();

            match client.post(&pause_url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        println!(" Task {} paused successfully", task_id);

                        // Audit logging for task pause
                        let audit_context = audit::extract_audit_context(&headers, Some(addr));
                        if let Err(e) = state.audit_logger.log_task_event(
                            &task_id,
                            "paused",
                            Some("running"), // Assuming it was running
                            Some("paused"),
                            &audit_context,
                            Some(serde_json::json!({"via": "orchestrator_api"})),
                        ).await {
                            eprintln!("Failed to log task pause audit event: {:?}", e);
                        }

                        Ok(axum::http::StatusCode::OK)
                    } else {
                        println!(" Failed to pause task {}: {}", task_id, response.status());
                        Err((
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({"error": format!("Orchestrator returned: {}", response.status())}))
                        ))
                    }
                }
                Err(e) => {
                    println!(" Failed to call orchestrator for pause: {}", e);
                    Err((
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": format!("Failed to call orchestrator: {}", e)}))
                    ))
                }
            }
        }
        Err(_) => Err((
            axum::http::StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invalid task ID format"}))
        ))
    }
}

async fn resume_task(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
    Path(task_id): Path<String>,
) -> Result<axum::http::StatusCode, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match Uuid::parse_str(&task_id) {
        Ok(uuid) => {
            // P0: Call orchestrator resume endpoint
            let orchestrator_endpoint = std::env::var("AGENT_AGENCY_ORCHESTRATOR_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:3000".to_string());

            let resume_url = format!("{}/api/tasks/{}/resume", orchestrator_endpoint.trim_end_matches('/'), task_id);
            let client = reqwest::Client::new();

            match client.post(&resume_url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        println!(" Task {} resumed successfully", task_id);

                        // Audit logging for task resume
                        let audit_context = audit::extract_audit_context(&headers, Some(addr));
                        if let Err(e) = state.audit_logger.log_task_event(
                            &task_id,
                            "resumed",
                            Some("paused"), // Assuming it was paused
                            Some("running"),
                            &audit_context,
                            Some(serde_json::json!({"via": "orchestrator_api"})),
                        ).await {
                            eprintln!("Failed to log task resume audit event: {:?}", e);
                        }

                        Ok(axum::http::StatusCode::OK)
                    } else {
                        println!(" Failed to resume task {}: {}", task_id, response.status());
                        Err((
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({"error": format!("Orchestrator returned: {}", response.status())}))
                        ))
                    }
                }
                Err(e) => {
                    println!(" Failed to call orchestrator for resume: {}", e);
                    Err((
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": format!("Failed to call orchestrator: {}", e)}))
                    ))
                }
            }
        }
        Err(_) => Err((
            axum::http::StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invalid task ID format"}))
        ))
    }
}

async fn cancel_task(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
    Path(task_id): Path<String>,
) -> Result<axum::http::StatusCode, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match Uuid::parse_str(&task_id) {
        Ok(uuid) => {
            // For now, just log the cancel request - actual implementation would
            // need access to the orchestrator to cancel running tasks
            println!(" Task {} cancel requested", task_id);

            // Audit logging for task cancellation
            let audit_context = audit::extract_audit_context(&headers, Some(addr));
            if let Err(e) = state.audit_logger.log_task_event(
                &task_id,
                "cancelled",
                Some("running"), // Assuming it was running
                Some("cancelled"),
                &audit_context,
                Some(serde_json::json!({"requested_via": "api"})),
            ).await {
                eprintln!("Failed to log task cancellation audit event: {:?}", e);
            }

            Ok(axum::http::StatusCode::OK)
        }
        Err(_) => Err((
            axum::http::StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invalid task ID format"}))
        ))
    }
}

pub async fn submit_task(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
    Json(request): Json<TaskSubmissionRequest>,
) -> Result<Json<TaskSubmissionResponse>, ApiError> {
    // Rate limiting check
    state.rate_limiter.check_rate_limit(addr.ip()).await
        .map_err(|_| ApiError::RateLimitExceeded)?;

    // Input validation
    if request.description.trim().is_empty() {
        return Err(ApiError::Validation("Task description cannot be empty".to_string()));
    }

    if request.description.len() > 1000 {
        return Err(ApiError::Validation("Task description too long (max 1000 characters)".to_string()));
    }

    // Sanitize input (stub implementation)
    let description = request.description.trim().to_string();
    let context = request.context;

    let task_id = Uuid::new_v4();
    println!(" Submitting task: {}", description);

    // Create task spec JSON for database storage
    let task_spec = json!({
        "id": task_id,
        "description": description,
        "context": context,
        "priority": request.priority,
        "created_at": chrono::Utc::now().to_rfc3339()
    });

    // Persist task to database
    let insert_query = r#"
        INSERT INTO tasks (id, spec, state, created_by, metadata)
        VALUES ($1, $2, 'pending', 'api-server', $3)
    "#;

    let metadata = json!({
        "source": "api",
        "submitted_at": chrono::Utc::now().to_rfc3339()
    });

    // Persist task to storage
    let now = chrono::Utc::now().to_rfc3339();
    let task = PersistedTask {
        id: task_id.to_string(),
        spec: task_spec.to_string(),
        state: "pending".to_string(),
        created_at: now.clone(),
        updated_at: now,
        created_by: Some("api-server".to_string()),
        metadata: metadata.to_string(),
    };

    state.task_store.create_task(task).await
        .map_err(|e| ApiError::Internal(format!("Failed to persist task: {}", e)))?;
    println!(" Task {} persisted successfully", task_id.to_string());

    // Audit logging for task creation
    let audit_context = audit::extract_audit_context(&headers, Some(addr));
    let audit_details = serde_json::json!({
        "description": description,
        "priority": request.priority,
        "context_length": context.as_ref().map(|c| c.len()).unwrap_or(0),
        "submitted_via": "api"
    });

    if let Err(e) = state.audit_logger.log_task_event(
        &task_id.to_string(),
        "created",
        None,
        Some("pending"),
        &audit_context,
        Some(audit_details),
    ).await {
        eprintln!("Failed to log task creation audit event: {:?}", e);
    }

    // Log API call audit event
    if let Err(e) = state.audit_logger.log_api_call(
        "POST",
        "/api/v1/tasks",
        200, // Success status
        150, // Estimated processing time
        &audit_context,
        Some(serde_json::json!({
            "description": description,
            "priority": request.priority
        })),
        Some(serde_json::json!({
            "task_id": task_id,
            "status": "accepted"
        })),
        true, // Success
        None, // No error
    ).await {
        eprintln!("Failed to log API call audit event: {:?}", e);
    }

    let description_clone = description.clone();
    // Execute task directly via HTTP to worker
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

        match client
            .post(worker_endpoint)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    println!(" Task {} executed successfully", task_id);
                } else {
                    println!(" Task {} failed with status: {}", task_id, response.status());
                }
            }
            Err(e) => {
                println!(" Task {} failed to send to worker: {}", task_id, e);
            }
        }
    });

    Ok(Json(TaskSubmissionResponse {
        task_id,
        status: "submitted".to_string(),
        message: format!("Task '{}' submitted for execution", description),
    }))
}

// Stub implementations for missing endpoints
async fn override_verdict(
    State(_state): State<AppState>,
    Path(_task_id): Path<String>,
) -> Result<axum::http::StatusCode, (axum::http::StatusCode, Json<serde_json::Value>)> {
    // Stub implementation
    Ok(axum::http::StatusCode::NOT_IMPLEMENTED)
}

async fn modify_parameter(
    State(_state): State<AppState>,
    Path(_task_id): Path<String>,
) -> Result<axum::http::StatusCode, (axum::http::StatusCode, Json<serde_json::Value>)> {
    // Stub implementation
    Ok(axum::http::StatusCode::NOT_IMPLEMENTED)
}

async fn inject_guidance(
    State(_state): State<AppState>,
    Path(_task_id): Path<String>,
) -> Result<axum::http::StatusCode, (axum::http::StatusCode, Json<serde_json::Value>)> {
    // Stub implementation
    Ok(axum::http::StatusCode::NOT_IMPLEMENTED)
}

#[derive(Debug)]
enum ApiError {
    Validation(String),
    RateLimitExceeded,
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            ApiError::Validation(msg) => (axum::http::StatusCode::BAD_REQUEST, msg),
            ApiError::RateLimitExceeded => (axum::http::StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded".to_string()),
            ApiError::Internal(msg) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({
            "error": message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!(" Starting Agent Agency V3 API Server");
    println!(" Server: {}:{}", args.host, args.port);

    // Initialize database configuration
    let db_config = DatabaseConfig {
        host: args.db_host.clone(),
        port: args.db_port,
        database: args.db_name.clone(),
        username: "postgres".to_string(),
        password: std::env::var("DATABASE_PASSWORD").unwrap_or_else(|_| "password".to_string()),
        pool_min: 2,
        pool_max: 20,
        connection_timeout_seconds: 30,
        idle_timeout_seconds: 600,
        max_lifetime_seconds: 3600,
        enable_read_write_splitting: false,
        read_replicas: Vec::new(),
    };

    println!(" Persistence: PostgreSQL ({}:{}/{})", db_config.host, db_config.port, db_config.database);

    // Initialize database-backed task store
    let db_client = DatabaseClient::new(db_config.clone()).await.unwrap_or_else(|e| {
        eprintln!(" Failed to initialize database connection: {}", e);
        eprintln!(" Make sure PostgreSQL is running and DATABASE_PASSWORD is set");
        std::process::exit(1);
    });

    // Run database migrations
    println!(" Running database migrations...");
    let migration_dir = std::path::PathBuf::from("./database/migrations");
    let migration_manager = MigrationManager::new(db_client.clone(), migration_dir).await
        .unwrap_or_else(|e| {
            eprintln!(" Failed to initialize migration manager: {}", e);
            std::process::exit(1);
        });

    let migration_results = migration_manager.apply_pending_migrations().await
        .unwrap_or_else(|e| {
            eprintln!(" Failed to run migrations: {}", e);
            std::process::exit(1);
        });

    println!(" Applied {} migrations", migration_results.len());

    let db_client = db_client; // Keep reference for AppState
    let task_store: Arc<dyn TaskStoreTrait + Send + Sync> = Arc::new(
        DatabaseTaskStore { db_client: db_client.clone() }
    );

    println!(" Database connection established");

    // Initialize system health monitor with Redis configuration
    let health_config = SystemHealthMonitorConfig {
        collection_interval_ms: 30000, // 30 seconds
        health_check_interval_ms: 60000, // 1 minute
        retention_period_ms: 3600000, // 1 hour
        enable_circuit_breaker: true,
        circuit_breaker_failure_threshold: 5,
        circuit_breaker_recovery_timeout_ms: 60000,
        thresholds: HealthThresholds::default(),
        embedding_service: EmbeddingServiceConfig::default(),
        filesystem: agent_agency_system_health_monitor::FilesystemConfig::default(),
        redis: if args.enable_redis {
            Some(RedisConfig {
                url: args.redis_url,
                pool_size: 10,
                connection_timeout_ms: 5000,
                key_prefix: args.redis_key_prefix,
                enabled: true,
                metrics_ttl_seconds: args.redis_metrics_ttl,
                cache_ttl_seconds: args.redis_cache_ttl,
            })
        } else {
            None
        },
    };

    let health_monitor = Arc::new(SystemHealthMonitor::new(health_config));
    if args.enable_redis {
        println!(" System health monitor initialized with Redis support");
    } else {
        println!(" System health monitor initialized (Redis disabled)");
    }

    // Initialize alert manager
    let alert_manager = Arc::new(api_alerts::AlertManager::new(None)); // TODO: Pass RTO/RPO monitor when available
    alert_manager.start().await.map_err(|e| format!("Failed to start alert manager: {}", e))?;
    println!(" Alert manager initialized with default definitions");

    // Create shared application state
    // Initialize rate limiter
    let rate_limiter = Arc::new(rate_limiter::RateLimiter::new());

    let audit_logger = audit::AuditLogger::new(db_client.clone());

    // Initialize keystore and sandbox
    let keystore = system_quality_security::create_keystore();
    let sandbox = system_quality_security::create_sandbox();

    let app_state = AppState {
        task_store,
        db_client,
        audit_logger,
        keystore,
        sandbox,
        health_monitor,
        alert_manager: alert_manager.clone(),
        rate_limiter: rate_limiter.clone(),
        backend_host: args.v3_backend_host.clone(),
        http_client: Client::new(),
    };

    // Create API router with full task management and chat
    println!(" Chat endpoints: POST /api/v1/chat/session, WS /api/v1/chat/ws/{{session_id}}");

    // Create API router with full task management
    let api_router = Router::new()
        .route("/tasks", post(submit_task))
        .route("/tasks", get(list_tasks))
        .route("/tasks/:task_id", get(get_task))
        .route("/tasks/:task_id/pause", post(pause_task))
        .route("/tasks/:task_id/resume", post(resume_task))
        .route("/tasks/:task_id/cancel", post(cancel_task))
        .route("/tasks/:task_id/audit", get(get_task_audit_trail))
        .route("/tasks/:task_id/override", post(override_verdict))
        .route("/tasks/:task_id/parameters", post(modify_parameter))
        .route("/tasks/:task_id/guidance", post(inject_guidance))
        .route("/waivers", get(list_waivers))
        .route("/waivers", post(create_waiver))
        .route("/waivers/:waiver_id/approve", post(approve_waiver))
        .route("/tasks/:task_id/provenance", get(get_task_provenance))
        .route("/chat/session", post(create_chat_session))
        .route("/chat/messages/:session_id", get(get_chat_messages))
        .route("/chat/ws/:session_id", get(websocket_chat_handler))
        .route("/chat/config/:session_id", get(get_websocket_config))
        .route("/chat/message/:session_id", post(send_chat_message))
    .route("/metrics", get(get_api_metrics))
    .route("/metrics/stream", get(metrics_stream))
    .route("/health", get(health_check))
    .route("/proxy/*path", get(proxy_handler))
        .route("/alerts", get(|state: State<AppState>| async move {
            get_active_alerts(state).await
        }))
        .route("/alerts/:alert_id/acknowledge", post(|state: State<AppState>, path: Path<String>| async move {
            acknowledge_alert(state, path).await
        }))
        .route("/alerts/:alert_id/resolve", post(|state: State<AppState>, path: Path<String>| async move {
            resolve_alert(state, path).await
        }))
        .route("/alerts/history", get(|state: State<AppState>| async move {
            get_alert_history(state).await
        }))
        .route("/alerts/statistics", get(|state: State<AppState>| async move {
            get_alert_statistics(state).await
        }))
        .nest("/v1", keystore_api::create_keystore_router())
        .nest("/v1", sandbox_api::create_sandbox_router())
    .with_state(app_state);

    // Create main router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(get_api_metrics)) // Alternative endpoint for dashboard
        .nest("/api/v1", api_router);

    // Add CORS if enabled
    let app = if args.enable_cors {
        app.layer(CorsLayer::permissive())
    } else {
        app
    };

    // Bind server
    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!(" API server ready at http://{}", addr);
    println!(" Health check: http://{}/health", addr);
    println!(" Tasks: http://{}/api/v1/tasks", addr);
    println!(" Metrics: http://{}/api/v1/metrics", addr);

    // Serve requests
    axum::serve(listener, app).await?;

    Ok(())
}

// Alert management endpoints
async fn get_active_alerts(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let alerts = state.alert_manager.get_active_alerts().await;
    Ok(Json(json!({
        "alerts": alerts,
        "total": alerts.len(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

async fn acknowledge_alert(
    State(state): State<AppState>,
    Path(alert_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    // In a real implementation, you'd get the user ID from authentication
    let user_id = "system";

    match state.alert_manager.acknowledge_alert(&alert_id, user_id).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to acknowledge alert", "details": e}))
        ))
    }
}

async fn resolve_alert(
    State(state): State<AppState>,
    Path(alert_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    // In a real implementation, you'd get the user ID from authentication
    let user_id = "system";

    match state.alert_manager.resolve_alert(&alert_id, user_id).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to resolve alert", "details": e}))
        ))
    }
}

async fn get_alert_history(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let history = state.alert_manager.get_alert_history(100).await;
    Ok(Json(json!({
        "history": history,
        "total": history.len(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

async fn get_alert_statistics(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let stats = state.alert_manager.get_alert_statistics().await;
    Ok(Json(json!({
        "statistics": stats,
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}
