//! Advanced Credit Assignment with TD Learning
//!
//! Implements sophisticated temporal difference (TD) learning algorithms for
//! credit assignment in multi-turn tasks, including TD(λ) with eligibility traces
//! and value function approximation.
//!
//! @author @darianrosebrook

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::Result;

use crate::progress_tracker::turn_level::{TurnProgress, TaskOutcome, CreditAssignment};

/// TD learning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TdLearningConfig {
    /// Discount factor (gamma) for future rewards (0.0-1.0)
    pub gamma: f64,
    /// Lambda parameter for eligibility traces (0.0-1.0)
    /// 0.0 = TD(0), 1.0 = Monte Carlo
    pub lambda: f64,
    /// Learning rate (alpha) for value updates
    pub alpha: f64,
    /// Recency weight - how much to weight recent turns
    pub recency_weight: f64,
    /// Quality improvement weight
    pub quality_weight: f64,
    /// Success weight
    pub success_weight: f64,
    /// Trajectory pattern weight (for using detected patterns)
    pub pattern_weight: f64,
}

impl Default for TdLearningConfig {
    fn default() -> Self {
        Self {
            gamma: 0.9,      // Standard discount factor
            lambda: 0.7,     // TD(λ) with moderate eligibility traces
            alpha: 0.1,      // Conservative learning rate
            recency_weight: 0.3,
            quality_weight: 0.4,
            success_weight: 0.2,
            pattern_weight: 0.1,
        }
    }
}

/// Value function approximation for TD learning
#[derive(Debug, Clone)]
pub struct ValueFunction {
    /// State values (turn index -> value estimate)
    values: Vec<f64>,
    /// Eligibility traces for TD(λ)
    eligibility_traces: Vec<f64>,
}

impl ValueFunction {
    pub fn new(num_turns: usize) -> Self {
        Self {
            values: vec![0.0; num_turns],
            eligibility_traces: vec![0.0; num_turns],
        }
    }

    /// Initialize eligibility traces
    pub fn reset_eligibility(&mut self) {
        self.eligibility_traces.fill(0.0);
    }

    /// Update eligibility trace for a turn
    pub fn update_eligibility(&mut self, turn_idx: usize, lambda: f64, gamma: f64) {
        // Accumulating eligibility traces
        for i in 0..=turn_idx {
            self.eligibility_traces[i] *= gamma * lambda;
            if i == turn_idx {
                self.eligibility_traces[i] += 1.0;
            }
        }
    }

    /// Update value estimate using TD error
    pub fn update_value(&mut self, turn_idx: usize, td_error: f64, alpha: f64) {
        if turn_idx < self.values.len() {
            self.values[turn_idx] += alpha * td_error * self.eligibility_traces[turn_idx];
        }
    }

    /// Get value estimate for a turn
    pub fn get_value(&self, turn_idx: usize) -> f64 {
        self.values.get(turn_idx).copied().unwrap_or(0.0)
    }
}

/// Advanced credit assignment using TD(λ) learning
pub struct AdvancedCreditAssigner {
    config: TdLearningConfig,
}

impl AdvancedCreditAssigner {
    pub fn new(config: TdLearningConfig) -> Self {
        Self { config }
    }

    /// Assign credit using TD(λ) learning algorithm
    pub fn assign_credit_td_lambda(
        &self,
        trajectory: &[TurnProgress],
        final_outcome: &TaskOutcome,
    ) -> Vec<CreditAssignment> {
        if trajectory.is_empty() {
            return Vec::new();
        }

        let num_turns = trajectory.len();
        let mut value_function = ValueFunction::new(num_turns);
        value_function.reset_eligibility();

        // Extract rewards from trajectory
        let rewards: Vec<f64> = trajectory.iter()
            .map(|turn| self.compute_immediate_reward(turn, final_outcome))
            .collect();

        // Compute value estimates using TD(λ)
        self.compute_value_estimates(&mut value_function, &rewards, trajectory, final_outcome);

        // Convert value estimates to credit assignments
        self.value_to_credit(&value_function, trajectory, final_outcome)
    }

