//! Embedding provider trait and implementations

use crate::types::*;
use anyhow::Result;
use async_trait::async_trait;
use std::hash::Hasher;
use std::path::PathBuf;
use std::sync::Arc;

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

#[async_trait]
impl EmbeddingProvider for OnnxEmbeddingProvider {
    async fn generate_embeddings(&self, texts: &[String]) -> Result<Vec<EmbeddingVector>> {
        let mut embeddings = Vec::new();

        for text in texts {
            // Tokenize input text
            let token_ids = self.tokenizer.encode(text).await?;
            let tokens: Vec<i64> = token_ids.into_iter().map(|id| id as i64).collect();

            // Create attention mask (all 1s for now)
            let attention_mask: Vec<i64> = vec![1; tokens.len()];

            // Pad or truncate to max_length
            let mut padded_tokens = tokens;
            let mut padded_mask = attention_mask;

            if padded_tokens.len() > self.max_length {
                padded_tokens.truncate(self.max_length);
                padded_mask.truncate(self.max_length);
            } else {
                let padding = self.max_length - padded_tokens.len();
                padded_tokens.extend(vec![0; padding]); // Assuming 0 is padding token
                padded_mask.extend(vec![0; padding]);
            }

            // Create ONNX inputs (placeholder - not used in current simplified implementation)
            let _input_ids = Value::from_array((
                vec![1, self.max_length],
                padded_tokens.clone().into_boxed_slice(),
            ))?;

            let _attention_mask = Value::from_array((
                vec![1, self.max_length],
                padded_mask.into_boxed_slice(),
            ))?;

            // Placeholder embedding - generate deterministic embedding based on tokens
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            std::hash::Hash::hash(&padded_tokens, &mut hasher);
            let hash = hasher.finish();

            let embedding: EmbeddingVector = (0..self.dimension)
                .map(|i| {
                    let seed = hash.wrapping_add(i as u64);
                    let normalized = (seed % 1000) as f32 / 1000.0;
                    normalized * 2.0 - 1.0 // Scale to [-1, 1]
                })
                .collect();

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
        // Try a simple inference to check if model is working
        let test_text = vec!["test".to_string()];
        match self.generate_embeddings(&test_text).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}
*/

/// Stub SafeTensors embedding provider (placeholder for future implementation)
pub struct SafeTensorsEmbeddingProvider {
    dimension: usize,
    model_name: String,
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
        Ok(true) // Stub always reports healthy
    }
}
