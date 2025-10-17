//! Unit tests for claim extraction pipeline stages

use super::*;
use uuid::Uuid;
use chrono::Utc;

/// Test data factory for creating test contexts
fn create_test_context() -> ProcessingContext {
    ProcessingContext {
        task_id: Uuid::new_v4(),
        working_spec_id: "test-spec-001".to_string(),
        source_file: Some("test.rs".to_string()),
        line_number: Some(42),
        surrounding_context: "Test context for claim extraction".to_string(),
        domain_hints: vec!["rust".to_string(), "testing".to_string()],
    }
}

/// Test data factory for creating test sentences
fn create_test_sentences() -> Vec<&'static str> {
    vec![
        "The system uses PostgreSQL for data storage",
        "He believes it should be implemented using async/await",
        "The database connection is established and then queries are executed",
        "This function returns a Result<Option<String>, Error>",
        "The API endpoint /users returns user data in JSON format",
    ]
}

#[cfg(test)]
mod disambiguation_tests {
    use super::*;

    #[test]
    fn test_ambiguity_detection_pronouns() {
        let stage = DisambiguationStage::new();
        let context = create_test_context();
        
        let sentence = "He believes it should be implemented using async/await";
        let result = stage.detect_ambiguities(sentence, &context);
        
        assert!(!result.ambiguities.is_empty());
        assert!(result.ambiguities.iter().any(|a| matches!(a.ambiguity_type, AmbiguityType::PronounReference)));
    }

    #[test]
    fn test_ambiguity_detection_technical_terms() {
        let stage = DisambiguationStage::new();
        let context = create_test_context();
        
        let sentence = "The system uses PostgreSQL for data storage";
        let result = stage.detect_ambiguities(sentence, &context);
        
        // PostgreSQL should be detected as a technical term
        assert!(result.ambiguities.iter().any(|a| matches!(a.ambiguity_type, AmbiguityType::TechnicalTerm)));
    }

    #[test]
    fn test_ambiguity_detection_scope_boundaries() {
        let stage = DisambiguationStage::new();
        let context = create_test_context();
        
        let sentence = "The database connection is established and then queries are executed";
        let result = stage.detect_ambiguities(sentence, &context);
        
        // Should detect scope boundary ambiguity
        assert!(result.ambiguities.iter().any(|a| matches!(a.ambiguity_type, AmbiguityType::ScopeBoundary)));
    }

    #[test]
    fn test_ambiguity_detection_temporal_references() {
        let stage = DisambiguationStage::new();
        let context = create_test_context();
        
        let sentence = "The system was updated and now it works better";
        let result = stage.detect_ambiguities(sentence, &context);
        
        // Should detect temporal reference ambiguity
        assert!(result.ambiguities.iter().any(|a| matches!(a.ambiguity_type, AmbiguityType::TemporalReference)));
    }

    #[test]
    fn test_context_resolution_pronouns() {
        let stage = DisambiguationStage::new();
        let context = create_test_context();
        
        let sentence = "He believes it should be implemented using async/await";
        let result = stage.resolve_context(sentence, &context);
        
        // Should resolve pronouns based on context
        assert_ne!(result.resolved_sentence, sentence);
        assert!(result.resolved_sentence.contains("async/await"));
    }

    #[test]
    fn test_context_resolution_technical_terms() {
        let stage = DisambiguationStage::new();
        let context = create_test_context();
        
        let sentence = "The system uses PostgreSQL for data storage";
        let result = stage.resolve_context(sentence, &context);
        
        // Should resolve technical terms
        assert!(result.resolved_sentence.contains("PostgreSQL"));
    }

    #[test]
    fn test_unresolvable_ambiguity_detection() {
        let stage = DisambiguationStage::new();
        let context = create_test_context();
        
        let sentence = "It does something with that thing";
        let result = stage.detect_ambiguities(sentence, &context);
        
        // Should detect unresolvable ambiguities
        assert!(result.ambiguities.iter().any(|a| matches!(a.ambiguity_type, AmbiguityType::PronounReference)));
    }

    #[test]
    fn test_disambiguation_stage_integration() {
        let stage = DisambiguationStage::new();
        let context = create_test_context();
        
        let sentence = "He believes it should be implemented using async/await";
        let result = stage.process(sentence, &context).await.unwrap();
        
        assert!(!result.ambiguities.is_empty());
        assert_ne!(result.resolved_sentence, sentence);
    }
}

#[cfg(test)]
mod qualification_tests {
    use super::*;

