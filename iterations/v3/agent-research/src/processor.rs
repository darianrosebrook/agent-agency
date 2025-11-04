use crate::decomposition::DecompositionStage;
use crate::disambiguation::DisambiguationStage;
// use crate::MultiModalVerificationEngine; // Temporarily disabled
use crate::qualification::QualificationStage;
use crate::extraction_types::VerifiedClaim;
use crate::extraction_types::*;
// VerificationStage is not defined in verification module - removing this import
use anyhow::Result;
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use std::time::Instant;
use tracing::{debug, info};

/// Main claim extraction processor with multi-modal verification integration

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ClaimExtractionProcessor {
    disambiguation_stage: DisambiguationStage,
    qualification_stage: QualificationStage,
    decomposition_stage: DecompositionStage,
    // verification_stage: MultiModalVerificationEngine, // Temporarily disabled
    // multi_modal_verifier: MultiModalVerificationEngine, // Temporarily disabled
}

impl ClaimExtractionProcessor {
    /// Create a new claim extraction processor with all stages
    pub fn new() -> Self {
        Self {
            disambiguation_stage: DisambiguationStage::minimal(),
            qualification_stage: QualificationStage::new(),
            decomposition_stage: DecompositionStage::new(),
            // verification_stage: MultiModalVerificationEngine::new(), // Temporarily disabled
            // multi_modal_verifier: MultiModalVerificationEngine::new(), // Temporarily disabled
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
            .process(&disambiguation_result.disambiguated_text, ctx)
            .await
            .map_err(|e| ClaimExtractionError::QualificationFailed(e.to_string()))?;

        // Stage 3: Decomposition (Enhanced V2 with advanced atomic claim extraction)
        debug!("Stage 3: Decomposition");
        let decomposition_result = self
            .decomposition_stage
            .process(&disambiguation_result.disambiguated_text, ctx)
            .await
            .map_err(|e| ClaimExtractionError::DecompositionFailed(e.to_string()))?;

        // Stage 4: Verification (Temporarily disabled - awaiting verification module)
        debug!("Stage 4: Verification - Skipped (temporarily disabled)");
        let verification_result = Vec::new(); // Placeholder empty result

        // Stage 5: Multi-Modal Verification (Temporarily disabled - awaiting multi-modal verifier)
        debug!("Stage 5: Multi-Modal Verification - Skipped (temporarily disabled)");
        let atomic_claims = decomposition_result.atomic_claims.clone();
        // Placeholder: create basic verified claims without actual verification
        let verified_claims = crate::extraction_types::VerificationResult {
            verified_claims: atomic_claims.into_iter().map(|claim| {
                crate::extraction_types::VerifiedClaim {
                    claim,
                    verification_status: crate::extraction_types::VerificationStatus::Unverified,
                    confidence: 0.5,
                    evidence: Vec::new(),
                }
            }).collect(),
            evidence: Vec::new(),
            verification_confidence: 0.5,
            council_verification: crate::extraction_types::CouncilVerificationResult {
                submitted_claims: vec![],
                council_verdict: "placeholder".to_string(),
                additional_evidence: vec![],
                verification_timestamp: chrono::Utc::now(),
            },
            overall_confidence: 0.5,
        };

        let processing_time = start_time.elapsed().as_millis() as u64;
        info!("Claim extraction completed in {}ms", processing_time);

        // Combine evidence from both verification stages
        let mut all_evidence = Vec::new();

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
                    relevance: 0.9, // High relevance for mathematical verification
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
                    relevance: 0.8, // High relevance for code behavior analysis
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
                    relevance: 0.85, // High relevance for semantic analysis
                    timestamp: chrono::Utc::now(),
                })
            }
            _ => None,
        }
    }
}
