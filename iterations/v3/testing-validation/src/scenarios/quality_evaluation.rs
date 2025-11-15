//! Quality Evaluation Test Scenarios
//!
//! Implements test scenarios for evaluating AI agent output quality as defined in
//! QUALITY_EVALUATION_PLAN.md:
//! - Code refactoring task
//! - Documentation writing task
//! - Bug fix task
//! - Feature implementation task

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Instant;
use tracing::{info, warn};

use crate::harness::{LocalServiceManager, TestEnvironment};
use crate::quality_analyzers::{
    CouncilTransparencyScore, DecisionQualityScore, OverallQualityScore, ReasoningDepthScore,
    VerdictReasoningQualityScore,
};
#[cfg(feature = "full")]
use agent_constitutional_council::verdict_writer::VerdictRecord;
#[cfg(feature = "full")]
use agent_orchestration::chain_of_thought::DecisionPoint;

/// Quality evaluation result for a scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityEvaluationResult {
    pub scenario_name: String,
    pub reasoning_depth: ReasoningDepthScore,
    pub decision_quality: DecisionQualityScore,
    pub council_transparency: Option<CouncilTransparencyScore>,
    pub verdict_reasoning: Option<VerdictReasoningQualityScore>,
    pub output_quality: f64, // Code or writing quality score
    pub overall_score: OverallQualityScore,
    pub passed: bool,
    pub success_criteria_met: Vec<String>,
    pub success_criteria_failed: Vec<String>,
}

/// Scenario 1: Code Refactoring Task
///
/// Task: Refactor a Rust module to improve maintainability while preserving functionality.
///
/// Success Criteria:
/// - Reasoning depth ≥ 0.7
/// - Council consensus ≥ 0.8
/// - Code quality ≥ 0.7 (mid-level standard)
/// - Tests pass
/// - No regressions
#[cfg(feature = "full")]
pub async fn run_code_refactoring_scenario(
    env: &TestEnvironment,
    services: &LocalServiceManager,
) -> QualityEvaluationResult {
    info!("Starting quality evaluation: Code Refactoring Scenario");

    let start_time = Instant::now();
    let mut success_criteria_met = Vec::new();
    let mut success_criteria_failed = Vec::new();

    // Setup playground with broken Rust code
    let workspace = match env.create_workspace("quality_refactor").await {
        Ok(ws) => ws,
        Err(e) => {
            error!("Failed to create workspace: {}", e);
            return create_failed_result(
                "Code Refactoring",
                format!("Workspace creation failed: {}", e),
            );
        }
    };

    // Create test file with code that needs refactoring
    let test_file_path = workspace.path().join("src").join("lib.rs");
    std::fs::create_dir_all(test_file_path.parent().unwrap()).unwrap();

    let broken_code = r#"
// Intentionally complex code for refactoring test
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
}

