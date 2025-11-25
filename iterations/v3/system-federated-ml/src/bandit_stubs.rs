//! Bandit Policy Stubs
//!
//! Provides stub implementations of bandit policy types when the `bandit_policy`
//! feature is not enabled. These stubs allow dependent modules to compile without
//! the full bandit policy implementation.
//!
//! @author @darianrosebrook

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Stub ParameterSet for when bandit_policy feature is disabled
/// 
/// This is a re-export from bandit_policy when the feature is enabled,
/// or a stub when disabled. For full functionality, enable the `bandit_policy` feature.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ParameterSet {
    pub temperature: f32,
    pub max_tokens: u32,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub stop_sequences: Vec<String>,
    pub seed: Option<u64>,
    pub origin: String,
    pub policy_version: String,
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
    /// Execution ID for tracking
    pub execution_id: Option<String>,
    /// Parameters map for dynamic values
    pub parameters: std::collections::HashMap<String, serde_json::Value>,
    /// Execution mode (e.g., "sync", "async")
    pub execution_mode: Option<String>,
    /// Quality gates configuration
    pub quality_gates: Option<serde_json::Value>,
}

impl Default for ParameterSet {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            max_tokens: 4096,
            top_p: Some(0.9),
            frequency_penalty: Some(0.0),
            presence_penalty: Some(0.0),
            stop_sequences: Vec::new(),
            seed: None,
            origin: "stub".to_string(),
            policy_version: "0.0.0".to_string(),
            created_at: Utc::now(),
            execution_id: None,
            parameters: std::collections::HashMap::new(),
            execution_mode: None,
            quality_gates: None,
        }
    }
}

/// Stub TaskFeatures for when bandit_policy feature is disabled
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskFeatures {
    pub risk_tier: u32,
    pub title_length: u32,
    pub description_length: u32,
    pub acceptance_criteria_count: u32,
    pub scope_files_count: u32,
    pub max_files: u32,
    pub max_loc: u32,
    pub has_external_deps: bool,
    pub complexity_indicators: Vec<String>,
    pub model_name: Option<String>,
    pub prompt_tokens: Option<u32>,
    pub prior_failures: Option<u32>,
    /// Context fingerprint for caching and deduplication
    pub context_fingerprint: u64,
}

impl Default for TaskFeatures {
    fn default() -> Self {
        Self {
            risk_tier: 2,
            title_length: 0,
            description_length: 0,
            acceptance_criteria_count: 0,
            scope_files_count: 0,
            max_files: 25,
            max_loc: 1000,
            has_external_deps: false,
            complexity_indicators: Vec::new(),
            model_name: None,
            prompt_tokens: None,
            prior_failures: None,
            context_fingerprint: 0,
        }
    }
}

impl TaskFeatures {
    /// Stable fingerprint for counterfactual logging (stub implementation)
    pub fn fingerprint(&self) -> u64 {
        use blake3::Hasher as B3;
        let mut h = B3::new();
        h.update(&self.risk_tier.to_le_bytes());
        h.update(&self.title_length.to_le_bytes());
        h.update(&self.description_length.to_le_bytes());
        h.update(&self.acceptance_criteria_count.to_le_bytes());
        h.update(&self.scope_files_count.to_le_bytes());
        h.update(&self.max_files.to_le_bytes());
        h.update(&self.max_loc.to_le_bytes());
        h.update(&[self.has_external_deps as u8]);
        u64::from_le_bytes(h.finalize().as_bytes()[..8].try_into().unwrap())
    }
}

/// Stub SelectionResult for when bandit_policy feature is disabled
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SelectionResult {
    pub arm_index: usize,
    pub parameters: ParameterSet,
    pub propensity: f64,
    pub confidence: f64,
    pub reasoning: Vec<String>,
}

/// Stub BanditPolicy trait for when bandit_policy feature is disabled
pub trait BanditPolicy: Send + Sync {
    /// Select arm (parameter set) given context
    fn select(&self, ctx: &TaskFeatures, arms: &[ParameterSet]) -> SelectionResult;

