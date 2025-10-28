//! Autonomous Workflow E2E Test Runner
//!
//! Runs the real E2E test that validates autonomous agent execution
//! using actual Ollama and PostgreSQL services with NO mocks.

use testing_validation::scenarios::autonomous_workflow::run_test;
use testing_validation::harness::TestEnvironment;
use testing_validation::services::{OllamaService, PostgresService};
use tracing::{info, error};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 Starting Agent Agency V3 Autonomous Workflow E2E Test");

    // Create test environment
    let env = TestEnvironment::new().await?;
    info!("✅ Test environment initialized");

    // Create services with real integrations
    let ollama = OllamaService::with_model("gemma3n:e2b").await?;

    let postgres = PostgresService::new(
        "localhost".to_string(),
        5432,
        "test_db".to_string(),
        "test_user".to_string(),
        "test_password".to_string(),
    );

    info!("🔧 Services configured for real integrations:");
    info!("   - Ollama: HTTP calls to localhost:11434");
    info!("   - PostgreSQL: Real database connections");

    // Note: In a full implementation, we'd have a LocalServiceManager
    // For now, we pass services directly to the test

    // Run the autonomous workflow test
    match run_test(&env, &ollama, &postgres).await {
        Ok(result) => {
            if result.passed {
                info!("✅ Test PASSED!");
                info!("   Duration: {}ms", result.duration_ms);
                info!("   Model calls: {}", result.metrics.model_calls);
                info!("   Iterations: {}", result.metrics.iterations);
                info!("   Tokens used: {}", result.metrics.tokens_used);
            } else {
                error!("❌ Test FAILED: {}", result.error_message.unwrap_or("Unknown error".to_string()));
                std::process::exit(1);
            }
        }
        Err(e) => {
            error!("❌ Test execution failed: {}", e);
            std::process::exit(1);
        }
    }

    info!("🧹 Cleaning up test environment...");
    env.cleanup().await?;
    info!("✅ Cleanup complete");

    info!("🎉 Autonomous workflow E2E test completed successfully!");
    info!("   This proves Agent Agency V3 can execute real autonomous workflows");
    info!("   with actual LLM inference and database persistence - NO MOCKS!");

    Ok(())
}