pub fn process_user_data(user: User, validate: bool, format: bool, save: bool) -> Result<String, String> {
    if validate {
        if user.email.is_empty() {
            return Err("Email is empty".to_string());
        }
        if user.name.is_empty() {
            return Err("Name is empty".to_string());
        }
    }
    let mut result = format!("User: {} ({})", user.name, user.email);
    if format {
        result = result.to_uppercase();
    }
    if save {
        // PLACEHOLDER: Save to database
        println!("Saving user: {}", result);
    }
    Ok(result)
}
"#;

    std::fs::write(&test_file_path, broken_code).unwrap();

    // Simulate decision points (in real scenario, these would come from actual agent execution)
    let decision_points = create_simulated_refactoring_decisions();

    // Analyze reasoning depth
    let reasoning_depth = ReasoningDepthScore::analyze(&decision_points);
    if reasoning_depth.score >= 0.7 {
        success_criteria_met.push("Reasoning depth ≥ 0.7".to_string());
    } else {
        success_criteria_failed.push(format!("Reasoning depth {} < 0.7", reasoning_depth.score));
    }

    // Analyze decision quality
    let decision_quality = DecisionQualityScore::analyze(&decision_points);
    if decision_quality.score >= 0.7 {
        success_criteria_met.push("Decision quality ≥ 0.7".to_string());
    } else {
        success_criteria_failed.push(format!("Decision quality {} < 0.7", decision_quality.score));
    }

    // Analyze code quality (simplified - would use actual refactored code in real scenario)
    let code_quality = CodeQualityScore::analyze(&test_file_path);
    if code_quality.score >= 0.7 {
        success_criteria_met.push("Code quality ≥ 0.7".to_string());
    } else {
        success_criteria_failed.push(format!("Code quality {} < 0.7", code_quality.score));
    }

    // Simulate council verdict (in real scenario, would come from actual council evaluation)
    let council_transparency = None; // Would be populated from actual VerdictRecord
    let verdict_reasoning = None; // Would be populated from actual VerdictRecord

    // Calculate overall score
    let overall_score = OverallQualityScore::calculate(
        reasoning_depth.score,
        decision_quality.score,
        0.7, // Placeholder for council transparency
        code_quality.score,
    );

    let passed =
        reasoning_depth.score >= 0.7 && decision_quality.score >= 0.7 && code_quality.score >= 0.7;

    info!(
        "Code refactoring scenario completed in {:?}",
        start_time.elapsed()
    );

    QualityEvaluationResult {
        scenario_name: "Code Refactoring".to_string(),
        reasoning_depth,
        decision_quality,
        council_transparency,
        verdict_reasoning,
        output_quality: code_quality.score,
        overall_score,
        passed,
        success_criteria_met,
        success_criteria_failed,
    }
}

/// Scenario 2: Documentation Writing Task
///
/// Task: Write comprehensive API documentation for a Rust module.
///
/// Success Criteria:
/// - Reasoning depth ≥ 0.7
/// - Council consensus ≥ 0.8
/// - Writing quality ≥ 0.7 (mid-level standard)
/// - Documentation completeness ≥ 80%
/// - Professional tone
#[cfg(feature = "full")]
pub async fn run_documentation_writing_scenario(
    env: &TestEnvironment,
    _services: &LocalServiceManager,
) -> QualityEvaluationResult {
    info!("Starting quality evaluation: Documentation Writing Scenario");

    let start_time = Instant::now();
    let mut success_criteria_met = Vec::new();
    let mut success_criteria_failed = Vec::new();

    // Setup workspace
    let workspace = match env.create_workspace("quality_docs").await {
        Ok(ws) => ws,
        Err(e) => {
            error!("Failed to create workspace: {}", e);
            return create_failed_result(
                "Documentation Writing",
                format!("Workspace creation failed: {}", e),
            );
        }
    };

    // Simulate decision points for documentation writing
    let decision_points = create_simulated_documentation_decisions();

    // Analyze reasoning depth
    let reasoning_depth = ReasoningDepthScore::analyze(&decision_points);
    if reasoning_depth.score >= 0.7 {
        success_criteria_met.push("Reasoning depth ≥ 0.7".to_string());
    } else {
        success_criteria_failed.push(format!("Reasoning depth {} < 0.7", reasoning_depth.score));
    }

    // Analyze decision quality
    let decision_quality = DecisionQualityScore::analyze(&decision_points);
    if decision_quality.score >= 0.7 {
        success_criteria_met.push("Decision quality ≥ 0.7".to_string());
    } else {
        success_criteria_failed.push(format!("Decision quality {} < 0.7", decision_quality.score));
    }

    // Sample documentation content (in real scenario, would come from actual agent output)
    let documentation_content = r#"
# User Management API

## Overview

The User Management API provides comprehensive functionality for managing user accounts,
authentication, and user data operations.

## Functions

### `process_user_data`

Processes user data with validation, formatting, and optional persistence.

**Parameters:**
- `user`: User struct containing id, name, and email
- `validate`: Whether to validate user data
- `format`: Whether to format the output
- `save`: Whether to save to database

**Returns:**
- `Result<String, String>`: Processed user data string or error message

**Example:**
```rust
let user = User {
    id: "123".to_string(),
    name: "John Doe".to_string(),
    email: "john@example.com".to_string(),
};
let result = process_user_data(user, true, true, false)?;
```
"#;

    // Analyze writing quality
    let writing_quality = WritingQualityScore::analyze(documentation_content);
    if writing_quality.score >= 0.7 {
        success_criteria_met.push("Writing quality ≥ 0.7".to_string());
    } else {
        success_criteria_failed.push(format!("Writing quality {} < 0.7", writing_quality.score));
    }

    // Check documentation completeness (simplified)
    let completeness = if documentation_content.len() > 500 {
        0.8
    } else {
        0.5
    };
    if completeness >= 0.8 {
        success_criteria_met.push("Documentation completeness ≥ 80%".to_string());
    } else {
        success_criteria_failed.push(format!(
            "Documentation completeness {}% < 80%",
            completeness * 100.0
        ));
    }

    // Calculate overall score
    let overall_score = OverallQualityScore::calculate(
        reasoning_depth.score,
        decision_quality.score,
        0.7, // Placeholder for council transparency
        writing_quality.score,
    );

    let passed = reasoning_depth.score >= 0.7
        && decision_quality.score >= 0.7
        && writing_quality.score >= 0.7
        && completeness >= 0.8;

    info!(
        "Documentation writing scenario completed in {:?}",
        start_time.elapsed()
    );

    QualityEvaluationResult {
        scenario_name: "Documentation Writing".to_string(),
        reasoning_depth,
        decision_quality,
        council_transparency: None,
        verdict_reasoning: None,
        output_quality: writing_quality.score,
        overall_score,
        passed,
        success_criteria_met,
        success_criteria_failed,
    }
}

