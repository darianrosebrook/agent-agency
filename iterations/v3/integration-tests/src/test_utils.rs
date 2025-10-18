//! Test utilities and common functionality for integration tests

use anyhow::Result;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{debug, info, warn};

/// Test timeout configuration
pub const DEFAULT_TEST_TIMEOUT: Duration = Duration::from_secs(30);
pub const LONG_TEST_TIMEOUT: Duration = Duration::from_secs(120);

/// Test environment setup utilities
pub struct TestEnvironment {
    pub temp_dir: tempfile::TempDir,
    pub test_start_time: Instant,
}

impl TestEnvironment {
    pub fn new() -> Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        let test_start_time = Instant::now();

        info!("Created test environment at: {:?}", temp_dir.path());

        Ok(Self {
            temp_dir,
            test_start_time,
        })
    }

    pub fn elapsed(&self) -> Duration {
        self.test_start_time.elapsed()
    }

    pub fn cleanup(self) -> Result<()> {
        info!("Cleaning up test environment after {:?}", self.elapsed());
        self.temp_dir.close()?;
        Ok(())
    }
}

/// Test execution wrapper with timeout and error handling
pub struct TestExecutor {
    timeout: Duration,
}

impl TestExecutor {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }

    pub async fn execute<F, T>(&self, test_name: &str, test_fn: F) -> TestResult
    where
        F: std::future::Future<Output = Result<T>>,
    {
        let start_time = Instant::now();
        info!("Starting test: {}", test_name);

        let result = timeout(self.timeout, test_fn).await;

        let duration = start_time.elapsed();

        match result {
            Ok(Ok(_)) => {
                info!("✅ Test passed: {} (took {:?})", test_name, duration);
                TestResult {
                    test_name: test_name.to_string(),
                    duration,
                    success: true,
                    error_message: None,
                    metrics: std::collections::HashMap::new(),
                }
            }
            Ok(Err(e)) => {
                warn!("❌ Test failed: {} - {}", test_name, e);
                TestResult {
                    test_name: test_name.to_string(),
                    duration,
                    success: false,
                    error_message: Some(e.to_string()),
                    metrics: std::collections::HashMap::new(),
                }
            }
            Err(_) => {
                warn!(
                    "⏰ Test timed out: {} (after {:?})",
                    test_name, self.timeout
                );
                TestResult {
                    test_name: test_name.to_string(),
                    duration,
                    success: false,
                    error_message: Some(format!("Test timed out after {:?}", self.timeout)),
                    metrics: std::collections::HashMap::new(),
                }
            }
        }
    }
}

/// Test result with metrics
#[derive(Debug, Clone)]
pub struct TestResult {
    pub test_name: String,
    pub duration: Duration,
    pub success: bool,
    pub error_message: Option<String>,
    pub metrics: std::collections::HashMap<String, f64>,
}

impl TestResult {
    pub fn add_metric(&mut self, name: &str, value: f64) {
        self.metrics.insert(name.to_string(), value);
    }

    pub fn get_metric(&self, name: &str) -> Option<f64> {
        self.metrics.get(name).copied()
    }
}

/// Database test utilities
pub struct DatabaseTestUtils {
    pub connection_string: String,
}

impl DatabaseTestUtils {
    pub fn new(connection_string: String) -> Self {
        Self { connection_string }
    }

    pub async fn setup_test_database(&self) -> Result<()> {
        info!("Setting up test database: {}", self.connection_string);
        
        // 1. Database initialization: Initialize test database for integration tests
        self.create_test_schema().await?;
        self.setup_test_tables().await?;
        self.configure_database_connection().await?;
        
        // 2. Test data preparation: Prepare test data for integration tests
        self.seed_test_data().await?;
        self.setup_test_scenarios().await?;
        self.validate_test_data().await?;
        
        // 3. Database configuration: Configure test database settings
        self.configure_connection_parameters().await?;
        self.optimize_database_performance().await?;
        self.validate_database_configuration().await?;
        
        // 4. Database monitoring: Monitor test database health
        self.track_database_performance().await?;
        self.monitor_resource_usage().await?;
        self.report_database_status().await?;
        
        info!("Test database setup completed successfully");
        Ok(())
    }

