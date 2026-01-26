//! Unified Orchestrator Task Executor
//!
//! Implements TaskExecutor trait to connect OrchestratorService with UnifiedOrchestrator.
//! Converts TaskDescriptor to WorkingSpec and executes via UnifiedOrchestrator.
//!
//! @author @darianrosebrook

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::info;

use agent_agency_contracts::working_spec::{BudgetLimits, ScopeRestrictions};
use agent_agency_contracts::{
    AcceptanceCriterion, ExecutionArtifacts, RollbackPlan, TaskDescriptor, TestPlan, WorkingSpec,
    WorkingSpecConstraints, WorkingSpecContext,
};
use agent_orchestration::orchestration::unified_orchestrator::UnifiedOrchestrator;
use data_infrastructure::{
    CouncilSessionData, CoordinationEventData, DecisionPointData, ExecutionResultWithObservability,
    JudgeContributionData, TaskExecutor, TaskObservabilityData, WorkerActionData,
};

/// Task executor that wraps UnifiedOrchestrator
pub struct UnifiedOrchestratorTaskExecutor {
    orchestrator: Arc<UnifiedOrchestrator>,
}

impl UnifiedOrchestratorTaskExecutor {
    /// Create a new UnifiedOrchestratorTaskExecutor
    pub fn new(orchestrator: Arc<UnifiedOrchestrator>) -> Self {
        Self { orchestrator }
    }
}

#[async_trait]
impl TaskExecutor for UnifiedOrchestratorTaskExecutor {
    async fn execute_task(
        &self,
        task_descriptor: &TaskDescriptor,
    ) -> Result<ExecutionArtifacts, anyhow::Error> {
        // Delegate to execute_task_with_observability and extract artifacts
        let result = self.execute_task_with_observability(task_descriptor).await?;
        Ok(result.artifacts)
    }

    async fn execute_task_with_observability(
        &self,
        task_descriptor: &TaskDescriptor,
    ) -> Result<ExecutionResultWithObservability, anyhow::Error> {
        info!(
            "Executing task {} via UnifiedOrchestrator with observability",
            task_descriptor.task_id
        );

        // Convert TaskDescriptor to WorkingSpec
        let working_spec = task_descriptor_to_working_spec(task_descriptor)?;

        // Execute plan via UnifiedOrchestrator
        let execution_result = self
            .orchestrator
            .execute_plan(working_spec.clone())
            .await
            .map_err(|e| anyhow!("UnifiedOrchestrator execution failed: {}", e))?;

        info!(
            "UnifiedOrchestrator completed plan {} with {} artifacts",
            execution_result.plan_id,
            execution_result.artifacts.len()
        );

        // Extract ExecutionArtifacts from ExecutionResult
        let artifacts = if let Some(first_artifact) = execution_result.artifacts.first() {
            first_artifact.clone()
        } else {
            // Create a summary artifact from execution result
            let mut artifact = ExecutionArtifacts::default();
            artifact.task_id = execution_result.plan_id;
            artifact.working_spec_id = working_spec.id.clone();
            artifact.iteration = execution_result.iterations;
            artifact.metadata =
                Some(agent_agency_contracts::execution_artifacts::ArtifactMetadata::default());
            artifact
        };

        // Convert ExecutionObservability to TaskObservabilityData
        let observability = execution_result.observability.map(|obs| {
            TaskObservabilityData {
                council_sessions: obs.council_sessions.into_iter().map(|session| {
                    CouncilSessionData {
                        session_id: session.session_id,
                        timestamp: session.timestamp,
                        phase: session.phase,
                        verdict: session.verdict,
                        confidence: session.confidence,
                        judge_contributions: session.judge_contributions.into_iter().map(|contrib| {
                            JudgeContributionData {
                                judge_id: contrib.judge_id,
                                judge_name: contrib.judge_name,
                                verdict: contrib.verdict,
                                confidence: contrib.confidence,
                                reasoning: contrib.reasoning,
                            }
                        }).collect(),
                    }
                }).collect(),
                worker_actions_data: obs.worker_actions.into_iter().map(|action| {
                    WorkerActionData {
                        worker_id: action.worker_id,
                        action: action.action,
                        milestone_id: action.milestone_id,
                        timestamp: action.timestamp,
                        duration_ms: action.duration_ms,
                        success: action.success,
                        error: action.error,
                    }
                }).collect(),
                decision_points: obs.decision_points.into_iter().map(|dp| {
                    DecisionPointData {
                        decision_id: dp.decision_id,
                        decision_type: dp.decision_type,
                        timestamp: dp.timestamp,
                        chosen_option: dp.chosen_option,
                        alternatives_count: dp.alternatives_count,
                        confidence: dp.confidence,
                        reasoning: dp.reasoning,
                    }
                }).collect(),
                coordination_events: obs.coordination_events.into_iter().map(|event| {
                    CoordinationEventData {
                        event_type: event.event_type,
                        timestamp: event.timestamp,
                        details: event.details,
                    }
                }).collect(),
            }
        });

        Ok(ExecutionResultWithObservability {
            artifacts,
            observability,
        })
    }
}

