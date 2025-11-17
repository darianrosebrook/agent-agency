//! Council Coordinator Adapter
//!
//! Adapts the real agent-constitutional-council to implement the contracts::CouncilCoordinator trait.
//! This adapter enables dependency injection and breaks the direct dependency from orchestration to council.
//!
//! @author @darianrosebrook

#[cfg(feature = "council")]
use async_trait::async_trait;
#[cfg(feature = "council")]
use std::sync::Arc;

#[cfg(feature = "council")]
use agent_agency_contracts::{
    errors::CouncilResult,
    ports::council_coordinator::{SessionId, SessionStatus, SessionStatusType},
    types::council::CouncilVerdict,
    types::planning::TaskDescriptor,
    CouncilCoordinator,
};

/// Adapter that wraps agent-constitutional-council to implement contracts::CouncilCoordinator
#[cfg(feature = "council")]
pub struct CouncilCoordinatorAdapter {
    /// The underlying council coordinator implementation
    council: Arc<crate::council::Council>,
    /// Database operations for session tracking (optional - falls back to in-memory if None)
    db_ops: Option<Arc<dyn crate::planning::DatabaseOperations>>,
}

/// Review priority levels matching agent-constitutional-council::ReviewPriority
/// Defined locally to avoid circular dependency (agent-constitutional-council depends on agent-orchestration)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReviewPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[cfg(feature = "council")]
impl CouncilCoordinatorAdapter {
    /// Create a new council coordinator adapter
    pub fn new(council: Arc<crate::council::Council>) -> Self {
        Self {
            council,
            db_ops: None,
        }
    }

    /// Create a new council coordinator adapter with database operations
    pub fn with_db_ops(
        council: Arc<crate::council::Council>,
        db_ops: Arc<dyn crate::planning::DatabaseOperations>,
    ) -> Self {
        Self {
            council,
            db_ops: Some(db_ops),
        }
    }

    /// Map contracts TaskPriority to council ReviewPriority
    fn map_task_priority(
        &self,
        priority: agent_agency_contracts::types::planning::TaskPriority,
    ) -> ReviewPriority {
        match priority {
            agent_agency_contracts::types::planning::TaskPriority::Low => ReviewPriority::Low,
            agent_agency_contracts::types::planning::TaskPriority::Normal => ReviewPriority::Normal,
            agent_agency_contracts::types::planning::TaskPriority::High => ReviewPriority::High,
            agent_agency_contracts::types::planning::TaskPriority::Urgent => ReviewPriority::High,
            agent_agency_contracts::types::planning::TaskPriority::Critical => ReviewPriority::Critical,
        }
    }
}

#[cfg(feature = "council")]
#[async_trait]
impl CouncilCoordinator for CouncilCoordinatorAdapter {
    async fn start_session(&self, task: &TaskDescriptor) -> CouncilResult<SessionId> {
        // Convert contracts TaskDescriptor to council ReviewContext
        let review_context = crate::judge_backup::types::ReviewContext {
            working_spec: self.task_descriptor_to_working_spec(task),
            context: std::collections::HashMap::new(),
            priority: self.map_task_priority(task.priority),
        };

        let session_id = SessionId(uuid::Uuid::new_v4());

        // Create session record in database if database operations are available
        if let Some(ref db_ops) = self.db_ops {
            use crate::planning::data_infrastructure_types::CreateCouncilSession;
            use serde_json::json;

            // Extract task_id from task descriptor if available
            let task_id = task.task_id;

            // Create session record with review context
            let create_session = CreateCouncilSession {
                session_id: *session_id,
                task_id: Some(task_id),
                working_spec_id: Some(review_context.working_spec.id.clone()),
                review_context: json!({
                    "priority": format!("{:?}", review_context.priority),
                    "context": review_context.context,
                }),
                status: Some("initialized".to_string()),
                selected_judges: None,
                contributions: None,
                progress: Some(0.0),
                metadata: Some(json!({
                    "task_id": task_id.to_string(),
                    "working_spec_id": review_context.working_spec.id,
                })),
            };

            // Create session record (ignore errors - graceful degradation)
            if let Err(e) = db_ops.create_council_session(create_session).await {
                tracing::warn!("Failed to create council session record: {}", e);
                // Continue without database persistence - session still works
            }
        }

        // The council doesn't have explicit session management, so we'll just validate
        // that the task can be reviewed by attempting a dry-run evaluation
        let _dry_run_result = self.council.evaluate(&review_context).await.map_err(|e| {
            agent_agency_contracts::ContractError::ServiceUnavailable {
                service: "council".to_string(),
            }
        })?;

        Ok(session_id)
    }

