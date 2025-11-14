//! Main evidence collector implementation

use super::code_analysis::CodeAnalysisCollector;
use super::constitutional::ConstitutionalCollector;
use super::documentation::DocumentationCollector;
use super::filtering::EvidenceFilter;
use super::performance::PerformanceCollector;
use super::security::SecurityCollector;
use super::test_execution::TestExecutionCollector;
use super::types::*;
use crate::evidence::evidence_types::EvidenceCollectorConfig;
use crate::evidence::evidence_types::VerificationMethod;
use crate::extraction_types::{
    AtomicClaim, ClaimType, Evidence, EvidenceSource, EvidenceType, ProcessingContext,
};
use anyhow::Result;
use std::sync::Arc;
use system_quality_security::provenance_service::ProvenanceService;
use tracing::{debug, info, warn};

use schemars::JsonSchema;
/// Main evidence collector that orchestrates evidence collection from multiple sources
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct EvidenceCollector {
    config: EvidenceCollectorConfig,
    code_analyzer: CodeAnalysisCollector,
    test_executor: TestExecutionCollector,
    doc_reviewer: DocumentationCollector,
    performance_analyzer: PerformanceCollector,
    security_scanner: SecurityCollector,
    constitutional_checker: ConstitutionalCollector,
    evidence_filter: EvidenceFilter,
    #[serde(skip)]
    provenance_service: Option<Arc<ProvenanceService>>,
}

impl EvidenceCollector {
    /// Create a new evidence collector with default configuration
    pub fn new() -> Self {
        Self {
            config: EvidenceCollectorConfig::default(),
            code_analyzer: CodeAnalysisCollector::new(),
            test_executor: TestExecutionCollector::new(Default::default()),
            doc_reviewer: DocumentationCollector::new(),
            performance_analyzer: PerformanceCollector::new(),
            security_scanner: SecurityCollector::new(),
            constitutional_checker: ConstitutionalCollector::new(),
            evidence_filter: EvidenceFilter::new(),
            provenance_service: None,
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
            provenance_service: None,
        }
    }

    /// Create a new evidence collector with provenance service
    pub fn with_provenance_service(provenance_service: Arc<ProvenanceService>) -> Self {
        Self {
            config: EvidenceCollectorConfig::default(),
            code_analyzer: CodeAnalysisCollector::new(),
            test_executor: TestExecutionCollector::new(Default::default()),
            doc_reviewer: DocumentationCollector::new(),
            performance_analyzer: PerformanceCollector::new(),
            security_scanner: SecurityCollector::new(),
            constitutional_checker: ConstitutionalCollector::new(),
            evidence_filter: EvidenceFilter::new(),
            provenance_service: Some(provenance_service),
        }
    }

    /// Set provenance service (for dependency injection)
    pub fn set_provenance_service(&mut self, provenance_service: Arc<ProvenanceService>) {
        self.provenance_service = Some(provenance_service);
    }

    /// Main entry point: collect evidence for a single atomic claim
    pub async fn collect_evidence(
        &mut self,
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
        let filtered_evidence = self
            .evidence_filter
            .filter_and_rank_evidence(all_evidence, claim);

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
            ClaimType::Behavioral
            | ClaimType::Functional
            | ClaimType::Structural
            | ClaimType::Informational
            | ClaimType::Causal
            | ClaimType::Conditional
            | ClaimType::Quantitative
            | ClaimType::Requirement => {
                // Default verification methods for other claim types
                methods.push(VerificationMethod::CodeAnalysis);
                methods.push(VerificationMethod::DocumentationReview);
            }
        }

