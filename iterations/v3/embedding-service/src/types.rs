//! Core types for embedding service

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Embedding vector type - 768 dimensions for embeddinggemma
pub type EmbeddingVector = Vec<f32>;

/// Unique identifier for an embedding
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EmbeddingId(String);

impl EmbeddingId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EmbeddingId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for EmbeddingId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Metadata associated with an embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingMetadata {
    pub source: String,
    pub content_type: ContentType,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub tags: Vec<String>,
    pub context: HashMap<String, String>,
}

/// Type of content being embedded
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ContentType {
    Text,
    Code,
    Documentation,
    TaskDescription,
    Evidence,
    Knowledge,
    VideoFrame,         // Keyframes from video ingestion
    SlideContent,       // Text/structure from slide pages
    DiagramNode,        // Named entities in diagrams
    SpeechTranscript,   // ASR/diarization output
    VisualCaption,      // Auto-generated captions for figures
}

/// Stored embedding with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEmbedding {
    pub id: EmbeddingId,
    pub vector: EmbeddingVector,
    pub metadata: EmbeddingMetadata,
}

/// Request to generate embeddings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    pub texts: Vec<String>,
    pub content_type: ContentType,
    pub source: String,
    pub tags: Vec<String>,
    pub context: HashMap<String, String>,
}

/// Response containing generated embeddings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub embeddings: Vec<StoredEmbedding>,
    pub processing_time_ms: u64,
}

/// Similarity search request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityRequest {
    pub query_vector: EmbeddingVector,
    pub limit: usize,
    pub threshold: f32,
    pub content_types: Vec<ContentType>,
    pub tags: Vec<String>,
}

/// Similarity search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityResult {
    pub embedding: StoredEmbedding,
    pub similarity_score: f32,
}

/// Semantic context for tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticContext {
    pub task_description: String,
    pub context_vector: EmbeddingVector,
    pub related_embeddings: Vec<SimilarityResult>,
    pub confidence: f32,
}

/// Configuration for embedding service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub ollama_url: String,
    pub model_name: String,
    pub dimension: usize,
    pub batch_size: usize,
    pub cache_size: usize,
    pub timeout_ms: u64,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            ollama_url: "http://localhost:11434".to_string(),
            model_name: "embeddinggemma".to_string(),
            dimension: 768,
            batch_size: 10,
            cache_size: 1000,
            timeout_ms: 30000,
        }
    }
}

/// Embedding model registry entry (config-driven)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingModel {
    pub id: String,           // registry key (e.g., 'e5-small-v2', 'clip-vit-b32')
    pub modality: String,     // 'text' | 'image' | 'audio'
    pub dim: usize,           // vector dimensions
    pub metric: String,       // 'cosine' | 'ip' | 'l2'
    pub active: bool,         // whether to use this model
}

/// Per-block vector (one row per block-model pair)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockVector {
    pub block_id: String,     // UUID of block
    pub model_id: String,     // embedding model identifier
    pub modality: String,     // 'text' | 'image' | 'audio'
    pub vec: EmbeddingVector, // the actual vector
}

/// Search result feature with per-index scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultFeature {
    pub score_text: Option<f32>,    // BM25 + dense text similarity
    pub score_image: Option<f32>,   // CLIP/visual similarity
    pub score_graph: Option<f32>,   // diagram graph relevance
    pub fused_score: f32,           // combined via RRF or learned weights
    pub features_json: serde_json::Value, // audit trail
}

/// Multimodal search result with citation and feature trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalSearchResult {
    pub ref_id: String,       // UUID of block/segment
    pub kind: ContentType,
    pub snippet: String,
    pub citation: Option<String>,  // uri + (t0–t1 | bbox)
    pub feature: SearchResultFeature,
    pub project_scope: Option<String>,  // scoping info for filtering
}
