//! Counterfactual Logging for Offline Policy Evaluation
//!
//! Implements counterfactual logging with propensity scoring for offline
//! policy evaluation using IPS (Inverse Propensity Scoring) and DR (Doubly Robust) estimators.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[cfg(feature = "bandit_policy")]
use crate::bandit_policy::{ParameterSet, TaskFeatures, BanditPolicy};

#[cfg(not(feature = "bandit_policy"))]
use crate::bandit_stubs::{ParameterSet, TaskFeatures, BanditPolicy};

/// Logged decision for offline policy evaluation (IPS/DR)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggedDecision {
    pub request_id: Uuid,
    pub task_type: String,
    pub model_name: String,
    pub context_fingerprint: u64,
    pub context_features: TaskFeatures,
    pub chosen_params: ParameterSet,
    pub log_propensity: f64,           // π(a|x) - probability policy assigned
    pub outcome: TaskOutcome,
    pub policy_version: String,
    pub timestamp: DateTime<Utc>,
}

/// Task outcome metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskOutcome {
    pub quality_score: f64,            // 0.0-1.0
    pub latency_ms: u64,
    pub tokens_used: usize,
    pub success: bool,
    pub caws_compliance: bool,
}

/// Offline policy evaluator using IPS/DR estimators
pub struct OfflineEvaluator {
    logged_corpus: Arc<RwLock<Vec<LoggedDecision>>>,
}

