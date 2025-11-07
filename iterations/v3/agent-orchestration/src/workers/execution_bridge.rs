//! Worker Execution Bridge
//!
//! Bridges agent-orchestration with agent-workers for task execution.
//! Converts between orchestrator types (Milestone, ExecutionArtifacts) and
//! worker types (TaskDefinition, TaskResult).

use std::path::PathBuf;
use std::sync::Arc;
use std::process::Command;
use anyhow::{anyhow, Result, Context};
use uuid::Uuid;
use tracing::{info, warn};
use std::collections::HashMap;

use agent_agency_contracts::planning_io::Milestone;
use agent_agency_contracts::execution_artifacts::ExecutionArtifacts;
use agent_agency_contracts::TaskPriority;
use agent_workers::MCPWorkerPool;
use agent_workers::TaskExecutor;
use agent_workers::{TaskDefinition, Priority as WorkerPriority};
use agent_workers::{TaskResult as WorkerTaskResult, SubTask, ParallelExecutionPlan};

/// Bridge between orchestrator and worker execution systems
pub struct WorkerExecutionBridge {
    /// MCP worker pool for task execution
    worker_pool: Arc<MCPWorkerPool>,
    
    /// Task executor for individual task execution
    task_executor: Arc<TaskExecutor>,
}

impl WorkerExecutionBridge {
    /// Create a new worker execution bridge
    pub fn new(
        worker_pool: Arc<MCPWorkerPool>,
        task_executor: Arc<TaskExecutor>,
    ) -> Self {
        Self {
            worker_pool,
            task_executor,
        }
    }

    /// Execute a milestone using agent-workers
    ///
    /// Converts milestone to TaskDefinition, executes via worker pool,
    /// and converts result back to ExecutionArtifacts.
    pub async fn execute_milestone(
        &self,
        milestone: &Milestone,
        worktree_path: &PathBuf,
        worker_id: Uuid,
    ) -> Result<ExecutionArtifacts> {
        info!(
            "Executing milestone {} via worker {} in worktree {}",
            milestone.id,
            worker_id,
            worktree_path.display()
        );

        // Convert milestone to TaskDefinition
        let task_def = self.milestone_to_task_definition(milestone, worktree_path)?;

        // Execute via worker pool - convert error to anyhow
        let worker_result = self.worker_pool.execute_task(task_def).await
            .map_err(|e| anyhow!("Worker execution failed: {}", e))?;

        // Convert worker result to ExecutionArtifacts
        let artifacts = self.worker_result_to_artifacts(
            &worker_result,
            milestone,
            worktree_path,
        )?;

        Ok(artifacts)
    }

    /// Execute parallel tasks using agent-workers ParallelCoordinator
    pub async fn execute_parallel(
        &self,
        plan: &ParallelExecutionPlan,
        worktree_paths: &std::collections::HashMap<Uuid, PathBuf>,
    ) -> Result<Vec<ExecutionArtifacts>> {
        info!("Executing parallel plan with {} subtasks", plan.subtasks.len());

        // Use agent-workers ParallelCoordinator for parallel execution
        // Note: This will be implemented when we integrate ParallelCoordinator
        // For now, execute sequentially
        let mut results = Vec::new();
        
        for task in &plan.subtasks {
            // Find worktree path for this task's worker
            let worker_id = task.assigned_worker.map(|w| w.0).unwrap_or_else(Uuid::new_v4);
            let worktree_path = worktree_paths.get(&worker_id)
                .ok_or_else(|| anyhow!("No worktree path for worker {}", worker_id))?;

            // Convert task to milestone-like structure
            let milestone = self.parallel_task_to_milestone(task)?;
            
            // Execute milestone
            let artifacts = self.execute_milestone(&milestone, worktree_path, worker_id).await?;
            results.push(artifacts);
        }

        Ok(results)
    }

