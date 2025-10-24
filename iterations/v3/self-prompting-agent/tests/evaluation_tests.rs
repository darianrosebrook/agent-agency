#[cfg(test)]
mod tests {
    use super::*;
    use self_prompting_agent::evaluation::{EvaluationOrchestrator, Evaluator, EvaluationResult};
    use self_prompting_agent::types::{TaskResult, EvalReport, EvalStatus, TaskType, Artifact, ArtifactType};
    use self_prompting_agent::SelfPromptingAgentError;
    use async_trait::async_trait;
    use chrono::Utc;
    use uuid::Uuid;

    // Mock evaluator for testing
    struct MockEvaluator {
        name: String,
        priority: i32,
        return_score: f64,
        return_status: EvalStatus,
        return_issues: Vec<String>,
    }

    #[async_trait]
    impl Evaluator for MockEvaluator {
        async fn evaluate(&self, _result: &TaskResult) -> Result<EvaluationResult, SelfPromptingAgentError> {
            Ok(EvaluationResult {
                score: self.return_score,
                status: self.return_status.clone(),
                issues: self.return_issues.clone(),
                recommendations: vec!["Fix issues".to_string()],
            })
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn priority(&self) -> i32 {
            self.priority
        }
    }

    fn create_mock_task_result() -> TaskResult {
        TaskResult {
            task_id: Uuid::new_v4(),
            task_type: TaskType::CodeGeneration,
            final_report: EvalReport {
                score: 0.85,
                status: EvalStatus::Pass,
                thresholds_met: vec!["quality".to_string()],
                failed_criteria: vec![],
            },
            execution_time_ms: 1500,
            artifacts: vec![
                Artifact {
                    id: Uuid::new_v4(),
                    file_path: "test.rs".to_string(),
                    content: "fn test() {}".to_string(),
                    artifact_type: ArtifactType::Code,
                    created_at: Utc::now(),
                }
            ],
        }
    }

    #[tokio::test]
    async fn test_evaluation_orchestrator_creation() {
        let orchestrator = EvaluationOrchestrator::new();
        // Just test that it can be created
        assert!(true); // If we get here, creation succeeded
    }

    #[tokio::test]
    async fn test_add_evaluator() {
        let mut orchestrator = EvaluationOrchestrator::new();

        let evaluator = Box::new(MockEvaluator {
            name: "test_evaluator".to_string(),
            priority: 5,
            return_score: 0.9,
            return_status: EvalStatus::Pass,
            return_issues: vec![],
        });

        orchestrator.add_evaluator(evaluator);
        // Test that adding works (we can't inspect internal state directly)
        assert!(true);
    }

    #[tokio::test]
    async fn test_evaluate_no_evaluators() {
        let orchestrator = EvaluationOrchestrator::new();
        let task_result = create_mock_task_result();

        let result = orchestrator.evaluate_result(&task_result).await.unwrap();

        // Should return the task's final report directly
        assert_eq!(result.score, 0.85);
        assert_eq!(result.status, EvalStatus::Pass);
        assert_eq!(result.issues, vec![] as Vec<String>);
        assert_eq!(result.recommendations, vec!["Add more evaluators for comprehensive evaluation".to_string()]);
    }

    #[tokio::test]
    async fn test_evaluate_single_evaluator() {
        let mut orchestrator = EvaluationOrchestrator::new();

        let evaluator = Box::new(MockEvaluator {
            name: "quality_evaluator".to_string(),
            priority: 1,
            return_score: 0.85,
            return_status: EvalStatus::Pass,
            return_issues: vec![],
        });

        orchestrator.add_evaluator(evaluator);
        let task_result = create_mock_task_result();

        let result = orchestrator.evaluate_result(&task_result).await.unwrap();

        // Should return the evaluator's result
        assert_eq!(result.score, 0.85);
        assert_eq!(result.status, EvalStatus::Pass);
        assert_eq!(result.issues, vec![] as Vec<String>);
        assert_eq!(result.recommendations, vec!["Fix issues".to_string()]);
    }

