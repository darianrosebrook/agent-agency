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
use uuid::Uuid;

use agent_agency_database::client::{DatabaseClient, MockDatabaseClient};

// Simple test API server for validating core database functionality

#[derive(Clone)]
struct AppState {
    db_client: Arc<DatabaseClient>,
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
    println!("📝 Creating task: {}", payload.description);

    // Create task in database
    let task_id = Uuid::new_v4();
    let spec = payload.spec.clone();

    // Insert task
    let insert_query = r#"
        INSERT INTO tasks (id, spec, state, created_by, metadata)
        VALUES ($1, $2, 'pending', 'test-user', '{}')
    "#;

    match state.db_client.execute_safe_query(insert_query).await {
        Ok(_) => {
            // Log audit event
            let audit_query = r#"
                INSERT INTO audit_logs (action, actor, resource_id, resource_type, change_summary)
                VALUES ('task_created', 'test-user', $1, 'task', $2)
            "#;
            let change_summary = json!({
                "description": payload.description,
                "spec": spec
            });

            if let Err(e) = state.db_client.execute(audit_query, &[&task_id, &change_summary]).await {
                println!("⚠️  Failed to log audit event: {}", e);
            }

            let response = TaskResponse {
                id: task_id,
                description: payload.description,
                status: "pending".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
            };

            println!("✅ Task created: {}", task_id);
            Ok(Json(response))
        }
        Err(e) => {
            println!("❌ Failed to create task: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    println!("📖 Getting task: {}", task_id);

    let uuid = match Uuid::parse_str(&task_id) {
        Ok(uuid) => uuid,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    // Get task
    let task_query = r#"
        SELECT id, spec, state, created_at, updated_at, acceptance_criteria
        FROM tasks WHERE id = $1
    "#;

    match state.db_client.query_one(task_query, &[&uuid]).await {
        Ok(row) => {
            let task_id: Uuid = row.get("id");
            let spec: serde_json::Value = row.get("spec");
            let state: String = row.get("state");
            let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
            let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");
            let acceptance_criteria: Vec<String> = row.get("acceptance_criteria");

            // Get audit events
            let events_query = r#"
                SELECT id, action, actor, change_summary, created_at
                FROM audit_logs
                WHERE resource_id = $1 AND resource_type = 'task'
                ORDER BY created_at DESC
                LIMIT 10
            "#;

            let events = match state.db_client.query(events_query, &[&uuid]).await {
                Ok(rows) => {
                    rows.iter().map(|row| {
                        let event_id: Uuid = row.get("id");
                        let action: String = row.get("action");
                        let actor: Option<String> = row.get("actor");
                        let change_summary: serde_json::Value = row.get("change_summary");
                        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");

                        json!({
                            "id": event_id,
                            "action": action,
                            "actor": actor,
                            "details": change_summary,
                            "timestamp": created_at.to_rfc3339()
                        })
                    }).collect::<Vec<_>>()
                }
                Err(e) => {
                    println!("⚠️  Failed to get events: {}", e);
                    Vec::new()
                }
            };

            let response = json!({
                "id": task_id,
                "title": spec.get("description").and_then(|d| d.as_str()).unwrap_or("Untitled Task"),
                "description": spec.get("context").and_then(|c| c.as_str()).unwrap_or(""),
                "status": state,
                "createdAt": created_at.to_rfc3339(),
                "updatedAt": updated_at.to_rfc3339(),
                "acceptanceCriteria": acceptance_criteria,
                "events": events
            });

            println!("✅ Task retrieved with {} events", events.len());
            Ok(Json(response))
        }
        Err(e) => {
            println!("❌ Task not found: {}", e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

async fn list_tasks(State(state): State<AppState>) -> Result<Json<Vec<TaskResponse>>, StatusCode> {
    println!("📋 Listing tasks");

    let query = r#"
        SELECT id, spec, state, created_at
        FROM tasks
        ORDER BY created_at DESC
        LIMIT 20
    "#;

    match state.db_client.query(query, &[]).await {
        Ok(rows) => {
            let tasks = rows.iter().map(|row| {
                let id: Uuid = row.get("id");
                let spec: serde_json::Value = row.get("spec");
                let state: String = row.get("state");
                let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");

                TaskResponse {
                    id,
                    description: spec.get("description").and_then(|d| d.as_str()).unwrap_or("Untitled").to_string(),
                    status: state,
                    created_at: created_at.to_rfc3339(),
                }
            }).collect::<Vec<_>>();

            println!("✅ Found {} tasks", tasks.len());
            Ok(Json(tasks))
        }
        Err(e) => {
            println!("❌ Failed to list tasks: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn create_waiver(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    println!("📋 Creating waiver");

    let title = payload.get("title").and_then(|t| t.as_str()).unwrap_or("Test Waiver");
    let reason = payload.get("reason").and_then(|r| r.as_str()).unwrap_or("emergency_hotfix");
    let description = payload.get("description").and_then(|d| d.as_str()).unwrap_or("Test waiver");
    let gates: Vec<String> = payload.get("gates")
        .and_then(|g| g.as_array())
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    let approved_by = payload.get("approved_by").and_then(|a| a.as_str()).unwrap_or("test-user");
    let impact_level = payload.get("impact_level").and_then(|i| i.as_str()).unwrap_or("low");
    let mitigation_plan = payload.get("mitigation_plan").and_then(|m| m.as_str()).unwrap_or("Test mitigation");
    let expires_at = payload.get("expires_at")
        .and_then(|e| e.as_str())
        .unwrap_or(&(chrono::Utc::now() + chrono::Duration::days(30)).to_rfc3339());

    let waiver_id = Uuid::new_v4();
    let expires_at_dt = chrono::DateTime::parse_from_rfc3339(&expires_at)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|_| chrono::Utc::now() + chrono::Duration::days(30));

    let query = r#"
        INSERT INTO waivers (id, title, reason, description, gates, approved_by, impact_level, mitigation_plan, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
    "#;

    match state.db_client.execute(query, &[
        &waiver_id,
        &title,
        &reason,
        &description,
        &gates,
        &approved_by,
        &impact_level,
        &mitigation_plan,
        &expires_at_dt,
    ]).await {
        Ok(_) => {
            println!("✅ Waiver created: {}", waiver_id);
            Ok(Json(json!({
                "id": waiver_id,
                "title": title,
                "status": "active"
            })))
        }
        Err(e) => {
            println!("❌ Failed to create waiver: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Test API Server");

    // Initialize database connection
    // For testing, we'll use a mock database client
    // In a real environment, this would connect to PostgreSQL
    println!("⚠️  Using mock database client for testing (no real database connection)");
    let db_client = Arc::new(MockDatabaseClient::new());

    let app_state = AppState { db_client };

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

    println!("✅ Test API server ready at http://{}", addr);
    println!("📊 Health check: http://{}", addr);
    println!("📝 Create task: POST http://{}/tasks", addr);
    println!("📖 Get task: GET http://{}/tasks/<uuid>", addr);
    println!("📋 List tasks: GET http://{}/tasks", addr);
    println!("📋 Create waiver: POST http://{}/waivers", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
