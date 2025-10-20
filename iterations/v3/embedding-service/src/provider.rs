//! Embedding provider trait and implementations

use crate::types::*;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use tracing::{debug, info, warn};
use std::hash::Hasher;
use std::path::PathBuf;
use std::sync::Arc;
use std::collections::HashMap;
use ndarray::s;

/// Trait for embedding providers
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate embeddings for a batch of texts
    async fn generate_embeddings(&self, texts: &[String]) -> Result<Vec<EmbeddingVector>>;

    /// Get the dimension of embeddings produced by this provider
    fn dimension(&self) -> usize;

    /// Get the model name
    fn model_name(&self) -> &str;

    /// Check if the provider is available
    async fn health_check(&self) -> Result<bool>;
}

/// Ollama embedding provider using embeddinggemma
pub struct OllamaEmbeddingProvider {
    client: reqwest::Client,
    base_url: String,
    model_name: String,
    dimension: usize,
    timeout: std::time::Duration,
}

impl OllamaEmbeddingProvider {
    pub fn new(config: &EmbeddingConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(config.timeout_ms))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: config.ollama_url.clone(),
            model_name: config.model_name.clone(),
            dimension: config.dimension,
            timeout: std::time::Duration::from_millis(config.timeout_ms),
        }
    }
}

#[async_trait]
impl EmbeddingProvider for OllamaEmbeddingProvider {
    async fn generate_embeddings(&self, texts: &[String]) -> Result<Vec<EmbeddingVector>> {
        let mut embeddings = Vec::new();

        for text in texts {
            let request_body = serde_json::json!({
                "model": self.model_name,
                "prompt": text
            });

            let response = self
                .client
                .post(&format!("{}/api/embeddings", self.base_url))
                .json(&request_body)
                .send()
                .await?;

            if !response.status().is_success() {
                return Err(anyhow::anyhow!("Ollama API error: {}", response.status()));
            }

            let response_json: serde_json::Value = response.json().await?;
            let embedding_data = response_json["embedding"]
                .as_array()
                .ok_or_else(|| anyhow::anyhow!("Invalid embedding response format"))?;

            let embedding: EmbeddingVector = embedding_data
                .iter()
                .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                .collect();

            if embedding.len() != self.dimension {
                return Err(anyhow::anyhow!(
                    "Expected embedding dimension {}, got {}",
                    self.dimension,
                    embedding.len()
                ));
            }

            embeddings.push(embedding);
        }

        Ok(embeddings)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn model_name(&self) -> &str {
        &self.model_name
    }

    async fn health_check(&self) -> Result<bool> {
        let response = self
            .client
            .get(&format!("{}/api/tags", self.base_url))
            .send()
            .await?;

        Ok(response.status().is_success())
    }
}

/// Dummy provider for testing
pub struct DummyEmbeddingProvider {
    dimension: usize,
    model_name: String,
}

impl DummyEmbeddingProvider {
    pub fn new(dimension: usize) -> Self {
        Self {
            dimension,
            model_name: "dummy".to_string(),
        }
    }
}

#[async_trait]
impl EmbeddingProvider for DummyEmbeddingProvider {
    async fn generate_embeddings(&self, texts: &[String]) -> Result<Vec<EmbeddingVector>> {
        // Generate deterministic dummy embeddings based on text hash
        let embeddings = texts
            .iter()
            .map(|text| {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                std::hash::Hash::hash(text, &mut hasher);
                let hash = hasher.finish();

                // Generate deterministic vector from hash
                (0..self.dimension)
                    .map(|i| {
                        let seed = hash.wrapping_add(i as u64);
                        let normalized = (seed % 1000) as f32 / 1000.0;
                        normalized * 2.0 - 1.0 // Scale to [-1, 1]
                    })
                    .collect()
            })
            .collect();

        Ok(embeddings)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn model_name(&self) -> &str {
        &self.model_name
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }
}

// Temporarily disabled due to ORT API complexity
// TODO: Re-enable when ORT API stabilizes
/*
/// ONNX embedding provider for local model inference
pub struct OnnxEmbeddingProvider {
    session: Arc<Session>,
    tokenizer: Arc<dyn crate::tokenization::Tokenizer>,
    dimension: usize,
    model_name: String,
    max_length: usize,
}

impl OnnxEmbeddingProvider {
    /// Create a new ONNX embedding provider
    pub async fn new(
        model_path: PathBuf,
        tokenizer: Arc<dyn crate::tokenization::Tokenizer>,
        dimension: usize,
        model_name: String,
        max_length: usize,
    ) -> Result<Self> {
        // Load ONNX model
        let session = Session::builder()?
            .with_execution_providers([
                ExecutionProvider::CPU(Default::default()),
            ])?
            .commit_from_file(model_path)?;

        Ok(Self {
            session: Arc::new(session),
            tokenizer,
            dimension,
            model_name,
            max_length,
        })
    }
}

*/

/// Stub SafeTensors embedding provider (placeholder for future implementation)
pub struct SafeTensorsEmbeddingProvider {
    dimension: usize,
    model_name: String,
}

/// ONNX embedding provider (placeholder - ONNX integration disabled for compatibility)
pub struct OnnxEmbeddingProvider {
    tokenizer: Arc<dyn crate::tokenization::Tokenizer>,
    dimension: usize,
    max_length: usize,
}

impl SafeTensorsEmbeddingProvider {
    /// Create a new SafeTensors embedding provider (stub)
    pub async fn new(
        _model_path: PathBuf,
        _tokenizer: Arc<dyn crate::tokenization::Tokenizer>,
        dimension: usize,
        model_name: String,
        _max_length: usize,
    ) -> Result<Self> {
        // TODO: Implement SafeTensors loading when Candle dependencies are resolved
        Ok(Self {
            dimension,
            model_name,
        })
    }
}

impl OnnxEmbeddingProvider {
    /// Create a new ONNX embedding provider (stub implementation)
    pub async fn new(
        _model_path: PathBuf,
        tokenizer: Arc<dyn crate::tokenization::Tokenizer>,
        dimension: usize,
        _model_name: String,
        max_length: usize,
    ) -> Result<Self> {
        // TODO: Implement ONNX model loading when API stabilizes
        warn!("ONNX embedding provider using stub implementation - actual ONNX integration disabled");

        Ok(Self {
            tokenizer,
            dimension,
            max_length,
        })
    }

