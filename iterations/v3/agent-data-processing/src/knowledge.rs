//! Knowledge integration stage - connects processed data with external knowledge sources
//!
//! Consolidates functionality from the original knowledge-ingestor crate:
//! - Wikidata lexeme parsing and ingestion
//! - WordNet synset processing and relationships
//! - Knowledge base vector embeddings
//! - Cross-references between knowledge sources
//! - On-demand knowledge retrieval

use crate::types::*;
use crate::{DataProcessingResult, DataProcessingError};
use async_trait::async_trait;
use std::collections::HashMap;

/// Result from knowledge operations
pub type KnowledgeResult = DataProcessingResult<ProcessingOutput>;

/// Stage for knowledge integration operations
#[async_trait]
pub trait KnowledgeStage: Send + Sync {
    /// Get the name of this knowledge stage
    fn name(&self) -> &'static str;

    /// Integrate external knowledge with processed content
    async fn integrate_knowledge(&self, input: DataInput, content: ProcessedContent) -> KnowledgeResult;

    /// Retrieve knowledge for a query
    async fn retrieve_knowledge(&self, query: &KnowledgeQuery) -> DataProcessingResult<Vec<KnowledgeItem>>;

    /// Get supported knowledge sources
    fn supported_sources(&self) -> &[KnowledgeSource];
}

/// Types of knowledge sources supported
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum KnowledgeSource {
    Wikidata,
    WordNet,
    Custom(String),
}

/// Query for knowledge retrieval
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KnowledgeQuery {
    pub query_type: KnowledgeQueryType,
    pub text_query: Option<String>,
    pub entity_ids: Vec<String>,
    pub concept_types: Vec<String>,
    pub limit: usize,
    pub include_relationships: bool,
}

/// Types of knowledge queries
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum KnowledgeQueryType {
    EntityLookup,
    ConceptSearch,
    RelationshipQuery,
    DefinitionLookup,
}