    /// Convert Milestone to TaskDefinition for worker execution
    fn milestone_to_task_definition(
        &self,
        milestone: &Milestone,
        worktree_path: &PathBuf,
    ) -> Result<TaskDefinition> {
        // Extract required tools from milestone scope and interfaces
        let mut required_tools = Vec::new();
        
        // Add tools based on milestone scope files/directories
        if !milestone.scope.files.is_empty() || !milestone.scope.directories.is_empty() {
            required_tools.push("code_editor".to_string());
        }

        // Default to code-editing tool if no tools specified
        if required_tools.is_empty() {
            required_tools.push("code_editor".to_string());
        }

        // Convert priority - TaskDefinition uses TaskPriority from contracts
        let priority: TaskPriority = match milestone.priority {
            agent_agency_contracts::planning_io::MilestonePriority::Low => TaskPriority::Low,
            agent_agency_contracts::planning_io::MilestonePriority::Normal => TaskPriority::Medium,
            agent_agency_contracts::planning_io::MilestonePriority::High => TaskPriority::High,
            agent_agency_contracts::planning_io::MilestonePriority::Critical => TaskPriority::Critical,
        };

        // Build task parameters from milestone
        let mut parameters = std::collections::HashMap::new();
        parameters.insert("objective".to_string(), serde_json::json!(milestone.objective));
        parameters.insert("scope".to_string(), serde_json::json!({
            "files": milestone.scope.files,
            "directories": milestone.scope.directories,
            "included_paths": milestone.scope.included_paths,
            "excluded_paths": milestone.scope.excluded_paths,
        }));
        parameters.insert("interfaces".to_string(), serde_json::json!(milestone.interfaces));
        parameters.insert("tests".to_string(), serde_json::json!(milestone.tests));
        parameters.insert("worktree_path".to_string(), serde_json::json!(worktree_path.display().to_string()));

        Ok(TaskDefinition {
            id: Uuid::new_v4(),
            name: format!("milestone_{}", milestone.id),
            description: milestone.objective.clone(),
            required_tools,
            parameters,
            timeout_seconds: milestone.estimated_duration.map(|m| m as u32 * 60),
            priority,
            deadline: None,
            metadata: std::collections::HashMap::from([
                ("milestone_id".to_string(), serde_json::json!(milestone.id)),
                ("risk_tier".to_string(), serde_json::json!(milestone.risk_tier)),
                ("estimated_effort".to_string(), serde_json::json!(milestone.estimated_effort)),
            ]),
        })
    }

    /// Convert WorkerTaskResult to ExecutionArtifacts
    fn worker_result_to_artifacts(
        &self,
        worker_result: &WorkerTaskResult,
        milestone: &Milestone,
        worktree_path: &PathBuf,
    ) -> Result<ExecutionArtifacts> {
        // Extract code changes from worker result metadata
        let code_changes = self.extract_code_changes(worker_result, worktree_path)?;
        
        // Extract test results from worker result
        let tests = self.extract_test_results(worker_result)?;
        
        // Extract coverage from worker result quality scores
        let coverage = self.extract_coverage(worker_result)?;
        
        // Extract linting results from worker result errors
        let linting = self.extract_linting(worker_result)?;
        
        // Build provenance from worker result
        let provenance = self.build_provenance(worker_result, milestone, worktree_path)?;

        Ok(ExecutionArtifacts {
            version: "1.0.0".to_string(),
            task_id: worker_result.task_id.0,
            working_spec_id: milestone.id.clone(),
            iteration: 0, // Will be set by caller
            code_changes,
            tests,
            coverage,
            linting,
            provenance,
            metadata: None,
        })
    }

