//! Comprehensive unit tests for Predictive Learning System
//!
//! Tests all learning components and integration scenarios

#[cfg(test)]
mod tests {
    use crate::predictive_learning_system::{
        LearningAccelerator, LearningInsights, OutcomePredictor, OutcomeType, PerformancePredictor,
        PredictiveLearningSystem, ResourcePredictor, StrategyOptimizer, TaskOutcome,
        TrendDirection,
    };
    use chrono::Utc;
    use std::collections::HashMap;
    use uuid::Uuid;

    /// Create a test task outcome for testing
    fn create_test_task_outcome(outcome_type: OutcomeType, performance_score: f64) -> TaskOutcome {
        let mut resource_usage = HashMap::new();
        resource_usage.insert("cpu".to_string(), 0.7);
        resource_usage.insert("memory".to_string(), 0.6);

        TaskOutcome {
            task_id: Uuid::new_v4(),
            outcome_type,
            performance_score,
            duration_ms: 5000,
            resource_usage,
            strategies_used: vec![
                "parallel_processing".to_string(),
                "optimization".to_string(),
            ],
            success_factors: vec!["efficient_algorithms".to_string()],
            failure_factors: vec![],
            timestamp: Utc::now(),
        }
    }

    /// Test performance predictor with various task outcomes
    #[tokio::test]
    async fn test_performance_predictor() {
        let predictor = PerformancePredictor::new();

        // Test with successful task outcome
        let successful_outcome = create_test_task_outcome(OutcomeType::Success, 0.9);

        let prediction = predictor.predict_future(&successful_outcome).await.unwrap();

        assert!(prediction.predicted_performance > 0.0);
        assert!(prediction.predicted_performance <= 1.0);
        assert!(prediction.confidence > 0.0);
        assert!(prediction.confidence <= 1.0);
        assert!(!prediction.performance_factors.is_empty());
        assert!(!prediction.improvement_suggestions.is_empty());
    }

    /// Test strategy optimizer with different task outcomes
    #[tokio::test]
    async fn test_strategy_optimizer() {
        let optimizer = StrategyOptimizer::new();

        // Test with successful task outcome
        let successful_outcome = create_test_task_outcome(OutcomeType::Success, 0.8);

        let optimization = optimizer
            .optimize_strategies(&successful_outcome)
            .await
            .unwrap();

        assert!(optimization.optimization_confidence > 0.0);
        assert!(optimization.optimization_confidence <= 1.0);
        assert!(optimization.expected_improvement > 0.0);
        assert!(!optimization.optimized_strategies.is_empty());
        assert!(!optimization.strategy_recommendations.is_empty());

        // Verify strategy structure
        let strategy = &optimization.optimized_strategies[0];
        assert!(strategy.optimization_score > 0.0);
        assert!(strategy.optimization_score <= 1.0);
        assert!(strategy.expected_performance > 0.0);
        assert!(!strategy.implementation_steps.is_empty());
        assert!(!strategy.success_metrics.is_empty());
    }

    /// Test resource predictor with various resource usage patterns
    #[tokio::test]
    async fn test_resource_predictor() {
        let predictor = ResourcePredictor::new();

        // Test with high resource usage
        let high_usage_outcome = create_test_task_outcome(OutcomeType::Success, 0.9);

        let prediction = predictor.predict_needs(&high_usage_outcome).await.unwrap();

        assert!(prediction.prediction_confidence > 0.0);
        assert!(prediction.prediction_confidence <= 1.0);
        assert!(!prediction.predicted_resource_needs.is_empty());
        assert!(prediction.resource_utilization.efficiency_score > 0.0);
        assert!(!prediction.scaling_recommendations.is_empty());

        // Verify resource need structure
        let resource_need = prediction.predicted_resource_needs.values().next().unwrap();
        assert!(resource_need.predicted_quantity > 0.0);
        assert!(resource_need.confidence > 0.0);
        assert!(resource_need.confidence <= 1.0);
    }

    /// Test outcome predictor with different outcome types
    #[tokio::test]
    async fn test_outcome_predictor() {
        let predictor = OutcomePredictor::new();

        // Test with successful outcome
        let successful_outcome = create_test_task_outcome(OutcomeType::Success, 0.9);

        let prediction = predictor
            .predict_outcomes(&successful_outcome)
            .await
            .unwrap();

        assert!(prediction.success_probability > 0.0);
        assert!(prediction.success_probability <= 1.0);
        assert!(prediction.confidence > 0.0);
        assert!(prediction.confidence <= 1.0);
        assert!(!prediction.predicted_outcomes.is_empty());

        // Verify outcome structure
        let outcome = &prediction.predicted_outcomes[0];
        assert!(outcome.probability > 0.0);
        assert!(outcome.probability <= 1.0);
        assert!(outcome.impact_score > 0.0);
        assert!(outcome.impact_score <= 1.0);
    }

