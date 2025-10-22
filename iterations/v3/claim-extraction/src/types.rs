//! Types for claim extraction and verification pipeline

use anyhow::Result;
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
    pub metadata: std::collections::HashMap<String, serde_json::Value>, // Additional metadata
    pub input_text: String, // Input text being processed
    pub language: Option<Language>,
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
    pub subject: Option<String>,
    pub predicate: Option<String>,
    pub object: Option<String>,
    pub context_brackets: Vec<String>, // Alias for contextual_brackets
    pub verification_requirements: Vec<VerificationRequirement>,
    pub position: (usize, usize),
    pub sentence_fragment: String,
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
    Behavioral,
    Functional,
    Structural,
    Informational,
}

/// Level of verifiability for a claim
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerifiabilityLevel {
    DirectlyVerifiable,
    IndirectlyVerifiable,
    Unverifiable,
    High,
    Medium,
    Low,
    RequiresContext,
    HighlyVerifiable,
    ModeratelyVerifiable,
    LowVerifiability,
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
    pub relevance: f64,
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
    MultiModalAnalysis,      // Multi-modal verification results
    ExternalSource,          // External API sources
    TestResult,              // Individual test result
    UserFeedback,            // User-provided feedback
    Measurement,
    LogicalAnalysis,
}

/// Source of evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum EvidenceSource {
    #[serde(rename = "code_search")]
    CodeSearch {
        location: String,
        authority: String,
        freshness: DateTime<Utc>
    },
    #[serde(rename = "code_analysis")]
    CodeAnalysis {
        location: String,
        authority: String,
        freshness: DateTime<Utc>
    },
    #[serde(rename = "documentation")]
    Documentation {
        location: String,
        authority: String,
        freshness: DateTime<Utc>
    },
    #[serde(rename = "measurement")]
    Measurement {
        location: String,
        authority: String,
        freshness: DateTime<Utc>
    },
    #[serde(rename = "logical_reasoning")]
    LogicalReasoning {
        location: String,
        authority: String,
        freshness: DateTime<Utc>
    },
    #[serde(rename = "general")]
    General {
        location: String,
        authority: String,
        freshness: DateTime<Utc>
    },
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
    External,      // External API sources
    Documentation, // Documentation sources
    Measurement,
    LogicalAnalysis,
    General,
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
    pub unverifiable_breakdown: UnverifiableBreakdown,
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
            unverifiable_breakdown: UnverifiableBreakdown::default(),
            errors: Vec::new(),
        }
    }
}

/// Breakdown of unverifiable content reasons encountered during qualification
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UnverifiableBreakdown {
    pub subjective_language: u32,
    pub vague_criteria: u32,
    pub missing_context: u32,
    pub opinion_based: u32,
    pub future_prediction: u32,
    pub emotional_content: u32,
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
    pub verified_claims: Vec<VerifiedClaim>,
    pub council_verification: CouncilVerificationResult,
    pub overall_confidence: f64,
}

/// Result from council verification process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilVerificationResult {
    pub submitted_claims: Vec<Uuid>,
    pub council_verdict: String,
    pub additional_evidence: Vec<Evidence>,
    pub verification_timestamp: DateTime<Utc>,
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
    pub original_content: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum VerificationMethod {
    CodeAnalysis,
    TestExecution,
    DocumentationReview,
    PerformanceMeasurement,
    SecurityScan,
    ConstitutionalCheck, // CAWS compliance
    Measurement,
    LogicalAnalysis,
    ProcessAnalysis,
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum UnverifiableReason {
    SubjectiveLanguage,
    VagueCriteria,
    MissingContext,
    OpinionBased,
    FuturePrediction,
    EmotionalContent,
    ImprovementClaim, // V2 enhancement: claims of improvement without baseline metrics
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedClaim {
    pub original_claim: String,
    pub verification_results: VerificationStatus,
    pub overall_confidence: f64,
    pub verification_timestamp: DateTime<Utc>,
    pub id: Uuid,
    pub claim_text: String,
    pub verification_status: VerificationStatus,
    pub confidence: f64,
    pub evidence: Vec<Evidence>,
    pub timestamp: DateTime<Utc>,
}

/// Status of verification process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationStatus {
    Verified,
    Refuted,
    Pending,
    Unverified,
    Error(String),
}

/// Council environment settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CouncilEnvironment {
    Development,
    Staging,
    Production,
}

/// Programming languages supported for verification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    Rust,
    TypeScript,
    Python,
    JavaScript,
    Go,
    Java,
}

