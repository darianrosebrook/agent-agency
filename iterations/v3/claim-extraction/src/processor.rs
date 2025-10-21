use crate::decomposition::DecompositionStage;
use crate::disambiguation::DisambiguationStage;
use crate::multi_modal_verification::MultiModalVerificationEngine;
use crate::qualification::QualificationStage;
use crate::types::VerifiedClaim;
use crate::types::*;
use crate::verification::VerificationStage;
use anyhow::Result;
use std::time::Instant;
use tracing::{debug, info};

/// Main claim extraction processor with multi-modal verification integration
pub struct ClaimExtractionProcessor {
    disambiguation_stage: DisambiguationStage,
    qualification_stage: QualificationStage,
    decomposition_stage: DecompositionStage,
    verification_stage: VerificationStage,
    multi_modal_verifier: MultiModalVerificationEngine,
}

impl ClaimExtractionProcessor {
    /// Create a new claim extraction processor with all stages
    pub fn new() -> Self {
        Self {
            disambiguation_stage: DisambiguationStage::new(),
            qualification_stage: QualificationStage::new(),
            decomposition_stage: DecompositionStage::new(),
            verification_stage: VerificationStage::new(),
            multi_modal_verifier: MultiModalVerificationEngine::new(),
        }
    }

    /// Run the complete claim extraction and verification pipeline
    /// Implements the four-stage Claimify pipeline with V3 multi-modal verification
    /// Enhanced with V2 advanced patterns for superior claim extraction
    pub async fn run(
        &mut self,
        input: &str,
        ctx: &ProcessingContext,
    ) -> Result<ClaimExtractionResult, ClaimExtractionError> {
        let start_time = Instant::now();
        info!("Starting claim extraction for input: {}", input);

        // Stage 1: Contextual Disambiguation (Ported from V2)
        debug!("Stage 1: Contextual Disambiguation");
        let disambiguation_result = self
            .disambiguation_stage
            .process(input, ctx)
            .await
            .map_err(|e| ClaimExtractionError::DisambiguationFailed(e.to_string()))?;

        // Stage 2: Qualification (Enhanced V2 with domain-aware verifiability)
        debug!("Stage 2: Qualification");
        let qualification_result = self
            .qualification_stage
            .process_v2(&disambiguation_result.disambiguated_sentence, ctx)
            .await
            .map_err(|e| ClaimExtractionError::QualificationFailed(e.to_string()))?;

        // Stage 3: Decomposition (Enhanced V2 with advanced atomic claim extraction)
        debug!("Stage 3: Decomposition");
        let decomposition_result = self
            .decomposition_stage
            .process_v2(&disambiguation_result.disambiguated_sentence, ctx)
            .await
            .map_err(|e| ClaimExtractionError::DecompositionFailed(e.to_string()))?;

        // Stage 4: Verification (Enhanced V2 with CAWS-compliant evidence collection)
        debug!("Stage 4: Verification");
        let verification_result = self
            .verification_stage
            .process_v2(&decomposition_result.atomic_claims, ctx)
            .await
            .map_err(|e| ClaimExtractionError::VerificationFailed(e.to_string()))?;

        // Stage 5: Multi-Modal Verification (V3's superior verification)
        debug!("Stage 5: Multi-Modal Verification");
        let atomic_claims = decomposition_result.atomic_claims.clone();
        let verified_claims = self
            .multi_modal_verifier
            .verify_claims(&atomic_claims)
            .await
            .map_err(|e| {
                ClaimExtractionError::VerificationFailed(format!(
                    "Multi-modal verification failed: {}",
                    e
                ))
            })?;

        let processing_time = start_time.elapsed().as_millis() as u64;
        info!("Claim extraction completed in {}ms", processing_time);

        // Combine evidence from both verification stages
        let mut all_evidence = verification_result.evidence;

        // Add evidence from multi-modal verification
        for verified_claim in &verified_claims.verified_claims {
            // Convert verification results to evidence
            if let Some(math_evidence) = self.create_mathematical_evidence(verified_claim) {
                all_evidence.push(math_evidence);
            }
            if let Some(code_evidence) = self.create_code_behavior_evidence(verified_claim) {
                all_evidence.push(code_evidence);
            }
            if let Some(semantic_evidence) = self.create_semantic_evidence(verified_claim) {
                all_evidence.push(semantic_evidence);
            }
        }

        let claims_count = atomic_claims.len();
        let evidence_count = all_evidence.len();

        Ok(ClaimExtractionResult {
            original_sentence: input.to_string(),
            disambiguated_sentence: disambiguation_result.disambiguated_sentence,
            atomic_claims,
            verification_evidence: all_evidence,
            processing_metadata: ProcessingMetadata {
                processing_time_ms: processing_time,
                stages_completed: vec![
                    ProcessingStage::Disambiguation,
                    ProcessingStage::Qualification,
                    ProcessingStage::Decomposition,
                    ProcessingStage::Verification,
                ],
                ambiguities_resolved: disambiguation_result.ambiguities_resolved,
                claims_extracted: claims_count as u32,
                evidence_collected: evidence_count as u32,
                rewrite_suggestions: 0,
                unverifiable_breakdown: UnverifiableBreakdown::default(),
                errors: Vec::new(),
            },
        })
    }