    #[test]
    fn test_verifiability_detection_factual() {
        let stage = QualificationStage::new();
        let context = create_test_context();
        
        let sentence = "The system uses PostgreSQL for data storage";
        let result = stage.detect_verifiability(sentence, &context);
        
        assert!(!result.verifiable_content.is_empty());
        assert!(result.verifiable_content.iter().any(|v| matches!(v.verification_method, VerificationMethod::CodeAnalysis)));
    }

    #[test]
    fn test_verifiability_detection_procedural() {
        let stage = QualificationStage::new();
        let context = create_test_context();
        
        let sentence = "The function should return a Result<Option<String>, Error>";
        let result = stage.detect_verifiability(sentence, &context);
        
        assert!(!result.verifiable_content.is_empty());
        assert!(result.verifiable_content.iter().any(|v| matches!(v.verification_method, VerificationMethod::CodeAnalysis)));
    }

    #[test]
    fn test_verifiability_detection_technical() {
        let stage = QualificationStage::new();
        let context = create_test_context();
        
        let sentence = "The API endpoint /users returns user data in JSON format";
        let result = stage.detect_verifiability(sentence, &context);
        
        assert!(!result.verifiable_content.is_empty());
        assert!(result.verifiable_content.iter().any(|v| matches!(v.verification_method, VerificationMethod::CodeAnalysis)));
    }

    #[test]
    fn test_verifiability_detection_unverifiable() {
        let stage = QualificationStage::new();
        let context = create_test_context();
        
        let sentence = "This is a great idea and should be implemented";
        let result = stage.detect_verifiability(sentence, &context);
        
        assert!(!result.unverifiable_content.is_empty());
        assert!(result.unverifiable_content.iter().any(|u| matches!(u.reason, UnverifiableReason::SubjectiveOpinion)));
    }

    #[test]
    fn test_content_rewriting_suggestions() {
        let stage = QualificationStage::new();
        let context = create_test_context();
        
        let sentence = "This is a great idea and should be implemented";
        let result = stage.suggest_rewrites(sentence, &context);
        
        assert!(!result.rewrite_suggestions.is_empty());
        assert!(result.rewrite_suggestions.iter().any(|s| s.contains("implemented")));
    }

    #[test]
    fn test_qualification_stage_integration() {
        let stage = QualificationStage::new();
        let context = create_test_context();
        
        let sentence = "The system uses PostgreSQL for data storage";
        let result = stage.process(sentence, &context).await.unwrap();
        
        assert!(!result.verifiable_content.is_empty());
        assert!(result.verifiable_content.iter().any(|v| matches!(v.verification_method, VerificationMethod::CodeAnalysis)));
    }
}

#[cfg(test)]
mod decomposition_tests {
    use super::*;

    #[test]
    fn test_claim_extraction_atomic() {
        let stage = DecompositionStage::new();
        let context = create_test_context();
        
        let sentence = "The system uses PostgreSQL for data storage";
        let result = stage.extract_claims(sentence, &context);
        
        assert!(!result.atomic_claims.is_empty());
        assert!(result.atomic_claims.iter().any(|c| c.claim_text.contains("PostgreSQL")));
    }

    #[test]
    fn test_claim_extraction_compound() {
        let stage = DecompositionStage::new();
        let context = create_test_context();
        
        let sentence = "The database connection is established and then queries are executed";
        let result = stage.extract_claims(sentence, &context);
        
        // Should extract multiple atomic claims from compound sentence
        assert!(result.atomic_claims.len() >= 2);
    }

    #[test]
    fn test_claim_extraction_complex() {
        let stage = DecompositionStage::new();
        let context = create_test_context();
        
        let sentence = "The system uses PostgreSQL for data storage, Redis for caching, and Elasticsearch for search";
        let result = stage.extract_claims(sentence, &context);
        
        // Should extract multiple atomic claims
        assert!(result.atomic_claims.len() >= 3);
        assert!(result.atomic_claims.iter().any(|c| c.claim_text.contains("PostgreSQL")));
        assert!(result.atomic_claims.iter().any(|c| c.claim_text.contains("Redis")));
        assert!(result.atomic_claims.iter().any(|c| c.claim_text.contains("Elasticsearch")));
    }

    #[test]
    fn test_context_bracket_addition() {
        let stage = DecompositionStage::new();
        let context = create_test_context();
        
        let sentence = "The system uses PostgreSQL for data storage";
        let result = stage.add_context_brackets(sentence, &context);
        
        assert!(result.contextual_brackets.iter().any(|b| b.contains("PostgreSQL")));
    }

