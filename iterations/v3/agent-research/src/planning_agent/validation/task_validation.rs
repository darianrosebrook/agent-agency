//! Task Validation and Risk Assessment
//!
//! This module handles validation of task requests and risk assessment
//! for the planning agent.

use crate::planning_errors::{PlanningError, PlanningResult};
use system_configuration::types::*;

/// Validate a task request for basic requirements
pub fn validate_task_request(task_request: &agent_agency_contracts::task_request::TaskRequest) -> PlanningResult<()> {
    // Validate task ID
    if task_request.id.is_nil() {
        return Err(PlanningError::InvalidTaskRequest("Task ID cannot be nil".to_string()));
    }

    // Validate description
    if task_request.description.trim().is_empty() {
        return Err(PlanningError::InvalidTaskRequest("Task description cannot be empty".to_string()));
    }

    if task_request.description.len() < 10 {
        return Err(PlanningError::InvalidTaskRequest("Task description must be at least 10 characters".to_string()));
    }

    if task_request.description.len() > 10000 {
        return Err(PlanningError::InvalidTaskRequest("Task description cannot exceed 10,000 characters".to_string()));
    }

    // Validate constraints if present
    if let Some(ref constraints) = task_request.constraints {
        if let Some(ref budget) = constraints.budget_limits {
            // Additional validation for T1 tasks
            if matches!(constraints.risk_tier, agent_agency_contracts::task_request::RiskTier::Tier1) {
                if budget.max_files.is_none() && budget.max_loc.is_none() {
                    return Err(PlanningError::InvalidTaskRequest(
                        "T1 tasks must specify budget limits".to_string()
                    ));
                }
            }
        }
    }

    Ok(())
}

/// Assess risk for a task request
pub fn assess_risk(task_request: &agent_agency_contracts::task_request::TaskRequest) -> PlanningResult<RiskAssessment> {
    let mut risk_factors = Vec::new();
    let mut escalation_recommended = false;

    // Assess based on description length and complexity
    let description = &task_request.description;
    let word_count = description.split_whitespace().count();

    if word_count < 5 {
        risk_factors.push("Task description is too brief (< 5 words)".to_string());
    }

    if word_count > 500 {
        risk_factors.push("Task description is very long (> 500 words)".to_string());
    }

    // Assess based on constraints
    let risk_tier = task_request.constraints.as_ref()
        .map(|c| c.risk_tier.clone())
        .unwrap_or(agent_agency_contracts::task_request::RiskTier::Tier2);

    // Assess based on risk tier
    let assessed_tier = match risk_tier {
        agent_agency_contracts::task_request::RiskTier::Tier1 => {
            // Low risk - but check for inconsistencies
            if word_count > 200 {
                risk_factors.push("Low risk tier but complex task characteristics".to_string());
            }
            agent_agency_contracts::task_request::RiskTier::Tier1
        },
        agent_agency_contracts::task_request::RiskTier::Tier2 => {
            // Medium risk - default
            if word_count > 300 {
                risk_factors.push("Medium risk tier but high complexity indicators".to_string());
            }
            agent_agency_contracts::task_request::RiskTier::Tier2
        },
        agent_agency_contracts::task_request::RiskTier::Tier3 => {
            // High risk - requires careful review
            risk_factors.push("High risk tier - requires additional oversight".to_string());
            escalation_recommended = true;
            agent_agency_contracts::task_request::RiskTier::Tier3
        }
    };

    // Check for potentially problematic patterns
    if description.to_lowercase().contains("delete") && description.to_lowercase().contains("all") {
        risk_factors.push("Task involves bulk deletion operations".to_string());
        escalation_recommended = true;
    }

    if description.to_lowercase().contains("production") && description.to_lowercase().contains("emergency") {
        risk_factors.push("Emergency production changes requested".to_string());
        escalation_recommended = true;
    }

    Ok(RiskAssessment {
        assessed_tier,
        risk_factors,
        escalation_recommended,
    })
}
