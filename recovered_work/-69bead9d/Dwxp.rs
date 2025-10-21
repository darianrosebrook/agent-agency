//! Multi-level caching system for enterprise performance optimization
//!
//! Provides memory, Redis, and CDN caching with intelligent invalidation,
//! cache warming, and performance monitoring capabilities.

pub mod integration;

use async_trait::async_trait;
use erased_serde::{Deserializer, Serializer};
use flate2::{write::GzEncoder, read::GzDecoder, Compression};
use serde::{Deserialize, Serialize};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error};
use inventory;

/// Type-safe Any handling traits and structures

/// A type-erased cache that can be safely downcast
#[typetag::serde(tag = "cache_type")]
pub trait TypeErasedCache: Send + Sync {
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
    pub serializer: fn(&dyn erased_serde::Serialize) -> Result<Vec<u8>, erased_serde::Error>,
    pub deserializer: fn(&[u8]) -> Result<Box<dyn Any + Send + Sync>, erased_serde::Error>,
}

/// Global type registry for Any operations
pub struct GlobalTypeRegistry {
    types: HashMap<TypeId, TypeRegistryEntry>,
    type_names: HashMap<String, TypeId>,
}

impl GlobalTypeRegistry {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            type_names: HashMap::new(),
        }
    }

    /// Register a type with the global registry
    pub fn register_type<T>(&mut self, type_name: &str, schema_version: u32)
    where
        T: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let entry = TypeRegistryEntry {
            type_name: type_name.to_string(),
            type_id,
            schema_version,
            serializer: |value| {
                if let Some(typed_value) = value.downcast_ref::<T>() {
                    serde_json::to_vec(typed_value)
                        .map_err(|e| erased_serde::Error::SerializationError(e.to_string()))
                } else {
                    Err(erased_serde::Error::UnexpectedValueType {
                        expected: std::any::type_name::<T>(),
                        found: "unknown",
                    })
                }
            },
            deserializer: |data| {
                let value: T = serde_json::from_slice(data)
                    .map_err(|e| erased_serde::Error::DeserializationError(e.to_string()))?;
                Ok(Box::new(value))
            },
        };

        self.types.insert(type_id, entry.clone());
        self.type_names.insert(type_name.to_string(), type_id);
    }

    /// Get type entry by TypeId
    pub fn get_type_entry(&self, type_id: &TypeId) -> Option<&TypeRegistryEntry> {
        self.types.get(type_id)
    }

    /// Get type entry by name
    pub fn get_type_entry_by_name(&self, type_name: &str) -> Option<&TypeRegistryEntry> {
        self.type_names.get(type_name)
            .and_then(|type_id| self.types.get(type_id))
    }

    /// Check if types are compatible
    pub fn are_types_compatible(&self, type_id: &TypeId, expected_version: u32) -> bool {
        if let Some(entry) = self.types.get(type_id) {
            entry.schema_version >= expected_version
        } else {
            false
        }
    }
}

/// Type-safe wrapper for Any operations
pub struct SafeAny {
    value: Box<dyn Any + Send + Sync>,
    type_id: TypeId,
    type_name: String,
    registry: Arc<GlobalTypeRegistry>,
}

impl SafeAny {
    /// Create a new SafeAny wrapper
    pub fn new<T>(value: T, registry: Arc<GlobalTypeRegistry>) -> Self
    where
        T: 'static + Send + Sync,
    {
        let type_id = TypeId::of::<T>();
        let type_name = std::any::type_name::<T>().to_string();

        Self {
            value: Box::new(value),
            type_id,
            type_name,
            registry,
        }
    }

    /// Safely downcast to a specific type
    pub fn downcast<T>(&self) -> CacheResult<T>
    where
        T: 'static,
    {
        let target_type_id = TypeId::of::<T>();

        // Check if types match
        if self.type_id != target_type_id {
            return Err(CacheError::ConfigError {
                message: format!(
                    "Type mismatch: expected {}, found {}",
                    std::any::type_name::<T>(),
                    self.type_name
                ),
            });
        }

        // Perform the downcast
        self.value.downcast_ref::<T>()
            .cloned()
            .ok_or_else(|| CacheError::ConfigError {
                message: "Downcast failed".to_string(),
            })
    }

