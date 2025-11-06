//! Common infrastructure for specialized workers
//!
//! This module provides shared types and utilities for implementing
//! specialized workers with consistent patterns and behaviors.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, instrument};
use uuid::Uuid;

use crate::worker_errors::WorkerError;

/// Common execution context for workers
#[derive(Debug, Clone)]
pub struct WorkerContext {
    pub worker_id: Uuid,
    pub task_id: Uuid,
    pub execution_timeout: std::time::Duration,
    pub quality_requirements: QualityRequirements,
    pub trace_context: HashMap<String, String>,
}

impl Default for WorkerContext {
    fn default() -> Self {
        Self {
            worker_id: Uuid::new_v4(),
            task_id: Uuid::new_v4(),
            execution_timeout: std::time::Duration::from_secs(300), // 5 minutes
            quality_requirements: QualityRequirements::default(),
            trace_context: HashMap::new(),
        }
    }
}

/// Quality requirements for worker execution
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualityRequirements {
    pub min_coverage: Option<f64>,
    pub max_complexity: Option<f64>,
    pub required_tests: bool,
    pub documentation_required: bool,
}

impl Default for QualityRequirements {
    fn default() -> Self {
        Self {
            min_coverage: Some(0.8),
            max_complexity: Some(10.0),
            required_tests: true,
            documentation_required: false,
        }
    }
}

/// Common trait for specialized workers with default implementations
#[async_trait]
pub trait Worker: Send + Sync {
    /// The worker's unique identifier
    fn id(&self) -> &'static str;

    /// The worker's capabilities
    fn capabilities(&self) -> Vec<String>;

    /// Parse task parameters from string input
    fn parse_task(&self, task: &str) -> Result<serde_json::Value, WorkerError>;

    /// Execute the task with parsed parameters
    async fn execute_task(
        &self,
        params: &serde_json::Value,
        context: &WorkerContext,
    ) -> Result<WorkerResult, WorkerError>;

    /// Execute a task from string input (default implementation)
    #[instrument(skip(self, task, context), fields(worker = %self.id()))]
    async fn execute(
        &self,
        task: String,
        context: Option<WorkerContext>,
    ) -> Result<String, WorkerError> {
        let context = context.unwrap_or_default();

        info!("Starting execution of task for worker {}", self.id());

        // Parse the task
        let params = self.parse_task(&task).map_err(|e| {
            error!("Failed to parse task: {}", e);
            e
        })?;

        // Execute with parsed parameters
        let result = self.execute_task(&params, &context).await.map_err(|e| {
            error!("Task execution failed: {}", e);
            e
        })?;

        // Return the result
        Ok(result.output)
    }
}

/// Result of worker execution
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkerResult {
    pub output: String,
    pub success: bool,
    pub execution_time_ms: u64,
    pub quality_score: Option<f32>,
    pub artifacts: Vec<WorkerArtifact>,
    pub metrics: WorkerExecutionMetrics,
}

/// Artifact produced by worker execution
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkerArtifact {
    pub name: String,
    pub content_type: String,
    pub data: Vec<u8>,
    pub metadata: HashMap<String, String>,
}

/// Metrics from worker execution
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkerExecutionMetrics {
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: f64,
    pub io_operations: u64,
    pub network_bytes: u64,
}

impl Default for WorkerExecutionMetrics {
    fn default() -> Self {
        Self {
            cpu_usage_percent: 0.0,
            memory_usage_mb: 0.0,
            io_operations: 0,
            network_bytes: 0,
        }
    }
}

/// Helper for creating worker results
pub struct WorkerResultBuilder {
    result: WorkerResult,
}

impl WorkerResultBuilder {
    pub fn new(output: String) -> Self {
        Self {
            result: WorkerResult {
                output,
                success: true,
                execution_time_ms: 0,
                quality_score: None,
                artifacts: Vec::new(),
                metrics: WorkerExecutionMetrics::default(),
            },
        }
    }

    pub fn success(mut self, success: bool) -> Self {
        self.result.success = success;
        self
    }

    pub fn execution_time_ms(mut self, time: u64) -> Self {
        self.result.execution_time_ms = time;
        self
    }

    pub fn quality_score(mut self, score: f32) -> Self {
        self.result.quality_score = Some(score);
        self
    }

    pub fn add_artifact(mut self, artifact: WorkerArtifact) -> Self {
        self.result.artifacts.push(artifact);
        self
    }

    pub fn with_metrics(mut self, metrics: WorkerExecutionMetrics) -> Self {
        self.result.metrics = metrics;
        self
    }

    pub fn build(self) -> WorkerResult {
        self.result
    }
}

/// Helper for parsing common task parameters
pub struct TaskParser;