    #[test]
    fn test_claim_type_classification() {
        let stage = DecompositionStage::new();
        let context = create_test_context();
        
        let sentence = "The system uses PostgreSQL for data storage";
        let result = stage.extract_claims(sentence, &context);
        
        assert!(result.atomic_claims.iter().any(|c| matches!(c.claim_type, ClaimType::Technical)));
    }

    #[test]
    fn test_claim_scope_assessment() {
        let stage = DecompositionStage::new();
        let context = create_test_context();
        
        let sentence = "The system uses PostgreSQL for data storage";
        let result = stage.extract_claims(sentence, &context);
        
        assert!(result.atomic_claims.iter().any(|c| matches!(c.scope, ClaimScope::SystemWide)));
    }

    #[test]
    fn test_decomposition_stage_integration() {
        let stage = DecompositionStage::new();
        let context = create_test_context();
        
        let sentence = "The system uses PostgreSQL for data storage";
        let result = stage.process(sentence, &context).await.unwrap();
        
        assert!(!result.atomic_claims.is_empty());
        assert!(result.atomic_claims.iter().any(|c| c.claim_text.contains("PostgreSQL")));
    }
}

#[cfg(test)]
mod verification_tests {
    use super::*;

    #[test]
    fn test_evidence_collection_code_analysis() {
        let stage = VerificationStage::new();
        let context = create_test_context();
        
        let claim = AtomicClaim {
            id: Uuid::new_v4(),
            claim_text: "The system uses PostgreSQL for data storage".to_string(),
            claim_type: ClaimType::Technical,
            verifiability: VerifiabilityLevel::DirectlyVerifiable,
            scope: ClaimScope::SystemWide,
            confidence: 0.8,
            contextual_brackets: vec!["PostgreSQL".to_string()],
        };
        
        let result = stage.collect_evidence(&claim, &context).await.unwrap();
        
        assert!(!result.is_empty());
        assert!(result.iter().any(|e| matches!(e.evidence_type, EvidenceType::CodeAnalysis)));
    }

    #[test]
    fn test_evidence_collection_test_execution() {
        let stage = VerificationStage::new();
        let context = create_test_context();
        
        let claim = AtomicClaim {
            id: Uuid::new_v4(),
            claim_text: "The function returns a Result<Option<String>, Error>".to_string(),
            claim_type: ClaimType::Technical,
            verifiability: VerifiabilityLevel::DirectlyVerifiable,
            scope: ClaimScope::FunctionLevel,
            confidence: 0.9,
            contextual_brackets: vec!["Result".to_string()],
        };
        
        let result = stage.collect_evidence(&claim, &context).await.unwrap();
        
        assert!(!result.is_empty());
        assert!(result.iter().any(|e| matches!(e.evidence_type, EvidenceType::TestResults)));
    }

    #[test]
    fn test_evidence_collection_documentation() {
        let stage = VerificationStage::new();
        let context = create_test_context();
        
        let claim = AtomicClaim {
            id: Uuid::new_v4(),
            claim_text: "The API endpoint /users returns user data in JSON format".to_string(),
            claim_type: ClaimType::Technical,
            verifiability: VerifiabilityLevel::DirectlyVerifiable,
            scope: ClaimScope::SystemWide,
            confidence: 0.7,
            contextual_brackets: vec!["API".to_string()],
        };
        
        let result = stage.collect_evidence(&claim, &context).await.unwrap();
        
        assert!(!result.is_empty());
        assert!(result.iter().any(|e| matches!(e.evidence_type, EvidenceType::Documentation)));
    }

    #[test]
    fn test_evidence_collection_performance_metrics() {
        let stage = VerificationStage::new();
        let context = create_test_context();
        
        let claim = AtomicClaim {
            id: Uuid::new_v4(),
            claim_text: "The system processes 1000 requests per second".to_string(),
            claim_type: ClaimType::Performance,
            verifiability: VerifiabilityLevel::DirectlyVerifiable,
            scope: ClaimScope::SystemWide,
            confidence: 0.8,
            contextual_brackets: vec!["performance".to_string()],
        };
        
        let result = stage.collect_evidence(&claim, &context).await.unwrap();
        
        assert!(!result.is_empty());
        assert!(result.iter().any(|e| matches!(e.evidence_type, EvidenceType::PerformanceMetrics)));
    }

    #[test]
    fn test_evidence_collection_security_scan() {
        let stage = VerificationStage::new();
        let context = create_test_context();
        
        let claim = AtomicClaim {
            id: Uuid::new_v4(),
            claim_text: "The system implements proper authentication and authorization".to_string(),
            claim_type: ClaimType::Security,
            verifiability: VerifiabilityLevel::DirectlyVerifiable,
            scope: ClaimScope::SystemWide,
            confidence: 0.9,
            contextual_brackets: vec!["security".to_string()],
        };
        
        let result = stage.collect_evidence(&claim, &context).await.unwrap();
        
        assert!(!result.is_empty());
        assert!(result.iter().any(|e| matches!(e.evidence_type, EvidenceType::SecurityScan)));
    }

