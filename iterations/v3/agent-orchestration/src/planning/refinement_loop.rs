//! Refinement Loop Coordinator
//!
//! Coordinates iterative refinement cycles with council feedback,
//! quality tracking, and execution orchestration.
//!
//! @author @darianrosebrook

use anyhow::Result;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use agent_agency_contracts::final_verdict::FinalVerdictContract;
use agent_agency_contracts::types::prelude::*;
use agent_agency_contracts::ExecutionStatus;
use agent_agency_contracts::WorkingSpec;
use agent_evaluation::{EvaluationHook, EvaluationOrchestrator, StopReason};

/// Iteration record for tracking refinement history
#[derive(Debug, Clone)]
pub struct IterationRecord {
    /// Iteration number (1-indexed)
    pub iteration: u32,
    /// Timestamp of this iteration
    pub timestamp: chrono::DateTime<Utc>,
    /// Snapshot of the working spec at this iteration
    pub working_spec_snapshot: WorkingSpec,
    /// Quality score achieved in this iteration
    pub quality_score: f64,
    /// Whether the council approved this iteration
    pub council_approved: bool,
    /// Reason for refinement (from council feedback)
    pub refinement_reason: Option<String>,
    /// Detailed council feedback
    pub council_feedback: Option<String>,
    /// Artifacts produced in this iteration
    pub artifacts_produced: Vec<String>,
    /// Pre-execution review result (for iterations > 1)
    pub pre_execution_review: Option<PreExecutionReview>,
    /// Refinement actions applied in this iteration
    pub refinement_actions: Vec<RefinementActionRecord>,
    /// Quality improvement from previous iteration
    pub quality_delta: Option<f64>,
    /// Debate result if multiple solutions were debated
    pub debate_result: Option<crate::council::DebateResult>,
}

/// Pre-execution review result
#[derive(Debug, Clone)]
pub struct PreExecutionReview {
    /// Whether the pre-execution review approved
    pub approved: bool,
    /// Whether refinement was requested
    pub needs_refinement: bool,
    /// Reason for the decision
    pub reason: String,
    /// Whether refinement was applied before execution
    pub refinement_applied: bool,
}

/// Record of a refinement action
#[derive(Debug, Clone)]
pub struct RefinementActionRecord {
    /// Area that was refined
    pub area: String,
    /// Description of the refinement
    pub description: String,
    /// Whether the refinement was successful
    pub successful: bool,
    /// Timestamp of the action
    pub timestamp: chrono::DateTime<Utc>,
}

/// Result of refinement loop execution
#[derive(Debug, Clone)]
pub struct RefinementLoopResult {
    pub final_verdict: FinalVerdictContract,
    pub iterations: u32,
    pub quality_scores: Vec<f64>,
    pub iteration_history: Vec<IterationRecord>,
}

/// Trait for executing orchestration
#[async_trait::async_trait]
pub trait OrchestrationExecutor: Send + Sync {
    async fn execute_orchestration(
        &self,
        working_spec: &WorkingSpec,
        task_descriptor: &TaskDescriptor,
    ) -> Result<FinalVerdictContract>;
}

/// Trait for validating execution artifacts
#[async_trait::async_trait]
pub trait ArtifactValidator: Send + Sync {
    async fn validate_execution_artifacts(
        &self,
        verdict: &FinalVerdictContract,
        task_descriptor: &TaskDescriptor,
    ) -> Result<bool>;
}

/// Trait for council review
#[async_trait::async_trait]
pub trait CouncilReviewer: Send + Sync {
    async fn perform_council_review(
        &self,
        working_spec: &WorkingSpec,
        task_descriptor: &TaskDescriptor,
    ) -> Result<(bool, bool, String)>; // (approved, needs_refinement, reason)
}

/// Trait for refining working specs
#[async_trait::async_trait]
pub trait SpecRefiner: Send + Sync {
    async fn refine_working_spec(
        &self,
        current_spec: &WorkingSpec,
        refinement_reason: &str,
    ) -> Result<WorkingSpec>;
}

/// Trait for progress tracking
#[async_trait::async_trait]
pub trait ProgressTracker: Send + Sync {
    async fn update_task_progress(
        &self,
        task_id: Uuid,
        progress: f32,
        message: Option<String>,
    ) -> Result<()>;

    async fn update_task_status(
        &self,
        task_id: Uuid,
        status: ExecutionStatus,
        message: Option<String>,
    ) -> Result<()>;

