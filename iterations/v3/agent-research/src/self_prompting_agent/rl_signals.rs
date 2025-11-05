//! Reinforcement learning signals for agent adaptation
//!
//! Provides signals and policy adjustments for RL-based agent improvement.

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use crate::reinforcement::QLearning;
use crate::reflexive_types::AlgorithmConfig;
use crate::self_prompting_agent::prompting_types::SelfPromptingAgentError;

/// RL signal for feedback

#[derive(Debug, Clone, Serialize, Deserialize) ]
pub struct RLSignal {
    pub signal_type: String,
    pub value: f64,
    pub context: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}


/// RL signal generator
pub struct RLSignalGenerator {
    q_learning: Arc<RwLock<QLearning>>,
}

impl RLSignalGenerator {
    /// Create a new signal generator
    pub fn new() -> Self {
        let config = AlgorithmConfig {
            learning_rate: 0.1,
            discount_factor: 0.9,
            exploration_rate: 0.1,
            min_exploration_rate: Some(0.01),
            exploration_decay: Some(0.99),
            max_iterations: 1000,
            max_episodes: Some(1000),
            convergence_threshold: 0.001,
        };
        Self {
            q_learning: Arc::new(RwLock::new(QLearning::new(config))),
        }
    }

    /// Generate signal from state
    pub async fn generate(&self, state: &str) -> Result<RLSignal, SelfPromptingAgentError> {
        // Analyze state and generate appropriate RL signal
        let state_lower = state.to_lowercase();
        
        // Determine signal value based on state characteristics
        let value = if state_lower.contains("success") || state_lower.contains("complete") {
            1.0
        } else if state_lower.contains("failure") || state_lower.contains("error") {
            -1.0
        } else if state_lower.contains("partial") || state_lower.contains("progress") {
            0.5
        } else {
            // Use Q-learning to estimate value
            let q_learning = self.q_learning.read().await;
            // Get best Q-value for this state as signal value
            let available_actions = vec![
                "continue".to_string(),
                "retry".to_string(),
                "abort".to_string(),
            ];
            let best_action = q_learning.get_q_table().get_best_action(state)
                .unwrap_or_else(|| available_actions[0].clone());
            let q_value = q_learning.get_q_table().get(state, &best_action);
            // Normalize Q-value to [-1, 1] range
            q_value.tanh()
        };

        Ok(RLSignal {
            signal_type: "state_analysis".to_string(),
            value,
            context: state.to_string(),
            timestamp: chrono::Utc::now(),
        })
    }

