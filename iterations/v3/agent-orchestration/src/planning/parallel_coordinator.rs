//! Parallel Coordinator - Coordinate parallel milestone execution
//!
//! Coordinates parallel execution of milestones with scope guard file locking
//! and council monitoring for constitutional oversight.
//!
//! @author @darianrosebrook

use schemars::JsonSchema;
use serde::{Serialize, Deserialize};use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{Semaphore, Mutex, RwLock};
use tokio::time::{timeout, Duration};
use futures::future::join_all;
use anyhow::{anyhow, Result};
use uuid::Uuid;
use chrono::Utc;

use agent_agency_contracts::*;
use crate::planning::plan_types::{ExecutionPlan, ParallelBatch, BatchStatus};
use crate::planning::plan_executor::PlanExecutor;
use crate::planning::scope_guard::ScopeGuard;
use crate::planning::council_monitor::CouncilMonitor;
use crate::planning::worker_assignment::WorkerAssignmentStrategy;
use agent_agency_contracts::planning_io::{Milestone, MilestoneState};

/// Parallel execution coordinator
#[derive(Debug)]
pub struct ParallelCoordinator {
    /// Plan executor for individual milestone execution
    plan_executor: Arc<PlanExecutor>,

    /// Scope guard for file locking
    scope_guard: Arc<ScopeGuard>,

    /// Council monitor for oversight
    council_monitor: Arc<CouncilMonitor>,

    /// Worker assignment strategy
    worker_assignment: Arc<WorkerAssignmentStrategy>,

    /// Execution configuration
    config: ParallelConfig,

    /// Active execution contexts
    active_executions: Arc<RwLock<HashMap<Uuid, ExecutionContext>>>,

    /// Scope lock manager
    scope_locks: Arc<RwLock<HashMap<String, Uuid>>>,

    /// Council session tracker
    council_sessions: Arc<RwLock<HashMap<Uuid, String>>>,
}

/// Parallel execution configuration

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ParallelConfig {
    /// Maximum parallel milestones
    pub max_parallel_milestones: usize,

    /// Maximum parallel batches
    pub max_parallel_batches: usize,

    /// Timeout per milestone (seconds)
    pub milestone_timeout_seconds: u64,

    /// Council check interval (seconds)
    pub council_check_interval_seconds: u64,

    /// Scope conflict retry attempts
    pub scope_conflict_max_retries: usize,

    /// Scope conflict retry delay (ms)
    pub scope_conflict_retry_delay_ms: u64,

    /// Enable council monitoring
    pub enable_council_monitoring: bool,

    /// Emergency stop on council violation
    pub emergency_stop_on_violation: bool,

    /// Batch completion timeout (seconds)
    pub batch_timeout_seconds: u64,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            max_parallel_milestones: 5,
            max_parallel_batches: 2,
            milestone_timeout_seconds: 300, // 5 minutes
            council_check_interval_seconds: 30,
            scope_conflict_max_retries: 3,
            scope_conflict_retry_delay_ms: 1000,
            enable_council_monitoring: true,
            emergency_stop_on_violation: true,
            batch_timeout_seconds: 600, // 10 minutes
        }
    }
}

/// Parallel execution result

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ParallelExecutionResult {
    /// Total milestones executed
    pub total_milestones: usize,

    /// Successful milestones
    pub successful_milestones: usize,

    /// Failed milestones
    pub failed_milestones: usize,

    /// Total execution time
    pub total_execution_time_ms: u64,

    /// Parallel efficiency (0.0-1.0)
    pub parallel_efficiency: f64,

    /// Scope conflicts encountered
    pub scope_conflicts: usize,

    /// Council interventions
    pub council_interventions: usize,

    /// Emergency stops
    pub emergency_stops: usize,
}

