//! Refinement engine for automatic working spec improvement
//!
//! The refinement engine analyzes validation issues and applies
//! automated improvements to working specifications.

use async_trait::async_trait;

use crate::error::PlanningResult;
use crate::validation_pipeline::ValidationIssue;

/// Refinement suggestion from the engine
#[derive(Debug, Clone)]
pub struct RefinementSuggestion {
    /// Actions that were applied
    pub applied_actions: Vec<String>,

    /// Whether the refinement was successful
    pub successful: bool,

    /// Additional suggestions for manual review
    pub manual_suggestions: Vec<String>,
}

/// Refinement engine trait
#[async_trait]
pub trait RefinementEngine: Send + Sync {
    /// Refine a working specification based on validation issues
    async fn refine_working_spec(
        &self,
        working_spec: &mut agent_agency_contracts::working_spec::WorkingSpec,
        issues: &[ValidationIssue],
    ) -> PlanningResult<RefinementSuggestion>;
}

/// Default refinement engine implementation
pub struct DefaultRefinementEngine {
    // Configuration and state would go here
}

impl DefaultRefinementEngine {
    /// Create a new default refinement engine
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl RefinementEngine for DefaultRefinementEngine {
    async fn refine_working_spec(
        &self,
        working_spec: &mut agent_agency_contracts::working_spec::WorkingSpec,
        issues: &[ValidationIssue],
    ) -> PlanningResult<RefinementSuggestion> {
        let mut applied_actions = Vec::new();
        let mut manual_suggestions = Vec::new();
        let mut successful = true;

        // Process issues and apply fixes
        for issue in issues {
            match self.apply_refinement(working_spec, issue).await {
                Ok(action) => {
                    applied_actions.push(action);
                }
                Err(suggestion) => {
                    manual_suggestions.push(suggestion);
                    successful = false; // If we can't auto-fix, mark as unsuccessful
                }
            }
        }

        Ok(RefinementSuggestion {
            applied_actions,
            successful,
            manual_suggestions,
        })
    }
}

impl DefaultRefinementEngine {
    /// Apply a single refinement based on an issue
    async fn apply_refinement(
        &self,
        working_spec: &mut agent_agency_contracts::working_spec::WorkingSpec,
        issue: &ValidationIssue,
    ) -> Result<String, String> {
        match issue.category.as_str() {
            "constraints" => self.refine_constraints(working_spec, issue),
            "acceptance_criteria" => self.refine_acceptance_criteria(working_spec, issue),
            "test_plan" => self.refine_test_plan(working_spec, issue),
            "schema" => self.refine_schema(working_spec, issue),
            "risk_assessment" => self.refine_risk_assessment(working_spec, issue),
            "dependencies" => self.refine_dependencies(working_spec, issue),
            _ => Err(format!("Cannot auto-fix issue in category '{}': {}", issue.category, issue.description)),
        }
    }

    fn refine_constraints(
        &self,
        working_spec: &mut agent_agency_contracts::working_spec::WorkingSpec,
        issue: &ValidationIssue,
    ) -> Result<String, String> {
        match issue.description.as_str() {
            "Maximum files limit cannot be zero" => {
                if let Some(budget) = &mut working_spec.constraints.budget_limits {
                    budget.max_files = Some(10); // Set reasonable default
                    Ok("Set max_files to 10".to_string())
                } else {
                    Err("Cannot fix budget limits - no budget configuration".to_string())
                }
            }
            "Maximum LOC limit cannot be zero" => {
                if let Some(budget) = &mut working_spec.constraints.budget_limits {
                    budget.max_loc = Some(1000); // Set reasonable default
                    Ok("Set max_loc to 1000".to_string())
                } else {
                    Err("Cannot fix budget limits - no budget configuration".to_string())
                }
            }
            msg if msg.contains("is both allowed and blocked") => {
                // Remove conflicting paths
                if let Some(scope) = &mut working_spec.constraints.scope_restrictions {
                    let conflicting_path = msg
                        .split("'")
                        .nth(1)
                        .unwrap_or("")
                        .to_string();

                    scope.allowed_paths.retain(|p| p != &conflicting_path);
                    Ok(format!("Removed conflicting path '{}' from allowed paths", conflicting_path))
                } else {
                    Err("Cannot fix scope restrictions - no scope configuration".to_string())
                }
            }
            _ => Err(format!("Cannot auto-fix constraint issue: {}", issue.description)),
        }
    }

    fn refine_acceptance_criteria(
        &self,
        working_spec: &mut agent_agency_contracts::working_spec::WorkingSpec,
        issue: &ValidationIssue,
    ) -> Result<String, String> {
        if issue.description.contains("should follow format") {
            // Fix acceptance criteria ID format
            for criterion in &mut working_spec.acceptance_criteria {
                if !criterion.id.starts_with('A') {
                    let new_id = format!("A{}", criterion.id.trim_start_matches(|c: char| !c.is_ascii_digit()));
                    criterion.id = new_id;
                    return Ok(format!("Fixed acceptance criterion ID format"));
                }
            }
        }

        if issue.description.contains("has empty fields") {
            // Try to generate missing fields
            for criterion in &mut working_spec.acceptance_criteria {
                if criterion.given.trim().is_empty() {
                    criterion.given = "Given a valid system state".to_string();
                }
                if criterion.when.trim().is_empty() {
                    criterion.when = format!("When {}", working_spec.description.to_lowercase());
                }
                if criterion.then.trim().is_empty() {
                    criterion.then = "Then the task completes successfully".to_string();
                }
            }
            Ok("Filled empty acceptance criterion fields".to_string())
        } else {
            Err(format!("Cannot auto-fix acceptance criteria issue: {}", issue.description))
        }
    }

