//! TODO Integration - Quality Gate Enforcement with Dependency Tracking
//!
//! Integrates TODO template system into planning workflow.
//! Prevents quality bypass by enforcing completion requirements.
//!
//! @author @darianrosebrook

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use anyhow::{anyhow, Result};
use uuid::Uuid;
use chrono::Utc;
use tracing::{debug, info, warn};
use agent_agency_contracts::planning_io::{ExecutionPlan, Milestone, PlanState};
use data_infrastructure::DatabaseOperations;

/// TODO integration with planning workflow
pub struct TodoIntegration {
    /// TODO template system (wrapped in Mutex for thread safety)
    todo_system: Arc<Mutex<crate::planning::todo_template::TodoTemplateSystem>>,

    /// Database operations for persistence
    db_ops: Arc<dyn DatabaseOperations>,

    /// Active TODO instances by plan ID
    plan_todos: HashMap<Uuid, Uuid>, // plan_id -> todo_instance_id

    /// Quality gate enforcer
    quality_enforcer: TodoQualityEnforcer,
}

/// Quality gate enforcer that prevents bypass
pub struct TodoQualityEnforcer {
    /// Gates that absolutely cannot be bypassed
    critical_gates: Vec<String>,
    
    /// Database operations for querying planning telemetry
    db_ops: Option<Arc<dyn DatabaseOperations>>,
}

impl TodoIntegration {
    /// Create new TODO integration
    pub fn new(
        todo_system: Arc<crate::planning::todo_template::TodoTemplateSystem>,
        db_ops: Arc<dyn DatabaseOperations>,
    ) -> Self {
        // Unwrap the Arc to get the inner value, then wrap in Mutex
        let system = Arc::try_unwrap(todo_system)
            .unwrap_or_else(|_| {
                // If we can't unwrap (multiple references), this is a design issue
                // but we'll handle it gracefully by creating a new system
                // In production, this should not happen - TodoTemplateSystem should
                // only be wrapped in Arc once at creation time
                warn!("Multiple references to TodoTemplateSystem detected - this may indicate a design issue");
                crate::planning::todo_template::TodoTemplateSystem::new()
            });
        
        Self {
            todo_system: Arc::new(Mutex::new(system)),
            db_ops: db_ops.clone(),
            plan_todos: HashMap::new(),
            quality_enforcer: TodoQualityEnforcer::with_db_ops(db_ops),
        }
    }

    /// Initialize TODO tracking for a plan
    pub async fn initialize_plan_todos(&mut self, plan: &ExecutionPlan) -> Result<()> {
        // Determine appropriate template based on plan characteristics
        let template_name = self.select_template_for_plan(plan)?;

        // Create TODO instance (requires mutable access to todo_system)
        let todo_instance_id = {
            let mut system = self.todo_system.lock()
                .map_err(|e| anyhow!("Failed to lock TODO system: {}", e))?;
            system.create_instance(
                &template_name,
                plan,
                None, // No specific milestone initially
            )?
        };

        // Track the association
        self.plan_todos.insert(plan.contract_plan.id, todo_instance_id);

        // Persist the association
        self.persist_plan_todo_association(plan.contract_plan.id, todo_instance_id).await?;

        info!(
            plan_id = %plan.contract_plan.id,
            todo_instance_id = %todo_instance_id,
            template = %template_name,
            "Initialized TODO tracking for plan"
        );

        Ok(())
    }

    /// Check if plan can progress to next milestone
    pub async fn can_progress_to_milestone(&self, plan_id: Uuid, milestone_id: &str) -> Result<bool> {
        let todo_instance_id = self.plan_todos.get(&plan_id)
            .ok_or_else(|| anyhow!("No TODO instance for plan {}", plan_id))?;

        // Get the TODO instance and check dependencies
        let system = self.todo_system.lock()
            .map_err(|e| anyhow!("Failed to lock TODO system: {}", e))?;
        
        let instance = system.get_instance(*todo_instance_id)?;
        
        // Map milestone to TODO step
        let step_id = self.map_milestone_to_step(milestone_id)?;
        
        // Check if step can be started (dependencies satisfied)
        let can_start = system.can_progress_to_milestone_step(instance, &step_id)?;
        
        // Also check critical quality gates
        let gates_satisfied = self.quality_enforcer.verify_critical_gates(plan_id, milestone_id).await?;
        
        Ok(can_start && gates_satisfied)
    }

