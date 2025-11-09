//! Data indexing stage - creates searchable indexes for processed data
//!
//! Consolidates functionality from the original indexers crate:
//! - BM25 full-text search indexing
//! - HNSW approximate nearest neighbor search for embeddings
//! - Database persistence with connection pooling
//! - Job scheduler with concurrency governance

use schemars::JsonSchema;
use crate::data_processing_types::*;
use crate::{DataProcessingResult, DataProcessingError};
use async_trait::async_trait;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::debug;
#[cfg(feature = "memory-integration")]
use agent_memory::graph_engine::Relationship;
use std::sync::Arc;
use parking_lot::{Mutex, RwLock};

/// Result from indexing operations
pub type IndexingResult = DataProcessingResult<ProcessingOutput>;

/// Stage for data indexing operations
#[async_trait]
pub trait IndexingStage: Send + Sync {
    /// Get the name of this indexing stage
    fn name(&self) -> &'static str;

    /// Index the given processed content
    async fn index(&self, input: DataInput, content: ProcessedContent) -> IndexingResult;

    /// Search the index
    async fn search(&self, query: &IndexQuery) -> DataProcessingResult<IndexResult>;

    /// Get supported index types
    fn supported_indexes(&self) -> &[IndexType];
}

/// Types of indexes supported
#[derive(Debug, Clone, PartialEq, Eq, Hash, JsonSchema)]
pub enum IndexType {
    FullText,
    Vector,
    Entity,
    Relationship,
}

/// Query for searching indexes
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct IndexQuery {
    pub query_type: IndexQueryType,
    pub text_query: Option<String>,
    pub vector_query: Option<Vec<f32>>,
    pub entity_filters: Vec<EntityFilter>,
    pub limit: usize,
    pub include_metadata: bool,
}

/// Types of index queries
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, JsonSchema)]
pub enum IndexQueryType {
    Text,
    Semantic,
    Hybrid,
    Entity,
}

/// Result from index search
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct IndexResult {
    pub total_matches: usize,
    pub results: Vec<IndexMatch>,
    pub search_time_ms: u64,
    pub facets: HashMap<String, HashMap<String, usize>>,
}

/// Individual search result match
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct IndexMatch {
    pub id: ProcessingId,
    pub score: f64,
    pub content_snippet: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub matched_entities: Vec<Entity>,
}

/// Default implementation combining all indexing capabilities
pub struct DefaultIndexingStage {
    fulltext_indexer: FullTextIndexer,
    vector_indexer: VectorIndexer,
    entity_indexer: EntityIndexer,
    _job_scheduler: JobScheduler,
}

impl DefaultIndexingStage {
    /// Create a new default indexing stage
    pub async fn new() -> DataProcessingResult<Self> {
        Ok(Self {
            fulltext_indexer: FullTextIndexer::new().await?,
            vector_indexer: VectorIndexer::new().await?,
            entity_indexer: EntityIndexer::new().await?,
            _job_scheduler: JobScheduler::new(),
        })
    }
}

#[async_trait]
impl IndexingStage for DefaultIndexingStage {
    fn name(&self) -> &'static str {
        "default_indexing"
    }

    async fn index(&self, input: DataInput, content: ProcessedContent) -> IndexingResult {
        let input_id = input.id.clone(); // Clone once to avoid multiple moves
        let start_time = std::time::Instant::now();
        let mut errors = Vec::new();

        // Index full-text content
        if let Some(text) = &content.text_content {
            match self.fulltext_indexer.index_text(input_id.clone(), text).await {
                Ok(_) => {}
                Err(e) => errors.push(format!("Full-text indexing failed: {}", e)),
            }
        }

        // Index embeddings if available
        if let Some(embedding) = &content.embeddings {
            match self.vector_indexer.index_vector(input_id.clone(), embedding.clone()).await {
                Ok(_) => {}
                Err(e) => errors.push(format!("Vector indexing failed: {}", e)),
            }
        }

        // Index entities
        if !content.entities.is_empty() {
            match self.entity_indexer.index_entities(input_id.clone(), &content.entities).await {
                Ok(_) => {}
                Err(e) => errors.push(format!("Entity indexing failed: {}", e)),
            }
        }

        // Index relationships
        if !content.relationships.is_empty() {
            match self.entity_indexer.index_relationships(input_id.clone(), &content.relationships).await {
                Ok(_) => {}
                Err(e) => errors.push(format!("Relationship indexing failed: {}", e)),
            }
        }

        // Create metadata about indexing
        let mut metadata = input.metadata.clone();
        metadata.insert("indexing_completed".to_string(), (errors.is_empty()).into());
        metadata.insert("index_types_created".to_string(),
            serde_json::to_value(vec!["fulltext", "vector", "entity"]).unwrap_or(serde_json::Value::Null));

        let stats = ProcessingStats {
            processing_time_ms: start_time.elapsed().as_millis() as u64,
            bytes_processed: 0, // Would track indexed data size
            entities_extracted: content.entities.len(),
            relationships_found: content.relationships.len(),
            embeddings_generated: content.embeddings.as_ref().map(|e| e.len() / 384).unwrap_or(0), // Assuming 384-dim embeddings
            errors_encountered: errors,
        };

        Ok(ProcessingOutput {
            id: input_id,
            original_input: input,
            processed_content: content,
            extracted_metadata: metadata,
            processing_stats: stats,
            created_at: chrono::Utc::now(),
        })
    }

    async fn search(&self, query: &IndexQuery) -> DataProcessingResult<IndexResult> {
        let start_time = std::time::Instant::now();

        let mut all_results = Vec::new();
        let facets = HashMap::new();

        // Perform search based on query type
        match query.query_type {
            IndexQueryType::Text => {
                if let Some(text) = &query.text_query {
                    let results = self.fulltext_indexer.search_text(text, query.limit).await?;
                    all_results.extend(results.into_iter().map(|(id, score, snippet)| IndexMatch {
                        id,
                        score,
                        content_snippet: snippet,
                        metadata: HashMap::new(),
                        matched_entities: vec![],
                    }));
                }
            }

            IndexQueryType::Semantic => {
                if let Some(vector) = &query.vector_query {
                    let results = self.vector_indexer.search_similar(vector, query.limit).await?;
                    all_results.extend(results.into_iter().map(|(id, score)| IndexMatch {
                        id,
                        score,
                        content_snippet: None,
                        metadata: HashMap::new(),
                        matched_entities: vec![],
                    }));
                }
            }

            IndexQueryType::Hybrid => {
                // Combine text and semantic search
                let mut hybrid_results = Vec::new();

                if let Some(text) = &query.text_query {
                    if let Ok(text_results) = self.fulltext_indexer.search_text(text, query.limit * 2).await {
                        hybrid_results.extend(text_results);
                    }
                }

                if let Some(vector) = &query.vector_query {
                    if let Ok(vector_results) = self.vector_indexer.search_similar(vector, query.limit * 2).await {
                        // Normalize and combine scores
                        for (id, vector_score) in vector_results {
                            if let Some((_, text_score, _)) = hybrid_results.iter_mut().find(|(existing_id, _, _)| *existing_id == id) {
                                *text_score = (*text_score + vector_score) / 2.0; // Simple average
                            } else {
                                hybrid_results.push((id, vector_score, None));
                            }
                        }
                    }
                }

                // Sort by combined score and take top results
                hybrid_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                hybrid_results.truncate(query.limit);

                all_results.extend(hybrid_results.into_iter().map(|(id, score, snippet)| IndexMatch {
                    id,
                    score,
                    content_snippet: snippet,
                    metadata: HashMap::new(),
                    matched_entities: vec![],
                }));
            }

            IndexQueryType::Entity => {
                // Search for entities
                for filter in &query.entity_filters {
                    let results = self.entity_indexer.search_entities(filter, query.limit).await?;
                    all_results.extend(results.into_iter().map(|(id, entity, score)| IndexMatch {
                        id,
                        score,
                        content_snippet: None,
                        metadata: HashMap::from([
                            ("entity_name".to_string(), entity.name.clone().into()),
                            ("entity_type".to_string(), format!("{:?}", entity.entity_type).into()),
                        ]),
                        matched_entities: vec![entity],
                    }));
                }
            }
        }

        // Sort results by score
        all_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Apply limit
        let total_matches = all_results.len();
        all_results.truncate(query.limit);

        // Add metadata if requested
        if query.include_metadata {
            for result in &mut all_results {
                if let Ok(metadata) = self.fulltext_indexer.get_metadata(&result.id).await {
                    result.metadata = metadata;
                }
            }
        }

        Ok(IndexResult {
            total_matches,
            results: all_results,
            search_time_ms: start_time.elapsed().as_millis() as u64,
            facets,
        })
    }

    fn supported_indexes(&self) -> &[IndexType] {
        &[
            IndexType::FullText,
            IndexType::Vector,
            IndexType::Entity,
            IndexType::Relationship,
        ]
    }
}

