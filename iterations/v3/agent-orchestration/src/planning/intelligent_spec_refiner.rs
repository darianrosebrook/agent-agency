//! Intelligent Spec Refiner
//!
//! Implements intelligent working spec refinement based on council feedback
//! and validation issues. Integrates with the planning agent's refinement engine
//! for structural fixes and applies semantic improvements based on council directives.
//!
//! When the `research` feature is enabled, this module can use the planning agent's
//! `DefaultRefinementEngine` for validation-based structural fixes in addition to
//! the semantic improvements based on council feedback.
//!
//! @author @darianrosebrook

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info, warn};

use agent_agency_contracts::types::validation::{ValidationCategory, ValidationIssue, ValidationSeverity};
use agent_agency_contracts::working_spec::{
    AcceptanceCriterion, CoverageTargets, MoSCoWPriority, UnitTestSpec, WorkingSpec,
};

use crate::decision_making::{RefinementChange, RefinementDirective};
use crate::planning::refinement_loop::SpecRefiner;

// Optional integration with planning agent's refinement engine
#[cfg(feature = "research")]
use agent_research::planning_agent::refinement_engine::{
    DefaultRefinementEngine as PlanningAgentRefinementEngine,
    RefinementEngine as PlanningAgentRefinementEngineTrait,
};

/// Parsed refinement directive with structured improvement areas
#[derive(Debug, Clone)]
pub struct ParsedRefinementDirective {
    /// Areas that need improvement
    pub improvement_areas: Vec<ImprovementArea>,
    /// Overall refinement priority
    pub priority: RefinementPriority,
    /// Maximum iterations allowed for this refinement
    pub max_iterations: u32,
    /// Acceptance criteria for successful refinement
    pub acceptance_criteria: Vec<String>,
}

/// Specific area of the spec that needs improvement
#[derive(Debug, Clone)]
pub enum ImprovementArea {
    /// Description needs more clarity or detail
    Description {
        issue: String,
        suggestion: String,
    },
    /// Acceptance criteria need improvement
    AcceptanceCriteria {
        issue: String,
        missing_scenarios: Vec<String>,
    },
    /// Constraints need adjustment
    Constraints {
        issue: String,
        constraint_type: ConstraintType,
    },
    /// Test plan needs enhancement
    TestPlan {
        issue: String,
        missing_tests: Vec<String>,
        target_coverage: Option<f64>,
    },
    /// Risk assessment needs update
    RiskAssessment {
        issue: String,
        risk_tier_adjustment: Option<i32>,
    },
    /// Dependencies need clarification
    Dependencies {
        issue: String,
        missing_dependencies: Vec<String>,
    },
    /// Scope boundaries need adjustment
    Scope {
        issue: String,
        paths_to_add: Vec<String>,
        paths_to_remove: Vec<String>,
    },
    /// Generic improvement area
    Other {
        category: String,
        issue: String,
        suggestion: String,
    },
}

/// Type of constraint that needs adjustment
#[derive(Debug, Clone)]
pub enum ConstraintType {
    MaxFiles,
    MaxLoc,
    ScopeRestrictions,
    BudgetLimits,
    TimeConstraints,
}

/// Priority of the refinement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefinementPriority {
    Critical,
    High,
    Medium,
    Low,
}

impl Default for RefinementPriority {
    fn default() -> Self {
        Self::Medium
    }
}

/// Refinement action that was applied
#[derive(Debug, Clone)]
pub struct RefinementAction {
    /// Area that was refined
    pub area: String,
    /// Description of the change
    pub description: String,
    /// Whether the action was successful
    pub successful: bool,
}

/// Result of a refinement operation
#[derive(Debug, Clone)]
pub struct RefinementResult {
    /// The refined working spec
    pub refined_spec: WorkingSpec,
    /// Actions that were applied
    pub actions: Vec<RefinementAction>,
    /// Issues that could not be automatically resolved
    pub unresolved_issues: Vec<String>,
    /// Quality improvement estimate (0.0 - 1.0)
    pub estimated_quality_improvement: f64,
}

/// Intelligent spec refiner that uses council feedback and validation
/// to intelligently refine working specifications
pub struct IntelligentSpecRefiner {
    /// Configuration for refinement behavior
    config: IntelligentRefinerConfig,
    /// Optional planning agent refinement engine for validation-based fixes
    #[cfg(feature = "research")]
    planning_agent_refiner: Option<PlanningAgentRefinementEngine>,
}

/// Configuration for the intelligent refiner
#[derive(Debug, Clone)]
pub struct IntelligentRefinerConfig {
    /// Enable verbose logging of refinement decisions
    pub verbose: bool,
    /// Maximum number of changes per refinement pass
    pub max_changes_per_pass: usize,
    /// Minimum quality improvement required to continue refinement
    pub min_quality_improvement: f64,
    /// Default coverage target for T1 tasks
    pub t1_coverage_target: f64,
    /// Default coverage target for T2 tasks
    pub t2_coverage_target: f64,
    /// Default coverage target for T3 tasks
    pub t3_coverage_target: f64,
}

impl Default for IntelligentRefinerConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            max_changes_per_pass: 10,
            min_quality_improvement: 0.01,
            t1_coverage_target: 0.90,
            t2_coverage_target: 0.80,
            t3_coverage_target: 0.70,
        }
    }
}

impl IntelligentSpecRefiner {
    /// Create a new intelligent spec refiner with default configuration
    pub fn new() -> Self {
        Self {
            config: IntelligentRefinerConfig::default(),
            #[cfg(feature = "research")]
            planning_agent_refiner: Some(PlanningAgentRefinementEngine::new()),
        }
    }

    /// Create a new intelligent spec refiner with custom configuration
    pub fn with_config(config: IntelligentRefinerConfig) -> Self {
        Self {
            config,
            #[cfg(feature = "research")]
            planning_agent_refiner: Some(PlanningAgentRefinementEngine::new()),
        }
    }

    /// Create a new intelligent spec refiner without planning agent integration
    #[allow(dead_code)]
    pub fn without_planning_agent(config: IntelligentRefinerConfig) -> Self {
        Self {
            config,
            #[cfg(feature = "research")]
            planning_agent_refiner: None,
        }
    }

