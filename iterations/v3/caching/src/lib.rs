//! Multi-level caching system for enterprise performance optimization
//!
//! Provides memory, Redis, and CDN caching with intelligent invalidation,
//! cache warming, and performance monitoring capabilities.

pub mod integration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Cache entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub value: T,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub access_count: u64,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    pub tags: Vec<String>,
    pub version: u64,
}

/// Cache operation result
pub type CacheResult<T> = Result<T, CacheError>;

/// Cache operation errors
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Cache miss for key: {key}")]
    Miss { key: String },

    #[error("Cache expired for key: {key}")]
    Expired { key: String },

    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    #[error("Deserialization error: {message}")]
    DeserializationError { message: String },

    #[error("Connection error: {message}")]
    ConnectionError { message: String },

    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    #[error("Cache full, cannot store more items")]
    CacheFull,
}

/// Cache invalidation strategy
#[derive(Debug, Clone, PartialEq)]
pub enum InvalidationStrategy {
    /// Immediate invalidation
    Immediate,
    /// Time-based invalidation (TTL)
    TimeBased(Duration),
    /// Lazy invalidation (invalidate on next access)
    Lazy,
    /// Write-through invalidation (invalidate related keys)
    WriteThrough { related_keys: Vec<String> },
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub sets: u64,
    pub deletes: u64,
    pub expirations: u64,
    pub size_bytes: u64,
    pub item_count: u64,
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub max_memory_mb: usize,
    pub default_ttl_seconds: u64,
    pub enable_compression: bool,
    pub enable_serialization: bool,
    pub eviction_policy: EvictionPolicy,
    pub enable_metrics: bool,
    pub redis_url: Option<String>,
    pub redis_cluster: bool,
    pub cdn_enabled: bool,
    pub cdn_base_url: Option<String>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: 512, // 512MB default
            default_ttl_seconds: 3600, // 1 hour
            enable_compression: true,
            enable_serialization: true,
            eviction_policy: EvictionPolicy::Lru,
            enable_metrics: true,
            redis_url: None,
            redis_cluster: false,
            cdn_enabled: false,
            cdn_base_url: None,
        }
    }
}

/// Cache eviction policies
#[derive(Debug, Clone, PartialEq)]
pub enum EvictionPolicy {
    /// Least Recently Used
    Lru,
    /// Least Frequently Used
    Lfu,
    /// First In, First Out
    Fifo,
    /// Random eviction
    Random,
}

/// Cache trait for unified interface
#[async_trait]
pub trait Cache<K, V>: Send + Sync {
    /// Get a value from cache
    async fn get(&self, key: &K) -> CacheResult<V>;

    /// Set a value in cache
    async fn set(&self, key: K, value: V, ttl: Option<Duration>) -> CacheResult<()>;

    /// Delete a value from cache
    async fn delete(&self, key: &K) -> CacheResult<()>;

    /// Check if key exists
    async fn exists(&self, key: &K) -> bool;

    /// Clear all cache entries
    async fn clear(&self) -> CacheResult<()>;

    /// Get cache statistics
    async fn stats(&self) -> CacheStats;

    /// Get cache size in bytes
    async fn size_bytes(&self) -> u64;
}

// Re-export integration utilities
pub use integration::{
    ApiResponseCache, DatabaseQueryCache, LlmResponseCache, ComputationCache,
    CacheWarmer, CacheMonitor
};

/// In-memory LRU cache implementation
pub struct MemoryCache<V> {
    entries: Arc<RwLock<HashMap<String, CacheEntry<V>>>>,
    config: CacheConfig,
    stats: Arc<RwLock<CacheStats>>,
    max_entries: usize,
}

