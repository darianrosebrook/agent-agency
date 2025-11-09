//! Main embedding service implementation

use super::embedding_cache::*;
use super::provider::*;
use super::similarity::*;
use super::embedding_types::*;
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
    _config: EmbeddingConfig,
}

impl EmbeddingServiceImpl {
    pub fn new(provider: Arc<dyn EmbeddingProvider>, config: EmbeddingConfig) -> Self {
        Self {
            provider,
            cache: EmbeddingCache::new(config.cache_size),
            index: Arc::new(EmbeddingIndex::new()),
            _config: config,
        }
    }

    /// Generate cache key for text
    fn cache_key(&self, text: &str, content_type: &ContentType, source: &str) -> String {
        format!("{:?}:{}:{}", content_type, source, text)
    }

    /// Create embedding metadata
    fn create_metadata(
        &self,
        _text: &str,
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
        let _processing_time = start_time.elapsed().as_millis() as u64;

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
    /// Create embedding service using ONNX Runtime provider (preferred for ANE acceleration)
    ///
    /// Attempts to load ONNX embedding model with CoreMLExecutionProvider for ANE acceleration.
    /// Falls back to DummyEmbeddingProvider if unavailable.
    ///
    /// # Arguments
    /// * `model_path` - Path to ONNX embedding model (.onnx)
    /// * `model_name` - Model identifier ("embeddinggemma" for 768-dim)
    /// * `config` - Embedding configuration
    /// * `tokenizer` - Tokenizer for text preprocessing
    pub async fn create_onnx_service(
        model_path: std::path::PathBuf,
        model_name: String,
        config: EmbeddingConfig,
        tokenizer: std::sync::Arc<dyn crate::embedding::Tokenizer>,
    ) -> Box<dyn EmbeddingService> {
        use crate::embedding::provider::OnnxEmbeddingProvider;
        
        let dimension = match model_name.as_str() {
            "embeddinggemma" => 768,
            _ => config.dimension,
        };

        match OnnxEmbeddingProvider::new(
            model_path,
            tokenizer,
            dimension,
            model_name.clone(),
            // TODO: Make max_length configurable based on model capabilities:
            // 1. Model capability detection: Detect model's maximum sequence length
            //    - Query model metadata for max_length capability
            //    - Support different max_length values per model
            //    - Handle models with variable max_length
            // 2. Configuration support: Add max_length to configuration
            //    - Allow per-model max_length configuration
            //    - Support default max_length values
            //    - Validate max_length against model capabilities
            // 3. Dynamic adjustment: Adjust max_length dynamically
            //    - Consider input text length when setting max_length
            //    - Optimize max_length for performance vs quality tradeoff
            //    - Handle max_length errors gracefully
            // ACCEPTANCE CRITERIA:
            // - Max_length is configurable per model
            // - Max_length respects model capabilities
            // - Configuration supports default and per-model values
            // DEPENDENCIES:
            // - Model metadata API (Required)
            // - Configuration system (Required)
            // PRIORITY: Medium
            512,
        ).await {
            Ok(provider) => {
                tracing::info!("ONNX Runtime embedding provider created successfully");
                let provider = std::sync::Arc::new(provider);
                let service = EmbeddingServiceImpl::new(provider, config);
                Box::new(service)
            }
            Err(e) => {
                tracing::warn!("Failed to load ONNX Runtime embedding provider: {}, falling back to DummyEmbeddingProvider", e);
                Self::create_dummy_service(config)
            }
        }
    }
    
    /// Create embedding service using CoreML provider (legacy fallback)
    ///
    /// Attempts to load CoreML embedding model, falls back to DummyEmbeddingProvider if unavailable.
    ///
    /// # Arguments
    /// * `model_path` - Path to CoreML embedding model (.mlmodel or .mlpackage)
    /// * `model_name` - Model identifier ("embeddinggemma" for 768-dim)
    /// * `config` - Embedding configuration
    /// * `tokenizer` - Tokenizer for text preprocessing
    pub async fn create_coreml_service(
        model_path: std::path::PathBuf,
        model_name: String,
        config: EmbeddingConfig,
        tokenizer: std::sync::Arc<dyn crate::embedding::Tokenizer>,
    ) -> Box<dyn EmbeddingService> {
        use crate::embedding::provider::CoreMLEmbeddingProvider;
        
        let dimension = match model_name.as_str() {
            "embeddinggemma" => 768,
            _ => config.dimension,
        };

        match CoreMLEmbeddingProvider::new(
            model_path,
            model_name.clone(),
            dimension,
            tokenizer,
            Some(config.batch_size),
        ).await {
            Ok(provider) => {
                let provider = std::sync::Arc::new(provider);
                let service = EmbeddingServiceImpl::new(provider, config);
                Box::new(service)
            }
            Err(e) => {
                tracing::warn!("Failed to load CoreML embedding provider: {}, falling back to DummyEmbeddingProvider", e);
                Self::create_dummy_service(config)
            }
        }
    }

    /// Create embedding service with automatic CoreML detection (recommended)
    ///
    /// Attempts to load CoreML embedding model from standard locations:
    /// - Checks `COREML_EMBEDDING_MODEL_PATH` environment variable
    /// - Falls back to `{COREML_MODELS_PATH}/embeddinggemma.mlmodel`
    /// - Falls back to `{COREML_MODELS_PATH}/embeddinggemma.gguf` (Ollama format)
    /// - Finally falls back to DummyEmbeddingProvider if CoreML unavailable
    ///
    /// Note: GGUF files (from Ollama) may need conversion to .mlmodel format for CoreML.
    /// The bridge may handle this automatically, or conversion may be required.
    ///
    /// Decision: Uses embeddinggemma as the standard CoreML embedding model.
    /// Selected over e5-small-v2 due to better quality (768 dimensions) and availability.
    ///
    /// # Arguments
    /// * `config` - Embedding configuration
    /// * `preferred_model` - Preferred model name (default: "embeddinggemma")
    pub async fn create_with_auto_detect(
        config: EmbeddingConfig,
        preferred_model: Option<String>,
    ) -> Box<dyn EmbeddingService> {
        
        use std::sync::Arc;
        use std::path::PathBuf;

        let model_name = preferred_model.unwrap_or_else(|| "embeddinggemma".to_string());
        
        // Try to find ONNX or CoreML model path (ONNX preferred for ANE acceleration)
        let model_result = std::env::var("COREML_EMBEDDING_MODEL_PATH")
            .map(|p| (PathBuf::from(p), "coreml"))
            .or_else(|_| {
                let base_path = std::env::var("COREML_MODELS_PATH")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| PathBuf::from("/Users/darianrosebrook/Desktop/Projects/agent-agency/models/coreml"));
                
                // Try embeddinggemma in various formats (priority order)
                // 1. ONNX format (preferred for ANE acceleration via CoreMLExecutionProvider)
                let gemma_onnx = base_path.join("embeddinggemma.onnx");
                if gemma_onnx.exists() {
                    tracing::info!("Found embeddinggemma.onnx - using ONNX Runtime with ANE acceleration");
                    return Ok((gemma_onnx, "onnx"));
                }
                
                // 2. ML Package format (ML Program)
                let gemma_mlpackage = base_path.join("embeddinggemma.mlpackage");
                if gemma_mlpackage.exists() {
                    return Ok((gemma_mlpackage, "coreml"));
                }
                
                // 3. ML Model format (legacy Neural Network)
                let gemma_mlmodel = base_path.join("embeddinggemma.mlmodel");
                if gemma_mlmodel.exists() {
                    return Ok((gemma_mlmodel, "coreml"));
                }
                
                // 4. GGUF format (from Ollama, may need conversion)
                let gemma_gguf = base_path.join("embeddinggemma.gguf");
                if gemma_gguf.exists() {
                    tracing::info!("Found embeddinggemma.gguf - may need conversion to .onnx/.mlmodel for inference");
                    return Ok((gemma_gguf, "coreml"));
                }
                
                Err(())
            });
        
        match model_result {
            Ok((path, model_type)) => {
                // Try to load HuggingFace tokenizer from saved location
                let tokenizer_path = path.parent()
                    .map(|p| p.join("embeddinggemma_tokenizer").join("tokenizer.json"));
                
                let tokenizer: Arc<dyn crate::embedding::Tokenizer> = if let Some(ref tokenizer_path) = tokenizer_path {
                    if tokenizer_path.exists() {
                        match crate::embedding::tokenization::HfTokenizer::from_file(tokenizer_path) {
                            Ok(hf_tokenizer) => {
                                tracing::info!("Loaded HuggingFace tokenizer from {}", tokenizer_path.display());
                                Arc::new(hf_tokenizer)
                            }
                            Err(e) => {
                                tracing::warn!("Failed to load HuggingFace tokenizer: {}, using SimpleTokenizer", e);
                                Arc::new(crate::embedding::tokenization::SimpleTokenizer::new())
                            }
                        }
                    } else {
                        tracing::info!("Tokenizer not found at {}, using SimpleTokenizer", tokenizer_path.display());
                        Arc::new(crate::embedding::tokenization::SimpleTokenizer::new())
                    }
                } else {
                    tracing::info!("Using SimpleTokenizer (tokenizer path not available)");
                    Arc::new(crate::embedding::tokenization::SimpleTokenizer::new())
                };
                
                // Use ONNX Runtime or CoreML service based on model type
                match model_type {
                    "onnx" => Self::create_onnx_service(path, model_name, config, tokenizer).await,
                    "coreml" => Self::create_coreml_service(path, model_name, config, tokenizer).await,
                    _ => {
                        tracing::warn!("Unknown model type: {}, falling back to DummyEmbeddingProvider", model_type);
                        Self::create_dummy_service(config)
                    }
                }
            }
            Err(_) => {
                tracing::info!("No embedding model found, using DummyEmbeddingProvider");
                Self::create_dummy_service(config)
            }
        }
    }

    /// Create embedding service using DummyEmbeddingProvider
    /// 
    /// PLACEHOLDER: Will be replaced with CoreML-based embeddings
    /// TODO: Implement CoreML embedding provider (see todo-1762001962177-gg7fpzx98)
    pub fn create_dummy_service(config: EmbeddingConfig) -> Box<dyn EmbeddingService> {
        let provider = Arc::new(DummyEmbeddingProvider::new(config.dimension));
        let service = EmbeddingServiceImpl::new(provider, config);
        Box::new(service)
    }

    /// Create service with custom provider
    pub fn create_with_provider(
        provider: Arc<dyn EmbeddingProvider>,
        config: EmbeddingConfig,
    ) -> Box<dyn EmbeddingService> {
        let service = EmbeddingServiceImpl::new(provider, config);
        Box::new(service)
    }
}