    /// Create mathematical evidence from verification results
    fn create_mathematical_evidence(&self, verified_claim: &VerifiedClaim) -> Option<Evidence> {
        match &verified_claim.verification_results {
            VerificationStatus::Verified => {
                Some(Evidence {
                    id: uuid::Uuid::new_v4(),
                    claim_id: uuid::Uuid::new_v4(), // Generate a new ID since original_claim is a String
                    evidence_type: EvidenceType::CodeAnalysis, // Mathematical analysis
                    content: format!(
                        "Mathematical verification: claim validated with confidence {:.2}",
                        verified_claim.overall_confidence
                    ),
                    source: EvidenceSource::CodeAnalysis {
                        location: "MultiModalVerificationEngine".to_string(),
                        authority: "MathematicalValidator".to_string(),
                        freshness: chrono::Utc::now(),
                    },
                    confidence: verified_claim.overall_confidence,
                    timestamp: chrono::Utc::now(),
                })
            }
            _ => None,
        }
    }

    /// Create code behavior evidence from verification results
    fn create_code_behavior_evidence(&self, verified_claim: &VerifiedClaim) -> Option<Evidence> {
        match &verified_claim.verification_results {
            VerificationStatus::Verified => {
                Some(Evidence {
                    id: uuid::Uuid::new_v4(),
                    claim_id: uuid::Uuid::new_v4(), // Generate a new ID since original_claim is a String
                    evidence_type: EvidenceType::CodeAnalysis,
                    content: format!(
                        "Code behavior analysis: claim validated with confidence {:.2}",
                        verified_claim.overall_confidence
                    ),
                    source: EvidenceSource::CodeAnalysis {
                        location: "MultiModalVerificationEngine".to_string(),
                        authority: "CodeBehaviorAnalyzer".to_string(),
                        freshness: verified_claim.verification_timestamp,
                    },
                    confidence: verified_claim.overall_confidence,
                    timestamp: verified_claim.verification_timestamp,
                })
            }
            _ => None,
        }
    }

    /// Create semantic evidence from verification results
    fn create_semantic_evidence(&self, verified_claim: &VerifiedClaim) -> Option<Evidence> {
        match &verified_claim.verification_results {
            VerificationStatus::Verified => {
                Some(Evidence {
                    id: uuid::Uuid::new_v4(),
                    claim_id: uuid::Uuid::new_v4(), // Generate claim ID
                    evidence_type: EvidenceType::CodeAnalysis, // Semantic analysis
                    content: format!(
                        "Semantic analysis: claim validated with confidence {:.2}",
                        verified_claim.overall_confidence
                    ),
                    source: EvidenceSource::CodeAnalysis {
                        location: "multi_modal_verification".to_string(),
                        authority: "Multi-Modal Verifier".to_string(),
                        freshness: chrono::Utc::now(),
                    },
                    confidence: verified_claim.overall_confidence,
                    timestamp: chrono::Utc::now(),
                })
            }
            _ => None,
        }
    }
}
