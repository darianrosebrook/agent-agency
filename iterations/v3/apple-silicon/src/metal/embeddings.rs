//! Embedding layers for GPU-accelerated token processing

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Comprehensive embedding layer with trained embeddings support
#[derive(Debug)]
pub struct EmbeddingLayer {
    /// Token embeddings with vocabulary management
    token_embeddings: Arc<RwLock<TokenEmbeddings>>,
    /// Position embeddings for sequence modeling
    position_embeddings: Arc<RwLock<PositionEmbeddings>>,
    /// Segment/type embeddings for multi-part inputs
    segment_embeddings: Arc<RwLock<SegmentEmbeddings>>,
    /// Embedding dimensionality
    embedding_dim: usize,
}

/// Embeddings trait for different embedding types
#[async_trait::async_trait]
pub trait Embeddings: Send + Sync {
    /// Get embedding dimension
    fn dim(&self) -> usize;

    /// Get token embedding
    async fn token(&self, token: &str) -> Result<Vec<f32>>;

    /// Get position embedding
    async fn position(&self, pos: usize) -> Result<Vec<f32>>;

    /// Get segment embedding
    async fn segment(&self, seg: usize) -> Result<Vec<f32>>;
}

impl EmbeddingLayer {
    /// Create new embedding layer with default configurations
    pub fn new() -> Result<Self> {
        let embedding_dim = 768; // Default BERT-like dimension

        Ok(Self {
            token_embeddings: Arc::new(RwLock::new(TokenEmbeddings::default())),
            position_embeddings: Arc::new(RwLock::new(PositionEmbeddings::default())),
            segment_embeddings: Arc::new(RwLock::new(SegmentEmbeddings::default())),
            embedding_dim,
        })
    }

    /// Get embedding dimension
    pub fn dim(&self) -> usize {
        self.embedding_dim
    }

    /// Create combined embeddings for tokens, positions, and segments
    pub async fn create_combined_embeddings(
        &self,
        tokens: &[String],
        positions: &[usize],
        segments: &[usize],
    ) -> Result<Vec<Vec<f32>>> {
        if tokens.len() != positions.len() || tokens.len() != segments.len() {
            return Err(anyhow::anyhow!(
                "Token, position, and segment arrays must have equal length"
            ));
        }

        let mut combined = Vec::with_capacity(tokens.len());

        for i in 0..tokens.len() {
            let token_emb = self.token_embeddings.read().await.token(&tokens[i]).await?;
            let pos_emb = self.position_embeddings.read().await.position(positions[i]).await?;
            let seg_emb = self.segment_embeddings.read().await.segment(segments[i]).await?;

            // Combine embeddings by addition
            let mut combined_emb = vec![0.0; self.embedding_dim];
            for j in 0..self.embedding_dim {
                combined_emb[j] = token_emb[j] + pos_emb[j] + seg_emb[j];
            }

            combined.push(combined_emb);
        }

        Ok(combined)
    }
}

#[async_trait::async_trait]
impl Embeddings for EmbeddingLayer {
    fn dim(&self) -> usize {
        self.embedding_dim
    }

    async fn token(&self, token: &str) -> Result<Vec<f32>> {
        self.token_embeddings.read().await.token(token).await
    }

    async fn position(&self, pos: usize) -> Result<Vec<f32>> {
        self.position_embeddings.read().await.position(pos).await
    }

    async fn segment(&self, seg: usize) -> Result<Vec<f32>> {
        self.segment_embeddings.read().await.segment(seg).await
    }
}

/// Token embeddings with vocabulary and vector management
#[derive(Debug)]
pub struct TokenEmbeddings {
    /// Embedding vectors (vocabulary_size x embedding_dim)
    embeddings: Vec<Vec<f32>>,
    /// Vocabulary mapping token strings to indices
    vocabulary: HashMap<String, usize>,
    /// Reverse mapping from indices to tokens
    reverse_vocab: Vec<String>,
    /// Special token indices
    special_tokens: SpecialTokens,
    /// Embedding dimensionality
    embedding_dim: usize,
    /// Unknown token handling strategy
    unknown_strategy: UnknownTokenStrategy,
}