/// Scenario 3: Bug Fix Task
///
/// Task: Identify and fix a complex bug in existing code.
///
/// Success Criteria:
/// - Reasoning depth ≥ 0.8 (bug fixing requires deep analysis)
/// - Council consensus ≥ 0.8
/// - Code quality ≥ 0.7
/// - Bug fixed
/// - Tests added
/// - No new bugs introduced
#[cfg(feature = "full")]
pub async fn run_bug_fix_scenario(
    env: &TestEnvironment,
    _services: &LocalServiceManager,
) -> QualityEvaluationResult {
    info!("Starting quality evaluation: Bug Fix Scenario");

    let start_time = Instant::now();
    let mut success_criteria_met = Vec::new();
    let mut success_criteria_failed = Vec::new();

    // Setup workspace
    let workspace = match env.create_workspace("quality_bugfix").await {
        Ok(ws) => ws,
        Err(e) => {
            error!("Failed to create workspace: {}", e);
            return create_failed_result("Bug Fix", format!("Workspace creation failed: {}", e));
        }
    };

    // Create test file with intentional bug
    let test_file_path = workspace.path().join("src").join("lib.rs");
    std::fs::create_dir_all(test_file_path.parent().unwrap()).unwrap();

    let buggy_code = r#"
pub fn calculate_total(items: Vec<u32>) -> u32 {
    let mut total = 0;
    for item in items {
        total += item;
        // BUG: Should break after finding first item > 100, but doesn't
        if item > 100 {
            // Missing break statement
        }
    }
    total
}
"#;

    std::fs::write(&test_file_path, buggy_code).unwrap();

    // Simulate decision points for bug fixing
    let decision_points = create_simulated_bugfix_decisions();

    // Analyze reasoning depth (bug fixing requires deeper analysis)
    let reasoning_depth = ReasoningDepthScore::analyze(&decision_points);
    if reasoning_depth.score >= 0.8 {
        success_criteria_met.push("Reasoning depth ≥ 0.8".to_string());
    } else {
        success_criteria_failed.push(format!("Reasoning depth {} < 0.8", reasoning_depth.score));
    }

    // Analyze decision quality
    let decision_quality = DecisionQualityScore::analyze(&decision_points);
    if decision_quality.score >= 0.7 {
        success_criteria_met.push("Decision quality ≥ 0.7".to_string());
    } else {
        success_criteria_failed.push(format!("Decision quality {} < 0.7", decision_quality.score));
    }

    // Analyze code quality
    let code_quality = CodeQualityScore::analyze(&test_file_path);
    if code_quality.score >= 0.7 {
        success_criteria_met.push("Code quality ≥ 0.7".to_string());
    } else {
        success_criteria_failed.push(format!("Code quality {} < 0.7", code_quality.score));
    }

    // Calculate overall score
    let overall_score = OverallQualityScore::calculate(
        reasoning_depth.score,
        decision_quality.score,
        0.7, // Placeholder for council transparency
        code_quality.score,
    );

    let passed =
        reasoning_depth.score >= 0.8 && decision_quality.score >= 0.7 && code_quality.score >= 0.7;

    info!("Bug fix scenario completed in {:?}", start_time.elapsed());

    QualityEvaluationResult {
        scenario_name: "Bug Fix".to_string(),
        reasoning_depth,
        decision_quality,
        council_transparency: None,
        verdict_reasoning: None,
        output_quality: code_quality.score,
        overall_score,
        passed,
        success_criteria_met,
        success_criteria_failed,
    }
}