    /// Apply validation-based refinements using the planning agent's refinement engine
    #[cfg(feature = "research")]
    pub async fn apply_validation_refinements(
        &self,
        spec: &mut WorkingSpec,
        issues: &[ValidationIssue],
    ) -> Result<Vec<RefinementAction>> {
        let mut actions = Vec::new();

        if let Some(ref refiner) = self.planning_agent_refiner {
            match refiner.refine_working_spec(spec, issues).await {
                Ok(suggestion) => {
                    for action_desc in suggestion.applied_actions {
                        actions.push(RefinementAction {
                            area: "validation".to_string(),
                            description: action_desc,
                            successful: true,
                        });
                    }
                    for manual in suggestion.manual_suggestions {
                        actions.push(RefinementAction {
                            area: "manual_review".to_string(),
                            description: manual,
                            successful: false,
                        });
                    }
                    info!(
                        "Planning agent refinement applied {} actions",
                        actions.len()
                    );
                }
                Err(e) => {
                    warn!("Planning agent refinement failed: {:?}", e);
                    actions.push(RefinementAction {
                        area: "validation".to_string(),
                        description: format!("Planning agent refinement failed: {:?}", e),
                        successful: false,
                    });
                }
            }
        }

        Ok(actions)
    }

    /// Apply validation-based refinements (no-op when research feature is disabled)
    #[cfg(not(feature = "research"))]
    pub async fn apply_validation_refinements(
        &self,
        _spec: &mut WorkingSpec,
        _issues: &[ValidationIssue],
    ) -> Result<Vec<RefinementAction>> {
        debug!("Planning agent refinement not available (research feature disabled)");
        Ok(Vec::new())
    }

    /// Parse council feedback into structured refinement directives
    pub fn parse_council_feedback(&self, refinement_reason: &str) -> ParsedRefinementDirective {
        let mut improvement_areas = Vec::new();
        let reason_lower = refinement_reason.to_lowercase();

        // Parse refinement directive format: "Refinement required: RefinementDirective { ... }"
        if let Some(directive) = self.extract_refinement_directive(refinement_reason) {
            return directive;
        }

        // Fallback: Parse natural language feedback
        // Check for description issues
        if reason_lower.contains("description")
            || reason_lower.contains("unclear")
            || reason_lower.contains("vague")
        {
            improvement_areas.push(ImprovementArea::Description {
                issue: self.extract_issue_context(refinement_reason, "description"),
                suggestion: "Improve clarity and add specific details".to_string(),
            });
        }

        // Check for acceptance criteria issues
        if reason_lower.contains("acceptance")
            || reason_lower.contains("criteria")
            || reason_lower.contains("given")
            || reason_lower.contains("when")
            || reason_lower.contains("then")
        {
            improvement_areas.push(ImprovementArea::AcceptanceCriteria {
                issue: self.extract_issue_context(refinement_reason, "acceptance criteria"),
                missing_scenarios: self.extract_missing_scenarios(refinement_reason),
            });
        }

        // Check for constraint issues
        if reason_lower.contains("constraint")
            || reason_lower.contains("budget")
            || reason_lower.contains("limit")
            || reason_lower.contains("max_files")
            || reason_lower.contains("max_loc")
        {
            let constraint_type = if reason_lower.contains("file") {
                ConstraintType::MaxFiles
            } else if reason_lower.contains("loc") || reason_lower.contains("lines") {
                ConstraintType::MaxLoc
            } else if reason_lower.contains("scope") {
                ConstraintType::ScopeRestrictions
            } else if reason_lower.contains("budget") {
                ConstraintType::BudgetLimits
            } else {
                ConstraintType::TimeConstraints
            };

            improvement_areas.push(ImprovementArea::Constraints {
                issue: self.extract_issue_context(refinement_reason, "constraints"),
                constraint_type,
            });
        }

        // Check for test plan issues
        if reason_lower.contains("test")
            || reason_lower.contains("coverage")
            || reason_lower.contains("unit")
            || reason_lower.contains("integration")
        {
            let target_coverage = if reason_lower.contains("90") {
                Some(0.90)
            } else if reason_lower.contains("80") {
                Some(0.80)
            } else if reason_lower.contains("70") {
                Some(0.70)
            } else {
                None
            };

            improvement_areas.push(ImprovementArea::TestPlan {
                issue: self.extract_issue_context(refinement_reason, "test plan"),
                missing_tests: self.extract_missing_tests(refinement_reason),
                target_coverage,
            });
        }

        // Check for risk assessment issues
        if reason_lower.contains("risk")
            || reason_lower.contains("tier")
            || reason_lower.contains("security")
            || reason_lower.contains("safety")
        {
            improvement_areas.push(ImprovementArea::RiskAssessment {
                issue: self.extract_issue_context(refinement_reason, "risk assessment"),
                risk_tier_adjustment: None,
            });
        }

        // Check for scope issues
        if reason_lower.contains("scope")
            || reason_lower.contains("path")
            || reason_lower.contains("allowed")
            || reason_lower.contains("blocked")
        {
            improvement_areas.push(ImprovementArea::Scope {
                issue: self.extract_issue_context(refinement_reason, "scope"),
                paths_to_add: Vec::new(),
                paths_to_remove: Vec::new(),
            });
        }

        // If no specific areas identified, add a generic improvement
        if improvement_areas.is_empty() {
            improvement_areas.push(ImprovementArea::Other {
                category: "general".to_string(),
                issue: refinement_reason.to_string(),
                suggestion: "Review and improve the working specification".to_string(),
            });
        }

        // Determine priority based on content
        let priority = if reason_lower.contains("critical")
            || reason_lower.contains("security")
            || reason_lower.contains("urgent")
        {
            RefinementPriority::Critical
        } else if reason_lower.contains("important") || reason_lower.contains("high") {
            RefinementPriority::High
        } else if reason_lower.contains("low") || reason_lower.contains("minor") {
            RefinementPriority::Low
        } else {
            RefinementPriority::Medium
        };

        ParsedRefinementDirective {
            improvement_areas,
            priority,
            max_iterations: 3,
            acceptance_criteria: vec!["Council approves the refined specification".to_string()],
        }
    }

