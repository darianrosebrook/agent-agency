//! Refinement Loop Coordinator
//!
//! Coordinates iterative refinement cycles with council feedback,
//! quality tracking, and execution orchestration.
//!
//! @author @darianrosebrook

use std::sync::Arc;
use std::collections::HashMap;
use anyhow::Result;
use uuid::Uuid;
use chrono::Utc;
use tokio::sync::RwLock;

use agent_agency_contracts::WorkingSpec;
use agent_agency_contracts::types::prelude::*;
use agent_agency_contracts::final_verdict::FinalVerdictContract;
use agent_agency_contracts::ExecutionStatus;
use agent_evaluation::{EvaluationOrchestrator, EvaluationHook, StopReason};

/// Iteration record for tracking refinement history
#[derive(Debug, Clone)]
pub struct IterationRecord {
    pub iteration: u32,
    pub timestamp: chrono::DateTime<Utc>,
    pub working_spec_snapshot: WorkingSpec,
    pub quality_score: f64,
    pub council_approved: bool,
    pub refinement_reason: Option<String>,
    pub council_feedback: Option<String>,
    pub artifacts_produced: Vec<String>,
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
            state.insert(task_id, RefinementState {
                task_id,
                current_iteration: 0,
                quality_scores: Vec::new(),
                iteration_history: Vec::new(),
            });
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
                    tracing::warn!("Evaluation hook error before iteration {}: {}", iteration, e);
                }
            }

            // Check iteration limit
            if self.evaluation_orchestrator.is_iteration_limit_reached(iteration) {
                tracing::warn!("Max refinement iterations ({}) reached for task {}", 
                    self.evaluation_orchestrator.config().max_iterations, task_id);
                progress_tracker.update_task_status(
                    task_id,
                    ExecutionStatus::Failed,
                    Some(format!("Max refinement iterations ({}) reached", 
                        self.evaluation_orchestrator.config().max_iterations))
                ).await?;
                return Err(anyhow::anyhow!("Max refinement iterations ({}) reached", 
                    self.evaluation_orchestrator.config().max_iterations));
            }

            // Execute task orchestration for this iteration
            let verdict = match task_descriptor.execution_mode {
                ExecutionMode::DryRun => {
                    tracing::info!("Dry-run mode: Skipping actual orchestration, simulating results");
                    // Create a mock verdict for dry-run
                    FinalVerdictContract {
                        decision: agent_agency_contracts::final_verdict::FinalDecision::Accept,
                        votes: vec![],
                        dissent: String::new(),
                        remediation: vec![],
                        constitutional_refs: vec![],
                        verification_summary: agent_agency_contracts::final_verdict::VerificationSummary {
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
                        match executor.execute_orchestration(&current_spec, task_descriptor).await {
                            Ok(verdict) => {
                                // Validate artifacts were collected
                                match validator.validate_execution_artifacts(&verdict, task_descriptor).await {
                                    Ok(valid) => {
                                        if valid {
                                            break verdict;
                                        } else {
                                            tracing::warn!("Artifact validation failed for task {}, retrying...", task_id);
                                            if attempts >= self.config.max_retries {
                                                return Err(anyhow::anyhow!("Artifact validation failed after retries"));
                                            }
                                            continue;
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!("Artifact validation error for task {}: {}", task_id, e);
                                        if attempts >= self.config.max_retries {
                                            return Err(anyhow::anyhow!("Artifact validation failed: {}", e));
                                        }
                                        last_error = Some(e.to_string());
                                        continue;
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::warn!("Execution attempt {} failed for task {}: {}", attempts, task_id, e);
                                last_error = Some(e.to_string());
                                if attempts >= self.config.max_retries {
                                    return Err(anyhow::anyhow!("Execution failed after {} attempts: {}", 
                                        self.config.max_retries, 
                                        last_error.unwrap_or_else(|| "Unknown error".to_string())));
                                }
                                // Wait before retry (exponential backoff)
                                let delay_ms = self.config.retry_delay_ms * (attempts as u64);
                                tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                                continue;
                            }
                        }
                    }
                }
            };

            // Calculate quality score from verdict
            let quality_score = self.evaluation_orchestrator.calculate_quality_score(&verdict);
            quality_scores.push(quality_score);
            tracing::info!("Quality score for iteration {}: {:.3}", iteration, quality_score);

            // Track iteration progress metrics
            let improvement_delta = if iteration >= 2 {
                let previous_score = quality_scores[quality_scores.len() - 2];
                quality_score - previous_score
            } else {
                0.0
            };

            // Track iteration metrics
            if let Err(e) = progress_tracker.track_iteration_progress(
                task_id, iteration, quality_score, improvement_delta
            ).await {
                tracing::warn!("Failed to track iteration progress: {}", e);
            }

            // Detect quality plateaus
            if iteration >= 3 {
                if let Err(e) = progress_tracker.detect_and_report_plateaus(
                    task_id, &quality_scores, iteration
                ).await {
                    tracing::warn!("Failed to detect plateaus: {}", e);
                }
            }

            // Evaluate iteration
            let verdict_arc = Arc::new(verdict.clone());
            let council_approved = false; // Will be updated after council review
            let evaluation = self.evaluation_orchestrator.evaluate_iteration(
                iteration,
                quality_score,
                &quality_scores,
                (*verdict_arc).clone(),
                council_approved,
            ).await;

            // Call evaluation hook after iteration
            if let Some(ref hook) = self.evaluation_hook {
                if let Err(e) = hook.after_iteration(&evaluation).await {
                    tracing::warn!("Evaluation hook error after iteration {}: {}", iteration, e);
                }
            }

            // Record iteration history
            let iteration_record = IterationRecord {
                iteration,
                timestamp: Utc::now(),
                working_spec_snapshot: current_spec.clone(),
                quality_score,
                council_approved: false,
                refinement_reason: None,
                council_feedback: None,
                artifacts_produced: vec![],
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
                    tracing::warn!("Failed to save execution state after iteration {}: {}", iteration, e);
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
                    match council.perform_council_review(&current_spec, task_descriptor).await {
                        Ok((approved, needs_refinement, refinement_reason)) => {
                            // Update iteration record with council feedback
                            {
                                let mut state = self.execution_state.write().await;
                                if let Some(s) = state.get_mut(&task_id) {
                                    if let Some(last_record) = s.iteration_history.last_mut() {
                                        last_record.council_approved = approved;
                                        last_record.refinement_reason = Some(refinement_reason.clone());
                                        last_record.council_feedback = Some(format!("Approved: {}, Needs refinement: {}", approved, needs_refinement));
                                    }
                                }
                            }

                            if approved {
                                // Council approves - exit refinement loop
                                tracing::info!("Task {} approved by council after iteration {}", task_id, iteration);
                                
                                // Re-evaluate with council approval
                                let evaluation_with_approval = self.evaluation_orchestrator.evaluate_iteration(
                                    iteration,
                                    quality_score,
                                    &quality_scores,
                                    (*verdict_arc).clone(),
                                    true, // Council approved
                                ).await;
                                
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
                            } else if needs_refinement && iteration < self.evaluation_orchestrator.config().max_iterations {
                                // Council requests refinement - refine and continue
                                tracing::info!("Task {} requires refinement after iteration {}: {}", task_id, iteration, refinement_reason);
                                progress_tracker.update_task_progress(
                                    task_id,
                                    60.0 + (iteration as f32 * 5.0),
                                    Some(format!("Refining based on council feedback: {}", refinement_reason))
                                ).await?;
                                
                                // Refine working spec based on feedback
                                if let Some(ref refiner) = spec_refiner {
                                    match refiner.refine_working_spec(&current_spec, &refinement_reason).await {
                                        Ok(refined_spec) => {
                                            current_spec = refined_spec;
                                            tracing::info!("Working spec refined for iteration {}", iteration + 1);
                                            continue; // Continue to next iteration
                                        }
                                        Err(e) => {
                                            tracing::error!("Failed to refine working spec: {}", e);
                                            return Err(anyhow::anyhow!("Refinement failed: {}", e));
                                        }
                                    }
                                } else {
                                    tracing::warn!("Refinement requested but no spec refiner available");
                                    final_verdict = Some(verdict);
                                    break;
                                }
                            } else {
                                // Council rejected and no refinement possible
                                tracing::warn!("Task {} rejected by council after iteration {}", task_id, iteration);
                                
                                // Call evaluation hook on stop
                                if let Some(ref hook) = self.evaluation_hook {
                                    if let Err(e) = hook.on_stop(&StopReason::CouncilRejected, quality_score).await {
                                        tracing::warn!("Evaluation hook error on stop: {}", e);
                                    }
                                }
                                
                                progress_tracker.update_task_status(
                                    task_id,
                                    ExecutionStatus::Failed,
                                    Some("Council rejected task after refinement".to_string())
                                ).await?;
                                return Err(anyhow::anyhow!("Task rejected by council after refinement attempts"));
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

        let final_verdict_value = final_verdict.expect("Final verdict should be set after refinement loop");

        // Get final state
        let state = self.execution_state.read().await;
        let refinement_state = state.get(&task_id)
            .ok_or_else(|| anyhow::anyhow!("Refinement state not found for task {}", task_id))?;

        Ok(RefinementLoopResult {
            final_verdict: final_verdict_value,
            iterations: iteration,
            quality_scores: refinement_state.quality_scores.clone(),
            iteration_history: refinement_state.iteration_history.clone(),
        })
    }
}

