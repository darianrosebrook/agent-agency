//! HTTP Handlers
//!
//! Axum handlers for API endpoints.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::service::ApiService;
use crate::types::{
    ChainOfThoughtResponse, CouncilDecisionsResponse, ErrorResponse, HealthResponse,
    MetricsResponse, ProbeRequest, ProbeResponse, SubmitTaskRequest, SubmitTaskResponse,
    TaskStatusResponse, WorkerActionsResponse,
};

/// Shared application state
pub type AppState = Arc<ApiService>;

/// Submit a new task for evaluation
///
/// POST /api/v1/tasks
pub async fn submit_task(
    State(service): State<AppState>,
    Json(request): Json<SubmitTaskRequest>,
) -> Result<Json<SubmitTaskResponse>, (StatusCode, Json<ErrorResponse>)> {
    tracing::info!(
        title = %request.title,
        priority = ?request.priority,
        "Submitting task"
    );

    service.submit_task(request).await.map(Json).map_err(|e| {
        tracing::error!(error = %e.message, "Task submission failed");
        (StatusCode::BAD_REQUEST, Json(e))
    })
}

/// Get task status by ID
///
/// GET /api/v1/tasks/:id
pub async fn get_task(
    State(service): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskStatusResponse>, (StatusCode, Json<ErrorResponse>)> {
    tracing::debug!(task_id = %task_id, "Getting task status");

    service.get_task(&task_id).await.map(Json).map_err(|e| {
        let status = if e.code == "NOT_FOUND" {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        };
        (status, Json(e))
    })
}

/// Health check endpoint
///
/// GET /health
pub async fn health(State(service): State<AppState>) -> Json<HealthResponse> {
    Json(service.health().await)
}

/// Metrics endpoint for LLM probing analysis
///
/// GET /metrics
pub async fn metrics(State(service): State<AppState>) -> Json<MetricsResponse> {
    Json(service.metrics().await)
}

/// Probe LLM endpoint for testing inference
///
/// POST /api/v1/probe
pub async fn probe_llm(
    State(service): State<AppState>,
    Json(request): Json<ProbeRequest>,
) -> Result<Json<ProbeResponse>, (StatusCode, Json<ErrorResponse>)> {
    tracing::info!(
        prompt_len = request.prompt.len(),
        max_tokens = request.max_tokens,
        temperature = request.temperature,
        "LLM probe request"
    );

    service.probe_llm(request).await.map(Json).map_err(|e| {
        tracing::error!(error = %e.message, "LLM probe failed");
        (StatusCode::INTERNAL_SERVER_ERROR, Json(e))
    })
}

/// Get chain-of-thought reasoning for a task
///
/// GET /api/v1/tasks/:id/chain-of-thought
pub async fn get_chain_of_thought(
    State(service): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<ChainOfThoughtResponse>, (StatusCode, Json<ErrorResponse>)> {
    tracing::debug!(task_id = %task_id, "Getting chain-of-thought");

    service
        .get_chain_of_thought(&task_id)
        .await
        .map(Json)
        .map_err(|e| {
            let status = if e.code == "NOT_FOUND" {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (status, Json(e))
        })
}

/// Get council decisions for a task
///
/// GET /api/v1/tasks/:id/council-decisions
pub async fn get_council_decisions(
    State(service): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<CouncilDecisionsResponse>, (StatusCode, Json<ErrorResponse>)> {
    tracing::debug!(task_id = %task_id, "Getting council decisions");

    service
        .get_council_decisions(&task_id)
        .await
        .map(Json)
        .map_err(|e| {
            let status = if e.code == "NOT_FOUND" {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (status, Json(e))
        })
}

/// Get worker actions for a task
///
/// GET /api/v1/tasks/:id/worker-actions
pub async fn get_worker_actions(
    State(service): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<WorkerActionsResponse>, (StatusCode, Json<ErrorResponse>)> {
    tracing::debug!(task_id = %task_id, "Getting worker actions");

    service
        .get_worker_actions(&task_id)
        .await
        .map(Json)
        .map_err(|e| {
            let status = if e.code == "NOT_FOUND" {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (status, Json(e))
        })
}

/// API version info
///
/// GET /api/v1
pub async fn api_info() -> impl IntoResponse {
    Json(serde_json::json!({
        "name": "Agent Agency V4 API",
        "version": env!("CARGO_PKG_VERSION"),
        "api_version": "v1",
        "endpoints": {
            "tasks": "/api/v1/tasks",
            "task_status": "/api/v1/tasks/:id",
            "chain_of_thought": "/api/v1/tasks/:id/chain-of-thought",
            "council_decisions": "/api/v1/tasks/:id/council-decisions",
            "worker_actions": "/api/v1/tasks/:id/worker-actions",
            "probe": "/api/v1/probe",
            "health": "/health",
            "metrics": "/metrics"
        }
    }))
}

/// 404 handler
pub async fn not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse::new("NOT_FOUND", "Endpoint not found")),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::{get, post},
        Router,
    };
    use tower::ServiceExt;

    fn app() -> Router {
        let service = Arc::new(ApiService::with_defaults());
        Router::new()
            .route("/api/v1/tasks", post(submit_task))
            .route("/api/v1/tasks/:id", get(get_task))
            .route("/api/v1/probe", post(probe_llm))
            .route("/health", get(health))
            .route("/metrics", get(metrics))
            .with_state(service)
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let app = app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_metrics_endpoint() {
        let app = app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/metrics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_submit_task_endpoint() {
        let app = app();

        let body = serde_json::json!({
            "title": "Test task",
            "description": "Test description"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/tasks")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_nonexistent_task() {
        let app = app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/tasks/nonexistent-id")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_probe_llm_endpoint() {
        let app = app();

        let body = serde_json::json!({
            "prompt": "Hello, how are you?",
            "max_tokens": 50,
            "temperature": 0.7
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/probe")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