/// Convert TaskDescriptor to WorkingSpec
fn task_descriptor_to_working_spec(task_descriptor: &TaskDescriptor) -> Result<WorkingSpec> {
    use agent_agency_contracts::task_request::Environment;
    use chrono::Utc;

    // Extract title from description (first line or first 50 chars)
    let title = task_descriptor
        .description
        .lines()
        .next()
        .unwrap_or(&task_descriptor.description)
        .chars()
        .take(50)
        .collect::<String>()
        .trim()
        .to_string();

    // Determine risk tier
    let risk_tier = task_descriptor
        .risk_tier
        .clone()
        .map(|rt| match rt {
            agent_agency_contracts::types::planning::RiskTier::Tier1 => 1,
            agent_agency_contracts::types::planning::RiskTier::Tier2 => 2,
            agent_agency_contracts::types::planning::RiskTier::Tier3 => 3,
        })
        .unwrap_or(2);

    let acceptance_criteria = if let Some(ref acceptance_str) = task_descriptor.acceptance {
        vec![AcceptanceCriterion {
            id: "A1".to_string(),
            given: "Task is submitted".to_string(),
            when: "Execution completes".to_string(),
            then: acceptance_str.clone(),
            priority: None,
        }]
    } else {
        vec![AcceptanceCriterion {
            id: "A1".to_string(),
            given: "Task is submitted".to_string(),
            when: "Execution completes".to_string(),
            then: format!("Task '{}' is completed successfully", title),
            priority: None,
        }]
    };

    // Create test plan
    let test_plan = TestPlan {
        unit_tests: vec![],
        integration_tests: vec![],
        e2e_scenarios: vec![],
        coverage_targets: None,
    };

    // Create rollback plan
    let rollback_plan = RollbackPlan {
        strategy: agent_agency_contracts::working_spec::RollbackStrategy::GitRevert,
        automated_steps: vec!["Revert all file changes".to_string()],
        manual_steps: vec!["Verify original state restored".to_string()],
        data_impact: agent_agency_contracts::working_spec::DataImpact::None,
        downtime_required: Some(false),
        rollback_window_minutes: Some(5),
    };

    // Create context
    let context = WorkingSpecContext {
        workspace_root: std::env::current_dir()
            .ok()
            .and_then(|p| p.to_str().map(|s| s.to_string()))
            .unwrap_or_else(|| ".".to_string()),
        git_branch: "main".to_string(),
        recent_changes: vec![],
        dependencies: std::collections::HashMap::new(),
        environment: Environment::Development,
    };

    // Create constraints from change budget
    let constraints = WorkingSpecConstraints {
        max_duration_minutes: None,
        max_iterations: None,
        budget_limits: Some(BudgetLimits {
            max_files: Some(task_descriptor.change_budget.max_files as u32),
            max_loc: Some(task_descriptor.change_budget.max_loc as u32),
        }),
        scope_restrictions: Some(ScopeRestrictions {
            allowed_paths: task_descriptor.scope_in.allowed_paths.clone(),
            blocked_paths: task_descriptor.scope_in.blocked_paths.clone(),
        }),
    };

    // Extract goals from description
    let goals: Vec<String> = task_descriptor
        .description
        .lines()
        .filter(|line| !line.trim().is_empty())
        .take(5)
        .map(|line| line.trim().to_string())
        .collect();

    let goals = if goals.is_empty() {
        vec![format!("Complete task: {}", title)]
    } else {
        goals
    };

    // Create change budget (convert from TaskDescriptor format)
    let change_budget = agent_agency_contracts::planning_io::ChangeBudget {
        max_files: task_descriptor.change_budget.max_files,
        max_loc: task_descriptor.change_budget.max_loc,
        max_migrations: task_descriptor.change_budget.max_migrations,
        allow_breaking_changes: task_descriptor.change_budget.allow_breaking_changes,
        allow_new_dependencies: task_descriptor.change_budget.allow_new_dependencies,
        enforcement_mode: task_descriptor.change_budget.enforcement_mode.clone(),
    };

    Ok(WorkingSpec {
        version: "1.0".to_string(),
        id: format!("TASK-{}", task_descriptor.task_id),
        title,
        description: task_descriptor.description.clone(),
        goals,
        risk_tier,
        constraints,
        acceptance_criteria,
        test_plan,
        rollback_plan,
        context,
        non_functional_requirements: None,
        validation_results: None,
        quality_gates: None,
        scope: vec![],
        metadata: Some(agent_agency_contracts::WorkingSpecMetadata {
            created_at: Utc::now(),
            created_by: Some("task-executor".to_string()),
            last_modified: Some(Utc::now()),
            version: Some(1),
            tags: vec![],
        }),
        milestones: vec![],
        change_budget,
        file_changes: vec![],
        coverage_targets: None,
        overview: task_descriptor.description.clone(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    })
}