#[async_trait]
impl crate::pipeline::PipelineStage for DefaultIndexingStage {
    fn name(&self) -> &'static str {
        "indexing"
    }

    async fn process(&self, input: DataInput) -> DataProcessingResult<ProcessingOutput> {
        let _input_id = input.id.clone(); // Clone once to avoid multiple moves

        // For indexing, we expect the input to contain enriched content
        let processed_content = match &input.content {
            DataContent::Structured(data) => {
                // Try to deserialize as ProcessedContent
                match serde_json::from_value(data.clone()) {
                    Ok(content) => content,
                    Err(_) => return Err(DataProcessingError::Validation(
                        "Expected ProcessedContent in structured data".to_string()
                    )),
                }
            }
            _ => return Err(DataProcessingError::Validation(
                "Indexing stage expects structured content".to_string()
            )),
        };

        self.index(input, processed_content).await
    }

    async fn query(&self, query: &crate::DataQuery) -> DataProcessingResult<Vec<RetrievedData>> {
        // Convert pipeline query to index query
        let index_query = IndexQuery {
            query_type: match query.query_type {
                crate::QueryType::TextSearch => IndexQueryType::Text,
                crate::QueryType::SemanticSearch => IndexQueryType::Semantic,
                crate::QueryType::HybridSearch => IndexQueryType::Hybrid,
                crate::QueryType::EntitySearch => IndexQueryType::Entity,
            },
            text_query: query.text_query.clone(),
            vector_query: query.semantic_vector.clone(),
            entity_filters: query.entity_filters.clone(),
            limit: query.limit,
            include_metadata: true,
        };

        let index_result = self.search(&index_query).await?;

        // Convert index results to retrieved data
        let retrieved_data = index_result.results.into_iter().map(|match_| {
            RetrievedData {
                id: match_.id,
                content: ProcessedContent {
                    text_content: match_.content_snippet.clone(),
                    structured_data: None,
                    embeddings: None,
                    entities: match_.matched_entities.clone(),
                    relationships: vec![],
                    visual_elements: vec![],
                    audio_transcript: None,
                    content_type: ContentType::Text,
                    data: ProcessedContentData::Text(match_.content_snippet.clone().unwrap_or_default()),
                },
                relevance_score: match_.score,
                matched_entities: match_.matched_entities,
                source_metadata: match_.metadata,
            }
        }).collect();

        Ok(retrieved_data)
    }
}

/// Full-text search indexer using BM25
pub struct FullTextIndexer {
    // BM25 index implementation
    documents: std::sync::Mutex<HashMap<ProcessingId, DocumentRecord>>,
    vocabulary: std::sync::Mutex<HashMap<String, TermStats>>,
    total_documents: std::sync::Mutex<usize>,
}

/// Document record for BM25 indexing
#[derive(Debug, Clone, JsonSchema)]
struct DocumentRecord {
    text: String,
    term_freqs: HashMap<String, u32>,
    length: usize,
    modality: String,
}

/// Term statistics for BM25
#[derive(Debug, Clone, JsonSchema)]
struct TermStats {
    document_frequency: u32,
    total_frequency: u32,
}

impl FullTextIndexer {
    pub async fn new() -> DataProcessingResult<Self> {
        Ok(Self {
            documents: std::sync::Mutex::new(HashMap::new()),
            vocabulary: std::sync::Mutex::new(HashMap::new()),
            total_documents: std::sync::Mutex::new(0),
        })
    }

    pub async fn index_text(&self, id: ProcessingId, text: &str) -> DataProcessingResult<()> {
        let tokens = self.tokenize(text);
        let term_freqs = self.calculate_term_frequencies(&tokens);
        let doc_length = tokens.len();
        
        // Update document record
        {
            let mut documents = self.documents.lock().unwrap();
            documents.insert(id.clone(), DocumentRecord {
                text: text.to_string(),
                term_freqs: term_freqs.clone(),
                length: doc_length,
                modality: "text".to_string(),
            });
        }
        
        // Update vocabulary statistics
        {
            let mut vocabulary = self.vocabulary.lock().unwrap();
            let mut total_docs = self.total_documents.lock().unwrap();
            
            for (term, freq) in &term_freqs {
                let stats = vocabulary.entry(term.clone()).or_insert(TermStats {
                    document_frequency: 0,
                    total_frequency: 0,
                });
                stats.document_frequency += 1;
                stats.total_frequency += freq;
            }
            
            *total_docs += 1;
        }
        
        Ok(())
    }

    pub async fn search_text(&self, query: &str, limit: usize) -> DataProcessingResult<Vec<(ProcessingId, f64, Option<String>)>> {
        let query_terms = self.tokenize(query);
        let mut results = Vec::new();
        
        let documents = self.documents.lock().unwrap();
        let vocabulary = self.vocabulary.lock().unwrap();
        let total_docs = *self.total_documents.lock().unwrap();
        
        if total_docs == 0 {
            return Ok(results);
        }
        
        // Calculate BM25 scores for each document
        for (id, doc) in documents.iter() {
            let score = self.calculate_bm25_score(&query_terms, doc, &vocabulary, total_docs);
            if score > 0.0 {
                let snippet = self.extract_snippet(&doc.text, &query_terms);
                results.push((id.clone(), score, snippet));
            }
        }
        
        // Sort by score descending
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        
        Ok(results)
    }