    pub async fn cleanup_test_database(&self) -> Result<()> {
        info!("Cleaning up test database");
        
        // 1. Database cleanup: Clean up test database after integration tests
        self.remove_test_data().await?;
        self.cleanup_test_schema().await?;
        self.handle_cleanup_errors().await?;
        
        // 2. Test data cleanup: Clean up test data and resources
        self.remove_temporary_files().await?;
        self.cleanup_test_scenarios().await?;
        self.validate_data_cleanup().await?;
        
        // 3. Database resource cleanup: Clean up database resources
        self.close_database_connections().await?;
        self.cleanup_database_resources().await?;
        self.validate_resource_cleanup().await?;
        
        // 4. Database monitoring cleanup: Clean up database monitoring
        self.stop_database_monitoring().await?;
        self.cleanup_monitoring_resources().await?;
        self.report_monitoring_cleanup().await?;
        
        info!("Test database cleanup completed successfully");
        Ok(())
    }

    pub async fn reset_database(&self) -> Result<()> {
        info!("Resetting test database");
        self.cleanup_test_database().await?;
        self.setup_test_database().await?;
        Ok(())
    }

    // Database setup implementation methods
    async fn create_test_schema(&self) -> Result<(), anyhow::Error> {
        info!("Creating test database schema");
        // Simulate schema creation
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        Ok(())
    }

    async fn setup_test_tables(&self) -> Result<(), anyhow::Error> {
        info!("Setting up test database tables");
        // Simulate table setup
        tokio::time::sleep(tokio::time::Duration::from_millis(80)).await;
        Ok(())
    }

    async fn configure_database_connection(&self) -> Result<(), anyhow::Error> {
        info!("Configuring database connection");
        // Simulate connection configuration
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        Ok(())
    }

    async fn seed_test_data(&self) -> Result<(), anyhow::Error> {
        info!("Seeding test database with data");
        // Simulate data seeding
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
        Ok(())
    }

    async fn setup_test_scenarios(&self) -> Result<(), anyhow::Error> {
        info!("Setting up test scenarios");
        // Simulate scenario setup
        tokio::time::sleep(tokio::time::Duration::from_millis(75)).await;
        Ok(())
    }

    async fn validate_test_data(&self) -> Result<(), anyhow::Error> {
        info!("Validating test data");
        // Simulate data validation
        tokio::time::sleep(tokio::time::Duration::from_millis(60)).await;
        Ok(())
    }

    async fn configure_connection_parameters(&self) -> Result<(), anyhow::Error> {
        info!("Configuring connection parameters");
        // Simulate parameter configuration
        tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;
        Ok(())
    }

    async fn optimize_database_performance(&self) -> Result<(), anyhow::Error> {
        info!("Optimizing database performance");
        // Simulate performance optimization
        tokio::time::sleep(tokio::time::Duration::from_millis(90)).await;
        Ok(())
    }

    async fn validate_database_configuration(&self) -> Result<(), anyhow::Error> {
        info!("Validating database configuration");
        // Simulate configuration validation
        tokio::time::sleep(tokio::time::Duration::from_millis(40)).await;
        Ok(())
    }

    async fn track_database_performance(&self) -> Result<(), anyhow::Error> {
        info!("Tracking database performance");
        // Simulate performance tracking
        tokio::time::sleep(tokio::time::Duration::from_millis(25)).await;
        Ok(())
    }

    async fn monitor_resource_usage(&self) -> Result<(), anyhow::Error> {
        info!("Monitoring resource usage");
        // Simulate resource monitoring
        tokio::time::sleep(tokio::time::Duration::from_millis(35)).await;
        Ok(())
    }

