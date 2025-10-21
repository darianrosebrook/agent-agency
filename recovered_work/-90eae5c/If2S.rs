//! Core Planning Agent implementation
//!
//! The PlanningAgent is responsible for transforming TaskRequests into
//! validated WorkingSpecs through a multi-stage planning pipeline.

use std::sync::Arc;
use tokio::time::{timeout, Duration};
use uuid::Uuid;

use crate::error::{PlanningError, PlanningResult};
use crate::caws_integration::CawsValidator;
use crate::validation_pipeline::ValidationPipeline;
use crate::refinement_engine::RefinementEngine;

/// Configuration for the planning agent
#[derive(Debug, Clone)]
pub struct PlanningConfig {
    /// Maximum time allowed for planning (in seconds)
    pub max_planning_time_seconds: u64,

    /// Maximum refinement iterations
    pub max_refinement_iterations: u32,

    /// Whether to enable automatic refinement
    pub enable_auto_refinement: bool,

    /// Risk tier escalation thresholds
    pub risk_escalation_thresholds: RiskEscalationThresholds,
}

impl Default for PlanningConfig {
    fn default() -> Self {
        Self {
            max_planning_time_seconds: 300, // 5 minutes
            max_refinement_iterations: 3,
            enable_auto_refinement: true,
            risk_escalation_thresholds: RiskEscalationThresholds::default(),
        }
    }
}

/// Risk escalation thresholds
#[derive(Debug, Clone)]
pub struct RiskEscalationThresholds {
    /// Maximum files for T1 tasks before escalation
    pub t1_max_files: u32,

    /// Maximum LOC for T1 tasks before escalation
    pub t1_max_loc: u32,

    /// Maximum duration for T1 tasks before escalation
    pub t1_max_duration_hours: u32,
}

impl Default for RiskEscalationThresholds {
    fn default() -> Self {
        Self {
            t1_max_files: 25,
            t1_max_loc: 1000,
            t1_max_duration_hours: 8,
        }
    }
}

/// Request to the planning agent
#[derive(Debug, Clone)]
pub struct PlanningRequest {
    /// The task request to plan
    pub task_request: agent_agency_contracts::task_request::TaskRequest,

    /// Planning configuration override (optional)
    pub config_override: Option<PlanningConfig>,
}

/// Response from the planning agent
#[derive(Debug, Clone)]
pub struct PlanningResponse {
    /// Generated working specification
    pub working_spec: agent_agency_contracts::working_spec::WorkingSpec,

    /// Planning metadata
    pub metadata: PlanningMetadata,

    /// Validation results
    pub validation_results: ValidationResults,

    /// Refinement history (if any refinements were applied)
    pub refinement_history: Vec<RefinementRecord>,
}

/// Planning operation metadata
#[derive(Debug, Clone)]
pub struct PlanningMetadata {
    /// Total planning time
    pub planning_duration: Duration,

    /// Number of refinement iterations performed
    pub refinement_iterations: u32,

    /// Whether human intervention was required
    pub human_intervention_required: bool,

    /// Risk assessment result
    pub risk_assessment: RiskAssessment,
}

/// Risk assessment result
#[derive(Debug, Clone)]
pub struct RiskAssessment {
    /// Assessed risk tier
    pub assessed_tier: agent_agency_contracts::task_request::RiskTier,

    /// Risk factors identified
    pub risk_factors: Vec<String>,

    /// Whether escalation is recommended
    pub escalation_recommended: bool,
}

/// Validation results summary
#[derive(Debug, Clone)]
pub struct ValidationResults {
    /// Overall validation status
    pub overall_status: ValidationStatus,

    /// CAWS compliance score (0.0-1.0)
    pub caws_compliance_score: f64,

    /// Individual validation issues
    pub issues: Vec<ValidationIssue>,

    /// Applied refinements
    pub applied_refinements: Vec<String>,
}

/// Validation status
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationStatus {
    Passed,
    PassedWithRefinements,
    Failed,
    EscalationRequired,
}

