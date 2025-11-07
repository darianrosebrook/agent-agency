//! Worker Execution Bridge
//!
//! Bridges agent-orchestration with agent-workers for task execution.
//! Converts between orchestrator types (Milestone, ExecutionArtifacts) and
//! worker types (TaskDefinition, TaskResult).

use std::path::PathBuf;
use std::sync::Arc;
use anyhow::{anyhow, Result};
use uuid::Uuid;
use tracing::{info, warn};

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
        // Extract diffs from worker result metadata
        let mut diffs = Vec::new();
        let mut new_files = Vec::new();
        let mut deleted_files = Vec::new();

        // Check metadata for code change information
        if let Some(diff_info) = worker_result.metadata.get("code_changes") {
            // Parse code changes from metadata
            // This is a simplified extraction - real implementation would parse actual diffs
            warn!("Code change extraction from worker metadata not fully implemented");
        }

        Ok(agent_agency_contracts::execution_artifacts::CodeChanges {
            diffs,
            new_files,
            deleted_files,
            statistics: agent_agency_contracts::execution_artifacts::CodeChangeStats {
                files_modified: 0,
                lines_added: 0,
                lines_removed: 0,
                total_loc: 0,
            },
        })
    }

    /// Extract test results from worker result
    fn extract_test_results(
        &self,
        worker_result: &WorkerTaskResult,
    ) -> Result<agent_agency_contracts::execution_artifacts::TestArtifacts> {
        // Extract test information from quality scores
        let total_tests = worker_result.quality_scores.get("tests_total")
            .copied()
            .unwrap_or(0.0) as u64 as u32;
        let passed_tests = worker_result.quality_scores.get("tests_passed")
            .copied()
            .unwrap_or(0.0) as u64 as u32;

        Ok(agent_agency_contracts::execution_artifacts::TestArtifacts {
            unit_tests: agent_agency_contracts::execution_artifacts::TestSuiteResults {
                total: total_tests,
                passed: passed_tests,
                failed: total_tests - passed_tests,
                skipped: 0,
                duration_ms: worker_result.execution_time_ms,
                results: Vec::new(),
            },
            integration_tests: agent_agency_contracts::execution_artifacts::TestSuiteResults::default(),
            e2e_tests: agent_agency_contracts::execution_artifacts::E2eTestResults::default(),
            test_files: Vec::new(),
        })
    }

    /// Extract coverage from worker result
    fn extract_coverage(
        &self,
        worker_result: &WorkerTaskResult,
    ) -> Result<agent_agency_contracts::execution_artifacts::CoverageResults> {
        // Extract coverage from quality scores
        let line_coverage = worker_result.quality_scores.get("coverage_line")
            .copied()
            .unwrap_or(0.0);
        let branch_coverage = worker_result.quality_scores.get("coverage_branch")
            .copied()
            .unwrap_or(0.0);

        Ok(agent_agency_contracts::execution_artifacts::CoverageResults {
            line_coverage,
            branch_coverage,
            function_coverage: 0.0,
            mutation_score: 0.0,
            coverage_report_path: None,
            uncovered_lines: Vec::new(),
            uncovered_branches: Vec::new(),
        })
    }

    /// Extract linting results from worker result
    fn extract_linting(
        &self,
        worker_result: &WorkerTaskResult,
    ) -> Result<agent_agency_contracts::execution_artifacts::LintingResults> {
        let errors = if worker_result.success { 0 } else { 1 };
        
        Ok(agent_agency_contracts::execution_artifacts::LintingResults {
            total_issues: errors,
            errors,
            warnings: 0,
            info: 0,
            issues_by_file: std::collections::HashMap::new(),
            linter_version: None,
            config_used: None,
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

        Ok(agent_agency_contracts::execution_artifacts::Provenance {
            execution_id,
            worker_id: worker_result.worker_breakdown.first()
                .map(|wb| wb.worker_id.to_string()),
            worker_version: None,
            started_at,
            completed_at,
            duration_ms: worker_result.execution_time_ms,
            environment: agent_agency_contracts::execution_artifacts::ExecutionEnvironment::default(),
            git_info: agent_agency_contracts::execution_artifacts::GitInfo {
                commit_hash: "unknown".to_string(),
                branch: format!("worktree_{}", milestone.id),
                dirty: false,
                uncommitted_changes: Vec::new(),
            },
            seeds_used: agent_agency_contracts::execution_artifacts::ExecutionSeeds::default(),
            audit_trail: vec![
                agent_agency_contracts::execution_artifacts::AuditEvent {
                    timestamp: started_at,
                    event: "worker_execution_started".to_string(),
                    details: Some(serde_json::json!({
                        "milestone_id": milestone.id,
                        "worktree_path": worktree_path.display().to_string(),
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

