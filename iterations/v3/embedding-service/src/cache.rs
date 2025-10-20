//! Embedding cache for performance optimization

use crate::types::*;
use crate::provider::EmbeddingProvider;
use dashmap::DashMap;
use lru::LruCache;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::path::PathBuf;
use futures::future::BoxFuture;

/// LRU cache for embeddings
pub struct EmbeddingCache {
    cache: Arc<RwLock<LruCache<String, StoredEmbedding>>>,
    max_size: usize,
    hit_counter: Arc<RwLock<CacheHitCounter>>,
}

impl EmbeddingCache {
    pub fn new(max_size: usize) -> Self {
        let non_zero_size = NonZeroUsize::new(max_size.max(1)).unwrap();
        Self {
            cache: Arc::new(RwLock::new(LruCache::new(non_zero_size))),
            max_size,
            hit_counter: Arc::new(RwLock::new(CacheHitCounter::new())),
        }
    }

    /// Get embedding from cache
    pub async fn get(&self, key: &str) -> Option<StoredEmbedding> {
        let mut cache = self.cache.write().await;
        let result = cache.get(key).cloned();

        // Track hit/miss
        let mut hit_counter = self.hit_counter.write().await;
        if result.is_some() {
            hit_counter.record_hit();
        } else {
            hit_counter.record_miss();
        }

        result
    }

    /// Store embedding in cache
    pub async fn put(&self, key: String, embedding: StoredEmbedding) {
        let mut cache = self.cache.write().await;
        cache.put(key, embedding);
    }

    /// Check if key exists in cache
    pub async fn contains(&self, key: &str) -> bool {
        let cache = self.cache.read().await;
        cache.contains(key)
    }

    /// Clear the cache
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let hit_counter = self.hit_counter.read().await;
        CacheStats {
            size: cache.len(),
            max_size: self.max_size,
            hit_rate: hit_counter.calculate_hit_rate(),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub max_size: usize,
    pub hit_rate: f32,
}

/// Thread-safe hash map for quick lookups
pub struct EmbeddingIndex {
    by_id: DashMap<String, StoredEmbedding>,
    by_content_type: DashMap<ContentType, Vec<String>>,
    by_tag: DashMap<String, Vec<String>>,
}

impl EmbeddingIndex {
    pub fn new() -> Self {
        Self {
            by_id: DashMap::new(),
            by_content_type: DashMap::new(),
            by_tag: DashMap::new(),
        }
    }

    pub fn insert(&self, embedding: StoredEmbedding) {
        let id = embedding.id.to_string();
        self.by_id.insert(id.clone(), embedding.clone());

        // Index by content type
        self.by_content_type
            .entry(embedding.metadata.content_type.clone())
            .or_insert_with(Vec::new)
            .push(id.clone());

        // Index by tags
        for tag in &embedding.metadata.tags {
            self.by_tag
                .entry(tag.clone())
                .or_insert_with(Vec::new)
                .push(id.clone());
        }
    }

    pub fn get_by_id(&self, id: &str) -> Option<StoredEmbedding> {
        self.by_id.get(id).map(|entry| entry.value().clone())
    }

