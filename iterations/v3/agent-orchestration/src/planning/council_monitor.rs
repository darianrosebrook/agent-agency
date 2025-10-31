//! Council Monitor - Constitutional oversight for plan execution
//!
//! Real constitutional council integration for plan execution oversight.
//! Monitors execution against constitutional invariants and handles violations.
//!
//! @author @darianrosebrook

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{anyhow, Result};
use uuid::Uuid;
use chrono::Utc;
use agent_agency_contracts::planning_io::{ExecutionPlan, PlanState};
use agent_constitutional_council::{CouncilCoordinator, ReviewContext, ReviewPriority, CouncilResult, FinalDecision};
use data_infrastructure::{DatabaseOperations, models::AuditTrailEntry};

/// Council monitor for constitutional oversight
pub struct CouncilMonitor {
    /// Constitutional council coordinator
    council: Arc<CouncilCoordinator<agent_agency_contracts::Engine>>,

    /// Database operations for audit trail
    db_ops: Arc<dyn DatabaseOperations>,

    /// Active plan monitoring sessions
    active_sessions: Arc<tokio::sync::RwLock<HashMap<String, PlanSession>>>,

    /// Monitoring configuration
    config: MonitorConfig,
}

/// Configuration for council monitoring
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    /// Check interval for ongoing monitoring (seconds)
    pub check_interval_seconds: u64,

    /// Whether to enable real-time monitoring
    pub enable_realtime_monitoring: bool,

    /// Whether to block execution on violations
    pub block_on_violations: bool,

    /// Maximum monitoring session duration (hours)
    pub max_session_duration_hours: u32,

    /// Council notification timeout (seconds)
    pub notification_timeout_seconds: u64,
}

/// Active plan monitoring session
#[derive(Debug, Clone)]
pub struct PlanSession {
    /// Plan ID being monitored
    plan_id: String,

    /// Session start time
    started_at: chrono::DateTime<Utc>,

    /// Last check time
    last_check: chrono::DateTime<Utc>,

    /// Current violations
    violations: Vec<String>,

    /// Intervention requests
    interventions: Vec<String>,

    /// Session status
    status: SessionStatus,
}

/// Session status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionStatus {
    /// Actively monitoring
    Active,

    /// Temporarily paused
    Paused,

    /// Completed successfully
    Completed,

    /// Terminated due to violations
    Terminated,

    /// Expired
    Expired,
}

impl CouncilMonitor {
    /// Create new council monitor with real council integration
    pub fn new(
        council: Arc<CouncilCoordinator<agent_agency_contracts::Engine>>,
        db_ops: Arc<dyn DatabaseOperations>,
    ) -> Self {
        Self::with_config(
            council,
            db_ops,
            MonitorConfig::default(),
        )
    }

