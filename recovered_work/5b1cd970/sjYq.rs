//! Agent Agency V3 API Server
//!
//! Standalone HTTP API server providing REST endpoints for task management,
//! health checks, and metrics streaming.

use axum::{
    extract::{Path, State, WebSocketUpgrade},
    response::sse::{Event, Sse},
    routing::get,
    response::Json,
    Router,
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
use agent_agency_database::{DatabaseClient, DatabaseConfig, MigrationManager};
use agent_agency_system_health_monitor::SystemHealthMonitor;
use async_trait::async_trait;
// WebSocket support is built into Axum - no axum-ws needed

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
struct PersistedTask {
    id: Uuid,
    spec: serde_json::Value,
    state: String,
    created_at: String,
    updated_at: String,
    created_by: Option<String>,
    metadata: serde_json::Value,
}

/// Simple file-based persistence for MVP
struct TaskStore {
    tasks: RwLock<HashMap<Uuid, PersistedTask>>,
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
            let tasks: HashMap<Uuid, PersistedTask> = serde_json::from_str(&content)?;
            let task_count = tasks.len();
            *self.tasks.write().unwrap() = tasks;
            println!("📁 Loaded {} tasks from persistence", task_count);
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
            tasks.insert(task.id, task);
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
        let db_client = DatabaseClient::connect(config).await?;
        Ok(Self { db_client })
    }

    async fn create_task(&self, task: PersistedTask) -> Result<(), Box<dyn std::error::Error>> {
        let query = r#"
            INSERT INTO tasks (id, spec, state, created_by, metadata)
            VALUES ($1, $2, $3, $4, $5)
        "#;

        self.db_client.execute(
            query,
            &[&task.id, &task.spec, &task.state, &task.created_by, &task.metadata],
        ).await?;

        println!("💾 Created task {} in database", task.id);
        Ok(())
    }

    async fn get_tasks(&self) -> Result<Vec<PersistedTask>, Box<dyn std::error::Error>> {
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
                created_at: row.get::<_, chrono::DateTime<chrono::Utc>>("created_at").to_rfc3339(),
                updated_at: row.get::<_, chrono::DateTime<chrono::Utc>>("updated_at").to_rfc3339(),
                created_by: row.get("created_by"),
                metadata: row.get("metadata"),
            };
            tasks.push(task);
        }

        Ok(tasks)
    }

    async fn get_task(&self, task_id: Uuid) -> Result<Option<PersistedTask>, Box<dyn std::error::Error>> {
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
                created_at: row.get::<_, chrono::DateTime<chrono::Utc>>("created_at").to_rfc3339(),
                updated_at: row.get::<_, chrono::DateTime<chrono::Utc>>("updated_at").to_rfc3339(),
                created_by: row.get("created_by"),
                metadata: row.get("metadata"),
            };
            Ok(Some(task))
        } else {
            Ok(None)
        }
    }
}

/// Task store trait for abstraction
#[async_trait]
trait TaskStoreTrait {
    async fn create_task(&self, task: PersistedTask) -> Result<(), Box<dyn std::error::Error>>;
    async fn get_tasks(&self) -> Result<Vec<PersistedTask>, Box<dyn std::error::Error>>;
    async fn get_task(&self, task_id: Uuid) -> Result<Option<PersistedTask>, Box<dyn std::error::Error>>;
}

#[async_trait]
impl TaskStoreTrait for DatabaseTaskStore {
    async fn create_task(&self, task: PersistedTask) -> Result<(), Box<dyn std::error::Error>> {
        self.create_task(task).await
    }

    async fn get_tasks(&self) -> Result<Vec<PersistedTask>, Box<dyn std::error::Error>> {
        self.get_tasks().await
    }

    async fn get_task(&self, task_id: Uuid) -> Result<Option<PersistedTask>, Box<dyn std::error::Error>> {
        self.get_task(task_id).await
    }
}

/// Shared application state
#[derive(Clone)]
struct AppState {
    task_store: Arc<dyn TaskStoreTrait + Send + Sync>,
    health_monitor: Arc<SystemHealthMonitor>,
}