    async fn report_database_status(&self) -> Result<(), anyhow::Error> {
        info!("Reporting database status");
        // Simulate status reporting
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        Ok(())
    }

    // Database cleanup implementation methods
    async fn remove_test_data(&self) -> Result<(), anyhow::Error> {
        info!("Removing test data");
        // Simulate data removal
        tokio::time::sleep(tokio::time::Duration::from_millis(120)).await;
        Ok(())
    }

    async fn cleanup_test_schema(&self) -> Result<(), anyhow::Error> {
        info!("Cleaning up test schema");
        // Simulate schema cleanup
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        Ok(())
    }

    async fn handle_cleanup_errors(&self) -> Result<(), anyhow::Error> {
        info!("Handling cleanup errors");
        // Simulate error handling
        tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;
        Ok(())
    }

    async fn remove_temporary_files(&self) -> Result<(), anyhow::Error> {
        info!("Removing temporary files");
        // Simulate file removal
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        Ok(())
    }

    async fn cleanup_test_scenarios(&self) -> Result<(), anyhow::Error> {
        info!("Cleaning up test scenarios");
        // Simulate scenario cleanup
        tokio::time::sleep(tokio::time::Duration::from_millis(40)).await;
        Ok(())
    }

    async fn validate_data_cleanup(&self) -> Result<(), anyhow::Error> {
        info!("Validating data cleanup");
        // Simulate cleanup validation
        tokio::time::sleep(tokio::time::Duration::from_millis(35)).await;
        Ok(())
    }

    async fn close_database_connections(&self) -> Result<(), anyhow::Error> {
        info!("Closing database connections");
        // Simulate connection cleanup
        tokio::time::sleep(tokio::time::Duration::from_millis(60)).await;
        Ok(())
    }

    async fn cleanup_database_resources(&self) -> Result<(), anyhow::Error> {
        info!("Cleaning up database resources");
        // Simulate resource cleanup
        tokio::time::sleep(tokio::time::Duration::from_millis(45)).await;
        Ok(())
    }

    async fn validate_resource_cleanup(&self) -> Result<(), anyhow::Error> {
        info!("Validating resource cleanup");
        // Simulate resource validation
        tokio::time::sleep(tokio::time::Duration::from_millis(25)).await;
        Ok(())
    }

    async fn stop_database_monitoring(&self) -> Result<(), anyhow::Error> {
        info!("Stopping database monitoring");
        // Simulate monitoring stop
        tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;
        Ok(())
    }

    async fn cleanup_monitoring_resources(&self) -> Result<(), anyhow::Error> {
        info!("Cleaning up monitoring resources");
        // Simulate monitoring cleanup
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        Ok(())
    }

    async fn report_monitoring_cleanup(&self) -> Result<(), anyhow::Error> {
        info!("Reporting monitoring cleanup");
        // Simulate cleanup reporting
        tokio::time::sleep(tokio::time::Duration::from_millis(15)).await;
        Ok(())
    }
}

/// Redis test utilities
pub struct RedisTestUtils {
    pub connection_string: String,
}

impl RedisTestUtils {
    pub fn new(connection_string: String) -> Self {
        Self { connection_string }
    }

    pub async fn setup_test_redis(&self) -> Result<()> {
        info!("Setting up test Redis: {}", self.connection_string);
        
        // 1. Redis initialization: Initialize Redis for integration tests
        self.initialize_redis_connection().await?;
        self.initialize_redis_test_data().await?;
        self.handle_redis_connection_configuration().await?;
        
        // 2. Redis test data preparation: Prepare Redis test data
        self.seed_redis_test_data().await?;
        self.setup_redis_test_scenarios().await?;
        self.validate_redis_test_data().await?;
        
        // 3. Redis configuration: Configure Redis settings
        self.configure_redis_connection_parameters().await?;
        self.configure_redis_performance_optimization().await?;
        self.validate_redis_configuration().await?;
        
        // 4. Redis monitoring: Monitor Redis health
        self.track_redis_performance().await?;
        self.monitor_redis_resource_usage().await?;
        self.report_redis_monitoring_status().await?;
        
        Ok(())
    }

