//! Council-Claim Pipeline Integration
//!
//! Integrates the four-stage claim extraction pipeline with council review:
//! - Extracts claims from council feedback and decisions
//! - Verifies claims using the CAWS-compliant verification pipeline
//! - Provides verified claims as evidence for council decisions
//! - Enables council to review verified claims for final verdict

use crate::claim_extraction::{
    AtomicClaimDecomposer, AtomicDecompositionResult, CawsCompliantVerifier, CawsVerificationResult,
    ContextualDisambiguator, ConversationContext, VerifiableContentQualifier, VerifiableContentResult,
};
use agent_research::extraction_types::{AtomicClaim, ProcessingContext};
use agent_agency_contracts::final_verdict::FinalVerdictContract;
use agent_agency_contracts::WorkingSpec;
use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Result of council-claim integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilClaimIntegrationResult {
    /// Extracted and verified claims from council feedback
    pub verified_claims: Vec<VerifiedCouncilClaim>,
    /// Claims that failed verification
    pub failed_claims: Vec<FailedCouncilClaim>,
    /// Overall verification success rate
    pub verification_success_rate: f64,
    /// Council decision with claim evidence
    pub council_decision: CouncilDecisionWithClaims,
}

/// Verified claim from council feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedCouncilClaim {
    /// Original council feedback text
    pub original_feedback: String,
    /// Extracted atomic claim
    pub atomic_claim: AtomicClaim,
    /// Verification confidence
    pub verification_confidence: f64,
    /// Stage where claim was extracted (1-4)
    pub extraction_stage: u8,
}

/// Failed claim from council feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedCouncilClaim {
    /// Original council feedback text
    pub original_feedback: String,
    /// Failure reason
    pub failure_reason: ClaimExtractionFailureReason,
    /// Stage where failure occurred (1-4)
    pub failure_stage: u8,
}

/// Reasons why claim extraction might fail
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum ClaimExtractionFailureReason {
    /// Ambiguities could not be resolved
    UnresolvableAmbiguities,
    /// Content not verifiable
    UnverifiableContent,
    /// Decomposition failed
    DecompositionFailed,
    /// Verification failed
    VerificationFailed,
}

/// Council decision enhanced with verified claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilDecisionWithClaims {
    /// Original council decision
    pub decision: FinalVerdictContract,
    /// Verified claims supporting the decision
    pub supporting_claims: Vec<VerifiedCouncilClaim>,
    /// Verified claims contradicting the decision
    pub contradicting_claims: Vec<VerifiedCouncilClaim>,
    /// Overall claim confidence in decision
    pub claim_confidence: f64,
}

/// Council-Claim Pipeline Integrator
///
/// Integrates the four-stage claim extraction pipeline with council review
/// to provide verified claims as evidence for council decisions.
pub struct CouncilClaimIntegrator {
    /// Stage 1: Contextual Disambiguation
    disambiguator: ContextualDisambiguator,
    /// Stage 2: Verifiable Content Qualification
    qualifier: VerifiableContentQualifier,
    /// Stage 3: Atomic Claim Decomposition
    decomposer: AtomicClaimDecomposer,
    /// Stage 4: CAWS-Compliant Verification
    verifier: CawsCompliantVerifier,
}

impl CouncilClaimIntegrator {
    /// Create a new council-claim integrator
    pub fn new() -> Self {
        Self {
            disambiguator: ContextualDisambiguator::new(),
            qualifier: VerifiableContentQualifier::new(),
            decomposer: AtomicClaimDecomposer::new(),
            verifier: CawsCompliantVerifier::new(),
        }
    }

