//! Hybrid Search Implementation
//!
//! Combines vector similarity and text-based search for optimal retrieval.

use async_trait::async_trait;
use std::collections::HashMap;
use crate::vector_search::*;
use crate::MemoryResult;

/// Hybrid search strategy configuration
#[derive(Debug, Clone)]
pub struct HybridSearchConfig {
    pub vector_weight: f32,
    pub text_weight: f32,
    pub reciprocal_rank_fusion_k: f32,
    pub enable_adaptive_weighting: bool,
}

/// Hybrid search engine combining multiple strategies
pub struct HybridSearchEngine {
    vector_engine: Box<dyn VectorSearchEngine>,
    text_engine: Box<dyn VectorSearchEngine>,
    config: HybridSearchConfig,
}

impl HybridSearchEngine {
    pub fn new(
        vector_engine: Box<dyn VectorSearchEngine>,
        text_engine: Box<dyn VectorSearchEngine>,
        config: HybridSearchConfig,
    ) -> Self {
        Self {
            vector_engine,
            text_engine,
            config,
        }
    }

    /// Perform hybrid search with reciprocal rank fusion
    pub async fn search_with_rrf(&self, query: &SearchQuery, top_k: usize) -> MemoryResult<Vec<SearchResult>> {
        // Get results from both engines
        let vector_results = if let Some(embedding) = &query.embedding {
            self.vector_engine.vector_search(embedding, top_k * 2, &query.filters).await?
        } else {
            Vec::new()
        };

        let text_results = if let Some(text) = &query.text {
            self.text_engine.text_search(text, top_k * 2, &query.filters).await?
        } else {
            Vec::new()
        };

        // Combine using reciprocal rank fusion
        let fused_results = self.reciprocal_rank_fusion(&vector_results, &text_results, top_k);

        Ok(fused_results)
    }

    /// Reciprocal Rank Fusion algorithm
    fn reciprocal_rank_fusion(&self, list_a: &[SearchResult], list_b: &[SearchResult], top_k: usize) -> Vec<SearchResult> {
        let mut score_map: HashMap<crate::memory_types::MemoryId, (f32, usize, usize)> = HashMap::new();

        // Score from first list
        for (rank, result) in list_a.iter().enumerate() {
            score_map.entry(result.memory_id.clone())
                .or_insert((0.0, 0, 0))
                .0 += 1.0 / (rank as f32 + self.config.reciprocal_rank_fusion_k);
            score_map.get_mut(&result.memory_id).unwrap().1 = rank;
        }

        // Score from second list
        for (rank, result) in list_b.iter().enumerate() {
            score_map.entry(result.memory_id.clone())
                .or_insert((0.0, 0, 0))
                .0 += 1.0 / (rank as f32 + self.config.reciprocal_rank_fusion_k);
            score_map.get_mut(&result.memory_id).unwrap().2 = rank;
        }

        // Sort by combined score
        let mut results: Vec<(crate::memory_types::MemoryId, f32, usize, usize)> = score_map.into_iter()
            .map(|(id, (score, rank_a, rank_b))| (id, score, rank_a, rank_b))
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Convert back to SearchResult format
        results.into_iter()
            .take(top_k)
            .enumerate()
            .map(|(final_rank, (memory_id, score, rank_a, rank_b))| {
                // Find original result data (preferring vector results for metadata)
                let original_result = list_a.iter()
                    .find(|r| r.memory_id == memory_id)
                    .or_else(|| list_b.iter().find(|r| r.memory_id == memory_id))
                    .cloned()
                    .unwrap_or_else(|| SearchResult {
                        memory_id: memory_id.clone(),
                        score: 0.0,
                        rank: final_rank,
                        memory_data: serde_json::Value::Null,
                        metadata: SearchMetadata {
                            search_type: SearchType::Hybrid,
                            retrieved_at: chrono::Utc::now(),
                            processing_time_ms: 0,
                            vector_similarity: None,
                            text_similarity: None,
                        },
                    });

                SearchResult {
                    score,
                    rank: final_rank,
                    ..original_result
                }
            })
            .collect()
    }

    /// Adaptive weighting based on query characteristics
    pub async fn adaptive_search(&self, query: &SearchQuery, config: &VectorSearchConfig) -> MemoryResult<SearchResponse> {
        let start_time = std::time::Instant::now();

        let results = if self.config.enable_adaptive_weighting {
            self.adaptive_weighted_search(query, config).await?
        } else {
            self.search_with_rrf(query, config.default_top_k).await?
        };

        let search_time = start_time.elapsed().as_millis() as u64;

        let results_len = results.len();
        Ok(SearchResponse {
            query: query.clone(),
            results,
            total_found: results_len,
            search_time_ms: search_time,
            strategy_used: SearchStrategy::HybridReranking,
        })
    }