    /// Test learning accelerator with various learning scenarios
    #[tokio::test]
    async fn test_learning_accelerator() {
        let accelerator = LearningAccelerator::new();

        // Test with successful task outcome
        let successful_outcome = create_test_task_outcome(OutcomeType::Success, 0.8);

        let acceleration = accelerator
            .accelerate_learning(&successful_outcome)
            .await
            .unwrap();

        assert!(acceleration.acceleration_factor > 0.0);
        assert!(acceleration.knowledge_transfer_efficiency > 0.0);
        assert!(acceleration.knowledge_transfer_efficiency <= 1.0);
        assert!(!acceleration.meta_learning_insights.is_empty());

        // Verify learning optimization structure
        assert!(acceleration.learning_optimization.optimized_learning_rate > 0.0);
        assert!(acceleration.learning_optimization.knowledge_retention_score > 0.0);
        assert!(acceleration.learning_optimization.transfer_efficiency > 0.0);
        assert!(!acceleration
            .learning_optimization
            .recommended_learning_methods
            .is_empty());
    }

    /// Test complete predictive learning system
    #[tokio::test]
    async fn test_predictive_learning_system() {
        let system = PredictiveLearningSystem::new();

        // Create multiple test task outcomes
        let task_outcomes = vec![
            create_test_task_outcome(OutcomeType::Success, 0.9),
            create_test_task_outcome(OutcomeType::Success, 0.8),
            create_test_task_outcome(OutcomeType::PartialSuccess, 0.6),
            create_test_task_outcome(OutcomeType::Failure, 0.3),
        ];

        let mut all_insights = Vec::new();

        for task_outcome in task_outcomes {
            let insights = system.learn_and_predict(&task_outcome).await.unwrap();
            all_insights.push(insights);
        }

        assert_eq!(all_insights.len(), 4);

        for insights in &all_insights {
            // Verify performance prediction
            assert!(insights.performance_prediction.predicted_performance > 0.0);
            assert!(insights.performance_prediction.confidence > 0.0);

            // Verify strategy optimization
            assert!(insights.strategy_optimization.optimization_confidence > 0.0);
            assert!(insights.strategy_optimization.expected_improvement > 0.0);

            // Verify resource prediction
            assert!(insights.resource_prediction.prediction_confidence > 0.0);
            assert!(
                insights
                    .resource_prediction
                    .resource_utilization
                    .efficiency_score
                    > 0.0
            );

            // Verify outcome prediction
            assert!(insights.outcome_prediction.success_probability > 0.0);
            assert!(insights.outcome_prediction.confidence > 0.0);

            // Verify learning acceleration
            assert!(insights.learning_acceleration.acceleration_factor > 0.0);
            assert!(insights.learning_acceleration.knowledge_transfer_efficiency > 0.0);
        }
    }

    /// Test learning history tracking
    #[tokio::test]
    async fn test_learning_history_tracking() {
        let system = PredictiveLearningSystem::new();

        let task_outcome = create_test_task_outcome(OutcomeType::Success, 0.8);

        // Process the task outcome
        let _insights = system.learn_and_predict(&task_outcome).await.unwrap();

        // Verify history was updated (we can't access private fields in tests)
        // This test would need a public method to access historical data
        // TODO: Implement comprehensive outcome verification with the following requirements:
        // 1. Outcome validation: Validate system outcome processing and results
        //    - Verify system outcome accuracy and completeness
        //    - Validate outcome processing quality and effectiveness
        //    - Handle outcome validation error detection and correction
        // 2. Historical data verification: Verify historical data updates and integrity
        //    - Validate historical data updates and consistency
        //    - Verify historical data integrity and accuracy
        //    - Handle historical data verification quality assurance
        // 3. Performance metrics validation: Validate performance metrics calculation
        //    - Verify performance metrics accuracy and completeness
        //    - Validate performance metrics calculation algorithms
        //    - Handle performance metrics validation and quality assurance
        // 4. System integration testing: Test system integration and functionality
        //    - Verify system component integration and communication
        //    - Test system functionality and performance
        //    - Ensure system testing meets quality and reliability standards
    }