    /// Generate signal from performance metrics
    pub fn generate_from_metrics(&self, accuracy: f64, efficiency: f64) -> RLSignal {
        let combined_value = (accuracy + efficiency) / 2.0;

        RLSignal {
            signal_type: "performance".to_string(),
            value: combined_value,
            context: format!("accuracy: {:.2}, efficiency: {:.2}", accuracy, efficiency),
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Policy adjustment based on RL signals

#[derive(Debug, Clone, Serialize, Deserialize) ]
pub struct PolicyAdjustment {
    pub parameter: String,
    pub current_value: f64,
    pub new_value: f64,
    pub reason: String,
}

/// Policy adjuster
pub struct PolicyAdjuster {
    q_learning: Arc<RwLock<QLearning>>,
    current_policy: Arc<RwLock<HashMap<String, f64>>>,
}

impl PolicyAdjuster {
    /// Create a new policy adjuster
    pub fn new() -> Self {
        let config = AlgorithmConfig {
            learning_rate: 0.1,
            discount_factor: 0.9,
            exploration_rate: 0.1,
            min_exploration_rate: Some(0.01),
            exploration_decay: Some(0.995),
            max_iterations: 1000,
            max_episodes: Some(10000),
            convergence_threshold: 0.001,
        };
        let mut default_policy = HashMap::new();
        default_policy.insert("temperature".to_string(), 0.7);
        default_policy.insert("max_iterations".to_string(), 5.0);
        default_policy.insert("risk_tolerance".to_string(), 0.5);
        
        Self {
            q_learning: Arc::new(RwLock::new(QLearning::new(config))),
            current_policy: Arc::new(RwLock::new(default_policy)),
        }
    }

    /// Adjust policy based on signal
    pub async fn adjust_policy(&self, signal: &RLSignal) -> Result<Option<PolicyAdjustment>, SelfPromptingAgentError> {
        // Use Q-learning to determine optimal policy adjustment
        let state = format!("signal_{}_{}", signal.signal_type, signal.value);
        let available_actions = vec![
            "increase_temperature".to_string(),
            "decrease_temperature".to_string(),
            "increase_iterations".to_string(),
            "decrease_iterations".to_string(),
            "increase_risk".to_string(),
            "decrease_risk".to_string(),
            "maintain_current".to_string(),
        ];
        
        let mut q_learning = self.q_learning.write().await;
        let action = q_learning.select_action(&state, &available_actions);
        
        // Update Q-learning with signal value as reward
        let next_state = format!("policy_adjusted_{}", action);
        q_learning.update(&state, &action, signal.value, &next_state);
        
        // Map action to policy adjustment
        let mut policy = self.current_policy.write().await;
        
        let adjustment = if action == "maintain_current" {
            None
        } else if action == "increase_temperature" {
            let current = policy.get("temperature").copied().unwrap_or(0.7);
            let new_value = (current + 0.1).min(1.0);
            policy.insert("temperature".to_string(), new_value);
            Some(PolicyAdjustment {
                parameter: "temperature".to_string(),
                current_value: current,
                new_value,
                reason: format!("Signal value {:.2} suggests increasing exploration", signal.value),
            })
        } else if action == "decrease_temperature" {
            let current = policy.get("temperature").copied().unwrap_or(0.7);
            let new_value = (current - 0.1).max(0.0);
            policy.insert("temperature".to_string(), new_value);
            Some(PolicyAdjustment {
                parameter: "temperature".to_string(),
                current_value: current,
                new_value,
                reason: format!("Signal value {:.2} suggests increasing precision", signal.value),
            })
        } else if action == "increase_iterations" {
            let current = policy.get("max_iterations").copied().unwrap_or(5.0);
            let new_value = (current + 2.0).min(10.0);
            policy.insert("max_iterations".to_string(), new_value);
            Some(PolicyAdjustment {
                parameter: "max_iterations".to_string(),
                current_value: current,
                new_value,
                reason: format!("Signal value {:.2} suggests more iterations needed", signal.value),
            })
        } else if action == "decrease_iterations" {
            let current = policy.get("max_iterations").copied().unwrap_or(5.0);
            let new_value = (current - 1.0).max(1.0);
            policy.insert("max_iterations".to_string(), new_value);
            Some(PolicyAdjustment {
                parameter: "max_iterations".to_string(),
                current_value: current,
                new_value,
                reason: format!("Signal value {:.2} suggests fewer iterations sufficient", signal.value),
            })
        } else if action == "increase_risk" {
            let current = policy.get("risk_tolerance").copied().unwrap_or(0.5);
            let new_value = (current + 0.1).min(1.0);
            policy.insert("risk_tolerance".to_string(), new_value);
            Some(PolicyAdjustment {
                parameter: "risk_tolerance".to_string(),
                current_value: current,
                new_value,
                reason: format!("Signal value {:.2} suggests higher risk tolerance", signal.value),
            })
        } else if action == "decrease_risk" {
            let current = policy.get("risk_tolerance").copied().unwrap_or(0.5);
            let new_value = (current - 0.1).max(0.0);
            policy.insert("risk_tolerance".to_string(), new_value);
            Some(PolicyAdjustment {
                parameter: "risk_tolerance".to_string(),
                current_value: current,
                new_value,
                reason: format!("Signal value {:.2} suggests lower risk tolerance", signal.value),
            })
        } else {
            None
        };
        
        Ok(adjustment)
    }

    /// Apply policy adjustment
    pub async fn apply_adjustment(&self, adjustment: &PolicyAdjustment) -> Result<(), SelfPromptingAgentError> {
        // Apply the adjustment to the stored policy
        let mut policy = self.current_policy.write().await;
        policy.insert(adjustment.parameter.clone(), adjustment.new_value);
        
        tracing::info!(
            "Applied policy adjustment: {} from {} to {} ({})",
            adjustment.parameter,
            adjustment.current_value,
            adjustment.new_value,
            adjustment.reason
        );
        Ok(())
    }
}

/// RL trainer for policy learning
pub struct RLTrainer {
    q_learning: Arc<RwLock<QLearning>>,
    experience_buffer: Arc<RwLock<ExperienceBuffer>>,
}

impl RLTrainer {
    /// Create a new RL trainer
    pub fn new(learning_rate: f64, discount_factor: f64) -> Self {
        let config = AlgorithmConfig {
            learning_rate,
            discount_factor,
            exploration_rate: 0.1,
            convergence_threshold: 0.001,
            exploration_decay: Some(0.99),
            max_episodes: Some(1000),
            min_exploration_rate: Some(0.01),
            max_iterations: 1000,
        };
        Self {
            q_learning: Arc::new(RwLock::new(QLearning::new(config))),
            experience_buffer: Arc::new(RwLock::new(ExperienceBuffer::new(1000))),
        }
    }

    /// Train on experience
    pub async fn train_on_experience(&self, state: &str, action: &str, reward: f64, next_state: &str) -> Result<(), SelfPromptingAgentError> {
        // Store experience in buffer
        {
            let mut buffer = self.experience_buffer.write().await;
            buffer.add_experience(Experience {
                state: state.to_string(),
                action: action.to_string(),
                reward,
                next_state: next_state.to_string(),
                done: false,
            });
        }
        
        // Update Q-learning with the experience
        {
            let mut q_learning = self.q_learning.write().await;
            q_learning.update(state, action, reward, next_state);
        }
        
        tracing::debug!(
            "Trained on experience: {} -> {} -> {} -> {} (reward: {:.2})",
            state, action, reward, next_state, reward
        );
        Ok(())
    }

    /// Get best action for state
    pub async fn get_best_action(&self, state: &str) -> String {
        let q_learning = self.q_learning.read().await;
        let available_actions = vec![
            "direct_execution".to_string(),
            "iterative_refinement".to_string(),
            "standard_approach".to_string(),
        ];
        
        // Use Q-learning to select best action
        q_learning.get_q_table().get_best_action(state)
            .filter(|action| available_actions.contains(action))
            .unwrap_or_else(|| {
                // Fallback based on state characteristics
                if state.contains("simple") {
                    "direct_execution".to_string()
                } else if state.contains("complex") {
                    "iterative_refinement".to_string()
                } else {
                    "standard_approach".to_string()
                }
            })
    }
}

/// Experience buffer for RL training
#[derive(Debug, Clone, Serialize, Deserialize) ]
pub struct ExperienceBuffer {
    experiences: Vec<Experience>,
    max_size: usize,
}

impl ExperienceBuffer {
    /// Create a new experience buffer
    pub fn new(max_size: usize) -> Self {
        Self {
            experiences: Vec::new(),
            max_size,
        }
    }

    /// Add experience
    pub fn add_experience(&mut self, experience: Experience) {
        self.experiences.push(experience);
        if self.experiences.len() > self.max_size {
            self.experiences.remove(0); // Remove oldest
        }
    }

    /// Sample batch of experiences randomly
    pub fn sample_batch(&self, batch_size: usize) -> Vec<&Experience> {
        use rand::seq::SliceRandom;
        use rand::thread_rng;
        
        if self.experiences.is_empty() {
            return Vec::new();
        }
        
        let batch_size = batch_size.min(self.experiences.len());
        let mut rng = thread_rng();
        
        // Randomly sample without replacement
        self.experiences.choose_multiple(&mut rng, batch_size).collect()
    }
}

/// RL experience tuple

#[derive(Debug, Clone, Serialize, Deserialize) ]
pub struct Experience {
    pub state: String,
    pub action: String,
    pub reward: f64,
    pub next_state: String,
    pub done: bool,
}