    /// Extract refinement directive from structured format
    fn extract_refinement_directive(&self, reason: &str) -> Option<ParsedRefinementDirective> {
        // Try to parse structured directive format
        if !reason.contains("RefinementDirective") && !reason.contains("required_changes") {
            return None;
        }

        let mut improvement_areas = Vec::new();
        let mut acceptance_criteria = Vec::new();
        let mut max_iterations = 3u32;

        // Extract max_iterations if present
        if let Some(iter_match) = reason.find("max_iterations:") {
            if let Some(num_start) = reason[iter_match..].find(char::is_numeric) {
                let num_str: String = reason[iter_match + num_start..]
                    .chars()
                    .take_while(|c| c.is_numeric())
                    .collect();
                if let Ok(num) = num_str.parse() {
                    max_iterations = num;
                }
            }
        }

        // Extract acceptance_criteria if present
        if reason.contains("acceptance_criteria:") {
            // Simple extraction - look for quoted strings after acceptance_criteria
            let criteria_part = reason.split("acceptance_criteria:").nth(1).unwrap_or("");
            for part in criteria_part.split('"') {
                let trimmed = part.trim();
                if !trimmed.is_empty()
                    && !trimmed.starts_with('[')
                    && !trimmed.starts_with(']')
                    && !trimmed.starts_with(',')
                {
                    acceptance_criteria.push(trimmed.to_string());
                }
            }
        }

        // Parse required_changes for improvement areas
        if reason.contains("required_changes:") {
            // Look for change categories
            if reason.contains("AcceptanceCriteria") || reason.contains("acceptance_criteria") {
                improvement_areas.push(ImprovementArea::AcceptanceCriteria {
                    issue: "Council requires acceptance criteria improvements".to_string(),
                    missing_scenarios: Vec::new(),
                });
            }
            if reason.contains("TestPlan") || reason.contains("test_plan") || reason.contains("Testing") {
                improvement_areas.push(ImprovementArea::TestPlan {
                    issue: "Council requires test plan improvements".to_string(),
                    missing_tests: Vec::new(),
                    target_coverage: None,
                });
            }
            if reason.contains("Constraints") || reason.contains("constraints") {
                improvement_areas.push(ImprovementArea::Constraints {
                    issue: "Council requires constraint adjustments".to_string(),
                    constraint_type: ConstraintType::BudgetLimits,
                });
            }
            if reason.contains("Description") || reason.contains("description") {
                improvement_areas.push(ImprovementArea::Description {
                    issue: "Council requires description improvements".to_string(),
                    suggestion: "Add more detail and clarity".to_string(),
                });
            }
            if reason.contains("RiskAssessment") || reason.contains("risk") {
                improvement_areas.push(ImprovementArea::RiskAssessment {
                    issue: "Council requires risk assessment review".to_string(),
                    risk_tier_adjustment: None,
                });
            }
        }

        // If we found structured content, return the parsed directive
        if !improvement_areas.is_empty() || !acceptance_criteria.is_empty() {
            Some(ParsedRefinementDirective {
                improvement_areas,
                priority: RefinementPriority::High,
                max_iterations,
                acceptance_criteria,
            })
        } else {
            None
        }
    }

    /// Extract issue context from the reason string
    fn extract_issue_context(&self, reason: &str, area: &str) -> String {
        // Try to find the specific issue related to the area
        let area_lower = area.to_lowercase();
        
        // Look for sentences containing the area keyword
        for sentence in reason.split(['.', ';', '\n']) {
            if sentence.to_lowercase().contains(&area_lower) {
                return sentence.trim().to_string();
            }
        }
        
        // Fallback to the full reason if no specific context found
        format!("Issue with {}: {}", area, reason)
    }

    /// Extract missing scenarios from acceptance criteria feedback
    fn extract_missing_scenarios(&self, reason: &str) -> Vec<String> {
        let mut scenarios = Vec::new();
        let reason_lower = reason.to_lowercase();

        // Common missing scenario patterns
        if reason_lower.contains("error") || reason_lower.contains("failure") {
            scenarios.push("Error handling scenario".to_string());
        }
        if reason_lower.contains("edge") || reason_lower.contains("boundary") {
            scenarios.push("Edge case scenario".to_string());
        }
        if reason_lower.contains("concurrent") || reason_lower.contains("parallel") {
            scenarios.push("Concurrent execution scenario".to_string());
        }
        if reason_lower.contains("empty") || reason_lower.contains("null") {
            scenarios.push("Empty/null input scenario".to_string());
        }
        if reason_lower.contains("permission") || reason_lower.contains("auth") {
            scenarios.push("Authorization scenario".to_string());
        }

        scenarios
    }

    /// Extract missing tests from test plan feedback
    fn extract_missing_tests(&self, reason: &str) -> Vec<String> {
        let mut tests = Vec::new();
        let reason_lower = reason.to_lowercase();

        if reason_lower.contains("unit") {
            tests.push("Unit tests for core functionality".to_string());
        }
        if reason_lower.contains("integration") {
            tests.push("Integration tests for component interaction".to_string());
        }
        if reason_lower.contains("e2e") || reason_lower.contains("end-to-end") {
            tests.push("End-to-end tests for user workflows".to_string());
        }
        if reason_lower.contains("performance") {
            tests.push("Performance tests for SLA compliance".to_string());
        }
        if reason_lower.contains("security") {
            tests.push("Security tests for vulnerability detection".to_string());
        }

        tests
    }