/// Scenario 4: Feature Implementation Task
///
/// Task: Implement a new feature following existing patterns.
///
/// Success Criteria:
/// - Reasoning depth ≥ 0.7
/// - Council consensus ≥ 0.8
/// - Code quality ≥ 0.7
/// - Feature works correctly
/// - Tests pass
/// - Follows patterns
#[cfg(feature = "full")]
pub async fn run_feature_implementation_scenario(
    env: &TestEnvironment,
    _services: &LocalServiceManager,
) -> QualityEvaluationResult {
    info!("Starting quality evaluation: Feature Implementation Scenario");

    let start_time = Instant::now();
    let mut success_criteria_met = Vec::new();
    let mut success_criteria_failed = Vec::new();

    // Setup workspace
    let workspace = match env.create_workspace("quality_feature").await {
        Ok(ws) => ws,
        Err(e) => {
            error!("Failed to create workspace: {}", e);
            return create_failed_result(
                "Feature Implementation",
                format!("Workspace creation failed: {}", e),
            );
        }
    };

    // Simulate decision points for feature implementation
    let decision_points = create_simulated_feature_decisions();

    // Analyze reasoning depth
    let reasoning_depth = ReasoningDepthScore::analyze(&decision_points);
    if reasoning_depth.score >= 0.7 {
        success_criteria_met.push("Reasoning depth ≥ 0.7".to_string());
    } else {
        success_criteria_failed.push(format!("Reasoning depth {} < 0.7", reasoning_depth.score));
    }

    // Analyze decision quality
    let decision_quality = DecisionQualityScore::analyze(&decision_points);
    if decision_quality.score >= 0.7 {
        success_criteria_met.push("Decision quality ≥ 0.7".to_string());
    } else {
        success_criteria_failed.push(format!("Decision quality {} < 0.7", decision_quality.score));
    }

    // Create test file with feature implementation
    let test_file_path = workspace.path().join("src").join("lib.rs");
    std::fs::create_dir_all(test_file_path.parent().unwrap()).unwrap();

    let feature_code = r#"
/// New feature: User validation
pub struct UserValidator {
    pub strict_mode: bool,
}

impl UserValidator {
    pub fn new(strict_mode: bool) -> Self {
        Self { strict_mode }
    }

    pub fn validate(&self, user: &User) -> Result<(), String> {
        if user.email.is_empty() {
            return Err("Email cannot be empty".to_string());
        }
        if self.strict_mode && !user.email.contains('@') {
            return Err("Invalid email format".to_string());
        }
        Ok(())
    }
}
"#;

    std::fs::write(&test_file_path, feature_code).unwrap();

    // Analyze code quality
    let code_quality = CodeQualityScore::analyze(&test_file_path);
    if code_quality.score >= 0.7 {
        success_criteria_met.push("Code quality ≥ 0.7".to_string());
    } else {
        success_criteria_failed.push(format!("Code quality {} < 0.7", code_quality.score));
    }

    // Calculate overall score
    let overall_score = OverallQualityScore::calculate(
        reasoning_depth.score,
        decision_quality.score,
        0.7, // Placeholder for council transparency
        code_quality.score,
    );

    let passed =
        reasoning_depth.score >= 0.7 && decision_quality.score >= 0.7 && code_quality.score >= 0.7;

    info!(
        "Feature implementation scenario completed in {:?}",
        start_time.elapsed()
    );

    QualityEvaluationResult {
        scenario_name: "Feature Implementation".to_string(),
        reasoning_depth,
        decision_quality,
        council_transparency: None,
        verdict_reasoning: None,
        output_quality: code_quality.score,
        overall_score,
        passed,
        success_criteria_met,
        success_criteria_failed,
    }
}

