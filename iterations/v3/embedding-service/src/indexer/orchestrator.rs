//! Multimodal indexer orchestrator
//!
//! Main orchestrator that coordinates text, visual, and graph indexing
//! with unified search capabilities and storage management.

use super::text::{TextIndexer, TextDocument};
use super::visual::{VisualIndexer, VisualDocument};
use super::graph::{GraphIndexer, NodeProperty, GraphEdge};
use super::search::{MultimodalSearchEngine, MultimodalQuery, UnifiedSearchResult};
use super::storage::EmbeddingStorage;
use super::super::types::*;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Multimodal indexer with per-modality search capabilities
#[derive(Debug)]
pub struct MultimodalIndexer {
    text_indexer: Arc<RwLock<TextIndexer>>,
    visual_indexer: Arc<RwLock<VisualIndexer>>,
    graph_indexer: Arc<RwLock<GraphIndexer>>,
    search_engine: Arc<RwLock<MultimodalSearchEngine>>,
    storage: Option<Arc<EmbeddingStorage>>,
}

impl MultimodalIndexer {
    /// Create a new multimodal indexer
    pub fn new() -> Self {
        let text_indexer = Arc::new(RwLock::new(TextIndexer::new()));
        let visual_indexer = Arc::new(RwLock::new(VisualIndexer::new()));
        let graph_indexer = Arc::new(RwLock::new(GraphIndexer::new()));

        let search_engine = Arc::new(RwLock::new(MultimodalSearchEngine::new(
            TextIndexer::new(),
            VisualIndexer::new(),
            GraphIndexer::new(),
        )));

        Self {
            text_indexer,
            visual_indexer,
            graph_indexer,
            search_engine,
            storage: None,
        }
    }

    /// Create indexer with database storage
    pub fn with_storage(storage: Arc<EmbeddingStorage>) -> Self {
        let mut indexer = Self::new();
        indexer.storage = Some(storage);
        indexer
    }

    /// Index text document
    pub async fn index_text(&self, document: TextDocument) -> Result<()> {
        let mut indexer = self.text_indexer.write().await;
        indexer.index_document(document.clone())?;

        // Persist if storage is available
        if let Some(storage) = &self.storage {
            storage.store_text_document(&document).await?;
        }

        Ok(())
    }

    /// Index visual document
    pub async fn index_visual(&self, document: VisualDocument) -> Result<()> {
        let mut indexer = self.visual_indexer.write().await;
        indexer.index_visual(document)?;

        Ok(())
    }

    /// Add graph node
    pub async fn add_graph_node(&self, node_id: Uuid, properties: NodeProperty) -> Result<()> {
        let mut indexer = self.graph_indexer.write().await;
        indexer.add_node(node_id, properties)?;
        Ok(())
    }

    /// Add graph edge
    pub async fn add_graph_edge(&self, edge: GraphEdge) -> Result<()> {
        let mut indexer = self.graph_indexer.write().await;
        indexer.add_edge(edge)?;
        Ok(())
    }

    /// Search across all modalities
    pub async fn multimodal_search(&self, query: MultimodalQuery) -> Result<Vec<UnifiedSearchResult>> {
        let engine = self.search_engine.read().await;
        engine.search(query).await
    }

    /// Text search only
    pub async fn text_search(&self, query: &str, limit: usize) -> Result<Vec<super::text::SearchResult>> {
        let indexer = self.text_indexer.read().await;
        Ok(indexer.bm25_search(query, limit))
    }

    /// Visual search only
    pub async fn visual_search(&self, embedding: &EmbeddingVector, limit: usize) -> Result<Vec<super::visual::VisualSearchResult>> {
        let indexer = self.visual_indexer.read().await;
        Ok(indexer.visual_search(embedding, limit))
    }

    /// Graph traversal
    pub async fn graph_neighbors(&self, node_id: Uuid) -> Vec<Uuid> {
        let indexer = self.graph_indexer.read().await;
        indexer.get_neighbors(node_id)
    }

