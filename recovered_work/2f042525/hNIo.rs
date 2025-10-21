//! Policy Hooks for Adaptive Agent Behavior
//!
//! Applies RL signals to dynamically adjust agent policies and strategies
//! for continuous improvement during autonomous operation.
//!
//! @author @darianrosebrook

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::evaluation::satisficing::{SatisficingConfig, SatisficingEvaluator};
use crate::models::selection::ModelSelectionPolicy;
use crate::caws::BudgetChecker;
use crate::rl_signals::{RLSignal, PolicyAdjustment};

/// Adaptive policy manager that applies RL-driven adjustments
pub struct PolicyManager {
    satisficing_config: Arc<RwLock<SatisficingConfig>>,
    model_policy: Arc<RwLock<ModelSelectionPolicy>>,
    budget_checker: Arc<RwLock<BudgetChecker>>,
    adjustment_history: Vec<(DateTime<Utc>, PolicyAdjustment, AdjustmentResult)>,
    max_history: usize,
}

/// Result of applying a policy adjustment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdjustmentResult {
    Applied,
    Rejected(String),
    Deferred(String),
    NoOp,
}

/// Policy configuration snapshot for rollback
#[derive(Debug, Clone)]
pub struct PolicySnapshot {
    timestamp: DateTime<Utc>,
    satisficing_config: SatisficingConfig,
    model_policy: ModelSelectionPolicy,
    budget_defaults: BudgetDefaults,
}

/// Budget default settings
#[derive(Debug, Clone)]
pub struct BudgetDefaults {
    pub default_max_files: usize,
    pub default_max_loc: usize,
    pub overrun_buffer: f64,
}

impl Default for PolicyManager {
    fn default() -> Self {
        Self {
            satisficing_config: Arc::new(RwLock::new(SatisficingConfig::default())),
            model_policy: Arc::new(RwLock::new(ModelSelectionPolicy::default())),
            budget_checker: Arc::new(RwLock::new(BudgetChecker::default())),
            adjustment_history: Vec::new(),
            max_history: 100,
        }
    }
}

impl PolicyManager {
    /// Create new policy manager
    pub fn new(
        satisficing_config: Arc<RwLock<SatisficingConfig>>,
        model_policy: Arc<RwLock<ModelSelectionPolicy>>,
        budget_checker: Arc<RwLock<BudgetChecker>>,
    ) -> Self {
        Self {
            satisficing_config,
            model_policy,
            budget_checker,
            adjustment_history: Vec::new(),
            max_history: 100,
        }
    }

    /// Apply policy adjustments from RL signals
    pub async fn apply_adjustments(&mut self, adjustments: Vec<PolicyAdjustment>) -> Vec<AdjustmentResult> {
        let mut results = Vec::new();

        for adjustment in adjustments {
            let result = self.apply_single_adjustment(&adjustment).await;
            self.adjustment_history.push((Utc::now(), adjustment, result.clone()));
            results.push(result);
        }

        // Trim history if needed
        if self.adjustment_history.len() > self.max_history {
            let excess = self.adjustment_history.len() - self.max_history;
            self.adjustment_history.drain(0..excess);
        }

        results
    }

