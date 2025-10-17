//! Integration Tests for V3 Agent Agency System
//!
//! This crate provides comprehensive integration testing for all V3 components,
//! including cross-component communication, end-to-end workflows, and performance benchmarks.

pub mod fixtures;
pub mod helpers;
pub mod mocks;
pub mod test_utils;

pub mod claim_extraction_tests;
pub mod council_tests;
pub mod cross_component_tests;
pub mod end_to_end_tests;
pub mod orchestration_tests;
pub mod performance_tests;
pub mod research_tests;

pub use fixtures::*;
pub use helpers::*;
pub use mocks::*;
pub use test_utils::*;

/// Integration test configuration
#[derive(Debug, Clone)]
pub struct IntegrationTestConfig {
    pub database_url: String,
    pub redis_url: String,
    pub test_timeout: std::time::Duration,
    pub max_concurrent_tests: usize,
    pub enable_performance_tests: bool,
    pub enable_load_tests: bool,
}

impl Default for IntegrationTestConfig {
    fn default() -> Self {
        Self {
            database_url: "postgresql://localhost:5432/agent_agency_test".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            test_timeout: std::time::Duration::from_secs(30),
            max_concurrent_tests: 10,
            enable_performance_tests: false,
            enable_load_tests: false,
        }
    }
}

/// Test result with detailed metrics
#[derive(Debug, Clone)]
pub struct TestResult {
    pub test_name: String,
    pub duration: std::time::Duration,
    pub success: bool,
    pub error_message: Option<String>,
    pub metrics: std::collections::HashMap<String, f64>,
}

/// Integration test suite runner
pub struct IntegrationTestRunner {
    config: IntegrationTestConfig,
    results: Vec<TestResult>,
}

impl IntegrationTestRunner {
    pub fn new(config: IntegrationTestConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
        }
    }

    /// Run all integration tests
    pub async fn run_all_tests(&mut self) -> Result<Vec<TestResult>, anyhow::Error> {
        tracing::info!("Starting integration test suite");

        // Initialize test environment
        self.setup_test_environment().await?;

        // Run test categories
        self.run_council_tests().await?;
        self.run_research_tests().await?;
        self.run_orchestration_tests().await?;
        self.run_claim_extraction_tests().await?;
        self.run_cross_component_tests().await?;
        self.run_end_to_end_tests().await?;

        if self.config.enable_performance_tests {
            self.run_performance_tests().await?;
        }

        if self.config.enable_load_tests {
            self.run_load_tests().await?;
        }

        // Cleanup test environment
        self.cleanup_test_environment().await?;

        tracing::info!("Integration test suite completed");
        Ok(self.results.clone())
    }

    async fn setup_test_environment(&self) -> Result<(), anyhow::Error> {
        tracing::info!("Setting up test environment");
        // TODO: Initialize test database, Redis, etc.
        Ok(())
    }

    async fn cleanup_test_environment(&self) -> Result<(), anyhow::Error> {
        tracing::info!("Cleaning up test environment");
        // TODO: Clean up test resources
        Ok(())
    }

    async fn run_council_tests(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Running council integration tests");
        // TODO: Implement council tests
        Ok(())
    }

    async fn run_research_tests(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Running research integration tests");
        // TODO: Implement research tests
        Ok(())
    }

    async fn run_orchestration_tests(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Running orchestration integration tests");
        // TODO: Implement orchestration tests
        Ok(())
    }

    async fn run_claim_extraction_tests(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Running claim extraction integration tests");
        // TODO: Implement claim extraction tests
        Ok(())
    }

    async fn run_cross_component_tests(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Running cross-component integration tests");
        // TODO: Implement cross-component tests
        Ok(())
    }

    async fn run_end_to_end_tests(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Running end-to-end integration tests");
        // TODO: Implement end-to-end tests
        Ok(())
    }

    async fn run_performance_tests(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Running performance tests");
        // TODO: Implement performance tests
        Ok(())
    }

    async fn run_load_tests(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Running load tests");
        // TODO: Implement load tests
        Ok(())
    }
}

