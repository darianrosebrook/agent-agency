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
use data_infrastructure::embedding::embedding_service::EmbeddingServiceFactory;
use data_infrastructure::embedding::embedding_types::EmbeddingConfig;
use data_infrastructure::embedding::EmbeddingService;
use std::sync::Arc;
use tokio::sync::OnceCell;

/// Search operations for vector search engine
pub struct SearchOperations {
    qdrant_client: Arc<QdrantClient>,
    cache_manager: Arc<CacheManager>,
    metrics: Arc<RwLock<VectorSearchMetrics>>,
    embedding_processor: EmbeddingProcessor,
    text_processor: TextProcessor,
    similarity_threshold: f32,
    max_results: u32,
    /// Shared embedding service (lazy-initialized)
    embedding_service: Arc<OnceCell<Box<dyn EmbeddingService>>>,
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
            embedding_service: Arc::new(OnceCell::new()),
        }
    }

    /// Initialize embedding service (lazy initialization)
    async fn get_embedding_service(&self) -> Result<&dyn EmbeddingService> {
        self.embedding_service.get_or_try_init(|| async {
            let config = EmbeddingConfig {
                model_name: "embeddinggemma".to_string(),
                dimension: 768,
                batch_size: 32,
                cache_size: 1000,
                timeout_ms: 30000,
            };
            
            Ok(EmbeddingServiceFactory::create_with_auto_detect(config, Some("embeddinggemma".to_string())).await)
        })
        .await
        .map(|s| s.as_ref())
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

        // TODO: Implement comprehensive sophisticated cache invalidation for deleted entries
        //       Currently relies on cache expiration; should implement comprehensive invalidation that tracks deleted entry IDs in cache, removes deleted entries immediately, and supports invalidation by entry ID or pattern.
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - Deleted entry IDs are tracked in cache
        // - Deleted entries are removed from cache immediately
        // - Cache invalidation supports entry ID or pattern matching
        // - Related entries are invalidated when appropriate
        //
        // DEPENDENCIES:
        // - Cache tracking system (Required)
        // - Pattern matching utilities (Required)
        // - Cache invalidation API (Required)
        //
        // ESTIMATED EFFORT: 8-12 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (cache invalidation functionality)
        // - Change Budget: ~200 LOC
        // - Reviewer Requirements: Cache management and invalidation expertise
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

    /// Generate embedding using CoreML (fallback to DummyEmbeddingProvider)
    async fn generate_embedding_from_api(&self, text: &str) -> Result<Vec<f32>> {
        debug!("Generating embedding for text (length: {})", text.len());

        // Get embedding service (lazy initialization)
        let service = self.get_embedding_service().await?;
        
        // Generate embedding
        let stored_embedding = service.generate_embedding(
            text,
            data_infrastructure::embedding::ContentType::Text,
            "vector_search"
        ).await?;
        
        // Extract vector values from EmbeddingVector
        Ok(stored_embedding.vector.values)
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
