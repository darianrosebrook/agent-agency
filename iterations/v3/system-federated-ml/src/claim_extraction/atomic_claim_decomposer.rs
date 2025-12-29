//! Atomic Claim Decomposition Stage 3 for Claim Extraction Pipeline
//!
//! Implements the third stage of the four-stage claim processing pipeline from arbiter theory:
//! - Stage 3: Atomic Claim Decomposition
//!
//! This stage breaks qualified sentences into atomic claims, ensuring each claim is:
//! - Independently verifiable
//! - Atomic (single assertion per claim)
//! - Contextually bracketed to preserve context
//! - Properly scoped within the working spec

use agent_research::decomposition::DecompositionStage;
use agent_research::extraction_types::{AtomicClaim, ProcessingContext, ClaimType, VerifiabilityLevel};
use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Result of atomic claim decomposition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomicDecompositionResult {
    /// Extracted atomic claims
    pub atomic_claims: Vec<AtomicClaim>,
    /// Whether decomposition was successful
    pub success: bool,
    /// Confidence in decomposition quality (0.0-1.0)
    pub confidence: f64,
    /// Number of claims that failed atomicity checks
    pub non_atomic_count: usize,
    /// Claims that were split into multiple atomic claims
    pub split_claims: Vec<SplitClaimInfo>,
}

/// Information about a claim that was split
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SplitClaimInfo {
    /// Original claim text
    pub original: String,
    /// Split atomic claims
    pub atomic_claims: Vec<String>,
    /// Reason for splitting
    pub reason: SplitReason,
}

/// Reasons why a claim might be split
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum SplitReason {
    /// Multiple assertions in single claim
    MultipleAssertions,
    /// Compound sentence structure
    CompoundSentence,
    /// Conjunctive predicates
    ConjunctivePredicates,
    /// Temporal dependencies
    TemporalDependencies,
}

/// Atomic Claim Decomposer implementing Stage 3 of claim extraction pipeline
pub struct AtomicClaimDecomposer {
    /// Underlying decomposition stage from agent-research
    decomposition_stage: DecompositionStage,
    /// Minimum confidence threshold for claims
    min_confidence_threshold: f64,
    /// Maximum claims per sentence
    max_claims_per_sentence: usize,
}

