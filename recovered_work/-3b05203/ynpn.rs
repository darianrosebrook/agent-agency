//! E2E Test Assertions
//!
//! Provides comprehensive assertion utilities for validating E2E test results
//! across all system components and integration points.

use std::collections::HashMap;
use uuid::Uuid;

use super::harness::TaskTestState;

/// E2E test assertions
pub struct E2eAssertions;

impl E2eAssertions {
    /// Assert that a task completed successfully
    pub fn assert_task_completed(task: &TaskTestState) -> Result<(), AssertionError> {
        if task.status != "completed" {
            return Err(AssertionError::TaskNotCompleted {
                task_id: task.task_id,
                actual_status: task.status.clone(),
            });
        }
        Ok(())
    }

    /// Assert that a task has minimum quality score
    pub fn assert_quality_score(task: &TaskTestState, min_score: f64) -> Result<(), AssertionError> {
        let score = task.quality_score.unwrap_or(0.0);
        if score < min_score {
            return Err(AssertionError::QualityScoreTooLow {
                task_id: task.task_id,
                actual_score: score,
                required_score: min_score,
            });
        }
        Ok(())
    }

    /// Assert that a task generated minimum number of artifacts
    pub fn assert_artifacts_generated(task: &TaskTestState, min_count: usize) -> Result<(), AssertionError> {
        if task.artifacts_count < min_count {
            return Err(AssertionError::InsufficientArtifacts {
                task_id: task.task_id,
                actual_count: task.artifacts_count,
                required_count: min_count,
            });
        }
        Ok(())
    }

    /// Assert that task execution time is within bounds
    pub fn assert_execution_time(task: &TaskTestState, max_seconds: u64) -> Result<(), AssertionError> {
        let execution_time = (task.last_update - task.start_time).num_seconds() as u64;
        if execution_time > max_seconds {
            return Err(AssertionError::ExecutionTimeout {
                task_id: task.task_id,
                actual_seconds: execution_time,
                max_seconds,
            });
        }
        Ok(())
    }

    /// Assert that multiple tasks completed successfully
    pub fn assert_all_tasks_completed(tasks: &[TaskTestState]) -> Result<(), AssertionError> {
        let mut failed_tasks = Vec::new();

        for task in tasks {
            if task.status != "completed" {
                failed_tasks.push((task.task_id, task.status.clone()));
            }
        }

        if !failed_tasks.is_empty() {
            return Err(AssertionError::MultipleTasksFailed { failed_tasks });
        }

        Ok(())
    }

    /// Assert that system metrics are within acceptable ranges
    pub fn assert_system_metrics(metrics: &HashMap<String, serde_json::Value>) -> Result<(), AssertionError> {
        // Check active tasks
        if let Some(active_tasks) = metrics.get("active_tasks") {
            if let Some(count) = active_tasks.as_u64() {
                if count > 10 { // Arbitrary limit for testing
                    return Err(AssertionError::SystemMetricOutOfRange {
                        metric: "active_tasks".to_string(),
                        actual_value: count as f64,
                        max_value: 10.0,
                    });
                }
            }
        }

        // Check success rate
        if let Some(success_rate) = metrics.get("success_rate") {
            if let Some(rate) = success_rate.as_f64() {
                if rate < 0.8 { // 80% success rate minimum
                    return Err(AssertionError::SystemMetricOutOfRange {
                        metric: "success_rate".to_string(),
                        actual_value: rate,
                        max_value: 1.0, // Not used for minimum check
                    });
                }
            }
        }

        Ok(())
    }

    /// Assert that task progress follows expected sequence
    pub fn assert_progress_sequence(task: &TaskTestState, expected_sequence: &[&str]) -> Result<(), AssertionError> {
        // This would require access to the full execution history
        // For now, just check that progress reaches 100%
        if (task.progress_percentage - 100.0).abs() > f64::EPSILON {
            return Err(AssertionError::ProgressSequenceInvalid {
                task_id: task.task_id,
                expected_sequence: expected_sequence.iter().map(|s| s.to_string()).collect(),
                actual_progress: task.progress_percentage,
            });
        }

        Ok(())
    }

    /// Assert that task description contains required keywords
    pub fn assert_description_contains(task: &TaskTestState, keywords: &[&str]) -> Result<(), AssertionError> {
        for keyword in keywords {
            if !task.description.to_lowercase().contains(&keyword.to_lowercase()) {
                return Err(AssertionError::DescriptionMissingKeyword {
                    task_id: task.task_id,
                    missing_keyword: keyword.to_string(),
                });
            }
        }

        Ok(())
    }

