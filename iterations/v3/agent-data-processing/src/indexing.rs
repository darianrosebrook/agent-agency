//! Data indexing stage - creates searchable indexes for processed data
//!
//! Consolidates functionality from the original indexers crate:
//! - BM25 full-text search indexing
//! - HNSW approximate nearest neighbor search for embeddings
//! - Database persistence with connection pooling
//! - Job scheduler with concurrency governance

use crate::types::*;
use crate::{DataProcessingResult, DataProcessingError};
use async_trait::async_trait;
use std::collections::HashMap;

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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IndexType {
    FullText,
    Vector,
    Entity,
    Relationship,
}

/// Query for searching indexes
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IndexQuery {
    pub query_type: IndexQueryType,
    pub text_query: Option<String>,
    pub vector_query: Option<Vec<f32>>,
    pub entity_filters: Vec<EntityFilter>,
    pub limit: usize,
    pub include_metadata: bool,
}

/// Types of index queries
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum IndexQueryType {
    Text,
    Semantic,
    Hybrid,
    Entity,
}

/// Result from index search
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IndexResult {
    pub total_matches: usize,
    pub results: Vec<IndexMatch>,
    pub search_time_ms: u64,
    pub facets: HashMap<String, HashMap<String, usize>>,
}

/// Individual search result match
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
    job_scheduler: JobScheduler,
}

impl DefaultIndexingStage {
    /// Create a new default indexing stage
    pub async fn new() -> DataProcessingResult<Self> {
        Ok(Self {
            fulltext_indexer: FullTextIndexer::new().await?,
            vector_indexer: VectorIndexer::new().await?,
            entity_indexer: EntityIndexer::new().await?,
            job_scheduler: JobScheduler::new(),
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
        let input_id = input.id.clone(); // Clone once to avoid multiple moves

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
                    text_content: match_.content_snippet,
                    structured_data: None,
                    embeddings: None,
                    entities: match_.matched_entities.clone(),
                    relationships: vec![],
                    visual_elements: vec![],
                    audio_transcript: None,
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
    // Would contain BM25 index implementation
    index: std::sync::Mutex<HashMap<ProcessingId, String>>, // Placeholder
}

impl FullTextIndexer {
    pub async fn new() -> DataProcessingResult<Self> {
        Ok(Self {
            index: std::sync::Mutex::new(HashMap::new()),
        })
    }

    pub async fn index_text(&self, id: ProcessingId, text: &str) -> DataProcessingResult<()> {
        let mut index = self.index.lock().unwrap();
        index.insert(id, text.to_string());
        Ok(())
    }

    pub async fn search_text(&self, query: &str, limit: usize) -> DataProcessingResult<Vec<(ProcessingId, f64, Option<String>)>> {
        let index = self.index.lock().unwrap();
        let mut results = Vec::new();

        for (id, text) in index.iter() {
            if text.contains(query) {
                // Simple scoring based on occurrence count
                let score = text.matches(query).count() as f64;
                let snippet = text.lines().next().map(|s| s.to_string());
                results.push((id.clone(), score, snippet));
            }
        }

        // Sort by score descending
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);

        Ok(results)
    }

    pub async fn get_metadata(&self, _id: &ProcessingId) -> DataProcessingResult<HashMap<String, serde_json::Value>> {
        // Placeholder - would retrieve metadata from index
        Ok(HashMap::from([
            ("indexed_at".to_string(), chrono::Utc::now().to_rfc3339().into()),
            ("content_length".to_string(), 1000.into()),
        ]))
    }
}

/// Vector similarity search indexer using HNSW
pub struct VectorIndexer {
    // Would contain HNSW index implementation
    vectors: std::sync::Mutex<HashMap<ProcessingId, Vec<f32>>>, // Placeholder
}

impl VectorIndexer {
    pub async fn new() -> DataProcessingResult<Self> {
        Ok(Self {
            vectors: std::sync::Mutex::new(HashMap::new()),
        })
    }

    pub async fn index_vector(&self, id: ProcessingId, vector: Vec<f32>) -> DataProcessingResult<()> {
        let mut vectors = self.vectors.lock().unwrap();
        vectors.insert(id, vector);
        Ok(())
    }

    pub async fn search_similar(&self, query_vector: &[f32], limit: usize) -> DataProcessingResult<Vec<(ProcessingId, f64)>> {
        let vectors = self.vectors.lock().unwrap();
        let mut results = Vec::new();

        for (id, vector) in vectors.iter() {
            // Simple cosine similarity (placeholder)
            let similarity = cosine_similarity(query_vector, vector);
            results.push((id.clone(), similarity));
        }

        // Sort by similarity descending
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);

        Ok(results)
    }
}

/// Entity and relationship indexer
pub struct EntityIndexer {
    // Would contain entity/relationship index implementation
    entities: std::sync::Mutex<HashMap<String, Vec<(ProcessingId, Entity, f64)>>>, // Placeholder
}

impl EntityIndexer {
    pub async fn new() -> DataProcessingResult<Self> {
        Ok(Self {
            entities: std::sync::Mutex::new(HashMap::new()),
        })
    }

    pub async fn index_entities(&self, id: ProcessingId, entities: &[Entity]) -> DataProcessingResult<()> {
        let mut entity_index = self.entities.lock().unwrap();

        for entity in entities {
            entity_index.entry(entity.name.clone())
                .or_insert_with(Vec::new)
                .push((id.clone(), entity.clone(), entity.confidence));
        }

        Ok(())
    }

    pub async fn index_relationships(&self, _id: ProcessingId, _relationships: &[Relationship]) -> DataProcessingResult<()> {
        // Placeholder - would index relationships
        Ok(())
    }

    pub async fn search_entities(&self, filter: &EntityFilter, limit: usize) -> DataProcessingResult<Vec<(ProcessingId, Entity, f64)>> {
        let entity_index = self.entities.lock().unwrap();
        let mut results = Vec::new();

        if let Some(entity_matches) = entity_index.get(&filter.entity_names[0]) {
            for (id, entity, confidence) in entity_matches {
                if *confidence >= filter.min_confidence {
                    results.push((id.clone(), entity.clone(), *confidence));
                }
            }
        }

        // Sort by confidence
        results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);

        Ok(results)
    }
}

/// Job scheduler for indexing operations
pub struct JobScheduler {
    // Would contain job scheduling implementation
}

impl JobScheduler {
    pub fn new() -> Self {
        Self {}
    }

    // Would implement job scheduling methods
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        (dot_product / (norm_a * norm_b)) as f64
    }
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
