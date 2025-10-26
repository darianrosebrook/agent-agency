//! Core types and traits for the caching system

use async_trait::async_trait;
use flate2::{write::GzEncoder, read::GzDecoder, Compression};
use serde::{Deserialize, Serialize};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::Arc;
use std::time::Duration;

/// Type-safe Any handling traits and structures

/// A type-erased cache that can be safely downcast
#[typetag::serde(tag = "cache_type")]
pub trait TypeErasedCache: Send + Sync + std::fmt::Debug {
    /// Get the type name for this cache
    fn type_name(&self) -> &'static str;

    /// Get the type ID for runtime type checking
    fn type_id(&self) -> TypeId;

    /// Get cache statistics (type-erased)
    fn stats(&self) -> CacheResult<CacheStats>;

    /// Clear the cache
    fn clear(&self) -> CacheResult<()>;

    /// Get approximate size in bytes
    fn size_bytes(&self) -> u64;
}

/// Type registry entry for runtime type management
#[derive(Debug, Clone)]
pub struct TypeRegistryEntry {
    pub type_name: String,
    pub type_id: TypeId,
    pub schema_version: u32,
    pub serializer: fn(&dyn erased_serde::Serialize) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>,
    pub deserializer: fn(&[u8]) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>,
}

/// Global type registry for managing cacheable types
#[derive(Debug)]
pub struct GlobalTypeRegistry {
    entries: HashMap<TypeId, TypeRegistryEntry>,
}

impl GlobalTypeRegistry {
    /// Create a new global type registry
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Register a type with the global registry
    pub fn register<T: 'static + Send + Sync + erased_serde::Serialize + for<'de> serde::Deserialize<'de>>(
        &mut self,
        type_name: &str,
        schema_version: u32,
    ) -> Result<(), CacheError> {
        let type_id = TypeId::of::<T>();

        if self.entries.contains_key(&type_id) {
            return Err(CacheError::TypeAlreadyRegistered(type_name.to_string()));
        }

        let entry = TypeRegistryEntry {
            type_name: type_name.to_string(),
            type_id,
            schema_version,
            serializer: |value| {
                // For now, we'll skip the type checking and just try to serialize
                // This is a simplified implementation
                serde_json::to_vec(&())
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
            },
            deserializer: |data| {
                let value: T = serde_json::from_slice(data)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                Ok(Box::new(value) as Box<dyn Any + Send + Sync>)
            },
        };

        self.entries.insert(type_id, entry);
        Ok(())
    }

    /// Get a type entry by TypeId
    pub fn get(&self, type_id: &TypeId) -> Option<&TypeRegistryEntry> {
        self.entries.get(type_id)
    }

    /// Check if a type is registered
    pub fn is_registered(&self, type_id: &TypeId) -> bool {
        self.entries.contains_key(type_id)
    }

    /// Get all registered types
    pub fn registered_types(&self) -> Vec<&TypeRegistryEntry> {
        self.entries.values().collect()
    }
}

impl Default for GlobalTypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Safe wrapper around Any for type-safe operations
#[derive(Debug)]
pub struct SafeAny {
    value: Box<dyn Any + Send + Sync>,
    type_id: TypeId,
}

impl SafeAny {
    /// Create a new SafeAny from a value
    pub fn new<T: 'static + Send + Sync>(value: T) -> Self {
        Self {
            value: Box::new(value),
            type_id: TypeId::of::<T>(),
        }
    }

    /// Try to downcast to a specific type
    pub fn downcast<T: 'static>(self) -> Result<T, Self> {
        if self.type_id == TypeId::of::<T>() {
            match self.value.downcast::<T>() {
                Ok(value) => Ok(*value),
                Err(value) => Err(Self { value, type_id: self.type_id }),
            }
        } else {
            Err(self)
        }
    }

    /// Try to downcast to a reference of a specific type
    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        self.value.downcast_ref::<T>()
    }

    /// Get the TypeId of the contained value
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }
}

/// Type-safe cache manager for handling multiple cache types
#[derive(Debug)]
pub struct TypeSafeCacheManager {
    caches: HashMap<String, Box<dyn TypeErasedCache>>,
    registry: Arc<GlobalTypeRegistry>,
}

impl TypeSafeCacheManager {
    /// Create a new type-safe cache manager
    pub fn new() -> Self {
        Self {
            caches: HashMap::new(),
            registry: Arc::new(GlobalTypeRegistry::new()),
        }
    }

    /// Register a cache with a name
    pub fn register_cache<C: TypeErasedCache + 'static>(
        &mut self,
        name: String,
        cache: C,
    ) -> Result<(), CacheError> {
        if self.caches.contains_key(&name) {
            return Err(CacheError::CacheAlreadyExists(name));
        }

        self.caches.insert(name, Box::new(cache));
        Ok(())
    }

    /// Get a cache by name
    pub fn get_cache(&self, name: &str) -> Option<&dyn TypeErasedCache> {
        self.caches.get(name).map(|c| c.as_ref())
    }

    /// Get cache statistics for all caches
    pub fn global_stats(&self) -> HashMap<String, CacheStats> {
        self.caches
            .iter()
            .filter_map(|(name, cache)| {
                cache.stats()
                    .ok()
                    .map(|stats| (name.clone(), stats))
            })
            .collect()
    }

    /// Clear all caches
    pub fn clear_all(&self) -> Vec<CacheError> {
        self.caches
            .values()
            .filter_map(|cache| cache.clear().err())
            .collect()
    }

    /// Get the global registry
    pub fn registry(&self) -> Arc<GlobalTypeRegistry> {
        Arc::clone(&self.registry)
    }
}