    /// Tokenize text into terms
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.split_whitespace()
            .map(|word| word.to_lowercase())
            .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|word| !word.is_empty())
            .collect()
    }

    /// Calculate term frequencies for a document
    fn calculate_term_frequencies(&self, tokens: &[String]) -> HashMap<String, u32> {
        let mut freqs = HashMap::new();
        for token in tokens {
            *freqs.entry(token.clone()).or_insert(0) += 1;
        }
        freqs
    }

    /// Calculate BM25 score for a document
    fn calculate_bm25_score(
        &self,
        query_terms: &[String],
        doc: &DocumentRecord,
        vocabulary: &HashMap<String, TermStats>,
        total_docs: usize,
    ) -> f64 {
        const K1: f64 = 1.2;
        const B: f64 = 0.75;
        
        let mut score = 0.0;
        
        for term in query_terms {
            if let Some(term_stats) = vocabulary.get(term) {
                let tf = *doc.term_freqs.get(term).unwrap_or(&0) as f64;
                let df = term_stats.document_frequency as f64;
                let idf = ((total_docs as f64 - df + 0.5) / (df + 0.5)).ln();
                
                let numerator = tf * (K1 + 1.0);
                let denominator = tf + K1 * (1.0 - B + B * (doc.length as f64 / self.get_average_document_length()));
                
                score += idf * (numerator / denominator);
            }
        }
        
        score
    }

    /// Get average document length
    // TODO: Implement comprehensive document length calculation with statistical analysis
    //       Currently uses simple arithmetic mean; should include statistical measures for better BM25 scoring.
    //
    // COMPLETION CHECKLIST:
    // [ ] Calculate mean, median, and standard deviation of document lengths
    // [ ] Implement length normalization for BM25 scoring
    // [ ] Add caching for document length statistics
    // [ ] Handle edge cases (empty documents, very long documents)
    // [ ] Add unit tests for statistical calculations
    // [ ] Add integration tests with real document collections
    // [ ] Verify improved BM25 scoring accuracy
    //
    // ACCEPTANCE CRITERIA:
    // - Document length statistics include mean, median, and standard deviation
    // - Length normalization improves BM25 scoring accuracy
    // - Statistics are cached and updated efficiently
    // - Edge cases are handled gracefully
    //
    // DEPENDENCIES:
    // - Document collection data structure (Required)
    // - BM25 scoring algorithm (Required)
    // - Statistical calculation utilities (Optional)
    //
    // ESTIMATED EFFORT: 3-5 hours (medium confidence)
    // PRIORITY: Low
    // BLOCKING: No
    //
    // GOVERNANCE:
    // - CAWS Tier: 3 (low risk optimization)
    // - Change Budget: ~80 LOC
    // - Reviewer Requirements: Search algorithm expertise
    fn get_average_document_length(&self) -> f64 {
        let documents = self.documents.lock().unwrap();
        if documents.is_empty() {
            return 100.0; // Default average
        }
        
        let total_length: usize = documents.values().map(|doc| doc.length).sum();
        total_length as f64 / documents.len() as f64
    }

    /// Extract relevant snippet from document
    fn extract_snippet(&self, text: &str, query_terms: &[String]) -> Option<String> {
        let lines: Vec<&str> = text.lines().collect();
        
        // Find the line with the most query term matches
        let mut best_line = None;
        let mut max_matches = 0;
        
        for line in &lines {
            let line_lower = line.to_lowercase();
            let matches = query_terms.iter()
                .filter(|term| line_lower.contains(term.as_str()))
                .count();
            
            if matches > max_matches {
                max_matches = matches;
                best_line = Some(line);
            }
        }
        
        best_line.map(|line| {
            if line.len() > 200 {
                format!("{}...", &line[..200])
            } else {
                line.to_string()
            }
        })
    }

    pub async fn get_metadata(&self, id: &ProcessingId) -> DataProcessingResult<HashMap<String, serde_json::Value>> {
        let documents = self.documents.lock().unwrap();
        
        if let Some(doc) = documents.get(id) {
            Ok(HashMap::from([
                ("indexed_at".to_string(), chrono::Utc::now().to_rfc3339().into()),
                ("content_length".to_string(), doc.length.into()),
                ("unique_terms".to_string(), doc.term_freqs.len().into()),
            ]))
        } else {
            Err(DataProcessingError::NotFound(format!("Document {} not found in index", id)))
        }
    }
}

/// Vector similarity search indexer using HNSW-like structure
pub struct VectorIndexer {
    // HNSW-like index implementation
    vectors: std::sync::Mutex<HashMap<ProcessingId, VectorRecord>>,
    graph: std::sync::Mutex<HashMap<ProcessingId, Vec<ProcessingId>>>, // Simple graph structure
    dimension: usize,
}

/// Vector record for HNSW indexing
#[derive(Debug, Clone, JsonSchema)]
struct VectorRecord {
    vector: Vec<f32>,
    norm: f32,
    #[schemars(with = "String")]
    indexed_at: chrono::DateTime<chrono::Utc>,
}

impl VectorIndexer {
    pub async fn new() -> DataProcessingResult<Self> {
        Ok(Self {
            vectors: std::sync::Mutex::new(HashMap::new()),
            graph: std::sync::Mutex::new(HashMap::new()),
            dimension: 384, // Default dimension
        })
    }

    pub async fn index_vector(&self, id: ProcessingId, vector: Vec<f32>) -> DataProcessingResult<()> {
        if vector.len() != self.dimension {
            return Err(DataProcessingError::Validation(format!(
                "Vector dimension {} does not match expected dimension {}",
                vector.len(),
                self.dimension
            )));
        }

        let norm = self.calculate_norm(&vector);
        let record = VectorRecord {
            vector: vector.clone(),
            norm,
            indexed_at: chrono::Utc::now(),
        };

        // Store vector record
        {
            let mut vectors = self.vectors.lock().unwrap();
            vectors.insert(id.clone(), record);
        }

        // TODO: Implement full HNSW (Hierarchical Navigable Small World) graph structure
        //       Currently uses basic single-layer graph with fixed top-5 connections.
        //
        // COMPLETION CHECKLIST:
        // [ ] Implement multi-layer HNSW graph structure (entry layer + multiple levels)
        // [ ] Implement dynamic connection count based on layer and vector density
        // [ ] Add graph pruning and optimization algorithms
        // [ ] Implement efficient search algorithm using graph structure
        // [ ] Add graph maintenance and rebalancing logic
        // [ ] Add unit tests for HNSW graph operations
        // [ ] Add integration tests with large vector collections
        // [ ] Benchmark search performance vs basic implementation
        //
        // ACCEPTANCE CRITERIA:
        // - Multi-layer HNSW graph structure is implemented
        // - Search performance improves significantly over basic version
        // - Graph maintains good connectivity and search quality
        // - Memory usage is reasonable for large vector collections
        //
        // DEPENDENCIES:
        // - Vector similarity calculation (Required)
        // - Graph data structures (Required)
        // - HNSW algorithm specification (Required)
        //
        // ESTIMATED EFFORT: 8-12 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (standard feature)
        // - Change Budget: ~300 LOC
        // - Reviewer Requirements: Vector search algorithm expertise
        self.build_graph_connections(&id, &vector).await?;

        Ok(())
    }

    pub async fn search_similar(&self, query_vector: &[f32], limit: usize) -> DataProcessingResult<Vec<(ProcessingId, f64)>> {
        if query_vector.len() != self.dimension {
            return Err(DataProcessingError::Validation(format!(
                "Query vector dimension {} does not match expected dimension {}",
                query_vector.len(),
                self.dimension
            )));
        }

        let query_norm = self.calculate_norm(query_vector);
        let mut results = Vec::new();

        let vectors = self.vectors.lock().unwrap();
        let graph = self.graph.lock().unwrap();

        // TODO: Implement proper graph-based search algorithm
        //       Currently uses basic graph search; should implement efficient graph traversal for vector similarity search.
        let mut visited = std::collections::HashSet::new();
        let mut candidates = Vec::new();

        // TODO: Implement proper entry point selection
        //       Currently uses random entry point; should select optimal entry point based on graph structure and query characteristics.
        if let Some(entry_point) = vectors.keys().next() {
            let score = self.cosine_similarity(query_vector, query_norm, &vectors[entry_point]);
            candidates.push((score, entry_point.clone()));
        }

        while let Some((score, id)) = candidates.pop() {
            if visited.contains(&id) {
                continue;
            }
            visited.insert(id.clone());

            results.push((id.clone(), score));

            // Add neighbors to candidates
            if let Some(neighbors) = graph.get(&id) {
                for neighbor_id in neighbors {
                    if !visited.contains(neighbor_id) {
                        if let Some(neighbor_record) = vectors.get(neighbor_id) {
                            let neighbor_score = self.cosine_similarity(query_vector, query_norm, neighbor_record);
                            candidates.push((neighbor_score, neighbor_id.clone()));
                        }
                    }
                }
            }
            
            // Sort candidates by score descending
            candidates.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        }

        // Sort by similarity descending
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);

