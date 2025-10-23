use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::resolution::{CawsResolutionResult, ResolutionType};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpertAuthorityLevel {
    /// Junior expert - can participate in standard debates
    Junior,
    /// Senior expert - can request overrides on low-confidence decisions
    Senior,
    /// Principal expert - can override medium-confidence decisions
    Principal,
    /// Chief expert - can override any decision with escalation
    Chief,
}

/// Expert qualification criteria and verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpertQualification {
    pub participant_id: String,
    pub authority_level: ExpertAuthorityLevel,
    pub domain_expertise: Vec<String>, // e.g., ["security", "performance", "reliability"]
    pub qualification_score: f32, // 0.0 to 1.0
    pub verified_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub verification_method: String,
}

/// Override request with justification and conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideRequest {
    pub id: Uuid,
    pub requester_id: String,
    pub target_decision_id: Uuid,
    pub override_reason: OverrideReason,
    pub justification: String,
    pub required_authority_level: ExpertAuthorityLevel,
    pub risk_assessment: OverrideRiskAssessment,
    pub requested_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: OverrideStatus,
}

/// Reasons for requesting an expert override
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OverrideReason {
    /// Critical security concern
    SecurityCritical,
    /// High business impact decision
    BusinessCritical,
    /// Technical correctness issue
    TechnicalCorrectness,
    /// Performance optimization opportunity
    PerformanceCritical,
    /// Regulatory compliance requirement
    RegulatoryCompliance,
    /// Domain expertise gap in participants
    ExpertiseGap,
}

/// Risk assessment for override impact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideRiskAssessment {
    pub impact_level: ImpactLevel,
    pub confidence_in_override: f32, // 0.0 to 1.0
    pub potential_consequences: Vec<String>,
    pub mitigation_measures: Vec<String>,
}

/// Impact levels for override decisions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Override request status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OverrideStatus {
    Pending,
    Approved,
    Rejected,
    Escalated,
    Expired,
}

/// Expert authority manager for qualification and override handling
#[derive(Debug)]
pub struct ExpertAuthorityManager {
    expert_registry: HashMap<String, ExpertQualification>,
    active_overrides: HashMap<Uuid, OverrideRequest>,
    audit_trail: Vec<OverrideAuditEntry>,
}

/// Audit trail entry for override actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideAuditEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub override_id: Uuid,
    pub action: OverrideAction,
    pub actor_id: String,
    pub details: String,
}

/// Types of override actions for audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OverrideAction {
    Requested,
    Approved,
    Rejected,
    Escalated,
    Applied,
    Expired,
    Reviewed,
}

impl Default for ExpertAuthorityManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpertAuthorityManager {
    pub fn new() -> Self {
        Self {
            expert_registry: HashMap::new(),
            active_overrides: HashMap::new(),
            audit_trail: Vec::new(),
        }
    }

    /// Register an expert with their qualifications
    pub fn register_expert(&mut self, qualification: ExpertQualification) -> Result<()> {
        // Validate qualification
        if qualification.qualification_score < 0.0 || qualification.qualification_score > 1.0 {
            return Err(anyhow::anyhow!("Invalid qualification score"));
        }

        // Check for expiration
        if let Some(expires_at) = qualification.expires_at {
            if expires_at <= Utc::now() {
                return Err(anyhow::anyhow!("Expert qualification has expired"));
            }
        }

        self.expert_registry.insert(
            qualification.participant_id.clone(),
            qualification
        );

        Ok(())
    }

    /// Get expert qualification for a participant
    pub fn get_expert_qualification(&self, participant_id: &str) -> Option<&ExpertQualification> {
        self.expert_registry.get(participant_id)
    }

    /// Check if participant has authority level for override
    pub fn has_override_authority(&self, participant_id: &str, required_level: &ExpertAuthorityLevel) -> bool {
        if let Some(qualification) = self.get_expert_qualification(participant_id) {
            // Check if qualification is still valid
            if let Some(expires_at) = qualification.expires_at {
                if expires_at <= Utc::now() {
                    return false;
                }
            }

            // Check authority level hierarchy
            match (&qualification.authority_level, required_level) {
                (ExpertAuthorityLevel::Chief, _) => true,
                (ExpertAuthorityLevel::Principal, ExpertAuthorityLevel::Principal | ExpertAuthorityLevel::Senior | ExpertAuthorityLevel::Junior) => true,
                (ExpertAuthorityLevel::Senior, ExpertAuthorityLevel::Senior | ExpertAuthorityLevel::Junior) => true,
                (ExpertAuthorityLevel::Junior, ExpertAuthorityLevel::Junior) => true,
                _ => false,
            }
        } else {
            false
        }
    }

    /// Submit an override request
    pub fn submit_override_request(&mut self, request: OverrideRequest) -> Result<Uuid> {
        // Validate requester has required authority
        if !self.has_override_authority(&request.requester_id, &request.required_authority_level) {
            return Err(anyhow::anyhow!("Requester lacks required authority level"));
        }

        // Check for expiration
        if request.expires_at <= Utc::now() {
            return Err(anyhow::anyhow!("Override request already expired"));
        }

        let request_id = request.id;
        self.active_overrides.insert(request_id, request);

        // Add audit entry
        self.audit_trail.push(OverrideAuditEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            override_id: request_id,
            action: OverrideAction::Requested,
            actor_id: self.active_overrides[&request_id].requester_id.clone(),
            details: format!("Override request submitted for decision {}", self.active_overrides[&request_id].target_decision_id),
        });

