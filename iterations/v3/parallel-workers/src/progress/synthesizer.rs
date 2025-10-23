//! Progress synthesis and result combination

use crate::types::*;
use crate::error::*;

/// Synthesizes final results from worker outputs
pub struct ProgressSynthesizer;

impl ProgressSynthesizer {
    pub fn new() -> Self {
        Self
    }

    /// Synthesize final task result from worker results
    pub fn synthesize_results(&self, results: Vec<WorkerResult>) -> SynthesisResult<TaskResult> {
        if results.is_empty() {
            return Err(SynthesisError::IncompleteResults {
                completed: 0,
                total: 0,
            });
        }

        // Extract task ID from first result (all should be the same)
        let task_id = results[0].subtask_id.clone(); // This should be task_id, but we use subtask_id for now

        // Calculate overall success
        let success = results.iter().all(|r| r.success);
        let total_subtasks = results.len();
        let completed_subtasks = results.iter().filter(|r| r.success).count();

        // Calculate execution time
        let start_time = results.iter()
            .map(|r| r.metrics.start_time)
            .min()
            .unwrap_or_else(chrono::Utc::now);

        let end_time = results.iter()
            .map(|r| r.metrics.end_time)
            .max()
            .unwrap_or_else(chrono::Utc::now);

        let execution_time = end_time.signed_duration_since(start_time).to_std()
            .unwrap_or(std::time::Duration::from_secs(0));

        // Generate summary
        let summary = self.generate_summary(&results);

        // Create worker breakdown
        let worker_breakdown = self.create_worker_breakdown(&results);

        // Calculate quality scores
        let quality_scores = self.calculate_quality_scores(&results);

        // Detect conflicts between results
        self.detect_conflicts(&results)?;

        Ok(TaskResult {
            task_id: TaskId(task_id.0), // Convert from SubTaskId to TaskId - this needs proper handling
            success,
            subtasks_completed: completed_subtasks,
            total_subtasks,
            execution_time,
            summary,
            worker_breakdown,
            quality_scores,
        })
    }

    /// Generate a comprehensive summary of the parallel execution
    fn generate_summary(&self, results: &[WorkerResult]) -> String {
        let total_workers = results.len();
        let successful_workers = results.iter().filter(|r| r.success).count();
        let failed_workers = total_workers - successful_workers;

        let total_files_modified: usize = results.iter()
            .map(|r| r.metrics.files_modified)
            .sum();

        let total_lines_changed: usize = results.iter()
            .map(|r| r.metrics.lines_changed)
            .sum();

        let total_execution_time: std::time::Duration = results.iter()
            .map(|r| r.metrics.end_time.signed_duration_since(r.metrics.start_time))
            .filter_map(|d| d.to_std().ok())
            .sum();

        let mut summary = format!(
            "Parallel execution completed: {}/{} workers successful",
            successful_workers, total_workers
        );

        if failed_workers > 0 {
            summary.push_str(&format!(" ({} failed)", failed_workers));
        }

        summary.push_str(&format!(
            ". Modified {} files, {} lines in {:.2}s",
            total_files_modified,
            total_lines_changed,
            total_execution_time.as_secs_f32()
        ));

        if let Some(avg_cpu) = self.calculate_average_metric(results, |r| r.metrics.cpu_usage_percent) {
            summary.push_str(&format!(" at {:.1}% avg CPU", avg_cpu));
        }

        summary
    }

    /// Create detailed breakdown by worker
    fn create_worker_breakdown(&self, results: &[WorkerResult]) -> Vec<WorkerBreakdown> {
        results.iter().map(|result| {
            let execution_time = result.metrics.end_time
                .signed_duration_since(result.metrics.start_time)
                .to_std()
                .unwrap_or(std::time::Duration::from_secs(0));

            WorkerBreakdown {
                worker_id: WorkerId(format!("worker-{}", result.subtask_id.0)), // Generate worker ID from subtask
                subtask_id: result.subtask_id.clone(),
                specialty: WorkerSpecialty::Custom {
                    domain: "parallel-execution".to_string(),
                    capabilities: vec!["execution".to_string()],
                }, // This should be passed in from the actual worker
                execution_time,
                success: result.success,
                files_modified: result.metrics.files_modified,
                lines_changed: result.metrics.lines_changed,
            }
        }).collect()
    }

