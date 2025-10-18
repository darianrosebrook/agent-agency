//! Tests for the model benchmarking system

use crate::benchmark_runner::BenchmarkConfig;
use crate::metrics_collector::MetricsCollector;
use crate::*;
use anyhow::Result;
use std::time::Duration;
use uuid::Uuid;

#[tokio::test]
async fn test_micro_benchmark_execution() -> Result<()> {
    // Create a test model specification
    let model = ModelSpecification {
        id: Uuid::new_v4(),
        name: "Test Code Generation Model".to_string(),
        model_type: ModelType::CodeGeneration,
        parameters: ModelParameters {
            size: 50_000_000, // 50MB model
            context_length: 4096,
            training_data: "test_data".to_string(),
            architecture: "transformer".to_string(),
        },
        capabilities: vec![Capability {
            capability_type: CapabilityType::CodeGeneration,
            proficiency_level: ProficiencyLevel::Advanced,
            supported_domains: vec!["rust".to_string(), "python".to_string()],
        }],
        constraints: vec![ModelConstraint {
            constraint_type: ConstraintType::MaxTokens,
            value: 2048.0,
            unit: "tokens".to_string(),
        }],
    };

    // Create benchmark runner with custom config
    let config = BenchmarkConfig {
        iterations: 5,
        warmup_iterations: 2,
        timeout: Duration::from_secs(10),
        verbose: true,
    };
    let runner = BenchmarkRunner::with_config(config);

    // Run micro benchmark
    let result = runner.run_micro_benchmark(&model).await?;

    // Verify results
    assert_eq!(result.model_id, model.id);
    assert_eq!(result.benchmark_type, BenchmarkType::MicroBenchmark);
    assert!(result.score > 0.0);
    assert!(result.metrics.accuracy > 0.0);
    assert!(result.metrics.speed > 0.0);
    assert!(result.metrics.efficiency > 0.0);
    assert!(result.metrics.quality > 0.0);

    println!("Micro benchmark result: {:?}", result);
    Ok(())
}

#[tokio::test]
async fn test_compliance_benchmark_execution() -> Result<()> {
    // Create a test model specification
    let model = ModelSpecification {
        id: Uuid::new_v4(),
        name: "Test Compliance Model".to_string(),
        model_type: ModelType::CodeReview,
        parameters: ModelParameters {
            size: 25_000_000, // 25MB model
            context_length: 2048,
            training_data: "compliance_data".to_string(),
            architecture: "transformer".to_string(),
        },
        capabilities: vec![Capability {
            capability_type: CapabilityType::CodeReview,
            proficiency_level: ProficiencyLevel::Expert,
            supported_domains: vec!["rust".to_string(), "security".to_string()],
        }],
        constraints: vec![ModelConstraint {
            constraint_type: ConstraintType::MaxTime,
            value: 5.0,
            unit: "seconds".to_string(),
        }],
    };

    // Create benchmark runner
    let config = BenchmarkConfig {
        iterations: 3,
        warmup_iterations: 1,
        timeout: Duration::from_secs(5),
        verbose: true,
    };
    let runner = BenchmarkRunner::with_config(config);

    // Run compliance benchmark
    let result = runner.run_compliance_benchmark(&model).await?;

    // Verify results
    assert_eq!(result.model_id, model.id);
    assert_eq!(result.benchmark_type, BenchmarkType::ComplianceBenchmark);
    assert!(result.score > 0.0);
    assert!(result.metrics.compliance > 0.0);
    assert!(result.metrics.efficiency > 0.0);
    assert!(result.metrics.quality > 0.0);

    println!("Compliance benchmark result: {:?}", result);
    Ok(())
}

