//! Integration test configuration
//!
//! This module contains configuration structures for integration tests.

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