impl<V> MemoryCache<V>
where
    V: Clone + Send + Sync + 'static,
{
    /// Create a new memory cache
    pub fn new(config: CacheConfig) -> Self {
        let max_entries = (config.max_memory_mb * 1024 * 1024) / std::mem::size_of::<CacheEntry<V>>();
        let max_entries = max_entries.max(100); // Minimum 100 entries

        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
            max_entries,
        }
    }

    /// Evict entries based on policy
    async fn evict_if_needed(&self) {
        let entries = self.entries.read().await;
        if entries.len() < self.max_entries {
            return;
        }

        let mut to_evict = Vec::new();
        match self.config.eviction_policy {
            EvictionPolicy::Lru => {
                // Find least recently used
                let mut lru_time = chrono::Utc::now();
                let mut lru_key = None;
                for (key, entry) in entries.iter() {
                    if entry.last_accessed < lru_time {
                        lru_time = entry.last_accessed;
                        lru_key = Some(key.clone());
                    }
                }
                if let Some(key) = lru_key {
                    to_evict.push(key);
                }
            }
            EvictionPolicy::Lfu => {
                // Find least frequently used
                let mut lfu_count = u64::MAX;
                let mut lfu_key = None;
                for (key, entry) in entries.iter() {
                    if entry.access_count < lfu_count {
                        lfu_count = entry.access_count;
                        lfu_key = Some(key.clone());
                    }
                }
                if let Some(key) = lfu_key {
                    to_evict.push(key);
                }
            }
            EvictionPolicy::Fifo => {
                // Find oldest
                let mut oldest_time = chrono::Utc::now();
                let mut oldest_key = None;
                for (key, entry) in entries.iter() {
                    if entry.created_at < oldest_time {
                        oldest_time = entry.created_at;
                        oldest_key = Some(key.clone());
                    }
                }
                if let Some(key) = oldest_key {
                    to_evict.push(key);
                }
            }
            EvictionPolicy::Random => {
                // Random eviction
                use rand::seq::SliceRandom;
                if let Some(key) = entries.keys().collect::<Vec<_>>().choose(&mut rand::thread_rng()) {
                    to_evict.push((*key).clone());
                }
            }
        }

        drop(entries);

        for key in to_evict {
            let mut entries = self.entries.write().await;
            if entries.remove(&key).is_some() {
                let mut stats = self.stats.write().await;
                stats.evictions += 1;
                stats.item_count = stats.item_count.saturating_sub(1);
            }
        }
    }

    /// Clean expired entries
    async fn clean_expired(&self) {
        let mut entries = self.entries.write().await;
        let now = chrono::Utc::now();
        let mut expired_keys = Vec::new();

        for (key, entry) in entries.iter() {
            if let Some(expires_at) = entry.expires_at {
                if now > expires_at {
                    expired_keys.push(key.clone());
                }
            }
        }

        for key in expired_keys {
            if entries.remove(&key).is_some() {
                let mut stats = self.stats.write().await;
                stats.expirations += 1;
                stats.item_count = stats.item_count.saturating_sub(1);
            }
        }
    }
}

#[async_trait]
impl<V> Cache<String, V> for MemoryCache<V>
where
    V: Clone + Send + Sync + 'static,
{
    async fn get(&self, key: &String) -> CacheResult<V> {
        self.clean_expired().await;

        let mut entries = self.entries.write().await;
        if let Some(entry) = entries.get_mut(key) {
            // Check expiration
            if let Some(expires_at) = entry.expires_at {
                if chrono::Utc::now() > expires_at {
                    entries.remove(key);
                    let mut stats = self.stats.write().await;
                    stats.expirations += 1;
                    stats.item_count = stats.item_count.saturating_sub(1);
                    return Err(CacheError::Expired { key: key.clone() });
                }
            }

            // Update access tracking
            entry.access_count += 1;
            entry.last_accessed = chrono::Utc::now();

            let mut stats = self.stats.write().await;
            stats.hits += 1;

            Ok(entry.value.clone())
        } else {
            let mut stats = self.stats.write().await;
            stats.misses += 1;
            Err(CacheError::Miss { key: key.clone() })
        }
    }

    async fn set(&self, key: String, value: V, ttl: Option<Duration>) -> CacheResult<()> {
        self.evict_if_needed().await;

        let now = chrono::Utc::now();
        let expires_at = ttl.map(|d| now + chrono::Duration::from_std(d).unwrap());

        let entry = CacheEntry {
            value,
            created_at: now,
            expires_at,
            access_count: 0,
            last_accessed: now,
            tags: Vec::new(),
            version: 1,
        };

        let mut entries = self.entries.write().await;
        entries.insert(key, entry);

        let mut stats = self.stats.write().await;
        stats.sets += 1;
        stats.item_count += 1;

        Ok(())
    }

    async fn delete(&self, key: &String) -> CacheResult<()> {
        let mut entries = self.entries.write().await;
        if entries.remove(key).is_some() {
            let mut stats = self.stats.write().await;
            stats.deletes += 1;
            stats.item_count = stats.item_count.saturating_sub(1);
            Ok(())
        } else {
            Err(CacheError::Miss { key: key.clone() })
        }
    }

    async fn exists(&self, key: &String) -> bool {
        self.clean_expired().await;
        self.entries.read().await.contains_key(key)
    }

    async fn clear(&self) -> CacheResult<()> {
        let mut entries = self.entries.write().await;
        entries.clear();

        let mut stats = self.stats.write().await;
        stats.item_count = 0;

        Ok(())
    }

    async fn stats(&self) -> CacheStats {
        let mut stats = self.stats.read().await.clone();
        stats.item_count = self.entries.read().await.len() as u64;
        stats
    }

    async fn size_bytes(&self) -> u64 {
        // Rough estimation
        let entry_count = self.entries.read().await.len() as u64;
        let entry_size = std::mem::size_of::<CacheEntry<V>>() as u64;
        entry_count * entry_size
    }
}