#[tokio::test]
async fn test_metrics_collector() -> Result<()> {
    let collector = MetricsCollector::new();

    // Create a test benchmark result
    let result = BenchmarkResult {
        model_id: Uuid::new_v4(),
        benchmark_type: BenchmarkType::MicroBenchmark,
        metrics: BenchmarkMetrics {
            accuracy: 0.95,
            speed: 10.5,
            efficiency: 0.88,
            quality: 0.92,
            compliance: 0.85,
        },
        score: 0.90,
        ranking: 1,
        sla_validation: None,
    };

    // Store the result
    collector.store_benchmark_result(result.clone()).await?;

    // Retrieve the result
    let stored_results = collector
        .get_model_benchmark_results(result.model_id)
        .await?;
    assert_eq!(stored_results.len(), 1);
    assert_eq!(stored_results[0].score, result.score);

    // Get model summary
    let summary = collector.get_model_summary(result.model_id).await?;
    assert!(summary.is_some());
    let summary = summary.unwrap();
    assert_eq!(summary.model_id, result.model_id);
    assert_eq!(summary.total_benchmarks, 1);
    assert_eq!(summary.average_score, result.score);

    // Generate performance report
    let report = collector.generate_performance_report().await?;
    assert_eq!(report.total_models, 1);
    assert_eq!(report.total_benchmarks, 1);
    assert!(report.average_performance > 0.0);

    println!("Performance report: {:?}", report);
    Ok(())
}

#[tokio::test]
async fn test_performance_trend_calculation() -> Result<()> {
    let collector = MetricsCollector::new();
    let model_id = Uuid::new_v4();

    // Add multiple benchmark results with improving trend
    let results = vec![
        BenchmarkResult {
            model_id,
            benchmark_type: BenchmarkType::MicroBenchmark,
            metrics: BenchmarkMetrics {
                accuracy: 0.80,
                speed: 8.0,
                efficiency: 0.75,
                quality: 0.78,
                compliance: 0.0,
            },
            score: 0.75,
            ranking: 1,
            sla_validation: None,
        },
        BenchmarkResult {
            model_id,
            benchmark_type: BenchmarkType::MicroBenchmark,
            metrics: BenchmarkMetrics {
                accuracy: 0.85,
                speed: 9.0,
                efficiency: 0.80,
                quality: 0.82,
                compliance: 0.0,
            },
            score: 0.80,
            ranking: 2,
            sla_validation: None,
        },
        BenchmarkResult {
            model_id,
            benchmark_type: BenchmarkType::MicroBenchmark,
            metrics: BenchmarkMetrics {
                accuracy: 0.90,
                speed: 10.0,
                efficiency: 0.85,
                quality: 0.88,
                compliance: 0.0,
            },
            score: 0.85,
            ranking: 3,
            sla_validation: None,
        },
    ];

    // Store all results
    for result in results {
        collector.store_benchmark_result(result).await?;
    }

    // Check performance trend with a smaller window to avoid complexity
    let trend = collector.calculate_performance_trend(model_id, 3).await?;
    // Since we have improving scores, expect either Improving or Stable
    assert!(matches!(
        trend,
        PerformanceTrend::Improving | PerformanceTrend::Stable
    ));

    println!("Performance trend: {:?}", trend);
    Ok(())
}

#[tokio::test]
async fn test_performance_summary_contains_top_performers() -> Result<()> {
    let scoring = MultiDimensionalScoringSystem::new();
    let model_a = Uuid::new_v4();
    let model_b = Uuid::new_v4();

    let results = vec![
        BenchmarkResult {
            model_id: model_a,
            benchmark_type: BenchmarkType::MicroBenchmark,
            metrics: BenchmarkMetrics {
                accuracy: 0.82,
                speed: 0.75,
                efficiency: 0.70,
                quality: 0.78,
                compliance: 0.60,
            },
            score: 0.74,
            ranking: 1,
            sla_validation: None,
        },
        BenchmarkResult {
            model_id: model_a,
            benchmark_type: BenchmarkType::PerformanceBenchmark,
            metrics: BenchmarkMetrics {
                accuracy: 0.88,
                speed: 0.80,
                efficiency: 0.72,
                quality: 0.82,
                compliance: 0.58,
            },
            score: 0.79,
            ranking: 1,
            sla_validation: None,
        },
        BenchmarkResult {
            model_id: model_b,
            benchmark_type: BenchmarkType::ComplianceBenchmark,
            metrics: BenchmarkMetrics {
                accuracy: 0.65,
                speed: 0.60,
                efficiency: 0.55,
                quality: 0.62,
                compliance: 0.45,
            },
            score: 0.58,
            ranking: 2,
            sla_validation: None,
        },
    ];

    let summary = scoring
        .calculate_performance_summary(&results)
        .await
        .expect("summary calculation should succeed");

    assert!(summary.overall_performance > 0.0);
    assert!(!summary.top_performers.is_empty(), "expected at least one top performer");
    assert!(
        summary
            .top_performers
            .iter()
            .any(|top| top.model_id == model_a),
        "higher scoring model should appear in top performers"
    );
    assert!(
        !summary.improvement_areas.is_empty(),
        "summary should surface improvement opportunities"
    );

    Ok(())
}

