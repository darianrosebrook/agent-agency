//! Comprehensive unit tests for Multi-Modal Verification Engine
//!
//! Tests all verification components and integration scenarios

#[cfg(test)]
mod tests {
    use crate::multi_modal_verification::{
        MultiModalVerificationEngine,
    };
    use crate::types::*;
    use chrono::Utc;
    use uuid::Uuid;

    /// Create a test atomic claim for testing
    fn create_test_claim(claim_text: &str, claim_type: ClaimType) -> AtomicClaim {
        AtomicClaim {
            id: Uuid::new_v4(),
            claim_text: claim_text.to_string(),
            claim_type,
            verifiability: VerifiabilityLevel::DirectlyVerifiable,
            scope: ClaimScope {
                working_spec_id: "test-spec-001".to_string(),
                component_boundaries: vec!["test-component".to_string()],
                data_impact: DataImpact::ReadOnly,
            },
            confidence: 0.8,
            contextual_brackets: vec![],
        }
    }


    /// Test complete multi-modal verification engine
    #[tokio::test]
    async fn test_multi_modal_verification_engine() {
        let mut engine = MultiModalVerificationEngine::new();

        // Create multiple test claims
        let claims = vec![
            create_test_claim(
                "The algorithm has O(n log n) time complexity",
                ClaimType::Technical,
            ),
            create_test_claim(
                "The system requires database connectivity",
                ClaimType::Technical,
            ),
            create_test_claim(
                "User authentication provides secure access control",
                ClaimType::Security,
            ),
        ];

        let results = engine.verify_claims(&claims).await.unwrap();

        assert_eq!(results.verified_claims.len(), 3);

        for verified_claim in &results.verified_claims {
            assert!(verified_claim.overall_confidence > 0.0);
            assert!(verified_claim.overall_confidence <= 1.0);

            // Verify verification status is valid
            match verified_claim.verification_results {
                VerificationStatus::Verified => {
                    // Verification passed
                    assert!(verified_claim.overall_confidence >= 0.5);
                }
                VerificationStatus::Refuted => {
                    // Verification failed
                    assert!(verified_claim.overall_confidence < 0.5);
                }
                VerificationStatus::Pending => {
                    // Verification in progress
                }
                VerificationStatus::Error(_) => {
                    // Verification error occurred
                    assert!(verified_claim.overall_confidence < 0.3);
                }
            }
        }
    }

    /// Test overall confidence calculation
    #[tokio::test]
    async fn test_overall_confidence_calculation() {
        let mut engine = MultiModalVerificationEngine::new();

        let claim = create_test_claim(
            "Test claim for confidence calculation",
            ClaimType::Technical,
        );

        let results = engine.verify_claims(&vec![claim]).await.unwrap();
        let verified_claim = &results.verified_claims[0];

        // Overall confidence is calculated by our multi-modal verification engine
        assert!(verified_claim.overall_confidence >= 0.0);
        assert!(verified_claim.overall_confidence <= 1.0);
    }

    /// Test verification with different claim types
    #[tokio::test]
    async fn test_verification_with_different_claim_types() {
        let mut engine = MultiModalVerificationEngine::new();

        let claims = vec![
            create_test_claim("Factual statement", ClaimType::Factual),
            create_test_claim("Procedural instruction", ClaimType::Procedural),
            create_test_claim("Technical specification", ClaimType::Technical),
            create_test_claim("CAWS compliance requirement", ClaimType::Constitutional),
            create_test_claim("Performance metric", ClaimType::Performance),
            create_test_claim("Security requirement", ClaimType::Security),
        ];

        let results = engine.verify_claims(&claims).await.unwrap();

        assert_eq!(results.verified_claims.len(), 6);

        // All claims should be processed successfully
        for (i, verified_claim) in results.verified_claims.iter().enumerate() {
            assert!(verified_claim.overall_confidence > 0.0);
            assert!(verified_claim.verification_timestamp <= Utc::now());

            // Verify original claim is preserved
            assert_eq!(
                verified_claim.original_claim,
                format!(
                    "{}",
                    match i {
                        0 => "Factual statement",
                        1 => "Procedural instruction",
                        2 => "Technical specification",
                        3 => "CAWS compliance requirement",
                        4 => "Performance metric",
                        5 => "Security requirement",
                        _ => unreachable!(),
                    }
                )
            );
        }
    }

    /// Test verification results structure
    #[tokio::test]
    async fn test_verification_results_structure() {
        let mut engine = MultiModalVerificationEngine::new();

        let claim = create_test_claim(
            "The function processes data efficiently",
            ClaimType::Technical,
        );

        let results = engine.verify_claims(&vec![claim]).await.unwrap();
        let verified_claim = &results.verified_claims[0];

        // Test that verification status is valid
        match verified_claim.verification_results {
            VerificationStatus::Verified => {
                assert!(verified_claim.overall_confidence >= 0.5);
            }
            VerificationStatus::Refuted => {
                assert!(verified_claim.overall_confidence < 0.5);
            }
            VerificationStatus::Pending => {
                // Verification in progress
            }
            VerificationStatus::Error(_) => {
                assert!(verified_claim.overall_confidence < 0.3);
            }
        }

        // Test overall confidence bounds
        assert!(verified_claim.overall_confidence >= 0.0);
        assert!(verified_claim.overall_confidence <= 1.0);

    }

    /// Test edge cases and error handling
    #[tokio::test]
    async fn test_edge_cases_and_error_handling() {
        let mut engine = MultiModalVerificationEngine::new();

        // Test with empty claims list
        let empty_results = engine.verify_claims(&vec![]).await.unwrap();
        assert_eq!(empty_results.verified_claims.len(), 0);

        // Test with very long claim text
        let long_claim = create_test_claim(&"A".repeat(1000), ClaimType::Technical);

        let results = engine.verify_claims(&vec![long_claim]).await.unwrap();
        assert_eq!(results.verified_claims.len(), 1);
        assert!(results.verified_claims[0].overall_confidence > 0.0);

        // Test with special characters
        let special_claim = create_test_claim(
            "Claim with special chars: !@#$%^&*()_+-=[]{}|;':\",./<>?",
            ClaimType::Technical,
        );

        let results = engine.verify_claims(&vec![special_claim]).await.unwrap();
        assert_eq!(results.verified_claims.len(), 1);
        assert!(results.verified_claims[0].overall_confidence > 0.0);
    }

}
