//! Comprehensive unit tests for the Claim Extraction Pipeline
//!
//! Tests all stages: disambiguation, qualification, decomposition, and verification

use crate::decomposition::DecompositionStage;
use crate::disambiguation::DisambiguationStage;
use crate::qualification::QualificationStage;
use crate::types::*;
use crate::verification::VerificationStage;
use anyhow::Result;
use std::collections::HashMap;
use uuid::Uuid;

#[cfg(test)]
mod disambiguation_tests {
    use super::*;

    /// Test pronoun resolution in disambiguation stage
    #[tokio::test]
    async fn test_pronoun_resolution() -> Result<()> {
        let disambiguation = DisambiguationStage::minimal();

        // Test case with ambiguous pronouns
        let sentence = "The system should handle user requests. It must be secure and reliable.";
        let context = ProcessingContext {
            task_id: Uuid::new_v4(),
            working_spec_id: "test_spec".to_string(),
            source_file: None,
            line_number: None,
            surrounding_context: "We need to secure the application".to_string(),
            domain_hints: vec!["authentication".to_string()],
            metadata: HashMap::new(),
            input_text: sentence.to_string(),
            language: Some(Language::English),
        };

        let result = disambiguation.process(sentence, &context).await?;

        // Validate pronoun resolution
        assert!(
            result.ambiguities_resolved > 0,
            "Should resolve ambiguous pronouns"
        );

        // Check that pronouns are resolved
        let resolved_text = result.disambiguated_sentence;
        assert!(
            !resolved_text.contains("It must"),
            "Pronouns should be resolved to specific entities"
        );
        assert!(
            resolved_text.contains("system must") || resolved_text.contains("application must"),
            "Pronouns should be resolved to specific entities"
        );

        Ok(())
    }

    /// Test technical term detection in disambiguation stage
    #[tokio::test]
    async fn test_technical_term_detection() -> Result<()> {
        let disambiguation = DisambiguationStage::minimal();

        // Test case with technical terms
        let technical_input = ClaimExtractionInput {
            id: Uuid::new_v4(),
            text: "The API should use JWT tokens for authentication and implement rate limiting."
                .to_string(),
            context: HashMap::from([
                ("domain".to_string(), "api_security".to_string()),
                ("tech_stack".to_string(), "rust, jwt, redis".to_string()),
            ]),
            metadata: HashMap::new(),
        };

        let result = disambiguation.process(&technical_input).await?;

        // Validate technical term detection
        assert!(
            !result.resolved_claims.is_empty(),
            "Should detect and resolve technical terms"
        );
        assert!(result.confidence > 0.0, "Should have positive confidence");

        // Check that technical terms are properly identified
        let resolved_text = result.disambiguated_sentence;
        assert!(
            resolved_text.contains("JWT") || resolved_text.contains("JSON Web Token"),
            "Technical terms should be properly identified"
        );
        assert!(
            resolved_text.contains("API")
                || resolved_text.contains("Application Programming Interface"),
            "Technical terms should be properly identified"
        );

        Ok(())
    }

    /// Test context-aware disambiguation
    #[tokio::test]
    async fn test_context_aware_disambiguation() -> Result<()> {
        let disambiguation = DisambiguationStage::minimal();

        // Test case with context-dependent ambiguity
        let context_input = ClaimExtractionInput {
            id: Uuid::new_v4(),
            text: "The service should be fast and handle concurrent requests efficiently."
                .to_string(),
            context: HashMap::from([
                ("domain".to_string(), "performance".to_string()),
                ("service_type".to_string(), "web_api".to_string()),
                (
                    "performance_requirements".to_string(),
                    "sub-100ms response time".to_string(),
                ),
            ]),
            metadata: HashMap::new(),
        };

        let result = disambiguation.process(&context_input).await?;

        // Validate context-aware disambiguation
        assert!(
            !result.resolved_claims.is_empty(),
            "Should use context for disambiguation"
        );
        assert!(result.confidence > 0.0, "Should have positive confidence");

        // Check that context is used for disambiguation
        let resolved_text = result.disambiguated_sentence;
        assert!(
            resolved_text.contains("web API") || resolved_text.contains("API service"),
            "Context should be used to resolve ambiguous terms"
        );

        Ok(())
    }
}

