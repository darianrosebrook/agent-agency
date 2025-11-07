//! Data Infrastructure - Unified data layer & API services
//!
//! Consolidates database, interfaces, and api-server functionality into
//! a comprehensive data layer with persistence, API services, and data contracts.

use std::sync::Arc;
use anyhow::Result;

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
pub mod database_init;
pub mod migrations;
pub mod models;
pub mod optimization;
pub mod pooling;
pub mod queries;
pub mod queue;
pub mod vector_store;
pub mod wal_storage;
pub mod wal_replay;

// API and interface modules (from consolidated interfaces and api-server crates)
pub mod api;
pub mod api_alerts;
pub mod api_circuit_breaker;
pub mod artifact_store;
pub mod cli_implementation;
pub mod cli_interface;
pub mod chat_service;
pub mod client; // client/mod.rs module
pub mod simple_client; // simple_client.rs is automatically included as a module
pub mod health;
pub mod keystore_api;
pub mod knowledge_queries;
pub mod mcp;
pub mod rate_limiter;
pub mod rto_rpo_monitor;
pub mod sandbox_api;
pub mod service_failover;
pub mod system_observability;
pub mod orchestrator_service;

// Data infrastructure modules (from consolidated caching, embedding-service, file_ops crates)
pub mod caching;
pub mod embedding;
pub mod file_operations;
pub mod file_operations_service;

// Export core database types
pub use database_config::DatabaseConfig;
pub use simple_client::DatabaseClient;
pub use simple_client::ProvenanceClientAdapter;

// Export orchestrator service
pub use orchestrator_service::{OrchestratorService, TaskExecutor};

// Export database operations factory
pub use database_operations::{create_database_operations, create_database_audit_operations, DatabaseOperations};

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
        // TODO: Implement real worker pool health check
        // - [ ] Integrate worker registry service from agent-workers crate
        // - [ ] Verify worker health endpoints are responding
        // - [ ] Check worker capacity and current load
        // - [ ] Return error if critical workers are unavailable
        // - [ ] Add worker pool metrics and monitoring
        // - [ ] Add unit tests with mock worker pool
        // - [ ] Add integration tests with real worker registry
        // DEPENDENCY: Real worker pool implementation not yet available
        // When integrated, this should:
        // 1. Check worker registry for available workers
        // 2. Verify worker health endpoints are responding
        // 3. Check worker capacity and load
        // 4. Return error if critical workers are unavailable
        //
        // For now, SimpleWorkerPool is a placeholder that always returns healthy.
        // Real implementation requires:
        // - Worker registry service (agent-workers crate)
        // - Worker health check endpoints
        // - Worker pool metrics and monitoring
        Ok(())
    }
}

/// System health monitor combining database and worker pool health
pub struct SystemHealthMonitor {
    database_health: Arc<health::DatabaseHealthMonitor>,
    worker_pool: Arc<dyn WorkerPoolHealth>,
}

impl SystemHealthMonitor {
    pub fn new(
        database_health: Arc<health::DatabaseHealthMonitor>,
        worker_pool: Arc<dyn WorkerPoolHealth>,
    ) -> Self {
        Self {
            database_health,
            worker_pool,
        }
    }

    /// Perform comprehensive health check
    pub async fn health_check(&self) -> Result<(), String> {
        // Check database health
        self.database_health.perform_health_check()
            .await
            .map_err(|e| format!("Database health check failed: {}", e))?;

        // Check worker pool health
        self.worker_pool.health_check().await?;

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
    pub health_monitor: Arc<SystemHealthMonitor>,
    pub alert_manager: std::sync::Arc<api_alerts::AlertManager>,
    pub rate_limiter: std::sync::Arc<rate_limiter::RateLimiter>,
    pub backend_host: String,
    pub http_client: reqwest::Client,
    pub worker_pool: std::sync::Arc<dyn WorkerPoolHealth>,
}

// Re-export API and interface types (from consolidated interfaces and api-server crates)
pub use api::types::{PersistedTask, TaskStoreTrait};
pub use api::handlers::{list_tasks, get_task_status as get_task, submit_task, get_metrics as get_api_metrics};
pub use api::handlers::{create_chat_session, get_websocket_config, list_waivers, create_waiver};
pub use api::handlers::{approve_waiver, get_task_provenance};
pub use client::orchestrator::DatabaseClient as ApiDatabaseClient; // Complex DatabaseClient

// Re-export health check from api module
pub use api::health::health_check;
