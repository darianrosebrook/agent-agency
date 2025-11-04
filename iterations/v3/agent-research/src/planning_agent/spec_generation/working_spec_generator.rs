//! Working Specification Generation
//!
//! This module handles the generation of WorkingSpec objects from task requests,
//! including goal extraction, acceptance criteria generation, and constraint creation.

use crate::planning_agent::planning_errors::{PlanningError, PlanningResult};
use chrono;

/// Generate a working specification from a task request
pub async fn generate_working_spec(
    task_request: &agent_agency_contracts::task_request::TaskRequest
) -> PlanningResult<agent_agency_contracts::working_spec::WorkingSpec> {
    let goals = extract_goals_from_description(&task_request.description)?;
    let acceptance_criteria = generate_acceptance_criteria(&task_request.description)?;
    let title = generate_title_from_description(&task_request.description);
    let constraints = create_working_spec_constraints(task_request)?;
    let test_plan = generate_test_plan(task_request)?;
    let rollback_plan = generate_rollback_plan(task_request)?;
    let context = create_working_spec_context(task_request)?;

    let now = chrono::Utc::now();

    Ok(agent_agency_contracts::working_spec::WorkingSpec {
        version: "1.0".to_string(),
        id: format!("task-{}", task_request.id),
        title,
        description: task_request.description.clone(),
        goals,
        risk_tier: 2, // Default to medium risk
        constraints,
        acceptance_criteria,
        test_plan,
        rollback_plan,
        context,
        non_functional_requirements: None,
        validation_results: None,
        quality_gates: None,
        scope: Vec::new(),
        metadata: None,
        milestones: Vec::new(),
        change_budget: agent_agency_contracts::planning_io::ChangeBudget {
            max_files: 25,
            max_loc: 1000,
            max_migrations: 0,
            allow_breaking_changes: false,
            allow_new_dependencies: false,
            enforcement_mode: agent_agency_contracts::planning_io::BudgetEnforcement::Strict,
        },
        file_changes: Vec::new(),
        coverage_targets: None,
        overview: String::new(),
        created_at: now,
        updated_at: now,
    })
}

/// Extract goals from task description
fn extract_goals_from_description(description: &str) -> PlanningResult<Vec<String>> {
    let mut goals = Vec::new();

    // Simple goal extraction based on keywords and structure
    let sentences: Vec<&str> = description.split(|c| c == '.' || c == '!' || c == '?').collect();

    for sentence in sentences {
        let sentence = sentence.trim();
        if sentence.is_empty() {
            continue;
        }

        // Look for goal indicators
        let goal_indicators = [
            "should", "must", "need to", "required to", "implement",
            "create", "build", "develop", "add", "support", "provide",
            "ensure", "verify", "validate", "test", "check"
        ];

        let sentence_lower = sentence.to_lowercase();
        if goal_indicators.iter().any(|&indicator| sentence_lower.contains(indicator)) {
            goals.push(sentence.to_string());
        }

        // Limit to reasonable number of goals
        if goals.len() >= 10 {
            break;
        }
    }

    if goals.is_empty() {
        goals.push(format!("Complete the requested task: {}", description));
    }

    Ok(goals)
}

/// Generate acceptance criteria from description
fn generate_acceptance_criteria(description: &str) -> PlanningResult<Vec<agent_agency_contracts::working_spec::AcceptanceCriterion>> {
    let mut criteria = Vec::new();

    // Generate basic acceptance criteria based on common patterns
    criteria.push(agent_agency_contracts::working_spec::AcceptanceCriterion {
        id: "A1".to_string(),
        given: "Task is executed".to_string(),
        when: "All preconditions are met".to_string(),
        then: "Task completes without errors".to_string(),
        priority: Some(agent_agency_contracts::working_spec::MoSCoWPriority::Must),
    });

    criteria.push(agent_agency_contracts::working_spec::AcceptanceCriterion {
        id: "A2".to_string(),
        given: "Task requirements are implemented".to_string(),
        when: "Task is executed".to_string(),
        then: "Functionality works as described in requirements".to_string(),
        priority: Some(agent_agency_contracts::working_spec::MoSCoWPriority::Must),
    });

    // Add domain-specific criteria
    if description.to_lowercase().contains("api") {
        criteria.push(agent_agency_contracts::working_spec::AcceptanceCriterion {
            id: "A3".to_string(),
            given: "API is called".to_string(),
            when: "Valid request is made".to_string(),
            then: "API endpoints return correct responses".to_string(),
            priority: Some(agent_agency_contracts::working_spec::MoSCoWPriority::Should),
        });
    }

    if description.to_lowercase().contains("database") {
        criteria.push(agent_agency_contracts::working_spec::AcceptanceCriterion {
            id: "A4".to_string(),
            given: "Database operations are performed".to_string(),
            when: "Valid data is provided".to_string(),
            then: "Database operations complete successfully".to_string(),
            priority: Some(agent_agency_contracts::working_spec::MoSCoWPriority::Should),
        });
    }

    Ok(criteria)
}