impl OfflineEvaluator {
    pub fn new() -> Self {
        Self {
            logged_corpus: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add a logged decision to the corpus
    pub fn add_decision(&self, decision: LoggedDecision) -> Result<()> {
        let mut corpus = self.logged_corpus.write().unwrap();
        corpus.push(decision);
        Ok(())
    }

    /// Evaluate a new policy against logged corpus using Inverse Propensity Scoring
    pub fn evaluate_ips(
        &self,
        new_policy: &dyn BanditPolicy,
        task_type: &str,
    ) -> Result<PolicyEvaluationResult> {
        let corpus = self.logged_corpus.read().unwrap();
        let relevant_decisions: Vec<_> = corpus.iter()
            .filter(|d| d.task_type == task_type)
            .collect();

        if relevant_decisions.is_empty() {
            return Ok(PolicyEvaluationResult {
                estimated_reward: 0.0,
                confidence_interval: (0.0, 0.0),
                sample_size: 0,
                effective_sample_size: 0.0,
            });
        }

        // Calculate IPS estimator with proper propensity calculation
        let mut weighted_rewards = Vec::new();
        let mut total_weight = 0.0;
        let mut effective_sample_size = 0.0;

        for decision in &relevant_decisions {
            // Calculate new policy propensity for this context
            let new_propensity = self.calculate_new_propensity(new_policy, decision)?;
            
            // IPS weight: π_new(a|x) / π_old(a|x)
            let old_propensity = decision.log_propensity.exp();
            let ips_weight = new_propensity / old_propensity;
            
            // Weighted reward
            let weighted_reward = ips_weight * decision.outcome.quality_score;
            weighted_rewards.push(weighted_reward);
            total_weight += ips_weight;
            effective_sample_size += ips_weight;
        }

        if total_weight == 0.0 {
            return Ok(PolicyEvaluationResult {
                estimated_reward: 0.0,
                confidence_interval: (0.0, 0.0),
                sample_size: relevant_decisions.len(),
                effective_sample_size: 0.0,
            });
        }

        // Calculate IPS estimate
        let ips_estimate = weighted_rewards.iter().sum::<f64>() / total_weight;
        
        // Calculate confidence interval using bootstrap
        let confidence_interval = self.bootstrap_confidence_interval(&weighted_rewards, ips_estimate)?;

        Ok(PolicyEvaluationResult {
            estimated_reward: ips_estimate,
            confidence_interval,
            sample_size: relevant_decisions.len(),
            effective_sample_size,
        })
    }

    /// Evaluate using Doubly Robust estimator (IPS + model-based)
    pub fn evaluate_doubly_robust(
        &self,
        new_policy: &dyn BanditPolicy,
        outcome_model: &OutcomeModel,
        task_type: &str,
    ) -> Result<PolicyEvaluationResult> {
        let corpus = self.logged_corpus.read().unwrap();
        let relevant_decisions: Vec<_> = corpus.iter()
            .filter(|d| d.task_type == task_type)
            .collect();

        if relevant_decisions.is_empty() {
            return Ok(PolicyEvaluationResult {
                estimated_reward: 0.0,
                confidence_interval: (0.0, 0.0),
                sample_size: 0,
                effective_sample_size: 0.0,
            });
        }

        // Calculate Doubly Robust estimator
        let mut dr_estimates = Vec::new();
        let mut total_weight = 0.0;
        let mut effective_sample_size = 0.0;

        for decision in &relevant_decisions {
            // Calculate new policy propensity
            let new_propensity = self.calculate_new_propensity(new_policy, decision)?;
            let old_propensity = decision.log_propensity.exp();
            let ips_weight = new_propensity / old_propensity;
            
            // Model-based prediction
            let model_prediction = outcome_model.predict(&decision.context_features, &decision.chosen_params);
            
            // Doubly Robust: IPS + model correction
            let dr_estimate = ips_weight * decision.outcome.quality_score 
                + (new_propensity - ips_weight) * model_prediction;
            
            dr_estimates.push(dr_estimate);
            total_weight += new_propensity;
            effective_sample_size += ips_weight;
        }

        if total_weight == 0.0 {
            return Ok(PolicyEvaluationResult {
                estimated_reward: 0.0,
                confidence_interval: (0.0, 0.0),
                sample_size: relevant_decisions.len(),
                effective_sample_size: 0.0,
            });
        }

        let dr_estimate = dr_estimates.iter().sum::<f64>() / total_weight;
        let confidence_interval = self.bootstrap_confidence_interval(&dr_estimates, dr_estimate)?;

        Ok(PolicyEvaluationResult {
            estimated_reward: dr_estimate,
            confidence_interval,
            sample_size: relevant_decisions.len(),
            effective_sample_size,
        })
    }

    /// Get corpus size for monitoring
    pub fn corpus_size(&self) -> usize {
        self.logged_corpus.read().unwrap().len()
    }

    /// Calculate propensity for new policy on a logged decision
    fn calculate_new_propensity(
        &self,
        new_policy: &dyn BanditPolicy,
        decision: &LoggedDecision,
    ) -> Result<f64> {
        // Create a mock arm set with the chosen parameters
        let arms = vec![decision.chosen_params.clone()];
        
        // Get selection result from new policy
        let selection = new_policy.select(&decision.context_features, &arms);
        
        // Return the propensity (probability) of selecting the same arm
        Ok(selection.propensity)
    }

    /// Calculate bootstrap confidence interval
    fn bootstrap_confidence_interval(
        &self,
        estimates: &[f64],
        point_estimate: f64,
    ) -> Result<(f64, f64)> {
        if estimates.is_empty() {
            return Ok((0.0, 0.0));
        }

        // Simple bootstrap: resample with replacement and calculate percentiles
        let n_bootstrap = 1000;
        let mut bootstrap_estimates = Vec::new();
        
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        for _ in 0..n_bootstrap {
            let mut bootstrap_sum = 0.0;
            for _ in 0..estimates.len() {
                let idx = rng.gen_range(0..estimates.len());
                bootstrap_sum += estimates[idx];
            }
            bootstrap_estimates.push(bootstrap_sum / estimates.len() as f64);
        }
        
        bootstrap_estimates.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let lower_idx = (0.025 * n_bootstrap as f64) as usize;
        let upper_idx = (0.975 * n_bootstrap as f64) as usize;
        
        let lower_bound = bootstrap_estimates[lower_idx.min(bootstrap_estimates.len() - 1)];
        let upper_bound = bootstrap_estimates[upper_idx.min(bootstrap_estimates.len() - 1)];
        
        Ok((lower_bound, upper_bound))
    }

    /// Get decisions for a specific task type
    pub fn get_decisions(&self, task_type: &str) -> Vec<LoggedDecision> {
        let corpus = self.logged_corpus.read().unwrap();
        corpus.iter()
            .filter(|d| d.task_type == task_type)
            .cloned()
            .collect()
    }
}

/// Policy evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEvaluationResult {
    pub estimated_reward: f64,
    pub confidence_interval: (f64, f64),  // Lower, upper at α=0.05
    pub sample_size: usize,
    pub effective_sample_size: f64,       // After propensity weighting
}

/// Outcome model for doubly robust estimation
pub struct OutcomeModel {
    // Simplified outcome model
    // In practice, this would be a trained ML model
}

impl OutcomeModel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn predict(&self, _features: &TaskFeatures, _params: &ParameterSet) -> f64 {
        // Simplified prediction
        0.8 // Would use actual model
    }
}

/// Counterfactual logger for real-time decision logging
pub struct CounterfactualLogger {
    evaluator: Arc<OfflineEvaluator>,
}

impl CounterfactualLogger {
    pub fn new() -> Self {
        Self {
            evaluator: Arc::new(OfflineEvaluator::new()),
        }
    }

    /// Log a decision with outcome
    pub async fn log_decision(
        &self,
        request_id: Uuid,
        task_type: String,
        model_name: String,
        context_features: TaskFeatures,
        chosen_params: ParameterSet,
        log_propensity: f64,
        outcome: TaskOutcome,
        policy_version: String,
    ) -> Result<()> {
        let decision = LoggedDecision {
            request_id,
            task_type,
            model_name,
            context_fingerprint: context_features.fingerprint(),
            context_features,
            chosen_params,
            log_propensity,
            outcome,
            policy_version,
            timestamp: Utc::now(),
        };

        self.evaluator.add_decision(decision)?;
        Ok(())
    }

    /// Get the underlying evaluator
    pub fn evaluator(&self) -> Arc<OfflineEvaluator> {
        self.evaluator.clone()
    }
}

pub type Result<T> = std::result::Result<T, anyhow::Error>;
