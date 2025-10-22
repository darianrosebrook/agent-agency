//! Bandit Policy Interface for LLM Parameter Optimization
//!
//! Implements contextual bandit policies for safe, interpretable parameter tuning
//! with counterfactual logging and offline evaluation support.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Parameter set for LLM generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSet {
    pub temperature: f32,
    pub max_tokens: u32,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub stop_sequences: Vec<String>,
    pub seed: Option<u64>,
    pub origin: String,          // e.g., "bandit:thompson@0.1.0"
    pub policy_version: String,  // semver of learner
    pub created_at: DateTime<Utc>,
}

/// Task features for contextual bandit learning
#[derive(Debug, Clone, Serialize, Deserialize)]
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

    // NEW: model & prompt context that strongly drive optimal params
    pub model_name: Option<String>,
    pub prompt_tokens: Option<u32>,
    pub prior_failures: Option<u32>,
}

impl TaskFeatures {
    /// Stable 64-bit fingerprint for counterfactual logging/stratification
    pub fn fingerprint(&self) -> u64 {
        use blake3::Hasher as B3;
        let mut h = B3::new();
        // keep order stable; avoid serializing free-form strings directly
        h.update(&self.risk_tier.to_le_bytes());
        h.update(&self.title_length.to_le_bytes());
        h.update(&self.description_length.to_le_bytes());
        h.update(&self.acceptance_criteria_count.to_le_bytes());
        h.update(&self.scope_files_count.to_le_bytes());
        h.update(&self.max_files.to_le_bytes());
        h.update(&self.max_loc.to_le_bytes());
        h.update(&[self.has_external_deps as u8]);
        if let Some(tokens) = self.prompt_tokens { h.update(&tokens.to_le_bytes()); }
        if let Some(f) = self.prior_failures { h.update(&f.to_le_bytes()); }
        if let Some(ref m) = self.model_name { h.update(m.as_bytes()); }
        // complexity_indicators can be high-entropy; hash individually
        for s in &self.complexity_indicators {
            h.update(blake3::hash(s.as_bytes()).as_bytes());
        }
        u64::from_le_bytes(h.finalize().as_bytes()[..8].try_into().unwrap())
    }
}

/// Bandit policy interface - pluggable learning strategies
pub trait BanditPolicy: Send + Sync {
    /// Select arm (parameter set) given context
    fn select(
        &self,
        ctx: &TaskFeatures,
        arms: &[ParameterSet],
    ) -> SelectionResult;
    
    /// Update policy with observed outcome
    fn update(
        &mut self,
        ctx: &TaskFeatures,
        arm: &ParameterSet,
        reward: f64,
    );
    
    /// Get policy version for provenance
    fn version(&self) -> String;
}

/// Result of bandit selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionResult {
    pub arm_index: usize,
    pub parameters: ParameterSet,
    pub propensity: f64,              // For counterfactual logging
    pub confidence: f64,
    pub reasoning: Vec<String>,
}

/// Thompson Sampling for Gaussian rewards
pub struct ThompsonGaussian {
    /// Posterior mean and precision for each arm
    posterior: HashMap<u64, GaussianPosterior>, // key: arm fingerprint
    prior_mean: f64,
    prior_precision: f64,
    noise_precision: f64,  // 1/variance of observation noise
    update_count: usize,
}

/// Gaussian posterior parameters
#[derive(Debug, Clone)]
struct GaussianPosterior {
    mean: f64,
    precision: f64,  // 1/variance
    count: usize,
}

impl GaussianPosterior {
    fn new(prior_mean: f64, prior_precision: f64) -> Self {
        Self {
            mean: prior_mean,
            precision: prior_precision,
            count: 0,
        }
    }

    /// Update posterior with new observation using Bayesian update
    fn update(&mut self, observation: f64, noise_precision: f64) {
        // Bayesian update for Gaussian-Gaussian conjugate prior
        // New precision = old_precision + noise_precision
        // New mean = (old_precision * old_mean + noise_precision * observation) / new_precision
        let new_precision = self.precision + noise_precision;
        let new_mean = (self.precision * self.mean + noise_precision * observation) / new_precision;
        
        self.mean = new_mean;
        self.precision = new_precision;
        self.count += 1;
    }

    /// Sample from posterior distribution
    fn sample<R: rand::Rng>(&self, rng: &mut R) -> f64 {
        use rand_distr::{Distribution, Normal};
        let variance = 1.0 / self.precision;
        let normal = Normal::new(self.mean, variance.sqrt()).unwrap();
        normal.sample(rng)
    }
}

impl ThompsonGaussian {
    pub fn new() -> Self {
        Self {
            posterior: HashMap::new(),
            prior_mean: 0.0,
            prior_precision: 1.0,
            noise_precision: 1.0,
            update_count: 0,
        }
    }

