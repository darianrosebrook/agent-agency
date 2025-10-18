//! Types for claim extraction and verification pipeline

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Context for claim processing operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingContext {
    pub task_id: Uuid,
    pub working_spec_id: String,
    pub source_file: Option<String>,
    pub line_number: Option<u32>,
    pub surrounding_context: String,
    pub domain_hints: Vec<String>,
}

/// Result of claim extraction process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimExtractionResult {
    pub original_sentence: String,
    pub disambiguated_sentence: String,
    pub atomic_claims: Vec<AtomicClaim>,
    pub verification_evidence: Vec<Evidence>,
    pub processing_metadata: ProcessingMetadata,
}

/// Analysis of ambiguities in a sentence (ported from V2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisambiguationAnalysis {
    pub referential_ambiguities: Vec<String>,
    pub structural_ambiguities: Vec<String>,
    pub temporal_ambiguities: Vec<String>,
    pub can_resolve: bool,
    pub resolution_confidence: f64,
}

/// Factors used to compute disambiguation confidence
#[derive(Debug, Clone)]
pub struct DisambiguationConfidenceFactors {
    pub referential_ambiguities: usize,
    pub structural_ambiguities: usize,
    pub temporal_ambiguities: usize,
    pub referential_resolvable: bool,
    pub temporal_resolvable: bool,
    pub structural_resolvable: bool,
}

/// Information about a pronoun referent
#[derive(Debug, Clone)]
pub struct ReferentInfo {
    pub entity: String,
    pub confidence: f64,
    pub source: String,
}

/// Individual atomic claim extracted from sentence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomicClaim {
    pub id: Uuid,
    pub claim_text: String,
    pub claim_type: ClaimType,
    pub verifiability: VerifiabilityLevel,
    pub scope: ClaimScope,
    pub confidence: f64,
    pub contextual_brackets: Vec<String>,
}

/// Types of claims that can be extracted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClaimType {
    Factual,
    Procedural,
    Technical,
    Constitutional, // CAWS-related
    Performance,
    Security,
}

/// Level of verifiability for a claim
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerifiabilityLevel {
    DirectlyVerifiable,
    IndirectlyVerifiable,
    Unverifiable,
    RequiresContext,
}

/// Scope of a claim within the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimScope {
    pub working_spec_id: String,
    pub component_boundaries: Vec<String>,
    pub data_impact: DataImpact,
}

/// Impact of claim on data/system state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataImpact {
    None,
    ReadOnly,
    Write,
    Critical, // Affects core system state
}

/// Evidence supporting or refuting a claim
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub id: Uuid,
    pub claim_id: Uuid,
    pub evidence_type: EvidenceType,
    pub content: String,
    pub source: EvidenceSource,
    pub confidence: f64,
    pub timestamp: DateTime<Utc>,
}

/// Types of evidence that can be collected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceType {
    CodeAnalysis,
    TestResults,
    Documentation,
    ResearchFindings,
    PerformanceMetrics,
    SecurityScan,
    ConstitutionalReference, // CAWS compliance
    CouncilDecision,         // Council verification results
}

/// Source of evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceSource {
    pub source_type: SourceType,
    pub location: String,
    pub authority: String,
    pub freshness: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceType {
    FileSystem,
    Database,
    Web,
    ResearchAgent,
    CouncilDecision,
    TestSuite,
    ToolOutput,
    Analysis,
}

/// Metadata about the processing operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingMetadata {
    pub processing_time_ms: u64,
    pub stages_completed: Vec<ProcessingStage>,
    pub ambiguities_resolved: u32,
    pub claims_extracted: u32,
    pub evidence_collected: u32,
    pub rewrite_suggestions: u32,
    pub errors: Vec<ProcessingError>,
}

