//! Core E2E test runner functionality

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::execution::TestExecutionEngine;
use super::reporting::TestReporter;
use super::monitoring::TestMonitor;
use super::environment::EnvironmentManager;

/// Test runner configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRunnerConfig {
    pub max_concurrent_tests: usize,
    pub test_timeout_seconds: u64,
    pub retry_attempts: u32,
    pub enable_monitoring: bool,
    pub enable_reporting: bool,
    pub environment_config: HashMap<String, serde_json::Value>,
    pub custom_settings: HashMap<String, String>,
}

impl Default for TestRunnerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tests: 5,
            test_timeout_seconds: 300,
            retry_attempts: 2,
            enable_monitoring: true,
            enable_reporting: true,
            environment_config: HashMap::new(),
            custom_settings: HashMap::new(),
        }
    }
}

/// Main E2E test runner
#[derive(Debug)]
pub struct TestRunner {
    config: TestRunnerConfig,
    execution_engine: Arc<TestExecutionEngine>,
    reporter: Arc<TestReporter>,
    monitor: Arc<TestMonitor>,
    environment_manager: Arc<EnvironmentManager>,
    test_suites: Arc<RwLock<HashMap<String, TestSuite>>>,
    running_tests: Arc<RwLock<HashMap<Uuid, RunningTest>>>,
}

/// Test suite definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuite {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tests: Vec<TestDefinition>,
    pub tags: Vec<String>,
    pub timeout_seconds: Option<u64>,
    pub required_environment: Vec<String>,
    pub dependencies: Vec<String>,
}

/// Individual test definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<TestStep>,
    pub assertions: Vec<TestAssertion>,
    pub tags: Vec<String>,
    pub timeout_seconds: Option<u64>,
    pub required_capabilities: Vec<String>,
}

/// Test execution step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestStep {
    pub id: String,
    pub name: String,
    pub action: TestAction,
    pub parameters: HashMap<String, serde_json::Value>,
    pub timeout_seconds: Option<u64>,
    pub retry_policy: Option<RetryPolicy>,
}

/// Test action types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestAction {
    HttpRequest(HttpRequestAction),
    DatabaseQuery(DatabaseQueryAction),
    FileOperation(FileOperationAction),
    Wait(WaitAction),
    Custom(String),
}

/// HTTP request action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequestAction {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<serde_json::Value>,
    pub expected_status: Option<u16>,
}

/// Database query action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseQueryAction {
    pub connection_string: String,
    pub query: String,
    pub parameters: Vec<serde_json::Value>,
    pub expected_rows: Option<usize>,
}

/// File operation action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperationAction {
    pub operation: FileOperation,
    pub path: String,
    pub content: Option<String>,
    pub expected_content: Option<String>,
}

/// File operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileOperation {
    Read,
    Write,
    Delete,
    Exists,
    Copy,
    Move,
}

/// Wait action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitAction {
    pub duration_seconds: u64,
    pub condition: Option<String>,
}

/// Retry policy for test steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub backoff_seconds: u64,
    pub exponential_backoff: bool,
}

/// Test assertion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestAssertion {
    pub id: String,
    pub name: String,
    pub assertion_type: AssertionType,
    pub target: String,
    pub expected_value: serde_json::Value,
    pub operator: AssertionOperator,
    pub message: Option<String>,
}

/// Assertion types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssertionType {
    HttpResponse,
    DatabaseResult,
    FileContent,
    Variable,
    Custom(String),
}

/// Assertion operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssertionOperator {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    GreaterThan,
    LessThan,
    RegexMatch,
    Exists,
    NotExists,
}

/// Currently running test
#[derive(Debug, Clone)]
pub struct RunningTest {
    pub id: Uuid,
    pub suite_id: String,
    pub test_id: String,
    pub start_time: DateTime<Utc>,
    pub status: TestStatus,
    pub current_step: Option<String>,
    pub results: Vec<TestStepResult>,
}

/// Test execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TestStatus {
    Pending,
    Running,
    Passed,
    Failed,
    Skipped,
    Error,
}