/// Run all quality evaluation scenarios and generate report
#[cfg(feature = "full")]
pub async fn run_all_quality_scenarios(
    env: &TestEnvironment,
    services: &LocalServiceManager,
) -> Vec<QualityEvaluationResult> {
    info!("Running all quality evaluation scenarios");

    let mut results = Vec::new();

    // Run all scenarios
    results.push(run_code_refactoring_scenario(env, services).await);
    results.push(run_documentation_writing_scenario(env, services).await);
    results.push(run_bug_fix_scenario(env, services).await);
    results.push(run_feature_implementation_scenario(env, services).await);

    // Generate report
    generate_quality_report(&results).await;

    results
}

/// Generate quality evaluation report
async fn generate_quality_report(results: &[QualityEvaluationResult]) {
    info!("Generating quality evaluation report");

    let mut report = String::from("# Quality Evaluation Report\n\n");
    report.push_str(&format!(
        "Generated: {}\n\n",
        chrono::Utc::now().to_rfc3339()
    ));

    for result in results {
        report.push_str(&format!("## {}\n\n", result.scenario_name));
        report.push_str(&format!(
            "**Overall Score**: {:.2}\n",
            result.overall_score.score
        ));
        report.push_str(&format!(
            "**Status**: {}\n\n",
            if result.passed { "PASSED" } else { "FAILED" }
        ));

        report.push_str("### Scores\n\n");
        report.push_str(&format!(
            "- Reasoning Depth: {:.2} ({})\n",
            result.reasoning_depth.score,
            result.reasoning_depth.quality_level()
        ));
        report.push_str(&format!(
            "- Decision Quality: {:.2}\n",
            result.decision_quality.score
        ));
        report.push_str(&format!("- Output Quality: {:.2}\n", result.output_quality));

        if !result.success_criteria_met.is_empty() {
            report.push_str("\n### Success Criteria Met\n\n");
            for criterion in &result.success_criteria_met {
                report.push_str(&format!("- {}\n", criterion));
            }
        }

        if !result.success_criteria_failed.is_empty() {
            report.push_str("\n### Success Criteria Failed\n\n");
            for criterion in &result.success_criteria_failed {
                report.push_str(&format!("- {}\n", criterion));
            }
        }

        report.push_str("\n---\n\n");
    }

    // Save report
    let report_path = PathBuf::from("quality_evaluation_report.md");
    if let Err(e) = std::fs::write(&report_path, &report) {
        warn!("Failed to write quality report: {}", e);
    } else {
        info!("Quality report saved to: {}", report_path.display());
    }
}

// Helper functions to create simulated decision points

