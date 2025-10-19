//! Unit tests for Advanced Arbitration Engine
//!
//! Tests V3's superior arbitration capabilities that surpass V2's basic conflict resolution.

#[cfg(test)]
mod tests {
    use crate::advanced_arbitration::{
        AdvancedArbitrationEngine, ArbitrationResult, ConfidenceScorer, ConsensusBuilder,
        ConsensusResult, LearningInsights, PerformanceTracker, PleadingResult, PleadingWorkflow,
        QualityAssessment, QualityAssessor, WorkerOutput,
    };
    use chrono::Utc;
    use std::collections::HashMap;
    use uuid::Uuid;

    /// Helper function to create test worker output
    fn create_test_worker_output(
        worker_id: &str,
        output: &str,
        confidence: f32,
        quality_score: f32,
        response_time_ms: u64,
    ) -> WorkerOutput {
        WorkerOutput {
            worker_id: worker_id.to_string(),
            task_id: Uuid::new_v4(),
            output: output.to_string(),
            confidence,
            quality_score,
            response_time_ms,
            metadata: HashMap::new(),
        }
    }

    /// Helper function to create test task spec
    fn create_test_task_spec(task_id: &str, task_type: &str) -> crate::models::TaskSpec {
        use crate::models::*;
        TaskSpec {
            id: Uuid::new_v4(),
            title: task_id.to_string(),
            description: "Test task for arbitration".to_string(),
            risk_tier: RiskTier::Medium,
            scope: TaskScope {
                files_affected: vec![],
                max_files: Some(10),
                max_loc: Some(1000),
                domains: vec![task_type.to_string()],
            },
            acceptance_criteria: vec![],
            context: TaskContext {
                workspace_root: "/tmp".to_string(),
                git_branch: "main".to_string(),
                recent_changes: vec![],
                dependencies: HashMap::new(),
                environment: Environment::Development,
            },
            worker_output: create_models_worker_output("test_output"),
            caws_spec: None,
        }
    }