/// Knowledge item retrieved from sources
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KnowledgeItem {
    pub id: String,
    pub source: KnowledgeSource,
    pub title: String,
    pub description: Option<String>,
    pub content: String,
    pub entity_type: String,
    pub confidence_score: f64,
    pub relationships: Vec<KnowledgeRelationship>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Relationship in knowledge graph
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KnowledgeRelationship {
    pub target_id: String,
    pub relationship_type: String,
    pub confidence: f64,
    pub evidence: Vec<String>,
}

/// Default implementation combining all knowledge sources
pub struct DefaultKnowledgeStage {
    wikidata_integrator: WikidataIntegrator,
    wordnet_integrator: WordNetIntegrator,
    knowledge_cache: KnowledgeCache,
}

impl DefaultKnowledgeStage {
    /// Create a new default knowledge stage
    pub async fn new() -> DataProcessingResult<Self> {
        Ok(Self {
            wikidata_integrator: WikidataIntegrator::new().await?,
            wordnet_integrator: WordNetIntegrator::new().await?,
            knowledge_cache: KnowledgeCache::new(),
        })
    }
}

#[async_trait]
impl KnowledgeStage for DefaultKnowledgeStage {
    fn name(&self) -> &'static str {
        "default_knowledge"
    }

    async fn integrate_knowledge(&self, input: DataInput, mut content: ProcessedContent) -> KnowledgeResult {
        let start_time = std::time::Instant::now();
        let mut errors = Vec::new();
        let mut knowledge_relationships = Vec::new();

        // Extract concepts and entities that might have external knowledge
        let concepts_to_lookup = self.extract_concepts_for_lookup(&content);

        // Query each knowledge source
        for concept in concepts_to_lookup {
            // Try Wikidata first
            match self.wikidata_integrator.lookup_concept(&concept).await {
                Ok(Some(knowledge)) => {
                    knowledge_relationships.extend(self.create_relationships_from_knowledge(&concept, &knowledge));
                }
                Ok(None) => {} // No match found
                Err(e) => errors.push(format!("Wikidata lookup failed for '{}': {}", concept, e)),
            }

            // Try WordNet
            match self.wordnet_integrator.lookup_concept(&concept).await {
                Ok(Some(knowledge)) => {
                    knowledge_relationships.extend(self.create_relationships_from_knowledge(&concept, &knowledge));
                }
                Ok(None) => {} // No match found
                Err(e) => errors.push(format!("WordNet lookup failed for '{}': {}", concept, e)),
            }

            // Cache successful lookups
            if knowledge_relationships.last().is_some() {
                let relationship = knowledge_relationships.last().unwrap();
                let knowledge_item = KnowledgeItem {
                    id: format!("relationship_{}", relationship.target_id),
                    source: KnowledgeSource::WordNet,
                    title: format!("Relationship: {}", relationship.relationship_type),
                    description: Some(format!("Knowledge relationship from WordNet")),
                    content: format!("{} -> {} (confidence: {:.2})", relationship.target_id, relationship.relationship_type, relationship.confidence),
                    entity_type: "concept".to_string(),
                    confidence_score: relationship.confidence,
                    relationships: vec![relationship.clone()],
                    metadata: HashMap::new(),
                    last_updated: chrono::Utc::now(),
                };
                self.knowledge_cache.store_concept_knowledge(&concept, knowledge_item).await?;
            }
        }

        // Add knowledge relationships to content
        for relationship in &knowledge_relationships {
            content.relationships.push(Relationship {
                id: format!("knowledge_{}_{}", relationship.target_id, relationship.relationship_type),
                source_entity: relationship.target_id.clone(), // This would be matched to existing entities
                target_entity: relationship.target_id.clone(),
                relationship_type: match relationship.relationship_type.as_str() {
                    "related_to" => RelationshipType::RelatedTo,
                    "part_of" => RelationshipType::PartOf,
                    "instance_of" => RelationshipType::RelatedTo,
                    "subclass_of" => RelationshipType::RelatedTo,
                    _ => RelationshipType::Other(relationship.relationship_type.clone()),
                },
                confidence: relationship.confidence,
                evidence: relationship.evidence.clone(),
            });
        }

        // Create metadata about knowledge integration
        let mut metadata = input.metadata.clone();
        metadata.insert("knowledge_integrated".to_string(), (!knowledge_relationships.is_empty()).into());
        metadata.insert("knowledge_sources_queried".to_string(),
            serde_json::to_value(vec!["wikidata", "wordnet"]).unwrap_or(serde_json::Value::Null));
        metadata.insert("knowledge_relationships_added".to_string(), knowledge_relationships.len().into());

        let stats = ProcessingStats {
            processing_time_ms: start_time.elapsed().as_millis() as u64,
            bytes_processed: 0, // Knowledge lookups don't process bytes directly
            entities_extracted: 0, // Knowledge integration doesn't extract new entities
            relationships_found: knowledge_relationships.len(),
            embeddings_generated: 0,
            errors_encountered: errors,
        };

        Ok(ProcessingOutput {
            id: input.id.clone(),
            original_input: input,
            processed_content: content,
            extracted_metadata: metadata,
            processing_stats: stats,
            created_at: chrono::Utc::now(),
        })
    }

    async fn retrieve_knowledge(&self, query: &KnowledgeQuery) -> DataProcessingResult<Vec<KnowledgeItem>> {
        let mut all_results = Vec::new();

        match query.query_type {
            KnowledgeQueryType::EntityLookup => {
                for entity_id in &query.entity_ids {
                    // Try cache first
                    if let Some(cached) = self.knowledge_cache.get_concept_knowledge(entity_id).await? {
                        all_results.push(cached);
                        continue;
                    }

                    // Query external sources
                    if let Ok(Some(item)) = self.wikidata_integrator.lookup_concept(entity_id).await {
                        all_results.push(item);
                    } else if let Ok(Some(item)) = self.wordnet_integrator.lookup_concept(entity_id).await {
                        all_results.push(item);
                    }
                }
            }

            KnowledgeQueryType::ConceptSearch => {
                if let Some(text) = &query.text_query {
                    // Search across all sources
                    if let Ok(results) = self.wikidata_integrator.search_concepts(text, query.limit).await {
                        all_results.extend(results);
                    }
                    if let Ok(results) = self.wordnet_integrator.search_concepts(text, query.limit).await {
                        all_results.extend(results);
                    }
                }
            }

            KnowledgeQueryType::DefinitionLookup => {
                for entity_id in &query.entity_ids {
                    if let Ok(Some(item)) = self.wikidata_integrator.get_definition(entity_id).await {
                        all_results.push(item);
                    } else if let Ok(Some(item)) = self.wordnet_integrator.get_definition(entity_id).await {
                        all_results.push(item);
                    }
                }
            }

            KnowledgeQueryType::RelationshipQuery => {
                for entity_id in &query.entity_ids {
                    if let Ok(relationships) = self.wikidata_integrator.get_relationships(entity_id).await {
                        let item = KnowledgeItem {
                            id: format!("relationships_{}", entity_id),
                            source: KnowledgeSource::Wikidata,
                            title: format!("Relationships for {}", entity_id),
                            description: Some(format!("Knowledge relationships for entity {}", entity_id)),
                            content: format!("Entity has {} relationships", relationships.len()),
                            entity_type: "relationship_set".to_string(),
                            confidence_score: 0.9,
                            relationships,
                            metadata: HashMap::new(),
                            last_updated: chrono::Utc::now(),
                        };
                        all_results.push(item);
                    }
                }
            }
        }

        // Sort by confidence and limit results
        all_results.sort_by(|a, b| b.confidence_score.partial_cmp(&a.confidence_score).unwrap_or(std::cmp::Ordering::Equal));
        all_results.truncate(query.limit);

        Ok(all_results)
    }

    fn supported_sources(&self) -> &[KnowledgeSource] {
        &[
            KnowledgeSource::Wikidata,
            KnowledgeSource::WordNet,
        ]
    }
}