        Ok(results)
    }

    /// Calculate vector norm
    fn calculate_norm(&self, vector: &[f32]) -> f32 {
        vector.iter().map(|x| x * x).sum::<f32>().sqrt()
    }

    /// Calculate cosine similarity between query and stored vector
    fn cosine_similarity(&self, query_vector: &[f32], query_norm: f32, record: &VectorRecord) -> f64 {
        if query_norm == 0.0 || record.norm == 0.0 {
            return 0.0;
        }

        let dot_product: f32 = query_vector.iter()
            .zip(record.vector.iter())
            .map(|(x, y)| x * y)
            .sum();

        (dot_product / (query_norm * record.norm)) as f64
    }

    /// Build graph connections for HNSW-like structure
    async fn build_graph_connections(&self, id: &ProcessingId, vector: &[f32]) -> DataProcessingResult<()> {
        let mut graph = self.graph.lock().unwrap();
        let vectors = self.vectors.lock().unwrap();

        let mut connections = Vec::new();
        let mut similarities = Vec::new();

        // Find most similar vectors
        for (other_id, other_record) in vectors.iter() {
            if other_id != id {
                let similarity = self.cosine_similarity(vector, self.calculate_norm(vector), other_record);
                similarities.push((similarity, other_id.clone()));
            }
        }

        // Sort by similarity and take top connections
        similarities.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        
        // TODO: Implement full HNSW graph structure (see TODO above)
        //       Currently uses temporary top-5 connections; should implement full HNSW graph structure for efficient similarity search.
        for (_, other_id) in similarities.iter().take(5) {
            connections.push(other_id.clone());
        }

        graph.insert(id.clone(), connections);
        Ok(())
    }

    pub async fn get_metadata(&self, id: &ProcessingId) -> DataProcessingResult<HashMap<String, serde_json::Value>> {
        let vectors = self.vectors.lock().unwrap();
        
        if let Some(record) = vectors.get(id) {
            Ok(HashMap::from([
                ("indexed_at".to_string(), record.indexed_at.to_rfc3339().into()),
                ("vector_dimension".to_string(), record.vector.len().into()),
                ("vector_norm".to_string(), record.norm.into()),
            ]))
        } else {
            Err(DataProcessingError::NotFound(format!("Vector {} not found in index", id)))
        }
    }
}

/// Relationship record for indexing
#[derive(Debug, Clone, JsonSchema)]
pub struct RelationshipRecord {
    _source_entity: String,
    _target_entity: String,
    relationship_type: String,
    confidence: f64,
    _context: Option<String>,
    processing_id: ProcessingId,
}

/// Entity and relationship indexer with advanced search capabilities
pub struct EntityIndexer {
    // Entity index with multiple search strategies
    entities: std::sync::Mutex<HashMap<String, Vec<(ProcessingId, Entity, f64)>>>,
    entity_types: std::sync::Mutex<HashMap<String, Vec<(ProcessingId, Entity, f64)>>>,
    entity_text_index: std::sync::Mutex<HashMap<String, Vec<(ProcessingId, Entity, f64)>>>,
    relationships: std::sync::Mutex<HashMap<String, Vec<RelationshipRecord>>>,
}

impl EntityIndexer {
    pub async fn new() -> DataProcessingResult<Self> {
        Ok(Self {
            entities: std::sync::Mutex::new(HashMap::new()),
            entity_types: std::sync::Mutex::new(HashMap::new()),
            entity_text_index: std::sync::Mutex::new(HashMap::new()),
            relationships: std::sync::Mutex::new(HashMap::new()),
        })
    }

    pub async fn index_entities(&self, id: ProcessingId, entities: &[Entity]) -> DataProcessingResult<()> {
        let mut entity_index = self.entities.lock().unwrap();
        let mut type_index = self.entity_types.lock().unwrap();
        let mut text_index = self.entity_text_index.lock().unwrap();
        
        for entity in entities {
            let confidence = self.calculate_entity_confidence(entity);
            
            // Index by full entity name
            entity_index.entry(entity.name.clone()).or_insert_with(Vec::new).push((
                id.clone(),
                entity.clone(),
                confidence,
            ));
            
            // Index by entity type
            let type_key = format!("{:?}", entity.entity_type);
            type_index.entry(type_key).or_insert_with(Vec::new).push((
                id.clone(),
                entity.clone(),
                confidence,
            ));
            
            // Index by text content for fuzzy search
            let text_key = entity.name.to_lowercase();
            text_index.entry(text_key).or_insert_with(Vec::new).push((
                id.clone(),
                entity.clone(),
                confidence,
            ));
        }
        
        Ok(())
    }

    pub async fn index_relationships(&self, id: ProcessingId, relationships: &[Relationship]) -> DataProcessingResult<()> {
        let mut relationship_index = self.relationships.lock().unwrap();
        
        for relationship in relationships {
            let record = RelationshipRecord {
                _source_entity: relationship.source_entity.clone(),
                _target_entity: relationship.target_entity.clone(),
                relationship_type: format!("{:?}", relationship.relationship_type),
                confidence: relationship.confidence,
                _context: relationship.evidence.first().cloned(),
                processing_id: id.clone(),
            };
            
            // Index by relationship type
            let type_key = format!("{:?}", relationship.relationship_type);
            relationship_index.entry(type_key)
                .or_insert_with(Vec::new)
                .push(record.clone());
            
            // Index by source entity
            relationship_index.entry(format!("source:{}", relationship.source_entity))
                .or_insert_with(Vec::new)
                .push(record.clone());
            
            // Index by target entity
            relationship_index.entry(format!("target:{}", relationship.target_entity))
                .or_insert_with(Vec::new)
                .push(record);
        }
        
        Ok(())
    }

    pub async fn search_entities(&self, filter: &EntityFilter, limit: usize) -> DataProcessingResult<Vec<(ProcessingId, Entity, f64)>> {
        let mut results = Vec::new();
        
        // Search by entity name
        if let Some(entity_name) = filter.entity_names.first() {
            let entity_index = self.entities.lock().unwrap();
            if let Some(entities) = entity_index.get(entity_name) {
                for (id, entity, confidence) in entities {
                    if *confidence >= filter.min_confidence {
                        results.push((id.clone(), entity.clone(), *confidence));
                    }
                }
            }
            
            // Also search text index for fuzzy matching
            let text_index = self.entity_text_index.lock().unwrap();
            let query_lower = entity_name.to_lowercase();
            for (text_key, entries) in text_index.iter() {
                if self.fuzzy_match(text_key, &query_lower) {
                    for (id, entity, confidence) in entries {
                        if *confidence >= filter.min_confidence {
                            results.push((id.clone(), entity.clone(), *confidence));
                        }
                    }
                }
            }
        }
        
        // Remove duplicates and sort by confidence
        results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        results.dedup_by(|a, b| a.0 == b.0 && a.1.name == b.1.name);
        results.truncate(limit);
        
        Ok(results)
    }

