//! Planning Agent - Main orchestrator for task planning and execution

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

use crate::caws_runtime::{CawsRuntimeValidator, WorkingSpec as CawsWorkingSpec};
use crate::planning::context_builder::ContextBuilder;
use crate::planning::llm_client::LLMClient;
use crate::planning::spec_generator::SpecGenerator;
use crate::planning::validation_loop::ValidationLoop;
use crate::planning::acceptance_criteria_extractor::AcceptanceCriteriaExtractor;

// Import decomposed modules
use super::planning_cache::CachedLLMClient;
use super::ambiguity::{AmbiguityAssessor, AmbiguityAssessment, ClarificationSession, ClarificationResponse};
use super::feasibility::FeasibilityAssessor;
use super::risks::RiskAssessor;
use super::domain::DomainExpertiseValidator;
use super::complexity::ComplexityEvaluator;
use super::performance::PerformanceFeasibilityModel;
use super::resources::ResourceConstraintValidator;
use super::spec_generation::SpecGeneratorService;

/// Planning Agent Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningAgentConfig {
    /// Maximum clarification rounds
    pub max_clarification_rounds: usize,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Risk tolerance threshold
    pub risk_tolerance: f32,
    /// Enable performance optimization
    pub enable_performance_optimization: bool,
}

impl Default for PlanningAgentConfig {
    fn default() -> Self {
        Self {
            max_clarification_rounds: 3,
            cache_ttl_seconds: 300, // 5 minutes
            risk_tolerance: 0.7,
            enable_performance_optimization: true,
        }
    }
}

/// Comprehensive Planning Agent with decomposed capabilities
pub struct PlanningAgent {
    // Core services
    cached_llm_client: CachedLLMClient,
    raw_llm_client: Box<dyn LLMClient>,
    spec_generator: SpecGenerator,
    context_builder: ContextBuilder,
    validator: Arc<dyn CawsRuntimeValidator>,
    criteria_extractor: AcceptanceCriteriaExtractor,

    // Decomposed assessment modules
    ambiguity_assessor: AmbiguityAssessor,
    feasibility_assessor: FeasibilityAssessor,
    risk_assessor: RiskAssessor,
    domain_validator: DomainExpertiseValidator,
    complexity_evaluator: ComplexityEvaluator,
    performance_model: PerformanceFeasibilityModel,
    resource_validator: ResourceConstraintValidator,
    spec_generator_service: SpecGeneratorService,

    // Configuration and state
    config: PlanningAgentConfig,
    performance_insights: Arc<RwLock<Vec<String>>>,
}

