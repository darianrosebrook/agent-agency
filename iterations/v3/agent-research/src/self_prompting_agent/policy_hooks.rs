//! Policy hooks for adaptive agent behavior
//!
//! Provides hooks for policy adaptation and reinforcement learning integration.

use crate::self_prompting_agent::learning_bridge::LearningBridge;
use crate::self_prompting_agent::prompting_types::SelfPromptingAgentError;
use schemars::JsonSchema;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
/// Adaptive agent with policy hooks
pub struct AdaptiveAgent {
    learning_bridge: Arc<LearningBridge>,
    policy_state: Arc<tokio::sync::RwLock<PolicyState>>,
}

impl AdaptiveAgent {
    /// Create a new adaptive agent
    pub fn new() -> Self {
        Self {
            learning_bridge: Arc::new(LearningBridge::new()),
            policy_state: Arc::new(tokio::sync::RwLock::new(PolicyState {
                temperature: 0.7,
                max_iterations: 5,
                risk_tolerance: 0.5,
            })),
        }
    }

    /// Adapt policy based on feedback
    pub async fn adapt_policy(&self, feedback: &str) -> Result<(), SelfPromptingAgentError> {
        tracing::info!("Adapting policy based on feedback: {}", feedback);

        // Get learning recommendations based on feedback
        match self.learning_bridge.get_recommendations(feedback).await {
            Ok(recommendations) => {
                // Apply recommendations to policy state
                let mut state = self.policy_state.write().await;

                // Adjust temperature based on recommendations
                if recommendations
                    .iter()
                    .any(|r| r.contains("more creative") || r.contains("exploration"))
                {
                    state.temperature = (state.temperature + 0.1).min(1.0);
                } else if recommendations
                    .iter()
                    .any(|r| r.contains("more focused") || r.contains("precision"))
                {
                    state.temperature = (state.temperature - 0.1).max(0.0);
                }

                // Adjust iterations based on feedback
                if recommendations
                    .iter()
                    .any(|r| r.contains("complex") || r.contains("multiple steps"))
                {
                    state.max_iterations = (state.max_iterations + 2).min(10);
                }

                // Adjust risk tolerance based on recommendations
                if recommendations
                    .iter()
                    .any(|r| r.contains("conservative") || r.contains("safe"))
                {
                    state.risk_tolerance = (state.risk_tolerance - 0.1).max(0.0);
                } else if recommendations
                    .iter()
                    .any(|r| r.contains("aggressive") || r.contains("fast"))
                {
                    state.risk_tolerance = (state.risk_tolerance + 0.1).min(1.0);
                }

                tracing::debug!(
                    "Policy adapted: temp={:.2}, iterations={}, risk={:.2}",
                    state.temperature,
                    state.max_iterations,
                    state.risk_tolerance
                );
            }
            Err(e) => {
                tracing::warn!("Failed to get learning recommendations: {}", e);
                // Continue with basic adaptation
            }
        }

        Ok(())
    }

    /// Get current policy state
    pub async fn get_policy_state(&self) -> PolicyState {
        self.policy_state.read().await.clone()
    }
}

/// Policy state snapshot

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyState {
    pub temperature: f64,
    pub max_iterations: usize,
    pub risk_tolerance: f64,
}

/// Policy manager for rule-based adaptations
pub struct PolicyManager {
    rules: Arc<tokio::sync::RwLock<Vec<String>>>,
}

impl PolicyManager {
    /// Create a new policy manager
    pub fn new() -> Self {
        Self {
            rules: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    /// Update policy rules
    pub async fn update_policy(&self, policy: &str) -> Result<(), SelfPromptingAgentError> {
        // Validate policy first
        self.validate_policy(policy)?;

        // Parse policy into rules (simple line-based parsing)
        let rules: Vec<String> = policy
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .collect();

        // Update stored rules
        {
            let mut stored_rules = self.rules.write().await;
            stored_rules.clear();
            stored_rules.extend(rules);
        }

        tracing::info!(
            "Updated policy with {} rules",
            self.rules.read().await.len()
        );
        Ok(())
    }

    /// Get current policy rules
    pub async fn get_rules(&self) -> Vec<String> {
        self.rules.read().await.clone()
    }

    /// Validate policy against constraints
    pub fn validate_policy(&self, policy: &str) -> Result<(), SelfPromptingAgentError> {
        if policy.trim().is_empty() {
            return Err(SelfPromptingAgentError::Validation(
                "Policy cannot be empty".to_string(),
            ));
        }

        // Check for basic safety constraints
        let policy_lower = policy.to_lowercase();
        if policy_lower.contains("allow_all") && !policy_lower.contains("with_safety_check") {
            return Err(SelfPromptingAgentError::Validation(
                "Unsafe policy detected: 'allow_all' without safety checks".to_string(),
            ));
        }

        Ok(())
    }
}

/// Policy hook for pre-execution validation
pub trait PolicyHook: Send + Sync {
    /// Execute policy check
    async fn check(&self, context: &str) -> Result<PolicyDecision, SelfPromptingAgentError>;

    /// Get hook name
    fn name(&self) -> &str;
}

/// Policy decision

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyDecision {
    Allow,
    Deny(String),
    Modify(String),
}

/// Safety policy hook
pub struct SafetyPolicyHook;

impl SafetyPolicyHook {
    pub fn new() -> Self {
        Self
    }
}

impl PolicyHook for SafetyPolicyHook {
    async fn check(&self, context: &str) -> Result<PolicyDecision, SelfPromptingAgentError> {
        let context_lower = context.to_lowercase();

        // Check for unsafe keywords
        let unsafe_patterns = [
            "delete all",
            "remove all",
            "format",
            "rm -rf",
            "drop database",
            "shutdown",
            "kill process",
            "overwrite",
            "modify system",
            "execute shell",
            "eval(",
            "exec(",
            "system(",
        ];

        for pattern in &unsafe_patterns {
            if context_lower.contains(pattern) {
                return Ok(PolicyDecision::Deny(format!(
                    "Unsafe operation detected: '{}'",
                    pattern
                )));
            }
        }

        // Check for suspicious file operations
        if context_lower.contains("/etc/")
            || context_lower.contains("/sys/")
            || context_lower.contains("/proc/")
        {
            return Ok(PolicyDecision::Deny(
                "Access to system directories not allowed".to_string(),
            ));
        }

        // Check for network operations without proper context
        if (context_lower.contains("http://") || context_lower.contains("https://"))
            && !context_lower.contains("allow_network")
        {
            return Ok(PolicyDecision::Modify(
                "Network operations require explicit allowance".to_string(),
            ));
        }

        Ok(PolicyDecision::Allow)
    }

    fn name(&self) -> &str {
        "Safety Policy Hook"
    }
}
