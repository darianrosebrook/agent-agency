//! Validation pipeline implementation
//!
//! This module provides a validation pipeline that can run multiple validation
//! stages and collect/aggregate validation results according to configurable policies.

use crate::{
    config::ValidationPipelineConfig,
    error::{PipelineError, PipelineResult},
    metrics::PipelineMetrics,
    traits::ExecutablePipeline,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Validation result from a single validation stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether validation passed
    pub passed: bool,
    /// Validation severity level
    pub severity: ValidationSeverity,
    /// Validation category
    pub category: String,
    /// Human-readable message
    pub message: String,
    /// Optional suggestion for fixing the issue
    pub suggestion: Option<String>,
    /// Additional context data
    pub context: serde_json::Value,
}

impl ValidationResult {
    /// Create a passing validation result
    pub fn pass(category: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            passed: true,
            severity: ValidationSeverity::Info,
            category: category.into(),
            message: message.into(),
            suggestion: None,
            context: serde_json::Value::Null,
        }
    }

    /// Create a failing validation result
    pub fn fail(
        severity: ValidationSeverity,
        category: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            passed: false,
            severity,
            category: category.into(),
            message: message.into(),
            suggestion: None,
            context: serde_json::Value::Null,
        }
    }

    /// Add a suggestion to the result
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Add context data to the result
    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.context = context;
        self
    }
}

/// Validation severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidationSeverity {
    /// Informational messages
    Info,
    /// Warning messages
    Warning,
    /// Error messages
    Error,
    /// Critical errors
    Critical,
}

/// Comprehensive validation results from all stages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResults {
    /// Overall validation status
    pub overall_passed: bool,
    /// All validation results
    pub results: Vec<ValidationResult>,
    /// Summary statistics
    pub summary: ValidationSummary,
    /// Processing metadata
    pub metadata: ValidationMetadata,
}

impl ValidationResults {
    /// Create new validation results
    pub fn new() -> Self {
        Self {
            overall_passed: true,
            results: Vec::new(),
            summary: ValidationSummary::default(),
            metadata: ValidationMetadata::default(),
        }
    }

    /// Add a validation result
    pub fn add_result(&mut self, result: ValidationResult) {
        self.results.push(result);
        self.update_summary();
    }

    /// Check if validation should stop based on configuration
    pub fn should_stop(&self, config: &ValidationPipelineConfig) -> bool {
        if config.stop_on_first_error {
            // Stop on any error or critical issue
            self.results.iter().any(|r| {
                !r.passed
                    && (r.severity >= ValidationSeverity::Error
                        || r.severity >= config.severity_threshold)
            })
        } else {
            false
        }
    }

    /// Update summary statistics
    fn update_summary(&mut self) {
        let mut summary = ValidationSummary::default();

        for result in &self.results {
            if result.passed {
                summary.passed += 1;
            } else {
                summary.failed += 1;

                match result.severity {
                    ValidationSeverity::Info => summary.info_count += 1,
                    ValidationSeverity::Warning => summary.warning_count += 1,
                    ValidationSeverity::Error => summary.error_count += 1,
                    ValidationSeverity::Critical => summary.critical_count += 1,
                }
            }
        }

        summary.total = self.results.len();
        self.overall_passed = summary.failed == 0 && summary.critical_count == 0;
        self.summary = summary;
    }
}

/// Validation summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    /// Total validations run
    pub total: usize,
    /// Number of validations that passed
    pub passed: usize,
    /// Number of validations that failed
    pub failed: usize,
    /// Number of info-level issues
    pub info_count: usize,
    /// Number of warning-level issues
    pub warning_count: usize,
    /// Number of error-level issues
    pub error_count: usize,
    /// Number of critical-level issues
    pub critical_count: usize,
}

impl Default for ValidationSummary {
    fn default() -> Self {
        Self {
            total: 0,
            passed: 0,
            failed: 0,
            info_count: 0,
            warning_count: 0,
            error_count: 0,
            critical_count: 0,
        }
    }
}

/// Validation processing metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationMetadata {
    /// Start timestamp
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// Completion timestamp
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Total processing time (ms)
    pub processing_time_ms: Option<u64>,
    /// Validation stages that were executed
    pub stages_executed: Vec<String>,
}

impl Default for ValidationMetadata {
    fn default() -> Self {
        Self {
            started_at: chrono::Utc::now(),
            completed_at: None,
            processing_time_ms: None,
            stages_executed: Vec::new(),
        }
    }
}

/// Validation stage trait
#[async_trait]
pub trait ValidationStage: Send + Sync + std::fmt::Debug {
    /// Get the name of this validation stage
    fn name(&self) -> &str;

    /// Validate the given input
    async fn validate(&self, input: &serde_json::Value) -> PipelineResult<Vec<ValidationResult>>;

