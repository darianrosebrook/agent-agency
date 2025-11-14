//! Search coordination across multiple modalities

use anyhow::Result;
use schemars::JsonSchema;
use std::sync::Arc;
use tracing::{debug, info};

use super::core::{MultimodalQuery, MultimodalRetrieverConfig, MultimodalSearchResult};
use super::query_processing::ProcessedQuery;
use super::text_search::TextSearchEngine;
use super::visual_search::VisualSearchEngine;
use agent_agency_contracts::types::research::QueryType;

/// Search coordinator managing multimodal search execution
use serde::{Deserialize, Serialize};
#[derive(Debug)]
pub struct SearchCoordinator {
    config: MultimodalRetrieverConfig,
    text_engine: Arc<TextSearchEngine>,
    visual_engine: Arc<VisualSearchEngine>,
}

impl SearchCoordinator {
    /// Create a new search coordinator
    pub fn new(config: MultimodalRetrieverConfig) -> Result<Self> {
        let text_engine = Arc::new(TextSearchEngine::new(config.clone())?);
        let visual_engine = Arc::new(VisualSearchEngine::new(config.clone())?);

        Ok(Self {
            config,
            text_engine,
            visual_engine,
        })
    }

    /// Create a new search coordinator with database integration
    pub async fn new_with_database(
        database_pool: Arc<data_infrastructure::DatabaseClient>,
        config: MultimodalRetrieverConfig,
    ) -> Result<Self> {
        let text_engine = Arc::new(
            TextSearchEngine::new_with_database(database_pool.clone(), config.clone()).await?,
        );
        let visual_engine = Arc::new(VisualSearchEngine::new(config.clone())?);

        Ok(Self {
            config,
            text_engine,
            visual_engine,
        })
    }

    /// Execute multimodal search across all relevant modalities
    pub async fn execute_multimodal_search(
        &self,
        query: &ProcessedQuery,
        k: usize,
    ) -> Result<Vec<Vec<MultimodalSearchResult>>> {
        info!(
            "Executing multimodal search for query type: {:?}",
            query.query_type
        );

        let mut modality_results = Vec::new();

        // Execute text-based search if applicable
        if self.should_search_text(query) {
            debug!("Executing text search");
            let text_results = self.text_engine.search(query, k).await?;
            modality_results.push(text_results);
        }

        // Execute visual search if applicable
        if self.should_search_visual(query) {
            debug!("Executing visual search");
            let visual_results = self.visual_engine.search(query, k).await?;
            modality_results.push(visual_results);
        }

        // Execute code search if applicable
        if self.should_search_code(query) {
            debug!("Executing code search");
            let code_results = self.text_engine.search_code(query, k).await?;
            modality_results.push(code_results);
        }

        Ok(modality_results)
    }

    /// Determine if text search should be executed
    fn should_search_text(&self, query: &ProcessedQuery) -> bool {
        matches!(query.query_type, QueryType::Text | QueryType::Hybrid) && query.text.is_some()
    }

    /// Determine if visual search should be executed
    fn should_search_visual(&self, query: &ProcessedQuery) -> bool {
        matches!(
            query.query_type,
            QueryType::Visual | QueryType::Image | QueryType::Hybrid
        ) && query.image_path.is_some()
    }

    /// Determine if code search should be executed
    fn should_search_code(&self, query: &ProcessedQuery) -> bool {
        matches!(query.query_type, QueryType::Code | QueryType::Hybrid) && query.text.is_some()
    }

    /// Get search statistics
    pub async fn get_search_stats(&self) -> Result<SearchStats> {
        let text_stats = self.text_engine.get_stats().await?;
        let visual_stats = self.visual_engine.get_stats().await?;

        Ok(SearchStats {
            text_searches: text_stats.total_searches,
            visual_searches: visual_stats.total_searches,
            total_searches: text_stats.total_searches + visual_stats.total_searches,
            average_text_latency_ms: text_stats.average_latency_ms,
            average_visual_latency_ms: visual_stats.average_visual_latency_ms,
        })
    }
}

/// Search execution statistics

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchStats {
    pub text_searches: u64,
    pub visual_searches: u64,
    pub total_searches: u64,
    pub average_text_latency_ms: f64,
    pub average_visual_latency_ms: f64,
}