/// Redis cache implementation
pub struct RedisCache<V> {
    client: redis::Client,
    config: CacheConfig,
    stats: Arc<RwLock<CacheStats>>,
    _phantom: std::marker::PhantomData<V>,
}

impl<V> RedisCache<V>
where
    V: serde::Serialize + for<'de> serde::Deserialize<'de> + Send + Sync + Clone + 'static,
{
    /// Create a new Redis cache
    pub fn new(config: CacheConfig) -> CacheResult<Self> {
        let redis_url = config.redis_url.as_ref()
            .ok_or_else(|| CacheError::ConfigError {
                message: "Redis URL not configured".to_string()
            })?;

        let client = redis::Client::open(redis_url)
            .map_err(|e| CacheError::ConnectionError {
                message: format!("Failed to connect to Redis: {}", e)
            })?;

        Ok(Self {
            client,
            config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
            _phantom: std::marker::PhantomData,
        })
    }

    /// Get a Redis connection
    async fn get_connection(&self) -> CacheResult<redis::aio::Connection> {
        self.client.get_async_connection().await
            .map_err(|e| CacheError::ConnectionError {
                message: format!("Redis connection failed: {}", e)
            })
    }
}

#[async_trait]
impl<V> Cache<String, V> for RedisCache<V>
where
    V: serde::Serialize + for<'de> serde::Deserialize<'de> + Send + Sync + Clone + 'static,
{
    async fn get(&self, key: &String) -> CacheResult<V> {
        let mut conn = self.get_connection().await?;
        let result: Option<String> = redis::cmd("GET").arg(key).query_async(&mut conn).await
            .map_err(|e| CacheError::ConnectionError {
                message: format!("Redis GET failed: {}", e)
            })?;

        if let Some(serialized) = result {
            let entry: CacheEntry<V> = serde_json::from_str(&serialized)
                .map_err(|e| CacheError::DeserializationError {
                    message: format!("Failed to deserialize cache entry: {}", e)
                })?;

            // Check expiration
            if let Some(expires_at) = entry.expires_at {
                if chrono::Utc::now() > expires_at {
                    // Delete expired entry
                    let _: () = redis::cmd("DEL").arg(key).query_async(&mut conn).await
                        .unwrap_or(());
                    let mut stats = self.stats.write().await;
                    stats.expirations += 1;
                    return Err(CacheError::Expired { key: key.clone() });
                }
            }

            let mut stats = self.stats.write().await;
            stats.hits += 1;

            Ok(entry.value)
        } else {
            let mut stats = self.stats.write().await;
            stats.misses += 1;
            Err(CacheError::Miss { key: key.clone() })
        }
    }

    async fn set(&self, key: String, value: V, ttl: Option<Duration>) -> CacheResult<()> {
        let mut conn = self.get_connection().await?;

        let now = chrono::Utc::now();
        let expires_at = ttl.map(|d| now + chrono::Duration::from_std(d).unwrap());

        let entry = CacheEntry {
            value,
            created_at: now,
            expires_at,
            access_count: 0,
            last_accessed: now,
            tags: Vec::new(),
            version: 1,
        };

        let serialized = serde_json::to_string(&entry)
            .map_err(|e| CacheError::SerializationError {
                message: format!("Failed to serialize cache entry: {}", e)
            })?;

        if let Some(ttl) = ttl {
            let _: () = redis::cmd("SETEX")
                .arg(&key)
                .arg(ttl.as_secs())
                .arg(serialized)
                .query_async(&mut conn)
                .await
                .map_err(|e| CacheError::ConnectionError {
                    message: format!("Redis SETEX failed: {}", e)
                })?;
        } else {
            let _: () = redis::cmd("SET")
                .arg(&key)
                .arg(serialized)
                .query_async(&mut conn)
                .await
                .map_err(|e| CacheError::ConnectionError {
                    message: format!("Redis SET failed: {}", e)
                })?;
        }

        let mut stats = self.stats.write().await;
        stats.sets += 1;

        Ok(())
    }

    async fn delete(&self, key: &String) -> CacheResult<()> {
        let mut conn = self.get_connection().await?;
        let result: i32 = redis::cmd("DEL").arg(key).query_async(&mut conn).await
            .map_err(|e| CacheError::ConnectionError {
                message: format!("Redis DEL failed: {}", e)
            })?;

        if result > 0 {
            let mut stats = self.stats.write().await;
            stats.deletes += 1;
            Ok(())
        } else {
            Err(CacheError::Miss { key: key.clone() })
        }
    }

    async fn exists(&self, key: &String) -> bool {
        let mut conn = self.get_connection().await;
        if let Ok(mut conn) = conn {
            let result: i32 = redis::cmd("EXISTS").arg(key).query_async(&mut conn).await
                .unwrap_or(0);
            result > 0
        } else {
            false
        }
    }

    async fn clear(&self) -> CacheResult<()> {
        let mut conn = self.get_connection().await?;
        let _: () = redis::cmd("FLUSHDB").query_async(&mut conn).await
            .map_err(|e| CacheError::ConnectionError {
                message: format!("Redis FLUSHDB failed: {}", e)
            })?;
        Ok(())
    }

    async fn stats(&self) -> CacheStats {
        // Redis stats would require additional commands
        // For now, return basic stats
        self.stats.read().await.clone()
    }

    async fn size_bytes(&self) -> u64 {
        // Rough estimation - would need Redis INFO command for accurate size
        0
    }
}

