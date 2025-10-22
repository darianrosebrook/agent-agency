//! LLM Parameter Feedback Loop Integration Example
//!
//! Demonstrates how to integrate the LLM parameter optimization system
//! with the planning agent for adaptive parameter tuning.

use crate::{
    bandit_policy::{TaskFeatures, ThompsonGaussian},
    counterfactual_log::{CounterfactualLogger, TaskOutcome},
    parameter_optimizer::{LLMParameterOptimizer, OptimizationConstraints},
    rollout::{RolloutManager, RolloutPhase},
    caws_integration::{CAWSComplianceValidator, CAWSBudgetTracker},
};
use std::sync::Arc;
use uuid::Uuid;

/// Example integration of LLM parameter feedback loop
pub struct LLMParameterFeedbackExample {
    parameter_optimizer: Arc<LLMParameterOptimizer>,
    rollout_manager: Arc<RolloutManager>,
    caws_validator: Arc<CAWSComplianceValidator>,
    cf_logger: Arc<CounterfactualLogger>,
}

impl LLMParameterFeedbackExample {
    pub fn new() -> Self {
        Self {
            parameter_optimizer: Arc::new(LLMParameterOptimizer::new()),
            rollout_manager: Arc::new(RolloutManager::new()),
            caws_validator: Arc::new(CAWSComplianceValidator::new()),
            cf_logger: Arc::new(CounterfactualLogger::new()),
        }
    }

    /// Example: Generate LLM response with optimized parameters
    pub async fn generate_with_optimized_parameters(
        &self,
        prompt: &str,
        task_type: &str,
        task_features: &TaskFeatures,
    ) -> Result<String> {
        // 1. Get constraints for this task
        let constraints = self.get_constraints_for_task_type(task_type);
        
        // 2. Check rollout phase
        let should_apply = self.rollout_manager
            .should_apply(task_type, 0.8) // Min confidence
            .await?;
        
        let params = if should_apply {
            // 3. Get optimized parameters
            let recommendations = self.parameter_optimizer
                .recommend_parameters(task_type, task_features, &constraints)
                .await?;
            
            if recommendations.deployment_safe {
                recommendations.set
            } else {
                // Fall back to baseline if not deployment safe
                self.get_baseline_parameters(task_type).await?
            }
        } else {
            // Use baseline parameters
            self.get_baseline_parameters(task_type).await?
        };
        
        // 4. Execute generation with parameters
        let request_id = Uuid::new_v4();
        let response = self.execute_generation(prompt, &params, request_id).await?;
        
        // 5. Record outcome for learning
        let outcome = self.measure_outcome(&response, &params).await?;
        
        // 6. Record for counterfactual logging and learning
        self.parameter_optimizer
            .record_outcome(
                request_id,
                task_type,
                task_features.fingerprint(),
                self.convert_to_used_parameters(&params),
                outcome.clone(),
                0.8, // Propensity from bandit policy
            )
            .await?;
        
        // 7. Check for auto-rollback
        self.rollout_manager
            .check_and_rollback(task_type, &[outcome])
            .await?;
        
        Ok(response.content)
    }

    /// Get constraints for a task type
    fn get_constraints_for_task_type(&self, task_type: &str) -> OptimizationConstraints {
        match task_type {
            "feasibility_analysis" => OptimizationConstraints {
                max_latency_ms: 2000,
                max_tokens: 2000,
                require_caws: true,
                max_delta_temperature: 0.1,
                max_delta_max_tokens: 100,
            },
            "task_breakdown" => OptimizationConstraints {
                max_latency_ms: 5000,
                max_tokens: 4000,
                require_caws: true,
                max_delta_temperature: 0.2,
                max_delta_max_tokens: 200,
            },
            "implementation_planning" => OptimizationConstraints {
                max_latency_ms: 10000,
                max_tokens: 8000,
                require_caws: true,
                max_delta_temperature: 0.3,
                max_delta_max_tokens: 500,
            },
            _ => OptimizationConstraints {
                max_latency_ms: 5000,
                max_tokens: 2000,
                require_caws: true,
                max_delta_temperature: 0.2,
                max_delta_max_tokens: 200,
            },
        }
    }