    /// Extract code changes from worker result
    fn extract_code_changes(
        &self,
        worker_result: &WorkerTaskResult,
        worktree_path: &PathBuf,
    ) -> Result<agent_agency_contracts::execution_artifacts::CodeChanges> {
        let mut diffs = Vec::new();
        let mut new_files = Vec::new();
        let mut deleted_files = Vec::new();
        let mut files_modified = 0;
        let mut lines_added = 0;
        let mut lines_removed = 0;

        // First, try to get diffs from git in worktree
        if worktree_path.exists() {
            // Get git diff for uncommitted changes in worktree
            let diff_output = Command::new("git")
                .current_dir(worktree_path)
                .arg("diff")
                .arg("--no-color")
                .arg("HEAD")
                .output()
                .context("Failed to execute git diff")?;

            if diff_output.status.success() {
                let diff_text = String::from_utf8_lossy(&diff_output.stdout);
                if !diff_text.trim().is_empty() {
                    diffs.push(diff_text.to_string());
                    
                    // Parse diff statistics
                    let stat_output = Command::new("git")
                        .current_dir(worktree_path)
                        .arg("diff")
                        .arg("--stat")
                        .arg("HEAD")
                        .output()
                        .context("Failed to execute git diff --stat")?;

                    if stat_output.status.success() {
                        let stat_text = String::from_utf8_lossy(&stat_output.stdout);
                        for line in stat_text.lines() {
                            if let Some(pipe_idx) = line.rfind('|') {
                                if let Some(plus_minus) = line[pipe_idx + 1..].trim().split_whitespace().next() {
                                    // Parse "X +Y -Z" format
                                    let parts: Vec<&str> = plus_minus.split_whitespace().collect();
                                    for part in parts {
                                        if part.starts_with('+') {
                                            if let Ok(adds) = part[1..].parse::<u32>() {
                                                lines_added += adds;
                                            }
                                        } else if part.starts_with('-') {
                                            if let Ok(removes) = part[1..].parse::<u32>() {
                                                lines_removed += removes;
                                            }
                                        }
                                    }
                                }
                                files_modified += 1;
                            }
                        }
                    }
                }
            }

            // Get list of new files (untracked)
            let untracked_output = Command::new("git")
                .current_dir(worktree_path)
                .arg("ls-files")
                .arg("--others")
                .arg("--exclude-standard")
                .output()
                .context("Failed to list untracked files")?;

            if untracked_output.status.success() {
                let untracked_text = String::from_utf8_lossy(&untracked_output.stdout);
                for line in untracked_text.lines() {
                    if !line.trim().is_empty() {
                        new_files.push(line.trim().to_string());
                    }
                }
            }

            // Get list of deleted files
            let deleted_output = Command::new("git")
                .current_dir(worktree_path)
                .arg("diff")
                .arg("--name-only")
                .arg("--diff-filter=D")
                .arg("HEAD")
                .output()
                .context("Failed to list deleted files")?;

            if deleted_output.status.success() {
                let deleted_text = String::from_utf8_lossy(&deleted_output.stdout);
                for line in deleted_text.lines() {
                    if !line.trim().is_empty() {
                        deleted_files.push(line.trim().to_string());
                    }
                }
            }
        }

        // Also check metadata for code change information (fallback or additional data)
        if let Some(diff_info) = worker_result.metadata.get("code_changes") {
            if let Some(diff_str) = diff_info.as_str() {
                if !diff_str.is_empty() && diffs.is_empty() {
                    diffs.push(diff_str.to_string());
                }
            }
        }

        // Extract statistics from metadata if available
        if let Some(stats) = worker_result.metadata.get("code_change_stats") {
            if let Some(obj) = stats.as_object() {
                if let Some(files) = obj.get("files_modified").and_then(|v| v.as_u64()) {
                    files_modified = files as u32;
                }
                if let Some(added) = obj.get("lines_added").and_then(|v| v.as_u64()) {
                    lines_added = added as u32;
                }
                if let Some(removed) = obj.get("lines_removed").and_then(|v| v.as_u64()) {
                    lines_removed = removed as u32;
                }
            }
        }

        Ok(agent_agency_contracts::execution_artifacts::CodeChanges {
            diffs,
            new_files,
            deleted_files,
            statistics: agent_agency_contracts::execution_artifacts::CodeChangeStats {
                files_modified,
                lines_added,
                lines_removed,
                total_loc: lines_added + lines_removed,
            },
        })
    }

