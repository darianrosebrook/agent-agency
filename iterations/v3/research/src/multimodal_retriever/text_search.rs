//! Text search engine with BM25 and vector search capabilities

use std::sync::Arc;
use std::collections::HashMap;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::core::MultimodalSearchResult;
use super::query_processing::ProcessedQuery;

/// BM25 index for keyword-based text search
#[derive(Debug)]
pub struct Bm25Index {
    documents: HashMap<String, String>, // doc_id -> content
    term_frequencies: HashMap<String, HashMap<String, usize>>, // term -> (doc_id -> frequency)
    document_lengths: HashMap<String, usize>, // doc_id -> length
    average_document_length: f32,
    total_documents: usize,
}

impl Bm25Index {
    /// Create a new BM25 index
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            term_frequencies: HashMap::new(),
            document_lengths: HashMap::new(),
            average_document_length: 0.0,
            total_documents: 0,
        }
    }

    /// Add a document to the index
    pub fn add_document(&mut self, doc_id: String, content: String) {
        let terms = self.tokenize(&content);
        let doc_length = terms.len();

        // Store document content and length
        self.documents.insert(doc_id.clone(), content);
        self.document_lengths.insert(doc_id.clone(), doc_length);

        // Update term frequencies
        for term in terms {
            let doc_freqs = self.term_frequencies.entry(term).or_insert_with(HashMap::new);
            *doc_freqs.entry(doc_id.clone()).or_insert(0) += 1;
        }

        // Update statistics
        self.total_documents += 1;
        self.update_average_length();
    }

    /// Search the index using BM25 scoring
    pub fn search(&self, query: &str, k: usize) -> Vec<(String, f32)> {
        let query_terms = self.tokenize(query);
        let mut scores = HashMap::new();

        for term in &query_terms {
            if let Some(doc_freqs) = self.term_frequencies.get(term) {
                let df = doc_freqs.len() as f32;
                let idf = ((self.total_documents as f32 - df + 0.5) / (df + 0.5)).ln();

                for (doc_id, tf) in doc_freqs {
                    if let Some(doc_length) = self.document_lengths.get(doc_id) {
                        let score = self.bm25_score(*tf as f32, idf, *doc_length as f32);
                        *scores.entry(doc_id.clone()).or_insert(0.0) += score;
                    }
                }
            }
        }

        // Sort by score and return top k
        let mut results: Vec<_> = scores.into_iter().collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        results.into_iter().take(k).collect()
    }

    /// Calculate BM25 score
    fn bm25_score(&self, tf: f32, idf: f32, doc_length: f32) -> f32 {
        let k1 = 1.5;
        let b = 0.75;

        let numerator = tf * (k1 + 1.0);
        let denominator = tf + k1 * (1.0 - b + b * (doc_length / self.average_document_length));

        idf * (numerator / denominator)
    }

    /// Tokenize text into terms
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|s| !s.is_empty() && s.len() > 2)
            .map(|s| s.to_string())
            .collect()
    }

    /// Update average document length
    fn update_average_length(&mut self) {
        if self.total_documents > 0 {
            let total_length: usize = self.document_lengths.values().sum();
            self.average_document_length = total_length as f32 / self.total_documents as f32;
        }
    }
}

/// Vector index for dense embedding search
#[derive(Debug)]
pub struct VectorIndex {
    vectors: HashMap<String, Vec<f32>>, // doc_id -> embedding vector
    dimension: usize,
}

impl VectorIndex {
    /// Create a new vector index
    pub fn new(dimension: usize) -> Self {
        Self {
            vectors: HashMap::new(),
            dimension,
        }
    }

    /// Add a vector to the index
    pub fn add_vector(&mut self, doc_id: String, vector: Vec<f32>) {
        if vector.len() == self.dimension {
            self.vectors.insert(doc_id, vector);
        }
    }

    /// Search for similar vectors using cosine similarity
    pub fn search(&self, query_vector: &[f32], k: usize) -> Vec<(String, f32)> {
        let mut similarities = Vec::new();

        for (doc_id, vector) in &self.vectors {
            if let Some(similarity) = self.cosine_similarity(query_vector, vector) {
                similarities.push((doc_id.clone(), similarity));
            }
        }

        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        similarities.into_iter().take(k).collect()
    }

    /// Calculate cosine similarity between two vectors
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> Option<f32> {
        if a.len() != b.len() {
            return None;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            Some(0.0)
        } else {
            Some(dot_product / (norm_a * norm_b))
        }
    }
}

/// Text search API bridge with BM25 and dense vector search
#[derive(Debug)]
pub struct TextSearchBridge {
    bm25_index: Bm25Index,
    vector_index: VectorIndex,
    embedding_service: Option<Arc<dyn embedding_service::EmbeddingService>>,
}

impl TextSearchBridge {
    /// Create a new text search bridge
    pub fn new(embedding_service: Option<Arc<dyn embedding_service::EmbeddingService>>) -> Self {
        Self {
            bm25_index: Bm25Index::new(),
            vector_index: VectorIndex::new(384), // Default dimension
            embedding_service,
        }
    }