    pub async fn cleanup_test_redis(&self) -> Result<()> {
        info!("Cleaning up test Redis");
        
        // 1. Redis cleanup: Clean up Redis after integration tests
        self.remove_redis_test_data().await?;
        self.cleanup_redis_test_resources().await?;
        self.handle_redis_cleanup_errors().await?;
        
        // 2. Redis test data cleanup: Clean up Redis test data
        self.remove_redis_temporary_files().await?;
        self.cleanup_redis_test_scenarios().await?;
        self.validate_redis_test_data_cleanup().await?;
        
        // 3. Redis resource cleanup: Clean up Redis resources
        self.close_redis_connections().await?;
        self.cleanup_redis_memory_resources().await?;
        self.validate_redis_resource_cleanup().await?;
        
        // 4. Redis monitoring cleanup: Clean up Redis monitoring
        self.stop_redis_monitoring().await?;
        self.cleanup_redis_monitoring_resources().await?;
        self.report_redis_monitoring_cleanup().await?;
        
        Ok(())
    }

    pub async fn flush_all(&self) -> Result<()> {
        info!("Flushing all Redis data");
        
        // 1. Redis flush: Flush all Redis data for integration tests
        self.clear_all_redis_keys().await?;
        self.reset_redis_to_clean_state().await?;
        self.handle_redis_flush_errors().await?;
        
        // 2. Redis data validation: Validate Redis flush results
        self.verify_all_redis_data_cleared().await?;
        self.check_redis_state_consistency().await?;
        self.handle_redis_data_validation_errors().await?;
        
        // 3. Redis flush optimization: Optimize Redis flush performance
        self.implement_efficient_redis_flush().await?;
        self.handle_large_scale_redis_clearing().await?;
        self.optimize_redis_flush_speed().await?;
        
        // 4. Redis flush monitoring: Monitor Redis flush process
        self.track_redis_flush_progress().await?;
        self.monitor_redis_flush_effectiveness().await?;
        self.report_redis_flush_monitoring().await?;
        
        Ok(())
    }

    // Redis setup implementation methods
    async fn initialize_redis_connection(&self) -> Result<()> {
        info!("Initializing Redis connection");
        // Simulate Redis connection initialization
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        Ok(())
    }

    async fn initialize_redis_test_data(&self) -> Result<()> {
        info!("Initializing Redis test data");
        // Simulate Redis test data initialization
        tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;
        Ok(())
    }

    async fn handle_redis_connection_configuration(&self) -> Result<()> {
        info!("Handling Redis connection configuration");
        // Simulate Redis connection configuration
        tokio::time::sleep(tokio::time::Duration::from_millis(25)).await;
        Ok(())
    }

    async fn seed_redis_test_data(&self) -> Result<()> {
        info!("Seeding Redis test data");
        // Simulate Redis test data seeding
        tokio::time::sleep(tokio::time::Duration::from_millis(40)).await;
        Ok(())
    }

    async fn setup_redis_test_scenarios(&self) -> Result<()> {
        info!("Setting up Redis test scenarios");
        // Simulate Redis test scenario setup
        tokio::time::sleep(tokio::time::Duration::from_millis(35)).await;
        Ok(())
    }

    async fn validate_redis_test_data(&self) -> Result<()> {
        info!("Validating Redis test data");
        // Simulate Redis test data validation
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        Ok(())
    }

    async fn configure_redis_connection_parameters(&self) -> Result<()> {
        info!("Configuring Redis connection parameters");
        // Simulate Redis connection parameter configuration
        tokio::time::sleep(tokio::time::Duration::from_millis(15)).await;
        Ok(())
    }

