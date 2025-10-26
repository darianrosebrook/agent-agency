//! Planning Agent Integration with LLM Parameter Optimization
//!
//! Demonstrates how to integrate the LLM parameter feedback loop into the
//! PlanningAgent for optimized parameter selection and outcome tracking.

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::bandit_policy::{BanditPolicy, ParameterSet, TaskFeatures, ThompsonGaussian, LinUCB};
use crate::parameter_optimizer::LLMParameterOptimizer;
use crate::quality_gate_validator::{QualityGateValidator, ComplianceValidator};
use crate::reward::{RewardFunction, ObjectiveWeights, TaskOutcome, BaselineMetrics};
use crate::rollout::{RolloutManager, RolloutPhase};
use crate::caws_integration::{CAWSBudgetTracker, ParameterChangeProvenance};

/// Enhanced Planning Agent with parameter optimization
pub struct OptimizedPlanningAgent {
    /// Core planning agent (from orchestration module)
    // planning_agent: Arc<dyn PlanningAgent>, // Would be imported from orchestration
    
    /// LLM parameter optimizer
    parameter_optimizer: Arc<LLMParameterOptimizer>,
    
    /// Quality gate validator
    quality_validator: Arc<QualityGateValidator>,
    
    /// Reward function for optimization
    reward_function: Arc<RewardFunction>,
    
    /// Rollout manager for safe deployment
    rollout_manager: Arc<RolloutManager>,
    
    /// CAWS budget tracker
    budget_tracker: Arc<CAWSBudgetTracker>,
    
    /// Current task features for context
    current_task_features: Arc<RwLock<Option<TaskFeatures>>>,
}

impl OptimizedPlanningAgent {
    /// Create a new optimized planning agent
    pub async fn new(
        // planning_agent: Arc<dyn PlanningAgent>,
        bandit_policy: Box<dyn BanditPolicy>,
        quality_validator: QualityGateValidator,
        budget_tracker: CAWSBudgetTracker,
    ) -> Result<Self> {
        // Initialize parameter optimizer
        let parameter_optimizer = Arc::new(LLMParameterOptimizer::new(
            bandit_policy,
            quality_validator.clone(),
            budget_tracker.clone(),
        ));
        
        // Initialize reward function
        let reward_function = Arc::new(RewardFunction::new(ObjectiveWeights::default()));
        
        // Initialize rollout manager
        let rollout_manager = Arc::new(RolloutManager::new(
            crate::rollout::SLOMonitor::new(5000, 0.7, chrono::Duration::hours(1))
        ));
        
        Ok(Self {
            // planning_agent,
            parameter_optimizer,
            quality_validator: Arc::new(quality_validator),
            reward_function,
            rollout_manager,
            budget_tracker: Arc::new(budget_tracker),
            current_task_features: Arc::new(RwLock::new(None)),
        })
    }
    
