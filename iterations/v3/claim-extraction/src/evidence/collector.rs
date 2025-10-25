//! Main evidence collector implementation

use super::types::*;
use super::code_analysis::CodeAnalysisCollector;
use super::test_execution::TestExecutionCollector;
use super::documentation::DocumentationCollector;
use super::performance::PerformanceCollector;
use super::security::SecurityCollector;
use super::constitutional::ConstitutionalCollector;
use super::filtering::EvidenceFilter;
use crate::types::{AtomicClaim, ClaimType, Evidence, EvidenceType, EvidenceSource, ProcessingContext};
use anyhow::Result;
use tracing::{debug, info, warn};

/// Main evidence collector that orchestrates evidence collection from multiple sources
#[derive(Debug)]
pub struct EvidenceCollector {
    config: EvidenceCollectorConfig,
    code_analyzer: CodeAnalysisCollector,
    test_executor: TestExecutionCollector,
    doc_reviewer: DocumentationCollector,
    performance_analyzer: PerformanceCollector,
    security_scanner: SecurityCollector,
    constitutional_checker: ConstitutionalCollector,
    evidence_filter: EvidenceFilter,
}

impl EvidenceCollector {
    /// Create a new evidence collector with default configuration
    pub fn new() -> Self {
        Self {
            config: EvidenceCollectorConfig::default(),
            code_analyzer: CodeAnalysisCollector::new(),
            test_executor: TestExecutionCollector::new(),
            doc_reviewer: DocumentationCollector::new(),
            performance_analyzer: PerformanceCollector::new(),
            security_scanner: SecurityCollector::new(),
            constitutional_checker: ConstitutionalCollector::new(),
            evidence_filter: EvidenceFilter::new(),
        }
    }

    /// Create a new evidence collector with custom configuration
    pub fn with_config(config: EvidenceCollectorConfig) -> Self {
        Self {
            config: config.clone(),
            code_analyzer: CodeAnalysisCollector::with_config(config.clone()),
            test_executor: TestExecutionCollector::with_config(config.clone()),
            doc_reviewer: DocumentationCollector::with_config(config.clone()),
            performance_analyzer: PerformanceCollector::with_config(config.clone()),
            security_scanner: SecurityCollector::with_config(config.clone()),
            constitutional_checker: ConstitutionalCollector::with_config(config.clone()),
            evidence_filter: EvidenceFilter::with_config(config),
        }
    }

    /// Main entry point: collect evidence for a single atomic claim
    pub async fn collect_evidence(
        &self,
        claim: &AtomicClaim,
        context: &ProcessingContext,
    ) -> Result<Vec<Evidence>> {
        debug!("Collecting evidence for claim: {}", claim.claim_text);

        // Determine verification methods based on claim type
        let verification_methods = self.determine_verification_methods(claim);

        let mut all_evidence = Vec::new();

        for method in verification_methods {
            match self.collect_by_method(&method, claim, context).await {
                Ok(evidence) => {
                    debug!(
                        "Collected {} evidence items via {:?}",
                        evidence.len(),
                        method
                    );
                    all_evidence.extend(evidence);
                }
                Err(e) => {
                    warn!("Failed to collect evidence via {:?}: {}", method, e);
                }
            }
        }

        // Filter and rank evidence
        let filtered_evidence = self.evidence_filter.filter_and_rank_evidence(all_evidence, claim);

        info!(
            "Collected {} relevant evidence items for claim {}",
            filtered_evidence.len(),
            claim.id
        );

        Ok(filtered_evidence)
    }

