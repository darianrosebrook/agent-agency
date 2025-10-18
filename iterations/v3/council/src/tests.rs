//! Comprehensive unit tests for the Council System
//! 
//! Tests all judge types, consensus coordination, and evidence enrichment

use crate::types::*;
use crate::coordinator::ConsensusCoordinator;
use crate::verdicts::*;
use crate::evidence_enrichment::EvidenceEnrichmentCoordinator;
use anyhow::Result;
use std::collections::HashMap;
use uuid::Uuid;

#[cfg(test)]
mod judge_verdict_tests {
    use super::*;

    /// Test Constitutional Judge verdict reasoning and confidence scoring
    #[tokio::test]
    async fn test_constitutional_judge_verdict() -> Result<()> {
        let judge = ConstitutionalJudge::new();
        let task_spec = create_test_task_spec();
        let evidence = create_test_evidence();

        let verdict = judge.evaluate(&task_spec, &evidence).await?;

        // Validate verdict reasoning
        assert!(!verdict.reasoning.is_empty(), "Constitutional judge should provide reasoning");
        assert!(verdict.confidence >= 0.0 && verdict.confidence <= 1.0, "Confidence should be between 0 and 1");
        
        // Validate constitutional compliance
        match verdict.decision {
            VerdictDecision::Accept => {
                assert!(verdict.reasoning.contains("constitutional") || 
                       verdict.reasoning.contains("compliant"), 
                       "Accept verdict should mention constitutional compliance");
            }
            VerdictDecision::Reject => {
                assert!(verdict.reasoning.contains("violation") || 
                       verdict.reasoning.contains("non-compliant"), 
                       "Reject verdict should mention constitutional violations");
            }
            VerdictDecision::RequireModification => {
                assert!(verdict.reasoning.contains("modification") || 
                       verdict.reasoning.contains("improvement"), 
                       "Modification verdict should mention required changes");
            }
        }

        Ok(())
    }

    /// Test Technical Judge verdict reasoning and confidence scoring
    #[tokio::test]
    async fn test_technical_judge_verdict() -> Result<()> {
        let judge = TechnicalJudge::new();
        let task_spec = create_test_task_spec();
        let evidence = create_test_evidence();

        let verdict = judge.evaluate(&task_spec, &evidence).await?;

        // Validate technical assessment
        assert!(!verdict.reasoning.is_empty(), "Technical judge should provide reasoning");
        assert!(verdict.confidence >= 0.0 && verdict.confidence <= 1.0, "Confidence should be between 0 and 1");
        
        // Validate technical focus
        let technical_keywords = ["code", "implementation", "algorithm", "performance", "security", "architecture"];
        let has_technical_focus = technical_keywords.iter().any(|keyword| 
            verdict.reasoning.to_lowercase().contains(keyword)
        );
        assert!(has_technical_focus, "Technical judge reasoning should focus on technical aspects");

        Ok(())
    }

    /// Test Quality Judge verdict reasoning and confidence scoring
    #[tokio::test]
    async fn test_quality_judge_verdict() -> Result<()> {
        let judge = QualityJudge::new();
        let task_spec = create_test_task_spec();
        let evidence = create_test_evidence();

        let verdict = judge.evaluate(&task_spec, &evidence).await?;

        // Validate quality assessment
        assert!(!verdict.reasoning.is_empty(), "Quality judge should provide reasoning");
        assert!(verdict.confidence >= 0.0 && verdict.confidence <= 1.0, "Confidence should be between 0 and 1");
        
        // Validate quality focus
        let quality_keywords = ["quality", "standards", "best practices", "maintainability", "readability", "testing"];
        let has_quality_focus = quality_keywords.iter().any(|keyword| 
            verdict.reasoning.to_lowercase().contains(keyword)
        );
        assert!(has_quality_focus, "Quality judge reasoning should focus on quality aspects");

        Ok(())
    }

