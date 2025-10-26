//! Tokenization utilities for embedding providers

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

/// Trait for text tokenizers
#[async_trait]
pub trait Tokenizer: Send + Sync {
    /// Encode text into token IDs
    async fn encode(&self, text: &str) -> Result<Vec<u32>>;

    /// Decode token IDs back to text
    async fn decode(&self, tokens: &[u32]) -> Result<String>;
}

/// Simple word-based tokenizer for testing
pub struct SimpleTokenizer;

impl SimpleTokenizer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tokenizer for SimpleTokenizer {
    async fn encode(&self, text: &str) -> Result<Vec<u32>> {
        // Simple word-based tokenization for testing
        let tokens: Vec<u32> = text
            .split_whitespace()
            .enumerate()
            .map(|(i, _)| i as u32 + 1) // Simple token IDs starting from 1
            .collect();

        Ok(tokens)
    }

    async fn decode(&self, _tokens: &[u32]) -> Result<String> {
        // Simple decoding for testing
        Ok("decoded text".to_string())
    }
}

/// HuggingFace tokenizer implementation
pub struct HfTokenizer {
    tokenizer: Arc<tokenizers::Tokenizer>,
}

impl HfTokenizer {
    /// Create a new HuggingFace tokenizer from a model identifier
    pub async fn from_pretrained(model_id: &str) -> Result<Self> {
        // Download tokenizer files from HuggingFace Hub
        let tokenizer_path = hf_hub::api::sync::Api::new()?
            .model(model_id.to_string())
            .get("tokenizer.json")?;

        // Load the tokenizer
        let tokenizer = tokenizers::Tokenizer::from_file(tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?;

        Ok(Self {
            tokenizer: Arc::new(tokenizer),
        })
    }

    /// Create a new HuggingFace tokenizer from a local file
    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        let tokenizer = tokenizers::Tokenizer::from_file(path)
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer from file: {}", e))?;

        Ok(Self {
            tokenizer: Arc::new(tokenizer),
        })
    }
}

#[async_trait]
impl Tokenizer for HfTokenizer {
    async fn encode(&self, text: &str) -> Result<Vec<u32>> {
        let encoding = self.tokenizer.encode(text, true)
            .map_err(|e| anyhow::anyhow!("Failed to encode text: {}", e))?;

        Ok(encoding.get_ids().to_vec())
    }

    async fn decode(&self, tokens: &[u32]) -> Result<String> {
        let decoded = self.tokenizer.decode(tokens, true)
            .map_err(|e| anyhow::anyhow!("Failed to decode tokens: {}", e))?;

        Ok(decoded)
    }
}


