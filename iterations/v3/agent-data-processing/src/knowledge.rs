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

// Production-grade additions
use governor::{Quota, RateLimiter};
use nonzero_ext::nonzero;
use backoff::{ExponentialBackoff, future::retry};
use parking_lot::Mutex;
use moka::future::Cache;
use std::sync::Arc;
use futures::{stream, StreamExt};
// Use workspace sqlx instead of rusqlite to avoid conflicts
// use deadpool_sqlite::{Config, Pool, Runtime};

/// Tagged payload for safer knowledge stage processing
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "payload")]
enum KnowledgePayload {
    ProcessedContent(ProcessedContent),
}

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

/// Normalized relationship types for better downstream reasoning
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CanonRel {
    RelatedTo,
    PartOf,
    LocatedIn,
    CitizenOf,
    Employer,
    Occupation,
    Hypernym,
    InstanceOf,
    SubclassOf,
    Other(String),
}

impl CanonRel {
    /// Map Wikidata/WordNet relationship strings to canonical types
    pub fn from_str(s: &str) -> Self {
        match s {
            "part_of" => CanonRel::PartOf,
            "located_in" | "located_in_adm" => CanonRel::LocatedIn,
            "citizen_of" => CanonRel::CitizenOf,
            "employer" => CanonRel::Employer,
            "occupation" => CanonRel::Occupation,
            "hypernym" => CanonRel::Hypernym,
            "instance_of" => CanonRel::InstanceOf,
            "subclass_of" => CanonRel::SubclassOf,
            _ => CanonRel::Other(s.to_string()),
        }
    }
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

        // Concurrent lookup with bounded fan-out
        let (knowledge_items, lookup_errors) = self.lookup_many(concepts_to_lookup).await;
        errors.extend(lookup_errors);

        // Process results and create relationships
        for knowledge in knowledge_items {
            let title = knowledge.title.clone();
            knowledge_relationships.extend(self.create_relationships_from_knowledge(&title, &knowledge));

            // Cache successful lookups
            self.knowledge_cache.store_concept_knowledge(&title, knowledge).await?;
        }

        // Fuse relationships by target to reduce redundancy and boost confidence
        knowledge_relationships = Self::fuse_relationships_by_target(knowledge_relationships);

