//! Vector Search Optimization
//!
//! Advanced vector search with hybrid strategies, reranking, and efficient retrieval.

use serde::{Deserialize, Serialize};
pub mod hybrid_search;
pub mod reranking;
pub mod search_engine;
pub mod similarity_metrics;

pub use hybrid_search::*;
pub use reranking::*;
pub use search_engine::*;
pub use similarity_metrics::*;

/// Search configuration
#[derive(Debug, Clone)]
pub struct VectorSearchConfig {
    pub default_top_k: usize,
    pub max_results: usize,
    pub similarity_threshold: f32,
    pub enable_hybrid_search: bool,
    pub enable_reranking: bool,
    pub rerank_top_k: usize,
}

/// Search query
#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub text: Option<String>,
    pub embedding: Option<Vec<f32>>,
    pub filters: SearchFilters,
    pub search_type: SearchType,
}

/// Search filters
#[derive(Debug, Clone, Default)]
pub struct SearchFilters {
    pub memory_types: Option<Vec<crate::memory_types::MemoryType>>,
    pub importance_range: Option<(f32, f32)>,
    pub time_range: Option<(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>,
    pub workspace_id: Option<uuid::Uuid>,
    pub tags: Option<Vec<String>>,
}

/// Search types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchType {
    TextOnly,
    VectorOnly,
    Hybrid,
}

/// Search result
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub memory_id: crate::memory_types::MemoryId,
    pub score: f32,
    pub rank: usize,
    pub memory_data: serde_json::Value,
    pub metadata: SearchMetadata,
}

/// Search metadata
#[derive(Debug, Clone)]
pub struct SearchMetadata {
    pub search_type: SearchType,
    pub retrieved_at: chrono::DateTime<chrono::Utc>,
    pub processing_time_ms: u64,
    pub vector_similarity: Option<f32>,
    pub text_similarity: Option<f32>,
}

/// Search response
#[derive(Debug, Clone)]
pub struct SearchResponse {
    pub query: SearchQuery,
    pub results: Vec<SearchResult>,
    pub total_found: usize,
    pub search_time_ms: u64,
    pub strategy_used: SearchStrategy,
}

/// Search strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchStrategy {
    TextOnly,
    VectorOnly,
    HybridConcatenation,
    HybridReranking,
    Adaptive,
}

/// Vector search engine trait
#[async_trait::async_trait]
pub trait VectorSearchEngine: Send + Sync {
    /// Search memories using vector similarity
    async fn vector_search(
        &self,
        query_embedding: &[f32],
        top_k: usize,
        filters: &SearchFilters,
    ) -> crate::MemoryResult<Vec<SearchResult>>;

    /// Search memories using text similarity
    async fn text_search(
        &self,
        query_text: &str,
        top_k: usize,
        filters: &SearchFilters,
    ) -> crate::MemoryResult<Vec<SearchResult>>;

    /// Perform hybrid search combining vector and text
    async fn hybrid_search(
        &self,
        query: &SearchQuery,
        config: &VectorSearchConfig,
    ) -> crate::MemoryResult<SearchResponse>;

    /// Add memory to search index
    async fn index_memory(
        &self,
        memory_id: &crate::memory_types::MemoryId,
        embedding: &[f32],
        text_content: &str,
    ) -> crate::MemoryResult<()>;

    /// Remove memory from search index
    async fn remove_from_index(
        &self,
        memory_id: &crate::memory_types::MemoryId,
    ) -> crate::MemoryResult<()>;

    /// Rebuild search index
    async fn rebuild_index(&self) -> crate::MemoryResult<()>;

    /// Get search statistics
    async fn get_stats(&self) -> crate::MemoryResult<SearchStats>;
}

/// Search statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchStats {
    pub total_memories_indexed: usize,
    pub average_search_time_ms: f64,
    pub cache_hit_rate: f64,
    pub index_size_mb: f64,
    pub last_rebuild: Option<chrono::DateTime<chrono::Utc>>,
}
