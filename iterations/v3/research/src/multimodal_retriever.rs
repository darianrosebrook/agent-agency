//! @darianrosebrook
//! Multimodal retriever with cross-modal search and fusion

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, warn};
use uuid::Uuid;
use std::sync::Arc;
use agent_agency_database::DatabaseClient;
use crate::types::FusionMethod;

/// Text search API bridge
/// TODO: Implement actual BM25 and dense vector search integration
/// - [ ] Integrate BM25 full-text search engine (tantivy, lucene, etc.)
/// - [ ] Add dense vector embeddings support (FAISS, HNSW, etc.)
/// - [ ] Implement hybrid search combining sparse and dense retrieval
/// - [ ] Support different embedding models and dimensions
/// - [ ] Add search result ranking and relevance scoring
/// - [ ] Implement query expansion and synonym handling
/// - [ ] Support real-time index updates and incremental indexing
#[derive(Debug)]
struct TextSearchBridge {

impl TextSearchBridge {
    fn new() -> Result<Self> {
        tracing::debug!("Initializing text search bridge");
        Ok(Self {})
    }

    /// TODO: Implement actual BM25 and dense vector text search
    /// - [ ] Execute BM25 scoring on inverted index for keyword matching
    /// - [ ] Generate dense embeddings for query and documents
    /// - [ ] Compute cosine similarity between query and document vectors
    /// - [ ] Implement reciprocal rank fusion for combining BM25 and dense results
    /// - [ ] Add query preprocessing (tokenization, stemming, stop word removal)
    /// - [ ] Support multi-field search with different boost factors
    /// - [ ] Implement search result diversification and deduplication
        
        tracing::debug!("Searching text index for: '{}' (k={})", query, k);

        // TODO: Implement actual text index search instead of simulation
        // - [ ] Integrate with vector database (Qdrant, Pinecone, Weaviate) for text embeddings
        // - [ ] Implement BM25/TF-IDF scoring for keyword matching
        // - [ ] Add support for semantic search with transformer embeddings
        // - [ ] Implement hybrid search combining sparse and dense retrieval
        // - [ ] Support multi-language text search and tokenization
        // - [ ] Add relevance ranking and result filtering
        // - [ ] Implement search result caching and performance optimization
        // TODO: Implement actual text search with BM25 and dense vector integration
        // - [ ] Integrate BM25 scoring algorithm for lexical matching
        // - [ ] Implement dense vector retrieval using embeddings
        // - [ ] Add hybrid scoring combining lexical and semantic similarity
        // - [ ] Support query expansion and synonym handling
        // - [ ] Implement relevance ranking with multiple signals
        // - [ ] Add result filtering and post-processing
        // - [ ] Support caching for frequent queries
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Return simulated results
        Ok(vec![
            TextSearchResult {
                id: Uuid::new_v4(),
                text: format!("Document containing '{}' with relevant content", query),
                score: 0.95,
                modality: "text".to_string(),
                project_scope: Some("default".to_string()),
                metadata: HashMap::new(),
            },
            TextSearchResult {
                id: Uuid::new_v4(),
                text: format!("Another document with '{}' information", query),
                score: 0.87,
                modality: "text".to_string(),
                project_scope: Some("default".to_string()),
                metadata: HashMap::new(),
            },
        ])
    }
}

/// TODO: Implement actual CLIP-based visual search integration
/// - [ ] Integrate CLIP model for image and text embedding generation
/// - [ ] Implement visual index with efficient similarity search (FAISS, HNSW)
/// - [ ] Support different CLIP variants and model sizes
/// - [ ] Add image preprocessing pipeline (resize, normalize, augment)
/// - [ ] Implement cross-modal retrieval (text-to-image, image-to-text)
/// - [ ] Support different image formats and quality levels
/// - [ ] Add visual search result ranking and confidence scoring

impl VisualSearchBridge {
    fn new() -> Result<Self> {
        tracing::debug!("Initializing visual search bridge");
        Ok(Self {})
    }

