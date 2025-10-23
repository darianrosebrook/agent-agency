//! Core data structures used within verification
//!
//! This module contains all the internal data structures used by the verification engine.
//! Public types are re-exported through the parent module.

use std::collections::HashMap;

/// Coreference resolution data structures
#[derive(Debug, Clone)]
pub struct Entity {
    pub id: String,
    pub text: String,
    pub entity_type: EntityType,
    pub confidence: f64,
    pub position: (usize, usize), // (start, end) character positions
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityType {
    Person,
    Organization,
    Location,
    CodeEntity, // functions, classes, variables
    SystemComponent, // APIs, services, databases
    Concept, // abstract concepts
    Other,
}

#[derive(Debug, Clone)]
pub struct CoreferenceChain {
    pub representative: Entity,
    pub mentions: Vec<Entity>,
    pub confidence: f64,
    pub chain_type: CoreferenceType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreferenceType {
    Identity, // Same entity (he/she/it -> specific entity)
    Appositive, // Descriptive (John, the developer -> John)
    Predicate, // Predicative (he is John -> John)
    Anaphoric, // Backward reference
    Cataphoric, // Forward reference
}

#[derive(Debug, Clone)]
pub struct CoreferenceResolution {
    pub chains: Vec<CoreferenceChain>,
    pub unresolved_pronouns: Vec<String>,
    pub confidence_score: f64,
    pub processing_time_ms: u64,
}

/// Entity disambiguation result
#[derive(Debug, Clone)]
pub struct EntityDisambiguation {
    pub original_entity: Entity,
    pub candidates: Vec<EntityCandidate>,
    pub best_match: Option<EntityCandidate>,
    pub disambiguation_method: DisambiguationMethod,
}

#[derive(Debug, Clone)]
pub struct EntityCandidate {
    pub entity: Entity,
    pub similarity_score: f64,
    pub context_match: bool,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisambiguationMethod {
    ExactMatch,
    FuzzyMatch,
    ContextBased,
    KnowledgeGraph,
    EmbeddingSimilarity,
}

/// Code output structure for claim extraction
#[derive(Debug, Clone)]
pub struct CodeOutput {
    pub content: String,
    pub language: Language,
    pub file_path: Option<String>,
}

/// Code specification for validation
#[derive(Debug, Clone)]
pub struct CodeSpecification {
    pub expected_signatures: HashMap<String, String>,
    pub expected_types: HashMap<String, String>,
    pub implementation_requirements: Vec<String>,
}

/// Code structure analysis results
#[derive(Debug, Clone)]
pub struct CodeStructure {
    pub functions: Vec<FunctionDefinition>,
    pub types: Vec<TypeDefinition>,
    pub implementations: Vec<ImplementationBlock>,
}

/// Function definition in code
#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub name: String,
    pub parameters: Vec<String>,
    pub return_type: Option<String>,
    pub body: String,
}

/// Type definition in code
#[derive(Debug, Clone)]
pub struct TypeDefinition {
    pub name: String,
    pub kind: String, // "struct", "enum", "trait", etc.
    pub fields: Vec<String>,
}

/// Implementation block in code
#[derive(Debug, Clone)]
pub struct ImplementationBlock {
    pub target: String,
    pub methods: Vec<String>,
}

/// Documentation output structure
#[derive(Debug, Clone)]
pub struct DocumentationOutput {
    pub content: String,
    pub format: String,
    pub completeness_score: f64,
}

/// Documentation standards for validation
#[derive(Debug, Clone)]
pub struct DocumentationStandards {
    pub required_sections: Vec<String>,
    pub style_guide: HashMap<String, String>,
    pub example_requirements: Vec<String>,
}

/// Documentation structure analysis
#[derive(Debug, Clone)]
pub struct DocumentationStructure {
    pub sections: Vec<String>,
    pub examples: Vec<UsageExample>,
    pub api_references: Vec<String>,
}

/// API documentation structure
#[derive(Debug, Clone)]
pub struct ApiDocumentation {
    pub endpoints: Vec<String>,
    pub parameters: HashMap<String, Vec<String>>,
    pub responses: HashMap<String, String>,
}

/// Usage example in documentation
#[derive(Debug, Clone)]
pub struct UsageExample {
    pub description: String,
    pub code: String,
    pub language: String,
}

/// Data analysis output for claim validation
#[derive(Debug, Clone)]
pub struct DataAnalysisOutput {
    pub results: Vec<StatisticalResult>,
    pub correlations: Vec<CorrelationResult>,
    pub patterns: Vec<PatternResult>,
    pub raw_text: Option<String>,      // for parser fallbacks
    pub analysis_type: Option<String>, // analysis type identifier
}

/// Data schema for validation
#[derive(Debug, Clone)]
pub struct DataSchema {
    pub fields: HashMap<String, String>,
    pub constraints: Vec<String>,
    pub relationships: Vec<String>,
}

/// Data analysis results container
#[derive(Debug, Clone)]
pub struct DataAnalysisResults {
    pub statistics: Vec<StatisticalResult>,
    pub correlations: Vec<CorrelationResult>,
    pub insights: Vec<String>,
}

/// Statistical result from data analysis
#[derive(Debug, Clone)]
pub struct StatisticalResult {
    pub variable: String,
    pub metric: String, // "mean", "median", "std_dev", etc.
    pub value: f64,
    pub p_value: f64,
}

/// Pattern result from data analysis
#[derive(Debug, Clone)]
pub struct PatternResult {
    pub pattern_type: String,
    pub description: String,
    pub confidence: f64,
}

/// Correlation result from data analysis
#[derive(Debug, Clone)]
pub struct CorrelationResult {
    pub variable1: String,
    pub variable2: String,
    pub correlation_coefficient: f64,
    pub p_value: f64,
}

/// Keyword match result
#[derive(Debug, Clone)]
pub struct KeywordMatch {
    pub keyword: String,
    pub file_path: String,
    pub line_number: usize,
    pub context: String,
    pub match_type: MatchType,
    pub relevance_score: f64,
}

/// Type of keyword match
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchType {
    Exact,
    Fuzzy,
    Context,
    Header,
}

/// Helper for verification check results
#[derive(Debug, Default)]
pub struct CheckResult {
    pub score: f64,
    pub evidence: Vec<String>, // Simple string evidence for now
}

impl CheckResult {
    pub fn new(score: f64) -> Self {
        Self {
            score,
            evidence: vec![],
        }
    }

    pub fn with_evidence(mut self, e: String) -> Self {
        self.evidence.push(e);
        self
    }

    pub fn with_many<I: IntoIterator<Item = String>>(mut self, it: I) -> Self {
        self.evidence.extend(it);
        self
    }
}

// Import shared types to avoid duplication
use crate::Language;