    /// Get stage priority (higher = run first)
    fn priority(&self) -> i32 {
        0
    }
}

/// Validation pipeline for running multiple validation stages
#[derive(Debug)]
pub struct ValidationPipeline {
    config: ValidationPipelineConfig,
    stages: Vec<Box<dyn ValidationStage>>,
    metrics: PipelineMetrics,
}

impl ValidationPipeline {
    /// Create a new validation pipeline
    pub fn new(config: ValidationPipelineConfig) -> Self {
        Self {
            config,
            stages: Vec::new(),
            metrics: PipelineMetrics::new(),
        }
    }

    /// Add a validation stage
    pub fn add_stage(&mut self, stage: Box<dyn ValidationStage>) {
        self.stages.push(stage);
        // Sort by priority (highest first)
        self.stages.sort_by(|a, b| b.priority().cmp(&a.priority()));
    }

    /// Get stage count
    pub fn stage_count(&self) -> usize {
        self.stages.len()
    }

    /// Get stage names
    pub fn stage_names(&self) -> Vec<String> {
        self.stages.iter().map(|s| s.name().to_string()).collect()
    }

    /// Run validation on the given input
    async fn run_validation(&self, input: &serde_json::Value) -> PipelineResult<ValidationResults> {
        let mut results = ValidationResults::new();
        let start_time = std::time::Instant::now();

        results.metadata.stages_executed = self.stage_names();

        for stage in &self.stages {
            let stage_name = stage.name();
            let stage_start = std::time::Instant::now();

            debug!("Running validation stage: {}", stage_name);

            match tokio::time::timeout(self.config.max_validation_time, stage.validate(input)).await
            {
                Ok(Ok(stage_results)) => {
                    let stage_duration = stage_start.elapsed().as_millis() as u64;
                    self.metrics
                        .record_stage_execution(stage_name, stage_duration, true)
                        .await;

                    for result in stage_results {
                        results.add_result(result);

                        // Check if we should stop
                        if results.should_stop(&self.config) {
                            debug!("Stopping validation due to failure policy");
                            return Ok(results);
                        }
                    }
                }
                Ok(Err(e)) => {
                    let stage_duration = stage_start.elapsed().as_millis() as u64;
                    self.metrics
                        .record_stage_execution(stage_name, stage_duration, false)
                        .await;
                    self.metrics
                        .record_error(&format!("stage_{}", stage_name))
                        .await;

                    warn!("Validation stage {} failed: {}", stage_name, e);

                    // Add error result
                    let error_result = ValidationResult::fail(
                        ValidationSeverity::Error,
                        format!("stage_{}", stage_name),
                        format!("Stage failed: {}", e),
                    );
                    results.add_result(error_result);

                    // Check if we should stop on errors
                    if results.should_stop(&self.config) {
                        break;
                    }
                }
                Err(_) => {
                    self.metrics
                        .record_stage_execution(
                            stage_name,
                            self.config.max_validation_time.as_millis() as u64,
                            false,
                        )
                        .await;
                    self.metrics.record_error("stage_timeout").await;

                    warn!("Validation stage {} timed out", stage_name);

                    let timeout_result = ValidationResult::fail(
                        ValidationSeverity::Error,
                        format!("stage_{}", stage_name),
                        "Stage timed out",
                    );
                    results.add_result(timeout_result);

                    if results.should_stop(&self.config) {
                        break;
                    }
                }
            }
        }

        let total_duration = start_time.elapsed().as_millis() as u64;
        results.metadata.completed_at = Some(chrono::Utc::now());
        results.metadata.processing_time_ms = Some(total_duration);

        Ok(results)
    }
}

#[async_trait]
impl ExecutablePipeline<serde_json::Value, ValidationResults> for ValidationPipeline {
    async fn execute(&self, input: serde_json::Value) -> PipelineResult<ValidationResults> {
        let start_time = std::time::Instant::now();

        info!(
            "Starting validation pipeline with {} stages",
            self.stages.len()
        );

        let result = self.run_validation(&input).await;
        let duration = start_time.elapsed().as_millis() as u64;
        let success = result.is_ok();

        self.metrics.record_execution(duration, success).await;

        match &result {
            Ok(results) => {
                info!(
                    "Validation pipeline completed in {}ms: {} passed, {} failed",
                    duration, results.summary.passed, results.summary.failed
                );
            }
            Err(e) => {
                self.metrics.record_error("pipeline_execution").await;
                warn!("Validation pipeline failed after {}ms: {}", duration, e);
            }
        }

        result
    }

    fn metrics(&self) -> PipelineResult<serde_json::Value> {
        futures::executor::block_on(async { self.metrics.to_json().await })
            .map_err(|e| PipelineError::Metrics(e.to_string()))
    }

