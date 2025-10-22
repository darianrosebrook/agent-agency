//! Reward Function for LLM Parameter Optimization
//!
//! Implements explicit, constrained reward functions with scalarization
//! and constraint checking for safe parameter optimization.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Objective weights for reward scalarization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveWeights {
    /// Weight for quality component (positive - higher is better)
    pub w_quality: f64,
    /// Weight for latency penalty (negative - lower is better)
    pub w_latency: f64,
    /// Weight for token penalty (negative - lower is better)
    pub w_tokens: f64,
}

impl Default for ObjectiveWeights {
    fn default() -> Self {
        Self {
            w_quality: 1.0,
            w_latency: 0.1,
            w_tokens: 0.01,
        }
    }
}

/// Optimization constraints for parameter validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConstraints {
    pub max_latency_ms: u64,
    pub max_tokens: u32,
    pub min_quality: f64,
    pub require_caws_compliance: bool,
    /// Trust region around baseline temperature
    pub max_delta_temperature: f32,
    /// Trust region around baseline max_tokens
    pub max_delta_max_tokens: u32,
}

impl Default for OptimizationConstraints {
    fn default() -> Self {
        Self {
            max_latency_ms: 5000,
            max_tokens: 4000,
            min_quality: 0.7,
            require_caws_compliance: true,
            max_delta_temperature: 0.2,
            max_delta_max_tokens: 200,
        }
    }
}

/// Task outcome from LLM generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskOutcome {
    pub quality_score: f64,
    pub latency_ms: u64,
    pub tokens_used: usize,
    pub success: bool,
    pub caws_compliance: bool,
}

/// Baseline metrics for normalization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineMetrics {
    pub avg_quality: f64,
    pub avg_latency: u64,
    pub avg_tokens: f64,
    pub temperature: f32,
    pub max_tokens: u32,
}

/// Reward calculation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardResult {
    pub reward: f64,
    pub components: RewardComponents,
    pub constraint_violations: Vec<String>,
}

/// Individual reward components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardComponents {
    pub quality_contrib: f64,
    pub latency_penalty: f64,
    pub token_penalty: f64,
}

/// Constraint check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintCheckResult {
    pub passed: bool,
    pub violations: Vec<String>,
    pub severity: ConstraintSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintSeverity {
    Info,
    Warning,
    Error,
}

/// Explicit, constrained reward function
pub struct RewardFunction {
    /// Scalarized reward weights
    weights: ObjectiveWeights,
    /// Baseline metrics for normalization
    baselines: HashMap<String, BaselineMetrics>,
}

impl RewardFunction {
    pub fn new(weights: ObjectiveWeights) -> Self {
        Self {
            weights,
            baselines: HashMap::new(),
        }
    }

    /// Set baseline metrics for a task type
    pub fn set_baseline(&mut self, task_type: String, baseline: BaselineMetrics) {
        self.baselines.insert(task_type, baseline);
    }

    /// Calculate scalarized reward: R = w_q * Q - w_l * norm(L) - w_t * norm(T)
    pub fn calculate(
        &self,
        outcome: &TaskOutcome,
        baseline: &BaselineMetrics,
    ) -> RewardResult {
        // Normalize components to [0, 1] range
        let norm_quality = outcome.quality_score;
        let norm_latency = (outcome.latency_ms as f64) / (baseline.avg_latency as f64);
        let norm_tokens = (outcome.tokens_used as f64) / baseline.avg_tokens;
        
        // Calculate individual components
        let quality_contrib = self.weights.w_quality * norm_quality;
        let latency_penalty = self.weights.w_latency * norm_latency;
        let token_penalty = self.weights.w_tokens * norm_tokens;
        
        // Scalarized reward
        let reward = quality_contrib - latency_penalty - token_penalty;
        
        RewardResult {
            reward,
            components: RewardComponents {
                quality_contrib,
                latency_penalty,
                token_penalty,
            },
            constraint_violations: Vec::new(), // Will be populated by constraint checker
        }
    }
    