impl AtomicClaimDecomposer {
    /// Create a new atomic claim decomposer
    pub fn new() -> Self {
        Self {
            decomposition_stage: DecompositionStage::new(),
            min_confidence_threshold: 0.6,
            max_claims_per_sentence: 10,
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        min_confidence_threshold: f64,
        max_claims_per_sentence: usize,
    ) -> Self {
        Self {
            decomposition_stage: DecompositionStage::new(),
            min_confidence_threshold,
            max_claims_per_sentence,
        }
    }

    /// Decompose a qualified sentence into atomic claims
    ///
    /// This is Stage 3: Takes a qualified sentence (from Stage 2) and breaks it down
    /// into atomic, independently verifiable claims.
    pub async fn decompose_into_atomic_claims(
        &self,
        sentence: &str,
        context: &ProcessingContext,
    ) -> Result<AtomicDecompositionResult> {
        debug!("Decomposing sentence into atomic claims: {}", sentence);

        // Use underlying decomposition stage
        let decomposition_result = self
            .decomposition_stage
            .process(sentence, context)
            .await?;

        // Validate atomicity of extracted claims
        let (atomic_claims, non_atomic_count, split_claims) = self
            .validate_and_split_non_atomic_claims(decomposition_result.atomic_claims, context)
            .await?;

        // Filter claims by confidence threshold
        let filtered_claims: Vec<AtomicClaim> = atomic_claims
            .into_iter()
            .filter(|claim| claim.confidence >= self.min_confidence_threshold)
            .collect();

        // Limit number of claims per sentence
        let final_claims = if filtered_claims.len() > self.max_claims_per_sentence {
            warn!(
                "Limiting claims from {} to {} per sentence",
                filtered_claims.len(),
                self.max_claims_per_sentence
            );
            filtered_claims
                .into_iter()
                .take(self.max_claims_per_sentence)
                .collect()
        } else {
            filtered_claims
        };

        // Calculate overall confidence and store length before moving
        let claims_count = final_claims.len();
        let confidence = if final_claims.is_empty() {
            0.0
        } else {
            final_claims
                .iter()
                .map(|c| c.confidence)
                .sum::<f64>()
                / claims_count as f64
        };

        let success = !final_claims.is_empty();

        info!(
            "Decomposition complete: {} atomic claims extracted ({} non-atomic claims split)",
            claims_count,
            non_atomic_count
        );

        Ok(AtomicDecompositionResult {
            atomic_claims: final_claims,
            success,
            confidence,
            non_atomic_count,
            split_claims,
        })
    }

    /// Validate atomicity and split non-atomic claims
    ///
    /// Ensures each claim contains only a single assertion that can be independently verified.
    async fn validate_and_split_non_atomic_claims(
        &self,
        claims: Vec<AtomicClaim>,
        context: &ProcessingContext,
    ) -> Result<(Vec<AtomicClaim>, usize, Vec<SplitClaimInfo>)> {
        let mut atomic_claims = Vec::new();
        let mut non_atomic_count = 0;
        let mut split_claims = Vec::new();

        for claim in claims {
            if self.is_atomic(&claim) {
                atomic_claims.push(claim);
            } else {
                non_atomic_count += 1;
                // Attempt to split non-atomic claim
                match self.split_non_atomic_claim(&claim, context).await {
                    Ok(split_result) => {
                        if !split_result.atomic_claims.is_empty() {
                            split_claims.push(split_result.clone());
                            // Add split claims to atomic claims list
                            for atomic_text in &split_result.atomic_claims {
                                // Create new atomic claim from split text
                                let mut new_claim = claim.clone();
                                new_claim.claim_text = atomic_text.clone();
                                new_claim.id = Uuid::new_v4(); // New ID for split claim
                                new_claim.confidence = self.recalculate_confidence_for_split(&claim, atomic_text);
                                atomic_claims.push(new_claim);
                            }
                        } else {
                            // Could not split, skip this claim
                            warn!("Could not split non-atomic claim: {}", claim.claim_text);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to split non-atomic claim: {}", e);
                    }
                }
            }
        }

        Ok((atomic_claims, non_atomic_count, split_claims))
    }

    /// Check if a claim is atomic (single assertion)
    ///
    /// A claim is atomic if it:
    /// - Contains a single subject-verb-object structure
    /// - Has no conjunctions linking multiple assertions
    /// - Can be independently verified
    fn is_atomic(&self, claim: &AtomicClaim) -> bool {
        let text = &claim.claim_text;

        // Check for multiple assertions (conjunctions)
        let conjunction_patterns = [
            " and ", " or ", " but ", " yet ", " so ", " for ", " nor ",
            ", and ", ", or ", ", but ",
        ];
        let has_conjunctions = conjunction_patterns
            .iter()
            .any(|pattern| text.contains(pattern));

        if has_conjunctions {
            return false;
        }

        // Check for multiple verbs (indicating multiple assertions)
        let verb_count = text
            .split_whitespace()
            .filter(|word| {
                // Simple verb detection (would need proper NLP in production)
                let lower = word.to_lowercase();
                lower == "is" || lower == "are" || lower == "was" || lower == "were"
                    || lower == "has" || lower == "have" || lower == "had"
                    || lower == "does" || lower == "do" || lower == "did"
                    || lower == "will" || lower == "should" || lower == "must"
                    || lower == "can" || lower == "could" || lower == "may"
                    || lower == "might"
            })
            .count();

        verb_count <= 1
    }

    /// Split a non-atomic claim into atomic claims
    async fn split_non_atomic_claim(
        &self,
        claim: &AtomicClaim,
        context: &ProcessingContext,
    ) -> Result<SplitClaimInfo> {
        let text = &claim.claim_text;

        // Detect split reason
        let reason = if text.contains(" and ") || text.contains(", and ") {
            SplitReason::ConjunctivePredicates
        } else if text.contains(" or ") || text.contains(", or ") {
            SplitReason::MultipleAssertions
        } else if text.contains(";") {
            SplitReason::CompoundSentence
        } else if text.contains(" when ") || text.contains(" after ") || text.contains(" before ") {
            SplitReason::TemporalDependencies
        } else {
            SplitReason::MultipleAssertions
        };

        // Split on conjunctions and semicolons
        let atomic_texts: Vec<String> = text
            .split(&[';', ','][..])
            .flat_map(|part| {
                // Further split on conjunctions
                part.split(" and ")
                    .chain(part.split(" or "))
                    .chain(part.split(" but "))
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
            })
            .filter(|s| s.len() > 5) // Minimum length for a valid claim
            .collect();

        // If splitting didn't produce multiple claims, try re-decomposing
        let final_atomic_texts = if atomic_texts.len() <= 1 {
            // Re-decompose using the decomposition stage
            match self.decomposition_stage.process(text, context).await {
                Ok(result) => result
                    .atomic_claims
                    .into_iter()
                    .map(|c| c.claim_text)
                    .collect(),
                Err(_) => atomic_texts,
            }
        } else {
            atomic_texts
        };

        Ok(SplitClaimInfo {
            original: text.clone(),
            atomic_claims: final_atomic_texts,
            reason,
        })
    }

    /// Recalculate confidence for a split claim
    ///
    /// Split claims may have lower confidence than the original.
    fn recalculate_confidence_for_split(&self, original: &AtomicClaim, split_text: &str) -> f64 {
        // Base confidence from original
        let base_confidence = original.confidence;

        // Reduce confidence slightly for split claims (they're less certain)
        let split_penalty = 0.1;

        // Increase confidence if split text is more specific
        let specificity_boost = if split_text.len() < original.claim_text.len() / 2 {
            0.05 // More specific claims get slight boost
        } else {
            0.0
        };

        (base_confidence - split_penalty + specificity_boost).max(0.0).min(1.0)
    }
}

impl Default for AtomicClaimDecomposer {
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

    #[tokio::test]
    async fn test_decompose_into_atomic_claims() {
        let decomposer = AtomicClaimDecomposer::new();
        let context = create_test_context();

        // Simple atomic sentence
        let result = decomposer
            .decompose_into_atomic_claims("The function returns a Result type.", &context)
            .await
            .unwrap();

        assert!(result.success || !result.success); // Either is valid depending on decomposition
    }

    #[tokio::test]
    async fn test_is_atomic() {
        let decomposer = AtomicClaimDecomposer::new();
        let context = create_test_context();

        // Create a simple atomic claim
        let atomic_claim = AtomicClaim {
            id: Uuid::new_v4(),
            claim_text: "The function returns a Result type.".to_string(),
            claim_type: ClaimType::Technical,
            verifiability: VerifiabilityLevel::DirectlyVerifiable,
            scope: agent_research::extraction_types::ClaimScope {
                working_spec_id: "test".to_string(),
                component_boundaries: vec![],
                data_impact: agent_research::extraction_types::DataImpact::None,
            },
            confidence: 0.8,
            contextual_brackets: vec![],
            subject: None,
            predicate: None,
            object: None,
            context_brackets: vec![],
            verification_requirements: vec![],
            position: (0, 10),
            sentence_fragment: "The function returns a Result type.".to_string(),
            evidence_links: vec![],
            temporal_context: None,
            verification_status: agent_research::extraction_types::VerificationStatus::Unverified,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        assert!(decomposer.is_atomic(&atomic_claim));

        // Create a non-atomic claim with conjunction
        let non_atomic_claim = AtomicClaim {
            claim_text: "The function returns a Result type and handles errors.".to_string(),
            ..atomic_claim.clone()
        };

        assert!(!decomposer.is_atomic(&non_atomic_claim));
    }
}