    /// Extract test results from worker result
    fn extract_test_results(
        &self,
        worker_result: &WorkerTaskResult,
    ) -> Result<agent_agency_contracts::execution_artifacts::TestArtifacts> {
        // Try to parse structured test results from metadata first
        let mut unit_total = 0u32;
        let mut unit_passed = 0u32;
        let mut unit_failed = 0u32;
        let mut unit_skipped = 0u32;
        let mut integration_total = 0u32;
        let mut integration_passed = 0u32;
        let mut integration_failed = 0u32;
        let mut test_files = Vec::new();
        let mut test_results = Vec::new();

        // Check metadata for structured test results (e.g., from Jest tool)
        if let Some(test_data) = worker_result.metadata.get("test_results") {
            if let Some(test_obj) = test_data.as_object() {
                // Parse Jest-style test results
                if let Some(test_results_obj) = test_obj.get("testResults").and_then(|v| v.as_object()) {
                    if let Some(num_passed) = test_results_obj.get("numPassedTests").and_then(|v| v.as_u64()) {
                        unit_passed = num_passed as u32;
                    }
                    if let Some(num_failed) = test_results_obj.get("numFailedTests").and_then(|v| v.as_u64()) {
                        unit_failed = num_failed as u32;
                    }
                    if let Some(num_pending) = test_results_obj.get("numPendingTests").and_then(|v| v.as_u64()) {
                        unit_skipped = num_pending as u32;
                    }
                    unit_total = unit_passed + unit_failed + unit_skipped;

                    // Extract test file paths
                    if let Some(test_files_array) = test_results_obj.get("testResults").and_then(|v| v.as_array()) {
                        for test_file in test_files_array {
                            if let Some(file_obj) = test_file.as_object() {
                                if let Some(file_path) = file_obj.get("testFilePath").and_then(|v| v.as_str()) {
                                    test_files.push(file_path.to_string());
                                }
                            }
                        }
                    }
                }

                // Parse Rust test results (cargo test format)
                if let Some(cargo_tests) = test_obj.get("cargo_tests").and_then(|v| v.as_object()) {
                    if let Some(passed) = cargo_tests.get("passed").and_then(|v| v.as_u64()) {
                        unit_passed = passed as u32;
                    }
                    if let Some(failed) = cargo_tests.get("failed").and_then(|v| v.as_u64()) {
                        unit_failed = failed as u32;
                    }
                    if let Some(ignored) = cargo_tests.get("ignored").and_then(|v| v.as_u64()) {
                        unit_skipped = ignored as u32;
                    }
                    unit_total = unit_passed + unit_failed + unit_skipped;
                }

                // Parse integration test results
                if let Some(integration_obj) = test_obj.get("integration_tests").and_then(|v| v.as_object()) {
                    if let Some(passed) = integration_obj.get("passed").and_then(|v| v.as_u64()) {
                        integration_passed = passed as u32;
                    }
                    if let Some(failed) = integration_obj.get("failed").and_then(|v| v.as_u64()) {
                        integration_failed = failed as u32;
                    }
                    integration_total = integration_passed + integration_failed;
                }
            }
        }

        // Fallback to quality scores if metadata doesn't have structured data
        if unit_total == 0 {
            unit_total = worker_result.quality_scores.get("tests_total")
                .copied()
                .unwrap_or(0.0) as u64 as u32;
            unit_passed = worker_result.quality_scores.get("tests_passed")
                .copied()
                .unwrap_or(0.0) as u64 as u32;
            unit_failed = unit_total.saturating_sub(unit_passed);
        }

        Ok(agent_agency_contracts::execution_artifacts::TestArtifacts {
            unit_tests: agent_agency_contracts::execution_artifacts::TestSuiteResults {
                total: unit_total,
                passed: unit_passed,
                failed: unit_failed,
                skipped: unit_skipped,
                duration_ms: worker_result.execution_time_ms,
                results: test_results,
            },
            integration_tests: agent_agency_contracts::execution_artifacts::TestSuiteResults {
                total: integration_total,
                passed: integration_passed,
                failed: integration_failed,
                skipped: 0,
                duration_ms: 0,
                results: Vec::new(),
            },
            e2e_tests: agent_agency_contracts::execution_artifacts::E2eTestResults::default(),
            test_files,
        })
    }