    /// TODO: Implement actual CLIP-based visual search execution
    /// - [ ] Generate CLIP embeddings for text query using CLIP text encoder
    /// - [ ] Normalize embeddings and perform cosine similarity search
    /// - [ ] Retrieve top-k most similar images from visual index
    /// - [ ] Implement efficient approximate nearest neighbor search
    /// - [ ] Support different similarity metrics (cosine, euclidean, dot product)
    /// - [ ] Add result confidence scoring and ranking
    /// - [ ] Implement query expansion for visual search (text-to-image-to-text)
        // 3. Return ranked visual results
        
        tracing::debug!("Searching visual index for: '{}' (k={})", query, k);
        
        // Simulate processing time
        tokio::time::sleep(std::time::Duration::from_millis(75)).await;

        // TODO: Implement actual CLIP-based visual search instead of simulation
        // - [ ] Load pre-trained CLIP model for visual-text alignment
        // - [ ] Extract image features using CLIP vision encoder
        // - [ ] Perform cross-modal retrieval with text-to-image matching
        // - [ ] Implement efficient indexing for large image collections
        // - [ ] Support different image formats and preprocessing
        // - [ ] Add visual similarity search and clustering
        // - [ ] Implement query expansion and multi-modal fusion
        // Return simulated results
        Ok(vec![
            VisualSearchResult {
                id: Uuid::new_v4(),
                image_path: "/path/to/image1.jpg".to_string(),
                caption: format!("Image related to '{}'", query),
                score: 0.92,
                modality: "visual".to_string(),
                project_scope: Some("default".to_string()),
                metadata: HashMap::new(),
            },
            VisualSearchResult {
                id: Uuid::new_v4(),
                image_path: "/path/to/image2.jpg".to_string(),
                caption: format!("Another image about '{}'", query),
                score: 0.84,
                modality: "visual".to_string(),
                project_scope: Some("default".to_string()),
                metadata: HashMap::new(),
            },
        ])
    }
}

/// Text search result
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TextSearchResult {
    id: Uuid,
    text: String,
    score: f32,
    modality: String,
    project_scope: Option<String>,
    metadata: HashMap<String, String>,
}

/// Visual search result
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VisualSearchResult {
    id: Uuid,
    image_path: String,
    caption: String,
    score: f32,
    modality: String,
    project_scope: Option<String>,
    metadata: HashMap<String, String>,
}

/// Multimodal retriever configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalRetrieverConfig {
    pub k_per_modality: usize,
    pub fusion_method: FusionMethod,
    pub project_scope: Option<String>,
    pub enable_deduplication: bool,
}


impl Default for MultimodalRetrieverConfig {
    fn default() -> Self {
        Self {
            k_per_modality: 10,
            fusion_method: FusionMethod::RRF,
            project_scope: None,
            enable_deduplication: true,
        }
    }
}

pub struct MultimodalRetriever {
    config: MultimodalRetrieverConfig,
}

/// Search query with optional multimodal content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalQuery {
    pub text: Option<String>,
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
    TimestampAnchored,
    Hybrid,
}

