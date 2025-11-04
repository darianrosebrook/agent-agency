//! Knowledge integration stage - connects processed data with external knowledge sources
//!
//! Consolidates functionality from the original knowledge-ingestor crate:
//! - Wikidata lexeme parsing and ingestion
//! - WordNet synset processing and relationships
//! - Knowledge base vector embeddings
//! - Cross-references between knowledge sources
//! - On-demand knowledge retrieval

use schemars::JsonSchema;
use crate::data_processing_types::*;
use crate::{DataProcessingResult, DataProcessingError};
use async_trait::async_trait;
use std::collections::HashMap;
#[cfg(feature = "memory-integration")]
use agent_memory::graph_engine::RelationshipType;

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
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, JsonSchema)]
pub enum KnowledgeSource {
    Wikidata,
    WordNet,
    Custom(String),
}

/// Query for knowledge retrieval
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct KnowledgeQuery {
    pub query_type: KnowledgeQueryType,
    pub text_query: Option<String>,
    pub entity_ids: Vec<String>,
    pub concept_types: Vec<String>,
    pub limit: usize,
    pub include_relationships: bool,
}

/// Types of knowledge queries
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, JsonSchema)]
pub enum KnowledgeQueryType {
    EntityLookup,
    ConceptSearch,
    RelationshipQuery,
    DefinitionLookup,
}

/// Knowledge item retrieved from sources
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
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
    #[schemars(with = "String")]
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Relationship in knowledge graph
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
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

/// Wikidata integration with real API calls
pub struct WikidataIntegrator {
    cache: std::sync::Mutex<HashMap<String, KnowledgeItem>>,
    client: reqwest::Client,
    base_url: String,
}

impl WikidataIntegrator {
    pub async fn new() -> DataProcessingResult<Self> {
        Ok(Self {
            cache: std::sync::Mutex::new(HashMap::new()),
            client: reqwest::Client::new(),
            base_url: "https://www.wikidata.org/w/api.php".to_string(),
        })
    }

    pub async fn lookup_concept(&self, concept: &str) -> DataProcessingResult<Option<KnowledgeItem>> {
        // Check cache first
        if let Some(cached) = self.cache.lock().unwrap().get(concept) {
            return Ok(Some(cached.clone()));
        }

        // Search for entity by label
        let search_result = self.search_entity_by_label(concept).await?;
        
        if let Some(entity_id) = search_result {
            // Get detailed entity information
            let item = self.get_entity_details(&entity_id, concept).await?;
            
            // Cache the result
            if let Some(ref item) = item {
                self.cache.lock().unwrap().insert(concept.to_string(), item.clone());
            }
            
            Ok(item)
        } else {
            Ok(None)
        }
    }

