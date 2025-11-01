//! Research and Evidence Types - DTOs for research operations
//!
//! Defines the data transfer objects used by the research evidence collector.
//! These types enable clean communication between orchestration and research services.
//!
//! @author @darianrosebrook

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Evidence collected from research and analysis
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// Types of evidence that can be collected
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceQuery {
    /// Topic or claim to collect evidence for
    pub query: String,
    /// Types of evidence to collect
    pub evidence_types: Vec<EvidenceType>,
    /// Context information for the query
    pub context: std::collections::HashMap<String, serde_json::Value>,
    /// Maximum number of results to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    /// Minimum confidence threshold
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_confidence: Option<f64>,
}

/// Validation result for evidence
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidationResult {
    /// Whether the evidence is valid
    pub is_valid: bool,
    /// Validation score (0.0 to 1.0)
    pub score: f64,
    /// Validation issues or concerns
    pub issues: Vec<String>,
    /// Validation metadata
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// Statistics about evidence collection system
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<String>")]
    pub last_collection_time: Option<DateTime<Utc>>,
}