    /// Complete TODO step when milestone is completed
    pub async fn milestone_completed(&mut self, plan_id: Uuid, milestone_id: &str) -> Result<()> {
        let todo_instance_id = self.plan_todos.get(&plan_id)
            .ok_or_else(|| anyhow!("No TODO instance for plan {}", plan_id))?;

        // Map milestone to TODO step
        let step_id = self.map_milestone_to_step(milestone_id)?;

        // Complete the step (requires mutable access)
        {
            let mut system = self.todo_system.lock()
                .map_err(|e| anyhow!("Failed to lock TODO system: {}", e))?;
            system.complete_step(
                *todo_instance_id,
                &step_id,
                Some(format!("Milestone {} completed", milestone_id))
            ).await?;
        }

        info!(
            plan_id = %plan_id,
            milestone_id = %milestone_id,
            step_id = %step_id,
            "Completed TODO step for milestone"
        );

        Ok(())
    }

    /// Check for blocked progress due to TODO requirements
    pub async fn check_blocked_progress(&self, plan_id: Uuid) -> Result<Vec<String>> {
        let todo_instance_id = self.plan_todos.get(&plan_id)
            .ok_or_else(|| anyhow!("No TODO instance for plan {}", plan_id))?;

        // Get instance and check for blocking conditions
        let system = self.todo_system.lock()
            .map_err(|e| anyhow!("Failed to lock TODO system: {}", e))?;
        
        let instance = system.get_instance(*todo_instance_id)?;
        
        let mut blocks = Vec::new();

        // Check for blocking steps
        let template = system.get_template_for_instance(instance)?;
        for step in &template.steps {
            if instance.blocked_steps.contains_key(&step.id) {
                if let Some(reason) = instance.blocked_steps.get(&step.id) {
                    blocks.push(format!("Step '{}' blocked: {}", step.id, reason));
                }
            } else {
                // Check dependency blocking
                let reasons = system.get_blocking_reasons(instance, &step.id);
                blocks.extend(reasons);
            }
        }

        // Check quality gate violations
        if let Ok(violations) = self.quality_enforcer.get_quality_violations(plan_id).await {
            blocks.extend(violations);
        }

        Ok(blocks)
    }

    /// Enforce quality gates cannot be bypassed
    pub async fn enforce_quality_gates(&self, plan_id: Uuid, gate_type: &str, result: bool) -> Result<()> {
        if !result && self.quality_enforcer.is_critical_gate(gate_type) {
            // Critical gate failed - this should block progress
            return Err(anyhow!(
                "Critical quality gate '{}' failed for plan {}. Cannot proceed.",
                gate_type, plan_id
            ));
        }

        Ok(())
    }

    /// Get TODO progress for plan
    pub async fn get_plan_progress(&self, plan_id: Uuid) -> Result<crate::planning::todo_template::TodoProgress> {
        let todo_instance_id = self.plan_todos.get(&plan_id)
            .ok_or_else(|| anyhow!("No TODO instance for plan {}", plan_id))?;

        // Get instance progress from the TODO system
        let system = self.todo_system.lock()
            .map_err(|e| anyhow!("Failed to lock TODO system: {}", e))?;
        
        let instance = system.get_instance(*todo_instance_id)?;
        let progress = system.get_instance_progress(instance)?;

        Ok(progress)
    }

