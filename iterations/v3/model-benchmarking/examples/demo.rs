//! Demonstration of the model benchmarking system

use model_benchmarking::benchmark_runner::{BenchmarkConfig, BenchmarkRunner};
use model_benchmarking::metrics_collector::MetricsCollector;
use model_benchmarking::*;
use std::time::Duration;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    println!("🚀 Model Benchmarking System Demo");
    println!("==================================");

    // Create a metrics collector
    let collector = MetricsCollector::new();

    // Create test models
    let models = vec![
        ModelSpecification {
            id: Uuid::new_v4(),
            name: "Code Generation Model v1.0".to_string(),
            model_type: ModelType::CodeGeneration,
            parameters: ModelParameters {
                size: 50_000_000, // 50MB
                context_length: 4096,
                training_data: "rust_codebase".to_string(),
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
        },
        ModelSpecification {
            id: Uuid::new_v4(),
            name: "Code Review Model v2.1".to_string(),
            model_type: ModelType::CodeReview,
            parameters: ModelParameters {
                size: 75_000_000, // 75MB
                context_length: 8192,
                training_data: "code_reviews".to_string(),
                architecture: "transformer".to_string(),
            },
            capabilities: vec![Capability {
                capability_type: CapabilityType::CodeReview,
                proficiency_level: ProficiencyLevel::Expert,
                supported_domains: vec!["security".to_string(), "performance".to_string()],
            }],
            constraints: vec![ModelConstraint {
                constraint_type: ConstraintType::MaxTime,
                value: 10.0,
                unit: "seconds".to_string(),
            }],
        },
    ];

    // Create benchmark runner
    let config = BenchmarkConfig {
        iterations: 5,
        warmup_iterations: 2,
        timeout: Duration::from_secs(30),
        verbose: true,
    };
    let runner = BenchmarkRunner::with_config(config);

    println!("\n📊 Running Micro Benchmarks");
    println!("---------------------------");

    // Run micro benchmarks for all models
    for model in &models {
        println!("\n🔍 Testing: {}", model.name);

        let result = runner.run_micro_benchmark(model).await?;
        println!("   Score: {:.2}", result.score);
        println!("   Accuracy: {:.2}", result.metrics.accuracy);
        println!("   Speed: {:.1} ops/sec", result.metrics.speed);
        println!("   Efficiency: {:.2}", result.metrics.efficiency);
        println!("   Quality: {:.2}", result.metrics.quality);

        // Store result
        collector.store_benchmark_result(result).await?;
    }

    println!("\n⚖️ Running Compliance Benchmarks");
    println!("--------------------------------");

    // Run compliance benchmarks for all models
    for model in &models {
        println!("\n🔍 Testing: {}", model.name);

        let result = runner.run_compliance_benchmark(model).await?;
        println!("   Score: {:.2}", result.score);
        println!("   Compliance: {:.2}", result.metrics.compliance);
        println!("   Efficiency: {:.2}", result.metrics.efficiency);
        println!("   Quality: {:.2}", result.metrics.quality);

        // Store result
        collector.store_benchmark_result(result).await?;
    }

    println!("\n📈 Performance Analysis");
    println!("----------------------");

    // Generate performance report
    let report = collector.generate_performance_report().await?;
    println!("Total Models: {}", report.total_models);
    println!("Total Benchmarks: {}", report.total_benchmarks);
    println!("Average Performance: {:.2}", report.average_performance);

    println!("\n🏆 Top Performers:");
    for (i, performer) in report.top_performers.iter().enumerate() {
        println!(
            "   {}. {} - Score: {:.2}",
            i + 1,
            performer.model_name,
            performer.average_score
        );
    }

    println!("\n📊 Performance Distribution:");
    for (category, count) in &report.performance_distribution {
        println!("   {}: {} models", category, count);
    }

    if !report.recommendations.is_empty() {
        println!("\n💡 Recommendations:");
        for (i, rec) in report.recommendations.iter().enumerate() {
            println!("   {}. {} ({:?})", i + 1, rec.description, rec.priority);
        }
    }

    // Show individual model summaries
    println!("\n📋 Individual Model Summaries");
    println!("------------------------------");

    for model in &models {
        if let Some(summary) = collector.get_model_summary(model.id).await? {
            println!("\nModel: {}", summary.model_name);
            println!("   Total Benchmarks: {}", summary.total_benchmarks);
            println!("   Average Score: {:.2}", summary.average_score);
            println!("   Best Score: {:.2}", summary.best_score);
            println!("   Worst Score: {:.2}", summary.worst_score);
            println!("   Performance Trend: {:?}", summary.performance_trend);
            println!("   Last Updated: {}", summary.last_updated);
        }
    }

    println!("\n✅ Demo completed successfully!");
    println!("The benchmarking system is now collecting real performance metrics.");

    Ok(())
}