        Ok(request_id)
    }

    /// Approve an override request
    pub fn approve_override_request(&mut self, request_id: Uuid, approver_id: &str) -> Result<()> {
        // First check if the request exists and get the required authority level
        let required_level = if let Some(request) = self.active_overrides.get(&request_id) {
            request.required_authority_level.clone()
        } else {
            return Err(anyhow::anyhow!("Override request not found"));
        };

        // Check if approver has authority to approve
        if !self.has_override_authority(approver_id, &required_level) {
            return Err(anyhow::anyhow!("Approver lacks required authority level"));
        }

        // Now we can get the mutable reference and update the request
        if let Some(request) = self.active_overrides.get_mut(&request_id) {
            request.status = OverrideStatus::Approved;

            // Add audit entry
            self.audit_trail.push(OverrideAuditEntry {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                override_id: request_id,
                action: OverrideAction::Approved,
                actor_id: approver_id.to_string(),
                details: format!("Override approved by {}", approver_id),
            });

            Ok(())
        } else {
            Err(anyhow::anyhow!("Override request not found"))
        }
    }

    /// Apply an approved override to a resolution
    pub fn apply_override(
        &mut self,
        request_id: Uuid,
        resolution: &mut CawsResolutionResult,
        override_participant: &str,
    ) -> Result<()> {
        if let Some(request) = self.active_overrides.get(&request_id) {
            if !matches!(request.status, OverrideStatus::Approved) {
                return Err(anyhow::anyhow!("Override request not approved"));
            }

            // Apply the override
            resolution.resolution_type = ResolutionType::ExpertOverride;
            resolution.winning_participant = Some(override_participant.to_string());
            resolution.confidence_score = request.risk_assessment.confidence_in_override.max(0.8);
            resolution.rationale = format!(
                "EXPERT OVERRIDE: {} - {}",
                request.justification,
                resolution.rationale
            );

            resolution.applied_rules.push(format!("EXPERT-OVERRIDE-{}", request_id));

            // Add audit entry
            self.audit_trail.push(OverrideAuditEntry {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                override_id: request_id,
                action: OverrideAction::Applied,
                actor_id: override_participant.to_string(),
                details: format!("Override applied to decision {}", request.target_decision_id),
            });

            Ok(())
        } else {
            Err(anyhow::anyhow!("Override request not found"))
        }
    }

    /// Get active override requests
    pub fn get_active_overrides(&self) -> Vec<&OverrideRequest> {
        self.active_overrides.values()
            .filter(|req| matches!(req.status, OverrideStatus::Pending | OverrideStatus::Approved))
            .collect()
    }

    /// Get audit trail for accountability
    pub fn get_audit_trail(&self, override_id: Option<Uuid>) -> Vec<&OverrideAuditEntry> {
        self.audit_trail.iter()
            .filter(|entry| override_id.map_or(true, |id| entry.override_id == id))
            .collect()
    }

    /// Clean up expired overrides
    pub fn cleanup_expired_overrides(&mut self) -> Vec<Uuid> {
        let mut expired_ids = Vec::new();
        let now = Utc::now();

        self.active_overrides.retain(|id, request| {
            if request.expires_at <= now && matches!(request.status, OverrideStatus::Pending) {
                request.status = OverrideStatus::Expired;
                expired_ids.push(*id);

                // Add audit entry
                self.audit_trail.push(OverrideAuditEntry {
                    id: Uuid::new_v4(),
                    timestamp: now,
                    override_id: *id,
                    action: OverrideAction::Expired,
                    actor_id: "system".to_string(),
                    details: "Override request expired".to_string(),
                });

                false
            } else {
                true
            }
        });

        expired_ids
    }
}

/// This function previously lived with resolution; keep it here to avoid a circular dep.
pub async fn apply_override_policies(
    mut resolution: CawsResolutionResult,
    expert_manager: Option<&std::sync::RwLock<ExpertAuthorityManager>>,
) -> Result<CawsResolutionResult> {
    if resolution.confidence_score < 0.5 {
        if let Some(manager) = expert_manager {
            let manager_read = manager.read().unwrap();
            if let Some(active_override) = manager_read.get_active_overrides()
                .into_iter().find(|req| matches!(req.status, OverrideStatus::Approved)) {
                resolution.resolution_type = ResolutionType::ExpertOverride;
                resolution.confidence_score = active_override.risk_assessment.confidence_in_override.max(0.8);
                resolution.rationale = format!(
                    "EXPERT OVERRIDE ({}): {} - {}",
                    active_override.requester_id, active_override.justification, resolution.rationale
                );
                resolution.applied_rules.push(format!("EXPERT-OVERRIDE-{}", active_override.id));
                return Ok(resolution);
            }
        }
        resolution.resolution_type = ResolutionType::ExpertOverride;
        resolution.confidence_score = 0.6;
        resolution.rationale = format!("Emergency override applied: {}", resolution.rationale);
        resolution.applied_rules.push("CAWS-EMERGENCY-OVERRIDE".into());
    }
    Ok(resolution)
}
