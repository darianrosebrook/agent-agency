//! E2E Test Runner
//!
//! Orchestrates execution of comprehensive E2E test scenarios with reporting
//! and result aggregation for the complete autonomous task execution system.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use chrono::{DateTime, Utc};
use sysinfo::{System, SystemExt, ProcessExt, CpuExt, DiskExt, NetworkExt};
use heim::cpu;
use heim::memory;
use heim::disk;
use heim::net;
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;

use super::harness::{E2eTestHarness, TestEnvironmentConfig};
use super::scenarios::{E2eTestScenarios, ScenarioRunner, ScenarioResult};
use super::assertions::E2eAssertions;

/// E2E test runner configuration
#[derive(Debug, Clone)]
pub struct TestRunnerConfig {
    pub environment_config: TestEnvironmentConfig,
    pub parallel_execution: bool,
    pub max_parallel_scenarios: usize,
    pub fail_fast: bool,
    pub generate_report: bool,
    pub report_path: PathBuf,
    pub include_performance_metrics: bool,
}

/// Test run result
#[derive(Debug, Clone)]
pub struct TestRunResult {
    pub total_scenarios: usize,
    pub passed_scenarios: usize,
    pub failed_scenarios: usize,
    pub skipped_scenarios: usize,
    pub total_duration: Duration,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub scenario_results: Vec<ScenarioResult>,
    pub performance_metrics: Option<PerformanceMetrics>,
}

/// Performance metrics for the test run
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub average_scenario_duration: Duration,
    pub min_scenario_duration: Duration,
    pub max_scenario_duration: Duration,
    pub total_tasks_executed: usize,
    pub average_tasks_per_scenario: f64,
    pub system_resource_usage: HashMap<String, f64>,
}

/// E2E test runner
pub struct E2eTestRunner {
    config: TestRunnerConfig,
    harness: Arc<Mutex<Option<E2eTestHarness>>>,
}

impl E2eTestRunner {
    pub fn new(config: TestRunnerConfig) -> Self {
        Self {
            config,
            harness: Arc::new(Mutex::new(None)),
        }
    }

    /// Run all E2E test scenarios
    pub async fn run_all_scenarios(&self) -> Result<TestRunResult, TestRunnerError> {
        self.run_scenarios(None).await
    }

    /// Run specific test scenarios by name
    pub async fn run_scenarios(&self, scenario_names: Option<Vec<String>>) -> Result<TestRunResult, TestRunnerError> {
        let start_time = Utc::now();
        let mut scenario_results = Vec::new();
        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;

        tracing::info!("Starting E2E test run...");

        // Initialize test harness
        self.initialize_harness().await?;

        // Get scenarios to run
        let all_scenarios = E2eTestScenarios::all_scenarios();
        let scenarios_to_run: Vec<_> = if let Some(names) = scenario_names {
            all_scenarios.into_iter()
                .filter(|s| names.contains(&s.name))
                .collect()
        } else {
            all_scenarios
        };

        if scenarios_to_run.is_empty() {
            return Err(TestRunnerError::NoScenariosFound);
        }

        tracing::info!("Running {} E2E scenarios...", scenarios_to_run.len());

        // Run scenarios
        if self.config.parallel_execution {
            scenario_results = self.run_scenarios_parallel(scenarios_to_run).await?;
        } else {
            for scenario in scenarios_to_run {
                let result = self.run_single_scenario(scenario).await?;
                scenario_results.push(result);
            }
        }

        // Count results
        for result in &scenario_results {
            if result.passed {
                passed += 1;
            } else {
                failed += 1;
            }
        }

        let total_duration = (Utc::now() - start_time).to_std()
            .map_err(|_| TestRunnerError::TimeConversionError)?;

        // Collect performance metrics if enabled
        let performance_metrics = if self.config.include_performance_metrics {
            Some(self.collect_performance_metrics(&scenario_results).await?)
        } else {
            None
        };

        // Generate report if requested
        if self.config.generate_report {
            self.generate_test_report(&scenario_results, &performance_metrics).await?;
        }

        // Cleanup
        self.cleanup().await?;

        let result = TestRunResult {
            total_scenarios: scenario_results.len(),
            passed_scenarios: passed,
            failed_scenarios: failed,
            skipped_scenarios: skipped,
            total_duration,
            start_time,
            end_time: Utc::now(),
            scenario_results,
            performance_metrics,
        };

        self.log_final_results(&result);

        Ok(result)
    }