    /// Calculate quality scores from results
    fn calculate_quality_scores(&self, results: &[WorkerResult]) -> std::collections::HashMap<String, f32> {
        let mut scores = std::collections::HashMap::new();

        // Success rate
        let success_rate = results.iter().filter(|r| r.success).count() as f32 / results.len() as f32;
        scores.insert("success_rate".to_string(), success_rate * 100.0);

        // Average CPU usage
        if let Some(avg_cpu) = self.calculate_average_metric(results, |r| r.metrics.cpu_usage_percent) {
            scores.insert("avg_cpu_usage".to_string(), avg_cpu);
        }

        // Average memory usage
        if let Some(avg_memory) = self.calculate_average_metric(results, |r| r.metrics.memory_usage_mb) {
            scores.insert("avg_memory_usage_mb".to_string(), avg_memory);
        }

        // Productivity score (lines changed per minute)
        let total_lines_changed: usize = results.iter().map(|r| r.metrics.lines_changed).sum();
        let total_execution_time: std::time::Duration = results.iter()
            .map(|r| r.metrics.end_time.signed_duration_since(r.metrics.start_time))
            .filter_map(|d| d.to_std().ok())
            .sum();

        if total_execution_time.as_secs() > 0 {
            let lines_per_minute = total_lines_changed as f32 / (total_execution_time.as_secs() as f32 / 60.0);
            scores.insert("productivity_lines_per_minute".to_string(), lines_per_minute);
        }

        scores
    }

    /// Detect conflicts between worker results
    fn detect_conflicts(&self, results: &[WorkerResult]) -> SynthesisResult<()> {
        // Check for overlapping file modifications
        let mut file_modifications = std::collections::HashMap::new();

        for result in results {
            for artifact in &result.artifacts {
                if matches!(artifact.artifact_type, ArtifactType::SourceCode) {
                    let entry = file_modifications.entry(artifact.path.clone())
                        .or_insert_with(Vec::new);
                    entry.push(result.subtask_id.clone());
                }
            }
        }

        // Report conflicts where multiple workers modified the same file
        let conflicts: Vec<_> = file_modifications.iter()
            .filter(|(_, workers)| workers.len() > 1)
            .collect();

        if !conflicts.is_empty() {
            let conflict_descriptions: Vec<String> = conflicts.iter()
                .map(|(path, workers)| {
                    format!("{} modified by workers: {:?}", path.display(),
                           workers.iter().map(|w| w.0.clone()).collect::<Vec<_>>())
                })
                .collect();

            return Err(SynthesisError::ConflictingResults {
                conflict_description: format!("File conflicts detected: {}", conflict_descriptions.join(", ")),
            });
        }

        Ok(())
    }

    /// Calculate average of an optional metric across results
    fn calculate_average_metric<F>(&self, results: &[WorkerResult], extractor: F) -> Option<f32>
    where
        F: Fn(&WorkerResult) -> Option<f32>,
    {
        let values: Vec<f32> = results.iter()
            .filter_map(extractor)
            .collect();

        if values.is_empty() {
            None
        } else {
            Some(values.iter().sum::<f32>() / values.len() as f32)
        }
    }

    /// Merge artifacts from multiple workers
    pub fn merge_artifacts(&self, results: &[WorkerResult]) -> Vec<Artifact> {
        let mut merged = Vec::new();
        let mut seen_paths = std::collections::HashSet::new();

        for result in results {
            for artifact in &result.artifacts {
                if seen_paths.insert(artifact.path.clone()) {
                    merged.push(artifact.clone());
                } else {
                    // Merge duplicate artifacts (same path)
                    if let Some(existing) = merged.iter_mut()
                        .find(|a| a.path == artifact.path) {
                        // Combine sizes and update metadata
                        existing.size_bytes = existing.size_bytes.max(artifact.size_bytes);
                        // Could add conflict markers here
                    }
                }
            }
        }

        merged
    }

