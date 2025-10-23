//! Embedding cache for performance optimization

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Embedding cache for performance optimization
#[derive(Debug)]
pub struct EmbeddingCache {
    /// Cache configuration
    config: CacheConfig,
    /// Token embedding cache
    token_cache: Arc<RwLock<lru::LruCache<String, Vec<f32>>>>,
    /// Position embedding cache
    position_cache: Arc<RwLock<lru::LruCache<usize, Vec<f32>>>>,
    /// Segment embedding cache
    segment_cache: Arc<RwLock<lru::LruCache<usize, Vec<f32>>>>,
    /// Cache statistics
    stats: Arc<RwLock<EmbeddingCacheStats>>,
}

impl EmbeddingCache {
    /// Create new embedding cache with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(CacheConfig::default())
    }

    /// Create new embedding cache with custom configuration
    pub fn with_config(config: CacheConfig) -> Result<Self> {
        Ok(Self {
            config,
            token_cache: Arc::new(RwLock::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(config.max_token_entries).unwrap(),
            ))),
            position_cache: Arc::new(RwLock::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(config.max_position_entries).unwrap(),
            ))),
            segment_cache: Arc::new(RwLock::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(config.max_segment_entries).unwrap(),
            ))),
            stats: Arc::new(RwLock::new(EmbeddingCacheStats::default())),
        })
    }

    /// Get token embedding from cache
    pub async fn get_token(&self, token: &str) -> Option<Vec<f32>> {
        let mut stats = self.stats.write().await;
        let mut cache = self.token_cache.write().await;

        match cache.get(token) {
            Some(embedding) => {
                stats.token_hits += 1;
                Some(embedding.clone())
            }
            None => {
                stats.token_misses += 1;
                None
            }
        }
    }

    /// Put token embedding in cache
    pub async fn put_token(&self, token: String, embedding: Vec<f32>) {
        let mut cache = self.token_cache.write().await;
        cache.put(token, embedding);
    }

    /// Get position embedding from cache
    pub async fn get_position(&self, pos: usize) -> Option<Vec<f32>> {
        let mut stats = self.stats.write().await;
        let mut cache = self.position_cache.write().await;

        match cache.get(&pos) {
            Some(embedding) => {
                stats.position_hits += 1;
                Some(embedding.clone())
            }
            None => {
                stats.position_misses += 1;
                None
            }
        }
    }

    /// Put position embedding in cache
    pub async fn put_position(&self, pos: usize, embedding: Vec<f32>) {
        let mut cache = self.position_cache.write().await;
        cache.put(pos, embedding);
    }

    /// Get segment embedding from cache
    pub async fn get_segment(&self, seg: usize) -> Option<Vec<f32>> {
        let mut stats = self.stats.write().await;
        let mut cache = self.segment_cache.write().await;

        match cache.get(&seg) {
            Some(embedding) => {
                stats.segment_hits += 1;
                Some(embedding.clone())
            }
            None => {
                stats.segment_misses += 1;
                None
            }
        }
    }

    /// Put segment embedding in cache
    pub async fn put_segment(&self, seg: usize, embedding: Vec<f32>) {
        let mut cache = self.segment_cache.write().await;
        cache.put(seg, embedding);
    }

    /// Get cache statistics
    pub async fn stats(&self) -> EmbeddingCacheStats {
        self.stats.read().await.clone()
    }

    /// Clear all caches
    pub async fn clear(&self) {
        let mut token_cache = self.token_cache.write().await;
        let mut position_cache = self.position_cache.write().await;
        let mut segment_cache = self.segment_cache.write().await;
        let mut stats = self.stats.write().await;

        token_cache.clear();
        position_cache.clear();
        segment_cache.clear();

        stats.token_hits = 0;
        stats.token_misses = 0;
        stats.position_hits = 0;
        stats.position_misses = 0;
        stats.segment_hits = 0;
        stats.segment_misses = 0;
    }
}

/// Cache configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum number of token entries
    pub max_token_entries: usize,
    /// Maximum number of position entries
    pub max_position_entries: usize,
    /// Maximum number of segment entries
    pub max_segment_entries: usize,
    /// TTL for cache entries in seconds (0 = no expiration)
    pub ttl_seconds: u64,
    /// Whether to enable cache warming
    pub enable_warming: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_token_entries: 10000,
            max_position_entries: 512,
            max_segment_entries: 2,
            ttl_seconds: 3600, // 1 hour
            enable_warming: false,
        }
    }
}

/// Cache statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingCacheStats {
    /// Token cache hits
    pub token_hits: u64,
    /// Token cache misses
    pub token_misses: u64,
    /// Position cache hits
    pub position_hits: u64,
    /// Position cache misses
    pub position_misses: u64,
    /// Segment cache hits
    pub segment_hits: u64,
    /// Segment cache misses
    pub segment_misses: u64,
}

impl Default for EmbeddingCacheStats {
    fn default() -> Self {
        Self {
            token_hits: 0,
            token_misses: 0,
            position_hits: 0,
            position_misses: 0,
            segment_hits: 0,
            segment_misses: 0,
        }
    }
}

impl EmbeddingCacheStats {
    /// Calculate token cache hit rate
    pub fn token_hit_rate(&self) -> f32 {
        let total = self.token_hits + self.token_misses;
        if total == 0 {
            0.0
        } else {
            self.token_hits as f32 / total as f32
        }
    }

    /// Calculate position cache hit rate
    pub fn position_hit_rate(&self) -> f32 {
        let total = self.position_hits + self.position_misses;
        if total == 0 {
            0.0
        } else {
            self.position_hits as f32 / total as f32
        }
    }

    /// Calculate segment cache hit rate
    pub fn segment_hit_rate(&self) -> f32 {
        let total = self.segment_hits + self.segment_misses;
        if total == 0 {
            0.0
        } else {
            self.segment_hits as f32 / total as f32
        }
    }
}