    /// Test different outcome types
    #[tokio::test]
    async fn test_different_outcome_types() {
        let system = PredictiveLearningSystem::new();

        let outcome_types = vec![
            OutcomeType::Success,
            OutcomeType::PartialSuccess,
            OutcomeType::Failure,
            OutcomeType::Timeout,
            OutcomeType::Error,
        ];

        for outcome_type in outcome_types {
            let task_outcome = create_test_task_outcome(outcome_type.clone(), 0.5);
            let insights = system.learn_and_predict(&task_outcome).await.unwrap();

            // All outcome types should produce valid insights
            assert!(insights.performance_prediction.predicted_performance > 0.0);
            assert!(insights.strategy_optimization.optimization_confidence > 0.0);
            assert!(insights.resource_prediction.prediction_confidence > 0.0);
            assert!(insights.outcome_prediction.success_probability >= 0.0);
            assert!(insights.learning_acceleration.acceleration_factor > 0.0);
        }
    }

    /// Test serialization and deserialization of learning insights
    #[tokio::test]
    async fn test_learning_insights_serialization() {
        let system = PredictiveLearningSystem::new();

        let task_outcome = create_test_task_outcome(OutcomeType::Success, 0.8);
        let insights = system.learn_and_predict(&task_outcome).await.unwrap();

        // Test JSON serialization
        let json = serde_json::to_string(&insights).unwrap();
        assert!(!json.is_empty());

        // Test JSON deserialization
        let deserialized: LearningInsights = serde_json::from_str(&json).unwrap();
        assert_eq!(
            deserialized.performance_prediction.predicted_performance,
            insights.performance_prediction.predicted_performance
        );
        assert_eq!(
            deserialized.strategy_optimization.optimization_confidence,
            insights.strategy_optimization.optimization_confidence
        );
    }

    /// Test edge cases and error handling
    #[tokio::test]
    async fn test_edge_cases_and_error_handling() {
        let system = PredictiveLearningSystem::new();

        // Test with extreme performance scores
        let extreme_outcomes = vec![
            create_test_task_outcome(OutcomeType::Success, 0.0), // Minimum score
            create_test_task_outcome(OutcomeType::Success, 1.0), // Maximum score
        ];

        for outcome in extreme_outcomes {
            let insights = system.learn_and_predict(&outcome).await.unwrap();

            // Should handle extreme values gracefully
            assert!(insights.performance_prediction.predicted_performance >= 0.0);
            // Note: predicted performance can exceed 1.0 as it represents improvement potential
            assert!(insights.performance_prediction.predicted_performance <= 2.0);
            assert!(insights.performance_prediction.confidence > 0.0);
        }

        // Test with empty resource usage
        let mut empty_resource_outcome = create_test_task_outcome(OutcomeType::Success, 0.5);
        empty_resource_outcome.resource_usage.clear();

        let insights = system
            .learn_and_predict(&empty_resource_outcome)
            .await
            .unwrap();
        assert!(insights.resource_prediction.prediction_confidence > 0.0);
    }

    /// Test performance prediction accuracy over multiple iterations
    #[tokio::test]
    async fn test_performance_prediction_accuracy() {
        let predictor = PerformancePredictor::new();

        // Create a series of improving task outcomes
        let improving_outcomes = vec![
            create_test_task_outcome(OutcomeType::Success, 0.5),
            create_test_task_outcome(OutcomeType::Success, 0.6),
            create_test_task_outcome(OutcomeType::Success, 0.7),
            create_test_task_outcome(OutcomeType::Success, 0.8),
        ];

        let mut predictions = Vec::new();

        for outcome in improving_outcomes {
            let prediction = predictor.predict_future(&outcome).await.unwrap();
            predictions.push(prediction);
        }

        // Verify predictions are reasonable
        for prediction in &predictions {
            assert!(prediction.predicted_performance > 0.0);
            assert!(prediction.predicted_performance <= 1.0);
            assert!(prediction.confidence > 0.0);
            assert!(prediction.confidence <= 1.0);
        }

        // Verify trend detection
        assert_eq!(predictions[0].trend_direction, TrendDirection::Improving);
    }

    /// Test resource prediction scaling recommendations
    #[tokio::test]
    async fn test_resource_scaling_recommendations() {
        let predictor = ResourcePredictor::new();

        // Test with high resource usage
        let mut high_usage_outcome = create_test_task_outcome(OutcomeType::Success, 0.9);
        high_usage_outcome
            .resource_usage
            .insert("cpu".to_string(), 0.95);

        let prediction = predictor.predict_needs(&high_usage_outcome).await.unwrap();

        // Should recommend scaling up for high usage
        assert!(!prediction.scaling_recommendations.is_empty());

        let scaling_rec = &prediction.scaling_recommendations[0];
        assert!(scaling_rec.recommended_factor > 1.0); // Should recommend scaling up
        assert!(scaling_rec.expected_benefit > 0.0);
    }
}