    /// Compute immediate reward for a turn
    fn compute_immediate_reward(&self, turn: &TurnProgress, final_outcome: &TaskOutcome) -> f64 {
        let mut reward = 0.0;

        // Base reward from quality score
        reward += turn.outcome.quality_score * self.config.quality_weight;

        // Success bonus
        if turn.outcome.success {
            reward += self.config.success_weight;
        }

        // Final outcome bonus (if this turn contributed to success)
        if final_outcome.success && turn.outcome.success {
            reward += 0.1; // Small bonus for contributing to final success
        }

        reward
    }

    /// Compute value estimates using TD(λ) algorithm
    fn compute_value_estimates(
        &self,
        value_function: &mut ValueFunction,
        rewards: &[f64],
        trajectory: &[TurnProgress],
        final_outcome: &TaskOutcome,
    ) {
        let num_turns = trajectory.len();

        // Forward pass: compute TD errors and update values
        for t in 0..num_turns {
            // Update eligibility traces
            value_function.update_eligibility(t, self.config.lambda, self.config.gamma);

            // Compute TD error
            let current_value = value_function.get_value(t);
            let next_value = if t < num_turns - 1 {
                value_function.get_value(t + 1)
            } else {
                // Terminal state: use final outcome value
                self.compute_final_value(final_outcome)
            };

            let td_error = rewards[t] + self.config.gamma * next_value - current_value;

            // Update all values using eligibility traces
            for i in 0..=t {
                value_function.update_value(i, td_error, self.config.alpha);
            }
        }

        // Backward pass: refine estimates using final outcome
        self.refine_with_final_outcome(value_function, trajectory, final_outcome);
    }

    /// Compute final value from outcome
    fn compute_final_value(&self, final_outcome: &TaskOutcome) -> f64 {
        let mut value = final_outcome.quality_score;

        // Success bonus
        if final_outcome.success {
            value += 0.2;
        }

        value.min(1.0).max(0.0)
    }

    /// Refine value estimates using final outcome (Monte Carlo correction)
    fn refine_with_final_outcome(
        &self,
        value_function: &mut ValueFunction,
        trajectory: &[TurnProgress],
        final_outcome: &TaskOutcome,
    ) {
        let final_value = self.compute_final_value(final_outcome);
        let num_turns = trajectory.len();

        // Compute returns (discounted cumulative rewards)
        let mut returns = vec![0.0; num_turns];
        let mut cumulative_return = final_value;

        for t in (0..num_turns).rev() {
            cumulative_return = cumulative_return * self.config.gamma;
            returns[t] = cumulative_return;
        }

        // Update values towards returns (Monte Carlo correction)
        for t in 0..num_turns {
            let current_value = value_function.get_value(t);
            let mc_error = returns[t] - current_value;
            value_function.update_value(t, mc_error, self.config.alpha * 0.5); // Smaller learning rate for MC correction
        }
    }

    /// Convert value estimates to credit assignments
    fn value_to_credit(
        &self,
        value_function: &ValueFunction,
        trajectory: &[TurnProgress],
        final_outcome: &TaskOutcome,
    ) -> Vec<CreditAssignment> {
        let num_turns = trajectory.len();
        let mut assignments = Vec::new();

        // Extract values and compute credit
        let values: Vec<f64> = (0..num_turns)
            .map(|i| value_function.get_value(i))
            .collect();

        // Normalize values to sum to 1.0 (credit distribution)
        let total_value: f64 = values.iter().sum();
        let normalized_values = if total_value > 0.0 {
            values.iter().map(|&v| v / total_value).collect::<Vec<f64>>()
        } else {
            // Fallback: equal distribution
            vec![1.0 / num_turns as f64; num_turns]
        };

        // Create credit assignments with detailed reasoning
        for (idx, turn) in trajectory.iter().enumerate() {
            let mut factors = Vec::new();

            // Add TD learning factors
            let value = value_function.get_value(idx);
            factors.push(format!("TD value estimate: {:.4}", value));

            // Recency factor
            let recency = (idx + 1) as f64 / num_turns as f64;
            factors.push(format!("Recency: {:.2}", recency));

            // Quality contribution
            let quality_contribution = turn.outcome.quality_score;
            factors.push(format!("Quality: {:.2}", quality_contribution));

            // Success contribution
            if turn.outcome.success {
                factors.push("Successful turn".to_string());
            }

            // Temporal proximity to final outcome
            let temporal_proximity = 1.0 - (idx as f64 / num_turns as f64);
            factors.push(format!("Temporal proximity: {:.2}", temporal_proximity));

            // Quality improvement
            if idx > 0 {
                let prev_quality = trajectory[idx - 1].outcome.quality_score;
                let improvement = turn.outcome.quality_score - prev_quality;
                if improvement > 0.0 {
                    factors.push(format!("Quality improvement: +{:.2}", improvement));
                }
            }

            assignments.push(CreditAssignment {
                turn_number: turn.turn_number,
                credit_value: normalized_values[idx],
                reasoning: format!(
                    "TD(λ) credit assignment for turn {}: {}",
                    turn.turn_number,
                    factors.join(", ")
                ),
                factors,
            });
        }

        assignments
    }

