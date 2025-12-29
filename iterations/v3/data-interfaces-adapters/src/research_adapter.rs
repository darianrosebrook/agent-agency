//! Research Service Adapter
//!
//! Adapts `agent-research` implementations to `data-interfaces` service traits.

use agent_agency_contracts::{
    types::validation::ValidationIssue, TaskRequest, TaskResponse, WorkingSpec,
};
use agent_research::planning_agent::planning_caws_integration::CawsValidator;
use agent_research::planning_agent::refinement_engine::{
    DefaultRefinementEngine, RefinementEngine,
};
use agent_research::planning_agent::validation_pipeline::ValidationPipeline;
use agent_research::planning_agent::{
    planning_errors::PlanningError,
    types::{PlanningConfig, ValidationStatus},
    PlanningAgent, PlanningRequest,
};
use async_trait::async_trait;
use data_interfaces::service_contracts::{ResearchService, ServiceError};
use std::sync::Arc;

/// Adapter for research service
pub struct ResearchServiceAdapter {
    planning_agent: Arc<PlanningAgent>,
}

impl ResearchServiceAdapter {
    /// Create a new research service adapter
    pub fn new(
        config: PlanningConfig,
        caws_validator: Arc<dyn CawsValidator>,
        validation_pipeline: Arc<ValidationPipeline>,
        refinement_engine: Arc<dyn RefinementEngine>,
    ) -> Self {
        let planning_agent = Arc::new(PlanningAgent::new(
            config,
            caws_validator,
            validation_pipeline,
            refinement_engine,
        ));

        Self { planning_agent }
    }

    /// Create with default components
    pub fn with_defaults() -> Self {
        use agent_research::planning_agent::validation_pipeline::ValidationPipelineConfig;

        let config = PlanningConfig::default();
        let caws_validator: Arc<dyn CawsValidator> = Arc::new(
            agent_research::planning_agent::planning_caws_integration::DefaultCawsValidator::new(),
        );
        let validation_pipeline = Arc::new(ValidationPipeline::new(
            Arc::clone(&caws_validator),
            ValidationPipelineConfig::default(),
        ));
        let refinement_engine = Arc::new(DefaultRefinementEngine::new());

        Self::new(
            config,
            caws_validator,
            validation_pipeline,
            refinement_engine,
        )
    }
}

#[async_trait]
impl ResearchService for ResearchServiceAdapter {
    async fn execute_task(&self, request: TaskRequest) -> Result<TaskResponse, ServiceError> {
        // Convert TaskRequest to PlanningRequest
        let planning_request = PlanningRequest {
            task_request: request,
            config_override: None,
        };

        // Call planning agent
        let planning_response = self
            .planning_agent
            .plan_task(planning_request)
            .await
            .map_err(|e| match e {
                PlanningError::Timeout(msg) => ServiceError::Timeout(msg),
                PlanningError::RiskEscalation { reason } => ServiceError::InvalidRequest(reason),
                _ => ServiceError::Internal(format!("Planning failed: {}", e)),
            })?;

        // Convert PlanningResponse to TaskResponse
        let task_id = uuid::Uuid::new_v4();
        Ok(TaskResponse {
            version: agent_agency_contracts::api_version().to_string(),
            task_id,
            status: if planning_response.validation_results.overall_status
                == ValidationStatus::Passed
            {
                agent_agency_contracts::TaskStatus::Completed
            } else {
                agent_agency_contracts::TaskStatus::Failed
            },
            working_spec: Some(agent_agency_contracts::WorkingSpecSummary {
                id: planning_response.working_spec.id.clone(),
                title: planning_response.working_spec.title.clone(),
                description: planning_response.working_spec.description.clone(),
                goals: planning_response.working_spec.goals.clone(),
                risk_tier: planning_response.working_spec.risk_tier,
                acceptance_criteria_count: planning_response.working_spec.acceptance_criteria.len(),
            }),
            tracking_url: None,
            estimated_completion: None,
            progress: None,
            error: if planning_response.validation_results.overall_status
                == ValidationStatus::Passed
            {
                None
            } else {
                Some(agent_agency_contracts::TaskError {
                    code: "VALIDATION_FAILED".to_string(),
                    message: format!(
                        "Validation issues: {:?}",
                        planning_response.validation_results.issues
                    ),
                    details: None,
                    retryable: Some(false),
                })
            },
            metadata: Default::default(),
        })
    }

    async fn generate_working_spec(
        &self,
        request: &TaskRequest,
    ) -> Result<WorkingSpec, ServiceError> {
        // Create planning request
        let planning_request = PlanningRequest {
            task_request: request.clone(),
            config_override: None,
        };

        // Call planning agent
        let planning_response = self
            .planning_agent
            .plan_task(planning_request)
            .await
            .map_err(|e| match e {
                PlanningError::Timeout(msg) => ServiceError::Timeout(msg),
                PlanningError::RiskEscalation { reason } => ServiceError::InvalidRequest(reason),
                _ => ServiceError::Internal(format!("Planning failed: {}", e)),
            })?;

        Ok(planning_response.working_spec)
    }

    async fn refine_working_spec(
        &self,
        _spec: &mut WorkingSpec,
        _validation_issues: &[ValidationIssue],
    ) -> Result<(), ServiceError> {
        Ok(())
    }
}
