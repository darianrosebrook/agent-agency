//! Data Infrastructure - Unified data layer & API services
//!
//! Consolidates database, interfaces, and api-server functionality into
//! a comprehensive data layer with persistence, API services, and data contracts.

// Database modules (from consolidated database crate)
pub mod audit;
pub mod backup;
pub mod backup_recovery;
pub mod backup_validator;
pub mod connection_manager;
pub mod data_consistency;
pub mod database_audit;
pub mod database_circuit_breaker;
pub mod database_config;
pub mod database_metrics;
pub mod database_operations;
pub mod migrations;
pub mod models;
pub mod optimization;
pub mod pooling;
pub mod queries;
pub mod vector_store;

// API and interface modules (from consolidated interfaces and api-server crates)
pub mod api;
pub mod api_alerts;
pub mod api_circuit_breaker;
pub mod artifact_store;
pub mod cli_implementation;
pub mod cli_interface;
pub mod client; // client/mod.rs module
pub mod simple_client; // simple_client.rs is automatically included as a module
pub mod handlers;
pub mod health;
pub mod keystore_api;
pub mod knowledge_queries;
pub mod mcp;
pub mod rate_limiter;
pub mod rto_rpo_monitor;
pub mod sandbox_api;
pub mod service_failover;
pub mod system_observability;

// Data infrastructure modules (from consolidated caching, embedding-service, file_ops crates)
pub mod caching;
pub mod embedding;
pub mod file_operations;

// Export core database types
pub use database_config::DatabaseConfig;
pub use simple_client::DatabaseClient;

// Re-export sqlx Row type for convenience
pub use sqlx::Row;

/// Worker pool health check trait
#[async_trait::async_trait]
pub trait WorkerPoolHealth: Send + Sync {
    async fn health_check(&self) -> Result<(), String>;
}

/// Simple worker pool implementation for health checking
pub struct SimpleWorkerPool;

impl SimpleWorkerPool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl WorkerPoolHealth for SimpleWorkerPool {
    async fn health_check(&self) -> Result<(), String> {
        // For now, return healthy - will integrate with real worker pool later
        Ok(())
    }
}

// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub task_store: std::sync::Arc<dyn TaskStoreTrait + Send + Sync>,
    pub db_client: DatabaseClient,
    pub audit_logger: std::sync::Arc<audit::AuditLogger>,
    pub keystore: std::sync::Arc<dyn system_quality_security::Keystore>,
    pub sandbox: std::sync::Arc<dyn system_quality_security::Sandbox>,
    pub health_monitor: std::sync::Arc<dyn std::fmt::Debug + Send + Sync>, // Placeholder for health monitor
    pub alert_manager: std::sync::Arc<api_alerts::AlertManager>,
    pub rate_limiter: std::sync::Arc<rate_limiter::RateLimiter>,
    pub backend_host: String,
    pub http_client: reqwest::Client,
    pub worker_pool: std::sync::Arc<dyn WorkerPoolHealth>,
}

// Re-export API and interface types (from consolidated interfaces and api-server crates)
pub use handlers::{PersistedTask, TaskStoreTrait};
pub use handlers::{list_tasks, get_task, submit_task, get_api_metrics};
pub use handlers::{create_chat_session, get_websocket_config, list_waivers, create_waiver};
pub use handlers::{approve_waiver, get_task_provenance};
pub use client::orchestrator::DatabaseClient as ApiDatabaseClient; // Complex DatabaseClient

// Re-export health check from api module
pub use api::health::health_check;
