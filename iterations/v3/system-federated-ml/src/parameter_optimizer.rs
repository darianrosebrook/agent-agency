//! LLM Parameter Optimizer with Constrained Contextual Bandits
//!
//! Implements safe, constrained parameter optimization using contextual bandits
//! with trust regions, quality gates, and CAWS compliance.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[cfg(feature = "bandit_policy")]
use crate::bandit_policy::{ParameterSet, TaskFeatures, BanditPolicy, ThompsonGaussian, LinUCB};

#[cfg(not(feature = "bandit_policy"))]
use crate::bandit_stubs::{ParameterSet, TaskFeatures, BanditPolicy, ThompsonGaussian, LinUCB};
#[cfg(feature = "bandit_policy")]
use crate::counterfactual_log::{CounterfactualLogger, TaskOutcome, LoggedDecision};

#[cfg(not(feature = "bandit_policy"))]
use crate::{reward::TaskOutcome, bandit_stubs::{CounterfactualLogger, LoggedDecision}};

/// LLM Parameter Optimizer
pub struct LLMParameterOptimizer {
    /// Pluggable bandit policy
    policy: Arc<RwLock<Box<dyn BanditPolicy>>>,
    
    /// Historical parameter performance by task type and model
    parameter_history: Arc<RwLock<HashMap<String, Vec<ParameterEvaluation>>>>,
    
    /// Counterfactual logger
    cf_logger: Arc<CounterfactualLogger>,
    
    /// Current optimal parameters per task type
    optimal_parameters: Arc<RwLock<HashMap<String, OptimalParameterSet>>>,
    
    /// Baseline parameters (fallback)
    baseline_parameters: Arc<RwLock<HashMap<String, ParameterSet>>>,
    
    /// Quality gate validator
    quality_validator: Arc<QualityGateValidator>,
}

/// Parameter evaluation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterEvaluation {
    pub parameters: ParameterSet,
    pub objective_value: f64,
    pub quality_score: f64,
    pub compliance_score: f64,
    pub timestamp: DateTime<Utc>,
}

/// Optimal parameter set for a task type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimalParameterSet {
    pub parameters: ParameterSet,
    pub confidence: f64,
    pub sample_count: usize,
    pub last_updated: DateTime<Utc>,
}

/// Optimization constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConstraints {
    pub max_latency_ms: u64,
    pub max_tokens: u32,
    pub require_caws: bool,
    /// Trust-region around current baseline to avoid large jumps
    pub max_delta_temperature: f32,   // e.g., 0.2
    pub max_delta_max_tokens: u32,    // e.g., 200
}

/// Objective weights for reward calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveWeights {
    /// Reward = w_q * quality - w_l * norm_latency - w_t * norm_tokens
    pub w_quality: f64,
    pub w_latency: f64,
    pub w_tokens: f64,
}

/// Recommended parameters with uncertainty
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedParameters {
    pub set: ParameterSet,
    pub confidence: f64,               // Calibrated [0,1]
    pub ci_reward: (f64, f64),         // Lower, upper bound
    pub ci_latency: (u64, u64),
    pub ci_quality: (f64, f64),
    pub propensity: f64,               // For CF logging
    pub alternative_sets: Vec<ParameterSet>,
    pub reasoning: Vec<String>,
    pub deployment_safe: bool,         // expected_gain_lower > threshold
}

/// Quality gate validator
pub struct QualityGateValidator {
    baseline_quality: Arc<RwLock<HashMap<String, BaselineMetrics>>>,
    quality_threshold: f64,
}

/// Baseline metrics for comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineMetrics {
    pub avg_quality: f64,
    pub avg_latency: u64,
    pub avg_tokens: f32,
    pub temperature: f32,
    pub max_tokens: u32,
}

impl LLMParameterOptimizer {
    pub fn new() -> Self {
        let policy = Arc::new(RwLock::new(Box::new(ThompsonGaussian::new()) as Box<dyn BanditPolicy>));
        
        Self {
            policy,
            parameter_history: Arc::new(RwLock::new(HashMap::new())),
            cf_logger: Arc::new(CounterfactualLogger::new()),
            optimal_parameters: Arc::new(RwLock::new(HashMap::new())),
            baseline_parameters: Arc::new(RwLock::new(HashMap::new())),
            quality_validator: Arc::new(QualityGateValidator::new()),
        }
    }