#[cfg(test)]
mod qualification_tests {
    use super::*;

    /// Test verifiability assessment in qualification stage
    #[tokio::test]
    async fn test_verifiability_assessment() -> Result<()> {
        let qualification = QualificationStage::new();

        // Test case with verifiable claims
        let verifiable_input = ClaimExtractionInput {
            id: Uuid::new_v4(),
            text: "The function should return a 200 status code when authentication succeeds."
                .to_string(),
            context: HashMap::from([
                ("domain".to_string(), "api_testing".to_string()),
                (
                    "verification_sources".to_string(),
                    "unit_tests,integration_tests".to_string(),
                ),
            ]),
            metadata: HashMap::new(),
        };

        let result = qualification.process(&verifiable_input).await?;

        // Validate verifiability assessment
        assert!(
            !result.verifiable_claims.is_empty(),
            "Should identify verifiable claims"
        );
        assert!(
            result.verifiable_claims.len() > 0,
            "Should have at least one verifiable claim"
        );

        // Check that claims are properly categorized
        for claim in &result.verifiable_claims {
            assert!(
                claim.verifiability_level == VerifiabilityLevel::DirectlyVerifiable
                    || claim.verifiability_level == VerifiabilityLevel::IndirectlyVerifiable,
                "Claims should be categorized as verifiable"
            );
        }

        Ok(())
    }

    /// Test content rewriting in qualification stage
    #[tokio::test]
    async fn test_content_rewriting() -> Result<()> {
        let qualification = QualificationStage::new();

        // Test case with unclear content
        let unclear_input = ClaimExtractionInput {
            id: Uuid::new_v4(),
            text: "The thing should work better and be more good.".to_string(),
            context: HashMap::from([
                ("domain".to_string(), "performance".to_string()),
                (
                    "rewrite_requirements".to_string(),
                    "clarity, specificity".to_string(),
                ),
            ]),
            metadata: HashMap::new(),
        };

        let result = qualification.process(&unclear_input).await?;

        // Validate content rewriting
        assert!(
            !result.rewritten_content.is_empty(),
            "Should rewrite unclear content"
        );

        // Check that content is improved
        let rewritten = &result.rewritten_content;
        assert!(
            !rewritten.contains("thing"),
            "Vague terms should be replaced"
        );
        assert!(
            !rewritten.contains("more good"),
            "Grammatical errors should be corrected"
        );
        assert!(
            rewritten.len() > unclear_input.text.len(),
            "Rewritten content should be more detailed"
        );

        Ok(())
    }

    /// Test mixed verifiability handling
    #[tokio::test]
    async fn test_mixed_verifiability_handling() -> Result<()> {
        let qualification = QualificationStage::new();

        // Test case with mixed verifiable and non-verifiable claims
        let mixed_input = ClaimExtractionInput {
            id: Uuid::new_v4(),
            text: "The API should return 200 status codes and provide a great user experience."
                .to_string(),
            context: HashMap::from([
                ("domain".to_string(), "api_design".to_string()),
                (
                    "verification_sources".to_string(),
                    "testing,monitoring".to_string(),
                ),
            ]),
            metadata: HashMap::new(),
        };

        let result = qualification.process(&mixed_input).await?;

        // Validate mixed verifiability handling
        assert!(
            !result.verifiable_claims.is_empty(),
            "Should identify verifiable claims"
        );
        assert!(
            !result.non_verifiable_claims.is_empty(),
            "Should identify non-verifiable claims"
        );

        // Check that claims are properly separated
        let total_claims = result.verifiable_claims.len() + result.non_verifiable_claims.len();
        assert!(total_claims > 0, "Should have claims in both categories");

        Ok(())
    }
}

#[cfg(test)]
mod decomposition_tests {
    use super::*;