    /// Calculate confidence score for an entity
    fn calculate_entity_confidence(&self, entity: &Entity) -> f64 {
        let mut confidence = entity.confidence; // Start with entity's confidence
        
        // Boost confidence for longer entity names
        if entity.name.len() > 10 {
            confidence += 0.1;
        }
        
        // Boost confidence for certain entity types
        match entity.entity_type {
            EntityType::Person | EntityType::Organization | EntityType::Location => confidence += 0.1,
            EntityType::Date | EntityType::Time | EntityType::Money => confidence += 0.05,
            _ => {}
        }
        
        // Cap at 1.0
        confidence.min(1.0)
    }

    /// Simple fuzzy matching for entity search
    fn fuzzy_match(&self, entity_text: &str, query: &str) -> bool {
        let entity_lower = entity_text.to_lowercase();
        let query_lower = query.to_lowercase();
        
        // Exact match
        if entity_lower.contains(&query_lower) {
            return true;
        }
        
        // Word boundary match
        let entity_words: Vec<&str> = entity_lower.split_whitespace().collect();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        
        for query_word in &query_words {
            for entity_word in &entity_words {
                if entity_word.starts_with(query_word) || query_word.starts_with(entity_word) {
                    return true;
                }
            }
        }
        
        false
    }

    pub async fn search_relationships(&self, entity: &str, relationship_type: Option<&str>, limit: usize) -> DataProcessingResult<Vec<RelationshipRecord>> {
        let relationship_index = self.relationships.lock().unwrap();
        let mut results = Vec::new();
        
        // Search by source entity
        if let Some(relationships) = relationship_index.get(&format!("source:{}", entity)) {
            for rel in relationships {
                if relationship_type.is_none() || rel.relationship_type == relationship_type.unwrap() {
                    results.push(rel.clone());
                }
            }
        }
        
        // Search by target entity
        if let Some(relationships) = relationship_index.get(&format!("target:{}", entity)) {
            for rel in relationships {
                if relationship_type.is_none() || rel.relationship_type == relationship_type.unwrap() {
                    results.push(rel.clone());
                }
            }
        }
        
        // Sort by confidence
        results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        
        Ok(results)
    }

    pub async fn get_metadata(&self, id: &ProcessingId) -> DataProcessingResult<HashMap<String, serde_json::Value>> {
        let entity_index = self.entities.lock().unwrap();
        let relationship_index = self.relationships.lock().unwrap();
        
        let mut entity_count = 0;
        let mut relationship_count = 0;
        
        // Count entities for this processing ID
        for entries in entity_index.values() {
            entity_count += entries.iter().filter(|(pid, _, _)| pid == id).count();
        }
        
        // Count relationships for this processing ID
        for entries in relationship_index.values() {
            relationship_count += entries.iter().filter(|rel| &rel.processing_id == id).count();
        }
        
        Ok(HashMap::from([
            ("indexed_at".to_string(), chrono::Utc::now().to_rfc3339().into()),
            ("entity_count".to_string(), entity_count.into()),
            ("relationship_count".to_string(), relationship_count.into()),
        ]))
    }
}

// Removed unused cosine_similarity function - will be re-added in v4 if needed

/// Consolidated indexer implementations from indexers crate

/// Full-text search query
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchQuery {
    pub text: String,
    pub project_scope: Option<String>,
    pub k: usize,
    pub max_tokens: usize,
}

/// Full-text search result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchResult {
    #[schemars(with = "String")]
    pub block_id: Uuid,
    pub score: f32,
    pub text_snippet: String,
    pub modality: String,
}

/// Vector search query
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VectorQuery {
    pub vector: Vec<f32>,
    pub model_id: String,
    pub k: usize,
    pub project_scope: Option<String>,
}

/// Vector search result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VectorSearchResult {
    #[schemars(with = "String")]
    pub block_id: Uuid,
    pub similarity: f32,
    pub modality: String,
}

/// BM25 statistics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Bm25Stats {
    pub total_documents: u64,
    pub total_terms: u64,
    pub avg_doc_length: f32,
    pub k1: f32,
    pub b: f32,
}

impl Default for Bm25Stats {
    fn default() -> Self {
        Self {
            total_documents: 0,
            total_terms: 0,
            avg_doc_length: 0.0,
            k1: 1.5,
            b: 0.75,
        }
    }
}

/// HNSW metadata
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HnswMetadata {
    pub total_vectors: usize,
    pub dimension: usize,
    pub max_neighbors: usize,
    pub ef_construction: usize,
    pub ef_search: usize,
}

/// Job types for indexing operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, JsonSchema)]
pub enum JobType {
    VideoIngest,
    SlidesIngest,
    DiagramIngest,
    CaptionsIngest,
    VisionOcr,
    AsrTranscription,
    EntityExtraction,
    VisualCaptioning,
    Embedding,
}

impl JobType {
    /// Get concurrency cap for this job type
    pub fn concurrency_cap(&self) -> usize {
        match self {
            JobType::VideoIngest => 2,
            JobType::SlidesIngest => 3,
            JobType::DiagramIngest => 3,
            JobType::CaptionsIngest => 5,
            JobType::VisionOcr => 2,
            JobType::AsrTranscription => 1, // ASR is expensive
            JobType::EntityExtraction => 4,
            JobType::VisualCaptioning => 1, // Expensive model inference
            JobType::Embedding => 2,
        }
    }

    /// Get timeout in milliseconds
    pub fn timeout_ms(&self) -> u64 {
        match self {
            JobType::VideoIngest => 300_000,      // 5 minutes
            JobType::SlidesIngest => 60_000,      // 1 minute
            JobType::DiagramIngest => 30_000,     // 30 seconds
            JobType::CaptionsIngest => 15_000,    // 15 seconds
            JobType::VisionOcr => 10_000,         // 10 seconds
            JobType::AsrTranscription => 120_000, // 2 minutes
            JobType::EntityExtraction => 30_000,  // 30 seconds
            JobType::VisualCaptioning => 30_000,  // 30 seconds
            JobType::Embedding => 45_000,         // 45 seconds
        }
    }
}

/// Job priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
pub enum JobPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Job status
#[derive(Debug, Clone, Copy, PartialEq, Eq, JsonSchema)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// BM25 full-text search indexer
#[derive(Debug)]
pub struct Bm25Indexer {
    documents: Arc<RwLock<HashMap<Uuid, DocumentRecord>>>,
    inverted_index: Arc<RwLock<HashMap<String, HashMap<Uuid, u32>>>>,
    stats: Arc<Mutex<Bm25Stats>>,
}


impl Bm25Indexer {
    /// Create a new BM25 indexer
    pub fn new() -> Self {
        Self {
            documents: Arc::new(RwLock::new(HashMap::new())),
            inverted_index: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(Mutex::new(Bm25Stats::default())),
        }
    }