    /// Adaptive weighting based on query analysis
    async fn adaptive_weighted_search(&self, query: &SearchQuery, config: &VectorSearchConfig) -> MemoryResult<Vec<SearchResult>> {
        // Analyze query to determine optimal weights
        let (vector_weight, text_weight) = self.analyze_query_weights(query);

        // Get results from both engines
        let mut all_results = Vec::new();

        if let Some(embedding) = &query.embedding {
            let vector_results = self.vector_engine.vector_search(embedding, config.default_top_k, &query.filters).await?;
            all_results.extend(vector_results.into_iter().map(|r| (r, vector_weight)));
        }

        if let Some(text) = &query.text {
            let text_results = self.text_engine.text_search(text, config.default_top_k, &query.filters).await?;
            all_results.extend(text_results.into_iter().map(|r| (r, text_weight)));
        }

        // Combine and deduplicate results
        let mut result_map: HashMap<crate::memory_types::MemoryId, (SearchResult, f32)> = HashMap::new();

        for (result, weight) in all_results {
            result_map.entry(result.memory_id.clone())
                .and_modify(|(existing_result, total_weight)| {
                    // Weighted combination of scores
                    existing_result.score = existing_result.score * *total_weight + result.score * weight;
                    *total_weight += weight;
                    existing_result.score /= *total_weight;
                })
                .or_insert((result, weight));
        }

        // Sort by combined score
        let mut final_results: Vec<SearchResult> = result_map.into_iter()
            .map(|(_, (result, _))| result)
            .collect();

        final_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        Ok(final_results.into_iter().take(config.default_top_k).collect())
    }

    /// Analyze query to determine optimal search weights
    fn analyze_query_weights(&self, query: &SearchQuery) -> (f32, f32) {
        let mut vector_weight = self.config.vector_weight;
        let mut text_weight = self.config.text_weight;

        // Adjust weights based on query characteristics
        if let Some(text) = &query.text {
            // Longer queries tend to be more semantic, favor vector search
            if text.len() > 100 {
                vector_weight *= 1.2;
                text_weight *= 0.8;
            }
            // Short queries might be keywords, favor text search
            else if text.len() < 20 {
                vector_weight *= 0.8;
                text_weight *= 1.2;
            }
        }

        // Normalize weights
        let total = vector_weight + text_weight;
        (vector_weight / total, text_weight / total)
    }
}

#[async_trait]
impl VectorSearchEngine for HybridSearchEngine {
    async fn vector_search(&self, query_embedding: &[f32], top_k: usize, filters: &SearchFilters) -> MemoryResult<Vec<SearchResult>> {
        self.vector_engine.vector_search(query_embedding, top_k, filters).await
    }

    async fn text_search(&self, query_text: &str, top_k: usize, filters: &SearchFilters) -> MemoryResult<Vec<SearchResult>> {
        self.text_engine.text_search(query_text, top_k, filters).await
    }

    async fn hybrid_search(&self, query: &SearchQuery, config: &VectorSearchConfig) -> MemoryResult<SearchResponse> {
        self.adaptive_search(query, config).await
    }

    async fn index_memory(&self, memory_id: &crate::memory_types::MemoryId, embedding: &[f32], text_content: &str) -> MemoryResult<()> {
        // Index in both engines
        self.vector_engine.index_memory(memory_id, embedding, text_content).await?;
        self.text_engine.index_memory(memory_id, embedding, text_content).await?;
        Ok(())
    }

    async fn remove_from_index(&self, memory_id: &crate::memory_types::MemoryId) -> MemoryResult<()> {
        self.vector_engine.remove_from_index(memory_id).await?;
        self.text_engine.remove_from_index(memory_id).await?;
        Ok(())
    }

    async fn rebuild_index(&self) -> MemoryResult<()> {
        self.vector_engine.rebuild_index().await?;
        self.text_engine.rebuild_index().await?;
        Ok(())
    }

    async fn get_stats(&self) -> MemoryResult<SearchStats> {
        // Combine stats from both engines
        let vector_stats = self.vector_engine.get_stats().await?;
        let text_stats = self.text_engine.get_stats().await?;

        Ok(SearchStats {
            total_memories_indexed: vector_stats.total_memories_indexed.max(text_stats.total_memories_indexed),
            average_search_time_ms: (vector_stats.average_search_time_ms + text_stats.average_search_time_ms) / 2.0,
            cache_hit_rate: (vector_stats.cache_hit_rate + text_stats.cache_hit_rate) / 2.0,
            index_size_mb: vector_stats.index_size_mb + text_stats.index_size_mb,
            last_rebuild: vector_stats.last_rebuild.max(text_stats.last_rebuild),
        })
    }
}
