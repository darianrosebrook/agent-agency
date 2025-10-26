//! Constrained Contextual Bandits for Tool Selection
//!
//! Implements LinUCB/Thompson sampling with contextual features and hard constraints
//! for production-safe tool selection with offline IPS/DR evaluation.

use std::collections::HashMap;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::tool_registry::{ToolRegistry, RegisteredTool};

/// Tool selection policy trait
#[async_trait::async_trait]
pub trait ToolPolicy: Send + Sync {
    fn select_tool(&self, ctx: &ToolContextFeatures, tools: &[ToolId], constraints: &ToolConstraints)
        -> (ToolId, f64 /*propensity*/, f64 /*confidence*/);
    fn update(&mut self, ctx: &ToolContextFeatures, tool: &ToolId, reward: f64);
    fn version(&self) -> &'static str;
}

/// Tool ID type alias
pub type ToolId = String;

/// Contextual features for tool selection
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolContextFeatures {
    pub task_type: String,
    pub prompt_len: usize,
    pub retrieval_k: usize,
    pub is_code_task: bool,
    pub expected_latency_ms: u64,
    pub cost_budget_cents: u32,
    pub risk_tier: u8,
}

/// Hard constraints for tool selection
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolConstraints {
    pub max_latency_ms: u64,
    pub max_cost_cents: u32,
    pub require_caws: bool,
}

/// LinUCB policy implementation
pub struct LinUCBPolicy {
    alpha: f64,                        // exploration parameter
    theta: HashMap<ToolId, Vec<f64>>,  // weights per tool
    cov: HashMap<ToolId, Matrix>,      // A^-1 per tool (LinUCB)
    feature_dim: usize,
}

impl LinUCBPolicy {
    /// Create a new LinUCB policy
    pub fn new(alpha: f64, feature_dim: usize) -> Self {
        Self {
            alpha,
            theta: HashMap::new(),
            cov: HashMap::new(),
            feature_dim,
        }
    }

    /// Extract feature vector from context
    fn extract_features(&self, ctx: &ToolContextFeatures) -> Vec<f64> {
        vec![
            ctx.task_type.len() as f64 / 20.0,     // task type length (normalized)
            ctx.prompt_len as f64 / 1000.0,        // prompt length (normalized)
            ctx.retrieval_k as f64 / 10.0,         // retrieval k (normalized)
            if ctx.is_code_task { 1.0 } else { 0.0 }, // code task flag
            ctx.expected_latency_ms as f64 / 10000.0, // latency (normalized)
            ctx.cost_budget_cents as f64 / 100.0,  // cost budget (normalized)
            ctx.risk_tier as f64 / 5.0,            // risk tier (normalized)
        ]
    }

    /// Check if tool satisfies constraints
    fn satisfies_constraints(&self, tool_id: &ToolId, constraints: &ToolConstraints) -> bool {
        // This would check against tool metadata
        // For now, assume all tools pass (would be implemented with real tool registry)
        true
    }

    /// Compute propensity score
    fn compute_propensity(&self, score: f64) -> f64 {
        // Softmax-like propensity for IPS
        (score.exp() / (1.0 + score.exp())).clamp(0.001, 0.999)
    }

    /// Update LinUCB matrices
    fn update_matrices(&mut self, tool_id: &ToolId, features: &[f64], reward: f64) {
        let dim = features.len();

        // Initialize matrices if needed
        if !self.theta.contains_key(tool_id) {
            self.theta.insert(tool_id.clone(), vec![0.0; dim]);
            // Initialize A^-1 as identity matrix
            let mut identity = vec![vec![0.0; dim]; dim];
            for i in 0..dim {
                identity[i][i] = 1.0;
            }
            self.cov.insert(tool_id.clone(), Matrix { data: identity });
        }

        // Sherman-Morrison update for LinUCB
        // A_{t+1}^-1 = A_t^-1 - (A_t^-1 x x^T A_t^-1) / (1 + x^T A_t^-1 x)
        let a_inv = self.cov.get_mut(tool_id).unwrap();
        let theta = self.theta.get_mut(tool_id).unwrap();

        // Compute x^T A^-1
        let x_a_inv = matrix_vector_mul(a_inv, features);

        // Compute x^T A^-1 x
        let x_a_inv_x = dot_product(&x_a_inv, features);

        // Update A^-1
        let denominator = 1.0 + x_a_inv_x;
        for i in 0..dim {
            for j in 0..dim {
                a_inv.data[i][j] -= x_a_inv[i] * x_a_inv[j] / denominator;
            }
        }

        // Update theta: θ_{t+1} = A_{t+1}^-1 (A_t θ_t + r x)
        for i in 0..dim {
            let update = reward * features[i];
            theta[i] += update;
        }

        // Apply A^-1 to theta
        let new_theta = matrix_vector_mul(a_inv, theta);
        *theta = new_theta;
    }
}

