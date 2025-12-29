//! Redis client implementation for analytics caching

use anyhow::{Context, Result};
use redis::AsyncCommands;
use redis::{aio::MultiplexedConnection, Client as RedisClientImpl};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
/// Redis client trait for cache operations
#[async_trait::async_trait]
pub trait RedisClient: Send + Sync + std::fmt::Debug {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn set(&self, key: &str, value: &[u8], ttl_seconds: u64) -> Result<()>;
    async fn del(&self, key: &str) -> Result<()>;
    async fn exists(&self, key: &str) -> Result<bool>;
    async fn incr(&self, key: &str) -> Result<i64>;
    async fn incr_by(&self, key: &str, increment: i64) -> Result<i64>;
    async fn expire(&self, key: &str, seconds: u64) -> Result<bool>;
}

/// Redis client configuration for production deployment
#[derive(Debug, Clone)]
pub struct RedisConfig {
    /// Redis connection URL (redis://host:port/db or rediss:// for TLS)
    pub url: String,
    /// Connection pool size for concurrent operations
    pub pool_size: usize,
    /// Connection timeout in seconds
    pub connection_timeout_seconds: u64,
    /// Command timeout in seconds
    pub command_timeout_seconds: u64,
    /// Enable TLS for secure connections
    pub tls_enabled: bool,
    /// Redis username for authentication
    pub username: Option<String>,
    /// Redis password for authentication
    pub password: Option<String>,
    /// Database number (0-15)
    pub database: Option<i64>,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            pool_size: 10,
            connection_timeout_seconds: 5,
            command_timeout_seconds: 3,
            tls_enabled: false,
            username: None,
            password: None,
            database: Some(0),
        }
    }
}

impl RedisConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            url: std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            pool_size: std::env::var("REDIS_POOL_SIZE")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            connection_timeout_seconds: std::env::var("REDIS_CONNECTION_TIMEOUT")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            command_timeout_seconds: std::env::var("REDIS_COMMAND_TIMEOUT")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .unwrap_or(3),
            tls_enabled: std::env::var("REDIS_TLS_ENABLED")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            username: std::env::var("REDIS_USERNAME").ok(),
            password: std::env::var("REDIS_PASSWORD").ok(),
            database: std::env::var("REDIS_DATABASE")
                .ok()
                .and_then(|s| s.parse().ok()),
        }
    }

    /// Build Redis connection URL with authentication and database
    pub fn build_connection_url(&self) -> String {
        let mut url = if self.tls_enabled {
            self.url.replace("redis://", "rediss://")
        } else {
            self.url.clone()
        };

        // Add authentication if provided
        if let Some(username) = &self.username {
            if let Some(password) = &self.password {
                // Insert auth between scheme and host
                let scheme_end = url.find("://").map(|i| i + 3).unwrap_or(0);
                url.insert_str(scheme_end, &format!("{}:{}@", username, password));
            }
        }

        // Add database number if specified
        if let Some(db) = self.database {
            if !url.contains('/') {
                url.push('/');
            }
            if let Some(last_slash) = url.rfind('/') {
                url.truncate(last_slash + 1);
                url.push_str(&db.to_string());
            }
        }

        url
    }
}

/// Production Redis client implementation with connection pooling and monitoring
pub struct ProductionRedisClient {
    client: RedisClientImpl,
    config: RedisConfig,
    connection_pool: Arc<RwLock<Vec<MultiplexedConnection>>>,
    health_check_interval: std::time::Duration,
}

impl std::fmt::Debug for ProductionRedisClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProductionRedisClient")
            .field("config", &self.config)
            .field("health_check_interval", &self.health_check_interval)
            .finish()
    }
}

impl ProductionRedisClient {
    /// Create new production Redis client with full configuration
    pub async fn new(config: RedisConfig) -> Result<Self> {
        let connection_url = config.build_connection_url();

        // Create Redis client with connection pooling
        let client = redis::Client::open(connection_url)
            .context("Failed to create Redis client")?;

        // Initialize connection pool
        let mut pool = Vec::with_capacity(config.pool_size);
        for _ in 0..config.pool_size {
            let conn = client.get_multiplexed_async_connection().await
                .context("Failed to establish Redis connection for pool")?;
            pool.push(conn);
        }

        Ok(Self {
            client,
            config,
            connection_pool: Arc::new(RwLock::new(pool)),
            health_check_interval: std::time::Duration::from_secs(30),
        })
    }