    /// Run scenarios in parallel
    async fn run_scenarios_parallel(&self, scenarios: Vec<super::scenarios::TestScenario>) -> Result<Vec<ScenarioResult>, TestRunnerError> {
        use tokio::sync::Semaphore;
        use std::sync::Arc;

        let semaphore = Arc::new(Semaphore::new(self.config.max_parallel_scenarios));
        let mut tasks = Vec::new();

        for scenario in scenarios {
            let semaphore = Arc::clone(&semaphore);
            let harness = Arc::clone(&self.harness);

            let task = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                let harness_guard = harness.lock().await;
                if let Some(harness) = harness_guard.as_ref() {
                    ScenarioRunner::run_scenario(&scenario, harness).await
                } else {
                    Err(super::scenarios::ScenarioError::TaskSubmissionError("Harness not initialized".to_string()))
                }
            });

            tasks.push(task);
        }

        // Collect results
        let mut results = Vec::new();
        for task in tasks {
            match task.await {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(e)) => {
                    tracing::error!("Scenario execution error: {:?}", e);
                    // Create failed result
                    results.push(ScenarioResult {
                        scenario_name: "unknown".to_string(),
                        passed: false,
                        total_duration: Duration::from_secs(0),
                        task_results: vec![],
                        assertion_results: vec![],
                        failed_assertions: vec![format!("Execution error: {:?}", e)],
                    });
                }
                Err(e) => {
                    tracing::error!("Task join error: {:?}", e);
                    results.push(ScenarioResult {
                        scenario_name: "unknown".to_string(),
                        passed: false,
                        total_duration: Duration::from_secs(0),
                        task_results: vec![],
                        assertion_results: vec![],
                        failed_assertions: vec![format!("Task panic: {:?}", e)],
                    });
                }
            }
        }

        Ok(results)
    }

    /// Run a single scenario
    async fn run_single_scenario(&self, scenario: super::scenarios::TestScenario) -> Result<ScenarioResult, TestRunnerError> {
        let harness_guard = self.harness.lock().await;
        if let Some(harness) = harness_guard.as_ref() {
            let result = ScenarioRunner::run_scenario(&scenario, harness).await
                .map_err(|e| TestRunnerError::ScenarioExecutionError(format!("{:?}", e)))?;

            // Check fail-fast
            if !result.passed && self.config.fail_fast {
                return Err(TestRunnerError::FailFast(result.scenario_name));
            }

            Ok(result)
        } else {
            Err(TestRunnerError::HarnessNotInitialized)
        }
    }

    /// Initialize the test harness
    async fn initialize_harness(&self) -> Result<(), TestRunnerError> {
        let mut harness_guard = self.harness.lock().await;

        if harness_guard.is_some() {
            return Ok(()); // Already initialized
        }

        let mut harness = E2eTestHarness::new(self.config.environment_config.clone());
        harness.initialize().await
            .map_err(|e| TestRunnerError::HarnessInitializationError(format!("{:?}", e)))?;

        *harness_guard = Some(harness);
        Ok(())
    }

    /// Collect performance metrics
    async fn collect_performance_metrics(&self, results: &[ScenarioResult]) -> Result<PerformanceMetrics, TestRunnerError> {
        if results.is_empty() {
            return Ok(PerformanceMetrics {
                average_scenario_duration: Duration::from_secs(0),
                min_scenario_duration: Duration::from_secs(0),
                max_scenario_duration: Duration::from_secs(0),
                total_tasks_executed: 0,
                average_tasks_per_scenario: 0.0,
                system_resource_usage: HashMap::new(),
            });
        }

        let durations: Vec<Duration> = results.iter().map(|r| r.total_duration).collect();
        let total_duration: Duration = durations.iter().sum();
        let average_duration = total_duration / results.len() as u32;

        let min_duration = durations.iter().min().copied().unwrap_or(Duration::from_secs(0));
        let max_duration = durations.iter().max().copied().unwrap_or(Duration::from_secs(0));

        let total_tasks: usize = results.iter().map(|r| r.task_results.len()).sum();
        let average_tasks = total_tasks as f64 / results.len() as f64;

        // TODO: Implement comprehensive system resource monitoring
        // - Add detailed CPU usage tracking per process and core
        // - Implement memory usage analysis with heap/stack breakdown
        // - Support disk I/O monitoring and bottleneck detection
        // - Add network usage tracking and bandwidth analysis
        // - Implement GPU memory and utilization monitoring
        // - Support resource usage profiling and flame graphs
        // PLACEHOLDER: Using simplified resource collection
        let mut resource_usage = HashMap::new();

        if let Some(harness) = self.harness.lock().await.as_ref() {
            if let Ok(metrics) = harness.get_metrics().await {
                if let Some(memory) = super::assertions::E2eAssertions::extract_metric_value(&metrics, "memory_usage_mb") {
                    resource_usage.insert("memory_mb".to_string(), memory);
                }
                if let Some(cpu) = super::assertions::E2eAssertions::extract_metric_value(&metrics, "cpu_usage_percent") {
                    resource_usage.insert("cpu_percent".to_string(), cpu);
                }
            }
        }

        Ok(PerformanceMetrics {
            average_scenario_duration: average_duration,
            min_scenario_duration,
            max_scenario_duration,
            total_tasks_executed: total_tasks,
            average_tasks_per_scenario: average_tasks,
            system_resource_usage: resource_usage,
        })
    }

    /// Generate test report
    async fn generate_test_report(
        &self,
        results: &[ScenarioResult],
        performance: &Option<PerformanceMetrics>,
    ) -> Result<(), TestRunnerError> {
        use tokio::fs;
        use std::io::Write;

        let mut report = String::new();
        report.push_str("# E2E Test Report\n\n");
        report.push_str(&format!("Generated: {}\n\n", Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));

        // Summary
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = results.iter().filter(|r| !r.passed).count();
        let total_duration: Duration = results.iter().map(|r| r.total_duration).sum();

        report.push_str("## Summary\n\n");
        report.push_str(&format!("- Total Scenarios: {}\n", results.len()));
        report.push_str(&format!("- Passed: {} ({:.1}%)\n", passed, (passed as f64 / results.len() as f64) * 100.0));
        report.push_str(&format!("- Failed: {}\n", failed));
        report.push_str(&format!("- Total Duration: {:.2}s\n\n", total_duration.as_secs_f64()));

        // Performance metrics
        if let Some(perf) = performance {
            report.push_str("## Performance Metrics\n\n");
            report.push_str(&format!("- Average Scenario Duration: {:.2}s\n", perf.average_scenario_duration.as_secs_f64()));
            report.push_str(&format!("- Min Scenario Duration: {:.2}s\n", perf.min_scenario_duration.as_secs_f64()));
            report.push_str(&format!("- Max Scenario Duration: {:.2}s\n", perf.max_scenario_duration.as_secs_f64()));
            report.push_str(&format!("- Total Tasks Executed: {}\n", perf.total_tasks_executed));
            report.push_str(&format!("- Average Tasks per Scenario: {:.1}\n\n", perf.average_tasks_per_scenario));

            if !perf.system_resource_usage.is_empty() {
                report.push_str("### Resource Usage\n\n");
                for (resource, usage) in &perf.system_resource_usage {
                    report.push_str(&format!("- {}: {:.1}\n", resource, usage));
                }
                report.push_str("\n");
            }
        }

        // Detailed results
        report.push_str("## Scenario Results\n\n");

        for result in results {
            report.push_str(&format!("### {}: {}\n\n", result.scenario_name,
                if result.passed { "✅ PASSED" } else { "❌ FAILED" }));

            report.push_str(&format!("- Duration: {:.2}s\n", result.total_duration.as_secs_f64()));
            report.push_str(&format!("- Tasks Executed: {}\n", result.task_results.len()));

            if !result.failed_assertions.is_empty() {
                report.push_str("- Failed Assertions:\n");
                for assertion in &result.failed_assertions {
                    report.push_str(&format!("  - {}\n", assertion));
                }
            }

            report.push_str("\n");
        }

        // Write report to file
        fs::write(&self.config.report_path, report).await
            .map_err(|e| TestRunnerError::ReportGenerationError(format!("{:?}", e)))?;

        tracing::info!("Test report generated: {}", self.config.report_path.display());
        Ok(())
    }

    /// Cleanup test environment
    async fn cleanup(&self) -> Result<(), TestRunnerError> {
        let mut harness_guard = self.harness.lock().await;
        if let Some(harness) = harness_guard.take() {
            harness.cleanup().await
                .map_err(|e| TestRunnerError::CleanupError(format!("{:?}", e)))?;
        }
        Ok(())
    }

    /// Log final test results
    fn log_final_results(&self, result: &TestRunResult) {
        let success_rate = (result.passed_scenarios as f64 / result.total_scenarios as f64) * 100.0;

        tracing::info!("E2E Test Run Complete");
        tracing::info!("=====================");
        tracing::info!("Scenarios: {} total, {} passed, {} failed",
            result.total_scenarios, result.passed_scenarios, result.failed_scenarios);
        tracing::info!("Success Rate: {:.1}%", success_rate);
        tracing::info!("Total Duration: {:.2}s", result.total_duration.as_secs_f64());

        if let Some(perf) = &result.performance_metrics {
            tracing::info!("Avg Scenario Duration: {:.2}s", perf.average_scenario_duration.as_secs_f64());
            tracing::info!("Tasks Executed: {}", perf.total_tasks_executed);
        }

        if result.failed_scenarios > 0 {
            tracing::warn!("❌ {} scenarios failed", result.failed_scenarios);
        } else {
            tracing::info!("✅ All scenarios passed!");
        }
    }
}