    /// Check hard constraints (pre-action)
    pub fn check_constraints(
        &self,
        proposed: &crate::bandit_policy::ParameterSet,
        constraints: &OptimizationConstraints,
        baseline: &BaselineMetrics,
    ) -> ConstraintCheckResult {
        let mut violations = Vec::new();
        let mut severity = ConstraintSeverity::Info;

        // 1. Trust region checks
        let temp_delta = (proposed.temperature - baseline.temperature).abs();
        if temp_delta > constraints.max_delta_temperature {
            violations.push(format!(
                "Temperature delta {:.3} exceeds trust region {:.3}",
                temp_delta, constraints.max_delta_temperature
            ));
            severity = ConstraintSeverity::Error;
        }

        let tokens_delta = (proposed.max_tokens as i64 - baseline.max_tokens as i64).abs();
        if tokens_delta > constraints.max_delta_max_tokens as i64 {
            violations.push(format!(
                "Token delta {} exceeds trust region {}",
                tokens_delta, constraints.max_delta_max_tokens
            ));
            severity = ConstraintSeverity::Error;
        }

        // 2. Hard constraint checks
        if proposed.max_tokens > constraints.max_tokens {
            violations.push(format!(
                "Token limit {} exceeds constraint {}",
                proposed.max_tokens, constraints.max_tokens
            ));
            severity = ConstraintSeverity::Error;
        }

        // 3. Quality floor check (if we have historical data)
        if let Some(avg_quality) = self.get_expected_quality(proposed) {
            if avg_quality < constraints.min_quality {
                violations.push(format!(
                    "Expected quality {:.3} below threshold {:.3}",
                    avg_quality, constraints.min_quality
                ));
                severity = ConstraintSeverity::Warning;
            }
        }

        // 4. CAWS compliance check
        if constraints.require_caws_compliance {
            if !self.check_caws_compliance(proposed) {
                violations.push("CAWS compliance check failed".to_string());
                severity = ConstraintSeverity::Error;
            }
        }

        ConstraintCheckResult {
            passed: violations.is_empty(),
            violations,
            severity,
        }
    }

    /// Get expected quality for a parameter set (placeholder)
    fn get_expected_quality(&self, _params: &crate::bandit_policy::ParameterSet) -> Option<f64> {
        // In a real implementation, this would query historical performance data
        // For now, return None to indicate no historical data
        None
    }

    /// Check CAWS compliance for parameters
    fn check_caws_compliance(&self, params: &crate::bandit_policy::ParameterSet) -> bool {
        // Basic CAWS compliance checks
        // 1. Temperature within reasonable bounds
        if params.temperature < 0.0 || params.temperature > 2.0 {
            return false;
        }

        // 2. Token limits reasonable
        if params.max_tokens == 0 || params.max_tokens > 10000 {
            return false;
        }

        // 3. Optional parameters within bounds
        if let Some(top_p) = params.top_p {
            if top_p < 0.0 || top_p > 1.0 {
                return false;
            }
        }

        if let Some(freq_penalty) = params.frequency_penalty {
            if freq_penalty < -2.0 || freq_penalty > 2.0 {
                return false;
            }
        }

        if let Some(pres_penalty) = params.presence_penalty {
            if pres_penalty < -2.0 || pres_penalty > 2.0 {
                return false;
            }
        }

        true
    }

    /// Calculate confidence interval for reward estimate
    pub fn confidence_interval(
        &self,
        outcome: &TaskOutcome,
        baseline: &BaselineMetrics,
        confidence_level: f64,
    ) -> (f64, f64) {
        let reward = self.calculate(outcome, baseline).reward;
        
        // Simplified confidence interval calculation
        // In practice, this would use proper statistical methods
        let margin = 0.1 * reward.abs(); // 10% margin
        let lower = reward - margin;
        let upper = reward + margin;
        
        (lower, upper)
    }

    /// Get reward function version for provenance
    pub fn version(&self) -> String {
        "reward_function@1.0.0".to_string()
    }
}

impl Default for RewardFunction {
    fn default() -> Self {
        Self::new(ObjectiveWeights::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reward_calculation() {
        let weights = ObjectiveWeights {
            w_quality: 1.0,
            w_latency: 0.1,
            w_tokens: 0.01,
        };
        let reward_fn = RewardFunction::new(weights);

        let outcome = TaskOutcome {
            quality_score: 0.8,
            latency_ms: 1000,
            tokens_used: 500,
            success: true,
            caws_compliance: true,
        };

        let baseline = BaselineMetrics {
            avg_quality: 0.7,
            avg_latency: 1200,
            avg_tokens: 600.0,
            temperature: 0.7,
            max_tokens: 1000,
        };

        let result = reward_fn.calculate(&outcome, &baseline);
        
        assert!(result.reward > 0.0, "Reward should be positive for good outcome");
        assert!(result.components.quality_contrib > 0.0, "Quality contribution should be positive");
    }

    #[test]
    fn test_constraint_checking() {
        let reward_fn = RewardFunction::default();
        let constraints = OptimizationConstraints::default();
        
        let baseline = BaselineMetrics {
            avg_quality: 0.7,
            avg_latency: 1200,
            avg_tokens: 600.0,
            temperature: 0.7,
            max_tokens: 1000,
        };

        let params = crate::bandit_policy::ParameterSet {
            temperature: 0.5, // Within trust region
            max_tokens: 800,  // Within trust region
            top_p: Some(0.9),
            frequency_penalty: None,
            presence_penalty: None,
            stop_sequences: vec![],
            seed: None,
            origin: "test".to_string(),
            policy_version: "1.0.0".to_string(),
            created_at: chrono::Utc::now(),
        };

        let result = reward_fn.check_constraints(&params, &constraints, &baseline);
        assert!(result.passed, "Valid parameters should pass constraint check");
    }
}
