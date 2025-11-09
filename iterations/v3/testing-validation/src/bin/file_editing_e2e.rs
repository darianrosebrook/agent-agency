//! E2E File Editing Test Runner
//!
//! Standalone binary to run the autonomous file editing E2E test
//! with real Git worktrees and file operations.

use testing_validation::scenarios::scenario_4_file_editing::run_file_editing_e2e_test;
use tracing::{info, error};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 Starting Autonomous File Editing E2E Test");
    info!("This test uses REAL Git worktrees and file operations (NO MOCKS)");

    // Run the E2E test
    let result = run_file_editing_e2e_test().await;

    // Report results
    if result.passed {
        info!("✅ E2E File Editing Test PASSED");
        info!("   Duration: {}ms", result.duration_ms);
        info!("   CAWS Compliance Checks: {}", result.metrics.caws_compliance_checks);
        info!("   Provenance Entries: {}", result.metrics.provenance_entries);
    } else {
        error!("❌ E2E File Editing Test FAILED");
        error!("   Duration: {}ms", result.duration_ms);
        if let Some(err_msg) = &result.error_message {
            error!("   Error: {}", err_msg);
        }
        std::process::exit(1);
    }

    Ok(())
}

// Simple smoke test to verify our implementations compile and can be instantiated
#[cfg(test)]
mod smoke_tests {
    use super::*;

    #[tokio::test]
    async fn test_imports_compile() {
        // This test just verifies that our imports work and types can be created
        // We don't actually run the full E2E test here since it requires external dependencies

        // Verify we can import our test scenarios (only those available without full feature)
        use testing_validation::scenarios::scenario_4_file_editing::run_file_editing_e2e_test;
        use testing_validation::scenarios::security_privacy::run_security_test;
        
        #[cfg(feature = "full")]
        {
            use testing_validation::scenarios::claim_verification::run_claim_verification_test;
            use testing_validation::scenarios::multi_agent_coordination::run_multi_agent_test;
            use testing_validation::scenarios::reflexive_learning::run_reflexive_learning_test;
            use testing_validation::scenarios::self_prompting_loops::run_self_prompting_test;
        }

        println!("✅ All E2E test scenario imports successful");

        // Verify we can import our services
        use testing_validation::services::postgres::PostgresService;
        
        #[cfg(feature = "full")]
        {
            use data_infrastructure::file_operations_service::create_file_operations_service;
        }

        println!("✅ All service imports successful");

        // Verify we can create basic types
        let task_id = uuid::Uuid::new_v4();
        assert!(!task_id.to_string().is_empty());

        println!("✅ Basic type creation successful");
    }
}