    /// Apply refinements based on parsed directives
    pub fn apply_refinements(
        &self,
        spec: &WorkingSpec,
        directive: &ParsedRefinementDirective,
    ) -> RefinementResult {
        let mut refined_spec = spec.clone();
        let mut actions = Vec::new();
        let mut unresolved_issues = Vec::new();
        let mut quality_improvement: f64 = 0.0;

        for area in &directive.improvement_areas {
            match area {
                ImprovementArea::Description { issue, suggestion } => {
                    let result = self.refine_description(&mut refined_spec, issue, suggestion);
                    if result.successful {
                        quality_improvement += 0.05;
                    }
                    actions.push(result);
                }
                ImprovementArea::AcceptanceCriteria { issue, missing_scenarios } => {
                    let result = self.refine_acceptance_criteria(&mut refined_spec, issue, missing_scenarios);
                    if result.successful {
                        quality_improvement += 0.10;
                    }
                    actions.push(result);
                }
                ImprovementArea::Constraints { issue, constraint_type } => {
                    let result = self.refine_constraints(&mut refined_spec, issue, constraint_type);
                    if result.successful {
                        quality_improvement += 0.05;
                    }
                    actions.push(result);
                }
                ImprovementArea::TestPlan { issue, missing_tests, target_coverage } => {
                    let result = self.refine_test_plan(&mut refined_spec, issue, missing_tests, *target_coverage);
                    if result.successful {
                        quality_improvement += 0.15;
                    }
                    actions.push(result);
                }
                ImprovementArea::RiskAssessment { issue, risk_tier_adjustment } => {
                    let result = self.refine_risk_assessment(&mut refined_spec, issue, *risk_tier_adjustment);
                    if result.successful {
                        quality_improvement += 0.05;
                    }
                    actions.push(result);
                }
                ImprovementArea::Dependencies { issue, missing_dependencies } => {
                    let result = self.refine_dependencies(&mut refined_spec, issue, missing_dependencies);
                    if result.successful {
                        quality_improvement += 0.03;
                    }
                    actions.push(result);
                }
                ImprovementArea::Scope { issue, paths_to_add, paths_to_remove } => {
                    let result = self.refine_scope(&mut refined_spec, issue, paths_to_add, paths_to_remove);
                    if result.successful {
                        quality_improvement += 0.05;
                    }
                    actions.push(result);
                }
                ImprovementArea::Other { category, issue, suggestion } => {
                    // For generic improvements, add to unresolved for manual review
                    unresolved_issues.push(format!(
                        "[{}] {}: {}",
                        category, issue, suggestion
                    ));
                }
            }
        }

        // Update timestamp
        refined_spec.updated_at = chrono::Utc::now();

        RefinementResult {
            refined_spec,
            actions,
            unresolved_issues,
            estimated_quality_improvement: quality_improvement.min(0.5), // Cap at 50% improvement per pass
        }
    }

    /// Refine the description for clarity and completeness
    fn refine_description(
        &self,
        spec: &mut WorkingSpec,
        issue: &str,
        suggestion: &str,
    ) -> RefinementAction {
        let original_description = spec.description.clone();
        
        // Add context about the refinement
        if !spec.description.contains("Refined:") {
            spec.description = format!(
                "{}\n\n[Refined based on council feedback: {}]",
                spec.description.trim(),
                suggestion
            );
        }

        info!(
            "Refined description from '{}' based on issue: {}",
            original_description, issue
        );

        RefinementAction {
            area: "description".to_string(),
            description: format!("Enhanced description clarity: {}", suggestion),
            successful: true,
        }
    }

    /// Refine acceptance criteria based on feedback
    fn refine_acceptance_criteria(
        &self,
        spec: &mut WorkingSpec,
        issue: &str,
        missing_scenarios: &[String],
    ) -> RefinementAction {
        let mut added_criteria = Vec::new();

        // Add missing scenario-based acceptance criteria
        for (i, scenario) in missing_scenarios.iter().enumerate() {
            let id = format!("A{}", spec.acceptance_criteria.len() + i + 1);
            
            let (given, when, then) = self.generate_acceptance_criterion_parts(scenario, &spec.title);
            
            spec.acceptance_criteria.push(AcceptanceCriterion {
                id: id.clone(),
                given,
                when,
                then,
                priority: Some(MoSCoWPriority::Should),
            });
            
            added_criteria.push(id);
        }

        // Ensure minimum criteria based on risk tier
        let min_criteria = match spec.risk_tier {
            1 => 3,
            2 => 2,
            _ => 1,
        };

        while spec.acceptance_criteria.len() < min_criteria {
            let id = format!("A{}", spec.acceptance_criteria.len() + 1);
            spec.acceptance_criteria.push(AcceptanceCriterion {
                id: id.clone(),
                given: "Given the system is properly configured".to_string(),
                when: format!("When the {} functionality is executed", spec.title.to_lowercase()),
                then: "Then it behaves according to specifications".to_string(),
                priority: Some(MoSCoWPriority::Should),
            });
            added_criteria.push(id);
        }

        let description = if added_criteria.is_empty() {
            "Validated existing acceptance criteria".to_string()
        } else {
            format!("Added acceptance criteria: {}", added_criteria.join(", "))
        };

        info!("Refined acceptance criteria based on issue: {}", issue);

        RefinementAction {
            area: "acceptance_criteria".to_string(),
            description,
            successful: true,
        }
    }

    /// Generate acceptance criterion parts based on scenario type
    fn generate_acceptance_criterion_parts(&self, scenario: &str, title: &str) -> (String, String, String) {
        let scenario_lower = scenario.to_lowercase();
        
        if scenario_lower.contains("error") || scenario_lower.contains("failure") {
            (
                "Given the system encounters an error condition".to_string(),
                format!("When {} fails", title.to_lowercase()),
                "Then an appropriate error message is returned and the system remains stable".to_string(),
            )
        } else if scenario_lower.contains("edge") || scenario_lower.contains("boundary") {
            (
                "Given edge case input values".to_string(),
                format!("When {} processes boundary conditions", title.to_lowercase()),
                "Then it handles them gracefully without errors".to_string(),
            )
        } else if scenario_lower.contains("concurrent") || scenario_lower.contains("parallel") {
            (
                "Given multiple concurrent requests".to_string(),
                format!("When {} handles parallel execution", title.to_lowercase()),
                "Then all requests are processed correctly without race conditions".to_string(),
            )
        } else if scenario_lower.contains("empty") || scenario_lower.contains("null") {
            (
                "Given empty or null input".to_string(),
                format!("When {} receives invalid input", title.to_lowercase()),
                "Then it validates input and returns appropriate response".to_string(),
            )
        } else if scenario_lower.contains("auth") || scenario_lower.contains("permission") {
            (
                "Given a user with specific permissions".to_string(),
                format!("When they attempt to access {}", title.to_lowercase()),
                "Then access is granted or denied based on authorization rules".to_string(),
            )
        } else {
            (
                format!("Given the {} scenario", scenario),
                format!("When {} is executed", title.to_lowercase()),
                "Then the expected outcome is achieved".to_string(),
            )
        }
    }