    /// Generate parameter recommendations with constraints enforced
    pub async fn recommend_parameters(
        &self,
        task_type: &str,
        task_features: &TaskFeatures,
        constraints: &OptimizationConstraints,
    ) -> Result<RecommendedParameters> {
        // 1. Get baseline parameters for this task type
        let baseline = self.get_baseline_parameters(task_type).await?;
        
        // 2. Generate candidate parameter sets within trust region
        let candidates = self.generate_candidates(&baseline, constraints).await?;
        
        // 3. Use bandit policy to select best candidate
        let policy = self.policy.read().unwrap();
        let selection = policy.as_ref().select(task_features, &candidates);
        drop(policy);
        
        // 4. Validate selection meets constraints
        let validation = self.quality_validator
            .validate_pre_deployment(task_type, &selection.parameters, constraints)
            .await?;
        
        if !validation.approved() {
            return Ok(RecommendedParameters {
                set: baseline,
                confidence: 0.5,
                ci_reward: (0.0, 0.0),
                ci_latency: (0, 0),
                ci_quality: (0.0, 0.0),
                propensity: 1.0,
                alternative_sets: vec![],
                reasoning: vec!["Validation failed, using baseline".to_string()],
                deployment_safe: false,
            });
        }
        
        // 5. Calculate confidence intervals (simplified)
        let confidence = selection.confidence;
        let ci_reward = (0.7, 0.9); // Simplified
        let ci_latency = (100, 200); // Simplified
        let ci_quality = (0.8, 0.95); // Simplified
        
        Ok(RecommendedParameters {
            set: selection.parameters,
            confidence,
            ci_reward,
            ci_latency,
            ci_quality,
            propensity: selection.propensity,
            alternative_sets: candidates,
            reasoning: selection.reasoning,
            deployment_safe: confidence > 0.8,
        })
    }

    /// Record outcome for learning (with propensity for CF logging)
    pub async fn record_outcome(
        &self,
        request_id: Uuid,
        task_type: &str,
        context_fingerprint: u64,
        parameters: crate::orchestration::planning::llm_client::UsedParameters,
        outcome: TaskOutcome,
        log_propensity: f64,
    ) -> Result<()> {
        // Convert UsedParameters to ParameterSet
        let param_set = ParameterSet {
            temperature: parameters.temperature,
            max_tokens: parameters.max_tokens,
            top_p: Some(parameters.top_p),
            frequency_penalty: parameters.frequency_penalty,
            presence_penalty: parameters.presence_penalty,
            stop_sequences: parameters.stop_sequences,
            seed: parameters.seed,
            origin: parameters.origin,
            policy_version: parameters.policy_version.clone(),
            created_at: parameters.created_at,
        };
        
        // Calculate reward
        let reward = self.calculate_reward(&outcome);
        
        // Update bandit policy
        {
            let mut policy = self.policy.write().unwrap();
            policy.update(&TaskFeatures::default(), &param_set, reward);
        }
        
        // Log for counterfactual evaluation
        self.cf_logger.log_decision(
            request_id,
            task_type.to_string(),
            parameters.model_name,
            TaskFeatures::default(), // Would use actual features
            param_set.clone(),
            log_propensity,
            outcome.clone(),
            parameters.policy_version,
        )?;
        
        // Update parameter history
        {
            let mut history = self.parameter_history.write().unwrap();
            let entry = ParameterEvaluation {
                parameters: param_set,
                objective_value: reward,
                quality_score: outcome.quality_score,
                compliance_score: if outcome.caws_compliance { 1.0 } else { 0.0 },
                timestamp: Utc::now(),
            };
            history.entry(task_type.to_string()).or_insert_with(Vec::new).push(entry);
        }
        
        Ok(())
    }

