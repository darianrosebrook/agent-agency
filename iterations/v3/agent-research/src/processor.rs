use crate::decomposition::DecompositionStage;
use crate::disambiguation::DisambiguationStage;
use crate::qualification::QualificationStage;
use crate::extraction_types::VerifiedClaim;
use crate::extraction_types::*;
use crate::verification::MultiModalVerificationEngine;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use std::time::Instant;
use tracing::{debug, info};

/// Main claim extraction processor with multi-modal verification integration

#[derive(Debug)]
pub struct ClaimExtractionProcessor {
    disambiguation_stage: DisambiguationStage,
    qualification_stage: QualificationStage,
    decomposition_stage: DecompositionStage,
    verification_engine: MultiModalVerificationEngine,
}

impl ClaimExtractionProcessor {
    /// Create a new claim extraction processor with all stages
    pub fn new() -> Self {
        Self {
            disambiguation_stage: DisambiguationStage::minimal(),
            qualification_stage: QualificationStage::new(),
            decomposition_stage: DecompositionStage::new(),
            verification_engine: MultiModalVerificationEngine::new(),
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
            .process(&disambiguation_result.disambiguated_sentence, ctx)
            .await
            .map_err(|e| ClaimExtractionError::QualificationFailed(e.to_string()))?;

        // Stage 3: Decomposition (Enhanced V2 with advanced atomic claim extraction)
        debug!("Stage 3: Decomposition");
        let decomposition_result = self
            .decomposition_stage
            .process(&disambiguation_result.disambiguated_sentence, ctx)
            .await
            .map_err(|e| ClaimExtractionError::DecompositionFailed(e.to_string()))?;

        // Stage 4: Verification - Multi-modal verification of atomic claims
        debug!("Stage 4: Verification - Running multi-modal verification");
        let atomic_claims = decomposition_result.atomic_claims.clone();
        let verification_results = self
            .verification_engine
            .verify_claims(&atomic_claims)
            .await
            .map_err(|e| ClaimExtractionError::VerificationFailed(e.to_string()))?;

        info!(
            "Verification completed: {}/{} claims verified successfully",
            verification_results.successful_verifications,
            verification_results.total_processed
        );

        // Convert VerificationResults to VerificationResult format
        let verified_claims = VerificationResult {
            verified_claims: verification_results.verified_claims.clone(),
            evidence: verification_results.verified_claims.iter()
                .flat_map(|vc| vc.evidence.clone())
                .collect(),
            verification_confidence: if verification_results.total_processed > 0 {
                verification_results.successful_verifications as f64 
                    / verification_results.total_processed as f64
            } else {
                0.0
            },
            council_verification: CouncilVerificationResult {
                submitted_claims: atomic_claims.iter()
                    .map(|c| c.id)
                    .collect(),
                council_verdict: format!(
                    "Verified {}/{} claims with multi-modal analysis",
                    verification_results.successful_verifications,
                    verification_results.total_processed
                ),
                additional_evidence: verification_results.verified_claims.iter()
                    .flat_map(|vc| vc.evidence.clone())
                    .collect(),
                verification_timestamp: chrono::Utc::now(),
            },
            overall_confidence: if verification_results.total_processed > 0 {
                verification_results.verified_claims.iter()
                    .map(|vc| vc.overall_confidence)
                    .sum::<f64>() / verification_results.total_processed as f64
            } else {
                0.0
            },
        };

        let processing_time = start_time.elapsed().as_millis() as u64;
        info!("Claim extraction completed in {}ms", processing_time);

        // Combine evidence from verification stage
        let mut all_evidence = verified_claims.evidence.clone();

        // Add evidence from verified claims
        for verified_claim in &verified_claims.verified_claims {
            all_evidence.extend(verified_claim.evidence.clone());
        }

        let claims_count = decomposition_result.atomic_claims.len();
        let evidence_count = all_evidence.len();

        Ok(ClaimExtractionResult {
            original_sentence: input.to_string(),
            disambiguated_sentence: disambiguation_result.disambiguated_sentence,
            atomic_claims: decomposition_result.atomic_claims,
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

}