    /// Refine constraints based on feedback
    fn refine_constraints(
        &self,
        spec: &mut WorkingSpec,
        issue: &str,
        constraint_type: &ConstraintType,
    ) -> RefinementAction {
        let mut description = String::new();

        match constraint_type {
            ConstraintType::MaxFiles => {
                if let Some(ref mut budget) = spec.constraints.budget_limits {
                    if budget.max_files.unwrap_or(0) == 0 {
                        budget.max_files = Some(25);
                        description = "Set max_files to 25 (reasonable default)".to_string();
                    }
                }
            }
            ConstraintType::MaxLoc => {
                if let Some(ref mut budget) = spec.constraints.budget_limits {
                    if budget.max_loc.unwrap_or(0) == 0 {
                        budget.max_loc = Some(1000);
                        description = "Set max_loc to 1000 (reasonable default)".to_string();
                    }
                }
            }
            ConstraintType::ScopeRestrictions => {
                // Validate scope restrictions
                if let Some(ref mut scope) = spec.constraints.scope_restrictions {
                    // Remove any paths that appear in both allowed and blocked
                    let blocked_set: std::collections::HashSet<_> = scope.blocked_paths.iter().cloned().collect();
                    scope.allowed_paths.retain(|p| !blocked_set.contains(p));
                    description = "Resolved conflicting scope restrictions".to_string();
                }
            }
            ConstraintType::BudgetLimits => {
                if spec.constraints.budget_limits.is_none() {
                    spec.constraints.budget_limits = Some(agent_agency_contracts::working_spec::BudgetLimits {
                        max_files: Some(25),
                        max_loc: Some(1000),
                    });
                    description = "Added default budget limits".to_string();
                }
            }
            ConstraintType::TimeConstraints => {
                description = "Time constraints reviewed".to_string();
            }
        }

        if description.is_empty() {
            description = format!("Reviewed {:?} constraints", constraint_type);
        }

        info!("Refined constraints based on issue: {}", issue);

        RefinementAction {
            area: "constraints".to_string(),
            description,
            successful: true,
        }
    }

    /// Refine test plan based on feedback
    fn refine_test_plan(
        &self,
        spec: &mut WorkingSpec,
        issue: &str,
        missing_tests: &[String],
        target_coverage: Option<f64>,
    ) -> RefinementAction {
        let mut changes = Vec::new();

        // Add missing unit tests
        for test_desc in missing_tests {
            if test_desc.to_lowercase().contains("unit") && spec.test_plan.unit_tests.is_empty() {
                spec.test_plan.unit_tests.push(UnitTestSpec {
                    description: "Core functionality tests".to_string(),
                    target_function: None,
                    test_cases: vec![
                        "valid_input".to_string(),
                        "invalid_input".to_string(),
                        "edge_cases".to_string(),
                    ],
                });
                changes.push("Added unit test specification");
            }
        }

        // Set coverage targets based on risk tier
        let coverage_target = target_coverage.unwrap_or_else(|| match spec.risk_tier {
            1 => self.config.t1_coverage_target,
            2 => self.config.t2_coverage_target,
            _ => self.config.t3_coverage_target,
        });

        if spec.test_plan.coverage_targets.is_none() {
            spec.test_plan.coverage_targets = Some(CoverageTargets {
                line_coverage: Some(coverage_target),
                branch_coverage: Some(coverage_target - 0.10),
                mutation_score: Some(if spec.risk_tier == 1 { 0.70 } else { 0.50 }),
            });
            changes.push("Added coverage targets");
        }

        // Ensure T1 tasks have unit tests
        if spec.risk_tier == 1 && spec.test_plan.unit_tests.is_empty() {
            spec.test_plan.unit_tests.push(UnitTestSpec {
                description: "Tier 1 required unit tests".to_string(),
                target_function: None,
                test_cases: vec![
                    "critical_path".to_string(),
                    "error_handling".to_string(),
                    "security_validation".to_string(),
                ],
            });
            changes.push("Added T1 required unit tests");
        }

        let description = if changes.is_empty() {
            "Validated existing test plan".to_string()
        } else {
            changes.join("; ")
        };

        info!("Refined test plan based on issue: {}", issue);

        RefinementAction {
            area: "test_plan".to_string(),
            description,
            successful: true,
        }
    }

    /// Refine risk assessment based on feedback
    fn refine_risk_assessment(
        &self,
        spec: &mut WorkingSpec,
        issue: &str,
        risk_tier_adjustment: Option<i32>,
    ) -> RefinementAction {
        let mut description = String::new();

        if let Some(adjustment) = risk_tier_adjustment {
            let new_tier = (spec.risk_tier as i32 + adjustment).clamp(1, 3) as u32;
            if new_tier != spec.risk_tier {
                spec.risk_tier = new_tier;
                description = format!("Adjusted risk tier to T{}", new_tier);
            }
        }

        // Ensure acceptance criteria match risk tier requirements
        let min_criteria = match spec.risk_tier {
            1 => 3,
            2 => 2,
            _ => 1,
        };

        if spec.acceptance_criteria.len() < min_criteria {
            description = format!(
                "Risk tier T{} requires at least {} acceptance criteria (current: {})",
                spec.risk_tier,
                min_criteria,
                spec.acceptance_criteria.len()
            );
        }

        if description.is_empty() {
            description = format!("Validated risk assessment for T{}", spec.risk_tier);
        }

        info!("Refined risk assessment based on issue: {}", issue);

        RefinementAction {
            area: "risk_assessment".to_string(),
            description,
            successful: true,
        }
    }

    /// Refine dependencies based on feedback
    fn refine_dependencies(
        &self,
        spec: &mut WorkingSpec,
        issue: &str,
        missing_dependencies: &[String],
    ) -> RefinementAction {
        for dep in missing_dependencies {
            if !spec.context.dependencies.contains_key(dep) {
                spec.context.dependencies.insert(dep.clone(), "*".to_string());
            }
        }

        // Fix empty version strings
        for (_, version) in spec.context.dependencies.iter_mut() {
            if version.trim().is_empty() {
                *version = "*".to_string();
            }
        }

        info!("Refined dependencies based on issue: {}", issue);

        RefinementAction {
            area: "dependencies".to_string(),
            description: format!("Validated dependencies (added {} new)", missing_dependencies.len()),
            successful: true,
        }
    }