/// Individual validation issue
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    /// Issue severity
    pub severity: IssueSeverity,

    /// Issue category
    pub category: String,

    /// Human-readable description
    pub description: String,

    /// Suggested fix
    pub suggestion: Option<String>,
}

/// Issue severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

/// Refinement record
#[derive(Debug, Clone)]
pub struct RefinementRecord {
    /// Refinement iteration number
    pub iteration: u32,

    /// Issues that triggered refinement
    pub triggering_issues: Vec<String>,

    /// Applied refinement actions
    pub applied_actions: Vec<String>,

    /// Whether refinement was successful
    pub successful: bool,
}

/// The main Planning Agent
pub struct PlanningAgent {
    config: PlanningConfig,
    caws_validator: Arc<CawsValidator>,
    validation_pipeline: Arc<ValidationPipeline>,
    refinement_engine: Arc<RefinementEngine>,
}

impl PlanningAgent {
    /// Create a new planning agent
    pub fn new(
        config: PlanningConfig,
        caws_validator: Arc<CawsValidator>,
        validation_pipeline: Arc<ValidationPipeline>,
        refinement_engine: Arc<RefinementEngine>,
    ) -> Self {
        Self {
            config,
            caws_validator,
            validation_pipeline,
            refinement_engine,
        }
    }

    /// Plan a task by transforming TaskRequest into validated WorkingSpec
    pub async fn plan_task(&self, request: PlanningRequest) -> PlanningResult<PlanningResponse> {
        let start_time = std::time::Instant::now();
        let config = request.config_override.unwrap_or_else(|| self.config.clone());

        // Validate input task request
        self.validate_task_request(&request.task_request)?;

        // Perform risk assessment
        let risk_assessment = self.assess_risk(&request.task_request)?;

        // Check for immediate escalation
        if risk_assessment.escalation_recommended {
            return Err(PlanningError::RiskEscalation {
                reason: format!("Risk assessment indicates escalation required: {:?}", risk_assessment.risk_factors),
            });
        }

        // Generate initial working specification
        let mut working_spec = self.generate_working_spec(&request.task_request).await?;

        // Run planning pipeline with validation and refinement
        let planning_result = timeout(
            Duration::from_secs(config.max_planning_time_seconds),
            self.run_planning_pipeline(&mut working_spec, &config)
        ).await
        .map_err(|_| PlanningError::Timeout(format!("Planning exceeded {} seconds", config.max_planning_time_seconds)))?;

        let (validation_results, refinement_history) = planning_result?;

        // Create response metadata
        let metadata = PlanningMetadata {
            planning_duration: start_time.elapsed(),
            refinement_iterations: refinement_history.len() as u32,
            human_intervention_required: validation_results.overall_status == ValidationStatus::EscalationRequired,
            risk_assessment,
        };

        Ok(PlanningResponse {
            working_spec,
            metadata,
            validation_results,
            refinement_history,
        })
    }

    /// Validate the input task request
    fn validate_task_request(&self, task_request: &agent_agency_contracts::task_request::TaskRequest) -> PlanningResult<()> {
        // Basic validation - ensure required fields are present
        if task_request.description.trim().is_empty() {
            return Err(PlanningError::InvalidTaskRequest("Task description cannot be empty".to_string()));
        }

        // Validate risk tier constraints
        match task_request.constraints.as_ref().and_then(|c| c.risk_tier.as_ref()) {
            Some(risk_tier) => {
                // Additional validation for T1 tasks
                if matches!(risk_tier, agent_agency_contracts::task_request::RiskTier::Tier1) {
                    // T1 tasks require explicit budget limits
                    if let Some(constraints) = &task_request.constraints {
                        if constraints.budget_limits.is_none() {
                            return Err(PlanningError::InvalidTaskRequest(
                                "T1 tasks must specify budget limits".to_string()
                            ));
                        }
                    }
                }
            }
            None => {
                return Err(PlanningError::InvalidTaskRequest("Risk tier must be specified".to_string()));
            }
        }

        Ok(())
    }