        // Add knowledge relationships to content with normalized types and provenance
        for relationship in &knowledge_relationships {
            content.relationships.push(Relationship {
                id: format!("knowledge_{}_{}", relationship.target_id, relationship.relationship_type),
                source_entity: relationship.target_id.clone(), // This would be matched to existing entities
                target_entity: relationship.target_id.clone(),
                relationship_type: match CanonRel::from_str(&relationship.relationship_type) {
                    CanonRel::RelatedTo => RelationshipType::RelatedTo,
                    CanonRel::PartOf => RelationshipType::PartOf,
                    CanonRel::LocatedIn => RelationshipType::Other("located_in".into()),
                    CanonRel::CitizenOf => RelationshipType::Other("citizen_of".into()),
                    CanonRel::Employer => RelationshipType::Other("employer".into()),
                    CanonRel::Occupation => RelationshipType::Other("occupation".into()),
                    CanonRel::Hypernym => RelationshipType::Other("hypernym".into()),
                    CanonRel::InstanceOf => RelationshipType::RelatedTo,
                    CanonRel::SubclassOf => RelationshipType::RelatedTo,
                    CanonRel::Other(x) => RelationshipType::Other(x),
                },
                confidence: relationship.confidence,
                evidence: {
                    let mut e = relationship.evidence.clone();
                    e.push("source:wikidata".into()); // Add provenance
                    e
                },
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
        // Use tagged payload for safer deserialization
        let processed_content = match &input.content {
            DataContent::Structured(data) => {
                let payload: KnowledgePayload = serde_json::from_value(data.clone())
                    .map_err(|e| DataProcessingError::Validation(
                        format!("Expected KnowledgePayload in structured data: {}", e)
                    ))?;

                match payload {
                    KnowledgePayload::ProcessedContent(pc) => pc,
                }
            }
            _ => return Err(DataProcessingError::Validation(
                "Knowledge stage expects structured content with KnowledgePayload".to_string()
            )),
        };

        self.integrate_knowledge(input, processed_content).await
    }
}

impl DefaultKnowledgeStage {
    /// Extract concepts from content that should be looked up in knowledge bases
    /// Prefers structured entities, falls back to light NP heuristics, avoids noise
    fn extract_concepts_for_lookup(&self, content: &ProcessedContent) -> Vec<String> {
        let mut concepts = Vec::new();

        // 1) Trust structured entities first (highest confidence)
        for entity in &content.entities {
            match entity.entity_type {
                EntityType::Person | EntityType::Organization | EntityType::Location | EntityType::Event => {
                    concepts.push(entity.name.trim().to_string());
                }
                _ => {} // Skip other entity types for now
            }
        }

        // 2) Text fallback: collect TitleCase multiword spans (very light NP heuristic)
        if let Some(text) = &content.text_content {
            let mut current = Vec::new();
            for token in text.split_whitespace() {
                let clean = token.trim_matches(|c: char| !c.is_alphanumeric());
                let is_title = clean.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);

                if is_title && clean.len() > 1 {
                    current.push(clean);
                } else if !current.is_empty() {
                    // End of potential NP span
                    concepts.push(current.join(" "));
                    current.clear();
                }
            }
            // Handle trailing span
            if !current.is_empty() {
                concepts.push(current.join(" "));
            }
        }

        // Clean and dedupe
        concepts.retain(|s| !s.is_empty());
        concepts.sort();
        concepts.dedup();
        concepts.truncate(10); // Limit to avoid excessive lookups

        concepts
    }

    /// Concurrent lookup with bounded fan-out and deduplication
    async fn lookup_many(&self, concepts: Vec<String>) -> (Vec<KnowledgeItem>, Vec<String>) {
        use futures::{stream, StreamExt};

        let mut errors = Vec::new();
        let unique: Vec<String> = {
            let mut v = concepts;
            v.sort();
            v.dedup();
            v
        };

        let sem = Arc::new(tokio::sync::Semaphore::new(5));

        let wikidata = &self.wikidata_integrator;
        let wordnet = &self.wordnet_integrator;

        let items = stream::iter(unique.into_iter().map(|c| {
            let sem = sem.clone();
            async move {
                let _permit = sem.acquire_owned().await.unwrap();
                // Try cache via integrators first
                if let Ok(Some(i)) = wikidata.lookup_concept(&c).await { return Ok::<KnowledgeItem, String>(i); }
                if let Ok(Some(i)) = wordnet.lookup_concept(&c).await { return Ok(i); }
                Err::<KnowledgeItem, String>(c)
            }
        }))
        .buffer_unordered(5)
        .filter_map(|r| async {
            match r {
                Ok(k) => Some(Ok(k)),
                Err(missed) => Some(Err(missed)),
            }
        })
        .collect::<Vec<_>>()
        .await;

        let mut ok = Vec::new();
        for r in items {
            match r {
                Ok(i) => ok.push(i),
                Err(c) => errors.push(format!("no result for concept: {c}")),
            }
        }

        (ok, errors)
    }

    /// Create relationships from knowledge items
    fn create_relationships_from_knowledge(&self, _concept: &str, knowledge: &KnowledgeItem) -> Vec<KnowledgeRelationship> {
        knowledge.relationships.clone()
    }

    /// Fuse relationships by target, preferring corroborated relationships
    fn fuse_relationships_by_target(relationships: Vec<KnowledgeRelationship>) -> Vec<KnowledgeRelationship> {
        use std::collections::BTreeMap;

        let mut map: BTreeMap<(String, String), (f64, Vec<String>)> = BTreeMap::new();

        for r in relationships {
            let k = (r.target_id.clone(), r.relationship_type.clone());
            let entry = map.entry(k).or_insert((0.0, Vec::new()));
            // Noisy-OR combination for multiple sources
            entry.0 = 1.0 - (1.0 - entry.0) * (1.0 - r.confidence);
            entry.1.extend(r.evidence);
        }

        map.into_iter().map(|((t, rel), (c, ev))| KnowledgeRelationship {
            target_id: t,
            relationship_type: rel,
            confidence: c.min(1.0),
            evidence: ev,
        }).collect()
    }
}

/// Wikidata integration with production-grade reliability
pub struct WikidataIntegrator {
    cache: parking_lot::Mutex<HashMap<String, KnowledgeItem>>,
    client: reqwest::Client,
    base_url: String,
    limiter: RateLimiter<
        governor::state::NotKeyed,
        governor::state::InMemoryState,
        governor::clock::DefaultClock,
        governor::middleware::NoOpMiddleware,
    >,
}

impl WikidataIntegrator {
    pub async fn new() -> DataProcessingResult<Self> {
        let client = reqwest::Client::builder()
            .user_agent(concat!("AgentAgency/KnowledgeStage ", env!("CARGO_PKG_VERSION")))
            .pool_idle_timeout(std::time::Duration::from_secs(30))
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| DataProcessingError::Http(format!("client build: {e}")))?;