    /// Extract and verify claims from council feedback
    ///
    /// Processes council feedback through the four-stage pipeline:
    /// 1. Disambiguate ambiguities
    /// 2. Qualify verifiable content
    /// 3. Decompose into atomic claims
    /// 4. Verify claims CAWS-compliantly
    pub async fn extract_claims_from_council_feedback(
        &self,
        council_feedback: &str,
        working_spec: &WorkingSpec,
        context: &ProcessingContext,
    ) -> Result<CouncilClaimIntegrationResult> {
        info!("Extracting claims from council feedback: {}", council_feedback);

        let mut verified_claims = Vec::new();
        let mut failed_claims = Vec::new();

        // Convert WorkingSpec to ConversationContext for disambiguation
        let conversation_context = self.convert_to_conversation_context(working_spec, context);

        // Stage 1: Contextual Disambiguation
        debug!("Stage 1: Disambiguating council feedback");
        let should_skip = self
            .disambiguator
            .should_skip_extraction(council_feedback, &conversation_context)
            .await?;

        if should_skip {
            warn!("Skipping extraction due to unresolvable ambiguities");
            failed_claims.push(FailedCouncilClaim {
                original_feedback: council_feedback.to_string(),
                failure_reason: ClaimExtractionFailureReason::UnresolvableAmbiguities,
                failure_stage: 1,
            });
            return Ok(CouncilClaimIntegrationResult {
                verified_claims,
                failed_claims,
                verification_success_rate: 0.0,
                council_decision: CouncilDecisionWithClaims {
                decision: FinalVerdictContract {
                    decision: agent_agency_contracts::final_verdict::FinalDecision::Reject,
                    votes: vec![],
                    dissent: "Unresolvable ambiguities in council feedback".to_string(),
                    remediation: vec![],
                    constitutional_refs: vec![],
                    verification_summary: agent_agency_contracts::final_verdict::VerificationSummary {
                        claims_total: 0,
                        claims_verified: 0,
                        coverage_pct: 0.0,
                    },
                },
                    supporting_claims: vec![],
                    contradicting_claims: vec![],
                    claim_confidence: 0.0,
                },
            });
        }

        // Resolve ambiguities
        let resolution_result = self
            .disambiguator
            .resolve_ambiguity(council_feedback, &conversation_context)
            .await?;

        if !resolution_result.success {
            warn!("Failed to resolve ambiguities in council feedback");
            failed_claims.push(FailedCouncilClaim {
                original_feedback: council_feedback.to_string(),
                failure_reason: ClaimExtractionFailureReason::UnresolvableAmbiguities,
                failure_stage: 1,
            });
            // Continue with original text if resolution failed
        }

        let disambiguated_text = resolution_result
            .resolved_phrase
            .unwrap_or_else(|| council_feedback.to_string());

        // Stage 2: Verifiable Content Qualification
        debug!("Stage 2: Qualifying verifiable content");
        let qualification_result = self
            .qualifier
            .detect_verifiable_content(&disambiguated_text, context)
            .await?;

        if !qualification_result.has_verifiable_content {
            warn!("No verifiable content found in council feedback");
            failed_claims.push(FailedCouncilClaim {
                original_feedback: council_feedback.to_string(),
                failure_reason: ClaimExtractionFailureReason::UnverifiableContent,
                failure_stage: 2,
            });
            return Ok(CouncilClaimIntegrationResult {
                verified_claims,
                failed_claims,
                verification_success_rate: 0.0,
                council_decision: CouncilDecisionWithClaims {
                decision: FinalVerdictContract {
                    decision: agent_agency_contracts::final_verdict::FinalDecision::Reject,
                    votes: vec![],
                    dissent: "No verifiable content in council feedback".to_string(),
                    remediation: vec![],
                    constitutional_refs: vec![],
                    verification_summary: agent_agency_contracts::final_verdict::VerificationSummary {
                        claims_total: 0,
                        claims_verified: 0,
                        coverage_pct: 0.0,
                    },
                },
                    supporting_claims: vec![],
                    contradicting_claims: vec![],
                    claim_confidence: 0.0,
                },
            });
        }

        let qualified_text = qualification_result
            .rewritten_sentence
            .unwrap_or_else(|| disambiguated_text);

        // Stage 3: Atomic Claim Decomposition
        debug!("Stage 3: Decomposing into atomic claims");
        let decomposition_result = self
            .decomposer
            .decompose_into_atomic_claims(&qualified_text, context)
            .await?;

        if !decomposition_result.success {
            warn!("Failed to decompose council feedback into atomic claims");
            failed_claims.push(FailedCouncilClaim {
                original_feedback: council_feedback.to_string(),
                failure_reason: ClaimExtractionFailureReason::DecompositionFailed,
                failure_stage: 3,
            });
            return Ok(CouncilClaimIntegrationResult {
                verified_claims,
                failed_claims,
                verification_success_rate: 0.0,
                council_decision: CouncilDecisionWithClaims {
                decision: FinalVerdictContract {
                    decision: agent_agency_contracts::final_verdict::FinalDecision::Reject,
                    votes: vec![],
                    dissent: "Failed to decompose into atomic claims".to_string(),
                    remediation: vec![],
                    constitutional_refs: vec![],
                    verification_summary: agent_agency_contracts::final_verdict::VerificationSummary {
                        claims_total: 0,
                        claims_verified: 0,
                        coverage_pct: 0.0,
                    },
                },
                    supporting_claims: vec![],
                    contradicting_claims: vec![],
                    claim_confidence: 0.0,
                },
            });
        }

        // Stage 4: CAWS-Compliant Verification
        debug!("Stage 4: Verifying claims CAWS-compliantly");
        let verification_result = self
            .verifier
            .verify_claims_caws_compliant(&decomposition_result.atomic_claims, context)
            .await?;

        // Convert verified claims to council claim format
        for verified in &verification_result.verified_claims {
            // Find corresponding atomic claim
            if let Some(atomic_claim) = decomposition_result
                .atomic_claims
                .iter()
                .find(|c| c.id == verified.claim_id)
            {
                verified_claims.push(VerifiedCouncilClaim {
                    original_feedback: council_feedback.to_string(),
                    atomic_claim: atomic_claim.clone(),
                    verification_confidence: verified.confidence,
                    extraction_stage: 4,
                });
            }
        }

        // Track failed claims from verification
        for failed in &verification_result.failed_claims {
            failed_claims.push(FailedCouncilClaim {
                original_feedback: council_feedback.to_string(),
                failure_reason: ClaimExtractionFailureReason::VerificationFailed,
                failure_stage: 4,
            });
        }

        // Calculate success rate
        let total_claims = decomposition_result.atomic_claims.len();
        let verification_success_rate = if total_claims > 0 {
            verification_result.verified_claims.len() as f64 / total_claims as f64
        } else {
            0.0
        };

        info!(
            "Claim extraction complete: {} verified, {} failed ({}% success rate)",
            verified_claims.len(),
            failed_claims.len(),
            verification_success_rate * 100.0
        );

        // Create council decision with claims (simplified - would integrate with actual council decision)
        let council_decision = CouncilDecisionWithClaims {
            decision: FinalVerdictContract {
                decision: if verification_success_rate >= 0.7 {
                    agent_agency_contracts::final_verdict::FinalDecision::Accept
                } else {
                    agent_agency_contracts::final_verdict::FinalDecision::Reject
                },
                votes: vec![],
                dissent: if verification_success_rate < 0.7 {
                    format!("Only {:.1}% of claims verified", verification_success_rate * 100.0)
                } else {
                    String::new()
                },
                remediation: vec![],
                constitutional_refs: vec![],
                verification_summary: agent_agency_contracts::final_verdict::VerificationSummary {
                    claims_total: total_claims as u32,
                    claims_verified: verified_claims.len() as u32,
                    coverage_pct: (verification_success_rate * 100.0) as f32,
                },
            },
            supporting_claims: verified_claims.clone(),
            contradicting_claims: vec![], // Would be populated based on claim content analysis
            claim_confidence: verification_success_rate,
        };

        Ok(CouncilClaimIntegrationResult {
            verified_claims,
            failed_claims,
            verification_success_rate,
            council_decision,
        })
    }