    /// Get baseline parameters for a task type
    async fn get_baseline_parameters(&self, task_type: &str) -> Result<crate::bandit_policy::ParameterSet> {
        use crate::bandit_policy::ParameterSet;
        use chrono::Utc;
        
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

    /// Execute LLM generation with parameters
    async fn execute_generation(
        &self,
        prompt: &str,
        params: &crate::bandit_policy::ParameterSet,
        request_id: Uuid,
    ) -> Result<GenerationResponse> {
        // This would integrate with the actual LLM client
        // For this example, we'll simulate the response
        
        let start_time = std::time::Instant::now();
        
        // Simulate LLM generation
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        let latency = start_time.elapsed().as_millis() as u64;
        
        Ok(GenerationResponse {
            request_id,
            content: format!("Generated response for: {}", prompt),
            usage: TokenUsage {
                prompt_tokens: 50,
                completion_tokens: 100,
                total_tokens: 150,
            },
            finish_reason: crate::orchestration::planning::llm_client::FinishReason::Stop,
            parameters_used: self.convert_to_used_parameters(params),
        })
    }

    /// Measure outcome metrics
    async fn measure_outcome(
        &self,
        response: &GenerationResponse,
        params: &crate::bandit_policy::ParameterSet,
    ) -> Result<TaskOutcome> {
        // Simulate quality measurement
        let quality_score = self.estimate_quality(&response.content);
        
        // Simulate latency measurement
        let latency_ms = 100; // Would be measured from actual generation
        
        // Simulate CAWS compliance check
        let caws_compliance = self.check_caws_compliance(&response.content);
        
        Ok(TaskOutcome {
            quality_score,
            latency_ms,
            tokens_used: response.usage.total_tokens as usize,
            success: matches!(response.finish_reason, crate::orchestration::planning::llm_client::FinishReason::Stop),
            caws_compliance,
        })
    }

    /// Estimate quality score (simplified)
    fn estimate_quality(&self, content: &str) -> f64 {
        // Simplified quality estimation
        // In practice, this would use a quality assessment model
        if content.len() > 50 {
            0.9
        } else {
            0.7
        }
    }

    /// Check CAWS compliance (simplified)
    fn check_caws_compliance(&self, content: &str) -> bool {
        // Simplified compliance check
        // In practice, this would use CAWS compliance validator
        !content.contains("TODO") && !content.contains("PLACEHOLDER")
    }

    /// Convert ParameterSet to UsedParameters
    fn convert_to_used_parameters(&self, params: &crate::bandit_policy::ParameterSet) -> crate::orchestration::planning::llm_client::UsedParameters {
        use crate::orchestration::planning::llm_client::UsedParameters;
        use chrono::Utc;
        
        UsedParameters {
            model_name: "gpt-4".to_string(),
            temperature: params.temperature,
            max_tokens: params.max_tokens,
            top_p: params.top_p,
            frequency_penalty: params.frequency_penalty,
            presence_penalty: params.presence_penalty,
            stop_sequences: params.stop_sequences.clone(),
            seed: params.seed,
            schema_version: 1,
            origin: params.origin.clone(),
            policy_version: params.policy_version.clone(),
            timestamp: params.created_at,
        }
    }

    /// Initialize rollout for a task type
    pub async fn initialize_rollout(&self, task_type: &str) -> Result<()> {
        self.rollout_manager.advance_phase(task_type).await?;
        Ok(())
    }

    /// Get rollout status for a task type
    pub fn get_rollout_status(&self, task_type: &str) -> Option<crate::rollout::RolloutState> {
        self.rollout_manager.get_state(task_type)
    }

    /// Run offline evaluation
    pub async fn run_offline_evaluation(&self, task_type: &str) -> Result<()> {
        // This would run offline evaluation using the counterfactual logger
        // to validate that the learned policy improves over baseline
        
        let evaluator = self.cf_logger.evaluator();
        let decisions = evaluator.get_decisions(task_type);
        
        if decisions.len() < 100 {
            return Err(anyhow::anyhow!("Insufficient data for offline evaluation: {} decisions", decisions.len()));
        }
        
        // Run offline evaluation
        let policy = ThompsonGaussian::new();
        let result = evaluator.evaluate_ips(&policy, task_type)?;
        
        println!("Offline evaluation result for {}: {:.3} estimated reward (CI: {:.3}-{:.3})", 
                 task_type, result.estimated_reward, result.confidence_interval.0, result.confidence_interval.1);
        
        Ok(())
    }
}

/// Mock response structure for the example
#[derive(Debug, Clone)]
pub struct GenerationResponse {
    pub request_id: Uuid,
    pub content: String,
    pub usage: TokenUsage,
    pub finish_reason: crate::orchestration::planning::llm_client::FinishReason,
    pub parameters_used: crate::orchestration::planning::llm_client::UsedParameters,
}

#[derive(Debug, Clone)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

pub type Result<T> = std::result::Result<T, anyhow::Error>;
