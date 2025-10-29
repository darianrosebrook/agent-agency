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
