//! Embedding cache for performance optimization

use crate::types::*;
use dashmap::DashMap;
use lru::LruCache;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::sync::RwLock;

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
