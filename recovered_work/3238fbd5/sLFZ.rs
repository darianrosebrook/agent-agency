//! Redis cache backend implementation
//!
//! Provides Redis-based caching with connection pooling, TTL support,
//! and circuit breaker pattern for reliability.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use redis::{AsyncCommands, Client, ConnectionAddr, ConnectionInfo, RedisResult};
use tokio::sync::Mutex;

/// Redis cache backend
pub struct RedisCache {
    client: Client,
    connection_pool: Arc<Mutex<Vec<redis::aio::Connection>>>,
    pool_size: usize,
    default_ttl: Duration,
    circuit_breaker: RedisCircuitBreaker,
}

impl RedisCache {
    /// Create a new Redis cache backend
    pub async fn new(
        host: &str,
        port: u16,
        password: Option<&str>,
        database: u8,
        pool_size: usize,
        default_ttl: Duration,
    ) -> Result<Self, RedisCacheError> {
        let addr = if let Some(pass) = password {
            ConnectionAddr::Tcp(host.to_string(), port)
        } else {
            ConnectionAddr::Tcp(host.to_string(), port)
        };

        let mut conn_info = ConnectionInfo {
            addr,
            redis: redis::RedisConnectionInfo {
                db: database as i64,
                username: None,
                password: password.map(|s| s.to_string()),
            },
        };

        let client = Client::open(conn_info)
            .map_err(|e| RedisCacheError::ConnectionError(e.to_string()))?;

        // Pre-connect some connections
        let mut pool = Vec::new();
        for _ in 0..pool_size.min(5) { // Limit initial connections
            match client.get_async_connection().await {
                Ok(conn) => pool.push(conn),
                Err(e) => {
                    tracing::warn!("Failed to establish Redis connection: {}", e);
                }
            }
        }

        Ok(Self {
            client,
            connection_pool: Arc::new(Mutex::new(pool)),
            pool_size,
            default_ttl,
            circuit_breaker: RedisCircuitBreaker::new(5, 30000), // 5 failures, 30s reset
        })
    }

    /// Create with default localhost configuration
    pub async fn localhost(pool_size: usize, default_ttl: Duration) -> Result<Self, RedisCacheError> {
        Self::new("127.0.0.1", 6379, None, 0, pool_size, default_ttl).await
    }

    /// Get a connection from the pool
    async fn get_connection(&self) -> Result<redis::aio::Connection, RedisCacheError> {
        if self.circuit_breaker.is_open() {
            return Err(RedisCacheError::CircuitBreakerOpen);
        }

        let mut pool = self.connection_pool.lock().await;

        if let Some(conn) = pool.pop() {
            return Ok(conn);
        }

        // Create new connection if pool is empty
        match self.client.get_async_connection().await {
            Ok(conn) => Ok(conn),
            Err(e) => {
                self.circuit_breaker.record_failure().await;
                Err(RedisCacheError::ConnectionError(e.to_string()))
            }
        }
    }

    /// Return a connection to the pool
    async fn return_connection(&self, conn: redis::aio::Connection) {
        let mut pool = self.connection_pool.lock().await;
        if pool.len() < self.pool_size {
            pool.push(conn);
        }
        // If pool is full, connection will be dropped
    }

    /// Check if Redis is healthy
    pub async fn health_check(&self) -> bool {
        if self.circuit_breaker.is_open() {
            return false;
        }

        match self.get_connection().await {
            Ok(mut conn) => {
                let result: RedisResult<String> = redis::cmd("PING").query_async(&mut conn).await;
                self.return_connection(conn).await;

                match result {
                    Ok(response) if response == "PONG" => {
                        self.circuit_breaker.record_success().await;
                        true
                    }
                    _ => {
                        self.circuit_breaker.record_failure().await;
                        false
                    }
                }
            }
            Err(_) => false,
        }
    }
}

#[async_trait]
impl super::CacheBackend for RedisCache {
    async fn get(&self, key: &str) -> Result<Option<String>, CacheError> {
        if self.circuit_breaker.is_open() {
            return Err(CacheError::CircuitBreakerOpen);
        }

        let mut conn = self.get_connection().await?;
        let result: RedisResult<Option<String>> = conn.get(key).await;

        self.return_connection(conn).await;

        match result {
            Ok(value) => {
                self.circuit_breaker.record_success().await;
                Ok(value)
            }
            Err(e) => {
                self.circuit_breaker.record_failure().await;
                Err(CacheError::RedisError(e.to_string()))
            }
        }
    }

    async fn set(&self, key: &str, value: &str, ttl: Option<Duration>) -> Result<(), CacheError> {
        if self.circuit_breaker.is_open() {
            return Err(CacheError::CircuitBreakerOpen);
        }

        let ttl = ttl.unwrap_or(self.default_ttl);
        let ttl_seconds = ttl.as_secs() as usize;

        let mut conn = self.get_connection().await?;
        let result: RedisResult<()> = conn.set_ex(key, value, ttl_seconds).await;

        self.return_connection(conn).await;

        match result {
            Ok(()) => {
                self.circuit_breaker.record_success().await;
                Ok(())
            }
            Err(e) => {
                self.circuit_breaker.record_failure().await;
                Err(CacheError::RedisError(e.to_string()))
            }
        }
    }

