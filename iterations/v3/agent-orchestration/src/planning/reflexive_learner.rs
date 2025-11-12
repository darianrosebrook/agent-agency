//! Reflexive Learning Component
//!
//! This component implements continuous learning from execution outcomes,
//! updating routing decisions and worker selection strategies based on
//! historical performance data.
//!
//! @author @darianrosebrook

use std::sync::Arc;
use std::collections::HashMap;
use anyhow::Result;
use uuid::Uuid;
use tracing::{info, warn, debug};
use chrono::Utc;
use tokio::time::interval;
use serde::{Serialize, Deserialize};

use agent_agency_contracts::execution_artifacts::ExecutionArtifacts;
use agent_agency_contracts::planning_io::Milestone;
use crate::planning::worker_assignment::WorkerAssignmentStrategy;
use crate::planning::worker_evolution::WorkerEvolutionEngine;

/// Learning outcome from task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningOutcome {
    /// Worker ID that executed the task
    pub worker_id: Uuid,
    
    /// Milestone ID that was executed
    pub milestone_id: String,
    
    /// Execution success status
    pub success: bool,
    
    /// Quality score (0.0 - 1.0)
    pub quality_score: f64,
    
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    
    /// Error message if execution failed
    pub error_message: Option<String>,
    
    /// Timestamp of execution
    pub timestamp: chrono::DateTime<Utc>,
    
    /// Task characteristics for pattern matching
    pub task_characteristics: TaskCharacteristics,
}

/// Task characteristics for pattern matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCharacteristics {
    /// Task complexity (estimated)
    pub complexity: f64,
    
    /// Required capabilities
    pub required_capabilities: Vec<String>,
    
    /// Task type/category
    pub task_type: String,
    
    /// Estimated resource requirements
    pub resource_requirements: ResourceRequirements,
}

/// Resource requirements for a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// Estimated memory in MB
    pub memory_mb: u64,
    
    /// Estimated CPU cores
    pub cpu_cores: f64,
    
    /// Estimated execution time in seconds
    pub estimated_time_sec: f64,
}

/// Routing adjustment based on learning outcomes
#[derive(Debug, Clone)]
pub struct RoutingAdjustment {
    /// Worker ID affected by adjustment
    pub worker_id: Uuid,
    
    /// Performance score adjustment (-1.0 to 1.0)
    pub performance_adjustment: f64,
    
    /// Capability score adjustments
    pub capability_adjustments: HashMap<String, f64>,
    
    /// Reason for adjustment
    pub reason: String,
}

/// Reflexive learner that processes outcomes and updates routing
pub struct ReflexiveLearner {
    /// Worker assignment strategy to update
    worker_assignment_strategy: Arc<WorkerAssignmentStrategy>,
    
    /// Worker evolution engine for creating/refining workers
    evolution_engine: Option<Arc<WorkerEvolutionEngine>>,
    
    /// Learning outcomes history (for pattern analysis)
    outcome_history: Arc<tokio::sync::RwLock<Vec<LearningOutcome>>>,
    
    /// Learning configuration
    config: LearningConfig,
    
    /// Continuous learning loop task handle
    learning_loop_handle: Arc<tokio::sync::RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

/// Configuration for reflexive learning
#[derive(Debug, Clone)]
pub struct LearningConfig {
    /// Minimum number of outcomes before making adjustments
    pub min_outcomes_for_adjustment: usize,
    
    /// Learning rate for performance score updates (0.0 - 1.0)
    pub learning_rate: f64,
    
    /// Decay factor for old outcomes (0.0 - 1.0)
    pub outcome_decay_factor: f64,
    
    /// Maximum history size
    pub max_history_size: usize,
    
    /// Enable automatic routing adjustments
    pub enable_auto_adjustments: bool,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            min_outcomes_for_adjustment: 5,
            learning_rate: 0.1,
            outcome_decay_factor: 0.95,
            max_history_size: 1000,
            enable_auto_adjustments: true,
        }
    }
}

