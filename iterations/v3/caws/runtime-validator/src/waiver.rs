//! CAWS Waiver Management
//!
//! Consolidated waiver generation and approval logic
//! extracted from self-prompting-agent implementations.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;

/// Waiver request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Waiver {
    pub id: String,
    pub task_id: String,
    pub requester: String,
    pub violations: Vec<String>,
    pub justification: String,
    pub risk_assessment: String,
    pub mitigation_plan: String,
    pub requested_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: WaiverStatus,
    pub approver: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejection_reason: Option<String>,
}

/// Waiver status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WaiverStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
}

/// Risk tier for waiver assessment
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskTier {
    Low,
    Medium,
    High,
    Critical,
}

/// Waiver approval request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaiverApprovalRequest {
    pub waiver_id: String,
    pub approver: String,
    pub decision: WaiverDecision,
    pub comments: Option<String>,
}

/// Waiver decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WaiverDecision {
    Approve,
    Reject(String), // rejection reason
}

/// Waiver generation context
#[derive(Debug, Clone)]
pub struct WaiverContext {
    pub task_id: String,
    pub violations: Vec<String>,
    pub risk_tier: String,
    pub requester: String,
    pub budget_overrun: Option<BudgetOverrunDetails>,
}

/// Budget overrun details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetOverrunDetails {
    pub resource_type: String,
    pub requested_amount: u64,
    pub approved_amount: u64,
    pub percentage_over: f32,
}

/// Waiver manager - consolidated waiver logic
pub struct WaiverManager {
    waivers: HashMap<String, Waiver>,
}

impl WaiverManager {
    pub fn new() -> Self {
        Self {
            waivers: HashMap::new(),
        }
    }

    /// Generate waiver for violations
    pub fn generate_waiver(&mut self, context: WaiverContext) -> Waiver {
        let id = format!("waiver-{}-{}", context.task_id, chrono::Utc::now().timestamp());
        let expires_at = Utc::now() + Duration::days(7); // Default 7 days

        let justification = self.generate_justification(&context);
        let risk_assessment = self.assess_risk(&context);
        let mitigation_plan = self.create_mitigation_plan(&context);

        let waiver = Waiver {
            id: id.clone(),
            task_id: context.task_id,
            requester: context.requester,
            violations: context.violations,
            justification,
            risk_assessment,
            mitigation_plan,
            requested_at: Utc::now(),
            expires_at,
            status: WaiverStatus::Pending,
            approver: None,
            approved_at: None,
            rejection_reason: None,
        };

        self.waivers.insert(id.clone(), waiver.clone());
        waiver
    }

    /// Process waiver approval/rejection
    pub fn process_approval(&mut self, request: WaiverApprovalRequest) -> Result<(), String> {
        let waiver = self.waivers.get_mut(&request.waiver_id)
            .ok_or_else(|| format!("Waiver {} not found", request.waiver_id))?;

        if waiver.status != WaiverStatus::Pending {
            return Err(format!("Waiver {} is not in pending status", request.waiver_id));
        }

        waiver.approver = Some(request.approver.clone());

        match request.decision {
            WaiverDecision::Approve => {
                waiver.status = WaiverStatus::Approved;
                waiver.approved_at = Some(Utc::now());
                if let Some(_comments) = request.comments {
                    // Could store approval comments if needed
                }
            }
            WaiverDecision::Reject(reason) => {
                waiver.status = WaiverStatus::Rejected;
                waiver.rejection_reason = Some(reason);
            }
        }

        Ok(())
    }

    /// Check if waiver is valid and approved
    pub fn is_waiver_valid(&self, waiver_id: &str) -> bool {
        if let Some(waiver) = self.waivers.get(waiver_id) {
            waiver.status == WaiverStatus::Approved
                && waiver.expires_at > Utc::now()
        } else {
            false
        }
    }

    /// Get waiver by ID
    pub fn get_waiver(&self, waiver_id: &str) -> Option<&Waiver> {
        self.waivers.get(waiver_id)
    }

    /// List waivers with optional filtering
    pub fn list_waivers(&self, status_filter: Option<WaiverStatus>) -> Vec<&Waiver> {
        self.waivers.values()
            .filter(|w| status_filter.as_ref().map_or(true, |s| w.status == *s))
            .collect()
    }

    /// Clean up expired waivers
    pub fn cleanup_expired(&mut self) -> Vec<String> {
        let now = Utc::now();
        let expired_ids: Vec<String> = self.waivers.iter()
            .filter(|(_, w)| w.expires_at <= now && w.status == WaiverStatus::Pending)
            .map(|(id, _)| id.clone())
            .collect();

        for id in &expired_ids {
            if let Some(waiver) = self.waivers.get_mut(id) {
                waiver.status = WaiverStatus::Expired;
            }
        }

        expired_ids
    }

    fn generate_justification(&self, context: &WaiverContext) -> String {
        let mut justification = format!(
            "Requesting waiver for task '{}' with violations: {}\n\n",
            context.task_id,
            context.violations.join(", ")
        );

        if let Some(overrun) = &context.budget_overrun {
            justification.push_str(&format!(
                "Budget overrun in {}: requested {}, approved {}, {}% over.\n",
                overrun.resource_type,
                overrun.requested_amount,
                overrun.approved_amount,
                overrun.percentage_over
            ));
        }

        justification.push_str(&format!(
            "Risk tier: {}\n",
            context.risk_tier
        ));

        justification.push_str("This waiver is necessary because: [auto-generated justification based on context]");

        justification
    }

    fn assess_risk(&self, context: &WaiverContext) -> String {
        let base_risk = match context.risk_tier.as_str() {
            "high" => "High risk task requiring careful review",
            "medium" => "Medium risk task with standard controls",
            "low" => "Low risk task with minimal impact",
            _ => "Unknown risk tier",
        };

        let violation_risk = if context.violations.len() > 2 {
            "Multiple violations increase overall risk"
        } else {
            "Limited violations with contained impact"
        };

        format!("{}\n{}", base_risk, violation_risk)
    }

    fn create_mitigation_plan(&self, context: &WaiverContext) -> String {
        let mut plan = String::new();

        for violation in &context.violations {
            match violation.as_str() {
                "budget-files" => {
                    plan.push_str("- Split work into smaller, focused PRs\n");
                }
                "budget-loc" => {
                    plan.push_str("- Refactor changes to be more modular\n");
                }
                "test-coverage" => {
                    plan.push_str("- Add comprehensive test coverage\n");
                }
                "security-scan" => {
                    plan.push_str("- Address security vulnerabilities\n");
                }
                _ => {
                    plan.push_str(&format!("- Address {} violation\n", violation));
                }
            }
        }

        plan.push_str("- Implement monitoring and rollback procedures\n");
        plan.push_str("- Schedule follow-up review within waiver period\n");

        plan
    }
}

/// Waiver generator - simplified interface for creating waivers
pub struct WaiverGenerator {
    manager: WaiverManager,
}

impl WaiverGenerator {
    pub fn new() -> Self {
        Self {
            manager: WaiverManager::new(),
        }
    }

    /// Create waiver for budget overrun
    pub fn create_budget_waiver(&mut self, context: WaiverContext) -> Waiver {
        self.manager.generate_waiver(context)
    }

    /// Create waiver for quality gate violations
    pub fn create_quality_waiver(&mut self, context: WaiverContext) -> Waiver {
        self.manager.generate_waiver(context)
    }

    /// Get waiver manager for advanced operations
    pub fn manager(&mut self) -> &mut WaiverManager {
        &mut self.manager
    }
}

impl Default for WaiverGenerator {
    fn default() -> Self {
        Self::new()
    }
}