#[async_trait]
impl crate::pipeline::PipelineStage for DefaultKnowledgeStage {
    fn name(&self) -> &'static str {
        "knowledge"
    }

    async fn process(&self, input: DataInput) -> DataProcessingResult<ProcessingOutput> {
        // For knowledge integration, we expect enriched content
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
                "Knowledge stage expects structured content".to_string()
            )),
        };

        self.integrate_knowledge(input, processed_content).await
    }
}

impl DefaultKnowledgeStage {
    /// Extract concepts from content that should be looked up in knowledge bases
    fn extract_concepts_for_lookup(&self, content: &ProcessedContent) -> Vec<String> {
        let mut concepts = Vec::new();

        // Extract from entities
        for entity in &content.entities {
            if matches!(entity.entity_type, EntityType::Person | EntityType::Organization |
                       EntityType::Location | EntityType::Event) {
                concepts.push(entity.name.clone());
            }
        }

        // Extract from text content (simple keyword extraction)
        if let Some(text) = &content.text_content {
            // Look for capitalized words that might be proper nouns
            for word in text.split_whitespace() {
                if word.len() > 3 && word.chars().next().unwrap().is_uppercase() &&
                   !word.contains('.') && !word.contains(',') {
                    concepts.push(word.to_string());
                }
            }
        }

        // Remove duplicates and limit
        concepts.sort();
        concepts.dedup();
        concepts.truncate(10); // Limit to avoid excessive lookups

        concepts
    }

    /// Create relationships from knowledge items
    fn create_relationships_from_knowledge(&self, _concept: &str, knowledge: &KnowledgeItem) -> Vec<KnowledgeRelationship> {
        knowledge.relationships.clone()
    }
}

/// Wikidata integration
pub struct WikidataIntegrator {
    // Would contain Wikidata API client and caching
    cache: std::sync::Mutex<HashMap<String, KnowledgeItem>>,
}

impl WikidataIntegrator {
    pub async fn new() -> DataProcessingResult<Self> {
        Ok(Self {
            cache: std::sync::Mutex::new(HashMap::new()),
        })
    }

