//! Core Vector Search Engine
//!
//! Contains the main VectorSearchEngine struct and initialization logic.

use crate::research_types::*;
use anyhow::{Context, Result};
use chrono::Utc;
use lru::LruCache;
use qdrant_client::Qdrant;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info};
use uuid::Uuid;

use super::qdrant::QdrantClient;
use super::search::SearchOperations;
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
        debug!(
            "Creating new VectorSearchEngine with collection: {}",
            collection_name
        );

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

    /// Create a SearchOperations instance for this engine
    fn search_operations(&self) -> SearchOperations {
        let qdrant_client = Arc::new(QdrantClient::new(
            Arc::clone(&self.client),
            self.collection_name.clone(),
        ));
        SearchOperations::new(
            qdrant_client,
            Arc::clone(&self.cache_manager),
            Arc::clone(&self.metrics),
            self.similarity_threshold,
            self.max_results,
        )
    }

    /// Generate embedding for text
    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        self.search_operations().generate_embedding(text).await
    }

    /// Perform vector search
    pub async fn search(
        &self,
        query_embedding: &[f32],
        limit: Option<usize>,
        score_threshold: Option<f32>,
    ) -> Result<Vec<KnowledgeEntry>> {
        let limit = limit.map(|l| l as u32).unwrap_or(self.max_results);
        let threshold = score_threshold.unwrap_or(self.similarity_threshold);
        let qdrant_client = Arc::new(QdrantClient::new(
            Arc::clone(&self.client),
            self.collection_name.clone(),
        ));
        let search_results = qdrant_client
            .search_similar(query_embedding, limit, threshold)
            .await?;

        // Convert SearchResult to KnowledgeEntry
        let knowledge_entries: Vec<KnowledgeEntry> = search_results
            .iter()
            .filter_map(|sr| {
                // Reconstruct KnowledgeEntry from SearchResult
                Some(KnowledgeEntry {
                    id: sr.id,
                    content: sr.content.clone(),
                    title: sr.title.clone(),
                    source: KnowledgeSource::InternalKnowledgeBase(sr.source.clone()),
                    content_type: ContentType::Text,
                    tags: vec![],
                    metadata: sr.metadata.clone(),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    access_count: 0,
                    last_accessed: None,
                    language: None,
                    embedding: None,
                    source_url: sr.url.clone(),
                })
            })
            .collect();

        Ok(knowledge_entries)
    }
}