    #[tokio::test]
    async fn test_evaluate_multiple_evaluators() {
        let mut orchestrator = EvaluationOrchestrator::new();

        // Add multiple evaluators
        orchestrator.add_evaluator(Box::new(MockEvaluator {
            name: "quality".to_string(),
            priority: 1,
            return_score: 0.9,
            return_status: EvalStatus::Pass,
            return_issues: vec![],
        }));

        orchestrator.add_evaluator(Box::new(MockEvaluator {
            name: "security".to_string(),
            priority: 2,
            return_score: 0.6,
            return_status: EvalStatus::Partial,
            return_issues: vec!["security_issue".to_string()],
        }));

        let task_result = create_mock_task_result();
        let result = orchestrator.evaluate_result(&task_result).await.unwrap();

        // Should return combined result
        assert_eq!(result.score, 0.75); // Average of 0.9 and 0.6
        assert_eq!(result.status, EvalStatus::Fail); // Worst status wins
        assert_eq!(result.issues, vec!["security_issue".to_string()]);
        assert_eq!(result.recommendations, vec!["Fix issues".to_string(), "Fix issues".to_string()]);
    }

    #[tokio::test]
    async fn test_evaluate_with_failing_evaluator() {
        let mut orchestrator = EvaluationOrchestrator::new();

        let evaluator = Box::new(MockEvaluator {
            name: "failing_evaluator".to_string(),
            priority: 1,
            return_score: 0.0,
            return_status: EvalStatus::Fail,
            return_issues: vec!["critical_error".to_string()],
        });

        orchestrator.add_evaluator(evaluator);
        let task_result = create_mock_task_result();

        let result = orchestrator.evaluate_result(&task_result).await.unwrap();

        assert_eq!(result.score, 0.0);
        assert_eq!(result.status, EvalStatus::Fail);
        assert_eq!(result.issues, vec!["critical_error".to_string()]);
    }

    #[tokio::test]
    async fn test_code_quality_evaluator() {
        let evaluator = self_prompting_agent::evaluation::CodeQualityEvaluator;

        let task_result = create_mock_task_result();
        let result = evaluator.evaluate(&task_result).await.unwrap();

        // Code quality evaluator should return a result
        assert!(result.score >= 0.0 && result.score <= 1.0);
        assert!(matches!(result.status, EvalStatus::Pass | EvalStatus::Partial | EvalStatus::Fail));
    }

    #[tokio::test]
    async fn test_performance_evaluator() {
        let evaluator = self_prompting_agent::evaluation::PerformanceEvaluator;

        let task_result = create_mock_task_result();
        let result = evaluator.evaluate(&task_result).await.unwrap();

        // Performance evaluator should return a result based on execution time
        assert!(result.score >= 0.0 && result.score <= 1.0);
        assert!(matches!(result.status, EvalStatus::Pass | EvalStatus::Partial | EvalStatus::Fail));

        // For 1500ms execution time, should be acceptable but not excellent
        assert!(result.score >= 0.5 && result.score < 0.9);
    }

    #[tokio::test]
    async fn test_performance_evaluator_fast_execution() {
        let evaluator = self_prompting_agent::evaluation::PerformanceEvaluator;

        let mut task_result = create_mock_task_result();
        task_result.execution_time_ms = 50; // Very fast

        let result = evaluator.evaluate(&task_result).await.unwrap();

        // Should get excellent score for fast execution
        assert_eq!(result.score, 1.0);
        assert_eq!(result.status, EvalStatus::Pass);
        assert!(result.issues.is_empty());
    }

    #[tokio::test]
    async fn test_performance_evaluator_slow_execution() {
        let evaluator = self_prompting_agent::evaluation::PerformanceEvaluator;

        let mut task_result = create_mock_task_result();
        task_result.execution_time_ms = 10000; // Very slow

        let result = evaluator.evaluate(&task_result).await.unwrap();

        // Should get poor score for slow execution
        assert_eq!(result.score, 0.3);
        assert_eq!(result.status, EvalStatus::Partial);
        assert!(!result.issues.is_empty());
    }
}