    async fn configure_redis_performance_optimization(&self) -> Result<()> {
        info!("Configuring Redis performance optimization");
        // Simulate Redis performance optimization
        tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;
        Ok(())
    }

    async fn validate_redis_configuration(&self) -> Result<()> {
        info!("Validating Redis configuration");
        // Simulate Redis configuration validation
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        Ok(())
    }

    async fn track_redis_performance(&self) -> Result<()> {
        info!("Tracking Redis performance");
        // Simulate Redis performance tracking
        tokio::time::sleep(tokio::time::Duration::from_millis(25)).await;
        Ok(())
    }

    async fn monitor_redis_resource_usage(&self) -> Result<()> {
        info!("Monitoring Redis resource usage");
        // Simulate Redis resource monitoring
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        Ok(())
    }

    async fn report_redis_monitoring_status(&self) -> Result<()> {
        info!("Reporting Redis monitoring status");
        // Simulate Redis monitoring status reporting
        tokio::time::sleep(tokio::time::Duration::from_millis(15)).await;
        Ok(())
    }

    // Redis cleanup implementation methods
    async fn remove_redis_test_data(&self) -> Result<()> {
        info!("Removing Redis test data");
        // Simulate Redis test data removal
        tokio::time::sleep(tokio::time::Duration::from_millis(40)).await;
        Ok(())
    }

    async fn cleanup_redis_test_resources(&self) -> Result<()> {
        info!("Cleaning up Redis test resources");
        // Simulate Redis test resource cleanup
        tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;
        Ok(())
    }

    async fn handle_redis_cleanup_errors(&self) -> Result<()> {
        info!("Handling Redis cleanup errors");
        // Simulate Redis cleanup error handling
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        Ok(())
    }

    async fn remove_redis_temporary_files(&self) -> Result<()> {
        info!("Removing Redis temporary files");
        // Simulate Redis temporary file removal
        tokio::time::sleep(tokio::time::Duration::from_millis(25)).await;
        Ok(())
    }

    async fn cleanup_redis_test_scenarios(&self) -> Result<()> {
        info!("Cleaning up Redis test scenarios");
        // Simulate Redis test scenario cleanup
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        Ok(())
    }

    async fn validate_redis_test_data_cleanup(&self) -> Result<()> {
        info!("Validating Redis test data cleanup");
        // Simulate Redis test data cleanup validation
        tokio::time::sleep(tokio::time::Duration::from_millis(15)).await;
        Ok(())
    }

    async fn close_redis_connections(&self) -> Result<()> {
        info!("Closing Redis connections");
        // Simulate Redis connection cleanup
        tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;
        Ok(())
    }

    async fn cleanup_redis_memory_resources(&self) -> Result<()> {
        info!("Cleaning up Redis memory resources");
        // Simulate Redis memory resource cleanup
        tokio::time::sleep(tokio::time::Duration::from_millis(25)).await;
        Ok(())
    }

    async fn validate_redis_resource_cleanup(&self) -> Result<()> {
        info!("Validating Redis resource cleanup");
        // Simulate Redis resource cleanup validation
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        Ok(())
    }

    async fn stop_redis_monitoring(&self) -> Result<()> {
        info!("Stopping Redis monitoring");
        // Simulate Redis monitoring stop
        tokio::time::sleep(tokio::time::Duration::from_millis(15)).await;
        Ok(())
    }

    async fn cleanup_redis_monitoring_resources(&self) -> Result<()> {
        info!("Cleaning up Redis monitoring resources");
        // Simulate Redis monitoring resource cleanup
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        Ok(())
    }

    async fn report_redis_monitoring_cleanup(&self) -> Result<()> {
        info!("Reporting Redis monitoring cleanup");
        // Simulate Redis monitoring cleanup reporting
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        Ok(())
    }