    pub async fn lookup_concept(&self, concept: &str) -> DataProcessingResult<Option<KnowledgeItem>> {
        // Check cache first
        if let Some(cached) = self.cache.lock().unwrap().get(concept) {
            return Ok(Some(cached.clone()));
        }

        // Placeholder - would query Wikidata API
        // For demo purposes, return mock data for known concepts
        let item = match concept {
            "Paris" => Some(KnowledgeItem {
                id: "Q90".to_string(),
                source: KnowledgeSource::Wikidata,
                title: "Paris".to_string(),
                description: Some("Capital and most populous city of France".to_string()),
                content: "Paris is the capital and most populous city of France. It is located in northern France.".to_string(),
                entity_type: "city".to_string(),
                confidence_score: 0.95,
                relationships: vec![
                    KnowledgeRelationship {
                        target_id: "Q142".to_string(), // France
                        relationship_type: "located_in".to_string(),
                        confidence: 0.99,
                        evidence: vec!["Geographic fact".to_string()],
                    }
                ],
                metadata: HashMap::from([
                    ("wikidata_id".to_string(), "Q90".into()),
                    ("population".to_string(), 2140526.into()),
                ]),
                last_updated: chrono::Utc::now(),
            }),
            _ => None,
        };

        // Cache the result
        if let Some(ref item) = item {
            self.cache.lock().unwrap().insert(concept.to_string(), item.clone());
        }

        Ok(item)
    }

    pub async fn search_concepts(&self, _query: &str, _limit: usize) -> DataProcessingResult<Vec<KnowledgeItem>> {
        // Placeholder - would search Wikidata
        Ok(vec![])
    }

    pub async fn get_definition(&self, entity_id: &str) -> DataProcessingResult<Option<KnowledgeItem>> {
        self.lookup_concept(entity_id).await
    }

    pub async fn get_relationships(&self, entity_id: &str) -> DataProcessingResult<Vec<KnowledgeRelationship>> {
        if let Some(item) = self.lookup_concept(entity_id).await? {
            Ok(item.relationships)
        } else {
            Ok(vec![])
        }
    }
}

/// WordNet integration
pub struct WordNetIntegrator {
    // Would contain WordNet database access
    synsets: std::sync::Mutex<HashMap<String, KnowledgeItem>>,
}

impl WordNetIntegrator {
    pub async fn new() -> DataProcessingResult<Self> {
        Ok(Self {
            synsets: std::sync::Mutex::new(HashMap::new()),
        })
    }

    pub async fn lookup_concept(&self, concept: &str) -> DataProcessingResult<Option<KnowledgeItem>> {
        // Check cache first
        if let Some(cached) = self.synsets.lock().unwrap().get(concept) {
            return Ok(Some(cached.clone()));
        }

        // Placeholder - would query WordNet database
        // For demo purposes, return mock data for known concepts
        let item = match concept.to_lowercase().as_str() {
            "dog" => Some(KnowledgeItem {
                id: "wn_dog_1".to_string(),
                source: KnowledgeSource::WordNet,
                title: "dog".to_string(),
                description: Some("A domesticated carnivorous mammal".to_string()),
                content: "dog: a domesticated carnivorous mammal that typically has a long snout, an acute sense of smell, and a barking, howling, or whining voice.".to_string(),
                entity_type: "noun".to_string(),
                confidence_score: 0.9,
                relationships: vec![
                    KnowledgeRelationship {
                        target_id: "wn_animal_1".to_string(),
                        relationship_type: "hypernym".to_string(),
                        confidence: 0.95,
                        evidence: vec!["WordNet hierarchy".to_string()],
                    },
                    KnowledgeRelationship {
                        target_id: "wn_pet_1".to_string(),
                        relationship_type: "related_to".to_string(),
                        confidence: 0.8,
                        evidence: vec!["Common association".to_string()],
                    }
                ],
                metadata: HashMap::from([
                    ("wordnet_id".to_string(), "n01503061".into()),
                    ("part_of_speech".to_string(), "noun".into()),
                ]),
                last_updated: chrono::Utc::now(),
            }),
            _ => None,
        };

        // Cache the result
        if let Some(ref item) = item {
            self.synsets.lock().unwrap().insert(concept.to_string(), item.clone());
        }

        Ok(item)
    }

