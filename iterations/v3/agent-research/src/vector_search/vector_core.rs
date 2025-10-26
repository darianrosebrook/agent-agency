//! Core Vector Search Engine
//!
//! Contains the main VectorSearchEngine struct and initialization logic.

use crate::research_types::*;
use anyhow::{Context, Result};
use qdrant_client::Qdrant;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info};
use uuid::Uuid;
use lru::LruCache;

use super::vector_metrics::VectorSearchMetrics;
use super::vector_search_cache::CacheManager;

/// Default cache sizes for in-memory LRU caches
const DEFAULT_SEARCH_CACHE_SIZE: usize = 1000;
const DEFAULT_EMBEDDING_CACHE_SIZE: usize = 5000;

/// Vector search engine for semantic knowledge retrieval
pub struct VectorSearchEngine {
    client: Arc<Qdrant>,
    collection_name: String,
    vector_size: u32,
    similarity_threshold: f32,
    max_results: u32,
    cache_manager: Arc<CacheManager>,
    metrics: Arc<RwLock<VectorSearchMetrics>>,
}

impl std::fmt::Debug for VectorSearchEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VectorSearchEngine")
            .field("collection_name", &self.collection_name)
            .field("vector_size", &self.vector_size)
            .field("similarity_threshold", &self.similarity_threshold)
            .field("max_results", &self.max_results)
            .field("metrics", &self.metrics)
            .finish()
    }
}

const PERSISTENT_CACHE_ENV_KEY: &str = "AA_VECTOR_CACHE_DIR";
const PERSISTENT_CACHE_LIMIT_ENV_KEY: &str = "AA_VECTOR_CACHE_LIMIT";
const DEFAULT_PERSISTENT_CACHE_DIR: &str = "cache/vector_search";
const DEFAULT_PERSISTENT_CACHE_LIMIT: usize = 10_000;

impl VectorSearchEngine {
    /// Create a new vector search engine
    pub async fn new(
        qdrant_url: &str,
        collection_name: &str,
        vector_size: u32,
        similarity_threshold: f32,
        max_results: u32,
    ) -> Result<Self> {
        debug!("Creating new VectorSearchEngine with collection: {}", collection_name);

        let client = Arc::new(
            Qdrant::from_url(qdrant_url)
                .build()
                .context("Failed to create Qdrant client")?,
        );

        let cache_manager = Arc::new(
            CacheManager::new(DEFAULT_SEARCH_CACHE_SIZE, DEFAULT_EMBEDDING_CACHE_SIZE)
                .await
                .context("Failed to initialize cache manager")?,
        );

        let metrics = Arc::new(RwLock::new(VectorSearchMetrics::default()));

        Ok(Self {
            client,
            collection_name: collection_name.to_string(),
            vector_size,
            similarity_threshold,
            max_results,
            cache_manager,
            metrics,
        })
    }

    /// Create a new vector search engine with custom cache directory
    pub async fn new_with_cache_dir(
        qdrant_url: &str,
        collection_name: &str,
        vector_size: u32,
        similarity_threshold: f32,
        max_results: u32,
        cache_dir: impl Into<PathBuf>,
    ) -> Result<Self> {
        debug!("Creating new VectorSearchEngine with custom cache dir");

        let mut engine = Self::new(
            qdrant_url,
            collection_name,
            vector_size,
            similarity_threshold,
            max_results,
        )
        .await?;

        engine.cache_manager = Arc::new(
            CacheManager::new_with_cache_dir(
                DEFAULT_SEARCH_CACHE_SIZE,
                DEFAULT_EMBEDDING_CACHE_SIZE,
                cache_dir,
            )
            .await
            .context("Failed to initialize cache manager with custom directory")?,
        );

        Ok(engine)
    }

    /// Get the Qdrant client
    pub fn client(&self) -> Arc<Qdrant> {
        Arc::clone(&self.client)
    }

    /// Get the collection name
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Get the vector size
    pub fn vector_size(&self) -> u32 {
        self.vector_size
    }

    /// Get the similarity threshold
    pub fn similarity_threshold(&self) -> f32 {
        self.similarity_threshold
    }

    /// Get the max results
    pub fn max_results(&self) -> u32 {
        self.max_results
    }

    /// Get the cache manager
    pub fn cache_manager(&self) -> Arc<CacheManager> {
        Arc::clone(&self.cache_manager)
    }

    /// Get the metrics
    pub fn metrics(&self) -> Arc<RwLock<VectorSearchMetrics>> {
        Arc::clone(&self.metrics)
    }
}