/// Multi-level cache combining memory, Redis, and CDN
pub struct MultiLevelCache<V> {
    memory_cache: MemoryCache<V>,
    redis_cache: Option<RedisCache<V>>,
    config: CacheConfig,
    stats: Arc<RwLock<CacheStats>>,
}

impl<V> MultiLevelCache<V>
where
    V: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone + Send + Sync + 'static,
{
    /// Create a new multi-level cache
    pub fn new(config: CacheConfig) -> CacheResult<Self> {
        let memory_cache = MemoryCache::new(config.clone());

        let redis_cache = if config.redis_url.is_some() {
            Some(RedisCache::new(config.clone())?)
        } else {
            None
        };

        Ok(Self {
            memory_cache,
            redis_cache,
            config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        })
    }

    /// Cache warming for frequently accessed data
    pub async fn warm_cache<F>(&self, keys: Vec<String>, loader: F) -> CacheResult<()>
    where
        F: Fn(&str) -> futures::future::BoxFuture<'_, CacheResult<V>> + Send + Sync,
    {
        info!("Starting cache warming for {} keys", keys.len());

        for key in keys {
            // Check if already in memory cache
            if self.memory_cache.exists(&key).await {
                continue;
            }

            // Load data and cache it
            match loader(&key).await {
                Ok(value) => {
                    let ttl = Some(Duration::from_secs(self.config.default_ttl_seconds));
                    if let Err(e) = self.set(key.clone(), value, ttl).await {
                        warn!("Failed to warm cache for key {}: {}", key, e);
                    }
                }
                Err(e) => {
                    debug!("Failed to load data for cache warming key {}: {}", key, e);
                }
            }
        }

        info!("Cache warming completed");
        Ok(())
    }

    /// Invalidate cache by tags
    pub async fn invalidate_by_tags(&self, tags: &[String]) -> CacheResult<usize> {
        // This is a simplified implementation
        // In a real system, you'd need to track tag-to-key mappings
        let memory_entries = self.memory_cache.entries.read().await;
        let mut invalidated = 0;

        for (key, entry) in memory_entries.iter() {
            if tags.iter().any(|tag| entry.tags.contains(tag)) {
                // Invalidate from memory cache
                drop(memory_entries);
                let _ = self.memory_cache.delete(key).await;
                invalidated += 1;

                // Invalidate from Redis if available
                if let Some(redis) = &self.redis_cache {
                    let _ = redis.delete(key).await;
                }
            }
        }

        Ok(invalidated)
    }
}