    /// Index a block of text
    pub async fn index_block(&self, block_id: Uuid, text: &str, modality: &str) -> Result<(), anyhow::Error> {
        debug!(
            "Indexing block {} with {} chars in {}",
            block_id,
            text.len(),
            modality
        );

        // Tokenize and count terms
        let terms = self.tokenize(text);
        let mut term_freqs = HashMap::new();

        for term in &terms {
            *term_freqs.entry(term.clone()).or_insert(0) += 1;
        }

        let doc_length = terms.len();

        // Store document record
        let record = DocumentRecord {
            text: text.to_string(),
            modality: modality.to_string(),
            term_freqs: term_freqs.clone(),
            length: doc_length,
        };

        self.documents.write().insert(block_id, record);

        // Update inverted index
        // Update statistics
        let mut stats = self.stats.lock();
        stats.total_documents += 1;
        stats.total_terms += terms.len() as u64;
        let unique_terms_count = term_freqs.len();

        let mut inverted_index = self.inverted_index.write();
        for (term, freq) in term_freqs {
            inverted_index
                .entry(term)
                .or_insert_with(HashMap::new)
                .insert(block_id, freq);
        }

        debug!("Indexed block {} with {} unique terms", block_id, unique_terms_count);
        Ok(())
    }

    /// Search the index
    pub async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, anyhow::Error> {
        let terms = self.tokenize(&query.text);
        if terms.is_empty() {
            return Ok(Vec::new());
        }

        let documents = self.documents.read();
        let inverted_index = self.inverted_index.read();
        let stats = self.stats.lock();

        let mut scores = HashMap::new();

        // Calculate BM25 scores for each term
        for term in &terms {
            if let Some(doc_freqs) = inverted_index.get(term) {
                let idf = self.idf(stats.total_documents as usize, doc_freqs.len());

                for (doc_id, term_freq) in doc_freqs {
                    if let Some(doc) = documents.get(doc_id) {
                        let tf = *term_freq as f32;
                        let doc_len = doc.length as f32;
                        let avg_doc_len = stats.avg_doc_length;

                        let bm25_score = self.bm25_score(tf, doc_len, avg_doc_len, idf, stats.k1, stats.b);

                        *scores.entry(*doc_id).or_insert(0.0) += bm25_score;
                    }
                }
            }
        }

        // Convert to results and sort
        let mut results: Vec<SearchResult> = scores
            .into_iter()
            .map(|(block_id, score)| {
                let doc = &documents[&block_id];
                let snippet = self.extract_snippet(&doc.text, &query.text, query.max_tokens);

                SearchResult {
                    block_id,
                    score,
                    text_snippet: snippet,
                    modality: doc.modality.clone(),
                }
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(query.k);

        Ok(results)
    }

    /// Get statistics
    pub fn stats(&self) -> Bm25Stats {
        self.stats.lock().clone()
    }

    fn tokenize(&self, text: &str) -> Vec<String> {
        text.split_whitespace()
            .map(|word| word.to_lowercase().chars().filter(|c| c.is_alphanumeric()).collect())
            .filter(|word: &String| !word.is_empty() && word.len() > 2)
            .collect()
    }

    fn idf(&self, total_docs: usize, doc_freq: usize) -> f32 {
        if doc_freq == 0 {
            return 0.0;
        }
        ((total_docs as f32 - doc_freq as f32 + 0.5) / (doc_freq as f32 + 0.5)).ln()
    }

    fn bm25_score(&self, tf: f32, doc_len: f32, avg_doc_len: f32, idf: f32, k1: f32, b: f32) -> f32 {
        let numerator = tf * (k1 + 1.0);
        let denominator = tf + k1 * (1.0 - b + b * (doc_len / avg_doc_len));
        (numerator / denominator) * idf
    }

    fn extract_snippet(&self, text: &str, query: &str, max_tokens: usize) -> String {
        // Simple snippet extraction - find first occurrence of query terms
        let query_terms: std::collections::HashSet<_> = query.split_whitespace().collect();
        let words: Vec<&str> = text.split_whitespace().collect();

        for (_i, window) in words.windows(max_tokens).enumerate() {
            let window_set: std::collections::HashSet<_> = window.iter().cloned().collect();
            if !query_terms.is_disjoint(&window_set) {
                return window.join(" ");
            }
        }

        // Fallback to beginning of text
        words.iter().take(max_tokens).cloned().collect::<Vec<_>>().join(" ")
    }
}

/// HNSW indexer for vector search
#[derive(Debug)]
pub struct HnswIndexer {
    index: Arc<Mutex<SimpleHnswIndex>>,
    metadata: Arc<Mutex<HnswMetadata>>,
}

#[derive(Debug)]
struct SimpleHnswIndex {
    vectors: Vec<Vec<f32>>,
    dimension: usize,
}

impl SimpleHnswIndex {
    fn new(dimension: usize, _max_neighbors: usize) -> Self {
        Self {
            vectors: Vec::new(),
            dimension,
        }
    }

    fn insert(&mut self, vector: &[f32]) -> Result<usize, anyhow::Error> {
        if vector.len() != self.dimension {
            return Err(anyhow::anyhow!(
                "Vector dimension mismatch: expected {}, got {}",
                self.dimension,
                vector.len()
            ));
        }

        let id = self.vectors.len();
        self.vectors.push(vector.to_vec());
        Ok(id)
    }

    fn search(&self, query: &[f32], k: usize) -> Result<Vec<(usize, f32)>, anyhow::Error> {
        if query.len() != self.dimension {
            return Err(anyhow::anyhow!(
                "Query vector dimension mismatch: expected {}, got {}",
                self.dimension,
                query.len()
            ));
        }

        let mut similarities: Vec<(usize, f32)> = self
            .vectors
            .iter()
            .enumerate()
            .map(|(i, v)| (i, Self::cosine_similarity(query, v)))
            .collect();

        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        similarities.truncate(k);

        Ok(similarities)
    }

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }
}

impl HnswIndexer {
    /// Create a new HNSW indexer
    pub fn new(dimension: usize, max_neighbors: usize) -> Self {
        Self {
            index: Arc::new(Mutex::new(SimpleHnswIndex::new(dimension, max_neighbors))),
            metadata: Arc::new(Mutex::new(HnswMetadata {
                total_vectors: 0,
                dimension,
                max_neighbors,
                ef_construction: 200,
                ef_search: 64,
            })),
        }
    }

    /// Index a vector
    pub async fn index_vector(&self, vector: &[f32]) -> Result<Uuid, anyhow::Error> {
        let mut index = self.index.lock();
        let _id = index.insert(vector)?;

        let mut metadata = self.metadata.lock();
        metadata.total_vectors += 1;

        Ok(Uuid::new_v4()) // Return a UUID for the vector
    }

    /// Search for similar vectors
    pub async fn search(&self, query: &VectorQuery) -> Result<Vec<VectorSearchResult>, anyhow::Error> {
        let index = self.index.lock();

        let similarities = index.search(&query.vector, query.k)?;

        let results: Vec<VectorSearchResult> = similarities
            .into_iter()
            .map(|(_id, similarity)| VectorSearchResult {
                // TODO: Map similarity results back to original block IDs
                // - [ ] Store original ID mapping during indexing
                // - [ ] Retrieve original block_id from similarity result
                // - [ ] Handle ID mapping for different search backends
                // - [ ] Add unit tests for ID mapping accuracy
                block_id: Uuid::new_v4(), // In practice, this would map back to the original ID
                similarity,
                // TODO: Extract actual modality from indexed content
                // - [ ] Store modality metadata during indexing
                // - [ ] Retrieve modality from block metadata
                // - [ ] Support multiple modalities (text, image, audio, video)
                // - [ ] Add unit tests for modality extraction
                modality: "vector".to_string(), // TODO: Extract actual modality from content analysis
            })
            .collect();

        Ok(results)
    }