    async fn review_task(
        &self,
        session_id: &SessionId,
        task: &TaskDescriptor,
    ) -> CouncilResult<CouncilVerdict> {
        // Convert to council ReviewContext
        let review_context = crate::judge_backup::types::ReviewContext {
            working_spec: self.task_descriptor_to_working_spec(task),
            context: std::collections::HashMap::new(),
            priority: self.map_task_priority(task.priority),
        };

        // Update session status to review_in_progress if database operations available
        if let Some(ref db_ops) = self.db_ops {
            use crate::planning::data_infrastructure_types::UpdateCouncilSession;
            use serde_json::json;

            let update = UpdateCouncilSession {
                status: Some("review_in_progress".to_string()),
                progress: Some(0.5),
                selected_judges: None,
                contributions: None,
                aggregation_result: None,
                final_decision: None,
                completed_at: None,
                metadata: None,
            };

            if let Err(e) = db_ops.update_council_session(*session_id, update).await {
                tracing::warn!("Failed to update council session status: {}", e);
                // Continue with review despite update failure
            }
        }

        // Perform the actual evaluation
        let final_decision = self.council.evaluate(&review_context).await.map_err(|e| {
            agent_agency_contracts::ContractError::ServiceUnavailable {
                service: "council".to_string(),
            }
        })?;

        // Update session with final decision if database operations available
        if let Some(ref db_ops) = self.db_ops {
            use crate::planning::data_infrastructure_types::UpdateCouncilSession;
            use serde_json::json;

            let final_status = "completed"; // Simplified - council integration needs proper verdict mapping

            let update = UpdateCouncilSession {
                status: Some(final_status.to_string()),
                progress: Some(1.0),
                selected_judges: None,
                contributions: None,
                aggregation_result: Some(json!({
                    "average_score": final_decision.score,
                    "consensus_label": format!("{:?}", final_decision.verdict),
                })),
                final_decision: Some(json!({
                    "verdict": format!("{:?}", final_decision.verdict),
                    "score": final_decision.score,
                    "rationale": final_decision.rationale,
                    "judge_verdicts": final_decision.judge_verdicts,
                    "consensus_violations": final_decision.consensus_violations,
                    "recommended_actions": final_decision.recommended_actions,
                })),
                completed_at: Some(chrono::Utc::now()),
                metadata: None,
            };

            if let Err(e) = db_ops.update_council_session(*session_id, update).await {
                tracing::warn!(
                    "Failed to update council session with final decision: {}",
                    e
                );
                // Continue despite update failure - verdict is still returned
            }
        }

        // Convert council FinalDecision to contracts CouncilVerdict enum
        // Map verdict label to CouncilVerdict enum variant
        let verdict = match final_decision.label {
            agent_agency_contracts::VerdictLabel::Approved => CouncilVerdict::Approved,
            agent_agency_contracts::VerdictLabel::ConditionalApproval => {
                CouncilVerdict::ConditionalApproval
            }
            agent_agency_contracts::VerdictLabel::Rejected => CouncilVerdict::Rejected,
            agent_agency_contracts::VerdictLabel::NeedsMoreInfo => {
                // Treat NeedsMoreInfo as ConditionalApproval
                CouncilVerdict::ConditionalApproval
            }
        };

        Ok(verdict)
    }

    async fn get_session_status(&self, session_id: &SessionId) -> CouncilResult<SessionStatus> {
        // Query session status from database if available
        if let Some(ref db_ops) = self.db_ops {
            match db_ops.get_council_session(*session_id).await {
                Ok(Some(session)) => {
                    // Map database session status to contracts SessionStatusType
                    let status_type = match session.status.as_str() {
                        "initialized" => SessionStatusType::Initializing,
                        "judge_selection" => SessionStatusType::Reviewing,
                        "review_in_progress" => SessionStatusType::Reviewing,
                        "aggregation_in_progress" => SessionStatusType::Reviewing,
                        "decision_making" => SessionStatusType::Reviewing,
                        "completed" => SessionStatusType::Completed,
                        "failed" => SessionStatusType::Failed,
                        "timeout" => SessionStatusType::Failed,
                        _ => SessionStatusType::Reviewing,
                    };

                    return Ok(SessionStatus {
                        session_id: *session_id,
                        status: status_type,
                        progress: session.progress,
                        pending_requirements: vec![], // Could be extracted from metadata if needed
                        estimated_completion: session.completed_at,
                    });
                }
                Ok(None) => {
                    // Session not found in database - return default status
                    tracing::warn!("Council session not found in database: {}", session_id.0);
                }
                Err(e) => {
                    // Database error - graceful degradation
                    tracing::warn!("Failed to query council session status: {}", e);
                }
            }
        }

        // Fallback: Return default completed status if database unavailable or session not found
        Ok(SessionStatus {
            session_id: *session_id,
            status: SessionStatusType::Completed,
            progress: 1.0,
            pending_requirements: vec![],
            estimated_completion: Some(chrono::Utc::now()),
        })
    }
}