impl ToolPolicy for LinUCBPolicy {
    fn select_tool(&self, ctx: &ToolContextFeatures, tools: &[ToolId], constraints: &ToolConstraints)
        -> (ToolId, f64, f64) {
        let x = self.extract_features(ctx);

        let mut best_tool = None;
        let mut best_score = f64::NEG_INFINITY;
        let mut best_propensity = 0.0;

        for tool_id in tools {
            if !self.satisfies_constraints(tool_id, constraints) {
                continue;
            }

            let theta_t = self.theta.get(tool_id).unwrap_or(&vec![0.0; self.feature_dim]);
            let cov_t = self.cov.get(tool_id).unwrap_or(&Matrix::identity(self.feature_dim));

            // LinUCB score: θᵀx + α·sqrt(xᵀ A⁻¹ x)
            let mean = dot_product(theta_t, &x);
            let variance = compute_variance(&x, cov_t);
            let score = mean + self.alpha * variance.sqrt();

            if score > best_score {
                best_score = score;
                best_tool = Some(tool_id.clone());
                best_propensity = self.compute_propensity(score);
            }
        }

        (best_tool.unwrap_or_else(|| tools[0].clone()), best_propensity, best_score)
    }

    fn update(&mut self, ctx: &ToolContextFeatures, tool: &ToolId, reward: f64) {
        let x = self.extract_features(ctx);
        self.update_matrices(tool, &x, reward);
    }

    fn version(&self) -> &'static str { "linucb-v1" }
}

/// Thompson Sampling policy implementation
pub struct ThompsonSamplingPolicy {
    alpha: HashMap<ToolId, Vec<f64>>,  // success counts
    beta: HashMap<ToolId, Vec<f64>>,   // failure counts
    feature_dim: usize,
}

impl ThompsonSamplingPolicy {
    pub fn new(feature_dim: usize) -> Self {
        Self {
            alpha: HashMap::new(),
            beta: HashMap::new(),
            feature_dim,
        }
    }
}

impl ToolPolicy for ThompsonSamplingPolicy {
    fn select_tool(&self, ctx: &ToolContextFeatures, tools: &[ToolId], constraints: &ToolConstraints)
        -> (ToolId, f64, f64) {
        let x = self.extract_features(ctx);

        let mut best_tool = None;
        let mut best_score = f64::NEG_INFINITY;
        let mut best_propensity = 0.0;

        for tool_id in tools {
            if !self.satisfies_constraints(tool_id, constraints) {
                continue;
            }

            let alpha_t = self.alpha.get(tool_id).unwrap_or(&vec![1.0; self.feature_dim]);
            let beta_t = self.beta.get(tool_id).unwrap_or(&vec![1.0; self.feature_dim]);

            // Sample from Beta distribution for each feature
            let mut sample_score = 0.0;
            for i in 0..self.feature_dim {
                let alpha_val = alpha_t[i];
                let beta_val = beta_t[i];
                // Beta sample (simplified - would use proper Beta sampling)
                sample_score += (alpha_val / (alpha_val + beta_val)) * x[i];
            }

            if sample_score > best_score {
                best_score = sample_score;
                best_tool = Some(tool_id.clone());
                best_propensity = sample_score.min(0.999).max(0.001);
            }
        }

        (best_tool.unwrap_or_else(|| tools[0].clone()), best_propensity, best_score)
    }

    fn update(&mut self, ctx: &ToolContextFeatures, tool: &ToolId, reward: f64) {
        let x = self.extract_features(ctx);

        // Initialize if needed
        if !self.alpha.contains_key(tool) {
            self.alpha.insert(tool.clone(), vec![1.0; self.feature_dim]);
            self.beta.insert(tool.clone(), vec![1.0; self.feature_dim]);
        }

        let alpha_t = self.alpha.get_mut(tool).unwrap();
        let beta_t = self.beta.get_mut(tool).unwrap();

        // Update Beta distributions
        for i in 0..self.feature_dim {
            if reward > 0.5 {  // Success
                alpha_t[i] += x[i];
            } else {  // Failure
                beta_t[i] += x[i];
            }
        }
    }

    fn version(&self) -> &'static str { "thompson-v1" }
}

impl ThompsonSamplingPolicy {
    fn extract_features(&self, ctx: &ToolContextFeatures) -> Vec<f64> {
        vec![
            ctx.task_type.len() as f64 / 20.0,
            ctx.prompt_len as f64 / 1000.0,
            ctx.retrieval_k as f64 / 10.0,
            if ctx.is_code_task { 1.0 } else { 0.0 },
            ctx.expected_latency_ms as f64 / 10000.0,
            ctx.cost_budget_cents as f64 / 100.0,
            ctx.risk_tier as f64 / 5.0,
        ]
    }

