//! Reward design and Goodhart guardrails for parallel worker learning

use crate::types::ExecutionMetrics;
use serde::{Deserialize, Serialize};

/// Reward weights for composite reward function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardWeights {
    pub quality: f64,
    pub latency: f64,
    pub rework: f64,
    pub cost: f64,
}

impl Default for RewardWeights {
    fn default() -> Self {
        Self {
            quality: 1.0,
            latency: 0.5,
            rework: 2.0,
            cost: 0.3,
        }
    }
}

/// Baseline metrics for normalization
#[derive(Debug, Clone)]
pub struct Baseline {
    pub p50_ms: f64,
    pub p50_quality: f64,
    pub p50_tokens: f64,
}

impl Default for Baseline {
    fn default() -> Self {
        Self {
            p50_ms: 1000.0,  // 1 second baseline
            p50_quality: 0.8, // 80% quality baseline
            p50_tokens: 1000.0, // 1000 tokens baseline
        }
    }
}

/// Compute composite reward with Goodhart guardrails
pub fn compute_reward(
    metrics: &ExecutionMetrics,
    weights: &RewardWeights,
    baseline: &Baseline,
    rework_within_24h: bool,
) -> f64 {
    // Quality component [0, 1] - higher is better
    let quality_score = metrics.quality_score as f64;
    
    // Latency component (normalized, lower is better)
    let latency_ratio = (metrics.execution_time_ms as f64) / baseline.p50_ms;
    
    // Rework penalty (binary) - higher penalty for rework
    let rework_penalty = if rework_within_24h { 1.0 } else { 0.0 };
    
    // Cost component (token usage, normalized) - lower is better
    let cost_ratio = (metrics.tokens.unwrap_or(0) as f64) / baseline.p50_tokens;
    
    // Composite reward (higher is better)
    weights.quality * quality_score
        - weights.latency * latency_ratio
        - weights.rework * rework_penalty
        - weights.cost * cost_ratio
}

/// Flag to prevent learning from experimental executions
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LearningMode {
    /// Normal execution, contribute to learning
    Learn,
    /// Experimental execution, do not update policies
    DoNotLearn,
    /// Shadow execution for evaluation only
    Shadow,
}

impl Default for LearningMode {
    fn default() -> Self {
        Self::Learn
    }
}

/// Reward calculation with mode awareness
pub fn compute_reward_with_mode(
    metrics: &ExecutionMetrics,
    weights: &RewardWeights,
    baseline: &Baseline,
    rework_within_24h: bool,
    mode: LearningMode,
) -> Option<f64> {
    match mode {
        LearningMode::Learn | LearningMode::Shadow => {
            Some(compute_reward(metrics, weights, baseline, rework_within_24h))
        }
        LearningMode::DoNotLearn => None,
    }
}

/// Reward normalization to prevent scale issues
pub fn normalize_reward(reward: f64, min_reward: f64, max_reward: f64) -> f64 {
    if max_reward == min_reward {
        0.5 // Neutral if no variation
    } else {
        (reward - min_reward) / (max_reward - min_reward)
    }
}

/// Reward validation to detect potential Goodhart problems
pub fn validate_reward(reward: f64, metrics: &ExecutionMetrics) -> bool {
    // Check for suspicious patterns that might indicate gaming
    let quality_too_high = metrics.quality_score > 0.99;
    let latency_too_low = metrics.execution_time_ms < 10; // Suspiciously fast
    let no_tokens = metrics.tokens.unwrap_or(0) == 0; // No work done
    
    // Flag suspicious rewards
    if quality_too_high && latency_too_low && no_tokens {
        return false; // Likely gaming
    }
    
    // Check for reasonable reward range
    reward >= -10.0 && reward <= 10.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ExecutionMetrics;
    use chrono::Utc;

    fn create_test_metrics(execution_time_ms: u64, quality_score: f32, tokens: Option<u64>) -> ExecutionMetrics {
        ExecutionMetrics {
            start_time: Utc::now(),
            end_time: Utc::now(),
            execution_time_ms,
            cpu_usage_percent: Some(50.0),
            memory_usage_mb: Some(100.0),
            files_modified: 1,
            lines_changed: 10,
            quality_score,
            tokens,
        }
    }

    #[test]
    fn test_reward_calculation() {
        let weights = RewardWeights::default();
        let baseline = Baseline::default();
        
        let metrics = create_test_metrics(500, 0.9, Some(800));
        let reward = compute_reward(&metrics, &weights, &baseline, false);
        
        // Should be positive for good performance
        assert!(reward > 0.0);
    }
    
    #[test]
    fn test_reward_with_rework() {
        let weights = RewardWeights::default();
        let baseline = Baseline::default();
        
        let metrics = create_test_metrics(500, 0.9, Some(800));
        let reward_with_rework = compute_reward(&metrics, &weights, &baseline, true);
        let reward_without_rework = compute_reward(&metrics, &weights, &baseline, false);
        
        // Reward should be lower with rework
        assert!(reward_with_rework < reward_without_rework);
    }
    
    #[test]
    fn test_reward_validation() {
        let metrics = create_test_metrics(5, 1.0, Some(0)); // Suspicious metrics
        let reward = 5.0;
        
        // Should fail validation due to suspicious pattern
        assert!(!validate_reward(reward, &metrics));
    }
    
    #[test]
    fn test_learning_mode() {
        let weights = RewardWeights::default();
        let baseline = Baseline::default();
        let metrics = create_test_metrics(500, 0.9, Some(800));
        
        // Learn mode should compute reward
        let reward_learn = compute_reward_with_mode(&metrics, &weights, &baseline, false, LearningMode::Learn);
        assert!(reward_learn.is_some());
        
        // DoNotLearn mode should return None
        let reward_no_learn = compute_reward_with_mode(&metrics, &weights, &baseline, false, LearningMode::DoNotLearn);
        assert!(reward_no_learn.is_none());
    }
    
    #[test]
    fn test_reward_normalization() {
        let normalized = normalize_reward(0.5, 0.0, 1.0);
        assert_eq!(normalized, 0.5);
        
        let normalized_edge = normalize_reward(0.5, 0.5, 0.5);
        assert_eq!(normalized_edge, 0.5);
    }
}