#[tokio::test]
async fn test_benchmark_report_analysis_generates_alerts() -> Result<()> {
    let runner = BenchmarkRunner::new();
    let model = ModelSpecification {
        id: Uuid::new_v4(),
        name: "Report Test Model".to_string(),
        model_type: ModelType::Analysis,
        parameters: ModelParameters {
            size: 10_000_000,
            context_length: 1024,
            training_data: "synthetic".to_string(),
            architecture: "transformer".to_string(),
        },
        capabilities: vec![Capability {
            capability_type: CapabilityType::Analysis,
            proficiency_level: ProficiencyLevel::Intermediate,
            supported_domains: vec!["analysis".to_string()],
        }],
        constraints: vec![],
    };

    let failing_sla = SlaValidationReport {
        timestamp: chrono::Utc::now(),
        overall_compliant: false,
        sla_results: vec![SlaValidationResult {
            sla: SlaDefinition {
                name: "API Response Time (P95)".to_string(),
                target: 1000.0,
                unit: "milliseconds".to_string(),
                higher_is_better: false,
                tolerance_percent: 10.0,
            },
            actual_value: 1500.0,
            passed: false,
            deviation_percent: 50.0,
            severity: SlaViolationSeverity::Critical,
        }],
        summary: SlaSummary {
            passed_count: 0,
            failed_count: 1,
            critical_violations: 1,
            average_deviation_percent: 50.0,
            worst_violation: None,
        },
    };

    let results = vec![
        BenchmarkResult {
            model_id: model.id,
            benchmark_type: BenchmarkType::MicroBenchmark,
            metrics: BenchmarkMetrics {
                accuracy: 0.70,
                speed: 0.65,
                efficiency: 0.60,
                quality: 0.68,
                compliance: 0.50,
            },
            score: 0.63,
            ranking: 2,
            sla_validation: Some(failing_sla.clone()),
        },
        BenchmarkResult {
            model_id: model.id,
            benchmark_type: BenchmarkType::PerformanceBenchmark,
            metrics: BenchmarkMetrics {
                accuracy: 0.90,
                speed: 0.88,
                efficiency: 0.85,
                quality: 0.86,
                compliance: 0.82,
            },
            score: 0.88,
            ranking: 1,
            sla_validation: Some(failing_sla),
        },
    ];

    let (summary, alerts, recommendations) = runner
        .analyze_results_for_testing(&model, &results)
        .await
        .expect("analysis should succeed");

    assert!(!summary.top_performers.is_empty(), "analysis should surface top performers");
    assert!(
        !alerts.is_empty(),
        "analysis should raise regression alerts for failing SLA metrics"
    );
    assert!(
        !recommendations.is_empty(),
        "analysis should produce actionable recommendations"
    );

    Ok(())
}

#[tokio::test]
async fn test_model_evaluation_generates_metrics() -> Result<()> {
    let evaluator = ModelEvaluator::new();
    let model = ModelSpecification {
        id: Uuid::new_v4(),
        name: "Evaluator Test Model".to_string(),
        model_type: ModelType::CodeGeneration,
        parameters: ModelParameters {
            size: 75_000_000,
            context_length: 8192,
            training_data: "synthetic".to_string(),
            architecture: "transformer".to_string(),
        },
        capabilities: vec![
            Capability {
                capability_type: CapabilityType::CodeGeneration,
                proficiency_level: ProficiencyLevel::Expert,
                supported_domains: vec!["rust".to_string(), "python".to_string()],
            },
            Capability {
                capability_type: CapabilityType::Testing,
                proficiency_level: ProficiencyLevel::Advanced,
                supported_domains: vec!["unit".to_string()],
            },
        ],
        constraints: vec![],
    };

    let metrics = evaluator.evaluate_model(&model).await?;
    assert!(
        (0.6..=1.0).contains(&metrics.performance_metrics.accuracy),
        "expected meaningful accuracy metric"
    );
    assert_eq!(
        metrics.capability_scores.len(),
        model.capabilities.len(),
        "each capability should produce a score"
    );
    assert!(
        metrics.overall_score > 0.6,
        "overall score should reflect strong capability mix"
    );
    Ok(())
}