    /// Determine verification methods based on claim type
    fn determine_verification_methods(&self, claim: &AtomicClaim) -> Vec<VerificationMethod> {
        let mut methods = Vec::new();

        match claim.claim_type {
            ClaimType::Factual => {
                methods.push(VerificationMethod::CodeAnalysis);
                if self.config.enable_cross_reference {
                    methods.push(VerificationMethod::DocumentationReview);
                }
            }
            ClaimType::Procedural => {
                methods.push(VerificationMethod::TestExecution);
                methods.push(VerificationMethod::CodeAnalysis);
            }
            ClaimType::Technical => {
                methods.push(VerificationMethod::CodeAnalysis);
                methods.push(VerificationMethod::DocumentationReview);
            }
            ClaimType::Performance => {
                methods.push(VerificationMethod::PerformanceMeasurement);
                methods.push(VerificationMethod::TestExecution);
            }
            ClaimType::Security => {
                methods.push(VerificationMethod::SecurityScan);
                methods.push(VerificationMethod::ConstitutionalCheck);
            }
            ClaimType::Constitutional => {
                methods.push(VerificationMethod::ConstitutionalCheck);
                methods.push(VerificationMethod::DocumentationReview);
            }
            ClaimType::Behavioral |
            ClaimType::Functional |
            ClaimType::Structural |
            ClaimType::Informational => {
                // Default verification methods for other claim types
                methods.push(VerificationMethod::CodeAnalysis);
                methods.push(VerificationMethod::DocumentationReview);
            }
        }

        methods
    }

    /// Collect evidence using a specific verification method
    async fn collect_by_method(
        &self,
        method: &VerificationMethod,
        claim: &AtomicClaim,
        context: &ProcessingContext,
    ) -> Result<Vec<Evidence>> {
        match method {
            VerificationMethod::CodeAnalysis => {
                self.code_analyzer.collect_evidence(claim, context).await
            }
            VerificationMethod::TestExecution => {
                self.test_executor.collect_evidence(claim, context).await
            }
            VerificationMethod::DocumentationReview => {
                self.doc_reviewer.collect_evidence(claim, context).await
            }
            VerificationMethod::PerformanceMeasurement => {
                self.performance_analyzer.collect_evidence(claim, context).await
            }
            VerificationMethod::SecurityScan => {
                self.security_scanner.collect_evidence(claim, context).await
            }
            VerificationMethod::ConstitutionalCheck => {
                self.constitutional_checker.collect_evidence(claim, context).await
            }
            VerificationMethod::Measurement |
            VerificationMethod::LogicalAnalysis |
            VerificationMethod::ProcessAnalysis => {
                // Placeholder for other verification methods
                Ok(vec![Evidence {
                    id: uuid::Uuid::new_v4(),
                    claim_id: claim.id,
                    evidence_type: EvidenceType::Supporting,
                    content: "Verification method not yet implemented".to_string(),
                    source: EvidenceSource::General {
                        location: "system".to_string(),
                        authority: "system".to_string(),
                        freshness: chrono::Utc::now(),
                    },
                    confidence: 0.5,
                    relevance: 0.5,
                    timestamp: chrono::Utc::now(),
                }])
            }
        }
    }

    /// Get collector configuration
    pub fn config(&self) -> &EvidenceCollectorConfig {
        &self.config
    }

    /// Collect CAWS provenance evidence for claims about development process
    pub async fn collect_caws_provenance_evidence(&self, claim: &AtomicClaim) -> Result<Vec<Evidence>> {
        debug!("Collecting CAWS provenance evidence for claim: {}", claim.id);

        // This would integrate with CAWS provenance tracking
        // For now, return placeholder evidence
        Ok(vec![Evidence {
            id: uuid::Uuid::new_v4(),
            claim_id: claim.id,
            evidence_type: EvidenceType::Supporting,
            content: "CAWS provenance evidence collection not yet implemented".to_string(),
            source: EvidenceSource::General {
                location: "caws".to_string(),
                authority: "caws".to_string(),
                freshness: chrono::Utc::now(),
            },
            confidence: 0.6,
            relevance: 0.7,
            timestamp: chrono::Utc::now(),
        }])
    }
}

impl Default for EvidenceCollector {
    fn default() -> Self {
        Self::new()
    }
}