    /// Get metadata
    pub fn metadata(&self) -> HnswMetadata {
        self.metadata.lock().clone()
    }
}

/// Database connection pool for indexers
#[derive(Debug)]
pub struct DatabasePool {
    _pool: sqlx::Pool<sqlx::Postgres>,
}

impl DatabasePool {
    /// Create a new database pool
    pub async fn new(database_url: &str, max_connections: u32) -> Result<Self, anyhow::Error> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(database_url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create database pool: {}", e))?;

        // Test connection
        sqlx::query("SELECT 1")
            .execute(&pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;

        debug!(
            "Database pool initialized with max {} connections",
            max_connections
        );

        Ok(Self { _pool: pool })
    }

    /// Get number of idle connections
    pub fn num_idle(&self) -> usize {
        // TODO: Implement actual idle connection count based on pool type
        //       Currently returns placeholder; should query actual pool type for accurate idle connection count.
        0
    }

    /// Get pool size
    pub fn size(&self) -> usize {
        // TODO: Implement actual pool size based on pool type
        //       Currently returns placeholder; should query actual pool type for accurate pool size.
        0
    }
}

/// Vector store for database persistence
#[derive(Debug)]
pub struct VectorStore {
    _pool: DatabasePool,
}

impl VectorStore {
    pub fn new(pool: DatabasePool) -> Self {
        Self { _pool: pool }
    }

    /// Store a vector record
    pub async fn store_vector(&self, _record: BlockVectorRecord) -> Result<(), anyhow::Error> {
        // Implementation would store in database
        Ok(())
    }

    /// Retrieve vectors by block IDs
    pub async fn get_vectors(&self, _block_ids: &[Uuid]) -> Result<Vec<BlockVectorRecord>, anyhow::Error> {
        // Implementation would retrieve from database
        Ok(Vec::new())
    }