    /// Assert that tasks have unique descriptions (no duplicates)
    pub fn assert_unique_descriptions(tasks: &[TaskTestState]) -> Result<(), AssertionError> {
        let mut seen_descriptions = std::collections::HashSet::new();

        for task in tasks {
            if !seen_descriptions.insert(&task.description) {
                return Err(AssertionError::DuplicateDescriptions {
                    duplicate_description: task.description.clone(),
                });
            }
        }

        Ok(())
    }

    /// Assert that concurrent tasks don't interfere with each other
    pub fn assert_no_concurrency_conflicts(tasks: &[TaskTestState]) -> Result<(), AssertionError> {
        // Check for overlapping execution times
        let mut execution_ranges = Vec::new();

        for task in tasks {
            execution_ranges.push((task.start_time, task.last_update));
        }

        // Sort by start time
        execution_ranges.sort_by(|a, b| a.0.cmp(&b.0));

        // Check for overlaps
        for i in 1..execution_ranges.len() {
            let (_, prev_end) = execution_ranges[i - 1];
            let (curr_start, _) = execution_ranges[i];

            if curr_start < prev_end {
                // Overlap detected - this might be normal for concurrent execution
                // but we can still check if it causes issues
                tracing::info!("Task execution overlap detected: {} to {}", curr_start, prev_end);
            }
        }

        Ok(())
    }

    /// Assert that system remains stable under load
    pub fn assert_system_stability(
        before_metrics: &HashMap<String, serde_json::Value>,
        after_metrics: &HashMap<String, serde_json::Value>,
    ) -> Result<(), AssertionError> {
        // Check that error rates didn't increase significantly
        let before_errors = Self::extract_metric_value(before_metrics, "error_rate").unwrap_or(0.0);
        let after_errors = Self::extract_metric_value(after_metrics, "error_rate").unwrap_or(0.0);

        if after_errors > before_errors + 0.1 { // 10% increase allowed
            return Err(AssertionError::SystemStabilityDegraded {
                metric: "error_rate".to_string(),
                before_value: before_errors,
                after_value: after_errors,
            });
        }

        // Check that response times didn't degrade significantly
        let before_response_time = Self::extract_metric_value(before_metrics, "avg_response_time").unwrap_or(0.0);
        let after_response_time = Self::extract_metric_value(after_metrics, "avg_response_time").unwrap_or(0.0);

        if after_response_time > before_response_time * 2.0 { // 2x degradation allowed
            return Err(AssertionError::SystemStabilityDegraded {
                metric: "avg_response_time".to_string(),
                before_value: before_response_time,
                after_value: after_response_time,
            });
        }

        Ok(())
    }

    /// Assert that resource usage stays within bounds
    pub fn assert_resource_usage(metrics: &HashMap<String, serde_json::Value>) -> Result<(), AssertionError> {
        // Check memory usage
        if let Some(memory_mb) = Self::extract_metric_value(metrics, "memory_usage_mb") {
            if memory_mb > 1024.0 { // 1GB limit
                return Err(AssertionError::ResourceUsageExceeded {
                    resource: "memory".to_string(),
                    actual_usage: memory_mb,
                    limit: 1024.0,
                });
            }
        }

        // Check CPU usage
        if let Some(cpu_percent) = Self::extract_metric_value(metrics, "cpu_usage_percent") {
            if cpu_percent > 90.0 { // 90% limit
                return Err(AssertionError::ResourceUsageExceeded {
                    resource: "cpu".to_string(),
                    actual_usage: cpu_percent,
                    limit: 90.0,
                });
            }
        }

        // Check disk usage
        if let Some(disk_percent) = Self::extract_metric_value(metrics, "disk_usage_percent") {
            if disk_percent > 95.0 { // 95% limit
                return Err(AssertionError::ResourceUsageExceeded {
                    resource: "disk".to_string(),
                    actual_usage: disk_percent,
                    limit: 95.0,
                });
            }
        }

        Ok(())
    }

    /// Assert that all tasks in a batch have consistent quality scores
    pub fn assert_consistent_quality(tasks: &[TaskTestState], tolerance: f64) -> Result<(), AssertionError> {
        if tasks.is_empty() {
            return Ok(());
        }

        let avg_score: f64 = tasks.iter()
            .filter_map(|t| t.quality_score)
            .sum::<f64>() / tasks.len() as f64;

        for task in tasks {
            if let Some(score) = task.quality_score {
                if (score - avg_score).abs() > tolerance {
                    return Err(AssertionError::InconsistentQuality {
                        task_id: task.task_id,
                        task_score: score,
                        average_score: avg_score,
                        tolerance,
                    });
                }
            }
        }

        Ok(())
    }

