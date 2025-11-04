//! Centralized Database Connection Manager
//!
//! Provides a unified connection pooling strategy that aligns with the V2 TypeScript
//! ConnectionPoolManager implementation. This ensures consistent connection patterns
//! across all database clients and modules.

use schemars::JsonSchema;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgConnectOptions};
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;
use tracing::{error, info};

/// Centralized database connection configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DatabaseConnectionConfig {
    /// Database host address
    pub host: String,
    /// Database port number
    pub port: u16,
    /// Database name
    pub database: String,
    /// Database username
    pub username: String,
    /// Database password
    pub password: String,
    /// Whether to use SSL for database connections
    pub ssl: bool,
    /// Minimum connections to maintain
    pub min_connections: u32,
    /// Maximum connections allowed
    pub max_connections: u32,
    /// How long idle connections stay open (seconds)
    pub idle_timeout_seconds: u64,
    /// Max time to wait for connection (seconds)
    pub connection_timeout_seconds: u64,
    /// Max query execution time (seconds)
    pub statement_timeout_seconds: u64,
    /// Application name (appears in pg_stat_activity)
    pub application_name: String,
}

impl Default for DatabaseConnectionConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            database: "agent_agency".to_string(),
            username: "postgres".to_string(),
            password: "password".to_string(),
            ssl: false,
            min_connections: 5,
            max_connections: 20,
            idle_timeout_seconds: 600, // 10 minutes
            connection_timeout_seconds: 30,
            statement_timeout_seconds: 300, // 5 minutes
            application_name: "agent-agency-v3".to_string(),
        }
    }
}

/// Connection pool statistics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PoolStats {
    /// Total number of connections
    pub total_count: u32,
    /// Number of idle connections
    pub idle_count: u32,
    /// Number of active connections
    pub active_count: u32,
    /// Number of waiting clients
    pub waiting_count: u32,
    /// Maximum connections allowed
    pub max_connections: u32,
    /// Minimum connections maintained
    pub min_connections: u32,
}

/// Centralized connection pool manager
///
/// Singleton pattern ensures only ONE connection pool exists for the entire application,
/// preventing resource waste and connection exhaustion. All database clients should
/// use get_pool() instead of creating their own Pool instances.
#[derive(Debug)]
pub struct ConnectionPoolManager {
    pool: Arc<PgPool>,
    config: DatabaseConnectionConfig,
    stats: Arc<RwLock<PoolStats>>,
}

/// Global instance of the connection pool manager
static INSTANCE: OnceLock<Arc<ConnectionPoolManager>> = OnceLock::new();

impl ConnectionPoolManager {
    /// Get the singleton instance of the connection pool manager
    pub async fn get_instance() -> Result<Arc<ConnectionPoolManager>> {
        if let Some(instance) = INSTANCE.get() {
            return Ok(instance.clone());
        }
        
        let config = Self::load_config_from_env()?;
        let instance = Arc::new(Self::new(config).await?);
        
        INSTANCE.set(instance.clone()).map_err(|_| anyhow::anyhow!("Failed to set singleton instance"))?;
        
        Ok(instance)
    }

    /// Create a new connection pool manager
    pub async fn new(config: DatabaseConnectionConfig) -> Result<Self> {
        info!("Initializing centralized database connection pool");

        // Build connection options
        let mut connect_options = PgConnectOptions::new()
            .host(&config.host)
            .port(config.port)
            .database(&config.database)
            .username(&config.username)
            .password(&config.password)
            .application_name(&config.application_name);

        if config.ssl {
            connect_options = connect_options.ssl_mode(sqlx::postgres::PgSslMode::Require);
        } else {
            connect_options = connect_options.ssl_mode(sqlx::postgres::PgSslMode::Disable);
        }

        // Create connection pool
        let pool = PgPool::connect_with(connect_options)
            .await
            .context("Failed to create database connection pool")?;

        // Test the connection
        pool.acquire()
            .await
            .context("Failed to acquire initial connection for testing")?;

        info!("Database connection pool initialized successfully");

        let stats = Arc::new(RwLock::new(PoolStats {
            total_count: 0,
            idle_count: 0,
            active_count: 0,
            waiting_count: 0,
            max_connections: config.max_connections,
            min_connections: config.min_connections,
        }));

        Ok(Self {
            pool: Arc::new(pool),
            config,
            stats,
        })
    }