    async fn delete(&self, key: &str) -> Result<bool, CacheError> {
        if self.circuit_breaker.is_open() {
            return Err(CacheError::CircuitBreakerOpen);
        }

        let mut conn = self.get_connection().await?;
        let result: RedisResult<i32> = conn.del(key).await;

        self.return_connection(conn).await;

        match result {
            Ok(count) => {
                self.circuit_breaker.record_success().await;
                Ok(count > 0)
            }
            Err(e) => {
                self.circuit_breaker.record_failure().await;
                Err(CacheError::RedisError(e.to_string()))
            }
        }
    }

    async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        if self.circuit_breaker.is_open() {
            return Err(CacheError::CircuitBreakerOpen);
        }

        let mut conn = self.get_connection().await?;
        let result: RedisResult<i32> = conn.exists(key).await;

        self.return_connection(conn).await;

        match result {
            Ok(count) => {
                self.circuit_breaker.record_success().await;
                Ok(count > 0)
            }
            Err(e) => {
                self.circuit_breaker.record_failure().await;
                Err(CacheError::RedisError(e.to_string()))
            }
        }
    }

    async fn expire(&self, key: &str, ttl: Duration) -> Result<bool, CacheError> {
        if self.circuit_breaker.is_open() {
            return Err(CacheError::CircuitBreakerOpen);
        }

        let ttl_seconds = ttl.as_secs() as usize;

        let mut conn = self.get_connection().await?;
        let result: RedisResult<i32> = redis::cmd("EXPIRE").arg(key).arg(ttl_seconds).query_async(&mut conn).await;

        self.return_connection(conn).await;

        match result {
            Ok(count) => {
                self.circuit_breaker.record_success().await;
                Ok(count > 0)
            }
            Err(e) => {
                self.circuit_breaker.record_failure().await;
                Err(CacheError::RedisError(e.to_string()))
            }
        }
    }
}

/// Circuit breaker for Redis reliability
pub struct RedisCircuitBreaker {
    failure_count: std::sync::atomic::AtomicUsize,
    last_failure: std::sync::atomic::AtomicU64,
    failure_threshold: usize,
    reset_timeout_ms: u64,
}

impl RedisCircuitBreaker {
    pub fn new(failure_threshold: usize, reset_timeout_ms: u64) -> Self {
        Self {
            failure_count: std::sync::atomic::AtomicUsize::new(0),
            last_failure: std::sync::atomic::AtomicU64::new(0),
            failure_threshold,
            reset_timeout_ms,
        }
    }

    pub async fn record_success(&self) {
        self.failure_count.store(0, std::sync::atomic::Ordering::Relaxed);
    }

    pub async fn record_failure(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        self.failure_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.last_failure.store(now, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn is_open(&self) -> bool {
        let failures = self.failure_count.load(std::sync::atomic::Ordering::Relaxed);
        if failures >= self.failure_threshold {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;

            let last_failure = self.last_failure.load(std::sync::atomic::Ordering::Relaxed);

            // Check if reset timeout has passed
            if now - last_failure > self.reset_timeout_ms {
                // Reset the circuit breaker
                self.failure_count.store(0, std::sync::atomic::Ordering::Relaxed);
                false
            } else {
                true
            }
        } else {
            false
        }
    }
}

/// Cache backend trait (to be defined in parent module)
#[async_trait]
pub trait CacheBackend: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<String>, CacheError>;
    async fn set(&self, key: &str, value: &str, ttl: Option<Duration>) -> Result<(), CacheError>;
    async fn delete(&self, key: &str) -> Result<bool, CacheError>;
    async fn exists(&self, key: &str) -> Result<bool, CacheError>;
    async fn expire(&self, key: &str, ttl: Duration) -> Result<bool, CacheError>;
}

/// Cache error types
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Redis connection error: {0}")]
    ConnectionError(String),

    #[error("Redis operation error: {0}")]
    RedisError(String),

    #[error("Circuit breaker is open")]
    CircuitBreakerOpen,

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub type RedisCacheError = CacheError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redis_circuit_breaker() {
        let breaker = RedisCircuitBreaker::new(3, 1000);

        // Initially closed
        assert!(!breaker.is_open());

        // Record failures
        breaker.record_failure();
        assert!(!breaker.is_open());

        breaker.record_failure();
        assert!(!breaker.is_open());

        breaker.record_failure();
        assert!(breaker.is_open());
    }

    #[tokio::test]
    async fn test_redis_creation_fails_invalid_host() {
        // This should fail with invalid host
        let result = RedisCache::new("invalid.host", 6379, None, 0, 5, Duration::from_secs(300)).await;
        assert!(result.is_err());
    }
}