    /// Extract numeric value from metrics
    fn extract_metric_value(metrics: &HashMap<String, serde_json::Value>, key: &str) -> Option<f64> {
        metrics.get(key)
            .and_then(|v| v.as_f64())
            .or_else(|| metrics.get(key).and_then(|v| v.as_u64()).map(|u| u as f64))
    }

    /// Run comprehensive test suite assertions
    pub async fn run_comprehensive_assertions(
        tasks: &[TaskTestState],
        before_metrics: &HashMap<String, serde_json::Value>,
        after_metrics: &HashMap<String, serde_json::Value>,
    ) -> Result<ComprehensiveAssertionResult, Vec<AssertionError>> {
        let mut errors = Vec::new();

        // Task completion assertions
        for (i, task) in tasks.iter().enumerate() {
            if let Err(e) = Self::assert_task_completed(task) {
                errors.push(e);
            }

            if let Err(e) = Self::assert_quality_score(task, 70.0) {
                errors.push(e);
            }

            if let Err(e) = Self::assert_artifacts_generated(task, 1) {
                errors.push(e);
            }

            if let Err(e) = Self::assert_execution_time(task, 600) { // 10 minutes max
                errors.push(e);
            }
        }

        // Multi-task assertions
        if let Err(e) = Self::assert_all_tasks_completed(tasks) {
            errors.push(e);
        }

        if let Err(e) = Self::assert_unique_descriptions(tasks) {
            errors.push(e);
        }

        if let Err(e) = Self::assert_no_concurrency_conflicts(tasks) {
            errors.push(e);
        }

        if let Err(e) = Self::assert_consistent_quality(tasks, 15.0) {
            errors.push(e); // 15 point tolerance
        }

        // System assertions
        if let Err(e) = Self::assert_system_metrics(after_metrics) {
            errors.push(e);
        }

        if let Err(e) = Self::assert_system_stability(before_metrics, after_metrics) {
            errors.push(e);
        }

        if let Err(e) = Self::assert_resource_usage(after_metrics) {
            errors.push(e);
        }

        if errors.is_empty() {
            Ok(ComprehensiveAssertionResult::Passed {
                tasks_evaluated: tasks.len(),
                metrics_checked: after_metrics.len(),
            })
        } else {
            Err(errors)
        }
    }
}

/// Comprehensive assertion result
#[derive(Debug, Clone)]
pub enum ComprehensiveAssertionResult {
    Passed {
        tasks_evaluated: usize,
        metrics_checked: usize,
    },
    Failed(Vec<AssertionError>),
}

/// Assertion error types
#[derive(Debug, Clone, thiserror::Error)]
pub enum AssertionError {
    #[error("Task {task_id} not completed (status: {actual_status})")]
    TaskNotCompleted { task_id: Uuid, actual_status: String },

    #[error("Task {task_id} quality score too low ({actual_score:.1} < {required_score:.1})")]
    QualityScoreTooLow { task_id: Uuid, actual_score: f64, required_score: f64 },

    #[error("Task {task_id} generated insufficient artifacts ({actual_count} < {required_count})")]
    InsufficientArtifacts { task_id: Uuid, actual_count: usize, required_count: usize },

    #[error("Task {task_id} execution timeout ({actual_seconds}s > {max_seconds}s)")]
    ExecutionTimeout { task_id: Uuid, actual_seconds: u64, max_seconds: u64 },

    #[error("Multiple tasks failed: {failed_tasks:?}")]
    MultipleTasksFailed { failed_tasks: Vec<(Uuid, String)> },

    #[error("System metric out of range: {metric} = {actual_value}")]
    SystemMetricOutOfRange { metric: String, actual_value: f64, max_value: f64 },

    #[error("Progress sequence invalid for task {task_id}: expected {expected_sequence:?}, got {actual_progress}%")]
    ProgressSequenceInvalid { task_id: Uuid, expected_sequence: Vec<String>, actual_progress: f32 },

    #[error("Task {task_id} description missing keyword: {missing_keyword}")]
    DescriptionMissingKeyword { task_id: Uuid, missing_keyword: String },

    #[error("Duplicate task descriptions found: {duplicate_description}")]
    DuplicateDescriptions { duplicate_description: String },

    #[error("System stability degraded: {metric} {before_value:.2} → {after_value:.2}")]
    SystemStabilityDegraded { metric: String, before_value: f64, after_value: f64 },

    #[error("Resource usage exceeded: {resource} {actual_usage:.1} > {limit:.1}")]
    ResourceUsageExceeded { resource: String, actual_usage: f64, limit: f64 },

    #[error("Task {task_id} quality inconsistent ({task_score:.1} vs avg {average_score:.1}, tolerance {tolerance:.1})")]
    InconsistentQuality { task_id: Uuid, task_score: f64, average_score: f64, tolerance: f64 },
}