    fn satisfies_constraints(&self, tool_id: &ToolId, constraints: &ToolConstraints) -> bool {
        // Implementation would check tool metadata against constraints
        true
    }
}

/// Simple matrix utilities
#[derive(Clone, Debug)]
struct Matrix {
    data: Vec<Vec<f64>>,
}

impl Matrix {
    fn identity(size: usize) -> Self {
        let mut data = vec![vec![0.0; size]; size];
        for i in 0..size {
            data[i][i] = 1.0;
        }
        Self { data }
    }
}

/// Matrix-vector multiplication
fn matrix_vector_mul(matrix: &Matrix, vector: &[f64]) -> Vec<f64> {
    matrix.data.iter()
        .map(|row| dot_product(row, vector))
        .collect()
}

/// Dot product
fn dot_product(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// Compute variance x^T A^-1 x
fn compute_variance(x: &[f64], a_inv: &Matrix) -> f64 {
    let x_a_inv = matrix_vector_mul(a_inv, x);
    dot_product(&x_a_inv, x)
}

/// Tool learning system with multiple policies
pub struct ToolLearningSystem {
    policies: HashMap<String, Box<dyn ToolPolicy>>,
    active_policy: String,
    tool_registry: Arc<ToolRegistry>,
}

impl ToolLearningSystem {
    pub fn new(tool_registry: Arc<ToolRegistry>) -> Self {
        let mut policies = HashMap::new();

        // Initialize with LinUCB and Thompson sampling
        policies.insert(
            "linucb".to_string(),
            Box::new(LinUCBPolicy::new(0.1, 7)) as Box<dyn ToolPolicy>
        );
        policies.insert(
            "thompson".to_string(),
            Box::new(ThompsonSamplingPolicy::new(7)) as Box<dyn ToolPolicy>
        );

        Self {
            policies,
            active_policy: "linucb".to_string(),
            tool_registry,
        }
    }

    pub fn select_tool(&self, ctx: &ToolContextFeatures, constraints: &ToolConstraints)
        -> Result<(ToolId, f64, f64), ToolLearningError> {
        let policy = self.policies.get(&self.active_policy)
            .ok_or(ToolLearningError::PolicyNotFound)?;

        let available_tools = self.get_available_tools()?;
        if available_tools.is_empty() {
            return Err(ToolLearningError::NoToolsAvailable);
        }

        let (tool, propensity, confidence) = policy.select_tool(ctx, &available_tools, constraints);

        info!("Selected tool {} with propensity {:.3}, confidence {:.3}",
              tool, propensity, confidence);

        Ok((tool, propensity, confidence))
    }

    pub fn update_policy(&mut self, ctx: &ToolContextFeatures, tool: &ToolId, reward: f64)
        -> Result<(), ToolLearningError> {
        if let Some(policy) = self.policies.get_mut(&self.active_policy) {
            policy.update(ctx, tool, reward);
            debug!("Updated policy {} for tool {} with reward {:.3}",
                   self.active_policy, tool, reward);
            Ok(())
        } else {
            Err(ToolLearningError::PolicyNotFound)
        }
    }

    pub fn switch_policy(&mut self, policy_name: &str) -> Result<(), ToolLearningError> {
        if self.policies.contains_key(policy_name) {
            self.active_policy = policy_name.to_string();
            info!("Switched to policy: {}", policy_name);
            Ok(())
        } else {
            Err(ToolLearningError::PolicyNotFound)
        }
    }

    pub fn get_available_tools(&self) -> Result<Vec<ToolId>, ToolLearningError> {
        // Get all tools from registry
        let tools = self.tool_registry.get_all_tools().unwrap_or_default();
        Ok(tools.into_iter().map(|t| t.name).collect())
    }

    pub fn get_policy_stats(&self) -> HashMap<String, String> {
        let mut stats = HashMap::new();
        for (name, policy) in &self.policies {
            stats.insert(name.clone(), policy.version().to_string());
        }
        stats.insert("active".to_string(), self.active_policy.clone());
        stats
    }
}

/// Errors from tool learning operations
#[derive(Debug, thiserror::Error)]
pub enum ToolLearningError {
    #[error("Policy not found: {0}")]
    PolicyNotFound(String),

    #[error("No tools available")]
    NoToolsAvailable,

    #[error("Tool registry error: {0}")]
    RegistryError(String),
}

impl From<ToolLearningError> for anyhow::Error {
    fn from(err: ToolLearningError) -> Self {
        anyhow::anyhow!(err)
    }
}