    /// Test atomic claim extraction in decomposition stage
    #[tokio::test]
    async fn test_atomic_claim_extraction() -> Result<()> {
        let decomposition = DecompositionStage::new();

        // Test case with compound claims
        let compound_input = ClaimExtractionInput {
            id: Uuid::new_v4(),
            text: "The system should authenticate users securely, handle errors gracefully, and log all activities.".to_string(),
            context: HashMap::from([
                ("domain".to_string(), "security".to_string()),
                ("decomposition_requirements".to_string(), "atomic_claims".to_string()),
            ]),
            metadata: HashMap::new(),
        };

        let result = decomposition.process(&compound_input).await?;

        // Validate atomic claim extraction
        assert!(
            !result.atomic_claims.is_empty(),
            "Should extract atomic claims"
        );
        assert!(
            result.atomic_claims.len() >= 3,
            "Should extract multiple atomic claims from compound statement"
        );

        // Check that claims are atomic
        for claim in &result.atomic_claims {
            assert!(
                !claim.content.contains(" and "),
                "Atomic claims should not contain conjunctions"
            );
            assert!(
                !claim.content.contains(" or "),
                "Atomic claims should not contain disjunctions"
            );
            assert!(
                claim.content.len() < compound_input.text.len(),
                "Atomic claims should be smaller than original"
            );
        }

        Ok(())
    }

    /// Test context bracket handling in decomposition stage
    #[tokio::test]
    async fn test_context_bracket_handling() -> Result<()> {
        let decomposition = DecompositionStage::new();

        // Test case with context-dependent claims
        let context_input = ClaimExtractionInput {
            id: Uuid::new_v4(),
            text:
                "When a user logs in, the system should validate credentials and create a session."
                    .to_string(),
            context: HashMap::from([
                ("domain".to_string(), "authentication".to_string()),
                ("context_requirements".to_string(), "brackets".to_string()),
            ]),
            metadata: HashMap::new(),
        };

        let result = decomposition.process(&context_input).await?;

        // Validate context bracket handling
        assert!(
            !result.atomic_claims.is_empty(),
            "Should extract claims with context"
        );

        // Check that context is preserved
        for claim in &result.atomic_claims {
            assert!(
                claim.context_brackets.len() > 0,
                "Claims should have context brackets"
            );
            assert!(
                claim
                    .context_brackets
                    .iter()
                    .any(|b| b.contains("user logs in")),
                "Context brackets should preserve original context"
            );
        }

        Ok(())
    }

    /// Test complex sentence decomposition
    #[tokio::test]
    async fn test_complex_sentence_decomposition() -> Result<()> {
        let decomposition = DecompositionStage::new();

        // Test case with complex nested structure
        let complex_input = ClaimExtractionInput {
            id: Uuid::new_v4(),
            text: "If the user is authenticated and has the required permissions, then the system should process the request and return the appropriate response.".to_string(),
            context: HashMap::from([
                ("domain".to_string(), "authorization".to_string()),
                ("complexity".to_string(), "high".to_string()),
            ]),
            metadata: HashMap::new(),
        };

        let result = decomposition.process(&complex_input).await?;

        // Validate complex sentence decomposition
        assert!(
            !result.atomic_claims.is_empty(),
            "Should decompose complex sentences"
        );
        assert!(
            result.atomic_claims.len() >= 4,
            "Complex sentences should yield multiple atomic claims"
        );

        // Check that logical structure is preserved
        let has_condition = result
            .atomic_claims
            .iter()
            .any(|c| c.content.contains("authenticated"));
        let has_action = result
            .atomic_claims
            .iter()
            .any(|c| c.content.contains("process"));
        assert!(
            has_condition && has_action,
            "Logical structure should be preserved in decomposition"
        );

        Ok(())
    }
}

#[cfg(test)]
mod verification_tests {
    use super::*;

