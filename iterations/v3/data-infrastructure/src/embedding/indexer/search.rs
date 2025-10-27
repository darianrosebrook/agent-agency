//! Multimodal search and retrieval
//!
//! Unified search interface combining text, visual, and graph
//! search capabilities with result fusion and ranking.

use super::text::TextIndexer;
use super::visual::VisualIndexer;
use super::graph::GraphIndexer;
use super::super::embedding_types::*;
use anyhow::Result;
use std::collections::HashMap;
use uuid::Uuid;

/// Multimodal search query
#[derive(Debug)]
pub struct MultimodalQuery {
    pub text_query: Option<String>,
    pub visual_embedding: Option<EmbeddingVector>,
    pub graph_constraints: Option<GraphQuery>,
    pub modality_weights: ModalityWeights,
    pub limit: usize,
}

/// Graph query constraints
#[derive(Debug)]
pub struct GraphQuery {
    pub start_node: Option<Uuid>,
    pub node_types: Vec<String>,
    pub max_depth: usize,
}

/// Modality weights for result fusion
#[derive(Debug)]
pub struct ModalityWeights {
    pub text_weight: f64,
    pub visual_weight: f64,
    pub graph_weight: f64,
}

impl Default for ModalityWeights {
    fn default() -> Self {
        Self {
            text_weight: 0.4,
            visual_weight: 0.4,
            graph_weight: 0.2,
        }
    }
}

/// Unified search result
#[derive(Debug)]
pub struct UnifiedSearchResult {
    pub document_id: Uuid,
    pub combined_score: f64,
    pub modality_scores: HashMap<String, f64>,
    pub content_preview: String,
    pub metadata: HashMap<String, String>,
}

/// Multimodal search engine
#[derive(Debug)]
pub struct MultimodalSearchEngine {
    text_indexer: TextIndexer,
    visual_indexer: VisualIndexer,
    graph_indexer: GraphIndexer,
}

impl MultimodalSearchEngine {
    pub fn new(
        text_indexer: TextIndexer,
        visual_indexer: VisualIndexer,
        graph_indexer: GraphIndexer,
    ) -> Self {
        Self {
            text_indexer,
            visual_indexer,
            graph_indexer,
        }
    }

    /// Execute multimodal search
    pub async fn search(&self, query: MultimodalQuery) -> Result<Vec<UnifiedSearchResult>> {
        let mut all_results = HashMap::new();

        // Text search
        if let Some(text) = &query.text_query {
            let text_results = self.text_indexer.bm25_search(text, query.limit * 2);
            self.add_results(&mut all_results, text_results, "text", query.modality_weights.text_weight);
        }

        // Visual search
        if let Some(embedding) = &query.visual_embedding {
            let visual_results = self.visual_indexer.visual_search(embedding, query.limit * 2);
            self.add_visual_results(&mut all_results, visual_results, query.modality_weights.visual_weight);
        }

        // Graph constraints
        if let Some(graph_query) = &query.graph_constraints {
            let graph_nodes = self.execute_graph_query(graph_query);
            self.apply_graph_filter(&mut all_results, &graph_nodes, query.modality_weights.graph_weight);
        }

        // Fuse and rank results
        let mut final_results: Vec<UnifiedSearchResult> = all_results.into_values().collect();
        final_results.sort_by(|a, b| b.combined_score.partial_cmp(&a.combined_score).unwrap());
        final_results.truncate(query.limit);

        Ok(final_results)
    }

    /// Hybrid search combining all modalities
    pub async fn hybrid_search(&self, text: &str, visual_embedding: &EmbeddingVector, limit: usize) -> Result<Vec<UnifiedSearchResult>> {
        let query = MultimodalQuery {
            text_query: Some(text.to_string()),
            visual_embedding: Some(visual_embedding.clone()),
            graph_constraints: None,
            modality_weights: ModalityWeights::default(),
            limit,
        };

        self.search(query).await
    }

    fn add_results(
        &self,
        results: &mut HashMap<Uuid, UnifiedSearchResult>,
        text_results: Vec<super::text::SearchResult>,
        modality: &str,
        weight: f64,
    ) {
        for result in text_results {
            let entry = results.entry(result.document_id).or_insert(UnifiedSearchResult {
                document_id: result.document_id,
                combined_score: 0.0,
                modality_scores: HashMap::new(),
                content_preview: result.content_preview.clone(),
                metadata: result.metadata.clone(),
            });

            entry.modality_scores.insert(modality.to_string(), result.score * weight);
            entry.combined_score += result.score * weight;
        }
    }

    fn add_visual_results(
        &self,
        results: &mut HashMap<Uuid, UnifiedSearchResult>,
        visual_results: Vec<super::visual::VisualSearchResult>,
        weight: f64,
    ) {
        for result in visual_results {
            let entry = results.entry(result.document_id).or_insert(UnifiedSearchResult {
                document_id: result.document_id,
                combined_score: 0.0,
                modality_scores: HashMap::new(),
                content_preview: "Visual content".to_string(),
                metadata: result.metadata.clone(),
            });

            entry.modality_scores.insert("visual".to_string(), result.similarity_score * weight);
            entry.combined_score += result.similarity_score * weight;
        }
    }

    fn execute_graph_query(&self, graph_query: &GraphQuery) -> Vec<Uuid> {
        // Placeholder - would execute actual graph query
        if let Some(start) = graph_query.start_node {
            self.graph_indexer.get_neighbors(start)
        } else {
            Vec::new()
        }
    }

    fn apply_graph_filter(
        &self,
        results: &mut HashMap<Uuid, UnifiedSearchResult>,
        graph_nodes: &[Uuid],
        weight: f64,
    ) {
        let graph_node_set: std::collections::HashSet<_> = graph_nodes.iter().collect();

        for result in results.values_mut() {
            let graph_score = if graph_node_set.contains(&result.document_id) {
                1.0
            } else {
                0.0
            };

            result.modality_scores.insert("graph".to_string(), graph_score * weight);
            result.combined_score += graph_score * weight;
        }
    }
}

/// Search analytics and metrics
#[derive(Debug)]
pub struct SearchAnalytics {
    pub total_queries: u64,
    pub average_response_time_ms: f64,
    pub modality_usage: HashMap<String, u64>,
    pub result_quality_metrics: ResultQualityMetrics,
}

#[derive(Debug)]
pub struct ResultQualityMetrics {
    pub precision_at_1: f64,
    pub precision_at_5: f64,
    pub recall_at_10: f64,
    pub ndcg_score: f64,
}

impl Default for ResultQualityMetrics {
    fn default() -> Self {
        Self {
            precision_at_1: 0.0,
            precision_at_5: 0.0,
            recall_at_10: 0.0,
            ndcg_score: 0.0,
        }
    }
}


