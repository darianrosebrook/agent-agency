//! CAWS Integration for Parameter Optimization
//!
//! Implements CAWS budget tracking, waiver management, and provenance
//! for LLM parameter optimization with compliance enforcement.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[cfg(feature = "bandit_policy")]
use ParameterSet;

#[cfg(not(feature = "bandit_policy"))]
use crate::bandit_stubs::ParameterSet;

/// CAWS budget tracker for token usage
pub struct CAWSBudgetTracker {
    /// Tokens/day budget per task_type
    token_budgets: Arc<RwLock<HashMap<String, TokenBudget>>>,
}

/// Token budget for a task type
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TokenBudget {
    pub task_type: String,
    pub daily_limit: u64,
    pub used_today: u64,
    #[schemars(with = "String")]

    pub reset_at: DateTime<Utc>,
}

/// Budget check result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BudgetCheckResult {
    pub within_budget: bool,
    pub remaining_tokens: u64,
    pub usage_percentage: f64,
    #[schemars(with = "String")]

    pub reset_time: DateTime<Utc>,
}

/// Waiver ID for budget exceptions
pub type WaiverId = Uuid;

impl CAWSBudgetTracker {
    pub fn new() -> Self {
        Self {
            token_budgets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if proposed parameters would exceed budget
    pub async fn check_budget(
        &self,
        task_type: &str,
        expected_volume: u64,
        expected_tokens_per_task: u32,
    ) -> Result<BudgetCheckResult> {
        let budgets = self.token_budgets.read().unwrap();
        let budget = budgets.get(task_type);

        match budget {
            Some(budget) => {
                let total_expected = expected_volume * expected_tokens_per_task as u64;
                let remaining = budget.daily_limit.saturating_sub(budget.used_today);
                let within_budget = total_expected <= remaining;
                let usage_percentage = (budget.used_today as f64) / (budget.daily_limit as f64) * 100.0;

                Ok(BudgetCheckResult {
                    within_budget,
                    remaining_tokens: remaining,
                    usage_percentage,
                    reset_time: budget.reset_at,
                })
            }
            None => {
                // No budget set - assume unlimited
                Ok(BudgetCheckResult {
                    within_budget: true,
                    remaining_tokens: u64::MAX,
                    usage_percentage: 0.0,
                    reset_time: Utc::now() + chrono::Duration::days(1),
                })
            }
        }
    }

    /// Request waiver for budget exceed
    pub async fn request_waiver(
        &self,
        _task_type: &str,
        _reason: String,
        _approver: String,
        _expiry: DateTime<Utc>,
    ) -> Result<WaiverId> {
        let waiver_id = Uuid::new_v4();

        // TODO: Implement waiver request creation and approval workflow
        //       Currently generates ID only; should create waiver request, send to approver, store in database, and return waiver ID.
        //
        // COMPLETION CHECKLIST:
        // [ ] Create waiver request with all required fields
        // [ ] Send waiver request to approver
        // [ ] Store waiver in database
        // [ ] Track waiver approval status
        // [ ] Handle approval workflow errors
        // [ ] Add unit tests with mock approver
        // [ ] Add integration tests with real waiver workflow
        // [ ] Performance: Request creation should complete in <100ms
        // [ ] Documentation: Document waiver workflow
        //
        // ACCEPTANCE CRITERIA:
        // - Waiver request is created with all fields
        // - Request is sent to approver successfully
        // - Waiver is stored in database
        // - Waiver ID is returned correctly
        // - Approval workflow errors are handled
        //
        // DEPENDENCIES:
        // - Waiver request creation API (Required)
        // - Approver notification system (Required)
        // - Database storage (Required)
        //
        // ESTIMATED EFFORT: 6-8 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (waiver management feature)
        // - Change Budget: ~200 LOC
        // - Reviewer Requirements: Workflow and database expertise

        Ok(waiver_id)
    }

    /// Set budget for a task type
    pub async fn set_budget(
        &self,
        task_type: &str,
        daily_limit: u64,
    ) -> Result<()> {
        let mut budgets = self.token_budgets.write().unwrap();
        budgets.insert(task_type.to_string(), TokenBudget {
            task_type: task_type.to_string(),
            daily_limit,
            used_today: 0,
            reset_at: Utc::now() + chrono::Duration::days(1),
        });
        Ok(())
    }

    /// Record token usage
    pub async fn record_usage(
        &self,
        task_type: &str,
        tokens_used: u64,
    ) -> Result<()> {
        let mut budgets = self.token_budgets.write().unwrap();
        if let Some(budget) = budgets.get_mut(task_type) {
            budget.used_today += tokens_used;
        }
        Ok(())
    }
}

/// Parameter change provenance for CAWS compliance
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ParameterChangeProvenance {
    #[schemars(with = "String")]
    pub change_id: Uuid,
    pub task_type: String,
    pub old_params: ParameterSet,
    pub new_params: ParameterSet,
    pub reason: String,
    pub approved_by: Option<String>,
    pub waiver_id: Option<WaiverId>,
    pub policy_version: String,
    #[schemars(with = "String")]

    pub timestamp: DateTime<Utc>,
}

/// CAWS compliance validator
pub struct CAWSComplianceValidator {
    budget_tracker: Arc<CAWSBudgetTracker>,
    provenance_log: Arc<RwLock<Vec<ParameterChangeProvenance>>>,
}

impl CAWSComplianceValidator {
    pub fn new() -> Self {
        Self {
            budget_tracker: Arc::new(CAWSBudgetTracker::new()),
            provenance_log: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Validate parameter change for CAWS compliance
    pub async fn validate_parameter_change(
        &self,
        task_type: &str,
        old_params: &ParameterSet,
        new_params: &ParameterSet,
        reason: String,
        approver: Option<String>,
    ) -> Result<ComplianceValidationResult> {
        // 1. Check budget constraints
        // TODO: Calculate actual volume estimate from task parameters
        //       Currently uses hardcoded value; should calculate actual volume estimate from task parameters for accurate budget checking.
        //
        // COMPLETION CHECKLIST:
        // [ ] Analyze task parameters to estimate volume
        // [ ] Calculate token count from task description and context
        // [ ] Estimate request/response sizes
        // [ ] Account for model-specific tokenization
        // [ ] Handle estimation errors gracefully
        // [ ] Add unit tests with various task parameters
        // [ ] Add integration tests with real task volumes
        // [ ] Performance: Estimation should complete in <1ms
        // [ ] Documentation: Document volume estimation methodology
        //
        // ACCEPTANCE CRITERIA:
        // - Volume estimate reflects actual task parameters
        // - Token count is calculated accurately
        // - Request/response sizes are estimated correctly
        // - Estimation errors are handled gracefully
        // - Estimation performance is acceptable
        //
        // DEPENDENCIES:
        // - Token counting utilities (Required)
        // - Task parameter analysis (Required)
        // - Model tokenization info (Required)
        //
        // ESTIMATED EFFORT: 4-6 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (budget management feature)
        // - Change Budget: ~150 LOC
        // - Reviewer Requirements: Tokenization expertise
        let budget_check = self.budget_tracker
            .check_budget(task_type, 1000, new_params.max_tokens as u32) // Hardcoded volume estimate
            .await?;

        if !budget_check.within_budget {
            return Ok(ComplianceValidationResult {
                approved: false,
                reason: format!("Budget exceeded: {}% usage", budget_check.usage_percentage),
                waiver_required: true,
            });
        }

        // 2. Check parameter constraints
        let constraint_violations = self.check_parameter_constraints(new_params);
        if !constraint_violations.is_empty() {
            return Ok(ComplianceValidationResult {
                approved: false,
                reason: format!("Parameter constraints violated: {:?}", constraint_violations),
                waiver_required: false,
            });
        }

        // 3. Log provenance
        let change_id = Uuid::new_v4();
        let provenance = ParameterChangeProvenance {
            change_id,
            task_type: task_type.to_string(),
            old_params: old_params.clone(),
            new_params: new_params.clone(),
            reason,
            approved_by: approver,
            waiver_id: None,
            policy_version: new_params.policy_version.clone(),
            timestamp: Utc::now(),
        };

        {
            let mut log = self.provenance_log.write().unwrap();
            log.push(provenance);
        }

        Ok(ComplianceValidationResult {
            approved: true,
            reason: "CAWS compliance validated".to_string(),
            waiver_required: false,
        })
    }

    /// Check parameter constraints
    fn check_parameter_constraints(
        &self,
        params: &ParameterSet,
    ) -> Vec<String> {
        let mut violations = Vec::new();

        // Temperature constraints
        if params.temperature < 0.0 || params.temperature > 2.0 {
            violations.push("Temperature out of range [0.0, 2.0]".to_string());
        }

        // Token constraints
        if params.max_tokens == 0 || params.max_tokens > 100000 {
            violations.push("Max tokens out of range [1, 100000]".to_string());
        }

        // Top-p constraints
        if let Some(top_p) = params.top_p {
            if top_p < 0.0 || top_p > 1.0 {
                violations.push("Top-p out of range [0.0, 1.0]".to_string());
            }
        }

        violations
    }

    /// Get provenance log
    pub fn get_provenance_log(&self) -> Vec<ParameterChangeProvenance> {
        self.provenance_log.read().unwrap().clone()
    }

    /// Get budget tracker
    pub fn budget_tracker(&self) -> Arc<CAWSBudgetTracker> {
        self.budget_tracker.clone()
    }
}

/// Compliance validation result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ComplianceValidationResult {
    pub approved: bool,
    pub reason: String,
    pub waiver_required: bool,
}

pub type Result<T> = std::result::Result<T, anyhow::Error>;
