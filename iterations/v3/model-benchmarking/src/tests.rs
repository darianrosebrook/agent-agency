//! Tests for the model benchmarking system

use crate::*;
use crate::benchmark_runner::BenchmarkConfig;
use crate::metrics_collector::MetricsCollector;
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
    };

    // Store the result
    collector.store_benchmark_result(result.clone()).await?;

    // Retrieve the result
    let stored_results = collector.get_model_benchmark_results(result.model_id).await?;
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
            ranking: 1,
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
            ranking: 1,
        },
    ];

    // Store all results
    for result in results {
        collector.store_benchmark_result(result).await?;
    }

    // Check performance trend
    let trend = collector.calculate_performance_trend(model_id, 5).await?;
    assert_eq!(trend, PerformanceTrend::Improving);

    println!("Performance trend: {:?}", trend);
    Ok(())
}
