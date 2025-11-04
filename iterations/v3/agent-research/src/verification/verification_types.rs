//! Core data structures used within verification
//!
//! This module contains all the internal data structures used by the verification engine.
//! Public types are re-exported through the parent module.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::extraction_types::Language;
use crate::verification::types::DisambiguationMethod;

/// Coreference resolution data structures

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub text: String,  // Alias for name for compatibility
    pub entity_type: EntityType,
    pub confidence: f64,
    pub context: Option<String>,
    pub position: Option<(usize, usize)>,  // (start, end) position
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum EntityType {
    Person,
    Organization,
    Location,
    Technology,
    Concept,
    Code,
    CodeEntity, // Alias for Code - used in disambiguation
    SystemComponent, // System components like APIs, services
    Documentation,
    Data,
    Other(String),
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CoreferenceChain {
    pub representative: Entity,
    pub mentions: Vec<Entity>,
    pub confidence: f64,
    pub chain_type: CoreferenceType,
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum CoreferenceType {
    Identity, // Same entity (he/she/it -> specific entity)
    Appositive, // Descriptive (John, the developer -> John)
    Predicate, // Predicative (he is John -> John)
    Anaphoric, // Backward reference
    Cataphoric, // Forward reference
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CoreferenceResolution {
    pub chains: Vec<CoreferenceChain>,
    pub unresolved_pronouns: Vec<String>,
    pub confidence_score: f64,
    pub processing_time_ms: u64,
}

/// Entity disambiguation result

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntityDisambiguation {
    pub original_entity: Entity,
    pub candidates: Vec<EntityCandidate>,
    pub best_match: Option<EntityCandidate>,
    pub disambiguation_method: DisambiguationMethod,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntityCandidate {
    pub entity: Entity,
    pub similarity_score: f64,
    pub context_match: bool,
    pub source: String,
}


// DisambiguationMethod is imported from types module

/// Code output structure for claim extraction

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CodeOutput {
    pub content: String,
    pub language: Language,
    pub file_path: Option<String>,
}

/// Code specification for validation

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CodeSpecification {
    pub expected_signatures: HashMap<String, String>,
    pub expected_types: HashMap<String, String>,
    pub implementation_requirements: Vec<String>,
}

/// Code structure analysis results

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CodeStructure {
    pub functions: Vec<FunctionDefinition>,
    pub types: Vec<TypeDefinition>,
    pub implementations: Vec<ImplementationBlock>,
}

/// Function definition in code

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FunctionDefinition {
    pub name: String,
    pub parameters: Vec<String>,
    pub return_type: Option<String>,
    pub body: String,
}

/// Type definition in code

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TypeDefinition {
    pub name: String,
    pub kind: String, // "struct", "enum", "trait", etc.
    pub fields: Vec<String>,
}

/// Implementation block in code

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ImplementationBlock {
    pub target: String,
    pub methods: Vec<String>,
}

/// Documentation output structure

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DocumentationOutput {
    pub content: String,
    pub format: String,
    pub completeness_score: f64,
}

/// Documentation standards for validation

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DocumentationStandards {
    pub required_sections: Vec<String>,
    pub style_guide: HashMap<String, String>,
    pub example_requirements: Vec<String>,
}

/// Documentation structure analysis

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DocumentationStructure {
    pub sections: Vec<String>,
    pub examples: Vec<UsageExample>,
    pub api_references: Vec<String>,
}

/// API documentation structure

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ApiDocumentation {
    pub endpoints: Vec<String>,
    pub parameters: HashMap<String, Vec<String>>,
    pub responses: HashMap<String, String>,
}

/// Usage example in documentation

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UsageExample {
    pub description: String,
    pub code: String,
    pub language: String,
}

/// Data analysis output for claim validation

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DataAnalysisOutput {
    pub results: Vec<StatisticalResult>,
    pub correlations: Vec<CorrelationResult>,
    pub patterns: Vec<PatternResult>,
    pub raw_text: Option<String>,      // for parser fallbacks
    pub analysis_type: Option<String>, // analysis type identifier
}

/// Data schema for validation

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DataSchema {
    pub fields: HashMap<String, String>,
    pub constraints: Vec<String>,
    pub relationships: Vec<String>,
}

/// Data analysis results container

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DataAnalysisResults {
    pub statistics: Vec<StatisticalResult>,
    pub correlations: Vec<CorrelationResult>,
    pub insights: Vec<String>,
}

/// Statistical result from data analysis

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StatisticalResult {
    pub variable: String,
    pub metric: String, // "mean", "median", "std_dev", etc.
    pub value: f64,
    pub p_value: f64,
}

/// Pattern result from data analysis

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PatternResult {
    pub pattern_type: String,
    pub description: String,
    pub confidence: f64,
}

/// Correlation result from data analysis

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CorrelationResult {
    pub variable1: String,
    pub variable2: String,
    pub correlation_coefficient: f64,
    pub p_value: f64,
}

/// Keyword match result

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KeywordMatch {
    pub keyword: String,
    pub file_path: String,
    pub line_number: usize,
    pub context: String,
    pub match_type: MatchType,
    pub relevance_score: f64,
}

/// Type of keyword match

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
    pub enum MatchType {
    Exact,
    Fuzzy,
    Context,
    Header,
}

/// Test output structure for code verification

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TestOutput {
    pub test_results: Vec<TestResult>,
    pub coverage: f64,
    pub passed: usize,
    pub failed: usize,
    pub total: usize,
}

/// Individual test result

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub duration_ms: u64,
    pub output: String,
}

/// Test consistency analysis

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TestConsistency {
    pub overall_score: f64,
    pub consistency_issues: Vec<String>,
}

/// Test coverage analysis

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TestCoverage {
    pub overall_score: f64,
    pub line_coverage: f64,
    pub branch_coverage: f64,
    pub function_coverage: f64,
}

/// Test relevance analysis

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TestRelevance {
    pub overall_score: f64,
    pub relevance_factors: Vec<String>,
}

/// Test quality analysis

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TestQuality {
    pub overall_score: f64,
    pub quality_metrics: Vec<String>,
}

/// Helper for verification check results

#[derive(Debug, Serialize, Deserialize, JsonSchema, Default)]
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

// Language is already imported from extraction_types above