    /// Get comprehensive index statistics
    pub async fn get_statistics(&self) -> IndexStatistics {
        let text_stats = self.text_indexer.read().await.get_statistics();
        let visual_stats = self.visual_indexer.read().await.get_statistics();
        let graph_stats = self.graph_indexer.read().await.get_statistics();

        let storage_stats = if let Some(storage) = &self.storage {
            storage.get_stats().await.ok()
        } else {
            None
        };

        IndexStatistics {
            text_documents: text_stats.total_documents,
            visual_documents: visual_stats.total_images,
            graph_nodes: graph_stats.total_nodes,
            graph_edges: graph_stats.total_edges,
            total_terms: text_stats.total_terms,
            models_indexed: text_stats.models_indexed + visual_stats.models_indexed,
            storage_stats,
        }
    }

    /// Perform health check across all components
    pub async fn health_check(&self) -> Result<HealthStatus> {
        let mut issues = Vec::new();

        // Check storage health
        if let Some(storage) = &self.storage {
            if storage.health_check().await.is_err() {
                issues.push("Database storage unavailable".to_string());
            }
        }

        // Check index sizes
        let stats = self.get_statistics().await;
        if stats.text_documents == 0 && stats.visual_documents == 0 && stats.graph_nodes == 0 {
            issues.push("No content indexed".to_string());
        }

        Ok(if issues.is_empty() {
            HealthStatus::Healthy
        } else {
            HealthStatus::Degraded(issues)
        })
    }

    /// Optimize all indices
    pub async fn optimize_indices(&self) -> Result<()> {
        // Placeholder - would implement index optimization
        Ok(())
    }

    /// Clear all indices
    pub async fn clear_indices(&self) -> Result<()> {
        let mut text_indexer = self.text_indexer.write().await;
        *text_indexer = TextIndexer::new();

        let mut visual_indexer = self.visual_indexer.write().await;
        *visual_indexer = VisualIndexer::new();

        let mut graph_indexer = self.graph_indexer.write().await;
        *graph_indexer = GraphIndexer::new();

        Ok(())
    }
}

/// Comprehensive index statistics
#[derive(Debug)]
pub struct IndexStatistics {
    pub text_documents: usize,
    pub visual_documents: usize,
    pub graph_nodes: usize,
    pub graph_edges: usize,
    pub total_terms: usize,
    pub models_indexed: usize,
    pub storage_stats: Option<super::storage::DatabaseStats>,
}

/// Health status of the indexer
#[derive(Debug)]
pub enum HealthStatus {
    Healthy,
    Degraded(Vec<String>),
    Unhealthy(String),
}

/// Indexer configuration
#[derive(Debug, Clone)]
pub struct IndexerConfig {
    pub enable_persistence: bool,
    pub max_index_size: usize,
    pub enable_health_checks: bool,
    pub optimization_interval_minutes: u64,
}

impl Default for IndexerConfig {
    fn default() -> Self {
        Self {
            enable_persistence: true,
            max_index_size: 1_000_000,
            enable_health_checks: true,
            optimization_interval_minutes: 60,
        }
    }
}

/// Builder for creating multimodal indexer instances
pub struct MultimodalIndexerBuilder {
    config: IndexerConfig,
    storage: Option<Arc<EmbeddingStorage>>,
}

impl MultimodalIndexerBuilder {
    pub fn new() -> Self {
        Self {
            config: IndexerConfig::default(),
            storage: None,
        }
    }

    pub fn with_config(mut self, config: IndexerConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_storage(mut self, storage: Arc<EmbeddingStorage>) -> Self {
        self.storage = Some(storage);
        self
    }

    pub fn build(self) -> MultimodalIndexer {
        let indexer = if let Some(storage) = self.storage {
            MultimodalIndexer::with_storage(storage)
        } else {
            MultimodalIndexer::new()
        };

        // Could apply additional configuration here
        indexer
    }
}
