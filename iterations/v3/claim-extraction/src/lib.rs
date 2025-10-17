//! Claim Extraction & Verification Pipeline
//!
//! Implements the 4-stage claim processing pipeline required by theory:
//! 1. Contextual disambiguation
//! 2. Verifiable content qualification  
//! 3. Atomic claim decomposition
//! 4. CAWS-compliant verification
//!
//! Based on V2 ClaimExtractor.ts (1677 lines) with Rust adaptations and
//! council integration for evidence collection in debate protocol.

pub mod decomposition;
pub mod disambiguation;
pub mod evidence;
pub mod multi_modal_verification;
pub mod processor;
pub mod qualification;
pub mod types;
pub mod verification;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod multi_modal_verification_tests;

pub use multi_modal_verification::{
    MultiModalVerificationEngine, VerificationResults, VerifiedClaim,
};
pub use processor::ClaimExtractionProcessor;
pub use types::*;

use anyhow::Result;
use std::time::Instant;
use tracing::{info, warn};

/// Main claim extraction and verification processor
///
/// Integrates with council debate protocol to provide evidence
/// for claim verification during judicial evaluation.
#[derive(Debug)]
pub struct ClaimExtractionAndVerificationProcessor {
    disambiguation_stage: disambiguation::DisambiguationStage,
    qualification_stage: qualification::QualificationStage,
    decomposition_stage: decomposition::DecompositionStage,
    verification_stage: verification::VerificationStage,
}

impl ClaimExtractionAndVerificationProcessor {
    /// Create a new claim extraction processor
    pub fn new() -> Self {
        Self {
            disambiguation_stage: disambiguation::DisambiguationStage::new(),
            qualification_stage: qualification::QualificationStage::new(),
            decomposition_stage: decomposition::DecompositionStage::new(),
            verification_stage: verification::VerificationStage::new(),
        }
    }

    /// Process a sentence through the complete 4-stage pipeline
    pub async fn process_sentence(
        &self,
        sentence: &str,
        context: &ProcessingContext,
    ) -> Result<ClaimExtractionResult, ClaimExtractionError> {
        let start_time = Instant::now();
        info!("Starting claim extraction for sentence: {}", sentence);

        let mut stages_completed = Vec::new();
        let mut errors = Vec::new();
        let mut disambiguated_sentence = sentence.to_string();
        let mut atomic_claims = Vec::new();
        let mut verification_evidence = Vec::new();

        // Stage 1: Disambiguation
        match self.disambiguation_stage.process(sentence, context).await {
            Ok(disambiguation_result) => {
                disambiguated_sentence = disambiguation_result.disambiguated_sentence;
                stages_completed.push(ProcessingStage::Disambiguation);
                info!(
                    "Disambiguation completed: {} ambiguities resolved",
                    disambiguation_result.ambiguities_resolved
                );
            }
            Err(e) => {
                let error = ProcessingError {
                    stage: ProcessingStage::Disambiguation,
                    error_type: "DisambiguationFailed".to_string(),
                    message: e.to_string(),
                    recoverable: true,
                };
                errors.push(error);
                warn!(
                    "Disambiguation failed, continuing with original sentence: {}",
                    e
                );
            }
        }

        // Stage 2: Qualification
        match self
            .qualification_stage
            .process(&disambiguated_sentence, context)
            .await
        {
            Ok(qualification_result) => {
                stages_completed.push(ProcessingStage::Qualification);
                info!(
                    "Qualification completed: {} verifiable parts found",
                    qualification_result.verifiable_parts.len()
                );
            }
            Err(e) => {
                let error = ProcessingError {
                    stage: ProcessingStage::Qualification,
                    error_type: "QualificationFailed".to_string(),
                    message: e.to_string(),
                    recoverable: true,
                };
                errors.push(error);
                warn!("Qualification failed, continuing: {}", e);
            }
        }

        // Stage 3: Decomposition
        match self
            .decomposition_stage
            .process(&disambiguated_sentence, context)
            .await
        {
            Ok(decomposition_result) => {
                atomic_claims = decomposition_result.atomic_claims;
                stages_completed.push(ProcessingStage::Decomposition);
                info!(
                    "Decomposition completed: {} atomic claims extracted",
                    atomic_claims.len()
                );
            }
            Err(e) => {
                let error = ProcessingError {
                    stage: ProcessingStage::Decomposition,
                    error_type: "DecompositionFailed".to_string(),
                    message: e.to_string(),
                    recoverable: true,
                };
                errors.push(error);
                warn!("Decomposition failed: {}", e);
            }
        }

        // Stage 4: Verification (evidence collection)
        if !atomic_claims.is_empty() {
            match self
                .verification_stage
                .process(&atomic_claims, context)
                .await
            {
                Ok(verification_result) => {
                    verification_evidence = verification_result.evidence;
                    stages_completed.push(ProcessingStage::Verification);
                    info!(
                        "Verification completed: {} evidence items collected",
                        verification_evidence.len()
                    );
                }
                Err(e) => {
                    let error = ProcessingError {
                        stage: ProcessingStage::Verification,
                        error_type: "VerificationFailed".to_string(),
                        message: e.to_string(),
                        recoverable: true,
                    };
                    errors.push(error);
                    warn!("Verification failed: {}", e);
                }
            }
        }

        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        // Capture lengths before moving
        let claims_count = atomic_claims.len() as u32;
        let evidence_count = verification_evidence.len() as u32;

        let result = ClaimExtractionResult {
            original_sentence: sentence.to_string(),
            disambiguated_sentence,
            atomic_claims,
            verification_evidence,
            processing_metadata: ProcessingMetadata {
                processing_time_ms,
                stages_completed,
                ambiguities_resolved: 0, // TODO: Track from disambiguation stage
                claims_extracted: claims_count,
                evidence_collected: evidence_count,
                errors,
            },
        };

        info!(
            "Claim extraction completed in {}ms with {} claims and {} evidence items",
            processing_time_ms,
            result.processing_metadata.claims_extracted,
            result.processing_metadata.evidence_collected
        );

        Ok(result)
    }
}

impl Default for ClaimExtractionAndVerificationProcessor {
    fn default() -> Self {
        Self::new()
    }
}
