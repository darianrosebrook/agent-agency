//! Shared types and traits for disambiguation module

use schemars::JsonSchema;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;

/// Programming languages supported by the system

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum Language {
    Rust,
    TypeScript,
    Python,
    JavaScript,
    English, // For natural language processing
}

/// Result of disambiguation process

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DisambiguationResult {
    pub original_sentence: String,
    pub disambiguated_sentence: String,
    pub ambiguities_resolved: u32,
    pub unresolvable_ambiguities: Vec<UnresolvableAmbiguity>,
}

/// Represents an ambiguity found in text

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Ambiguity {
    pub text: String,
    pub ambiguity_type: AmbiguityType,
    pub start_pos: usize,
    pub end_pos: usize,
    pub position: (usize, usize),  // (start_pos, end_pos) tuple
    pub original_text: String,  // Alias for text
    pub possible_resolutions: Vec<String>,
    pub confidence: f64,
    pub context: Option<String>,
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum AmbiguityType {
    Pronoun,
    TechnicalTerm,
    ScopeBoundary,
    TemporalReference,
    Quantifier,
}

/// Ambiguity that cannot be resolved with available context

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UnresolvableAmbiguity {
    pub ambiguity: Ambiguity,
    pub reason: UnresolvableReason,
    pub suggested_context: Vec<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum UnresolvableReason {
    InsufficientContext,
    MultipleValidInterpretations,
    DomainSpecificUnknown,
    TemporalUncertainty,
}

/// Information about a pronoun referent

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReferentInfo {
    pub entity: String,
    pub confidence: f64,
    pub source: String,
}

/// Entity type classification

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum EntityType {
    Person,
    Organization,
    Location,
    Date,
    TechnicalTerm,
    Percent,
    Money,
}

/// Named entity with standardized field names

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NamedEntity {
    pub text: String,
    pub entity_type: EntityType,
    pub start: usize,
    pub end: usize,
    pub confidence: f64,
    pub context: Option<String>,
}

/// Entity match result for caching

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntityMatch {
    pub entity: NamedEntity,
    pub confidence: f64,
    pub match_type: String,
    pub source: String,
}

/// Knowledge base search result

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeBaseResult {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub canonical_name: String,
    pub source: KbSource, // Renamed to avoid collision
    pub properties: HashMap<String, String>,
}

/// Knowledge source types (renamed from KnowledgeSource to avoid collision)

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum KbSource {
    Wikidata,
    WordNet,
    Custom,
}

/// Related entity information

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RelatedEntity {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub canonical_name: String,
    pub relationship_type: String,
    pub confidence: f64,
}

/// Analysis helpers

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HistoricalEntityAnalysis {
    #[schemars(with = "String")]
    pub entity_id: Uuid,
    pub historical_context: String,
    pub temporal_relevance: f64,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntityRelationship {
    pub source_entity: String,
    pub target_entity: String,
    pub relationship: String,
    pub confidence: f64,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResolvedEntity {
    pub text: String,
    pub canonical_form: String,
    pub source: String,
    pub confidence: f64,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContextAwareDisambiguation {
    pub original_text: String,
    pub resolved_text: String,
    pub context_used: Vec<String>,
    pub confidence: f64,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DomainIntegration {
    pub domain: String,
    pub entities: Vec<String>,
    pub relationships: Vec<EntityRelationship>,
}

/// Trait for embedding providers
#[async_trait]
pub trait ClaimExtractionEmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
}

/// Trait for knowledge base operations
#[async_trait]
pub trait KnowledgeBase: Send + Sync {
    async fn semantic_search(
        &self,
        vector: &[f32],
        model_id: &str,
        limit: usize,
        min_score: f32,
    ) -> Result<Vec<KnowledgeBaseResult>>;

    async fn get_related(&self, id: Uuid, limit: usize) -> Result<Vec<RelatedEntity>>;

    async fn record_usage(&self, id: Uuid) -> Result<()>;

    async fn get_entity(&self, source: &str, key: &str) -> Result<Option<ExternalKnowledgeEntity>>;

    async fn upsert_entity(
        &self,
        entity: ExternalKnowledgeEntity,
        vectors: Vec<(String, Vec<f32>)>,
    ) -> Result<Uuid>;
}

/// External knowledge entity (simplified for disambiguation)

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExternalKnowledgeEntity {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub canonical_name: String,
    pub source: String,
    pub properties: HashMap<String, String>,
}

/// Trait for knowledge ingestion operations
#[async_trait]
pub trait KnowledgeIngest: Send + Sync {
    async fn trigger(&self, label: &str) -> Result<()>;
}

/// Supported ingestion channels for on-demand knowledge acquisition

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, Copy)]
pub enum IngestionChannell {
    Web,
    Api,
    Database,
    File,
}

impl IngestionChannel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Web => "web",
            Self::Api => "api",
            Self::Database => "database",
            Self::File => "file",
        }
    }
}

/// Ingestion candidate

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct IngestionCandidate {
    pub channel: IngestionChannel,
    pub label: String,
    pub priority: i32,
    pub estimated_cost: f64,
}

/// Scheduled ingestion source

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ScheduledSource {
    pub channel: IngestionChannel,
    pub schedule: String,
    pub last_run: Option<chrono::DateTime<chrono::Utc>>,
}

/// Ingestion cache entry

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct IngestionCacheEntry {
    pub key: String,
    pub data: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub ttl: std::time::Duration,
}

/// Pipeline statistics for ingestion

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct IngestionPipelineStats {
    pub total_candidates: usize,
    pub processed: usize,
    pub failed: usize,
    pub avg_processing_time: std::time::Duration,
}