    /// Test evidence collection in verification stage
    #[tokio::test]
    async fn test_evidence_collection() -> Result<()> {
        let verification = VerificationStage::new();

        // Test case with atomic claims
        let atomic_claims = vec![AtomicClaim {
            id: Uuid::new_v4(),
            claim_text: "The API should return 200 status code for valid requests".to_string(),
            claim_type: ClaimType::Technical,
            verifiability: VerifiabilityLevel::DirectlyVerifiable,
            scope: ClaimScope {
                working_spec_id: "test".to_string(),
                component_boundaries: vec!["api".to_string()],
                data_impact: DataImpact::ReadOnly,
            },
            confidence: 0.8,
            contextual_brackets: vec!["API endpoint".to_string()],
            subject: None,
            predicate: None,
            object: None,
            context_brackets: vec!["API endpoint".to_string()],
            verification_requirements: vec![],
            position: (0, 0),
            sentence_fragment: "The API should return 200 status code for valid requests".to_string(),
        }];

        let result = verification.process(&atomic_claims).await?;

        // Validate evidence collection
        assert!(
            !result.verified_claims.is_empty(),
            "Should collect evidence for claims"
        );

        // Check that evidence is collected
        for verified_claim in &result.verified_claims {
            assert!(
                !verified_claim.evidence.is_empty(),
                "Verified claims should have evidence"
            );
            assert!(
                verified_claim.verification_status != VerificationStatus::Unverified,
                "Claims should be verified"
            );
        }

        Ok(())
    }

    /// Test council integration in verification stage
    #[tokio::test]
    async fn test_council_integration() -> Result<()> {
        let verification = VerificationStage::new();

        // Test case with claims requiring council evaluation
        let council_claims = vec![AtomicClaim {
            id: Uuid::new_v4(),
            claim_text: "The system should comply with security standards".to_string(),
            claim_type: ClaimType::Security,
            verifiability: VerifiabilityLevel::RequiresContext,
            scope: ClaimScope {
                working_spec_id: "security".to_string(),
                component_boundaries: vec!["system".to_string()],
                data_impact: DataImpact::ReadOnly,
            },
            confidence: 0.7,
            contextual_brackets: vec!["Security compliance".to_string()],
            subject: None,
            predicate: None,
            object: None,
            context_brackets: vec!["Security compliance".to_string()],
            verification_requirements: vec![],
            position: (0, 0),
            sentence_fragment: "The system should comply with security standards".to_string(),
        }];

        let result = verification.process(&council_claims).await?;

        // Validate council integration
        assert!(
            !result.verified_claims.is_empty(),
            "Should integrate with council for verification"
        );

        // Check that council evaluation is performed
        for verified_claim in &result.verified_claims {
            assert!(
                verified_claim.verification_status == VerificationStatus::Verified
                    || verified_claim.verification_status == VerificationStatus::Rejected,
                "Council should provide verification status"
            );
        }

        Ok(())
    }