        let limiter = RateLimiter::direct(
            Quota::per_minute(nonzero!(60u32))
        );

        Ok(Self {
            cache: parking_lot::Mutex::new(HashMap::new()),
            client,
            base_url: "https://www.wikidata.org/w/api.php".to_string(),
            limiter,
        })
    }

    /// Get JSON with rate limiting, retries, and backoff
    async fn get_json(&self, params: &[(&str, &str)]) -> DataProcessingResult<serde_json::Value> {
        self.limiter.until_ready().await;

        let op = || async {
            let resp = self.client
                .get(&self.base_url)
                .query(params)
                .send()
                .await
                .map_err(anyhow::Error::from)?;

            if resp.status().is_success() {
                let v = resp.json::<serde_json::Value>().await
                    .map_err(anyhow::Error::from)?;
                Ok(v)
            } else if resp.status().as_u16() == 429 {
                // Rate limited - retry
                Err(backoff::Error::transient(anyhow::anyhow!("429")))
            } else {
                Err(backoff::Error::permanent(anyhow::anyhow!("HTTP {}", resp.status())))
            }
        };

        retry(ExponentialBackoff::default(), op).await
            .map_err(|e| DataProcessingError::Http(format!("wikidata request failed: {e}")))
    }

    pub async fn lookup_concept(&self, concept: &str) -> DataProcessingResult<Option<KnowledgeItem>> {
        // Check cache first
        {
            let cache_guard = self.cache.lock();
            if let Some(cached) = cache_guard.get(concept) {
                return Ok(Some(cached.clone()));
            }
        }

        // Search for entity by label
        let search_result = self.search_entity_by_label(concept).await?;
        
        if let Some(entity_id) = search_result {
            // Get detailed entity information
            let item = self.get_entity_details(&entity_id, concept).await?;
            
            // Cache the result
            if let Some(ref item) = item {
                self.cache.lock().insert(concept.to_string(), item.clone());
            }
            
            Ok(item)
        } else {
            Ok(None)
        }
    }

    /// Search for entity by label using Wikidata API with production-grade reliability
    async fn search_entity_by_label(&self, label: &str) -> DataProcessingResult<Option<String>> {
        let params = [
            ("action", "wbsearchentities"),
            ("format", "json"),
            ("language", "en"),
            ("search", label),
            ("limit", "1"),
        ];

        let data = self.get_json(&params).await?;

        if let Some(search_results) = data["search"].as_array() {
            if let Some(first_result) = search_results.first() {
                if let Some(entity_id) = first_result["id"].as_str() {
                    return Ok(Some(entity_id.to_string()));
                }
            }
        }

        Ok(None)
    }

    /// Get detailed entity information from Wikidata with production-grade reliability
    async fn get_entity_details(&self, entity_id: &str, concept: &str) -> DataProcessingResult<Option<KnowledgeItem>> {
        let params = [
            ("action", "wbgetentities"),
            ("format", "json"),
            ("ids", entity_id),
            ("props", "labels|descriptions|claims"),
            ("languages", "en"),
        ];

        let data = self.get_json(&params).await?;

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

        let data = self.get_json(&params).await?;

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

/// Pluggable WordNet backend trait
#[async_trait::async_trait]
pub trait WordNetBackend: Send + Sync {
    async fn lookup(&self, lemma: &str) -> anyhow::Result<Option<KnowledgeItem>>;
    async fn search(&self, query: &str, limit: usize) -> anyhow::Result<Vec<KnowledgeItem>>;
}

/// SQLite-backed WordNet implementation using Princeton WordNet 3.1 (sqlx version)
pub struct SqliteWordNet {
    pool: sqlx::SqlitePool,
}

impl SqliteWordNet {
    pub async fn new() -> DataProcessingResult<Self> {
        // TODO: Load WordNet data from models/wiki-wordnet/wn3.1.dict.tar.gz
        // For now, create in-memory database - would need data loading script
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await
            .map_err(DataProcessingError::Database)?;

        // Initialize schema (simplified - would need full WordNet schema)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS synsets (
                lemma TEXT PRIMARY KEY,
                synset_id TEXT NOT NULL,
                definition TEXT NOT NULL
            )"
        )
        .execute(&pool)
        .await
        .map_err(DataProcessingError::Database)?;

        Ok(Self { pool })
    }
}

