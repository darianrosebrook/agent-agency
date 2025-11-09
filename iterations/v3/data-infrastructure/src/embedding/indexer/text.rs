//! Text indexing and search capabilities
//!
//! BM25 sparse indexing, dense embeddings, and HNSW-based
//! approximate nearest neighbor search for text documents.

use schemars::JsonSchema;
use crate::embedding::embedding_types::*;
use crate::embedding::embedding_service::EmbeddingServiceFactory;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Text document representation for indexing
#[derive(Debug, Clone, JsonSchema)]
pub struct TextDocument {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub content: String,
    pub title: Option<String>,
    pub metadata: HashMap<String, String>,
    pub term_frequencies: HashMap<String, u32>,
}

/// HNSW index metadata for efficient search
#[derive(Debug, Clone, JsonSchema)]
pub struct HnswMetadata {
    pub dimensions: usize,
    pub max_connections: usize,
    pub ef_construction: usize,
    pub ef_search: usize,
    pub index_size: usize,
}

/// Text indexer with BM25 and dense embedding support
pub struct TextIndexer {
    /// BM25 sparse index with term frequencies
    bm25_index: HashMap<String, Vec<TextDocument>>,
    /// Dense embeddings with HNSW indices per model
    dense_embeddings: HashMap<Uuid, EmbeddingVector>,
    /// Per-model HNSW metadata
    hnsw_metadata: HashMap<String, HnswMetadata>,
    /// Shared embedding service (lazy-initialized)
    embedding_service: Arc<tokio::sync::OnceCell<Box<dyn crate::embedding::EmbeddingService>>>,
}

impl std::fmt::Debug for TextIndexer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextIndexer")
            .field("bm25_index", &self.bm25_index)
            .field("dense_embeddings", &self.dense_embeddings)
            .field("hnsw_metadata", &self.hnsw_metadata)
            .field("embedding_service", &"<lazy-initialized>")
            .finish()
    }
}

impl TextIndexer {
    /// Create a new text indexer
    pub fn new() -> Self {
        Self {
            bm25_index: HashMap::new(),
            dense_embeddings: HashMap::new(),
            hnsw_metadata: HashMap::new(),
            embedding_service: Arc::new(tokio::sync::OnceCell::new()),
        }
    }

    /// Initialize embedding service (lazy initialization)
    async fn get_embedding_service(&self) -> Result<&dyn crate::embedding::EmbeddingService> {
        self.embedding_service.get_or_try_init(|| async {
            let config = crate::embedding::EmbeddingConfig {
                model_name: "embeddinggemma".to_string(),
                dimension: 768,
                batch_size: 32,
                cache_size: 1000,
                timeout_ms: 30000,
            };
            
            let service = EmbeddingServiceFactory::create_with_auto_detect(config, Some("embeddinggemma".to_string())).await;
            // EmbeddingService trait already has Send + Sync bounds
            Ok(service)
        })
        .await
        .map(|s| s.as_ref())
    }

    /// Index a text document
    pub fn index_document(&mut self, doc: TextDocument) -> Result<()> {
        // Build BM25 index
        self.build_bm25_index(&doc);

        // Generate dense embeddings (placeholder)
        let embedding = self.generate_embedding(&doc.content)?;
        self.dense_embeddings.insert(doc.id, embedding);

        // Initialize HNSW metadata for new models
        self.initialize_hnsw_metadata("default_model");

        Ok(())
    }

    /// Search documents using BM25
    pub fn bm25_search(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        let query_terms: Vec<&str> = query.split_whitespace().collect();
        let mut results = Vec::new();

        for term in &query_terms {
            if let Some(docs) = self.bm25_index.get(*term) {
                for doc in docs {
                    let score = self.calculate_bm25_score(doc, &query_terms);
                    results.push(SearchResult {
                        document_id: doc.id,
                        score,
                        content_preview: self.extract_preview(&doc.content, query),
                        metadata: doc.metadata.clone(),
                    });
                }
            }
        }

        // Sort by score and limit results
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(limit);
        results
    }