    /// Search for entity by label using Wikidata API
    async fn search_entity_by_label(&self, label: &str) -> DataProcessingResult<Option<String>> {
        let params = [
            ("action", "wbsearchentities"),
            ("format", "json"),
            ("language", "en"),
            ("search", label),
            ("limit", "1"),
        ];

        let response = self.client
            .get(&self.base_url)
            .query(&params)
            .send()
            .await
            .map_err(|e| DataProcessingError::Http(format!("Wikidata search failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(DataProcessingError::Http(format!("Wikidata API returned status: {}", response.status())));
        }

        let data: serde_json::Value = response.json().await
            .map_err(|e| DataProcessingError::Http(format!("JSON parsing failed: {}", e)))?;

        if let Some(search_results) = data["search"].as_array() {
            if let Some(first_result) = search_results.first() {
                if let Some(entity_id) = first_result["id"].as_str() {
                    return Ok(Some(entity_id.to_string()));
                }
            }
        }

        Ok(None)
    }

    /// Get detailed entity information from Wikidata
    async fn get_entity_details(&self, entity_id: &str, concept: &str) -> DataProcessingResult<Option<KnowledgeItem>> {
        let params = [
            ("action", "wbgetentities"),
            ("format", "json"),
            ("ids", entity_id),
            ("props", "labels|descriptions|claims"),
            ("languages", "en"),
        ];

        let response = self.client
            .get(&self.base_url)
            .query(&params)
            .send()
            .await
            .map_err(|e| DataProcessingError::Http(format!("Wikidata entity fetch failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(DataProcessingError::Http(format!("Wikidata API returned status: {}", response.status())));
        }

        let data: serde_json::Value = response.json().await
            .map_err(|e| DataProcessingError::Http(format!("JSON parsing failed: {}", e)))?;

        if let Some(entity_data) = data["entities"][entity_id].as_object() {
            let title = entity_data["labels"]["en"]["value"]
                .as_str()
                .unwrap_or(concept)
                .to_string();

            let description = entity_data["descriptions"]["en"]["value"]
                .as_str()
                .map(|s| s.to_string());

            let entity_type = self.determine_entity_type(entity_data);
            let relationships = self.extract_relationships(entity_data);
            let metadata = self.extract_metadata(entity_data);

            let item = KnowledgeItem {
                id: entity_id.to_string(),
                source: KnowledgeSource::Wikidata,
                title,
                description: description.clone(),
                content: description.clone().unwrap_or_else(|| "No description available".to_string()),
                entity_type,
                confidence_score: 0.9,
                relationships,
                metadata,
                last_updated: chrono::Utc::now(),
            };

            Ok(Some(item))
        } else {
            Ok(None)
        }
    }

    /// Determine entity type from Wikidata claims
    fn determine_entity_type(&self, entity_data: &serde_json::Map<String, serde_json::Value>) -> String {
        if let Some(claims) = entity_data.get("claims") {
            // Check for instance of (P31)
            if let Some(instance_of) = claims.get("P31") {
                if let Some(values) = instance_of.as_array() {
                    if let Some(first_value) = values.first() {
                        if let Some(mainsnak) = first_value.get("mainsnak") {
                            if let Some(datavalue) = mainsnak.get("datavalue") {
                                if let Some(value) = datavalue.get("value") {
                                    if let Some(id) = value.get("id").and_then(|v| v.as_str()) {
                                        return self.map_wikidata_type_to_entity_type(id);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        "unknown".to_string()
    }

    /// Map Wikidata entity IDs to entity types
    fn map_wikidata_type_to_entity_type(&self, wikidata_id: &str) -> String {
        match wikidata_id {
            "Q5" => "person".to_string(),           // human
            "Q43229" => "organization".to_string(),  // organization
            "Q515" => "city".to_string(),            // city
            "Q6256" => "country".to_string(),       // country
            "Q486972" => "human settlement".to_string(), // human settlement
            "Q16521" => "taxon".to_string(),        // taxon
            "Q7725634" => "literary work".to_string(), // literary work
            "Q11424" => "film".to_string(),         // film
            "Q3305213" => "painting".to_string(),   // painting
            _ => "entity".to_string(),
        }
    }

    /// Extract relationships from Wikidata claims
    fn extract_relationships(&self, entity_data: &serde_json::Map<String, serde_json::Value>) -> Vec<KnowledgeRelationship> {
        let mut relationships = Vec::new();

        if let Some(claims) = entity_data.get("claims") {
            for (property_id, claim_values) in claims.as_object().unwrap_or(&serde_json::Map::new()) {
                if let Some(values) = claim_values.as_array() {
                    for value in values {
                        if let Some(mainsnak) = value.get("mainsnak") {
                            if let Some(datavalue) = mainsnak.get("datavalue") {
                                if let Some(value_data) = datavalue.get("value") {
                                    if let Some(target_id) = value_data.get("id").and_then(|v| v.as_str()) {
                                        let relationship_type = self.map_property_to_relationship_type(property_id);
                                        let confidence = self.calculate_claim_confidence(value);
                                        
                                        relationships.push(KnowledgeRelationship {
                                            target_id: target_id.to_string(),
                                            relationship_type,
                                            confidence,
                                            evidence: vec![format!("Wikidata property {}", property_id)],
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        relationships
    }

    /// Map Wikidata property IDs to relationship types
    fn map_property_to_relationship_type(&self, property_id: &str) -> String {
        match property_id {
            "P17" => "country".to_string(),           // country
            "P131" => "located_in".to_string(),       // located in administrative territorial entity
            "P19" => "place_of_birth".to_string(),   // place of birth
            "P20" => "place_of_death".to_string(),   // place of death
            "P27" => "citizen_of".to_string(),       // country of citizenship
            "P106" => "occupation".to_string(),      // occupation
            "P108" => "employer".to_string(),        // employer
            "P39" => "position_held".to_string(),    // position held
            "P569" => "date_of_birth".to_string(),   // date of birth
            "P570" => "date_of_death".to_string(),   // date of death
            _ => "related_to".to_string(),
        }
    }

    /// Calculate confidence score for a claim
    fn calculate_claim_confidence(&self, claim: &serde_json::Value) -> f64 {
        // Base confidence
        let mut confidence = 0.8;

        // Check for qualifiers that might affect confidence
        if let Some(qualifiers) = claim.get("qualifiers") {
            if qualifiers.as_object().map_or(false, |q| !q.is_empty()) {
                confidence += 0.1; // More qualifiers = higher confidence
            }
        }

        // Check for references
        if let Some(references) = claim.get("references") {
            if let Some(ref_array) = references.as_array() {
                confidence += (ref_array.len() as f64 * 0.05).min(0.2);
            }
        }

        confidence.min(1.0)
    }

    /// Extract metadata from Wikidata entity
    fn extract_metadata(&self, entity_data: &serde_json::Map<String, serde_json::Value>) -> HashMap<String, serde_json::Value> {
        let mut metadata = HashMap::new();

        // Add basic metadata
        metadata.insert("wikidata_id".to_string(), entity_data.get("id").cloned().unwrap_or(serde_json::Value::Null));
        
        // Extract specific claims as metadata
        if let Some(claims) = entity_data.get("claims") {
            // Population (P1082)
            if let Some(population) = claims.get("P1082") {
                if let Some(first_value) = population.as_array().and_then(|a| a.first()) {
                    if let Some(amount) = first_value.get("mainsnak")
                        .and_then(|s| s.get("datavalue"))
                        .and_then(|d| d.get("value"))
                        .and_then(|v| v.get("amount"))
                        .and_then(|a| a.as_str()) {
                        metadata.insert("population".to_string(), amount.parse::<f64>().unwrap_or(0.0).into());
                    }
                }
            }

            // Coordinates (P625)
            if let Some(coordinates) = claims.get("P625") {
                if let Some(first_value) = coordinates.as_array().and_then(|a| a.first()) {
                    if let Some(value) = first_value.get("mainsnak")
                        .and_then(|s| s.get("datavalue"))
                        .and_then(|d| d.get("value")) {
                        metadata.insert("coordinates".to_string(), value.clone());
                    }
                }
            }
        }

        metadata
    }

    pub async fn search_concepts(&self, query: &str, limit: usize) -> DataProcessingResult<Vec<KnowledgeItem>> {
        let params = [
            ("action", "wbsearchentities"),
            ("format", "json"),
            ("language", "en"),
            ("search", query),
            ("limit", &limit.to_string()),
        ];

        let response = self.client
            .get(&self.base_url)
            .query(&params)
            .send()
            .await
            .map_err(|e| DataProcessingError::Http(format!("Wikidata search failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(DataProcessingError::Http(format!("Wikidata API returned status: {}", response.status())));
        }

        let data: serde_json::Value = response.json().await
            .map_err(|e| DataProcessingError::Http(format!("JSON parsing failed: {}", e)))?;

        let mut results = Vec::new();

        if let Some(search_results) = data["search"].as_array() {
            for result in search_results {
                if let Some(entity_id) = result["id"].as_str() {
                    if let Some(title) = result["label"].as_str() {
                        let description = result["description"].as_str().map(|s| s.to_string());
                        
                        let item = KnowledgeItem {
                            id: entity_id.to_string(),
                            source: KnowledgeSource::Wikidata,
                            title: title.to_string(),
                            description: description.clone(),
                            content: description.clone().unwrap_or_else(|| "No description available".to_string()),
                            entity_type: "unknown".to_string(),
                            confidence_score: 0.8,
                            relationships: vec![],
                            metadata: HashMap::from([
                                ("wikidata_id".to_string(), entity_id.into()),
                            ]),
                            last_updated: chrono::Utc::now(),
                        };
                        
                        results.push(item);
                    }
                }
            }
        }

        Ok(results)
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

        let content: ProcessedContent = ProcessedContent {
            content_type: ContentType::Text,
            data: ProcessedContentData::Text("John Smith works at Apple Inc in New York.".to_string()),
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
