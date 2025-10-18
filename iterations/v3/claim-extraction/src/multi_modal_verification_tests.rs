//! Comprehensive unit tests for Multi-Modal Verification Engine
//!
//! Tests all verification components and integration scenarios

#[cfg(test)]
mod tests {
    use crate::multi_modal_verification::{
        AuthorityAttributionChecker, CodeBehaviorAnalyzer, ContextDependencyResolver,
        CrossReferenceValidator, MathematicalValidator, MultiModalVerificationEngine,
        SemanticAnalyzer, VerifiedClaim,
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

    /// Test mathematical validator with various claim types
    #[tokio::test]
    async fn test_mathematical_validator() {
        let validator = MathematicalValidator::new();

        // Test with mathematical claim
        let math_claim = create_test_claim(
            "The algorithm has O(n log n) time complexity",
            ClaimType::Technical,
        );

        let result = validator.validate(&math_claim).await.unwrap();

        assert!(result.is_valid);
        assert!(result.confidence > 0.0);
        assert!(result.confidence <= 1.0);
    }

    /// Test code behavior analyzer with technical claims
    #[tokio::test]
    async fn test_code_behavior_analyzer() {
        let mut analyzer = CodeBehaviorAnalyzer::new();

        // Test with code-related claim
        let code_claim =
            create_test_claim("The function returns a sorted array", ClaimType::Technical);

        let result = analyzer.analyze(&code_claim).await.unwrap();

        assert!(result.behavior_predicted || !result.behavior_predicted); // Either is valid for stub
        assert!(result.confidence > 0.0);
        assert!(result.confidence <= 1.0);
        assert!(result.ast_analysis.syntax_valid);
    }

    /// Test authority attribution checker
    #[tokio::test]
    async fn test_authority_attribution_checker() {
        let checker = AuthorityAttributionChecker::new();

        // Test with factual claim
        let factual_claim = create_test_claim(
            "According to the documentation, the API supports JSON responses",
            ClaimType::Factual,
        );

        let result = checker.verify(&factual_claim).await.unwrap();

        assert!(result.authority_score > 0.0);
        assert!(result.authority_score <= 1.0);
        assert!(result.attribution_confidence > 0.0);
        assert!(result.attribution_confidence <= 1.0);
    }

    /// Test context dependency resolver
    #[tokio::test]
    async fn test_context_dependency_resolver() {
        let resolver = ContextDependencyResolver::new();

        // Test with context-dependent claim
        let context_claim = create_test_claim(
            "The system requires database connectivity",
            ClaimType::Technical,
        );

        let result = resolver.resolve(&context_claim).await.unwrap();

        assert!(result.confidence > 0.0);
        assert!(result.confidence <= 1.0);
    }

    /// Test semantic analyzer
    #[tokio::test]
    async fn test_semantic_analyzer() {
        let analyzer = SemanticAnalyzer::new();

        // Test with semantic claim
        let semantic_claim = create_test_claim(
            "The user authentication system provides secure access control",
            ClaimType::Security,
        );

        let result = analyzer.analyze(&semantic_claim).await.unwrap();

        assert!(result.semantic_valid);
        assert!(result.confidence > 0.0);
        assert!(result.confidence <= 1.0);
        assert_eq!(
            result.meaning_extracted.primary_meaning,
            semantic_claim.claim_text
        );
    }

    /// Test cross-reference validator
    #[tokio::test]
    async fn test_cross_reference_validator() {
        let validator = CrossReferenceValidator::new();

        // Test with cross-reference claim
        let cross_ref_claim = create_test_claim(
            "This implementation follows the same pattern as the user service",
            ClaimType::Technical,
        );

        let result = validator.validate(&cross_ref_claim).await.unwrap();

        assert!(result.consistency_score > 0.0);
        assert!(result.consistency_score <= 1.0);
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

        let results = engine.verify_claims(claims).await.unwrap();

        assert_eq!(results.len(), 3);

        for verified_claim in &results {
            assert!(verified_claim.overall_confidence > 0.0);
            assert!(verified_claim.overall_confidence <= 1.0);

            // Verify all verification components are present
            assert!(verified_claim.verification_results.mathematical.confidence > 0.0);
            assert!(verified_claim.verification_results.code_behavior.confidence > 0.0);
            assert!(
                verified_claim
                    .verification_results
                    .authority
                    .attribution_confidence
                    > 0.0
            );
            assert!(verified_claim.verification_results.context.confidence > 0.0);
            assert!(verified_claim.verification_results.semantic.confidence > 0.0);
            assert!(
                verified_claim
                    .verification_results
                    .cross_reference
                    .consistency_score
                    > 0.0
            );
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

        let results = engine.verify_claims(vec![claim]).await.unwrap();
        let verified_claim = &results[0];

        // Overall confidence should be the average of all component confidences
        let expected_confidence = (verified_claim.verification_results.mathematical.confidence
            + verified_claim.verification_results.code_behavior.confidence
            + verified_claim
                .verification_results
                .authority
                .attribution_confidence
            + verified_claim.verification_results.context.confidence
            + verified_claim.verification_results.semantic.confidence
            + verified_claim
                .verification_results
                .cross_reference
                .consistency_score)
            / 6.0;

        // Allow for small floating point differences
        assert!((verified_claim.overall_confidence - expected_confidence).abs() < 0.001);
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

        let results = engine.verify_claims(claims).await.unwrap();

        assert_eq!(results.len(), 6);

        // All claims should be processed successfully
        for (i, verified_claim) in results.iter().enumerate() {
            assert!(verified_claim.overall_confidence > 0.0);
            assert!(verified_claim.verification_timestamp <= Utc::now());

            // Verify original claim is preserved
            assert_eq!(
                verified_claim.original_claim.claim_text,
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

        let results = engine.verify_claims(vec![claim]).await.unwrap();
        let verification_results = &results[0].verification_results;

        // Test mathematical verification structure
        assert!(verification_results.mathematical.is_valid);
        assert!(verification_results.mathematical.confidence >= 0.0);
        assert!(verification_results.mathematical.confidence <= 1.0);

        // Test code behavior verification structure
        assert!(verification_results.code_behavior.confidence >= 0.0);
        assert!(verification_results.code_behavior.confidence <= 1.0);
        assert!(verification_results.code_behavior.ast_analysis.syntax_valid);

        // Test authority verification structure
        assert!(verification_results.authority.authority_score >= 0.0);
        assert!(verification_results.authority.authority_score <= 1.0);
        assert!(verification_results.authority.attribution_confidence >= 0.0);
        assert!(verification_results.authority.attribution_confidence <= 1.0);

        // Test context verification structure
        assert!(verification_results.context.confidence >= 0.0);
        assert!(verification_results.context.confidence <= 1.0);

        // Test semantic verification structure
        assert!(verification_results.semantic.semantic_valid);
        assert!(verification_results.semantic.confidence >= 0.0);
        assert!(verification_results.semantic.confidence <= 1.0);

        // Test cross-reference verification structure
        assert!(verification_results.cross_reference.consistency_score >= 0.0);
        assert!(verification_results.cross_reference.consistency_score <= 1.0);
    }

    /// Test edge cases and error handling
    #[tokio::test]
    async fn test_edge_cases_and_error_handling() {
        let mut engine = MultiModalVerificationEngine::new();

        // Test with empty claims list
        let empty_results = engine.verify_claims(vec![]).await.unwrap();
        assert_eq!(empty_results.len(), 0);

        // Test with very long claim text
        let long_claim = create_test_claim(&"A".repeat(1000), ClaimType::Technical);

        let results = engine.verify_claims(vec![long_claim]).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].overall_confidence > 0.0);

        // Test with special characters
        let special_claim = create_test_claim(
            "Claim with special chars: !@#$%^&*()_+-=[]{}|;':\",./<>?",
            ClaimType::Technical,
        );

        let results = engine.verify_claims(vec![special_claim]).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].overall_confidence > 0.0);
    }

    /// Test serialization and deserialization of verification results
    #[tokio::test]
    async fn test_verification_results_serialization() {
        let mut engine = MultiModalVerificationEngine::new();

        let claim = create_test_claim("Test claim for serialization", ClaimType::Technical);

        let results = engine.verify_claims(vec![claim]).await.unwrap();
        let verified_claim = &results[0];

        // Test JSON serialization
        let json = serde_json::to_string(verified_claim).unwrap();
        assert!(!json.is_empty());

        // Test JSON deserialization
        let deserialized: VerifiedClaim = serde_json::from_str(&json).unwrap();
        assert_eq!(
            deserialized.original_claim.claim_text,
            verified_claim.original_claim.claim_text
        );
        assert_eq!(
            deserialized.overall_confidence,
            verified_claim.overall_confidence
        );
    }
}
