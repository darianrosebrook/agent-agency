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
pub mod test_helpers;

use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, error};

use harness::{TestEnvironment, LocalServiceManager};
#[cfg(feature = "full")]
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
            #[cfg(feature = "full")]
            Scenario::Scenario1Refactor => {
                scenarios::scenario_1_refactor::run_test(&self.environment, &self.services).await
            }
            #[cfg(feature = "full")]
            Scenario::Scenario2Research => {
                scenarios::scenario_2_research::run_test(&self.environment, &self.services).await
            }
            #[cfg(feature = "full")]
            Scenario::Scenario3Mutation => {
                scenarios::scenario_3_mutation::run_test(&self.environment, &self.services).await
            }
            Scenario::Scenario4FileEditing => {
                scenarios::scenario_4_file_editing::run_file_editing_e2e_test().await
            }
            // CAWS Constitutional Authority tests
            Scenario::CawsGovernance => {
                scenarios::caws_governance::run_caws_governance_test(&self.environment, &self.services).await
            }
            // Self-Prompting Loop tests
            #[cfg(feature = "full")]
            Scenario::SelfPromptingLoops => {
                scenarios::self_prompting_loops::run_self_prompting_test(&self.environment, &self.services).await
            }
            #[cfg(not(feature = "full"))]
            Scenario::SelfPromptingLoops => {
                error!("Self-Prompting Loop test requires 'full' feature");
                TestResult {
                    scenario: Scenario::SelfPromptingLoops,
                    passed: false,
                    duration_ms: 0,
                    error_message: Some("Self-Prompting Loop test requires 'full' feature".to_string()),
                    metrics: TestMetrics::default(),
                }
            }
            // Human Intervention tests
            Scenario::HumanIntervention => {
                scenarios::human_intervention::run_human_intervention_test(&self.environment, &self.services).await
            }
            // Reflexive Learning tests
            #[cfg(feature = "full")]
            Scenario::ReflexiveLearning => {
                scenarios::reflexive_learning::run_reflexive_learning_test(&self.environment, &self.services).await
            }
            #[cfg(not(feature = "full"))]
            Scenario::ReflexiveLearning => {
                error!("Reflexive Learning test requires 'full' feature");
                TestResult {
                    scenario: Scenario::ReflexiveLearning,
                    passed: false,
                    duration_ms: 0,
                    error_message: Some("Reflexive Learning test requires 'full' feature".to_string()),
                    metrics: TestMetrics::default(),
                }
            }
            // Multi-Agent Coordination tests
            #[cfg(feature = "full")]
            Scenario::MultiAgentCoordination => {
                scenarios::multi_agent_coordination::run_multi_agent_test(&self.environment, &self.services).await
            }
            #[cfg(not(feature = "full"))]
            Scenario::MultiAgentCoordination => {
                error!("Multi-Agent Coordination test requires 'full' feature");
                TestResult {
                    scenario: Scenario::MultiAgentCoordination,
                    passed: false,
                    duration_ms: 0,
                    error_message: Some("Multi-Agent Coordination test requires 'full' feature".to_string()),
                    metrics: TestMetrics::default(),
                }
            }
            // Claim Extraction & Verification tests
            #[cfg(feature = "full")]
            Scenario::ClaimVerification => {
                scenarios::claim_verification::run_claim_verification_test(&self.environment, &self.services).await
            }
            #[cfg(not(feature = "full"))]
            Scenario::ClaimVerification => {
                error!("Claim Verification test requires 'full' feature");
                TestResult {
                    scenario: Scenario::ClaimVerification,
                    passed: false,
                    duration_ms: 0,
                    error_message: Some("Claim Verification test requires 'full' feature".to_string()),
                    metrics: TestMetrics::default(),
                }
            }
            // Performance & Scalability tests
            Scenario::PerformanceScalability => {
                scenarios::performance_scalability::run_performance_test(&self.environment, &self.services).await
            }
            // Security & Privacy tests
            Scenario::SecurityPrivacy => {
                scenarios::security_privacy::run_security_test(&self.environment, &self.services).await
            }
            #[cfg(not(feature = "full"))]
            _ => {
                error!("Scenario requires 'full' feature: {:?}", scenario);
                TestResult {
                    scenario,
                    passed: false,
                    duration_ms: 0,
                    error_message: Some("Scenario requires 'full' feature".to_string()),
                    metrics: TestMetrics::default(),
                }
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
    Scenario4FileEditing,
    // CAWS Constitutional Authority tests
    CawsGovernance,
    // Self-Prompting Loop tests
    SelfPromptingLoops,
    // Human Intervention tests
    HumanIntervention,
    // Reflexive Learning tests
    ReflexiveLearning,
    // Multi-Agent Coordination tests
    MultiAgentCoordination,
    // Claim Extraction & Verification tests
    ClaimVerification,
    // Performance & Scalability tests
    PerformanceScalability,
    // Security & Privacy tests
    SecurityPrivacy,
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
    // CAWS Governance metrics
    pub waiver_requests: usize,
    pub waiver_approvals: usize,
    pub budget_violations: usize,
    pub scope_violations: usize,
    // Self-Prompting Loop metrics
    pub satisficing_stops: usize,
    pub max_iteration_stops: usize,
    pub quality_ceiling_stops: usize,
    pub model_swaps: usize,
    pub evaluation_scores: Vec<f64>,
    // Human Intervention metrics
    pub task_pauses: usize,
    pub task_resumes: usize,
    pub task_cancellations: usize,
    pub human_overrides: usize,
    pub intervention_api_calls: usize,
    // Reflexive Learning metrics
    pub performance_data_points: usize,
    pub learning_iterations: usize,
    pub model_improvements: usize,
    pub curriculum_advancements: usize,
    // Multi-Agent Coordination metrics
    pub agent_communications: usize,
    pub arbitration_events: usize,
    pub conflict_resolutions: usize,
    pub task_decompositions: usize,
    pub consensus_achieved: usize,
    // Claim Verification metrics
    pub claims_extracted: usize,
    pub claims_verified: usize,
    pub hallucinations_detected: usize,
    pub evidence_checks: usize,
    pub disambiguations_resolved: usize,
    // Performance & Scalability metrics
    pub concurrent_operations: usize,
    pub response_times_ms: Vec<u64>,
    pub resource_utilization: Vec<f64>,
    pub memory_usage_mb: Vec<f64>,
    pub throughput_operations_per_sec: Vec<f64>,
    // Security & Privacy metrics
    pub security_violations: usize,
    pub privacy_breaches: usize,
    pub encryption_operations: usize,
    pub audit_log_entries: usize,
    pub access_control_checks: usize,
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