    /// Generate working spec with optimized LLM parameters
    pub async fn generate_working_spec_optimized(
        &self,
        task_description: &str,
        task_features: TaskFeatures,
    ) -> Result<String> {
        tracing::info!("Generating working spec with optimized parameters for task: {}", task_description);
        
        // 1. Set current task features for context
        {
            let mut features = self.current_task_features.write().await;
            *features = Some(task_features.clone());
        }
        
        // 2. Get constraints for this task type
        let constraints = self.get_constraints_for_task(&task_features).await?;
        
        // 3. Check rollout phase and confidence
        let rollout_decision = self.rollout_manager
            .should_apply(&task_features.risk_tier.to_string(), 0.8)
            .await?;
        
        let params = if rollout_decision {
            // 4. Get optimized parameters
            self.parameter_optimizer
                .recommend_parameters(&task_features.risk_tier.to_string(), &task_features, &constraints)
                .await?
        } else {
            // Use baseline parameters
            self.get_baseline_parameters(&task_features).await?
        };
        
        // 5. Execute generation with optimized parameters
        let request_id = Uuid::new_v4();
        let prompt_hash = self.hash_prompt(task_description);
        
        // Create optimized generation request
        let request = crate::orchestration::planning::llm_client::GenerationRequest {
            request_id,
            messages: vec![crate::orchestration::planning::llm_client::Message {
                role: crate::orchestration::planning::llm_client::MessageRole::User,
                content: task_description.to_string(),
            }],
            temperature: Some(params.set.temperature),
            max_tokens: Some(params.set.max_tokens),
            top_p: params.set.top_p,
            frequency_penalty: params.set.frequency_penalty,
            presence_penalty: params.set.presence_penalty,
            seed: params.set.seed,
            prompt_hash: Some(prompt_hash),
            schema_version: Some(1),
            stop_sequences: params.set.stop_sequences.clone(),
        };
        
        // 6. Execute generation and measure performance
        let start = std::time::Instant::now();
        // let response = self.planning_agent.generate_with_request(&request).await?;
        let response_content = "Generated working spec content"; // Placeholder
        let latency = start.elapsed().as_millis() as u64;
        
        // 7. Record outcome for learning
        let outcome = TaskOutcome {
            quality_score: self.estimate_quality(&response_content),
            latency_ms: latency,
            tokens_used: response_content.len(), // Simplified
            success: true,
            caws_compliance: self.validate_caws_compliance(&response_content)?,
        };
        
        self.parameter_optimizer
            .record_outcome(
                request_id,
                &task_features.risk_tier.to_string(),
                task_features.context_fingerprint,
                crate::orchestration::planning::llm_client::UsedParameters {
                    model_name: "gpt-4".to_string(),
                    temperature: params.set.temperature,
                    max_tokens: params.set.max_tokens,
                    top_p: params.set.top_p,
                    frequency_penalty: params.set.frequency_penalty,
                    presence_penalty: params.set.presence_penalty,
                    stop_sequences: params.set.stop_sequences,
                    seed: params.set.seed,
                    schema_version: 1,
                    origin: params.set.origin,
                    policy_version: params.set.policy_version,
                    timestamp: params.set.created_at,
                },
                outcome,
                params.propensity,
            )
            .await?;
        
        // 8. Check for auto-rollback
        self.rollout_manager
            .check_and_rollback(&task_features.risk_tier.to_string(), &[outcome])
            .await?;
        
        Ok(response_content.to_string())
    }
    
    /// Get constraints for a specific task type
    async fn get_constraints_for_task(&self, task_features: &TaskFeatures) -> Result<crate::reward::OptimizationConstraints> {
        // Define constraints based on risk tier
        let constraints = match task_features.risk_tier {
            1 => crate::reward::OptimizationConstraints {
                max_latency_ms: 2000,
                max_tokens: 2000,
                min_quality: 0.9,
                require_caws_compliance: true,
                max_delta_temperature: 0.1,
                max_delta_max_tokens: 100,
            },
            2 => crate::reward::OptimizationConstraints {
                max_latency_ms: 5000,
                max_tokens: 4000,
                min_quality: 0.8,
                require_caws_compliance: true,
                max_delta_temperature: 0.2,
                max_delta_max_tokens: 200,
            },
            _ => crate::reward::OptimizationConstraints {
                max_latency_ms: 10000,
                max_tokens: 8000,
                min_quality: 0.7,
                require_caws_compliance: false,
                max_delta_temperature: 0.3,
                max_delta_max_tokens: 500,
            },
        };
        
        Ok(constraints)
    }
    
    /// Get baseline parameters for a task type
    async fn get_baseline_parameters(&self, task_features: &TaskFeatures) -> Result<crate::parameter_optimizer::RecommendedParameters> {
        // Return baseline parameters based on risk tier
        let baseline_params = match task_features.risk_tier {
            1 => ParameterSet {
                temperature: 0.3,
                max_tokens: 1000,
                top_p: Some(0.9),
                frequency_penalty: None,
                presence_penalty: None,
                stop_sequences: vec![],
                seed: None,
                origin: "baseline".to_string(),
                policy_version: "1.0.0".to_string(),
                created_at: Utc::now(),
            },
            2 => ParameterSet {
                temperature: 0.5,
                max_tokens: 2000,
                top_p: Some(0.9),
                frequency_penalty: None,
                presence_penalty: None,
                stop_sequences: vec![],
                seed: None,
                origin: "baseline".to_string(),
                policy_version: "1.0.0".to_string(),
                created_at: Utc::now(),
            },
            _ => ParameterSet {
                temperature: 0.7,
                max_tokens: 4000,
                top_p: Some(0.9),
                frequency_penalty: None,
                presence_penalty: None,
                stop_sequences: vec![],
                seed: None,
                origin: "baseline".to_string(),
                policy_version: "1.0.0".to_string(),
                created_at: Utc::now(),
            },
        };
        
        Ok(crate::parameter_optimizer::RecommendedParameters {
            set: baseline_params,
            confidence: 1.0,
            ci_reward: (0.0, 0.0),
            ci_latency: (0, 0),
            ci_quality: (0.0, 0.0),
            propensity: 1.0,
            alternative_sets: vec![],
            reasoning: vec!["Baseline parameters".to_string()],
            deployment_safe: true,
        })
    }
    
