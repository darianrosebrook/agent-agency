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
pub mod client;
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

// TODO: Add missing types when available
// pub use models::{DatabaseConfig, DatabaseClient, Row};
// pub use migrations::{Migration, MigrationRunner};
// pub use pooling::ConnectionPool;
// pub use vector_store::{VectorStore, VectorQuery, VectorResult};
// pub use backup_recovery::{BackupManager, RecoveryManager};
// pub use audit::{DatabaseAuditor, AuditEvent};

// Re-export API and interface types (from consolidated interfaces and api-server crates)
pub use handlers::{AppState, PersistedTask, TaskStoreTrait};
pub use handlers::{health_check, list_tasks, get_task, submit_task, get_api_metrics};
pub use handlers::{create_chat_session, get_websocket_config, list_waivers, create_waiver};
pub use handlers::{approve_waiver, get_task_provenance};
pub use client::orchestrator::DatabaseClient as ApiDatabaseClient;