/// Historical claim data structure
#[derive(Debug, Clone)]
pub struct HistoricalClaim {
    pub id: String,
    pub claim_text: String,
    pub verification_status: VerificationStatus,
    pub evidence: Vec<Evidence>,
    pub confidence_score: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub source_count: Option<i32>,
    pub last_verified: Option<chrono::DateTime<chrono::Utc>>,
    pub related_entities: Option<Vec<String>>,
    pub claim_type: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub source_references: Option<Vec<String>>,
    pub cross_references: Option<Vec<String>>,
    pub validation_metadata: Option<serde_json::Value>,
    pub validation_confidence: f64,
    pub validation_timestamp: chrono::DateTime<chrono::Utc>,
    pub validation_outcome: ValidationOutcome,
}

/// Keyword match result
#[derive(Debug, Clone)]
pub struct KeywordMatch {
    pub keyword: String,
    pub position: usize,
    pub context: String,
    pub relevance_score: f64,
    pub match_type: MatchType,
    pub confidence: f64,
}

/// Type of keyword match
#[derive(Debug, Clone)]
pub enum MatchType {
    Exact,
    Fuzzy,
    Context,
}

/// Named entity with type information
#[derive(Debug, Clone)]
pub struct NamedEntity {
    pub text: String,
    pub entity_type: EntityType,
    pub start_position: usize,
    pub end_position: usize,
    pub confidence: f64,
}

/// Entity type classification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityType {
    Person,
    Organization,
    Location,
    Date,
    TechnicalTerm,
    Percent,
    Money,
}

/// Validation outcome for historical claims
#[derive(Debug, Clone)]
pub enum ValidationOutcome {
    Validated,
    Invalidated,
    Uncertain,
}

/// Entity in text with disambiguation information
#[derive(Debug, Clone)]
pub struct Entity {
    pub text: String,
    pub entity_type: EntityType,
    pub start_position: usize,
    pub end_position: usize,
    pub confidence: f64,
    pub disambiguation_candidates: Vec<EntityCandidate>,
}

/// Entity candidate for disambiguation
#[derive(Debug, Clone)]
pub struct EntityCandidate {
    pub text: String,
    pub confidence: f64,
    pub source: String,
    pub context: String,
}

/// Coreference resolution result
#[derive(Debug, Clone)]
pub struct CoreferenceResolution {
    pub resolved_entities: Vec<NamedEntity>,
    pub coreference_chains: Vec<Vec<usize>>, // Indices into resolved_entities
    pub confidence_score: f64,
}

/// Historical entity analysis for pattern detection
#[derive(Debug, Clone)]
pub struct HistoricalEntityAnalysis {
    pub total_entities: usize,
    pub entity_frequency: std::collections::HashMap<String, usize>,
    pub entity_relationships: Vec<EntityRelationship>,
    pub entity_evolution: Vec<String>,
}

/// Entity relationship information
#[derive(Debug, Clone)]
pub struct EntityRelationship {
    pub entity1: String,
    pub entity2: String,
    pub relationship_type: String,
    pub confidence: f64,
    pub evidence: Vec<String>,
}

/// Context-aware disambiguation result
#[derive(Debug, Clone)]
pub struct ContextAwareDisambiguation {
    pub resolved_entities: Vec<ResolvedEntity>,
    pub disambiguation_confidence: f64,
    pub context_sources: Vec<String>,
}

/// Resolved entity with disambiguation
#[derive(Debug, Clone)]
pub struct ResolvedEntity {
    pub original_text: String,
    pub resolved_entity: Entity,
    pub disambiguation_method: String,
    pub confidence: f64,
}

/// Domain integration result
#[derive(Debug, Clone)]
pub struct DomainIntegration {
    pub domain_hints: Vec<String>,
    pub integrated_entities: Vec<Entity>,
    pub domain_relevance_score: f64,
}

/// Subject-predicate-object triple for claim decomposition
#[derive(Debug, Clone)]
pub struct SubjectPredicateObject {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence: f64,
}

/// Verification requirement for claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRequirement {
    pub requirement_type: String,
    pub description: String,
    pub priority: VerificationPriority,
    pub evidence_needed: Vec<String>,
}

/// Verification priority levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationPriority {
    High,
    Medium,
    Low,
}

/// Entity match result
#[derive(Debug, Clone)]
pub struct EntityMatch {
    pub entity: Entity,
    pub confidence: f64,
    pub match_type: String,
    pub source: String,
}

/// Embedding service trait (placeholder)
pub trait EmbeddingService {
    fn embed(&self, text: &str) -> Result<Vec<f32>>;
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
}

/// Embedding request structure
#[derive(Debug, Clone)]
pub struct EmbeddingRequest {
    pub text: String,
    pub context: Option<String>,
    pub content_type: String,
}