    fn refine_test_plan(
        &self,
        working_spec: &mut agent_agency_contracts::working_spec::WorkingSpec,
        issue: &ValidationIssue,
    ) -> Result<String, String> {
        match issue.description.as_str() {
            "T1 tasks must include unit tests" | "T1 tasks require unit tests" => {
                if working_spec.test_plan.unit_tests.is_empty() {
                    working_spec.test_plan.unit_tests.push(
                        agent_agency_contracts::working_spec::UnitTestSpec {
                            description: "Basic functionality test".to_string(),
                            target_function: None,
                            test_cases: vec![
                                "valid_input".to_string(),
                                "invalid_input".to_string(),
                                "edge_cases".to_string(),
                            ],
                        }
                    );
                    Ok("Added basic unit test specification".to_string())
                } else {
                    Ok("Unit tests already present".to_string())
                }
            }
            "T2 tasks should have 80%+ line coverage" | "T1 tasks require 90%+ line coverage" => {
                let target_coverage = if working_spec.risk_tier == 1 { 0.9 } else { 0.8 };
                working_spec.test_plan.coverage_targets = Some(
                    agent_agency_contracts::working_spec::CoverageTargets {
                        line_coverage: Some(target_coverage),
                        branch_coverage: Some(target_coverage - 0.1),
                        mutation_score: Some(if working_spec.risk_tier == 1 { 0.7 } else { 0.5 }),
                    }
                );
                Ok(format!("Set coverage targets to {}% line coverage", (target_coverage * 100.0) as u32))
            }
            _ => Err(format!("Cannot auto-fix test plan issue: {}", issue.description)),
        }
    }

    fn refine_schema(
        &self,
        working_spec: &mut agent_agency_contracts::working_spec::WorkingSpec,
        issue: &ValidationIssue,
    ) -> Result<String, String> {
        // Schema issues are usually structural and hard to auto-fix
        // Most should be caught by validation before reaching here
        Err(format!("Schema issues require manual review: {}", issue.description))
    }

    fn refine_risk_assessment(
        &self,
        working_spec: &mut agent_agency_contracts::working_spec::WorkingSpec,
        issue: &ValidationIssue,
    ) -> Result<String, String> {
        match issue.description.as_str() {
            "T1 tasks require at least 3 acceptance criteria" => {
                while working_spec.acceptance_criteria.len() < 3 {
                    let id = format!("A{}", working_spec.acceptance_criteria.len() + 1);
                    working_spec.acceptance_criteria.push(
                        agent_agency_contracts::working_spec::AcceptanceCriterion {
                            id,
                            given: "Given the system is properly configured".to_string(),
                            when: format!("When the {} functionality is executed", working_spec.title.to_lowercase()),
                            then: "Then it behaves according to specifications".to_string(),
                            priority: Some(agent_agency_contracts::working_spec::MoSCoWPriority::Should),
                        }
                    );
                }
                Ok("Added acceptance criteria to meet T1 requirements".to_string())
            }
            "T2 tasks should have at least 2 acceptance criteria" => {
                while working_spec.acceptance_criteria.len() < 2 {
                    let id = format!("A{}", working_spec.acceptance_criteria.len() + 1);
                    working_spec.acceptance_criteria.push(
                        agent_agency_contracts::working_spec::AcceptanceCriterion {
                            id,
                            given: "Given valid preconditions".to_string(),
                            when: "When the task is performed".to_string(),
                            then: "Then expected outcomes occur".to_string(),
                            priority: Some(agent_agency_contracts::working_spec::MoSCoWPriority::Should),
                        }
                    );
                }
                Ok("Added acceptance criteria to meet T2 recommendations".to_string())
            }
            "Tasks should have acceptance criteria" => {
                if working_spec.acceptance_criteria.is_empty() {
                    working_spec.acceptance_criteria.push(
                        agent_agency_contracts::working_spec::AcceptanceCriterion {
                            id: "A1".to_string(),
                            given: "Given the task is properly set up".to_string(),
                            when: format!("When {}", working_spec.description.to_lowercase()),
                            then: "Then the task completes successfully".to_string(),
                            priority: Some(agent_agency_contracts::working_spec::MoSCoWPriority::Must),
                        }
                    );
                    Ok("Added basic acceptance criterion".to_string())
                } else {
                    Ok("Acceptance criteria already present".to_string())
                }
            }
            _ => Err(format!("Cannot auto-fix risk assessment issue: {}", issue.description)),
        }
    }

    fn refine_dependencies(
        &self,
        working_spec: &mut agent_agency_contracts::working_spec::WorkingSpec,
        issue: &ValidationIssue,
    ) -> Result<String, String> {
        if issue.description.contains("has empty version") {
            // Try to set a reasonable default version
            let dep_name = issue.description
                .split("'")
                .nth(1)
                .unwrap_or("");

            if let Some(version) = working_spec.context.dependencies.get_mut(dep_name) {
                if version.trim().is_empty() {
                    *version = "*".to_string(); // Allow any version
                    return Ok(format!("Set version constraint for '{}' to '*'", dep_name));
                }
            }
        }

        // For version conflicts, we can't easily auto-resolve
        Err(format!("Dependency issues require manual resolution: {}", issue.description))
    }
}

/// Create a default refinement engine
pub fn create_refinement_engine() -> std::sync::Arc<dyn RefinementEngine> {
    std::sync::Arc::new(DefaultRefinementEngine::new())
}