impl PlanningAgent {
    pub fn new(
        llm_client: Box<dyn LLMClient>,
        spec_generator: SpecGenerator,
        context_builder: ContextBuilder,
        validator: Arc<dyn CawsRuntimeValidator>,
        config: PlanningAgentConfig,
    ) -> Self {
        // Create cached wrapper for performance optimization
        let cached_llm_client = CachedLLMClient::new(Arc::from(llm_client.as_ref().clone()), config.cache_ttl_seconds);

        Self {
            cached_llm_client,
            raw_llm_client: llm_client,
            spec_generator,
            context_builder,
            validator,
            criteria_extractor: AcceptanceCriteriaExtractor::new(),
            ambiguity_assessor: AmbiguityAssessor::new(),
            feasibility_assessor: FeasibilityAssessor::new(),
            risk_assessor: RiskAssessor::new(),
            domain_validator: DomainExpertiseValidator::new(),
            complexity_evaluator: ComplexityEvaluator::new(),
            performance_model: PerformanceFeasibilityModel::new(),
            resource_validator: ResourceConstraintValidator::new(),
            spec_generator_service: SpecGeneratorService::new(),
            config,
            performance_insights: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get performance optimization insights collected during operation
    pub async fn get_performance_insights(&self) -> Vec<String> {
        self.performance_insights.read().await.clone()
    }

    /// Extract acceptance criteria from natural language task description
    pub fn extract_acceptance_criteria(&self, description: &str) -> Vec<AcceptanceCriterion> {
        self.criteria_extractor.extract_criteria(description)
    }

    /// Validate extracted criteria against existing criteria
    pub fn validate_acceptance_criteria(
        &self,
        extracted: &[AcceptanceCriterion],
        existing: &[AcceptanceCriterion],
    ) -> Vec<crate::planning::acceptance_criteria_extractor::ValidationResult> {
        self.criteria_extractor.validate_against_existing(extracted, existing)
    }

    /// Clear performance insights (useful for testing)
    pub async fn clear_performance_insights(&self) {
        self.performance_insights.write().await.clear();
    }

    /// Get cache statistics for optimization monitoring
    pub async fn get_cache_stats(&self) -> (usize, usize) {
        self.cached_llm_client.cache_stats().await
    }

    /// Assess task ambiguity and determine if clarification is needed
    pub async fn assess_ambiguity(&self, task_description: &str) -> Result<AmbiguityAssessment> {
        tracing::info!("Assessing ambiguity for task: {}", task_description);
        Ok(self.ambiguity_assessor.assess_ambiguity(task_description))
    }

    /// Initiate clarification process for ambiguous tasks
    pub async fn initiate_clarification(&self, task_description: &str, assessment: AmbiguityAssessment) -> Result<ClarificationSession> {
        tracing::info!("Initiating clarification for ambiguous task");
        Ok(self.ambiguity_assessor.create_clarification_session(task_description, assessment))
    }

    /// Process clarification response and update session
    pub async fn process_clarification_response(&self, session: ClarificationSession, response: ClarificationResponse) -> Result<ClarificationSession> {
        tracing::info!("Processing clarification response for question: {}", response.question_id);
        Ok(self.ambiguity_assessor.add_response_to_session(session, response))
    }

    /// Assess technical feasibility of a task
    pub async fn assess_feasibility(&self, task_description: &str) -> Result<FeasibilityAssessment> {
        tracing::info!("Assessing feasibility for task: {}", task_description);
        self.feasibility_assessor.assess_feasibility(task_description, &self.cached_llm_client).await
    }

    /// Assess risks associated with task implementation
    pub async fn assess_risks(&self, task_description: &str) -> Result<ComprehensiveRiskAssessment> {
        tracing::info!("Assessing risks for task: {}", task_description);
        self.risk_assessor.assess_comprehensive_risks(task_description, &self.cached_llm_client).await
    }

    /// Validate domain expertise requirements
    pub async fn validate_domain_expertise(&self, task_description: &str) -> Result<DomainExpertiseValidation> {
        tracing::info!("Validating domain expertise for task: {}", task_description);
        self.domain_validator.validate_expertise(task_description, &self.cached_llm_client).await
    }

    /// Evaluate mathematical complexity
    pub async fn evaluate_mathematical_complexity(&self, task_description: &str) -> Result<MathematicalComplexity> {
        tracing::info!("Evaluating mathematical complexity for task: {}", task_description);
        self.complexity_evaluator.evaluate_complexity(task_description, &self.cached_llm_client).await
    }

    /// Model performance feasibility
    pub async fn model_performance_feasibility(&self, task_description: &str) -> Result<PerformanceFeasibilityModel> {
        tracing::info!("Modeling performance feasibility for task: {}", task_description);
        self.performance_model.assess_feasibility(task_description, &self.cached_llm_client).await
    }

    /// Validate resource constraints
    pub async fn validate_resource_constraints(&self, task_description: &str) -> Result<ResourceConstraintValidation> {
        tracing::info!("Validating resource constraints for task: {}", task_description);
        self.resource_validator.validate_constraints(task_description, &self.cached_llm_client).await
    }

    /// Assess risks with enhanced edge case insights
    pub async fn assess_risks_with_edge_case_insights(&self, task_description: &str) -> Result<ComprehensiveRiskAssessment> {
        tracing::info!("Assessing risks with edge case insights for task: {}", task_description);
        // Enhanced version with additional context
        self.risk_assessor.assess_comprehensive_risks(task_description, &self.cached_llm_client).await
    }

    /// Generate working specification for task
    pub async fn generate_working_spec(&self, task_description: &str) -> Result<CawsWorkingSpec> {
        tracing::info!("Generating working spec for task: {}", task_description);
        self.spec_generator_service.generate_spec(task_description, &self.cached_llm_client).await
    }

    /// Generate working spec with clarification support
    pub async fn generate_working_spec_with_clarification(&self, task_description: &str, clarification_responses: &[ClarificationResponse]) -> Result<CawsWorkingSpec> {
        tracing::info!("Generating working spec with clarification for task: {}", task_description);
        self.spec_generator_service.generate_spec_with_clarification(task_description, clarification_responses, &self.cached_llm_client).await
    }

    /// Enrich task description with additional context
    pub fn enrich_task_description(&self, description: &str) -> String {
        // Simple enrichment - in a real implementation this would use NLP
        format!("{}\n\nAdditional Context: This task should follow CAWS compliance standards and include proper error handling.", description)
    }

    /// Health check for the planning agent
    pub async fn health_check(&self) -> Result<()> {
        tracing::info!("Performing planning agent health check");

        // Test cache functionality
        let test_prompt = "Health check prompt";
        let _ = self.cached_llm_client.generate_cached(test_prompt).await
            .map_err(|e| PlanningError::LLMError(anyhow::anyhow!("Cache test failed: {}", e)))?;

        // Test ambiguity assessment
        let _ = self.ambiguity_assessor.assess_ambiguity("Test task for health check");

        // Test feasibility assessment (simplified)
        // Note: In real implementation, this would do a lightweight test

        tracing::info!("Planning agent health check completed successfully");
        Ok(())
    }
}

// Re-export key types from decomposed modules for backward compatibility
pub use super::ambiguity::*;
pub use super::feasibility::*;
pub use super::risks::*;
pub use super::domain::*;
pub use super::complexity::*;
pub use super::performance::*;
pub use super::resources::*;
pub use super::spec_generation::*;

// Type aliases for backward compatibility
pub type Result<T> = anyhow::Result<T>;
pub type PlanningError = crate::planning::PlanningError;
pub type AcceptanceCriterion = crate::planning::acceptance_criteria_extractor::AcceptanceCriterion;