    /// Refine scope based on feedback
    fn refine_scope(
        &self,
        spec: &mut WorkingSpec,
        issue: &str,
        paths_to_add: &[String],
        paths_to_remove: &[String],
    ) -> RefinementAction {
        if let Some(ref mut scope) = spec.constraints.scope_restrictions {
            // Add new allowed paths
            for path in paths_to_add {
                if !scope.allowed_paths.contains(path) {
                    scope.allowed_paths.push(path.clone());
                }
            }

            // Remove paths
            for path in paths_to_remove {
                scope.allowed_paths.retain(|p| p != path);
            }

            // Remove duplicates
            scope.allowed_paths.sort();
            scope.allowed_paths.dedup();
            scope.blocked_paths.sort();
            scope.blocked_paths.dedup();
        }

        info!("Refined scope based on issue: {}", issue);

        RefinementAction {
            area: "scope".to_string(),
            description: format!(
                "Updated scope: +{} paths, -{} paths",
                paths_to_add.len(),
                paths_to_remove.len()
            ),
            successful: true,
        }
    }

    /// Convert validation issues to improvement areas
    pub fn validation_issues_to_improvement_areas(
        &self,
        issues: &[ValidationIssue],
    ) -> Vec<ImprovementArea> {
        issues
            .iter()
            .filter_map(|issue| {
                let category_str = match &issue.category {
                    ValidationCategory::Enum(cat) => format!("{:?}", cat),
                    ValidationCategory::String(s) => s.clone(),
                };
                let category_lower = category_str.to_lowercase();

                if category_lower.contains("constraint") || category_lower.contains("budget") {
                    Some(ImprovementArea::Constraints {
                        issue: issue.description.clone(),
                        constraint_type: ConstraintType::BudgetLimits,
                    })
                } else if category_lower.contains("acceptance") || category_lower.contains("criteria") {
                    Some(ImprovementArea::AcceptanceCriteria {
                        issue: issue.description.clone(),
                        missing_scenarios: Vec::new(),
                    })
                } else if category_lower.contains("test") || category_lower.contains("coverage") {
                    Some(ImprovementArea::TestPlan {
                        issue: issue.description.clone(),
                        missing_tests: Vec::new(),
                        target_coverage: None,
                    })
                } else if category_lower.contains("risk") || category_lower.contains("tier") {
                    Some(ImprovementArea::RiskAssessment {
                        issue: issue.description.clone(),
                        risk_tier_adjustment: None,
                    })
                } else if category_lower.contains("scope") || category_lower.contains("path") {
                    Some(ImprovementArea::Scope {
                        issue: issue.description.clone(),
                        paths_to_add: Vec::new(),
                        paths_to_remove: Vec::new(),
                    })
                } else if category_lower.contains("depend") {
                    Some(ImprovementArea::Dependencies {
                        issue: issue.description.clone(),
                        missing_dependencies: Vec::new(),
                    })
                } else {
                    Some(ImprovementArea::Other {
                        category: category_str,
                        issue: issue.description.clone(),
                        suggestion: issue.suggestion.clone().unwrap_or_default(),
                    })
                }
            })
            .collect()
    }
}

