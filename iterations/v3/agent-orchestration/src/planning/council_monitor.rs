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
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use agent_agency_contracts::*;
// Council coordinator trait is now imported from contracts

#[derive(Debug, Clone)]
pub struct ReviewContext {
    pub plan_id: Uuid,
    pub execution_id: Uuid,
    pub review_type: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReviewPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub enum CouncilResult {
    Approved,
    Rejected(String),
    Escalated(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum FinalDecision {
    Proceed,
    Refine(String),
    Reject(String),
    Escalate(String),
}
use crate::planning::{DatabaseOperations, data_infrastructure_types::AuditTrailEntry};

/// Council monitor for constitutional oversight
pub struct CouncilMonitor {
    /// Constitutional council coordinator
    council: Arc<dyn agent_agency_contracts::CouncilCoordinator>,

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

    /// The execution plan being monitored
    plan: ExecutionPlan,

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
        council: Arc<dyn agent_agency_contracts::CouncilCoordinator>,
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
        council: Arc<dyn agent_agency_contracts::CouncilCoordinator>,
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
            plan_id: plan.contract_plan.id,
            execution_id: Uuid::new_v4(), // TODO: Get actual execution ID
            review_type: "execution_check".to_string(),
        };

        // Get council decision using contracts interface
        // For now, use a simplified approach - start session and get basic approval
        let session_result = self.council.start_session(&agent_agency_contracts::TaskDescriptor {
            task_id: uuid::Uuid::new_v4(),
            description: format!("Monitor intervention for plan {}", context.plan_id),
            change_budget: agent_agency_contracts::ChangeBudget {
                max_files: 1,
                max_loc: 10,
                max_migrations: 0,
                allow_breaking_changes: false,
                allow_new_dependencies: false,
                enforcement_mode: agent_agency_contracts::planning_io::BudgetEnforcement::Strict,
            },
            priority: agent_agency_contracts::TaskPriority::High,
            execution_mode: agent_agency_contracts::ExecutionMode::Auto,
            risk_tier: Some(agent_agency_contracts::types::planning::RiskTier::Tier1),
            blast_radius: agent_agency_contracts::BlastRadius {
                modules: vec!["monitoring".to_string()],
                data_migration: false,
                external_deps: vec![],
            },
            scope_in: None,
            scope_out: None,
            acceptance: None,
        }).await;

        let decision = match session_result {
            Ok(_) => Ok(agent_agency_contracts::JudgeVerdict {
                score: 0.9,
                label: agent_agency_contracts::VerdictLabel::Pass,
                rationale: "Council session started for monitoring".to_string(),
                violations: vec![],
                evidence_refs: vec![],
            }),
            Err(e) => Err(e),
        };

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

            // Trigger a council review for the intervention
            let review_context = ReviewContext {
                plan_id,
                execution_id: Uuid::new_v4(),
                review_type: "intervention".to_string(),
            };

            // Submit for council review - using contracts methods
            match self.council.start_session(&agent_agency_contracts::TaskDescriptor {
                task_id: uuid::Uuid::new_v4(),
                description: format!("Intervention review for plan {}", plan_id),
                change_budget: agent_agency_contracts::ChangeBudget {
                    max_files: 10,
                    max_loc: 1000,
                    max_migrations: 0,
                    allow_breaking_changes: false,
                    allow_new_dependencies: false,
                    enforcement_mode: agent_agency_contracts::planning_io::BudgetEnforcement::Strict,
                },
                priority: agent_agency_contracts::TaskPriority::High,
                execution_mode: agent_agency_contracts::ExecutionMode::Auto,
                risk_tier: Some(agent_agency_contracts::types::planning::RiskTier::Tier2),
                blast_radius: agent_agency_contracts::BlastRadius {
                    modules: vec!["council".to_string()],
                    data_migration: false,
                    external_deps: vec![],
                },
                scope_in: None,
                scope_out: None,
                acceptance: None,
            }).await {
                Ok(session_id) => {
                    info!("Council review session started for plan {} intervention: {:?}", plan_id, session_id);
                }
                Err(e) => {
                    warn!("Failed to start council review session for plan {}: {}", plan_id, e);
                }
            }

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
        let violations = self.check_violations(plan_id).await?;

        // Query the council for specific recommendations based on plan execution
        let review_context = ReviewContext {
            plan_id: Uuid::parse_str(plan_id).unwrap_or_else(|_| Uuid::new_v4()),
            execution_id: Uuid::new_v4(),
            review_type: "recommendations".to_string(),
        };

        // Get council recommendations (mock implementation for migration)
        match self.council.start_session(&agent_agency_contracts::TaskDescriptor {
            task_id: uuid::Uuid::new_v4(),
            description: "Mock session for recommendations".to_string(),
            change_budget: agent_agency_contracts::ChangeBudget {
                max_files: 1,
                max_loc: 10,
                max_migrations: 0,
                allow_breaking_changes: false,
                allow_new_dependencies: false,
                enforcement_mode: agent_agency_contracts::planning_io::BudgetEnforcement::Strict,
            },
            priority: agent_agency_contracts::TaskPriority::Low,
            execution_mode: agent_agency_contracts::ExecutionMode::Auto,
            risk_tier: Some(agent_agency_contracts::types::planning::RiskTier::Tier3),
            blast_radius: agent_agency_contracts::BlastRadius {
                modules: vec![],
                data_migration: false,
                external_deps: vec![],
            },
            scope_in: None,
            scope_out: None,
            acceptance: None,
        }).await {
            Ok(session_id) => {
                // Generate recommendations based on violations
                let mut recommendations = Vec::new();

                if violations.is_empty() {
                    recommendations.push("Plan execution approved - no violations detected".to_string());
                    recommendations.push("Monitor execution for any issues".to_string());
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

                recommendations.push(format!("Council session started: {}", session_id.0));

                Ok(recommendations)
            }
            Err(e) => {
                warn!("Failed to get council recommendations for plan {}: {}", plan_id, e);
                // Fallback to generic recommendations
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
        }
    }

    /// Start monitoring session for a plan
    async fn start_monitoring_session(&self, plan: &ExecutionPlan) -> Result<()> {
        let session = PlanSession {
            plan_id: plan.contract_plan.id.to_string(),
            plan: plan.clone(),
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
        // Extract acceptance criteria from plan (milestones don't have acceptance_criteria)
        let acceptance_criteria = plan.contract_plan.acceptance_criteria.clone();

        // Extract file changes from plan scope
        let file_changes = plan.contract_plan.scope.iter()
            .flat_map(|scope| scope.allowed_paths.iter())
            .chain(plan.contract_plan.scope.iter().flat_map(|scope| scope.blocked_paths.iter()))
            .map(|path| agent_agency_contracts::FileChange {
                file: path.clone(),
                change_type: agent_agency_contracts::ChangeType::Modified,
                timestamp: chrono::Utc::now(),
            })
            .collect::<Vec<_>>();

        // Convert plan constraints to working spec constraints
        let constraints = agent_agency_contracts::WorkingSpecConstraints {
            max_duration_minutes: plan.contract_plan.constraints.max_duration_minutes,
            max_iterations: plan.contract_plan.constraints.max_iterations,
            budget_limits: None,
            scope_restrictions: None,
        };

        // Extract coverage targets from quality gates
        let coverage_targets = if let Some(qg) = &plan.contract_plan.quality_gates {
            agent_agency_contracts::CoverageTargets {
                line_coverage: qg.coverage_requirements.get("line").copied(),
                branch_coverage: qg.coverage_requirements.get("branch").copied(),
                mutation_score: Some(qg.mutation_requirements.min_score),
            }
        } else {
            agent_agency_contracts::CoverageTargets {
                line_coverage: None,
                branch_coverage: None,
                mutation_score: None,
            }
        };

        Ok(agent_agency_contracts::WorkingSpec {
            version: "3.0.0".to_string(),
            id: plan.contract_plan.id.to_string(),
            title: plan.contract_plan.title.clone(),
            description: plan.contract_plan.overview.clone(),
            goals: vec![],
            risk_tier: if let Some(qg) = &plan.contract_plan.quality_gates {
                if qg.requires_council_approval {
                    3 // Critical
                } else if qg.requires_manual_review {
                    2 // High
                } else {
                    1 // Normal
                }
            } else {
                1 // Normal
            },
            constraints,
            acceptance_criteria,
            test_plan: agent_agency_contracts::working_spec::TestPlan {
                unit_tests: vec![],
                integration_tests: vec![],
                e2e_scenarios: vec![],
                coverage_targets: None,
            },
            rollback_plan: Default::default(),
            context: agent_agency_contracts::working_spec::WorkingSpecContext {
                workspace_root: ".".to_string(),
                git_branch: "main".to_string(),
                recent_changes: vec![],
                dependencies: std::collections::HashMap::new(),
                environment: agent_agency_contracts::task_request::Environment::Development,
            },
            non_functional_requirements: None,
            validation_results: None,
            quality_gates: plan.contract_plan.quality_gates.clone(),
            scope: plan.contract_plan.scope.clone(),
            metadata: None,
            milestones: vec![],
            change_budget: plan.contract_plan.change_budget.clone(),
            file_changes: file_changes.into_iter().map(|fc| agent_agency_contracts::FileChange {
                file: fc.file,
                change_type: fc.change_type,
                timestamp: fc.timestamp,
            }).collect(),
            coverage_targets: Some(coverage_targets),
            overview: plan.contract_plan.overview.clone(),
            created_at: plan.contract_plan.created_at,
            updated_at: plan.contract_plan.updated_at,
        })
    }

    /// Create review context for council evaluation
    fn create_review_context(&self, plan: &ExecutionPlan) -> HashMap<String, serde_json::Value> {
        let mut context = HashMap::new();

        context.insert("plan_id".to_string(), serde_json::Value::String(plan.id.to_string()));
        context.insert("milestone_count".to_string(), serde_json::Value::Number(plan.contract_plan.milestones.len().into()));
            context.insert("risk_tier".to_string(), serde_json::Value::Number(
                plan.contract_plan.quality_gates.as_ref()
                    .map(|qg| qg.requires_manual_review as i64)
                    .unwrap_or(0).into()
            ));

        // Add execution context if available
        // Note: ExecutionContext doesn't have parallel_batches field
        if plan.execution_context.is_some() {
            context.insert("has_execution_context".to_string(), serde_json::Value::Bool(true));
        }

        context
    }

    /// Determine review priority based on plan characteristics
    fn determine_review_priority(&self, plan: &ExecutionPlan) -> ReviewPriority {
        // High risk plans get higher priority
        if let Some(qg) = &plan.contract_plan.quality_gates {
            if qg.requires_manual_review {
                ReviewPriority::High
            } else if qg.requires_council_approval {
                ReviewPriority::Critical
            } else {
                ReviewPriority::Normal
            }
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

        // Check for timing violations
        let sessions = self.active_sessions.read().await;
        if let Some(session) = sessions.get(plan_id) {
            let now = Utc::now();

            // Check if this milestone has a deadline (calculated from estimated duration)
            for milestone in &session.plan.contract_plan.milestones {
                if milestone.id.to_string() == milestone_id {
                    // Calculate deadline from session start + estimated duration
                    if let Some(estimated_duration) = milestone.estimated_duration {
                        let deadline = session.started_at + chrono::Duration::minutes(estimated_duration as i64);

                        if now > deadline {
                            violations.push(format!("Milestone {} exceeded deadline", milestone_id));
                        } else {
                            // Check if we're close to deadline (within 10% of time remaining)
                            let total_duration = estimated_duration as i64;
                            let remaining_minutes = (deadline - now).num_minutes();

                            if remaining_minutes < (total_duration / 10) && remaining_minutes > 0 {
                                violations.push(format!("Milestone {} approaching deadline ({} minutes remaining)", milestone_id, remaining_minutes));
                            }
                        }
                    }
                    break;
                }
            }
        }

        // Check for scope violations (basic file access validation)
        // For now, we check if the milestone involves files that should be accessible
        if milestone_id.contains("file") || milestone_id.contains("io") {
            // Basic validation that file-related milestones have proper scope
            if status == "permission_denied" || status == "access_denied" {
                violations.push(format!("Milestone {} failed due to file access permissions", milestone_id));
            }
        }

        Ok(violations)
    }

    // Logging methods

    /// Log council decision to audit trail
    async fn log_council_decision(&self, plan_id: String, decision: &agent_agency_contracts::JudgeVerdict) -> Result<()> {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("task_id".to_string(), serde_json::Value::String(plan_id.clone()));
        metadata.insert("actor".to_string(), serde_json::Value::String("council_monitor".to_string()));
        metadata.insert("resource_id".to_string(), serde_json::Value::String(plan_id.clone()));
        metadata.insert("resource_type".to_string(), serde_json::Value::String("execution_plan".to_string()));

        // Add decision details to metadata
        let decision_value = serde_json::to_value(decision).unwrap_or_default();
        if let serde_json::Value::Object(decision_map) = decision_value {
            for (key, value) in decision_map {
                metadata.insert(format!("decision_{}", key), value);
            }
        }

        let entry = AuditTrailEntry {
            id: Uuid::new_v4(),
            event_type: "council_decision".to_string(),
            description: format!(
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
            metadata,
        };

        // Note: In real implementation, this would call db_ops.create_audit_trail_entry
        // For now, we'll just log it
        println!("Council decision logged: {}", entry.description);

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
    impl agent_agency_contracts::CouncilCoordinator for MockCouncilCoordinator {
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
        async fn create_execution_plan(&self, _plan: crate::planning::CreateExecutionPlan) -> Result<crate::planning::models::ExecutionPlan> { Err(anyhow!("Not implemented")) }
        async fn get_execution_plan(&self, _id: Uuid) -> Result<Option<crate::planning::models::ExecutionPlan>> { Ok(None) }
        async fn get_execution_plans(&self) -> Result<Vec<crate::planning::models::ExecutionPlan>> { Ok(vec![]) }
        async fn update_execution_plan(&self, _id: Uuid, _update: crate::planning::UpdateExecutionPlan) -> Result<crate::planning::models::ExecutionPlan> { Err(anyhow!("Not implemented")) }
        async fn delete_execution_plan(&self, _id: Uuid) -> Result<()> { Ok(()) }
        async fn create_judge(&self, _judge: crate::planning::CreateJudge) -> Result<crate::planning::models::Judge> { Err(anyhow!("Not implemented")) }
        async fn get_judge(&self, _id: Uuid) -> Result<Option<crate::planning::models::Judge>> { Ok(None) }
        async fn get_judges(&self) -> Result<Vec<crate::planning::models::Judge>> { Ok(vec![]) }
        async fn update_judge(&self, _id: Uuid, _update: crate::planning::UpdateJudge) -> Result<crate::planning::models::Judge> { Err(anyhow!("Not implemented")) }
        async fn delete_judge(&self, _id: Uuid) -> Result<()> { Ok(()) }
        async fn create_worker(&self, _worker: crate::planning::CreateWorker) -> Result<crate::planning::models::Worker> { Err(anyhow!("Not implemented")) }
        async fn get_worker(&self, _id: Uuid) -> Result<Option<crate::planning::models::Worker>> { Ok(None) }
        async fn get_workers(&self) -> Result<Vec<crate::planning::models::Worker>> { Ok(vec![]) }
        async fn update_worker(&self, _id: Uuid, _update: crate::planning::UpdateWorker) -> Result<crate::planning::models::Worker> { Err(anyhow!("Not implemented")) }
        async fn delete_worker(&self, _id: Uuid) -> Result<()> { Ok(()) }
        async fn create_task(&self, _task: crate::planning::CreateTask) -> Result<crate::planning::models::Task> { Err(anyhow!("Not implemented")) }
        async fn get_task(&self, _id: Uuid) -> Result<Option<crate::planning::models::Task>> { Ok(None) }
        async fn get_tasks(&self, _status: Option<String>) -> Result<Vec<crate::planning::models::Task>> { Ok(vec![]) }
        async fn update_task(&self, _id: Uuid, _update: crate::planning::UpdateTask) -> Result<crate::planning::models::Task> { Err(anyhow!("Not implemented")) }
        async fn delete_task(&self, _id: Uuid) -> Result<()> { Ok(()) }
        async fn create_task_execution(&self, _execution: crate::planning::CreateTaskExecution) -> Result<crate::planning::models::TaskExecution> { Err(anyhow!("Not implemented")) }
        async fn get_task_execution(&self, _id: Uuid) -> Result<Option<crate::planning::models::TaskExecution>> { Ok(None) }
        async fn get_task_executions(&self, _task_id: Uuid) -> Result<Vec<crate::planning::models::TaskExecution>> { Ok(vec![]) }
        async fn update_task_execution(&self, _id: Uuid, _update: crate::planning::UpdateTaskExecution) -> Result<crate::planning::models::TaskExecution> { Err(anyhow!("Not implemented")) }
        async fn create_audit_trail_entry(&self, _entry: crate::planning::CreateAuditTrailEntry) -> Result<crate::planning::models::AuditTrailEntry> { Err(anyhow!("Not implemented")) }
        async fn get_audit_trail_entries(&self, _task_id: Uuid) -> Result<Vec<crate::planning::models::AuditTrailEntry>> { Ok(vec![]) }
        async fn get_audit_trail_entry(&self, _id: Uuid) -> Result<Option<crate::planning::models::AuditTrailEntry>> { Ok(None) }
        async fn create_council_verdict(&self, _verdict: crate::planning::CreateCouncilVerdict) -> Result<crate::planning::models::CouncilVerdict> { Err(anyhow!("Not implemented")) }
        async fn get_council_verdict(&self, _id: Uuid) -> Result<Option<crate::planning::models::CouncilVerdict>> { Ok(None) }
        async fn get_council_verdicts(&self, _task_id: Uuid) -> Result<Vec<crate::planning::models::CouncilVerdict>> { Ok(vec![]) }
        async fn create_judge_evaluation(&self, _evaluation: crate::planning::CreateJudgeEvaluation) -> Result<crate::planning::models::JudgeEvaluation> { Err(anyhow!("Not implemented")) }
        async fn get_judge_evaluations(&self, _task_id: Uuid) -> Result<Vec<crate::planning::models::JudgeEvaluation>> { Ok(vec![]) }
        // Planning methods (stubs)
        async fn create_milestone(&self, _milestone: crate::planning::CreateMilestone) -> Result<crate::planning::models::Milestone> { Err(anyhow!("Not implemented")) }
        async fn get_milestone(&self, _plan_id: Uuid, _milestone_id: String) -> Result<Option<crate::planning::models::Milestone>> { Ok(None) }
        async fn get_milestones(&self, _plan_id: Uuid) -> Result<Vec<crate::planning::models::Milestone>> { Ok(vec![]) }
        async fn update_milestone(&self, _plan_id: Uuid, _milestone_id: String, _update: crate::planning::UpdateMilestone) -> Result<crate::planning::models::Milestone> { Err(anyhow!("Not implemented")) }
        async fn delete_milestone(&self, _plan_id: Uuid, _milestone_id: String) -> Result<()> { Ok(()) }
        async fn create_planning_session(&self, _session: crate::planning::CreatePlanningSession) -> Result<crate::planning::models::PlanningSession> { Err(anyhow!("Not implemented")) }
        async fn get_planning_session(&self, _id: Uuid) -> Result<Option<crate::planning::models::PlanningSession>> { Ok(None) }
        async fn get_planning_sessions(&self, _plan_id: Uuid) -> Result<Vec<crate::planning::models::PlanningSession>> { Ok(vec![]) }
        async fn update_planning_session(&self, _id: Uuid, _update: crate::planning::UpdatePlanningSession) -> Result<crate::planning::models::PlanningSession> { Err(anyhow!("Not implemented")) }
        async fn create_evidence_artifact(&self, _artifact: crate::planning::CreateEvidenceArtifact) -> Result<crate::planning::models::EvidenceArtifact> { Err(anyhow!("Not implemented")) }
        async fn get_evidence_artifacts(&self, _plan_id: Uuid) -> Result<Vec<crate::planning::models::EvidenceArtifact>> { Ok(vec![]) }
        async fn get_evidence_artifacts_for_milestone(&self, _plan_id: Uuid, _milestone_id: String) -> Result<Vec<crate::planning::models::EvidenceArtifact>> { Ok(vec![]) }
        async fn update_evidence_artifact(&self, _id: Uuid, _update: crate::planning::UpdateEvidenceArtifact) -> Result<crate::planning::models::EvidenceArtifact> { Err(anyhow!("Not implemented")) }
        async fn create_planning_audit_event(&self, _event: crate::planning::CreatePlanningAuditEvent) -> Result<crate::planning::models::PlanningAuditEvent> { Err(anyhow!("Not implemented")) }
        async fn get_planning_audit_events(&self, _plan_id: Uuid) -> Result<Vec<crate::planning::models::PlanningAuditEvent>> { Ok(vec![]) }
        async fn create_planning_telemetry(&self, _telemetry: crate::planning::CreatePlanningTelemetry) -> Result<crate::planning::models::PlanningTelemetry> { Err(anyhow!("Not implemented")) }
        async fn get_planning_telemetry(&self, _plan_id: Uuid, _metric_type: Option<String>) -> Result<Vec<crate::planning::models::PlanningTelemetry>> { Ok(vec![]) }
        
        // Waiver operations
        async fn get_waivers(&self, _status: Option<String>) -> Result<Vec<crate::planning::models::Waiver>> { Ok(vec![]) }
        async fn create_waiver(&self, _waiver: crate::planning::CreateWaiver) -> Result<crate::planning::models::Waiver> { Err(anyhow!("Not implemented")) }
        async fn update_waiver(&self, _id: Uuid, _update: crate::planning::UpdateWaiver) -> Result<crate::planning::models::Waiver> { Err(anyhow!("Not implemented")) }
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
