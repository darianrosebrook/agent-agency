//! Council workflow management
//!
//! This module manages the state transitions and workflow orchestration
//! for council review sessions.

use std::collections::HashMap;
use chrono::{DateTime, Utc};

use crate::error::{CouncilError, CouncilResult};
use crate::council::{CouncilSession, SessionStatus};
use crate::decision_making::FinalDecision;

/// Workflow orchestrator for council sessions
#[derive(Debug)]
pub struct CouncilWorkflow {
    session: CouncilSession,
    workflow_state: WorkflowState,
    state_history: Vec<StateTransition>,
    start_time: DateTime<Utc>,
}

/// Current workflow state
#[derive(Debug, Clone, PartialEq)]
pub enum WorkflowState {
    /// Session initialized
    Initialized,

    /// Judges selected and assigned
    JudgesAssigned,

    /// Reviews in progress
    ReviewsInProgress,

    /// Reviews completed, aggregation starting
    ReviewsCompleted,

    /// Verdict aggregation in progress
    AggregationInProgress,

    /// Aggregation completed, decision making starting
    AggregationCompleted,

    /// Final decision being made
    DecisionMaking,

    /// Session completed successfully
    Completed,

    /// Session failed
    Failed { reason: String },

    /// Session timed out
    Timeout,

    /// Session escalated to human review
    Escalated { reason: String, stakeholders: Vec<String> },
}

/// State transition record
#[derive(Debug, Clone)]
pub struct StateTransition {
    pub from_state: WorkflowState,
    pub to_state: WorkflowState,
    pub timestamp: DateTime<Utc>,
    pub trigger: TransitionTrigger,
    pub metadata: HashMap<String, String>,
}

/// What triggered the state transition
#[derive(Debug, Clone)]
pub enum TransitionTrigger {
    /// Automatic transition based on workflow rules
    Automatic { rule: String },

    /// Manual intervention
    Manual { user: String, reason: String },

    /// Error condition
    Error { error_type: String, message: String },

    /// Timeout condition
    Timeout,

    /// External event
    External { event_type: String, source: String },
}

impl CouncilWorkflow {
    /// Create a new workflow for a council session
    pub fn new(session: CouncilSession) -> Self {
        let start_time = Utc::now();
        let initial_state = WorkflowState::Initialized;

        Self {
            session,
            workflow_state: initial_state.clone(),
            state_history: vec![StateTransition {
                from_state: initial_state.clone(),
                to_state: initial_state,
                timestamp: start_time,
                trigger: TransitionTrigger::Automatic { rule: "session_initialization".to_string() },
                metadata: HashMap::new(),
            }],
            start_time,
        }
    }

    /// Transition to a new workflow state
    pub fn transition_to(&mut self, new_state: WorkflowState, trigger: TransitionTrigger) -> CouncilResult<()> {
        // Validate the transition
        self.validate_transition(&new_state)?;

        let transition = StateTransition {
            from_state: self.workflow_state.clone(),
            to_state: new_state.clone(),
            timestamp: Utc::now(),
            trigger,
            metadata: HashMap::new(),
        };

        self.state_history.push(transition);
        self.workflow_state = new_state;

        Ok(())
    }

    /// Get current workflow state
    pub fn current_state(&self) -> &WorkflowState {
        &self.workflow_state
    }

    /// Check if workflow is in a terminal state
    pub fn is_terminal_state(&self) -> bool {
        matches!(
            self.workflow_state,
            WorkflowState::Completed |
            WorkflowState::Failed { .. } |
            WorkflowState::Timeout |
            WorkflowState::Escalated { .. }
        )
    }

    /// Get workflow duration
    pub fn duration(&self) -> chrono::Duration {
        Utc::now().signed_duration_since(self.start_time)
    }

    /// Get state transition history
    pub fn state_history(&self) -> &[StateTransition] {
        &self.state_history
    }

    /// Check if workflow can proceed to judge assignment
    pub fn can_assign_judges(&self) -> bool {
        matches!(self.workflow_state, WorkflowState::Initialized)
    }

    /// Check if workflow can start reviews
    pub fn can_start_reviews(&self) -> bool {
        matches!(self.workflow_state, WorkflowState::JudgesAssigned)
    }

    /// Check if workflow can start aggregation
    pub fn can_start_aggregation(&self) -> bool {
        matches!(self.workflow_state, WorkflowState::ReviewsCompleted)
    }

    /// Check if workflow can make final decision
    pub fn can_make_decision(&self) -> bool {
        matches!(self.workflow_state, WorkflowState::AggregationCompleted)
    }