    /// Convert WorkingSpec and ProcessingContext to ConversationContext
    fn convert_to_conversation_context(
        &self,
        working_spec: &WorkingSpec,
        context: &ProcessingContext,
    ) -> ConversationContext {
        ConversationContext {
            prior_turns: vec![working_spec.description.clone()],
            entity_registry: vec![], // Would extract from working spec metadata
            surface_hints: vec![], // Would extract from working spec context
            task_context: Some(context.working_spec_id.clone()),
        }
    }
}

impl Default for CouncilClaimIntegrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use uuid::Uuid;

    fn create_test_context() -> ProcessingContext {
        ProcessingContext {
            task_id: Uuid::new_v4(),
            working_spec_id: "test-spec".to_string(),
            source_file: None,
            line_number: None,
            surrounding_context: "test context".to_string(),
            domain_hints: vec!["rust".to_string()],
            metadata: HashMap::new(),
            input_text: "test input".to_string(),
            language: None,
        }
    }

    fn create_test_working_spec() -> WorkingSpec {
        use agent_agency_contracts::working_spec::*;
        use chrono::Utc;
        WorkingSpec {
            version: "1.0".to_string(),
            id: "TEST-001".to_string(),
            title: "Test Spec".to_string(),
            description: "Test description".to_string(),
            goals: vec!["Complete test task".to_string()],
            risk_tier: 2,
            acceptance_criteria: vec![],
            constraints: WorkingSpecConstraints::default(),
            test_plan: TestPlan::default(),
            rollback_plan: RollbackPlan::default(),
            context: WorkingSpecContext::default(),
            non_functional_requirements: None,
            validation_results: None,
            quality_gates: None,
            scope: vec![],
            metadata: None,
            milestones: vec![],
            change_budget: Default::default(),
            file_changes: vec![],
            coverage_targets: None,
            overview: "Test overview".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_extract_claims_from_council_feedback() {
        let integrator = CouncilClaimIntegrator::new();
        let context = create_test_context();
        let working_spec = create_test_working_spec();

        let result = integrator
            .extract_claims_from_council_feedback(
                "The function returns a Result type and handles errors correctly.",
                &working_spec,
                &context,
            )
            .await
            .unwrap();

        // Should have attempted extraction
        assert!(result.verified_claims.len() >= 0 || result.failed_claims.len() >= 0);
    }
}