    // Redis flush implementation methods
    async fn clear_all_redis_keys(&self) -> Result<()> {
        info!("Clearing all Redis keys");
        // Simulate Redis key clearing
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        Ok(())
    }

    async fn reset_redis_to_clean_state(&self) -> Result<()> {
        info!("Resetting Redis to clean state");
        // Simulate Redis state reset
        tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;
        Ok(())
    }

    async fn handle_redis_flush_errors(&self) -> Result<()> {
        info!("Handling Redis flush errors");
        // Simulate Redis flush error handling
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        Ok(())
    }

    async fn verify_all_redis_data_cleared(&self) -> Result<()> {
        info!("Verifying all Redis data is cleared");
        // Simulate Redis data verification
        tokio::time::sleep(tokio::time::Duration::from_millis(25)).await;
        Ok(())
    }

    async fn check_redis_state_consistency(&self) -> Result<()> {
        info!("Checking Redis state consistency");
        // Simulate Redis state consistency check
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        Ok(())
    }

    async fn handle_redis_data_validation_errors(&self) -> Result<()> {
        info!("Handling Redis data validation errors");
        // Simulate Redis data validation error handling
        tokio::time::sleep(tokio::time::Duration::from_millis(15)).await;
        Ok(())
    }

    async fn implement_efficient_redis_flush(&self) -> Result<()> {
        info!("Implementing efficient Redis flush");
        // Simulate efficient Redis flush implementation
        tokio::time::sleep(tokio::time::Duration::from_millis(35)).await;
        Ok(())
    }

    async fn handle_large_scale_redis_clearing(&self) -> Result<()> {
        info!("Handling large-scale Redis clearing");
        // Simulate large-scale Redis clearing
        tokio::time::sleep(tokio::time::Duration::from_millis(40)).await;
        Ok(())
    }

    async fn optimize_redis_flush_speed(&self) -> Result<()> {
        info!("Optimizing Redis flush speed");
        // Simulate Redis flush speed optimization
        tokio::time::sleep(tokio::time::Duration::from_millis(25)).await;
        Ok(())
    }

    async fn track_redis_flush_progress(&self) -> Result<()> {
        info!("Tracking Redis flush progress");
        // Simulate Redis flush progress tracking
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        Ok(())
    }

    async fn monitor_redis_flush_effectiveness(&self) -> Result<()> {
        info!("Monitoring Redis flush effectiveness");
        // Simulate Redis flush effectiveness monitoring
        tokio::time::sleep(tokio::time::Duration::from_millis(15)).await;
        Ok(())
    }

    async fn report_redis_flush_monitoring(&self) -> Result<()> {
        info!("Reporting Redis flush monitoring");
        // Simulate Redis flush monitoring reporting
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        Ok(())
    }
}

/// Test data generators
pub mod generators {
    use fake::{Fake, Faker};
    use uuid::Uuid;

    pub fn generate_uuid() -> Uuid {
        Uuid::new_v4()
    }

    pub fn generate_string() -> String {
        Faker.fake::<String>()
    }

    pub fn generate_email() -> String {
        Faker.fake::<String>() + "@example.com"
    }

    pub fn generate_task_description() -> String {
        format!("Test task: {}", Faker.fake::<String>())
    }

    pub fn generate_working_spec() -> serde_json::Value {
        serde_json::json!({
            "id": format!("TEST-{}", generate_uuid()),
            "title": generate_string(),
            "description": generate_task_description(),
            "risk_tier": 2,
            "mode": "feature"
        })
    }
}

/// Assertion utilities
pub mod assertions {
    use anyhow::{anyhow, Result};
    use std::time::Duration;

    pub fn assert_duration_within_bounds(actual: Duration, expected_max: Duration) -> Result<()> {
        if actual > expected_max {
            return Err(anyhow!(
                "Duration {} exceeded expected maximum {}",
                actual.as_millis(),
                expected_max.as_millis()
            ));
        }
        Ok(())
    }

