//! Model loading utilities for embedding providers

use crate::types::*;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;

/// Trait for embedding models
#[async_trait]
pub trait EmbeddingModel: Send + Sync {
    /// Forward pass to generate embeddings
    async fn forward(&self, tokens: &[u32]) -> Result<EmbeddingVector>;
}

/// SafeTensors model implementation
pub struct SafeTensorsModel {
    tensors: HashMap<String, safetensors::tensor::TensorView<'static>>,
    dimension: usize,
    vocab_size: usize,
}

impl SafeTensorsModel {
    /// Load a model from SafeTensors files
    pub async fn load_from_path(model_path: &Path) -> Result<Self> {
        // Load the SafeTensors file
        let data = tokio::fs::read(model_path).await?;
        let tensors = safetensors::SafeTensors::deserialize(&data)?;

        // Infer dimension from embeddings tensor
        let embeddings_tensor = tensors.tensor("embeddings")
            .or_else(|_| tensors.tensor("embed_tokens"))
            .or_else(|_| tensors.tensor("model.embed_tokens"))?;

        let shape = embeddings_tensor.shape();
        if shape.len() != 2 {
            return Err(anyhow::anyhow!("Expected 2D embeddings tensor, got shape {:?}", shape));
        }

        let vocab_size = shape[0];
        let dimension = shape[1];

        // Convert SafeTensors to owned tensor map
        let tensor_map = tensors.tensors().iter().map(|(name, tensor)| {
            (name.clone(), tensor.clone())
        }).collect();

        Ok(Self {
            tensors: tensor_map,
            dimension: dimension as usize,
            vocab_size: vocab_size as usize,
        })
    }

    /// Load a model from HuggingFace Hub
    pub async fn from_pretrained(model_id: &str) -> Result<Self> {
        // Download model files from HuggingFace Hub
        let api = hf_hub::api::sync::Api::new()?;
        let model_path = api.model(model_id.to_string()).get("model.safetensors")?;

        Self::load_from_path(&model_path).await
    }
}

#[async_trait]
impl EmbeddingModel for SafeTensorsModel {
    async fn forward(&self, tokens: &[u32]) -> Result<EmbeddingVector> {
        // Simple mean pooling of token embeddings
        // In a real implementation, this would run the full transformer model
        let embeddings_tensor = self.tensors.get("embeddings")
            .or_else(|| self.tensors.get("embed_tokens"))
            .or_else(|| self.tensors.get("model.embed_tokens"))
            .ok_or_else(|| anyhow::anyhow!("No embeddings tensor found"))?;

        let data = embeddings_tensor.data();
        let shape = embeddings_tensor.shape();

        if shape.len() != 2 {
            return Err(anyhow::anyhow!("Expected 2D embeddings tensor"));
        }

        let vocab_size = shape[0] as usize;
        let embedding_dim = shape[1] as usize;

        // Simple averaging of token embeddings
        let mut result = vec![0.0f32; embedding_dim];

        for &token_id in tokens {
            if token_id as usize >= vocab_size {
                continue; // Skip out-of-vocab tokens
            }

            let token_offset = (token_id as usize * embedding_dim) * std::mem::size_of::<f32>();

            if token_offset + embedding_dim * std::mem::size_of::<f32>() > data.len() {
                continue; // Skip if out of bounds
            }

            // Add token embedding to result (mean pooling)
            for i in 0..embedding_dim {
                let offset = token_offset + i * std::mem::size_of::<f32>();
                let bytes = &data[offset..offset + std::mem::size_of::<f32>()];
                let value = f32::from_le_bytes(bytes.try_into().unwrap_or([0, 0, 0, 0]));
                result[i] += value;
            }
        }

        // Average the embeddings
        let num_tokens = tokens.len().max(1) as f32;
        for value in &mut result {
            *value /= num_tokens;
        }

        Ok(result)
    }
}

impl EmbeddingModel for SafeTensorsModel {
    async fn forward(&self, tokens: &[u32]) -> Result<EmbeddingVector> {
        SafeTensorsModel::forward(self, tokens).await
    }
}

/// Placeholder model implementation for fallback
pub struct PlaceholderModel {
    dimension: usize,
}

impl PlaceholderModel {
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

#[async_trait]
impl EmbeddingModel for PlaceholderModel {
    async fn forward(&self, _tokens: &[u32]) -> Result<EmbeddingVector> {
        // Generate a simple placeholder embedding
        // Used when SafeTensors loading fails
        let embedding: EmbeddingVector = (0..self.dimension)
            .map(|i| (i as f32 * 0.1).sin())
            .collect();

        Ok(embedding)
    }
}