    /// Assign credit using advantage-weighted method (alternative approach)
    pub fn assign_credit_advantage_weighted(
        &self,
        trajectory: &[TurnProgress],
        final_outcome: &TaskOutcome,
    ) -> Vec<CreditAssignment> {
        if trajectory.is_empty() {
            return Vec::new();
        }

        let num_turns = trajectory.len();
        let mut assignments = Vec::new();

        // Compute advantages (how much better/worse than baseline)
        let baseline_quality = trajectory.iter()
            .map(|t| t.outcome.quality_score)
            .sum::<f64>() / num_turns as f64;

        let advantages: Vec<f64> = trajectory.iter()
            .map(|t| t.outcome.quality_score - baseline_quality)
            .collect();

        // Compute weights using softmax with temperature
        let temperature = 2.0; // Controls sharpness of distribution
        let weights: Vec<f64> = advantages.iter()
            .map(|&adv| (adv / temperature).exp())
            .collect();
        let weight_sum: f64 = weights.iter().sum();
        let normalized_weights: Vec<f64> = weights.iter()
            .map(|&w| w / weight_sum)
            .collect();

        // Apply recency discount
        let final_weights: Vec<f64> = normalized_weights.iter()
            .enumerate()
            .map(|(idx, &w)| {
                let recency_factor = (idx + 1) as f64 / num_turns as f64;
                w * (0.5 + 0.5 * recency_factor) // Blend with recency
            })
            .collect();

        // Normalize again
        let final_sum: f64 = final_weights.iter().sum();
        let credit_values: Vec<f64> = final_weights.iter()
            .map(|&w| w / final_sum)
            .collect();

        // Create assignments
        for (idx, turn) in trajectory.iter().enumerate() {
            let mut factors = Vec::new();
            factors.push(format!("Advantage: {:.4}", advantages[idx]));
            factors.push(format!("Baseline quality: {:.2}", baseline_quality));
            
            if turn.outcome.success {
                factors.push("Successful turn".to_string());
            }

            assignments.push(CreditAssignment {
                turn_number: turn.turn_number,
                credit_value: credit_values[idx],
                reasoning: format!(
                    "Advantage-weighted credit for turn {}: {}",
                    turn.turn_number,
                    factors.join(", ")
                ),
                factors,
            });
        }

        assignments
    }