    /// Search using dense embeddings and HNSW
    pub fn semantic_search(&self, query_embedding: &EmbeddingVector, limit: usize) -> Vec<SearchResult> {
        let mut results = Vec::new();

        for (doc_id, embedding) in &self.dense_embeddings {
            let score = self.cosine_similarity(query_embedding, embedding);
            results.push(SearchResult {
                document_id: *doc_id,
                score,
                content_preview: "Semantic search result".to_string(),
                metadata: HashMap::new(),
            });
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(limit);
        results
    }

    /// Hybrid search combining BM25 and semantic search
    pub fn hybrid_search(&self, query: &str, query_embedding: &EmbeddingVector, limit: usize) -> Vec<SearchResult> {
        let bm25_results = self.bm25_search(query, limit * 2);
        let semantic_results = self.semantic_search(query_embedding, limit * 2);

        // Combine and rerank results
        self.combine_search_results(bm25_results, semantic_results, limit)
    }

    /// Get index statistics
    pub fn get_statistics(&self) -> IndexStatistics {
        IndexStatistics {
            total_documents: self.dense_embeddings.len(),
            total_terms: self.bm25_index.len(),
            models_indexed: self.hnsw_metadata.len(),
            index_sizes: self.hnsw_metadata.values().map(|m| m.index_size).sum(),
        }
    }

    // Private helper methods

    fn build_bm25_index(&mut self, doc: &TextDocument) {
        let terms: Vec<String> = doc.content
            .split_whitespace()
            .map(|s| s.to_lowercase())
            .collect();

        for term in terms {
            self.bm25_index
                .entry(term)
                .or_insert_with(Vec::new)
                .push(doc.clone());
        }
    }

    fn calculate_bm25_score(&self, doc: &TextDocument, query_terms: &[&str]) -> f64 {
        // TODO: Implement proper BM25 scoring algorithm
        //       Currently uses basic calculation; should implement full BM25 algorithm with proper term frequency and inverse document frequency calculations.
        let mut score = 0.0;
        let doc_length = doc.content.split_whitespace().count() as f64;
        let avg_doc_length = 1000.0; // Placeholder

        for &term in query_terms {
            if let Some(term_docs) = self.bm25_index.get(term) {
                let tf = doc.term_frequencies.get(term).copied().unwrap_or(0) as f64;
                let df = term_docs.len() as f64;
                let total_docs = self.dense_embeddings.len() as f64;

                // BM25 formula components
                let k1 = 1.5;
                let b = 0.75;
                let idf = (total_docs - df + 0.5) / (df + 0.5).ln();

                let numerator = tf * (k1 + 1.0);
                let denominator = tf + k1 * (1.0 - b + b * doc_length / avg_doc_length);

                score += idf * numerator / denominator;
            }
        }

        score
    }

    /// Generate dense embedding using CoreML (fallback to DummyEmbeddingProvider)
    async fn generate_embedding_async(&self, content: &str) -> Result<EmbeddingVector> {
        use tracing::{info, debug};
        
        info!("Generating dense embedding for content (length: {})", content.len());
        
        // Get embedding service (lazy initialization)
        let service = self.get_embedding_service().await?;
        
        // Generate embedding
        let stored_embedding = service.generate_embedding(content, crate::embedding::ContentType::Text, "text_indexer").await?;
        
        debug!("Generated embedding with {} dimensions", stored_embedding.vector.values.len());
        
        // StoredEmbedding already contains EmbeddingVector, so we can use it directly
        Ok(stored_embedding.vector)
    }

    /// Generate dense embedding using CoreML (synchronous wrapper)
    fn generate_embedding(&self, content: &str) -> Result<EmbeddingVector> {
        tokio::runtime::Handle::current().block_on(async {
            self.generate_embedding_async(content).await
        })
    }

    fn cosine_similarity(&self, a: &EmbeddingVector, b: &EmbeddingVector) -> f64 {
        let dot_product: f64 = a.values.iter().zip(&b.values).map(|(x, y)| (*x as f64) * (*y as f64)).sum();
        let norm_a: f64 = a.values.iter().map(|x| (*x as f64) * (*x as f64)).sum();
        let norm_b: f64 = b.values.iter().map(|x| (*x as f64) * (*x as f64)).sum();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }

    fn extract_preview(&self, content: &str, query: &str) -> String {
        // Simple preview extraction around query terms
        if let Some(pos) = content.to_lowercase().find(&query.to_lowercase()) {
            let start = pos.saturating_sub(50);
            let end = (pos + query.len() + 50).min(content.len());
            format!("...{}...", &content[start..end])
        } else {
            content.chars().take(100).collect::<String>() + "..."
        }
    }

    fn combine_search_results(&self, bm25: Vec<SearchResult>, semantic: Vec<SearchResult>, limit: usize) -> Vec<SearchResult> {
        let mut combined = HashMap::new();

        // Add BM25 results with weight
        for result in bm25 {
            combined.insert(result.document_id, (result.score * 0.6, result));
        }

        // Add semantic results with weight
        for result in semantic {
            let entry = combined.entry(result.document_id).or_insert((0.0, result.clone()));
            entry.0 += result.score * 0.4;
            // Keep the result with higher individual score
            if result.score > entry.1.score {
                entry.1 = result;
            }
        }

        // Sort by combined score and limit
        let mut results: Vec<_> = combined.into_values().map(|(_, result)| result).collect();
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(limit);
        results
    }

    fn initialize_hnsw_metadata(&mut self, model: &str) {
        if !self.hnsw_metadata.contains_key(model) {
            self.hnsw_metadata.insert(model.to_string(), HnswMetadata {
                dimensions: 768, // Default embedding dimension
                max_connections: 32,
                ef_construction: 200,
                ef_search: 64,
                index_size: 0,
            });
        }
    }
}

/// Search result structure
#[derive(Debug, Clone, JsonSchema)]
pub struct SearchResult {
    #[schemars(with = "String")]
    pub document_id: Uuid,
    pub score: f64,
    pub content_preview: String,
    pub metadata: HashMap<String, String>,
}

/// Index statistics
#[derive(Debug, Clone, JsonSchema)]
pub struct IndexStatistics {
    pub total_documents: usize,
    pub total_terms: usize,
    pub models_indexed: usize,
    pub index_sizes: usize,
}


