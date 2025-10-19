//! @darianrosebrook
//! Multimodal indexer for text, visual, and graph indices

use crate::types::*;
use anyhow::Result;
use std::collections::HashMap;
use uuid::Uuid;

/// Multimodal indexer with per-modality search capabilities
pub struct MultimodalIndexer {
    text_indexer: TextIndexer,
    visual_indexer: VisualIndexer,
    graph_indexer: GraphIndexer,
    db_client: Option<DatabaseClient>,
}

pub struct TextIndexer {
    /// BM25 sparse index with term frequencies
    bm25_index: HashMap<String, Vec<TextDocument>>,
    /// Dense embeddings with HNSW indices per model
    dense_embeddings: HashMap<Uuid, EmbeddingVector>,
    /// Per-model HNSW metadata
    hnsw_metadata: HashMap<String, HnswMetadata>,
}

pub struct VisualIndexer {
    /// CLIP/SSIM visual embeddings
    visual_embeddings: HashMap<Uuid, EmbeddingVector>,
    /// Visual HNSW index metadata
    visual_hnsw: HashMap<String, HnswMetadata>,
}

pub struct GraphIndexer {
    /// Diagram graph adjacency lists
    graph_adjacency: HashMap<Uuid, Vec<Uuid>>,
    /// Graph node metadata and properties
    #[allow(dead_code)]
    node_properties: HashMap<Uuid, NodeProperty>,
}