    /// Test Integration Judge verdict reasoning and confidence scoring
    #[tokio::test]
    async fn test_integration_judge_verdict() -> Result<()> {
        let judge = IntegrationJudge::new();
        let task_spec = create_test_task_spec();
        let evidence = create_test_evidence();

        let verdict = judge.evaluate(&task_spec, &evidence).await?;

        // Validate integration assessment
        assert!(!verdict.reasoning.is_empty(), "Integration judge should provide reasoning");
        assert!(verdict.confidence >= 0.0 && verdict.confidence <= 1.0, "Confidence should be between 0 and 1");
        
        // Validate integration focus
        let integration_keywords = ["integration", "compatibility", "interface", "api", "dependencies", "system"];
        let has_integration_focus = integration_keywords.iter().any(|keyword| 
            verdict.reasoning.to_lowercase().contains(keyword)
        );
        assert!(has_integration_focus, "Integration judge reasoning should focus on integration aspects");

        Ok(())
    }

    /// Test evidence enrichment integration with judges
    #[tokio::test]
    async fn test_evidence_enrichment_integration() -> Result<()> {
        let enrichment = EvidenceEnrichment::new();
        let original_evidence = create_test_evidence();
        
        let enriched_evidence = enrichment.enrich_evidence(&original_evidence).await?;

        // Validate evidence enrichment
        assert!(enriched_evidence.sources.len() >= original_evidence.sources.len(), 
               "Enriched evidence should have same or more sources");
        assert!(enriched_evidence.confidence >= original_evidence.confidence, 
               "Enriched evidence should have same or higher confidence");
        
        // Validate source diversity
        let source_types: std::collections::HashSet<_> = enriched_evidence.sources
            .iter()
            .map(|s| s.source_type.clone())
            .collect();
        assert!(source_types.len() > 1, "Enriched evidence should have diverse source types");

        Ok(())
    }
}

#[cfg(test)]
mod consensus_coordinator_tests {
    use super::*;

    /// Test weighted voting algorithms
    #[tokio::test]
    async fn test_weighted_voting_algorithms() -> Result<()> {
        let coordinator = ConsensusCoordinator::new();
        let verdicts = create_test_verdicts();

        let consensus = coordinator.build_consensus(&verdicts).await?;

        // Validate consensus building
        assert!(consensus.consensus_score >= 0.0 && consensus.consensus_score <= 1.0, 
               "Consensus score should be between 0 and 1");
        assert!(!consensus.final_decision.is_empty(), "Final decision should not be empty");
        
        // Validate weighted voting
        let total_weight: f64 = verdicts.iter().map(|v| v.judge_weight).sum();
        assert!(total_weight > 0.0, "Total judge weights should be positive");

        Ok(())
    }

    /// Test consensus score calculations
    #[tokio::test]
    async fn test_consensus_score_calculations() -> Result<()> {
        let coordinator = ConsensusCoordinator::new();
        
        // Test with unanimous verdicts
        let unanimous_verdicts = create_unanimous_verdicts();
        let unanimous_consensus = coordinator.build_consensus(&unanimous_verdicts).await?;
        assert!(unanimous_consensus.consensus_score > 0.8, 
               "Unanimous verdicts should have high consensus score");

        // Test with split verdicts
        let split_verdicts = create_split_verdicts();
        let split_consensus = coordinator.build_consensus(&split_verdicts).await?;
        assert!(split_consensus.consensus_score < 0.8, 
               "Split verdicts should have lower consensus score");

        Ok(())
    }

    /// Test debate protocol triggers
    #[tokio::test]
    async fn test_debate_protocol_triggers() -> Result<()> {
        let coordinator = ConsensusCoordinator::new();
        
        // Test low consensus triggers debate
        let low_consensus_verdicts = create_low_consensus_verdicts();
        let should_debate = coordinator.should_trigger_debate(&low_consensus_verdicts).await?;
        assert!(should_debate, "Low consensus should trigger debate protocol");

        // Test high consensus doesn't trigger debate
        let high_consensus_verdicts = create_high_consensus_verdicts();
        let should_not_debate = coordinator.should_trigger_debate(&high_consensus_verdicts).await?;
        assert!(!should_not_debate, "High consensus should not trigger debate protocol");

        Ok(())
    }
}

#[cfg(test)]
mod evidence_enrichment_tests {
    use super::*;

