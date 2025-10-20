//! Embedding provider trait and implementations

use crate::types::*;
use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use ort::session::{Session, SessionOutputs};
use ort::execution_providers::ExecutionProvider;
use ort::session::builder::GraphOptimizationLevel;
use ort::tensor::TensorElementDataType;
use ort::value::{Tensor, Value};
use tracing::warn;
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

*/

/// Stub SafeTensors embedding provider (placeholder for future implementation)
pub struct SafeTensorsEmbeddingProvider {
    dimension: usize,
    model_name: String,
}

/// ONNX embedding provider with actual model loading and inference
pub struct OnnxEmbeddingProvider {
    session: Session,
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
    /// Create a new ONNX embedding provider with actual model loading
    pub async fn new(
        model_path: PathBuf,
        tokenizer: Arc<dyn crate::tokenization::Tokenizer>,
        dimension: usize,
        max_length: usize,
    ) -> Result<Self> {
        use std::fs;

        // Read the ONNX model file
        let model_data = fs::read(&model_path)
            .with_context(|| format!("Failed to read ONNX model file: {}", model_path.display()))?;

        // Create ONNX session with optimized settings for embeddings
        let session = Session::builder()?
            .with_execution_providers([
                // Try CPU (CUDA not available in this version)
                ExecutionProvider::CPU(Default::default()),
            ])?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(4)? // Optimize for embedding generation
            .commit_from_memory(&model_data)
            .with_context(|| format!("Failed to create ONNX session for: {}", model_path.display()))?;

        // Validate that this is an embedding model by checking input/output shapes
        Self::validate_embedding_model(&session, dimension)?;

        Ok(Self {
            session,
            tokenizer,
            dimension,
            max_length,
        })
    }

    /// Validate that the ONNX model is suitable for embedding generation
    fn validate_embedding_model(session: &Session, expected_dimension: usize) -> Result<()> {
        // Check that we have inputs
        if session.inputs.is_empty() {
            bail!("ONNX model has no inputs");
        }

        // Check that we have outputs
        if session.outputs.is_empty() {
            bail!("ONNX model has no outputs");
        }

        // For embedding models, typically expect:
        // - Input: token_ids (int64 or int32), attention_mask (optional)
        // - Output: embeddings with shape [batch_size, seq_len, hidden_size] or [batch_size, hidden_size]

        let output = &session.outputs[0];
        let output_shape = output.dimensions().collect::<Vec<_>>();

        // Validate output dimension matches expected
        if let Some(&dim) = output_shape.last() {
            if dim != -1 && dim as usize != expected_dimension {
                warn!("ONNX model output dimension {} doesn't match expected {}", dim, expected_dimension);
            }
        }

        Ok(())
    }

    /// Extract embeddings from model output
    fn extract_embeddings_from_output(&self, outputs: &SessionOutputs) -> Result<Vec<f32>> {
        // Get the first output (typically the embeddings)
        let output_tensor = outputs
            .get(0)
            .context("No outputs from ONNX model")?;

        // Convert to f32 vector
        match output_tensor.dtype() {
            TensorElementDataType::Float32 => {
                let data: Vec<f32> = output_tensor.try_extract_tensor()?;
                Ok(data)
            }
            _ => bail!("Unsupported output tensor type for embeddings"),
        }
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
        let mut embeddings = Vec::with_capacity(texts.len());

        for text in texts {
            // Tokenize the input text
            let tokens = self.tokenizer.encode(text).await
                .with_context(|| format!("Failed to tokenize text: {}", text))?;

            // Truncate if too long
            let tokens = if tokens.len() > self.max_length {
                &tokens[..self.max_length]
            } else {
                &tokens
            };

            // Convert tokens to i64 (common input type for ONNX models)
            let token_ids: Vec<i64> = tokens.iter().map(|&t| t as i64).collect();

            // Create attention mask (all 1s for now)
            let attention_mask: Vec<i64> = vec![1; token_ids.len()];

            // Create input tensors
            let input_ids_tensor = Tensor::from_array(([1, token_ids.len() as i64], token_ids.clone()))?;
            let attention_mask_tensor = Tensor::from_array(([1, attention_mask.len() as i64], attention_mask.clone()))?;

            // Create inputs - assume first input is token_ids, second is attention_mask if available
            let inputs: Vec<Value> = if self.session.inputs.len() > 1 {
                vec![
                    Value::from(input_ids_tensor),
                    Value::from(attention_mask_tensor),
                ]
            } else {
                vec![Value::from(input_ids_tensor)]
            };

            // Run inference
            let outputs = self.session.run(inputs)
                .with_context(|| format!("Failed to run ONNX inference for text: {}", text))?;

            // Extract embeddings from output
            let embedding_values = self.extract_embeddings_from_output(&outputs)?;

            // For sequence embeddings, we typically take the mean or the [CLS] token embedding
            // For now, we'll take the first embedding from the sequence
            let final_embedding = if embedding_values.len() >= self.dimension {
                embedding_values[..self.dimension].to_vec()
            } else {
                // Pad if necessary
                let mut padded = embedding_values;
                padded.resize(self.dimension, 0.0);
                padded
            };

            embeddings.push(final_embedding);
        }

        Ok(embeddings)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn model_name(&self) -> &str {
        "onnx-embedding-model" // Could be made configurable
    }

    async fn health_check(&self) -> Result<bool> {
        // Try a simple inference to check if the model is working
        match self.generate_embeddings(&["test".to_string()]).await {
            Ok(_) => Ok(true),
            Err(e) => {
                warn!("ONNX model health check failed: {}", e);
                Ok(false)
            }
        }
    }
}