    /// Hybrid credit assignment combining TD(λ) and advantage weighting
    pub fn assign_credit_hybrid(
        &self,
        trajectory: &[TurnProgress],
        final_outcome: &TaskOutcome,
    ) -> Vec<CreditAssignment> {
        // Get credit from both methods
        let td_credits = self.assign_credit_td_lambda(trajectory, final_outcome);
        let advantage_credits = self.assign_credit_advantage_weighted(trajectory, final_outcome);

        // Combine with weighted average (70% TD, 30% advantage)
        let td_weight = 0.7;
        let advantage_weight = 0.3;

        let mut combined = Vec::new();
        for (td, adv) in td_credits.iter().zip(advantage_credits.iter()) {
            let combined_credit = td.credit_value * td_weight + adv.credit_value * advantage_weight;
            
            let mut factors = td.factors.clone();
            factors.push(format!("TD credit: {:.4}", td.credit_value));
            factors.push(format!("Advantage credit: {:.4}", adv.credit_value));
            factors.push(format!("Combined ({}% TD, {}% advantage)", 
                td_weight * 100.0, advantage_weight * 100.0));

            combined.push(CreditAssignment {
                turn_number: td.turn_number,
                credit_value: combined_credit,
                reasoning: format!(
                    "Hybrid credit assignment for turn {}: {}",
                    td.turn_number,
                    factors.join(", ")
                ),
                factors,
            });
        }

        // Normalize to sum to 1.0
        let total: f64 = combined.iter().map(|a| a.credit_value).sum();
        if total > 0.0 {
            for assignment in &mut combined {
                assignment.credit_value /= total;
            }
        }

        combined
    }
}

impl Default for AdvancedCreditAssigner {
    fn default() -> Self {
        Self::new(TdLearningConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::progress_tracker::turn_level::{AgentAction, TurnOutcome};
    use chrono::Utc;
    use std::collections::HashMap;

    fn create_test_turn(turn_number: u32, quality: f64, success: bool) -> TurnProgress {
        TurnProgress {
            turn_number,
            task_id: uuid::Uuid::new_v4(),
            action: AgentAction {
                action_type: "test_action".to_string(),
                description: format!("Turn {}", turn_number),
                worker_id: None,
                milestone_id: None,
                timestamp: Utc::now(),
                metadata: HashMap::new(),
            },
            outcome: TurnOutcome {
                success,
                quality_score: quality,
                artifacts: None,
                error: None,
                execution_time_ms: Some(100),
                metadata: HashMap::new(),
            },
            reward: None,
            credit_assignment: None,
            started_at: Utc::now(),
            completed_at: Utc::now(),
        }
    }

    #[test]
    fn test_td_lambda_credit_assignment() {
        let assigner = AdvancedCreditAssigner::default();
        
        let trajectory = vec![
            create_test_turn(1, 0.5, true),
            create_test_turn(2, 0.6, true),
            create_test_turn(3, 0.8, true),
            create_test_turn(4, 0.9, true),
        ];

        let final_outcome = TaskOutcome {
            success: true,
            quality_score: 0.9,
            artifacts: vec![],
            completed_at: Utc::now(),
        };

        let credits = assigner.assign_credit_td_lambda(&trajectory, &final_outcome);
        
        assert_eq!(credits.len(), 4);
        
        // Credits should sum to approximately 1.0
        let total: f64 = credits.iter().map(|c| c.credit_value).sum();
        assert!((total - 1.0).abs() < 0.01);
        
        // Later turns should generally get more credit (but not always)
        // This depends on the TD learning dynamics
        println!("TD(λ) Credits:");
        for credit in &credits {
            println!("  Turn {}: {:.4} - {}", credit.turn_number, credit.credit_value, credit.reasoning);
        }
    }

    #[test]
    fn test_hybrid_credit_assignment() {
        let assigner = AdvancedCreditAssigner::default();
        
        let trajectory = vec![
            create_test_turn(1, 0.5, true),
            create_test_turn(2, 0.6, true),
            create_test_turn(3, 0.8, true),
            create_test_turn(4, 0.9, true),
        ];

        let final_outcome = TaskOutcome {
            success: true,
            quality_score: 0.9,
            artifacts: vec![],
            completed_at: Utc::now(),
        };

        let credits = assigner.assign_credit_hybrid(&trajectory, &final_outcome);
        
        assert_eq!(credits.len(), 4);
        
        // Credits should sum to approximately 1.0
        let total: f64 = credits.iter().map(|c| c.credit_value).sum();
        assert!((total - 1.0).abs() < 0.01);
        
        println!("Hybrid Credits:");
        for credit in &credits {
            println!("  Turn {}: {:.4} - {}", credit.turn_number, credit.credit_value, credit.reasoning);
        }
    }
}