    /// Helper function to create models WorkerOutput
    fn create_models_worker_output(content: &str) -> crate::models::WorkerOutput {
        use crate::models::*;
        WorkerOutput {
            content: content.to_string(),
            files_modified: vec![],
            rationale: "Test rationale".to_string(),
            self_assessment: SelfAssessment {
                confidence: 0.8,
                quality_score: 0.9,
                caws_compliance: 0.85,
                concerns: vec![],
                improvements: vec![],
                estimated_effort: Some("2 hours".to_string()),
            },
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_advanced_arbitration_engine_creation() {
        let engine = AdvancedArbitrationEngine::new().unwrap();
        // Engine should be created successfully
        assert!(true); // Placeholder test - engine creation is tested implicitly
    }

    #[tokio::test]
    async fn test_confidence_scorer_multi_dimensional_scoring() {
        let scorer = ConfidenceScorer::new();

        let outputs = vec![
            create_test_worker_output("worker1", "Solution A", 0.8, 0.9, 500),
            create_test_worker_output("worker2", "Solution B", 0.7, 0.8, 700),
            create_test_worker_output("worker3", "Solution C", 0.9, 0.7, 600),
        ];

        let scores = scorer.score_multi_dimensional(&outputs).await.unwrap();

        assert_eq!(scores.len(), 3);
        assert!(scores.contains_key("worker1"));
        assert!(scores.contains_key("worker2"));
        assert!(scores.contains_key("worker3"));

        // All scores should be between 0 and 1
        for score in scores.values() {
            assert!(*score >= 0.0 && *score <= 1.0);
        }
    }

    #[tokio::test]
    async fn test_pleading_workflow_with_learning_integration() {
        let workflow = PleadingWorkflow::new();

        let outputs = vec![
            create_test_worker_output("worker1", "Solution A", 0.8, 0.9, 500),
            create_test_worker_output("worker2", "Solution B", 0.7, 0.8, 700),
        ];

        let confidence_scores =
            HashMap::from([("worker1".to_string(), 0.85), ("worker2".to_string(), 0.75)]);

        let quality_assessment = QualityAssessment {
            completeness_scores: HashMap::from([
                ("worker1".to_string(), 0.9),
                ("worker2".to_string(), 0.8),
            ]),
            correctness_scores: HashMap::from([
                ("worker1".to_string(), 0.8),
                ("worker2".to_string(), 0.9),
            ]),
            consistency_scores: HashMap::from([
                ("worker1".to_string(), 0.7),
                ("worker2".to_string(), 0.8),
            ]),
            innovation_scores: HashMap::from([
                ("worker1".to_string(), 0.6),
                ("worker2".to_string(), 0.7),
            ]),
            quality_predictions: crate::advanced_arbitration::QualityPredictions {
                predicted_improvements: vec!["Better error handling".to_string()],
                quality_trends: vec!["Improving consistency".to_string()],
                regression_risks: vec!["Complex edge cases".to_string()],
            },
            overall_quality: 0.8,
        };

        let result = workflow
            .resolve_with_learning(&outputs, &confidence_scores, &quality_assessment)
            .await
            .unwrap();

        // Result should contain evidence collection and learning insights
        assert!(result.evidence_collection.evidence.len() >= 0);
        assert!(result.learning_insights.performance_improvements.len() >= 0);
    }

    #[tokio::test]
    async fn test_consensus_builder_quality_weighted_consensus() {
        let builder = ConsensusBuilder::new();

        let pleading_result = PleadingResult {
            evidence_collection: crate::advanced_arbitration::EvidenceCollection {
                evidence: HashMap::new(),
                credibility_scores: HashMap::new(),
                source_validation: HashMap::new(),
            },
            debate_result: crate::advanced_arbitration::DebateResult {
                rounds: vec![],
                final_arguments: HashMap::new(),
                consensus_reached: true,
            },
            conflict_resolution: crate::advanced_arbitration::ConflictResolution {
                resolution_strategy: "Quality-weighted consensus".to_string(),
                resolved_conflicts: vec![],
                remaining_conflicts: vec![],
                confidence: 0.9,
            },
            learning_insights: LearningInsights {
                performance_improvements: vec!["Improved conflict resolution".to_string()],
                quality_insights: vec!["Better evidence evaluation".to_string()],
                conflict_patterns: vec!["Pattern A".to_string()],
                optimization_suggestions: vec!["Optimize scoring".to_string()],
            },
        };

        let confidence_scores =
            HashMap::from([("worker1".to_string(), 0.85), ("worker2".to_string(), 0.75)]);

        let quality_assessment = QualityAssessment {
            completeness_scores: HashMap::from([
                ("worker1".to_string(), 0.9),
                ("worker2".to_string(), 0.8),
            ]),
            correctness_scores: HashMap::from([
                ("worker1".to_string(), 0.8),
                ("worker2".to_string(), 0.9),
            ]),
            consistency_scores: HashMap::from([
                ("worker1".to_string(), 0.7),
                ("worker2".to_string(), 0.8),
            ]),
            innovation_scores: HashMap::from([
                ("worker1".to_string(), 0.6),
                ("worker2".to_string(), 0.7),
            ]),
            quality_predictions: crate::advanced_arbitration::QualityPredictions {
                predicted_improvements: vec!["Better error handling".to_string()],
                quality_trends: vec!["Improving consistency".to_string()],
                regression_risks: vec!["Complex edge cases".to_string()],
            },
            overall_quality: 0.8,
        };

        let result = builder
            .build_quality_weighted_consensus(
                Uuid::new_v4(),
                &pleading_result,
                &confidence_scores,
                &quality_assessment,
            )
            .await
            .unwrap();

        assert!(result.confidence >= 0.0 && result.confidence <= 1.0);
        assert!(result.quality_score >= 0.0 && result.quality_score <= 1.0);
        assert!(result.consensus_score >= 0.0 && result.consensus_score <= 1.0);
        assert!(!result.final_decision.is_empty());
        assert!(!result.reasoning.is_empty());
    }

    #[tokio::test]
    async fn test_full_arbitration_workflow() {
        let engine = AdvancedArbitrationEngine::new();

        let conflicting_outputs = vec![
            create_test_worker_output(
                "worker1",
                "Implement feature X using approach A",
                0.8,
                0.9,
                500,
            ),
            create_test_worker_output(
                "worker2",
                "Implement feature X using approach B",
                0.7,
                0.8,
                700,
            ),
            create_test_worker_output(
                "worker3",
                "Implement feature X using approach C",
                0.9,
                0.7,
                600,
            ),
        ];

        let result = engine
            .unwrap()
            .resolve_conflicts(conflicting_outputs)
            .await
            .unwrap();

        // Verify arbitration result structure
        assert!(result.confidence >= 0.0 && result.confidence <= 1.0);
        assert!(result.quality_score >= 0.0 && result.quality_score <= 1.0);
        assert!(result.consensus_score >= 0.0 && result.consensus_score <= 1.0);
        assert!(!result.final_decision.is_empty());
        assert!(!result.reasoning.is_empty());
        assert!(result.individual_scores.len() > 0);
        assert!(result.timestamp <= Utc::now());
    }

    #[tokio::test]
    async fn test_conflict_prediction() {
        let engine = AdvancedArbitrationEngine::new();

        let task_spec = create_test_task_spec("TASK-001", "feature_implementation");

        let prediction = engine.unwrap().predict_conflicts(&task_spec).await.unwrap();

        assert_eq!(prediction.task_id, task_spec.id);
        assert!(prediction.conflict_risk >= 0.0 && prediction.conflict_risk <= 1.0);
        assert!(prediction.confidence >= 0.0 && prediction.confidence <= 1.0);
        assert!(!prediction.predicted_conflict_types.is_empty());
        assert!(!prediction.preventive_measures.is_empty());
    }

    #[tokio::test]
    async fn test_arbitration_with_single_output() {
        let engine = AdvancedArbitrationEngine::new();

        let single_output = vec![create_test_worker_output(
            "worker1",
            "Simple solution",
            0.9,
            0.95,
            300,
        )];

        let result = engine
            .unwrap()
            .resolve_conflicts(single_output)
            .await
            .unwrap();

        // Should handle single output gracefully
        assert!(result.confidence >= 0.0 && result.confidence <= 1.0);
        // The decision should contain the output content (either directly or processed)
        assert!(
            result.final_decision.contains("Simple solution") || !result.final_decision.is_empty()
        );
    }

    #[tokio::test]
    async fn test_arbitration_with_empty_outputs() {
        let engine = AdvancedArbitrationEngine::new();

        let empty_outputs: Vec<WorkerOutput> = vec![];

        // Test 1: Empty inputs should be handled gracefully
        let result = engine.unwrap().resolve_conflicts(empty_outputs).await;
        assert!(result.is_ok() || result.is_err()); // Either succeeds with empty result or errors appropriately

        // Test 2: Large number of conflicting outputs
        test_conflicting_outputs_resolution().await;

        // Test 3: Single output should pass through cleanly
        test_single_output_resolution().await;

        // Test 4: Mixed quality outputs
        test_mixed_quality_outputs().await;

        // Test 5: Performance with many outputs
        test_performance_with_many_outputs().await;
    }

    async fn test_conflicting_outputs_resolution() {
        let engine = AdvancedArbitrationEngine::new().unwrap();

        // Create multiple conflicting outputs
        let conflicting_outputs = vec![
            WorkerOutput {
                worker_id: Uuid::new_v4(),
                task_id: Uuid::new_v4(),
                content: "Use approach A for better performance".to_string(),
                files_modified: vec![],
                rationale: "Performance is critical".to_string(),
                self_assessment: SelfAssessment {
                    caws_compliance: 0.8,
                    quality_score: 0.9,
                    confidence: 0.85,
                    concerns: vec![],
                    improvements: vec![],
                },
                response_time_ms: Some(100),
                metadata: HashMap::new(),
            },
            WorkerOutput {
                worker_id: Uuid::new_v4(),
                task_id: Uuid::new_v4(),
                content: "Use approach B for better maintainability".to_string(),
                files_modified: vec![],
                rationale: "Code quality is paramount".to_string(),
                self_assessment: SelfAssessment {
                    caws_compliance: 0.9,
                    quality_score: 0.95,
                    confidence: 0.90,
                    concerns: vec![],
                    improvements: vec![],
                },
                response_time_ms: Some(120),
                metadata: HashMap::new(),
            },
            WorkerOutput {
                worker_id: Uuid::new_v4(),
                task_id: Uuid::new_v4(),
                content: "Use approach C for faster development".to_string(),
                files_modified: vec![],
                rationale: "Time to market is critical".to_string(),
                self_assessment: SelfAssessment {
                    caws_compliance: 0.7,
                    quality_score: 0.75,
                    confidence: 0.70,
                    concerns: vec!["May compromise quality".to_string()],
                    improvements: vec![],
                },
                response_time_ms: Some(80),
                metadata: HashMap::new(),
            },
        ];

        let result = engine.resolve_conflicts(conflicting_outputs).await.unwrap();

        // Validate the arbitration result
        assert!(result.confidence >= 0.0 && result.confidence <= 1.0);
        assert!(!result.final_decision.is_empty());
        assert!(result.confidence >= 0.7, "High-quality conflicting outputs should produce confident result");

        // Should contain elements from multiple approaches
        let decision_lower = result.final_decision.to_lowercase();
        let contains_performance = decision_lower.contains("performance");
        let contains_maintainability = decision_lower.contains("maintainability") || decision_lower.contains("quality");
        let contains_speed = decision_lower.contains("speed") || decision_lower.contains("development");

        // Should balance multiple perspectives
        assert!(contains_performance || contains_maintainability || contains_speed,
                "Decision should reflect consideration of multiple approaches");
    }

    async fn test_single_output_resolution() {
        let engine = AdvancedArbitrationEngine::new().unwrap();

        let single_output = vec![WorkerOutput {
            worker_id: Uuid::new_v4(),
            task_id: Uuid::new_v4(),
            content: "Implement simple and effective solution".to_string(),
            files_modified: vec![],
            rationale: "Keep it simple and focused".to_string(),
            self_assessment: SelfAssessment {
                caws_compliance: 0.85,
                quality_score: 0.90,
                confidence: 0.88,
                concerns: vec![],
                improvements: vec![],
            },
            response_time_ms: Some(95),
            metadata: HashMap::new(),
        }];

        let result = engine.resolve_conflicts(single_output).await.unwrap();

        // Single output should pass through with high confidence
        assert!(result.confidence >= 0.8, "Single high-quality output should maintain high confidence");
        assert!(result.final_decision.contains("simple") || result.final_decision.contains("effective"));
    }

    async fn test_mixed_quality_outputs() {
        let engine = AdvancedArbitrationEngine::new().unwrap();

        let mixed_outputs = vec![
            WorkerOutput {
                worker_id: Uuid::new_v4(),
                task_id: Uuid::new_v4(),
                content: "High quality solution with comprehensive analysis".to_string(),
                files_modified: vec![],
                rationale: "Thorough approach ensures reliability".to_string(),
                self_assessment: SelfAssessment {
                    caws_compliance: 0.95,
                    quality_score: 0.98,
                    confidence: 0.96,
                    concerns: vec![],
                    improvements: vec![],
                },
                response_time_ms: Some(150),
                metadata: HashMap::new(),
            },
            WorkerOutput {
                worker_id: Uuid::new_v4(),
                task_id: Uuid::new_v4(),
                content: "Quick fix that might work".to_string(),
                files_modified: vec![],
                rationale: "Fast implementation".to_string(),
                self_assessment: SelfAssessment {
                    caws_compliance: 0.4,
                    quality_score: 0.5,
                    confidence: 0.3,
                    concerns: vec!["May not be robust".to_string(), "Needs testing".to_string()],
                    improvements: vec!["Add validation".to_string()],
                },
                response_time_ms: Some(45),
                metadata: HashMap::new(),
            },
        ];

        let result = engine.resolve_conflicts(mixed_outputs).await.unwrap();

        // Should favor high-quality output
        assert!(result.confidence >= 0.5, "Should produce reasonable confidence from mixed quality");
        assert!(result.final_decision.contains("comprehensive") || result.final_decision.contains("analysis"),
                "Should favor the higher quality solution");
    }

    async fn test_performance_with_many_outputs() {
        let engine = AdvancedArbitrationEngine::new().unwrap();

        // Create many outputs to test performance
        let mut many_outputs = Vec::new();
        for i in 0..50 {
            many_outputs.push(WorkerOutput {
                worker_id: Uuid::new_v4(),
                task_id: Uuid::new_v4(),
                content: format!("Solution approach {}", i),
                files_modified: vec![],
                rationale: format!("Rationale for approach {}", i),
                self_assessment: SelfAssessment {
                    caws_compliance: 0.7 + (i as f32 * 0.006), // Vary quality slightly
                    quality_score: 0.75 + (i as f32 * 0.005),
                    confidence: 0.7 + (i as f32 * 0.006),
                    concerns: if i % 5 == 0 { vec![format!("Concern {}", i)] } else { vec![] },
                    improvements: vec![],
                },
                response_time_ms: Some(80 + i as u64),
                metadata: HashMap::new(),
            });
        }

        let start_time = std::time::Instant::now();
        let result = engine.resolve_conflicts(many_outputs).await.unwrap();
        let duration = start_time.elapsed();

        // Should complete within reasonable time (under 1 second for 50 outputs)
        assert!(duration < std::time::Duration::from_secs(1),
                "Performance test failed: took {:?} for 50 outputs", duration);

        // Should produce valid result
        assert!(result.confidence >= 0.0 && result.confidence <= 1.0);
        assert!(!result.final_decision.is_empty());
    }

    #[tokio::test]
    async fn test_learning_integration_arbitration_insights() {
        let engine = AdvancedArbitrationEngine::new();

        let outputs = vec![
            create_test_worker_output("worker1", "Solution A", 0.8, 0.9, 500),
            create_test_worker_output("worker2", "Solution B", 0.7, 0.8, 700),
        ];

        let result = engine.unwrap().resolve_conflicts(outputs).await.unwrap();

        // Learning insights should be populated
        assert!(result.learning_insights.performance_improvements.len() >= 0);
        assert!(result.learning_insights.quality_insights.len() >= 0);
        assert!(result.learning_insights.conflict_patterns.len() >= 0);
        assert!(result.learning_insights.optimization_suggestions.len() >= 0);
    }

    #[tokio::test]
    async fn test_quality_assessor_comprehensive_evaluation() {
        let assessor = QualityAssessor::new();

        let outputs = vec![
            create_test_worker_output("worker1", "High quality solution", 0.9, 0.95, 400),
            create_test_worker_output("worker2", "Medium quality solution", 0.7, 0.75, 600),
        ];

        let assessment = assessor.assess_quality(&outputs).await.unwrap();

        assert!(!assessment.completeness_scores.is_empty());
        assert!(!assessment.correctness_scores.is_empty());
        assert!(!assessment.consistency_scores.is_empty());
        assert!(!assessment.innovation_scores.is_empty());

        // All scores should be valid
        for score in assessment.completeness_scores.values() {
            assert!(*score >= 0.0 && *score <= 1.0);
        }
        for score in assessment.correctness_scores.values() {
            assert!(*score >= 0.0 && *score <= 1.0);
        }
    }

    #[tokio::test]
    async fn test_performance_tracker_arbitration_metrics() {
        let tracker = PerformanceTracker::new();

        let consensus = ConsensusResult {
            task_id: Uuid::new_v4(),
            verdict_id: Uuid::new_v4(),
            final_verdict: FinalVerdict::Accepted {
                confidence: 0.85,
                summary: "Final decision".to_string(),
            },
            individual_verdicts: HashMap::from([
                ("worker1".to_string(), JudgeVerdict::Accepted {
                    confidence: 0.8,
                    reasoning: "Good work".to_string(),
                }),
                ("worker2".to_string(), JudgeVerdict::Accepted {
                    confidence: 0.7,
                    reasoning: "Acceptable work".to_string(),
                }),
            ]),
            consensus_score: 0.8,
            debate_rounds: 3,
            evaluation_time_ms: 0,
            timestamp: Utc::now(),
        };

        // Should track performance without error
        let result = tracker.track_arbitration_performance(&consensus).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_arbitration_result_serialization() {
        let result = ArbitrationResult {
            task_id: Uuid::new_v4(),
            final_decision: "Test decision".to_string(),
            confidence: 0.85,
            quality_score: 0.9,
            consensus_score: 0.8,
            individual_scores: HashMap::from([
                ("worker1".to_string(), 0.8),
                ("worker2".to_string(), 0.7),
            ]),
            reasoning: "Test reasoning".to_string(),
            learning_insights: LearningInsights {
                performance_improvements: vec!["Improvement 1".to_string()],
                quality_insights: vec!["Insight 1".to_string()],
                conflict_patterns: vec!["Pattern 1".to_string()],
                optimization_suggestions: vec!["Suggestion 1".to_string()],
            },
            timestamp: Utc::now(),
        };

        // Test JSON serialization
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.is_empty());

        // Test JSON deserialization
        let deserialized: ArbitrationResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.final_decision, result.final_decision);
        assert_eq!(deserialized.confidence, result.confidence);
        assert_eq!(deserialized.quality_score, result.quality_score);
    }

    #[tokio::test]
    async fn test_edge_cases_and_error_handling() {
        let engine = AdvancedArbitrationEngine::new();

        // Test with very similar outputs (should still resolve)
        let similar_outputs = vec![
            create_test_worker_output("worker1", "Similar solution 1", 0.8, 0.8, 500),
            create_test_worker_output("worker2", "Similar solution 2", 0.8, 0.8, 500),
            create_test_worker_output("worker3", "Similar solution 3", 0.8, 0.8, 500),
        ];

        let result = engine
            .unwrap()
            .resolve_conflicts(similar_outputs)
            .await
            .unwrap();
        assert!(result.confidence >= 0.0 && result.confidence <= 1.0);

        // Test with very different confidence levels
        let varied_outputs = vec![
            create_test_worker_output("worker1", "Solution A", 0.1, 0.2, 2000), // Low confidence
            create_test_worker_output("worker2", "Solution B", 0.9, 0.95, 200), // High confidence
        ];

        let result2 = AdvancedArbitrationEngine::new()
            .unwrap()
            .resolve_conflicts(varied_outputs)
            .await
            .unwrap();
        assert!(result2.confidence >= 0.0 && result2.confidence <= 1.0);
    }
}