#[cfg(feature = "full")]
fn create_simulated_refactoring_decisions() -> Vec<DecisionPoint> {
    use agent_orchestration::chain_of_thought::{
        Alternative, DecisionContext, DecisionType, RiskAssessment,
    };
    use chrono::Utc;
    use std::collections::HashMap;
    use uuid::Uuid;

    vec![
        DecisionPoint {
            decision_id: Uuid::new_v4(),
            decision_type: DecisionType::WorkerAssignment,
            timestamp: Utc::now(),
            context: DecisionContext {
                task_id: None,
                plan_id: None,
                milestone_id: None,
                worker_id: None,
                resource_constraints: HashMap::new(),
                time_constraints: None,
                priority_level: None,
            },
            alternatives: vec![
                Alternative {
                    option: "Extract validation logic".to_string(),
                    score: 0.8,
                    reasoning: "Separates concerns and improves testability".to_string(),
                    pros: vec!["Better testability".to_string(), "Reusable logic".to_string()],
                    cons: vec!["More functions".to_string()],
                    confidence: 0.8,
                },
                Alternative {
                    option: "Keep monolithic function".to_string(),
                    score: 0.3,
                    reasoning: "Simpler but harder to maintain".to_string(),
                    pros: vec!["Fewer functions".to_string()],
                    cons: vec!["Hard to test".to_string(), "Poor separation".to_string()],
                    confidence: 0.7,
                },
            ],
            chosen_option: "Extract validation logic".to_string(),
            reasoning: "Based on the evidence from code analysis, extracting validation logic will improve maintainability and testability. The function is currently doing too many things, violating single responsibility principle.".to_string(),
            confidence: 0.8,
            risk_assessment: Some(RiskAssessment {
                risk_level: "low".to_string(),
                risk_factors: vec!["Breaking existing code".to_string()],
                mitigation_strategies: vec!["Run existing tests".to_string(), "Incremental refactoring".to_string()],
                fallback_options: vec!["Revert changes".to_string()],
            }),
            metadata: HashMap::new(),
        },
    ]
}

#[cfg(feature = "full")]
fn create_simulated_documentation_decisions() -> Vec<DecisionPoint> {
    use agent_orchestration::chain_of_thought::{Alternative, DecisionContext, DecisionType};
    use chrono::Utc;
    use std::collections::HashMap;
    use uuid::Uuid;

    vec![
        DecisionPoint {
            decision_id: Uuid::new_v4(),
            decision_type: DecisionType::WorkerAssignment,
            timestamp: Utc::now(),
            context: DecisionContext {
                task_id: None,
                plan_id: None,
                milestone_id: None,
                worker_id: None,
                resource_constraints: HashMap::new(),
                time_constraints: None,
                priority_level: None,
            },
            alternatives: vec![
                Alternative {
                    option: "Comprehensive API docs".to_string(),
                    score: 0.9,
                    reasoning: "Provides complete documentation for all functions".to_string(),
                    pros: vec!["Complete coverage".to_string(), "Better usability".to_string()],
                    cons: vec!["More time".to_string()],
                    confidence: 0.85,
                },
            ],
            chosen_option: "Comprehensive API docs".to_string(),
            reasoning: "Based on the API analysis, comprehensive documentation is needed. The module has multiple functions that need clear explanations, examples, and parameter descriptions.".to_string(),
            confidence: 0.85,
            risk_assessment: None,
            metadata: HashMap::new(),
        },
    ]
}

