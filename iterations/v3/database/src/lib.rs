//! Agent Agency V3 - Database Layer
//!
//! Provides database connectivity, connection pooling, and data access patterns
//! for the council-based arbiter system.

pub mod artifact_store;
pub mod backup;
pub mod client;
pub mod health;
pub mod knowledge_queries;
pub mod migrations;
pub mod models;
pub mod queries;
pub mod vector_store;
pub mod optimization;

pub use artifact_store::{DatabaseArtifactStorage, VersionMetadata, VersionDiff};
pub use backup::{BackupManager, BackupResult};
pub use client::{DatabaseClient, DatabaseHealthStatus};
pub use health::{DatabaseHealthChecker, HealthCheckResult};
pub use optimization::{
    DatabaseOptimizationManager, DatabaseOptimizationConfig, ReadWriteSplitClient,
    DatabasePerformanceMonitor, DatabaseIndexManager, MonitoredQueryExecutor,
    QueryMetrics, IndexRecommendation, IndexPriority, DatabaseOptimizationReport,
    TableStats
};
pub use migrations::{MigrationManager, MigrationResult};
pub use models::*;
pub use vector_store::{DatabaseVectorStore, VectorStoreStats};

/// Database configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub pool_min: u32,
    pub pool_max: u32,
    pub connection_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
    pub max_lifetime_seconds: u64,
    /// Enable read/write splitting
    pub enable_read_write_splitting: bool,
    /// Read replica configurations
    pub read_replicas: Vec<DatabaseReplicaConfig>,
}

/// Read replica configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DatabaseReplicaConfig {
    pub host: String,
    pub port: u16,
    pub weight: u32, // For load balancing (higher = more traffic)
    pub is_sync: bool, // Synchronous replication
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            database: "agent_agency_v3".to_string(),
            username: "postgres".to_string(),
            password: "".to_string(),
            pool_min: 2,
            pool_max: 20,
            connection_timeout_seconds: 30,
            idle_timeout_seconds: 600,
            max_lifetime_seconds: 3600,
            enable_read_write_splitting: false,
            read_replicas: Vec::new(),
        }
    }
}

impl DatabaseConfig {
    /// Create database URL for connection
    pub fn database_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }

    /// Create configuration from environment variables
    pub fn from_env() -> Result<Self, std::env::VarError> {
        Ok(Self {
            host: std::env::var("DATABASE_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: std::env::var("DATABASE_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5432),
            database: std::env::var("DATABASE_NAME").unwrap_or_else(|_| "agent_agency_v3".to_string()),
            username: std::env::var("DATABASE_USER").unwrap_or_else(|_| "postgres".to_string()),
            password: std::env::var("DATABASE_PASSWORD").unwrap_or_else(|_| "".to_string()),
            pool_min: std::env::var("DATABASE_POOL_MIN")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(2),
            pool_max: std::env::var("DATABASE_POOL_MAX")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(20),
            connection_timeout_seconds: std::env::var("DATABASE_CONNECTION_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
            idle_timeout_seconds: std::env::var("DATABASE_IDLE_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(600),
            max_lifetime_seconds: std::env::var("DATABASE_MAX_LIFETIME")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3600),
            enable_read_write_splitting: std::env::var("DATABASE_ENABLE_RW_SPLITTING")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(false),
            read_replicas: Vec::new(), // TODO: Implement parsing from env if needed
        })
    }

    /// Create database URL without database name (for creating database)
    pub fn server_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password, self.host, self.port
        )
    }

    /// Validate configuration values for production safety
    pub fn validate(&self) -> Result<(), String> {
        // Host validation
        if self.host.is_empty() {
            return Err("Database host cannot be empty".to_string());
        }
        if self.host.len() > 253 {
            return Err("Database host name too long".to_string());
        }

        // Port validation
        if self.port == 0 {
            return Err("Database port cannot be zero".to_string());
        }

        // Database name validation
        if self.database.is_empty() {
            return Err("Database name cannot be empty".to_string());
        }
        if self.database.len() > 63 {
            return Err("Database name too long (max 63 characters)".to_string());
        }

        // Username validation
        if self.username.is_empty() {
            return Err("Database username cannot be empty".to_string());
        }
        if self.username.len() > 63 {
            return Err("Database username too long".to_string());
        }

        // Password validation - allow empty in development but warn
        if self.password.is_empty() && std::env::var("NODE_ENV").unwrap_or_else(|_| "development".to_string()) == "production" {
            return Err("Database password cannot be empty in production".to_string());
        }

        // Pool size validation
        if self.pool_min > self.pool_max {
            return Err("Pool minimum size cannot be greater than maximum size".to_string());
        }
        if self.pool_max > 1000 {
            return Err("Pool maximum size too large (max recommended: 1000)".to_string());
        }
        if self.pool_min == 0 {
            return Err("Pool minimum size must be at least 1".to_string());
        }

        // Timeout validation
        if self.connection_timeout_seconds == 0 {
            return Err("Connection timeout must be greater than 0".to_string());
        }
        if self.connection_timeout_seconds > 300 {
            return Err("Connection timeout too long (max recommended: 300 seconds)".to_string());
        }

        if self.idle_timeout_seconds == 0 {
            return Err("Idle timeout must be greater than 0".to_string());
        }
        if self.idle_timeout_seconds > 3600 {
            return Err("Idle timeout too long (max recommended: 3600 seconds)".to_string());
        }

        if self.max_lifetime_seconds == 0 {
            return Err("Max lifetime must be greater than 0".to_string());
        }
        if self.max_lifetime_seconds > 86400 {
            return Err("Max lifetime too long (max recommended: 86400 seconds)".to_string());
        }

        Ok(())
    }

    /// Create a PostgreSQL connection pool from config
    /// 
    /// # Arguments
    /// * `config` - Database configuration
    /// 
    /// # Returns
    /// PostgreSQL connection pool or error
    /// 
    /// # Example
    /// ```ignore
    /// let config = DatabaseConfig::default();
    /// let pool = create_pool(&config).await?;
    /// ```
    pub async fn create_pool(&self) -> Result<sqlx::PgPool, sqlx::Error> {
        use sqlx::postgres::PgPoolOptions;
        use std::time::Duration;

        PgPoolOptions::new()
            .min_connections(self.pool_min)
            .max_connections(self.pool_max)
            .acquire_timeout(Duration::from_secs(self.connection_timeout_seconds))
            .idle_timeout(Some(Duration::from_secs(self.idle_timeout_seconds)))
            .max_lifetime(Some(Duration::from_secs(self.max_lifetime_seconds)))
            .connect(&self.database_url())
            .await
    }
}

/// Convenient shorthand to create a pool from environment or config
/// 
/// # Errors
/// Returns error if database connection fails
/// 
/// # Example
/// ```ignore
/// let pool = create_default_pool().await?;
/// ```
pub async fn create_default_pool() -> Result<sqlx::PgPool, sqlx::Error> {
    let config = DatabaseConfig::from_env()
        .unwrap_or_else(|_| DatabaseConfig::default());
    config.create_pool().await
}

// Re-export commonly used types
pub use sqlx::PgPool;
pub use sqlx::Row;
pub use sqlx::Transaction;
pub use sqlx::Postgres;