    async fn track_iteration_progress(
        &self,
        task_id: Uuid,
        iteration: u32,
        quality_score: f64,
        improvement_delta: f64,
    ) -> Result<()>;

    async fn detect_and_report_plateaus(
        &self,
        task_id: Uuid,
        quality_scores: &[f64],
        iteration: u32,
    ) -> Result<()>;
}

/// Trait for state persistence
#[async_trait::async_trait]
pub trait StatePersistence: Send + Sync {
    async fn save_execution_state(&self, task_id: Uuid) -> Result<()>;
}

/// Trait for multi-solution debate coordination
#[async_trait::async_trait]
pub trait DebateCoordinator: Send + Sync {
    async fn coordinate_solution_debate(
        &self,
        solutions: Vec<crate::council::WorkerSolution>,
        review_context: crate::judge_backup::types::ReviewContext,
    ) -> Result<crate::council::DebateResult>;
}

/// Configuration for refinement loop
#[derive(Debug, Clone)]
pub struct RefinementLoopConfig {
    pub enable_council_review: bool,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
}

impl Default for RefinementLoopConfig {
    fn default() -> Self {
        Self {
            enable_council_review: true,
            max_retries: 3,
            retry_delay_ms: 1000,
        }
    }
}

/// Refinement loop coordinator
pub struct RefinementLoopCoordinator {
    config: RefinementLoopConfig,
    evaluation_orchestrator: EvaluationOrchestrator,
    evaluation_hook: Option<Arc<dyn EvaluationHook>>,
    execution_state: Arc<RwLock<HashMap<Uuid, RefinementState>>>,
}

/// Internal refinement state
#[derive(Debug, Clone)]
struct RefinementState {
    #[allow(dead_code)] // Reserved for future use
    task_id: Uuid,
    current_iteration: u32,
    quality_scores: Vec<f64>,
    iteration_history: Vec<IterationRecord>,
}