    /// Apply a single policy adjustment
    async fn apply_single_adjustment(&self, adjustment: &PolicyAdjustment) -> AdjustmentResult {
        match adjustment {
            PolicyAdjustment::UpdateSatisficing {
                min_improvement_threshold,
                quality_ceiling_budget,
                hysteresis_window,
            } => {
                let mut config = self.satisficing_config.write().await;
                let mut changed = false;

                if let Some(threshold) = min_improvement_threshold {
                    // Only adjust if significantly different from current
                    if (threshold - config.min_improvement_threshold).abs() > 0.005 {
                        config.min_improvement_threshold = *threshold;
                        changed = true;
                    }
                }

                if let Some(ceiling) = quality_ceiling_budget {
                    if *ceiling != config.quality_ceiling_budget {
                        config.quality_ceiling_budget = *ceiling;
                        changed = true;
                    }
                }

                if let Some(window) = hysteresis_window {
                    if *window != config.hysteresis_window {
                        config.hysteresis_window = *window;
                        changed = true;
                    }
                }

                if changed {
                    AdjustmentResult::Applied
                } else {
                    AdjustmentResult::NoOp
                }
            }

            PolicyAdjustment::UpdateModelWeights {
                model_id,
                performance_weight,
                reliability_weight,
            } => {
                let mut policy = self.model_policy.write().await;

                // Update model weights in the selection policy
                if let Some(model_weights) = policy.model_weights.get_mut(model_id) {
                    model_weights.performance_score = *performance_weight;
                    model_weights.reliability_score = *reliability_weight;
                    model_weights.last_updated = Utc::now();
                    AdjustmentResult::Applied
                } else {
                    // Add new model weights
                    policy.model_weights.insert(model_id.clone(), crate::types::ModelWeights {
                        performance_score: *performance_weight,
                        reliability_score: *reliability_weight,
                        usage_count: 0,
                        last_updated: Utc::now(),
                    });
                    AdjustmentResult::Applied
                }
            }

            PolicyAdjustment::UpdateBudgetDefaults {
                default_max_files,
                default_max_loc,
                overrun_buffer,
            } => {
                // Note: Budget defaults are typically set at task creation time
                // This adjustment would affect future task initialization
                // For now, we'll log that this would be applied at task start
                AdjustmentResult::Deferred("Applied to future tasks".to_string())
            }

            PolicyAdjustment::UpdatePromptStrategy {
                context_window_size,
                evidence_weight,
                iteration_summary_depth,
            } => {
                // These adjustments affect prompt generation
                // Would be applied in the prompting module
                // For now, return deferred as this requires prompt strategy updates
                AdjustmentResult::Deferred("Requires prompt strategy update".to_string())
            }

            PolicyAdjustment::UpdateEvaluationPolicy {
                evaluation_interval,
                early_stop_threshold,
            } => {
                // These affect evaluation frequency and thresholds
                // Would be applied in the evaluation orchestrator
                AdjustmentResult::Deferred("Requires evaluation policy update".to_string())
            }
        }
    }

    /// Create policy snapshot for rollback
    pub async fn create_snapshot(&self) -> PolicySnapshot {
        PolicySnapshot {
            timestamp: Utc::now(),
            satisficing_config: self.satisficing_config.read().await.clone(),
            model_policy: self.model_policy.read().await.clone(),
            budget_defaults: BudgetDefaults {
                default_max_files: 25,
                default_max_loc: 1000,
                overrun_buffer: 0.1,
            },
        }
    }

    /// Rollback to previous policy snapshot
    pub async fn rollback_to_snapshot(&mut self, snapshot: PolicySnapshot) -> Result<(), PolicyError> {
        *self.satisficing_config.write().await = snapshot.satisficing_config;
        *self.model_policy.write().await = snapshot.model_policy;
        // Note: budget defaults rollback would require task initialization changes

        Ok(())
    }

    /// Get policy performance metrics
    pub async fn get_policy_metrics(&self) -> PolicyMetrics {
        let satisficing = self.satisficing_config.read().await;
        let model_policy = self.model_policy.read().await;

        let recent_adjustments: Vec<_> = self.adjustment_history.iter()
            .rev()
            .take(10)
            .collect();

        let applied_count = recent_adjustments.iter()
            .filter(|(_, _, result)| matches!(result, AdjustmentResult::Applied))
            .count();

        let success_rate = if !recent_adjustments.is_empty() {
            applied_count as f64 / recent_adjustments.len() as f64
        } else {
            0.0
        };

        PolicyMetrics {
            satisficing_threshold: satisficing.min_improvement_threshold,
            quality_ceiling_budget: satisficing.quality_ceiling_budget,
            hysteresis_window: satisficing.hysteresis_window,
            active_models: model_policy.model_weights.len(),
            recent_adjustments: recent_adjustments.len(),
            adjustment_success_rate: success_rate,
        }
    }

    /// Check if policy adjustments are needed based on signal patterns
    pub async fn should_adjust_policies(&self, recent_signals: &[RLSignal]) -> PolicyAdjustmentNeeded {
        let metrics = self.get_policy_metrics().await;

        // Analyze signal patterns to determine if adjustments are needed
        let plateau_signals = recent_signals.iter()
            .filter(|s| matches!(s, RLSignal::PlateauEarly { .. }))
            .count();

        let success_signals = recent_signals.iter()
            .filter(|s| matches!(s, RLSignal::TaskSuccess { .. }))
            .count();

        let failure_signals = recent_signals.iter()
            .filter(|s| matches!(s, RLSignal::TaskFailure { .. }))
            .count();

        let total_signals = recent_signals.len();

        // If plateauing too frequently, suggest satisficing adjustments
        if plateau_signals > success_signals && total_signals > 5 {
            return PolicyAdjustmentNeeded::Satisficing;
        }

        // If high failure rate, suggest model or budget adjustments
        if failure_signals as f64 / total_signals as f64 > 0.5 && total_signals > 3 {
            return PolicyAdjustmentNeeded::ModelSelection;
        }

        // If very high success rate, suggest conservative budget adjustments
        if success_signals as f64 / total_signals as f64 > 0.8 && total_signals > 5 {
            return PolicyAdjustmentNeeded::BudgetOptimization;
        }

        PolicyAdjustmentNeeded::None
    }
}

