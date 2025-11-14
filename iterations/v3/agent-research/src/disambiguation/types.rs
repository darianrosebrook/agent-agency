//! Core types for disambiguation module

use chrono;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Import port traits from contracts instead of defining locally
pub use agent_agency_contracts::types::research::{
    EmbeddingProvider, KnowledgeBase, KnowledgeIngest, UnresolvableReason,
};

// Note: EntityMatch is kept local here because it has additional 'context' field
// that contracts::EntityMatch doesn't have. Use contracts::EntityMatch when
// the context field is not needed.

/// Entity match result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMatch {
    pub entity: String,
    pub entity_type: EntityType,
    pub confidence: f64,
    pub start_pos: usize,
    pub end_pos: usize,
    pub context: Option<String>,
}

/// Entity types for disambiguation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EntityType {
    Person,
    Organization,
    Location,
    Technology,
    TechnicalTerm,
    Concept,
    Date,
    Money,
    Percent,
    Other(String),
}

/// Ambiguity detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbiguityResult {
    pub text: String,
    pub ambiguities: Vec<Ambiguity>,
    pub confidence: f64,
}

/// Individual ambiguity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ambiguity {
    pub text: String,
    pub ambiguity_type: AmbiguityType,
    pub start_pos: usize,
    pub end_pos: usize,
    pub position: (usize, usize), // (start_pos, end_pos) tuple
    pub original_text: String,    // Alias for text
    pub possible_resolutions: Vec<String>,
    pub confidence: f64,
    pub context: Option<String>,
}

/// Types of ambiguities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AmbiguityType {
    Pronoun,
    TechnicalTerm,
    ScopeBoundary,
    TemporalReference,
    EntityReference,
    Quantifier,
    Other(String),
}

/// Disambiguation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisambiguationResult {
    pub original_text: String,
    pub disambiguated_text: String,
    pub entities: Vec<EntityMatch>,
    pub ambiguities_resolved: Vec<Ambiguity>,
    pub confidence: f64,
}

/// Context resolution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextResolutionResult {
    pub entity: String,
    pub resolved_meaning: String,
    pub confidence: f64,
    pub context_source: ContextSource,
}

/// Source of context resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextSource {
    DomainKnowledge,
    EmbeddingSimilarity,
    KnowledgeBase,
    PatternMatching,
    Other(String),
}

/// Configuration for disambiguation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisambiguationConfig {
    pub confidence_threshold: f64,
    pub enable_entity_recognition: bool,
    pub enable_context_resolution: bool,
    pub max_ambiguities: usize,
    pub domain_context: HashMap<String, String>,
}

impl Default for DisambiguationConfig {
    fn default() -> Self {
        Self {
            confidence_threshold: 0.7,
            enable_entity_recognition: true,
            enable_context_resolution: true,
            max_ambiguities: 100,
            domain_context: HashMap::new(),
        }
    }
}

/// Language enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Language {
    English,
    Spanish,
    French,
    German,
    Chinese,
    Japanese,
    Other(String),
}

// Import UnresolvableReason from contracts (already imported above, removing duplicate)
// UnresolvableReason is available via the pub use above

/// Unresolvable ambiguity record (legacy - different structure from contracts)
/// Use contracts::UnresolvableAmbiguity when possible
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnresolvableAmbiguity {
    pub text: String,
    pub reason: UnresolvableReason,
    pub context: Option<String>,
}

/// Named entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedEntity {
    pub name: String,
    pub entity_type: EntityType,
    pub start_pos: usize,
    pub end_pos: usize,
    pub confidence: f64,
    pub context: Option<String>,
}

/// Knowledge base source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KbSource {
    Internal,
    External,
    DomainSpecific,
    General,
}

/// Knowledge base result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBaseResult {
    pub entity: String,
    pub result: String,
    pub source: KbSource,
    pub confidence: f64,
    pub metadata: HashMap<String, String>,
}

/// Related entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedEntity {
    pub entity: String,
    pub relation: String,
    pub related_to: String,
    pub confidence: f64,
}

/// Historical entity analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalEntityAnalysis {
    pub entity: String,
    pub occurrences: Vec<EntityOccurrence>,
    pub trends: Vec<Trend>,
    pub confidence: f64,
}

/// Entity occurrence in history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityOccurrence {
    pub text: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub context: String,
}

/// Trend in entity usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trend {
    pub metric: String,
    pub value: f64,
    pub direction: TrendDirection,
}

/// Trend direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
}

/// Entity relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRelationship {
    pub entity1: String,
    pub entity2: String,
    pub relationship_type: RelationshipType,
    pub strength: f64,
    pub context: Option<String>,
}

/// Relationship type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipType {
    PartOf,
    RelatedTo,
    Causes,
    Precedes,
    DependsOn,
    Other(String),
}

/// Resolved entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedEntity {
    pub original: String,
    pub resolved: String,
    pub entity_type: EntityType,
    pub confidence: f64,
    pub resolution_method: ResolutionMethod,
}

/// Resolution method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionMethod {
    Context,
    KnowledgeBase,
    Embedding,
    Pattern,
}

/// Context-aware disambiguation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAwareDisambiguation {
    pub entity: String,
    pub context: HashMap<String, String>,
    pub candidates: Vec<ResolvedEntity>,
    pub selected: Option<ResolvedEntity>,
}

/// Domain integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainIntegration {
    pub domain: String,
    pub entities: Vec<String>,
    pub relationships: Vec<EntityRelationship>,
    pub confidence: f64,
}

/// External knowledge entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalKnowledgeEntity {
    pub id: String,
    pub name: String,
    pub source: String,
    pub entity_type: EntityType,
    pub metadata: HashMap<String, String>,
}

/// Ingestion channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IngestionChannel {
    File,
    Api,
    Database,
    Stream,
    Other(String),
}

/// Ingestion candidate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionCandidate {
    pub content: String,
    pub channel: IngestionChannel,
    pub metadata: HashMap<String, String>,
    pub priority: u32,
}

/// Ingestion cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionCacheEntry {
    pub key: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub ttl_seconds: u64,
}

/// Ingestion pipeline statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionPipelineStats {
    pub total_ingested: u64,
    pub successful: u64,
    pub failed: u64,
    pub average_processing_time_ms: f64,
    pub last_ingestion: Option<chrono::DateTime<chrono::Utc>>,
}

/// Referent information for pronoun resolution
/// Uses contracts::EntityType for consistency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferentInfo {
    pub referent: String,
    pub entity_type: agent_agency_contracts::types::research::EntityType,
    pub confidence: f64,
    pub context: Option<String>,
}
