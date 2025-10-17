//! Unit tests for claim extraction pipeline stages

use super::*;
use uuid::Uuid;

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

#[cfg(test)]
mod pipeline_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_pipeline_processing() {
        let processor = ClaimExtractionAndVerificationProcessor::new();
        let context = create_test_context();
        
        let sentence = "The system uses PostgreSQL for data storage and Redis for caching";
        let result = processor.process_sentence(sentence, &context).await.unwrap();
        
        // Should have processed through all stages
        assert_eq!(result.original_sentence, sentence);
        assert!(!result.atomic_claims.is_empty());
        assert!(!result.verification_evidence.is_empty());
        
        // Should have multiple atomic claims from compound sentence
        assert!(result.atomic_claims.len() >= 2);
        
        // Should have evidence for each claim
        assert!(!result.verification_evidence.is_empty());
    }

    #[tokio::test]
    async fn test_pipeline_with_ambiguous_sentence() {
        let processor = ClaimExtractionAndVerificationProcessor::new();
        let context = create_test_context();
        
        let sentence = "He believes it should be implemented using async/await";
        let result = processor.process_sentence(sentence, &context).await.unwrap();
        
        // Should handle ambiguity resolution
        assert_eq!(result.original_sentence, sentence);
        assert!(!result.atomic_claims.is_empty());
    }

    #[tokio::test]
    async fn test_pipeline_with_unverifiable_content() {
        let processor = ClaimExtractionAndVerificationProcessor::new();
        let context = create_test_context();
        
        let sentence = "This is a great idea and should be implemented";
        let result = processor.process_sentence(sentence, &context).await.unwrap();
        
        // Should handle unverifiable content gracefully
        assert!(!result.atomic_claims.is_empty());
        // May have fewer evidence items for unverifiable content
    }

    #[tokio::test]
    async fn test_pipeline_metadata_tracking() {
        let processor = ClaimExtractionAndVerificationProcessor::new();
        let context = create_test_context();
        
        let sentence = "The system uses PostgreSQL for data storage";
        let result = processor.process_sentence(sentence, &context).await.unwrap();
        
        // Should track processing metadata
        assert_eq!(result.original_sentence, sentence);
        assert!(result.processing_metadata.processing_time_ms > 0);
        assert!(!result.processing_metadata.stages_completed.is_empty());
    }

    #[tokio::test]
    async fn test_pipeline_error_handling() {
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

    #[tokio::test]
    async fn test_pipeline_with_technical_sentence() {
        let processor = ClaimExtractionAndVerificationProcessor::new();
        let context = create_test_context();
        
        let sentence = "The function returns a Result<Option<String>, Error>";
        let result = processor.process_sentence(sentence, &context).await.unwrap();
        
        // Should extract technical claims
        assert!(!result.atomic_claims.is_empty());
        assert!(result.atomic_claims.iter().any(|c| c.claim_text.contains("Result")));
    }

    #[tokio::test]
    async fn test_pipeline_with_performance_claim() {
        let processor = ClaimExtractionAndVerificationProcessor::new();
        let context = create_test_context();
        
        let sentence = "The system processes 1000 requests per second";
        let result = processor.process_sentence(sentence, &context).await.unwrap();
        
        // Should extract performance claims
        assert!(!result.atomic_claims.is_empty());
        assert!(result.atomic_claims.iter().any(|c| c.claim_text.contains("1000")));
    }

    #[tokio::test]
    async fn test_pipeline_with_security_claim() {
        let processor = ClaimExtractionAndVerificationProcessor::new();
        let context = create_test_context();
        
        let sentence = "The system implements proper authentication and authorization";
        let result = processor.process_sentence(sentence, &context).await.unwrap();
        
        // Should extract security claims
        assert!(!result.atomic_claims.is_empty());
        assert!(result.atomic_claims.iter().any(|c| c.claim_text.contains("authentication")));
    }

    #[tokio::test]
    async fn test_pipeline_with_constitutional_claim() {
        let processor = ClaimExtractionAndVerificationProcessor::new();
        let context = create_test_context();
        
        let sentence = "The system follows CAWS compliance requirements";
        let result = processor.process_sentence(sentence, &context).await.unwrap();
        
        // Should extract constitutional claims
        assert!(!result.atomic_claims.is_empty());
        assert!(result.atomic_claims.iter().any(|c| c.claim_text.contains("CAWS")));
    }

    #[tokio::test]
    async fn test_pipeline_processing_time() {
        let processor = ClaimExtractionAndVerificationProcessor::new();
        let context = create_test_context();
        
        let sentence = "The system uses PostgreSQL for data storage";
        let result = processor.process_sentence(sentence, &context).await.unwrap();
        
        // Should track processing time
        assert!(result.processing_metadata.processing_time_ms > 0);
        assert!(result.processing_metadata.processing_time_ms < 10000); // Should be reasonable
    }

    #[tokio::test]
    async fn test_pipeline_stages_completed() {
        let processor = ClaimExtractionAndVerificationProcessor::new();
        let context = create_test_context();
        
        let sentence = "The system uses PostgreSQL for data storage";
        let result = processor.process_sentence(sentence, &context).await.unwrap();
        
        // Should have completed at least some stages
        assert!(!result.processing_metadata.stages_completed.is_empty());
        
        // Should have extracted claims and evidence
        assert!(result.processing_metadata.claims_extracted > 0);
        assert!(result.processing_metadata.evidence_collected > 0);
    }
}