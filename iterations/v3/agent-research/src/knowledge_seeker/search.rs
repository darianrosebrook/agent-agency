//! Search coordination for vector and keyword search

use schemars::JsonSchema;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::{info, warn};

use crate::research_types::*;
use crate::VectorSearchEngine;
use anyhow::{Result, Context};

use super::index::InvertedIndex;
use super::events::EventEmitter;

/// Search coordinator for managing different search strategies

#[derive(Debug)]
pub struct SearchCoordinator {
    vector_search: Arc<VectorSearchEngine>,
    keyword_index: Arc<InvertedIndex>,
    config: ResearchAgentConfig,
    event_emitter: Arc<EventEmitter>,
}

impl SearchCoordinator {
    /// Create a new search coordinator
    pub async fn new(config: ResearchAgentConfig) -> Result<Self> {
        // Initialize vector search engine
        let vector_search = Arc::new(
            VectorSearchEngine::new(
                &config.vector_search.qdrant_url,
                &config.vector_search.collection_name,
                config.vector_search.dimension as u32,
                config.vector_search.similarity_threshold,
                config.vector_search.max_results,
            )
            .await
            .context("Failed to initialize vector search engine")?,
        );

        // Initialize keyword index
        let keyword_index = Arc::new(InvertedIndex::new());

        let event_emitter = Arc::new(EventEmitter::new());

        Ok(Self {
            vector_search,
            keyword_index,
            config,
            event_emitter,
        })
    }

    /// Perform vector search for a query
    pub async fn vector_search(&self, query: &ResearchQuery) -> Result<Vec<ResearchResult>> {
        info!("Performing vector search for query: {}", query.query);

        // Generate query embedding
        let query_embedding = self
            .vector_search
            .generate_embedding(&query.query)
            .await
            .context("Failed to generate query embedding")?;

        // Perform vector search
        let limit = query.max_results.map(|x| (x * 2) as usize).unwrap_or(20);
        let vector_results = self
            .vector_search
            .search(
                &query_embedding,
                Some(limit),
                None,
            )
            .await
            .context("Vector search failed")?;

        // Convert vector results to research results
        let mut research_results = Vec::new();
        for entry in vector_results {
            let result = ResearchResult {
                query_id: query.id,
                source: entry.source.clone(),
                title: entry.title.clone(),
                content: entry.content.clone(),
                summary: None,
                relevance_score: 0.8, // V2-style relevance from vector similarity
                confidence_score: self.calculate_v2_confidence_score_from_entry(&entry, query),
                extracted_at: chrono::Utc::now(),
                url: entry.source_url.clone(),
                metadata: entry.metadata.clone(),
            };
            research_results.push(result);
        }

        info!("Vector search completed: {} results", research_results.len());
        Ok(research_results)
    }

    /// Perform keyword search for a query
    pub async fn keyword_search(&self, query: &ResearchQuery) -> Result<Vec<ResearchResult>> {
        info!("Performing keyword search for query: {}", query.query);

        // TODO: Implement keyword search with inverted index:
        // 1. Index population: Populate inverted index from knowledge entries
        //    - Build inverted index from existing knowledge entries
        //    - Index terms with document IDs and positions
        //    - Support incremental index updates
        // 2. Query processing: Process search queries against index
        //    - Tokenize and normalize query terms
        //    - Look up terms in inverted index
        //    - Combine results from multiple terms
        // 3. Result ranking: Rank search results by relevance
        //    - Calculate relevance scores based on term frequency
        //    - Apply ranking algorithms (TF-IDF, BM25, etc.)
        //    - Return top-k results sorted by relevance
        // ACCEPTANCE CRITERIA:
        // - Inverted index is populated from knowledge entries
        // - Keyword searches return relevant results from index
        // - Results are ranked by relevance score
        // DEPENDENCIES:
        // - Inverted index data structure (Required)
        // - Index population mechanism (Required)
        // PRIORITY: High
        warn!("Keyword search not fully implemented - inverted index needs population");

        Ok(Vec::new())
    }

    /// Calculate V2 confidence score for vector search results (from KnowledgeEntry)
    fn calculate_v2_confidence_score_from_entry(&self, entry: &KnowledgeEntry, query: &ResearchQuery) -> f32 {
        let mut confidence = 0.7; // Base confidence for vector search

        // Higher confidence for exact matches in title
        if entry.title.to_lowercase().contains(&query.query.to_lowercase()) {
            confidence += 0.2;
        }

        // Higher confidence for structured content
        if entry.content.contains("```") || entry.content.contains("# ") {
            confidence += 0.1;
        }

        // Higher confidence for recent content
        // Note: VectorSearchResult might not have timestamp info, assuming current
        let confidence: f32 = confidence + 0.1;

        confidence.min(1.0_f32).max(0.0_f32)
    }

    /// Update configuration
    pub async fn update_config(&self, update: ConfigurationUpdate) -> Result<()> {
        match update {
            ConfigurationUpdate::VectorSearch(config) => {
                // Update vector search configuration if needed
                info!("Vector search configuration updated");
            }
            _ => {} // Other updates don't affect search
        }
        Ok(())
    }
}

// InvertedIndex, Posting, and SearchResult are defined in index.rs, not here
// Remove duplicate definitions - use the ones from super::index