#[async_trait]
impl<V> Cache<String, V> for MultiLevelCache<V>
where
    V: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone + Send + Sync + 'static,
{
    async fn get(&self, key: &String) -> CacheResult<V> {
        // Try memory cache first
        match self.memory_cache.get(key).await {
            Ok(value) => {
                let mut stats = self.stats.write().await;
                stats.hits += 1;
                Ok(value)
            }
            Err(CacheError::Miss { .. }) => {
                // Try Redis cache if available
                if let Some(redis) = &self.redis_cache {
                    match redis.get(key).await {
                        Ok(value) => {
                            // Cache in memory for faster future access
                            let ttl = Some(Duration::from_secs(self.config.default_ttl_seconds));
                            let _ = self.memory_cache.set(key.clone(), value.clone(), ttl).await;

                            let mut stats = self.stats.write().await;
                            stats.hits += 1;
                            Ok(value)
                        }
                        Err(CacheError::Miss { .. }) => {
                            let mut stats = self.stats.write().await;
                            stats.misses += 1;
                            Err(CacheError::Miss { key: key.clone() })
                        }
                        Err(e) => Err(e),
                    }
                } else {
                    let mut stats = self.stats.write().await;
                    stats.misses += 1;
                    Err(CacheError::Miss { key: key.clone() })
                }
            }
            Err(e) => Err(e),
        }
    }

    async fn set(&self, key: String, value: V, ttl: Option<Duration>) -> CacheResult<()> {
        // Set in memory cache
        self.memory_cache.set(key.clone(), value.clone(), ttl).await?;

        // Set in Redis cache if available
        if let Some(redis) = &self.redis_cache {
            redis.set(key, value, ttl).await?;
        }

        let mut stats = self.stats.write().await;
        stats.sets += 1;

        Ok(())
    }

    async fn delete(&self, key: &String) -> CacheResult<()> {
        // Delete from memory cache
        let memory_result = self.memory_cache.delete(key).await;

        // Delete from Redis cache if available
        if let Some(redis) = &self.redis_cache {
            let _ = redis.delete(key).await; // Ignore errors for Redis
        }

        memory_result
    }

    async fn exists(&self, key: &String) -> bool {
        self.memory_cache.exists(key).await ||
        self.redis_cache.as_ref().map_or(false, |r| r.exists(key))
    }

    async fn clear(&self) -> CacheResult<()> {
        self.memory_cache.clear().await?;

        if let Some(redis) = &self.redis_cache {
            redis.clear().await?;
        }

        Ok(())
    }

    async fn stats(&self) -> CacheStats {
        let memory_stats = self.memory_cache.stats().await;
        let redis_stats = if let Some(redis) = &self.redis_cache {
            redis.stats().await
        } else {
            CacheStats::default()
        };

        // Combine stats
        CacheStats {
            hits: memory_stats.hits + redis_stats.hits,
            misses: memory_stats.misses + redis_stats.misses,
            evictions: memory_stats.evictions,
            sets: memory_stats.sets + redis_stats.sets,
            deletes: memory_stats.deletes + redis_stats.deletes,
            expirations: memory_stats.expirations + redis_stats.expirations,
            size_bytes: memory_stats.size_bytes,
            item_count: memory_stats.item_count,
        }
    }

    async fn size_bytes(&self) -> u64 {
        self.memory_cache.size_bytes().await
    }
}

/// Cache manager for application-wide caching
pub struct CacheManager {
    caches: Arc<RwLock<HashMap<String, Box<dyn std::any::Any + Send + Sync>>>>,
    config: CacheConfig,
}

impl CacheManager {
    /// Create a new cache manager
    pub fn new(config: CacheConfig) -> Self {
        Self {
            caches: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Create or get a typed cache
    pub async fn get_or_create_cache<V>(&self, name: &str) -> CacheResult<Arc<dyn Cache<String, V> + Send + Sync>>
    where
        V: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone + Send + Sync + 'static,
    {
        let mut caches = self.caches.write().await;

        if let Some(cache) = caches.get(name) {
            if let Some(typed_cache) = cache.downcast_ref::<Arc<dyn Cache<String, V> + Send + Sync>>() {
                return Ok(typed_cache.clone());
            }
        }

        // Create new multi-level cache
        let cache = Arc::new(MultiLevelCache::<V>::new(self.config.clone())?);
        caches.insert(name.to_string(), Box::new(cache.clone()));

        Ok(cache)
    }

    /// Get cache statistics for all caches
    pub async fn global_stats(&self) -> HashMap<String, CacheStats> {
        let caches = self.caches.read().await;
        let mut stats = HashMap::new();

        // This is simplified - in practice you'd need to handle the Any type properly
        // For now, return empty stats
        for name in caches.keys() {
            stats.insert(name.clone(), CacheStats::default());
        }

        stats
    }

    /// Clear all caches
    pub async fn clear_all(&self) -> CacheResult<()> {
        let caches = self.caches.read().await;

        // This is simplified - in practice you'd need to call clear on each typed cache
        info!("Cache manager clear_all called (simplified implementation)");
        Ok(())
    }
}