    pub fn with_priors(prior_mean: f64, prior_precision: f64, noise_precision: f64) -> Self {
        Self {
            posterior: HashMap::new(),
            prior_mean,
            prior_precision,
            noise_precision,
            update_count: 0,
        }
    }

    /// Generate fingerprint for parameter set (for posterior lookup)
    fn arm_fingerprint(arm: &ParameterSet) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        arm.temperature.to_bits().hash(&mut hasher);
        arm.max_tokens.hash(&mut hasher);
        if let Some(top_p) = arm.top_p {
            top_p.to_bits().hash(&mut hasher);
        }
        if let Some(freq) = arm.frequency_penalty {
            freq.to_bits().hash(&mut hasher);
        }
        if let Some(pres) = arm.presence_penalty {
            pres.to_bits().hash(&mut hasher);
        }
        hasher.finish()
    }
}

impl BanditPolicy for ThompsonGaussian {
    fn select(&self, _ctx: &TaskFeatures, arms: &[ParameterSet]) -> SelectionResult {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        if arms.is_empty() {
            return SelectionResult {
                arm_index: 0,
                parameters: ParameterSet {
                    temperature: 0.7,
                    max_tokens: 1000,
                    top_p: Some(0.9),
                    frequency_penalty: None,
                    presence_penalty: None,
                    stop_sequences: vec![],
                    seed: None,
                    origin: "thompson_gaussian".to_string(),
                    policy_version: "1.0.0".to_string(),
                    created_at: Utc::now(),
                },
                propensity: 1.0,
                confidence: 0.5,
                reasoning: vec!["No arms available, using default".to_string()],
            };
        }

        let mut best_arm_idx = 0;
        let mut best_sample = f64::NEG_INFINITY;
        let mut propensities = Vec::new();

        // Sample from posterior for each arm and select the best
        for (idx, arm) in arms.iter().enumerate() {
            let fingerprint = Self::arm_fingerprint(arm);
            let posterior = self.posterior.get(&fingerprint)
                .cloned()
                .unwrap_or_else(|| GaussianPosterior::new(self.prior_mean, self.prior_precision));
            
            let sample = posterior.sample(&mut rng);
            propensities.push(sample);
            
            if sample > best_sample {
                best_sample = sample;
                best_arm_idx = idx;
            }
        }

        // Calculate propensity (softmax over samples for proper probability)
        let max_sample = propensities.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let exp_samples: Vec<f64> = propensities.iter().map(|&s| (s - max_sample).exp()).collect();
        let sum_exp: f64 = exp_samples.iter().sum();
        let propensity = exp_samples[best_arm_idx] / sum_exp;

        SelectionResult {
            arm_index: best_arm_idx,
            parameters: arms[best_arm_idx].clone(),
            propensity,
            confidence: (best_sample - self.prior_mean).abs() / (1.0 / self.prior_precision).sqrt(),
            reasoning: vec![
                format!("Thompson sampling selected arm {} with sample {:.3}", best_arm_idx, best_sample),
                format!("Propensity: {:.3}", propensity),
            ],
        }
    }

    fn update(&mut self, _ctx: &TaskFeatures, arm: &ParameterSet, reward: f64) {
        let fingerprint = Self::arm_fingerprint(arm);
        
        let mut posterior = self.posterior.remove(&fingerprint)
            .unwrap_or_else(|| GaussianPosterior::new(self.prior_mean, self.prior_precision));
        
        posterior.update(reward, self.noise_precision);
        self.posterior.insert(fingerprint, posterior);
        self.update_count += 1;
    }

    fn version(&self) -> String {
        "thompson_gaussian@1.0.0".to_string()
    }
}

/// Linear UCB (optimistic under uncertainty)
pub struct LinUCB {
    /// Linear model: reward = θ^T x + ε
    theta: HashMap<String, Vec<f64>>,  // per task_type
    covariance: HashMap<String, Vec<Vec<f64>>>,
    alpha: f64,                        // Exploration parameter
    lambda: f64,                      // Regularization parameter
}

impl LinUCB {
    pub fn new(alpha: f64) -> Self {
        Self {
            theta: HashMap::new(),
            covariance: HashMap::new(),
            alpha,
            lambda: 1.0,
        }
    }

    pub fn with_regularization(alpha: f64, lambda: f64) -> Self {
        Self {
            theta: HashMap::new(),
            covariance: HashMap::new(),
            alpha,
            lambda,
        }
    }

    /// Convert task features to feature vector for linear model
    fn features_to_vector(ctx: &TaskFeatures) -> Vec<f64> {
        vec![
            ctx.risk_tier as f64,
            ctx.title_length as f64,
            ctx.description_length as f64,
            ctx.acceptance_criteria_count as f64,
            ctx.scope_files_count as f64,
            ctx.max_files as f64,
            ctx.max_loc as f64,
            if ctx.has_external_deps { 1.0 } else { 0.0 },
            ctx.prompt_tokens.unwrap_or(0) as f64,
            ctx.prior_failures.unwrap_or(0) as f64,
        ]
    }