    /// Handle final decision and update workflow state
    pub fn handle_final_decision(&mut self, decision: &FinalDecision) -> CouncilResult<()> {
        match decision {
            crate::decision_making::FinalDecision::Proceed { .. } => {
                self.transition_to(
                    WorkflowState::Completed,
                    TransitionTrigger::Automatic { rule: "decision_proceed".to_string() }
                )?;
            },
            crate::decision_making::FinalDecision::Refine { .. } => {
                self.transition_to(
                    WorkflowState::Completed,
                    TransitionTrigger::Automatic { rule: "decision_refine".to_string() }
                )?;
            },
            crate::decision_making::FinalDecision::Reject { .. } => {
                self.transition_to(
                    WorkflowState::Completed,
                    TransitionTrigger::Automatic { rule: "decision_reject".to_string() }
                )?;
            },
            crate::decision_making::FinalDecision::Escalate { reason, required_stakeholders, .. } => {
                self.transition_to(
                    WorkflowState::Escalated {
                        reason: reason.clone(),
                        stakeholders: required_stakeholders.clone(),
                    },
                    TransitionTrigger::Automatic { rule: "decision_escalate".to_string() }
                )?;
            },
        }

        Ok(())
    }

    /// Handle workflow error
    pub fn handle_error(&mut self, error: &CouncilError) -> CouncilResult<()> {
        let error_message = match error {
            CouncilError::JudgeError { judge_id, message } => {
                format!("Judge {} failed: {}", judge_id, message)
            },
            CouncilError::ConsensusFailure { reason } => {
                format!("Consensus failure: {}", reason)
            },
            CouncilError::SessionTimeout { session_id, timeout_seconds } => {
                format!("Session {} timed out after {} seconds", session_id, timeout_seconds)
            },
            CouncilError::QuorumFailure { available, required } => {
                format!("Quorum failure: {}/{} judges available", available, required)
            },
            _ => format!("Workflow error: {}", error),
        };

        self.transition_to(
            WorkflowState::Failed { reason: error_message },
            TransitionTrigger::Error {
                error_type: format!("{:?}", error.category()),
                message: format!("{}", error),
            }
        )?;

        Ok(())
    }

    /// Validate state transition
    fn validate_transition(&self, new_state: &WorkflowState) -> CouncilResult<()> {
        let valid_transitions = self.get_valid_transitions(&self.workflow_state);

        if !valid_transitions.contains(new_state) {
            return Err(CouncilError::WorkflowTransition {
                from: format!("{:?}", self.workflow_state),
                to: format!("{:?}", new_state),
                reason: "Invalid state transition".to_string(),
            });
        }

        Ok(())
    }

    /// Get valid transitions from a given state
    fn get_valid_transitions(&self, from_state: &WorkflowState) -> Vec<WorkflowState> {
        match from_state {
            WorkflowState::Initialized => vec![
                WorkflowState::JudgesAssigned,
                WorkflowState::Failed { reason: "Initialization failed".to_string() },
            ],

            WorkflowState::JudgesAssigned => vec![
                WorkflowState::ReviewsInProgress,
                WorkflowState::Failed { reason: "Judge assignment failed".to_string() },
            ],

            WorkflowState::ReviewsInProgress => vec![
                WorkflowState::ReviewsCompleted,
                WorkflowState::Failed { reason: "Review process failed".to_string() },
                WorkflowState::Timeout,
            ],

            WorkflowState::ReviewsCompleted => vec![
                WorkflowState::AggregationInProgress,
                WorkflowState::Failed { reason: "Review completion failed".to_string() },
            ],

            WorkflowState::AggregationInProgress => vec![
                WorkflowState::AggregationCompleted,
                WorkflowState::Failed { reason: "Aggregation failed".to_string() },
            ],

            WorkflowState::AggregationCompleted => vec![
                WorkflowState::DecisionMaking,
                WorkflowState::Failed { reason: "Aggregation completion failed".to_string() },
            ],

            WorkflowState::DecisionMaking => vec![
                WorkflowState::Completed,
                WorkflowState::Failed { reason: "Decision making failed".to_string() },
                WorkflowState::Escalated {
                    reason: "Decision escalation".to_string(),
                    stakeholders: vec![],
                },
            ],

            // Terminal states have no valid transitions
            WorkflowState::Completed |
            WorkflowState::Failed { .. } |
            WorkflowState::Timeout |
            WorkflowState::Escalated { .. } => vec![],
        }
    }

    /// Get workflow summary
    pub fn summary(&self) -> WorkflowSummary {
        let total_transitions = self.state_history.len().saturating_sub(1); // Subtract initial state
        let duration_ms = self.duration().num_milliseconds() as u64;

        let error_transitions = self.state_history.iter()
            .filter(|t| matches!(t.trigger, TransitionTrigger::Error { .. }))
            .count();

        WorkflowSummary {
            session_id: self.session.session_id.clone(),
            current_state: self.workflow_state.clone(),
            total_transitions,
            error_transitions,
            duration_ms,
            is_terminal: self.is_terminal_state(),
            final_decision: self.session.final_decision.as_ref().map(|d| match d {
                crate::decision_making::FinalDecision::Proceed { .. } => "Proceed".to_string(),
                crate::decision_making::FinalDecision::Refine { .. } => "Refine".to_string(),
                crate::decision_making::FinalDecision::Reject { .. } => "Reject".to_string(),
                crate::decision_making::FinalDecision::Escalate { .. } => "Escalate".to_string(),
            }),
        }
    }

