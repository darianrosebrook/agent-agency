//! Shared types and traits for disambiguation module

use schemars::JsonSchema;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use uuid::Uuid;

/// Programming languages supported by the system

use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize) ]
pub enum Language {
    Rust,
    TypeScript,
    Python,
    JavaScript,
    English, // For natural language processing
}

/// Result of disambiguation process

#[derive(Debug, Clone, Serialize, Deserialize) ]
pub struct DisambiguationResult {
    pub original_sentence: String,
    pub disambiguated_sentence: String,
    pub ambiguities_resolved: u32,
    pub unresolvable_ambiguities: Vec<agent_agency_contracts::types::research::UnresolvableAmbiguity>,
}

/// Represents an ambiguity found in text

#[derive(Debug, Clone, Serialize, Deserialize) ]
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


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize) ]
pub enum AmbiguityType {
    Pronoun,
    TechnicalTerm,
    ScopeBoundary,
    TemporalReference,
    Quantifier,
}

/// Ambiguity that cannot be resolved with available context

#[derive(Debug, Clone, Serialize, Deserialize) ]
pub struct UnresolvableAmbiguity {
    pub ambiguity: Ambiguity,
    pub reason: agent_agency_contracts::types::research::UnresolvableReason,
    pub suggested_context: Vec<String>,
}

// Note: Traits and UnresolvableReason are imported via types.rs module, not here
// to avoid duplicate definitions

/// Information about a pronoun referent
/// Uses contracts::EntityType for consistency
#[derive(Debug, Clone, Serialize, Deserialize) ]
pub struct ReferentInfo {
    pub entity: String,
    pub confidence: f64,
    pub source: String,
    pub entity_type: agent_agency_contracts::types::research::EntityType,
}

/// Entity type classification

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize) ]
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
/// Uses contracts::EntityType for consistency
#[derive(Debug, Clone, Serialize, Deserialize) ]
pub struct NamedEntity {
    pub text: String,
    pub entity_type: agent_agency_contracts::types::research::EntityType,
    pub start: usize,
    pub end: usize,
    pub confidence: f64,
    pub context: Option<String>,
}

/// Entity match result for caching (domain-specific, different from contracts::EntityMatch)
/// This type is used internally in disambiguation and has different fields.
/// Use contracts::EntityMatch when you need the standard entity match structure.
#[derive(Debug, Clone, Serialize, Deserialize) ]
pub struct EntityMatch {
    pub entity: NamedEntity,
    pub confidence: f64,
    pub match_type: String,
    pub source: String,
}

/// Knowledge base search result

#[derive(Debug, Clone, Serialize, Deserialize) ]
pub struct KnowledgeBaseResult {
    pub id: Uuid,
    pub canonical_name: String,
    pub source: KbSource, // Renamed to avoid collision
    pub properties: HashMap<String, String>,
}

/// Knowledge source types (renamed from KnowledgeSource to avoid collision)

#[derive(Debug, Clone, Serialize, Deserialize) ]
pub enum KbSource {
    Wikidata,
    WordNet,
    Custom,
}

/// Related entity information

#[derive(Debug, Clone, Serialize, Deserialize) ]
pub struct RelatedEntity {
    pub id: Uuid,
    pub canonical_name: String,
    pub relationship_type: String,
    pub confidence: f64,
}

/// Analysis helpers

#[derive(Debug, Clone, Serialize, Deserialize) ]
pub struct HistoricalEntityAnalysis {
    pub entity_id: Uuid,
    pub historical_context: String,
    pub temporal_relevance: f64,
}


#[derive(Debug, Clone, Serialize, Deserialize) ]
pub struct EntityRelationship {
    pub source_entity: String,
    pub target_entity: String,
    pub relationship: String,
    pub confidence: f64,
}


#[derive(Debug, Clone, Serialize, Deserialize) ]
pub struct ResolvedEntity {
    pub text: String,
    pub canonical_form: String,
    pub source: String,
    pub confidence: f64,
}


#[derive(Debug, Clone, Serialize, Deserialize) ]
pub struct ContextAwareDisambiguation {
    pub original_text: String,
    pub resolved_text: String,
    pub context_used: Vec<String>,
    pub confidence: f64,
}


#[derive(Debug, Clone, Serialize, Deserialize) ]
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

// Note: Port traits (EmbeddingProvider, KnowledgeBase, KnowledgeIngest) are
// imported via types.rs module to avoid duplicate definitions.
// Domain-specific traits like ClaimExtractionEmbeddingProvider can coexist.

/// External knowledge entity

/// Supported ingestion channels for on-demand knowledge acquisition

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Copy)]
pub enum IngestionChannel {
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

#[derive(Debug, Clone, Serialize, Deserialize) ]
pub struct IngestionCandidate {
    pub channel: IngestionChannel,
    pub label: String,
    pub priority: i32,
    pub estimated_cost: f64,
}

/// Scheduled ingestion source

#[derive(Debug, Clone, Serialize, Deserialize) ]
pub struct ScheduledSource {
    pub channel: IngestionChannel,
    pub schedule: String,
    pub last_run: Option<chrono::DateTime<chrono::Utc>>,
}

/// Ingestion cache entry

#[derive(Debug, Clone, Serialize, Deserialize) ]
pub struct IngestionCacheEntry {
    pub key: String,
    pub data: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub ttl: std::time::Duration,
}

/// Pipeline statistics for ingestion

#[derive(Debug, Clone, Serialize, Deserialize) ]
pub struct IngestionPipelineStats {
    pub total_candidates: usize,
    pub processed: usize,
    pub failed: usize,
    pub avg_processing_time: std::time::Duration,
}