    pub async fn search_concepts(&self, _query: &str, _limit: usize) -> DataProcessingResult<Vec<KnowledgeItem>> {
        // Placeholder - would search WordNet
        Ok(vec![])
    }

    pub async fn get_definition(&self, entity_id: &str) -> DataProcessingResult<Option<KnowledgeItem>> {
        self.lookup_concept(entity_id).await
    }
}

/// Knowledge cache for performance
pub struct KnowledgeCache {
    cache: std::sync::Mutex<HashMap<String, KnowledgeItem>>,
}

impl KnowledgeCache {
    pub fn new() -> Self {
        Self {
            cache: std::sync::Mutex::new(HashMap::new()),
        }
    }

    pub async fn store_concept_knowledge(&self, concept: &str, knowledge: KnowledgeItem) -> DataProcessingResult<()> {
        self.cache.lock().unwrap().insert(concept.to_string(), knowledge);
        Ok(())
    }

    pub async fn get_concept_knowledge(&self, concept: &str) -> DataProcessingResult<Option<KnowledgeItem>> {
        Ok(self.cache.lock().unwrap().get(concept).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_knowledge_stage_creation() {
        let stage = DefaultKnowledgeStage::new().await;
        assert!(stage.is_ok());
    }

    #[tokio::test]
    async fn test_wikidata_lookup() {
        let integrator = WikidataIntegrator::new().await.unwrap();

        // Test known concept
        let result = integrator.lookup_concept("Paris").await.unwrap();
        assert!(result.is_some());
        let item = result.unwrap();
        assert_eq!(item.title, "Paris");
        assert_eq!(item.source, KnowledgeSource::Wikidata);

        // Test unknown concept
        let result = integrator.lookup_concept("UnknownConcept123").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_wordnet_lookup() {
        let integrator = WordNetIntegrator::new().await.unwrap();

        // Test known concept
        let result = integrator.lookup_concept("dog").await.unwrap();
        assert!(result.is_some());
        let item = result.unwrap();
        assert_eq!(item.title, "dog");
        assert_eq!(item.source, KnowledgeSource::WordNet);

        // Test unknown concept
        let result = integrator.lookup_concept("xyz123").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_knowledge_cache() {
        let cache = KnowledgeCache::new();

        let item = KnowledgeItem {
            id: "test".to_string(),
            source: KnowledgeSource::Wikidata,
            title: "Test Item".to_string(),
            description: Some("A test item".to_string()),
            content: "Test content".to_string(),
            entity_type: "test".to_string(),
            confidence_score: 0.8,
            relationships: vec![],
            metadata: HashMap::new(),
            last_updated: chrono::Utc::now(),
        };

        // Store and retrieve
        cache.store_concept_knowledge("test_concept", item.clone()).await.unwrap();
        let retrieved = cache.get_concept_knowledge("test_concept").await.unwrap();

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().title, "Test Item");
    }

    #[test]
    fn test_concept_extraction() {
        let stage = tokio::runtime::Runtime::new().unwrap().block_on(async {
            DefaultKnowledgeStage::new().await.unwrap()
        });

        let content = ProcessedContent {
            text_content: Some("John Smith works at Apple Inc in New York.".to_string()),
            structured_data: None,
            embeddings: None,
            entities: vec![
                Entity {
                    id: "john".to_string(),
                    name: "John Smith".to_string(),
                    entity_type: EntityType::Person,
                    confidence: 0.9,
                    positions: vec![],
                    metadata: HashMap::new(),
                }
            ],
            relationships: vec![],
            visual_elements: vec![],
            audio_transcript: None,
        };

        let concepts = stage.extract_concepts_for_lookup(&content);
        assert!(concepts.contains(&"John".to_string()) || concepts.contains(&"Smith".to_string()));
        assert!(concepts.contains(&"Apple".to_string()));
        assert!(concepts.contains(&"York".to_string()));
    }
}