    /// Assess risk level of the task
    fn assess_risk(&self, task_request: &agent_agency_contracts::task_request::TaskRequest) -> PlanningResult<RiskAssessment> {
        let mut risk_factors = Vec::new();
        let mut escalation_recommended = false;

        let risk_tier = task_request.constraints.as_ref()
            .and_then(|c| c.risk_tier.as_ref())
            .cloned()
            .unwrap_or(agent_agency_contracts::task_request::RiskTier::Tier2);

        // Assess based on constraints
        if let Some(constraints) = &task_request.constraints {
            if let Some(budget) = &constraints.budget_limits {
                match risk_tier {
                    agent_agency_contracts::task_request::RiskTier::Tier1 => {
                        if budget.max_files.unwrap_or(0) > self.config.risk_escalation_thresholds.t1_max_files {
                            risk_factors.push(format!("T1 task exceeds max files limit ({})", budget.max_files.unwrap()));
                            escalation_recommended = true;
                        }
                        if budget.max_loc.unwrap_or(0) > self.config.risk_escalation_thresholds.t1_max_loc {
                            risk_factors.push(format!("T1 task exceeds max LOC limit ({})", budget.max_loc.unwrap()));
                            escalation_recommended = true;
                        }
                    }
                    _ => {} // Other tiers have more flexibility
                }
            }

            if let Some(max_duration) = constraints.max_duration_minutes {
                let hours = max_duration / 60;
                if matches!(risk_tier, agent_agency_contracts::task_request::RiskTier::Tier1)
                    && hours > self.config.risk_escalation_thresholds.t1_max_duration_hours {
                    risk_factors.push(format!("T1 task exceeds max duration limit ({} hours)", hours));
                    escalation_recommended = true;
                }
            }
        }

        Ok(RiskAssessment {
            assessed_tier: risk_tier,
            risk_factors,
            escalation_recommended,
        })
    }

    /// Generate initial working specification from task request
    async fn generate_working_spec(&self, task_request: &agent_agency_contracts::task_request::TaskRequest) -> PlanningResult<agent_agency_contracts::working_spec::WorkingSpec> {
        // This is a simplified implementation - in practice this would use
        // more sophisticated analysis of the task description
        let working_spec_id = format!("{}-{}", task_request.id.simple(), Uuid::new_v4().simple());

        // Extract goals from task description (simplified)
        let goals = self.extract_goals_from_description(&task_request.description)?;

        // Generate basic acceptance criteria
        let acceptance_criteria = self.generate_acceptance_criteria(&task_request.description)?;

        // Determine risk tier
        let risk_tier = task_request.constraints.as_ref()
            .and_then(|c| c.risk_tier.as_ref())
            .map(|rt| match rt {
                agent_agency_contracts::task_request::RiskTier::Tier1 => 1,
                agent_agency_contracts::task_request::RiskTier::Tier2 => 2,
                agent_agency_contracts::task_request::RiskTier::Tier3 => 3,
            })
            .unwrap_or(2);

        // Create working spec constraints
        let constraints = self.create_working_spec_constraints(task_request)?;

        // Generate test plan
        let test_plan = self.generate_test_plan(task_request, risk_tier)?;

        // Create rollback plan
        let rollback_plan = self.generate_rollback_plan(task_request)?;

        Ok(agent_agency_contracts::working_spec::WorkingSpec {
            version: "1.0.0".to_string(),
            id: working_spec_id,
            title: self.generate_title_from_description(&task_request.description),
            description: task_request.description.clone(),
            goals,
            risk_tier,
            constraints,
            acceptance_criteria,
            test_plan,
            rollback_plan,
            context: self.create_working_spec_context(task_request)?,
            non_functional_requirements: None, // TODO: Extract from task request
            validation_results: None, // Will be filled by CAWS validation
            metadata: Some(agent_agency_contracts::working_spec::WorkingSpecMetadata {
                created_at: chrono::Utc::now(),
                created_by: task_request.metadata.as_ref().and_then(|m| m.requester.clone()),
                last_modified: None,
                version: Some(1),
                tags: task_request.metadata.as_ref().and_then(|m| m.tags.clone()).unwrap_or_default(),
            }),
        })
    }

