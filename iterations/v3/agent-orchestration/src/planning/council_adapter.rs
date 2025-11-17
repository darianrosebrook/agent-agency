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
            agent_agency_contracts::types::planning::TaskPriority::Medium => ReviewPriority::Normal,
            agent_agency_contracts::types::planning::TaskPriority::High => ReviewPriority::High,
            agent_agency_contracts::types::planning::TaskPriority::Urgent => ReviewPriority::High,
            agent_agency_contracts::types::planning::TaskPriority::Critical => ReviewPriority::Critical,
        }
    }

    /// Convert internal CouncilError to contracts CouncilError
    fn convert_council_error(
        &self,
        error: crate::council_errors::CouncilError,
    ) -> agent_agency_contracts::errors::CouncilError {
        match error {
            crate::council_errors::CouncilError::JudgeError { judge_id, message } => {
                agent_agency_contracts::errors::CouncilError::JudgeError {
                    judge_id: Some(judge_id),
                    reason: message,
                }
            }
            crate::council_errors::CouncilError::SessionTimeout {
                session_id,
                timeout_seconds,
            } => agent_agency_contracts::errors::CouncilError::Timeout {
                session_id,
                timeout_seconds,
            },
            crate::council_errors::CouncilError::AggregationFailure { reason } => {
                agent_agency_contracts::errors::CouncilError::AggregationError { reason }
            }
            crate::council_errors::CouncilError::DecisionFailure { reason, .. } => {
                agent_agency_contracts::errors::CouncilError::DecisionError { reason }
            }
            crate::council_errors::CouncilError::InvalidInput { message } => {
                agent_agency_contracts::errors::CouncilError::SessionError {
                    session_id: None,
                    reason: message,
                }
            }
            crate::council_errors::CouncilError::QuorumFailure { .. } => {
                agent_agency_contracts::errors::CouncilError::SessionError {
                    session_id: None,
                    reason: "Council quorum not met".to_string(),
                }
            }
            _ => agent_agency_contracts::errors::CouncilError::ReviewError {
                session_id: "unknown".to_string(),
                reason: format!("{}", error),
            }
        }
    }
}