#[async_trait::async_trait]
impl WordNetBackend for SqliteWordNet {
    async fn lookup(&self, lemma: &str) -> anyhow::Result<Option<KnowledgeItem>> {
        let lemma_lower = lemma.to_lowercase();

        let row: Option<(String, String)> = sqlx::query_as(
            "SELECT synset_id, definition FROM synsets WHERE lemma = ?1 LIMIT 1"
        )
        .bind(&lemma_lower)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(id, def)| KnowledgeItem {
            id: format!("wn_{id}"),
            source: KnowledgeSource::WordNet,
            title: lemma_lower.clone(),
            description: Some(def.clone()),
            content: def,
            entity_type: "synset".into(),
            confidence_score: 0.9,
            relationships: vec![], // Could add hypernyms via joins
            metadata: HashMap::new(),
            last_updated: chrono::Utc::now(),
        }))
    }

    async fn search(&self, query: &str, limit: usize) -> anyhow::Result<Vec<KnowledgeItem>> {
        let query = format!("%{}%", query.to_lowercase());

        let rows: Vec<(String, String, String)> = sqlx::query_as(
            "SELECT lemma, synset_id, definition FROM synsets WHERE lemma LIKE ?1 LIMIT ?2"
        )
        .bind(query)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        let items = rows.into_iter().map(|(lemma, id, def)| KnowledgeItem {
            id: format!("wn_{id}"),
            source: KnowledgeSource::WordNet,
            title: lemma.clone(),
            description: Some(def.clone()),
            content: def,
            entity_type: "synset".into(),
            confidence_score: 0.8,
            relationships: vec![],
            metadata: HashMap::new(),
            last_updated: chrono::Utc::now(),
        }).collect();

        Ok(items)
    }
}

/// Mock WordNet implementation for testing/CI
pub struct MockWordNet {
    synsets: parking_lot::Mutex<HashMap<String, KnowledgeItem>>,
}

impl MockWordNet {
    pub async fn new() -> DataProcessingResult<Self> {
        let mut synsets = HashMap::new();

        // Add mock data for testing
        synsets.insert("dog".to_string(), KnowledgeItem {
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
        });

        Ok(Self {
            synsets: parking_lot::Mutex::new(synsets),
        })
    }
}

#[async_trait::async_trait]
impl WordNetBackend for MockWordNet {
    async fn lookup(&self, lemma: &str) -> anyhow::Result<Option<KnowledgeItem>> {
        Ok(self.synsets.lock().get(lemma).cloned())
    }

    async fn search(&self, _query: &str, _limit: usize) -> anyhow::Result<Vec<KnowledgeItem>> {
        Ok(vec![])
    }
}

/// WordNet integration with pluggable backends
pub struct WordNetIntegrator {
    backend: Box<dyn WordNetBackend>,
}

impl WordNetIntegrator {
    pub async fn new() -> DataProcessingResult<Self> {
        // Default to mock for CI/testing; use SQLite in production
        let backend: Box<dyn WordNetBackend> = if std::env::var("WORDNET_SQLITE").is_ok() {
            Box::new(SqliteWordNet::new().await?)
        } else {
            Box::new(MockWordNet::new().await?)
        };

        Ok(Self { backend })
    }

    pub async fn lookup_concept(&self, concept: &str) -> DataProcessingResult<Option<KnowledgeItem>> {
        self.backend.lookup(concept).await
            .map_err(|e| DataProcessingError::Other(format!("WordNet lookup failed: {:?}", e)))
    }

    pub async fn search_concepts(&self, query: &str, limit: usize) -> DataProcessingResult<Vec<KnowledgeItem>> {
        self.backend.search(query, limit).await
            .map_err(|e| DataProcessingError::Other(format!("WordNet search failed: {:?}", e)))
    }

    pub async fn get_definition(&self, entity_id: &str) -> DataProcessingResult<Option<KnowledgeItem>> {
        self.lookup_concept(entity_id).await
    }
}

/// Production-grade knowledge cache with TTL and size bounds
pub struct KnowledgeCache {
    cache: Cache<String, KnowledgeItem>,
}

impl KnowledgeCache {
    pub fn new() -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(50_000)
                .time_to_live(std::time::Duration::from_secs(12 * 60 * 60))
                .build(),
        }
    }

    pub async fn store_concept_knowledge(&self, concept: &str, knowledge: KnowledgeItem) -> DataProcessingResult<()> {
        self.cache.insert(concept.to_string(), knowledge).await;
        Ok(())
    }

    pub async fn get_concept_knowledge(&self, concept: &str) -> DataProcessingResult<Option<KnowledgeItem>> {
        Ok(self.cache.get(concept).await)
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