    /// Hash prompt for tracking
    fn hash_prompt(&self, prompt: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        prompt.hash(&mut hasher);
        hasher.finish()
    }
    
    /// Estimate quality of generated content
    fn estimate_quality(&self, content: &str) -> f64 {
        // Simplified quality estimation
        // In practice, this would use a more sophisticated quality model
        let length_score = (content.len() as f64 / 1000.0).min(1.0);
        let structure_score = if content.contains("##") { 0.9 } else { 0.7 };
        (length_score + structure_score) / 2.0
    }
    
    /// Validate CAWS compliance
    fn validate_caws_compliance(&self, content: &str) -> Result<bool> {
        // Simplified CAWS compliance check
        // In practice, this would use the actual CAWS validator
        Ok(!content.is_empty() && content.len() > 10)
    }
    
    /// Get current optimization status
    pub async fn get_optimization_status(&self) -> OptimizationStatus {
        let features = self.current_task_features.read().await;
        let task_type = features.as_ref()
            .map(|f| f.risk_tier.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        
        OptimizationStatus {
            task_type,
            rollout_phase: RolloutPhase::Shadow, // Would get from rollout manager
            optimization_enabled: true,
            last_optimization: Utc::now(),
        }
    }
}

/// Optimization status information
#[derive(Debug, Clone)]
pub struct OptimizationStatus {
    pub task_type: String,
    pub rollout_phase: RolloutPhase,
    pub optimization_enabled: bool,
    pub last_optimization: DateTime<Utc>,
}

/// Example usage of the optimized planning agent
pub async fn example_usage() -> Result<()> {
    // Create bandit policy
    let bandit_policy = Box::new(ThompsonGaussian::new());
    
    // Create quality validator
    let quality_validator = QualityGateValidator::new(0.1);
    
    // Create budget tracker
    let budget_tracker = CAWSBudgetTracker::new();
    
    // Create optimized planning agent
    let agent = OptimizedPlanningAgent::new(
        bandit_policy,
        quality_validator,
        budget_tracker,
    ).await?;
    
    // Create task features
    let task_features = TaskFeatures {
        risk_tier: 2,
        title_length: 50,
        description_length: 200,
        acceptance_criteria_count: 3,
        scope_files_count: 5,
        max_files: 10,
        max_loc: 500,
        has_external_deps: false,
        complexity_indicators: vec!["api".to_string(), "database".to_string()],
        model_name: Some("gpt-4".to_string()),
        prompt_tokens: Some(100),
        prior_failures: Some(0),
    };
    
    // Generate working spec with optimized parameters
    let spec = agent.generate_working_spec_optimized(
        "Create a REST API for user management",
        task_features,
    ).await?;
    
    println!("Generated spec: {}", spec);
    
    // Get optimization status
    let status = agent.get_optimization_status().await;
    println!("Optimization status: {:?}", status);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_optimized_planning_agent_creation() {
        let bandit_policy = Box::new(ThompsonGaussian::new());
        let quality_validator = QualityGateValidator::new(0.1);
        let budget_tracker = CAWSBudgetTracker::new();
        
        let agent = OptimizedPlanningAgent::new(
            bandit_policy,
            quality_validator,
            budget_tracker,
        ).await;
        
        assert!(agent.is_ok());
    }

    #[tokio::test]
    async fn test_task_features_creation() {
        let features = TaskFeatures {
            risk_tier: 2,
            title_length: 50,
            description_length: 200,
            acceptance_criteria_count: 3,
            scope_files_count: 5,
            max_files: 10,
            max_loc: 500,
            has_external_deps: false,
            complexity_indicators: vec!["api".to_string()],
            model_name: Some("gpt-4".to_string()),
            prompt_tokens: Some(100),
            prior_failures: Some(0),
        };
        
        let fingerprint = features.fingerprint();
        assert!(fingerprint > 0);
    }
}
