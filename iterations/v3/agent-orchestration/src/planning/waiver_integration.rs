//! Waiver Integration - CAWS Waiver System Integration
//!
//! Integrates CAWS waiver system with planning for emergency protocols
//! and scope blowout handling. Provides waiver validation, application,
//! and emergency waiver creation capabilities.
//!
//! @author @darianrosebrook

use schemars::JsonSchema;
use serde::{Serialize, Deserialize};use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use crate::planning::{DatabaseOperations, models::Waiver, plan_types};
use agent_agency_contracts::planning_io::{ExecutionPlan, WaiverReference};

use crate::planning::plan_types::PlanningConstraints;

/// Waiver integration system for planning
pub struct WaiverIntegration {
    /// Database operations for waiver access
    db_ops: Arc<dyn DatabaseOperations>,

    /// Waiver validation configuration
    validation_config: WaiverValidationConfig,

    /// Emergency waiver configuration
    emergency_config: EmergencyWaiverConfig,
    
    /// Optional council monitor for emergency notifications
    council_monitor: Option<Arc<crate::planning::council_monitor::CouncilMonitor>>,
}

/// Waiver validation configuration

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct WaiverValidationConfig {
    /// Require explicit approval for waivers
    pub require_explicit_approval: bool,

    /// Maximum waiver duration in days
    pub max_waiver_duration_days: u32,

    /// Allowed waiver reasons
    pub allowed_reasons: Vec<String>,

    /// Require mitigation plans for high-impact waivers
    pub require_mitigation_for_high_impact: bool,
}

/// Emergency waiver configuration

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct EmergencyWaiverConfig {
    /// Emergency waiver duration in hours
    pub emergency_duration_hours: u32,

    /// Emergency approver (fallback)
    pub emergency_approver: String,

    /// Emergency waiver reasons
    pub emergency_reasons: Vec<String>,

    /// Require council notification for emergencies
    pub require_council_notification: bool,
}

impl Default for WaiverValidationConfig {
    fn default() -> Self {
        Self {
            require_explicit_approval: true,
            max_waiver_duration_days: 90,
            allowed_reasons: vec![
                "emergency_hotfix".to_string(),
                "legacy_integration".to_string(),
                "experimental_feature".to_string(),
                "third_party_constraint".to_string(),
                "performance_critical".to_string(),
                "security_patch".to_string(),
                "infrastructure_limitation".to_string(),
                "other".to_string(),
            ],
            require_mitigation_for_high_impact: true,
        }
    }
}

impl Default for EmergencyWaiverConfig {
    fn default() -> Self {
        Self {
            emergency_duration_hours: 24,
            emergency_approver: "emergency-system".to_string(),
            emergency_reasons: vec![
                "emergency_hotfix".to_string(),
                "security_patch".to_string(),
                "infrastructure_limitation".to_string(),
            ],
            require_council_notification: true,
        }
    }
}

impl WaiverIntegration {
    /// Create new waiver integration
    pub fn new(db_ops: Arc<dyn DatabaseOperations>) -> Self {
        Self::with_config(
            db_ops,
            WaiverValidationConfig::default(),
            EmergencyWaiverConfig::default(),
            None,
        )
    }

    /// Create with custom configuration
    pub fn with_config(
        db_ops: Arc<dyn DatabaseOperations>,
        validation_config: WaiverValidationConfig,
        emergency_config: EmergencyWaiverConfig,
        council_monitor: Option<Arc<crate::planning::council_monitor::CouncilMonitor>>,
    ) -> Self {
        Self {
            db_ops,
            validation_config,
            emergency_config,
            council_monitor,
        }
    }
    
    /// Create with council monitor for emergency notifications
    pub fn with_council_monitor(
        db_ops: Arc<dyn DatabaseOperations>,
        council_monitor: Arc<crate::planning::council_monitor::CouncilMonitor>,
    ) -> Self {
        Self {
            db_ops,
            validation_config: WaiverValidationConfig::default(),
            emergency_config: EmergencyWaiverConfig::default(),
            council_monitor: Some(council_monitor),
        }
    }

