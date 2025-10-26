//! Common types for evidence collection and validation

use serde::{Deserialize, Serialize};

/// Processing context for evidence collection operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingContext {
    /// Source document or content identifier
    pub source_id: String,
    /// Processing timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Processing configuration
    pub config: ProcessingConfig,
}

/// Processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingConfig {
    /// Maximum claims to extract
    pub max_claims: usize,
    /// Confidence threshold for claim acceptance
    pub confidence_threshold: f64,
    /// Enable fact verification
    pub enable_verification: bool,
    /// Enable source validation
    pub enable_source_validation: bool,
}

/// Named entity extracted from content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// Entity type (PERSON, ORGANIZATION, LOCATION, etc.)
    pub entity_type: String,
    /// Entity text
    pub text: String,
    /// Confidence score (0.0-1.0)
    pub confidence: f64,
    /// Character positions in source
    pub positions: Vec<(usize, usize)>,
}

/// Result of claim extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimExtractionResult {
    /// Extracted claims
    pub claims: Vec<AtomicClaim>,
    /// Extraction metadata
    pub metadata: ExtractionMetadata,
}

/// Metadata about the extraction process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionMetadata {
    /// Total processing time
    pub processing_time_ms: u64,
    /// Number of entities found
    pub entity_count: usize,
    /// Extraction confidence score
    pub confidence_score: f64,
    /// Source content length
    pub content_length: usize,
}

/// Atomic claim that can be verified
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomicClaim {
    /// Unique claim identifier
    pub id: String,
    /// Claim text
    pub text: String,
    /// Claim type
    pub claim_type: ClaimType,
    /// Entities mentioned in claim
    pub entities: Vec<Entity>,
    /// Confidence score
    pub confidence: f64,
    /// Source positions
    pub positions: Vec<(usize, usize)>,
    /// Supporting evidence
    pub evidence: Vec<String>,
}

/// Types of claims that can be extracted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClaimType {
    /// Factual claim (verifiable truth)
    Factual,
    /// Opinion claim (subjective statement)
    Opinion,
    /// Prediction claim (future-oriented)
    Prediction,
    /// Attribution claim (who said what)
    Attribution,
    /// Causal claim (cause-effect relationship)
    Causal,
    /// Definitional claim (what something is)
    Definitional,
}

/// Verification result for a claim
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Claim being verified
    pub claim: AtomicClaim,
    /// Verification status
    pub status: VerificationStatus,
    /// Confidence in verification
    pub confidence: f64,
    /// Supporting evidence
    pub evidence: Vec<String>,
    /// Counter-evidence if any
    pub counter_evidence: Vec<String>,
}

/// Status of claim verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationStatus {
    /// Claim verified as true
    Verified,
    /// Claim verified as false
    Refuted,
    /// Insufficient evidence to verify
    Unverifiable,
    /// Claim is subjective/opinion-based
    Subjective,
}

/// Source credibility assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceCredibility {
    /// Overall credibility score (0.0-1.0)
    pub overall_score: f64,
    /// Authority score
    pub authority_score: f64,
    /// Reliability score
    pub reliability_score: f64,
    /// Bias assessment
    pub bias_score: f64,
    /// Recency score
    pub recency_score: f64,
    /// Supporting factors
    pub supporting_factors: Vec<String>,
    /// Detracting factors
    pub detracting_factors: Vec<String>,
}

/// Evidence collection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceResult {
    /// Collection of verified claims
    pub claims: Vec<AtomicClaim>,
    /// Verification results
    pub verifications: Vec<VerificationResult>,
    /// Source credibility assessments
    pub source_credibility: SourceCredibility,
    /// Overall confidence score
    pub overall_confidence: f64,
    /// Processing metadata
    pub metadata: EvidenceMetadata,
}

/// Metadata about evidence collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceMetadata {
    /// Processing start time
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// Total processing time
    pub processing_time_ms: u64,
    /// Number of sources processed
    pub sources_processed: usize,
    /// Claims extracted
    pub claims_extracted: usize,
    /// Claims verified
    pub claims_verified: usize,
}
