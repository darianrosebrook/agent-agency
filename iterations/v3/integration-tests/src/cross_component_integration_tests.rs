//! Cross-Component Integration Tests
//! 
//! Tests communication and data flow between different system components

use crate::test_utils::*;
use anyhow::Result;
use std::collections::HashMap;
use uuid::Uuid;
use serde_json::json;

#[cfg(test)]
mod council_claim_extraction_integration {
    use super::*;

    /// Test Council ↔ Claim Extraction evidence flow
    #[tokio::test]
    async fn test_council_claim_extraction_evidence_flow() -> Result<()> {
        let test_utils = TestUtils::new().await?;
        
        // Setup test data
        let task_spec = create_test_task_spec();
        let claim_extraction_input = create_test_claim_extraction_input();
        
        // Initialize components
        let claim_extractor = test_utils.initialize_claim_extractor().await?;
        let council_coordinator = test_utils.initialize_council_coordinator().await?;
        
        // Stage 1: Extract claims from input
        let extraction_result = claim_extractor.process(&claim_extraction_input).await?;
        assert!(!extraction_result.atomic_claims.is_empty(), "Should extract atomic claims");
        
        // Stage 2: Council evaluates claims and generates evidence
        let evidence_result = council_coordinator.evaluate_claims(&extraction_result.atomic_claims).await?;
        assert!(!evidence_result.evidence.is_empty(), "Council should generate evidence");
        
        // Stage 3: Validate evidence confidence propagation
        for evidence in &evidence_result.evidence {
            assert!(evidence.confidence >= 0.0 && evidence.confidence <= 1.0, 
                   "Evidence confidence should be between 0 and 1");
        }
        
        // Stage 4: Test evidence enrichment integration
        let enriched_evidence = council_coordinator.enrich_evidence(&evidence_result.evidence).await?;
        assert!(enriched_evidence.len() >= evidence_result.evidence.len(), 
               "Enriched evidence should have same or more items");
        
        Ok(())
    }

    /// Test evidence confidence propagation through the pipeline
    #[tokio::test]
    async fn test_evidence_confidence_propagation() -> Result<()> {
        let test_utils = TestUtils::new().await?;
        
        // Setup high-confidence test data
        let high_confidence_input = create_high_confidence_claim_input();
        let claim_extractor = test_utils.initialize_claim_extractor().await?;
        let council_coordinator = test_utils.initialize_council_coordinator().await?;
        
        // Extract claims
        let extraction_result = claim_extractor.process(&high_confidence_input).await?;
        
        // Council evaluation
        let evidence_result = council_coordinator.evaluate_claims(&extraction_result.atomic_claims).await?;
        
        // Validate confidence propagation
        let avg_extraction_confidence: f64 = extraction_result.atomic_claims.iter()
            .map(|c| c.confidence)
            .sum::<f64>() / extraction_result.atomic_claims.len() as f64;
            
        let avg_evidence_confidence: f64 = evidence_result.evidence.iter()
            .map(|e| e.confidence)
            .sum::<f64>() / evidence_result.evidence.len() as f64;
        
        // Evidence confidence should be related to claim confidence
        assert!(avg_evidence_confidence > 0.0, "Evidence should have positive confidence");
        assert!(avg_evidence_confidence <= avg_extraction_confidence + 0.2, 
               "Evidence confidence should not significantly exceed claim confidence");
        
        Ok(())
    }