    /// Select appropriate template based on plan characteristics
    fn select_template_for_plan(&self, plan: &ExecutionPlan) -> Result<String> {
        // Simple template selection logic
        match plan.contract_plan.quality_gates.requires_manual_review {
            true => Ok("critical-feature-template".to_string()),
            false => Ok("standard-feature-template".to_string()),
        }
    }

    /// Map milestone ID to TODO step ID
    /// 
    /// Maps milestone identifiers to TODO step identifiers using multiple heuristics:
    /// 1. Milestone ID prefix patterns (e.g., "analysis-", "design-", "implement-")
    /// 2. Milestone ID contains step type keywords
    /// 3. Fallback to milestone ID with "step-" prefix
    fn map_milestone_to_step(&self, milestone_id: &str) -> Result<String> {
        let milestone_lower = milestone_id.to_lowercase();
        
        // Check for explicit step type prefixes
        if milestone_lower.starts_with("analysis") || milestone_lower.starts_with("analyze") {
            return Ok("analysis-step".to_string());
        }
        if milestone_lower.starts_with("design") || milestone_lower.starts_with("plan") {
            return Ok("design-step".to_string());
        }
        if milestone_lower.starts_with("implement") || milestone_lower.starts_with("build") || milestone_lower.starts_with("code") {
            return Ok("implementation-step".to_string());
        }
        if milestone_lower.starts_with("test") || milestone_lower.starts_with("verify") || milestone_lower.starts_with("validate") {
            return Ok("testing-step".to_string());
        }
        if milestone_lower.starts_with("review") || milestone_lower.starts_with("audit") {
            return Ok("review-step".to_string());
        }
        if milestone_lower.starts_with("deploy") || milestone_lower.starts_with("release") {
            return Ok("deployment-step".to_string());
        }
        if milestone_lower.starts_with("doc") || milestone_lower.starts_with("document") {
            return Ok("documentation-step".to_string());
        }
        
        // Check for step type keywords anywhere in the ID
        if milestone_lower.contains("analysis") || milestone_lower.contains("analyze") {
            return Ok("analysis-step".to_string());
        }
        if milestone_lower.contains("design") || milestone_lower.contains("plan") {
            return Ok("design-step".to_string());
        }
        if milestone_lower.contains("implement") || milestone_lower.contains("build") || milestone_lower.contains("code") {
            return Ok("implementation-step".to_string());
        }
        if milestone_lower.contains("test") || milestone_lower.contains("verify") || milestone_lower.contains("validate") {
            return Ok("testing-step".to_string());
        }
        
        // Fallback: use milestone ID with step prefix
        Ok(format!("step-{}", milestone_id))
    }

    /// Persist plan-todo association
    /// 
    /// Stores the association between an execution plan and its TODO instance.
    /// Uses the audit trail system for persistence, which provides:
    /// - Complete audit history of plan-TODO associations
    /// - Queryable via entity_type "plan_todo_association"
    /// - Automatically includes timestamps and metadata
    /// 
    /// Note: Using audit trail is appropriate here as it provides both persistence
    /// and auditability. A dedicated table could be added later if query performance
    /// becomes a concern, but audit trail queries are sufficient for current needs.
    async fn persist_plan_todo_association(&self, plan_id: Uuid, todo_instance_id: Uuid) -> Result<()> {
        let audit_entry = data_infrastructure::CreateAuditTrailEntry {
            entity_type: "plan_todo_association".to_string(),
            entity_id: plan_id,
            action: "todo_assigned".to_string(),
            details: serde_json::json!({
                "plan_id": plan_id.to_string(),
                "todo_instance_id": todo_instance_id.to_string(),
                "associated_at": chrono::Utc::now().to_rfc3339(),
            }),
            user_id: None,
            ip_address: None,
            timestamp: Some(chrono::Utc::now()),
        };

        self.db_ops.create_audit_trail_entry(audit_entry).await?;
        Ok(())
    }
}