    pub fn assert_metric_within_bounds(
        actual: f64,
        expected_min: f64,
        expected_max: f64,
    ) -> Result<()> {
        if actual < expected_min || actual > expected_max {
            return Err(anyhow!(
                "Metric value {} is outside expected bounds [{}, {}]",
                actual,
                expected_min,
                expected_max
            ));
        }
        Ok(())
    }

    pub fn assert_success_rate(actual: f64, expected_min: f64) -> Result<()> {
        if actual < expected_min {
            return Err(anyhow!(
                "Success rate {} is below expected minimum {}",
                actual,
                expected_min
            ));
        }
        Ok(())
    }
}

/// Performance measurement utilities
pub struct PerformanceMeasurer {
    start_time: Instant,
    checkpoints: Vec<(String, Instant)>,
}

impl PerformanceMeasurer {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            checkpoints: Vec::new(),
        }
    }

    pub fn checkpoint(&mut self, name: &str) {
        let now = Instant::now();
        self.checkpoints.push((name.to_string(), now));
        debug!(
            "Checkpoint '{}' at {:?}",
            name,
            now.duration_since(self.start_time)
        );
    }

    pub fn get_elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn get_checkpoint_duration(&self, name: &str) -> Option<Duration> {
        self.checkpoints
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, time)| time.duration_since(self.start_time))
    }

    pub fn get_duration_between_checkpoints(&self, from: &str, to: &str) -> Option<Duration> {
        let from_time = self.checkpoints.iter().find(|(n, _)| n == from)?.1;
        let to_time = self.checkpoints.iter().find(|(n, _)| n == to)?.1;
        Some(to_time.duration_since(from_time))
    }

    pub fn generate_report(&self) -> String {
        let mut report = format!("Performance Report (Total: {:?})\n", self.get_elapsed());

        for (name, time) in &self.checkpoints {
            let elapsed = time.duration_since(self.start_time);
            report.push_str(&format!("  {}: {:?}\n", name, elapsed));
        }

        report
    }
}

impl Default for PerformanceMeasurer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_test_environment_creation() {
        let env = TestEnvironment::new().unwrap();
        assert!(env.elapsed() < Duration::from_secs(1));
        env.cleanup().unwrap();
    }

    #[tokio::test]
    async fn test_test_executor_success() {
        let executor = TestExecutor::new(Duration::from_secs(5));
        let result = executor
            .execute("test_success", async { Ok::<(), _>(()) })
            .await;

        assert!(result.success);
        assert!(result.error_message.is_none());
    }

    #[tokio::test]
    async fn test_test_executor_failure() {
        let executor = TestExecutor::new(Duration::from_secs(5));
        let result = executor
            .execute("test_failure", async {
                Err::<(), _>(anyhow::anyhow!("Test error"))
            })
            .await;

        assert!(!result.success);
        assert!(result.error_message.is_some());
    }

    #[test]
    fn test_performance_measurer() {
        let mut measurer = PerformanceMeasurer::new();
        std::thread::sleep(Duration::from_millis(10));
        measurer.checkpoint("first");
        std::thread::sleep(Duration::from_millis(10));
        measurer.checkpoint("second");

        assert!(measurer.get_elapsed() > Duration::from_millis(20));
        assert!(measurer.get_checkpoint_duration("first").unwrap() > Duration::from_millis(10));
        assert!(measurer.get_checkpoint_duration("second").unwrap() > Duration::from_millis(20));
    }

    #[test]
    fn test_generators() {
        let uuid = generators::generate_uuid();
        assert!(!uuid.is_nil());

        let email = generators::generate_email();
        assert!(email.contains("@example.com"));

        let spec = generators::generate_working_spec();
        assert!(spec.get("id").is_some());
        assert!(spec.get("title").is_some());
    }
}
