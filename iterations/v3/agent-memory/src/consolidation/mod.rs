//! Memory Consolidation Engine
//!
//! Automatic memory consolidation through semantic clustering, summarization, and deduplication.

use serde::{Deserialize, Serialize};
pub mod consolidation_engine;
pub mod deduplication;
pub mod semantic_clustering;
pub mod summarization;

pub use consolidation_engine::*;
pub use deduplication::*;
pub use semantic_clustering::*;
pub use summarization::*;

/// Consolidation configuration
#[derive(Debug, Clone)]
pub struct ConsolidationConfig {
    pub enable_semantic_clustering: bool,
    pub enable_summarization: bool,
    pub enable_deduplication: bool,
    pub clustering_threshold: f32,
    pub summarization_threshold: usize,
    pub deduplication_threshold: f32,
    pub max_cluster_size: usize,
    pub consolidation_interval_hours: u64,
}

/// Consolidation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationResult {
    pub consolidated_memories: usize,
    pub created_clusters: usize,
    pub generated_summaries: usize,
    pub removed_duplicates: usize,
    pub processing_time_ms: u64,
    pub consolidation_timestamp: chrono::DateTime<chrono::Utc>,
}

/// Memory cluster for semantic grouping
#[derive(Debug, Clone)]
pub struct MemoryCluster {
    pub cluster_id: String,
    pub centroid_embedding: Vec<f32>,
    pub member_memories: Vec<crate::memory_types::MemoryId>,
    pub cluster_summary: Option<String>,
    pub importance_score: f32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Consolidation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationStats {
    pub total_memories_processed: usize,
    pub active_clusters: usize,
    pub total_summaries: usize,
    pub deduplication_savings: usize,
    pub average_cluster_size: f64,
    pub last_consolidation: Option<chrono::DateTime<chrono::Utc>>,
}

/// Consolidation engine trait
#[async_trait::async_trait]
pub trait ConsolidationEngine: Send + Sync {
    /// Run full consolidation cycle
    async fn consolidate(
        &self,
        config: &ConsolidationConfig,
    ) -> crate::MemoryResult<ConsolidationResult>;

    /// Consolidate specific memory subset
    async fn consolidate_subset(
        &self,
        memory_ids: &[crate::memory_types::MemoryId],
        config: &ConsolidationConfig,
    ) -> crate::MemoryResult<ConsolidationResult>;

    /// Get consolidation statistics
    async fn get_stats(&self) -> crate::MemoryResult<ConsolidationStats>;

    /// Force rebuild of clusters
    async fn rebuild_clusters(&self) -> crate::MemoryResult<()>;

    /// Get memory clusters
    async fn get_clusters(&self) -> crate::MemoryResult<Vec<MemoryCluster>>;
}