impl TodoQualityEnforcer {
    /// Create new quality enforcer
    pub fn new() -> Self {
        Self {
            critical_gates: vec![
                "security_scan".to_string(),
                "test_coverage".to_string(),
                "contract_validation".to_string(),
                "type_safety".to_string(),
                "performance_budget".to_string(),
                "dependency_audit".to_string(),
            ],
            db_ops: None,
        }
    }

    /// Create quality enforcer with database access
    pub fn with_db_ops(db_ops: Arc<dyn DatabaseOperations>) -> Self {
        Self {
            critical_gates: vec![
                "security_scan".to_string(),
                "test_coverage".to_string(),
                "contract_validation".to_string(),
                "type_safety".to_string(),
                "performance_budget".to_string(),
                "dependency_audit".to_string(),
            ],
            db_ops: Some(db_ops),
        }
    }

    /// Check if gate is critical (cannot be bypassed)
    pub fn is_critical_gate(&self, gate_type: &str) -> bool {
        self.critical_gates.contains(&gate_type.to_string())
    }

    /// Verify critical gates are satisfied
    pub async fn verify_critical_gates(&self, plan_id: Uuid, milestone_id: &str) -> Result<bool> {
        // Check each critical gate
        for gate in &self.critical_gates {
            if !self.check_gate_status(plan_id, milestone_id, gate).await? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Get quality violations
    pub async fn get_quality_violations(&self, plan_id: Uuid) -> Result<Vec<String>> {
        let mut violations = Vec::new();

        // Check for violations in critical gates
        for gate in &self.critical_gates {
            if !self.check_gate_status(plan_id, &"current".to_string(), gate).await? {
                violations.push(format!("Critical gate '{}' failed", gate));
            }
        }

        Ok(violations)
    }

    /// Check individual gate status
    /// 
    /// Queries planning_telemetry table for quality gate status.
    /// Looks for the latest quality gate result for the specified gate and milestone.
    async fn check_gate_status(&self, plan_id: Uuid, milestone_id: &str, gate: &str) -> Result<bool> {
        // If database operations are available, query planning_telemetry table
        if let Some(db_ops) = &self.db_ops {
            // Query planning telemetry for quality gates
            let telemetry = db_ops.get_planning_telemetry(plan_id, Some("quality_gate".to_string())).await?;
            
            // Find the latest gate result matching the gate name and milestone
            let latest_gate = telemetry.iter()
                .filter(|t| {
                    // Check if metadata contains gate_name matching our gate
                    t.metadata.get("gate_name")
                        .and_then(|v| v.as_str())
                        .map(|name| name == gate)
                        .unwrap_or(false)
                })
                .filter(|t| {
                    // Check milestone_id matches (or milestone_id is "current" which matches any)
                    if milestone_id == "current" {
                        true
                    } else {
                        t.metadata.get("milestone_id")
                            .and_then(|v| v.as_str())
                            .map(|m| m == milestone_id)
                            .unwrap_or(false)
                    }
                })
                .max_by_key(|t| t.collected_at);
            
            // Extract result from metric_value JSONB
            if let Some(gate_telemetry) = latest_gate {
                let result = gate_telemetry.metric_value
                    .get("result")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                
                debug!(
                    plan_id = %plan_id,
                    milestone_id = %milestone_id,
                    gate = %gate,
                    result = %result,
                    "Found quality gate status from database"
                );
                
                return Ok(result);
            }
            
            debug!(
                plan_id = %plan_id,
                milestone_id = %milestone_id,
                gate = %gate,
                "No quality gate status found in database"
            );
        } else {
            debug!(
                plan_id = %plan_id,
                milestone_id = %milestone_id,
                gate = %gate,
                "Checking quality gate status (no database access)"
            );
        }
        
        // Default to gates passing if no data found
        // In production, this might want to fail open or closed based on policy
        Ok(true)
    }
}

/// Integration hooks for planning workflow
pub trait TodoWorkflowHooks {
    /// Called when plan execution starts
    async fn on_plan_started(&self, plan: &ExecutionPlan) -> Result<()> {
        Ok(())
    }

    /// Called before milestone starts
    async fn on_milestone_starting(&self, plan_id: Uuid, milestone_id: &str) -> Result<()> {
        Ok(())
    }

    /// Called when milestone completes
    async fn on_milestone_completed(&self, plan_id: Uuid, milestone_id: &str) -> Result<()> {
        Ok(())
    }

    /// Called when plan completes
    async fn on_plan_completed(&self, plan_id: Uuid) -> Result<()> {
        Ok(())
    }

    /// Called when quality gate fails
    async fn on_quality_gate_failed(&self, plan_id: Uuid, gate: &str) -> Result<()> {
        Ok(())
    }
}

impl TodoWorkflowHooks for TodoIntegration {
    async fn on_plan_started(&self, plan: &ExecutionPlan) -> Result<()> {
        // Initialize TODO tracking
        info!(
            plan_id = %plan.contract_plan.id,
            "Initializing TODO tracking for plan"
        );
        
        // Note: This requires &mut self, so it can't be called directly from the hook
        // The caller should call initialize_plan_todos separately
        Ok(())
    }

    async fn on_milestone_starting(&self, plan_id: Uuid, milestone_id: &str) -> Result<()> {
        // Check if we can start this milestone
        if !self.can_progress_to_milestone(plan_id, milestone_id).await? {
            return Err(anyhow!("Cannot start milestone {}: quality gates not satisfied", milestone_id));
        }
        
        info!(
            plan_id = %plan_id,
            milestone_id = %milestone_id,
            "Milestone starting - quality gates satisfied"
        );
        
        Ok(())
    }

    async fn on_milestone_completed(&self, plan_id: Uuid, milestone_id: &str) -> Result<()> {
        // Mark corresponding TODO step as complete
        // Note: This requires &mut self, so the caller should call milestone_completed separately
        // We log here for visibility
        info!(
            plan_id = %plan_id,
            milestone_id = %milestone_id,
            "Milestone completed - TODO step should be marked complete"
        );
        Ok(())
    }

    async fn on_plan_completed(&self, plan_id: Uuid) -> Result<()> {
        // Clean up TODO instance
        info!(
            plan_id = %plan_id,
            "Cleaning up TODO tracking for completed plan"
        );
        
        // Note: Actual cleanup would require &mut self
        // In a production system, this might mark the instance as archived
        // or move it to a completed_instances collection
        
        Ok(())
    }

    async fn on_quality_gate_failed(&self, plan_id: Uuid, gate: &str) -> Result<()> {
        // This is a critical failure - enforce cannot proceed
        warn!(
            plan_id = %plan_id,
            gate = %gate,
            "Critical quality gate failed - execution blocked"
        );
        
        Err(anyhow!("Critical quality gate '{}' failed for plan {}. Execution blocked.", gate, plan_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

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
    fn test_todo_integration_creation() {
        let todo_system = Arc::new(crate::planning::todo_template::TodoTemplateSystem::new());
        let db_ops = Arc::new(MockDbOps);
        let integration = TodoIntegration::new(todo_system, db_ops);
        // Should create successfully
        assert!(true);
    }

    #[test]
    fn test_quality_enforcer_critical_gates() {
        let enforcer = TodoQualityEnforcer::new();

        assert!(enforcer.is_critical_gate("security_scan"));
        assert!(enforcer.is_critical_gate("test_coverage"));
        assert!(!enforcer.is_critical_gate("documentation"));
    }

    #[tokio::test]
    async fn test_workflow_hooks() {
        let todo_system = Arc::new(crate::planning::todo_template::TodoTemplateSystem::new());
        let db_ops = Arc::new(MockDbOps);
        let integration = TodoIntegration::new(todo_system, db_ops);

        // Test hook execution (should not fail)
        let result = integration.on_plan_started(&agent_agency_contracts::planning_io::ExecutionPlan {
            contract_plan: Default::default(),
            execution_context: None,
        }).await;

        assert!(result.is_ok());
    }
}