    /// Search vectors in database
    pub async fn search_vectors(&self, _query: &VectorQuery) -> Result<Vec<VectorSearchResult>, anyhow::Error> {
        // Implementation would perform vector search in database
        Ok(Vec::new())
    }
}

/// Vector record for database storage
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlockVectorRecord {
    #[schemars(with = "String")]
    pub block_id: Uuid,
    pub vector: Vec<f32>,
    pub model_id: String,
    pub modality: String,
    #[schemars(with = "String")]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Search audit entry for logging
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchAuditEntry {
    pub query: String,
    pub query_type: String,
    pub results_count: usize,
    pub search_time_ms: u64,
    #[schemars(with = "String")]
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Job scheduler for indexing operations
#[derive(Debug)]
pub struct JobScheduler {
    active_jobs: Arc<Mutex<HashMap<Uuid, IngestionJob>>>,
    job_queue: Arc<Mutex<Vec<IngestionJob>>>,
    concurrency_limits: HashMap<JobType, usize>,
}

#[derive(Debug, Clone, JsonSchema)]
pub struct IngestionJob {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub job_type: JobType,
    pub priority: JobPriority,
    pub status: JobStatus,
    pub payload: serde_json::Value,
    #[schemars(with = "String")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[schemars(with = "String")]
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    #[schemars(with = "String")]
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub timeout_ms: u64,
}

impl JobScheduler {
    pub fn new() -> Self {
        let mut concurrency_limits = HashMap::new();

        // Initialize concurrency limits for each job type
        for &job_type in &[
            JobType::VideoIngest,
            JobType::SlidesIngest,
            JobType::DiagramIngest,
            JobType::CaptionsIngest,
            JobType::VisionOcr,
            JobType::AsrTranscription,
            JobType::EntityExtraction,
            JobType::VisualCaptioning,
            JobType::Embedding,
        ] {
            concurrency_limits.insert(job_type, job_type.concurrency_cap());
        }

        Self {
            active_jobs: Arc::new(Mutex::new(HashMap::new())),
            job_queue: Arc::new(Mutex::new(Vec::new())),
            concurrency_limits,
        }
    }

    /// Submit a job for execution
    pub async fn submit_job(&self, job_type: JobType, payload: serde_json::Value, priority: JobPriority) -> Result<Uuid, anyhow::Error> {
        let job = IngestionJob {
            id: Uuid::new_v4(),
            job_type,
            priority,
            status: JobStatus::Pending,
            payload,
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            timeout_ms: job_type.timeout_ms(),
        };

        self.job_queue.lock().push(job.clone());

        debug!("Submitted job {} of type {:?}", job.id, job_type);
        Ok(job.id)
    }

    /// Get next job to execute
    pub async fn get_next_job(&self) -> Option<IngestionJob> {
        let mut queue = self.job_queue.lock();
        let mut active_jobs = self.active_jobs.lock();

        // Sort by priority (highest first)
        queue.sort_by(|a, b| b.priority.cmp(&a.priority));

        // Find first job that can run within concurrency limits
        for i in 0..queue.len() {
            let job = &queue[i];
            let active_count = active_jobs.values()
                .filter(|j| j.job_type == job.job_type && j.status == JobStatus::Running)
                .count();

            if active_count < *self.concurrency_limits.get(&job.job_type).unwrap_or(&1) {
                let job = queue.remove(i);
                active_jobs.insert(job.id, job.clone());
                return Some(job);
            }
        }

        None
    }

    /// Mark job as started
    pub async fn start_job(&self, job_id: Uuid) -> Result<(), anyhow::Error> {
        let mut active_jobs = self.active_jobs.lock();
        if let Some(job) = active_jobs.get_mut(&job_id) {
            job.status = JobStatus::Running;
            job.started_at = Some(chrono::Utc::now());
            Ok(())
        } else {
            Err(anyhow::anyhow!("Job {} not found", job_id))
        }
    }

    /// Mark job as completed
    pub async fn complete_job(&self, job_id: Uuid, success: bool) -> Result<(), anyhow::Error> {
        let mut active_jobs = self.active_jobs.lock();
        if let Some(job) = active_jobs.get_mut(&job_id) {
            job.status = if success { JobStatus::Completed } else { JobStatus::Failed };
            job.completed_at = Some(chrono::Utc::now());
            Ok(())
        } else {
            Err(anyhow::anyhow!("Job {} not found", job_id))
        }
    }

    /// Get job status
    pub async fn get_job_status(&self, job_id: Uuid) -> Option<IngestionJob> {
        let active_jobs = self.active_jobs.lock();
        active_jobs.get(&job_id).cloned()
    }

    /// Get queue statistics
    pub fn get_stats(&self) -> JobSchedulerStats {
        let queue = self.job_queue.lock();
        let active_jobs = self.active_jobs.lock();

        let pending_count = queue.len();
        let active_count = active_jobs.len();
        let completed_count = active_jobs.values()
            .filter(|j| j.status == JobStatus::Completed)
            .count();
        let failed_count = active_jobs.values()
            .filter(|j| j.status == JobStatus::Failed)
            .count();

        JobSchedulerStats {
            pending_jobs: pending_count,
            active_jobs: active_count,
            completed_jobs: completed_count,
            failed_jobs: failed_count,
        }
    }

    /// Get active job count - adapter method for multimodal orchestration
    pub fn get_active_job_count(&self) -> usize {
        self.get_stats().active_jobs
    }
}

/// Job scheduler statistics
#[derive(Debug, Clone, JsonSchema)]
pub struct JobSchedulerStats {
    pub pending_jobs: usize,
    pub active_jobs: usize,
    pub completed_jobs: usize,
    pub failed_jobs: usize,
}

/// Unified indexer combining BM25, HNSW, and database storage
#[derive(Debug)]
pub struct UnifiedIndexer {
    bm25_indexer: Bm25Indexer,
    hnsw_indexer: HnswIndexer,
    vector_store: Option<VectorStore>,
    job_scheduler: JobScheduler,
}

impl UnifiedIndexer {
    /// Create a new unified indexer
    pub fn new(dimension: usize, max_neighbors: usize) -> Self {
        Self {
            bm25_indexer: Bm25Indexer::new(),
            hnsw_indexer: HnswIndexer::new(dimension, max_neighbors),
            vector_store: None,
            job_scheduler: JobScheduler::new(),
        }
    }

    /// Index blocks - adapter method for multimodal orchestration
    pub async fn index_blocks(&self, blocks: Vec<EnrichedBlock>) -> Result<(), anyhow::Error> {
        for block in blocks {
            // Extract text content for indexing from the original block
            let text_content = match &block.block.data {
                BlockData::Text(text) => text.clone(),
                _ => continue, // Skip non-text blocks
            };

            // TODO: Generate real embeddings instead of placeholder vector
            // - [ ] Integrate with embedding service (agent-model-management or CoreML)
            // - [ ] Generate embeddings for text content using appropriate model
            // - [ ] Handle dimension consistency (match model output to index requirements)
            // - [ ] Add caching for repeated content
            // - [ ] Add batch processing for multiple blocks
            // - [ ] Add unit tests with mock embedding service
            // - [ ] Add integration tests with real embedding generation
            // TODO: Generate real embeddings from embedding service
            //       Currently uses placeholder vector; should generate actual embeddings from embedding service (agent-model-management or CoreML).
            let vector = vec![0.1; 384]; // Temporary: placeholder vector until embedding service integration

            // Index the content
            self.index_content(
                block.block.id.0, // Access the inner Uuid
                &text_content,
                &vector,
                &block.block.content_type.to_string(),
                // TODO: Use actual model name from embedding service
                // - [ ] Retrieve model name from embedding service configuration
                // - [ ] Store model metadata with indexed content
                // - [ ] Handle model versioning for compatibility
                "placeholder_model",
            ).await?;
        }

        Ok(())
    }

    /// Set vector store for persistence
    pub fn with_vector_store(mut self, vector_store: VectorStore) -> Self {
        self.vector_store = Some(vector_store);
        self
    }

    /// Index content with both text and vector indexing
    pub async fn index_content(
        &self,
        block_id: Uuid,
        text: &str,
        vector: &[f32],
        modality: &str,
        model_id: &str,
    ) -> Result<(), anyhow::Error> {
        // Index text with BM25
        self.bm25_indexer.index_block(block_id, text, modality).await?;

        // Index vector with HNSW
        self.hnsw_indexer.index_vector(vector).await?;

        // Store in database if available
        if let Some(vector_store) = &self.vector_store {
            let record = BlockVectorRecord {
                block_id,
                vector: vector.to_vec(),
                model_id: model_id.to_string(),
                modality: modality.to_string(),
                created_at: chrono::Utc::now(),
            };
            vector_store.store_vector(record).await?;
        }

        Ok(())
    }

    /// Search using hybrid approach
    pub async fn hybrid_search(&self, text_query: &str, vector_query: &[f32], k: usize) -> Result<Vec<HybridSearchResult>, anyhow::Error> {
        // Perform text search
        let text_results = self.bm25_indexer.search(&SearchQuery {
            text: text_query.to_string(),
            project_scope: None,
            k,
            max_tokens: 100,
        }).await?;

        // Perform vector search
        let _vector_results = self.hnsw_indexer.search(&VectorQuery {
            vector: vector_query.to_vec(),
            model_id: "default".to_string(),
            k,
            project_scope: None,
        }).await?;

        // TODO: Implement proper result fusion algorithm
        //       Currently uses basic combination; should implement proper fusion algorithm for combining search results from multiple sources.
        let mut hybrid_results = Vec::new();

        for text_result in text_results {
            hybrid_results.push(HybridSearchResult {
                block_id: text_result.block_id,
                text_score: text_result.score,
                vector_score: 0.0, // Would need to look up
                combined_score: text_result.score,
                modality: text_result.modality,
                text_snippet: text_result.text_snippet,
            });
        }

        // Sort by combined score
        hybrid_results.sort_by(|a, b| b.combined_score.partial_cmp(&a.combined_score).unwrap_or(std::cmp::Ordering::Equal));
        hybrid_results.truncate(k);

        Ok(hybrid_results)
    }

    /// Submit indexing job
    pub async fn submit_indexing_job(&self, job_type: JobType, payload: serde_json::Value) -> Result<Uuid, anyhow::Error> {
        self.job_scheduler.submit_job(job_type, payload, JobPriority::Normal).await
    }

    /// Get indexer statistics
    pub fn get_stats(&self) -> UnifiedIndexerStats {
        UnifiedIndexerStats {
            bm25_stats: self.bm25_indexer.stats(),
            hnsw_metadata: self.hnsw_indexer.metadata(),
            job_stats: self.job_scheduler.get_stats(),
        }
    }
}

/// Hybrid search result combining text and vector scores
#[derive(Debug, Clone, JsonSchema)]
pub struct HybridSearchResult {
    #[schemars(with = "String")]
    pub block_id: Uuid,
    pub text_score: f32,
    pub vector_score: f32,
    pub combined_score: f32,
    pub modality: String,
    pub text_snippet: String,
}

/// Unified indexer statistics
#[derive(Debug, Clone, JsonSchema)]
pub struct UnifiedIndexerStats {
    pub bm25_stats: Bm25Stats,
    pub hnsw_metadata: HnswMetadata,
    pub job_stats: JobSchedulerStats,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_indexing_stage_creation() {
        let stage = DefaultIndexingStage::new().await;
        assert!(stage.is_ok());
    }

    #[tokio::test]
    async fn test_fulltext_indexer() {
        let indexer = FullTextIndexer::new().await.unwrap();
        let id = ProcessingId::new();

        // Index some text
        indexer.index_text(id.clone(), "The quick brown fox jumps over the lazy dog").await.unwrap();

        // Search for text
        let results = indexer.search_text("fox", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, id);
        assert!(results[0].1 > 0.0);
    }

    #[tokio::test]
    async fn test_vector_indexer() {
        let indexer = VectorIndexer::new().await.unwrap();
        let id = ProcessingId::new();
        let vector = vec![1.0, 2.0, 3.0];

        // Index vector
        indexer.index_vector(id.clone(), vector.clone()).await.unwrap();

        // Search for similar vectors
        let results = indexer.search_similar(&vector, 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, id);
        assert_eq!(results[0].1, 1.0); // Perfect similarity with itself
    }

    #[tokio::test]
    async fn test_entity_indexer() {
        let indexer = EntityIndexer::new().await.unwrap();
        let id = ProcessingId::new();

        let entity = Entity {
            id: "test_entity".to_string(),
            name: "John Doe".to_string(),
            entity_type: EntityType::Person,
            confidence: 0.9,
            positions: vec![],
            metadata: HashMap::new(),
        };

        // Index entity
        indexer.index_entities(id.clone(), &[entity.clone()]).await.unwrap();

        // Search for entity
        let filter = EntityFilter {
            entity_type: EntityType::Person,
            entity_names: vec!["John Doe".to_string()],
            min_confidence: 0.5,
        };

        let results = indexer.search_entities(&filter, 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, id);
        assert_eq!(results[0].1.name, "John Doe");
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        let similarity = cosine_similarity(&a, &b);
        assert_eq!(similarity, 1.0);

        let c = vec![-1.0, -2.0, -3.0];
        let similarity_opposite = cosine_similarity(&a, &c);
        assert_eq!(similarity_opposite, -1.0);
    }
}
