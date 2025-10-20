//! Embedding provider trait and implementations

use crate::types::*;
use anyhow::{anyhow, Context, Result, bail};
use async_trait::async_trait;
use tracing::{debug, info, warn};
use std::hash::Hasher;
use std::path::PathBuf;
use std::sync::Arc;
use std::collections::HashMap;
use ndarray::s;
use ort::Session;

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

/// ONNX embedding provider for local model inference
pub struct OnnxEmbeddingProvider {
    session: Arc<Session>,
    tokenizer: Arc<dyn crate::tokenization::Tokenizer>,
    dimension: usize,
    model_name: String,
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
        model_name: String,
        max_length: usize,
    ) -> Result<Self> {
        info!("Loading ONNX model from: {}", model_path.display());

        // Validate model file exists
        if !model_path.exists() {
            return Err(anyhow!("ONNX model file not found: {}", model_path.display()));
        }

        // Create ONNX session with optimized settings
        let session = Session::builder()?
            .with_execution_providers([
                // Prefer CPU execution for embeddings (can be extended to GPU later)
                ExecutionProvider::CPU(Default::default()),
            ])?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(num_cpus::get().min(8) as i16)? // Limit threads for stability
            .commit_from_file(model_path)?;

        // Validate model inputs and outputs
        Self::validate_model_inputs(&session)?;
        Self::validate_model_outputs(&session)?;

        info!("ONNX model loaded successfully: {}", model_name);

        Ok(Self {
            session: Arc::new(session),
            tokenizer,
            dimension,
            model_name,
            max_length,
        })
    }

    /// Validate model input requirements
    fn validate_model_inputs(session: &Session) -> Result<()> {
        let inputs = session.inputs()?;
        if inputs.is_empty() {
            return Err(anyhow!("ONNX model has no inputs"));
        }

        // Check for expected input names (common in embedding models)
        let input_names: Vec<&str> = inputs.iter().map(|i| i.name()).collect();
        let has_input_ids = input_names.iter().any(|name| name.contains("input_ids"));
        let has_attention_mask = input_names.iter().any(|name| name.contains("attention_mask"));

        if !has_input_ids {
            warn!("Model may not have expected 'input_ids' input. Available inputs: {:?}", input_names);
        }
        if !has_attention_mask {
            debug!("Model does not have 'attention_mask' input - this is normal for some models");
        }

        Ok(())
    }

    /// Validate model output requirements
    fn validate_model_outputs(session: &Session) -> Result<()> {
        let outputs = session.outputs()?;
        if outputs.is_empty() {
            return Err(anyhow!("ONNX model has no outputs"));
        }

        debug!("Model outputs: {:?}", outputs.iter().map(|o| o.name()).collect::<Vec<_>>());
        Ok(())
    }

    /// Prepare ONNX model inputs from tokenized data
    fn prepare_model_inputs(
        &self,
        input_ids: &[i64],
        attention_masks: &[i64],
        batch_size: usize,
    ) -> Result<Vec<(&str, Value)>> {
        use ndarray::Array;

        // Reshape input_ids to [batch_size, max_length]
        let input_ids_shape = [batch_size, self.max_length];
        let input_ids_array = Array::from_shape_vec(input_ids_shape, input_ids.to_vec())
            .map_err(|e| anyhow!("Failed to reshape input_ids: {}", e))?;

        // Reshape attention_masks to [batch_size, max_length]
        let attention_mask_shape = [batch_size, self.max_length];
        let attention_mask_array = Array::from_shape_vec(attention_mask_shape, attention_masks.to_vec())
            .map_err(|e| anyhow!("Failed to reshape attention_mask: {}", e))?;

        // Create ONNX tensors
        let input_ids_tensor = Value::from_array(input_ids_array)?;
        let attention_mask_tensor = Value::from_array(attention_mask_array)?;

        // Return as vector of (name, value) pairs
        Ok(vec![
            ("input_ids", input_ids_tensor),
            ("attention_mask", attention_mask_tensor),
        ])
    }

    /// Extract embeddings from ONNX model outputs
    fn extract_embeddings_from_outputs(
        &self,
        outputs: SessionOutputs,
        batch_size: usize,
    ) -> Result<Vec<EmbeddingVector>> {
        // Get the first output (assuming it's the embeddings)
        let output_name = outputs.keys().next()
            .ok_or_else(|| anyhow!("No outputs from ONNX model"))?;
        let output_tensor = outputs.get(output_name)
            .ok_or_else(|| anyhow!("Failed to get output tensor: {}", output_name))?;

        // Extract the tensor data
        let tensor_data = output_tensor.try_extract_tensor::<f32>()?;

        // Get the tensor shape to understand the output format
        let shape = tensor_data.shape();

        debug!("Output tensor shape: {:?}", shape);

        let embeddings = match shape.as_slice() {
            // [batch_size, seq_len, hidden_dim] - take mean pooling over sequence dimension
            [batch, seq_len, hidden] if *batch == batch_size && *seq_len == self.max_length => {
                self.extract_pooled_embeddings(&tensor_data.view(), *batch, *seq_len, *hidden)?
            }
            // [batch_size, hidden_dim] - already pooled
            [batch, hidden] if *batch == batch_size => {
                self.extract_direct_embeddings(&tensor_data.view(), *batch, *hidden)?
            }
            // Unexpected shape
            _ => {
                return Err(anyhow!("Unexpected output tensor shape: {:?}. Expected [batch_size, seq_len, hidden_dim] or [batch_size, hidden_dim]", shape));
            }
        };

        // Validate embedding dimensions
        for embedding in &embeddings {
            if embedding.len() != self.dimension {
                return Err(anyhow!("Embedding dimension mismatch: expected {}, got {}", self.dimension, embedding.len()));
            }
        }

        Ok(embeddings)
    }

    /// Extract embeddings with mean pooling over sequence dimension
    fn extract_pooled_embeddings(
        &self,
        tensor_view: &ndarray::ArrayView<f32, ndarray::IxDyn>,
        batch_size: usize,
        seq_len: usize,
        hidden_dim: usize,
    ) -> Result<Vec<EmbeddingVector>> {
        let mut embeddings = Vec::with_capacity(batch_size);

        for batch_idx in 0..batch_size {
            let mut pooled_embedding = vec![0.0f32; hidden_dim];
            let mut valid_tokens = 0;

            // Mean pool over sequence dimension (ignoring padding tokens)
            for seq_idx in 0..seq_len {
                let token_embedding = tensor_view.slice(s![batch_idx, seq_idx, ..]);
                let token_vec: Vec<f32> = token_embedding.to_vec();

                // Simple check for padding tokens (all zeros or very small values)
                let magnitude: f32 = token_vec.iter().map(|x| x * x).sum::<f32>().sqrt();
                if magnitude > 1e-6 { // Non-padding token
                    for (i, &val) in token_vec.iter().enumerate() {
                        pooled_embedding[i] += val;
                    }
                    valid_tokens += 1;
                }
            }

            // Average the pooled embeddings
            if valid_tokens > 0 {
                for val in &mut pooled_embedding {
                    *val /= valid_tokens as f32;
                }
            }

            embeddings.push(pooled_embedding);
        }

        Ok(embeddings)
    }

    /// Extract direct embeddings (already pooled)
    fn extract_direct_embeddings(
        &self,
        tensor_view: &ndarray::ArrayView<f32, ndarray::IxDyn>,
        batch_size: usize,
        hidden_dim: usize,
    ) -> Result<Vec<EmbeddingVector>> {
        let mut embeddings = Vec::with_capacity(batch_size);

        for batch_idx in 0..batch_size {
            let embedding_slice = tensor_view.slice(s![batch_idx, ..]);
            embeddings.push(embedding_slice.to_vec());
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
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        debug!("Generating embeddings for {} texts using ONNX model: {}", texts.len(), self.model_name);

        // Tokenize all texts
        let mut all_input_ids = Vec::new();
        let mut all_attention_masks = Vec::new();
        let mut sequence_lengths = Vec::new();

        for text in texts {
            // Tokenize the text
            let tokens = self.tokenizer.encode(text).await?;
            let mut input_ids = tokens.clone();

            // Apply max length constraint
            if input_ids.len() > self.max_length {
                input_ids.truncate(self.max_length);
            }

            // Create attention mask (1 for real tokens, 0 for padding)
            let attention_mask = vec![1i64; input_ids.len()];

            // Pad to max_length if necessary
            while input_ids.len() < self.max_length {
                input_ids.push(0); // Assuming 0 is padding token
            }
            while attention_mask.len() < self.max_length {
                all_attention_masks.push(0i64);
            }

            all_input_ids.extend(input_ids.into_iter().map(|x| x as i64));
            all_attention_masks.extend(attention_mask);
            sequence_lengths.push(text.len());
        }

        // Prepare ONNX inputs
        let inputs = self.prepare_model_inputs(&all_input_ids, &all_attention_masks, texts.len())?;

        // Create input map for session.run
        let mut input_map = HashMap::new();
        for (name, value) in inputs {
            input_map.insert(name.to_string(), value);
        }

        // Run inference
        let outputs = self.session.run(input_map)?;

        // Extract embeddings from outputs
        let embeddings = self.extract_embeddings_from_outputs(outputs, texts.len())?;

        debug!("Successfully generated {} embeddings", embeddings.len());
        Ok(embeddings)
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