    /// Test verification confidence scoring
    #[tokio::test]
    async fn test_verification_confidence_scoring() -> Result<()> {
        let verification = VerificationStage::new();

        // Test case with high-confidence claims
        let high_confidence_claims = vec![AtomicClaim {
            id: Uuid::new_v4(),
            claim_text: "The function should return a boolean value".to_string(),
            claim_type: ClaimType::Technical,
            verifiability: VerifiabilityLevel::DirectlyVerifiable,
            scope: ClaimScope {
                working_spec_id: "function_test".to_string(),
                component_boundaries: vec!["function".to_string()],
                data_impact: DataImpact::ReadOnly,
            },
            confidence: 0.95,
            contextual_brackets: vec!["Function signature".to_string()],
            subject: None,
            predicate: None,
            object: None,
            context_brackets: vec!["Function signature".to_string()],
            verification_requirements: vec![],
            position: (0, 0),
            sentence_fragment: "The function should return a boolean value".to_string(),
        }];

        let result = verification.process(&high_confidence_claims).await?;

        // Validate verification confidence scoring
        assert!(
            !result.verified_claims.is_empty(),
            "Should score verification confidence"
        );

        // Check that confidence is properly calculated
        for verified_claim in &result.verified_claims {
            assert!(
                verified_claim.confidence >= 0.0 && verified_claim.confidence <= 1.0,
                "Verification confidence should be between 0 and 1"
            );
            assert!(
                verified_claim.confidence > 0.8,
                "High-confidence claims should maintain high verification confidence"
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod pipeline_integration_tests {
    use super::*;

    /// Test end-to-end pipeline processing
    #[tokio::test]
    async fn test_end_to_end_pipeline_processing() -> Result<()> {
        // Create a complete pipeline
        let disambiguation = DisambiguationStage::minimal();
        let qualification = QualificationStage::new();
        let decomposition = DecompositionStage::new();
        let verification = VerificationStage::new();

        // Test case for end-to-end processing
        let input = ClaimExtractionInput {
            id: Uuid::new_v4(),
            text: "The authentication system should validate user credentials securely and handle failed attempts appropriately.".to_string(),
            context: HashMap::from([
                ("domain".to_string(), "authentication".to_string()),
                ("security_requirements".to_string(), "high".to_string()),
            ]),
            metadata: HashMap::new(),
        };

        // Stage 1: Disambiguation
        let disambiguation_result = disambiguation.process(&input).await?;
        assert!(
            !disambiguation_result.resolved_claims.is_empty(),
            "Disambiguation should resolve claims"
        );

        // Stage 2: Qualification
        let qualification_input = ClaimExtractionInput {
            id: input.id,
            text: disambiguation_result.disambiguated_sentence,
            context: input.context.clone(),
            metadata: input.metadata.clone(),
        };
        let qualification_result = qualification.process(&qualification_input).await?;
        assert!(
            !qualification_result.verifiable_claims.is_empty(),
            "Qualification should identify verifiable claims"
        );

        // Stage 3: Decomposition
        let decomposition_input = ClaimExtractionInput {
            id: input.id,
            text: qualification_result.rewritten_content,
            context: input.context.clone(),
            metadata: input.metadata.clone(),
        };
        let decomposition_result = decomposition.process(&decomposition_input).await?;
        assert!(
            !decomposition_result.atomic_claims.is_empty(),
            "Decomposition should extract atomic claims"
        );

        // Stage 4: Verification
        let verification_result = verification
            .process(&decomposition_result.atomic_claims)
            .await?;
        assert!(
            !verification_result.verified_claims.is_empty(),
            "Verification should verify claims"
        );

        // Validate end-to-end processing
        assert!(
            verification_result.overall_confidence > 0.0,
            "End-to-end processing should have positive confidence"
        );
        assert!(
            !verification_result.verified_claims.is_empty(),
            "Should have verified claims at the end"
        );

        Ok(())
    }

    /// Test error handling and recovery
    #[tokio::test]
    async fn test_error_handling_and_recovery() -> Result<()> {
        let disambiguation = DisambiguationStage::minimal();

        // Test case with malformed input
        let malformed_input = ClaimExtractionInput {
            id: Uuid::new_v4(),
            text: "".to_string(), // Empty text should cause error
            context: HashMap::new(),
            metadata: HashMap::new(),
        };

        // Should handle error gracefully
        let result = disambiguation.process(&malformed_input).await;

        // Validate error handling
        match result {
            Ok(_) => {
                // If it succeeds, it should handle empty input gracefully
                let success_result = result.unwrap();
                assert!(
                    success_result.resolved_claims.is_empty() || success_result.confidence == 0.0,
                    "Empty input should result in no claims or zero confidence"
                );
            }
            Err(_) => {
                // Error handling is acceptable for malformed input
                // The important thing is that it doesn't panic
            }
        }

        Ok(())
    }

    /// Test metadata tracking validation
    #[tokio::test]
    async fn test_metadata_tracking_validation() -> Result<()> {
        let disambiguation = DisambiguationStage::minimal();

        // Test case with metadata
        let input = ClaimExtractionInput {
            id: Uuid::new_v4(),
            text: "The system should be reliable".to_string(),
            context: HashMap::from([("domain".to_string(), "reliability".to_string())]),
            metadata: HashMap::from([
                ("source".to_string(), "test".to_string()),
                ("priority".to_string(), "high".to_string()),
            ]),
        };

        let result = disambiguation.process(&input).await?;

        // Validate metadata tracking
        assert!(
            !result.processing_metadata.stages_completed.is_empty(),
            "Should track completed stages"
        );
        assert!(
            result.processing_metadata.processing_time_ms > 0,
            "Should track processing time"
        );
        assert!(
            result.processing_metadata.claims_extracted > 0,
            "Should track extracted claims count"
        );

        Ok(())
    }
}