    /// Test error handling in evidence chain
    #[tokio::test]
    async fn test_evidence_chain_error_handling() -> Result<()> {
        let test_utils = TestUtils::new().await?;
        
        // Setup malformed input
        let malformed_input = create_malformed_claim_input();
        let claim_extractor = test_utils.initialize_claim_extractor().await?;
        let council_coordinator = test_utils.initialize_council_coordinator().await?;
        
        // Test error handling in claim extraction
        let extraction_result = claim_extractor.process(&malformed_input).await;
        
        match extraction_result {
            Ok(result) => {
                // If extraction succeeds, it should handle errors gracefully
                assert!(result.processing_metadata.errors.len() > 0 || 
                       result.atomic_claims.is_empty(),
                       "Should either have errors or no claims for malformed input");
            }
            Err(_) => {
                // Error handling is acceptable for malformed input
                // The important thing is that it doesn't panic
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod orchestration_council_integration {
    use super::*;

    /// Test Orchestration ↔ Council task evaluation
    #[tokio::test]
    async fn test_orchestration_council_task_evaluation() -> Result<()> {
        let test_utils = TestUtils::new().await?;
        
        // Setup test data
        let task_spec = create_test_task_spec();
        let orchestration_request = create_test_orchestration_request();
        
        // Initialize components
        let orchestrator = test_utils.initialize_orchestrator().await?;
        let council_coordinator = test_utils.initialize_council_coordinator().await?;
        
        // Stage 1: Orchestrator routes task
        let routing_result = orchestrator.route_task(&orchestration_request).await?;
        assert!(!routing_result.task_id.is_empty(), "Should generate task ID");
        
        // Stage 2: Council evaluates task
        let evaluation_result = council_coordinator.evaluate_task(&task_spec).await?;
        assert!(!evaluation_result.verdicts.is_empty(), "Council should provide verdicts");
        
        // Stage 3: Validate final verdict
        assert!(evaluation_result.final_decision != VerdictDecision::Unspecified, 
               "Should have a final decision");
        
        // Stage 4: Test provenance emission
        let provenance_events = test_utils.get_provenance_events().await?;
        assert!(!provenance_events.is_empty(), "Should emit provenance events");
        
        Ok(())
    }

    /// Test consensus building between orchestrator and council
    #[tokio::test]
    async fn test_orchestration_council_consensus_building() -> Result<()> {
        let test_utils = TestUtils::new().await?;
        
        // Setup complex task requiring consensus
        let complex_task = create_complex_task_spec();
        let orchestrator = test_utils.initialize_orchestrator().await?;
        let council_coordinator = test_utils.initialize_council_coordinator().await?;
        
        // Initial evaluation
        let initial_evaluation = council_coordinator.evaluate_task(&complex_task).await?;
        
        // Test consensus building process
        if initial_evaluation.consensus_score < 0.8 {
            // Trigger debate protocol
            let debate_result = council_coordinator.initiate_debate(&initial_evaluation).await?;
            assert!(debate_result.rounds.len() > 0, "Debate should have rounds");
            
            // Final consensus
            let final_evaluation = council_coordinator.build_final_consensus(&debate_result).await?;
            assert!(final_evaluation.consensus_score >= initial_evaluation.consensus_score, 
                   "Final consensus should be at least as good as initial");
        }
        
        Ok(())
    }

    /// Test task evaluation with multiple judge types
    #[tokio::test]
    async fn test_multi_judge_task_evaluation() -> Result<()> {
        let test_utils = TestUtils::new().await?;
        
        // Setup task requiring multiple judge types
        let multi_faceted_task = create_multi_faceted_task_spec();
        let council_coordinator = test_utils.initialize_council_coordinator().await?;
        
        // Evaluate with all judge types
        let evaluation_result = council_coordinator.evaluate_task(&multi_faceted_task).await?;
        
        // Validate all judge types participated
        let judge_types: std::collections::HashSet<_> = evaluation_result.verdicts.iter()
            .map(|v| v.judge_type.clone())
            .collect();
        
        assert!(judge_types.contains(&JudgeType::Constitutional), "Constitutional judge should participate");
        assert!(judge_types.contains(&JudgeType::Technical), "Technical judge should participate");
        assert!(judge_types.contains(&JudgeType::Quality), "Quality judge should participate");
        assert!(judge_types.contains(&JudgeType::Integration), "Integration judge should participate");
        
        Ok(())
    }
}

#[cfg(test)]
mod research_knowledge_integration {
    use super::*;

    /// Test Research ↔ Knowledge Base integration
    #[tokio::test]
    async fn test_research_knowledge_base_integration() -> Result<()> {
        let test_utils = TestUtils::new().await?;
        
        // Setup test data
        let research_query = create_test_research_query();
        let knowledge_entry = create_test_knowledge_entry();
        
        // Initialize components
        let research_agent = test_utils.initialize_research_agent().await?;
        let knowledge_base = test_utils.initialize_knowledge_base().await?;
        
        // Stage 1: Store knowledge entry
        let storage_result = knowledge_base.store_entry(&knowledge_entry).await?;
        assert!(storage_result.success, "Should successfully store knowledge entry");
        
        // Stage 2: Research agent queries knowledge base
        let query_result = research_agent.query_knowledge_base(&research_query).await?;
        assert!(!query_result.results.is_empty(), "Should find relevant results");
        
        // Stage 3: Validate context synthesis
        let context_result = research_agent.synthesize_context(&query_result).await?;
        assert!(!context_result.synthesized_context.is_empty(), "Should synthesize context");
        
        // Stage 4: Test cross-reference detection
        let cross_refs = research_agent.detect_cross_references(&context_result).await?;
        assert!(cross_refs.len() > 0, "Should detect cross-references");
        
        Ok(())
    }

    /// Test context synthesis algorithms
    #[tokio::test]
    async fn test_context_synthesis_algorithms() -> Result<()> {
        let test_utils = TestUtils::new().await?;
        
        // Setup multiple knowledge entries
        let knowledge_entries = create_multiple_knowledge_entries();
        let research_agent = test_utils.initialize_research_agent().await?;
        let knowledge_base = test_utils.initialize_knowledge_base().await?;
        
        // Store multiple entries
        for entry in &knowledge_entries {
            knowledge_base.store_entry(entry).await?;
        }
        
        // Query with complex context
        let complex_query = create_complex_research_query();
        let query_result = research_agent.query_knowledge_base(&complex_query).await?;
        
        // Test context synthesis
        let synthesis_result = research_agent.synthesize_context(&query_result).await?;
        
        // Validate synthesis quality
        assert!(synthesis_result.confidence > 0.7, "Synthesis should have high confidence");
        assert!(synthesis_result.synthesized_context.len() > 100, "Synthesized context should be substantial");
        
        // Validate coherence
        let coherence_score = research_agent.calculate_coherence(&synthesis_result).await?;
        assert!(coherence_score > 0.6, "Synthesized context should be coherent");
        
        Ok(())
    }

    /// Test cross-reference detection
    #[tokio::test]
    async fn test_cross_reference_detection() -> Result<()> {
        let test_utils = TestUtils::new().await?;
        
        // Setup related knowledge entries
        let related_entries = create_related_knowledge_entries();
        let research_agent = test_utils.initialize_research_agent().await?;
        let knowledge_base = test_utils.initialize_knowledge_base().await?;
        
        // Store related entries
        for entry in &related_entries {
            knowledge_base.store_entry(entry).await?;
        }
        
        // Query for cross-references
        let cross_ref_query = create_cross_reference_query();
        let query_result = research_agent.query_knowledge_base(&cross_ref_query).await?;
        
        // Detect cross-references
        let cross_refs = research_agent.detect_cross_references(&query_result).await?;
        
        // Validate cross-reference detection
        assert!(cross_refs.len() > 0, "Should detect cross-references");
        
        for cross_ref in &cross_refs {
            assert!(!cross_ref.source_id.is_empty(), "Cross-reference should have source ID");
            assert!(!cross_ref.target_id.is_empty(), "Cross-reference should have target ID");
            assert!(cross_ref.relevance_score > 0.0, "Cross-reference should have positive relevance");
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod workers_caws_integration {
    use super::*;

    /// Test Workers ↔ CAWS compliance validation
    #[tokio::test]
    async fn test_workers_caws_compliance_validation() -> Result<()> {
        let test_utils = TestUtils::new().await?;
        
        // Setup test data
        let worker_output = create_test_worker_output();
        let caws_spec = create_test_caws_spec();
        
        // Initialize components
        let worker = test_utils.initialize_worker().await?;
        let caws_validator = test_utils.initialize_caws_validator().await?;
        
        // Stage 1: Worker generates output
        let output_result = worker.process_task(&caws_spec).await?;
        assert!(!output_result.output.is_empty(), "Worker should generate output");
        
        // Stage 2: CAWS compliance checking
        let compliance_result = caws_validator.validate_compliance(&output_result, &caws_spec).await?;
        assert!(compliance_result.overall_score >= 0.0 && compliance_result.overall_score <= 1.0, 
               "Compliance score should be between 0 and 1");
        
        // Stage 3: Validate violation detection
        if !compliance_result.violations.is_empty() {
            for violation in &compliance_result.violations {
                assert!(!violation.rule_id.is_empty(), "Violation should have rule ID");
                assert!(violation.severity != ViolationSeverity::Unknown, "Violation should have severity");
            }
        }
        
        // Stage 4: Test remediation suggestions
        if compliance_result.overall_score < 0.8 {
            let remediation_result = caws_validator.generate_remediation_suggestions(&compliance_result).await?;
            assert!(!remediation_result.suggestions.is_empty(), "Should provide remediation suggestions");
        }
        
        Ok(())
    }

    /// Test compliance scoring with different worker outputs
    #[tokio::test]
    async fn test_compliance_scoring_variations() -> Result<()> {
        let test_utils = TestUtils::new().await?;
        
        let caws_validator = test_utils.initialize_caws_validator().await?;
        let caws_spec = create_test_caws_spec();
        
        // Test high-quality output
        let high_quality_output = create_high_quality_worker_output();
        let high_quality_compliance = caws_validator.validate_compliance(&high_quality_output, &caws_spec).await?;
        assert!(high_quality_compliance.overall_score > 0.8, "High-quality output should have high compliance score");
        
        // Test low-quality output
        let low_quality_output = create_low_quality_worker_output();
        let low_quality_compliance = caws_validator.validate_compliance(&low_quality_output, &caws_spec).await?;
        assert!(low_quality_compliance.overall_score < 0.6, "Low-quality output should have low compliance score");
        
        // Validate score relationship
        assert!(high_quality_compliance.overall_score > low_quality_compliance.overall_score, 
               "High-quality output should score higher than low-quality");
        
        Ok(())
    }

    /// Test violation detection accuracy
    #[tokio::test]
    async fn test_violation_detection_accuracy() -> Result<()> {
        let test_utils = TestUtils::new().await?;
        
        // Setup output with known violations
        let violating_output = create_violating_worker_output();
        let caws_spec = create_test_caws_spec();
        let caws_validator = test_utils.initialize_caws_validator().await?;
        
        // Validate compliance
        let compliance_result = caws_validator.validate_compliance(&violating_output, &caws_spec).await?;
        
        // Should detect violations
        assert!(!compliance_result.violations.is_empty(), "Should detect violations in violating output");
        
        // Validate violation details
        for violation in &compliance_result.violations {
            assert!(!violation.description.is_empty(), "Violation should have description");
            assert!(violation.severity != ViolationSeverity::Unknown, "Violation should have severity");
            assert!(!violation.rule_id.is_empty(), "Violation should reference rule ID");
        }
        
        Ok(())
    }
}

// Test helper functions
fn create_test_task_spec() -> serde_json::Value {
    json!({
        "id": Uuid::new_v4().to_string(),
        "title": "Test Task",
        "description": "A test task for integration testing",
        "requirements": ["Requirement 1", "Requirement 2"],
        "acceptance_criteria": ["Criterion 1", "Criterion 2"],
        "complexity": "Medium",
        "priority": "Normal"
    })
}

fn create_test_claim_extraction_input() -> serde_json::Value {
    json!({
        "id": Uuid::new_v4().to_string(),
        "text": "The system should authenticate users securely and handle errors gracefully.",
        "context": {
            "domain": "authentication",
            "security_requirements": "high"
        }
    })
}

fn create_high_confidence_claim_input() -> serde_json::Value {
    json!({
        "id": Uuid::new_v4().to_string(),
        "text": "The function should return a boolean value indicating success or failure.",
        "context": {
            "domain": "api_design",
            "confidence_indicators": "high"
        }
    })
}

fn create_malformed_claim_input() -> serde_json::Value {
    json!({
        "id": Uuid::new_v4().to_string(),
        "text": "",
        "context": {}
    })
}

fn create_test_orchestration_request() -> serde_json::Value {
    json!({
        "id": Uuid::new_v4().to_string(),
        "task_spec": create_test_task_spec(),
        "priority": "Normal",
        "deadline": chrono::Utc::now() + chrono::Duration::hours(1)
    })
}

fn create_complex_task_spec() -> serde_json::Value {
    json!({
        "id": Uuid::new_v4().to_string(),
        "title": "Complex Task",
        "description": "A complex task requiring multiple judge evaluations",
        "requirements": ["Security", "Performance", "Usability", "Maintainability"],
        "acceptance_criteria": ["Secure", "Fast", "Intuitive", "Maintainable"],
        "complexity": "High",
        "priority": "High"
    })
}

fn create_multi_faceted_task_spec() -> serde_json::Value {
    json!({
        "id": Uuid::new_v4().to_string(),
        "title": "Multi-faceted Task",
        "description": "A task requiring constitutional, technical, quality, and integration evaluation",
        "requirements": ["Constitutional compliance", "Technical excellence", "Quality standards", "Integration compatibility"],
        "acceptance_criteria": ["Compliant", "Excellent", "High quality", "Compatible"],
        "complexity": "High",
        "priority": "High"
    })
}

fn create_test_research_query() -> serde_json::Value {
    json!({
        "id": Uuid::new_v4().to_string(),
        "query": "How to implement secure authentication?",
        "context": {
            "domain": "security",
            "focus": "authentication"
        }
    })
}

fn create_test_knowledge_entry() -> serde_json::Value {
    json!({
        "id": Uuid::new_v4().to_string(),
        "title": "Secure Authentication Best Practices",
        "content": "Use JWT tokens, implement rate limiting, hash passwords with bcrypt",
        "source_type": "documentation",
        "confidence": 0.9
    })
}

fn create_multiple_knowledge_entries() -> Vec<serde_json::Value> {
    vec![
        json!({
            "id": Uuid::new_v4().to_string(),
            "title": "Authentication Security",
            "content": "JWT tokens and rate limiting",
            "source_type": "documentation",
            "confidence": 0.9
        }),
        json!({
            "id": Uuid::new_v4().to_string(),
            "title": "Password Security",
            "content": "bcrypt hashing and salt",
            "source_type": "best_practices",
            "confidence": 0.95
        }),
        json!({
            "id": Uuid::new_v4().to_string(),
            "title": "Session Management",
            "content": "Secure session handling",
            "source_type": "implementation",
            "confidence": 0.85
        })
    ]
}

fn create_complex_research_query() -> serde_json::Value {
    json!({
        "id": Uuid::new_v4().to_string(),
        "query": "Comprehensive security implementation for web applications",
        "context": {
            "domain": "security",
            "scope": "comprehensive",
            "complexity": "high"
        }
    })
}

fn create_related_knowledge_entries() -> Vec<serde_json::Value> {
    vec![
        json!({
            "id": "auth-1".to_string(),
            "title": "Authentication",
            "content": "User authentication methods",
            "source_type": "documentation",
            "confidence": 0.9
        }),
        json!({
            "id": "auth-2".to_string(),
            "title": "Authorization",
            "content": "User authorization and permissions",
            "source_type": "documentation",
            "confidence": 0.9
        }),
        json!({
            "id": "auth-3".to_string(),
            "title": "Access Control",
            "content": "Access control mechanisms",
            "source_type": "implementation",
            "confidence": 0.85
        })
    ]
}

fn create_cross_reference_query() -> serde_json::Value {
    json!({
        "id": Uuid::new_v4().to_string(),
        "query": "Authentication and authorization",
        "context": {
            "domain": "security",
            "focus": "cross_references"
        }
    })
}

fn create_test_worker_output() -> serde_json::Value {
    json!({
        "id": Uuid::new_v4().to_string(),
        "worker_id": "test_worker",
        "output": "Generated code with proper error handling",
        "confidence": 0.8,
        "metadata": {
            "language": "rust",
            "quality": "high"
        }
    })
}

fn create_test_caws_spec() -> serde_json::Value {
    json!({
        "id": Uuid::new_v4().to_string(),
        "name": "Test CAWS Spec",
        "version": "1.0.0",
        "rules": [
            {
                "id": "rule-1",
                "description": "Error handling required",
                "severity": "high"
            },
            {
                "id": "rule-2", 
                "description": "Code quality standards",
                "severity": "medium"
            }
        ]
    })
}

fn create_high_quality_worker_output() -> serde_json::Value {
    json!({
        "id": Uuid::new_v4().to_string(),
        "worker_id": "high_quality_worker",
        "output": "High-quality code with comprehensive error handling, documentation, and tests",
        "confidence": 0.95,
        "metadata": {
            "language": "rust",
            "quality": "excellent",
            "error_handling": "comprehensive",
            "documentation": "complete",
            "tests": "included"
        }
    })
}

fn create_low_quality_worker_output() -> serde_json::Value {
    json!({
        "id": Uuid::new_v4().to_string(),
        "worker_id": "low_quality_worker",
        "output": "Basic code without error handling",
        "confidence": 0.4,
        "metadata": {
            "language": "rust",
            "quality": "basic",
            "error_handling": "none",
            "documentation": "minimal",
            "tests": "none"
        }
    })
}

fn create_violating_worker_output() -> serde_json::Value {
    json!({
        "id": Uuid::new_v4().to_string(),
        "worker_id": "violating_worker",
        "output": "Code with security vulnerabilities and no error handling",
        "confidence": 0.3,
        "metadata": {
            "language": "rust",
            "quality": "poor",
            "error_handling": "none",
            "security": "vulnerable",
            "documentation": "none"
        }
    })
}