    /// Update policy with observed outcome
    fn update(&mut self, ctx: &TaskFeatures, arm: &ParameterSet, reward: f64);

    /// Get policy version for provenance
    fn version(&self) -> String;
}

/// Stub ThompsonGaussian bandit policy
pub struct ThompsonGaussian {
    prior_mean: f64,
    prior_precision: f64,
}

impl ThompsonGaussian {
    pub fn new() -> Self {
        Self::with_params(0.0, 1.0, 1.0)
    }
    
    pub fn with_params(prior_mean: f64, prior_precision: f64, _noise_precision: f64) -> Self {
        Self {
            prior_mean,
            prior_precision,
        }
    }
}

impl Default for ThompsonGaussian {
    fn default() -> Self {
        Self::new()
    }
}

impl BanditPolicy for ThompsonGaussian {
    fn select(&self, _ctx: &TaskFeatures, arms: &[ParameterSet]) -> SelectionResult {
        // Stub: always select first arm
        SelectionResult {
            arm_index: 0,
            parameters: arms.first().cloned().unwrap_or_default(),
            propensity: 1.0 / arms.len().max(1) as f64,
            confidence: 0.5,
            reasoning: vec!["Stub selection".to_string()],
        }
    }

    fn update(&mut self, _ctx: &TaskFeatures, _arm: &ParameterSet, _reward: f64) {
        // Stub: no-op
    }

    fn version(&self) -> String {
        "stub-thompson-0.1.0".to_string()
    }
}

/// Stub LinUCB bandit policy
pub struct LinUCB {
    alpha: f64,
}

impl LinUCB {
    pub fn new(alpha: f64, _feature_dim: usize) -> Self {
        Self { alpha }
    }
}

impl BanditPolicy for LinUCB {
    fn select(&self, _ctx: &TaskFeatures, arms: &[ParameterSet]) -> SelectionResult {
        // Stub: always select first arm
        SelectionResult {
            arm_index: 0,
            parameters: arms.first().cloned().unwrap_or_default(),
            propensity: 1.0 / arms.len().max(1) as f64,
            confidence: 0.5,
            reasoning: vec!["Stub LinUCB selection".to_string()],
        }
    }

    fn update(&mut self, _ctx: &TaskFeatures, _arm: &ParameterSet, _reward: f64) {
        // Stub: no-op
    }

    fn version(&self) -> String {
        "stub-linucb-0.1.0".to_string()
    }
}

/// Stub LoggedDecision for counterfactual logging
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LoggedDecision {
    pub decision_id: uuid::Uuid,
    pub task_type: String,
    pub context: TaskFeatures,
    pub chosen_params: ParameterSet,
    pub propensity: f64,
    pub policy_version: String,
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    pub outcome: Option<crate::reward::TaskOutcome>,
}

impl LoggedDecision {
    pub fn new(
        task_type: String,
        context: TaskFeatures,
        chosen_params: ParameterSet,
        propensity: f64,
        policy_version: String,
    ) -> Self {
        Self {
            decision_id: uuid::Uuid::new_v4(),
            task_type,
            context,
            chosen_params,
            propensity,
            policy_version,
            timestamp: Utc::now(),
            outcome: None,
        }
    }
}

/// Stub PolicyEvaluationResult for offline policy evaluation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PolicyEvaluationResult {
    pub policy_version: String,
    pub estimated_reward: f64,
    pub confidence_interval: (f64, f64),
    pub sample_size: usize,
    pub effective_sample_size: f64,
}

impl Default for PolicyEvaluationResult {
    fn default() -> Self {
        Self {
            policy_version: "stub-0.1.0".to_string(),
            estimated_reward: 0.0,
            confidence_interval: (0.0, 0.0),
            sample_size: 0,
            effective_sample_size: 0.0,
        }
    }
}

/// Stub CounterfactualLogger for decision logging
pub struct CounterfactualLogger {
    decisions: std::sync::RwLock<Vec<LoggedDecision>>,
}

impl CounterfactualLogger {
    pub fn new() -> Self {
        Self {
            decisions: std::sync::RwLock::new(Vec::new()),
        }
    }

