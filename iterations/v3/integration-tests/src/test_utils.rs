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
                warn!("⏰ Test timed out: {} (after {:?})", test_name, self.timeout);
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
        // TODO: Implement database setup
        Ok(())
    }

    pub async fn cleanup_test_database(&self) -> Result<()> {
        info!("Cleaning up test database");
        // TODO: Implement database cleanup
        Ok(())
    }

    pub async fn reset_database(&self) -> Result<()> {
        info!("Resetting test database");
        self.cleanup_test_database().await?;
        self.setup_test_database().await?;
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
        // TODO: Implement Redis setup
        Ok(())
    }

    pub async fn cleanup_test_redis(&self) -> Result<()> {
        info!("Cleaning up test Redis");
        // TODO: Implement Redis cleanup
        Ok(())
    }

    pub async fn flush_all(&self) -> Result<()> {
        info!("Flushing all Redis data");
        // TODO: Implement Redis flush
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

    pub fn assert_metric_within_bounds(actual: f64, expected_min: f64, expected_max: f64) -> Result<()> {
        if actual < expected_min || actual > expected_max {
            return Err(anyhow!(
                "Metric value {} is outside expected bounds [{}, {}]",
                actual, expected_min, expected_max
            ));
        }
        Ok(())
    }

    pub fn assert_success_rate(actual: f64, expected_min: f64) -> Result<()> {
        if actual < expected_min {
            return Err(anyhow!(
                "Success rate {} is below expected minimum {}",
                actual, expected_min
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
        debug!("Checkpoint '{}' at {:?}", name, now.duration_since(self.start_time));
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
        let result = executor.execute("test_success", async { Ok(()) }).await;
        
        assert!(result.success);
        assert!(result.error_message.is_none());
    }

    #[tokio::test]
    async fn test_test_executor_failure() {
        let executor = TestExecutor::new(Duration::from_secs(5));
        let result = executor.execute("test_failure", async { 
            Err(anyhow::anyhow!("Test error")) 
        }).await;
        
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
