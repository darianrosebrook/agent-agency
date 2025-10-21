//! Tokenization infrastructure for Apple Silicon models
//!
//! Provides pluggable tokenizer implementations for converting text to tokens
//! and tokens back to text. Supports HuggingFace tokenizers and other formats.

use std::path::Path;
use std::sync::Arc;
use anyhow::{anyhow, Result};
use async_trait::async_trait;

/// Pluggable tokenizer trait for text processing
#[async_trait]
pub trait Tokenizer: Send + Sync {
    /// Encode text into tokens
    async fn encode(&self, text: &str) -> Result<Vec<u32>>;

    /// Decode tokens back to text
    async fn decode(&self, tokens: &[u32]) -> Result<String>;

    /// Get vocabulary size
    fn vocab_size(&self) -> usize;

    /// Get special token IDs
    fn special_tokens(&self) -> SpecialTokens;
}

/// Special token configuration
#[derive(Debug, Clone)]
pub struct SpecialTokens {
    pub pad_token: Option<u32>,
    pub eos_token: Option<u32>,
    pub bos_token: Option<u32>,
    pub unk_token: Option<u32>,
    pub mask_token: Option<u32>,
}

/// HuggingFace tokenizer implementation
pub struct HfTokenizer {
    tokenizer: Arc<tokenizers::tokenizer::Tokenizer>,
    special_tokens: SpecialTokens,
}

impl HfTokenizer {
    /// Load tokenizer from HuggingFace model directory
    pub async fn from_pretrained(model_path: &Path) -> Result<Self> {
        // Try to load tokenizer.json first
        let tokenizer_path = model_path.join("tokenizer.json");
        if tokenizer_path.exists() {
            let tokenizer = tokenizers::tokenizer::Tokenizer::from_file(tokenizer_path)
                .map_err(|e| anyhow!("Failed to load tokenizer: {}", e))?;

            let special_tokens = Self::extract_special_tokens(&tokenizer)?;

            Ok(Self {
                tokenizer: Arc::new(tokenizer),
                special_tokens,
            })
        } else {
            return Err(anyhow!("No tokenizer.json found in {}", model_path.display()));
        }
    }

    /// Create tokenizer from in-memory config
    pub fn from_config(config: serde_json::Value) -> Result<Self> {
        let tokenizer = tokenizers::tokenizer::Tokenizer::from_str(&config.to_string())
            .map_err(|e| anyhow!("Failed to create tokenizer from config: {}", e))?;

        let special_tokens = Self::extract_special_tokens(&tokenizer)?;

        Ok(Self {
            tokenizer: Arc::new(tokenizer),
            special_tokens,
        })
    }

    /// Extract special token IDs from tokenizer
    fn extract_special_tokens(tokenizer: &tokenizers::tokenizer::Tokenizer) -> Result<SpecialTokens> {
        let vocab = tokenizer.get_vocab(false);

        let pad_token = vocab.get("[PAD]").or_else(|| vocab.get("<pad>")).copied();
        let eos_token = vocab.get("[SEP]").or_else(|| vocab.get("</s>")).copied();
        let bos_token = vocab.get("[CLS]").or_else(|| vocab.get("<s>")).copied();
        let unk_token = vocab.get("[UNK]").or_else(|| vocab.get("<unk>")).copied();
        let mask_token = vocab.get("[MASK]").copied();

        Ok(SpecialTokens {
            pad_token,
            eos_token,
            bos_token,
            unk_token,
            mask_token,
        })
    }
}

#[async_trait]
impl Tokenizer for HfTokenizer {
    async fn encode(&self, text: &str) -> Result<Vec<u32>> {
        let encoding = self.tokenizer
            .encode(text, true)
            .map_err(|e| anyhow!("Failed to encode text: {}", e))?;

        Ok(encoding.get_ids().to_vec())
    }

    async fn decode(&self, tokens: &[u32]) -> Result<String> {
        let decoded = self.tokenizer
            .decode(tokens, true)
            .map_err(|e| anyhow!("Failed to decode tokens: {}", e))?;

        Ok(decoded)
    }

    fn vocab_size(&self) -> usize {
        self.tokenizer.get_vocab_size(true)
    }

    fn special_tokens(&self) -> SpecialTokens {
        self.special_tokens.clone()
    }
}

/// Simple word-level tokenizer for fallback
pub struct WordTokenizer {
    vocab_size: usize,
}

impl WordTokenizer {
    pub fn new() -> Self {
        Self { vocab_size: 50000 } // Reasonable default
    }
}

impl Default for WordTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tokenizer for WordTokenizer {
    async fn encode(&self, text: &str) -> Result<Vec<u32>> {
        // Simple word splitting - not suitable for production ML models
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut tokens = Vec::new();

        for word in words {
            // Hash word to get consistent token ID
            let token_id = (word.chars().map(|c| c as u32).sum::<u32>() % (self.vocab_size as u32 - 1000)) + 1000;
            tokens.push(token_id);
        }

        Ok(tokens)
    }

    async fn decode(&self, tokens: &[u32]) -> Result<String> {
        // Word tokenizer is lossy - we can't decode back to original text
        Ok(format!("<decoded {} tokens>", tokens.len()))
    }

    fn vocab_size(&self) -> usize {
        self.vocab_size
    }

    fn special_tokens(&self) -> SpecialTokens {
        SpecialTokens {
            pad_token: Some(0),
            eos_token: Some(1),
            bos_token: Some(2),
            unk_token: Some(3),
            mask_token: None,
        }
    }
}

/// Tokenizer configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TokenizerConfig {
    pub tokenizer_type: TokenizerType,
    pub model_path: Option<String>,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum TokenizerType {
    HuggingFace,
    WordLevel,
}

/// Create tokenizer from configuration
pub async fn create_tokenizer(config: &TokenizerConfig) -> Result<Box<dyn Tokenizer>> {
    match &config.tokenizer_type {
        TokenizerType::HuggingFace => {
            if let Some(model_path) = &config.model_path {
                let path = Path::new(model_path);
                let tokenizer = HfTokenizer::from_pretrained(path).await?;
                Ok(Box::new(tokenizer))
            } else if let Some(config_json) = &config.config {
                let tokenizer = HfTokenizer::from_config(config_json.clone())?;
                Ok(Box::new(tokenizer))
            } else {
                Err(anyhow!("HuggingFace tokenizer requires model_path or config"))
            }
        }
        TokenizerType::WordLevel => {
            Ok(Box::new(WordTokenizer::new()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_word_tokenizer_basic() {
        let tokenizer = WordTokenizer::new();

        let text = "hello world";
        let tokens = tokenizer.encode(text).await.unwrap();
        assert!(!tokens.is_empty());

        let decoded = tokenizer.decode(&tokens).await.unwrap();
        assert!(decoded.contains("decoded"));
    }

    #[test]
    fn test_special_tokens() {
        let tokenizer = WordTokenizer::new();
        let special = tokenizer.special_tokens();

        assert_eq!(special.pad_token, Some(0));
        assert_eq!(special.eos_token, Some(1));
        assert_eq!(special.bos_token, Some(2));
        assert_eq!(special.unk_token, Some(3));
        assert_eq!(special.mask_token, None);
    }
}