/// Create default test runner configuration
pub fn default_test_runner_config() -> TestRunnerConfig {
    TestRunnerConfig {
        environment_config: TestEnvironmentConfig {
            cleanup_after_test: true,
            test_timeout_seconds: 1800, // 30 minutes
            enable_detailed_logging: true,
            database_url: "postgres://test:test@localhost/test_db".to_string(),
            working_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/tmp")),
        },
        parallel_execution: true,
        max_parallel_scenarios: 3,
        fail_fast: false,
        generate_report: true,
        report_path: PathBuf::from("e2e-test-report.md"),
        include_performance_metrics: true,
    }
}

pub type Result<T> = std::result::Result<T, TestRunnerError>;

#[derive(Debug, thiserror::Error)]
pub enum TestRunnerError {
    #[error("Harness not initialized")]
    HarnessNotInitialized,

    #[error("Harness initialization failed: {0}")]
    HarnessInitializationError(String),

    #[error("No scenarios found to run")]
    NoScenariosFound,

    #[error("Scenario execution failed: {0}")]
    ScenarioExecutionError(String),

    #[error("Fail-fast triggered for scenario: {0}")]
    FailFast(String),

    #[error("Report generation failed: {0}")]
    ReportGenerationError(String),

    #[error("Cleanup failed: {0}")]
    CleanupError(String),

    #[error("Time conversion error")]
    TimeConversionError,

    #[error("Configuration error: {0}")]
    ConfigError(String),
}