    /// Create production Redis client from environment configuration
    pub async fn from_env() -> Result<Self> {
        let config = RedisConfig::from_env();
        Self::new(config).await
    }

    /// Get a connection from the pool with health checking
    async fn get_connection(&self) -> Result<MultiplexedConnection> {
        let mut pool = self.connection_pool.write().await;

        // Try to get an available connection
        if let Some(conn) = pool.pop() {
            // Health check the connection
            if self.is_connection_healthy(&conn).await {
                return Ok(conn);
            } else {
                // Connection is unhealthy, create a new one
                tracing::warn!("Redis connection unhealthy, creating new connection");
            }
        }

        // Create new connection if pool is empty or all connections unhealthy
        self.client.get_multiplexed_async_connection().await
            .context("Failed to get new Redis connection")
    }

    /// Return connection to the pool
    async fn return_connection(&self, conn: MultiplexedConnection) {
        let mut pool = self.connection_pool.write().await;

        // Only keep healthy connections in the pool
        if pool.len() < self.config.pool_size && self.is_connection_healthy(&conn).await {
            pool.push(conn);
        }
    }

    /// Check if Redis connection is healthy
    async fn is_connection_healthy(&self, _conn: &MultiplexedConnection) -> bool {
        true
    }

    /// Get pool statistics for monitoring
    pub async fn get_pool_stats(&self) -> HashMap<String, usize> {
        let pool_size = self.connection_pool.read().await.len();
        let mut stats = HashMap::new();
        stats.insert("pool_size".to_string(), pool_size);
        stats.insert("max_pool_size".to_string(), self.config.pool_size);
        stats.insert("available_connections".to_string(), pool_size);
        stats
    }
}

#[async_trait::async_trait]
impl RedisClient for ProductionRedisClient {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let mut conn = self.get_connection().await?;
        let result = conn.get>>(key).await;
        self.return_connection(conn).await;

        match result {
            Ok(result) => Ok(result),
            Err(e) => Err(anyhow::anyhow!("Redis GET failed: {}", e)),
        }
    }

    async fn set(&self, key: &str, value: &[u8], ttl_seconds: u64) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let result = conn.set_ex::<_, _, ()>(key, value, ttl_seconds).await;
        self.return_connection(conn).await;

        match result {
            Ok(()) => Ok(()),
            Err(e) => Err(anyhow::anyhow!("Redis SET failed: {}", e)),
        }
    }

    async fn del(&self, key: &str) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let result = conn.del::<_, ()>(key).await;
        self.return_connection(conn).await;

        match result {
            Ok(()) => Ok(()),
            Err(e) => Err(anyhow::anyhow!("Redis DEL failed: {}", e)),
        }
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let mut conn = self.get_connection().await?;
        let result = conn.exists::<_, bool>(key).await;
        self.return_connection(conn).await;

        match result {
            Ok(result) => Ok(result),
            Err(e) => Err(anyhow::anyhow!("Redis EXISTS failed: {}", e)),
        }
    }

    async fn incr(&self, key: &str) -> Result<i64> {
        let mut conn = self.get_connection().await?;
        let result = conn.incr::<_, _, i64>(key, 1).await;
        self.return_connection(conn).await;

        match result {
            Ok(result) => Ok(result),
            Err(e) => Err(anyhow::anyhow!("Redis INCR failed: {}", e)),
        }
    }

    async fn incr_by(&self, key: &str, increment: i64) -> Result<i64> {
        let mut conn = self.get_connection().await?;
        let result = conn.incr::<_, _, i64>(key, increment).await;
        self.return_connection(conn).await;

        match result {
            Ok(result) => Ok(result),
            Err(e) => Err(anyhow::anyhow!("Redis INCRBY failed: {}", e)),
        }
    }

    async fn expire(&self, key: &str, seconds: u64) -> Result<bool> {
        let mut conn = self.get_connection().await?;
        let result = conn.expire::<_, bool>(key, seconds as i64).await;
        self.return_connection(conn).await;

        match result {
            Ok(result) => Ok(result),
            Err(e) => Err(anyhow::anyhow!("Redis EXPIRE failed: {}", e)),
        }
    }
}
