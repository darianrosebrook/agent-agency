//! Core multimodal retriever functionality and configuration

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::search::SearchCoordinator;
use super::fusion::FusionEngine;
use super::query_processing::QueryProcessor;

/// Configuration for multimodal retrieval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalRetrieverConfig {
    /// Maximum number of results per modality
    pub k_per_modality: usize,
    /// Fusion method for combining results
    pub fusion_method: crate::research_types::FusionMethod,
    /// Optional project scope for filtering
    pub project_scope: Option<String>,
    /// Whether to enable deduplication
    pub enable_deduplication: bool,
    /// Maximum total results to return
    pub max_total_results: usize,
    /// Text search weight in fusion
    pub text_weight: f32,
    /// Visual search weight in fusion
    pub visual_weight: f32,
    /// Code search weight in fusion
    pub code_weight: f32,
}

impl Default for MultimodalRetrieverConfig {
    fn default() -> Self {
        Self {
            k_per_modality: 10,
            fusion_method: crate::research_types::FusionMethod::RRF,
            project_scope: None,
            enable_deduplication: true,
            max_total_results: 50,
            text_weight: 0.5,
            visual_weight: 0.3,
            code_weight: 0.2,
        }
    }
}

/// Main multimodal retriever coordinating search across multiple modalities
#[derive(Debug)]
pub struct MultimodalRetriever {
    config: MultimodalRetrieverConfig,
    search_coordinator: Arc<SearchCoordinator>,
    fusion_engine: Arc<FusionEngine>,
    query_processor: Arc<QueryProcessor>,
}

/// Search query with optional multimodal content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalQuery {
    pub text: Option<String>,
    pub image_path: Option<std::path::PathBuf>,
    pub query_type: QueryType,
    pub project_scope: Option<String>,
    pub max_results: usize,
    /// Anchor timestamp for timestamp-anchored searches
    pub anchor_timestamp: Option<DateTime<Utc>>,
    /// Time window in seconds around anchor timestamp
    pub time_window_seconds: Option<u64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum QueryType {
    Text,
    Visual,
    Image,
    Code,
    TimestampAnchored,
    Hybrid,
}

/// Advanced fusion strategies for multimodal results
#[derive(Debug, Clone)]
pub enum FusionStrategy {
    /// Simple weighted combination
    Weighted,
    /// Adaptive weighting based on modality confidence
    AdaptiveWeighted,
    /// Reciprocal Rank Fusion (RRF)
    RRF,
    /// Learned fusion using neural networks (future)
    Neural,
}

/// Search result combining multiple modalities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalSearchResult {
    pub id: String,
    pub content: String,
    pub modality_scores: HashMap<String, f32>,
    pub combined_score: f32,
    pub metadata: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    pub source_modality: String,
    pub project_scope: Option<String>,
}

use std::collections::HashMap;

impl MultimodalRetriever {
    /// Create a new multimodal retriever with default configuration
    pub fn new(config: Option<MultimodalRetrieverConfig>) -> Result<Self> {
        let config = config.unwrap_or_default();

        let search_coordinator = Arc::new(SearchCoordinator::new(config.clone())?);
        let fusion_engine = Arc::new(FusionEngine::new(config.clone())?);
        let query_processor = Arc::new(QueryProcessor::new(config.clone())?);

        Ok(Self {
            config,
            search_coordinator,
            fusion_engine,
            query_processor,
        })
    }

    /// Create a new multimodal retriever with database pool integration
    pub async fn new_with_database_pool(
        database_pool: Arc<data_infrastructure::DatabaseClient>,
        config: Option<MultimodalRetrieverConfig>,
    ) -> Result<Self> {
        let config = config.unwrap_or_default();

        let search_coordinator = Arc::new(SearchCoordinator::new_with_database(database_pool.clone(), config.clone()).await?);
        let fusion_engine = Arc::new(FusionEngine::new(config.clone())?);
        let query_processor = Arc::new(QueryProcessor::new(config.clone())?);

        Ok(Self {
            config,
            search_coordinator,
            fusion_engine,
            query_processor,
        })
    }

    /// Execute a multimodal search query
    pub async fn search(
        &self,
        query: &str,
        k: usize,
        project_scope: Option<&str>,
    ) -> Result<Vec<MultimodalSearchResult>> {
        // Parse and validate query
        let processed_query = self.query_processor.process_query(query, project_scope)?;

        // Coordinate search across modalities
        let search_results = self.search_coordinator.execute_multimodal_search(&processed_query, k).await?;

        // Fuse results from different modalities
        let fused_results = self.fusion_engine.fuse_results(search_results, k)?;

        Ok(fused_results)
    }

    /// Search with a structured multimodal query
    pub async fn search_multimodal(
        &self,
        query: MultimodalQuery,
    ) -> Result<Vec<MultimodalSearchResult>> {
        // Process the structured query
        let processed_query = self.query_processor.process_multimodal_query(query)?;

        // Execute search
        let search_results = self.search_coordinator.execute_multimodal_search(&processed_query, processed_query.max_results).await?;

        // Fuse results
        let fused_results = self.fusion_engine.fuse_results(search_results, processed_query.max_results)?;

        Ok(fused_results)
    }

    /// Get retriever configuration
    pub fn config(&self) -> &MultimodalRetrieverConfig {
        &self.config
    }

    /// Update retriever configuration
    pub fn update_config(&mut self, config: MultimodalRetrieverConfig) {
        self.config = config.clone();
        // Update component configs as needed
    }
}