#[async_trait::async_trait]
impl Embeddings for TokenEmbeddings {
    fn dim(&self) -> usize {
        self.embedding_dim
    }

    async fn token(&self, token: &str) -> Result<Vec<f32>> {
        match self.vocabulary.get(token) {
            Some(&idx) => Ok(self.embeddings[idx].clone()),
            None => self.handle_unknown_token(token).await,
        }
    }

    async fn position(&self, _pos: usize) -> Result<Vec<f32>> {
        Err(anyhow::anyhow!("TokenEmbeddings does not support position embeddings"))
    }

    async fn segment(&self, _seg: usize) -> Result<Vec<f32>> {
        Err(anyhow::anyhow!("TokenEmbeddings does not support segment embeddings"))
    }
}

impl Default for TokenEmbeddings {
    fn default() -> Self {
        let embedding_dim = 768;
        // Create basic vocabulary with common tokens
        let mut vocabulary = HashMap::new();
        let mut reverse_vocab = Vec::new();
        let mut embeddings = Vec::new();

        // Add special tokens
        let special_tokens = ["[PAD]", "[UNK]", "[CLS]", "[SEP]", "[MASK]"];
        for (i, token) in special_tokens.iter().enumerate() {
            vocabulary.insert(token.to_string(), i);
            reverse_vocab.push(token.to_string());
            // Use distinct vectors for special tokens
            embeddings.push(vec![(i as f32 + 1.0) / 10.0; embedding_dim]);
        }

        Self {
            embeddings,
            vocabulary,
            reverse_vocab,
            special_tokens: SpecialTokens {
                pad_token: Some(0),
                unk_token: Some(1),
                bos_token: Some(2),
                eos_token: Some(3),
                mask_token: Some(4),
            },
            embedding_dim,
            unknown_strategy: UnknownTokenStrategy::ZeroVector,
        }
    }
}

impl TokenEmbeddings {
    async fn handle_unknown_token(&self, token: &str) -> Result<Vec<f32>> {
        match self.unknown_strategy {
            UnknownTokenStrategy::ZeroVector => Ok(vec![0.0; self.embedding_dim]),
            UnknownTokenStrategy::AverageEmbedding => {
                if self.embeddings.is_empty() {
                    Ok(vec![0.0; self.embedding_dim])
                } else {
                    let mut avg = vec![0.0; self.embedding_dim];
                    for emb in &self.embeddings {
                        for i in 0..self.embedding_dim {
                            avg[i] += emb[i];
                        }
                    }
                    for i in 0..self.embedding_dim {
                        avg[i] /= self.embeddings.len() as f32;
                    }
                    Ok(avg)
                }
            }
            UnknownTokenStrategy::FuzzyMatch => {
                // Simple fuzzy matching - find closest by edit distance
                let mut best_match = None;
                let mut best_distance = usize::MAX;

                for candidate in self.vocabulary.keys() {
                    let distance = strsim::levenshtein(token, candidate);
                    if distance < best_distance {
                        best_distance = distance;
                        best_match = Some(candidate.clone());
                    }
                }

                match best_match {
                    Some(matched_token) => self.token(&matched_token).await,
                    None => Ok(vec![0.0; self.embedding_dim]),
                }
            }
            UnknownTokenStrategy::SubwordDecomposition => {
                // Placeholder - would implement BPE or similar
                tracing::warn!("Subword decomposition not implemented, using zero vector for {}", token);
                Ok(vec![0.0; self.embedding_dim])
            }
        }
    }
}

/// Position embeddings for sequence modeling
#[derive(Debug)]
pub struct PositionEmbeddings {
    /// Position embedding vectors
    embeddings: Vec<Vec<f32>>,
    /// Maximum supported sequence length
    max_position: usize,
    /// Position embedding type
    embedding_type: PositionEmbeddingType,
    /// Whether to extrapolate beyond max_position
    extrapolate: bool,
}

#[async_trait::async_trait]
impl Embeddings for PositionEmbeddings {
    fn dim(&self) -> usize {
        self.embeddings.first().map(|v| v.len()).unwrap_or(768)
    }

    async fn token(&self, _token: &str) -> Result<Vec<f32>> {
        Err(anyhow::anyhow!("PositionEmbeddings does not support token embeddings"))
    }