/// Generate a title from description
fn generate_title_from_description(description: &str) -> String {
    // Extract first sentence or first meaningful part
    let first_sentence = description.split(|c| c == '.' || c == '!' || c == '?')
        .next()
        .unwrap_or(description)
        .trim();

    // Limit length and capitalize
    let mut title = if first_sentence.len() > 80 {
        format!("{}...", &first_sentence[..77])
    } else {
        first_sentence.to_string()
    };

    // Capitalize first letter
    if let Some(first_char) = title.chars().next() {
        let capitalized = first_char.to_uppercase().collect::<String>() + &title[1..];
        title = capitalized;
    }

    title
}

/// Create working spec constraints
fn create_working_spec_constraints(task_request: &agent_agency_contracts::task_request::TaskRequest) -> PlanningResult<agent_agency_contracts::working_spec::WorkingSpecConstraints> {
    let mut max_duration_minutes = 60;
    let mut max_iterations = 10;

    // Adjust based on risk tier from constraints
    if let Some(constraints) = &task_request.constraints {
        match constraints.risk_tier {
            agent_agency_contracts::task_request::RiskTier::Tier1 => {
                max_duration_minutes = 30;
                max_iterations = 5;
            },
            agent_agency_contracts::task_request::RiskTier::Tier2 => {
                max_duration_minutes = 60;
                max_iterations = 10;
            },
            agent_agency_contracts::task_request::RiskTier::Tier3 => {
                max_duration_minutes = 120;
                max_iterations = 20;
            }
        }
    }

    Ok(agent_agency_contracts::working_spec::WorkingSpecConstraints {
        max_duration_minutes: Some(max_duration_minutes),
        max_iterations: Some(max_iterations),
        budget_limits: None, // Use default budget limits
        scope_restrictions: None, // No scope restrictions by default
    })
}

/// Generate test plan for task
fn generate_test_plan(task_request: &agent_agency_contracts::task_request::TaskRequest) -> PlanningResult<agent_agency_contracts::working_spec::TestPlan> {
    let unit_tests = vec![
        agent_agency_contracts::working_spec::UnitTestSpec {
            description: "Basic functionality test".to_string(),
            target_function: Some("main_function".to_string()),
            test_cases: vec!["// TODO: Implement basic functionality test".to_string()],
        },
        agent_agency_contracts::working_spec::UnitTestSpec {
            description: "Error handling test".to_string(),
            target_function: Some("error_handler".to_string()),
            test_cases: vec!["// TODO: Implement error handling test".to_string()],
        },
    ];

    let integration_tests = vec![
        agent_agency_contracts::working_spec::IntegrationTestSpec {
            description: "Component integration test".to_string(),
            components: vec!["component_a".to_string(), "component_b".to_string()],
            test_cases: vec!["Test component interaction".to_string()],
        },
    ];

    let coverage_targets = agent_agency_contracts::working_spec::CoverageTargets {
        line_coverage: Some(0.8),
        branch_coverage: Some(0.7),
        mutation_score: Some(0.6),
    };

    Ok(agent_agency_contracts::working_spec::TestPlan {
        unit_tests,
        integration_tests,
        e2e_scenarios: vec![], // No E2E scenarios by default
        coverage_targets: Some(coverage_targets),
    })
}

/// Generate rollback plan
fn generate_rollback_plan(task_request: &agent_agency_contracts::task_request::TaskRequest) -> PlanningResult<agent_agency_contracts::working_spec::RollbackPlan> {
    Ok(agent_agency_contracts::working_spec::RollbackPlan {
        strategy: agent_agency_contracts::working_spec::RollbackStrategy::GitRevert,
        automated_steps: vec![
            "Revert code changes to previous commit".to_string(),
            "Verify system stability".to_string(),
        ],
        manual_steps: vec![
            "Review rollback impact on data".to_string(),
            "Verify business logic integrity".to_string(),
        ],
        data_impact: agent_agency_contracts::working_spec::DataImpact::NoImpact, // Default to no impact
        downtime_required: Some(false),
        rollback_window_minutes: Some(30),
    })
}

/// Create working spec context
fn create_working_spec_context(task_request: &agent_agency_contracts::task_request::TaskRequest) -> PlanningResult<agent_agency_contracts::working_spec::WorkingSpecContext> {
    Ok(agent_agency_contracts::working_spec::WorkingSpecContext {
        workspace_root: ".".to_string(),
        git_branch: "main".to_string(),
        recent_changes: vec![], // No recent changes by default
        dependencies: std::collections::HashMap::new(), // Empty dependencies map
        environment: agent_agency_contracts::working_spec::Environment::Development,
    })
}