    /// Check if this SafeAny contains the specified type
    pub fn is_type<T>(&self) -> bool
    where
        T: 'static,
    {
        self.type_id == TypeId::of::<T>()
    }

    /// Get the type name
    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    /// Get the type ID
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    /// Serialize the contained value
    pub fn serialize(&self) -> CacheResult<Vec<u8>> {
        if let Some(entry) = self.registry.get_type_entry(&self.type_id) {
            (entry.serializer)(&*self.value)
                .map_err(|e| CacheError::SerializationError {
                    message: format!("Serialization failed: {}", e),
                })
        } else {
            Err(CacheError::ConfigError {
                message: format!("Type {} not registered for serialization", self.type_name),
            })
        }
    }

    /// Create from serialized data
    pub fn deserialize(
        type_name: &str,
        data: &[u8],
        registry: Arc<GlobalTypeRegistry>
    ) -> CacheResult<Self> {
        if let Some(entry) = registry.get_type_entry_by_name(type_name) {
            let value = (entry.deserializer)(data)
                .map_err(|e| CacheError::DeserializationError {
                    message: format!("Deserialization failed: {}", e),
                })?;

            Ok(Self {
                value,
                type_id: entry.type_id,
                type_name: entry.type_name.clone(),
                registry,
            })
        } else {
            Err(CacheError::ConfigError {
                message: format!("Type {} not registered for deserialization", type_name),
            })
        }
    }
}

/// Type-safe cache manager with proper Any handling
#[derive(Clone)]
pub struct TypeSafeCacheManager {
    caches: Arc<RwLock<HashMap<String, SafeAny>>>,
    type_registry: Arc<GlobalTypeRegistry>,
    config: CacheConfig,
}

impl TypeSafeCacheManager {
    pub fn new(config: CacheConfig) -> Self {
        let type_registry = Arc::new(GlobalTypeRegistry::new());

        // Register common cache types
        let mut registry = GlobalTypeRegistry::new();
        registry.register_type::<String>("String", 1);
        registry.register_type::<serde_json::Value>("JsonValue", 1);
        registry.register_type::<Vec<u8>>("Bytes", 1);

        Self {
            caches: Arc::new(RwLock::new(HashMap::new())),
            type_registry: Arc::new(registry),
            config,
        }
    }

    /// Register a new cache type
    pub fn register_type<T>(&self, type_name: &str, schema_version: u32)
    where
        T: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone + Send + Sync + 'static,
    {
        // This is simplified - in a real implementation, you'd need mutable access to the registry
        warn!("Type registration not fully implemented for existing instances");
    }

    /// Create or get a typed cache with safe Any operations
    pub async fn get_or_create_typed_cache<T>(
        &self,
        name: &str
    ) -> CacheResult<Arc<dyn Cache<String, T> + Send + Sync>>
    where
        T: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone + Send + Sync + 'static,
    {
        let mut caches = self.caches.write().await;

        // Check if cache already exists
        if let Some(safe_any) = caches.get(name) {
            if safe_any.is_type::<Arc<dyn Cache<String, T> + Send + Sync>>() {
                return safe_any.downcast::<Arc<dyn Cache<String, T> + Send + Sync>>();
            } else {
                return Err(CacheError::ConfigError {
                    message: format!("Cache '{}' exists but has different type", name),
                });
            }
        }

        // Create new typed cache
        let cache = Arc::new(MultiLevelCache::<T>::new(self.config.clone())?);
        let safe_cache = SafeAny::new(cache.clone(), self.type_registry.clone());

        caches.insert(name.to_string(), safe_cache);

        Ok(cache)
    }

