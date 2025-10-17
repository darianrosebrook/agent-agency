//! Embedding cache for performance optimization

use crate::types::*;
use dashmap::DashMap;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::sync::RwLock;

/// LRU cache for embeddings
pub struct EmbeddingCache {
    cache: Arc<RwLock<LruCache<String, StoredEmbedding>>>,
    max_size: usize,
}

impl EmbeddingCache {
    pub fn new(max_size: usize) -> Self {
        let non_zero_size = NonZeroUsize::new(max_size.max(1)).unwrap();
        Self {
            cache: Arc::new(RwLock::new(LruCache::new(non_zero_size))),
            max_size,
        }
    }

    /// Get embedding from cache
    pub async fn get(&self, key: &str) -> Option<StoredEmbedding> {
        let mut cache = self.cache.write().await;
        cache.get(key).cloned()
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
        CacheStats {
            size: cache.len(),
            max_size: self.max_size,
            hit_rate: 0.0, // TODO: Implement hit rate tracking
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

    /// Insert embedding into index
    pub fn insert(&self, embedding: StoredEmbedding) {
        let id = embedding.id.as_str().to_string();

        // Index by ID
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

    /// Get embedding by ID
    pub fn get_by_id(&self, id: &str) -> Option<StoredEmbedding> {
        self.by_id.get(id).map(|entry| entry.value().clone())
    }

    /// Get embeddings by content type
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

    /// Get embeddings by tag
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

    /// Remove embedding from index
    pub fn remove(&self, id: &str) -> Option<StoredEmbedding> {
        if let Some((_, embedding)) = self.by_id.remove(id) {
            // Remove from content type index
            if let Some(mut entry) = self
                .by_content_type
                .get_mut(&embedding.metadata.content_type)
            {
                entry.retain(|stored_id| stored_id != id);
            }

            // Remove from tag indices
            for tag in &embedding.metadata.tags {
                if let Some(mut entry) = self.by_tag.get_mut(tag) {
                    entry.retain(|stored_id| stored_id != id);
                }
            }

            Some(embedding)
        } else {
            None
        }
    }

    /// Get all embeddings
    pub fn get_all(&self) -> Vec<StoredEmbedding> {
        self.by_id
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get index statistics
    pub fn stats(&self) -> IndexStats {
        IndexStats {
            total_embeddings: self.by_id.len(),
            content_types: self.by_content_type.len(),
            tags: self.by_tag.len(),
        }
    }
}

/// Index statistics
#[derive(Debug, Clone)]
pub struct IndexStats {
    pub total_embeddings: usize,
    pub content_types: usize,
    pub tags: usize,
}