    /// Generate embeddings using stub implementation
    async fn generate_embeddings_stub(&self, texts: &[String]) -> Result<Vec<EmbeddingVector>> {
        warn!("OnnxEmbeddingProvider using stub implementation - no actual ONNX inference");

        // Generate deterministic mock embeddings based on text content
        let mut embeddings = Vec::with_capacity(texts.len());

        for text in texts {
            // Create a simple hash-based deterministic embedding
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            std::hash::Hash::hash(&text, &mut hasher);
            let hash = std::hash::Hasher::finish(&hasher) as u64;

            // Generate pseudo-random but deterministic values
            let mut embedding = Vec::with_capacity(self.dimension);
            for i in 0..self.dimension {
                let value = ((hash.wrapping_mul(31).wrapping_add(i as u64)) % 10000) as f32 / 5000.0 - 1.0;
                embedding.push(value);
            }

            // Normalize to unit vector (approximate)
            let norm = (embedding.iter().map(|x| x * x).sum::<f32>()).sqrt();
            for val in &mut embedding {
                *val /= norm;
            }

            embeddings.push(embedding);
        }

        Ok(embeddings)
    }
    }

#[async_trait]
impl EmbeddingProvider for SafeTensorsEmbeddingProvider {
    async fn generate_embeddings(&self, texts: &[String]) -> Result<Vec<EmbeddingVector>> {
        // Placeholder implementation - generate deterministic embeddings based on text hash
        let embeddings = texts
            .iter()
            .map(|text| {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                std::hash::Hash::hash(text, &mut hasher);
                let hash = hasher.finish();

                (0..self.dimension)
                    .map(|i| {
                        let seed = hash.wrapping_add(i as u64);
                        let normalized = (seed % 1000) as f32 / 1000.0;
                        normalized * 2.0 - 1.0 // Scale to [-1, 1]
                    })
                    .collect::<EmbeddingVector>()
            })
            .collect();

        Ok(embeddings)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn model_name(&self) -> &str {
        &self.model_name
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true) // Stub always reports healthy
    }
}

#[async_trait]
impl EmbeddingProvider for OnnxEmbeddingProvider {
    async fn generate_embeddings(&self, texts: &[String]) -> Result<Vec<EmbeddingVector>> {
        self.generate_embeddings_stub(texts).await
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn model_name(&self) -> &str {
        "onnx-embedding-model-stub"
    }

    async fn health_check(&self) -> Result<bool> {
        // Stub implementation always returns healthy
        warn!("ONNX embedding provider health check using stub - actual ONNX integration disabled");
        Ok(true)
    }
}