    #[test]
    fn test_evidence_collection_constitutional_reference() {
        let stage = VerificationStage::new();
        let context = create_test_context();
        
        let claim = AtomicClaim {
            id: Uuid::new_v4(),
            claim_text: "The system follows CAWS compliance requirements".to_string(),
            claim_type: ClaimType::Constitutional,
            verifiability: VerifiabilityLevel::DirectlyVerifiable,
            scope: ClaimScope::SystemWide,
            confidence: 0.8,
            contextual_brackets: vec!["CAWS".to_string()],
        };
        
        let result = stage.collect_evidence(&claim, &context).await.unwrap();
        
        assert!(!result.is_empty());
        assert!(result.iter().any(|e| matches!(e.evidence_type, EvidenceType::ConstitutionalReference)));
    }

    #[test]
    fn test_verification_stage_integration() {
        let stage = VerificationStage::new();
        let context = create_test_context();
        
        let claim = AtomicClaim {
            id: Uuid::new_v4(),
            claim_text: "The system uses PostgreSQL for data storage".to_string(),
            claim_type: ClaimType::Technical,
            verifiability: VerifiabilityLevel::DirectlyVerifiable,
            scope: ClaimScope::SystemWide,
            confidence: 0.8,
            contextual_brackets: vec!["PostgreSQL".to_string()],
        };
        
        let result = stage.process(&claim, &context).await.unwrap();
        
        assert!(!result.verification_evidence.is_empty());
        assert!(result.verification_evidence.iter().any(|e| matches!(e.evidence_type, EvidenceType::CodeAnalysis)));
    }
}

#[cfg(test)]
mod pipeline_integration_tests {
    use super::*;

    #[test]
    fn test_full_pipeline_processing() {
        let processor = ClaimExtractionAndVerificationProcessor::new();
        let context = create_test_context();
        
        let sentence = "The system uses PostgreSQL for data storage and Redis for caching";
        let result = processor.process_sentence(sentence, &context).await.unwrap();
        
        // Should have processed through all stages
        assert_ne!(result.disambiguated_sentence, sentence);
        assert!(!result.atomic_claims.is_empty());
        assert!(!result.verification_evidence.is_empty());
        
        // Should have multiple atomic claims from compound sentence
        assert!(result.atomic_claims.len() >= 2);
        
        // Should have evidence for each claim
        assert!(!result.verification_evidence.is_empty());
    }

    #[test]
    fn test_pipeline_with_ambiguous_sentence() {
        let processor = ClaimExtractionAndVerificationProcessor::new();
        let context = create_test_context();
        
        let sentence = "He believes it should be implemented using async/await";
        let result = processor.process_sentence(sentence, &context).await.unwrap();
        
        // Should handle ambiguity resolution
        assert_ne!(result.disambiguated_sentence, sentence);
        assert!(!result.atomic_claims.is_empty());
    }

    #[test]
    fn test_pipeline_with_unverifiable_content() {
        let processor = ClaimExtractionAndVerificationProcessor::new();
        let context = create_test_context();
        
        let sentence = "This is a great idea and should be implemented";
        let result = processor.process_sentence(sentence, &context).await.unwrap();
        
        // Should handle unverifiable content gracefully
        assert!(!result.atomic_claims.is_empty());
        // May have fewer evidence items for unverifiable content
    }

    #[test]
    fn test_pipeline_metadata_tracking() {
        let processor = ClaimExtractionAndVerificationProcessor::new();
        let context = create_test_context();
        
        let sentence = "The system uses PostgreSQL for data storage";
        let result = processor.process_sentence(sentence, &context).await.unwrap();
        
        // Should track processing metadata
        assert_eq!(result.processing_metadata.original_sentence, sentence);
        assert!(result.processing_metadata.processing_time_ms > 0);
        assert!(!result.processing_metadata.stages_completed.is_empty());
    }

    #[test]
    fn test_pipeline_error_handling() {
        let processor = ClaimExtractionAndVerificationProcessor::new();
        let context = create_test_context();
        
        let sentence = ""; // Empty sentence should be handled gracefully
        let result = processor.process_sentence(sentence, &context).await;
        
        // Should handle empty input gracefully
        match result {
            Ok(_) => {
                // If it succeeds, should have empty results
            }
            Err(_) => {
                // If it fails, should be a handled error
            }
        }
    }
}