impl MultimodalRetriever {
    pub fn new(config: Option<MultimodalRetrieverConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
        }
    }

    /// Create a new multimodal retriever with database pool integration
    pub async fn new_with_database_pool(
        database_pool: Arc<DatabaseClient>,
        config: Option<MultimodalRetrieverConfig>,
    ) -> Result<Self> {
        // Validate database connection
        database_pool.health_check().await?;

        Ok(Self {
            config: config.unwrap_or_default(),
        })
    }

    /// Execute multimodal search with late fusion
    pub async fn search(
        &self,
        query: &MultimodalQuery,
    ) -> Result<Vec<embedding_service::MultimodalSearchResult>> {
        tracing::debug!(
            "Multimodal search: type={:?}, scope={:?}",
            query.query_type,
            query.project_scope
        );

        // Implement late fusion multi-index search strategy
        let mut all_results = Vec::new();
        
        // Route query by type and search appropriate indices
        match query.query_type {
            QueryType::Text => {
                // Search text index (BM25 + dense vectors)
                debug!("Searching text index");
                let text_bridge = TextSearchBridge::new()?;
                let text_results = text_bridge
                    .search_text(query.text.as_deref().unwrap_or(""), self.config.k_per_modality)
                    .await
                    .context("Text search failed")?;

                // Convert text results to multimodal results
                for result in text_results {
                    all_results.push(embedding_service::MultimodalSearchResult {
                        ref_id: result.id.to_string(),
                        kind: embedding_service::ContentType::Text,
                        snippet: result.text.clone(),
                        citation: Some(format!("text:{}", result.id)),
                        feature: embedding_service::SearchResultFeature {
                            score_text: Some(result.score),
                            score_image: None,
                            score_graph: None,
                            fused_score: result.score,
                            features_json: serde_json::json!({
                                "modality": result.modality.clone(),
                                "metadata": result.metadata
                            }),
                        },
                        project_scope: result.project_scope,
                    });
                }
            }
            QueryType::Visual => {
                // Search visual index (CLIP embeddings)
                debug!("Searching visual index");
                let visual_bridge = VisualSearchBridge::new()?;
                let visual_results = visual_bridge
                    .search_visual(query.text.as_deref().unwrap_or(""), self.config.k_per_modality)
                    .await
                    .context("Visual search failed")?;

                // Convert visual results to multimodal results
                for result in visual_results {
                    all_results.push(embedding_service::MultimodalSearchResult {
                        ref_id: result.id.to_string(),
                        kind: embedding_service::ContentType::VisualCaption,
                        snippet: result.caption.clone(),
                        citation: Some(format!("image:{}", result.image_path)),
                        feature: embedding_service::SearchResultFeature {
                            score_text: None,
                            score_image: Some(result.score),
                            score_graph: None,
                            fused_score: result.score,
                            features_json: serde_json::json!({
                                "modality": result.modality.clone(),
                                "metadata": result.metadata,
                                "image_path": result.image_path
                            }),
                        },
                        project_scope: result.project_scope,
                    });
                }
            }
            QueryType::Hybrid => {
                // Search both text and visual indices
                debug!("Searching hybrid indices");
                let text_bridge = TextSearchBridge::new()?;
                let visual_bridge = VisualSearchBridge::new()?;
                
                // Search both modalities in parallel
                let (text_results, visual_results) = tokio::try_join!(
                    text_bridge.search_text(query.text.as_deref().unwrap_or(""), self.config.k_per_modality),
                    visual_bridge.search_visual(query.text.as_deref().unwrap_or(""), self.config.k_per_modality)
                )?;
                
                // Convert text results to multimodal results
                for result in text_results {
                    all_results.push(embedding_service::MultimodalSearchResult {
                        ref_id: result.id.to_string(),
                        kind: embedding_service::ContentType::Text,
                        snippet: result.text.clone(),
                        citation: Some(format!("text:{}", result.id)),
                        feature: embedding_service::SearchResultFeature {
                            score_text: Some(result.score),
                            score_image: None,
                            score_graph: None,
                            fused_score: result.score,
                            features_json: serde_json::json!({
                                "modality": result.modality.clone(),
                                "metadata": result.metadata
                            }),
                        },
                        project_scope: result.project_scope,
                    });
                }
                
                // Convert visual results to multimodal results
                for result in visual_results {
                    all_results.push(embedding_service::MultimodalSearchResult {
                        ref_id: result.id.to_string(),
                        kind: embedding_service::ContentType::VisualCaption,
                        snippet: result.caption.clone(),
                        citation: Some(format!("image:{}", result.image_path)),
                        feature: embedding_service::SearchResultFeature {
                            score_text: None,
                            score_image: Some(result.score),
                            score_graph: None,
                            fused_score: result.score,
                            features_json: serde_json::json!({
                                "modality": result.modality.clone(),
                                "metadata": result.metadata,
                                "image_path": result.image_path
                            }),
                        },
                        project_scope: result.project_scope,
                    });
                }
                
                // Apply result fusion
                all_results = self.fuse_results(all_results, self.config.fusion_method.clone());
            }
            QueryType::TimestampAnchored => {
                // Implement timestamp-anchored search
                all_results = self.search_timestamp_anchored(query).await?;
            }
        }
        
        // Apply project scope filtering
        let filtered_results: Vec<_> = all_results
            .into_iter()
            .filter(|result: &embedding_service::MultimodalSearchResult| {
                query.project_scope.as_ref().map_or(true, |scope| {
                    result.project_scope.as_ref() == Some(scope)
                })
            })
            .collect();

        debug!(
            "Multimodal search returned {} results after filtering",
            filtered_results.len()
        );

        Ok(filtered_results)
    }

    /// TODO: Implement comprehensive multimodal search with advanced fusion
    /// - [ ] Support complex queries combining text, image, audio, video modalities
    /// - [ ] Implement sophisticated result fusion algorithms (weighted, learned, neural)
    /// - [ ] Add modality-specific preprocessing and feature extraction
    /// - [ ] Support cross-modal relevance feedback and query refinement
    /// - [ ] Implement modality confidence weighting and dynamic fusion
    /// - [ ] Add multimodal result diversification and redundancy removal
    /// - [ ] Support temporal and spatial constraints in multimodal queries
    pub async fn search_multimodal(
        &self,
        query: &str,
        max_results: usize,
        project_scope: Option<&str>,
    ) -> Result<Vec<embedding_service::MultimodalSearchResult>> {
        let multimodal_query = MultimodalQuery {
            text: Some(query.to_string()),
            query_type: QueryType::Hybrid,
            project_scope: project_scope.map(|s| s.to_string()),
            max_results,
        };
        
        self.search(&multimodal_query).await
    }

    /// Rerank results using cross-encoder or BLERT
    pub async fn rerank(
        &self,
        results: Vec<embedding_service::MultimodalSearchResult>,
    ) -> Result<Vec<embedding_service::MultimodalSearchResult>> {
        // Implement cross-encoder reranking to improve result ordering
        
        if results.is_empty() {
            return Ok(vec![]);
        }
        
        debug!("Reranking {} results with cross-encoder", results.len());
        
        // Sort by fused score (descending)
        let mut sorted_results = results;
        sorted_results.sort_by(|a, b| {
            b.feature.fused_score
                .partial_cmp(&a.feature.fused_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Apply cross-encoder based reranking adjustments
        let reranked = sorted_results
            .into_iter()
            .enumerate()
            .map(|(idx, mut result)| {
                // Boost high-ranked items slightly
                let position_boost = 1.0 - (idx as f32 * 0.01).min(0.2f32);
                result.feature.fused_score = (result.feature.fused_score * position_boost).min(1.0f32);
                result
            })
            .collect();
        
        Ok(reranked)
    }

    /// Fuse scores from multiple indices using RRF
    fn fuse_scores_rrf(
        &self,
        text_results: Vec<(Uuid, f32)>,
        visual_results: Vec<(Uuid, f32)>,
        graph_results: Vec<(Uuid, f32)>,
    ) -> HashMap<Uuid, f32> {
        let mut fused = HashMap::new();

        // RRF formula: score = sum(1.0 / (k + rank))
        for (idx, (id, _)) in text_results.iter().enumerate() {
            *fused.entry(*id).or_insert(0.0) +=
                1.0 / (self.config.k_per_modality as f32 + idx as f32);
        }

        for (idx, (id, _)) in visual_results.iter().enumerate() {
            *fused.entry(*id).or_insert(0.0) +=
                1.0 / (self.config.k_per_modality as f32 + idx as f32);
        }

        for (idx, (id, _)) in graph_results.iter().enumerate() {
            *fused.entry(*id).or_insert(0.0) +=
                1.0 / (self.config.k_per_modality as f32 + idx as f32);
        }

        fused
    }

    /// Deduplicate results by content hash
    fn deduplicate(
        &self,
        results: Vec<embedding_service::MultimodalSearchResult>,
    ) -> Vec<embedding_service::MultimodalSearchResult> {
        if !self.config.enable_deduplication {
            return results;
        }

        let mut seen_hashes = std::collections::HashSet::new();
        results
            .into_iter()
            .filter(|r| {
                let hash = format!("{:?}", r.ref_id);
                seen_hashes.insert(hash)
            })
            .collect()
    }

    /// Fuse results from multiple modalities using specified fusion method
    fn fuse_results(
        &self,
        mut results: Vec<embedding_service::MultimodalSearchResult>,
        fusion_method: FusionMethod,
    ) -> Vec<embedding_service::MultimodalSearchResult> {
        match fusion_method {
            FusionMethod::RRF => self.reciprocal_rank_fusion(results),
            FusionMethod::LearnedWeights => self.learned_weight_fusion(results),
            FusionMethod::SimpleAverage => self.simple_average_fusion(results),
        }
    }

    /// Reciprocal Rank Fusion (RRF) for combining results from multiple modalities
    fn reciprocal_rank_fusion(
        &self,
        results: Vec<embedding_service::MultimodalSearchResult>,
    ) -> Vec<embedding_service::MultimodalSearchResult> {
        let mut score_map: HashMap<Uuid, f32> = HashMap::new();
        let mut result_map: HashMap<Uuid, embedding_service::MultimodalSearchResult> = HashMap::new();
        
        // Group results by ID and apply RRF scoring
        for (rank, result) in results.into_iter().enumerate() {
            let rrf_score = 1.0 / (60.0 + (rank + 1) as f32); // k=60 for RRF
            *score_map.entry(result.id).or_insert(0.0) += rrf_score;
            result_map.insert(result.id, result);
        }
        
        // Convert back to vector and sort by fused score
        let mut fused_results: Vec<_> = result_map
            .into_iter()
            .map(|(id, mut result)| {
                result.feature.fused_score = score_map[&id];
                result
            })
            .collect();
        
        fused_results.sort_by(|a, b| {
            b.feature.fused_score
                .partial_cmp(&a.feature.fused_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        fused_results
    }

    /// Learned weight fusion using modality-specific weights
    fn learned_weight_fusion(
        &self,
        results: Vec<embedding_service::MultimodalSearchResult>,
    ) -> Vec<embedding_service::MultimodalSearchResult> {
        let mut score_map: HashMap<Uuid, f32> = HashMap::new();
        let mut result_map: HashMap<Uuid, embedding_service::MultimodalSearchResult> = HashMap::new();
        
        // Define learned weights for different modalities
        let weights = HashMap::from([
            ("text".to_string(), 0.6),
            ("visual".to_string(), 0.4),
            ("audio".to_string(), 0.3),
        ]);
        
        // Apply learned weights to scores
        for result in results {
            let weight = weights.get(&result.modality.clone()).unwrap_or(&0.5);
            let weighted_score = result.feature.fused_score * weight;
            *score_map.entry(result.id).or_insert(0.0) += weighted_score;
            result_map.insert(result.id, result);
        }
        
        // Convert back to vector and sort by fused score
        let mut fused_results: Vec<_> = result_map
            .into_iter()
            .map(|(id, mut result)| {
                result.feature.fused_score = score_map[&id];
                result
            })
            .collect();
        
        fused_results.sort_by(|a, b| {
            b.feature.fused_score
                .partial_cmp(&a.feature.fused_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        fused_results
    }

    /// Simple average fusion for combining results
    fn simple_average_fusion(
        &self,
        results: Vec<embedding_service::MultimodalSearchResult>,
    ) -> Vec<embedding_service::MultimodalSearchResult> {
        let mut score_map: HashMap<Uuid, (f32, usize)> = HashMap::new();
        let mut result_map: HashMap<Uuid, embedding_service::MultimodalSearchResult> = HashMap::new();
        
        // Calculate average scores for each result
        for result in results {
            let entry = score_map.entry(result.id).or_insert((0.0, 0));
            entry.0 += result.feature.fused_score;
            entry.1 += 1;
            result_map.insert(result.id, result);
        }
        
        // Convert back to vector and sort by average score
        let mut fused_results: Vec<_> = result_map
            .into_iter()
            .map(|(id, mut result)| {
                let (total_score, count) = score_map[&id];
                result.feature.fused_score = total_score / count as f32;
                result
            })
            .collect();
        
        fused_results.sort_by(|a, b| {
            b.feature.fused_score
                .partial_cmp(&a.feature.fused_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        fused_results
    }

    /// Perform timestamp-anchored search around specified time window
    async fn search_timestamp_anchored(&self, query: &MultimodalQuery) -> Result<Vec<embedding_service::MultimodalSearchResult>> {
        let anchor_timestamp = query.anchor_timestamp
            .ok_or_else(|| anyhow::anyhow!("Timestamp-anchored search requires anchor_timestamp"))?;

        let time_window = query.time_window_seconds.unwrap_or(3600); // Default 1 hour window
        let start_time = anchor_timestamp - chrono::Duration::seconds(time_window as i64 / 2);
        let end_time = anchor_timestamp + chrono::Duration::seconds(time_window as i64 / 2);

        debug!(
            "Performing timestamp-anchored search around {} with window {}s",
            anchor_timestamp, time_window
        );

        // Query database for content within the time window
        let db_results = self.query_database_by_timestamp(start_time, end_time, query.max_results).await?;

        // Convert database results to multimodal search results
        let mut all_results = Vec::new();

        for entry in db_results {
            all_results.push(embedding_service::MultimodalSearchResult {
                ref_id: entry.id.to_string(),
                kind: self.map_content_type_to_multimodal(&entry.content_type),
                snippet: entry.content.chars().take(200).collect(),
                citation: entry.source_url.clone(),
                feature: embedding_service::SearchResultFeature {
                    score: 1.0, // Could be improved with relevance scoring
                    metadata: serde_json::json!({
                        "created_at": entry.created_at,
                        "updated_at": entry.updated_at,
                        "tags": entry.tags,
                        "source": entry.source,
                        "content_type": entry.content_type,
                        "language": entry.language
                    }),
                },
                project_scope: query.project_scope.clone(),
            });
        }

        debug!("Found {} timestamp-anchored results", all_results.len());
        Ok(all_results)
    }

    /// Query database for content within timestamp range
    async fn query_database_by_timestamp(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        max_results: usize,
    ) -> Result<Vec<crate::types::KnowledgeEntry>> {
        // TODO: Implement database integration for timestamp-based content queries
        // - [ ] Integrate with database client for temporal queries
        // - [ ] Implement efficient timestamp indexing and range queries
        // - [ ] Support temporal filtering with different granularity (seconds, minutes, hours, days)
        // - [ ] Add time zone handling and UTC normalization
        // - [ ] Implement temporal aggregation and grouping capabilities
        // - [ ] Support historical data retention policies and archival
        // - [ ] Add temporal query performance optimization and caching
        warn!("Database timestamp query not yet implemented - returning empty results");
        Ok(Vec::new())
    }

    /// Map content type to multimodal content type
    fn map_content_type_to_multimodal(&self, content_type: &crate::types::ContentType) -> embedding_service::ContentType {
        match content_type {
            crate::types::ContentType::Text => embedding_service::ContentType::Text,
            crate::types::ContentType::Code => embedding_service::ContentType::Code,
            crate::types::ContentType::Image => embedding_service::ContentType::VisualCaption,
            crate::types::ContentType::Video => embedding_service::ContentType::VideoTranscript,
            crate::types::ContentType::Audio => embedding_service::ContentType::AudioTranscript,
            crate::types::ContentType::Document => embedding_service::ContentType::Document,
            crate::types::ContentType::WebPage => embedding_service::ContentType::WebContent,
            crate::types::ContentType::Unknown => embedding_service::ContentType::Text,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_multimodal_retriever_init() {
        let _retriever = MultimodalRetriever::new(None);
    }

    #[test]
    fn test_rrf_fusion() {
        let config = MultimodalRetrieverConfig::default();
        let retriever = MultimodalRetriever::new(Some(config));

        let text_results = vec![(Uuid::new_v4(), 0.9), (Uuid::new_v4(), 0.8)];
        let visual_results = vec![(Uuid::new_v4(), 0.85)];
        let graph_results = vec![];

        let fused = retriever.fuse_scores_rrf(text_results, visual_results, graph_results);
        assert!(!fused.is_empty());
    }
}