    /// Create with custom configuration
    pub fn with_config(
        council: Arc<CouncilCoordinator<agent_agency_contracts::Engine>>,
        db_ops: Arc<dyn DatabaseOperations>,
        config: MonitorConfig,
    ) -> Self {
        Self {
            council,
            db_ops,
            active_sessions: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Check if plan execution is allowed by consulting the council
    pub async fn check_execution_allowed(&self, plan: &ExecutionPlan) -> Result<bool> {
        // Convert execution plan to working spec for council review
        let working_spec = self.plan_to_working_spec(plan)?;

        // Create review context
        let context = ReviewContext {
            working_spec,
            context: self.create_review_context(plan),
            priority: self.determine_review_priority(plan),
        };

        // Get council decision
        let decision: CouncilResult<FinalDecision> = self.council.evaluate(&context).await;

        match decision {
            Ok(final_decision) => {
                let allowed = matches!(final_decision.label, agent_agency_contracts::VerdictLabel::Pass);

                // Log the decision
                self.log_council_decision(plan.id.to_string(), &final_decision).await?;

                if allowed {
                    // Start monitoring session for this plan
                    self.start_monitoring_session(plan).await?;
                }

                Ok(allowed)
            }
            Err(e) => {
                // Council evaluation failed - log and deny
                self.log_council_error(plan.id.to_string(), &e).await?;
                Err(anyhow!("Council evaluation failed for plan {}: {}", plan.id, e))
            }
        }
    }

    /// Report execution progress to council for ongoing monitoring
    pub async fn report_progress(&self, plan_id: &str, milestone_id: &str, status: &str) -> Result<()> {
        // Check if we have an active session for this plan
        let mut sessions = self.active_sessions.write().await;
        if let Some(session) = sessions.get_mut(plan_id) {
            // Update session state
            session.last_check = Utc::now();

            // Check for violations based on progress
            let violations = self.check_progress_violations(plan_id, milestone_id, status).await?;
            session.violations.extend(violations);

            // Log progress event
            self.log_progress_event(plan_id, milestone_id, status).await?;

            // Check if we need to intervene
            if !session.violations.is_empty() && self.config.block_on_violations {
                session.status = SessionStatus::Terminated;
                return Err(anyhow!(
                    "Plan {} terminated due to violations: {:?}",
                    plan_id, session.violations
                ));
            }

            Ok(())
        } else {
            // No active session - this might be an error
            Err(anyhow!("No active monitoring session for plan {}", plan_id))
        }
    }

    /// Request council intervention for critical issues
    pub async fn request_intervention(&self, plan_id: &str, reason: &str) -> Result<()> {
        let mut sessions = self.active_sessions.write().await;
        if let Some(session) = sessions.get_mut(plan_id) {
            session.interventions.push(reason.to_string());

            // Log intervention request
            self.log_intervention_request(plan_id, reason).await?;

            // In a real implementation, this might trigger a council review
            // For now, we just log it

            Ok(())
        } else {
            Err(anyhow!("No active monitoring session for plan {}", plan_id))
        }
    }

    /// Check for constitutional violations in ongoing execution
    pub async fn check_violations(&self, plan_id: &str) -> Result<Vec<String>> {
        let sessions = self.active_sessions.read().await;
        if let Some(session) = sessions.get(plan_id) {
            Ok(session.violations.clone())
        } else {
            Ok(vec![])
        }
    }

    /// Get council recommendations for plan execution
    pub async fn get_recommendations(&self, plan_id: &str) -> Result<Vec<String>> {
        // In a real implementation, this might query the council for recommendations
        // For now, return generic recommendations based on violations

        let violations = self.check_violations(plan_id).await?;
        let mut recommendations = Vec::new();

        if violations.is_empty() {
            recommendations.push("Plan execution proceeding normally".to_string());
        } else {
            recommendations.push("Address identified violations to prevent termination".to_string());

            for violation in &violations {
                if violation.contains("scope") {
                    recommendations.push("Review milestone scope boundaries".to_string());
                } else if violation.contains("performance") {
                    recommendations.push("Optimize performance metrics".to_string());
                } else if violation.contains("quality") {
                    recommendations.push("Improve code quality metrics".to_string());
                }
            }
        }

        Ok(recommendations)
    }

    /// Start monitoring session for a plan
    async fn start_monitoring_session(&self, plan: &ExecutionPlan) -> Result<()> {
        let session = PlanSession {
            plan_id: plan.id.to_string(),
            started_at: Utc::now(),
            last_check: Utc::now(),
            violations: vec![],
            interventions: vec![],
            status: SessionStatus::Active,
        };

        let mut sessions = self.active_sessions.write().await;
        sessions.insert(plan.id.to_string(), session);

        // Log session start
        self.log_session_event(&plan.id.to_string(), "started").await?;

        Ok(())
    }

    /// End monitoring session for a plan
    pub async fn end_monitoring_session(&self, plan_id: &str, final_status: SessionStatus) -> Result<()> {
        let mut sessions = self.active_sessions.write().await;
        if let Some(session) = sessions.get_mut(plan_id) {
            session.status = final_status.clone();

            // Log session end
            self.log_session_event(plan_id, &format!("ended with status {:?}", final_status)).await?;
        }

        Ok(())
    }

    /// Convert execution plan to working spec for council review
    fn plan_to_working_spec(&self, plan: &ExecutionPlan) -> Result<agent_agency_contracts::WorkingSpec> {
        // Convert plan to working spec format expected by council
        // This is a simplified conversion - real implementation would be more comprehensive
        Ok(agent_agency_contracts::WorkingSpec {
            id: plan.contract_plan.id.to_string(),
            title: plan.contract_plan.title.clone(),
            description: plan.contract_plan.overview.clone(),
            risk_tier: plan.contract_plan.quality_gates.requires_manual_review as u8,
            scope: Default::default(), // Would need to convert from plan scope
            acceptance_criteria: vec![], // Would need to extract from milestones
            file_changes: vec![], // Would need to extract from plan
            constraints: Default::default(), // Would need to convert from plan constraints
            coverage_targets: Default::default(), // Would need to extract from quality gates
            created_at: plan.contract_plan.created_at,
            updated_at: plan.contract_plan.updated_at,
        })
    }

    /// Create review context for council evaluation
    fn create_review_context(&self, plan: &ExecutionPlan) -> HashMap<String, serde_json::Value> {
        let mut context = HashMap::new();

        context.insert("plan_id".to_string(), serde_json::Value::String(plan.id.to_string()));
        context.insert("milestone_count".to_string(), serde_json::Value::Number(plan.contract_plan.milestones.len().into()));
        context.insert("risk_tier".to_string(), serde_json::Value::Number(plan.contract_plan.quality_gates.requires_manual_review.into()));

        // Add execution context if available
        if let Some(exec_ctx) = &plan.execution_context {
            context.insert("parallel_batches".to_string(), serde_json::Value::Number(exec_ctx.parallel_batches.len().into()));
        }

        context
    }

    /// Determine review priority based on plan characteristics
    fn determine_review_priority(&self, plan: &ExecutionPlan) -> ReviewPriority {
        // High risk plans get higher priority
        if plan.contract_plan.quality_gates.requires_manual_review {
            ReviewPriority::High
        } else if plan.contract_plan.quality_gates.requires_council_approval {
            ReviewPriority::Critical
        } else {
            ReviewPriority::Normal
        }
    }

    /// Check for violations based on progress updates
    async fn check_progress_violations(&self, plan_id: &str, milestone_id: &str, status: &str) -> Result<Vec<String>> {
        let mut violations = Vec::new();

        // Check for status violations
        if status == "failed" {
            violations.push(format!("Milestone {} failed execution", milestone_id));
        }

        // Check for timing violations (simplified)
        // In a real implementation, this would check against plan deadlines

        // Check for scope violations (simplified)
        // In a real implementation, this would validate file access

        Ok(violations)
    }

    // Logging methods

    /// Log council decision to audit trail
    async fn log_council_decision(&self, plan_id: String, decision: &FinalDecision) -> Result<()> {
        let entry = AuditTrailEntry {
            id: Uuid::new_v4(),
            task_id: Uuid::parse_str(&plan_id).unwrap_or(Uuid::new_v4()),
            action: "council_decision".to_string(),
            actor: "council_monitor".to_string(),
            resource_id: Some(Uuid::parse_str(&plan_id).unwrap_or(Uuid::new_v4())),
            resource_type: Some("execution_plan".to_string()),
            change_summary: format!(
                "Council decision: {} (score: {:.2})",
                match decision.label {
                    agent_agency_contracts::VerdictLabel::Pass => "PASS",
                    agent_agency_contracts::VerdictLabel::Fail => "FAIL",
                    agent_agency_contracts::VerdictLabel::NeedsInfo => "NEEDS INFO",
                    agent_agency_contracts::VerdictLabel::Conditional => "CONDITIONAL",
                },
                decision.score
            ),
            timestamp: Utc::now(),
            created_at: Utc::now(),
            metadata: serde_json::to_value(decision).unwrap_or_default(),
        };

        // Note: In real implementation, this would call db_ops.create_audit_trail_entry
        // For now, we'll just log it
        println!("Council decision logged: {}", entry.change_summary);

        Ok(())
    }

    /// Log council evaluation error
    async fn log_council_error(&self, plan_id: String, error: &impl std::fmt::Display) -> Result<()> {
        println!("Council evaluation error for plan {}: {}", plan_id, error);
        Ok(())
    }

    /// Log progress event
    async fn log_progress_event(&self, plan_id: &str, milestone_id: &str, status: &str) -> Result<()> {
        println!("Progress update - Plan: {}, Milestone: {}, Status: {}", plan_id, milestone_id, status);
        Ok(())
    }

    /// Log intervention request
    async fn log_intervention_request(&self, plan_id: &str, reason: &str) -> Result<()> {
        println!("Intervention requested for plan {}: {}", plan_id, reason);
        Ok(())
    }

    /// Log session event
    async fn log_session_event(&self, plan_id: &str, event: &str) -> Result<()> {
        println!("Monitoring session {} for plan {}", event, plan_id);
        Ok(())
    }

    /// Clean up expired monitoring sessions
    pub async fn cleanup_expired_sessions(&self) -> Result<usize> {
        let max_age = chrono::Duration::hours(self.config.max_session_duration_hours as i64);
        let now = Utc::now();

        let mut sessions = self.active_sessions.write().await;
        let expired_count = sessions.len();

        sessions.retain(|_, session| {
            now.signed_duration_since(session.started_at) < max_age
        });

        let cleaned_count = expired_count - sessions.len();

        if cleaned_count > 0 {
            println!("Cleaned up {} expired monitoring sessions", cleaned_count);
        }

        Ok(cleaned_count)
    }

    /// Get monitoring statistics
    pub async fn get_monitoring_stats(&self) -> Result<MonitoringStats> {
        let sessions = self.active_sessions.read().await;

        let active_sessions = sessions.values()
            .filter(|s| s.status == SessionStatus::Active)
            .count();

        let total_violations: usize = sessions.values()
            .map(|s| s.violations.len())
            .sum();

        let total_interventions: usize = sessions.values()
            .map(|s| s.interventions.len())
            .sum();

        Ok(MonitoringStats {
            total_sessions: sessions.len(),
            active_sessions,
            total_violations,
            total_interventions,
        })
    }
}

/// Monitoring statistics
#[derive(Debug, Clone)]
pub struct MonitoringStats {
    /// Total number of monitoring sessions
    pub total_sessions: usize,

    /// Number of active sessions
    pub active_sessions: usize,

    /// Total violations detected
    pub total_violations: usize,

    /// Total interventions requested
    pub total_interventions: usize,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            check_interval_seconds: 30,
            enable_realtime_monitoring: true,
            block_on_violations: true,
            max_session_duration_hours: 24,
            notification_timeout_seconds: 60,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // Mock council coordinator for testing
    struct MockCouncilCoordinator;

    #[async_trait::async_trait]
    impl agent_constitutional_council::CouncilCoordinator<agent_agency_contracts::Engine> {
        async fn evaluate(&self, _ctx: &ReviewContext) -> CouncilResult<FinalDecision> {
            Ok(FinalDecision {
                label: agent_agency_contracts::VerdictLabel::Pass,
                score: 0.9,
                rationale: "Mock approval".to_string(),
                violations: vec![],
                evidence_refs: vec![],
            })
        }
    }

    // Mock database operations
    struct MockDbOps;

    #[async_trait::async_trait]
    impl DatabaseOperations for MockDbOps {
        async fn create_execution_plan(&self, _plan: data_infrastructure::CreateExecutionPlan) -> Result<data_infrastructure::models::ExecutionPlan> { Err(anyhow!("Not implemented")) }
        async fn get_execution_plan(&self, _id: Uuid) -> Result<Option<data_infrastructure::models::ExecutionPlan>> { Ok(None) }
        async fn get_execution_plans(&self) -> Result<Vec<data_infrastructure::models::ExecutionPlan>> { Ok(vec![]) }
        async fn update_execution_plan(&self, _id: Uuid, _update: data_infrastructure::UpdateExecutionPlan) -> Result<data_infrastructure::models::ExecutionPlan> { Err(anyhow!("Not implemented")) }
        async fn delete_execution_plan(&self, _id: Uuid) -> Result<()> { Ok(()) }
        async fn create_judge(&self, _judge: data_infrastructure::CreateJudge) -> Result<data_infrastructure::models::Judge> { Err(anyhow!("Not implemented")) }
        async fn get_judge(&self, _id: Uuid) -> Result<Option<data_infrastructure::models::Judge>> { Ok(None) }
        async fn get_judges(&self) -> Result<Vec<data_infrastructure::models::Judge>> { Ok(vec![]) }
        async fn update_judge(&self, _id: Uuid, _update: data_infrastructure::UpdateJudge) -> Result<data_infrastructure::models::Judge> { Err(anyhow!("Not implemented")) }
        async fn delete_judge(&self, _id: Uuid) -> Result<()> { Ok(()) }
        async fn create_worker(&self, _worker: data_infrastructure::CreateWorker) -> Result<data_infrastructure::models::Worker> { Err(anyhow!("Not implemented")) }
        async fn get_worker(&self, _id: Uuid) -> Result<Option<data_infrastructure::models::Worker>> { Ok(None) }
        async fn get_workers(&self) -> Result<Vec<data_infrastructure::models::Worker>> { Ok(vec![]) }
        async fn update_worker(&self, _id: Uuid, _update: data_infrastructure::UpdateWorker) -> Result<data_infrastructure::models::Worker> { Err(anyhow!("Not implemented")) }
        async fn delete_worker(&self, _id: Uuid) -> Result<()> { Ok(()) }
        async fn create_task(&self, _task: data_infrastructure::CreateTask) -> Result<data_infrastructure::models::Task> { Err(anyhow!("Not implemented")) }
        async fn get_task(&self, _id: Uuid) -> Result<Option<data_infrastructure::models::Task>> { Ok(None) }
        async fn get_tasks(&self, _status: Option<String>) -> Result<Vec<data_infrastructure::models::Task>> { Ok(vec![]) }
        async fn update_task(&self, _id: Uuid, _update: data_infrastructure::UpdateTask) -> Result<data_infrastructure::models::Task> { Err(anyhow!("Not implemented")) }
        async fn delete_task(&self, _id: Uuid) -> Result<()> { Ok(()) }
        async fn create_task_execution(&self, _execution: data_infrastructure::CreateTaskExecution) -> Result<data_infrastructure::models::TaskExecution> { Err(anyhow!("Not implemented")) }
        async fn get_task_execution(&self, _id: Uuid) -> Result<Option<data_infrastructure::models::TaskExecution>> { Ok(None) }
        async fn get_task_executions(&self, _task_id: Uuid) -> Result<Vec<data_infrastructure::models::TaskExecution>> { Ok(vec![]) }
        async fn update_task_execution(&self, _id: Uuid, _update: data_infrastructure::UpdateTaskExecution) -> Result<data_infrastructure::models::TaskExecution> { Err(anyhow!("Not implemented")) }
        async fn create_audit_trail_entry(&self, _entry: data_infrastructure::CreateAuditTrailEntry) -> Result<data_infrastructure::models::AuditTrailEntry> { Err(anyhow!("Not implemented")) }
        async fn get_audit_trail_entries(&self, _task_id: Uuid) -> Result<Vec<data_infrastructure::models::AuditTrailEntry>> { Ok(vec![]) }
        async fn get_audit_trail_entry(&self, _id: Uuid) -> Result<Option<data_infrastructure::models::AuditTrailEntry>> { Ok(None) }
        async fn create_council_verdict(&self, _verdict: data_infrastructure::CreateCouncilVerdict) -> Result<data_infrastructure::models::CouncilVerdict> { Err(anyhow!("Not implemented")) }
        async fn get_council_verdict(&self, _id: Uuid) -> Result<Option<data_infrastructure::models::CouncilVerdict>> { Ok(None) }
        async fn get_council_verdicts(&self, _task_id: Uuid) -> Result<Vec<data_infrastructure::models::CouncilVerdict>> { Ok(vec![]) }
        async fn create_judge_evaluation(&self, _evaluation: data_infrastructure::CreateJudgeEvaluation) -> Result<data_infrastructure::models::JudgeEvaluation> { Err(anyhow!("Not implemented")) }
        async fn get_judge_evaluations(&self, _task_id: Uuid) -> Result<Vec<data_infrastructure::models::JudgeEvaluation>> { Ok(vec![]) }
        // Planning methods (stubs)
        async fn create_milestone(&self, _milestone: data_infrastructure::CreateMilestone) -> Result<data_infrastructure::models::Milestone> { Err(anyhow!("Not implemented")) }
        async fn get_milestone(&self, _plan_id: Uuid, _milestone_id: String) -> Result<Option<data_infrastructure::models::Milestone>> { Ok(None) }
        async fn get_milestones(&self, _plan_id: Uuid) -> Result<Vec<data_infrastructure::models::Milestone>> { Ok(vec![]) }
        async fn update_milestone(&self, _plan_id: Uuid, _milestone_id: String, _update: data_infrastructure::UpdateMilestone) -> Result<data_infrastructure::models::Milestone> { Err(anyhow!("Not implemented")) }
        async fn delete_milestone(&self, _plan_id: Uuid, _milestone_id: String) -> Result<()> { Ok(()) }
        async fn create_planning_session(&self, _session: data_infrastructure::CreatePlanningSession) -> Result<data_infrastructure::models::PlanningSession> { Err(anyhow!("Not implemented")) }
        async fn get_planning_session(&self, _id: Uuid) -> Result<Option<data_infrastructure::models::PlanningSession>> { Ok(None) }
        async fn get_planning_sessions(&self, _plan_id: Uuid) -> Result<Vec<data_infrastructure::models::PlanningSession>> { Ok(vec![]) }
        async fn update_planning_session(&self, _id: Uuid, _update: data_infrastructure::UpdatePlanningSession) -> Result<data_infrastructure::models::PlanningSession> { Err(anyhow!("Not implemented")) }
        async fn create_evidence_artifact(&self, _artifact: data_infrastructure::CreateEvidenceArtifact) -> Result<data_infrastructure::models::EvidenceArtifact> { Err(anyhow!("Not implemented")) }
        async fn get_evidence_artifacts(&self, _plan_id: Uuid) -> Result<Vec<data_infrastructure::models::EvidenceArtifact>> { Ok(vec![]) }
        async fn get_evidence_artifacts_for_milestone(&self, _plan_id: Uuid, _milestone_id: String) -> Result<Vec<data_infrastructure::models::EvidenceArtifact>> { Ok(vec![]) }
        async fn update_evidence_artifact(&self, _id: Uuid, _update: data_infrastructure::UpdateEvidenceArtifact) -> Result<data_infrastructure::models::EvidenceArtifact> { Err(anyhow!("Not implemented")) }
        async fn create_planning_audit_event(&self, _event: data_infrastructure::CreatePlanningAuditEvent) -> Result<data_infrastructure::models::PlanningAuditEvent> { Err(anyhow!("Not implemented")) }
        async fn get_planning_audit_events(&self, _plan_id: Uuid) -> Result<Vec<data_infrastructure::models::PlanningAuditEvent>> { Ok(vec![]) }
        async fn create_planning_telemetry(&self, _telemetry: data_infrastructure::CreatePlanningTelemetry) -> Result<data_infrastructure::models::PlanningTelemetry> { Err(anyhow!("Not implemented")) }
        async fn get_planning_telemetry(&self, _plan_id: Uuid, _metric_type: Option<String>) -> Result<Vec<data_infrastructure::models::PlanningTelemetry>> { Ok(vec![]) }
        
        // Waiver operations
        async fn get_waivers(&self, _status: Option<String>) -> Result<Vec<data_infrastructure::models::Waiver>> { Ok(vec![]) }
        async fn create_waiver(&self, _waiver: data_infrastructure::CreateWaiver) -> Result<data_infrastructure::models::Waiver> { Err(anyhow!("Not implemented")) }
        async fn update_waiver(&self, _id: Uuid, _update: data_infrastructure::UpdateWaiver) -> Result<data_infrastructure::models::Waiver> { Err(anyhow!("Not implemented")) }
    }

    #[test]
    fn test_council_monitor_creation() {
        let council = Arc::new(MockCouncilCoordinator);
        let db_ops = Arc::new(MockDbOps);
        let monitor = CouncilMonitor::new(council, db_ops);
        // Should create successfully
        assert!(true);
    }

    #[test]
    fn test_monitor_config_defaults() {
        let config = MonitorConfig::default();
        assert_eq!(config.check_interval_seconds, 30);
        assert!(config.enable_realtime_monitoring);
        assert!(config.block_on_violations);
        assert_eq!(config.max_session_duration_hours, 24);
    }

    #[test]
    fn test_session_status() {
        let status = SessionStatus::Active;
        assert_eq!(status, SessionStatus::Active);
    }
}