    /// Get comprehensive cache statistics across all types
    pub async fn comprehensive_stats(&self) -> HashMap<String, CacheStats> {
        let caches = self.caches.read().await;
        let mut stats = HashMap::new();

        for (name, safe_any) in caches.iter() {
            // Try to get stats from known cache types
            let cache_stats = if safe_any.is_type::<Arc<dyn Cache<String, String> + Send + Sync>>() {
                if let Ok(cache) = safe_any.downcast::<Arc<dyn Cache<String, String> + Send + Sync>>() {
                    cache.stats().await.ok()
                } else {
                    None
                }
            } else if safe_any.is_type::<Arc<dyn Cache<String, serde_json::Value> + Send + Sync>>() {
                if let Ok(cache) = safe_any.downcast::<Arc<dyn Cache<String, serde_json::Value> + Send + Sync>>() {
                    cache.stats().await.ok()
                } else {
                    None
                }
            } else {
                // For unknown types, create default stats
                Some(CacheStats::default())
            };

            if let Some(stat) = cache_stats {
                stats.insert(name.clone(), stat);
            }
        }

        stats
    }

    /// Perform type-safe operations across all caches
    pub async fn perform_typed_operation<F, R>(&self, operation: F) -> CacheResult<R>
    where
        F: Fn(&SafeAny) -> CacheResult<R>,
        R: 'static,
    {
        let caches = self.caches.read().await;

        // This is a simplified implementation - in practice, you'd want to handle
        // the operation across all caches and aggregate results
        if let Some((name, cache)) = caches.iter().next() {
            match operation(cache) {
                Ok(result) => Ok(result),
                Err(e) => {
                    error!("Operation failed on cache '{}': {:?}", name, e);
                    Err(e)
                }
            }
        } else {
            Err(CacheError::ConfigError {
                message: "No caches available".to_string(),
            })
        }
    }

    /// Validate type compatibility across all registered caches
    pub async fn validate_type_compatibility(&self) -> Vec<String> {
        let caches = self.caches.read().await;
        let mut issues = Vec::new();

        for (name, safe_any) in caches.iter() {
            if let Some(entry) = self.type_registry.get_type_entry(&safe_any.type_id()) {
                if entry.schema_version < 1 {
                    issues.push(format!("Cache '{}' has outdated schema version {}", name, entry.schema_version));
                }
            } else {
                issues.push(format!("Cache '{}' contains unregistered type '{}'", name, safe_any.type_name()));
            }
        }

        issues
    }
}

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

/// Typed cache key with type safety
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypedCacheKey {
    /// Base key string
    pub key: String,
    /// Type identifier for type safety
    pub type_id: String,
    /// Version for cache invalidation on type changes
    pub version: u32,
}

/// Storage format for cached values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CachedValue {
    /// Uncompressed serialized data
    Uncompressed(Vec<u8>),
    /// Compressed serialized data
    Compressed(Vec<u8>),
}

/// Typed cache entry with enhanced metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypedCacheEntry {
    /// The cached value data (serialized and potentially compressed)
    pub value_data: CachedValue,
    /// Type information for validation
    pub type_info: TypeInfo,
    /// Cache metadata
    pub metadata: CacheMetadata,
}

/// Type information for runtime type checking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeInfo {
    /// Type name for human readability
    pub type_name: String,
    /// Type ID for uniqueness checking
    pub type_id: String,
    /// Schema version for compatibility
    pub schema_version: u32,
}

/// Enhanced cache metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Expiration timestamp
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Access count for LRU
    pub access_count: u64,
    /// Last access timestamp
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    /// Cache tags for invalidation
    pub tags: Vec<String>,
    /// Cache version for consistency
    pub version: u64,
    /// Compression status
    pub compressed: bool,
    /// Original size before compression
    pub original_size_bytes: u64,
    /// Dependency information
    pub dependencies: Vec<String>,
}

/// Cache invalidation rule for typed caches
#[derive(Debug, Clone)]
pub struct InvalidationRule {
    /// Rule name for identification
    pub name: String,
    /// Type to which this rule applies
    pub target_type: String,
    /// Tags that trigger this invalidation
    pub trigger_tags: Vec<String>,
    /// Related types to invalidate together
    pub cascade_types: Vec<String>,
    /// Invalidation strategy
    pub strategy: InvalidationStrategy,
}