/// Test step result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestStepResult {
    pub step_id: String,
    pub status: TestStatus,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>,
    pub duration_ms: Option<u64>,
    pub output: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub screenshots: Vec<String>,
    pub logs: Vec<String>,
}

/// Test suite result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteResult {
    pub suite_id: String,
    pub status: TestStatus,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub skipped_tests: usize,
    pub test_results: Vec<TestResult>,
}

/// Individual test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_id: String,
    pub status: TestStatus,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub step_results: Vec<TestStepResult>,
    pub error_message: Option<String>,
    pub screenshots: Vec<String>,
    pub logs: Vec<String>,
}

impl TestRunner {
    /// Create a new test runner with configuration
    pub async fn new(config: TestRunnerConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let execution_engine = Arc::new(TestExecutionEngine::new(config.max_concurrent_tests).await?);
        let reporter = Arc::new(TestReporter::new().await?);
        let monitor = Arc::new(TestMonitor::new(config.enable_monitoring).await?);
        let environment_manager = Arc::new(EnvironmentManager::new(config.clone()).await?);

        Ok(Self {
            config,
            execution_engine,
            reporter,
            monitor,
            environment_manager,
            test_suites: Arc::new(RwLock::new(HashMap::new())),
            running_tests: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Load a test suite from file or definition
    pub async fn load_test_suite(&self, suite: TestSuite) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Validate test suite
        self.validate_test_suite(&suite).await?;

        // Store test suite
        let mut suites = self.test_suites.write().await;
        suites.insert(suite.id.clone(), suite);

        Ok(())
    }

    /// Run a specific test suite
    pub async fn run_test_suite(&self, suite_id: &str) -> Result<TestSuiteResult, Box<dyn std::error::Error + Send + Sync>> {
        let suite = {
            let suites = self.test_suites.read().await;
            suites.get(suite_id).cloned().ok_or_else(|| format!("Test suite {} not found", suite_id))?
        };

        // Prepare environment
        self.environment_manager.prepare_environment(&suite.required_environment).await?;

        // Initialize monitoring
        if self.config.enable_monitoring {
            self.monitor.start_monitoring_session(suite_id).await?;
        }

        // Execute tests
        let result = self.execution_engine.run_test_suite(suite).await?;

        // Generate reports
        if self.config.enable_reporting {
            self.reporter.generate_suite_report(&result).await?;
        }

        // Cleanup monitoring
        if self.config.enable_monitoring {
            self.monitor.end_monitoring_session(suite_id).await?;
        }

        Ok(result)
    }

    /// Get status of all running tests
    pub async fn get_running_tests_status(&self) -> HashMap<Uuid, TestStatus> {
        let running = self.running_tests.read().await;
        running.iter().map(|(id, test)| (*id, test.status.clone())).collect()
    }

    /// Stop a running test
    pub async fn stop_test(&self, test_id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut running = self.running_tests.write().await;
        if let Some(test) = running.get_mut(&test_id) {
            test.status = TestStatus::Error;
        }

        self.execution_engine.stop_test(test_id).await
    }

    /// Validate test suite configuration
    async fn validate_test_suite(&self, suite: &TestSuite) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Basic validation
        if suite.id.is_empty() {
            return Err("Test suite ID cannot be empty".into());
        }

        if suite.tests.is_empty() {
            return Err("Test suite must contain at least one test".into());
        }

        // Validate each test
        for test in &suite.tests {
            self.validate_test_definition(test).await?;
        }

        Ok(())
    }

    /// Validate individual test definition
    async fn validate_test_definition(&self, test: &TestDefinition) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if test.id.is_empty() {
            return Err("Test ID cannot be empty".into());
        }

        if test.steps.is_empty() {
            return Err("Test must contain at least one step".into());
        }

        // Validate step dependencies
        for step in &test.steps {
            if step.id.is_empty() {
                return Err("Step ID cannot be empty".into());
            }
        }

        Ok(())
    }
}