    pub fn get_by_content_type(&self, content_type: &ContentType) -> Vec<StoredEmbedding> {
        self.by_content_type
            .get(content_type)
            .map(|entry| {
                entry
                    .value()
                    .iter()
                    .filter_map(|id| self.by_id.get(id).map(|e| e.value().clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_by_tag(&self, tag: &str) -> Vec<StoredEmbedding> {
        self.by_tag
            .get(tag)
            .map(|entry| {
                entry
                    .value()
                    .iter()
                    .filter_map(|id| self.by_id.get(id).map(|e| e.value().clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_all(&self) -> Vec<StoredEmbedding> {
        self.by_id
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    pub fn remove(&self, id: &str) -> Option<StoredEmbedding> {
        if let Some((_, embedding)) = self.by_id.remove(id) {
            // Remove from content type index
            if let Some(mut entry) = self
                .by_content_type
                .get_mut(&embedding.metadata.content_type)
            {
                entry.retain(|stored_id| stored_id != id);
                if entry.is_empty() {
                    drop(entry);
                    self.by_content_type
                        .remove_if(&embedding.metadata.content_type, |_, values| {
                            values.is_empty()
                        });
                }
            }

            // Remove from tag indexes
            for tag in &embedding.metadata.tags {
                if let Some(mut entry) = self.by_tag.get_mut(tag) {
                    entry.retain(|stored_id| stored_id != id);
                    if entry.is_empty() {
                        drop(entry);
                        self.by_tag.remove_if(tag, |_, values| values.is_empty());
                    }
                }
            }

            Some(embedding)
        } else {
            None
        }
    }

    pub fn stats(&self) -> EmbeddingIndexStats {
        let content_type_counts = self
            .by_content_type
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().len()))
            .collect::<HashMap<ContentType, usize>>();

        let tag_counts = self
            .by_tag
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().len()))
            .collect::<HashMap<String, usize>>();

        EmbeddingIndexStats {
            total_embeddings: self.by_id.len(),
            content_type_counts,
            tag_counts,
        }
    }

    pub fn clear(&self) {
        self.by_id.clear();
        self.by_content_type.clear();
        self.by_tag.clear();
    }

    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }
}

/// Summary statistics for the embedding index
#[derive(Debug, Clone)]
pub struct EmbeddingIndexStats {
    pub total_embeddings: usize,
    pub content_type_counts: HashMap<ContentType, usize>,
    pub tag_counts: HashMap<String, usize>,
}

/// Cache hit counter for tracking hit rate
pub struct CacheHitCounter {
    hits: u64,
    misses: u64,
}

impl CacheHitCounter {
    pub fn new() -> Self {
        Self { hits: 0, misses: 0 }
    }

    pub fn record_hit(&mut self) {
        self.hits += 1;
    }

    pub fn record_miss(&mut self) {
        self.misses += 1;
    }

    pub fn calculate_hit_rate(&self) -> f32 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f32 / total as f32
        }
    }

    pub fn get_hits(&self) -> u64 {
        self.hits
    }

    pub fn get_misses(&self) -> u64 {
        self.misses
    }

    pub fn get_total(&self) -> u64 {
        self.hits + self.misses
    }

    pub fn reset(&mut self) {
        self.hits = 0;
        self.misses = 0;
    }
}

/// Cached model entry with metadata
#[derive(Clone)]
pub struct CachedModel {
    /// The cached embedding provider
    pub provider: Arc<dyn EmbeddingProvider + Send + Sync>,
    /// When the model was loaded
    pub loaded_at: Instant,
    /// Last accessed time
    pub last_accessed: Instant,
    /// Model file path
    pub model_path: PathBuf,
    /// Model type/name
    pub model_name: String,
    /// Memory usage estimate in bytes
    pub memory_usage_bytes: usize,
}

/// LRU cache for loaded ML models with persistence support
pub struct ModelCache {
    /// LRU cache of loaded models keyed by model path + name
    cache: Arc<RwLock<LruCache<String, CachedModel>>>,
    /// Maximum number of models to cache
    max_models: usize,
    /// Maximum total memory usage in bytes
    max_memory_bytes: usize,
    /// Current total memory usage
    current_memory_bytes: Arc<RwLock<usize>>,
    /// Persistence storage for cache metadata
    persistence: Option<ModelCachePersistence>,
    /// Cache statistics
    stats: Arc<RwLock<ModelCacheStats>>,
}

impl ModelCache {
    /// Create a new model cache with specified limits
    pub fn new(max_models: usize, max_memory_bytes: usize) -> Self {
        let non_zero_size = NonZeroUsize::new(max_models.max(1)).unwrap();
        Self {
            cache: Arc::new(RwLock::new(LruCache::new(non_zero_size))),
            max_models,
            max_memory_bytes,
            current_memory_bytes: Arc::new(RwLock::new(0)),
            persistence: None,
            stats: Arc::new(RwLock::new(ModelCacheStats::new())),
        }
    }

    /// Create a model cache with persistence
    pub fn with_persistence(
        max_models: usize,
        max_memory_bytes: usize,
        persistence_path: PathBuf,
    ) -> Self {
        let mut cache = Self::new(max_models, max_memory_bytes);
        cache.persistence = Some(ModelCachePersistence::new(persistence_path));
        cache
    }

    /// Generate cache key from model path and name
    fn make_cache_key(model_path: &PathBuf, model_name: &str) -> String {
        format!("{}:{}", model_path.display(), model_name)
    }

    /// Get a cached model if it exists
    pub async fn get(&self, model_path: &PathBuf, model_name: &str) -> Option<Arc<dyn EmbeddingProvider + Send + Sync>> {
        let cache_key = Self::make_cache_key(model_path, model_name);
        let mut cache = self.cache.write().await;

        if let Some(cached_model) = cache.get_mut(&cache_key) {
            // Update last accessed time
            cached_model.last_accessed = Instant::now();

            // Update stats
            let mut stats = self.stats.write().await;
            stats.record_hit();

            Some(cached_model.provider.clone())
        } else {
            // Update stats
            let mut stats = self.stats.write().await;
            stats.record_miss();

            None
        }
    }

    /// Store a model in the cache with memory management
    pub async fn put(
        &self,
        model_path: PathBuf,
        model_name: String,
        provider: Arc<dyn EmbeddingProvider + Send + Sync>,
        memory_usage_bytes: usize,
    ) -> Result<(), ModelCacheError> {
        let cache_key = Self::make_cache_key(&model_path, &model_name);

        // Check if we need to evict models to make room
        self.evict_if_needed(memory_usage_bytes).await?;

        // Update current memory usage
        let mut current_memory = self.current_memory_bytes.write().await;
        *current_memory += memory_usage_bytes;

        let cached_model = CachedModel {
            provider,
            loaded_at: Instant::now(),
            last_accessed: Instant::now(),
            model_path: model_path.clone(),
            model_name: model_name.clone(),
            memory_usage_bytes,
        };

        // Store in cache
        let mut cache = self.cache.write().await;
        cache.put(cache_key, cached_model);

        // Persist cache metadata if enabled
        if let Some(persistence) = &self.persistence {
            if let Err(e) = persistence.save_cache_metadata(&cache).await {
                tracing::warn!("Failed to persist cache metadata: {}", e);
            }
        }

        Ok(())
    }

    /// Evict models if needed to make room for new model
    async fn evict_if_needed(&self, required_memory: usize) -> Result<(), ModelCacheError> {
        let mut current_memory = self.current_memory_bytes.write().await;

        // If we have room, no eviction needed
        if *current_memory + required_memory <= self.max_memory_bytes {
            return Ok(());
        }

        let mut cache = self.cache.write().await;
        let mut evicted_memory = 0usize;

        // Evict least recently used models until we have enough room
        while *current_memory + required_memory - evicted_memory > self.max_memory_bytes && !cache.is_empty() {
            if let Some((_, evicted_model)) = cache.pop_lru() {
                evicted_memory += evicted_model.memory_usage_bytes;

                // Update stats
                let mut stats = self.stats.write().await;
                stats.record_eviction();
            }
        }

        // Update memory counter
        *current_memory -= evicted_memory;

        // Check if we still don't have enough room
        if *current_memory + required_memory > self.max_memory_bytes {
            return Err(ModelCacheError::InsufficientMemory {
                required: required_memory,
                available: self.max_memory_bytes - *current_memory,
            });
        }

        Ok(())
    }

    /// Check if a model is cached
    pub async fn contains(&self, model_path: &PathBuf, model_name: &str) -> bool {
        let cache_key = Self::make_cache_key(model_path, model_name);
        let cache = self.cache.read().await;
        cache.contains(&cache_key)
    }

    /// Remove a specific model from cache
    pub async fn remove(&self, model_path: &PathBuf, model_name: &str) -> bool {
        let cache_key = Self::make_cache_key(model_path, model_name);
        let mut cache = self.cache.write().await;

        if let Some(evicted_model) = cache.pop(&cache_key) {
            // Update memory counter
            let mut current_memory = self.current_memory_bytes.write().await;
            *current_memory -= evicted_model.memory_usage_bytes;

            // Update stats
            let mut stats = self.stats.write().await;
            stats.record_manual_removal();

            true
        } else {
            false
        }
    }

    /// Clear all cached models
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();

        let mut current_memory = self.current_memory_bytes.write().await;
        *current_memory = 0;

        // Persist empty cache if enabled
        if let Some(persistence) = &self.persistence {
            if let Err(e) = persistence.save_cache_metadata(&cache).await {
                tracing::warn!("Failed to persist cleared cache metadata: {}", e);
            }
        }
    }

    /// Get cache statistics
    pub async fn stats(&self) -> ModelCacheStats {
        let cache = self.cache.read().await;
        let current_memory = self.current_memory_bytes.read().await;
        let stats = self.stats.read().await;

        ModelCacheStats {
            model_count: cache.len(),
            max_models: self.max_models,
            current_memory_bytes: *current_memory,
            max_memory_bytes: self.max_memory_bytes,
            hit_rate: stats.hit_rate,
            total_hits: stats.total_hits,
            total_misses: stats.total_misses,
            total_evictions: stats.total_evictions,
            total_manual_removals: stats.total_manual_removals,
        }
    }

    /// Warm up cache by pre-loading specified models
    pub async fn warmup<F>(
        &self,
        models_to_load: Vec<(PathBuf, String, usize)>, // (path, name, memory_estimate)
        loader: F,
    ) -> Result<(), ModelCacheError>
    where
        F: Fn(PathBuf, String) -> BoxFuture<'static, Result<Arc<dyn EmbeddingProvider + Send + Sync>, ModelCacheError>>,
    {
        for (model_path, model_name, memory_estimate) in models_to_load {
            if !self.contains(&model_path, &model_name).await {
                let provider = loader(model_path.clone(), model_name.clone()).await?;
                self.put(model_path, model_name, provider, memory_estimate).await?;
            }
        }
        Ok(())
    }

    /// Get list of currently cached models
    pub async fn list_cached_models(&self) -> Vec<ModelCacheInfo> {
        let cache = self.cache.read().await;
        cache.iter()
            .map(|(key, model)| ModelCacheInfo {
                cache_key: key.clone(),
                model_path: model.model_path.clone(),
                model_name: model.model_name.clone(),
                loaded_at: model.loaded_at,
                last_accessed: model.last_accessed,
                memory_usage_bytes: model.memory_usage_bytes,
            })
            .collect()
    }
}

/// Persistence layer for model cache metadata
pub struct ModelCachePersistence {
    metadata_path: PathBuf,
}

impl ModelCachePersistence {
    pub fn new(metadata_path: PathBuf) -> Self {
        Self { metadata_path }
    }

    /// Save cache metadata to disk
    async fn save_cache_metadata(&self, cache: &LruCache<String, CachedModel>) -> Result<(), ModelCacheError> {
        use tokio::fs;
        use std::collections::HashMap;

        let metadata: HashMap<String, PersistentModelMetadata> = cache
            .iter()
            .map(|(key, model)| {
                let loaded_at_epoch_secs = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or(Duration::ZERO)
                    .as_secs();

                (key.clone(), PersistentModelMetadata {
                    model_path: model.model_path.clone(),
                    model_name: model.model_name.clone(),
                    loaded_at_epoch_secs,
                    memory_usage_bytes: model.memory_usage_bytes,
                })
            })
            .collect();

        let json = serde_json::to_string_pretty(&metadata)
            .map_err(|e| ModelCacheError::PersistenceError(format!("Failed to serialize metadata: {}", e)))?;

        fs::write(&self.metadata_path, json)
            .await
            .map_err(|e| ModelCacheError::PersistenceError(format!("Failed to write metadata file: {}", e)))?;

        Ok(())
    }

    /// Load cache metadata from disk
    pub async fn load_cache_metadata(&self) -> Result<HashMap<String, PersistentModelMetadata>, ModelCacheError> {
        use tokio::fs;

        if !self.metadata_path.exists() {
            return Ok(HashMap::new());
        }

        let json = fs::read_to_string(&self.metadata_path)
            .await
            .map_err(|e| ModelCacheError::PersistenceError(format!("Failed to read metadata file: {}", e)))?;

        serde_json::from_str(&json)
            .map_err(|e| ModelCacheError::PersistenceError(format!("Failed to deserialize metadata: {}", e)))
    }
}

/// Persistent metadata for a cached model
#[derive(serde::Serialize, serde::Deserialize)]
pub struct PersistentModelMetadata {
    pub model_path: PathBuf,
    pub model_name: String,
    pub loaded_at_epoch_secs: u64,
    pub memory_usage_bytes: usize,
}

/// Information about a cached model
#[derive(Debug, Clone)]
pub struct ModelCacheInfo {
    pub cache_key: String,
    pub model_path: PathBuf,
    pub model_name: String,
    pub loaded_at: Instant,
    pub last_accessed: Instant,
    pub memory_usage_bytes: usize,
}

/// Statistics for model cache performance
#[derive(Debug, Clone)]
pub struct ModelCacheStats {
    pub model_count: usize,
    pub max_models: usize,
    pub current_memory_bytes: usize,
    pub max_memory_bytes: usize,
    pub hit_rate: f32,
    pub total_hits: u64,
    pub total_misses: u64,
    pub total_evictions: u64,
    pub total_manual_removals: u64,
}

impl ModelCacheStats {
    fn new() -> Self {
        Self {
            model_count: 0,
            max_models: 0,
            current_memory_bytes: 0,
            max_memory_bytes: 0,
            hit_rate: 0.0,
            total_hits: 0,
            total_misses: 0,
            total_evictions: 0,
            total_manual_removals: 0,
        }
    }

    fn record_hit(&mut self) {
        self.total_hits += 1;
    }

    fn record_miss(&mut self) {
        self.total_misses += 1;
    }

    fn record_eviction(&mut self) {
        self.total_evictions += 1;
    }

    fn record_manual_removal(&mut self) {
        self.total_manual_removals += 1;
    }

    fn calculate_hit_rate(&self) -> f32 {
        let total = self.total_hits + self.total_misses;
        if total == 0 {
            0.0
        } else {
            self.total_hits as f32 / total as f32
        }
    }
}

/// Errors that can occur during model caching operations
#[derive(Debug, thiserror::Error)]
pub enum ModelCacheError {
    #[error("Insufficient memory: required {required} bytes, available {available} bytes")]
    InsufficientMemory { required: usize, available: usize },

    #[error("Persistence error: {0}")]
    PersistenceError(String),

    #[error("Model loading error: {0}")]
    ModelLoadingError(String),
}

impl From<anyhow::Error> for ModelCacheError {
    fn from(err: anyhow::Error) -> Self {
        ModelCacheError::ModelLoadingError(err.to_string())
    }
}