impl RefinementLoopCoordinator {
    /// Create a new refinement loop coordinator
    pub fn new(
        config: RefinementLoopConfig,
        evaluation_orchestrator: EvaluationOrchestrator,
        evaluation_hook: Option<Arc<dyn EvaluationHook>>,
    ) -> Self {
        Self {
            config,
            evaluation_orchestrator,
            evaluation_hook,
            execution_state: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Execute refinement loop for a task
    pub async fn execute_refinement_loop(
        &self,
        task_id: Uuid,
        initial_spec: WorkingSpec,
        task_descriptor: &TaskDescriptor,
        executor: Arc<dyn OrchestrationExecutor>,
        validator: Arc<dyn ArtifactValidator>,
        council_reviewer: Option<Arc<dyn CouncilReviewer>>,
        spec_refiner: Option<Arc<dyn SpecRefiner>>,
        progress_tracker: Arc<dyn ProgressTracker>,
        state_persistence: Option<Arc<dyn StatePersistence>>,
    ) -> Result<RefinementLoopResult> {
        // Initialize state
        {
            let mut state = self.execution_state.write().await;
            state.insert(
                task_id,
                RefinementState {
                    task_id,
                    current_iteration: 0,
                    quality_scores: Vec::new(),
                    iteration_history: Vec::new(),
                },
            );
        }

        let mut current_spec = initial_spec;
        let mut iteration = 0u32;
        let mut final_verdict = None;
        let mut quality_scores: Vec<f64> = Vec::new();

        loop {
            iteration += 1;
            tracing::info!("Refinement iteration {} for task {}", iteration, task_id);

            // Call evaluation hook before iteration
            if let Some(ref hook) = self.evaluation_hook {
                if let Err(e) = hook.before_iteration(iteration).await {
                    tracing::warn!(
                        "Evaluation hook error before iteration {}: {}",
                        iteration,
                        e
                    );
                }
            }

            // Check iteration limit
            if self
                .evaluation_orchestrator
                .is_iteration_limit_reached(iteration)
            {
                tracing::warn!(
                    "Max refinement iterations ({}) reached for task {}",
                    self.evaluation_orchestrator.config().max_iterations,
                    task_id
                );
                progress_tracker
                    .update_task_status(
                        task_id,
                        ExecutionStatus::Failed,
                        Some(format!(
                            "Max refinement iterations ({}) reached",
                            self.evaluation_orchestrator.config().max_iterations
                        )),
                    )
                    .await?;
                return Err(anyhow::anyhow!(
                    "Max refinement iterations ({}) reached",
                    self.evaluation_orchestrator.config().max_iterations
                ));
            }

            // Pre-execution council review (for iterations > 1, review refined spec before execution)
            // This ensures the council approves the refined spec before we execute it
            if self.config.enable_council_review && iteration > 1 {
                if let Some(ref council) = council_reviewer {
                    tracing::info!(
                        "Pre-execution council review for iteration {} of task {}",
                        iteration,
                        task_id
                    );

                    match council
                        .perform_council_review(&current_spec, task_descriptor)
                        .await
                    {
                        Ok((pre_approved, pre_needs_refinement, pre_reason)) => {
                            // Update progress with pre-execution review status
                            progress_tracker
                                .update_task_progress(
                                    task_id,
                                    50.0 + (iteration as f32 * 3.0),
                                    Some(format!(
                                        "Pre-execution council review: approved={}, needs_refinement={}",
                                        pre_approved, pre_needs_refinement
                                    )),
                                )
                                .await?;

                            if !pre_approved && pre_needs_refinement {
                                // Council wants more refinement before execution
                                tracing::info!(
                                    "Pre-execution review requests further refinement: {}",
                                    pre_reason
                                );

                                // Refine before executing
                                if let Some(ref refiner) = spec_refiner {
                                    match refiner
                                        .refine_working_spec(&current_spec, &pre_reason)
                                        .await
                                    {
                                        Ok(refined_spec) => {
                                            current_spec = refined_spec;
                                            tracing::info!(
                                                "Pre-execution refinement applied for iteration {}",
                                                iteration
                                            );
                                            // Continue to next iteration without executing
                                            continue;
                                        }
                                        Err(e) => {
                                            tracing::warn!(
                                                "Pre-execution refinement failed, proceeding with current spec: {}",
                                                e
                                            );
                                            // Fall through to execution with current spec
                                        }
                                    }
                                } else {
                                    tracing::warn!(
                                        "Pre-execution refinement requested but no spec refiner available"
                                    );
                                    // Fall through to execution with current spec
                                }
                            } else if !pre_approved && !pre_needs_refinement {
                                // Council rejected without refinement option
                                tracing::warn!(
                                    "Pre-execution review rejected spec: {}",
                                    pre_reason
                                );
                                progress_tracker
                                    .update_task_status(
                                        task_id,
                                        ExecutionStatus::Failed,
                                        Some(format!("Pre-execution review rejected: {}", pre_reason)),
                                    )
                                    .await?;
                                return Err(anyhow::anyhow!(
                                    "Pre-execution council review rejected: {}",
                                    pre_reason
                                ));
                            }
                            // If pre_approved, continue to execution
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Pre-execution council review failed, proceeding with execution: {}",
                                e
                            );
                            // Continue to execution despite review failure
                        }
                    }
                }
            }

            // Execute task orchestration for this iteration
            let verdict = match task_descriptor.execution_mode {
                ExecutionMode::DryRun => {
                    tracing::info!(
                        "Dry-run mode: Skipping actual orchestration, simulating results"
                    );
                    // Create a mock verdict for dry-run
                    FinalVerdictContract {
                        decision: agent_agency_contracts::final_verdict::FinalDecision::Accept,
                        votes: vec![],
                        dissent: String::new(),
                        remediation: vec![],
                        constitutional_refs: vec![],
                        verification_summary:
                            agent_agency_contracts::final_verdict::VerificationSummary {
                                claims_total: 1,
                                claims_verified: 1,
                                coverage_pct: 100.0,
                            },
                    }
                }
                ExecutionMode::Strict | ExecutionMode::Auto => {
                    // Execute with error recovery and retry logic
                    let mut attempts = 0;
                    let mut last_error = None;

                    loop {
                        attempts += 1;
                        match executor
                            .execute_orchestration(&current_spec, task_descriptor)
                            .await
                        {
                            Ok(verdict) => {
                                // Validate artifacts were collected
                                match validator
                                    .validate_execution_artifacts(&verdict, task_descriptor)
                                    .await
                                {
                                    Ok(valid) => {
                                        if valid {
                                            break verdict;
                                        } else {
                                            tracing::warn!("Artifact validation failed for task {}, retrying...", task_id);
                                            if attempts >= self.config.max_retries {
                                                return Err(anyhow::anyhow!(
                                                    "Artifact validation failed after retries"
                                                ));
                                            }
                                            continue;
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!(
                                            "Artifact validation error for task {}: {}",
                                            task_id,
                                            e
                                        );
                                        if attempts >= self.config.max_retries {
                                            return Err(anyhow::anyhow!(
                                                "Artifact validation failed: {}",
                                                e
                                            ));
                                        }
                                        // Error logged above, continue retry loop
                                        continue;
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "Execution attempt {} failed for task {}: {}",
                                    attempts,
                                    task_id,
                                    e
                                );
                                last_error = Some(e.to_string());
                                if attempts >= self.config.max_retries {
                                    return Err(anyhow::anyhow!(
                                        "Execution failed after {} attempts: {}",
                                        self.config.max_retries,
                                        last_error.unwrap_or_else(|| "Unknown error".to_string())
                                    ));
                                }
                                // Wait before retry (exponential backoff)
                                let delay_ms = self.config.retry_delay_ms * (attempts as u64);
                                tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms))
                                    .await;
                                continue;
                            }
                        }
                    }
                }
            };

            // Calculate quality score from verdict
            let quality_score = self
                .evaluation_orchestrator
                .calculate_quality_score(&verdict);
            quality_scores.push(quality_score);
            tracing::info!(
                "Quality score for iteration {}: {:.3}",
                iteration,
                quality_score
            );

            // Track iteration progress metrics
            let improvement_delta = if iteration >= 2 {
                let previous_score = quality_scores[quality_scores.len() - 2];
                quality_score - previous_score
            } else {
                0.0
            };

            // Track iteration metrics
            if let Err(e) = progress_tracker
                .track_iteration_progress(task_id, iteration, quality_score, improvement_delta)
                .await
            {
                tracing::warn!("Failed to track iteration progress: {}", e);
            }

            // Detect quality plateaus
            if iteration >= 3 {
                if let Err(e) = progress_tracker
                    .detect_and_report_plateaus(task_id, &quality_scores, iteration)
                    .await
                {
                    tracing::warn!("Failed to detect plateaus: {}", e);
                }
            }

            // Evaluate iteration
            let verdict_arc = Arc::new(verdict.clone());
            let council_approved = false; // Will be updated after council review
            let evaluation = self
                .evaluation_orchestrator
                .evaluate_iteration(
                    iteration,
                    quality_score,
                    &quality_scores,
                    (*verdict_arc).clone(),
                    council_approved,
                )
                .await;

            // Call evaluation hook after iteration
            if let Some(ref hook) = self.evaluation_hook {
                if let Err(e) = hook.after_iteration(&evaluation).await {
                    tracing::warn!("Evaluation hook error after iteration {}: {}", iteration, e);
                }
            }

            // Record iteration history
            // Calculate quality delta from previous iteration
            let quality_delta = if iteration > 1 && !quality_scores.is_empty() {
                let prev_score = quality_scores[quality_scores.len() - 1];
                Some(quality_score - prev_score)
            } else {
                None
            };

            let iteration_record = IterationRecord {
                iteration,
                timestamp: Utc::now(),
                working_spec_snapshot: current_spec.clone(),
                quality_score,
                council_approved: false,
                refinement_reason: None,
                council_feedback: None,
                artifacts_produced: vec![],
                pre_execution_review: None,
                refinement_actions: vec![],
                quality_delta,
                debate_result: None, // Will be populated if debate occurred
            };

            // Update execution state with iteration data
            {
                let mut state = self.execution_state.write().await;
                if let Some(ref mut s) = state.get_mut(&task_id) {
                    s.current_iteration = iteration;
                    s.quality_scores = quality_scores.clone();
                    s.iteration_history.push(iteration_record.clone());
                }
            }

            // Save execution state after each iteration
            if let Some(ref persistence) = state_persistence {
                if let Err(e) = persistence.save_execution_state(task_id).await {
                    tracing::warn!(
                        "Failed to save execution state after iteration {}: {}",
                        iteration,
                        e
                    );
                }
            }

            // Use evaluation orchestrator to determine if we should continue
            if !evaluation.should_continue {
                if let Some(ref reason) = evaluation.stop_reason {
                    tracing::info!("Stopping refinement due to: {:?}", reason);

                    // Call evaluation hook on stop
                    if let Some(ref hook) = self.evaluation_hook {
                        if let Err(e) = hook.on_stop(reason, quality_score).await {
                            tracing::warn!("Evaluation hook error on stop: {}", e);
                        }
                    }

                    final_verdict = Some(verdict);
                    break;
                }
            }

            // Post-execution council review to check if refinement is needed
            if self.config.enable_council_review {
                if let Some(ref council) = council_reviewer {
                    match council
                        .perform_council_review(&current_spec, task_descriptor)
                        .await
                    {
                        Ok((approved, needs_refinement, refinement_reason)) => {
                            // Update iteration record with council feedback
                            {
                                let mut state = self.execution_state.write().await;
                                if let Some(s) = state.get_mut(&task_id) {
                                    if let Some(last_record) = s.iteration_history.last_mut() {
                                        last_record.council_approved = approved;
                                        last_record.refinement_reason =
                                            Some(refinement_reason.clone());
                                        last_record.council_feedback = Some(format!(
                                            "Approved: {}, Needs refinement: {}",
                                            approved, needs_refinement
                                        ));
                                    }
                                }
                            }

                            if approved {
                                // Council approves - exit refinement loop
                                tracing::info!(
                                    "Task {} approved by council after iteration {}",
                                    task_id,
                                    iteration
                                );

                                // Re-evaluate with council approval
                                let evaluation_with_approval = self
                                    .evaluation_orchestrator
                                    .evaluate_iteration(
                                        iteration,
                                        quality_score,
                                        &quality_scores,
                                        (*verdict_arc).clone(),
                                        true, // Council approved
                                    )
                                    .await;

                                // Call evaluation hook on stop
                                if let Some(ref hook) = self.evaluation_hook {
                                    if let Some(ref reason) = evaluation_with_approval.stop_reason {
                                        if let Err(e) = hook.on_stop(reason, quality_score).await {
                                            tracing::warn!("Evaluation hook error on stop: {}", e);
                                        }
                                    }
                                }

                                final_verdict = Some(verdict);
                                break;
                            } else if needs_refinement
                                && iteration < self.evaluation_orchestrator.config().max_iterations
                            {
                                // Council requests refinement - refine and continue
                                tracing::info!(
                                    "Task {} requires refinement after iteration {}: {}",
                                    task_id,
                                    iteration,
                                    refinement_reason
                                );
                                progress_tracker
                                    .update_task_progress(
                                        task_id,
                                        60.0 + (iteration as f32 * 5.0),
                                        Some(format!(
                                            "Refining based on council feedback: {}",
                                            refinement_reason
                                        )),
                                    )
                                    .await?;

                                // Refine working spec based on feedback
                                if let Some(ref refiner) = spec_refiner {
                                    match refiner
                                        .refine_working_spec(&current_spec, &refinement_reason)
                                        .await
                                    {
                                        Ok(refined_spec) => {
                                            current_spec = refined_spec;
                                            tracing::info!(
                                                "Working spec refined for iteration {}",
                                                iteration + 1
                                            );
                                            continue; // Continue to next iteration
                                        }
                                        Err(e) => {
                                            tracing::error!("Failed to refine working spec: {}", e);
                                            return Err(anyhow::anyhow!(
                                                "Refinement failed: {}",
                                                e
                                            ));
                                        }
                                    }
                                } else {
                                    tracing::warn!(
                                        "Refinement requested but no spec refiner available"
                                    );
                                    final_verdict = Some(verdict);
                                    break;
                                }
                            } else {
                                // Council rejected and no refinement possible
                                tracing::warn!(
                                    "Task {} rejected by council after iteration {}",
                                    task_id,
                                    iteration
                                );

                                // Call evaluation hook on stop
                                if let Some(ref hook) = self.evaluation_hook {
                                    if let Err(e) = hook
                                        .on_stop(&StopReason::CouncilRejected, quality_score)
                                        .await
                                    {
                                        tracing::warn!("Evaluation hook error on stop: {}", e);
                                    }
                                }

                                progress_tracker
                                    .update_task_status(
                                        task_id,
                                        ExecutionStatus::Failed,
                                        Some("Council rejected task after refinement".to_string()),
                                    )
                                    .await?;
                                return Err(anyhow::anyhow!(
                                    "Task rejected by council after refinement attempts"
                                ));
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Post-execution council review failed, accepting current verdict: {}", e);
                            final_verdict = Some(verdict);
                            break; // Accept current verdict if review fails
                        }
                    }
                } else {
                    // No council reviewer available - accept verdict after first execution
                    final_verdict = Some(verdict);
                    break;
                }
            } else {
                // No council review - accept verdict after first execution
                final_verdict = Some(verdict);
                break;
            }
        }

        let final_verdict_value =
            final_verdict.expect("Final verdict should be set after refinement loop");

        // Get final state
        let state = self.execution_state.read().await;
        let refinement_state = state
            .get(&task_id)
            .ok_or_else(|| anyhow::anyhow!("Refinement state not found for task {}", task_id))?;

        Ok(RefinementLoopResult {
            final_verdict: final_verdict_value,
            iterations: iteration,
            quality_scores: refinement_state.quality_scores.clone(),
            iteration_history: refinement_state.iteration_history.clone(),
        })
    }

    /// Handle debate between multiple competing solutions from different workers
    pub async fn debate_multiple_solutions(
        &self,
        solutions: Vec<crate::council::WorkerSolution>,
        task_descriptor: &TaskDescriptor,
        debate_coordinator: Arc<dyn DebateCoordinator>,
    ) -> Result<crate::council::DebateResult> {
        if solutions.len() <= 1 {
            return Err(anyhow::anyhow!(
                "Debate requires multiple solutions, got {}",
                solutions.len()
            ));
        }

        tracing::info!(
            "Conducting debate between {} competing solutions for task {}",
            solutions.len(),
            task_descriptor.task_id
        );

        // Create a minimal working spec for debate context
        use agent_agency_contracts::*;
        let risk_tier_value = match task_descriptor.risk_tier.clone().unwrap_or(task_request::RiskTier::Tier2) {
            task_request::RiskTier::Tier1 => 1,
            task_request::RiskTier::Tier2 => 2,
            task_request::RiskTier::Tier3 => 3,
        };

        let working_spec = WorkingSpec {
            version: "1.0".to_string(),
            id: format!("debate_{}", task_descriptor.task_id),
            title: "Debate Working Spec".to_string(),
            description: task_descriptor.description.clone(),
            goals: vec!["Resolve solution conflicts through debate".to_string()],
            risk_tier: risk_tier_value,
            constraints: working_spec::WorkingSpecConstraints {
                max_duration_minutes: None,
                max_iterations: None,
                budget_limits: None,
                scope_restrictions: None,
            },
            acceptance_criteria: vec![],
            test_plan: TestPlan {
                unit_tests: vec![],
                integration_tests: vec![],
                e2e_scenarios: vec![],
                coverage_targets: None,
            },
            rollback_plan: RollbackPlan::default(),
            context: WorkingSpecContext {
                workspace_root: ".".to_string(),
                git_branch: "main".to_string(),
                recent_changes: vec![],
                dependencies: std::collections::HashMap::new(),
                environment: task_request::Environment::Development,
            },
            non_functional_requirements: None,
            validation_results: None,
            quality_gates: None,
            scope: vec![],
            metadata: None,
            milestones: vec![],
            change_budget: task_descriptor.change_budget.clone(),
            file_changes: vec![],
            coverage_targets: None,
            overview: "Debate coordination working spec".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Create review context for debate
        let review_context = crate::judge_backup::types::ReviewContext {
            session_id: format!("debate_{}", task_descriptor.task_id),
            working_spec: serde_json::to_string(&working_spec)
                .map_err(|e| anyhow::anyhow!("Failed to serialize working spec: {}", e))?,
            risk_tier: working_spec.risk_tier as u8,
            previous_reviews: vec![],
            constraints: std::collections::HashMap::new(),
        };

        // Conduct multi-turn debate
        let debate_result = debate_coordinator
            .coordinate_solution_debate(solutions, review_context)
            .await?;

        tracing::info!(
            "Debate completed: winner={} (score={:.3}, confidence={:.2}) after {} rounds",
            debate_result.winner_solution_id,
            debate_result.winning_score,
            debate_result.confidence,
            debate_result.rounds.len()
        );

        Ok(debate_result)
    }

    /// Check if multiple solutions warrant a debate based on conflict analysis
    pub fn should_debate_solutions(&self, solutions: &[crate::council::WorkerSolution]) -> bool {
        if solutions.len() < 2 {
            return false;
        }

        // Debate if solutions have significantly different approaches
        // This is a simple heuristic - could be enhanced with more sophisticated conflict detection
        let titles: Vec<&str> = solutions
            .iter()
            .map(|s| s.working_spec.title.as_str())
            .collect();

        // Check for different implementation approaches in titles
        let has_different_approaches = titles
            .windows(2)
            .any(|window| !window[0].to_lowercase().contains(&window[1].to_lowercase())
                      && !window[1].to_lowercase().contains(&window[0].to_lowercase()));

        has_different_approaches
    }
}
