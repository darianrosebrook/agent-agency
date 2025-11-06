//! Comprehensive E2E Test Runner for Agent Agency V3
//!
//! Runs various E2E test scenarios that validate autonomous agent execution
//! using actual Ollama, PostgreSQL, and other services with NO mocks.
//!
//! Usage:
//!   cargo run --bin e2e_runner                    # Run autonomous workflow test
//!   cargo run --bin e2e_runner -- caws-governance # Run CAWS governance test
//!   cargo run --bin e2e_runner -- api-integration # Run API integration tests
//!   cargo run --bin e2e_runner -- --help          # Show available scenarios

#[cfg(feature = "full")]
use testing_validation::scenarios::autonomous_workflow::run_test as run_autonomous_test;

use testing_validation::{
    E2ETestRunner, Scenario,
    scenarios::{
        caws_governance::run_caws_governance_test,
        human_intervention::run_human_intervention_test,
        performance_scalability::run_performance_test,
        security_privacy::run_security_test,
    },
    harness::TestEnvironment,
    services::{OllamaService, PostgresService},
};
use tracing::{info, error, warn};
use tracing_subscriber;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let scenario_arg = args.get(1).map(|s| s.as_str()).unwrap_or("--autonomous");

    info!("🚀 Starting Agent Agency V3 E2E Test Suite");

    match scenario_arg {
        "--help" | "-h" => {
            print_help();
            return Ok(());
        }
        "--autonomous" | "--auto" => {
            run_legacy_autonomous_test().await
        }
        "--all" => {
            run_all_scenarios().await
        }
        scenario_name => {
            run_specific_scenario(scenario_name).await
        }
    }
}

async fn run_legacy_autonomous_test() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Running legacy autonomous workflow test");

    // Create test environment
    let env = TestEnvironment::new().await?;
    info!("✅ Test environment initialized");

    // Create services with real integrations
    let ollama = OllamaService::with_model("gemma3n:e2b").await?;
    let postgres = PostgresService::new().await?;

    info!("🔧 Services configured for real integrations:");
    info!("   - Ollama: HTTP calls to localhost:11434");
    info!("   - PostgreSQL: Real database connections");

    // Run the autonomous workflow test
    #[cfg(feature = "full")]
    match run_autonomous_test(&env, &ollama, &postgres).await {
        Ok(result) => {
            if result.passed {
                info!("✅ Autonomous workflow test PASSED!");
                info!("   Duration: {}ms", result.duration_ms);
                info!("   Model calls: {}", result.metrics.model_calls);
                info!("   Iterations: {}", result.metrics.iterations);
                info!("   Tokens used: {}", result.metrics.tokens_used);
            } else {
                error!("❌ Autonomous workflow test FAILED: {}",
                       result.error_message.unwrap_or("Unknown error".to_string()));
                std::process::exit(1);
            }
        }
        Err(e) => {
            error!("❌ Test execution failed: {}", e);
            std::process::exit(1);
        }
    }

    #[cfg(not(feature = "full"))]
    {
        error!("❌ Autonomous workflow test requires 'full' feature");
        std::process::exit(1);
    }

    info!("🧹 Cleaning up test environment...");
    env.cleanup().await?;
    info!("✅ Cleanup complete");

    info!("🎉 Legacy autonomous workflow E2E test completed successfully!");
    Ok(())
}

async fn run_specific_scenario(scenario_name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let scenario = parse_scenario_arg(scenario_name)?;

    info!("Running specific scenario: {:?}", scenario);

    // Create E2E test runner
    let runner = E2ETestRunner::setup().await?;
    info!("✅ E2E test runner initialized");

    // Run the specified scenario
    let result = runner.run_scenario(scenario).await;

    if result.passed {
        info!("✅ Scenario {:?} PASSED!", scenario);
        info!("   Duration: {}ms", result.duration_ms);
        print_scenario_metrics(&result);
    } else {
        error!("❌ Scenario {:?} FAILED: {}",
               scenario,
               result.error_message.unwrap_or("Unknown error".to_string()));
        std::process::exit(1);
    }

    // Cleanup
    runner.teardown().await?;
    info!("✅ Test runner cleanup complete");

    info!("🎉 Scenario {:?} completed successfully!", scenario);
    Ok(())
}

