//! Types for claim extraction and verification pipeline

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

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
}

/// Metadata about the processing operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingMetadata {
    pub processing_time_ms: u64,
    pub stages_completed: Vec<ProcessingStage>,
    pub ambiguities_resolved: u32,
    pub claims_extracted: u32,
    pub evidence_collected: u32,
    pub errors: Vec<ProcessingError>,
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
