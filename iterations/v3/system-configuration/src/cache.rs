//! Shared caching abstractions for pipeline performance optimization
//!
//! This module provides common caching patterns that can be used across
//! different pipeline implementations to improve performance.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Core cache trait for pipeline caching
#[async_trait]
pub trait PipelineCache<K, V>: Send + Sync {
    /// Get a value from cache
    async fn get(&self, key: &K) -> Option<V>;

    /// Put a value in cache
    async fn put(&self, key: K, value: V) -> Result<(), CacheError>;

    /// Remove a value from cache
    async fn remove(&self, key: &K) -> Result<(), CacheError>;

    /// Clear all cached values
    async fn clear(&self) -> Result<(), CacheError>;

    /// Get cache statistics
    fn stats(&self) -> CacheStats;
}

/// LRU cache implementation for pipelines
pub struct LruPipelineCache<K, V> {
    cache: Arc<RwLock<lru::LruCache<K, CacheEntry<V>>>>,
    config: CacheConfig,
    stats: Arc<RwLock<CacheStats>>,
}

impl<K, V> std::fmt::Debug for LruPipelineCache<K, V>
where
    K: std::fmt::Debug,
    V: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LruPipelineCache")
            .field("config", &self.config)
            .field("stats", &self.stats)
            .finish()
    }
}

impl<K, V> LruPipelineCache<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Create a new LRU cache
    pub fn new(config: CacheConfig) -> Result<Self, CacheError> {
        let cache = lru::LruCache::new(std::num::NonZeroUsize::new(config.max_size).ok_or(
            CacheError::InvalidConfig("max_size must be > 0".to_string()),
        )?);

        Ok(Self {
            cache: Arc::new(RwLock::new(cache)),
            config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        })
    }
}

#[async_trait]
impl<K, V> PipelineCache<K, V> for LruPipelineCache<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    async fn get(&self, key: &K) -> Option<V> {
        let mut cache = self.cache.write().await;
        let mut stats = self.stats.write().await;

        if let Some(entry) = cache.get(key) {
            // Check if entry is expired
            if entry.is_expired() {
                cache.pop(key);
                stats.misses += 1;
                stats.evictions += 1;
                return None;
            }

            stats.hits += 1;
            Some(entry.value.clone())
        } else {
            stats.misses += 1;
            None
        }
    }

    async fn put(&self, key: K, value: V) -> Result<(), CacheError> {
        let mut cache = self.cache.write().await;
        let mut stats = self.stats.write().await;

        let entry = CacheEntry::new(value, self.config.ttl_seconds);
        let evicted = cache.put(key, entry);

        if evicted.is_some() {
            stats.evictions += 1;
        }

        stats.total_entries = cache.len();
        Ok(())
    }

    async fn remove(&self, key: &K) -> Result<(), CacheError> {
        let mut cache = self.cache.write().await;
        let mut stats = self.stats.write().await;

        cache.pop(key);
        stats.total_entries = cache.len();
        Ok(())
    }

    async fn clear(&self) -> Result<(), CacheError> {
        let mut cache = self.cache.write().await;
        let mut stats = self.stats.write().await;

        cache.clear();
        *stats = CacheStats::default();
        Ok(())
    }

    fn stats(&self) -> CacheStats {
        futures::executor::block_on(async { self.stats.read().await.clone() })
    }
}

/// Cache entry with expiration
#[derive(Debug, Clone)]
struct CacheEntry<V> {
    value: V,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl<V> CacheEntry<V> {
    fn new(value: V, ttl_seconds: u64) -> Self {
        let expires_at = if ttl_seconds > 0 {
            Some(chrono::Utc::now() + chrono::Duration::seconds(ttl_seconds as i64))
        } else {
            None
        };

        Self { value, expires_at }
    }

    fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            chrono::Utc::now() > expires_at
        } else {
            false
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum number of entries
    pub max_size: usize,
    /// TTL in seconds (0 = no expiration)
    pub ttl_seconds: u64,
    /// Enable compression for large values
    pub enable_compression: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size: 1000,
            ttl_seconds: 300, // 5 minutes
            enable_compression: false,
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total entries in cache
    pub total_entries: usize,
    /// Cache hits
    pub hits: u64,
    /// Cache misses
    pub misses: u64,
    /// Hit rate (0.0-1.0)
    pub hit_rate: f64,
    /// Evictions due to size limits
    pub evictions: u64,
    /// Last updated
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl Default for CacheStats {
    fn default() -> Self {
        Self {
            total_entries: 0,
            hits: 0,
            misses: 0,
            hit_rate: 0.0,
            evictions: 0,
            last_updated: chrono::Utc::now(),
        }
    }
}

impl CacheStats {
    /// Update hit rate calculation
    pub fn update_hit_rate(&mut self) {
        let total = self.hits + self.misses;
        if total > 0 {
            self.hit_rate = self.hits as f64 / total as f64;
        }
        self.last_updated = chrono::Utc::now();
    }
}

/// Cache error types
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Invalid cache configuration: {0}")]
    InvalidConfig(String),

    #[error("Cache operation failed: {0}")]
    OperationFailed(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
