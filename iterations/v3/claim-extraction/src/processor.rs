use crate::decomposition::DecompositionStage;
use crate::disambiguation::DisambiguationStage;
use crate::multi_modal_verification::{MultiModalVerificationEngine, VerifiedClaim};
use crate::qualification::QualificationStage;
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
    pub async fn run(
        &self,
        input: &str,
        ctx: &ProcessingContext,
    ) -> Result<ClaimExtractionResult, ClaimExtractionError> {
        let start_time = Instant::now();
        info!("Starting claim extraction for input: {}", input);

        // Stage 1: Disambiguation
        debug!("Stage 1: Disambiguation");
        let disambiguation_result = self
            .disambiguation_stage
            .process(input, ctx)
            .await
            .map_err(|e| ClaimExtractionError::DisambiguationFailed(e.to_string()))?;

        // Stage 2: Qualification
        debug!("Stage 2: Qualification");
        let qualification_result = self
            .qualification_stage
            .process(&disambiguation_result.disambiguated_sentence, ctx)
            .await
            .map_err(|e| ClaimExtractionError::QualificationFailed(e.to_string()))?;

        // Stage 3: Decomposition
        debug!("Stage 3: Decomposition");
        // Process each verifiable part through decomposition
        let mut decomposition_results = Vec::new();
        for verifiable_part in &qualification_result.verifiable_parts {
            let result = self
                .decomposition_stage
                .process(&verifiable_part.content, ctx)
                .await
                .map_err(|e| ClaimExtractionError::DecompositionFailed(e.to_string()))?;
            decomposition_results.push(result);
        }

        // Collect all atomic claims from decomposition results
        let mut all_atomic_claims = Vec::new();
        for result in &decomposition_results {
            all_atomic_claims.extend(result.atomic_claims.clone());
        }

        // Stage 4: Traditional Verification (for evidence collection)
        debug!("Stage 4: Traditional Verification");
        let verification_result = self
            .verification_stage
            .process(&all_atomic_claims, ctx)
            .await
            .map_err(|e| ClaimExtractionError::VerificationFailed(e.to_string()))?;

        // Stage 5: Multi-Modal Verification (V3's superior verification)
        debug!("Stage 5: Multi-Modal Verification");
        let verified_claims = self
            .multi_modal_verifier
            .verify_claims(all_atomic_claims.clone())
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
        for verified_claim in &verified_claims {
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

        let claims_count = all_atomic_claims.len();
        let evidence_count = all_evidence.len();

        Ok(ClaimExtractionResult {
            original_sentence: input.to_string(),
            disambiguated_sentence: disambiguation_result.disambiguated_sentence,
            atomic_claims: all_atomic_claims,
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
                errors: Vec::new(),
            },
        })
    }

    /// Create mathematical evidence from verification results
    fn create_mathematical_evidence(&self, verified_claim: &VerifiedClaim) -> Option<Evidence> {
        let math_results = &verified_claim.verification_results.mathematical;
        if math_results.is_valid && !math_results.mathematical_claims.is_empty() {
            Some(Evidence {
                id: uuid::Uuid::new_v4(),
                claim_id: verified_claim.original_claim.id,
                evidence_type: EvidenceType::CodeAnalysis, // Mathematical analysis
                content: format!(
                    "Mathematical verification: {} claims validated with confidence {:.2}",
                    math_results.mathematical_claims.len(),
                    math_results.confidence
                ),
                source: EvidenceSource {
                    source_type: SourceType::ResearchAgent,
                    location: "MultiModalVerificationEngine".to_string(),
                    authority: "MathematicalValidator".to_string(),
                    freshness: chrono::Utc::now(),
                },
                confidence: math_results.confidence as f64,
                timestamp: chrono::Utc::now(),
            })
        } else {
            None
        }
    }

    /// Create code behavior evidence from verification results
    fn create_code_behavior_evidence(&self, verified_claim: &VerifiedClaim) -> Option<Evidence> {
        let code_results = &verified_claim.verification_results.code_behavior;
        if code_results.behavior_predicted && code_results.ast_analysis.syntax_valid {
            Some(Evidence {
                id: uuid::Uuid::new_v4(),
                claim_id: verified_claim.original_claim.id,
                evidence_type: EvidenceType::CodeAnalysis,
                content: format!(
                    "Code behavior analysis: AST parsed successfully, complexity score {:.2}",
                    code_results.ast_analysis.complexity_score
                ),
                source: EvidenceSource {
                    source_type: SourceType::ResearchAgent,
                    location: "MultiModalVerificationEngine".to_string(),
                    authority: "CodeBehaviorAnalyzer".to_string(),
                    freshness: chrono::Utc::now(),
                },
                confidence: code_results.confidence as f64,
                timestamp: chrono::Utc::now(),
            })
        } else {
            None
        }
    }

    /// Create semantic evidence from verification results
    fn create_semantic_evidence(&self, verified_claim: &VerifiedClaim) -> Option<Evidence> {
        let semantic_results = &verified_claim.verification_results.semantic;
        if semantic_results.semantic_valid
            && !semantic_results
                .meaning_extracted
                .primary_meaning
                .is_empty()
        {
            Some(Evidence {
                id: uuid::Uuid::new_v4(),
                claim_id: verified_claim.original_claim.id,
                evidence_type: EvidenceType::ResearchFindings,
                content: format!(
                    "Semantic analysis: {} entities identified, intent: {:?}",
                    semantic_results.meaning_extracted.semantic_entities.len(),
                    semantic_results.intent_analysis.primary_intent
                ),
                source: EvidenceSource {
                    source_type: SourceType::ResearchAgent,
                    location: "MultiModalVerificationEngine".to_string(),
                    authority: "SemanticAnalyzer".to_string(),
                    freshness: chrono::Utc::now(),
                },
                confidence: semantic_results.confidence as f64,
                timestamp: chrono::Utc::now(),
            })
        } else {
            None
        }
    }
}