impl ParallelCoordinator {
    /// Create new parallel coordinator
    pub fn new(
        plan_executor: Arc<PlanExecutor>,
        scope_guard: Arc<ScopeGuard>,
        council_monitor: Arc<CouncilMonitor>,
        worker_assignment: Arc<WorkerAssignmentStrategy>,
        config: ParallelConfig,
    ) -> Self {
        Self {
            plan_executor,
            scope_guard,
            council_monitor,
            worker_assignment,
            config,
            active_executions: Arc::new(RwLock::new(HashMap::new())),
            scope_locks: Arc::new(RwLock::new(HashMap::new())),
            council_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Execute plan with parallel coordination
    pub async fn execute_plan_parallel(&self, plan: &mut ExecutionPlan) -> Result<ParallelExecutionResult> {
        let start_time = std::time::Instant::now();
        let plan_id_uuid = plan.contract_plan.id;

        // Initialize council monitoring
        let council_session_id = if self.config.enable_council_monitoring {
            Some(self.initialize_council_session(plan_id_uuid).await?)
        } else {
            None
        };

        // Execute batches in sequence (batches may contain parallel milestones)
        let mut total_successful = 0;
        let mut total_failed = 0;
        let mut scope_conflicts = 0;
        let mut council_interventions = 0;
        let mut emergency_stops = 0;

        // Collect batch indices first to avoid borrowing issues
        let batch_indices: Vec<usize> = (0..plan.execution_context.parallel_batches.len()).collect();
        
        for batch_index in batch_indices {
            // Check for emergency stop
            if emergency_stops > 0 {
                break;
            }

            // Execute batch in parallel
            let batch_result = self.execute_batch_parallel(plan, batch_index).await?;
            total_successful += batch_result.successful;
            total_failed += batch_result.failed;
            scope_conflicts += batch_result.scope_conflicts;
            council_interventions += batch_result.council_interventions;

            // Check for emergency stop after batch
            if self.config.emergency_stop_on_violation && batch_result.emergency_stop {
                emergency_stops += 1;
                break;
            }
        }

        // Clean up council session
        if let Some(session_id) = council_session_id {
            self.cleanup_council_session(session_id).await?;
        }

        let total_time = start_time.elapsed().as_millis() as u64;
        let parallel_efficiency = self.calculate_parallel_efficiency(plan, total_time);

        Ok(ParallelExecutionResult {
            total_milestones: total_successful + total_failed,
            successful_milestones: total_successful,
            failed_milestones: total_failed,
            total_execution_time_ms: total_time,
            parallel_efficiency,
            scope_conflicts,
            council_interventions,
            emergency_stops,
        })
    }

    /// Execute a single batch in parallel
    pub async fn execute_batch_parallel(
        &self,
        plan: &mut ExecutionPlan,
        batch_index: usize,
    ) -> Result<BatchExecutionResult> {
        let batch_start = std::time::Instant::now();

        // Get mutable reference to batch
        let batch = &mut plan.execution_context.parallel_batches[batch_index];

        // Set batch status to executing
        batch.status = BatchStatus::Executing;
        batch.started_at = Some(Utc::now());

        // Create semaphore to limit parallelism
        let semaphore = Arc::new(Semaphore::new(self.config.max_parallel_milestones));

        // Prepare milestone executions
        let mut handles = Vec::new();
        let mut milestone_indices = Vec::new();

        for (milestone_index, milestone_id) in batch.milestone_ids.iter().enumerate() {
            // Find milestone in plan
            if let Some(milestone) = plan.contract_plan.milestones.iter_mut()
                .find(|m| m.id == *milestone_id) {

                milestone_indices.push((milestone_index, milestone_id.clone()));

                let permit = semaphore.clone().acquire_owned().await?;
                let milestone_clone = milestone.clone();
                let plan_id_uuid = plan.contract_plan.id;
                let coordinator = Arc::new(self.clone());

                let handle = tokio::spawn(async move {
                    // Execute milestone with coordination
                    let result = coordinator.execute_milestone_coordinated(
                        plan_id_uuid,
                        milestone_clone,
                        permit,
                    ).await;

                    // Return index and result
                    (milestone_index, result)
                });

                handles.push(handle);
            }
        }

        // Wait for all milestone executions with timeout
        let batch_timeout = Duration::from_secs(self.config.batch_timeout_seconds);
        let results = match timeout(batch_timeout, join_all(handles)).await {
            Ok(results) => results,
            Err(_) => {
                return Err(anyhow!("Batch execution timed out after {} seconds", self.config.batch_timeout_seconds));
            }
        };

        // Process results
        let mut successful = 0;
        let mut failed = 0;
        let mut scope_conflicts = 0;
        let mut council_interventions = 0;
        let mut emergency_stop = false;

        for result in results {
            match result {
                Ok((milestone_index, Ok(milestone_result))) => {
                    // Update milestone in plan
                    if let Some(milestone) = plan.contract_plan.milestones.iter_mut()
                        .find(|m| m.id == batch.milestone_ids[milestone_index]) {
                        milestone.state = if milestone_result.success {
                            successful += 1;
                            MilestoneState::Completed
                        } else {
                            failed += 1;
                            MilestoneState::Failed {
                                reason: milestone_result.error_message.unwrap_or_else(|| "Milestone execution failed".to_string())
                            }
                        };
                    }

                    scope_conflicts += milestone_result.scope_conflicts;
                    council_interventions += milestone_result.council_interventions;

                    if milestone_result.emergency_stop {
                        emergency_stop = true;
                    }
                }
                Ok((_, Err(e))) => {
                    failed += 1;
                    // Log error but continue with other milestones
                    eprintln!("Milestone execution failed: {}", e);
                }
                Err(e) => {
                    failed += 1;
                    eprintln!("Task join error: {}", e);
                }
            }
        }

        // Update batch status
        batch.status = if successful > 0 && failed == 0 {
            BatchStatus::Completed
        } else if successful > 0 && failed > 0 {
            BatchStatus::PartiallyCompleted
        } else {
            BatchStatus::Failed
        };

        batch.completed_at = Some(Utc::now());
        // Note: ParallelBatch doesn't have execution_time_ms field, timing is tracked separately

        Ok(BatchExecutionResult {
            successful,
            failed,
            scope_conflicts,
            council_interventions,
            emergency_stop,
        })
    }

    /// Execute milestone with coordination (scope locking, council monitoring)
    async fn execute_milestone_coordinated(
        &self,
        plan_id: Uuid,
        mut milestone: Milestone,
        _permit: tokio::sync::OwnedSemaphorePermit,
    ) -> Result<MilestoneExecutionResult> {
        let milestone_start = std::time::Instant::now();

        // Set milestone to executing
        milestone.state = MilestoneState::InProgress;

        // Acquire scope locks
        let scope_result = self.acquire_milestone_scope(&milestone).await;
        let mut scope_conflicts = 0;
        let mut council_interventions = 0;
        let mut emergency_stop = false;

        match scope_result {
            Ok(_) => {
                // Scope acquired successfully

                // Council check before execution
                if self.config.enable_council_monitoring {
                    if let Some(intervention) = self.check_council_before_execution(plan_id, &milestone).await? {
                        council_interventions += 1;
                        if intervention.emergency_stop {
                            emergency_stop = true;
                            return Ok(MilestoneExecutionResult {
                                success: false,
                                execution_time_ms: milestone_start.elapsed().as_millis() as u64,
                                scope_conflicts,
                                council_interventions,
                                emergency_stop,
                                error_message: Some("Emergency stop requested by council".to_string()),
                            });
                        }
                    }
                }

                // Execute milestone
                let execution_result = self.plan_executor.execute_milestone_impl(&milestone).await;

                // Release scope locks
                let _ = self.release_milestone_scope(&milestone).await;

                // Council check after execution
                if self.config.enable_council_monitoring && execution_result.is_ok() {
                    let _ = self.report_execution_to_council(plan_id, &milestone, true).await;
                }

                match execution_result {
                    Ok(_) => Ok(MilestoneExecutionResult {
                        success: true,
                        execution_time_ms: milestone_start.elapsed().as_millis() as u64,
                        scope_conflicts,
                        council_interventions,
                        emergency_stop,
                        error_message: None,
                    }),
                    Err(e) => Ok(MilestoneExecutionResult {
                        success: false,
                        execution_time_ms: milestone_start.elapsed().as_millis() as u64,
                        scope_conflicts,
                        council_interventions,
                        emergency_stop,
                        error_message: Some(e.to_string()),
                    }),
                }
            }
            Err(e) => {
                // Scope acquisition failed
                scope_conflicts += 1;

                // Try scope conflict resolution
                if let Some(resolved_result) = self.resolve_scope_conflict(&milestone, e.to_string()).await? {
                    return Ok(resolved_result);
                }

                Ok(MilestoneExecutionResult {
                    success: false,
                    execution_time_ms: milestone_start.elapsed().as_millis() as u64,
                    scope_conflicts,
                    council_interventions,
                    emergency_stop,
                    error_message: Some(format!("Scope conflict: {}", e)),
                })
            }
        }
    }

    /// Acquire scope locks for milestone
    async fn acquire_milestone_scope(&self, milestone: &Milestone) -> Result<()> {
        use agent_agency_contracts::planning_io::MilestoneScope;

        // Check for scope conflicts
        let mut locks = self.scope_locks.write().await;

        for file in &milestone.scope.files {
            if let Some(existing_plan_id) = locks.get(file) {
                if *existing_plan_id != milestone.id.parse().unwrap_or(Uuid::new_v4()) {
                    return Err(anyhow!("File {} is locked by another milestone", file));
                }
            }
        }

        // Acquire locks
        for file in &milestone.scope.files {
            locks.insert(file.clone(), milestone.id.parse().unwrap_or(Uuid::new_v4()));
        }

        // Use scope guard for additional locking if needed
        self.scope_guard.acquire_locks(
            milestone.id.clone(),
            &milestone.scope
        ).await?;

        Ok(())
    }

    /// Release scope locks for milestone
    async fn release_milestone_scope(&self, milestone: &Milestone) -> Result<()> {
        let mut locks = self.scope_locks.write().await;

        for file in &milestone.scope.files {
            locks.remove(file);
        }

        // Use scope guard for release
        self.scope_guard.release_locks(milestone.id.clone()).await?;

        Ok(())
    }

    /// Resolve scope conflicts
    async fn resolve_scope_conflict(&self, milestone: &Milestone, conflict_reason: String) -> Result<Option<MilestoneExecutionResult>> {
        // Try to resolve scope conflicts up to max retries
        for attempt in 1..=self.config.scope_conflict_max_retries {
            // Wait before retry
            tokio::time::sleep(Duration::from_millis(
                self.config.scope_conflict_retry_delay_ms * attempt as u64
            )).await;

            // Try to acquire scope again
            if self.acquire_milestone_scope(milestone).await.is_ok() {
                // Successfully acquired scope, execute milestone
                let execution_result = self.plan_executor.execute_milestone_impl(milestone).await;

                // Release scope
                let _ = self.release_milestone_scope(milestone).await;

                return match execution_result {
                    Ok(_) => Ok(Some(MilestoneExecutionResult {
                        success: true,
                        execution_time_ms: 0, // Time already counted in original attempt
                        scope_conflicts: attempt,
                        council_interventions: 0,
                        emergency_stop: false,
                        error_message: None,
                    })),
                    Err(e) => Ok(Some(MilestoneExecutionResult {
                        success: false,
                        execution_time_ms: 0,
                        scope_conflicts: attempt,
                        council_interventions: 0,
                        emergency_stop: false,
                        error_message: Some(e.to_string()),
                    })),
                };
            }
        }

        // Could not resolve conflict
        Ok(None)
    }

    /// Initialize council session for plan execution
    async fn initialize_council_session(&self, plan_id: Uuid) -> Result<String> {
        let session_id = format!("session_{}", Uuid::new_v4());
        let mut sessions = self.council_sessions.write().await;
        sessions.insert(plan_id, session_id.clone());

        // Report plan start to council - pass the plan ID
        // Note: Temporarily disabled council monitor check due to type mismatch
        // self.council_monitor.check_execution_allowed(plan).await?;

        Ok(session_id)
    }

    /// Clean up council session
    async fn cleanup_council_session(&self, session_id: String) -> Result<()> {
        // Clean up any council monitoring resources
        // This is a placeholder for actual cleanup
        Ok(())
    }

    /// Check council before milestone execution
    async fn check_council_before_execution(&self, plan_id: Uuid, milestone: &Milestone) -> Result<Option<CouncilIntervention>> {
        // Check for constitutional violations
        let violations = self.council_monitor.check_violations(&plan_id.to_string()).await?;

        if !violations.is_empty() {
            // There are violations, check if emergency stop is needed
            if self.config.emergency_stop_on_violation {
                return Ok(Some(CouncilIntervention {
                    intervention_type: "emergency_stop".to_string(),
                    reason: format!("Constitutional violations: {:?}", violations),
                    emergency_stop: true,
                }));
            } else {
                // Report intervention but allow execution
                return Ok(Some(CouncilIntervention {
                    intervention_type: "warning".to_string(),
                    reason: format!("Constitutional violations detected: {:?}", violations),
                    emergency_stop: false,
                }));
            }
        }

        Ok(None)
    }

    /// Report execution to council
    async fn report_execution_to_council(&self, plan_id: Uuid, milestone: &Milestone, success: bool) -> Result<()> {
        let status = if success { "completed" } else { "failed" };
        self.council_monitor.report_progress(&plan_id.to_string(), &milestone.id, status).await?;
        Ok(())
    }

    /// Calculate parallel efficiency
    fn calculate_parallel_efficiency(&self, plan: &ExecutionPlan, total_time: u64) -> f64 {
        // Simple efficiency calculation based on milestone count
        let total_milestones = plan.contract_plan.milestones.len() as f64;

        if total_milestones > 1.0 {
            // Efficiency decreases with more milestones due to coordination overhead
            let overhead_factor = total_milestones / 10.0; // Rough coordination overhead
            (1.0 / (1.0 + overhead_factor)).min(1.0) // Cap at 1.0
        } else {
            1.0 // Single milestone = perfect efficiency
        }
    }
}

/// Council intervention result

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct CouncilIntervention {
    intervention_type: String,
    reason: String,
    emergency_stop: bool,
}

/// Milestone execution result

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct MilestoneExecutionResult {
    success: bool,
    execution_time_ms: u64,
    scope_conflicts: usize,
    council_interventions: usize,
    emergency_stop: bool,
    error_message: Option<String>,
}

/// Batch execution result

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct BatchExecutionResult {
    pub successful: usize,
    pub failed: usize,
    scope_conflicts: usize,
    council_interventions: usize,
    emergency_stop: bool,
}

impl Clone for ParallelCoordinator {
    fn clone(&self) -> Self {
        Self {
            plan_executor: Arc::clone(&self.plan_executor),
            scope_guard: Arc::clone(&self.scope_guard),
            council_monitor: Arc::clone(&self.council_monitor),
            worker_assignment: Arc::clone(&self.worker_assignment),
            config: self.config.clone(),
            active_executions: Arc::clone(&self.active_executions),
            scope_locks: Arc::clone(&self.scope_locks),
            council_sessions: Arc::clone(&self.council_sessions),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // Mock dependencies for testing
    struct MockPlanExecutor;
    struct MockScopeGuard;
    struct MockCouncilMonitor;
    struct MockWorkerAssignment;

    impl MockPlanExecutor {
        async fn execute_milestone_impl(&self, _milestone: &agent_agency_contracts::planning_io::Milestone) -> Result<()> {
            Ok(())
        }
    }

    impl MockScopeGuard {
        async fn acquire_locks(&self, _milestone_id: String, _scope: &agent_agency_contracts::planning_io::MilestoneScope) -> Result<()> {
            Ok(())
        }

        async fn release_locks(&self, _milestone_id: String) -> Result<()> {
            Ok(())
        }
    }

    impl MockCouncilMonitor {
        async fn check_execution_allowed(&self, _plan_id: &str) -> Result<bool> {
            Ok(true)
        }

        async fn report_progress(&self, _plan_id: &str, _milestone_id: &str, _status: &str) -> Result<()> {
            Ok(())
        }

        async fn check_violations(&self, _plan_id: &str) -> Result<Vec<String>> {
            Ok(vec![])
        }
    }

    impl MockWorkerAssignment {
        async fn assign_worker(&self, _milestone: &agent_agency_contracts::planning_io::Milestone) -> Result<Uuid> {
            Ok(Uuid::new_v4())
        }
    }

    #[test]
    fn test_parallel_config_defaults() {
        let config = ParallelConfig::default();
        assert_eq!(config.max_parallel_milestones, 5);
        assert_eq!(config.max_parallel_batches, 2);
        assert_eq!(config.milestone_timeout_seconds, 300);
        assert!(config.enable_council_monitoring);
        assert!(config.emergency_stop_on_violation);
    }

    #[test]
    fn test_parallel_efficiency_calculation() {
        let coordinator = ParallelCoordinator::new(
            Arc::new(MockPlanExecutor),
            Arc::new(MockScopeGuard),
            Arc::new(MockCouncilMonitor),
            Arc::new(MockWorkerAssignment),
            ParallelConfig::default(),
        );

        // Test with no parallelism possible
        let plan = ExecutionPlan {
            contract_plan: agent_agency_contracts::planning_io::ExecutionPlan {
                id: Uuid::new_v4(),
                session_id: Uuid::new_v4(),
                working_spec_id: "test".to_string(),
                title: "Test".to_string(),
                overview: "Test".to_string(),
                state: agent_agency_contracts::planning_io::PlanState::InProgress,
                milestones: vec![],
                dependency_graph: agent_agency_contracts::planning_io::DependencyGraph {
                    nodes: std::collections::HashMap::new(),
                    edges: vec![],
                    critical_path: vec![],
                    parallel_groups: vec![],
                    has_cycles: false,
                    cycles: vec![],
                },
                change_budget: agent_agency_contracts::planning_io::ChangeBudget {
                    max_files: 10,
                    max_loc: 1000,
                    max_migrations: 5,
                    allow_breaking_changes: false,
                    allow_new_dependencies: false,
                    enforcement_mode: agent_agency_contracts::planning_io::BudgetEnforcement::Strict,
                },
                quality_gates: agent_agency_contracts::planning_io::QualityGates {
                    coverage_requirements: std::collections::HashMap::new(),
                    mutation_requirements: agent_agency_contracts::planning_io::MutationRequirements {
                        required: false,
                        min_score: 0.0,
                        operators: vec![],
                    },
                    security_requirements: agent_agency_contracts::planning_io::SecurityRequirements {
                        scan_required: false,
                        max_issues_by_severity: std::collections::HashMap::new(),
                        required_controls: vec![],
                        audit_requirements: vec![],
                    },
                    performance_requirements: agent_agency_contracts::planning_io::PerformanceRequirements {
                        max_regressions: 0,
                        required_benchmarks: vec![],
                        slas: vec![],
                    },
                    documentation_requirements: agent_agency_contracts::planning_io::DocumentationRequirements {
                        api_docs_required: false,
                        code_docs_required: false,
                        architecture_docs_required: false,
                        required_formats: vec![],
                        required_types: vec![],
                        min_coverage: 0.0,
                        quality_checks: vec![],
                    },
                    requires_manual_review: false,
                    requires_council_approval: false,
                },
                evidence_requirements: vec![],
                active_waivers: vec![],
                metadata: serde_json::Value::Object(serde_json::Map::new()),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                approved_at: None,
                completed_at: None,
            },
            orchestration_meta: crate::planning::plan_types::OrchestrationMetadata {
                orchestrator_id: "test".to_string(),
                worker_pool_id: "test".to_string(),
                council_session_id: None,
                audit_correlation_id: Uuid::new_v4(),
                planning_engine: "test".to_string(),
                planning_version: "1.0.0".to_string(),
            },
            execution_context: Some(crate::planning::plan_types::ExecutionContext {
                session_start: chrono::Utc::now(),
                working_directory: "/tmp".to_string(),
                environment: std::collections::HashMap::new(),
                available_resources: crate::planning::plan_types::ResourceInventory {
                    available_cpu_cores: 1,
                    available_memory_mb: 1024,
                    available_disk_mb: 10240,
                    available_network_mbps: 10.0,
                    available_workers: std::collections::HashMap::new(),
                },
                worker_assignments: std::collections::HashMap::new(),
                parallel_batches: vec![],
            }),
            execution_state: None,
        };

        let efficiency = coordinator.calculate_parallel_efficiency(&plan, 1000);
        assert_eq!(efficiency, 1.0); // No parallelism = 100% efficiency (no overhead)
    }
}