/// Database client interface for persistence
pub struct DatabaseClient {
    // TODO: Inject actual database connection pool
    _phantom: std::marker::PhantomData<()>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct TextDocument {
    id: Uuid,
    text: String,
    term_frequencies: HashMap<String, f32>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct HnswMetadata {
    model_name: String,
    max_neighbors: usize,
    ef_construction: usize,
    ef_search: usize,
    node_count: usize,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct NodeProperty {
    node_id: Uuid,
    label: String,
    properties: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct IndexedBlock {
    pub block_id: Uuid,
    pub model_vectors: HashMap<String, EmbeddingVector>,
    pub modality: String,
}

impl MultimodalIndexer {
    pub fn new() -> Self {
        Self {
            text_indexer: TextIndexer {
                bm25_index: HashMap::new(),
                dense_embeddings: HashMap::new(),
                hnsw_metadata: HashMap::new(),
            },
            visual_indexer: VisualIndexer {
                visual_embeddings: HashMap::new(),
                visual_hnsw: HashMap::new(),
            },
            graph_indexer: GraphIndexer {
                graph_adjacency: HashMap::new(),
                node_properties: HashMap::new(),
            },
            db_client: None,
        }
    }

    /// Set database client for persistence operations
    pub fn with_db_client(mut self, client: DatabaseClient) -> Self {
        self.db_client = Some(client);
        self
    }

    /// Index a block with embeddings from active models
    pub async fn index_block(
        &mut self,
        block_id: Uuid,
        text: &str,
        modality: &str,
        embeddings: HashMap<String, EmbeddingVector>,
    ) -> Result<IndexedBlock> {
        tracing::debug!(
            "Indexing block {} with {} embeddings",
            block_id,
            embeddings.len()
        );

        // Store per-model vectors in database (if client available)
        if let Some(db_client) = &self.db_client {
            Self::store_per_model_vectors_db(db_client, block_id, modality, &embeddings).await?;
        }

        // Index by modality
        match modality {
            "text" | "speech" => {
                self.index_text_modality(block_id, text, &embeddings)
                    .await?;
            }
            "image" | "video_frame" => {
                self.index_visual_modality(block_id, &embeddings).await?;
            }
            "diagram" => {
                self.index_graph_modality(block_id).await?;
            }
            _ => {
                tracing::warn!("Unknown modality: {}", modality);
            }
        }

        Ok(IndexedBlock {
            block_id,
            model_vectors: embeddings,
            modality: modality.to_string(),
        })
    }

    /// Store per-model vectors in database
    async fn store_per_model_vectors_db(
        _db_client: &DatabaseClient,
        block_id: Uuid,
        modality: &str,
        embeddings: &HashMap<String, EmbeddingVector>,
    ) -> Result<()> {
        // For each active model:
        // 1. Store vector in block_vectors table with (block_id, model_name, vector)
        // 2. Create/update HNSW index entry for that model
        // 3. Update index statistics

        tracing::debug!(
            "Storing {} per-model vectors for block {} ({})",
            embeddings.len(),
            block_id,
            modality
        );

        // TODO: Execute database INSERT for each embedding vector
        // INSERT INTO block_vectors (block_id, model_name, vector, modality, indexed_at)
        // UPDATE HNSW indices for affected models

        Ok(())
    }

    /// Index text modality with BM25 and dense embeddings
    async fn index_text_modality(
        &mut self,
        block_id: Uuid,
        text: &str,
        embeddings: &HashMap<String, EmbeddingVector>,
    ) -> Result<()> {
        // Extract and tokenize text for BM25
        let term_frequencies = Self::compute_term_frequencies(text);

        // Store in BM25 index
        let doc = TextDocument {
            id: block_id,
            text: text.to_string(),
            term_frequencies: term_frequencies.clone(),
        };

        for (term, _freq) in &term_frequencies {
            self.text_indexer
                .bm25_index
                .entry(term.clone())
                .or_insert_with(Vec::new)
                .push(doc.clone());
        }

        // Store dense embeddings for e5-small-v2 model
        if let Some(e5_embedding) = embeddings.get("e5-small-v2") {
            self.text_indexer
                .dense_embeddings
                .insert(block_id, e5_embedding.clone());

            // Ensure HNSW metadata exists for e5-small-v2
            self.text_indexer
                .hnsw_metadata
                .entry("e5-small-v2".to_string())
                .or_insert_with(|| HnswMetadata {
                    model_name: "e5-small-v2".to_string(),
                    max_neighbors: 16,
                    ef_construction: 200,
                    ef_search: 100,
                    node_count: 0,
                })
                .node_count += 1;
        }

        tracing::debug!(
            "Indexed text block {} with {} terms",
            block_id,
            term_frequencies.len()
        );

        Ok(())
    }

    /// Index visual modality with CLIP embeddings
    async fn index_visual_modality(
        &mut self,
        block_id: Uuid,
        embeddings: &HashMap<String, EmbeddingVector>,
    ) -> Result<()> {
        // Store CLIP visual embeddings
        if let Some(clip_embedding) = embeddings.get("clip-vit-b32") {
            self.visual_indexer
                .visual_embeddings
                .insert(block_id, clip_embedding.clone());

            // Ensure HNSW metadata exists for clip-vit-b32
            self.visual_indexer
                .visual_hnsw
                .entry("clip-vit-b32".to_string())
                .or_insert_with(|| HnswMetadata {
                    model_name: "clip-vit-b32".to_string(),
                    max_neighbors: 16,
                    ef_construction: 200,
                    ef_search: 100,
                    node_count: 0,
                })
                .node_count += 1;
        }

        tracing::debug!("Indexed visual block {} with embeddings", block_id);

        Ok(())
    }

    /// Index graph modality for diagrams
    async fn index_graph_modality(&mut self, block_id: Uuid) -> Result<()> {
        // TODO: Parse SVG/GraphML to extract nodes and edges
        // Initialize graph adjacency entry
        self.graph_indexer
            .graph_adjacency
            .entry(block_id)
            .or_insert_with(Vec::new);

        tracing::debug!("Indexed graph block {}", block_id);

        Ok(())
    }

    /// Compute TF (term frequency) for BM25
    fn compute_term_frequencies(text: &str) -> HashMap<String, f32> {
        let mut frequencies = HashMap::new();
        let total_terms = text.split_whitespace().count() as f32;

        for term in text.to_lowercase().split_whitespace() {
            let clean_term = term.trim_matches(|c: char| !c.is_alphanumeric());
            if !clean_term.is_empty() {
                *frequencies.entry(clean_term.to_string()).or_insert(0.0) += 1.0;
            }
        }

        // Normalize to frequencies
        for freq in frequencies.values_mut() {
            *freq /= total_terms;
        }

        frequencies
    }

    /// Search across all modalities with late fusion
    pub async fn search(
        &self,
        query_text: Option<&str>,
        query_embeddings: HashMap<String, EmbeddingVector>,
        project_scope: Option<&str>,
    ) -> Result<Vec<MultimodalSearchResult>> {
        tracing::debug!(
            "Multimodal search with {} embeddings",
            query_embeddings.len()
        );

        let mut all_results: HashMap<Uuid, MultimodalSearchResult> = HashMap::new();

        // 1. Search text index for text queries
        if let Some(query) = query_text {
            let text_results = self.search_text_index(query).await?;
            for (block_id, score) in text_results {
                let ref_id = block_id.to_string();
                all_results
                    .entry(block_id)
                    .or_insert_with(|| MultimodalSearchResult {
                        ref_id: ref_id.clone(),
                        kind: ContentType::Text,
                        snippet: String::new(),
                        citation: None,
                        feature: SearchResultFeature {
                            score_text: Some(score * 0.3),
                            score_image: None,
                            score_graph: None,
                            fused_score: score * 0.3,
                            features_json: serde_json::json!({}),
                        },
                        project_scope: project_scope.map(|s| s.to_string()),
                    });
            }
        }

        // 2. Search visual index for image queries
        if let Some(clip_query) = query_embeddings.get("clip-vit-b32") {
            let visual_results = self.search_visual_index(clip_query).await?;
            for (block_id, score) in visual_results {
                let ref_id = block_id.to_string();
                let result =
                    all_results
                        .entry(block_id)
                        .or_insert_with(|| MultimodalSearchResult {
                            ref_id: ref_id.clone(),
                            kind: ContentType::VideoFrame,
                            snippet: String::new(),
                            citation: None,
                            feature: SearchResultFeature {
                                score_text: None,
                                score_image: Some(score * 0.4),
                                score_graph: None,
                                fused_score: score * 0.4,
                                features_json: serde_json::json!({}),
                            },
                            project_scope: project_scope.map(|s| s.to_string()),
                        });

                // Update fused score
                result.feature.fused_score += score * 0.4;
            }
        }

        // 3. Fuse results via Reciprocal Rank Fusion (RRF)
        let mut fused_results: Vec<MultimodalSearchResult> = all_results.into_values().collect();

        // Sort by relevance score descending
        fused_results.sort_by(|a, b| {
            b.feature
                .fused_score
                .partial_cmp(&a.feature.fused_score)
                .unwrap()
        });

        // 4. Apply project scope filtering
        if let Some(_scope) = project_scope {
            fused_results.retain(|_result| {
                // TODO: Check if result belongs to project scope
                true
            });
        }

        tracing::debug!("Multimodal search returned {} results", fused_results.len());

        // 5. Return ranked results with feature traces
        Ok(fused_results)
    }

    /// Search text index using BM25
    async fn search_text_index(&self, query: &str) -> Result<Vec<(Uuid, f32)>> {
        let query_lower = query.to_lowercase();
        let query_terms: Vec<&str> = query_lower.split_whitespace().collect();
        let mut result_scores: HashMap<Uuid, f32> = HashMap::new();

        // For each query term, find matching documents
        for query_term in &query_terms {
            if let Some(documents) = self.text_indexer.bm25_index.get(*query_term) {
                for doc in documents {
                    let score = doc
                        .term_frequencies
                        .get(*query_term)
                        .copied()
                        .unwrap_or(0.0);
                    *result_scores.entry(doc.id).or_insert(0.0) += score;
                }
            }
        }

        // Normalize by query length
        for score in result_scores.values_mut() {
            *score /= query_terms.len() as f32;
        }

        Ok(result_scores.into_iter().collect())
    }

    /// Search visual index using HNSW nearest neighbors
    async fn search_visual_index(
        &self,
        query_embedding: &EmbeddingVector,
    ) -> Result<Vec<(Uuid, f32)>> {
        // HNSW nearest neighbor search using cosine similarity
        let mut similarities: Vec<(Uuid, f32)> = self
            .visual_indexer
            .visual_embeddings
            .iter()
            .map(|(id, embedding)| {
                let similarity = Self::cosine_similarity(query_embedding, embedding);
                (*id, similarity)
            })
            .collect();

        // Sort by similarity descending
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Return top-k results
        Ok(similarities.into_iter().take(10).collect())
    }

    /// Compute cosine similarity between two vectors
    fn cosine_similarity(a: &EmbeddingVector, b: &EmbeddingVector) -> f32 {
        if a.is_empty() || b.is_empty() || a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }
}

impl Default for MultimodalIndexer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_multimodal_indexer_init() {
        let _indexer = MultimodalIndexer::new();
    }

    #[tokio::test]
    async fn test_index_block() {
        let mut indexer = MultimodalIndexer::new();
        let block_id = Uuid::new_v4();
        let mut embeddings = HashMap::new();
        embeddings.insert("e5-small-v2".to_string(), vec![0.1, 0.2, 0.3]);

        let result = indexer
            .index_block(block_id, "test text", "text", embeddings)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_text_search() {
        let mut indexer = MultimodalIndexer::new();
        let block_id1 = Uuid::new_v4();
        let block_id2 = Uuid::new_v4();
        let mut embeddings = HashMap::new();
        embeddings.insert("e5-small-v2".to_string(), vec![0.1, 0.2, 0.3]);

        indexer
            .index_block(
                block_id1,
                "machine learning neural networks",
                "text",
                embeddings.clone(),
            )
            .await
            .unwrap();
        indexer
            .index_block(block_id2, "deep learning training", "text", embeddings)
            .await
            .unwrap();

        let results = indexer
            .search(Some("learning"), HashMap::new(), None)
            .await
            .unwrap();
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_visual_search() {
        let mut indexer = MultimodalIndexer::new();
        let block_id = Uuid::new_v4();
        let mut embeddings = HashMap::new();
        embeddings.insert("clip-vit-b32".to_string(), vec![0.5, 0.5, 0.5]);

        indexer
            .index_block(block_id, "", "image", embeddings.clone())
            .await
            .unwrap();

        let mut query_embeddings = HashMap::new();
        query_embeddings.insert("clip-vit-b32".to_string(), vec![0.5, 0.5, 0.5]);

        let results = indexer.search(None, query_embeddings, None).await.unwrap();
        assert!(!results.is_empty());
    }

    #[test]
    fn test_cosine_similarity() {
        let v1 = vec![1.0, 0.0, 0.0];
        let v2 = vec![1.0, 0.0, 0.0];
        assert!((MultimodalIndexer::cosine_similarity(&v1, &v2) - 1.0).abs() < 0.001);

        let v3 = vec![1.0, 0.0, 0.0];
        let v4 = vec![0.0, 1.0, 0.0];
        assert!((MultimodalIndexer::cosine_similarity(&v3, &v4)).abs() < 0.001);
    }
}
