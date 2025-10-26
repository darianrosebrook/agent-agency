//! @darianrosebrook
//! Shared types for indexing

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Full-text search query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub text: String,
    pub project_scope: Option<String>,
    pub k: usize,
    pub max_tokens: usize,
}

/// Full-text search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub block_id: Uuid,
    pub score: f32,
    pub text_snippet: String,
    pub modality: String,
}

/// Vector search query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorQuery {
    pub vector: Vec<f32>,
    pub model_id: String,
    pub k: usize,
    pub project_scope: Option<String>,
}

/// Vector search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchResult {
    pub block_id: Uuid,
    pub similarity: f32,
    pub modality: String,
}

/// BM25 statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bm25Stats {
    pub total_documents: u64,
    pub total_terms: u64,
    pub avg_doc_length: f32,
    pub k1: f32,
    pub b: f32,
}

impl Default for Bm25Stats {
    fn default() -> Self {
        Self {
            total_documents: 0,
            total_terms: 0,
            avg_doc_length: 0.0,
            k1: 1.5,
            b: 0.75,
        }
    }
}

/// HNSW index metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HnswMetadata {
    pub model_id: String,
    pub modality: String,
    pub dim: usize,
    pub metric: String,
    pub max_neighbors: usize,
    pub ef_construction: usize,
    pub ef_search: usize,
    pub node_count: usize,
}

/// Index statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub bm25: Option<Bm25Stats>,
    pub hnsw: HashMap<String, HnswMetadata>,
    pub last_updated: String,
    pub total_blocks: u64,
}

/// Block vector for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockVectorRecord {
    pub block_id: Uuid,
    pub model_id: String,
    pub modality: String,
    pub vector: Vec<f32>,
}

/// Search audit log entry
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SearchAuditEntry {
    pub id: Uuid,
    pub query: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub results: Option<serde_json::Value>,
    pub features: Option<serde_json::Value>,
}
