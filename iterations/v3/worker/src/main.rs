//! Agent Agency Worker Service
//!
//! Simple worker service that accepts task execution requests via HTTP POST
//! and simulates task execution with realistic timing and results.

use axum::{
    routing::post,
    Json, Router,
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "agent-agency-worker")]
#[command(about = "Agent Agency Worker Service")]
struct Args {
    /// Server host
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Server port
    #[arg(long, default_value = "8081")]
    port: u16,

    /// Enable CORS
    #[arg(long)]
    enable_cors: bool,

    /// Worker ID
    #[arg(long)]
    worker_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TaskExecutionRequest {
    task_id: Uuid,
    prompt: String,
    execution_mode: Option<String>,
    context: Option<String>,
    requirements: Option<String>,
    caws_spec: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct TaskExecutionResponse {
    task_id: Uuid,
    worker_id: String,
    status: String,
    stdout: String,
    stderr: String,
    exit_code: i32,
    execution_time_ms: u64,
    started_at: String,
    completed_at: String,
}

#[derive(Debug, Deserialize)]
struct TaskCancelRequest {
    task_id: Uuid,
    reason: String,
}

#[derive(Debug, Serialize)]
struct TaskCancelResponse {
    task_id: Uuid,
    worker_id: String,
    status: String,
    cancelled: bool,
    reason: String,
}

async fn execute_task(
    Json(request): Json<TaskExecutionRequest>,
) -> Json<TaskExecutionResponse> {
    let started_at = chrono::Utc::now();
    let worker_id = format!("worker-{}", request.task_id.simple());

    // Check execution mode
    let is_dry_run = request.execution_mode.as_deref() == Some("dry_run");
    let mode_indicator = if is_dry_run { "👁️  DRY-RUN" } else { "" };

    println!("{} Worker {} executing task {}", mode_indicator, worker_id, request.task_id);

    // Simulate task execution with realistic timing
    let execution_time = std::time::Duration::from_millis(500 + (request.task_id.as_u128() % 1000) as u64);
    tokio::time::sleep(execution_time).await;

    let completed_at = chrono::Utc::now();

    // Handle execution based on mode
    let (stdout, stderr, exit_code) = if is_dry_run {
        // Dry-run: simulate but indicate no changes made
        if request.task_id.as_u128() % 10 == 0 {
            // 10% failure rate (simulated)
            ("".to_string(), "Simulated task failure (dry-run)".to_string(), 1)
        } else {
            // Successful simulation
            (format!("DRY-RUN: Task {} would complete successfully\nSimulated output: {}\n\n No actual filesystem changes were made", request.task_id, request.prompt), "".to_string(), 0)
        }
    } else {
        // Normal execution
        if request.task_id.as_u128() % 10 == 0 {
            // 10% failure rate
            ("".to_string(), "Simulated task failure".to_string(), 1)
        } else {
            // Successful execution
            (format!("Task {} completed successfully\nOutput: {}", request.task_id, request.prompt), "".to_string(), 0)
        }
    };

    let response = TaskExecutionResponse {
        task_id: request.task_id,
        worker_id,
        status: if exit_code == 0 { "completed" } else { "failed" }.to_string(),
        stdout,
        stderr,
        exit_code,
        execution_time_ms: execution_time.as_millis() as u64,
        started_at: started_at.to_rfc3339(),
        completed_at: completed_at.to_rfc3339(),
    };

    println!(" Worker completed task {} in {}ms", request.task_id, execution_time.as_millis());

    Json(response)
}

async fn cancel_task(
    Json(request): Json<TaskCancelRequest>,
) -> Json<TaskCancelResponse> {
    let worker_id = format!("worker-{}", request.task_id.simple());

    println!(" Worker {} cancelling task {}: {}", worker_id, request.task_id, request.reason);

    // In a real implementation, this would signal the task execution to stop
    // For now, we simulate successful cancellation
    let cancelled = true;

    let response = TaskCancelResponse {
        task_id: request.task_id,
        worker_id,
        status: "cancelled".to_string(),
        cancelled,
        reason: request.reason.clone(),
    };

    println!(" Worker cancelled task {}", request.task_id);

    Json(response)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!(" Starting Agent Agency Worker");
    println!(" Server: {}:{}", args.host, args.port);

    let worker_id = args.worker_id.unwrap_or_else(|| "default-worker".to_string());
    println!(" Worker ID: {}", worker_id);

    // Create router
    let app = Router::new()
        .route("/execute", post(execute_task))
        .route("/cancel", post(cancel_task));

    // Add CORS if enabled
    let app = if args.enable_cors {
        app.layer(CorsLayer::permissive())
    } else {
        app
    };

    // Bind server
    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!(" Worker ready at http://{}", addr);
    println!(" Execution endpoint: http://{}/execute", addr);
    println!(" Cancel endpoint: http://{}/cancel", addr);

    // Serve requests
    axum::serve(listener, app).await?;

    Ok(())
}

