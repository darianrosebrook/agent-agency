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
    CouncilCoordinator,
    types::planning::TaskDescriptor,
    types::council::{CouncilVerdict, SessionId, SessionStatus, SessionStatusType},
    errors::CouncilResult,
};

/// Adapter that wraps agent-constitutional-council to implement contracts::CouncilCoordinator
#[cfg(feature = "council")]
pub struct CouncilCoordinatorAdapter<E: agent_agency_contracts::JudgeEngine> {
    /// The underlying council coordinator implementation
    council: Arc<agent_constitutional_council::CouncilCoordinator<E>>,
}

#[cfg(feature = "council")]
impl<E: agent_agency_contracts::JudgeEngine> CouncilCoordinatorAdapter<E> {
    /// Create a new council coordinator adapter
    pub fn new(council: Arc<agent_constitutional_council::CouncilCoordinator<E>>) -> Self {
        Self { council }
    }
}

#[cfg(feature = "council")]
#[async_trait]
impl<E: agent_agency_contracts::JudgeEngine> CouncilCoordinator for CouncilCoordinatorAdapter<E> {
    async fn start_session(&self, task: &TaskDescriptor) -> CouncilResult<SessionId> {
        // Convert contracts TaskDescriptor to council ReviewContext
        let review_context = agent_constitutional_council::ReviewContext {
            working_spec: self.task_descriptor_to_working_spec(task),
            context: std::collections::HashMap::new(),
            priority: self.map_task_priority(task.priority),
        };

        // TODO: Implement council session tracking
        //       Currently simulates session creation by generating UUID; should implement comprehensive council session tracking with session records, lifecycle state management, and proper session-context association.
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - Session records are created in council storage
        // - Sessions are associated with review contexts
        // - Session lifecycle state is properly maintained
        // - Session management operations are atomic and consistent
        //
        // DEPENDENCIES:
        // - Council storage system (Required)
        // - Session lifecycle management (Required)
        // - Review context association system (Required)
        //
        // ESTIMATED EFFORT: 8-12 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (session management core functionality)
        // - Change Budget: ~200 LOC
        // - Reviewer Requirements: Session management and council integration expertise
        let session_id = SessionId(uuid::Uuid::new_v4());

        // The council doesn't have explicit session management, so we'll just validate
        // that the task can be reviewed by attempting a dry-run evaluation
        let _dry_run_result = self.council.evaluate(&review_context).await
            .map_err(|e| agent_agency_contracts::ContractError::ServiceUnavailable {
                service: "council".to_string()
            })?;

        Ok(session_id)
    }

    async fn review_task(&self, session_id: &SessionId, task: &TaskDescriptor) -> CouncilResult<CouncilVerdict> {
        // Convert to council ReviewContext
        let review_context = agent_constitutional_council::ReviewContext {
            working_spec: self.task_descriptor_to_working_spec(task),
            context: std::collections::HashMap::new(),
            priority: self.map_task_priority(task.priority),
        };

        // Perform the actual evaluation
        let final_decision = self.council.evaluate(&review_context).await
            .map_err(|e| agent_agency_contracts::ContractError::ServiceUnavailable {
                service: "council".to_string()
            })?;

        // Convert council FinalDecision to contracts CouncilVerdict
        let verdict = match final_decision.verdict {
            agent_constitutional_council::CouncilVerdict::Approved => CouncilVerdict::Approved,
            agent_constitutional_council::CouncilVerdict::ConditionalApproval => CouncilVerdict::ConditionalApproval,
            agent_constitutional_council::CouncilVerdict::Rejected => CouncilVerdict::Rejected,
        };

        Ok(verdict)
    }