async fn run_all_scenarios() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Running ALL test scenarios");

    let mut results = Vec::new();
    let mut passed = 0;
    let mut failed = 0;

    // Create E2E test runner
    let runner = E2ETestRunner::setup().await?;
    info!("✅ E2E test runner initialized");

    // Run all scenarios
    let scenarios = vec![
        Scenario::CawsGovernance,
        Scenario::SelfPromptingLoops,
        Scenario::HumanIntervention,
        Scenario::ReflexiveLearning,
        Scenario::MultiAgentCoordination,
        Scenario::ClaimVerification,
        Scenario::PerformanceScalability,
        Scenario::SecurityPrivacy,
    ];

    for scenario in scenarios {
        info!("Running scenario: {:?}", scenario);
        let result = runner.run_scenario(scenario).await;
        let passed_test = result.passed;
        let error_msg = result.error_message.clone();

        results.push(result);

        if passed_test {
            passed += 1;
            info!("✅ {:?} PASSED", scenario);
        } else {
            failed += 1;
            error!("❌ {:?} FAILED: {}",
                   scenario,
                   error_msg.unwrap_or("Unknown error".to_string()));
        }
    }

    // Cleanup
    runner.teardown().await?;
    info!("✅ Test runner cleanup complete");

    // Summary
    info!("🎯 Test Suite Summary:");
    info!("   Total scenarios: {}", results.len());
    info!("   Passed: {}", passed);
    info!("   Failed: {}", failed);
    info!("   Success rate: {:.1}%", (passed as f64 / results.len() as f64) * 100.0);

    if failed > 0 {
        error!("❌ {} scenario(s) failed", failed);
        std::process::exit(1);
    } else {
        info!("🎉 All scenarios passed!");
    }

    Ok(())
}

fn parse_scenario_arg(arg: &str) -> Result<Scenario, Box<dyn std::error::Error + Send + Sync>> {
    match arg {
        "caws-governance" | "caws" => Ok(Scenario::CawsGovernance),
        "self-prompting" | "loops" => Ok(Scenario::SelfPromptingLoops),
        "human-intervention" | "intervention" => Ok(Scenario::HumanIntervention),
        "reflexive-learning" | "learning" => Ok(Scenario::ReflexiveLearning),
        "multi-agent" | "coordination" => Ok(Scenario::MultiAgentCoordination),
        "claim-verification" | "claims" => Ok(Scenario::ClaimVerification),
        "performance" | "scalability" => Ok(Scenario::PerformanceScalability),
        "security" | "privacy" => Ok(Scenario::SecurityPrivacy),
        "api-integration" | "api" => Ok(Scenario::ApiIntegration),
        _ => {
            error!("Unknown scenario: {}", arg);
            print_help();
            Err(format!("Unknown scenario: {}", arg).into())
        }
    }
}

fn print_scenario_metrics(result: &testing_validation::TestResult) {
    let metrics = &result.metrics;

    // Print core metrics
    if metrics.iterations > 0 {
        info!("   Iterations: {}", metrics.iterations);
    }
    if metrics.model_calls > 0 {
        info!("   Model calls: {}", metrics.model_calls);
    }
    if metrics.tokens_used > 0 {
        info!("   Tokens used: {}", metrics.tokens_used);
    }

    // Print scenario-specific metrics
    match result.scenario {
        Scenario::CawsGovernance => {
            if metrics.waiver_requests > 0 {
                info!("   Waiver requests: {}", metrics.waiver_requests);
            }
            if metrics.budget_violations > 0 {
                info!("   Budget violations: {}", metrics.budget_violations);
            }
        }
        Scenario::SelfPromptingLoops => {
            if metrics.satisficing_stops > 0 {
                info!("   Satisficing stops: {}", metrics.satisficing_stops);
            }
            if metrics.max_iteration_stops > 0 {
                info!("   Max iteration stops: {}", metrics.max_iteration_stops);
            }
        }
        Scenario::HumanIntervention => {
            if metrics.task_pauses > 0 {
                info!("   Task pauses: {}", metrics.task_pauses);
            }
            if metrics.task_cancellations > 0 {
                info!("   Task cancellations: {}", metrics.task_cancellations);
            }
        }
        _ => {} // Other scenarios don't have specific metrics to highlight
    }
}

fn print_help() {
    println!("Agent Agency V3 E2E Test Runner");
    println!();
    println!("USAGE:");
    println!("  cargo run --bin e2e_runner                    # Run legacy autonomous workflow test");
    println!("  cargo run --bin e2e_runner -- <scenario>      # Run specific scenario");
    println!("  cargo run --bin e2e_runner -- --all           # Run all scenarios");
    println!("  cargo run --bin e2e_runner -- --help          # Show this help");
    println!();
    println!("AVAILABLE SCENARIOS:");
    println!("  caws-governance, caws           - CAWS Constitutional Authority tests");
    println!("  self-prompting, loops           - Self-Prompting Loop tests");
    println!("  human-intervention, intervention - Human Intervention tests");
    println!("  reflexive-learning, learning     - Reflexive Learning tests");
    println!("  multi-agent, coordination        - Multi-Agent Coordination tests");
    println!("  claim-verification, claims       - Claim Extraction & Verification tests");
    println!("  performance, scalability        - Performance & Scalability tests");
    println!("  security, privacy               - Security & Privacy tests");
    println!("  api-integration, api            - API Handler Integration tests");
    println!();
    println!("EXAMPLES:");
    println!("  cargo run --bin e2e_runner -- caws");
    println!("  cargo run --bin e2e_runner -- --all");
}




