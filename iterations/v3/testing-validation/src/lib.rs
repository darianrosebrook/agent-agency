//! End-to-End Autonomous Flow Tests for Agent Agency V3
//!
//! This crate implements comprehensive E2E tests that validate autonomous workflows
//! using REAL integrations with local services (Ollama, PostgreSQL, Git) and NO mocks.
//!
//! ## Test Scenarios
//!
//! 1. **Autonomous Code Refactor**: Real LLM-driven code refactoring with file system operations
//! 2. **Research & Synthesis**: Real research with Ollama models and data persistence
//! 3. **Code Generation + Testing**: Real code generation with compilation validation
//!
//! ## Real Integrations (No Mocks)
//!
//! - **Ollama Service**: Direct HTTP calls to local Ollama instance
//! - **PostgreSQL**: Real database connections and queries
//! - **File System**: Actual file operations and Git version control
//! - **Process Execution**: Real command execution (cargo, git, etc.)
//!
//! @author @darianrosebrook

pub mod fixtures;
pub mod harness;
pub mod services;
pub mod scenarios;

use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, error};

use harness::{TestEnvironment, LocalServiceManager};
use services::{OrchestratorService, OllamaService, PostgresService};

/// Main E2E test runner
pub struct E2ETestRunner {
    environment: TestEnvironment,
    services: LocalServiceManager,
}

impl E2ETestRunner {
    /// Create and setup a new E2E test runner
    pub async fn setup() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        info!("Setting up E2E test environment");

        let environment = TestEnvironment::new().await?;
        let services = LocalServiceManager::new().await?;

        // Start all required services
        services.start_all().await?;

        // Wait for services to be healthy
        services.wait_for_healthy().await?;

        info!("E2E test environment ready");

        Ok(Self {
            environment,
            services,
        })
    }

    /// Run a specific test scenario
    pub async fn run_scenario(&self, scenario: Scenario) -> TestResult {
        info!("Running scenario: {:?}", scenario);

        match scenario {
            Scenario::Scenario1Refactor => {
                scenarios::scenario_1_refactor::run_test(&self.environment, &self.services).await
            }
            Scenario::Scenario2Research => {
                scenarios::scenario_2_research::run_test(&self.environment, &self.services).await
            }
            Scenario::Scenario3Mutation => {
                scenarios::scenario_3_mutation::run_test(&self.environment, &self.services).await
            }
        }
    }

    /// Tear down the test environment
    pub async fn teardown(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Tearing down E2E test environment");

        // Stop all services
        self.services.stop_all().await?;

        // Clean up environment
        self.environment.cleanup().await?;

        info!("E2E test environment cleaned up");

        Ok(())
    }
}

/// Test scenarios available for execution
#[derive(Debug, Clone, Copy)]
pub enum Scenario {
    Scenario1Refactor,
    Scenario2Research,
    Scenario3Mutation,
}

/// Result of a test scenario execution
#[derive(Debug)]
pub struct TestResult {
    pub scenario: Scenario,
    pub passed: bool,
    pub duration_ms: u64,
    pub error_message: Option<String>,
    pub metrics: TestMetrics,
}

/// Performance and validation metrics from test execution
#[derive(Debug, Clone, Default)]
pub struct TestMetrics {
    pub iterations: usize,
    pub model_calls: usize,
    pub tokens_used: usize,
    pub council_evaluations: usize,
    pub caws_compliance_checks: usize,
    pub provenance_entries: usize,
}

/// Error types for E2E testing
#[derive(Debug, thiserror::Error)]
pub enum E2ETestError {
    #[error("Service setup failed: {0}")]
    ServiceSetup(String),

    #[error("Test execution failed: {0}")]
    TestExecution(String),

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Environment setup failed: {0}")]
    Environment(String),
}
