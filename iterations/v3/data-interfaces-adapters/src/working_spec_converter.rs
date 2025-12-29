//! WorkingSpec Converter
//!
//! Converts API task submission requests to WorkingSpec format
//! for use with UnifiedOrchestrator.
//!
//! @author @darianrosebrook

use agent_agency_contracts::working_spec::BudgetLimits;
use agent_agency_contracts::{
    AcceptanceCriterion, RollbackPlan, TestPlan, WorkingSpec, WorkingSpecConstraints,
    WorkingSpecContext,
};
use chrono::Utc;
use data_infrastructure::api::types::TaskSubmissionRequest;
use uuid::Uuid;

/// Convert TaskSubmissionRequest to WorkingSpec
///
/// Creates a basic WorkingSpec from an API request. For more sophisticated
/// spec generation, use ResearchServiceAdapter.generate_working_spec().
pub fn convert_task_request_to_working_spec(
    request: TaskSubmissionRequest,
) -> Result<WorkingSpec, String> {
    // Generate a unique ID for this working spec using UUID
    // Format: TASK-<UUID> so we can extract the UUID later
    let task_uuid = Uuid::new_v4();
    let spec_id = format!("TASK-{}", task_uuid);

    // Extract title from description (first line or first 50 chars)
    let title = request
        .description
        .lines()
        .next()
        .unwrap_or(&request.description)
        .chars()
        .take(50)
        .collect::<String>()
        .trim()
        .to_string();

    // Parse risk tier (default to Tier 2 if not specified)
    let risk_tier = request
        .risk_tier
        .as_ref()
        .and_then(|rt| rt.parse::<u32>().ok())
        .filter(|rt| (1..=3).contains(rt))
        .unwrap_or(2);

    // Create basic acceptance criteria from description
    let acceptance_criteria = vec![AcceptanceCriterion {
        id: "A1".to_string(),
        given: "Task is submitted".to_string(),
        when: "Execution completes".to_string(),
        then: format!("Task '{}' is completed successfully", title),
        priority: None,
    }];

    // Create default test plan
    let test_plan = TestPlan {
        unit_tests: vec![],
        integration_tests: vec![],
        e2e_scenarios: vec![],
        coverage_targets: None,
    };

    // Create default rollback plan
    let rollback_plan = RollbackPlan {
        strategy: agent_agency_contracts::working_spec::RollbackStrategy::GitRevert,
        automated_steps: vec!["Revert all file changes".to_string()],
        manual_steps: vec!["Verify original state restored".to_string()],
        data_impact: agent_agency_contracts::working_spec::DataImpact::None,
        downtime_required: Some(false),
        rollback_window_minutes: Some(5),
    };

    // Create default context (can be enhanced with actual workspace detection)
    let context = WorkingSpecContext {
        workspace_root: std::env::current_dir()
            .ok()
            .and_then(|p| p.to_str().map(|s| s.to_string()))
            .unwrap_or_else(|| ".".to_string()),
        git_branch: "main".to_string(),
        recent_changes: vec![],
        dependencies: std::collections::HashMap::new(),
        environment: agent_agency_contracts::task_request::Environment::Development,
    };

    // Create default constraints with reasonable budgets
    let constraints = WorkingSpecConstraints {
        max_duration_minutes: Some(60), // Default 1 hour
        max_iterations: Some(3),        // Default 3 refinement iterations
        budget_limits: Some(BudgetLimits {
            max_files: Some(25), // Default CAWS budget
            max_loc: Some(1000), // Default CAWS budget
        }),
        scope_restrictions: None, // No restrictions by default
    };

    // Create default change budget
    let change_budget = agent_agency_contracts::planning_io::ChangeBudget {
        max_files: 25,
        max_loc: 1000,
        max_migrations: 0,
        allow_breaking_changes: false,
        allow_new_dependencies: false,
        enforcement_mode: agent_agency_contracts::planning_io::BudgetEnforcement::Strict,
    };

    // Extract goals from description (split by sentences or newlines)
    let goals: Vec<String> = request
        .description
        .lines()
        .filter(|line| !line.trim().is_empty())
        .take(5) // Limit to 5 goals
        .map(|line| line.trim().to_string())
        .collect();

    // If no goals extracted, create one from title
    let goals = if goals.is_empty() {
        vec![format!("Complete task: {}", title)]
    } else {
        goals
    };

    Ok(WorkingSpec {
        version: "1.0".to_string(),
        id: spec_id,
        title,
        description: request.description.clone(),
        goals,
        risk_tier,
        constraints,
        acceptance_criteria,
        test_plan,
        rollback_plan,
        context,
        non_functional_requirements: None,
        validation_results: None,
        quality_gates: None,
        scope: vec![],
        metadata: Some(agent_agency_contracts::WorkingSpecMetadata {
            created_at: Utc::now(),
            created_by: Some("api-server".to_string()),
            last_modified: Some(Utc::now()),
            version: Some(1),
            tags: vec![],
        }),
        milestones: vec![],
        change_budget,
        file_changes: vec![],
        coverage_targets: None,
        overview: request
            .context
            .unwrap_or_else(|| request.description.clone()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_basic_request() {
        let request = TaskSubmissionRequest {
            description: "Add user authentication feature".to_string(),
            execution_mode: Some("auto".to_string()),
            risk_tier: Some("2".to_string()),
            context: Some("High priority feature".to_string()),
            priority: Some("high".to_string()),
            deadline: None,
        };

        let spec = convert_task_request_to_working_spec(request).unwrap();

        assert_eq!(spec.risk_tier, 2);
        assert!(spec.title.contains("Add user authentication"));
        assert!(!spec.id.is_empty());
        assert!(!spec.acceptance_criteria.is_empty());
    }

    #[test]
    fn test_convert_with_default_risk_tier() {
        let request = TaskSubmissionRequest {
            description: "Simple task".to_string(),
            execution_mode: None,
            risk_tier: None,
            context: None,
            priority: None,
            deadline: None,
        };

        let spec = convert_task_request_to_working_spec(request).unwrap();

        // Should default to Tier 2
        assert_eq!(spec.risk_tier, 2);
    }
}