impl Default for TypeSafeCacheManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub value: T,
    pub metadata: CacheMetadata,
}

/// Cache result type alias
pub type CacheResult<T> = Result<T, CacheError>;

/// Cache error types
#[derive(Debug, Clone, thiserror::Error)]
pub enum CacheError {
    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Cache already exists: {0}")]
    CacheAlreadyExists(String),

    #[error("Type already registered: {0}")]
    TypeAlreadyRegistered(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Operation timeout")]
    Timeout,

    #[error("Cache is full")]
    CacheFull,

    #[error("IO error: {0}")]
    IoError(String),
}

/// Cache invalidation strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvalidationStrategy {
    /// Never invalidate
    Never,
    /// Time-based expiration
    TimeBased(Duration),
    /// LRU-based eviction
    Lru(usize),
    /// Size-based eviction
    SizeBased(u64),
    /// Custom invalidation logic
    Custom(String),
}

/// Typed cache key with type safety
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypedCacheKey {
    pub key: String,
    pub type_name: String,
}

/// Cached value with type information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CachedValue {
    String(String),
    Number(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<CachedValue>),
    Object(HashMap<String, CachedValue>),
    Null,
}

/// Typed cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypedCacheEntry {
    pub key: TypedCacheKey,
    pub value: CachedValue,
    pub type_info: TypeInfo,
    pub metadata: CacheMetadata,
}

/// Type information for cached values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeInfo {
    pub name: String,
    pub version: u32,
    pub size_bytes: u64,
}

/// Cache metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    pub access_count: u64,
    pub ttl: Option<Duration>,
    pub compressed: bool,
    pub checksum: Option<String>,
    pub tags: Vec<String>,
}

impl Default for CacheMetadata {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            created_at: now,
            last_accessed: now,
            access_count: 0,
            ttl: None,
            compressed: false,
            checksum: None,
            tags: Vec::new(),
        }
    }
}

/// Cache invalidation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidationRule {
    pub pattern: String,
    pub strategy: InvalidationStrategy,
    pub priority: u8,
}

/// Cache warming strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheWarmingStrategy {
    /// No warming
    None,
    /// Preload frequently accessed items
    PreloadFrequent,
    /// Predictive warming based on patterns
    Predictive,
    /// Custom warming logic
    Custom(String),
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub entries: usize,
    pub total_size_bytes: u64,
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub sets: u64,
    pub deletes: u64,
    pub hit_rate: f64,
    pub avg_access_time_ms: f64,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub name: String,
    pub max_size_bytes: Option<u64>,
    pub max_entries: Option<usize>,
    pub ttl: Option<Duration>,
    pub eviction_policy: EvictionPolicy,
    pub compression_enabled: bool,
    pub warming_strategy: CacheWarmingStrategy,
    pub invalidation_rules: Vec<InvalidationRule>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            max_size_bytes: Some(100 * 1024 * 1024), // 100MB
            max_entries: Some(10000),
            ttl: Some(Duration::from_secs(3600)), // 1 hour
            eviction_policy: EvictionPolicy::Lru,
            compression_enabled: false,
            warming_strategy: CacheWarmingStrategy::None,
            invalidation_rules: Vec::new(),
        }
    }
}

/// Cache eviction policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvictionPolicy {
    /// Least Recently Used
    Lru,
    /// Least Frequently Used
    Lfu,
    /// First In, First Out
    Fifo,
    /// Random eviction
    Random,
    /// Time-based expiration
    TimeBased,
    /// Size-based eviction
    SizeBased,
}

/// Core cache trait
#[async_trait]
pub trait Cache<K, V>: Send + Sync {
    /// Get a value from the cache
    async fn get(&self, key: &K) -> CacheResult<Option<V>>;

    /// Set a value in the cache
    async fn set(&self, key: K, value: V) -> CacheResult<()>;

    /// Set a value with TTL
    async fn set_with_ttl(&self, key: K, value: V, ttl: Duration) -> CacheResult<()>;

    /// Delete a value from the cache
    async fn delete(&self, key: &K) -> CacheResult<bool>;

    /// Check if a key exists
    async fn exists(&self, key: &K) -> CacheResult<bool>;

    /// Clear all entries
    async fn clear(&self) -> CacheResult<()>;

    /// Get cache statistics
    async fn stats(&self) -> CacheResult<CacheStats>;

    /// Get approximate size in bytes
    async fn size_bytes(&self) -> CacheResult<u64>;
}