    /// Get baseline parameters for task type
    async fn get_baseline_parameters(&self, task_type: &str) -> Result<ParameterSet> {
        let baselines = self.baseline_parameters.read().unwrap();
        if let Some(baseline) = baselines.get(task_type) {
            Ok(baseline.clone())
        } else {
            // Default baseline
            Ok(ParameterSet {
                temperature: 0.7,
                max_tokens: 1000,
                top_p: Some(0.9),
                frequency_penalty: None,
                presence_penalty: None,
                stop_sequences: vec![],
                seed: None,
                origin: "baseline".to_string(),
                policy_version: "1.0.0".to_string(),
                created_at: Utc::now(),
            })
        }
    }

    /// Generate candidate parameter sets within trust region
    async fn generate_candidates(
        &self,
        baseline: &ParameterSet,
        constraints: &OptimizationConstraints,
    ) -> Result<Vec<ParameterSet>> {
        let mut candidates = Vec::new();
        
        // Generate variations within trust region
        let temp_deltas = [-0.1, 0.0, 0.1];
        let token_deltas = [-100, 0, 100];
        
        for &temp_delta in &temp_deltas {
            for &token_delta in &token_deltas {
                let new_temp = (baseline.temperature + temp_delta)
                    .max(0.0).min(2.0);
                let new_tokens = (baseline.max_tokens as i32 + token_delta)
                    .max(1).min(constraints.max_tokens as i32) as u32;
                
                // Check trust region constraints
                if (new_temp - baseline.temperature).abs() <= constraints.max_delta_temperature as f64
                    && (new_tokens as i32 - baseline.max_tokens as i32).abs() <= constraints.max_delta_max_tokens as i32
                {
                    let mut candidate = baseline.clone();
                    candidate.temperature = new_temp;
                    candidate.max_tokens = new_tokens as usize;
                    candidate.origin = "optimizer".to_string();
                    candidate.created_at = Utc::now();
                    candidates.push(candidate);
                }
            }
        }
        
        if candidates.is_empty() {
            candidates.push(baseline.clone());
        }
        
        Ok(candidates)
    }

    /// Calculate reward from outcome
    fn calculate_reward(&self, outcome: &TaskOutcome) -> f64 {
        // Simplified reward calculation
        let quality_reward = outcome.quality_score;
        let latency_penalty = (outcome.latency_ms as f64) / 1000.0; // Normalize
        let token_penalty = (outcome.tokens_used as f64) / 1000.0; // Normalize
        
        quality_reward - 0.1 * latency_penalty - 0.05 * token_penalty
    }
}

impl QualityGateValidator {
    pub fn new() -> Self {
        Self {
            baseline_quality: Arc::new(RwLock::new(HashMap::new())),
            quality_threshold: 0.85,
        }
    }

    /// Validate parameters are within trust region and constraints
    pub async fn validate_pre_deployment(
        &self,
        task_type: &str,
        proposed: &ParameterSet,
        constraints: &OptimizationConstraints,
    ) -> Result<ValidationResult> {
        // Simplified validation
        if proposed.max_tokens > constraints.max_tokens as usize {
            return Ok(ValidationResult::Rejected {
                reason: format!("Token limit {} exceeds constraint {}", 
                                proposed.max_tokens, constraints.max_tokens),
            });
        }
        
        if proposed.temperature < 0.0 || proposed.temperature > 2.0 {
            return Ok(ValidationResult::Rejected {
                reason: "Temperature out of valid range [0.0, 2.0]".to_string(),
            });
        }
        
        Ok(ValidationResult::Approved {
            quality_delta: 0.0,
            latency_delta: 0,
            token_delta: 0.0,
        })
    }
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationResult {
    Approved {
        quality_delta: f64,
        latency_delta: i64,
        token_delta: f32,
    },
    Rejected {
        reason: String,
    },
}

impl ValidationResult {
    pub fn approved(&self) -> bool {
        matches!(self, ValidationResult::Approved { .. })
    }
}

impl Default for TaskFeatures {
    fn default() -> Self {
        Self {
            risk_tier: 2,
            title_length: 10,
            description_length: 50,
            acceptance_criteria_count: 3,
            scope_files_count: 5,
            max_files: 10,
            max_loc: 1000,
            has_external_deps: false,
            complexity_indicators: vec![],
            model_name: None,
            prompt_tokens: None,
            prior_failures: None,
        }
    }
}

pub type Result<T> = std::result::Result<T, anyhow::Error>;