    /// Get access to the underlying session
    pub fn session(&self) -> &CouncilSession {
        &self.session
    }

    /// Get mutable access to the underlying session (for internal use)
    pub(crate) fn session_mut(&mut self) -> &mut CouncilSession {
        &mut self.session
    }
}

/// Workflow summary for reporting
#[derive(Debug, Clone)]
pub struct WorkflowSummary {
    pub session_id: String,
    pub current_state: WorkflowState,
    pub total_transitions: usize,
    pub error_transitions: usize,
    pub duration_ms: u64,
    pub is_terminal: bool,
    pub final_decision: Option<String>,
}

/// Workflow manager for coordinating multiple council sessions
#[derive(Debug)]
pub struct WorkflowManager {
    active_workflows: HashMap<String, CouncilWorkflow>,
    completed_workflows: Vec<CouncilWorkflow>,
    max_concurrent_sessions: usize,
}

impl WorkflowManager {
    /// Create a new workflow manager
    pub fn new(max_concurrent_sessions: usize) -> Self {
        Self {
            active_workflows: HashMap::new(),
            completed_workflows: Vec::new(),
            max_concurrent_sessions,
        }
    }

    /// Start a new workflow for a council session
    pub fn start_workflow(&mut self, session: CouncilSession) -> CouncilResult<String> {
        if self.active_workflows.len() >= self.max_concurrent_sessions {
            return Err(CouncilError::WorkflowTransition {
                from: "manager".to_string(),
                to: "new_workflow".to_string(),
                reason: format!("Maximum concurrent sessions ({}) exceeded", self.max_concurrent_sessions),
            });
        }

        let workflow = CouncilWorkflow::new(session);
        let session_id = workflow.session().session_id.clone();

        self.active_workflows.insert(session_id.clone(), workflow);
        Ok(session_id)
    }

    /// Get a workflow by session ID
    pub fn get_workflow(&self, session_id: &str) -> Option<&CouncilWorkflow> {
        self.active_workflows.get(session_id)
    }

    /// Get a mutable workflow by session ID
    pub fn get_workflow_mut(&mut self, session_id: &str) -> Option<&mut CouncilWorkflow> {
        self.active_workflows.get_mut(session_id)
    }

    /// Complete a workflow and move it to completed list
    pub fn complete_workflow(&mut self, session_id: &str) -> CouncilResult<()> {
        if let Some(workflow) = self.active_workflows.remove(session_id) {
            if workflow.is_terminal_state() {
                self.completed_workflows.push(workflow);
                Ok(())
            } else {
                // Put it back if not terminal
                self.active_workflows.insert(session_id.to_string(), workflow);
                Err(CouncilError::WorkflowTransition {
                    from: "active".to_string(),
                    to: "completed".to_string(),
                    reason: "Workflow is not in terminal state".to_string(),
                })
            }
        } else {
            Err(CouncilError::WorkflowTransition {
                from: "unknown".to_string(),
                to: "completed".to_string(),
                reason: format!("Workflow {} not found", session_id),
            })
        }
    }

    /// Get workflow statistics
    pub fn statistics(&self) -> WorkflowStatistics {
        let total_active = self.active_workflows.len();
        let total_completed = self.completed_workflows.len();

        let completed_successful = self.completed_workflows.iter()
            .filter(|w| matches!(w.current_state(), WorkflowState::Completed))
            .count();

        let completed_failed = self.completed_workflows.iter()
            .filter(|w| matches!(w.current_state(), WorkflowState::Failed { .. }))
            .count();

        let average_duration_ms = if !self.completed_workflows.is_empty() {
            self.completed_workflows.iter()
                .map(|w| w.duration().num_milliseconds() as u64)
                .sum::<u64>() / self.completed_workflows.len() as u64
        } else {
            0
        };

        WorkflowStatistics {
            total_active,
            total_completed,
            completed_successful,
            completed_failed,
            average_duration_ms,
            success_rate: if total_completed > 0 {
                completed_successful as f64 / total_completed as f64
            } else {
                0.0
            },
        }
    }

    /// Clean up old completed workflows
    pub fn cleanup_old_workflows(&mut self, max_age_hours: i64) {
        let cutoff = Utc::now() - chrono::Duration::hours(max_age_hours);
        self.completed_workflows.retain(|w| w.start_time > cutoff);
    }
}

/// Workflow statistics
#[derive(Debug, Clone)]
pub struct WorkflowStatistics {
    pub total_active: usize,
    pub total_completed: usize,
    pub completed_successful: usize,
    pub completed_failed: usize,
    pub average_duration_ms: u64,
    pub success_rate: f64,
}
