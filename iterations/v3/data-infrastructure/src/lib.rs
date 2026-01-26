//! Data Infrastructure - Unified data layer & API services
//!
//! Consolidates database, interfaces, and api-server functionality into
//! a comprehensive data layer with persistence, API services, and data contracts.

use anyhow::Result;
use std::sync::Arc;

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
pub mod database_init;
pub mod database_metrics;
pub mod database_operations;
pub mod migrations;
pub mod models;
pub mod monitoring;
pub mod optimization;
pub mod pooling;
pub mod queries;
pub mod queue;
pub mod scripts;
pub mod vector_store;
pub mod wal_replay;
pub mod wal_storage;

// API and interface modules (from consolidated interfaces and api-server crates)
pub mod api;
pub mod api_alerts;
pub mod api_circuit_breaker;
pub mod artifact_store;
pub mod chat_service;
pub mod cli_implementation;
pub mod cli_interface;
pub mod client; // client/mod.rs module
pub mod health;
pub mod keystore_api;
pub mod knowledge_queries;
pub mod mcp;
pub mod orchestrator_service;
pub mod rate_limiter;
pub mod rto_rpo_monitor;
pub mod sandbox_api;
pub mod service_failover;
pub mod simple_client; // simple_client.rs is automatically included as a module
pub mod system_observability;
pub mod telemetry_service;

// Data infrastructure modules (from consolidated caching, embedding-service, file_ops crates)
pub mod caching;
pub mod embedding;
pub mod file_operations;
pub mod file_operations_service;
pub mod websocket;

// Export core database types
pub use database_config::DatabaseConfig;
pub use simple_client::DatabaseClient;
pub use simple_client::ProvenanceClientAdapter;

// Export orchestrator service
pub use orchestrator_service::{
    ChainOfThoughtEntry, CoordinationEventData, CouncilDecision, CouncilSessionData,
    DecisionPointData, ExecutionResultWithObservability, JudgeContributionData,
    OrchestratorService, TaskExecutor, TaskExecutionState, TaskObservabilityData, TaskStatus,
    TaskSummary, WorkerAction, WorkerActionData,
};

// Export database operations factory
pub use database_operations::{
    create_database_audit_operations, create_database_operations, DatabaseOperations,
    CreateLearningRecord, CreateCurriculumProfile, CreateMilestoneCompletion, MilestoneCompletionResult,
};

// Re-export sqlx Row type for convenience
pub use sqlx::Row;

/// Worker pool health check trait
#[async_trait::async_trait]
pub trait WorkerPoolHealth: Send + Sync {
    async fn health_check(&self) -> Result<(), String>;
}

/// Simple worker pool implementation for health checking
pub struct SimpleWorkerPool {
    db_client: Option<Arc<DatabaseClient>>,
}

impl SimpleWorkerPool {
    pub fn new() -> Self {
        Self { db_client: None }
    }

    pub fn with_database(db_client: Arc<DatabaseClient>) -> Self {
        Self {
            db_client: Some(db_client),
        }
    }
}

#[async_trait::async_trait]
impl WorkerPoolHealth for SimpleWorkerPool {
    async fn health_check(&self) -> Result<(), String> {
        use tracing::{debug, warn};

        // If database client is available, check worker pool health from database
        if let Some(db) = &self.db_client {
            match db.get_workers().await {
                Ok(workers) => {
                    let total_workers = workers.len();
                    let active_workers = workers.iter().filter(|w| w.is_active).count();

                    debug!(
                        total_workers = total_workers,
                        active_workers = active_workers,
                        "Worker pool health check completed"
                    );

                    // Health check criteria:
                    // - At least one active worker should be available
                    // - If we have workers registered, at least 50% should be active
                    if total_workers == 0 {
                        warn!("No workers registered in database");
                        // Don't fail health check if no workers - system might be starting up
                        return Ok(());
                    }

                    let active_ratio = active_workers as f64 / total_workers as f64;
                    if active_ratio < 0.5 {
                        return Err(format!(
                            "Worker pool degraded: only {:.1}% of workers are active ({} active / {} total)",
                            active_ratio * 100.0,
                            active_workers,
                            total_workers
                        ));
                    }

                    if active_workers == 0 {
                        return Err("No active workers available in worker pool".to_string());
                    }

                    Ok(())
                }
                Err(e) => {
                    warn!("Failed to query workers from database: {}", e);
                    // Don't fail health check if database query fails - might be transient
                    // In production, this could be configured to fail or pass based on requirements
                    Ok(())
                }
            }
        } else {
            // No database client available - return healthy (backward compatibility)
            debug!("No database client available for worker pool health check");
            Ok(())
        }
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
        // Note: No pool reference available here - using metrics-only assessment
        // For real connectivity checks, use the API health endpoint which has pool access
        self.database_health
            .perform_health_check(None)
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
// TODO: Re-export handlers when handlers module is restored
// pub use api::handlers::{list_tasks, get_task_status as get_task, submit_task, get_metrics as get_api_metrics};
// pub use api::handlers::chat_handlers::create_chat_session as create_api_chat_session;
// pub use api::handlers::{get_chat_sessions, get_chat_messages, stream_agent_response, list_waivers, create_waiver};
// pub use api::handlers::{approve_waiver, get_task_provenance};
pub use api::metrics::{BusinessMetrics, DiskIOMetrics, SystemMetrics};
pub use client::orchestrator::DatabaseClient as ApiDatabaseClient; // Complex DatabaseClient

// Re-export health check from api module
pub use api::health::health_check;
