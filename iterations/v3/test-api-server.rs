use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use uuid::Uuid;
use std::collections::HashMap;

// Simple in-memory database simulation for testing

#[derive(Clone)]
struct MockDatabase {
    tasks: Arc<RwLock<HashMap<Uuid, serde_json::Value>>>,
    audit_logs: Arc<RwLock<Vec<serde_json::Value>>>,
    waivers: Arc<RwLock<Vec<serde_json::Value>>>,
}

impl MockDatabase {
    fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            audit_logs: Arc::new(RwLock::new(Vec::new())),
            waivers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    async fn create_task(&self, task_id: Uuid, spec: &serde_json::Value, description: &str) -> Result<(), String> {
        let mut tasks = self.tasks.write().await;
        let task = json!({
            "id": task_id,
            "spec": spec,
            "state": "pending",
            "created_at": chrono::Utc::now().to_rfc3339(),
            "updated_at": chrono::Utc::now().to_rfc3339(),
            "created_by": "test-user",
            "metadata": {},
            "acceptance_criteria": []
        });
        tasks.insert(task_id, task);

        // Log audit event
        let mut audit_logs = self.audit_logs.write().await;
        let audit_event = json!({
            "id": Uuid::new_v4(),
            "action": "task_created",
            "actor": "test-user",
            "resource_id": task_id,
            "resource_type": "task",
            "change_summary": {
                "description": description,
                "spec": spec
            },
            "created_at": chrono::Utc::now().to_rfc3339()
        });
        audit_logs.push(audit_event);

        println!(" Mock DB: Created task {}", task_id);
        Ok(())
    }

    async fn get_task(&self, task_id: &Uuid) -> Option<serde_json::Value> {
        let tasks = self.tasks.read().await;
        tasks.get(task_id).cloned()
    }

    async fn list_tasks(&self) -> Vec<serde_json::Value> {
        let tasks = self.tasks.read().await;
        tasks.values().cloned().collect()
    }

    async fn get_task_events(&self, task_id: &Uuid) -> Vec<serde_json::Value> {
        let audit_logs = self.audit_logs.read().await;
        audit_logs.iter()
            .filter(|event| event.get("resource_id") == Some(&json!(task_id)))
            .cloned()
            .collect()
    }

    async fn create_waiver(&self, waiver: serde_json::Value) -> Result<serde_json::Value, String> {
        let waiver_id = Uuid::new_v4();
        let mut waiver_with_id = waiver.clone();
        if let Some(obj) = waiver_with_id.as_object_mut() {
            obj.insert("id".to_string(), json!(waiver_id));
            obj.insert("status".to_string(), json!("active"));
            obj.insert("created_at".to_string(), json!(chrono::Utc::now().to_rfc3339()));
        }

        let mut waivers = self.waivers.write().await;
        waivers.push(waiver_with_id.clone());

        println!(" Mock DB: Created waiver {}", waiver_id);
        Ok(waiver_with_id)
    }
}

// Simple test API server for validating core database functionality

#[derive(Clone)]
struct AppState {
    db: Arc<MockDatabase>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TaskRequest {
    description: String,
    spec: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct TaskResponse {
    id: Uuid,
    description: String,
    status: String,
    created_at: String,
}

async fn health_check() -> &'static str {
    "OK"
}

async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<TaskRequest>,
) -> Result<Json<TaskResponse>, StatusCode> {
    println!(" Creating task: {}", payload.description);

    // Create task in mock database
    let task_id = Uuid::new_v4();
    let spec = payload.spec.clone();

    match state.db.create_task(task_id, &spec, &payload.description).await {
        Ok(_) => {
            let response = TaskResponse {
                id: task_id,
                description: payload.description,
                status: "pending".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
            };

            println!(" Task created: {}", task_id);
            Ok(Json(response))
        }
        Err(e) => {
            println!(" Failed to create task: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    println!(" Getting task: {}", task_id);

    let uuid = match Uuid::parse_str(&task_id) {
        Ok(uuid) => uuid,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    // Get task from mock database
    match state.db.get_task(&uuid).await {
        Some(task) => {
            let task_id: Uuid = task.get("id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()).unwrap_or(uuid);
            let spec: serde_json::Value = task.get("spec").cloned().unwrap_or(json!({}));
            let state_val: String = task.get("state").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
            let created_at: String = task.get("created_at").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let updated_at: String = task.get("updated_at").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let acceptance_criteria: Vec<String> = task.get("acceptance_criteria")
                .and_then(|v| v.as_array())
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();

            // Get audit events
            let events = state.db.get_task_events(&uuid).await;

            let response = json!({
                "id": task_id,
                "title": spec.get("description").and_then(|d| d.as_str()).unwrap_or("Untitled Task"),
                "description": spec.get("context").and_then(|c| c.as_str()).unwrap_or(""),
                "status": state_val,
                "createdAt": created_at,
                "updatedAt": updated_at,
                "acceptanceCriteria": acceptance_criteria,
                "events": events
            });

            println!(" Task retrieved with {} events", events.len());
            Ok(Json(response))
        }
        None => {
            println!(" Task not found: {}", task_id);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

async fn list_tasks(State(state): State<AppState>) -> Result<Json<Vec<TaskResponse>>, StatusCode> {
    println!(" Listing tasks");

    let tasks_data = state.db.list_tasks().await;
    let tasks = tasks_data.iter().map(|task| {
        let id: Uuid = task.get("id").and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok()).unwrap_or(Uuid::new_v4());
        let spec: serde_json::Value = task.get("spec").cloned().unwrap_or(json!({}));
        let state_val: String = task.get("state").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
        let created_at: String = task.get("created_at").and_then(|v| v.as_str()).unwrap_or("").to_string();

        TaskResponse {
            id,
            description: spec.get("description").and_then(|d| d.as_str()).unwrap_or("Untitled").to_string(),
            status: state_val,
            created_at,
        }
    }).collect::<Vec<_>>();

    println!(" Found {} tasks", tasks.len());
    Ok(Json(tasks))
}

async fn create_waiver(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    println!(" Creating waiver");

    match state.db.create_waiver(payload).await {
        Ok(waiver) => {
            let waiver_id = waiver.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");
            println!(" Waiver created: {}", waiver_id);
            Ok(Json(waiver))
        }
        Err(e) => {
            println!(" Failed to create waiver: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(" Starting Test API Server");

    // Initialize mock database for testing
    println!("⚠️  Using in-memory mock database for testing");
    let db = Arc::new(MockDatabase::new());

    let app_state = AppState { db };

    // Create router
    let app = Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/tasks", post(create_task))
        .route("/tasks", get(list_tasks))
        .route("/tasks/:task_id", get(get_task))
        .route("/waivers", post(create_waiver))
        .with_state(app_state);

    // Start server
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await?;

    println!(" Test API server ready at http://{}", addr);
    println!(" Health check: http://{}", addr);
    println!(" Create task: POST http://{}/tasks", addr);
    println!(" Get task: GET http://{}/tasks/<uuid>", addr);
    println!(" List tasks: GET http://{}/tasks", addr);
    println!(" Create waiver: POST http://{}/waivers", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