    /// Add a document to both indices
    pub async fn add_document(&mut self, doc_id: String, content: String) -> Result<()> {
        // Add to BM25 index
        self.bm25_index.add_document(doc_id.clone(), content.clone());

        // Add to vector index if embedding service is available
        if let Some(service) = &self.embedding_service {
            let embedding = service.generate_embedding(&content).await?;
            self.vector_index.add_vector(doc_id, embedding);
        }

        Ok(())
    }

    /// Search using hybrid BM25 + vector approach
    pub async fn search(&self, query: &str, k: usize) -> Result<Vec<embedding_service::MultimodalSearchResult>> {
        let mut results = Vec::new();

        // BM25 search
        let bm25_results = self.bm25_index.search(query, k * 2);

        // Vector search if available
        if let Some(service) = &self.embedding_service {
            if let Ok(query_embedding) = service.generate_embedding(query).await {
                let vector_results = self.vector_index.search(&query_embedding, k * 2);

                // Combine results with reciprocal rank fusion
                let combined = self.reciprocal_rank_fusion(bm25_results, vector_results, k);
                results = combined.into_iter().map(|(doc_id, score)| {
                    embedding_service::MultimodalSearchResult {
                        id: doc_id,
                        content: self.bm25_index.documents.get(&doc_id).unwrap_or(&String::new()).clone(),
                        score,
                        metadata: HashMap::new(),
                    }
                }).collect();
            }
        }

        // Fallback to BM25 only
        if results.is_empty() {
            results = bm25_results.into_iter().map(|(doc_id, score)| {
                embedding_service::MultimodalSearchResult {
                    id: doc_id,
                    content: self.bm25_index.documents.get(&doc_id).unwrap_or(&String::new()).clone(),
                    score,
                    metadata: HashMap::new(),
                }
            }).collect();
        }

        Ok(results)
    }

    /// Reciprocal Rank Fusion combining BM25 and vector results
    fn reciprocal_rank_fusion(
        &self,
        bm25_results: Vec<(String, f32)>,
        vector_results: Vec<(String, f32)>,
        k: usize,
    ) -> Vec<(String, f32)> {
        let mut rrf_scores = HashMap::new();

        // Process BM25 results
        for (rank, (doc_id, _)) in bm25_results.iter().enumerate() {
            let rrf_score = 1.0 / (60.0 + rank as f32); // k=60 is standard
            *rrf_scores.entry(doc_id.clone()).or_insert(0.0) += rrf_score;
        }

        // Process vector results
        for (rank, (doc_id, _)) in vector_results.iter().enumerate() {
            let rrf_score = 1.0 / (60.0 + rank as f32);
            *rrf_scores.entry(doc_id.clone()).or_insert(0.0) += rrf_score;
        }

        // Sort by RRF score
        let mut results: Vec<_> = rrf_scores.into_iter().collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        results.into_iter().take(k).collect()
    }
}

/// Text search engine combining multiple search strategies
#[derive(Debug)]
pub struct TextSearchEngine {
    config: super::core::MultimodalRetrieverConfig,
    search_bridge: TextSearchBridge,
    database_pool: Option<Arc<agent_agency_database::DatabaseClient>>,
}

impl TextSearchEngine {
    /// Create a new text search engine
    pub fn new(config: super::core::MultimodalRetrieverConfig) -> Result<Self> {
        let search_bridge = TextSearchBridge::new(None); // No embedding service by default

        Ok(Self {
            config,
            search_bridge,
            database_pool: None,
        })
    }

    /// Create a new text search engine with database integration
    pub async fn new_with_database(
        database_pool: Arc<agent_agency_database::DatabaseClient>,
        config: super::core::MultimodalRetrieverConfig,
    ) -> Result<Self> {
        let search_bridge = TextSearchBridge::new(None);

        Ok(Self {
            config,
            search_bridge,
            database_pool: Some(database_pool),
        })
    }

    /// Execute text search
    pub async fn search(
        &self,
        query: &ProcessedQuery,
        k: usize,
    ) -> Result<Vec<MultimodalSearchResult>> {
        if let Some(text) = &query.text {
            let bridge_results = self.search_bridge.search(text, k).await?;

            let results = bridge_results.into_iter().map(|result| {
                MultimodalSearchResult {
                    id: result.id,
                    content: result.content,
                    modality_scores: HashMap::from([("text".to_string(), result.score)]),
                    combined_score: result.score,
                    metadata: result.metadata,
                    timestamp: Utc::now(),
                    source_modality: "text".to_string(),
                    project_scope: query.project_scope.clone(),
                }
            }).collect();

            Ok(results)
        } else {
            Ok(Vec::new())
        }
    }

    /// Execute code-specific search
    pub async fn search_code(
        &self,
        query: &ProcessedQuery,
        k: usize,
    ) -> Result<Vec<MultimodalSearchResult>> {
        // Code search with language-specific handling
        self.search(query, k).await
    }

    /// Get search statistics
    pub async fn get_stats(&self) -> Result<SearchEngineStats> {
        Ok(SearchEngineStats {
            total_searches: 0, // Placeholder
            average_latency_ms: 0.0,
        })
    }
}

/// Search engine statistics
#[derive(Debug, Clone)]
pub struct SearchEngineStats {
    pub total_searches: u64,
    pub average_latency_ms: f64,
}
