//! Agent Agency V3 API Server
//!
//! Standalone HTTP API server providing REST endpoints for task management,
//! health checks, and metrics streaming.

// Re-export modules
pub mod alerts;
pub mod circuit_breaker;
pub mod handlers;
pub mod rate_limiter;
pub mod rto_rpo_monitor;
pub mod service_failover;

// Re-export types and functions from handlers
pub use handlers::{AppState, PersistedTask, TaskStoreTrait};
pub use handlers::{health_check, list_tasks, get_task, submit_task, get_api_metrics};
pub use handlers::{create_chat_session, get_websocket_config, list_waivers, create_waiver};
pub use handlers::{approve_waiver, get_task_provenance};