impl Default for IntelligentSpecRefiner {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SpecRefiner for IntelligentSpecRefiner {
    async fn refine_working_spec(
        &self,
        current_spec: &WorkingSpec,
        refinement_reason: &str,
    ) -> Result<WorkingSpec> {
        info!(
            "Intelligently refining working spec '{}' based on feedback",
            current_spec.title
        );

        // Parse the council feedback into structured directives
        let directive = self.parse_council_feedback(refinement_reason);

        debug!(
            "Parsed {} improvement areas from council feedback",
            directive.improvement_areas.len()
        );

        // Apply refinements
        let result = self.apply_refinements(current_spec, &directive);

        // Log results
        for action in &result.actions {
            if action.successful {
                info!("[{}] {}", action.area, action.description);
            } else {
                warn!("[{}] Failed: {}", action.area, action.description);
            }
        }

        if !result.unresolved_issues.is_empty() {
            warn!(
                "Unresolved issues requiring manual review: {:?}",
                result.unresolved_issues
            );
        }

        info!(
            "Refinement complete. Estimated quality improvement: {:.1}%",
            result.estimated_quality_improvement * 100.0
        );

        Ok(result.refined_spec)
    }
}

/// Create a shared intelligent spec refiner
pub fn create_intelligent_spec_refiner() -> Arc<dyn SpecRefiner> {
    Arc::new(IntelligentSpecRefiner::new())
}

/// Create a shared intelligent spec refiner with custom configuration
pub fn create_intelligent_spec_refiner_with_config(
    config: IntelligentRefinerConfig,
) -> Arc<dyn SpecRefiner> {
    Arc::new(IntelligentSpecRefiner::with_config(config))
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_agency_contracts::working_spec::{
        BudgetLimits, ScopeRestrictions, WorkingSpecConstraints,
    };
    use agent_agency_contracts::planning_io::ChangeBudget;

    /// Create a minimal working spec for testing
    fn create_test_working_spec() -> WorkingSpec {
        WorkingSpec {
            version: "1.0.0".to_string(),
            id: "TEST-001".to_string(),
            title: "Test Task".to_string(),
            description: "A test task description".to_string(),
            goals: vec!["Complete the test task".to_string()],
            risk_tier: 2,
            constraints: WorkingSpecConstraints {
                max_duration_minutes: Some(60),
                max_iterations: Some(3),
                budget_limits: Some(BudgetLimits {
                    max_files: Some(10),
                    max_loc: Some(500),
                }),
                scope_restrictions: Some(ScopeRestrictions {
                    allowed_paths: vec!["src/".to_string()],
                    blocked_paths: vec!["node_modules/".to_string()],
                }),
            },
            acceptance_criteria: vec![AcceptanceCriterion {
                id: "A1".to_string(),
                given: "Given a valid input".to_string(),
                when: "When the task is executed".to_string(),
                then: "Then the expected output is produced".to_string(),
                priority: Some(MoSCoWPriority::Must),
            }],
            test_plan: agent_agency_contracts::working_spec::TestPlan {
                unit_tests: vec![],
                integration_tests: vec![],
                e2e_scenarios: vec![],
                coverage_targets: None,
            },
            rollback_plan: agent_agency_contracts::working_spec::RollbackPlan {
                strategy: agent_agency_contracts::working_spec::RollbackStrategy::GitRevert,
                automated_steps: vec![],
                manual_steps: vec![],
                data_impact: agent_agency_contracts::working_spec::DataImpact::None,
                downtime_required: Some(false),
                rollback_window_minutes: Some(30),
            },
            context: agent_agency_contracts::working_spec::WorkingSpecContext {
                workspace_root: ".".to_string(),
                git_branch: "main".to_string(),
                recent_changes: vec![],
                dependencies: std::collections::HashMap::new(),
                environment: agent_agency_contracts::task_request::Environment::Development,
            },
            non_functional_requirements: None,
            validation_results: None,
            quality_gates: None,
            scope: vec![],
            metadata: None,
            milestones: vec![],
            change_budget: ChangeBudget {
                max_files: 10,
                max_loc: 500,
                max_migrations: 0,
                allow_breaking_changes: false,
                allow_new_dependencies: true,
                enforcement_mode: agent_agency_contracts::planning_io::BudgetEnforcement::Warning,
            },
            file_changes: vec![],
            coverage_targets: None,
            overview: String::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_parse_council_feedback_description() {
        let refiner = IntelligentSpecRefiner::new();
        let directive = refiner.parse_council_feedback("The description is unclear and needs more detail");
        
        assert!(!directive.improvement_areas.is_empty());
        assert!(directive.improvement_areas.iter().any(|a| matches!(a, ImprovementArea::Description { .. })));
    }

    #[test]
    fn test_parse_council_feedback_acceptance_criteria() {
        let refiner = IntelligentSpecRefiner::new();
        let directive = refiner.parse_council_feedback("Missing acceptance criteria for error handling");
        
        assert!(!directive.improvement_areas.is_empty());
        assert!(directive.improvement_areas.iter().any(|a| matches!(a, ImprovementArea::AcceptanceCriteria { .. })));
    }

    #[test]
    fn test_parse_council_feedback_test_plan() {
        let refiner = IntelligentSpecRefiner::new();
        let directive = refiner.parse_council_feedback("Test coverage should be at least 80%");
        
        assert!(!directive.improvement_areas.is_empty());
        assert!(directive.improvement_areas.iter().any(|a| matches!(a, ImprovementArea::TestPlan { .. })));
    }

    #[test]
    fn test_parse_structured_refinement_directive() {
        let refiner = IntelligentSpecRefiner::new();
        let directive = refiner.parse_council_feedback(
            "Refinement required: RefinementDirective { required_changes: [AcceptanceCriteria], max_iterations: 5 }"
        );
        
        assert_eq!(directive.max_iterations, 5);
        assert!(!directive.improvement_areas.is_empty());
    }

    #[test]
    fn test_priority_detection() {
        let refiner = IntelligentSpecRefiner::new();
        
        let critical = refiner.parse_council_feedback("Critical security issue needs immediate attention");
        assert_eq!(critical.priority, RefinementPriority::Critical);
        
        let low = refiner.parse_council_feedback("Minor formatting issue, low priority");
        assert_eq!(low.priority, RefinementPriority::Low);
    }

    #[test]
    fn test_parse_council_feedback_constraints() {
        let refiner = IntelligentSpecRefiner::new();
        let directive = refiner.parse_council_feedback("Budget constraint max_files is too low");
        
        assert!(!directive.improvement_areas.is_empty());
        assert!(directive.improvement_areas.iter().any(|a| matches!(a, ImprovementArea::Constraints { .. })));
    }

    #[test]
    fn test_parse_council_feedback_risk() {
        let refiner = IntelligentSpecRefiner::new();
        let directive = refiner.parse_council_feedback("Risk tier should be elevated due to security concerns");
        
        assert!(!directive.improvement_areas.is_empty());
        assert!(directive.improvement_areas.iter().any(|a| matches!(a, ImprovementArea::RiskAssessment { .. })));
    }

    #[test]
    fn test_parse_council_feedback_scope() {
        let refiner = IntelligentSpecRefiner::new();
        let directive = refiner.parse_council_feedback("Scope paths need adjustment to include tests/");
        
        assert!(!directive.improvement_areas.is_empty());
        assert!(directive.improvement_areas.iter().any(|a| matches!(a, ImprovementArea::Scope { .. })));
    }

    #[test]
    fn test_parse_council_feedback_multiple_areas() {
        let refiner = IntelligentSpecRefiner::new();
        let directive = refiner.parse_council_feedback(
            "The description is unclear and test coverage should be 80%"
        );
        
        assert!(directive.improvement_areas.len() >= 2);
        assert!(directive.improvement_areas.iter().any(|a| matches!(a, ImprovementArea::Description { .. })));
        assert!(directive.improvement_areas.iter().any(|a| matches!(a, ImprovementArea::TestPlan { .. })));
    }

    #[test]
    fn test_apply_refinements_description() {
        let refiner = IntelligentSpecRefiner::new();
        let spec = create_test_working_spec();
        
        let directive = ParsedRefinementDirective {
            improvement_areas: vec![ImprovementArea::Description {
                issue: "Description too vague".to_string(),
                suggestion: "Add more implementation details".to_string(),
            }],
            priority: RefinementPriority::Medium,
            max_iterations: 3,
            acceptance_criteria: vec![],
        };
        
        let result = refiner.apply_refinements(&spec, &directive);
        
        assert!(!result.actions.is_empty());
        assert!(result.actions.iter().any(|a| a.area == "description" && a.successful));
        assert!(result.refined_spec.description.contains("Refined"));
    }

    #[test]
    fn test_apply_refinements_acceptance_criteria() {
        let refiner = IntelligentSpecRefiner::new();
        let mut spec = create_test_working_spec();
        spec.risk_tier = 1; // T1 requires at least 3 acceptance criteria
        
        let directive = ParsedRefinementDirective {
            improvement_areas: vec![ImprovementArea::AcceptanceCriteria {
                issue: "T1 tasks require at least 3 acceptance criteria".to_string(),
                missing_scenarios: vec!["Error handling scenario".to_string()],
            }],
            priority: RefinementPriority::High,
            max_iterations: 3,
            acceptance_criteria: vec![],
        };
        
        let result = refiner.apply_refinements(&spec, &directive);
        
        assert!(!result.actions.is_empty());
        assert!(result.actions.iter().any(|a| a.area == "acceptance_criteria" && a.successful));
        // Should have added criteria to meet T1 requirement
        assert!(result.refined_spec.acceptance_criteria.len() >= 3);
    }

    #[test]
    fn test_apply_refinements_test_plan() {
        let refiner = IntelligentSpecRefiner::new();
        let spec = create_test_working_spec();
        
        let directive = ParsedRefinementDirective {
            improvement_areas: vec![ImprovementArea::TestPlan {
                issue: "Missing unit tests".to_string(),
                missing_tests: vec!["Unit tests for core functionality".to_string()],
                target_coverage: Some(0.80),
            }],
            priority: RefinementPriority::Medium,
            max_iterations: 3,
            acceptance_criteria: vec![],
        };
        
        let result = refiner.apply_refinements(&spec, &directive);
        
        assert!(!result.actions.is_empty());
        assert!(result.actions.iter().any(|a| a.area == "test_plan" && a.successful));
        assert!(result.refined_spec.test_plan.coverage_targets.is_some());
    }

    #[test]
    fn test_apply_refinements_constraints() {
        let refiner = IntelligentSpecRefiner::new();
        let mut spec = create_test_working_spec();
        spec.constraints.budget_limits = Some(BudgetLimits {
            max_files: Some(0), // Invalid - should be fixed
            max_loc: Some(0),   // Invalid - should be fixed
        });
        
        let directive = ParsedRefinementDirective {
            improvement_areas: vec![ImprovementArea::Constraints {
                issue: "Budget limits are invalid".to_string(),
                constraint_type: ConstraintType::MaxFiles,
            }],
            priority: RefinementPriority::High,
            max_iterations: 3,
            acceptance_criteria: vec![],
        };
        
        let result = refiner.apply_refinements(&spec, &directive);
        
        assert!(!result.actions.is_empty());
        assert!(result.actions.iter().any(|a| a.area == "constraints" && a.successful));
    }

    #[test]
    fn test_quality_improvement_estimation() {
        let refiner = IntelligentSpecRefiner::new();
        let spec = create_test_working_spec();
        
        let directive = ParsedRefinementDirective {
            improvement_areas: vec![
                ImprovementArea::Description {
                    issue: "Description needs work".to_string(),
                    suggestion: "Add details".to_string(),
                },
                ImprovementArea::TestPlan {
                    issue: "Missing tests".to_string(),
                    missing_tests: vec![],
                    target_coverage: None,
                },
                ImprovementArea::AcceptanceCriteria {
                    issue: "Need more criteria".to_string(),
                    missing_scenarios: vec![],
                },
            ],
            priority: RefinementPriority::Medium,
            max_iterations: 3,
            acceptance_criteria: vec![],
        };
        
        let result = refiner.apply_refinements(&spec, &directive);
        
        // Quality improvement should be positive and capped at 0.5
        assert!(result.estimated_quality_improvement > 0.0);
        assert!(result.estimated_quality_improvement <= 0.5);
    }

    #[test]
    fn test_missing_scenario_extraction() {
        let refiner = IntelligentSpecRefiner::new();
        
        let scenarios = refiner.extract_missing_scenarios(
            "Missing error handling and edge cases for concurrent operations"
        );
        
        assert!(scenarios.iter().any(|s| s.to_lowercase().contains("error")));
        assert!(scenarios.iter().any(|s| s.to_lowercase().contains("edge")));
        assert!(scenarios.iter().any(|s| s.to_lowercase().contains("concurrent")));
    }

    #[test]
    fn test_missing_tests_extraction() {
        let refiner = IntelligentSpecRefiner::new();
        
        let tests = refiner.extract_missing_tests(
            "Need unit tests, integration tests, and performance tests"
        );
        
        assert!(tests.iter().any(|t| t.to_lowercase().contains("unit")));
        assert!(tests.iter().any(|t| t.to_lowercase().contains("integration")));
        assert!(tests.iter().any(|t| t.to_lowercase().contains("performance")));
    }

    #[test]
    fn test_generate_acceptance_criterion_parts() {
        let refiner = IntelligentSpecRefiner::new();
        
        let (given, when, then) = refiner.generate_acceptance_criterion_parts(
            "Error handling scenario",
            "Test Task"
        );
        
        assert!(given.to_lowercase().contains("error"));
        assert!(!when.is_empty());
        assert!(then.to_lowercase().contains("error") || then.to_lowercase().contains("stable"));
    }

    #[tokio::test]
    async fn test_spec_refiner_trait_implementation() {
        let refiner = IntelligentSpecRefiner::new();
        let spec = create_test_working_spec();
        
        let result = refiner
            .refine_working_spec(&spec, "The description is unclear")
            .await;
        
        assert!(result.is_ok());
        let refined = result.unwrap();
        assert!(refined.description.contains("Refined"));
    }

    #[test]
    fn test_default_config() {
        let config = IntelligentRefinerConfig::default();
        
        assert_eq!(config.t1_coverage_target, 0.90);
        assert_eq!(config.t2_coverage_target, 0.80);
        assert_eq!(config.t3_coverage_target, 0.70);
        assert!(config.max_changes_per_pass > 0);
    }

    #[test]
    fn test_custom_config() {
        let config = IntelligentRefinerConfig {
            verbose: true,
            max_changes_per_pass: 5,
            min_quality_improvement: 0.05,
            t1_coverage_target: 0.95,
            t2_coverage_target: 0.85,
            t3_coverage_target: 0.75,
        };
        
        let refiner = IntelligentSpecRefiner::with_config(config);
        assert!(refiner.config.verbose);
    }

    #[test]
    fn test_unresolved_issues() {
        let refiner = IntelligentSpecRefiner::new();
        let spec = create_test_working_spec();
        
        let directive = ParsedRefinementDirective {
            improvement_areas: vec![ImprovementArea::Other {
                category: "custom".to_string(),
                issue: "Some custom issue".to_string(),
                suggestion: "Manual review needed".to_string(),
            }],
            priority: RefinementPriority::Low,
            max_iterations: 3,
            acceptance_criteria: vec![],
        };
        
        let result = refiner.apply_refinements(&spec, &directive);
        
        // Other improvements should be added to unresolved issues
        assert!(!result.unresolved_issues.is_empty());
    }
}