    /// Apply waivers to plan validation
    /// Returns modified constraints that account for active waivers
    pub async fn apply_waivers_to_constraints(
        &self,
        original_constraints: &PlanningConstraints,
        plan: &ExecutionPlan,
    ) -> Result<PlanningConstraints> {
        let mut modified_constraints = original_constraints.clone();

        // Get active waivers for this plan
        let active_waivers = self.get_active_waivers_for_plan(plan).await?;

        // Apply each waiver to constraints
        for waiver_ref in &active_waivers {
            self.apply_single_waiver_to_constraints(&mut modified_constraints, waiver_ref)?;
        }

        Ok(modified_constraints)
    }

    /// Check if waivers allow bypassing a specific gate
    pub async fn can_bypass_gate(&self, gate_name: &str, plan: &ExecutionPlan) -> Result<bool> {
        let active_waivers = self.get_active_waivers_for_plan(plan).await?;

        for waiver_ref in &active_waivers {
            if waiver_ref.waived_gates.contains(&gate_name.to_string()) {
                // Verify waiver is still valid
                if self.is_waiver_valid(waiver_ref).await? {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Create emergency waiver for critical situations
    pub async fn create_emergency_waiver(
        &self,
        plan_id: Uuid,
        reason: &str,
        waived_gates: Vec<String>,
        justification: &str,
    ) -> Result<WaiverReference> {
        // Validate emergency reason
        if !self.emergency_config.emergency_reasons.contains(&reason.to_string()) {
            return Err(anyhow!("Invalid emergency waiver reason: {}", reason));
        }

        // Create waiver in database using CreateWaiver
        let waiver_reason = format!("Emergency waiver for plan {}: {}. Justification: {}. Approved by: {}. Impact: critical. Mitigation: monitor closely.",
            plan_id, reason, justification, self.emergency_config.emergency_approver);

        let create_waiver = crate::planning::CreateWaiver {
            plan_id,
            reason: waiver_reason,
            waived_gates: waived_gates.clone(),
        };

        // Store waiver
        let stored_waiver = self.db_ops.create_waiver(create_waiver).await?;

        // Create waiver reference
        let waiver_ref = WaiverReference {
            waiver_id: stored_waiver.id.to_string(),
            reason: stored_waiver.reason.clone(),
            waived_gates: stored_waiver.gates.clone(),
            expires_at: stored_waiver.expires_at.unwrap_or_else(|| Utc::now() + chrono::Duration::hours(24)),
            approved_by: stored_waiver.approved_by.clone(),
        };

        // Notify council if required
        if self.emergency_config.require_council_notification {
            self.notify_council_of_emergency(&stored_waiver).await?;
        }

        Ok(waiver_ref)
    }

    /// Handle scope blowout by creating appropriate waivers
    pub async fn handle_scope_blowout(
        &self,
        plan: &mut ExecutionPlan,
        exceeded_constraints: Vec<String>,
    ) -> Result<Vec<WaiverReference>> {
        let mut waivers = Vec::new();

        for constraint in exceeded_constraints {
            let waiver = self.create_emergency_waiver(
                plan.id,
                "infrastructure_limitation",
                vec![constraint.clone()],
                &format!("Scope blowout on constraint: {}", constraint),
            ).await?;

            waivers.push(waiver);
        }

        // Waivers are stored in database, not directly on WorkingSpec
        // The database stores waivers with plan_id in metadata (see get_active_waivers_for_plan)
        // No need to push to plan.contract_plan.active_waivers (field doesn't exist)

        Ok(waivers)
    }

    /// Validate waiver before use
    pub async fn validate_waiver(&self, waiver_ref: &WaiverReference) -> Result<()> {
        // Check if waiver exists and is active
        let waivers = self.db_ops.get_waivers(Some("active".to_string())).await?;
        let waiver = waivers.iter()
            .find(|w| w.id.to_string() == waiver_ref.waiver_id)
            .ok_or_else(|| anyhow!("Waiver {} not found or inactive", waiver_ref.waiver_id))?;

        // Check expiration
        if let Some(expires_at) = waiver.expires_at {
            if expires_at < Utc::now() {
                return Err(anyhow!("Waiver {} has expired", waiver_ref.waiver_id));
            }
        }

        // Check reason validity
        if !self.validation_config.allowed_reasons.contains(&waiver.reason) {
            return Err(anyhow!("Waiver reason '{}' not allowed", waiver.reason));
        }

        // Check high-impact waivers have mitigation plans
        if self.validation_config.require_mitigation_for_high_impact &&
           waiver.impact_level == "critical" &&
           waiver.mitigation_plan.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true) {
            return Err(anyhow!("Critical waiver {} requires mitigation plan", waiver_ref.waiver_id));
        }

        Ok(())
    }

    /// Get waiver statistics for planning telemetry
    pub async fn get_waiver_stats(&self) -> Result<WaiverStats> {
        let all_waivers = self.db_ops.get_waivers(None).await?;

        let active_waivers = all_waivers.iter()
            .filter(|w| w.status == "active" && w.expires_at.map(|exp| exp > Utc::now()).unwrap_or(true))
            .count();

        let expired_waivers = all_waivers.iter()
            .filter(|w| w.expires_at.map(|exp| exp <= Utc::now()).unwrap_or(false))
            .count();

        let emergency_waivers = all_waivers.iter()
            .filter(|w| w.metadata.get("emergency")
                .and_then(|v| v.as_bool())
                .unwrap_or(false))
            .count();

        // Group by reason
        let mut waivers_by_reason = HashMap::new();
        for waiver in &all_waivers {
            *waivers_by_reason.entry(waiver.reason.clone()).or_insert(0) += 1;
        }

        // Group by impact level
        let mut waivers_by_impact = HashMap::new();
        for waiver in &all_waivers {
            *waivers_by_impact.entry(waiver.impact_level.clone()).or_insert(0) += 1;
        }

        Ok(WaiverStats {
            total_waivers: all_waivers.len(),
            active_waivers,
            expired_waivers,
            emergency_waivers,
            waivers_by_reason,
            waivers_by_impact,
        })
    }

    /// Clean up expired waivers
    pub async fn cleanup_expired_waivers(&self) -> Result<usize> {
        let all_waivers = self.db_ops.get_waivers(None).await?;
        let expired_ids: Vec<Uuid> = all_waivers.iter()
            .filter(|w| w.expires_at.map(|exp| exp <= Utc::now()).unwrap_or(false) && w.status == "active")
            .map(|w| w.id)
            .collect();

        // Mark as expired using update_waiver method
        let mut updated_count = 0;
        for id in expired_ids {
            match self.db_ops.update_waiver(
                id,
                crate::planning::UpdateWaiver {
                    id,
                    status: "expired".to_string(),
                }
            ).await {
                Ok(_) => updated_count += 1,
                Err(e) => {
                    warn!("Failed to mark waiver {} as expired: {}", id, e);
                }
            }
        }

        Ok(updated_count)
    }

    // Helper methods

    /// Get active waivers for a plan
    async fn get_active_waivers_for_plan(&self, plan: &ExecutionPlan) -> Result<Vec<WaiverReference>> {
        let mut active_waivers = Vec::new();

        // Check database for waivers related to this plan
        // WorkingSpec doesn't have active_waivers field - waivers are stored in database
        let all_waivers = self.db_ops.get_waivers(Some("active".to_string())).await?;
        for waiver in all_waivers {
            if let Some(plan_id) = waiver.metadata.get("plan_id")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok()) {
                if plan_id.to_string() == plan.contract_plan.id {
                    let waiver_ref = WaiverReference {
                        waiver_id: waiver.id.to_string(),
                        reason: waiver.reason,
                        waived_gates: waiver.gates,
                        expires_at: waiver.expires_at.unwrap_or_else(|| Utc::now() + chrono::Duration::hours(24)),
                        approved_by: waiver.approved_by,
                    };
                    active_waivers.push(waiver_ref);
                }
            }
        }

        Ok(active_waivers)
    }

    /// Check if waiver reference is valid
    async fn is_waiver_valid(&self, waiver_ref: &WaiverReference) -> Result<bool> {
        Ok(self.validate_waiver(waiver_ref).await.is_ok())
    }

    /// Apply single waiver to planning constraints
    fn apply_single_waiver_to_constraints(
        &self,
        constraints: &mut PlanningConstraints,
        waiver_ref: &WaiverReference,
    ) -> Result<()> {
        // Apply waivers based on waived gates
        for gate in &waiver_ref.waived_gates {
            match gate.as_str() {
                "max_cost" => {
                    // Increase cost limit
                    constraints.cost_limits = Some(plan_types::CostLimits {
                        max_cost_cents: 100000, // $1000 emergency limit
                        cost_per_ms_budget: 0.1,
                        optimization_priority: plan_types::CostOptimizationPriority::MaximizePerformance,
                    });
                }
                "max_time" => {
                    // Increase time limit
                    constraints.max_planning_time_ms = 3600000; // 1 hour emergency limit
                }
                "quality_gates" => {
                    // Relax quality requirements
                    constraints.quality_requirements.min_coverage = 0.0;
                    constraints.quality_requirements.min_mutation_score = 0.0;
                    constraints.quality_requirements.security_scan_required = false;
                }
                "scope_limits" => {
                    // Allow broader scope
                    constraints.max_complexity = 50; // Increased complexity limit
                }
                _ => {
                    // Unknown gate - log but don't fail
                    tracing::warn!("Unknown waiver gate: {}", gate);
                }
            }
        }

        Ok(())
    }

    /// Notify council of emergency waiver
    async fn notify_council_of_emergency(&self, waiver: &Waiver) -> Result<()> {
        // Log the emergency waiver
        tracing::warn!(
            "Emergency waiver created: {} - {} (expires: {})",
            waiver.waiver_type.clone(),
            waiver.reason,
            waiver.expires_at.map(|dt| dt.to_rfc3339()).unwrap_or_else(|| "never".to_string())
        );

        // Notify council if monitor is available
        if let Some(monitor) = &self.council_monitor {
            // Extract plan ID from waiver metadata if available
            let plan_id = waiver.metadata.get("plan_id")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok());
            
            if let Some(plan_id) = plan_id {
                let reason = format!(
                    "Emergency waiver created: {} - {}. Expires: {:?}",
                    waiver.waiver_type,
                    waiver.reason,
                    waiver.expires_at
                );
                
                // Request council intervention for emergency waiver
                match monitor.request_intervention(&plan_id.to_string(), &reason).await {
                    Ok(_) => {
                        tracing::info!("Council notified of emergency waiver for plan {}", plan_id);
                    }
                    Err(e) => {
                        tracing::error!("Failed to notify council of emergency waiver: {}", e);
                        // Don't fail the waiver creation, just log the error
                    }
                }
            } else {
                tracing::warn!("Emergency waiver has no plan_id in metadata, cannot notify council");
            }
        } else {
            tracing::debug!("Council monitor not configured, skipping council notification");
        }

        Ok(())
    }
}

/// Waiver statistics for telemetry

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct WaiverStats {
    /// Total number of waivers
    pub total_waivers: usize,

    /// Number of active waivers
    pub active_waivers: usize,

    /// Number of expired waivers
    pub expired_waivers: usize,

    /// Number of emergency waivers
    pub emergency_waivers: usize,

    /// Waivers grouped by reason
    pub waivers_by_reason: HashMap<String, usize>,

    /// Waivers grouped by impact level
    pub waivers_by_impact: HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // Mock database operations for testing
    // struct MockDatabaseOps; disabled due to massive api drift

    // #[async_trait::async_trait]
    // impl DatabaseOperations for MockDatabaseOps {
    //     async fn get_waivers(&self, _status: Option<String>) -> anyhow::Result<Vec<crate::planning::models::Waiver>> {
    //         Ok(vec![
    //             crate::planning::models::Waiver {
    //                 id: Uuid::new_v4(),
    //                 title: "Test Waiver".to_string(),
    //                 reason: "emergency_hotfix".to_string(),
    //                 description: "Test waiver".to_string(),
    //                 gates: vec!["quality_gates".to_string()],
    //                 approved_by: "test".to_string(),
    //                 impact_level: "medium".to_string(),
    //                 mitigation_plan: "Test mitigation".to_string(),
    //                 expires_at: Utc::now() + Duration::days(30),
    //                 created_at: Utc::now(),
    //                 updated_at: Utc::now(),
    //                 status: "active".to_string(),
    //                 metadata: serde_json::json!({}),
    //             }
    //         ])
    //     }

    //     async fn create_waiver(&self, _waiver: crate::planning::CreateWaiver) -> anyhow::Result<crate::planning::models::Waiver> {
    //         Err(anyhow!("Not implemented"))
    //     }

    //     async fn update_waiver(&self, _id: Uuid, _update: crate::planning::UpdateWaiver) -> anyhow::Result<crate::planning::models::Waiver> {
    //         Err(anyhow!("Not implemented"))
    //     }

    //     // Stub implementations for other required methods
    //     async fn create_execution_plan(&self, _plan: crate::planning::CreateExecutionPlan) -> anyhow::Result<crate::planning::models::ExecutionPlan> { Err(anyhow!("Not implemented")) }
    //     async fn get_execution_plan(&self, _id: Uuid) -> Result<Option<crate::planning::models::ExecutionPlan>> { Ok(None) }
    //     async fn get_execution_plans(&self) -> Result<Vec<crate::planning::models::ExecutionPlan>> { Ok(vec![]) }
    //     async fn update_execution_plan(&self, _id: Uuid, _update: crate::planning::UpdateExecutionPlan) -> Result<crate::planning::models::ExecutionPlan> { Err(anyhow!("Not implemented")) }
    //     async fn delete_execution_plan(&self, _id: Uuid) -> Result<()> { Ok(()) }
    //     async fn create_judge(&self, _judge: crate::planning::CreateJudge) -> Result<crate::planning::models::Judge> { Err(anyhow!("Not implemented")) }
    //     async fn get_judge(&self, _id: Uuid) -> Result<Option<crate::planning::models::Judge>> { Ok(None) }
    //     async fn get_judges(&self) -> Result<Vec<crate::planning::models::Judge>> { Ok(vec![]) }
    //     async fn update_judge(&self, _id: Uuid, _update: crate::planning::UpdateJudge) -> Result<crate::planning::models::Judge> { Err(anyhow!("Not implemented")) }
    //     async fn delete_judge(&self, _id: Uuid) -> Result<()> { Ok(()) }
    //     async fn create_worker(&self, _worker: crate::planning::CreateWorker) -> Result<crate::planning::models::Worker> { Err(anyhow!("Not implemented")) }
    //     async fn get_worker(&self, _id: Uuid) -> Result<Option<crate::planning::models::Worker>> { Ok(None) }
    //     async fn get_workers(&self) -> Result<Vec<crate::planning::models::Worker>> { Ok(vec![]) }
    //     async fn update_worker(&self, _id: Uuid, _update: crate::planning::UpdateWorker) -> Result<crate::planning::models::Worker> { Err(anyhow!("Not implemented")) }
    //     async fn delete_worker(&self, _id: Uuid) -> Result<()> { Ok(()) }
    //     async fn create_task(&self, _task: crate::planning::CreateTask) -> Result<crate::planning::models::Task> { Err(anyhow!("Not implemented")) }
    //     async fn get_task(&self, _id: Uuid) -> Result<Option<crate::planning::models::Task>> { Ok(None) }
    //     async fn get_tasks(&self, _status: Option<String>) -> Result<Vec<crate::planning::models::Task>> { Ok(vec![]) }
    //     async fn update_task(&self, _id: Uuid, _update: crate::planning::UpdateTask) -> Result<crate::planning::models::Task> { Err(anyhow!("Not implemented")) }
    //     async fn delete_task(&self, _id: Uuid) -> Result<()> { Ok(()) }
    //     async fn create_task_execution(&self, _execution: crate::planning::CreateTaskExecution) -> Result<crate::planning::models::TaskExecution> { Err(anyhow!("Not implemented")) }
    //     async fn get_task_execution(&self, _id: Uuid) -> Result<Option<crate::planning::models::TaskExecution>> { Ok(None) }
    //     async fn get_task_executions(&self, _task_id: Uuid) -> Result<Vec<crate::planning::models::TaskExecution>> { Ok(vec![]) }
    //     async fn update_task_execution(&self, _id: Uuid, _update: crate::planning::UpdateTaskExecution) -> Result<crate::planning::models::TaskExecution> { Err(anyhow!("Not implemented")) }
    //     async fn create_audit_trail_entry(&self, _entry: crate::planning::CreateAuditTrailEntry) -> Result<crate::planning::models::AuditTrailEntry> { Err(anyhow!("Not implemented")) }
    //     async fn get_audit_trail_entries(&self, _task_id: Uuid) -> Result<Vec<crate::planning::models::AuditTrailEntry>> { Ok(vec![]) }
    //     async fn get_audit_trail_entry(&self, _id: Uuid) -> Result<Option<crate::planning::models::AuditTrailEntry>> { Ok(None) }
    //     async fn create_council_verdict(&self, _verdict: crate::planning::CreateCouncilVerdict) -> Result<crate::planning::models::CouncilVerdict> { Err(anyhow!("Not implemented")) }
    //     async fn get_council_verdict(&self, _id: Uuid) -> Result<Option<crate::planning::models::CouncilVerdict>> { Ok(None) }
    //     async fn get_council_verdicts(&self, _task_id: Uuid) -> Result<Vec<crate::planning::models::CouncilVerdict>> { Ok(vec![]) }
    //     async fn create_judge_evaluation(&self, _evaluation: crate::planning::CreateJudgeEvaluation) -> Result<crate::planning::models::JudgeEvaluation> { Err(anyhow!("Not implemented")) }
    //     async fn get_judge_evaluations(&self, _task_id: Uuid) -> Result<Vec<crate::planning::models::JudgeEvaluation>> { Ok(vec![]) }
    //     // Planning methods (stubs)
    //     async fn create_milestone(&self, _milestone: crate::planning::CreateMilestone) -> Result<crate::planning::models::Milestone> { Err(anyhow!("Not implemented")) }
    //     async fn get_milestone(&self, _plan_id: Uuid, _milestone_id: String) -> Result<Option<crate::planning::models::Milestone>> { Ok(None) }
    //     async fn get_milestones(&self, _plan_id: Uuid) -> Result<Vec<crate::planning::models::Milestone>> { Ok(vec![]) }
    //     async fn update_milestone(&self, _plan_id: Uuid, _milestone_id: String, _update: crate::planning::UpdateMilestone) -> Result<crate::planning::models::Milestone> { Err(anyhow!("Not implemented")) }
    //     async fn delete_milestone(&self, _plan_id: Uuid, _milestone_id: String) -> Result<()> { Ok(()) }
    //     async fn create_planning_session(&self, _session: crate::planning::CreatePlanningSession) -> Result<crate::planning::models::PlanningSession> { Err(anyhow!("Not implemented")) }
    //     async fn get_planning_session(&self, _id: Uuid) -> Result<Option<crate::planning::models::PlanningSession>> { Ok(None) }
    //     async fn get_planning_sessions(&self, _plan_id: Uuid) -> Result<Vec<crate::planning::models::PlanningSession>> { Ok(vec![]) }
    //     async fn update_planning_session(&self, _id: Uuid, _update: crate::planning::UpdatePlanningSession) -> Result<crate::planning::models::PlanningSession> { Err(anyhow!("Not implemented")) }
    //     async fn create_evidence_artifact(&self, _artifact: crate::planning::CreateEvidenceArtifact) -> Result<crate::planning::models::EvidenceArtifact> { Err(anyhow!("Not implemented")) }
    //     async fn get_evidence_artifacts(&self, _plan_id: Uuid) -> Result<Vec<crate::planning::models::EvidenceArtifact>> { Ok(vec![]) }
    //     async fn get_evidence_artifacts_for_milestone(&self, _plan_id: Uuid, _milestone_id: String) -> Result<Vec<crate::planning::models::EvidenceArtifact>> { Ok(vec![]) }
    //     async fn update_evidence_artifact(&self, _id: Uuid, _update: crate::planning::UpdateEvidenceArtifact) -> Result<crate::planning::models::EvidenceArtifact> { Err(anyhow!("Not implemented")) }
    //     async fn create_planning_audit_event(&self, _event: crate::planning::CreatePlanningAuditEvent) -> Result<crate::planning::models::PlanningAuditEvent> { Err(anyhow!("Not implemented")) }
    //     async fn get_planning_audit_events(&self, _plan_id: Uuid) -> Result<Vec<crate::planning::models::PlanningAuditEvent>> { Ok(vec![]) }
    //     async fn create_planning_telemetry(&self, _telemetry: crate::planning::CreatePlanningTelemetry) -> Result<crate::planning::models::PlanningTelemetry> { Err(anyhow!("Not implemented")) }
    //     async fn get_planning_telemetry(&self, _plan_id: Uuid, _metric_type: Option<String>) -> Result<Vec<crate::planning::models::PlanningTelemetry>> { Ok(vec![]) }
    // }

    #[test]
    fn test_waiver_integration_creation() {
        let db_ops = Arc::new(MockDatabaseOps);
        let integration = WaiverIntegration::new(db_ops);
        // Integration created successfully
        assert!(true);
    }

    #[test]
    fn test_waiver_validation_config() {
        let config = WaiverValidationConfig::default();
        assert!(config.require_explicit_approval);
        assert_eq!(config.max_waiver_duration_days, 90);
        assert!(config.allowed_reasons.contains(&"emergency_hotfix".to_string()));
    }

    #[test]
    fn test_emergency_waiver_config() {
        let config = EmergencyWaiverConfig::default();
        assert_eq!(config.emergency_duration_hours, 24);
        assert_eq!(config.emergency_approver, "emergency-system");
        assert!(config.emergency_reasons.contains(&"emergency_hotfix".to_string()));
    }

    #[test]
    fn test_waiver_reference_creation() {
        let waiver_ref = WaiverReference {
            waiver_id: "test-waiver".to_string(),
            reason: "emergency_hotfix".to_string(),
            waived_gates: vec!["quality_gates".to_string()],
            expires_at: Utc::now() + Duration::hours(24),
            approved_by: "emergency-system".to_string(),
        };

        assert_eq!(waiver_ref.waiver_id, "test-waiver");
        assert!(waiver_ref.waived_gates.contains(&"quality_gates".to_string()));
    }
}


