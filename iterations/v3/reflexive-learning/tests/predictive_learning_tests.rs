use chrono::{Duration, Utc};
use reflexive_learning::coordinator::{
    Action, ActionType, Feedback, FeedbackSource, LearningConfig, MultiTurnLearningCoordinator,
    Outcome, ResourceUsage, TurnData,
};
use reflexive_learning::{
    Constraint, ConstraintSeverity, ConstraintType, CriterionType, FailureCategory,
    LearningStrategy, LearningTask, PerformanceTrends, PredictiveLearningConfig,
    PredictiveLearningSystem, ProgressMetrics, QualityIndicator, ResourcePressureLevel,
    ResourceUtilization, RiskLevel, StrategyAdjustmentFocus, SuccessCriterion, TaskComplexity,
    TaskContext, TaskLearningSnapshot, TaskOutcome, TaskType, TrendData, TrendDirection,
};
use serde_json::json;
use uuid::Uuid;

fn default_progress() -> ProgressMetrics {
    ProgressMetrics {
        completion_percentage: 62.0,
        quality_score: 0.88,
        efficiency_score: 0.82,
        error_rate: 0.01,
        learning_velocity: 0.12,
    }
}

fn resource_intensive_utilization() -> ResourceUtilization {
    ResourceUtilization {
        cpu_usage: 0.92,
        memory_usage: 0.85,
        token_usage: 0.78,
        time_usage: 0.9,
        efficiency_ratio: 0.45,
    }
}

#[tokio::test]
async fn predictive_learning_high_quality_outcome_targets_low_risk() {
    let system = PredictiveLearningSystem::with_defaults();

    let outcome = TaskOutcome::Success {
        confidence: 0.92,
        quality_indicators: vec![
            QualityIndicator::HighConfidence,
            QualityIndicator::StrongCAWSCompliance,
            QualityIndicator::EfficientExecution,
        ],
    };

    let snapshot = TaskLearningSnapshot::from_outcome(outcome.clone())
        .with_progress(default_progress())
        .with_turn_count(4);

    let insights = system
        .learn_and_predict(&snapshot)
        .await
        .expect("Predictive learning should succeed for success outcome");

    assert!(
        insights.performance.success_probability > 0.9,
        "Expected high success probability, got {:?}",
        insights.performance.success_probability
    );
    assert_eq!(
        insights.performance.risk_level,
        RiskLevel::Low,
        "High quality should result in low risk"
    );
    assert_eq!(
        insights.strategy.recommended_strategy,
        LearningStrategy::Adaptive,
        "High quality tasks should continue with adaptive strategy"
    );
    assert_eq!(
        insights.resources.pressure_level,
        ResourcePressureLevel::Low,
        "Efficient execution should predict low resource pressure"
    );
}

#[tokio::test]
async fn predictive_learning_partial_success_pushes_quality_adjustments() {
    let system = PredictiveLearningSystem::new(PredictiveLearningConfig {
        success_baseline: 0.7,
        partial_penalty: 0.2,
        failure_penalty: 0.4,
        timeout_penalty: 0.25,
        completion_baseline_ms: 75_000,
    });

    let outcome = TaskOutcome::PartialSuccess {
        issues: vec![
            "Incomplete verification".to_string(),
            "Missing follow-up tests".to_string(),
        ],
        confidence: 0.62,
        remediation_applied: true,
    };

    let snapshot = TaskLearningSnapshot::from_outcome(outcome.clone())
        .with_progress(ProgressMetrics {
            completion_percentage: 55.0,
            quality_score: 0.58,
            efficiency_score: 0.54,
            error_rate: 0.12,
            learning_velocity: 0.07,
        })
        .with_turn_count(6);

    let insights = system
        .learn_and_predict(&snapshot)
        .await
        .expect("Predictive learning should succeed for partial success");

    assert_eq!(
        insights.strategy.recommended_strategy,
        LearningStrategy::Adaptive,
        "Partial success should stay adaptive for refinement"
    );
    assert!(
        insights
            .strategy
            .adjustments
            .iter()
            .any(|a| matches!(a.focus, StrategyAdjustmentFocus::Quality)),
        "Strategy adjustments should prioritize quality improvements"
    );
    assert!(
        insights.performance.success_probability < 0.85,
        "Partial success should yield moderated success probability"
    );
}