/// Initialize tracing for integration tests
pub fn init_test_logging() {
    use std::sync::Once;
    use tracing_subscriber::filter::EnvFilter;
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    static INIT: Once = Once::new();

    INIT.call_once(|| {
        tracing_subscriber::registry()
            .with(
                EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "integration_tests=debug,agent_agency=debug".into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claim_extraction_tests::ClaimExtractionIntegrationTests;
    use crate::council_tests::CouncilIntegrationTests;
    use crate::cross_component_tests::CrossComponentIntegrationTests;
    use crate::end_to_end_tests::EndToEndIntegrationTests;
    use crate::orchestration_tests::OrchestrationIntegrationTests;
    use crate::performance_tests::PerformanceTests;
    use crate::research_tests::ResearchIntegrationTests;

    #[tokio::test]
    async fn test_integration_runner_creation() {
        let config = IntegrationTestConfig::default();
        let runner = IntegrationTestRunner::new(config);
        assert_eq!(runner.results.len(), 0);
    }

    #[tokio::test]
    async fn test_test_config_default() {
        let config = IntegrationTestConfig::default();
        assert_eq!(config.max_concurrent_tests, 10);
        assert!(!config.enable_performance_tests);
        assert!(!config.enable_load_tests);
    }

    #[tokio::test]
    async fn test_council_integration() {
        init_test_logging();
        let tests = CouncilIntegrationTests::new();
        let results = tests.run_all_tests().await.unwrap();
        assert!(
            !results.is_empty(),
            "Council integration tests should produce results"
        );
        for result in results {
            assert!(
                result.success,
                "Council test '{}' failed: {:?}",
                result.test_name, result.error_message
            );
        }
    }

    #[tokio::test]
    async fn test_claim_extraction_integration() {
        init_test_logging();
        let tests = ClaimExtractionIntegrationTests::new();
        let results = tests.run_all_tests().await.unwrap();
        assert!(
            !results.is_empty(),
            "Claim extraction integration tests should produce results"
        );
        for result in results {
            assert!(
                result.success,
                "Claim extraction test '{}' failed: {:?}",
                result.test_name, result.error_message
            );
        }
    }

    #[tokio::test]
    async fn test_research_integration() {
        init_test_logging();
        let tests = ResearchIntegrationTests::new();
        let results = tests.run_all_tests().await.unwrap();
        assert!(
            !results.is_empty(),
            "Research integration tests should produce results"
        );
        for result in results {
            assert!(
                result.success,
                "Research test '{}' failed: {:?}",
                result.test_name, result.error_message
            );
        }
    }

    #[tokio::test]
    async fn test_orchestration_integration() {
        init_test_logging();
        let tests = OrchestrationIntegrationTests::new();
        let results = tests.run_all_tests().await.unwrap();
        assert!(
            !results.is_empty(),
            "Orchestration integration tests should produce results"
        );
        for result in results {
            assert!(
                result.success,
                "Orchestration test '{}' failed: {:?}",
                result.test_name, result.error_message
            );
        }
    }

    #[tokio::test]
    async fn test_cross_component_integration() {
        init_test_logging();
        let tests = CrossComponentIntegrationTests::new();
        let results = tests.run_all_tests().await.unwrap();
        assert!(
            !results.is_empty(),
            "Cross-component integration tests should produce results"
        );
        for result in results {
            assert!(
                result.success,
                "Cross-component test '{}' failed: {:?}",
                result.test_name, result.error_message
            );
        }
    }

    #[tokio::test]
    async fn test_end_to_end_integration() {
        init_test_logging();
        let tests = EndToEndIntegrationTests::new();
        let results = tests.run_all_tests().await.unwrap();
        assert!(
            !results.is_empty(),
            "End-to-end integration tests should produce results"
        );
        for result in results {
            assert!(
                result.success,
                "End-to-end test '{}' failed: {:?}",
                result.test_name, result.error_message
            );
        }
    }

    #[tokio::test]
    async fn test_performance_integration() {
        init_test_logging();
        let tests = PerformanceTests::new();
        let results = tests.run_all_tests().await.unwrap();
        assert!(
            !results.is_empty(),
            "Performance integration tests should produce results"
        );
        for result in results {
            assert!(
                result.success,
                "Performance test '{}' failed: {:?}",
                result.test_name, result.error_message
            );
        }
    }

    #[tokio::test]
    async fn test_all_integration_suites() {
        init_test_logging();

        // Run all integration test suites
        let suites = vec![
            (
                "Council",
                CouncilIntegrationTests::new().run_all_tests().await,
            ),
            (
                "Claim Extraction",
                ClaimExtractionIntegrationTests::new().run_all_tests().await,
            ),
            (
                "Research",
                ResearchIntegrationTests::new().run_all_tests().await,
            ),
            (
                "Orchestration",
                OrchestrationIntegrationTests::new().run_all_tests().await,
            ),
            (
                "Cross Component",
                CrossComponentIntegrationTests::new().run_all_tests().await,
            ),
            (
                "End to End",
                EndToEndIntegrationTests::new().run_all_tests().await,
            ),
            ("Performance", PerformanceTests::new().run_all_tests().await),
        ];

        let mut total_tests = 0;
        let mut passed_tests = 0;

        for (suite_name, result) in suites {
            match result {
                Ok(results) => {
                    total_tests += results.len();
                    let passed = results.iter().filter(|r| r.success).count();
                    passed_tests += passed;
                    println!(
                        "✅ {}: {}/{} tests passed",
                        suite_name,
                        passed,
                        results.len()
                    );

                    // Report failures
                    for result in results.iter().filter(|r| !r.success) {
                        println!(
                            "❌ {} - {}: {:?}",
                            suite_name, result.test_name, result.error_message
                        );
                    }
                }
                Err(e) => {
                    println!("❌ {}: Failed to run suite - {:?}", suite_name, e);
                }
            }
        }

        println!(
            "📊 Integration Test Summary: {}/{} tests passed across all suites",
            passed_tests, total_tests
        );
        assert!(
            passed_tests > 0,
            "At least some integration tests should pass"
        );
        assert!(
            passed_tests >= total_tests / 2,
            "At least half of integration tests should pass (got {}/{})",
            passed_tests,
            total_tests
        );
    }
}