/// Policy performance metrics
#[derive(Debug, Clone)]
pub struct PolicyMetrics {
    pub satisficing_threshold: f64,
    pub quality_ceiling_budget: usize,
    pub hysteresis_window: usize,
    pub active_models: usize,
    pub recent_adjustments: usize,
    pub adjustment_success_rate: f64,
}

/// Types of policy adjustments that might be needed
#[derive(Debug, Clone, PartialEq)]
pub enum PolicyAdjustmentNeeded {
    Satisficing,
    ModelSelection,
    BudgetOptimization,
    PromptStrategy,
    None,
}

/// Policy-related errors
#[derive(Debug, thiserror::Error)]
pub enum PolicyError {
    #[error("Policy adjustment failed: {reason}")]
    AdjustmentFailed { reason: String },

    #[error("Invalid policy configuration: {field}")]
    InvalidConfig { field: String },

    #[error("Policy rollback failed: {reason}")]
    RollbackFailed { reason: String },
}

/// Adaptive agent that integrates RL signals and policy adjustments
pub struct AdaptiveAgent {
    policy_manager: PolicyManager,
    signal_generator: crate::rl_signals::RLSignalGenerator,
    snapshot_history: Vec<PolicySnapshot>,
    adaptation_enabled: bool,
}

impl AdaptiveAgent {
    /// Create new adaptive agent
    pub fn new(
        satisficing_config: Arc<RwLock<SatisficingConfig>>,
        model_policy: Arc<RwLock<ModelSelectionPolicy>>,
        budget_checker: Arc<RwLock<BudgetChecker>>,
    ) -> Self {
        Self {
            policy_manager: PolicyManager::new(
                satisficing_config,
                model_policy,
                budget_checker,
            ),
            signal_generator: crate::rl_signals::RLSignalGenerator::new(),
            snapshot_history: Vec::new(),
            adaptation_enabled: true,
        }
    }

    /// Process task completion and apply adaptations
    pub async fn process_task_completion(
        &mut self,
        task_id: uuid::Uuid,
        result: &crate::types::TaskResult,
    ) -> Result<(), PolicyError> {
        if !self.adaptation_enabled {
            return Ok(());
        }

        // Generate signals from task outcome
        let signals = self.signal_generator.generate_completion_signals(task_id, result);

        // Create policy snapshot before adjustments
        let snapshot = self.policy_manager.create_snapshot().await;
        self.snapshot_history.push(snapshot);

        // Generate policy adjustments from signals
        let adjustments = self.signal_generator.generate_policy_adjustments(&signals);

        // Apply adjustments
        let results = self.policy_manager.apply_adjustments(adjustments).await;

        // Log adjustment results
        let applied_count = results.iter()
            .filter(|r| matches!(r, AdjustmentResult::Applied))
            .count();

        tracing::info!(
            task_id = %task_id,
            signals_generated = signals.len(),
            adjustments_applied = applied_count,
            "Applied policy adaptations from task completion"
        );

        Ok(())
    }