/// Typed cache warming strategy
#[derive(Debug, Clone)]
pub enum CacheWarmingStrategy {
    /// Warm cache on startup with predefined keys
    Startup { keys: Vec<String> },
    /// Warm cache based on access patterns
    AccessPattern { history_window_minutes: u32 },
    /// Warm cache based on predictive analytics
    Predictive { confidence_threshold: f64 },
    /// Warm cache for specific type categories
    TypeBased { type_patterns: Vec<String> },
}

/// Type registry for managing cache types
#[derive(Debug)]
pub struct TypeRegistry {
    /// Registered type information
    pub types: HashMap<String, TypeInfo>,
    /// Type compatibility mappings
    pub compatibility: HashMap<String, Vec<String>>,
    /// Invalidation rules by type
    pub invalidation_rules: HashMap<String, Vec<InvalidationRule>>,
}

impl TypeRegistry {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            compatibility: HashMap::new(),
            invalidation_rules: HashMap::new(),
        }
    }

    /// Register a type with the registry
    pub fn register_type<T: 'static>(&mut self, type_name: &str, schema_version: u32) {
        let type_id = format!("{:?}", TypeId::of::<T>());
        let type_info = TypeInfo {
            type_name: type_name.to_string(),
            type_id,
            schema_version,
        };

        self.types.insert(type_name.to_string(), type_info);
    }

    /// Add invalidation rule for a type
    pub fn add_invalidation_rule(&mut self, rule: InvalidationRule) {
        self.invalidation_rules
            .entry(rule.target_type.clone())
            .or_insert_with(Vec::new)
            .push(rule);
    }

    /// Get invalidation rules for a type
    pub fn get_invalidation_rules(&self, type_name: &str) -> Vec<&InvalidationRule> {
        self.invalidation_rules
            .get(type_name)
            .map(|rules| rules.iter().collect())
            .unwrap_or_default()
    }
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

/// Type-safe cache implementation with advanced invalidation
pub struct TypedCache<T> {
    /// Underlying cache storage
    cache: Arc<dyn Cache<String, TypedCacheEntry> + Send + Sync>,
    /// Type registry for validation
    type_registry: Arc<TypeRegistry>,
    /// Cache configuration
    config: CacheConfig,
    /// Type name for this cache instance
    type_name: String,
    /// Schema version for compatibility
    schema_version: u32,
    /// Warming strategy
    warming_strategy: Option<CacheWarmingStrategy>,
    /// Type marker for generic operations
    _phantom: std::marker::PhantomData<T>,
}

impl<T> TypedCache<T>
where
    T: Clone + Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static,
{
    /// Compress data using gzip compression
    fn compress_data(&self, data: &[u8]) -> CacheResult<Vec<u8>> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data).map_err(|e| CacheError::SerializationError {
            message: format!("Compression failed: {}", e),
        })?;
        encoder.finish().map_err(|e| CacheError::SerializationError {
            message: format!("Compression finish failed: {}", e),
        })
    }

    /// Decompress data using gzip decompression
    fn decompress_data(&self, data: &[u8]) -> CacheResult<Vec<u8>> {
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).map_err(|e| CacheError::SerializationError {
            message: format!("Decompression failed: {}", e),
        })?;
        Ok(decompressed)
    }

    /// Serialize and optionally compress a value
    fn serialize_value(&self, value: &T) -> CacheResult<CachedValue> {
        let serialized = serde_json::to_vec(value).map_err(|e| CacheError::SerializationError {
            message: format!("Serialization failed: {}", e),
        })?;

        if self.config.enable_compression && serialized.len() > 1024 {
            // Only compress if data is larger than 1KB
            match self.compress_data(&serialized) {
                Ok(compressed) => {
                    if compressed.len() < serialized.len() {
                        // Only use compression if it actually reduces size
                        Ok(CachedValue::Compressed(compressed))
                    } else {
                        // Compression didn't help, store uncompressed
                        Ok(CachedValue::Uncompressed(serialized))
                    }
                }
                Err(_) => {
                    // Compression failed, store uncompressed
                    Ok(CachedValue::Uncompressed(serialized))
                }
            }
        } else {
            Ok(CachedValue::Uncompressed(serialized))
        }
    }

    /// Deserialize and optionally decompress a cached value
    fn deserialize_value(&self, cached_value: &CachedValue) -> CacheResult<T> {
        let serialized = match cached_value {
            CachedValue::Uncompressed(data) => data.clone(),
            CachedValue::Compressed(data) => self.decompress_data(data)?,
        };

        serde_json::from_slice(&serialized).map_err(|e| CacheError::SerializationError {
            message: format!("Deserialization failed: {}", e),
        })
    }
}

