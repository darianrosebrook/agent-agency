//! Integration tests for the self-governing agent system
//!
//! These tests verify that all components work together correctly
//! and that the agent can execute complete workflows end-to-end.

use self_prompting_agent::*;
use std::time::Duration;
use tokio::time::timeout;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_self_prompting_workflow() {
        let config = SelfPromptingConfig {
            max_iterations: 3,
            quality_threshold: 0.8,
            enable_learning: false, // Disable for predictable testing
            ..Default::default()
        };

        let agent = SelfPromptingAgent::new(config).await
            .expect("Failed to create agent");

        // Create a test task with a simple fixable issue
        let task = create_test_task();

        // Execute with timeout to prevent hanging
        let result = timeout(
            Duration::from_secs(60),
            agent.execute_task(task)
        ).await
        .expect("Execution timed out")
        .expect("Execution failed");

        // Verify basic execution properties
        assert!(result.success, "Execution should succeed");
        assert!(result.final_quality_score > 0.0, "Quality score should be positive");
        assert!(result.iterations.len() > 0, "Should have at least one iteration");
        assert!(result.iterations.len() <= 3, "Should not exceed max iterations");

        // Verify artifacts were generated
        assert!(!result.final_artifacts.is_empty(), "Should produce final artifacts");

        // Check that some form of evaluation occurred
        assert!(result.evaluation_results.len() > 0, "Should have evaluation results");
    }

    #[tokio::test]
    async fn test_sandbox_safety_integration() {
        let agent = SelfPromptingAgent::new(SelfPromptingConfig::default()).await
            .expect("Failed to create agent");

        // Create a task that tries to access forbidden paths
        let dangerous_task = Task {
            id: uuid::Uuid::new_v4(),
            description: "Try to access /etc/passwd".to_string(),
            context: vec![],
            working_spec: WorkingSpec::default(),
        };

        let result = agent.execute_task(dangerous_task).await
            .expect("Execution should complete even with violations");

        // Verify sandbox violations were recorded
        assert!(!result.sandbox_violations.is_empty(),
               "Should detect sandbox violations");

        // Verify the task still completed (safety doesn't prevent execution)
        assert!(result.iterations.len() > 0,
               "Should still attempt execution despite violations");
    }

    #[tokio::test]
    async fn test_model_hot_swap_integration() {
        let agent = SelfPromptingAgent::new(SelfPromptingConfig::default()).await
            .expect("Failed to create agent");

        // Test model switching
        agent.set_active_model("ollama").await
            .expect("Should switch to ollama");

        let task = create_simple_task();
        let result1 = agent.execute_task(task).await
            .expect("Execution with ollama should succeed");

        agent.set_active_model("coreml").await
            .expect("Should switch to coreml");

        let task2 = create_simple_task();
        let result2 = agent.execute_task(task2).await
            .expect("Execution with coreml should succeed");

        // Both executions should have succeeded
        assert!(result1.success);
        assert!(result2.success);
    }

    #[tokio::test]
    async fn test_learning_system_integration() {
        let config = SelfPromptingConfig {
            enable_learning: true,
            ..Default::default()
        };

        let agent = SelfPromptingAgent::new(config).await
            .expect("Failed to create agent");

        // Execute multiple similar tasks to allow learning
        for i in 0..3 {
            let task = create_learning_test_task(i);
            let result = agent.execute_task(task).await
                .expect("Learning task execution should succeed");

            assert!(result.success);
            assert!(result.learning_signals.len() > 0,
                   "Should generate learning signals");
        }

        // Verify learning metrics are available
        let metrics = agent.get_learning_metrics().await
            .expect("Should retrieve learning metrics");

        assert!(metrics.iteration_efficiency >= 0.0);
        assert!(metrics.satisficing_accuracy >= 0.0);
    }

    // Helper functions for creating test data

    fn create_test_task() -> Task {
        Task {
            id: uuid::Uuid::new_v4(),
            description: "Fix the syntax error in this code".to_string(),
            context: vec![Artifact {
                content: r#"fn main() {
    println!("unclosed string);
}"#.to_string(),
                artifact_type: ArtifactType::Code,
            }],
            working_spec: WorkingSpec {
                acceptance_criteria: vec![
                    "Code compiles without syntax errors".to_string(),
                ],
                risk_tier: RiskTier::Tier3,
                ..Default::default()
            },
        }
    }

    fn create_simple_task() -> Task {
        Task {
            id: uuid::Uuid::new_v4(),
            description: "Add documentation to this function".to_string(),
            context: vec![Artifact {
                content: r#"fn calculate_total(items: Vec<i32>) -> i32 {
    items.iter().sum()
}"#.to_string(),
                artifact_type: ArtifactType::Code,
            }],
            working_spec: WorkingSpec::default(),
        }
    }

    fn create_learning_test_task(index: usize) -> Task {
        Task {
            id: uuid::Uuid::new_v4(),
            description: format!("Learning task {}", index),
            context: vec![Artifact {
                content: format!(r#"fn task_{}() {{
    // This is learning task {}
    println!("Task {} executed");
}}"#, index, index, index),
                artifact_type: ArtifactType::Code,
            }],
            working_spec: WorkingSpec {
                risk_tier: RiskTier::Tier3,
                ..Default::default()
            },
        }
    }
}