    async fn get_session_status(&self, session_id: &SessionId) -> CouncilResult<SessionStatus> {
        // TODO: Implement council session status querying
        //       Currently returns completed status; should implement comprehensive council session status querying that retrieves actual session state from council service and maps to SessionStatus enum.
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - Council service is queried for session information
        // - Current session state and progress are retrieved
        // - Session not found errors are handled gracefully
        // - Council status is properly mapped to SessionStatus enum
        //
        // DEPENDENCIES:
        // - Council service API (Required)
        // - Session status mapping utilities (Required)
        // - Error handling for session not found (Required)
        //
        // ESTIMATED EFFORT: 6-8 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (session status querying core functionality)
        // - Change Budget: ~150 LOC
        // - Reviewer Requirements: Council integration and status mapping expertise
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
impl<E: agent_agency_contracts::JudgeEngine> CouncilCoordinatorAdapter<E> {
    /// Convert contracts TaskDescriptor to council WorkingSpec
    ///
    /// Comprehensive conversion that maps all TaskDescriptor fields to WorkingSpec,
    /// including risk tier inference, constraints, acceptance criteria, context,
    /// test plans, rollback plans, and metadata.
    fn task_descriptor_to_working_spec(&self, task: &TaskDescriptor) -> agent_agency_contracts::WorkingSpec {
        use agent_agency_contracts::{WorkingSpec, WorkingSpecConstraints, WorkingSpecContext};
        use std::collections::HashMap;
        use chrono::Utc;

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
            max_iterations: None, // Could be configured based on risk tier
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
                    agent_agency_contracts::types::planning::TaskPriority::Critical => "critical".to_string(),
                    agent_agency_contracts::types::planning::TaskPriority::Urgent => "high".to_string(),
                    agent_agency_contracts::types::planning::TaskPriority::High => "high".to_string(),
                    agent_agency_contracts::types::planning::TaskPriority::Normal => "normal".to_string(),
                    agent_agency_contracts::types::planning::TaskPriority::Medium => "normal".to_string(),
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
            workspace_root: std::env::var("WORKSPACE_ROOT")
                .unwrap_or_else(|_| ".".to_string()),
            git_branch: std::env::var("GIT_BRANCH")
                .unwrap_or_else(|_| "main".to_string()),
            recent_changes: vec![], // Could be populated from git history if available
            dependencies: {
                // Build dependencies from blast radius
                let mut deps = HashMap::new();
                for module in &task.blast_radius.modules {
                    deps.insert(module.clone(), serde_json::json!({
                        "type": "module",
                        "impact": "affected"
                    }));
                }
                for ext_dep in &task.blast_radius.external_deps {
                    deps.insert(ext_dep.clone(), serde_json::json!({
                        "type": "external",
                        "impact": "affected"
                    }));
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
            unit_tests: vec![], // Would be populated from task requirements
            integration_tests: vec![], // Would be populated from task requirements
            e2e_scenarios: if risk_tier == 1 {
                vec![agent_agency_contracts::E2eScenario {
                    id: format!("e2e-{}", task.task_id),
                    name: format!("End-to-end test for {}", task.task_id),
                    description: format!("Complete end-to-end test scenario for task {}", task.task_id),
                    steps: vec![],
                    expected_outcomes: vec![],
                }]
            } else {
                vec![]
            },
            coverage_targets: Some(agent_agency_contracts::CoverageTargets {
                line_coverage: if risk_tier == 1 { 0.9 } else if risk_tier == 2 { 0.8 } else { 0.7 },
                branch_coverage: if risk_tier == 1 { 0.95 } else if risk_tier == 2 { 0.85 } else { 0.75 },
                function_coverage: if risk_tier == 1 { 0.9 } else if risk_tier == 2 { 0.8 } else { 0.7 },
            }),
        };

        // Build rollback plan based on blast radius
        let rollback_plan = agent_agency_contracts::RollbackPlan {
            strategy: if task.blast_radius.data_migration {
                agent_agency_contracts::RollbackStrategy::DatabaseRollback
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
                agent_agency_contracts::DataImpact::DataLoss
            } else {
                agent_agency_contracts::DataImpact::None
            },
            downtime_required: Some(task.blast_radius.data_migration),
            rollback_window_minutes: Some(if risk_tier == 1 { 60 } else { 30 }),
        };

        // Build metadata with comprehensive task information
        let metadata = Some(HashMap::from([
            ("task_id".to_string(), serde_json::json!(task.task_id.to_string())),
            ("priority".to_string(), serde_json::json!(format!("{:?}", task.priority))),
            ("execution_mode".to_string(), serde_json::json!(format!("{:?}", task.execution_mode))),
            ("blast_radius_modules".to_string(), serde_json::json!(task.blast_radius.modules)),
            ("data_migration".to_string(), serde_json::json!(task.blast_radius.data_migration)),
            ("external_deps".to_string(), serde_json::json!(task.blast_radius.external_deps)),
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

    /// Map contracts TaskPriority to council ReviewPriority
    fn map_task_priority(&self, priority: agent_agency_contracts::types::planning::TaskPriority) -> agent_constitutional_council::ReviewPriority {
        match priority {
            agent_agency_contracts::types::planning::TaskPriority::Low => agent_constitutional_council::ReviewPriority::Low,
            agent_agency_contracts::types::planning::TaskPriority::Normal => agent_constitutional_council::ReviewPriority::Normal,
            agent_agency_contracts::types::planning::TaskPriority::High => agent_constitutional_council::ReviewPriority::High,
            agent_agency_contracts::types::planning::TaskPriority::Urgent => agent_constitutional_council::ReviewPriority::High,
            agent_agency_contracts::types::planning::TaskPriority::Critical => agent_constitutional_council::ReviewPriority::Critical,
        }
    }
}
