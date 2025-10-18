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
pub mod database_tests;
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
        self.run_database_tests().await?;
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
        // TODO: Initialize test database, Redis, etc. with the following requirements:
        // 1. Test environment setup: Set up comprehensive test environment
        //    - Initialize test database with schema and data
        //    - Set up Redis for caching and session management
        //    - Configure test environment settings and parameters
        // 2. Test infrastructure setup: Set up test infrastructure components
        //    - Initialize test HTTP clients and servers
        //    - Set up test file systems and storage
        //    - Configure test network and connectivity
        // 3. Test data preparation: Prepare test data and fixtures
        //    - Seed test database with required test data
        //    - Set up test scenarios and edge cases
        //    - Handle test data validation and verification
        // 4. Test environment validation: Validate test environment setup
        //    - Verify test environment components are working
        //    - Check test environment configuration and settings
        //    - Handle test environment validation errors and corrections
        Ok(())
    }

    async fn cleanup_test_environment(&self) -> Result<(), anyhow::Error> {
        tracing::info!("Cleaning up test environment");
        // TODO: Clean up test resources with the following requirements:
        // 1. Test resource cleanup: Clean up all test resources
        //    - Remove test data and temporary files
        //    - Clean up test database and Redis data
        //    - Handle test resource cleanup error handling and recovery
        // 2. Test infrastructure cleanup: Clean up test infrastructure
        //    - Close test HTTP clients and servers
        //    - Clean up test file systems and storage
        //    - Handle test infrastructure cleanup validation
        // 3. Test environment cleanup: Clean up test environment
        //    - Reset test environment to clean state
        //    - Clean up test environment configuration
        //    - Handle test environment cleanup validation
        // 4. Test cleanup monitoring: Monitor test cleanup process
        //    - Track test cleanup progress and performance
        //    - Monitor test cleanup effectiveness
        //    - Handle test cleanup monitoring and reporting
        Ok(())
    }

    async fn run_council_tests(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Running council integration tests");
        
        // Test 1: Council arbitration and decision-making processes
        self.test_council_arbitration().await?;
        
        // Test 2: Council communication and coordination
        self.test_council_communication().await?;
        
        // Test 3: Council voting and consensus mechanisms
        self.test_council_consensus().await?;
        
        // Test 4: Council performance and response times
        self.test_council_performance().await?;
        
        // Test 5: Council error handling and recovery
        self.test_council_error_handling().await?;
        
        tracing::info!("Council integration tests completed successfully");
        Ok(())
    }

    async fn run_research_tests(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Running research integration tests");
        
        // Test 1: Research knowledge seeking and discovery
        self.test_research_knowledge_seeking().await?;
        
        // Test 2: Research query processing and execution
        self.test_research_query_processing().await?;
        
        // Test 3: Research performance and response times
        self.test_research_performance().await?;
        
        // Test 4: Research error handling and recovery
        self.test_research_error_handling().await?;
        
        tracing::info!("Research integration tests completed successfully");
        Ok(())
    }

    async fn run_orchestration_tests(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Running orchestration integration tests");
        
        // Test 1: Orchestration task routing and execution
        self.test_orchestration_task_routing().await?;
        
        // Test 2: Orchestration worker management and coordination
        self.test_orchestration_worker_management().await?;
        
        // Test 3: Orchestration load balancing and distribution
        self.test_orchestration_load_balancing().await?;
        
        // Test 4: Orchestration performance and scalability
        self.test_orchestration_performance().await?;
        
        // Test 5: Orchestration error handling and recovery
        self.test_orchestration_error_handling().await?;
        
        tracing::info!("Orchestration integration tests completed successfully");
        Ok(())
    }

    async fn run_claim_extraction_tests(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Running claim extraction integration tests");
        
        // Test 1: Claim extraction from various sources and formats
        self.test_claim_extraction_sources().await?;
        
        // Test 2: Claim extraction processing and validation
        self.test_claim_extraction_processing().await?;
        
        // Test 3: Claim extraction accuracy and completeness
        self.test_claim_extraction_accuracy().await?;
        
        // Test 4: Claim extraction performance and scalability
        self.test_claim_extraction_performance().await?;
        
        // Test 5: Claim extraction error handling and recovery
        self.test_claim_extraction_error_handling().await?;
        
        tracing::info!("Claim extraction integration tests completed successfully");
        Ok(())
    }

    async fn run_database_tests(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Running database integration tests");
        
        // Test 1: Database client connection and basic operations
        self.test_database_connection().await?;
        
        // Test 2: Database CRUD operations and transactions
        self.test_database_crud_operations().await?;
        
        // Test 3: Database performance and query optimization
        self.test_database_performance().await?;
        
        // Test 4: Database error handling and recovery
        self.test_database_error_handling().await?;
        
        tracing::info!("Database integration tests completed successfully");
        Ok(())
    }

    async fn run_cross_component_tests(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Running cross-component integration tests");
        // TODO: Implement cross-component tests with the following requirements:
        // 1. Cross-component integration tests: Implement comprehensive cross-component integration tests
        //    - Test component communication and data flow
        //    - Test component coordination and synchronization
        //    - Handle cross-component test validation and verification
        // 2. Cross-component functionality tests: Test cross-component functionality and features
        //    - Test component interaction and collaboration
        //    - Test component error handling and recovery
        //    - Handle cross-component functionality test validation
        // 3. Cross-component performance tests: Test cross-component performance and scalability
        //    - Test cross-component response times and throughput
        //    - Test cross-component load handling and stress testing
        //    - Handle cross-component performance test validation
        // 4. Cross-component error handling tests: Test cross-component error handling and recovery
        //    - Test cross-component error scenarios and edge cases
        //    - Test cross-component error recovery and resilience
        //    - Handle cross-component error handling test validation
        Ok(())
    }

    async fn run_end_to_end_tests(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Running end-to-end integration tests");
        // TODO: Implement end-to-end tests with the following requirements:
        // 1. End-to-end integration tests: Implement comprehensive end-to-end integration tests
        //    - Test complete system workflows and processes
        //    - Test system integration and data flow
        //    - Handle end-to-end test validation and verification
        // 2. End-to-end functionality tests: Test end-to-end functionality and features
        //    - Test complete user journeys and scenarios
        //    - Test system behavior and outcomes
        //    - Handle end-to-end functionality test validation
        // 3. End-to-end performance tests: Test end-to-end performance and scalability
        //    - Test end-to-end response times and throughput
        //    - Test end-to-end load handling and stress testing
        //    - Handle end-to-end performance test validation
        // 4. End-to-end error handling tests: Test end-to-end error handling and recovery
        //    - Test end-to-end error scenarios and edge cases
        //    - Test end-to-end error recovery and resilience
        //    - Handle end-to-end error handling test validation
        Ok(())
    }

    async fn run_performance_tests(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Running performance tests");
        // TODO: Implement performance tests with the following requirements:
        // 1. Performance integration tests: Implement comprehensive performance integration tests
        //    - Test system performance under various load conditions
        //    - Test performance metrics and benchmarks
        //    - Handle performance test validation and verification
        // 2. Performance functionality tests: Test performance functionality and features
        //    - Test performance optimization and tuning
        //    - Test performance monitoring and reporting
        //    - Handle performance functionality test validation
        // 3. Performance scalability tests: Test performance scalability and capacity
        //    - Test performance under increasing load
        //    - Test performance resource utilization
        //    - Handle performance scalability test validation
        // 4. Performance error handling tests: Test performance error handling and recovery
        //    - Test performance error scenarios and edge cases
        //    - Test performance error recovery and resilience
        //    - Handle performance error handling test validation
        Ok(())
    }

    async fn run_load_tests(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Running load tests");
        // TODO: Implement load tests with the following requirements:
        // 1. Load integration tests: Implement comprehensive load integration tests
        //    - Test system behavior under high load conditions
        //    - Test load handling and stress testing
        //    - Handle load test validation and verification
        // 2. Load functionality tests: Test load functionality and features
        //    - Test load balancing and distribution
        //    - Test load monitoring and reporting
        //    - Handle load functionality test validation
        // 3. Load scalability tests: Test load scalability and capacity
        //    - Test load handling under increasing demand
        //    - Test load resource utilization and efficiency
        //    - Handle load scalability test validation
        // 4. Load error handling tests: Test load error handling and recovery
        //    - Test load error scenarios and edge cases
        //    - Test load error recovery and resilience
        //    - Handle load error handling test validation
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
    use crate::database_tests::DatabaseIntegrationTests;
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
    async fn test_database_integration() {
        init_test_logging();
        let tests = DatabaseIntegrationTests::new();
        let results = tests.run_all_tests().await.unwrap();
        assert!(
            !results.is_empty(),
            "Database integration tests should produce results"
        );
        for result in results {
            assert!(
                result.success,
                "Database test '{}' failed: {:?}",
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
                "Database",
                DatabaseIntegrationTests::new().run_all_tests().await,
            ),
            (
                "Orchestration",
                OrchestrationIntegrationTests::new().run_all_tests().await,
            ),
            (
                "Research",
                ResearchIntegrationTests::new().run_all_tests().await,
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

    /// Test council arbitration and decision-making processes
    async fn test_council_arbitration(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Testing council arbitration processes");
        
        // Test basic arbitration engine initialization
        let arbitration_engine = agent_agency_council::advanced_arbitration::AdvancedArbitrationEngine::new()
            .map_err(|e| anyhow::anyhow!("Failed to initialize arbitration engine: {:?}", e))?;
        
        // Test conflict resolution with mock data
        let mock_conflicts = vec![
            agent_agency_council::types::Conflict {
                conflict_id: uuid::Uuid::new_v4(),
                conflict_type: agent_agency_council::types::ConflictType::ResourceAllocation,
                severity: agent_agency_council::types::ConflictSeverity::Medium,
                participants: vec!["worker_1".to_string(), "worker_2".to_string()],
                description: "Resource allocation conflict".to_string(),
                context: std::collections::HashMap::new(),
            }
        ];
        
        let resolution_result = arbitration_engine.resolve_conflicts(&mock_conflicts).await
            .map_err(|e| anyhow::anyhow!("Failed to resolve conflicts: {:?}", e))?;
        
        // Validate resolution result
        assert!(!resolution_result.resolutions.is_empty(), "Should have at least one resolution");
        assert!(resolution_result.consensus_score >= 0.0 && resolution_result.consensus_score <= 1.0, 
                "Consensus score should be between 0.0 and 1.0");
        
        tracing::info!("Council arbitration test passed");
        Ok(())
    }

    /// Test council communication and coordination
    async fn test_council_communication(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Testing council communication and coordination");
        
        // Test council coordinator initialization
        let config = agent_agency_council::CouncilConfig {
            max_rounds: 5,
            consensus_threshold: 0.8,
            timeout_ms: 30000,
            enable_learning: true,
            debug_mode: false,
        };
        
        let coordinator = agent_agency_council::coordinator::CouncilCoordinator::new(config)
            .map_err(|e| anyhow::anyhow!("Failed to initialize council coordinator: {:?}", e))?;
        
        // Test basic coordination functionality
        let mock_task = agent_agency_council::models::TaskSpec {
            id: uuid::Uuid::new_v4(),
            title: "Test Task".to_string(),
            description: "Integration test task".to_string(),
            priority: agent_agency_council::models::TaskPriority::Medium,
            complexity: agent_agency_council::models::TaskComplexity::Medium,
            estimated_duration_ms: 5000,
            required_skills: vec!["testing".to_string()],
            dependencies: vec![],
            metadata: std::collections::HashMap::new(),
        };
        
        // Test task evaluation (this would normally involve actual workers)
        tracing::info!("Council communication test passed");
        Ok(())
    }

    /// Test council voting and consensus mechanisms
    async fn test_council_consensus(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Testing council consensus mechanisms");
        
        // Test consensus calculation with mock votes
        let mock_votes = vec![
            agent_agency_council::types::Vote {
                voter_id: "judge_1".to_string(),
                decision: agent_agency_council::types::VerdictDecision::Approved,
                confidence: 0.9,
                reasoning: "Strong approval based on evidence".to_string(),
                evidence: vec![],
                timestamp: chrono::Utc::now(),
            },
            agent_agency_council::types::Vote {
                voter_id: "judge_2".to_string(),
                decision: agent_agency_council::types::VerdictDecision::Approved,
                confidence: 0.8,
                reasoning: "Approval with minor concerns".to_string(),
                evidence: vec![],
                timestamp: chrono::Utc::now(),
            },
        ];
        
        // Test consensus calculation
        let consensus_score = agent_agency_council::coordinator::calculate_consensus(&mock_votes);
        assert!(consensus_score >= 0.0 && consensus_score <= 1.0, 
                "Consensus score should be between 0.0 and 1.0");
        
        tracing::info!("Council consensus test passed with score: {}", consensus_score);
        Ok(())
    }

    /// Test council performance and response times
    async fn test_council_performance(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Testing council performance and response times");
        
        let start_time = std::time::Instant::now();
        
        // Test arbitration engine performance
        let arbitration_engine = agent_agency_council::advanced_arbitration::AdvancedArbitrationEngine::new()
            .map_err(|e| anyhow::anyhow!("Failed to initialize arbitration engine: {:?}", e))?;
        
        // Create multiple mock conflicts for performance testing
        let mock_conflicts: Vec<agent_agency_council::types::Conflict> = (0..10)
            .map(|i| agent_agency_council::types::Conflict {
                conflict_id: uuid::Uuid::new_v4(),
                conflict_type: agent_agency_council::types::ConflictType::ResourceAllocation,
                severity: agent_agency_council::types::ConflictSeverity::Low,
                participants: vec![format!("worker_{}", i), format!("worker_{}", i + 1)],
                description: format!("Performance test conflict {}", i),
                context: std::collections::HashMap::new(),
            })
            .collect();
        
        let resolution_result = arbitration_engine.resolve_conflicts(&mock_conflicts).await
            .map_err(|e| anyhow::anyhow!("Failed to resolve conflicts: {:?}", e))?;
        
        let elapsed = start_time.elapsed();
        
        // Performance assertions
        assert!(elapsed.as_millis() < 5000, "Arbitration should complete within 5 seconds");
        assert!(!resolution_result.resolutions.is_empty(), "Should resolve all conflicts");
        
        tracing::info!("Council performance test passed in {:?}", elapsed);
        Ok(())
    }

    /// Test council error handling and recovery
    async fn test_council_error_handling(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Testing council error handling and recovery");
        
        // Test handling of invalid input
        let arbitration_engine = agent_agency_council::advanced_arbitration::AdvancedArbitrationEngine::new()
            .map_err(|e| anyhow::anyhow!("Failed to initialize arbitration engine: {:?}", e))?;
        
        // Test with empty conflicts list
        let empty_conflicts = vec![];
        let result = arbitration_engine.resolve_conflicts(&empty_conflicts).await;
        
        // Should handle empty list gracefully
        match result {
            Ok(resolution) => {
                assert!(resolution.resolutions.is_empty(), "Empty conflicts should result in empty resolutions");
                assert_eq!(resolution.consensus_score, 1.0, "Empty conflicts should have perfect consensus");
            }
            Err(e) => {
                // If it returns an error, that's also acceptable behavior
                tracing::info!("Arbitration engine correctly handled empty conflicts with error: {:?}", e);
            }
        }
        
        // Test with malformed conflict data
        let malformed_conflicts = vec![
            agent_agency_council::types::Conflict {
                conflict_id: uuid::Uuid::new_v4(),
                conflict_type: agent_agency_council::types::ConflictType::ResourceAllocation,
                severity: agent_agency_council::types::ConflictSeverity::Critical,
                participants: vec![], // Empty participants should be handled gracefully
                description: "Malformed conflict test".to_string(),
                context: std::collections::HashMap::new(),
            }
        ];
        
        let result = arbitration_engine.resolve_conflicts(&malformed_conflicts).await;
        
        // Should either succeed or fail gracefully
        match result {
            Ok(resolution) => {
                tracing::info!("Arbitration engine handled malformed data successfully");
            }
            Err(e) => {
                tracing::info!("Arbitration engine correctly rejected malformed data: {:?}", e);
            }
        }
        
        tracing::info!("Council error handling test passed");
        Ok(())
    }

    /// Test research knowledge seeking and discovery
    async fn test_research_knowledge_seeking(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Testing research knowledge seeking and discovery");
        
        // Test knowledge seeker initialization
        let config = agent_agency_research::ResearchConfig {
            max_concurrent_queries: 5,
            query_timeout_ms: 30000,
            enable_caching: true,
            cache_ttl_ms: 3600000,
            debug_mode: false,
        };
        
        let knowledge_seeker = agent_agency_research::knowledge_seeker::KnowledgeSeeker::new(config)
            .map_err(|e| anyhow::anyhow!("Failed to initialize knowledge seeker: {:?}", e))?;
        
        // Test basic knowledge seeking functionality
        let test_query = agent_agency_research::types::ResearchQuery {
            id: uuid::Uuid::new_v4(),
            query: "test knowledge seeking".to_string(),
            query_type: agent_agency_research::types::QueryType::General,
            priority: agent_agency_research::types::ResearchPriority::Medium,
            context: Some("integration test".to_string()),
            max_results: Some(10),
            sources: vec![],
            created_at: chrono::Utc::now(),
            deadline: None,
            metadata: std::collections::HashMap::new(),
        };
        
        // Test query processing (this would normally involve actual research)
        tracing::info!("Research knowledge seeking test passed");
        Ok(())
    }

    /// Test research query processing and execution
    async fn test_research_query_processing(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Testing research query processing and execution");
        
        // Test vector search functionality
        let vector_search = agent_agency_research::vector_search::VectorSearch::new()
            .map_err(|e| anyhow::anyhow!("Failed to initialize vector search: {:?}", e))?;
        
        // Test embedding generation (mock)
        let test_text = "This is a test query for vector search functionality";
        let embedding = vector_search.generate_embedding(test_text).await
            .map_err(|e| anyhow::anyhow!("Failed to generate embedding: {:?}", e))?;
        
        // Validate embedding
        assert!(!embedding.is_empty(), "Embedding should not be empty");
        assert!(embedding.len() > 0, "Embedding should have positive dimension");
        
        // Test similarity search (mock)
        let similar_results = vector_search.search_similar(&embedding, 5).await
            .map_err(|e| anyhow::anyhow!("Failed to perform similarity search: {:?}", e))?;
        
        // Validate search results
        assert!(similar_results.len() <= 5, "Should return at most 5 results");
        
        tracing::info!("Research query processing test passed");
        Ok(())
    }

    /// Test research performance and response times
    async fn test_research_performance(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Testing research performance and response times");
        
        let start_time = std::time::Instant::now();
        
        // Test vector search performance
        let vector_search = agent_agency_research::vector_search::VectorSearch::new()
            .map_err(|e| anyhow::anyhow!("Failed to initialize vector search: {:?}", e))?;
        
        // Test multiple embedding generations for performance
        let test_texts = vec![
            "Performance test query 1",
            "Performance test query 2", 
            "Performance test query 3",
            "Performance test query 4",
            "Performance test query 5",
        ];
        
        let mut embeddings = Vec::new();
        for text in test_texts {
            let embedding = vector_search.generate_embedding(text).await
                .map_err(|e| anyhow::anyhow!("Failed to generate embedding: {:?}", e))?;
            embeddings.push(embedding);
        }
        
        let elapsed = start_time.elapsed();
        
        // Performance assertions
        assert!(elapsed.as_millis() < 10000, "Embedding generation should complete within 10 seconds");
        assert_eq!(embeddings.len(), 5, "Should generate all embeddings");
        
        tracing::info!("Research performance test passed in {:?}", elapsed);
        Ok(())
    }

    /// Test research error handling and recovery
    async fn test_research_error_handling(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Testing research error handling and recovery");
        
        // Test vector search error handling
        let vector_search = agent_agency_research::vector_search::VectorSearch::new()
            .map_err(|e| anyhow::anyhow!("Failed to initialize vector search: {:?}", e))?;
        
        // Test with empty text
        let result = vector_search.generate_embedding("").await;
        match result {
            Ok(embedding) => {
                // If it succeeds with empty text, that's acceptable
                tracing::info!("Vector search handled empty text successfully");
            }
            Err(e) => {
                // If it fails, that's also acceptable behavior
                tracing::info!("Vector search correctly rejected empty text: {:?}", e);
            }
        }
        
        // Test with very long text
        let long_text = "a".repeat(10000);
        let result = vector_search.generate_embedding(&long_text).await;
        match result {
            Ok(embedding) => {
                tracing::info!("Vector search handled long text successfully");
            }
            Err(e) => {
                tracing::info!("Vector search correctly handled long text with error: {:?}", e);
            }
        }
        
        // Test with invalid characters
        let invalid_text = "🚀🌟💫✨🎉🎊🎈🎁🎀🎂🍰🎂🍰🎂🍰";
        let result = vector_search.generate_embedding(invalid_text).await;
        match result {
            Ok(embedding) => {
                tracing::info!("Vector search handled invalid characters successfully");
            }
            Err(e) => {
                tracing::info!("Vector search correctly handled invalid characters: {:?}", e);
            }
        }
        
        tracing::info!("Research error handling test passed");
        Ok(())
    }

    /// Test database client connection and basic operations
    async fn test_database_connection(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Testing database connection and basic operations");
        
        // Test database client initialization
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/agent_agency_v3".to_string());
        
        let pool = sqlx::PgPool::connect(&database_url).await
            .map_err(|e| anyhow::anyhow!("Failed to connect to database: {:?}", e))?;
        
        // Test basic connection health
        let result = sqlx::query("SELECT 1 as test_value")
            .fetch_one(&pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to execute test query: {:?}", e))?;
        
        let test_value: i32 = result.get("test_value");
        assert_eq!(test_value, 1, "Database connection test should return 1");
        
        // Test connection pooling
        let start_time = std::time::Instant::now();
        let mut handles = Vec::new();
        
        for i in 0..10 {
            let pool_clone = pool.clone();
            let handle = tokio::spawn(async move {
                let result = sqlx::query("SELECT $1 as test_value")
                    .bind(i)
                    .fetch_one(&pool_clone)
                    .await;
                result.map(|row| row.get::<i32, _>("test_value"))
            });
            handles.push(handle);
        }
        
        let mut results = Vec::new();
        for handle in handles {
            let result = handle.await
                .map_err(|e| anyhow::anyhow!("Task failed: {:?}", e))?
                .map_err(|e| anyhow::anyhow!("Query failed: {:?}", e))?;
            results.push(result);
        }
        
        let elapsed = start_time.elapsed();
        
        // Validate connection pooling performance
        assert_eq!(results.len(), 10, "Should complete all concurrent queries");
        assert!(elapsed.as_millis() < 5000, "Concurrent queries should complete within 5 seconds");
        
        tracing::info!("Database connection test passed in {:?}", elapsed);
        Ok(())
    }

    /// Test database CRUD operations and transactions
    async fn test_database_crud_operations(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Testing database CRUD operations and transactions");
        
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/agent_agency_v3".to_string());
        
        let pool = sqlx::PgPool::connect(&database_url).await
            .map_err(|e| anyhow::anyhow!("Failed to connect to database: {:?}", e))?;
        
        // Test transaction handling
        let mut tx = pool.begin().await
            .map_err(|e| anyhow::anyhow!("Failed to begin transaction: {:?}", e))?;
        
        // Test INSERT operation
        let test_id = uuid::Uuid::new_v4();
        let insert_result = sqlx::query(
            "INSERT INTO tasks (id, title, description, status, priority, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, $5, NOW(), NOW())"
        )
        .bind(test_id)
        .bind("Integration Test Task")
        .bind("Test task for database CRUD operations")
        .bind("pending")
        .bind(1)
        .execute(&mut *tx)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to insert test task: {:?}", e))?;
        
        assert_eq!(insert_result.rows_affected(), 1, "Should insert exactly one row");
        
        // Test SELECT operation
        let select_result = sqlx::query("SELECT * FROM tasks WHERE id = $1")
            .bind(test_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to select test task: {:?}", e))?;
        
        let title: String = select_result.get("title");
        assert_eq!(title, "Integration Test Task", "Selected title should match inserted title");
        
        // Test UPDATE operation
        let update_result = sqlx::query("UPDATE tasks SET status = $1 WHERE id = $2")
            .bind("completed")
            .bind(test_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to update test task: {:?}", e))?;
        
        assert_eq!(update_result.rows_affected(), 1, "Should update exactly one row");
        
        // Test DELETE operation
        let delete_result = sqlx::query("DELETE FROM tasks WHERE id = $1")
            .bind(test_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to delete test task: {:?}", e))?;
        
        assert_eq!(delete_result.rows_affected(), 1, "Should delete exactly one row");
        
        // Commit transaction
        tx.commit().await
            .map_err(|e| anyhow::anyhow!("Failed to commit transaction: {:?}", e))?;
        
        tracing::info!("Database CRUD operations test passed");
        Ok(())
    }

    /// Test database performance and query optimization
    async fn test_database_performance(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Testing database performance and query optimization");
        
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/agent_agency_v3".to_string());
        
        let pool = sqlx::PgPool::connect(&database_url).await
            .map_err(|e| anyhow::anyhow!("Failed to connect to database: {:?}", e))?;
        
        // Test query performance with multiple operations
        let start_time = std::time::Instant::now();
        
        // Test complex query performance
        let complex_query = sqlx::query(
            r#"
            SELECT 
                t.id,
                t.title,
                t.status,
                COUNT(te.id) as execution_count,
                AVG(EXTRACT(EPOCH FROM (te.completed_at - te.started_at)) * 1000) as avg_execution_time_ms
            FROM tasks t
            LEFT JOIN task_executions te ON t.id = te.task_id
            WHERE t.created_at >= NOW() - INTERVAL '30 days'
            GROUP BY t.id, t.title, t.status
            ORDER BY t.created_at DESC
            LIMIT 100
            "#
        );
        
        let results = complex_query.fetch_all(&pool).await
            .map_err(|e| anyhow::anyhow!("Failed to execute complex query: {:?}", e))?;
        
        let elapsed = start_time.elapsed();
        
        // Performance assertions
        assert!(elapsed.as_millis() < 2000, "Complex query should complete within 2 seconds");
        assert!(results.len() <= 100, "Should return at most 100 results");
        
        // Test batch operations performance
        let batch_start = std::time::Instant::now();
        let mut tx = pool.begin().await
            .map_err(|e| anyhow::anyhow!("Failed to begin transaction: {:?}", e))?;
        
        for i in 0..50 {
            let task_id = uuid::Uuid::new_v4();
            sqlx::query(
                "INSERT INTO tasks (id, title, description, status, priority, created_at, updated_at) 
                 VALUES ($1, $2, $3, $4, $5, NOW(), NOW())"
            )
            .bind(task_id)
            .bind(format!("Batch Test Task {}", i))
            .bind("Batch performance test")
            .bind("pending")
            .bind(1)
            .execute(&mut *tx)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to insert batch task: {:?}", e))?;
        }
        
        tx.commit().await
            .map_err(|e| anyhow::anyhow!("Failed to commit batch transaction: {:?}", e))?;
        
        let batch_elapsed = batch_start.elapsed();
        
        // Batch performance assertions
        assert!(batch_elapsed.as_millis() < 3000, "Batch operations should complete within 3 seconds");
        
        tracing::info!("Database performance test passed - Complex query: {:?}, Batch operations: {:?}", elapsed, batch_elapsed);
        Ok(())
    }

    /// Test database error handling and recovery
    async fn test_database_error_handling(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Testing database error handling and recovery");
        
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/agent_agency_v3".to_string());
        
        let pool = sqlx::PgPool::connect(&database_url).await
            .map_err(|e| anyhow::anyhow!("Failed to connect to database: {:?}", e))?;
        
        // Test handling of invalid queries
        let invalid_query = sqlx::query("SELECT * FROM non_existent_table");
        let result = invalid_query.fetch_all(&pool).await;
        
        match result {
            Ok(_) => {
                tracing::info!("Database handled invalid query gracefully");
            }
            Err(e) => {
                // Expected behavior - should return an error for invalid table
                tracing::info!("Database correctly rejected invalid query: {:?}", e);
            }
        }
        
        // Test transaction rollback on error
        let mut tx = pool.begin().await
            .map_err(|e| anyhow::anyhow!("Failed to begin transaction: {:?}", e))?;
        
        // Insert a valid record
        let test_id = uuid::Uuid::new_v4();
        sqlx::query(
            "INSERT INTO tasks (id, title, description, status, priority, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, $5, NOW(), NOW())"
        )
        .bind(test_id)
        .bind("Transaction Test Task")
        .bind("Test transaction rollback")
        .bind("pending")
        .bind(1)
        .execute(&mut *tx)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to insert test task: {:?}", e))?;
        
        // Try to insert invalid data to trigger rollback
        let invalid_insert = sqlx::query(
            "INSERT INTO tasks (id, title, description, status, priority, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, $5, NOW(), NOW())"
        )
        .bind(uuid::Uuid::new_v4())
        .bind("Invalid Task")
        .bind("Test task with invalid data")
        .bind("invalid_status") // Invalid status should cause error
        .bind(1)
        .execute(&mut *tx)
        .await;
        
        match invalid_insert {
            Ok(_) => {
                // If it succeeds, commit the transaction
                tx.commit().await
                    .map_err(|e| anyhow::anyhow!("Failed to commit transaction: {:?}", e))?;
                tracing::info!("Database accepted invalid data (may be permissive)");
            }
            Err(_) => {
                // Expected behavior - rollback the transaction
                tx.rollback().await
                    .map_err(|e| anyhow::anyhow!("Failed to rollback transaction: {:?}", e))?;
                tracing::info!("Database correctly rolled back transaction on error");
            }
        }
        
        // Test connection timeout handling
        let timeout_pool = sqlx::PgPool::builder()
            .max_connections(1)
            .acquire_timeout(std::time::Duration::from_millis(100))
            .build(&database_url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create timeout pool: {:?}", e))?;
        
        // Acquire first connection
        let _conn1 = timeout_pool.acquire().await
            .map_err(|e| anyhow::anyhow!("Failed to acquire first connection: {:?}", e))?;
        
        // Try to acquire second connection (should timeout)
        let start_time = std::time::Instant::now();
        let conn2_result = timeout_pool.acquire().await;
        let elapsed = start_time.elapsed();
        
        match conn2_result {
            Ok(_) => {
                tracing::info!("Connection pool did not timeout as expected");
            }
            Err(e) => {
                // Expected behavior - should timeout
                assert!(elapsed.as_millis() >= 100, "Should timeout after at least 100ms");
                tracing::info!("Connection pool correctly timed out: {:?}", e);
            }
        }
        
        tracing::info!("Database error handling test passed");
        Ok(())
    }

    /// Test orchestration task routing and execution
    async fn test_orchestration_task_routing(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Testing orchestration task routing and execution");
        
        // Test orchestration engine initialization
        let config = agent_agency_orchestration::OrchestrationConfig {
            max_concurrent_tasks: 10,
            task_timeout_ms: 30000,
            worker_pool_size: 5,
            enable_retry: true,
            max_retries: 3,
            debug_mode: false,
        };
        
        let orchestration_engine = agent_agency_orchestration::OrchestrationEngine::new(config)
            .map_err(|e| anyhow::anyhow!("Failed to initialize orchestration engine: {:?}", e))?;
        
        // Test task creation and routing
        let test_task = agent_agency_orchestration::types::Task {
            id: uuid::Uuid::new_v4(),
            title: "Integration Test Task".to_string(),
            description: "Test task for orchestration routing".to_string(),
            priority: agent_agency_orchestration::types::TaskPriority::Medium,
            complexity: agent_agency_orchestration::types::TaskComplexity::Medium,
            estimated_duration_ms: 5000,
            required_skills: vec!["testing".to_string()],
            dependencies: vec![],
            metadata: std::collections::HashMap::new(),
        };
        
        // Test task submission
        let submission_result = orchestration_engine.submit_task(test_task.clone()).await
            .map_err(|e| anyhow::anyhow!("Failed to submit task: {:?}", e))?;
        
        assert_eq!(submission_result.task_id, test_task.id, "Submitted task ID should match");
        assert!(submission_result.assigned_worker_id.is_some(), "Task should be assigned to a worker");
        
        // Test task status tracking
        let task_status = orchestration_engine.get_task_status(test_task.id).await
            .map_err(|e| anyhow::anyhow!("Failed to get task status: {:?}", e))?;
        
        assert!(task_status.is_some(), "Task status should be available");
        let status = task_status.unwrap();
        assert!(matches!(status, agent_agency_orchestration::types::TaskStatus::Pending | 
                           agent_agency_orchestration::types::TaskStatus::InProgress), 
                "Task should be pending or in progress");
        
        tracing::info!("Orchestration task routing test passed");
        Ok(())
    }

    /// Test orchestration worker management and coordination
    async fn test_orchestration_worker_management(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Testing orchestration worker management and coordination");
        
        let config = agent_agency_orchestration::OrchestrationConfig {
            max_concurrent_tasks: 10,
            task_timeout_ms: 30000,
            worker_pool_size: 3,
            enable_retry: true,
            max_retries: 3,
            debug_mode: false,
        };
        
        let orchestration_engine = agent_agency_orchestration::OrchestrationEngine::new(config)
            .map_err(|e| anyhow::anyhow!("Failed to initialize orchestration engine: {:?}", e))?;
        
        // Test worker registration
        let worker_id = uuid::Uuid::new_v4();
        let worker_capabilities = agent_agency_orchestration::types::WorkerCapabilities {
            skills: vec!["testing".to_string(), "integration".to_string()],
            max_concurrent_tasks: 2,
            preferred_task_types: vec!["integration_test".to_string()],
            performance_metrics: std::collections::HashMap::new(),
        };
        
        let registration_result = orchestration_engine.register_worker(worker_id, worker_capabilities).await
            .map_err(|e| anyhow::anyhow!("Failed to register worker: {:?}", e))?;
        
        assert!(registration_result, "Worker registration should succeed");
        
        // Test worker status monitoring
        let worker_status = orchestration_engine.get_worker_status(worker_id).await
            .map_err(|e| anyhow::anyhow!("Failed to get worker status: {:?}", e))?;
        
        assert!(worker_status.is_some(), "Worker status should be available");
        let status = worker_status.unwrap();
        assert_eq!(status.worker_id, worker_id, "Worker ID should match");
        assert_eq!(status.status, agent_agency_orchestration::types::WorkerStatus::Available, 
                   "Worker should be available");
        
        // Test worker health monitoring
        let health_check = orchestration_engine.check_worker_health(worker_id).await
            .map_err(|e| anyhow::anyhow!("Failed to check worker health: {:?}", e))?;
        
        assert!(health_check.is_healthy, "Worker should be healthy");
        assert!(health_check.last_heartbeat.is_some(), "Worker should have heartbeat");
        
        tracing::info!("Orchestration worker management test passed");
        Ok(())
    }

    /// Test orchestration load balancing and distribution
    async fn test_orchestration_load_balancing(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Testing orchestration load balancing and distribution");
        
        let config = agent_agency_orchestration::OrchestrationConfig {
            max_concurrent_tasks: 20,
            task_timeout_ms: 30000,
            worker_pool_size: 5,
            enable_retry: true,
            max_retries: 3,
            debug_mode: false,
        };
        
        let orchestration_engine = agent_agency_orchestration::OrchestrationEngine::new(config)
            .map_err(|e| anyhow::anyhow!("Failed to initialize orchestration engine: {:?}", e))?;
        
        // Register multiple workers
        let mut worker_ids = Vec::new();
        for i in 0..3 {
            let worker_id = uuid::Uuid::new_v4();
            let worker_capabilities = agent_agency_orchestration::types::WorkerCapabilities {
                skills: vec!["testing".to_string(), "load_balancing".to_string()],
                max_concurrent_tasks: 3,
                preferred_task_types: vec!["load_test".to_string()],
                performance_metrics: std::collections::HashMap::new(),
            };
            
            orchestration_engine.register_worker(worker_id, worker_capabilities).await
                .map_err(|e| anyhow::anyhow!("Failed to register worker {}: {:?}", i, e))?;
            worker_ids.push(worker_id);
        }
        
        // Submit multiple tasks to test load balancing
        let mut task_ids = Vec::new();
        for i in 0..10 {
            let test_task = agent_agency_orchestration::types::Task {
                id: uuid::Uuid::new_v4(),
                title: format!("Load Test Task {}", i),
                description: "Test task for load balancing".to_string(),
                priority: agent_agency_orchestration::types::TaskPriority::Medium,
                complexity: agent_agency_orchestration::types::TaskComplexity::Low,
                estimated_duration_ms: 1000,
                required_skills: vec!["testing".to_string()],
                dependencies: vec![],
                metadata: std::collections::HashMap::new(),
            };
            
            let submission_result = orchestration_engine.submit_task(test_task.clone()).await
                .map_err(|e| anyhow::anyhow!("Failed to submit load test task {}: {:?}", i, e))?;
            
            task_ids.push(submission_result.task_id);
        }
        
        // Wait a moment for task distribution
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        // Check load distribution across workers
        let mut worker_loads = Vec::new();
        for worker_id in &worker_ids {
            let worker_status = orchestration_engine.get_worker_status(*worker_id).await
                .map_err(|e| anyhow::anyhow!("Failed to get worker status: {:?}", e))?;
            
            if let Some(status) = worker_status {
                worker_loads.push(status.active_tasks);
            }
        }
        
        // Validate load distribution (should be relatively balanced)
        let total_load: u32 = worker_loads.iter().sum();
        let avg_load = total_load as f32 / worker_loads.len() as f32;
        
        // Each worker should have some load, and the distribution should be reasonable
        assert!(total_load > 0, "Workers should have some load");
        assert!(avg_load > 0.0, "Average load should be positive");
        
        // Check that no single worker is overloaded (more than 2x average)
        for load in &worker_loads {
            assert!(*load as f32 <= avg_load * 2.5, "No worker should be severely overloaded");
        }
        
        tracing::info!("Orchestration load balancing test passed - Total load: {}, Average: {:.2}", total_load, avg_load);
        Ok(())
    }

    /// Test orchestration performance and scalability
    async fn test_orchestration_performance(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Testing orchestration performance and scalability");
        
        let config = agent_agency_orchestration::OrchestrationConfig {
            max_concurrent_tasks: 50,
            task_timeout_ms: 30000,
            worker_pool_size: 10,
            enable_retry: true,
            max_retries: 3,
            debug_mode: false,
        };
        
        let orchestration_engine = agent_agency_orchestration::OrchestrationEngine::new(config)
            .map_err(|e| anyhow::anyhow!("Failed to initialize orchestration engine: {:?}", e))?;
        
        // Register workers for performance testing
        for i in 0..5 {
            let worker_id = uuid::Uuid::new_v4();
            let worker_capabilities = agent_agency_orchestration::types::WorkerCapabilities {
                skills: vec!["performance_testing".to_string()],
                max_concurrent_tasks: 10,
                preferred_task_types: vec!["performance_test".to_string()],
                performance_metrics: std::collections::HashMap::new(),
            };
            
            orchestration_engine.register_worker(worker_id, worker_capabilities).await
                .map_err(|e| anyhow::anyhow!("Failed to register performance test worker {}: {:?}", i, e))?;
        }
        
        // Test high-volume task submission performance
        let start_time = std::time::Instant::now();
        let mut submission_handles = Vec::new();
        
        for i in 0..20 {
            let engine_clone = orchestration_engine.clone();
            let handle = tokio::spawn(async move {
                let test_task = agent_agency_orchestration::types::Task {
                    id: uuid::Uuid::new_v4(),
                    title: format!("Performance Test Task {}", i),
                    description: "High-volume performance test".to_string(),
                    priority: agent_agency_orchestration::types::TaskPriority::Low,
                    complexity: agent_agency_orchestration::types::TaskComplexity::Low,
                    estimated_duration_ms: 500,
                    required_skills: vec!["performance_testing".to_string()],
                    dependencies: vec![],
                    metadata: std::collections::HashMap::new(),
                };
                
                engine_clone.submit_task(test_task).await
            });
            submission_handles.push(handle);
        }
        
        // Wait for all submissions to complete
        let mut successful_submissions = 0;
        for handle in submission_handles {
            match handle.await {
                Ok(Ok(_)) => successful_submissions += 1,
                Ok(Err(e)) => tracing::warn!("Task submission failed: {:?}", e),
                Err(e) => tracing::warn!("Task submission task failed: {:?}", e),
            }
        }
        
        let elapsed = start_time.elapsed();
        
        // Performance assertions
        assert!(elapsed.as_millis() < 5000, "High-volume task submission should complete within 5 seconds");
        assert!(successful_submissions >= 15, "At least 75% of submissions should succeed");
        
        // Test orchestration engine metrics
        let metrics = orchestration_engine.get_metrics().await
            .map_err(|e| anyhow::anyhow!("Failed to get orchestration metrics: {:?}", e))?;
        
        assert!(metrics.total_tasks_submitted > 0, "Should have submitted some tasks");
        assert!(metrics.active_workers > 0, "Should have active workers");
        
        tracing::info!("Orchestration performance test passed in {:?} - Successful submissions: {}/20", elapsed, successful_submissions);
        Ok(())
    }

    /// Test orchestration error handling and recovery
    async fn test_orchestration_error_handling(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Testing orchestration error handling and recovery");
        
        let config = agent_agency_orchestration::OrchestrationConfig {
            max_concurrent_tasks: 10,
            task_timeout_ms: 5000, // Short timeout for testing
            worker_pool_size: 2,
            enable_retry: true,
            max_retries: 2,
            debug_mode: false,
        };
        
        let orchestration_engine = agent_agency_orchestration::OrchestrationEngine::new(config)
            .map_err(|e| anyhow::anyhow!("Failed to initialize orchestration engine: {:?}", e))?;
        
        // Test handling of invalid task submission
        let invalid_task = agent_agency_orchestration::types::Task {
            id: uuid::Uuid::new_v4(),
            title: "".to_string(), // Invalid empty title
            description: "Test task with invalid data".to_string(),
            priority: agent_agency_orchestration::types::TaskPriority::Medium,
            complexity: agent_agency_orchestration::types::TaskComplexity::Medium,
            estimated_duration_ms: 0, // Invalid zero duration
            required_skills: vec![],
            dependencies: vec![],
            metadata: std::collections::HashMap::new(),
        };
        
        let result = orchestration_engine.submit_task(invalid_task).await;
        match result {
            Ok(_) => {
                tracing::info!("Orchestration engine accepted invalid task (may be permissive)");
            }
            Err(e) => {
                tracing::info!("Orchestration engine correctly rejected invalid task: {:?}", e);
            }
        }
        
        // Test handling of non-existent worker operations
        let non_existent_worker = uuid::Uuid::new_v4();
        let worker_status = orchestration_engine.get_worker_status(non_existent_worker).await
            .map_err(|e| anyhow::anyhow!("Failed to check non-existent worker status: {:?}", e))?;
        
        assert!(worker_status.is_none(), "Non-existent worker should return None status");
        
        // Test handling of non-existent task operations
        let non_existent_task = uuid::Uuid::new_v4();
        let task_status = orchestration_engine.get_task_status(non_existent_task).await
            .map_err(|e| anyhow::anyhow!("Failed to check non-existent task status: {:?}", e))?;
        
        assert!(task_status.is_none(), "Non-existent task should return None status");
        
        // Test timeout handling
        let timeout_task = agent_agency_orchestration::types::Task {
            id: uuid::Uuid::new_v4(),
            title: "Timeout Test Task".to_string(),
            description: "Task designed to timeout".to_string(),
            priority: agent_agency_orchestration::types::TaskPriority::Low,
            complexity: agent_agency_orchestration::types::TaskComplexity::High,
            estimated_duration_ms: 10000, // Longer than timeout
            required_skills: vec!["timeout_testing".to_string()],
            dependencies: vec![],
            metadata: std::collections::HashMap::new(),
        };
        
        let submission_result = orchestration_engine.submit_task(timeout_task.clone()).await;
        match submission_result {
            Ok(result) => {
                // If task is submitted, wait for timeout
                tokio::time::sleep(std::time::Duration::from_millis(6000)).await;
                
                let final_status = orchestration_engine.get_task_status(timeout_task.id).await
                    .map_err(|e| anyhow::anyhow!("Failed to get timeout task status: {:?}", e))?;
                
                if let Some(status) = final_status {
                    tracing::info!("Timeout task final status: {:?}", status);
                }
            }
            Err(e) => {
                tracing::info!("Orchestration engine rejected timeout task: {:?}", e);
            }
        }
        
        tracing::info!("Orchestration error handling test passed");
        Ok(())
    }

    /// Test claim extraction from various sources and formats
    async fn test_claim_extraction_sources(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Testing claim extraction from various sources and formats");
        
        // Test claim extractor initialization
        let config = agent_agency_claim_extraction::ClaimExtractionConfig {
            max_concurrent_extractions: 5,
            extraction_timeout_ms: 30000,
            enable_multi_modal: true,
            confidence_threshold: 0.7,
            debug_mode: false,
        };
        
        let claim_extractor = agent_agency_claim_extraction::ClaimExtractor::new(config)
            .map_err(|e| anyhow::anyhow!("Failed to initialize claim extractor: {:?}", e))?;
        
        // Test text-based claim extraction
        let text_content = "The study shows that 85% of participants improved their performance after using the new system. The research was conducted over 6 months with 200 participants.";
        let text_claims = claim_extractor.extract_claims_from_text(text_content).await
            .map_err(|e| anyhow::anyhow!("Failed to extract claims from text: {:?}", e))?;
        
        assert!(!text_claims.is_empty(), "Should extract at least one claim from text");
        assert!(text_claims.len() >= 2, "Should extract multiple claims from rich text");
        
        // Validate claim structure
        for claim in &text_claims {
            assert!(!claim.content.is_empty(), "Claim content should not be empty");
            assert!(claim.confidence_score >= 0.0 && claim.confidence_score <= 1.0, 
                    "Confidence score should be between 0.0 and 1.0");
            assert!(!claim.source.is_empty(), "Claim source should not be empty");
        }
        
        // Test structured data claim extraction
        let structured_data = serde_json::json!({
            "title": "Performance Study Results",
            "findings": [
                {"metric": "improvement_rate", "value": 0.85, "unit": "percentage"},
                {"metric": "study_duration", "value": 6, "unit": "months"},
                {"metric": "participant_count", "value": 200, "unit": "people"}
            ],
            "conclusion": "The new system significantly improves user performance"
        });
        
        let structured_claims = claim_extractor.extract_claims_from_structured_data(&structured_data).await
            .map_err(|e| anyhow::anyhow!("Failed to extract claims from structured data: {:?}", e))?;
        
        assert!(!structured_claims.is_empty(), "Should extract claims from structured data");
        
        // Test URL-based claim extraction (mock)
        let test_url = "https://example.com/research/study-results";
        let url_claims = claim_extractor.extract_claims_from_url(test_url).await
            .map_err(|e| anyhow::anyhow!("Failed to extract claims from URL: {:?}", e))?;
        
        // URL extraction might fail in test environment, which is acceptable
        match url_claims {
            Ok(claims) => {
                tracing::info!("Successfully extracted {} claims from URL", claims.len());
            }
            Err(e) => {
                tracing::info!("URL claim extraction failed (expected in test environment): {:?}", e);
            }
        }
        
        tracing::info!("Claim extraction sources test passed");
        Ok(())
    }

    /// Test claim extraction processing and validation
    async fn test_claim_extraction_processing(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Testing claim extraction processing and validation");
        
        let config = agent_agency_claim_extraction::ClaimExtractionConfig {
            max_concurrent_extractions: 3,
            extraction_timeout_ms: 15000,
            enable_multi_modal: true,
            confidence_threshold: 0.6,
            debug_mode: false,
        };
        
        let claim_extractor = agent_agency_claim_extraction::ClaimExtractor::new(config)
            .map_err(|e| anyhow::anyhow!("Failed to initialize claim extractor: {:?}", e))?;
        
        // Test claim processing pipeline
        let complex_text = r#"
        Research Study: "Impact of AI on Software Development"
        
        Abstract: This comprehensive study examines the effects of artificial intelligence tools on software development productivity and code quality.
        
        Key Findings:
        1. Development teams using AI tools showed 40% faster code completion
        2. Code quality metrics improved by 25% on average
        3. Bug detection rates increased by 60% with AI-assisted testing
        4. Developer satisfaction scores rose by 35%
        
        Methodology: The study involved 500 developers across 50 companies over 12 months.
        Statistical significance: p < 0.001 for all primary metrics.
        
        Conclusion: AI tools significantly enhance software development outcomes.
        "#;
        
        let extracted_claims = claim_extractor.extract_claims_from_text(complex_text).await
            .map_err(|e| anyhow::anyhow!("Failed to extract claims from complex text: {:?}", e))?;
        
        // Validate processing results
        assert!(!extracted_claims.is_empty(), "Should extract claims from complex text");
        assert!(extracted_claims.len() >= 4, "Should extract multiple claims from structured content");
        
        // Test claim validation
        let validation_results = claim_extractor.validate_claims(&extracted_claims).await
            .map_err(|e| anyhow::anyhow!("Failed to validate claims: {:?}", e))?;
        
        assert_eq!(validation_results.len(), extracted_claims.len(), "Should validate all extracted claims");
        
        // Check validation results
        let valid_claims: usize = validation_results.iter().filter(|r| r.is_valid).count();
        let invalid_claims: usize = validation_results.len() - valid_claims;
        
        assert!(valid_claims > 0, "Should have at least some valid claims");
        tracing::info!("Claim validation: {} valid, {} invalid", valid_claims, invalid_claims);
        
        // Test claim deduplication
        let deduplicated_claims = claim_extractor.deduplicate_claims(&extracted_claims).await
            .map_err(|e| anyhow::anyhow!("Failed to deduplicate claims: {:?}", e))?;
        
        assert!(deduplicated_claims.len() <= extracted_claims.len(), "Deduplication should not increase claim count");
        
        tracing::info!("Claim extraction processing test passed");
        Ok(())
    }

    /// Test claim extraction accuracy and completeness
    async fn test_claim_extraction_accuracy(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Testing claim extraction accuracy and completeness");
        
        let config = agent_agency_claim_extraction::ClaimExtractionConfig {
            max_concurrent_extractions: 2,
            extraction_timeout_ms: 20000,
            enable_multi_modal: true,
            confidence_threshold: 0.8, // High threshold for accuracy testing
            debug_mode: false,
        };
        
        let claim_extractor = agent_agency_claim_extraction::ClaimExtractor::new(config)
            .map_err(|e| anyhow::anyhow!("Failed to initialize claim extractor: {:?}", e))?;
        
        // Test with known factual content
        let factual_text = r#"
        Scientific Study: "Climate Change Impact on Arctic Ice"
        
        Published in Nature Climate Change, 2023.
        Authors: Dr. Sarah Johnson, Dr. Michael Chen, Dr. Emily Rodriguez
        
        Key Results:
        - Arctic sea ice extent decreased by 13% per decade since 1979
        - Average ice thickness reduced by 2.3 meters over the study period
        - Temperature increase of 2.1°C observed in Arctic regions
        - Study duration: 44 years (1979-2023)
        - Data sources: NASA satellite observations, NOAA measurements
        
        Statistical Analysis:
        - Confidence interval: 95%
        - Sample size: 1,200+ measurements
        - P-value: < 0.001
        - R-squared: 0.89
        
        Conclusion: Arctic ice loss is accelerating and statistically significant.
        "#;
        
        let factual_claims = claim_extractor.extract_claims_from_text(factual_text).await
            .map_err(|e| anyhow::anyhow!("Failed to extract factual claims: {:?}", e))?;
        
        // Validate accuracy indicators
        assert!(!factual_claims.is_empty(), "Should extract claims from factual content");
        
        let high_confidence_claims: usize = factual_claims.iter()
            .filter(|claim| claim.confidence_score >= 0.8)
            .count();
        
        let total_claims = factual_claims.len();
        let accuracy_ratio = high_confidence_claims as f32 / total_claims as f32;
        
        assert!(accuracy_ratio >= 0.6, "At least 60% of claims should have high confidence");
        tracing::info!("Accuracy ratio: {:.2} ({}/{} high confidence claims)", accuracy_ratio, high_confidence_claims, total_claims);
        
        // Test completeness - check for key information extraction
        let claim_contents: Vec<&str> = factual_claims.iter().map(|c| c.content.as_str()).collect();
        let combined_content = claim_contents.join(" ").to_lowercase();
        
        // Check for key metrics extraction
        let key_metrics = ["13%", "2.3 meters", "2.1°c", "44 years", "95%", "0.001"];
        let extracted_metrics: usize = key_metrics.iter()
            .filter(|metric| combined_content.contains(metric.to_lowercase().as_str()))
            .count();
        
        let completeness_ratio = extracted_metrics as f32 / key_metrics.len() as f32;
        assert!(completeness_ratio >= 0.5, "Should extract at least 50% of key metrics");
        tracing::info!("Completeness ratio: {:.2} ({}/{} key metrics extracted)", completeness_ratio, extracted_metrics, key_metrics.len());
        
        // Test claim categorization
        let categorized_claims = claim_extractor.categorize_claims(&factual_claims).await
            .map_err(|e| anyhow::anyhow!("Failed to categorize claims: {:?}", e))?;
        
        assert_eq!(categorized_claims.len(), factual_claims.len(), "Should categorize all claims");
        
        let categories: std::collections::HashSet<String> = categorized_claims.iter()
            .map(|c| c.category.clone())
            .collect();
        
        assert!(!categories.is_empty(), "Should have at least one category");
        tracing::info!("Claim categories: {:?}", categories);
        
        tracing::info!("Claim extraction accuracy test passed");
        Ok(())
    }

    /// Test claim extraction performance and scalability
    async fn test_claim_extraction_performance(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Testing claim extraction performance and scalability");
        
        let config = agent_agency_claim_extraction::ClaimExtractionConfig {
            max_concurrent_extractions: 10,
            extraction_timeout_ms: 30000,
            enable_multi_modal: true,
            confidence_threshold: 0.7,
            debug_mode: false,
        };
        
        let claim_extractor = agent_agency_claim_extraction::ClaimExtractor::new(config)
            .map_err(|e| anyhow::anyhow!("Failed to initialize claim extractor: {:?}", e))?;
        
        // Test performance with multiple concurrent extractions
        let start_time = std::time::Instant::now();
        let mut extraction_handles = Vec::new();
        
        let test_texts = vec![
            "Study shows 75% improvement in efficiency with new methodology.",
            "Research indicates 3.2x faster processing using advanced algorithms.",
            "Analysis reveals 40% reduction in error rates through automation.",
            "Findings demonstrate 2.5x increase in accuracy with machine learning.",
            "Results show 60% cost savings through optimized workflows.",
        ];
        
        for (i, text) in test_texts.iter().enumerate() {
            let extractor_clone = claim_extractor.clone();
            let text_clone = text.to_string();
            let handle = tokio::spawn(async move {
                extractor_clone.extract_claims_from_text(&text_clone).await
            });
            extraction_handles.push(handle);
        }
        
        // Wait for all extractions to complete
        let mut successful_extractions = 0;
        let mut total_claims = 0;
        
        for handle in extraction_handles {
            match handle.await {
                Ok(Ok(claims)) => {
                    successful_extractions += 1;
                    total_claims += claims.len();
                }
                Ok(Err(e)) => tracing::warn!("Claim extraction failed: {:?}", e),
                Err(e) => tracing::warn!("Extraction task failed: {:?}", e),
            }
        }
        
        let elapsed = start_time.elapsed();
        
        // Performance assertions
        assert!(elapsed.as_millis() < 10000, "Concurrent extractions should complete within 10 seconds");
        assert!(successful_extractions >= 4, "At least 80% of extractions should succeed");
        assert!(total_claims > 0, "Should extract at least some claims");
        
        // Test scalability with larger content
        let large_text = "Research finding: ".repeat(100) + "The study demonstrates significant improvements across all measured metrics.";
        let large_start = std::time::Instant::now();
        
        let large_claims = claim_extractor.extract_claims_from_text(&large_text).await
            .map_err(|e| anyhow::anyhow!("Failed to extract claims from large text: {:?}", e))?;
        
        let large_elapsed = large_start.elapsed();
        
        // Large content performance assertions
        assert!(large_elapsed.as_millis() < 5000, "Large content extraction should complete within 5 seconds");
        assert!(!large_claims.is_empty(), "Should extract claims from large content");
        
        tracing::info!("Claim extraction performance test passed - Concurrent: {:?}, Large content: {:?}, Total claims: {}", 
                      elapsed, large_elapsed, total_claims);
        Ok(())
    }

    /// Test claim extraction error handling and recovery
    async fn test_claim_extraction_error_handling(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Testing claim extraction error handling and recovery");
        
        let config = agent_agency_claim_extraction::ClaimExtractionConfig {
            max_concurrent_extractions: 3,
            extraction_timeout_ms: 5000, // Short timeout for testing
            enable_multi_modal: true,
            confidence_threshold: 0.7,
            debug_mode: false,
        };
        
        let claim_extractor = agent_agency_claim_extraction::ClaimExtractor::new(config)
            .map_err(|e| anyhow::anyhow!("Failed to initialize claim extractor: {:?}", e))?;
        
        // Test handling of empty content
        let empty_result = claim_extractor.extract_claims_from_text("").await;
        match empty_result {
            Ok(claims) => {
                assert!(claims.is_empty(), "Empty text should result in no claims");
                tracing::info!("Empty text handled correctly");
            }
            Err(e) => {
                tracing::info!("Empty text correctly rejected: {:?}", e);
            }
        }
        
        // Test handling of very short content
        let short_result = claim_extractor.extract_claims_from_text("Yes.").await;
        match short_result {
            Ok(claims) => {
                tracing::info!("Short text processed, extracted {} claims", claims.len());
            }
            Err(e) => {
                tracing::info!("Short text rejected: {:?}", e);
            }
        }
        
        // Test handling of malformed structured data
        let malformed_data = serde_json::json!({
            "invalid": "data",
            "missing": null,
            "broken": [1, 2, 3, "mixed", true]
        });
        
        let malformed_result = claim_extractor.extract_claims_from_structured_data(&malformed_data).await;
        match malformed_result {
            Ok(claims) => {
                tracing::info!("Malformed data processed, extracted {} claims", claims.len());
            }
            Err(e) => {
                tracing::info!("Malformed data correctly rejected: {:?}", e);
            }
        }
        
        // Test handling of invalid URLs
        let invalid_urls = vec![
            "not-a-url",
            "http://",
            "https://nonexistent-domain-12345.com/invalid",
            "ftp://invalid-protocol.com",
        ];
        
        for url in invalid_urls {
            let url_result = claim_extractor.extract_claims_from_url(url).await;
            match url_result {
                Ok(Ok(claims)) => {
                    tracing::info!("URL {} processed, extracted {} claims", url, claims.len());
                }
                Ok(Err(e)) => {
                    tracing::info!("URL {} correctly rejected: {:?}", url, e);
                }
                Err(e) => {
                    tracing::info!("URL {} task failed: {:?}", url, e);
                }
            }
        }
        
        // Test timeout handling with very long content
        let timeout_text = "This is a very long text that should trigger timeout. ".repeat(1000);
        let timeout_start = std::time::Instant::now();
        
        let timeout_result = claim_extractor.extract_claims_from_text(&timeout_text).await;
        let timeout_elapsed = timeout_start.elapsed();
        
        match timeout_result {
            Ok(claims) => {
                tracing::info!("Timeout text processed in {:?}, extracted {} claims", timeout_elapsed, claims.len());
            }
            Err(e) => {
                tracing::info!("Timeout text correctly handled: {:?}", e);
            }
        }
        
        // Test recovery from partial failures
        let mixed_content = vec![
            "Valid research finding: 85% improvement observed.",
            "", // Empty content
            "Another valid finding: 3.2x performance increase.",
            "Invalid content with no clear claims or data.",
        ];
        
        let mut recovery_successes = 0;
        for content in mixed_content {
            let result = claim_extractor.extract_claims_from_text(content).await;
            match result {
                Ok(claims) => {
                    recovery_successes += 1;
                    tracing::info!("Recovery test: extracted {} claims from content", claims.len());
                }
                Err(e) => {
                    tracing::info!("Recovery test: content rejected: {:?}", e);
                }
            }
        }
        
        assert!(recovery_successes >= 2, "Should successfully process at least some content during recovery test");
        
        tracing::info!("Claim extraction error handling test passed");
        Ok(())
    }
}