    /// Check if adaptations are needed and apply them
    pub async fn check_and_adapt(&mut self, recent_signals: &[RLSignal]) -> Result<(), PolicyError> {
        if !self.adaptation_enabled {
            return Ok(());
        }

        let needed = self.policy_manager.should_adjust_policies(recent_signals).await;

        match needed {
            PolicyAdjustmentNeeded::None => Ok(()),
            _ => {
                // Generate adjustments based on what's needed
                let adjustments = match needed {
                    PolicyAdjustmentNeeded::Satisficing => {
                        vec![PolicyAdjustment::UpdateSatisficing {
                            min_improvement_threshold: Some(0.015),
                            quality_ceiling_budget: Some(4),
                            hysteresis_window: Some(5),
                        }]
                    }
                    PolicyAdjustmentNeeded::ModelSelection => {
                        // Would need to analyze model performance signals
                        vec![]
                    }
                    PolicyAdjustmentNeeded::BudgetOptimization => {
                        vec![PolicyAdjustment::UpdateBudgetDefaults {
                            default_max_files: Some(20),
                            default_max_loc: Some(800),
                            overrun_buffer: Some(0.15),
                        }]
                    }
                    PolicyAdjustmentNeeded::PromptStrategy => {
                        vec![PolicyAdjustment::UpdatePromptStrategy {
                            context_window_size: Some(4000),
                            evidence_weight: Some(0.8),
                            iteration_summary_depth: Some(3),
                        }]
                    }
                    PolicyAdjustmentNeeded::None => vec![],
                };

                if !adjustments.is_empty() {
                    let results = self.policy_manager.apply_adjustments(adjustments).await;
                    let applied_count = results.iter()
                        .filter(|r| matches!(r, AdjustmentResult::Applied))
                        .count();

                    tracing::info!(
                        adaptation_type = ?needed,
                        adjustments_applied = applied_count,
                        "Applied proactive policy adaptation"
                    );
                }

                Ok(())
            }
        }
    }

    /// Get current policy metrics
    pub async fn get_metrics(&self) -> PolicyMetrics {
        self.policy_manager.get_policy_metrics().await
    }

    /// Enable/disable adaptation
    pub fn set_adaptation_enabled(&mut self, enabled: bool) {
        self.adaptation_enabled = enabled;
    }

    /// Rollback to last policy snapshot
    pub async fn rollback_policy(&mut self) -> Result<(), PolicyError> {
        if let Some(snapshot) = self.snapshot_history.pop() {
            self.policy_manager.rollback_to_snapshot(snapshot).await?;
            tracing::info!("Rolled back to previous policy snapshot");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::satisficing::SatisficingConfig;
    use crate::models::selection::ModelSelectionPolicy;
    use crate::caws::BudgetChecker;

    #[tokio::test]
    async fn test_policy_adjustments() {
        let satisficing = Arc::new(RwLock::new(SatisficingConfig::default()));
        let model_policy = Arc::new(RwLock::new(ModelSelectionPolicy::default()));
        let budget_checker = Arc::new(RwLock::new(BudgetChecker::default()));

        let mut manager = PolicyManager::new(
            satisficing.clone(),
            model_policy.clone(),
            budget_checker,
        );

        // Test satisficing adjustment
        let adjustment = PolicyAdjustment::UpdateSatisficing {
            min_improvement_threshold: Some(0.01),
            quality_ceiling_budget: Some(4),
            hysteresis_window: Some(5),
        };

        let results = manager.apply_adjustments(vec![adjustment]).await;
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0], AdjustmentResult::Applied));

        // Verify adjustment was applied
        let config = satisficing.read().await;
        assert_eq!(config.min_improvement_threshold, 0.01);
        assert_eq!(config.quality_ceiling_budget, 4);
        assert_eq!(config.hysteresis_window, 5);
    }

    #[tokio::test]
    async fn test_adaptive_agent() {
        let satisficing = Arc::new(RwLock::new(SatisficingConfig::default()));
        let model_policy = Arc::new(RwLock::new(ModelSelectionPolicy::default()));
        let budget_checker = Arc::new(RwLock::new(BudgetChecker::default()));

        let mut agent = AdaptiveAgent::new(
            satisficing.clone(),
            model_policy.clone(),
            budget_checker,
        );

        // Simulate task completion
        let task_id = uuid::Uuid::new_v4();
        let result = crate::types::TaskResult::Completed(crate::types::TaskResultDetail {
            task_id,
            final_report: crate::evaluation::EvalReport {
                score: 0.9,
                files_modified: 2,
                loc_added: 100,
                loc_removed: 10,
                test_results: vec![],
                lint_errors: vec![],
                failed_criteria: vec![],
                recommendations: vec![],
            },
            iterations: 3,
            stop_reason: crate::types::StopReason::Satisficed,
            model_used: "test-model".to_string(),
            total_time_ms: 3000,
            artifacts: vec![],
        });

        // This should generate signals and potentially apply adjustments
        let _ = agent.process_task_completion(task_id, &result).await;

        // Check that signals were generated
        let recent_signals = agent.signal_generator.get_recent_signals(10);
        assert!(!recent_signals.is_empty());
    }
}
