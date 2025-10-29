//! Core types for disambiguation module

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Trait for embedding providers
pub trait EmbeddingProvider: Send + Sync {
    fn embed(&self, text: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>>;
}

/// Trait for knowledge base operations
pub trait KnowledgeBase: Send + Sync {
    fn lookup(&self, entity: &str) -> Result<Option<String>, Box<dyn std::error::Error>>;
    fn search(&self, query: &str) -> Result<Vec<String>, Box<dyn std::error::Error>>;
}

/// Trait for knowledge ingestion
pub trait KnowledgeIngest: Send + Sync {
    fn ingest(&self, content: &str) -> Result<(), Box<dyn std::error::Error>>;
}

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityType {
    Person,
    Organization,
    Location,
    Technology,
    Concept,
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
    pub confidence: f64,
    pub context: Option<String>,
}

/// Types of ambiguities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AmbiguityType {
    Pronoun,
    TechnicalTerm,
    ScopeBoundary,
    TemporalReference,
    EntityReference,
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