    fn health_status(&self) -> PipelineResult<crate::PipelineHealth> {
        if self.stages.is_empty() {
            return Ok(crate::PipelineHealth::Unhealthy);
        }

        // Validation pipelines are considered healthy if they have stages
        Ok(crate::PipelineHealth::Healthy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    // Mock validation stage
    #[derive(Debug)]
    struct MockValidationStage {
        name: String,
        results: Vec<ValidationResult>,
        should_fail: bool,
    }

    impl MockValidationStage {
        fn new(name: impl Into<String>, results: Vec<ValidationResult>) -> Self {
            Self {
                name: name.into(),
                results,
                should_fail: false,
            }
        }

        #[allow(dead_code)]
        fn failing(name: impl Into<String>) -> Self {
            Self {
                name: name.into(),
                results: vec![ValidationResult::fail(
                    ValidationSeverity::Error,
                    "test",
                    "Mock failure",
                )],
                should_fail: true,
            }
        }
    }

    #[async_trait]
    impl ValidationStage for MockValidationStage {
        fn name(&self) -> &str {
            &self.name
        }

        async fn validate(
            &self,
            _input: &serde_json::Value,
        ) -> PipelineResult<Vec<ValidationResult>> {
            if self.should_fail {
                return Err(PipelineError::Execution("Mock stage failure".to_string()));
            }
            Ok(self.results.clone())
        }
    }

    #[tokio::test]
    async fn test_validation_pipeline_success() {
        let config = ValidationPipelineConfig::default();
        let mut pipeline = ValidationPipeline::new(config);

        let stage1 = Box::new(MockValidationStage::new(
            "stage1",
            vec![
                ValidationResult::pass("test1", "Test 1 passed"),
                ValidationResult::pass("test2", "Test 2 passed"),
            ],
        ));

        let stage2 = Box::new(MockValidationStage::new(
            "stage2",
            vec![ValidationResult::pass("test3", "Test 3 passed")],
        ));

        pipeline.add_stage(stage1);
        pipeline.add_stage(stage2);

        let input = serde_json::json!({"test": "data"});
        let result = pipeline.execute(input).await;

        assert!(result.is_ok());
        let results = result.unwrap();

        assert!(results.overall_passed);
        assert_eq!(results.summary.total, 3);
        assert_eq!(results.summary.passed, 3);
        assert_eq!(results.summary.failed, 0);
    }

    #[tokio::test]
    async fn test_validation_pipeline_with_failures() {
        let config = ValidationPipelineConfig {
            collect_all_errors: true,
            ..Default::default()
        };
        let mut pipeline = ValidationPipeline::new(config);

        let stage1 = Box::new(MockValidationStage::new(
            "stage1",
            vec![
                ValidationResult::pass("test1", "Test 1 passed"),
                ValidationResult::fail(ValidationSeverity::Warning, "test2", "Test 2 failed"),
            ],
        ));

        let stage2 = Box::new(MockValidationStage::new(
            "stage2",
            vec![ValidationResult::fail(
                ValidationSeverity::Error,
                "test3",
                "Test 3 failed",
            )],
        ));

        pipeline.add_stage(stage1);
        pipeline.add_stage(stage2);

        let input = serde_json::json!({"test": "data"});
        let result = pipeline.execute(input).await;

        assert!(result.is_ok());
        let results = result.unwrap();

        assert!(!results.overall_passed); // Should fail due to errors
        assert_eq!(results.summary.total, 3);
        assert_eq!(results.summary.passed, 1);
        assert_eq!(results.summary.failed, 2);
        assert_eq!(results.summary.warning_count, 1);
        assert_eq!(results.summary.error_count, 1);
    }

    #[tokio::test]
    async fn test_validation_pipeline_stop_on_error() {
        let config = ValidationPipelineConfig {
            stop_on_first_error: true,
            ..Default::default()
        };
        let mut pipeline = ValidationPipeline::new(config);

        let stage1 = Box::new(MockValidationStage::new(
            "stage1",
            vec![ValidationResult::fail(
                ValidationSeverity::Error,
                "test1",
                "Test 1 failed",
            )],
        ));

        let stage2 = Box::new(MockValidationStage::new(
            "stage2",
            vec![
                ValidationResult::pass("test2", "Test 2 passed"), // This should not run
            ],
        ));

        pipeline.add_stage(stage1);
        pipeline.add_stage(stage2);

        let input = serde_json::json!({"test": "data"});
        let result = pipeline.execute(input).await;

        assert!(result.is_ok());
        let results = result.unwrap();

        // Should only have results from first stage
        assert_eq!(results.summary.total, 1);
        assert_eq!(results.summary.failed, 1);
    }
}