    /// Extract coverage from worker result
    fn extract_coverage(
        &self,
        worker_result: &WorkerTaskResult,
    ) -> Result<agent_agency_contracts::execution_artifacts::CoverageResults> {
        let mut line_coverage = 0.0;
        let mut branch_coverage = 0.0;
        let mut function_coverage = 0.0;
        let mut mutation_score = 0.0;
        let mut coverage_report_path = None;
        let mut uncovered_lines = Vec::new();
        let mut uncovered_branches = Vec::new();

        // Try to parse structured coverage from metadata first
        if let Some(coverage_data) = worker_result.metadata.get("coverage") {
            if let Some(cov_obj) = coverage_data.as_object() {
                // Parse Jest-style coverage
                if let Some(summary) = cov_obj.get("summary").and_then(|v| v.as_object()) {
                    if let Some(lines) = summary.get("lines").and_then(|v| v.as_object()) {
                        if let Some(pct) = lines.get("pct").and_then(|v| v.as_f64()) {
                            line_coverage = pct / 100.0;
                        }
                    }
                    if let Some(branches) = summary.get("branches").and_then(|v| v.as_object()) {
                        if let Some(pct) = branches.get("pct").and_then(|v| v.as_f64()) {
                            branch_coverage = pct / 100.0;
                        }
                    }
                    if let Some(functions) = summary.get("functions").and_then(|v| v.as_object()) {
                        if let Some(pct) = functions.get("pct").and_then(|v| v.as_f64()) {
                            function_coverage = pct / 100.0;
                        }
                    }
                }

                // Parse Rust coverage (cargo-tarpaulin format)
                if let Some(line_pct) = cov_obj.get("line_coverage").and_then(|v| v.as_f64()) {
                    line_coverage = line_pct;
                }
                if let Some(branch_pct) = cov_obj.get("branch_coverage").and_then(|v| v.as_f64()) {
                    branch_coverage = branch_pct;
                }

                // Extract coverage report path
                if let Some(path) = cov_obj.get("report_path").and_then(|v| v.as_str()) {
                    coverage_report_path = Some(path.to_string());
                }

                // Extract uncovered lines
                if let Some(uncovered) = cov_obj.get("uncovered_lines").and_then(|v| v.as_array()) {
                    for item in uncovered {
                        if let Some(line_obj) = item.as_object() {
                            if let Some(file) = line_obj.get("file").and_then(|v| v.as_str()) {
                                if let Some(line) = line_obj.get("line").and_then(|v| v.as_u64()) {
                                    uncovered_lines.push(agent_agency_contracts::execution_artifacts::UncoveredLine {
                                        file_path: file.to_string(),
                                        line_number: line as u32,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fallback to quality scores if metadata doesn't have structured data
        if line_coverage == 0.0 {
            line_coverage = worker_result.quality_scores.get("coverage_line")
                .copied()
                .unwrap_or(0.0);
        }
        if branch_coverage == 0.0 {
            branch_coverage = worker_result.quality_scores.get("coverage_branch")
                .copied()
                .unwrap_or(0.0);
        }

        // Extract mutation score if available
        if let Some(score) = worker_result.quality_scores.get("mutation_score") {
            mutation_score = *score;
        } else if let Some(score) = worker_result.metadata.get("mutation_score").and_then(|v| v.as_f64()) {
            mutation_score = score;
        }

        Ok(agent_agency_contracts::execution_artifacts::CoverageResults {
            line_coverage,
            branch_coverage,
            function_coverage,
            mutation_score,
            coverage_report_path,
            uncovered_lines,
            uncovered_branches,
        })
    }

    /// Extract linting results from worker result
    fn extract_linting(
        &self,
        worker_result: &WorkerTaskResult,
    ) -> Result<agent_agency_contracts::execution_artifacts::LintingResults> {
        let mut errors = 0u32;
        let mut warnings = 0u32;
        let mut info = 0u32;
        let mut issues_by_file = HashMap::new();
        let mut linter_version = None;
        let mut config_used = None;

        // Try to parse structured linting results from metadata
        if let Some(lint_data) = worker_result.metadata.get("linting") {
            if let Some(lint_obj) = lint_data.as_object() {
                // Parse ESLint-style results
                if let Some(issues_array) = lint_obj.get("results").and_then(|v| v.as_array()) {
                    for result in issues_array {
                        if let Some(result_obj) = result.as_object() {
                            if let Some(file_path) = result_obj.get("filePath").and_then(|v| v.as_str()) {
                                if let Some(messages) = result_obj.get("messages").and_then(|v| v.as_array()) {
                                    let mut file_errors = 0u32;
                                    let mut file_warnings = 0u32;
                                    let mut file_info = 0u32;

                                    for message in messages {
                                        if let Some(msg_obj) = message.as_object() {
                                            if let Some(severity) = msg_obj.get("severity").and_then(|v| v.as_u64()) {
                                                match severity {
                                                    1 => warnings += 1,
                                                    2 => errors += 1,
                                                    _ => info += 1,
                                                }
                                                match severity {
                                                    1 => file_warnings += 1,
                                                    2 => file_errors += 1,
                                                    _ => file_info += 1,
                                                }
                                            }
                                        }
                                    }

                                    if file_errors > 0 || file_warnings > 0 || file_info > 0 {
                                        issues_by_file.insert(file_path.to_string(), agent_agency_contracts::execution_artifacts::FileLintingIssues {
                                            errors: file_errors,
                                            warnings: file_warnings,
                                            info: file_info,
                                        });
                                    }
                                }
                            }
                }
                }

                // Parse Rust clippy-style results
                if let Some(clippy_obj) = lint_obj.get("clippy").and_then(|v| v.as_object()) {
                    if let Some(warnings_count) = clippy_obj.get("warnings").and_then(|v| v.as_u64()) {
                        warnings = warnings_count as u32;
                    }
                    if let Some(errors_count) = clippy_obj.get("errors").and_then(|v| v.as_u64()) {
                        errors = errors_count as u32;
                    }
                }

                // Extract linter version
                if let Some(version) = lint_obj.get("linter_version").and_then(|v| v.as_str()) {
                    linter_version = Some(version.to_string());
                }

                // Extract config used
                if let Some(config) = lint_obj.get("config_used").and_then(|v| v.as_str()) {
                    config_used = Some(config.to_string());
                }
            }
        }

        // Also check errors array for linting errors
        if errors == 0 && warnings == 0 {
            // Try to infer from error messages
            for error_msg in &worker_result.errors {
                let error_lower = error_msg.to_lowercase();
                if error_lower.contains("lint") || error_lower.contains("clippy") || error_lower.contains("eslint") {
                    errors += 1;
                } else if error_lower.contains("warning") {
                    warnings += 1;
                }
            }
        }

        // Fallback: if no structured data, infer from success status
        if errors == 0 && warnings == 0 && info == 0 {
            if !worker_result.success {
                errors = 1;
            }
        }

        Ok(agent_agency_contracts::execution_artifacts::LintingResults {
            total_issues: errors + warnings + info,
            errors,
            warnings,
            info,
            issues_by_file,
            linter_version,
            config_used,
        })
    }

    /// Build provenance from worker result
    fn build_provenance(
        &self,
        worker_result: &WorkerTaskResult,
        milestone: &Milestone,
        worktree_path: &PathBuf,
    ) -> Result<agent_agency_contracts::execution_artifacts::Provenance> {
        let execution_id = Uuid::new_v4();
        let started_at = chrono::Utc::now() - chrono::Duration::milliseconds(worker_result.execution_time_ms as i64);
        let completed_at = Some(chrono::Utc::now());

        // Extract git information from worktree
        let mut commit_hash = "unknown".to_string();
        let mut branch = format!("worktree_{}", milestone.id);
        let mut dirty = false;
        let mut uncommitted_changes = Vec::new();

        if worktree_path.exists() {
            // Get current commit hash
            let commit_output = Command::new("git")
                .current_dir(worktree_path)
                .arg("rev-parse")
                .arg("HEAD")
                .output()
                .context("Failed to get git commit hash")?;

            if commit_output.status.success() {
                commit_hash = String::from_utf8_lossy(&commit_output.stdout).trim().to_string();
            }

            // Get current branch name
            let branch_output = Command::new("git")
                .current_dir(worktree_path)
                .arg("rev-parse")
                .arg("--abbrev-ref")
                .arg("HEAD")
                .output()
                .context("Failed to get git branch")?;

            if branch_output.status.success() {
                branch = String::from_utf8_lossy(&branch_output.stdout).trim().to_string();
            }

            // Check if worktree is dirty (has uncommitted changes)
            let status_output = Command::new("git")
                .current_dir(worktree_path)
                .arg("status")
                .arg("--porcelain")
                .output()
                .context("Failed to check git status")?;

            if status_output.status.success() {
                let status_text = String::from_utf8_lossy(&status_output.stdout);
                dirty = !status_text.trim().is_empty();
                
                // Extract uncommitted file changes
                for line in status_text.lines() {
                    if !line.trim().is_empty() {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            uncommitted_changes.push(parts[1].to_string());
                        }
                    }
                }
            }
        }

        // Extract worker version from metadata if available
        let worker_version = worker_result.metadata.get("worker_version")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Build environment information
        let environment = agent_agency_contracts::execution_artifacts::ExecutionEnvironment {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            rust_version: None, // Could be extracted from rustc --version if needed
            toolchain: worker_result.metadata.get("toolchain")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        };

        Ok(agent_agency_contracts::execution_artifacts::Provenance {
            execution_id,
            worker_id: worker_result.worker_breakdown.first()
                .map(|wb| wb.worker_id.to_string()),
            worker_version,
            started_at,
            completed_at,
            duration_ms: worker_result.execution_time_ms,
            environment,
            git_info: agent_agency_contracts::execution_artifacts::GitInfo {
                commit_hash,
                branch,
                dirty,
                uncommitted_changes,
            },
            seeds_used: agent_agency_contracts::execution_artifacts::ExecutionSeeds::default(),
            audit_trail: vec![
                agent_agency_contracts::execution_artifacts::AuditEvent {
                    timestamp: started_at,
                    event: "worker_execution_started".to_string(),
                    details: Some(serde_json::json!({
                        "milestone_id": milestone.id,
                        "worktree_path": worktree_path.display().to_string(),
                        "worker_id": worker_result.worker_breakdown.first().map(|wb| wb.worker_id.to_string()),
                    })),
                },
                agent_agency_contracts::execution_artifacts::AuditEvent {
                    timestamp: completed_at.unwrap_or(chrono::Utc::now()),
                    event: if worker_result.success {
                        "worker_execution_completed".to_string()
                    } else {
                        "worker_execution_failed".to_string()
                    },
                    details: Some(serde_json::json!({
                        "success": worker_result.success,
                        "summary": worker_result.summary,
                        "execution_time_ms": worker_result.execution_time_ms,
                        "quality_scores": worker_result.quality_scores,
                    })),
                },
            ],
        })
    }

    /// Convert parallel task to milestone (helper for parallel execution)
    fn parallel_task_to_milestone(
        &self,
        task: &SubTask,
    ) -> Result<Milestone> {
        // This is a simplified conversion - real implementation would map all fields
        Ok(Milestone {
            id: task.id.0.to_string(),
            objective: task.description.clone(),
            scope: agent_agency_contracts::planning_io::MilestoneScope {
                files: task.scope.files.clone(),
                directories: task.scope.directories.clone(),
                included_paths: task.scope.patterns.clone(),
                excluded_paths: Vec::new(),
                will_modify: true,
                allowed_operations: vec!["read".to_string(), "write".to_string()],
                parallelism: None,
                resource_requirements: std::collections::HashMap::new(),
            },
            interfaces: Vec::new(),
            tests: Vec::new(),
            evidence_gate: agent_agency_contracts::planning_io::EvidenceGate {
                min_coverage: 0.0,
                min_branch_coverage: 0.0,
                min_mutation_score: 0.0,
                security_scan_required: false,
                performance_budget: None,
                required_artifacts: Vec::new(),
                custom_validations: Vec::new(),
            },
            quality_gates: Vec::new(),
            dependencies: task.dependencies.iter().map(|d| d.0.to_string()).collect(),
            estimated_duration: Some(task.estimated_duration.as_secs() as u32 / 60),
            rollback_plan: String::new(),
            state: agent_agency_contracts::planning_io::MilestoneState::Ready,
            assigned_workers: vec![task.assigned_worker.map(|w| w.0).unwrap_or_else(Uuid::new_v4)],
            estimated_effort: task.estimated_effort as f64,
            priority: match task.priority {
                WorkerPriority::Low => agent_agency_contracts::planning_io::MilestonePriority::Low,
                WorkerPriority::Medium => agent_agency_contracts::planning_io::MilestonePriority::Normal,
                WorkerPriority::High => agent_agency_contracts::planning_io::MilestonePriority::High,
                WorkerPriority::Critical => agent_agency_contracts::planning_io::MilestonePriority::Critical,
            },
            risk_tier: 2,
            is_blocking: false,
            blocking_reason: None,
            metrics: None,
        })
    }
}

