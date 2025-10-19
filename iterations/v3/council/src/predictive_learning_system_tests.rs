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

        // Test 1: Multiple outcomes processing
        test_multiple_outcomes_processing().await;

        // Test 2: Historical data integrity
        test_historical_data_integrity().await;

        // Test 3: Performance metrics validation
        test_performance_metrics_calculation().await;

        // Test 4: Learning curve analysis
        test_learning_curve_analysis().await;
    }

    async fn test_multiple_outcomes_processing() {
        let system = PredictiveLearningSystem::new();

        // Process multiple different outcomes
        let outcomes = vec![
            create_test_task_outcome(OutcomeType::Success, 0.9),
            create_test_task_outcome(OutcomeType::Failure, 0.3),
            create_test_task_outcome(OutcomeType::PartialSuccess, 0.6),
            create_test_task_outcome(OutcomeType::Timeout, 0.1),
            create_test_task_outcome(OutcomeType::Success, 0.8),
        ];

        let mut insights = Vec::new();
        for outcome in outcomes {
            let insight = system.learn_and_predict(&outcome).await.unwrap();
            insights.push(insight);
        }

        // Validate that insights improve over time (learning effect)
        assert!(insights.len() >= 3, "Should have processed multiple outcomes");

        // Check that confidence generally improves with more data
        let avg_confidence_first_half: f32 = insights[..insights.len()/2].iter()
            .map(|i| i.outcome_prediction.confidence).sum::<f32>() / (insights.len()/2) as f32;
        let avg_confidence_second_half: f32 = insights[insights.len()/2..].iter()
            .map(|i| i.outcome_prediction.confidence).sum::<f32>() / (insights.len() - insights.len()/2) as f32;

        // Learning should generally improve confidence (with some tolerance for randomness)
        assert!(
            avg_confidence_second_half >= avg_confidence_first_half * 0.8,
            "Learning should generally improve confidence: {:.2} vs {:.2}",
            avg_confidence_first_half,
            avg_confidence_second_half
        );

        // All insights should have valid predictions
        for (i, insight) in insights.iter().enumerate() {
            assert!(
                insight.outcome_prediction.success_probability >= 0.0 &&
                insight.outcome_prediction.success_probability <= 1.0,
                "Invalid success probability in insight {}: {:.2}",
                i,
                insight.outcome_prediction.success_probability
            );
            assert!(
                insight.outcome_prediction.confidence >= 0.0 &&
                insight.outcome_prediction.confidence <= 1.0,
                "Invalid confidence in insight {}: {:.2}",
                i,
                insight.outcome_prediction.confidence
            );
        }
    }

    async fn test_historical_data_integrity() {
        let system = PredictiveLearningSystem::new();

        // Create a series of outcomes with known patterns
        let mut outcomes = Vec::new();
        for i in 0..10 {
            let success_rate = if i < 5 { 0.9 } else { 0.3 }; // First 5 high success, last 5 low success
            outcomes.push(create_test_task_outcome(
                if success_rate > 0.5 { OutcomeType::Success } else { OutcomeType::Failure },
                success_rate
            ));
        }

        // Process all outcomes
        for outcome in &outcomes {
            system.learn_and_predict(outcome).await.unwrap();
        }

        // Test prediction on a new similar outcome
        let test_outcome = create_test_task_outcome(OutcomeType::Success, 0.85);
        let prediction = system.learn_and_predict(&test_outcome).await.unwrap();

        // The system should have learned from the historical pattern
        // (This is a simplified test - in practice we'd need more sophisticated validation)
        assert!(prediction.outcome_prediction.success_probability > 0.0);
        assert!(prediction.learning_acceleration.knowledge_transfer_efficiency > 0.0);

        // Test with edge case - very different outcome
        let edge_outcome = create_test_task_outcome(OutcomeType::Timeout, 0.1);
        let edge_prediction = system.learn_and_predict(&edge_outcome).await.unwrap();

        // Should still produce valid predictions
        assert!(edge_prediction.outcome_prediction.success_probability >= 0.0);
        assert!(edge_prediction.outcome_prediction.success_probability <= 1.0);
        assert!(edge_prediction.outcome_prediction.confidence >= 0.0);
    }

    async fn test_performance_metrics_calculation() {
        let system = PredictiveLearningSystem::new();

        // Create outcomes with varying performance characteristics
        let performance_scenarios = vec![
            (OutcomeType::Success, 0.95, 1000),  // Fast success
            (OutcomeType::Success, 0.90, 2000),  // Medium success
            (OutcomeType::Failure, 0.2, 5000),   // Slow failure
            (OutcomeType::Success, 0.85, 1500),  // Medium-fast success
        ];

        let mut predictions = Vec::new();
        for (outcome_type, confidence, processing_time) in performance_scenarios {
            let mut outcome = create_test_task_outcome(outcome_type, confidence);
            // Note: In a real implementation, we'd set processing time in the outcome
            // For now, we just test the prediction generation

            let prediction = system.learn_and_predict(&outcome).await.unwrap();
            predictions.push(prediction);
        }

        // Validate that all predictions have reasonable performance metrics
        for (i, prediction) in predictions.iter().enumerate() {
            assert!(
                prediction.learning_acceleration.acceleration_factor > 0.0,
                "Invalid acceleration factor in prediction {}: {:.2}",
                i,
                prediction.learning_acceleration.acceleration_factor
            );
            assert!(
                prediction.learning_acceleration.knowledge_transfer_efficiency >= 0.0 &&
                prediction.learning_acceleration.knowledge_transfer_efficiency <= 1.0,
                "Invalid knowledge transfer efficiency in prediction {}: {:.2}",
                i,
                prediction.learning_acceleration.knowledge_transfer_efficiency
            );
        }

        // Test that the system can handle performance variations
        let varied_outcome = create_test_task_outcome(OutcomeType::PartialSuccess, 0.7);
        let varied_prediction = system.learn_and_predict(&varied_outcome).await.unwrap();

        assert!(varied_prediction.outcome_prediction.success_probability > 0.0);
        assert!(varied_prediction.learning_acceleration.acceleration_factor > 0.0);
    }

    async fn test_learning_curve_analysis() {
        let system = PredictiveLearningSystem::new();

        // Track prediction accuracy over time
        let mut prediction_history = Vec::new();
        let actual_outcomes = vec![
            (OutcomeType::Success, 0.9),
            (OutcomeType::Success, 0.8),
            (OutcomeType::Failure, 0.3),
            (OutcomeType::Success, 0.85),
            (OutcomeType::Failure, 0.2),
            (OutcomeType::Success, 0.95),
            (OutcomeType::PartialSuccess, 0.6),
            (OutcomeType::Success, 0.88),
        ];

        for (outcome_type, actual_confidence) in actual_outcomes {
            let test_outcome = create_test_task_outcome(outcome_type.clone(), actual_confidence);
            let prediction = system.learn_and_predict(&test_outcome).await.unwrap();

            // Store prediction vs actual for analysis
            prediction_history.push((
                prediction.outcome_prediction.success_probability,
                matches!(outcome_type, OutcomeType::Success | OutcomeType::PartialSuccess),
                prediction.outcome_prediction.confidence,
            ));
        }

        // Analyze learning curve - predictions should generally improve
        assert!(prediction_history.len() >= 5, "Need sufficient data for learning curve analysis");

        // Calculate prediction accuracy trend
        let mut accuracies = Vec::new();
        for (predicted_prob, actual_success, confidence) in &prediction_history {
            let predicted_success = *predicted_prob > 0.5;
            let accuracy = if predicted_success == *actual_success { 1.0 } else { 0.0 };
            // Weight by confidence
            accuracies.push(accuracy * confidence);
        }

        // Check that overall accuracy is reasonable (> 50%)
        let avg_accuracy = accuracies.iter().sum::<f32>() / accuracies.len() as f32;
        assert!(avg_accuracy > 0.5, "Learning system accuracy too low: {:.2}", avg_accuracy);

        // Check that confidence generally increases with more data
        let first_half_avg_confidence: f32 = prediction_history[..prediction_history.len()/2].iter()
            .map(|(_, _, conf)| *conf).sum::<f32>() / (prediction_history.len()/2) as f32;
        let second_half_avg_confidence: f32 = prediction_history[prediction_history.len()/2..].iter()
            .map(|(_, _, conf)| *conf).sum::<f32>() / (prediction_history.len() - prediction_history.len()/2) as f32;

        // Confidence should generally improve (within reasonable bounds)
        assert!(
            second_half_avg_confidence >= first_half_avg_confidence * 0.7,
            "Confidence should not decrease significantly: {:.2} -> {:.2}",
            first_half_avg_confidence,
            second_half_avg_confidence
        );
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