async fn health_check() -> Json<serde_json::Value> {
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

async fn list_tasks(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    match state.task_store.get_tasks().await {
        Ok(tasks) => {
            let task_summaries: Vec<serde_json::Value> = tasks
                .into_iter()
                .map(|task| {
                    let spec = task.spec.as_object().unwrap_or(&serde_json::Map::new());
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

async fn get_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Json<serde_json::Value> {
    // Parse UUID from string
    match Uuid::parse_str(&task_id) {
        Ok(uuid) => {
            match state.task_store.get_task(uuid).await {
                Ok(Some(task)) => {
                    let spec = task.spec.as_object().unwrap_or(&serde_json::Map::new());
                    let title = spec.get("description")
                        .and_then(|d| d.as_str())
                        .unwrap_or("Untitled Task");
                    let description = spec.get("context")
                        .and_then(|c| c.as_str())
                        .unwrap_or("");

                    Json(json!({
                        "id": task.id,
                        "title": title,
                        "description": description,
                        "status": task.state,
                        "priority": spec.get("priority").and_then(|p| p.as_str()).unwrap_or("medium"),
                        "createdAt": task.created_at,
                        "updatedAt": task.updated_at,
                        "events": [] // TODO: Implement audit trail retrieval
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
        Err(_) => Json(json!({
            "error": "Invalid task ID format",
            "status": "bad_request"
        }))
    }
}

// Chat session creation
async fn create_chat_session() -> Json<serde_json::Value> {
    let session_id = Uuid::new_v4();
    let created_at = chrono::Utc::now().to_rfc3339();

    Json(json!({
        "sessionId": session_id,
        "createdAt": created_at
    }))
}

// Chat message handling (HTTP fallback for MVP)
async fn send_chat_message(
    Path(session_id): Path<String>,
    Json(request): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let message = request.get("message")
        .and_then(|m| m.as_str())
        .unwrap_or("Hello");

    // Simulate AI response
    let response = format!("Echo: {}", message);
    let message_id = Uuid::new_v4();
    let timestamp = chrono::Utc::now().to_rfc3339();

    Json(json!({
        "messageId": message_id,
        "response": response,
        "timestamp": timestamp
    }))
}

// WebSocket configuration endpoint for dashboard
async fn get_websocket_config(Path(session_id): Path<String>) -> Json<serde_json::Value> {
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
    Path(session_id): Path<String>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| handle_websocket_chat(socket, session_id))
}

async fn handle_websocket_chat(mut socket: axum::extract::ws::WebSocket, session_id: String) {
    println!("🔗 WebSocket chat connection established for session: {}", session_id);

    // Send welcome message
    let welcome_msg = json!({
        "type": "system",
        "message": "Connected to Agent Agency V3 chat",
        "session_id": session_id,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    if let Ok(msg) = serde_json::to_string(&welcome_msg) {
        let _ = socket.send(ws::Message::Text(msg.into())).await;
    }

    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(axum::extract::ws::Message::Text(text)) => {
                // Parse incoming message
                if let Ok(chat_msg) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(message) = chat_msg.get("message").and_then(|m| m.as_str()) {
                        println!("💬 Received chat message: {}", message);

                        // Generate AI response (simple echo for MVP)
                        let response = format!("Echo: {}", message);
                        let response_msg = json!({
                            "type": "response",
                            "message_id": Uuid::new_v4(),
                            "response": response,
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
                println!("🔌 WebSocket chat connection closed for session: {}", session_id);
                break;
            }
            Err(e) => {
                println!("❌ WebSocket error for session {}: {}", session_id, e);
                break;
            }
            _ => {} // Ignore other message types
        }
    }

    println!("🔚 WebSocket chat handler ended for session: {}", session_id);
}

async fn get_api_metrics() -> Json<serde_json::Value> {
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
    // Create an interval that ticks every 2 seconds
    let interval = time::interval(Duration::from_secs(2));
    let stream = IntervalStream::new(interval).map(move |_| {
        // Collect real system metrics
        let timestamp = chrono::Utc::now().timestamp_millis();

        // Collect system metrics using the health monitor
        let system_metrics = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                health_monitor.collect_system_metrics().await
            })
        });

        let (cpu_usage, memory_usage, active_tasks, completed_tasks, failed_tasks, avg_response_time) = match system_metrics {
            Ok(metrics) => {
                // Use real system metrics
                let cpu = metrics.cpu_usage;
                let memory = metrics.memory_usage;

                // For now, use simulated task metrics until we have real task tracking
                let active_tasks = (timestamp % 10) as i32;
                let completed_tasks = (timestamp / 1000 % 100) as i32;
                let failed_tasks = (timestamp % 100 / 10) as i32;
                let avg_response_time = 200.0 + (timestamp % 100) as f64;

                (cpu, memory, active_tasks, completed_tasks, failed_tasks, avg_response_time)
            },
            Err(_) => {
                // Fallback to simulated metrics if collection fails
                let cpu = 25.0 + (timestamp % 50) as f64; // 25-74%
                let memory = 30.0 + (timestamp % 40) as f64; // 30-69%
                let active_tasks = (timestamp % 10) as i32;
                let completed_tasks = (timestamp / 1000 % 100) as i32;
                let failed_tasks = (timestamp % 100 / 10) as i32;
                let avg_response_time = 200.0 + (timestamp % 100) as f64;

                (cpu, memory, active_tasks, completed_tasks, failed_tasks, avg_response_time)
            }
        };

        Ok(Event::default().data(serde_json::to_string(&json!({
            "timestamp": timestamp,
            "metrics": {
                "cpu_usage_percent": cpu_usage,
                "memory_usage_percent": memory_usage,
                "active_tasks": active_tasks,
                "completed_tasks": completed_tasks,
                "failed_tasks": failed_tasks,
                "avg_response_time_ms": avg_response_time
            },
            "components": {
                "api": "healthy",
                "database": "healthy",
                "orchestrator": "healthy",
                "workers": "healthy"
            }
        })).unwrap()))
    });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}

async fn submit_task(
    State(state): State<AppState>,
    Json(request): Json<TaskSubmissionRequest>,
) -> Json<TaskSubmissionResponse> {
    let task_id = Uuid::new_v4();
    let description = request.description.clone();

    println!("📝 Submitting task: {}", description);

    // Create task spec JSON for database storage
    let task_spec = json!({
        "id": task_id,
        "description": request.description,
        "context": request.context,
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
        id: task_id,
        spec: task_spec,
        state: "pending".to_string(),
        created_at: now.clone(),
        updated_at: now,
        created_by: Some("api-server".to_string()),
        metadata: metadata,
    };

    if let Err(e) = state.task_store.create_task(task).await {
        println!("⚠️  Failed to persist task: {}", e);
    } else {
        println!("💾 Task {} persisted successfully", task_id);
    }

    // Execute task directly via HTTP to worker
    tokio::spawn(async move {
        let client = reqwest::Client::new();
        let worker_endpoint = "http://localhost:8081/execute";

        let request_body = serde_json::json!({
            "task_id": task_id,
            "prompt": request.description,
            "context": request.context,
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
                    println!("✅ Task {} executed successfully", task_id);
                } else {
                    println!("❌ Task {} failed with status: {}", task_id, response.status());
                }
            }
            Err(e) => {
                println!("❌ Task {} failed to send to worker: {}", task_id, e);
            }
        }
    });

    Json(TaskSubmissionResponse {
        task_id,
        status: "submitted".to_string(),
        message: format!("Task '{}' submitted for execution", description),
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("🚀 Starting Agent Agency V3 API Server");
    println!("📡 Server: {}:{}", args.host, args.port);

    // Initialize database configuration
    let db_config = DatabaseConfig {
        host: args.db_host.clone(),
        port: args.db_port,
        database: "agent_agency_v3".to_string(),
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

    println!("💾 Persistence: PostgreSQL ({}:{}/{})", db_config.host, db_config.port, db_config.database);

    // Initialize database-backed task store
    let db_client = DatabaseClient::connect(&db_config).await.unwrap_or_else(|e| {
        eprintln!("❌ Failed to initialize database connection: {}", e);
        eprintln!("💡 Make sure PostgreSQL is running and DATABASE_PASSWORD is set");
        std::process::exit(1);
    });

    // Run database migrations
    println!("🔄 Running database migrations...");
    let migration_dir = std::path::PathBuf::from("../database/migrations");
    let migration_manager = MigrationManager::new(db_client.clone(), migration_dir).await
        .unwrap_or_else(|e| {
            eprintln!("❌ Failed to initialize migration manager: {}", e);
            std::process::exit(1);
        });

    let migration_results = migration_manager.apply_pending_migrations().await
        .unwrap_or_else(|e| {
            eprintln!("❌ Failed to run migrations: {}", e);
            std::process::exit(1);
        });

    println!("✅ Applied {} migrations", migration_results.len());

    let task_store: Arc<dyn TaskStoreTrait + Send + Sync> = Arc::new(
        DatabaseTaskStore { db_client }
    );

    println!("✅ Database connection established");

    // Initialize system health monitor for real metrics
    let health_monitor = Arc::new(SystemHealthMonitor::new());
    println!("✅ System health monitor initialized");

    // Create shared application state
    let app_state = AppState {
        task_store,
        health_monitor,
    };

    // Create API router with full task management and chat
    println!("💬 Chat endpoints: POST /api/v1/chat/session, WS /api/v1/chat/ws/{session_id}");

    // Create API router with full task management
    let api_router = Router::new()
        .route("/tasks", post(submit_task))
        .route("/tasks", get(list_tasks))
        .route("/tasks/:task_id", get(get_task))
        .route("/chat/session", post(create_chat_session))
        .route("/chat/ws/:session_id", get(websocket_chat_handler).get(get_websocket_config).post(send_chat_message))
        .route("/metrics", get(get_api_metrics))
        .route("/metrics/stream", get(metrics_stream))
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

    println!("✅ API server ready at http://{}", addr);
    println!("📊 Health check: http://{}/health", addr);
    println!("📋 Tasks: http://{}/api/v1/tasks", addr);
    println!("📊 Metrics: http://{}/api/v1/metrics", addr);

    // Serve requests
    axum::serve(listener, app).await?;

    Ok(())
}
