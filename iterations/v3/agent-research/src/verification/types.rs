//! Core types for verification module

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Coreference resolution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreferenceResolution {
    pub text: String,
    pub resolved_text: String,
    pub coreferences: Vec<Coreference>,
    pub confidence: f64,
}

/// Individual coreference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coreference {
    pub mention: String,
    pub entity: String,
    pub start_pos: usize,
    pub end_pos: usize,
    pub confidence: f64,
}

/// Check result for verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub check_type: CheckType,
    pub passed: bool,
    pub confidence: f64,
    pub details: String,
    pub evidence: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

/// Types of checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckType {
    CrossReference,
    Authority,
    Semantic,
    Keyword,
    Code,
    Documentation,
    Data,
    Historical,
    Other(String),
}

/// Verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub claim_id: String,
    pub verified: bool,
    pub confidence: f64,
    pub checks: Vec<CheckResult>,
    pub overall_assessment: Assessment,
    pub timestamp: DateTime<Utc>,
}

/// Overall assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Assessment {
    Verified,
    PartiallyVerified,
    Unverified,
    Contradicted,
    InsufficientEvidence,
}

/// Authority validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityValidationResult {
    pub source: String,
    pub authority_score: f64,
    pub credibility_factors: Vec<CredibilityFactor>,
    pub validation_status: ValidationStatus,
}

/// Credibility factors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredibilityFactor {
    pub factor_type: CredibilityType,
    pub score: f64,
    pub description: String,
}

/// Types of credibility factors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CredibilityType {
    Expertise,
    Reputation,
    PeerReview,
    Documentation,
    HistoricalAccuracy,
    Other(String),
}

/// Validation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationStatus {
    Valid,
    Invalid,
    Uncertain,
    InsufficientData,
}

/// Semantic analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticAnalysisResult {
    pub intent: String,
    pub entities: Vec<String>,
    pub relationships: Vec<Relationship>,
    pub confidence: f64,
}

/// Relationship between entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence: f64,
}

/// Keyword matching result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordMatchResult {
    pub matches: Vec<KeywordMatch>,
    pub total_matches: usize,
    pub confidence: f64,
}

/// Individual keyword match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordMatch {
    pub keyword: String,
    pub context: String,
    pub start_pos: usize,
    pub end_pos: usize,
    pub confidence: f64,
}

/// Historical lookup result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalLookupResult {
    pub query: String,
    pub results: Vec<HistoricalResult>,
    pub total_results: usize,
    pub confidence: f64,
}

/// Individual historical result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalResult {
    pub claim: String,
    pub source: String,
    pub timestamp: DateTime<Utc>,
    pub confidence: f64,
    pub relevance: f64,
}

/// Verification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationConfig {
    pub enable_cross_reference: bool,
    pub enable_authority_check: bool,
    pub enable_semantic_analysis: bool,
    pub enable_keyword_matching: bool,
    pub enable_code_analysis: bool,
    pub enable_documentation: bool,
    pub enable_data_analysis: bool,
    pub enable_historical_lookup: bool,
    pub confidence_threshold: f64,
    pub max_results: usize,
    pub timeout_seconds: u64,
}

impl Default for VerificationConfig {
    fn default() -> Self {
        Self {
            enable_cross_reference: true,
            enable_authority_check: true,
            enable_semantic_analysis: true,
            enable_keyword_matching: true,
            enable_code_analysis: true,
            enable_documentation: true,
            enable_data_analysis: true,
            enable_historical_lookup: true,
            confidence_threshold: 0.7,
            max_results: 100,
            timeout_seconds: 300,
        }
    }
}

/// Entity in verification context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub entity_type: EntityType,
    pub confidence: f64,
    pub context: Option<String>,
}

/// Coreference chain - sequence of coreferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreferenceChain {
    pub chain_id: String,
    pub coreferences: Vec<Coreference>,
    pub entity: String,
    pub confidence: f64,
}

/// Coreference type classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoreferenceType {
    Pronominal,
    Nominal,
    Demonstrative,
    Possessive,
    Other(String),
}

/// Entity disambiguation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityDisambiguation {
    pub entity: String,
    pub candidates: Vec<EntityCandidate>,
    pub selected_candidate: Option<EntityCandidate>,
    pub confidence: f64,
    pub method: DisambiguationMethod,
}

/// Entity candidate for disambiguation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityCandidate {
    pub id: String,
    pub name: String,
    pub entity_type: EntityType,
    pub confidence: f64,
    pub context: Option<String>,
}

/// Disambiguation method used
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisambiguationMethod {
    Contextual,
    KnowledgeBase,
    Embedding,
    PatternMatching,
    Hybrid,
}

/// Code output from analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeOutput {
    pub code: String,
    pub language: String,
    pub functions: Vec<String>,
    pub classes: Vec<String>,
    pub imports: Vec<String>,
}

/// Code specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSpecification {
    pub language: String,
    pub api_signature: Option<String>,
    pub return_type: Option<String>,
    pub parameters: Vec<String>,
}

/// Documentation output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationOutput {
    pub content: String,
    pub format: String,
    pub sections: Vec<String>,
    pub examples: Vec<String>,
}

/// Documentation standards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationStandards {
    pub format: String,
    pub required_sections: Vec<String>,
    pub style_guide: Option<String>,
}

/// Data analysis output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataAnalysisOutput {
    pub analysis_type: String,
    pub results: DataAnalysisResults,
    pub confidence: f64,
}

/// Data analysis results container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataAnalysisResults {
    pub statistical: Vec<StatisticalResult>,
    pub patterns: Vec<PatternResult>,
    pub correlations: Vec<CorrelationResult>,
    pub insights: Vec<String>,
}

/// Data schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSchema {
    pub fields: Vec<SchemaField>,
    pub constraints: Vec<String>,
}

/// Schema field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaField {
    pub name: String,
    pub field_type: String,
    pub required: bool,
}

/// Statistical result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalResult {
    pub metric: String,
    pub value: f64,
    pub confidence: f64,
    pub context: Option<String>,
}

/// Pattern result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternResult {
    pub pattern_type: String,
    pub matches: Vec<String>,
    pub confidence: f64,
}

/// Correlation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationResult {
    pub variable1: String,
    pub variable2: String,
    pub correlation: f64,
    pub significance: f64,
}

/// Match type for keyword matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchType {
    Exact,
    Fuzzy,
    Semantic,
    Regex,
}

/// Entity type for verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityType {
    Person,
    Organization,
    Location,
    Technology,
    Concept,
    Code,
    Documentation,
    Data,
    Other(String),
}