    /// Get the shared connection pool
    pub fn get_pool(&self) -> Arc<PgPool> {
        self.pool.clone()
    }

    /// Get current pool statistics
    pub async fn get_stats(&self) -> PoolStats {
        let pool_stats = self.pool.size();
        let idle_count = self.pool.num_idle();
        
        PoolStats {
            total_count: pool_stats as u32,
            idle_count: idle_count as u32,
            active_count: (pool_stats as u32).saturating_sub(idle_count as u32),
            waiting_count: 0, // SQLx doesn't expose waiting count directly
            max_connections: self.config.max_connections,
            min_connections: self.config.min_connections,
        }
    }

    /// Check if the pool is healthy
    pub async fn is_healthy(&self) -> bool {
        match self.pool.acquire().await {
            Ok(_) => true,
            Err(e) => {
                error!("Database connection pool health check failed: {}", e);
                false
            }
        }
    }

    /// Gracefully shutdown the connection pool
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down database connection pool");
        self.pool.close().await;
        info!("Database connection pool shutdown complete");
        Ok(())
    }

    /// Load configuration from environment variables
    fn load_config_from_env() -> Result<DatabaseConnectionConfig> {
        let config = DatabaseConnectionConfig {
            host: std::env::var("DATABASE_HOST")
                .unwrap_or_else(|_| "localhost".to_string()),
            port: std::env::var("DATABASE_PORT")
                .unwrap_or_else(|_| "5432".to_string())
                .parse()
                .context("Invalid DATABASE_PORT")?,
            database: std::env::var("DATABASE_NAME")
                .unwrap_or_else(|_| "agent_agency".to_string()),
            username: std::env::var("DATABASE_USER")
                .unwrap_or_else(|_| "postgres".to_string()),
            password: std::env::var("DATABASE_PASSWORD")
                .unwrap_or_else(|_| "password".to_string()),
            ssl: std::env::var("DATABASE_SSL")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            min_connections: std::env::var("DATABASE_MIN_CONNECTIONS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            max_connections: std::env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "20".to_string())
                .parse()
                .unwrap_or(20),
            idle_timeout_seconds: std::env::var("DATABASE_IDLE_TIMEOUT")
                .unwrap_or_else(|_| "600".to_string())
                .parse()
                .unwrap_or(600),
            connection_timeout_seconds: std::env::var("DATABASE_CONNECTION_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            statement_timeout_seconds: std::env::var("DATABASE_STATEMENT_TIMEOUT")
                .unwrap_or_else(|_| "300".to_string())
                .parse()
                .unwrap_or(300),
            application_name: std::env::var("DATABASE_APPLICATION_NAME")
                .unwrap_or_else(|_| "agent-agency-v3".to_string()),
        };

        Ok(config)
    }
}

/// Trait for database clients that use the centralized connection pool
#[async_trait]
pub trait PooledDatabaseClient {
    /// Initialize the client with the centralized pool
    async fn initialize(&self) -> Result<()>;
    
    /// Check if the client is available (pool is healthy)
    async fn is_available(&self) -> bool;
    
    /// Get the connection pool manager
    async fn get_pool_manager(&self) -> Arc<ConnectionPoolManager>;
}

/// Helper function to get the global connection pool manager
pub async fn get_connection_pool_manager() -> Result<Arc<ConnectionPoolManager>> {
    ConnectionPoolManager::get_instance().await
}

/// Helper function to get the global connection pool
pub async fn get_connection_pool() -> Result<Arc<PgPool>> {
    let manager = get_connection_pool_manager().await?;
    Ok(manager.get_pool())
}