impl TaskParser {
    /// Parse key-value pairs from task string
    pub fn parse_key_value(task: &str) -> Result<HashMap<String, String>, WorkerError> {
        let mut params = HashMap::new();

        for line in task.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once(':') {
                params.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        Ok(params)
    }

    /// Parse JSON from task string
    pub fn parse_json(task: &str) -> Result<serde_json::Value, WorkerError> {
        serde_json::from_str(task).map_err(|e| WorkerError::ExecutionError {
            message: format!("Invalid JSON: {}", e),
        })
    }

    /// Parse boolean values
    pub fn parse_bool(value: &str) -> Result<bool, WorkerError> {
        match value.to_lowercase().as_str() {
            "true" | "yes" | "1" | "on" => Ok(true),
            "false" | "no" | "0" | "off" => Ok(false),
            _ => Err(WorkerError::ExecutionError {
                message: format!("Invalid boolean value: {}", value),
            }),
        }
    }

    /// Parse list values (comma-separated)
    pub fn parse_list(value: &str) -> Vec<String> {
        value
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

/// Execution timing helper
pub struct ExecutionTimer {
    start: std::time::Instant,
}

impl ExecutionTimer {
    pub fn start() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }
}

/// Quality validation helper
pub struct QualityValidator;

impl QualityValidator {
    /// Validate execution result against quality requirements
    pub fn validate_result(
        result: &WorkerResult,
        requirements: &QualityRequirements,
    ) -> Result<(), WorkerError> {
        if let Some(min_coverage) = requirements.min_coverage {
            if let Some(quality_score) = result.quality_score {
                if quality_score < min_coverage as f32 {
                    return Err(WorkerError::ExecutionError {
                        message: format!(
                            "Quality score {:.2} below minimum {:.2}",
                            quality_score, min_coverage
                        ),
                    });
                }
            }
        }

        Ok(())
    }
}

/// Common error conversion helpers
pub struct ErrorConverter;

impl ErrorConverter {
    /// Convert std::io::Error to WorkerError
    pub fn from_io_error(error: std::io::Error, context: &str) -> WorkerError {
        WorkerError::ExecutionError {
            message: format!("{}: {}", context, error),
        }
    }

    /// Convert serde_json::Error to WorkerError
    pub fn from_json_error(error: serde_json::Error) -> WorkerError {
        WorkerError::ExecutionError {
            message: format!("JSON parsing error: {}", error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock worker for testing common infrastructure
    struct MockWorker;

    #[async_trait]
    impl Worker for MockWorker {
        fn id(&self) -> &'static str {
            "mock-worker"
        }

        fn capabilities(&self) -> Vec<String> {
            vec!["test".to_string()]
        }

        fn parse_task(&self, task: &str) -> Result<serde_json::Value, WorkerError> {
            TaskParser::parse_json(task)
        }

        async fn execute_task(
            &self,
            params: &serde_json::Value,
            _context: &WorkerContext,
        ) -> Result<WorkerResult, WorkerError> {
            Ok(WorkerResultBuilder::new(format!("Executed with: {}", params))
                .execution_time_ms(100)
                .quality_score(0.9)
                .build())
        }
    }

    #[tokio::test]
    async fn test_worker_default_execute() {
        let worker = MockWorker;
        let context = WorkerContext::default();

        let task = r#"{"action": "test", "value": 42}"#;
        let result = worker.execute(task.to_string(), Some(context)).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Executed with"));
    }

    #[test]
    fn test_task_parser_key_value() {
        let task = r#"
            action: test
            value: 42
            enabled: true
        "#;

        let params = TaskParser::parse_key_value(task).unwrap();
        assert_eq!(params.get("action"), Some(&"test".to_string()));
        assert_eq!(params.get("value"), Some(&"42".to_string()));
        assert_eq!(params.get("enabled"), Some(&"true".to_string()));
    }

    #[test]
    fn test_task_parser_json() {
        let task = r#"{"action": "test", "value": 42}"#;
        let params = TaskParser::parse_json(task).unwrap();

        assert_eq!(params["action"], "test");
        assert_eq!(params["value"], 42);
    }

    #[test]
    fn test_worker_result_builder() {
        let result = WorkerResultBuilder::new("test output".to_string())
            .success(true)
            .execution_time_ms(150)
            .quality_score(0.95)
            .build();

        assert_eq!(result.output, "test output");
        assert!(result.success);
        assert_eq!(result.execution_time_ms, 150);
        assert_eq!(result.quality_score, Some(0.95));
    }

    #[test]
    fn test_task_parser_bool() {
        assert!(TaskParser::parse_bool("true").unwrap());
        assert!(TaskParser::parse_bool("yes").unwrap());
        assert!(!TaskParser::parse_bool("false").unwrap());
        assert!(!TaskParser::parse_bool("no").unwrap());
        assert!(TaskParser::parse_bool("invalid").is_err());
    }

    #[test]
    fn test_task_parser_list() {
        let list = TaskParser::parse_list("a, b, c");
        assert_eq!(list, vec!["a", "b", "c"]);

        let empty = TaskParser::parse_list("");
        assert!(empty.is_empty());
    }
}