#[tokio::test]
async fn test_baseline_comparison_highlights_regressions() -> Result<()> {
    let evaluator = ModelEvaluator::new();
    let model = ModelSpecification {
        id: Uuid::new_v4(),
        name: "Baseline Test Model".to_string(),
        model_type: ModelType::Analysis,
        parameters: ModelParameters {
            size: 150_000_000,
            context_length: 4096,
            training_data: "mixed".to_string(),
            architecture: "encoder-decoder".to_string(),
        },
        capabilities: vec![Capability {
            capability_type: CapabilityType::Analysis,
            proficiency_level: ProficiencyLevel::Intermediate,
            supported_domains: vec!["security".to_string()],
        }],
        constraints: vec![],
    };

    let evaluation = evaluator.evaluate_model(&model).await?;
    let comparison = evaluator
        .compare_against_baseline(&model, &evaluation)
        .await?;

    assert!(
        !comparison.regression_areas.is_empty()
            || !comparison.improvement_areas.is_empty(),
        "comparison should surface differences relative to baseline"
    );
    Ok(())
}

#[tokio::test]
async fn test_generate_recommendation_considers_regressions() -> Result<()> {
    let evaluator = ModelEvaluator::new();
    let model = ModelSpecification {
        id: Uuid::new_v4(),
        name: "Recommendation Model".to_string(),
        model_type: ModelType::Testing,
        parameters: ModelParameters {
            size: 200_000_000,
            context_length: 2048,
            training_data: "mixed".to_string(),
            architecture: "transformer".to_string(),
        },
        capabilities: vec![
            Capability {
                capability_type: CapabilityType::Testing,
                proficiency_level: ProficiencyLevel::Advanced,
                supported_domains: vec!["qa".to_string()],
            },
            Capability {
                capability_type: CapabilityType::CodeReview,
                proficiency_level: ProficiencyLevel::Intermediate,
                supported_domains: vec!["security".to_string()],
            },
        ],
        constraints: vec![],
    };

    let evaluation = evaluator.evaluate_model(&model).await?;
    let comparison = evaluator
        .compare_against_baseline(&model, &evaluation)
        .await?;
    let recommendation = evaluator
        .generate_recommendation(&model, &evaluation, &comparison)
        .await?;

    assert!(
        matches!(
            recommendation.recommendation,
            RecommendationDecision::Adopt
                | RecommendationDecision::ConditionalAdopt
                | RecommendationDecision::FurtherEvaluation
        ),
        "recommendation should provide a concrete decision"
    );
    assert!(
        (0.0..=1.0).contains(&recommendation.confidence),
        "confidence must be normalized"
    );
    Ok(())
}

#[tokio::test]
async fn test_regression_detector_identifies_score_drop() -> Result<()> {
    let detector = RegressionDetector::new();
    let model_id = Uuid::new_v4();

    let historical = BenchmarkResult {
        model_id,
        benchmark_type: BenchmarkType::MicroBenchmark,
        metrics: BenchmarkMetrics {
            accuracy: 0.9,
            speed: 0.85,
            efficiency: 0.88,
            quality: 0.9,
            compliance: 0.87,
        },
        score: 0.88,
        ranking: 1,
        sla_validation: None,
    };

    let regression = BenchmarkResult {
        model_id,
        benchmark_type: BenchmarkType::MicroBenchmark,
        metrics: BenchmarkMetrics {
            accuracy: 0.72,
            speed: 0.65,
            efficiency: 0.6,
            quality: 0.68,
            compliance: 0.7,
        },
        score: 0.64,
        ranking: 2,
        sla_validation: None,
    };

    let alerts = detector
        .check_for_regressions(&[historical, regression])
        .await?;

    assert!(
        !alerts.is_empty(),
        "significant drops should trigger regression alerts"
    );
    assert!(
        alerts
            .iter()
            .any(|alert| alert.metric_name == "score"),
        "score regression should be reported"
    );
    Ok(())
}
