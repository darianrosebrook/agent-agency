//! Research domain types - DTOs and ports for research operations
//!
//! This module provides shared types for research, evidence collection,
//! and disambiguation operations across the agent-research crate.
//!
//! @author @darianrosebrook

pub mod dto;
pub mod errors;
pub mod ports;

// Re-export all new types
pub use dto::*;
pub use errors::*;
pub use ports::*;

// ============================================================================
// Research Evidence Types (shared between agent-research and agent-orchestration)
// ============================================================================

/// Research evidence collected during task execution
///
/// This struct is shared between agent-research and agent-orchestration
/// to avoid circular dependencies. Both crates should import from here.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", derive(schemars::JsonSchema))]
#[derive(Debug, Clone)]
pub struct ResearchEvidence {
    /// Unique identifier for this evidence
    #[cfg_attr(feature = "serde", schemars(with = "String"))]
    pub id: uuid::Uuid,
    /// Content of the evidence
    pub content: String,
    /// Type of evidence collected
    pub evidence_type: ResearchEvidenceType,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
    /// Source of the evidence
    pub source: String,
    /// When the evidence was collected
    #[cfg_attr(feature = "serde", schemars(with = "String"))]
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ResearchEvidence {
    /// Create new research evidence
    pub fn new(
        content: String,
        evidence_type: ResearchEvidenceType,
        confidence: f64,
        source: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            content,
            evidence_type,
            confidence,
            source,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Types of research evidence (used by agent-orchestration)
///
/// This enum maps to EvidenceType but uses different naming conventions
/// that match the agent-orchestration crate's expectations.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResearchEvidenceType {
    /// Code review evidence
    CodeReview,
    /// Code analysis evidence (alias for CodeReview)
    CodeAnalysis,
    /// Test execution results
    TestExecution,
    /// Performance metrics
    PerformanceMetrics,
    /// Performance evidence (alias for PerformanceMetrics)
    Performance,
    /// Security scan results
    SecurityScan,
    /// Security evidence (alias for SecurityScan)
    Security,
    /// Constitutional/CAWS compliance evidence
    Constitutional,
    /// Documentation evidence
    Documentation,
}

impl Default for ResearchEvidenceType {
    fn default() -> Self {
        Self::CodeReview
    }
}

impl From<EvidenceType> for ResearchEvidenceType {
    fn from(et: EvidenceType) -> Self {
        match et {
            EvidenceType::CodeAnalysis => ResearchEvidenceType::CodeAnalysis,
            EvidenceType::TestResults | EvidenceType::TestResult => ResearchEvidenceType::TestExecution,
            EvidenceType::Documentation => ResearchEvidenceType::Documentation,
            EvidenceType::PerformanceMetrics => ResearchEvidenceType::PerformanceMetrics,
            EvidenceType::SecurityScan => ResearchEvidenceType::SecurityScan,
            EvidenceType::ConstitutionalReference => ResearchEvidenceType::Constitutional,
            _ => ResearchEvidenceType::CodeReview,
        }
    }
}

/// Context for research evidence collection
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", derive(schemars::JsonSchema))]
#[derive(Debug, Clone)]
pub struct ResearchContext {
    /// Task identifier
    #[cfg_attr(feature = "serde", schemars(with = "String"))]
    pub task_id: uuid::Uuid,
    /// Milestone identifier
    pub milestone_id: String,
    /// Types of evidence to collect
    pub evidence_types: Vec<ResearchEvidenceType>,
    /// Priority level
    pub priority: String,
}

/// Trait for research evidence collection
///
/// Implementations can provide different evidence collection strategies.
#[async_trait::async_trait]
pub trait ResearchEvidenceCollector: Send + Sync {
    /// Collect evidence for a given context
    async fn collect_evidence(
        &self,
        context: &ResearchContext,
    ) -> anyhow::Result<Vec<ResearchEvidence>>;
}

/// No-op research evidence collector for when research feature is disabled
pub struct NoOpResearchEvidenceCollector;

#[async_trait::async_trait]
impl ResearchEvidenceCollector for NoOpResearchEvidenceCollector {
    async fn collect_evidence(
        &self,
        _context: &ResearchContext,
    ) -> anyhow::Result<Vec<ResearchEvidence>> {
        Ok(vec![])
    }
}

// Keep existing Evidence types for backward compatibility
// These are still used by the ResearchEvidenceCollector port

/// Evidence collected from research and analysis
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", derive(schemars::JsonSchema))]
#[derive(Debug, Clone)]
pub struct Evidence {
    /// Unique identifier for this evidence
    pub id: String,
    /// Type of evidence collected
    pub evidence_type: EvidenceType,
    /// Evidence content or findings
    pub content: String,
    /// Source of the evidence
    pub source: String,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Relevance score (0.0 to 1.0)
    pub relevance: f64,
    /// Timestamp when evidence was collected
    #[cfg_attr(feature = "serde", schemars(with = "String"))]
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// Types of evidence that can be collected
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceType {
    CodeAnalysis,
    TestResults,
    Documentation,
    ResearchFindings,
    PerformanceMetrics,
    SecurityScan,
    ConstitutionalReference,
    CouncilDecision,
    MultiModalAnalysis,
    ExternalSource,
    TestResult,
    UserFeedback,
    Measurement,
    LogicalAnalysis,
    Supporting,
}

/// Query for evidence collection
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", derive(schemars::JsonSchema))]
#[derive(Debug, Clone)]
pub struct EvidenceQuery {
    /// Topic or claim to collect evidence for
    pub query: String,
    /// Types of evidence to collect
    pub evidence_types: Vec<EvidenceType>,
    /// Context information for the query
    pub context: std::collections::HashMap<String, serde_json::Value>,
    /// Maximum number of results to return
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub limit: Option<usize>,
    /// Minimum confidence threshold
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub min_confidence: Option<f64>,
}

/// Validation result for evidence - uses string issues for simplicity
pub type ValidationResult = super::validation::ValidationResult<String>;

/// Statistics about evidence collection system
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", derive(schemars::JsonSchema))]
#[derive(Debug, Clone)]
pub struct EvidenceStats {
    /// Total number of evidence items collected
    pub total_evidence: u64,
    /// Average confidence score across all evidence
    pub average_confidence: f64,
    /// Number of evidence validations performed
    pub validations_performed: u64,
    /// Average validation score
    pub average_validation_score: f64,
    /// Evidence collection success rate (0.0 to 1.0)
    pub collection_success_rate: f64,
    /// Last collection timestamp
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "serde", schemars(with = "Option<String>"))]
    pub last_collection_time: Option<chrono::DateTime<chrono::Utc>>,
}
