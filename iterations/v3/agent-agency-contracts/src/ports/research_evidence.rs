//! Research Evidence Collector Port
//!
//! Defines the interface for collecting evidence and research findings.
//! This port enables dependency injection and testing for evidence-based validation.
//!
//! @author @darianrosebrook

use crate::errors::ResearchResult;
use crate::types::research::{Evidence, EvidenceQuery, EvidenceStats, ValidationResult};

/// Core research evidence collector interface
/// Implementations provide evidence collection and validation capabilities
#[async_trait::async_trait]
pub trait ResearchEvidenceCollector: Send + Sync {
    /// Collect evidence for a given query
    ///
    /// # Arguments
    /// * `query` - The evidence collection query
    ///
    /// # Returns
    /// Vector of collected evidence, or an error if collection fails
    async fn collect_evidence(&self, query: EvidenceQuery) -> ResearchResult<Vec<Evidence>>;

    /// Validate a piece of evidence
    ///
    /// # Arguments
    /// * `evidence` - The evidence to validate
    ///
    /// # Returns
    /// Validation result indicating validity and quality, or an error if validation fails
    async fn validate_evidence(&self, evidence: &Evidence) -> ResearchResult<ValidationResult>;

    /// Search for existing evidence matching criteria
    ///
    /// # Arguments
    /// * `criteria` - Search criteria as JSON value
    ///
    /// # Returns
    /// Vector of matching evidence, or an error if search fails
    async fn search_evidence(&self, criteria: serde_json::Value) -> ResearchResult<Vec<Evidence>>;

    /// Get evidence statistics and health metrics
    ///
    /// # Returns
    /// Statistics about evidence collection and validation
    async fn get_evidence_stats(&self) -> ResearchResult<EvidenceStats>;
}
