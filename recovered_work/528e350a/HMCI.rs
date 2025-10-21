//! Integration test for council plan review functionality
//!
//! Tests the complete workflow of plan generation through constitutional review.

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::sync::Arc;

    // Mock consensus coordinator for testing
    struct MockConsensusCoordinator;

    #[async_trait::async_trait]
    impl crate::coordinator::ConsensusCoordinator for MockConsensusCoordinator {
        async fn evaluate_task(
            &self,
            _task_spec: crate::models::TaskSpec,
        ) -> Result<crate::types::ConsensusResult> {
            // Mock consensus result with mixed verdicts
            Ok(crate::types::ConsensusResult {
                task_id: uuid::Uuid::new_v4(),
                final_verdict: crate::types::FinalVerdict::Accept,
                confidence_score: 0.85,
                participant_contributions: vec![
                    crate::models::ParticipantContribution {
                        participant_id: "constitutional-judge".to_string(),
                        contribution: "Plan meets constitutional requirements".to_string(),
                        confidence_score: 0.9,
                        rationale: "No ethical violations detected, CAWS compliance verified".to_string(),
                        evidence_packets: vec![],
                        timestamp: chrono::Utc::now(),
                    },
                    crate::models::ParticipantContribution {
                        participant_id: "technical-judge".to_string(),
                        contribution: "Technical implementation feasible".to_string(),
                        confidence_score: 0.8,
                        rationale: "Scope and constraints are reasonable for the tech stack".to_string(),
                        evidence_packets: vec![],
                        timestamp: chrono::Utc::now(),
                    },
                    crate::models::ParticipantContribution {
                        participant_id: "quality-judge".to_string(),
                        contribution: "Quality criteria met".to_string(),
                        confidence_score: 0.7,
                        rationale: "Acceptance criteria are clear and testable".to_string(),
                        evidence_packets: vec![],
                        timestamp: chrono::Utc::now(),
                    },
                ],
                evidence_packets: vec![],
                timestamp: chrono::Utc::now(),
            })
        }
    }

    // Mock multimodal context provider
    struct MockMultimodalContextProvider;

    #[async_trait::async_trait]
    impl agent_agency_research::MultimodalContextProvider for MockMultimodalContextProvider {
        async fn provide_context(
            &self,
            _query: &str,
            _context_budget: Option<agent_agency_research::ContextBudget>,
        ) -> Result<agent_agency_research::MultimodalContext> {
            Ok(agent_agency_research::MultimodalContext {
                text_content: "Mock context for plan review".to_string(),
                image_urls: vec![],
                audio_urls: vec![],
                video_urls: vec![],
                document_urls: vec![],
                metadata: std::collections::HashMap::new(),
            })
        }
    }

    #[tokio::test]
    async fn test_complete_plan_review_workflow() {
        // Setup
        let coordinator = Arc::new(MockConsensusCoordinator);
        let context_provider = Arc::new(MockMultimodalContextProvider);

        let config = PlanReviewConfig {
            min_constitutional_score: 0.7,
            min_technical_score: 0.6,
            min_quality_score: 0.5,
            max_review_time_seconds: 30,
            enable_detailed_rationale: true,
            require_multimodal_evidence: false,
        };

        let review_service = PlanReviewService::new(coordinator, context_provider, config);

        // Create a working spec to review
        let working_spec = crate::types::WorkingSpec {
            id: "SPEC-TEST-001".to_string(),
            title: "Implement User Authentication API".to_string(),
            scope: Some(crate::types::WorkingSpecScope {
                r#in: Some(vec!["src/api/auth/".to_string(), "tests/api/auth/".to_string()]),
                out: Some(vec!["src/billing/".to_string(), "node_modules/".to_string()]),
            }),
            risk_tier: crate::types::RiskTier::High,
            acceptance_criteria: vec![
                crate::types::AcceptanceCriterion {
                    id: "A1".to_string(),
                    description: "Given user has valid credentials, when POST to /login, then receive JWT token".to_string(),
                    evidence: vec![],
                    priority: crate::types::CriterionPriority::MustHave,
                },
                crate::types::AcceptanceCriterion {
                    id: "A2".to_string(),
                    description: "Given user has invalid credentials, when POST to /login, then receive 401 error".to_string(),
                    evidence: vec![],
                    priority: crate::types::CriterionPriority::MustHave,
                },
            ],
        };

        // Create task context
        let task_context = super::super::super::planning::agent::TaskContext {
            repo_info: super::super::super::planning::agent::RepositoryInfo {
                name: "api-service".to_string(),
                description: Some("REST API service".to_string()),
                primary_language: "Rust".to_string(),
                size_kb: 2048,
                last_commit: chrono::Utc::now(),
                contributors: vec!["alice".to_string(), "bob".to_string()],
            },
            recent_incidents: vec![],
            team_constraints: vec!["Use async/await".to_string()],
            tech_stack: super::super::super::planning::agent::TechStack {
                languages: vec!["Rust".to_string()],
                frameworks: vec!["Axum".to_string()],
                databases: vec!["PostgreSQL".to_string()],
                deployment: vec!["Docker".to_string()],
            },
            historical_data: super::super::super::planning::agent::HistoricalData {
                completed_tasks: vec![],
                average_completion_time: std::time::Duration::from_secs(8 * 3600),
                success_rate: 0.9,
            },
        };

        // Execute plan review
        let result = review_service.review_plan(&working_spec, &task_context).await;

        // Verify result
        assert!(result.is_ok(), "Plan review should succeed");

        let verdict = result.unwrap();

        // Verify verdict structure
        assert_eq!(verdict.working_spec_id, "SPEC-TEST-001");
        assert!(matches!(verdict.decision, PlanReviewDecision::Approved));
        assert!(verdict.constitutional_score >= 0.0 && verdict.constitutional_score <= 1.0);
        assert!(verdict.technical_score >= 0.0 && verdict.technical_score <= 1.0);
        assert!(verdict.quality_score >= 0.0 && verdict.quality_score <= 1.0);
        assert_eq!(verdict.judge_verdicts.len(), 3); // constitutional, technical, quality

        // Verify judge verdicts
        let judge_types: Vec<_> = verdict.judge_verdicts.iter()
            .map(|jv| &jv.judge_type)
            .collect();

        assert!(judge_types.contains(&JudgeType::Constitutional));
        assert!(judge_types.contains(&JudgeType::Technical));
        assert!(judge_types.contains(&JudgeType::Quality));

        // Verify rationale and suggestions
        assert!(!verdict.rationale.is_empty());
        assert!(!verdict.suggested_improvements.is_empty());

        println!("✅ Plan review integration test passed");
        println!("   Working Spec ID: {}", verdict.working_spec_id);
        println!("   Decision: {:?}", verdict.decision);
        println!("   Constitutional Score: {:.2}", verdict.constitutional_score);
        println!("   Technical Score: {:.2}", verdict.technical_score);
        println!("   Quality Score: {:.2}", verdict.quality_score);
        println!("   Judge Verdicts: {}", verdict.judge_verdicts.len());
        println!("   Rationale Length: {} chars", verdict.rationale.len());
        println!("   Suggested Improvements: {}", verdict.suggested_improvements.len());
    }

    #[test]
    fn test_plan_review_type_system() {
        // Test that all plan review types serialize correctly

        let verdict = PlanReviewVerdict {
            working_spec_id: "SPEC-001".to_string(),
            decision: PlanReviewDecision::Approved,
            constitutional_score: 0.9,
            technical_score: 0.8,
            quality_score: 0.7,
            judge_verdicts: vec![
                PlanJudgeVerdict {
                    judge_type: JudgeType::Constitutional,
                    participant_id: "constitutional-judge".to_string(),
                    verdict: PlanVerdict::Approved,
                    confidence: 0.9,
                    rationale: "No constitutional violations".to_string(),
                    suggested_improvements: vec![],
                }
            ],
            rationale: "Plan meets all constitutional requirements".to_string(),
            suggested_improvements: vec![
                "Consider adding rate limiting".to_string(),
                "Add comprehensive error handling".to_string(),
            ],
            reviewed_at: chrono::Utc::now(),
        };

        // Test JSON serialization
        let json = serde_json::to_string(&verdict).unwrap();
        let deserialized: PlanReviewVerdict = serde_json::from_str(&json).unwrap();

        assert_eq!(verdict.working_spec_id, deserialized.working_spec_id);
        assert!(matches!(verdict.decision, PlanReviewDecision::Approved));
        assert_eq!(verdict.constitutional_score, deserialized.constitutional_score);
        assert_eq!(verdict.judge_verdicts.len(), 1);

        println!("✅ Plan review type system test passed");
    }

    #[test]
    fn test_rejection_scenarios() {
        // Test rejection scenarios with different decision types

        let test_cases = vec![
            (
                PlanReviewDecision::Rejected {
                    reason: "Critical constitutional violation".to_string(),
                },
                "Rejected",
            ),
            (
                PlanReviewDecision::NeedsRevision {
                    revision_requirements: vec!["Add acceptance criteria".to_string()],
                },
                "NeedsRevision",
            ),
            (
                PlanReviewDecision::ApprovedWithConditions {
                    conditions: vec!["Address security concerns".to_string()],
                },
                "ApprovedWithConditions",
            ),
            (
                PlanReviewDecision::Approved,
                "Approved",
            ),
        ];

        for (decision, expected_type) in test_cases {
            let verdict = PlanReviewVerdict {
                working_spec_id: "SPEC-TEST".to_string(),
                decision,
                constitutional_score: 0.8,
                technical_score: 0.7,
                quality_score: 0.6,
                judge_verdicts: vec![],
                rationale: "Test rationale".to_string(),
                suggested_improvements: vec![],
                reviewed_at: chrono::Utc::now(),
            };

            // Serialize and check it contains the expected type
            let json = serde_json::to_string(&verdict).unwrap();
            assert!(json.contains(expected_type), "JSON should contain {}", expected_type);
        }

        println!("✅ Plan review rejection scenarios test passed");
    }
}