#[tokio::test]
async fn coordinator_process_turn_emits_predictive_insights() {
    let mut coordinator = MultiTurnLearningCoordinator::new(LearningConfig::default());

    let task = LearningTask {
        id: Uuid::new_v4(),
        task_type: TaskType::CodeGeneration,
        complexity: TaskComplexity::Moderate,
        expected_duration: Duration::minutes(5),
        success_criteria: vec![SuccessCriterion {
            criterion_type: CriterionType::Quality,
            description: "Maintain Tier-1 quality standards".to_string(),
            measurable: true,
            weight: 1.0,
        }],
        context: TaskContext {
            domain: "agentic-systems".to_string(),
            technology_stack: vec!["rust".to_string()],
            constraints: vec![Constraint {
                constraint_type: ConstraintType::Quality,
                description: "Maintain 0.9+ quality score".to_string(),
                severity: ConstraintSeverity::Hard,
            }],
            historical_performance: None,
        },
    };

    let mut session = coordinator
        .start_session(task)
        .await
        .expect("session start should succeed");

    let turn_data = TurnData {
        turn_number: 1,
        action_taken: Action {
            action_type: ActionType::CodeGeneration,
            parameters: json!({ "prompt": "Implement predictive analysis" }),
            resource_usage: ResourceUsage {
                cpu_time: Duration::seconds(12),
                memory_usage: 4_096,
                token_usage: 8_000,
                network_usage: 1_200,
            },
        },
        outcome: Outcome {
            success: true,
            quality_score: 0.9,
            efficiency_score: 0.82,
            error_count: 0,
            feedback: vec![Feedback {
                source: FeedbackSource::Council,
                content: "Comprehensive evidence compiled".to_string(),
                confidence: 0.92,
                timestamp: Utc::now(),
            }],
        },
        performance_metrics: PerformanceTrends {
            short_term: TrendData {
                direction: TrendDirection::Improving,
                magnitude: 0.2,
                confidence: 0.8,
                data_points: 3,
            },
            medium_term: TrendData {
                direction: TrendDirection::Improving,
                magnitude: 0.15,
                confidence: 0.75,
                data_points: 6,
            },
            long_term: TrendData {
                direction: TrendDirection::Stable,
                magnitude: 0.05,
                confidence: 0.6,
                data_points: 10,
            },
        },
        context_changes: Vec::new(),
    };

    let result = coordinator
        .process_turn(&mut session, turn_data)
        .await
        .expect("turn processing should succeed");

    let predictive = result
        .predictive_insights
        .expect("predictive insights should be present");

    assert!(
        predictive.performance.success_probability > 0.7,
        "expected meaningful success probability, got {:?}",
        predictive.performance.success_probability
    );
    assert!(
        predictive.strategy.confidence > 0.6,
        "strategy optimizer should emit confident recommendation"
    );
}

#[tokio::test]
async fn predictive_learning_detects_resource_pressure_for_failures() {
    let system = PredictiveLearningSystem::with_defaults();

    let outcome = TaskOutcome::Failure {
        reason: "Workers exhausted available compute".to_string(),
        failure_category: FailureCategory::ResourceExhaustion,
        recoverable: true,
    };

    let snapshot = TaskLearningSnapshot::from_outcome(outcome.clone())
        .with_resources(resource_intensive_utilization())
        .with_turn_count(8);

    let insights = system
        .learn_and_predict(&snapshot)
        .await
        .expect("Predictive learning should succeed for resource failure");

    assert!(
        insights.performance.success_probability < 0.5,
        "Resource exhaustion should lower success probability"
    );
    assert_eq!(
        insights.strategy.recommended_strategy,
        LearningStrategy::Conservative,
        "Resource issues should move toward conservative strategy"
    );
    assert_eq!(
        insights.resources.pressure_level,
        ResourcePressureLevel::Critical,
        "Resource exhaustion should be flagged as critical pressure"
    );
    assert!(
        insights.resources.expected_cpu_usage > 0.85,
        "Predicted CPU usage should remain high when exhaustion occurs"
    );
}

#[tokio::test]
async fn predictive_learning_handles_timeouts_with_longer_completion_prediction() {
    let system = PredictiveLearningSystem::with_defaults();

    let outcome = TaskOutcome::Timeout {
        duration_ms: 120_000,
        partial_results: None,
    };

    let snapshot = TaskLearningSnapshot::from_outcome(outcome)
        .with_progress(ProgressMetrics {
            completion_percentage: 40.0,
            quality_score: 0.5,
            efficiency_score: 0.48,
            error_rate: 0.2,
            learning_velocity: 0.05,
        })
        .with_turn_count(5);

    let insights = system
        .learn_and_predict(&snapshot)
        .await
        .expect("Predictive learning should succeed for timeout");

    assert!(
        insights.performance.predicted_completion_time_ms > 90_000,
        "Timeout should extend predicted completion time"
    );
    assert!(
        matches!(
            insights.resources.pressure_level,
            ResourcePressureLevel::Moderate | ResourcePressureLevel::High
        ),
        "Timeouts should reflect elevated resource pressure"
    );
}