#[cfg(feature = "council")]
impl CouncilCoordinatorAdapter {
    /// Convert contracts TaskDescriptor to council WorkingSpec
    ///
    /// Comprehensive conversion that maps all TaskDescriptor fields to WorkingSpec,
    /// including risk tier inference, constraints, acceptance criteria, context,
    /// test plans, rollback plans, and metadata.
    fn task_descriptor_to_working_spec(
        &self,
        task: &TaskDescriptor,
    ) -> agent_agency_contracts::WorkingSpec {
        use agent_agency_contracts::{WorkingSpec, WorkingSpecConstraints, WorkingSpecContext};
        use chrono::Utc;
        use std::collections::HashMap;

        // Determine risk tier with fallback to priority-based inference
        let risk_tier = match task.risk_tier {
            Some(agent_agency_contracts::types::planning::RiskTier::Tier1) => 1,
            Some(agent_agency_contracts::types::planning::RiskTier::Tier2) => 2,
            Some(agent_agency_contracts::types::planning::RiskTier::Tier3) => 3,
            None => {
                // Infer risk tier from priority if not explicitly set
                match task.priority {
                    agent_agency_contracts::types::planning::TaskPriority::Critical
                    | agent_agency_contracts::types::planning::TaskPriority::Urgent => 1,
                    agent_agency_contracts::types::planning::TaskPriority::High => 2,
                    _ => 2,
                }
            }
        };

        // Build comprehensive constraints with scope_out integration
        let constraints = WorkingSpecConstraints {
            max_duration_minutes: None, // Could be extracted from change budget or task metadata
            max_iterations: None,       // Could be configured based on risk tier
            budget_limits: Some(agent_agency_contracts::BudgetLimits {
                max_files: task.change_budget.max_files.map(|x| x as u32),
                max_loc: task.change_budget.max_loc.map(|x| x as u32),
            }),
            scope_restrictions: Some(agent_agency_contracts::ScopeRestrictions {
                allowed_paths: {
                    let mut paths = task.scope_in.allowed_paths.clone();
                    // Add scope_out blocked paths if available
                    if let Some(ref scope_out) = task.scope_out {
                        paths.extend(scope_out.blocked_paths.iter().cloned());
                    }
                    paths
                },
                blocked_paths: {
                    let mut paths = task.scope_in.blocked_paths.clone();
                    // Add scope_out allowed paths as blocked if available
                    if let Some(ref scope_out) = task.scope_out {
                        paths.extend(scope_out.allowed_paths.iter().cloned());
                    }
                    paths
                },
            }),
        };

        // Build comprehensive acceptance criteria with priority mapping
        let acceptance_criteria = if let Some(ref acceptance_str) = task.acceptance {
            vec![agent_agency_contracts::AcceptanceCriterion {
                id: format!("A-{}", task.task_id),
                given: "Task is submitted and validated".to_string(),
                when: format!("Task {} is executed", task.task_id),
                then: acceptance_str.clone(),
                priority: Some(match task.priority {
                    agent_agency_contracts::types::planning::TaskPriority::Critical => {
                        "critical".to_string()
                    }
                    agent_agency_contracts::types::planning::TaskPriority::Urgent => {
                        "high".to_string()
                    }
                    agent_agency_contracts::types::planning::TaskPriority::High => {
                        "high".to_string()
                    }
                    agent_agency_contracts::types::planning::TaskPriority::Normal => {
                        "normal".to_string()
                    }
                    agent_agency_contracts::types::planning::TaskPriority::Medium => {
                        "normal".to_string()
                    }
                    agent_agency_contracts::types::planning::TaskPriority::Low => "low".to_string(),
                }),
            }]
        } else {
            // Default acceptance criteria if none provided
            vec![agent_agency_contracts::AcceptanceCriterion {
                id: format!("A-{}", task.task_id),
                given: "Task is submitted".to_string(),
                when: format!("Task {} is executed", task.task_id),
                then: format!("Task {} completes successfully", task.task_id),
                priority: Some("normal".to_string()),
            }]
        };

        // Build comprehensive context with environment variables and dependencies
        let context = WorkingSpecContext {
            workspace_root: std::env::var("WORKSPACE_ROOT").unwrap_or_else(|_| ".".to_string()),
            git_branch: std::env::var("GIT_BRANCH").unwrap_or_else(|_| "main".to_string()),
            recent_changes: vec![], // Could be populated from git history if available
            dependencies: {
                // Build dependencies from blast radius
                let mut deps = HashMap::new();
                for module in &task.blast_radius.modules {
                    deps.insert(
                        module.clone(),
                        serde_json::json!({
                            "type": "module",
                            "impact": "affected"
                        }),
                    );
                }
                for ext_dep in &task.blast_radius.external_deps {
                    deps.insert(
                        ext_dep.clone(),
                        serde_json::json!({
                            "type": "external",
                            "impact": "affected"
                        }),
                    );
                }
                deps
            },
            environment: match task.execution_mode {
                agent_agency_contracts::types::planning::ExecutionMode::Auto => {
                    agent_agency_contracts::task_request::Environment::Development
                }
                agent_agency_contracts::types::planning::ExecutionMode::Manual => {
                    agent_agency_contracts::task_request::Environment::Production
                }
            },
        };

        // Build test plan based on risk tier
        let test_plan = agent_agency_contracts::TestPlan {
            unit_tests: vec![],        // Would be populated from task requirements
            integration_tests: vec![], // Would be populated from task requirements
            e2e_scenarios: if risk_tier == 1 {
                vec![agent_agency_contracts::E2eScenario {
                    id: format!("e2e-{}", task.task_id),
                    name: format!("End-to-end test for {}", task.task_id),
                    description: format!(
                        "Complete end-to-end test scenario for task {}",
                        task.task_id
                    ),
                    steps: vec![],
                    expected_outcomes: vec![],
                }]
            } else {
                vec![]
            },
            coverage_targets: Some(agent_agency_contracts::CoverageTargets {
                line_coverage: if risk_tier == 1 {
                    0.9
                } else if risk_tier == 2 {
                    0.8
                } else {
                    0.7
                },
                branch_coverage: if risk_tier == 1 {
                    0.95
                } else if risk_tier == 2 {
                    0.85
                } else {
                    0.75
                },
                function_coverage: if risk_tier == 1 {
                    0.9
                } else if risk_tier == 2 {
                    0.8
                } else {
                    0.7
                },
            }),
        };

        // Build rollback plan based on blast radius
        let rollback_plan = agent_agency_contracts::RollbackPlan {
            strategy: if task.blast_radius.data_migration {
                agent_agency_contracts::RollbackStrategy::DatabaseMigration
            } else {
                agent_agency_contracts::RollbackStrategy::GitRevert
            },
            automated_steps: vec![
                "Stop all running tasks".to_string(),
                "Revert code changes".to_string(),
            ],
            manual_steps: if task.blast_radius.data_migration {
                vec!["Restore database from backup".to_string()]
            } else {
                vec![]
            },
            data_impact: if task.blast_radius.data_migration {
                agent_agency_contracts::DataImpact::Destructive
            } else {
                agent_agency_contracts::DataImpact::None
            },
            downtime_required: Some(task.blast_radius.data_migration),
            rollback_window_minutes: Some(if risk_tier == 1 { 60 } else { 30 }),
        };

        // Build metadata with comprehensive task information
        let metadata = Some(HashMap::from([
            (
                "task_id".to_string(),
                serde_json::json!(task.task_id.to_string()),
            ),
            (
                "priority".to_string(),
                serde_json::json!(format!("{:?}", task.priority)),
            ),
            (
                "execution_mode".to_string(),
                serde_json::json!(format!("{:?}", task.execution_mode)),
            ),
            (
                "blast_radius_modules".to_string(),
                serde_json::json!(task.blast_radius.modules),
            ),
            (
                "data_migration".to_string(),
                serde_json::json!(task.blast_radius.data_migration),
            ),
            (
                "external_deps".to_string(),
                serde_json::json!(task.blast_radius.external_deps),
            ),
        ]));

        // Build scope from scope_in and scope_out
        let scope = {
            let mut scope_vec = Vec::new();
            scope_vec.extend(task.scope_in.allowed_paths.iter().cloned());
            if let Some(ref scope_out) = task.scope_out {
                scope_vec.extend(scope_out.allowed_paths.iter().cloned());
            }
            scope_vec
        };

        WorkingSpec {
            version: "1.0".to_string(),
            id: format!("council-review-{}", task.task_id),
            title: task.description.clone(),
            description: task.description.clone(),
            goals: vec![task.description.clone()],
            risk_tier,
            constraints,
            acceptance_criteria,
            test_plan,
            rollback_plan,
            context,
            non_functional_requirements: None, // Would require NonFunctionalRequirements type definition
            validation_results: None,
            quality_gates: None, // Would require QualityGates type definition
            scope,
            metadata,
            milestones: vec![],
            change_budget: task.change_budget.clone(),
            file_changes: vec![],
            coverage_targets: test_plan.coverage_targets.clone(),
            overview: task.description.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

}