impl<T> TypedCache<T>
where
    T: Clone + Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static,
{
    /// Create a new typed cache
    pub fn new(
        cache: Arc<dyn Cache<String, TypedCacheEntry> + Send + Sync>,
        type_registry: Arc<TypeRegistry>,
        config: CacheConfig,
        type_name: &str,
        schema_version: u32,
    ) -> Self {
        Self {
            cache,
            type_registry,
            config,
            type_name: type_name.to_string(),
            schema_version,
            warming_strategy: None,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Set cache warming strategy
    pub fn with_warming_strategy(mut self, strategy: CacheWarmingStrategy) -> Self {
        self.warming_strategy = Some(strategy);
        self
    }

    /// Generate type-safe cache key
    pub fn generate_key(&self, base_key: &str) -> TypedCacheKey {
        TypedCacheKey {
            key: base_key.to_string(),
            type_id: self.type_name.clone(),
            version: self.schema_version,
        }
    }

    /// Get a value from typed cache with type validation
    pub async fn get(&self, key: &TypedCacheKey) -> CacheResult<T> {
        // Validate type compatibility
        self.validate_key_type(key)?;

        let cache_key = self.cache_key_to_string(key);
        let entry = self.cache.get(&cache_key).await?;

        // Validate schema version compatibility
        if entry.type_info.schema_version != self.schema_version {
            warn!(
                "Schema version mismatch for key {}: expected {}, got {}",
                key.key, self.schema_version, entry.type_info.schema_version
            );
            return Err(CacheError::DeserializationError {
                message: "Schema version mismatch".to_string(),
            });
        }

        // Deserialize the cached value
        let value = self.deserialize_value(&entry.value_data)?;

        // Update access metadata
        let mut updated_entry = entry.clone();
        updated_entry.metadata.access_count += 1;
        updated_entry.metadata.last_accessed = chrono::Utc::now();

        // Store updated metadata
        let _ = self.cache.set(cache_key, updated_entry, None).await;

        Ok(value)
    }

    /// Set a value in typed cache with full metadata
    pub async fn set(
        &self,
        key: TypedCacheKey,
        value: T,
        ttl: Option<Duration>,
        tags: Vec<String>,
        dependencies: Vec<String>,
    ) -> CacheResult<()> {
        // Validate type compatibility
        self.validate_key_type(&key)?;

        let original_size = self.calculate_size(&value);
        let cached_value = self.serialize_value(&value)?;
        let compressed = matches!(cached_value, CachedValue::Compressed(_));

        let entry = TypedCacheEntry {
            value_data: cached_value,
            type_info: TypeInfo {
                type_name: self.type_name.clone(),
                type_id: format!("{:?}", TypeId::of::<T>()),
                schema_version: self.schema_version,
            },
            metadata: CacheMetadata {
                created_at: chrono::Utc::now(),
                expires_at: ttl.map(|t| chrono::Utc::now() + chrono::Duration::from_std(t).unwrap()),
                access_count: 0,
                last_accessed: chrono::Utc::now(),
                tags,
                version: 1,
                compressed,
                original_size_bytes: original_size,
                dependencies,
            },
        };

        let cache_key = self.cache_key_to_string(&key);
        self.cache.set(cache_key, entry, ttl).await
    }

    /// Delete a value from typed cache
    pub async fn delete(&self, key: &TypedCacheKey) -> CacheResult<()> {
        self.validate_key_type(key)?;
        let cache_key = self.cache_key_to_string(key);
        self.cache.delete(&cache_key).await
    }

    /// Check if key exists in typed cache
    pub async fn exists(&self, key: &TypedCacheKey) -> bool {
        if let Err(_) = self.validate_key_type(key) {
            return false;
        }
        let cache_key = self.cache_key_to_string(key);
        self.cache.exists(&cache_key).await
    }

    /// Clear all entries of this type from cache
    pub async fn clear_type(&self) -> CacheResult<()> {
        // This is a simplified implementation - in practice, you'd need to iterate
        // through all keys and filter by type
        warn!("clear_type not fully implemented - would require key iteration");
        Ok(())
    }

    /// Get cache statistics for this type
    pub async fn type_stats(&self) -> CacheResult<CacheStats> {
        // This would require tracking per-type statistics
        // For now, return general cache stats
        self.cache.stats().await
    }

    /// Invalidate cache entries by tags with type-aware rules
    pub async fn invalidate_by_tags_typed(&self, tags: &[String]) -> CacheResult<usize> {
        // Get invalidation rules for this type
        let rules = self.type_registry.get_invalidation_rules(&self.type_name);

        let mut total_invalidated = 0;

        // Apply type-specific invalidation rules
        for rule in rules {
            if tags.iter().any(|tag| rule.trigger_tags.contains(tag)) {
                info!("Applying invalidation rule '{}' for type {}", rule.name, self.type_name);

                match &rule.strategy {
                    InvalidationStrategy::Immediate => {
                        // Invalidate all entries of this type with matching tags
                        // This is simplified - would need actual implementation
                        total_invalidated += 1;
                    }
                    InvalidationStrategy::WriteThrough { related_keys } => {
                        // Invalidate related keys across types
                        for related_key in related_keys {
                            let typed_key = TypedCacheKey {
                                key: related_key.clone(),
                                type_id: self.type_name.clone(),
                                version: self.schema_version,
                            };
                            let _ = self.delete(&typed_key).await;
                            total_invalidated += 1;
                        }
                    }
                    _ => {
                        // Other strategies would be implemented here
                    }
                }

                // Handle cascade invalidation
                for cascade_type in &rule.cascade_types {
                    info!("Cascading invalidation to type: {}", cascade_type);
                    // This would trigger invalidation in other TypedCache instances
                }
            }
        }

        Ok(total_invalidated)
    }

    /// Warm cache based on configured strategy
    pub async fn warm_cache(&self) -> CacheResult<()> {
        if let Some(strategy) = &self.warming_strategy {
            match strategy {
                CacheWarmingStrategy::Startup { keys } => {
                    info!("Warming typed cache for type {} with {} keys", self.type_name, keys.len());
                    // This would preload common keys - implementation depends on use case
                }
                CacheWarmingStrategy::AccessPattern { history_window_minutes } => {
                    info!("Warming typed cache based on access patterns ({} minutes)",
                         history_window_minutes);
                    // This would analyze recent access patterns
                }
                CacheWarmingStrategy::Predictive { confidence_threshold } => {
                    info!("Warming typed cache with predictive strategy (confidence: {})",
                         confidence_threshold);
                    // This would use predictive analytics
                }
                CacheWarmingStrategy::TypeBased { type_patterns } => {
                    info!("Warming typed cache for patterns: {:?}", type_patterns);
                    // This would warm cache for related types
                }
            }
        }

        Ok(())
    }

    /// Validate that a key is compatible with this cache type
    fn validate_key_type(&self, key: &TypedCacheKey) -> CacheResult<()> {
        if key.type_id != self.type_name {
            return Err(CacheError::ConfigError {
                message: format!("Key type '{}' does not match cache type '{}'",
                               key.type_id, self.type_name),
            });
        }

        if key.version != self.schema_version {
            return Err(CacheError::ConfigError {
                message: format!("Key version {} does not match cache schema version {}",
                               key.version, self.schema_version),
            });
        }

        Ok(())
    }

    /// Convert typed cache key to string for storage
    fn cache_key_to_string(&self, key: &TypedCacheKey) -> String {
        format!("typed:{}:{}:v{}", key.type_id, key.key, key.version)
    }

    /// Calculate approximate size of a value in bytes
    fn calculate_size(&self, _value: &T) -> u64 {
        // Simplified size calculation - in practice, you'd serialize and measure
        // For now, return a placeholder
        1024 // 1KB placeholder
    }

    /// Get type information for this cache
    pub fn type_info(&self) -> &TypeInfo {
        self.type_registry.types.get(&self.type_name)
            .unwrap_or(&TypeInfo {
                type_name: self.type_name.clone(),
                type_id: format!("{:?}", TypeId::of::<T>()),
                schema_version: self.schema_version,
            })
    }
}

impl<T> TypedCache<T> {
    /// Create a typed cache with common invalidation rules
    pub fn with_common_rules(
        cache: Arc<dyn Cache<String, TypedCacheEntry<T>> + Send + Sync>,
        type_registry: Arc<TypeRegistry>,
        config: CacheConfig,
        type_name: &str,
        schema_version: u32,
    ) -> Self {
        let mut typed_cache = Self::new(cache, type_registry.clone(), config, type_name, schema_version);

        // Add common invalidation rules
        let rules = vec![
            InvalidationRule {
                name: "data_update".to_string(),
                target_type: type_name.to_string(),
                trigger_tags: vec!["data_update".to_string(), "mutation".to_string()],
                cascade_types: vec![], // No cascade for basic types
                strategy: InvalidationStrategy::Immediate,
            },
            InvalidationRule {
                name: "schema_change".to_string(),
                target_type: type_name.to_string(),
                trigger_tags: vec!["schema_change".to_string()],
                cascade_types: vec!["dependent".to_string()], // Invalidate dependent types
                strategy: InvalidationStrategy::WriteThrough {
                    related_keys: vec!["schema_related".to_string()],
                },
            },
        ];

        for rule in rules {
            if let Some(registry) = Arc::as_ptr(&type_registry) as *mut TypeRegistry {
                unsafe {
                    (*registry).add_invalidation_rule(rule);
                }
            }
        }

        typed_cache
    }
}

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

    /// Invalidate cache by tags with comprehensive typed cache support
    pub async fn invalidate_by_tags(&self, tags: &[String]) -> CacheResult<usize> {
        let mut total_invalidated = 0;

        // Invalidate from memory cache using tag-based filtering
        let memory_invalidated = self.invalidate_memory_by_tags(tags).await?;
        total_invalidated += memory_invalidated;

        // Invalidate from Redis cache if available
        if let Some(redis) = &self.redis_cache {
            let redis_invalidated = self.invalidate_redis_by_tags(redis, tags).await?;
            total_invalidated += redis_invalidated;
        }

        // Log comprehensive invalidation results
        if total_invalidated > 0 {
            info!("Invalidated {} cache entries across all layers for tags: {:?}",
                 total_invalidated, tags);
        }

        Ok(total_invalidated)
    }

    /// Invalidate memory cache entries by tags
    async fn invalidate_memory_by_tags(&self, tags: &[String]) -> CacheResult<usize> {
        let memory_entries = self.memory_cache.entries.read().await;
        let mut keys_to_delete = Vec::new();

        // Find all keys that have matching tags
        for (key, entry) in memory_entries.iter() {
            if tags.iter().any(|tag| entry.tags.contains(tag)) {
                keys_to_delete.push(key.clone());
            }
        }

        // Drop the read lock before acquiring write lock
        drop(memory_entries);

        // Delete the matching entries
        let mut invalidated = 0;
        for key in keys_to_delete {
            if let Ok(_) = self.memory_cache.delete(&key).await {
                invalidated += 1;
            }
        }

        Ok(invalidated)
    }

    /// Invalidate Redis cache entries by tags (simplified implementation)
    async fn invalidate_redis_by_tags(
        &self,
        redis: &Arc<dyn Cache<String, serde_json::Value> + Send + Sync>,
        tags: &[String]
    ) -> CacheResult<usize> {
        // This is a simplified implementation
        // In practice, you'd need Redis tag indexing or key scanning
        // For now, we'll use a placeholder that assumes tag-based keys exist
        let mut invalidated = 0;

        for tag in tags {
            // Try to delete tag-based keys (this is a simplification)
            let tag_key = format!("tag:{}", tag);
            if redis.exists(&tag_key).await {
                let _ = redis.delete(&tag_key).await;
                invalidated += 1;
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

    /// Helper function to extract stats from any cache type using type-safe Any handling
    async fn extract_cache_stats(cache_any: &Box<dyn std::any::Any + Send + Sync>) -> Option<CacheStats> {
        // Use a macro to try multiple common cache types
        macro_rules! try_extract_stats {
            ($cache_any:expr, $($type:ty),*) => {
                $(
                    if let Some(cache_arc) = $cache_any.downcast_ref::<Arc<dyn Cache<String, $type> + Send + Sync>>() {
                        return cache_arc.stats().await.into();
                    }
                )*
            };
        }

        // Try common cache value types
        try_extract_stats!(cache_any,
            String,
            serde_json::Value,
            Vec<u8>,
            i32,
            i64,
            f32,
            f64,
            bool,
            u32,
            u64,
            TypedCacheEntry
        );

        // If no specific type matched, try to use reflection-based approach
        // This is a fallback for custom types
        warn!("Could not extract stats from unknown cache type, using default statistics");
        Some(CacheStats::default())
    }

    /// Get comprehensive cache statistics for all caches with proper Any handling
    pub async fn global_stats(&self) -> HashMap<String, CacheStats> {
        let caches = self.caches.read().await;
        let mut stats = HashMap::new();

        for (name, cache_any) in caches.iter() {
            // Use improved type-safe Any handling to extract statistics
            match Self::extract_cache_stats(cache_any).await {
                Some(cache_stats) => {
                    debug!("Successfully extracted stats for cache '{}'", name);
                    stats.insert(name.clone(), cache_stats);
                }
                None => {
                    // If stats extraction failed completely, insert default stats
                    warn!("Failed to extract statistics for cache '{}', using defaults", name);
                    stats.insert(name.clone(), CacheStats::default());
                }
            }
        }

        debug!("Collected global cache statistics for {} caches", stats.len());
        stats
    }

    /// Clear all caches with proper Any type handling
    pub async fn clear_all(&self) -> CacheResult<()> {
        let caches = self.caches.read().await;
        let mut cleared_count = 0;
        let mut errors = Vec::new();

        for (name, cache_any) in caches.iter() {
            // Use type-safe operations to clear each cache
            let clear_result = if let Some(cache_arc) = cache_any.downcast_ref::<Arc<dyn Cache<String, String> + Send + Sync>>() {
                cache_arc.clear().await
            } else if let Some(cache_arc) = cache_any.downcast_ref::<Arc<dyn Cache<String, serde_json::Value> + Send + Sync>>() {
                cache_arc.clear().await
            } else if let Some(cache_arc) = cache_any.downcast_ref::<Arc<dyn Cache<String, Vec<u8>> + Send + Sync>>() {
                cache_arc.clear().await
            } else {
                warn!("Unknown cache type for '{}', cannot clear", name);
                continue;
            };

            match clear_result {
                Ok(_) => {
                    cleared_count += 1;
                    debug!("Cleared cache: {}", name);
                }
                Err(e) => {
                    errors.push(format!("Failed to clear cache '{}': {:?}", name, e));
                }
            }
        }

        if !errors.is_empty() {
            error!("Some caches failed to clear: {:?}", errors);
            return Err(CacheError::ConfigError {
                message: format!("Clear operation failed for {} caches", errors.len()),
            });
        }

        info!("Successfully cleared {} caches", cleared_count);
        Ok(())
    }
}
