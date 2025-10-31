//! Database Interface
//!
//! Common database interface that can be implemented by different database
//! backends (PostgreSQL, SQLite, etc.) without creating circular dependencies.
//!
//! This allows data-infrastructure to provide database implementations while
//! other crates can depend on the interface without depending on concrete DB code.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::{Result, QueryParams, ApiResponse};

/// Generic database record identifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecordId {
    Uuid(Uuid),
    String(String),
    Integer(i64),
}

/// Generic database value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DbValue {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Json(serde_json::Value),
    Uuid(Uuid),
    Timestamp(DateTime<Utc>),
    Binary(Vec<u8>),
}

/// Database row representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbRow {
    pub columns: HashMap<String, DbValue>,
}

/// Database query result
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub rows: Vec<DbRow>,
    pub row_count: u64,
    pub columns: Vec<String>,
}

/// Transaction interface for atomic operations
#[async_trait]
pub trait Transaction: Send + Sync {
    /// Execute a query within the transaction
    async fn execute(&mut self, query: &str, params: &[DbValue]) -> Result<QueryResult>;

    /// Commit the transaction
    async fn commit(self) -> Result<()>;

    /// Rollback the transaction
    async fn rollback(self) -> Result<()>;
}

/// Database connection interface
#[async_trait]
pub trait DatabaseConnection: Send + Sync {
    /// Execute a query and return results
    async fn execute(&self, query: &str, params: &[DbValue]) -> Result<QueryResult>;

    /// Execute a query that doesn't return results
    async fn execute_no_result(&self, query: &str, params: &[DbValue]) -> Result<u64>;

    /// Start a new transaction
    async fn begin_transaction(&self) -> Result<Box<dyn Transaction>>;

    /// Check if connection is healthy
    async fn health_check(&self) -> Result<()>;

    /// Get connection information
    fn connection_info(&self) -> ConnectionInfo;
}

/// Database pool interface for managing connections
#[async_trait]
pub trait DatabasePool: Send + Sync {
    /// Get a connection from the pool
    async fn get_connection(&self) -> Result<Box<dyn DatabaseConnection>>;

    /// Get pool statistics
    fn stats(&self) -> PoolStats;
}

/// Database migration interface
#[async_trait]
pub trait DatabaseMigration: Send + Sync {
    /// Apply pending migrations
    async fn migrate_up(&self) -> Result<Vec<String>>;

    /// Rollback migrations
    async fn migrate_down(&self, steps: u32) -> Result<Vec<String>>;

    /// Get current migration version
    async fn current_version(&self) -> Result<Option<String>>;

    /// Get list of applied migrations
    async fn applied_migrations(&self) -> Result<Vec<String>>;
}

/// Connection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub database_type: String,
    pub host: String,
    pub port: u16,
    pub database_name: String,
    pub connection_count: u32,
    pub max_connections: u32,
}

/// Pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    pub total_connections: u32,
    pub idle_connections: u32,
    pub active_connections: u32,
    pub waiting_connections: u32,
}

/// Generic repository pattern interface
#[async_trait]
pub trait Repository<T>: Send + Sync {
    /// Find entity by ID
    async fn find_by_id(&self, id: &RecordId) -> Result<Option<T>>;

    /// Find all entities with optional filtering
    async fn find_all(&self, params: Option<QueryParams>) -> Result<ApiResponse<Vec<T>>>;

    /// Create new entity
    async fn create(&self, entity: T) -> Result<T>;

    /// Update existing entity
    async fn update(&self, id: &RecordId, entity: T) -> Result<T>;

    /// Delete entity by ID
    async fn delete(&self, id: &RecordId) -> Result<bool>;

    /// Count entities with optional filtering
    async fn count(&self, filters: Option<HashMap<String, DbValue>>) -> Result<u64>;
}

/// Database health check interface
#[async_trait]
pub trait DatabaseHealthCheck: Send + Sync {
    /// Perform comprehensive health check
    async fn full_health_check(&self) -> Result<DatabaseHealth>;

    /// Quick connectivity check
    async fn ping(&self) -> Result<()>;

    /// Get database metrics
    async fn metrics(&self) -> Result<DatabaseMetrics>;
}

/// Database health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseHealth {
    pub status: super::HealthStatus,
    pub message: String,
    pub details: HashMap<String, serde_json::Value>,
    pub last_check: DateTime<Utc>,
}

/// Database performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseMetrics {
    pub connection_pool_size: u32,
    pub active_connections: u32,
    pub idle_connections: u32,
    pub total_queries: u64,
    pub slow_queries: u64,
    pub avg_query_time_ms: f64,
    pub error_count: u64,
    pub last_error: Option<String>,
}

/// Audit trail entry creation input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAuditEntry {
    pub entity_type: String,
    pub entity_id: Uuid,
    pub action: String,
    pub details: serde_json::Value,
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub timestamp: Option<DateTime<Utc>>,
}

/// Database audit operations interface for audit trail persistence
/// 
/// This trait provides audit-specific database operations without requiring
/// the full DatabaseOperations trait, allowing crates to use audit persistence
/// without creating circular dependencies.
#[async_trait]
pub trait DatabaseAuditOperations: Send + Sync {
    /// Create an audit trail entry
    async fn create_audit_entry(&self, entry: CreateAuditEntry) -> Result<()>;
}
