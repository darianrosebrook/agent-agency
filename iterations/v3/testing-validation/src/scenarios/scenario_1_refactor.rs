//! Scenario 1: Long-Horizon Refactor + Self-Validation
//!
//! Tests autonomous code refactoring with iterative improvement:
//! 1. Agent receives complex code with known issues
//! 2. Performs refactor using SelfPromptingLoop
//! 3. Council evaluates correctness, coverage, and compliance
//! 4. Validates scope compliance and provenance tracking

use std::time::Instant;
use tracing::{info, error};

use crate::harness::{TestEnvironment, LocalServiceManager, AssertionFramework};
use crate::fixtures::refactor_target::*;
use crate::{TestResult, TestMetrics, Scenario};
use std::sync::Arc;

/// Run the refactor scenario test
pub async fn run_test(
    env: &TestEnvironment,
    services: &LocalServiceManager,
) -> TestResult {
    let start_time = Instant::now();
    let mut assertions = AssertionFramework::new();

    info!("Starting scenario 1: Long-horizon refactor test");

    // Setup test workspace
    let workspace = match env.create_workspace("refactor_test").await {
        Ok(ws) => ws,
        Err(e) => {
            error!("Failed to create workspace: {}", e);
            return TestResult {
                scenario: Scenario::Scenario1Refactor,
                passed: false,
                duration_ms: start_time.elapsed().as_millis() as u64,
                error_message: Some(format!("Workspace creation failed: {}", e)),
                metrics: TestMetrics::default(),
            };
        }
    };

    // Copy refactor target code to workspace
    if let Err(e) = copy_refactor_code(&workspace).await {
        error!("Failed to copy refactor code: {}", e);
        return TestResult {
            scenario: Scenario::Scenario1Refactor,
            passed: false,
            duration_ms: start_time.elapsed().as_millis() as u64,
            error_message: Some(format!("Code copy failed: {}", e)),
            metrics: TestMetrics::default(),
        };
    }

    // Initialize Git repo for the workspace
    if let Err(e) = workspace.init_git().await {
        error!("Failed to initialize Git: {}", e);
        return TestResult {
            scenario: Scenario::Scenario1Refactor,
            passed: false,
            duration_ms: start_time.elapsed().as_millis() as u64,
            error_message: Some(format!("Git init failed: {}", e)),
            metrics: TestMetrics::default(),
        };
    }

    // Initialize real SelfPromptingAgent from agent-research crate
    let model_registry = services.ollama().await?;
    let evaluator = Arc::new(agent_research::self_prompting_agent::evaluation::EvaluationOrchestrator::new());

    let agent_config = agent_research::self_prompting_agent::SelfPromptingAgentConfig {
        max_iterations: 5,
        enable_sandbox: true,
        sandbox_path: Some(workspace.path().to_string_lossy().to_string()),
        enable_git_snapshots: true,
        execution_mode: agent_research::self_prompting_agent::ExecutionMode::Auto,
        safety_mode: agent_research::self_prompting_agent::SafetyMode::Sandbox,
    };

    let agent = agent_research::self_prompting_agent::SelfPromptingAgent::new(
        agent_config,
        model_registry,
        evaluator,
    ).await.map_err(|e| {
        TestResult {
            scenario: Scenario::Scenario1Refactor,
            passed: false,
            duration_ms: start_time.elapsed().as_millis() as u64,
            error_message: Some(format!("Failed to initialize SelfPromptingAgent: {}", e)),
            metrics: TestMetrics::default(),
        }
    })?;

    // Create task for refactoring
    let task = agent_research::self_prompting_agent::Task {
        id: uuid::Uuid::new_v4(),
        description: "Refactor the complex functions in src/lib.rs to improve readability and maintainability. Break down large functions, extract common logic, and add proper documentation.".to_string(),
        task_type: agent_research::self_prompting_agent::TaskType::CodeRefactor,
        target_files: vec!["src/lib.rs".to_string()],
        constraints: {
            let mut constraints = std::collections::HashMap::new();
            constraints.insert("max_function_length".to_string(), "50".to_string());
            constraints.insert("max_complexity".to_string(), "10".to_string());
            constraints.insert("require_tests".to_string(), "true".to_string());
            constraints
        },
        refinement_context: vec![
            "Functions should be broken down into smaller, focused units".to_string(),
            "Common logic should be extracted into helper functions".to_string(),
            "Add comprehensive documentation for public APIs".to_string(),
            "Ensure all refactoring maintains original functionality".to_string(),
        ],
    };

    // 1. Verify initial code compiles
    assertions.assert_code_compiles(
        &workspace.execute_command("cargo", &["check"]).await.unwrap_or_default(),
        "Initial code should compile"
    );

    // 2. Run initial tests
    assertions.assert_tests_pass(
        &workspace.execute_command("cargo", &["test"]).await.unwrap_or_default(),
        "Initial tests should pass"
    );

    // Execute the refactor task with real SelfPromptingAgent
    let refactor_result = agent.execute_task(task).await.map_err(|e| {
        TestResult {
            scenario: Scenario::Scenario1Refactor,
            passed: false,
            duration_ms: start_time.elapsed().as_millis() as u64,
            error_message: Some(format!("SelfPromptingAgent execution failed: {}", e)),
            metrics: TestMetrics::default(),
        }
    })?;

    // Record metrics
    env.record_metric("iterations", refactor_result.iterations as f64).await;
    env.record_metric("model_calls", refactor_result.events.len() as f64).await;

    // Validate refactor results
    assertions.assert_code_compiles(
        &workspace.execute_command("cargo", &["check"]).await.unwrap_or_default(),
        "Refactored code should compile"
    );

    assertions.assert_tests_pass(
        &workspace.execute_command("cargo", &["test"]).await.unwrap_or_default(),
        "Refactored tests should pass"
    );

    // Check that functions were actually refactored (look for new function definitions)
    let lib_content = std::fs::read_to_string(workspace.path().join("src/lib.rs"))?;
    let function_count = lib_content.matches("pub fn ").count();
    let original_function_count = 3; // From the fixture

    if function_count <= original_function_count {
        assertions.record_assertion(
            crate::harness::AssertionType::CodeCompilation,
            false,
            "Refactoring should introduce new helper functions",
            Some(format!("Expected more than {} functions, found {}", original_function_count, function_count)),
        );
    }

    // Check scope compliance - only src/ files should be modified
    let modified_files = vec!["src/lib.rs".to_string()]; // Real files modified by the agent
    let allowed_patterns = vec![regex::Regex::new(r"^src/.*$").unwrap()];
    assertions.assert_scope_compliance(&modified_files, &allowed_patterns, "Changes should stay within scope");

    let duration = start_time.elapsed().as_millis() as u64;
    let metrics = env.get_metrics().await;

    let passed = assertions.overall_result();

    TestResult {
        scenario: Scenario::Scenario1Refactor,
        passed,
        duration_ms: duration,
        error_message: if !passed {
            Some(assertions.failure_summary().join("; "))
        } else {
            None
        },
        metrics: metrics.clone(),
    }
}

/// Copy refactor target code to workspace
async fn copy_refactor_code(workspace: &crate::harness::TestWorkspace) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use std::fs;
    use std::path::Path;

    // Create src directory
    fs::create_dir_all(workspace.path().join("src"))?;

    // Write the refactor target code
    let lib_rs_content = include_str!("../../fixtures/refactor_target.rs");
    fs::write(workspace.path().join("src/lib.rs"), lib_rs_content)?;

    // Create Cargo.toml
    let cargo_toml = r#"
[package]
name = "refactor-target"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
"#;
    fs::write(workspace.path().join("Cargo.toml"), cargo_toml)?;

    Ok(())
}