    /// Test claim extraction integration
    #[tokio::test]
    async fn test_claim_extraction_integration() -> Result<()> {
        let enrichment = EvidenceEnrichment::new();
        let evidence = create_test_evidence();

        let enriched = enrichment.enrich_evidence(&evidence).await?;

        // Validate claim extraction integration
        assert!(enriched.claims.len() > 0, "Enriched evidence should have extracted claims");
        
        // Validate claim quality
        for claim in &enriched.claims {
            assert!(!claim.content.is_empty(), "Claim content should not be empty");
            assert!(claim.confidence >= 0.0 && claim.confidence <= 1.0, 
                   "Claim confidence should be between 0 and 1");
        }

        Ok(())
    }

    /// Test evidence confidence calculations
    #[tokio::test]
    async fn test_evidence_confidence_calculations() -> Result<()> {
        let enrichment = EvidenceEnrichment::new();
        
        // Test with high-quality sources
        let high_quality_evidence = create_high_quality_evidence();
        let high_confidence = enrichment.calculate_evidence_confidence(&high_quality_evidence).await?;
        assert!(high_confidence > 0.7, "High-quality evidence should have high confidence");

        // Test with low-quality sources
        let low_quality_evidence = create_low_quality_evidence();
        let low_confidence = enrichment.calculate_evidence_confidence(&low_quality_evidence).await?;
        assert!(low_confidence < 0.5, "Low-quality evidence should have low confidence");

        Ok(())
    }

    /// Test evidence source diversity scoring
    #[tokio::test]
    async fn test_evidence_source_diversity_scoring() -> Result<()> {
        let enrichment = EvidenceEnrichment::new();
        
        // Test diverse sources
        let diverse_evidence = create_diverse_evidence();
        let diversity_score = enrichment.calculate_source_diversity(&diverse_evidence).await?;
        assert!(diversity_score > 0.7, "Diverse evidence sources should have high diversity score");

        // Test single source
        let single_source_evidence = create_single_source_evidence();
        let single_diversity_score = enrichment.calculate_source_diversity(&single_source_evidence).await?;
        assert!(single_diversity_score < 0.3, "Single source evidence should have low diversity score");

        Ok(())
    }
}

// Test helper functions
fn create_test_task_spec() -> TaskSpec {
    TaskSpec {
        id: Uuid::new_v4(),
        title: "Test Task".to_string(),
        description: "A test task for unit testing".to_string(),
        requirements: vec!["Requirement 1".to_string(), "Requirement 2".to_string()],
        acceptance_criteria: vec!["Criterion 1".to_string(), "Criterion 2".to_string()],
        complexity: TaskComplexity::Medium,
        priority: TaskPriority::Normal,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        metadata: HashMap::new(),
    }
}

fn create_test_evidence() -> Evidence {
    Evidence {
        id: Uuid::new_v4(),
        sources: vec![
            EvidenceSource {
                source_type: SourceType::CodeAnalysis,
                source_id: "test_source_1".to_string(),
                confidence: 0.8,
                timestamp: chrono::Utc::now(),
                metadata: HashMap::new(),
            }
        ],
        claims: vec![],
        confidence: 0.8,
        created_at: chrono::Utc::now(),
        metadata: HashMap::new(),
    }
}

fn create_test_verdicts() -> Vec<JudgeVerdict> {
    vec![
        JudgeVerdict {
            judge_id: "constitutional_1".to_string(),
            judge_type: JudgeType::Constitutional,
            decision: VerdictDecision::Accept,
            reasoning: "Constitutionally compliant".to_string(),
            confidence: 0.9,
            judge_weight: 1.0,
            evidence_used: vec![],
            created_at: chrono::Utc::now(),
        },
        JudgeVerdict {
            judge_id: "technical_1".to_string(),
            judge_type: JudgeType::Technical,
            decision: VerdictDecision::Accept,
            reasoning: "Technically sound implementation".to_string(),
            confidence: 0.8,
            judge_weight: 1.0,
            evidence_used: vec![],
            created_at: chrono::Utc::now(),
        },
    ]
}

