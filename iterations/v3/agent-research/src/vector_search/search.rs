//! Vector Search Operations
//!
//! Handles search queries, result processing, and cache integration.

use crate::research_types::*;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use super::core::VectorSearchEngine;
use super::cache::CacheManager;
use super::metrics::VectorSearchMetrics;
use super::embedding::EmbeddingProcessor;
use super::qdrant::QdrantClient;
use super::text_processing::TextProcessor;
use data_infrastructure::embedding::provider::OllamaEmbeddingProvider;
use data_infrastructure::embedding::embedding_types::{EmbeddingConfig, EmbeddingProviderType};

/// Search operations for vector search engine
pub struct SearchOperations {
    qdrant_client: Arc<QdrantClient>,
    cache_manager: Arc<CacheManager>,
    metrics: Arc<RwLock<VectorSearchMetrics>>,
    embedding_processor: EmbeddingProcessor,
    text_processor: TextProcessor,
    similarity_threshold: f32,
    max_results: u32,
}

impl SearchOperations {
    /// Create new search operations
    pub fn new(
        qdrant_client: Arc<QdrantClient>,
        cache_manager: Arc<CacheManager>,
        metrics: Arc<RwLock<VectorSearchMetrics>>,
        similarity_threshold: f32,
        max_results: u32,
    ) -> Self {
        Self {
            qdrant_client,
            cache_manager,
            metrics,
            embedding_processor: EmbeddingProcessor::new(),
            text_processor: TextProcessor::new(),
            similarity_threshold,
            max_results,
        }
    }

    /// Perform semantic search
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        let start_time = std::time::Instant::now();

        debug!("Performing semantic search for query: {}", query);

        // Check cache first
        let cache_key = self.text_processor.create_cache_key(query);
        if let Some(cached_results) = self.cache_manager.get_search_cache(&cache_key).await {
            debug!("Cache hit for query: {}", query);
            let mut metrics = self.metrics.write().await;
            metrics.record_cache_hit();
            return Ok(cached_results);
        }

        // Generate embedding for query
        let query_embedding = self.generate_query_embedding(query).await?;
        self.embedding_processor.validate_embedding_quality(&query_embedding)?;

        // Search in Qdrant
        let results = self.qdrant_client
            .search_similar(&query_embedding, self.max_results, self.similarity_threshold)
            .await?;

        // Cache results
        let search_results = results.clone();
        self.cache_manager.put_search_cache(cache_key, search_results).await;

        // Record metrics
        let duration_ms = start_time.elapsed().as_millis() as f64;
        let result_count = results.len();
        let mut metrics = self.metrics.write().await;
        metrics.record_search(duration_ms, result_count);

        info!("Search completed in {:.2}ms, found {} results", duration_ms, result_count);

        Ok(results)
    }

    /// Add knowledge entry to search index
    pub async fn add_knowledge_entry(&self, entry: &KnowledgeEntry) -> Result<()> {
        debug!("Adding knowledge entry to search index: {}", entry.id);

        // Generate embedding
        let embedding = self.generate_embedding(&entry.content).await?;

        // Store in Qdrant
        self.qdrant_client.add_knowledge_entry(entry, &embedding).await?;

        // Cache embedding for future use
        let content_hash = self.text_processor.create_cache_key(&entry.content);
        self.cache_manager.put_embedding_cache(content_hash, embedding).await;

        Ok(())
    }

    /// Update knowledge entry in search index
    pub async fn update_knowledge_entry(&self, entry: &KnowledgeEntry) -> Result<()> {
        debug!("Updating knowledge entry in search index: {}", entry.id);

        // Generate new embedding
        let embedding = self.generate_embedding(&entry.content).await?;

        // Update in Qdrant
        self.qdrant_client.update_knowledge_entry(entry, &embedding).await?;

        // Update cache
        let content_hash = self.text_processor.create_cache_key(&entry.content);
        self.cache_manager.put_embedding_cache(content_hash, embedding).await;

        Ok(())
    }

    /// Delete knowledge entry from search index
    pub async fn delete_knowledge_entry(&self, entry_id: &uuid::Uuid) -> Result<()> {
        debug!("Deleting knowledge entry from search index: {}", entry_id);

        self.qdrant_client.delete_knowledge_entry(entry_id).await?;

        // Note: Cache invalidation for deleted entries would need more sophisticated tracking
        // For now, we rely on cache expiration

        Ok(())
    }

    /// Fetch all knowledge entries
    pub async fn fetch_all_entries(&self, batch_size: Option<u32>) -> Result<Vec<KnowledgeEntry>> {
        debug!("Fetching all knowledge entries");

        self.qdrant_client.fetch_all_entries(batch_size).await
    }

    /// Generate embedding for text (with caching)
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let content_hash = self.text_processor.create_cache_key(text);

        // Check cache first
        if let Some(cached_embedding) = self.cache_manager.get_embedding_cache(&content_hash).await {
            debug!("Embedding cache hit for text hash: {}", content_hash);
            return Ok(cached_embedding);
        }

        // Generate new embedding
        let processed_text = self.text_processor.preprocess_text_for_embedding(text)?;
        let embedding = self.generate_embedding_from_api(&processed_text).await?;

        // Process and validate embedding
        let processed_embedding = self.embedding_processor.process_embedding(embedding)?;
        self.embedding_processor.validate_embedding_quality(&processed_embedding)?;

        // Cache for future use
        self.cache_manager.put_embedding_cache(content_hash, processed_embedding.clone()).await;

        Ok(processed_embedding)
    }

    /// Generate embedding for query (separate from content embedding for better caching)
    async fn generate_query_embedding(&self, query: &str) -> Result<Vec<f32>> {
        let processed_query = self.text_processor.preprocess_text_for_embedding(query)?;
        let embedding = self.generate_embedding_from_api(&processed_query).await?;
        self.embedding_processor.process_embedding(embedding)
    }

    /// Generate embedding using external API
    async fn generate_embedding_from_api(&self, text: &str) -> Result<Vec<f32>> {
        debug!("Generating embedding for text (length: {})", text.len());

        // Use Ollama embedding provider for real embeddings
        let config = EmbeddingConfig {
            provider: EmbeddingProviderType::Ollama,
            model_name: "nomic-embed-text".to_string(),
            dimension: 768,
            ollama_url: "http://localhost:11434".to_string(),
            timeout_ms: 30000,
        };

        let provider = OllamaEmbeddingProvider::new(&config);
        
        // Generate embedding
        let embeddings = provider.generate_embeddings(&[text.to_string()]).await?;
        
        if let Some(embedding) = embeddings.first() {
            Ok(embedding.values.clone())
        } else {
            Err(anyhow::anyhow!("No embedding generated"))
        }
    }

    /// Clear all search caches
    pub async fn clear_cache(&self) -> Result<()> {
        debug!("Clearing search caches");
        self.cache_manager.clear_all_caches().await?;
        Ok(())
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> (usize, usize) {
        self.cache_manager.get_cache_stats().await
    }
}