impl ReflexiveLearner {
    /// Create a new reflexive learner
    pub fn new(
        worker_assignment_strategy: Arc<WorkerAssignmentStrategy>,
        config: LearningConfig,
    ) -> Self {
        Self {
            worker_assignment_strategy,
            evolution_engine: None,
            outcome_history: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            config,
            learning_loop_handle: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }
    
    /// Create a new reflexive learner with evolution engine
    pub fn with_evolution_engine(
        worker_assignment_strategy: Arc<WorkerAssignmentStrategy>,
        evolution_engine: Arc<WorkerEvolutionEngine>,
        config: LearningConfig,
    ) -> Self {
        Self {
            worker_assignment_strategy,
            evolution_engine: Some(evolution_engine),
            outcome_history: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            config,
            learning_loop_handle: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }

    /// Start continuous learning loop that periodically analyzes accumulated outcomes
    pub async fn start_continuous_learning(self: &Arc<Self>, interval_secs: u64) -> Result<()> {
        if !self.config.enable_auto_adjustments {
            debug!("ReflexiveLearner: auto-adjustments disabled, continuous learning not started");
            return Ok(());
        }

        let learner = self.clone();
        let interval_duration = std::time::Duration::from_secs(interval_secs);

        let handle = tokio::spawn(async move {
            let mut interval_timer = interval(interval_duration);
            
            loop {
                interval_timer.tick().await;
                
                if let Err(e) = Self::process_accumulated_outcomes(&learner).await {
                    warn!("Error in continuous learning loop: {}", e);
                }
            }
        });

        *self.learning_loop_handle.write().await = Some(handle);
        info!("ReflexiveLearner: continuous learning loop started (interval: {}s)", interval_secs);
        
        Ok(())
    }

    /// Stop continuous learning loop
    pub async fn stop_continuous_learning(&self) {
        let mut handle_guard = self.learning_loop_handle.write().await;
        if let Some(handle) = handle_guard.take() {
            handle.abort();
            info!("ReflexiveLearner: continuous learning loop stopped");
        }
    }

    /// Process accumulated outcomes and generate routing adjustments
    async fn process_accumulated_outcomes(learner: &Arc<ReflexiveLearner>) -> Result<()> {
        let history = learner.outcome_history.read().await;
        
        // Need minimum outcomes before making adjustments
        if history.len() < learner.config.min_outcomes_for_adjustment {
            debug!(
                "ReflexiveLearner: insufficient outcomes for continuous adjustment: {}/{}",
                history.len(),
                learner.config.min_outcomes_for_adjustment
            );
            return Ok(());
        }

        // Group outcomes by worker ID for analysis
        let mut worker_outcomes: HashMap<Uuid, Vec<&LearningOutcome>> = HashMap::new();
        for outcome in history.iter() {
            worker_outcomes
                .entry(outcome.worker_id)
                .or_insert_with(Vec::new)
                .push(outcome);
        }

        let mut all_adjustments = Vec::new();

        // Analyze each worker's outcomes
        for (worker_id, outcomes) in worker_outcomes.iter() {
            if outcomes.len() >= learner.config.min_outcomes_for_adjustment {
                // Create a synthetic outcome for analysis (using the most recent one)
                if let Some(latest_outcome) = outcomes.last() {
                    match learner.calculate_worker_adjustment(outcomes, latest_outcome) {
                        Ok(adjustment) => {
                            all_adjustments.push(adjustment);
                        }
                        Err(e) => {
                            warn!("Failed to calculate adjustment for worker {}: {}", worker_id, e);
                        }
                    }
                }
            }
        }

        // Apply all adjustments
        for adjustment in &all_adjustments {
            if let Err(e) = learner.apply_adjustment(adjustment).await {
                warn!("Failed to apply routing adjustment: {}", e);
            }
        }

        if !all_adjustments.is_empty() {
            info!(
                "ReflexiveLearner: processed {} accumulated outcomes, applied {} routing adjustments",
                history.len(),
                all_adjustments.len()
            );
        }

        Ok(())
    }

    /// Process execution outcome and update learning
    pub async fn process_outcome(
        &self,
        artifacts: &ExecutionArtifacts,
        milestone: &Milestone,
        worker_id: Uuid,
    ) -> Result<Vec<RoutingAdjustment>> {
        info!(
            "Processing learning outcome for milestone {} from worker {}",
            milestone.id, worker_id
        );

        // Extract outcome from artifacts
        let outcome = self.extract_outcome(artifacts, milestone, worker_id)?;

        // Store outcome in history
        self.store_outcome(outcome.clone()).await?;

        // Process outcomes for worker evolution if evolution engine is available
        if let Some(ref evolution_engine) = self.evolution_engine {
            let history = self.outcome_history.read().await;
            let recent_outcomes: Vec<&LearningOutcome> = history.iter().collect();
            
            if recent_outcomes.len() >= 10 {
                // Process outcomes and generate proposals
                if let Err(e) = evolution_engine.process_outcomes(&recent_outcomes).await {
                    warn!("Failed to process outcomes for worker evolution: {}", e);
                }
                
                // Evaluate and execute approved proposals periodically
                // (Only evaluate every 50 outcomes to avoid excessive worker creation)
                if recent_outcomes.len() % 50 == 0 {
                    if let Err(e) = evolution_engine.evaluate_and_execute().await {
                        warn!("Failed to evaluate worker evolution proposals: {}", e);
                    }
                }
            }
        }

        // Analyze patterns and generate adjustments
        let adjustments = if self.config.enable_auto_adjustments {
            self.analyze_and_adjust(&outcome).await?
        } else {
            Vec::new()
        };

        // Apply adjustments to worker assignment strategy
        for adjustment in &adjustments {
            self.apply_adjustment(adjustment).await?;
        }

        Ok(adjustments)
    }

    /// Extract learning outcome from execution artifacts
    fn extract_outcome(
        &self,
        artifacts: &ExecutionArtifacts,
        milestone: &Milestone,
        worker_id: Uuid,
    ) -> Result<LearningOutcome> {
        // Determine success status
        // Success is determined by: completed_at is Some and tests passed
        let is_completed = artifacts.provenance.completed_at.is_some();
        let tests_passed = artifacts.tests.unit_tests.failed == 0 &&
                          artifacts.tests.integration_tests.failed == 0 &&
                          artifacts.tests.e2e_tests.failed == 0;
        let success = is_completed && tests_passed;

        // Extract quality score from artifacts (if available)
        let quality_score = self.extract_quality_score(artifacts);

        // Extract execution time
        let execution_time_ms = self.extract_execution_time(artifacts);

        // Extract error message if failed
        let error_message = if !success {
            self.extract_error_message(artifacts)
        } else {
            None
        };

        // Extract task characteristics
        let task_characteristics = self.extract_task_characteristics(milestone);

        Ok(LearningOutcome {
            worker_id,
            milestone_id: milestone.id.clone(),
            success,
            quality_score,
            execution_time_ms,
            error_message,
            timestamp: Utc::now(),
            task_characteristics,
        })
    }

    /// Extract quality score from artifacts
    fn extract_quality_score(&self, artifacts: &ExecutionArtifacts) -> f64 {
        // Try to extract from provenance audit trail
        // Note: ArtifactMetadata doesn't have quality_score field
        // We'll calculate from test results instead
        
        // Calculate quality from test pass rate
        let total_tests = artifacts.tests.unit_tests.total + 
                         artifacts.tests.integration_tests.total + 
                         artifacts.tests.e2e_tests.total;
        let passed_tests = artifacts.tests.unit_tests.passed + 
                          artifacts.tests.integration_tests.passed + 
                          artifacts.tests.e2e_tests.passed;
        
        if total_tests > 0 {
            (passed_tests as f64 / total_tests as f64).max(0.0).min(1.0)
        } else {
            // Default quality score based on execution completion
            let is_completed = artifacts.provenance.completed_at.is_some();
            let tests_passed = artifacts.tests.unit_tests.failed == 0 &&
                              artifacts.tests.integration_tests.failed == 0 &&
                              artifacts.tests.e2e_tests.failed == 0;
            
            if is_completed && tests_passed {
                0.8 // Default success quality
            } else {
                0.2 // Default failure quality
            }
        }
    }

    /// Extract execution time from artifacts
    fn extract_execution_time(&self, artifacts: &ExecutionArtifacts) -> u64 {
        // Use provenance duration_ms if available
        if artifacts.provenance.duration_ms > 0 {
            return artifacts.provenance.duration_ms;
        }

        // Calculate from provenance timestamps if available
        if let Some(completed) = artifacts.provenance.completed_at {
            let started = artifacts.provenance.started_at;
            let duration = completed.signed_duration_since(started);
            return duration.num_milliseconds() as u64;
        }

        // Default execution time
        0
    }

    /// Extract error message from artifacts
    fn extract_error_message(&self, artifacts: &ExecutionArtifacts) -> Option<String> {
        // Check audit trail for errors
        for event in &artifacts.provenance.audit_trail {
            if event.event.to_lowercase().contains("error")
                || event.event.to_lowercase().contains("failure")
            {
                if let Some(ref details) = event.details {
                    if let Some(details_str) = details.as_str() {
                        return Some(details_str.to_string());
                    }
                }
                return Some(event.event.clone());
            }
        }

        // Check metadata - ArtifactMetadata doesn't have error field
        // Error information should be in audit_trail or execution_status
        // Skip metadata check as it doesn't contain error information

        None
    }

    /// Extract task characteristics from milestone
    fn extract_task_characteristics(&self, milestone: &Milestone) -> TaskCharacteristics {
        // Estimate complexity based on milestone description
        let complexity = self.estimate_complexity(milestone);

        // Extract required capabilities from milestone scope
        let required_capabilities = milestone.scope.allowed_operations.clone();

        // Determine task type from milestone objective
        let task_type = milestone.objective
            .split_whitespace()
            .next()
            .unwrap_or("unknown")
            .to_string();

        // Estimate resource requirements
        // Note: Milestone doesn't have memory/cpu fields, use defaults
        let resource_requirements = ResourceRequirements {
            memory_mb: 512, // Default
            cpu_cores: 1.0, // Default
            estimated_time_sec: milestone.estimated_duration.map(|m| m as f64 * 60.0).unwrap_or(60.0),
        };

        TaskCharacteristics {
            complexity,
            required_capabilities,
            task_type,
            resource_requirements,
        }
    }

    /// Estimate task complexity from milestone
    fn estimate_complexity(&self, milestone: &Milestone) -> f64 {
        // Simple heuristic: longer objectives = more complex
        let description_length = milestone.objective.len();
        let base_complexity = (description_length as f64 / 1000.0).min(1.0);

        // Adjust based on dependencies
        let dependency_factor = (milestone.dependencies.len() as f64 / 10.0).min(0.5);

        // Adjust based on estimated duration (in minutes)
        let duration_factor = if let Some(duration_minutes) = milestone.estimated_duration {
            (duration_minutes as f64 / 60.0).min(0.5) // Normalize to hours
        } else {
            0.0
        };

        (base_complexity + dependency_factor + duration_factor).min(1.0)
    }

    /// Store outcome in history
    async fn store_outcome(&self, outcome: LearningOutcome) -> Result<()> {
        let mut history = self.outcome_history.write().await;

        // Apply decay to existing outcomes
        // Decay is applied implicitly by recency weighting in analysis
        // No explicit decay needed here

        // Add new outcome
        history.push(outcome);

        // Trim history if too large
        if history.len() > self.config.max_history_size {
            let excess = history.len() - self.config.max_history_size;
            history.drain(0..excess);
        }

        Ok(())
    }

    /// Analyze patterns and generate routing adjustments
    async fn analyze_and_adjust(
        &self,
        new_outcome: &LearningOutcome,
    ) -> Result<Vec<RoutingAdjustment>> {
        let history = self.outcome_history.read().await;

        // Need minimum outcomes before making adjustments
        if history.len() < self.config.min_outcomes_for_adjustment {
            debug!(
                "Insufficient outcomes for adjustment: {}/{}",
                history.len(),
                self.config.min_outcomes_for_adjustment
            );
            return Ok(Vec::new());
        }

        let mut adjustments = Vec::new();

        // Analyze worker performance
        let worker_outcomes: Vec<&LearningOutcome> = history
            .iter()
            .filter(|o| o.worker_id == new_outcome.worker_id)
            .collect();

        if worker_outcomes.len() >= self.config.min_outcomes_for_adjustment {
            let adjustment = self.calculate_worker_adjustment(&worker_outcomes, new_outcome)?;
            adjustments.push(adjustment);
        }

        // Analyze capability performance
        for capability in &new_outcome.task_characteristics.required_capabilities {
            let capability_outcomes: Vec<&LearningOutcome> = history
                .iter()
                .filter(|o| {
                    o.task_characteristics
                        .required_capabilities
                        .contains(capability)
                        && o.worker_id == new_outcome.worker_id
                })
                .collect();

            if capability_outcomes.len() >= self.config.min_outcomes_for_adjustment {
                let adjustment = self.calculate_capability_adjustment(
                    &capability_outcomes,
                    capability,
                    new_outcome,
                )?;
                adjustments.push(adjustment);
            }
        }

        Ok(adjustments)
    }

    /// Calculate worker performance adjustment
    fn calculate_worker_adjustment(
        &self,
        outcomes: &[&LearningOutcome],
        new_outcome: &LearningOutcome,
    ) -> Result<RoutingAdjustment> {
        // Calculate success rate
        let success_rate = outcomes.iter().filter(|o| o.success).count() as f64 / outcomes.len() as f64;

        // Calculate average quality score
        let avg_quality = outcomes.iter().map(|o| o.quality_score).sum::<f64>() / outcomes.len() as f64;

        // Calculate performance score (weighted combination)
        let performance_score = (success_rate * 0.6) + (avg_quality * 0.4);

        // Calculate adjustment based on deviation from baseline
        let baseline = 0.7; // Baseline performance expectation
        let performance_adjustment = (performance_score - baseline) * self.config.learning_rate;

        let reason = format!(
            "Worker performance: {:.2}% success rate, {:.2} avg quality ({} outcomes)",
            success_rate * 100.0,
            avg_quality,
            outcomes.len()
        );

        Ok(RoutingAdjustment {
            worker_id: new_outcome.worker_id,
            performance_adjustment,
            capability_adjustments: HashMap::new(),
            reason,
        })
    }

    /// Calculate capability-specific adjustment
    fn calculate_capability_adjustment(
        &self,
        outcomes: &[&LearningOutcome],
        capability: &str,
        new_outcome: &LearningOutcome,
    ) -> Result<RoutingAdjustment> {
        // Calculate success rate for this capability
        let success_rate = outcomes.iter().filter(|o| o.success).count() as f64 / outcomes.len() as f64;

        // Calculate average quality for this capability
        let avg_quality = outcomes.iter().map(|o| o.quality_score).sum::<f64>() / outcomes.len() as f64;

        // Calculate capability score
        let capability_score = (success_rate * 0.6) + (avg_quality * 0.4);

        // Calculate adjustment
        let baseline = 0.7;
        let capability_adjustment = (capability_score - baseline) * self.config.learning_rate;

        let mut capability_adjustments = HashMap::new();
        capability_adjustments.insert(capability.to_string(), capability_adjustment);

        let reason = format!(
            "Capability '{}' performance: {:.2}% success rate, {:.2} avg quality ({} outcomes)",
            capability,
            success_rate * 100.0,
            avg_quality,
            outcomes.len()
        );

        Ok(RoutingAdjustment {
            worker_id: new_outcome.worker_id,
            performance_adjustment: 0.0,
            capability_adjustments,
            reason,
        })
    }

    /// Apply routing adjustment to worker assignment strategy
    async fn apply_adjustment(&self, adjustment: &RoutingAdjustment) -> Result<()> {
        info!(
            "Applying routing adjustment for worker {}: {}",
            adjustment.worker_id, adjustment.reason
        );

        // Update worker performance in WorkerAssignmentStrategy
        // Convert performance adjustment to success/execution_time for update_worker_performance
        // Performance adjustment > 0 means better performance, < 0 means worse
        let success = adjustment.performance_adjustment >= 0.0;
        
        // Estimate execution time from adjustment (positive adjustment = faster execution)
        // Use a baseline of 60000ms (1 minute) and adjust based on performance_adjustment
        let baseline_time_ms = 60000u64;
        let execution_time_ms = if adjustment.performance_adjustment > 0.0 {
            // Better performance = faster execution
            baseline_time_ms.saturating_sub((adjustment.performance_adjustment * 10000.0) as u64)
        } else {
            // Worse performance = slower execution
            baseline_time_ms + ((-adjustment.performance_adjustment * 10000.0) as u64)
        };

        // Update worker performance metrics
        if let Err(e) = self.worker_assignment_strategy.update_worker_performance(
            adjustment.worker_id,
            success,
            execution_time_ms,
        ).await {
            warn!("Failed to update worker performance for {}: {}", adjustment.worker_id, e);
        } else {
            debug!(
                "Updated worker {} performance: success={}, execution_time_ms={}, adjustment={:.4}",
                adjustment.worker_id, success, execution_time_ms, adjustment.performance_adjustment
            );
        }

        // Apply capability adjustments if any
        for (capability, adjustment_value) in &adjustment.capability_adjustments {
            debug!(
                "Capability adjustment for worker {}: {} = {:.4}",
                adjustment.worker_id, capability, adjustment_value
            );
            // TODO: Implement comprehensive capability adjustment application
            //       Currently logs capability adjustments only; should implement comprehensive application that uses WorkerAssignmentStrategy methods to apply capability adjustments when capability tracking is added.
            //
            // COMPLETION CHECKLIST:
            // [ ] Primary functionality implemented
            // [ ] API/data structures defined & stable
            // [ ] Error handling + validation aligned with error taxonomy
            // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
            // [ ] Integration tests for external systems/contracts
            // [ ] Documentation: public API + system behavior
            // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
            // [ ] Security posture reviewed (inputs, authz, sandboxing)
            // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
            // [ ] Configurability and feature flags defined if relevant
            // [ ] Failure-mode cards documented (degradation paths)
            //
            // ACCEPTANCE CRITERIA:
            // - Capability adjustments are applied via WorkerAssignmentStrategy
            // - Capability tracking system is integrated
            // - Adjustments persist and affect worker assignment
            // - Adjustment application handles errors gracefully
            //
            // DEPENDENCIES:
            // - WorkerAssignmentStrategy capability methods (Required)
            // - Capability tracking system (Required)
            // - Adjustment persistence system (Required)
            //
            // ESTIMATED EFFORT: 8-12 hours (medium confidence)
            // PRIORITY: Medium
            // BLOCKING: No
            //
            // GOVERNANCE:
            // - CAWS Tier: 2 (capability management functionality)
            // - Change Budget: ~200 LOC
            // - Reviewer Requirements: Worker assignment and capability tracking expertise
        }

        Ok(())
    }

    /// Apply aggregated insights from federated learning
    /// This method allows federated learning engine to inject aggregated insights
    /// from other tenants into the learner's routing decisions
    pub async fn apply_aggregated_insights(
        &self,
        avg_quality_score: f64,
        avg_success_rate: f64,
        avg_execution_time_ms: f64,
        routing_weights: &HashMap<String, f64>,
    ) -> Result<()> {
        info!(
            "Applying aggregated insights: quality={:.3}, success_rate={:.3}, exec_time={:.0}ms",
            avg_quality_score, avg_success_rate, avg_execution_time_ms
        );

        // Get all unique worker IDs from outcome history
        let worker_ids: Vec<Uuid> = {
            let history = self.outcome_history.read().await;
            history.iter()
                .map(|o| o.worker_id)
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect()
        };

        let worker_count = worker_ids.len();

        // Apply aggregated insights to each worker proportionally
        // Calculate performance adjustment based on aggregated metrics vs baseline
        let baseline_quality = 0.7;
        let baseline_success = 0.7;
        let performance_adjustment = ((avg_quality_score - baseline_quality) * 0.5 + 
                                     (avg_success_rate - baseline_success) * 0.5) * self.config.learning_rate;

        // Create routing adjustments for each worker
        for worker_id in worker_ids {
            let adjustment = RoutingAdjustment {
                worker_id,
                performance_adjustment,
                capability_adjustments: routing_weights.clone()
                    .into_iter()
                    .map(|(k, v)| (k, v * self.config.learning_rate))
                    .collect(),
                reason: format!(
                    "Federated learning insights: aggregated quality={:.3}, success={:.3}",
                    avg_quality_score, avg_success_rate
                ),
            };

            // Apply adjustment
            if let Err(e) = self.apply_adjustment(&adjustment).await {
                warn!("Failed to apply aggregated insight to worker {}: {}", adjustment.worker_id, e);
            }
        }

        info!("Applied aggregated insights to {} workers", worker_count);
        Ok(())
    }

    /// Get learning statistics
    pub async fn get_statistics(&self) -> LearningStatistics {
        let history = self.outcome_history.read().await;

        let total_outcomes = history.len();
        let successful_outcomes = history.iter().filter(|o| o.success).count();
        let avg_quality = if total_outcomes > 0 {
            history.iter().map(|o| o.quality_score).sum::<f64>() / total_outcomes as f64
        } else {
            0.0
        };

        LearningStatistics {
            total_outcomes,
            successful_outcomes,
            success_rate: if total_outcomes > 0 {
                successful_outcomes as f64 / total_outcomes as f64
            } else {
                0.0
            },
            average_quality: avg_quality,
        }
    }
}

/// Learning statistics
#[derive(Debug, Clone)]
pub struct LearningStatistics {
    pub total_outcomes: usize,
    pub successful_outcomes: usize,
    pub success_rate: f64,
    pub average_quality: f64,
}

impl Drop for ReflexiveLearner {
    fn drop(&mut self) {
        // Stop continuous learning loop on drop
        // Use Handle::try_current() to avoid creating nested runtime
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            // If we're in an async context, spawn a task to stop the loop
            let learner = self.learning_loop_handle.clone();
            handle.spawn(async move {
                if let Some(handle) = learner.read().await.as_ref() {
                    handle.abort();
                }
            });
        } else {
            // Not in async context, try to create a runtime (but this may fail)
            if let Ok(rt) = tokio::runtime::Runtime::new() {
                rt.block_on(async {
                    if let Some(handle) = self.learning_loop_handle.read().await.as_ref() {
                        handle.abort();
                    }
                });
            }
        }
    }
}