fn create_unanimous_verdicts() -> Vec<JudgeVerdict> {
    vec![
        JudgeVerdict {
            judge_id: "judge_1".to_string(),
            judge_type: JudgeType::Constitutional,
            decision: VerdictDecision::Accept,
            reasoning: "Unanimous acceptance".to_string(),
            confidence: 0.9,
            judge_weight: 1.0,
            evidence_used: vec![],
            created_at: chrono::Utc::now(),
        },
        JudgeVerdict {
            judge_id: "judge_2".to_string(),
            judge_type: JudgeType::Technical,
            decision: VerdictDecision::Accept,
            reasoning: "Unanimous acceptance".to_string(),
            confidence: 0.9,
            judge_weight: 1.0,
            evidence_used: vec![],
            created_at: chrono::Utc::now(),
        },
    ]
}

fn create_split_verdicts() -> Vec<JudgeVerdict> {
    vec![
        JudgeVerdict {
            judge_id: "judge_1".to_string(),
            judge_type: JudgeType::Constitutional,
            decision: VerdictDecision::Accept,
            reasoning: "Accept".to_string(),
            confidence: 0.6,
            judge_weight: 1.0,
            evidence_used: vec![],
            created_at: chrono::Utc::now(),
        },
        JudgeVerdict {
            judge_id: "judge_2".to_string(),
            judge_type: JudgeType::Technical,
            decision: VerdictDecision::Reject,
            reasoning: "Reject".to_string(),
            confidence: 0.6,
            judge_weight: 1.0,
            evidence_used: vec![],
            created_at: chrono::Utc::now(),
        },
    ]
}

fn create_low_consensus_verdicts() -> Vec<JudgeVerdict> {
    create_split_verdicts()
}

fn create_high_consensus_verdicts() -> Vec<JudgeVerdict> {
    create_unanimous_verdicts()
}

fn create_high_quality_evidence() -> Evidence {
    Evidence {
        id: Uuid::new_v4(),
        sources: vec![
            EvidenceSource {
                source_type: SourceType::CodeAnalysis,
                source_id: "high_quality_source".to_string(),
                confidence: 0.95,
                timestamp: chrono::Utc::now(),
                metadata: HashMap::new(),
            }
        ],
        claims: vec![],
        confidence: 0.95,
        created_at: chrono::Utc::now(),
        metadata: HashMap::new(),
    }
}

fn create_low_quality_evidence() -> Evidence {
    Evidence {
        id: Uuid::new_v4(),
        sources: vec![
            EvidenceSource {
                source_type: SourceType::UserReport,
                source_id: "low_quality_source".to_string(),
                confidence: 0.3,
                timestamp: chrono::Utc::now(),
                metadata: HashMap::new(),
            }
        ],
        claims: vec![],
        confidence: 0.3,
        created_at: chrono::Utc::now(),
        metadata: HashMap::new(),
    }
}

fn create_diverse_evidence() -> Evidence {
    Evidence {
        id: Uuid::new_v4(),
        sources: vec![
            EvidenceSource {
                source_type: SourceType::CodeAnalysis,
                source_id: "code_source".to_string(),
                confidence: 0.8,
                timestamp: chrono::Utc::now(),
                metadata: HashMap::new(),
            },
            EvidenceSource {
                source_type: SourceType::Documentation,
                source_id: "doc_source".to_string(),
                confidence: 0.7,
                timestamp: chrono::Utc::now(),
                metadata: HashMap::new(),
            },
            EvidenceSource {
                source_type: SourceType::Testing,
                source_id: "test_source".to_string(),
                confidence: 0.9,
                timestamp: chrono::Utc::now(),
                metadata: HashMap::new(),
            },
        ],
        claims: vec![],
        confidence: 0.8,
        created_at: chrono::Utc::now(),
        metadata: HashMap::new(),
    }
}

fn create_single_source_evidence() -> Evidence {
    Evidence {
        id: Uuid::new_v4(),
        sources: vec![
            EvidenceSource {
                source_type: SourceType::CodeAnalysis,
                source_id: "single_source".to_string(),
                confidence: 0.8,
                timestamp: chrono::Utc::now(),
                metadata: HashMap::new(),
            }
        ],
        claims: vec![],
        confidence: 0.8,
        created_at: chrono::Utc::now(),
        metadata: HashMap::new(),
    }
}
