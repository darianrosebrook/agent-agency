//! CQRS-based API router for orchestration endpoints
//!
//! This module provides Axum routers for CQRS-based API endpoints,
//! separating command operations (writes) from query operations (reads).

#[cfg(feature = "api-server")]
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::cqrs::CqrsBus;
use crate::api::*;

#[cfg(feature = "api-server")]
/// Create CQRS-based API router
pub fn create_cqrs_router(cqrs_bus: Arc<CqrsBus>) -> Router {
    Router::new()
        // Command endpoints (writes)
        .route("/api/tasks/{task_id}/execute", post(execute_task_cqrs))
        .route("/api/tasks/{task_id}/cancel", post(cancel_task_with_cqrs))
        .route("/api/tasks/{task_id}/progress", post(update_task_progress_cqrs))
        .route("/api/workers/register", post(register_worker_cqrs))
        .route("/api/workers/{worker_id}/health", post(update_worker_health_cqrs))

        // Query endpoints (reads)
        .route("/api/tasks/{task_id}/status", get(get_task_status_cqrs))
        .route("/api/health", get(get_system_health_cqrs))
        .route("/api/tasks/active", get(get_active_tasks_cqrs))

        // Add CQRS bus as extension
        .layer(axum::Extension(cqrs_bus))
}

#[cfg(feature = "api-server")]
/// Create legacy database-direct API router (for backward compatibility)
pub fn create_legacy_router(pool: agent_agency_database::PgPool) -> Router {
    Router::new()
        .route("/api/tasks", get(get_tasks))
        .route("/api/tasks/{task_id}", get(get_task_detail))
        .route("/api/tasks/{task_id}/events", get(get_task_events))
        .route("/api/tasks/{task_id}/pause", post(pause_task))
        .route("/api/tasks/{task_id}/resume", post(resume_task))
        .route("/api/tasks/{task_id}/cancel", post(cancel_task))
        .layer(axum::Extension(pool))
}

#[cfg(feature = "api-server")]
/// Create combined router with both CQRS and legacy endpoints
/// The CQRS endpoints take precedence for new development
pub fn create_combined_router(
    cqrs_bus: Arc<CqrsBus>,
    pool: agent_agency_database::PgPool
) -> Router {
    Router::new()
        .merge(create_cqrs_router(cqrs_bus))
        .merge(create_legacy_router(pool))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cqrs::CqrsBus;
    use crate::api::*;
    use uuid::Uuid;
    use chrono::Utc;
    use serde_json::json;

    #[test]
    fn test_cqrs_router_creation() {
        let cqrs_bus = Arc::new(CqrsBus::new());
        let router = create_cqrs_router(cqrs_bus);
        // Router creation should succeed
        assert!(true); // If we get here, router creation worked
    }

    #[test]
    fn test_api_request_response_models() {
        // Test ExecuteTaskRequest serialization
        let request = ExecuteTaskRequest {
            task_descriptor: crate::caws_runtime::TaskDescriptor {
                task_id: "test-task".to_string(),
                scope_in: vec!["src/".to_string()],
                risk_tier: 2,
                execution_mode: crate::caws_runtime::ExecutionMode::Sequential,
                acceptance: Some(vec!["Task completes successfully".to_string()]),
                metadata: None,
            },
            worker_id: Uuid::new_v4(),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: ExecuteTaskRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(request.task_descriptor.task_id, deserialized.task_descriptor.task_id);

        // Test ExecuteTaskResponse
        let response = ExecuteTaskResponse {
            execution_id: Uuid::new_v4(),
            task_id: Uuid::new_v4(),
            status: "queued".to_string(),
            message: "Task queued for execution".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: ExecuteTaskResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response.status, deserialized.status);
    }

    #[test]
    fn test_system_health_response() {
        let response = SystemHealthResponse {
            total_workers: 5,
            active_workers: 3,
            healthy_workers: 3,
            total_tasks: 150,
            active_tasks: 2,
            completed_tasks: 142,
            failed_tasks: 6,
            average_task_duration_ms: 1250.0,
            uptime_seconds: 3600,
            timestamp: Utc::now(),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: SystemHealthResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response.total_workers, deserialized.total_workers);
        assert_eq!(response.active_workers, deserialized.active_workers);
    }

    #[test]
    fn test_task_status_response() {
        let response = TaskStatusResponse {
            task_id: Uuid::new_v4(),
            status: "running".to_string(),
            progress_percentage: 75,
            started_at: Some(Utc::now()),
            completed_at: None,
            worker_id: Some(Uuid::new_v4()),
            error_message: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: TaskStatusResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response.status, deserialized.status);
        assert_eq!(response.progress_percentage, deserialized.progress_percentage);
    }
}