    /// Log a decision
    pub fn log_decision(&self, decision: LoggedDecision) {
        if let Ok(mut decisions) = self.decisions.write() {
            decisions.push(decision);
        }
    }

    /// Record outcome for a decision
    pub fn record_outcome(&self, decision_id: uuid::Uuid, outcome: crate::reward::TaskOutcome) {
        if let Ok(mut decisions) = self.decisions.write() {
            if let Some(decision) = decisions.iter_mut().find(|d| d.decision_id == decision_id) {
                decision.outcome = Some(outcome);
            }
        }
    }

    /// Get all decisions
    pub fn get_decisions(&self) -> Vec<LoggedDecision> {
        self.decisions.read().map(|d| d.clone()).unwrap_or_default()
    }

    /// Evaluate a policy offline using logged decisions
    pub fn evaluate_policy(
        &self,
        _policy: &dyn BanditPolicy,
        _task_type: &str,
    ) -> PolicyEvaluationResult {
        PolicyEvaluationResult::default()
    }
    
    /// Get an evaluator for offline policy evaluation
    pub fn evaluator(&self) -> PolicyEvaluator {
        PolicyEvaluator::new()
    }
}

/// Stub PolicyEvaluator for offline policy evaluation
pub struct PolicyEvaluator {
    decisions: Vec<LoggedDecision>,
}

impl PolicyEvaluator {
    pub fn new() -> Self {
        Self { decisions: Vec::new() }
    }
    
    /// Get decisions for a task type
    pub fn get_decisions(&self, _task_type: &str) -> Result<Vec<LoggedDecision>, anyhow::Error> {
        Ok(self.decisions.clone())
    }
    
    /// Evaluate using Inverse Propensity Scoring (IPS)
    pub fn evaluate_ips(
        &self,
        _policy: &dyn BanditPolicy,
        _task_type: &str,
    ) -> Result<PolicyEvaluationResult, anyhow::Error> {
        Ok(PolicyEvaluationResult::default())
    }
}

impl Default for PolicyEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CounterfactualLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// Parameters used for LLM generation - stub version for when orchestration module unavailable
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UsedParameters {
    /// Model name used for generation
    pub model_name: Option<String>,
    /// Temperature parameter
    pub temperature: f32,
    /// Maximum tokens to generate
    pub max_tokens: u32,
    /// Top-p sampling parameter
    pub top_p: f32,
    /// Frequency penalty
    pub frequency_penalty: Option<f32>,
    /// Presence penalty
    pub presence_penalty: Option<f32>,
    /// Stop sequences
    pub stop_sequences: Vec<String>,
    /// Random seed for reproducibility
    pub seed: Option<u64>,
    /// Origin of parameters (e.g., "bandit_policy", "default")
    pub origin: String,
    /// Policy version that generated these parameters
    pub policy_version: Option<String>,
    /// Timestamp when parameters were created
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
}

impl Default for UsedParameters {
    fn default() -> Self {
        Self {
            model_name: Some("gpt-4".to_string()),
            temperature: 0.7,
            max_tokens: 4096,
            top_p: 0.9,
            frequency_penalty: Some(0.0),
            presence_penalty: Some(0.0),
            stop_sequences: Vec::new(),
            seed: None,
            origin: "default".to_string(),
            policy_version: None,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_set_default() {
        let params = ParameterSet::default();
        assert_eq!(params.temperature, 0.7);
        assert_eq!(params.max_tokens, 4096);
        assert_eq!(params.origin, "stub");
    }

    #[test]
    fn test_task_features_fingerprint() {
        let features = TaskFeatures::default();
        let fp = features.fingerprint();
        assert_ne!(fp, 0);
    }

    #[test]
    fn test_thompson_gaussian_select() {
        let policy = ThompsonGaussian::with_params(0.0, 1.0, 1.0);
        let arms = vec![ParameterSet::default()];
        let result = policy.select(&TaskFeatures::default(), &arms);
        assert_eq!(result.arm_index, 0);
    }
}

