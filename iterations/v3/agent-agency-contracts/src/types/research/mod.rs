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