    /// Generate performance summary
    pub fn generate_performance_summary(&self, results: &[WorkerResult]) -> PerformanceSummary {
        let total_execution_time: std::time::Duration = results.iter()
            .map(|r| r.metrics.end_time.signed_duration_since(r.metrics.start_time))
            .filter_map(|d| d.to_std().ok())
            .sum();

        let avg_execution_time = if !results.is_empty() {
            total_execution_time / results.len() as u32
        } else {
            std::time::Duration::from_secs(0)
        };

        let total_files_modified: usize = results.iter()
            .map(|r| r.metrics.files_modified)
            .sum();

        let total_lines_changed: usize = results.iter()
            .map(|r| r.metrics.lines_changed)
            .sum();

        PerformanceSummary {
            total_workers: results.len(),
            total_execution_time,
            avg_execution_time_per_worker: avg_execution_time,
            total_files_modified,
            total_lines_changed,
            throughput_files_per_second: total_files_modified as f32 / total_execution_time.as_secs_f32(),
            throughput_lines_per_second: total_lines_changed as f32 / total_execution_time.as_secs_f32(),
            success_rate: results.iter().filter(|r| r.success).count() as f32 / results.len() as f32,
        }
    }
}

/// Performance summary for parallel execution
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub total_workers: usize,
    pub total_execution_time: std::time::Duration,
    pub avg_execution_time_per_worker: std::time::Duration,
    pub total_files_modified: usize,
    pub total_lines_changed: usize,
    pub throughput_files_per_second: f32,
    pub throughput_lines_per_second: f32,
    pub success_rate: f32,
}

/// Result merger for combining similar results
pub struct ResultMerger;

impl ResultMerger {
    pub fn new() -> Self {
        Self
    }

    /// Merge results that operate on the same domain
    pub fn merge_by_domain(&self, results: Vec<WorkerResult>) -> SynthesisResult<WorkerResult> {
        if results.is_empty() {
            return Err(SynthesisError::IncompleteResults {
                completed: 0,
                total: 0,
            });
        }

        // Use the first result as base
        let mut merged = results[0].clone();

        // Merge metrics from all results
        for result in &results[1..] {
            merged.metrics.files_modified += result.metrics.files_modified;
            merged.metrics.lines_changed += result.metrics.lines_changed;

            // Update time bounds
            merged.metrics.start_time = merged.metrics.start_time.min(result.metrics.start_time);
            merged.metrics.end_time = merged.metrics.end_time.max(result.metrics.end_time);

            // Combine CPU/memory averages (weighted)
            if let (Some(cpu1), Some(cpu2)) = (merged.metrics.cpu_usage_percent, result.metrics.cpu_usage_percent) {
                merged.metrics.cpu_usage_percent = Some((cpu1 + cpu2) / 2.0);
            }

            // Merge artifacts
            merged.artifacts.extend(result.artifacts.clone());
        }

        // Update success based on all results
        merged.success = results.iter().all(|r| r.success);

        // Combine outputs
        merged.output = results.iter()
            .map(|r| r.output.as_str())
            .collect::<Vec<_>>()
            .join("\n---\n");

        // Combine errors if any
        let error_messages: Vec<_> = results.iter()
            .filter_map(|r| r.error_message.as_ref())
            .map(|s| s.as_str())
            .collect();

        if !error_messages.is_empty() {
            merged.error_message = Some(error_messages.join("; "));
        }

        Ok(merged)
    }
}

impl Default for ProgressSynthesizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ResultMerger {
    fn default() -> Self {
        Self::new()
    }
}




