//! Main embedding service implementation

use crate::cache::*;
use crate::provider::*;
use crate::similarity::*;
use crate::types::*;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

/// Main embedding service trait
#[async_trait]
pub trait EmbeddingService: Send + Sync {
    /// Generate a single embedding
    async fn generate_embedding(
        &self,
        text: &str,
        content_type: ContentType,
        source: &str,
    ) -> Result<StoredEmbedding>;

    /// Generate multiple embeddings
    async fn generate_embeddings(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse>;

    /// Search for similar embeddings
    async fn search_similar(&self, request: SimilarityRequest) -> Result<Vec<SimilarityResult>>;

    /// Store an embedding
    async fn store_embedding(&self, embedding: StoredEmbedding) -> Result<()>;

    /// Get embedding by ID
    async fn get_embedding(&self, id: &str) -> Result<Option<StoredEmbedding>>;

    /// Health check
    async fn health_check(&self) -> Result<bool>;
}

/// Main embedding service implementation
pub struct EmbeddingServiceImpl {
    provider: Arc<dyn EmbeddingProvider>,
    cache: EmbeddingCache,
    index: Arc<EmbeddingIndex>,
    config: EmbeddingConfig,
}

impl EmbeddingServiceImpl {
    pub fn new(provider: Arc<dyn EmbeddingProvider>, config: EmbeddingConfig) -> Self {
        Self {
            provider,
            cache: EmbeddingCache::new(config.cache_size),
            index: Arc::new(EmbeddingIndex::new()),
            config,
        }
    }

    /// Generate cache key for text
    fn cache_key(&self, text: &str, content_type: &ContentType, source: &str) -> String {
        format!("{:?}:{}:{}", content_type, source, text)
    }

    /// Create embedding metadata
    fn create_metadata(
        &self,
        text: &str,
        content_type: ContentType,
        source: &str,
        tags: Vec<String>,
    ) -> EmbeddingMetadata {
        EmbeddingMetadata {
            source: source.to_string(),
            content_type,
            created_at: chrono::Utc::now(),
            tags,
            context: std::collections::HashMap::new(),
        }
    }
}

#[async_trait]
impl EmbeddingService for EmbeddingServiceImpl {
    async fn generate_embedding(
        &self,
        text: &str,
        content_type: ContentType,
        source: &str,
    ) -> Result<StoredEmbedding> {
        let cache_key = self.cache_key(text, &content_type, source);

        // Check cache first
        if let Some(cached) = self.cache.get(&cache_key).await {
            return Ok(cached);
        }

        // Generate new embedding
        let start_time = std::time::Instant::now();
        let vectors = self
            .provider
            .generate_embeddings(&[text.to_string()])
            .await?;
        let processing_time = start_time.elapsed().as_millis() as u64;

        let vector = vectors
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No embedding generated"))?;

        let id = EmbeddingId::new(Uuid::new_v4().to_string());
        let metadata = self.create_metadata(text, content_type, source, vec![]);

        let embedding = StoredEmbedding {
            id,
            vector,
            metadata,
        };

        // Cache the result
        self.cache.put(cache_key, embedding.clone()).await;

        Ok(embedding)
    }

    async fn generate_embeddings(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse> {
        let start_time = std::time::Instant::now();

        // Check cache for each text
        let mut cached_embeddings = Vec::new();
        let mut texts_to_generate = Vec::new();
        let mut cache_keys = Vec::new();

        for text in &request.texts {
            let cache_key = self.cache_key(text, &request.content_type, &request.source);
            cache_keys.push(cache_key.clone());

            if let Some(cached) = self.cache.get(&cache_key).await {
                cached_embeddings.push(cached);
            } else {
                texts_to_generate.push(text.clone());
            }
        }

        // Generate embeddings for uncached texts
        let mut new_embeddings = Vec::new();
        if !texts_to_generate.is_empty() {
            let vectors = self
                .provider
                .generate_embeddings(&texts_to_generate)
                .await?;

            for (i, vector) in vectors.into_iter().enumerate() {
                let id = EmbeddingId::new(Uuid::new_v4().to_string());
                let metadata = self.create_metadata(
                    &texts_to_generate[i],
                    request.content_type.clone(),
                    &request.source,
                    request.tags.clone(),
                );

                let embedding = StoredEmbedding {
                    id,
                    vector,
                    metadata,
                };

                // Cache the result
                let cache_key = &cache_keys[cached_embeddings.len() + i];
                self.cache.put(cache_key.clone(), embedding.clone()).await;

                new_embeddings.push(embedding);
            }
        }

        // Combine cached and new embeddings
        let mut all_embeddings = cached_embeddings;
        all_embeddings.extend(new_embeddings);

        let processing_time = start_time.elapsed().as_millis() as u64;

        Ok(EmbeddingResponse {
            embeddings: all_embeddings,
            processing_time_ms: processing_time,
        })
    }

    async fn search_similar(&self, request: SimilarityRequest) -> Result<Vec<SimilarityResult>> {
        let all_embeddings = self.index.get_all();

        find_similar_embeddings(
            &request.query_vector,
            &all_embeddings,
            request.limit,
            request.threshold,
            &request.content_types,
            &request.tags,
        )
    }

    async fn store_embedding(&self, embedding: StoredEmbedding) -> Result<()> {
        // Store in index
        self.index.insert(embedding.clone());

        // Store in cache
        let cache_key = self.cache_key(
            &embedding.metadata.source,
            &embedding.metadata.content_type,
            &embedding.metadata.source,
        );
        self.cache.put(cache_key, embedding).await;

        Ok(())
    }

    async fn get_embedding(&self, id: &str) -> Result<Option<StoredEmbedding>> {
        Ok(self.index.get_by_id(id))
    }

    async fn health_check(&self) -> Result<bool> {
        self.provider.health_check().await
    }
}

/// Factory for creating embedding services
pub struct EmbeddingServiceFactory;

impl EmbeddingServiceFactory {
    /// Create Ollama-based embedding service
    pub fn create_ollama_service(config: EmbeddingConfig) -> Result<Box<dyn EmbeddingService>> {
        let provider = Arc::new(OllamaEmbeddingProvider::new(&config));
        let service = EmbeddingServiceImpl::new(provider, config);
        Ok(Box::new(service))
    }

    /// Create dummy service for testing
    pub fn create_dummy_service(config: EmbeddingConfig) -> Box<dyn EmbeddingService> {
        let provider = Arc::new(DummyEmbeddingProvider::new(config.dimension));
        let service = EmbeddingServiceImpl::new(provider, config);
        Box::new(service)
    }
}