#[cfg(feature = "council")]
#[async_trait]
impl CouncilCoordinator for CouncilCoordinatorAdapter {
    async fn start_session(&self, task: &TaskDescriptor) -> CouncilResult<SessionId> {
        let session_id = SessionId(uuid::Uuid::new_v4());
        let session_id_str = session_id.0.to_string();

        // Convert TaskDescriptor to WorkingSpec and serialize to string
        let working_spec = self.task_descriptor_to_working_spec(task);
        let working_spec_json = serde_json::to_string(&working_spec)
            .map_err(|e| agent_agency_contracts::errors::CouncilError::SessionError {
                session_id: None,
                reason: format!("Failed to serialize working spec: {}", e),
            })?;

        // Convert TaskPriority to risk_tier (u8)
        let risk_tier = match task.risk_tier {
            Some(agent_agency_contracts::types::planning::RiskTier::Tier1) => 1,
            Some(agent_agency_contracts::types::planning::RiskTier::Tier2) => 2,
            Some(agent_agency_contracts::types::planning::RiskTier::Tier3) => 3,
            None => match task.priority {
                agent_agency_contracts::types::planning::TaskPriority::Low => 3,
                agent_agency_contracts::types::planning::TaskPriority::Normal => 2,
                agent_agency_contracts::types::planning::TaskPriority::Medium => 2,
                agent_agency_contracts::types::planning::TaskPriority::High => 1,
                agent_agency_contracts::types::planning::TaskPriority::Urgent => 1,
                agent_agency_contracts::types::planning::TaskPriority::Critical => 1,
            },
        };

        // Convert contracts TaskDescriptor to council ReviewContext
        let review_context = crate::judge_backup::types::ReviewContext {
            session_id: session_id_str.clone(),
            working_spec: working_spec_json,
            risk_tier,
            previous_reviews: Vec::new(),
            constraints: std::collections::HashMap::new(),
        };

        // Create session record in database if database operations are available
        if let Some(ref db_ops) = self.db_ops {
            use crate::planning::data_infrastructure_types::CreateCouncilSession;
            use serde_json::json;

            // Extract task_id from task descriptor if available
            let task_id = task.task_id;

            // Create session record with review context
            let create_session = CreateCouncilSession {
                session_id: session_id.0,
                task_id: Some(task_id),
                working_spec_id: Some(working_spec.id.clone()),
                review_context: json!({
                    "risk_tier": risk_tier,
                    "session_id": session_id_str,
                }),
                status: Some("initialized".to_string()),
                selected_judges: None,
                contributions: None,
                progress: Some(0.0),
                metadata: Some(json!({
                    "task_id": task_id.to_string(),
                    "working_spec_id": working_spec.id,
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
        // Use conduct_review which takes working_spec and review_context
        let _dry_run_session = self.council.conduct_review(working_spec, review_context).await
            .map_err(|e| self.convert_council_error(e))?;

        Ok(session_id)
    }

    async fn review_task(
        &self,
        session_id: &SessionId,
        task: &TaskDescriptor,
    ) -> CouncilResult<CouncilVerdict> {
        let session_id_str = session_id.0.to_string();

        // Convert TaskDescriptor to WorkingSpec (we need the actual WorkingSpec, not JSON for conduct_review)
        let working_spec = self.task_descriptor_to_working_spec(task);
        let working_spec_json = serde_json::to_string(&working_spec)
            .map_err(|e| agent_agency_contracts::errors::CouncilError::SessionError {
                session_id: Some(session_id_str.clone()),
                reason: format!("Failed to serialize working spec: {}", e),
            })?;

        // Convert TaskPriority to risk_tier (u8)
        let risk_tier = match task.risk_tier {
            Some(agent_agency_contracts::types::planning::RiskTier::Tier1) => 1,
            Some(agent_agency_contracts::types::planning::RiskTier::Tier2) => 2,
            Some(agent_agency_contracts::types::planning::RiskTier::Tier3) => 3,
            None => match task.priority {
                agent_agency_contracts::types::planning::TaskPriority::Low => 3,
                agent_agency_contracts::types::planning::TaskPriority::Normal => 2,
                agent_agency_contracts::types::planning::TaskPriority::Medium => 2,
                agent_agency_contracts::types::planning::TaskPriority::High => 1,
                agent_agency_contracts::types::planning::TaskPriority::Urgent => 1,
                agent_agency_contracts::types::planning::TaskPriority::Critical => 1,
            },
        };

        // Convert to council ReviewContext
        let review_context = crate::judge_backup::types::ReviewContext {
            session_id: session_id_str,
            working_spec: working_spec_json,
            risk_tier,
            previous_reviews: Vec::new(),
            constraints: std::collections::HashMap::new(),
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

            if let Err(e) = db_ops.update_council_session(session_id.0, update).await {
                tracing::warn!("Failed to update council session status: {}", e);
                // Continue with review despite update failure
            }
        }

        // Perform the actual evaluation using conduct_review
        let council_session = self.council.conduct_review(working_spec, review_context).await
            .map_err(|e| self.convert_council_error(e))?;
        
        // Extract final_decision from the session
        let final_decision = council_session.final_decision.ok_or_else(|| {
            agent_agency_contracts::errors::CouncilError::DecisionError {
                reason: "Council session completed without final decision".to_string(),
            }
        })?;

        // Update session with final decision if database operations available
        if let Some(ref db_ops) = self.db_ops {
            use crate::planning::data_infrastructure_types::UpdateCouncilSession;
            use serde_json::json;

            let final_status = "completed"; // Simplified - council integration needs proper verdict mapping

            // Extract information from FinalDecision enum variant
            let (decision_type, confidence, reason) = match &final_decision {
                crate::decision_making::FinalDecision::Proceed { confidence, .. } => {
                    ("proceed", *confidence, "Task approved by council".to_string())
                }
                crate::decision_making::FinalDecision::Refine { refinement_directive, .. } => {
                    ("refine", 0.5, format!("Refinement required: {} changes", refinement_directive.required_changes.len()))
                }
                crate::decision_making::FinalDecision::Reject { reason, .. } => {
                    ("reject", 0.0, reason.clone())
                }
                crate::decision_making::FinalDecision::Escalate { reason, .. } => {
                    ("escalate", 0.3, reason.clone())
                }
            };

            let update = UpdateCouncilSession {
                status: Some(final_status.to_string()),
                progress: Some(1.0),
                selected_judges: None,
                contributions: None,
                aggregation_result: Some(json!({
                    "decision_type": decision_type,
                    "confidence": confidence,
                })),
                final_decision: Some(json!({
                    "decision_type": decision_type,
                    "confidence": confidence,
                    "reason": reason,
                })),
                completed_at: Some(chrono::Utc::now()),
                metadata: None,
            };

            if let Err(e) = db_ops.update_council_session(session_id.0, update).await {
                tracing::warn!(
                    "Failed to update council session with final decision: {}",
                    e
                );
                // Continue despite update failure - verdict is still returned
            }
        }

        // Convert council FinalDecision enum to contracts CouncilVerdict enum
        // Map FinalDecision variant to CouncilVerdict enum variant
        let verdict = match final_decision {
            crate::decision_making::FinalDecision::Proceed { .. } => CouncilVerdict::Approved,
            crate::decision_making::FinalDecision::Refine { .. } => CouncilVerdict::ConditionalApproval,
            crate::decision_making::FinalDecision::Reject { .. } => CouncilVerdict::Rejected,
            crate::decision_making::FinalDecision::Escalate { .. } => {
                // Treat Escalate as ConditionalApproval
                CouncilVerdict::ConditionalApproval
            }
        };

        Ok(verdict)
    }

    async fn get_session_status(&self, session_id: &SessionId) -> CouncilResult<SessionStatus> {
        // Query session status from database if available
        if let Some(ref db_ops) = self.db_ops {
            match db_ops.get_council_session(session_id.0).await {
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
            budget_limits: Some(agent_agency_contracts::working_spec::BudgetLimits {
                max_files: Some(task.change_budget.max_files as u32),
                max_loc: Some(task.change_budget.max_loc as u32),
            }),
            scope_restrictions: Some(agent_agency_contracts::working_spec::ScopeRestrictions {
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
                        agent_agency_contracts::MoSCoWPriority::Must
                    }
                    agent_agency_contracts::types::planning::TaskPriority::Urgent => {
                        agent_agency_contracts::MoSCoWPriority::Must
                    }
                    agent_agency_contracts::types::planning::TaskPriority::High => {
                        agent_agency_contracts::MoSCoWPriority::Should
                    }
                    agent_agency_contracts::types::planning::TaskPriority::Normal => {
                        agent_agency_contracts::MoSCoWPriority::Should
                    }
                    agent_agency_contracts::types::planning::TaskPriority::Medium => {
                        agent_agency_contracts::MoSCoWPriority::Could
                    }
                    agent_agency_contracts::types::planning::TaskPriority::Low => {
                        agent_agency_contracts::MoSCoWPriority::Could
                    }
                }),
            }]
        } else {
            // Default acceptance criteria if none provided
            vec![agent_agency_contracts::AcceptanceCriterion {
                id: format!("A-{}", task.task_id),
                given: "Task is submitted".to_string(),
                when: format!("Task {} is executed", task.task_id),
                then: format!("Task {} completes successfully", task.task_id),
                priority: Some(agent_agency_contracts::MoSCoWPriority::Should),
            }]
        };

        // Build comprehensive context with environment variables and dependencies
        let context = WorkingSpecContext {
            workspace_root: std::env::var("WORKSPACE_ROOT").unwrap_or_else(|_| ".".to_string()),
            git_branch: std::env::var("GIT_BRANCH").unwrap_or_else(|_| "main".to_string()),
            recent_changes: vec![], // Could be populated from git history if available
            dependencies: {
                // Build dependencies from blast radius
                // WorkingSpecContext.dependencies expects HashMap<String, String>
                let mut deps = HashMap::new();
                for module in &task.blast_radius.modules {
                    deps.insert(
                        module.clone(),
                        "module".to_string(),
                    );
                }
                for ext_dep in &task.blast_radius.external_deps {
                    deps.insert(
                        ext_dep.clone(),
                        "external".to_string(),
                    );
                }
                deps
            },
            environment: match task.execution_mode {
                agent_agency_contracts::types::planning::ExecutionMode::DryRun => {
                    agent_agency_contracts::task_request::Environment::Development
                }
                agent_agency_contracts::types::planning::ExecutionMode::Auto => {
                    agent_agency_contracts::task_request::Environment::Development
                }
                agent_agency_contracts::types::planning::ExecutionMode::Strict => {
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
                    description: format!(
                        "Complete end-to-end test scenario for task {}",
                        task.task_id
                    ),
                    user_journey: format!("End-to-end test journey for task {}", task.task_id),
                    expected_outcomes: vec![],
                }]
            } else {
                vec![]
            },
            coverage_targets: Some(agent_agency_contracts::CoverageTargets {
                line_coverage: Some(if risk_tier == 1 {
                    0.9
                } else if risk_tier == 2 {
                    0.8
                } else {
                    0.7
                }),
                branch_coverage: Some(if risk_tier == 1 {
                    0.95
                } else if risk_tier == 2 {
                    0.85
                } else {
                    0.75
                }),
                mutation_score: if risk_tier == 1 {
                    Some(0.7)
                } else if risk_tier == 2 {
                    Some(0.5)
                } else {
                    Some(0.3)
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
        let metadata = Some(agent_agency_contracts::WorkingSpecMetadata {
            created_at: chrono::Utc::now(),
            created_by: Some("agent-orchestration".to_string()),
            last_modified: Some(chrono::Utc::now()),
            version: Some(1),
            tags: vec![
                format!("priority:{:?}", task.priority),
                format!("execution_mode:{:?}", task.execution_mode),
                format!("task_id:{}", task.task_id),
            ],
        });

        // Build scope from scope_in and scope_out
        // Convert to Vec<ScopeRestrictions> - combine scope_in and scope_out into a single ScopeRestrictions
        let scope = vec![agent_agency_contracts::working_spec::ScopeRestrictions {
            allowed_paths: {
                let mut paths = task.scope_in.allowed_paths.clone();
                if let Some(ref scope_out) = task.scope_out {
                    // scope_out.allowed_paths should be treated as blocked in the combined scope
                    // But we're combining into one ScopeRestrictions, so we keep allowed_paths from scope_in
                }
                paths
            },
            blocked_paths: {
                let mut paths = task.scope_in.blocked_paths.clone();
                if let Some(ref scope_out) = task.scope_out {
                    paths.extend(scope_out.allowed_paths.iter().cloned());
                }
                paths
            },
        }];

        // Extract coverage_targets before moving test_plan
        let coverage_targets = test_plan.coverage_targets.clone();
        
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
            coverage_targets,
            overview: task.description.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

}