        methods
    }

    /// Collect evidence using a specific verification method
    async fn collect_by_method(
        &mut self,
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
                self.performance_analyzer
                    .collect_evidence(claim, context)
                    .await
            }
            VerificationMethod::SecurityScan => {
                self.security_scanner.collect_evidence(claim, context).await
            }
            VerificationMethod::ConstitutionalCheck => {
                self.constitutional_checker
                    .collect_evidence(claim, context)
                    .await
            }
            VerificationMethod::Measurement
            | VerificationMethod::LogicalAnalysis
            | VerificationMethod::ProcessAnalysis => {
                // TODO: Implement verification methods (Measurement, LogicalAnalysis, ProcessAnalysis)
                // - [ ] Implement measurement-based verification (numeric validation, range checks)
                // - [ ] Implement logical analysis verification (logical consistency checks)
                // - [ ] Implement process analysis verification (workflow validation)
                // - [ ] Collect appropriate evidence for each method
                // - [ ] Calculate confidence scores for each verification method
                // - [ ] Add unit tests for each verification method
                // - [ ] Add integration tests with real verification
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
            // Catch-all for future non-exhaustive variants
            _ => Ok(vec![]),
        }
    }

    /// Get collector configuration
    pub fn config(&self) -> &EvidenceCollectorConfig {
        &self.config
    }

    /// Collect CAWS provenance evidence for claims about development process
    pub async fn collect_caws_provenance_evidence(
        &self,
        claim: &AtomicClaim,
    ) -> Result<Vec<Evidence>> {
        debug!(
            "Collecting CAWS provenance evidence for claim: {}",
            claim.id
        );

        // Extract task_id from claim scope
        // working_spec_id might be a UUID string or a plan ID like "PLAN-123"
        // Try parsing as UUID first, then try to extract from plan ID format
        let task_id = claim
            .scope
            .working_spec_id
            .parse::<uuid::Uuid>()
            .ok()
            .or_else(|| {
                // If working_spec_id is in format "PLAN-{uuid}", extract the UUID part
                if claim.scope.working_spec_id.starts_with("PLAN-") {
                    claim
                        .scope
                        .working_spec_id
                        .strip_prefix("PLAN-")
                        .and_then(|s| s.parse::<uuid::Uuid>().ok())
                } else {
                    None
                }
            });

        // If no provenance service available, return empty evidence with warning
        let provenance_service = match &self.provenance_service {
            Some(service) => service,
            None => {
                warn!("ProvenanceService not available - cannot collect CAWS provenance evidence");
                return Ok(vec![]);
            }
        };

        // If no task_id available, try to query by claim text or return empty
        let task_id = match task_id {
            Some(id) => id,
            None => {
                warn!("Cannot extract task_id from claim scope - skipping provenance evidence collection");
                return Ok(vec![]);
            }
        };

        // Query provenance chain for this task
        let provenance_chain = match provenance_service.get_provenance_chain(task_id).await {
            Ok(chain) => chain,
            Err(e) => {
                warn!(
                    "Failed to query provenance chain for task {}: {}",
                    task_id, e
                );
                return Ok(vec![]);
            }
        };

        // Convert provenance entries to evidence
        let mut evidence_items = Vec::new();

        for record in provenance_chain.entries {
            // Extract CAWS compliance information
            let caws_compliance = &record.caws_compliance;

            // Build evidence content from provenance record
            let mut content_parts = Vec::new();
            content_parts.push(format!(
                "CAWS Compliance Score: {:.2}",
                caws_compliance.compliance_score
            ));
            content_parts.push(format!("Compliant: {}", caws_compliance.is_compliant));

            if !caws_compliance.violations.is_empty() {
                content_parts.push(format!("Violations: {}", caws_compliance.violations.len()));
                for violation in &caws_compliance.violations {
                    let severity_str = format!("{:?}", violation.severity);
                    content_parts.push(format!("  - [{}] {}", severity_str, violation.description));
                }
            }

            if !caws_compliance.waivers_used.is_empty() {
                content_parts.push(format!(
                    "Waivers Used: {}",
                    caws_compliance.waivers_used.len()
                ));
            }

            // Calculate relevance based on claim type and provenance data
            let relevance = if matches!(claim.claim_type, ClaimType::Constitutional) {
                0.9 // High relevance for constitutional claims
            } else if caws_compliance.is_compliant {
                0.7 // Medium relevance for compliant records
            } else {
                0.5 // Lower relevance for non-compliant records
            };

            // Calculate confidence based on compliance score and chain integrity
            let confidence = if provenance_chain.integrity_verified {
                caws_compliance.compliance_score as f64
            } else {
                caws_compliance.compliance_score as f64 * 0.8 // Reduce confidence if chain integrity not verified
            };

            // Determine evidence type based on compliance status
            let evidence_type = if caws_compliance.is_compliant {
                EvidenceType::ConstitutionalReference
            } else {
                EvidenceType::Supporting
            };

            evidence_items.push(Evidence {
                id: record.id,
                claim_id: claim.id,
                evidence_type,
                content: content_parts.join("\n"),
                source: EvidenceSource::General {
                    location: format!("provenance:{}", record.id),
                    authority: "CAWS Provenance System".to_string(),
                    freshness: record.timestamp,
                },
                confidence,
                relevance,
                timestamp: record.timestamp,
            });
        }

        info!(
            "Collected {} CAWS provenance evidence items for claim {} from task {}",
            evidence_items.len(),
            claim.id,
            task_id
        );

        Ok(evidence_items)
    }
}

impl Default for EvidenceCollector {
    fn default() -> Self {
        Self::new()
    }
}