    /// Run the complete planning pipeline with validation and refinement
    async fn run_planning_pipeline(
        &self,
        working_spec: &mut agent_agency_contracts::working_spec::WorkingSpec,
        config: &PlanningConfig,
    ) -> PlanningResult<(ValidationResults, Vec<RefinementRecord>)> {
        let mut refinement_history = Vec::new();
        let mut iteration = 0u32;

        loop {
            iteration += 1;

            // Run validation pipeline
            let validation_result = self.validation_pipeline.validate_working_spec(working_spec).await?;

            // Check if validation passed
            if validation_result.overall_status == ValidationStatus::Passed {
                return Ok((validation_result, refinement_history));
            }

            // Check if we've exceeded max iterations
            if iteration >= config.max_refinement_iterations {
                let mut validation_results = validation_result;
                validation_results.overall_status = ValidationStatus::EscalationRequired;
                return Ok((validation_results, refinement_history));
            }

            // Check if refinement is disabled
            if !config.enable_auto_refinement {
                let mut validation_results = validation_result;
                validation_results.overall_status = ValidationStatus::EscalationRequired;
                return Ok((validation_results, refinement_history));
            }

            // Attempt refinement
            let refinement_result = self.refinement_engine.refine_working_spec(
                working_spec,
                &validation_result.issues
            ).await?;

            let record = RefinementRecord {
                iteration,
                triggering_issues: validation_result.issues.iter().map(|i| i.description.clone()).collect(),
                applied_actions: refinement_result.applied_actions,
                successful: refinement_result.successful,
            };

            refinement_history.push(record);

            if !refinement_result.successful {
                // Refinement failed, require escalation
                let mut validation_results = validation_result;
                validation_results.overall_status = ValidationStatus::EscalationRequired;
                return Ok((validation_results, refinement_history));
            }
        }
    }

    // Helper methods for working spec generation...

    fn extract_goals_from_description(&self, description: &str) -> PlanningResult<Vec<String>> {
        // Simplified goal extraction - in practice this would use NLP
        Ok(vec![format!("Successfully complete: {}", description)])
    }

    fn generate_acceptance_criteria(&self, description: &str) -> PlanningResult<Vec<agent_agency_contracts::working_spec::AcceptanceCriterion>> {
        // Simplified acceptance criteria generation
        Ok(vec![
            agent_agency_contracts::working_spec::AcceptanceCriterion {
                id: "A1".to_string(),
                given: "Valid task request".to_string(),
                when: format!("Task is executed: {}", description),
                then: "Task completes successfully with all requirements met".to_string(),
                priority: Some(agent_agency_contracts::working_spec::MoSCoWPriority::Must),
            }
        ])
    }

    fn generate_title_from_description(&self, description: &str) -> String {
        // Simple title generation - take first sentence or truncate
        let first_sentence = description.split('.').next().unwrap_or(description);
        if first_sentence.len() > 80 {
            format!("{}...", &first_sentence[..77])
        } else {
            first_sentence.to_string()
        }
    }

    fn create_working_spec_constraints(&self, task_request: &agent_agency_contracts::task_request::TaskRequest) -> PlanningResult<agent_agency_contracts::working_spec::WorkingSpecConstraints> {
        let constraints = task_request.constraints.as_ref();

        Ok(agent_agency_contracts::working_spec::WorkingSpecConstraints {
            max_duration_minutes: constraints.and_then(|c| c.max_duration_minutes),
            max_iterations: constraints.and_then(|c| c.max_iterations),
            budget_limits: constraints.and_then(|c| c.budget_limits.as_ref()).map(|b| {
                agent_agency_contracts::working_spec::BudgetLimits {
                    max_files: b.max_files,
                    max_loc: b.max_loc,
                }
            }),
            scope_restrictions: constraints.and_then(|c| c.scope_restrictions.as_ref()).map(|s| {
                agent_agency_contracts::working_spec::ScopeRestrictions {
                    allowed_paths: s.allowed_paths.clone(),
                    blocked_paths: s.blocked_paths.clone(),
                }
            }),
        })
    }

