//! V4 API - HTTP Server for Agent Agency
//!
//! This crate provides a thin HTTP API layer for:
//! - Task submission and evaluation
//! - LLM probing (measuring reaction timing, reasoning quality, inference speed)
//! - Health checks and metrics
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use v4_api::{Server, ServerConfig};
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = ServerConfig::local_dev();
//!     let server = Server::new(config);
//!     server.run().await.unwrap();
//! }
//! ```
//!
//! ## Endpoints
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | GET | `/health` | Health check |
//! | GET | `/metrics` | Timing metrics for LLM probing |
//! | GET | `/api/v1` | API info |
//! | POST | `/api/v1/tasks` | Submit task for evaluation |
//! | GET | `/api/v1/tasks/:id` | Get task status |
//! | POST | `/api/v1/probe` | Probe LLM inference |
//!
//! ## Architecture
//!
//! ```text
//! HTTP Request
//!     │
//!     ▼
//! ┌─────────────┐
//! │  handlers   │  Axum route handlers
//! └──────┬──────┘
//!        │
//!        ▼
//! ┌─────────────┐
//! │   service   │  Business logic, timing metrics
//! └──────┬──────┘
//!        │
//!        ▼
//! ┌─────────────┐
//! │  v4-arbiter │  Task evaluation pipeline
//! └─────────────┘
//! ```

pub mod handlers;
pub mod server;
pub mod service;
pub mod types;

// Re-export main types
pub use server::{build_router, Server, ServerConfig};
pub use service::{ApiService, ServiceConfig};
pub use types::{
    ErrorResponse, HealthResponse, MetricsResponse, ProbeRequest, ProbeResponse,
    SubmitTaskRequest, SubmitTaskResponse, TaskStatusResponse, TimingMetrics,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_exports() {
        // Verify main types are exported
        let _config = ServerConfig::default();
        let _service_config = ServiceConfig::default();
        let _timing = TimingMetrics::new();
    }

    #[tokio::test]
    async fn test_full_integration() {
        let service = Arc::new(ApiService::with_defaults());

        // Submit a task
        let request = SubmitTaskRequest {
            title: "Integration test".to_string(),
            description: "Test the full flow".to_string(),
            priority: v4_types::task::TaskPriority::Normal,
            environment: v4_types::task::Environment::Development,
            constraints: None,
            request_id: Some("int-test-1".to_string()),
        };

        let response = service.submit_task(request).await.unwrap();
        assert!(!response.task_id.is_empty());

        // Get the task
        let status = service.get_task(&response.task_id).await.unwrap();
        assert_eq!(status.task_id, response.task_id);

        // Check health
        let health = service.health().await;
        assert_eq!(health.status, "healthy");

        // Check metrics
        let metrics = service.metrics().await;
        assert_eq!(metrics.total_requests, 1);
    }
}