impl Default for ProcessingMetadata {
    fn default() -> Self {
        Self {
            processing_time_ms: 0,
            stages_completed: Vec::new(),
            ambiguities_resolved: 0,
            claims_extracted: 0,
            evidence_collected: 0,
            rewrite_suggestions: 0,
            errors: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessingStage {
    Disambiguation,
    Qualification,
    Decomposition,
    Verification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingError {
    pub stage: ProcessingStage,
    pub error_type: String,
    pub message: String,
    pub recoverable: bool,
}

/// Result of disambiguation stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisambiguationResult {
    pub original_sentence: String,
    pub disambiguated_sentence: String,
    pub ambiguities_resolved: u32,
    pub unresolvable_ambiguities: Vec<UnresolvableAmbiguity>,
}

/// Result of qualification stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualificationResult {
    pub verifiable_parts: Vec<VerifiableContent>,
    pub unverifiable_parts: Vec<UnverifiableContent>,
    pub overall_verifiability: VerifiabilityLevel,
}

/// Result of decomposition stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecompositionResult {
    pub atomic_claims: Vec<AtomicClaim>,
    pub decomposition_confidence: f64,
}

/// Result of verification stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub evidence: Vec<Evidence>,
    pub verification_confidence: f64,
}

/// Errors that can occur during claim extraction
#[derive(Debug, thiserror::Error)]
pub enum ClaimExtractionError {
    #[error("Disambiguation failed: {0}")]
    DisambiguationFailed(String),

    #[error("Qualification failed: {0}")]
    QualificationFailed(String),

    #[error("Decomposition failed: {0}")]
    DecompositionFailed(String),

    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    #[error("Evidence collection failed: {0}")]
    EvidenceCollectionFailed(String),

    #[error("Council integration failed: {0}")]
    CouncilIntegrationFailed(String),
}

/// Represents an ambiguity found in text
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Ambiguity {
    pub ambiguity_type: AmbiguityType,
    pub position: (usize, usize), // Start and end character positions
    pub original_text: String,
    pub possible_resolutions: Vec<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum AmbiguityType {
    Pronoun,
    TechnicalTerm,
    ScopeBoundary,
    TemporalReference,
    Quantifier,
}

/// Ambiguity that cannot be resolved with available context
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UnresolvableAmbiguity {
    pub ambiguity: Ambiguity,
    pub reason: UnresolvableReason,
    pub suggested_context: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum UnresolvableReason {
    InsufficientContext,
    MultipleValidInterpretations,
    DomainSpecificUnknown,
    TemporalUncertainty,
}

/// Content that can be verified
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VerifiableContent {
    pub position: (usize, usize),
    pub content: String,
    pub verification_method: VerificationMethod,
    pub evidence_requirements: Vec<EvidenceRequirement>,
}

/// Content that cannot be verified
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UnverifiableContent {
    pub position: (usize, usize),
    pub content: String,
    pub reason: UnverifiableReason,
    pub suggested_rewrite: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum VerificationMethod {
    CodeAnalysis,
    TestExecution,
    DocumentationReview,
    PerformanceMeasurement,
    SecurityScan,
    ConstitutionalCheck, // CAWS compliance
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EvidenceRequirement {
    pub evidence_type: EvidenceType,
    pub minimum_confidence: f64,
    pub source_requirements: Vec<SourceRequirement>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SourceRequirement {
    pub source_type: SourceType,
    pub authority_level: AuthorityLevel,
    pub freshness_requirement: Option<chrono::Duration>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AuthorityLevel {
    Primary,   // Direct source
    Secondary, // Referenced source
    Tertiary,  // Background context
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum UnverifiableReason {
    SubjectiveLanguage,
    VagueCriteria,
    MissingContext,
    OpinionBased,
    FuturePrediction,
    EmotionalContent,
}

/// Assessment of content verifiability
#[derive(Debug, Clone)]
pub struct VerifiabilityAssessment {
    pub overall_verifiability: VerifiabilityLevel,
    pub verifiable_parts: Vec<VerifiableContent>,
    pub unverifiable_parts: Vec<UnverifiableContent>,
    pub confidence: f64,
}

/// Results of multi-modal verification
#[derive(Debug, Clone, Default)]
pub struct VerificationResults {
    pub verified_claims: Vec<VerifiedClaim>,
    pub total_processed: usize,
    pub successful_verifications: usize,
}

/// Individual verified claim
#[derive(Debug, Clone)]
pub struct VerifiedClaim {
    pub original_claim: String,
    pub verification_results: VerificationStatus,
    pub overall_confidence: f64,
    pub verification_timestamp: DateTime<Utc>,
}

/// Status of verification process
#[derive(Debug, Clone)]
pub enum VerificationStatus {
    Verified,
    Refuted,
    Pending,
    Error(String),
}

/// Council environment settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CouncilEnvironment {
    Development,
    Staging,
    Production,
}