    async fn position(&self, pos: usize) -> Result<Vec<f32>> {
        if pos < self.embeddings.len() {
            Ok(self.embeddings[pos].clone())
        } else if self.extrapolate {
            // Simple extrapolation - repeat last position
            Ok(self.embeddings.last().cloned().unwrap_or_else(|| vec![0.0; self.dim()]))
        } else {
            Err(anyhow::anyhow!("Position {} exceeds maximum {}", pos, self.max_position))
        }
    }

    async fn segment(&self, _seg: usize) -> Result<Vec<f32>> {
        Err(anyhow::anyhow!("PositionEmbeddings does not support segment embeddings"))
    }
}

impl Default for PositionEmbeddings {
    fn default() -> Self {
        let dim = 768;
        let max_position = 512;
        let mut embeddings = vec![vec![0.0; dim]; max_position];

        // Pre-fill with sinusoidal embeddings
        for p in 0..max_position {
            for i in 0..dim {
                let angle = p as f32 / (10000.0_f32).powf(2.0 * i as f32 / dim as f32);
                embeddings[p][i] = if i % 2 == 0 { angle.sin() } else { angle.cos() };
            }
        }

        Self {
            embeddings,
            max_position,
            embedding_type: PositionEmbeddingType::Sinusoidal,
            extrapolate: true,
        }
    }
}

/// Segment/type embeddings for multi-part inputs
#[derive(Debug)]
pub struct SegmentEmbeddings {
    /// Segment embedding vectors
    embeddings: Vec<Vec<f32>>,
    /// Maximum number of segments supported
    max_segments: usize,
    /// Segment type names
    segment_types: Vec<String>,
}

#[async_trait::async_trait]
impl Embeddings for SegmentEmbeddings {
    fn dim(&self) -> usize {
        self.embeddings.first().map(|v| v.len()).unwrap_or(768)
    }

    async fn token(&self, _token: &str) -> Result<Vec<f32>> {
        Err(anyhow::anyhow!("SegmentEmbeddings does not support token embeddings"))
    }

    async fn position(&self, _pos: usize) -> Result<Vec<f32>> {
        Err(anyhow::anyhow!("SegmentEmbeddings does not support position embeddings"))
    }

    async fn segment(&self, seg: usize) -> Result<Vec<f32>> {
        if seg < self.embeddings.len() {
            Ok(self.embeddings[seg].clone())
        } else {
            Err(anyhow::anyhow!("Segment {} exceeds maximum {}", seg, self.max_segments))
        }
    }
}

impl Default for SegmentEmbeddings {
    fn default() -> Self {
        let dim = 768;
        let max_segments = 2;
        let embeddings = vec![
            vec![0.05; dim],  // Segment A
            vec![-0.05; dim], // Segment B
        ];
        let segment_types = vec!["sentence_a".into(), "sentence_b".into()];

        Self {
            embeddings,
            max_segments,
            segment_types,
        }
    }
}

/// Position embedding types
#[derive(Debug, Clone, PartialEq)]
pub enum PositionEmbeddingType {
    /// Learned positional embeddings
    Learned,
    /// Sinusoidal positional encoding
    Sinusoidal,
    /// Rotary Position Embedding (RoPE)
    RoPE,
    /// ALiBi (Attention with Linear Biases)
    ALiBi,
}

/// Special token indices
#[derive(Debug, Clone)]
pub struct SpecialTokens {
    /// Padding token index
    pub pad_token: Option<usize>,
    /// Unknown token index
    pub unk_token: Option<usize>,
    /// Beginning of sequence token index
    pub bos_token: Option<usize>,
    /// End of sequence token index
    pub eos_token: Option<usize>,
    /// Mask token index (for masked language modeling)
    pub mask_token: Option<usize>,
}

/// Strategy for handling unknown tokens
#[derive(Debug, Clone, PartialEq)]
pub enum UnknownTokenStrategy {
    /// Return zero vector
    ZeroVector,
    /// Return average of all embeddings
    AverageEmbedding,
    /// Use fuzzy matching to find closest token
    FuzzyMatch,
    /// Use subword decomposition if supported
    SubwordDecomposition,
}
