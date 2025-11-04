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

        // For now, we'll simulate session creation by generating a UUID
        // In a real implementation, this would track sessions in the council
        let session_id = SessionId(uuid::Uuid::new_v4());

        // The council doesn't have explicit session management, so we'll just validate
        // that the task can be reviewed by attempting a dry-run evaluation
        let _dry_run_result = self.council.evaluate(&review_context).await
            .map_err(|e| agent_agency_contracts::errors::ContractError::ServiceUnavailable {
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
            .map_err(|e| agent_agency_contracts::errors::ContractError::ServiceUnavailable {
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
        // For now, return a completed status since the council doesn't maintain session state
        // In a real implementation, this would query the council for session status
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
    fn task_descriptor_to_working_spec(&self, task: &TaskDescriptor) -> agent_agency_contracts::WorkingSpec {
        // This is a simplified conversion - in practice, we'd need to populate more fields
        use agent_agency_contracts::{WorkingSpec, WorkingSpecConstraints, WorkingSpecContext};

        WorkingSpec {
            version: "1.0".to_string(),
            id: format!("council-review-{}", task.task_id),
            title: task.description.clone(),
            description: task.description.clone(),
            goals: vec![task.description.clone()],
            risk_tier: match task.risk_tier {
                Some(agent_agency_contracts::types::planning::RiskTier::Tier1) => 1,
                Some(agent_agency_contracts::types::planning::RiskTier::Tier2) => 2,
                Some(agent_agency_contracts::types::planning::RiskTier::Tier3) => 3,
                None => 2,
            },
            constraints: WorkingSpecConstraints {
                max_duration_minutes: None,
                max_iterations: None,
                budget_limits: Some(agent_agency_contracts::BudgetLimits {
                    max_files: task.change_budget.max_files.map(|x| x as u32),
                    max_loc: task.change_budget.max_loc.map(|x| x as u32),
                }),
                scope_restrictions: Some(agent_agency_contracts::ScopeRestrictions {
                    allowed_paths: task.scope_in.allowed_paths.clone(),
                    blocked_paths: task.scope_in.blocked_paths.clone(),
                }),
            },
            acceptance_criteria: task.acceptance.clone().map(|a| vec![agent_agency_contracts::AcceptanceCriterion {
                id: "A1".to_string(),
                given: "Task is submitted".to_string(),
                when: "Council reviews".to_string(),
                then: a,
                priority: None,
            }]).unwrap_or_default(),
            test_plan: Default::default(),
            rollback_plan: Default::default(),
            context: WorkingSpecContext {
                workspace_root: ".".to_string(),
                git_branch: "main".to_string(),
                recent_changes: vec![],
                dependencies: std::collections::HashMap::new(),
                environment: agent_agency_contracts::task_request::Environment::Development,
            },
            non_functional_requirements: None,
            validation_results: None,
            quality_gates: None,
            scope: vec![],
            metadata: None,
            milestones: vec![],
            change_budget: task.change_budget.clone(),
            file_changes: vec![],
            coverage_targets: None,
            overview: task.description.clone(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
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