    fn generate_test_plan(&self, task_request: &agent_agency_contracts::task_request::TaskRequest, risk_tier: u32) -> PlanningResult<agent_agency_contracts::working_spec::TestPlan> {
        let coverage_targets = match risk_tier {
            1 => Some(agent_agency_contracts::working_spec::CoverageTargets {
                line_coverage: Some(0.9),
                branch_coverage: Some(0.85),
                mutation_score: Some(0.7),
            }),
            2 => Some(agent_agency_contracts::working_spec::CoverageTargets {
                line_coverage: Some(0.8),
                branch_coverage: Some(0.75),
                mutation_score: Some(0.5),
            }),
            _ => Some(agent_agency_contracts::working_spec::CoverageTargets {
                line_coverage: Some(0.7),
                branch_coverage: None,
                mutation_score: None,
            }),
        };

        Ok(agent_agency_contracts::working_spec::TestPlan {
            unit_tests: vec![agent_agency_contracts::working_spec::UnitTestSpec {
                description: "Basic functionality test".to_string(),
                target_function: None,
                test_cases: vec!["happy_path".to_string(), "edge_cases".to_string()],
            }],
            integration_tests: vec![],
            e2e_scenarios: vec![],
            coverage_targets,
        })
    }

    fn generate_rollback_plan(&self, task_request: &agent_agency_contracts::task_request::TaskRequest) -> PlanningResult<agent_agency_contracts::working_spec::RollbackPlan> {
        let risk_tier = task_request.constraints.as_ref()
            .and_then(|c| c.risk_tier.as_ref())
            .cloned()
            .unwrap_or(agent_agency_contracts::task_request::RiskTier::Tier2);

        let (strategy, data_impact) = match risk_tier {
            agent_agency_contracts::task_request::RiskTier::Tier1 => (
                agent_agency_contracts::working_spec::RollbackStrategy::GitRevert,
                agent_agency_contracts::working_spec::DataImpact::Reversible,
            ),
            _ => (
                agent_agency_contracts::working_spec::RollbackStrategy::ManualRevert,
                agent_agency_contracts::working_spec::DataImpact::None,
            ),
        };

        Ok(agent_agency_contracts::working_spec::RollbackPlan {
            strategy,
            automated_steps: vec!["git revert".to_string()],
            manual_steps: vec!["Verify system state".to_string()],
            data_impact,
            downtime_required: Some(matches!(risk_tier, agent_agency_contracts::task_request::RiskTier::Tier1)),
            rollback_window_minutes: Some(30),
        })
    }

    fn create_working_spec_context(&self, task_request: &agent_agency_contracts::task_request::TaskRequest) -> PlanningResult<agent_agency_contracts::working_spec::WorkingSpecContext> {
        let context = task_request.context.as_ref();

        Ok(agent_agency_contracts::working_spec::WorkingSpecContext {
            workspace_root: context
                .map(|c| c.workspace_root.clone())
                .unwrap_or_else(|| "/workspace".to_string()),
            git_branch: context
                .map(|c| c.git_branch.clone())
                .unwrap_or_else(|| "main".to_string()),
            recent_changes: context
                .map(|c| c.recent_changes.iter().map(|change| {
                    agent_agency_contracts::working_spec::FileChange {
                        file: change.file.clone(),
                        change_type: match change.change_type {
                            agent_agency_contracts::task_request::ChangeType::Added => agent_agency_contracts::working_spec::ChangeType::Added,
                            agent_agency_contracts::task_request::ChangeType::Modified => agent_agency_contracts::working_spec::ChangeType::Modified,
                            agent_agency_contracts::task_request::ChangeType::Deleted => agent_agency_contracts::working_spec::ChangeType::Deleted,
                        },
                        timestamp: change.timestamp,
                    }
                }).collect())
                .unwrap_or_default(),
            dependencies: context
                .map(|c| c.dependencies.clone())
                .unwrap_or_default(),
            environment: context
                .map(|c| c.environment.clone())
                .unwrap_or(agent_agency_contracts::task_request::Environment::Development),
        })
    }
}