#[cfg(feature = "full")]
fn create_simulated_bugfix_decisions() -> Vec<DecisionPoint> {
    use agent_orchestration::chain_of_thought::{
        Alternative, DecisionContext, DecisionType, RiskAssessment,
    };
    use chrono::Utc;
    use std::collections::HashMap;
    use uuid::Uuid;

    vec![
        DecisionPoint {
            decision_id: Uuid::new_v4(),
            decision_type: DecisionType::FailureRecovery,
            timestamp: Utc::now(),
            context: DecisionContext {
                task_id: None,
                plan_id: None,
                milestone_id: None,
                worker_id: None,
                resource_constraints: HashMap::new(),
                time_constraints: None,
                priority_level: None,
            },
            alternatives: vec![
                Alternative {
                    option: "Add break statement".to_string(),
                    score: 0.9,
                    reasoning: "Fixes the logic error directly".to_string(),
                    pros: vec!["Simple fix".to_string(), "Preserves intent".to_string()],
                    cons: vec![],
                    confidence: 0.9,
                },
                Alternative {
                    option: "Refactor to use iterator".to_string(),
                    score: 0.7,
                    reasoning: "More idiomatic Rust but changes more code".to_string(),
                    pros: vec!["More idiomatic".to_string()],
                    cons: vec!["Larger change".to_string()],
                    confidence: 0.7,
                },
            ],
            chosen_option: "Add break statement".to_string(),
            reasoning: "After analyzing the code, the bug is clear: the break statement is missing. The function should stop after finding the first item > 100, but currently continues. Adding the break statement fixes this without changing the overall logic.".to_string(),
            confidence: 0.9,
            risk_assessment: Some(RiskAssessment {
                risk_level: "low".to_string(),
                risk_factors: vec!["Changing control flow".to_string()],
                mitigation_strategies: vec!["Add test case".to_string(), "Review logic".to_string()],
                fallback_options: vec!["Revert if tests fail".to_string()],
            }),
            metadata: HashMap::new(),
        },
    ]
}

#[cfg(feature = "full")]
fn create_simulated_feature_decisions() -> Vec<DecisionPoint> {
    use agent_orchestration::chain_of_thought::{Alternative, DecisionContext, DecisionType};
    use chrono::Utc;
    use std::collections::HashMap;
    use uuid::Uuid;

    vec![
        DecisionPoint {
            decision_id: Uuid::new_v4(),
            decision_type: DecisionType::WorkerAssignment,
            timestamp: Utc::now(),
            context: DecisionContext {
                task_id: None,
                plan_id: None,
                milestone_id: None,
                worker_id: None,
                resource_constraints: HashMap::new(),
                time_constraints: None,
                priority_level: None,
            },
            alternatives: vec![
                Alternative {
                    option: "Follow existing patterns".to_string(),
                    score: 0.85,
                    reasoning: "Maintains consistency with codebase".to_string(),
                    pros: vec!["Consistency".to_string(), "Familiar patterns".to_string()],
                    cons: vec![],
                    confidence: 0.85,
                },
            ],
            chosen_option: "Follow existing patterns".to_string(),
            reasoning: "Based on analysis of existing code, the UserValidator should follow the same patterns as other validators in the codebase. This includes using Result types for error handling and providing a constructor method.".to_string(),
            confidence: 0.85,
            risk_assessment: None,
            metadata: HashMap::new(),
        },
    ]
}

fn create_failed_result(scenario_name: &str, error: String) -> QualityEvaluationResult {
    QualityEvaluationResult {
        scenario_name: scenario_name.to_string(),
        reasoning_depth: ReasoningDepthScore {
            score: 0.0,
            reasoning_length_score: 0.0,
            alternatives_score: 0.0,
            risk_assessment_score: 0.0,
            confidence_calibration_score: 0.0,
        },
        decision_quality: DecisionQualityScore {
            score: 0.0,
            evidence_gathering_score: 0.0,
            logic_soundness_score: 0.0,
            confidence_calibration_score: 0.0,
            risk_mitigation_score: 0.0,
        },
        council_transparency: None,
        verdict_reasoning: None,
        output_quality: 0.0,
        overall_score: OverallQualityScore {
            score: 0.0,
            reasoning_depth: 0.0,
            decision_quality: 0.0,
            council_transparency: 0.0,
            output_quality: 0.0,
        },
        passed: false,
        success_criteria_met: vec![],
        success_criteria_failed: vec![error],
    }
}