    /// Get or initialize model parameters for a task type
    fn get_or_init_model(&mut self, task_type: &str, feature_dim: usize) -> (&mut Vec<f64>, &mut Vec<Vec<f64>>) {
        if !self.theta.contains_key(task_type) {
            // Initialize with zero mean and identity covariance
            self.theta.insert(task_type.to_string(), vec![0.0; feature_dim]);
            self.covariance.insert(task_type.to_string(), 
                (0..feature_dim).map(|i| {
                    (0..feature_dim).map(|j| if i == j { 1.0 / self.lambda } else { 0.0 }).collect()
                }).collect()
            );
        }
        (self.theta.get_mut(task_type).unwrap(), 
         self.covariance.get_mut(task_type).unwrap())
    }
}

impl BanditPolicy for LinUCB {
    fn select(&self, ctx: &TaskFeatures, arms: &[ParameterSet]) -> SelectionResult {
        if arms.is_empty() {
            return SelectionResult {
                arm_index: 0,
                parameters: ParameterSet {
                    temperature: 0.7,
                    max_tokens: 1000,
                    top_p: Some(0.9),
                    frequency_penalty: None,
                    presence_penalty: None,
                    stop_sequences: vec![],
                    seed: None,
                    origin: "linucb".to_string(),
                    policy_version: "1.0.0".to_string(),
                    created_at: Utc::now(),
                },
                propensity: 1.0,
                confidence: 0.5,
                reasoning: vec!["No arms available, using default".to_string()],
            };
        }

        // For now, use a simple task type based on risk tier
        let task_type = format!("tier_{}", ctx.risk_tier);
        let features = Self::features_to_vector(ctx);
        let feature_dim = features.len();

        // Get model parameters (this is a simplified version - in practice you'd need proper locking)
        let theta = self.theta.get(&task_type).unwrap_or(&vec![0.0; feature_dim]);
        let covariance = self.covariance.get(&task_type).unwrap_or(&vec![vec![0.0; feature_dim]; feature_dim]);

        let mut best_arm_idx = 0;
        let mut best_ucb = f64::NEG_INFINITY;
        let mut ucb_values = Vec::new();

        // Calculate UCB for each arm
        for (idx, arm) in arms.iter().enumerate() {
            // Simplified: use arm parameters as additional features
            let mut arm_features = features.clone();
            arm_features.push(arm.temperature as f64);
            arm_features.push(arm.max_tokens as f64);
            if let Some(top_p) = arm.top_p {
                arm_features.push(top_p as f64);
            } else {
                arm_features.push(0.0);
            }

            // Calculate mean reward estimate
            let mean_reward = theta.iter().zip(arm_features.iter()).map(|(t, f)| t * f).sum::<f64>();

            // Calculate confidence bound (simplified)
            let confidence_bound = self.alpha * (1.0 / (1.0 + idx as f64)).sqrt();

            let ucb = mean_reward + confidence_bound;
            ucb_values.push(ucb);

            if ucb > best_ucb {
                best_ucb = ucb;
                best_arm_idx = idx;
            }
        }

        // Calculate propensity (softmax over UCB values)
        let max_ucb = ucb_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let exp_ucb: Vec<f64> = ucb_values.iter().map(|&u| (u - max_ucb).exp()).collect();
        let sum_exp: f64 = exp_ucb.iter().sum();
        let propensity = exp_ucb[best_arm_idx] / sum_exp;

        SelectionResult {
            arm_index: best_arm_idx,
            parameters: arms[best_arm_idx].clone(),
            propensity,
            confidence: best_ucb,
            reasoning: vec![
                format!("LinUCB selected arm {} with UCB {:.3}", best_arm_idx, best_ucb),
                format!("Propensity: {:.3}", propensity),
            ],
        }
    }

    fn update(&mut self, ctx: &TaskFeatures, arm: &ParameterSet, reward: f64) {
        let task_type = format!("tier_{}", ctx.risk_tier);
        let features = Self::features_to_vector(ctx);
        let feature_dim = features.len();

        // Get or initialize model parameters
        let (theta, covariance) = self.get_or_init_model(&task_type, feature_dim);

        // Simplified update (in practice, you'd implement proper ridge regression)
        // This is a placeholder - real LinUCB requires matrix operations
        for (i, feature) in features.iter().enumerate() {
            if i < theta.len() {
                theta[i] += 0.01 * feature * reward; // Simple gradient step
            }
        }
    }

    fn version(&self) -> String {
        "linucb@1.0.0".to_string()
    }
}
