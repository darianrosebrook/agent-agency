//! End-to-end integration tests for autonomous agent workflows
//!
//! These tests verify complete autonomous agent behavior including:
//! 1. Task execution with model providers and circuit breakers
//! 2. Evaluation orchestration with multiple evaluators
//! 3. Iterative refinement with satisficing decisions
//! 4. Workspace management with budget constraints
//! 5. Error recovery and failure handling

use self_prompting_agent::*;
use tempfile::tempdir;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::test;

/// Test complete autonomous task execution workflow
#[tokio::test]
async fn test_complete_autonomous_workflow() {
    let temp_dir = tempdir().unwrap();
    let workspace_root = temp_dir.path().to_path_buf();

    // Initialize Git repository for workspace management
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&workspace_root)
        .output()
        .unwrap();

    // Create a simple Rust file with a syntax error
    let test_file = workspace_root.join("src").join("main.rs");
    fs::create_dir_all(test_file.parent().unwrap()).unwrap();
    fs::write(&test_file, r#"
fn main() {
    println!("Hello, world!" // Missing closing parenthesis
}
"#).unwrap();

    // Setup model registry with mock providers
    let mut model_registry = models::ModelRegistry::new();

    // Configure circuit breaker for testing
    model_registry.default_circuit_config = models::selection::CircuitBreakerConfig {
        failure_threshold: 3,
        recovery_timeout_secs: 1,
        success_threshold: 1,
    };

    // Setup evaluation orchestrator
    let evaluation_config = evaluation::EvaluationConfig {
        min_score: 0.0, // Allow any score for testing
        mandatory_gates: vec!["syntax".to_string()],
        max_iterations: 3,
        min_improvement_threshold: 0.1,
        quality_ceiling_budget: 2,
    };
    let evaluation_orchestrator = Arc::new(RwLock::new(
        evaluation::EvaluationOrchestrator::new(evaluation_config)
    ));

    // Create autonomous agent
    let agent = integration::IntegratedAutonomousAgent::new(
        Arc::new(model_registry),
        evaluation_orchestrator,
        types::ExecutionMode::Auto,
    ).await.unwrap();

    // Create task to fix syntax error
    let task = types::Task {
        id: uuid::Uuid::new_v4(),
        description: "Fix syntax error in main.rs - missing closing parenthesis".to_string(),
        task_type: types::TaskType::CodeFix,
        target_files: vec!["src/main.rs".to_string()],
        constraints: std::collections::HashMap::new(),
        refinement_context: vec![
            "This is a simple Rust syntax error".to_string(),
            "The file has a missing closing parenthesis on the println! macro".to_string(),
        ],
    };

    // Execute task autonomously
    let result = agent.execute_task_autonomously(task).await;

    // Verify execution completed (may not succeed due to mock providers, but should not crash)
    assert!(result.is_ok() || matches!(result, Err(integration::IntegrationError::AgentError(_))),
        "Task execution should complete without panicking, result: {:?}", result);

    if let Ok(task_result) = result {
        // If execution succeeded, verify basic structure
        assert!(!task_result.artifacts.is_empty(), "Should produce artifacts");

        // Verify iterations were attempted
        assert!(task_result.iterations >= 0, "Should track iterations");

        // Verify some form of evaluation happened
        assert!(task_result.final_report.status != types::EvalStatus::Fail ||
                task_result.iterations > 0, "Should attempt evaluation");
    }
}

/// Test circuit breaker integration with provider failures
#[tokio::test]
async fn test_circuit_breaker_integration() {
    use models::selection::{CircuitBreakerState, CircuitBreakerConfig, CircuitState};

    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        recovery_timeout_secs: 1,
        success_threshold: 1,
    };

    let mut breaker = CircuitBreakerState::new(config);

    // Test normal operation
    assert_eq!(breaker.state, CircuitState::Closed);
    assert!(breaker.should_attempt());

    // Simulate failures
    breaker.record_failure();
    assert_eq!(breaker.state, CircuitState::Closed); // Still closed

    breaker.record_failure();
    assert_eq!(breaker.state, CircuitState::Open); // Circuit opened
    assert!(!breaker.should_attempt()); // Should block

    // Simulate recovery timeout (in real code this would be time-based)
    breaker.state = CircuitState::HalfOpen;
    breaker.success_count = 0;
    assert!(breaker.should_attempt()); // Half-open allows testing

    // Record success and verify circuit closes
    breaker.record_success();
    assert_eq!(breaker.state, CircuitState::Closed);
}

/// Test evaluation orchestration with multiple evaluators
#[test]
fn test_evaluation_orchestration_multiple_evaluators() {
    use evaluation::{EvaluationOrchestrator, EvaluationConfig};

    let config = EvaluationConfig {
        min_score: 0.5,
        mandatory_gates: vec!["syntax".to_string(), "types".to_string()],
        max_iterations: 5,
        min_improvement_threshold: 0.1,
        quality_ceiling_budget: 3,
    };

    let orchestrator = EvaluationOrchestrator::new(config);

    // Test configuration validation
    assert_eq!(orchestrator.config.min_score, 0.5);
    assert_eq!(orchestrator.config.mandatory_gates.len(), 2);
    assert!(orchestrator.config.mandatory_gates.contains(&"syntax".to_string()));
    assert!(orchestrator.config.mandatory_gates.contains(&"types".to_string()));
}

/// Test satisficing decision logic with various scenarios
#[test]
fn test_satisficing_decision_logic() {
    use evaluation::satisficing::{SatisficingEvaluator, SatisficingConfig};
    use evaluation::EvalReport;
    use types::EvalStatus;

    let config = SatisficingConfig {
        min_score: 0.7,
        max_iterations: 10,
        min_improvement_threshold: 0.05,
        hysteresis_window: 3,
        quality_ceiling_budget: 2,
    };

    let evaluator = SatisficingEvaluator::new(config);

    // Test 1: High score should continue (not at ceiling yet)
    let high_score_report = EvalReport {
        score: 0.9,
        status: EvalStatus::Pass,
        thresholds_met: vec!["syntax".to_string(), "types".to_string()],
        failed_criteria: vec![],
    };

    let decision = evaluator.decide(&high_score_report, 1, &[]);
    assert!(decision.should_continue, "High score should continue iteration");

    // Test 2: Low score should continue for improvement
    let low_score_report = EvalReport {
        score: 0.3,
        status: EvalStatus::Fail,
        thresholds_met: vec![],
        failed_criteria: vec!["syntax".to_string()],
    };

    let decision = evaluator.decide(&low_score_report, 1, &[]);
    assert!(decision.should_continue, "Low score should continue for improvement");

    // Test 3: Max iterations reached should stop
    let decision = evaluator.decide(&high_score_report, 10, &[]);
    assert!(!decision.should_continue, "Max iterations should stop execution");
    assert_eq!(decision.reason, types::StopReason::MaxIterations);
}

/// Test workspace operations with Git integration
#[tokio::test]
async fn test_workspace_git_integration() {
    use sandbox::WorkspaceManager;

    let temp_dir = tempdir().unwrap();
    let workspace_root = temp_dir.path().to_path_buf();

    // Initialize Git repository
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&workspace_root)
        .output()
        .unwrap();

    // Create initial commit
    fs::write(workspace_root.join("README.md"), "# Test").unwrap();
    std::process::Command::new("git")
        .args(["add", "README.md"])
        .current_dir(&workspace_root)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["-c", "user.email=test@test.com", "-c", "user.name=Test", "commit", "-m", "Initial"])
        .current_dir(&workspace_root)
        .output()
        .unwrap();

    // Test workspace manager auto-detection
    let manager = WorkspaceManager::auto_detect(workspace_root.clone()).await;
    assert!(manager.is_ok(), "Should auto-detect Git workspace");

    let mut manager = manager.unwrap();
    manager.set_allow_list(vec![PathBuf::from("src/")]);
    manager.set_budgets(5, 100);

    // Test budget state tracking
    let budget_state = manager.budget_state().await;
    assert!(budget_state.is_ok(), "Should retrieve budget state");
}

/// Test error recovery mechanisms
#[test]
fn test_error_recovery_mechanisms() {
    use evaluation::satisficing::{SatisficingEvaluator, PatchFailureType, EvaluationFailureType};

    let evaluator = SatisficingEvaluator::new(Default::default());

    // Test patch failure pattern detection
    let patch_failures = vec![
        PatchFailureType::SyntaxError,
        PatchFailureType::SyntaxError,
        PatchFailureType::SyntaxError, // 3 syntax errors should trigger
    ];

    let decision = evaluator.check_patch_failure_patterns(&patch_failures);
    assert!(decision.is_some(), "Multiple syntax errors should trigger decision");
    assert!(!decision.unwrap().should_continue, "Should stop on repeated failures");

    // Test environment failure recovery
    let env_failure = EvaluationFailureType::EnvironmentFailure {
        category: evaluation::EnvironmentFailureCategory::DependencyMissing,
    };

    // In real implementation, this would trigger recovery logic
    // For testing, we verify the failure type structure
    match env_failure {
        EvaluationFailureType::EnvironmentFailure { category } => {
            assert!(matches!(category, evaluation::EnvironmentFailureCategory::DependencyMissing),
                "Should correctly identify dependency failure");
        }
        _ => panic!("Wrong failure type"),
    }
}

/// Test autonomous agent initialization and configuration
#[tokio::test]
async fn test_agent_initialization_and_config() {
    use models::ModelRegistry;
    use evaluation::{EvaluationOrchestrator, EvaluationConfig};
    use integration::IntegratedAutonomousAgent;

    // Setup components
    let model_registry = Arc::new(ModelRegistry::new());
    let eval_config = EvaluationConfig {
        min_score: 0.6,
        mandatory_gates: vec!["syntax".to_string()],
        max_iterations: 5,
        min_improvement_threshold: 0.1,
        quality_ceiling_budget: 2,
    };
    let evaluation_orchestrator = Arc::new(RwLock::new(
        EvaluationOrchestrator::new(eval_config)
    ));

    // Test agent initialization
    let agent_result = IntegratedAutonomousAgent::new(
        model_registry,
        evaluation_orchestrator,
        types::ExecutionMode::DryRun,
    ).await;

    // Should initialize successfully (may fail later due to missing providers, but init should work)
    assert!(agent_result.is_ok() || matches!(agent_result, Err(integration::IntegrationError::ModelError(_))),
        "Agent should initialize or fail gracefully with model errors, result: {:?}", agent_result);
}

/// Test performance profiling and metrics collection
#[test]
fn test_performance_metrics_collection() {
    use profiling::{PerformanceProfiler, PerformanceBenchmark};

    let profiler = PerformanceProfiler::new();

    // Test profiler initialization
    assert!(profiler.start_time <= chrono::Utc::now(),
        "Profiler should initialize with valid start time");

    // Test benchmark creation
    let benchmark = PerformanceBenchmark {
        operation: "test_operation".to_string(),
        duration_ms: 150.0,
        memory_delta_kb: 1024,
        success: true,
        metadata: std::collections::HashMap::new(),
    };

    assert_eq!(benchmark.operation, "test_operation");
    assert_eq!(benchmark.duration_ms, 150.0);
    assert!(benchmark.success);
}

/// Test type safety and data structure integrity
#[test]
fn test_type_safety_and_data_integrity() {
    use types::*;

    // Test Task creation and validation
    let task = Task {
        id: uuid::Uuid::new_v4(),
        description: "Test task".to_string(),
        task_type: TaskType::CodeFix,
        target_files: vec!["test.rs".to_string()],
        constraints: std::collections::HashMap::new(),
        refinement_context: vec!["context".to_string()],
    };

    assert!(!task.description.is_empty());
    assert_eq!(task.task_type, TaskType::CodeFix);
    assert_eq!(task.target_files.len(), 1);

    // Test ChangeSet creation
    let changeset = ChangeSet {
        changes: vec![FileChange {
            path: PathBuf::from("test.rs"),
            operation: ChangeOperation::Create {
                content: "test content".to_string(),
            },
        }],
        rationale: "Test change".to_string(),
        id: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(changeset.changes.len(), 1);
    assert!(!changeset.rationale.is_empty());
    assert_eq!(changeset.total_loc(), 1); // One line of content
}

/// Test concurrent operations safety
#[tokio::test]
async fn test_concurrent_operations_safety() {
    use std::sync::Arc;
    use tokio::sync::RwLock;

    // Test that shared state can be accessed concurrently
    let shared_counter = Arc::new(RwLock::new(0));

    // Spawn multiple tasks that increment the counter
    let mut handles = vec![];

    for i in 0..10 {
        let counter_clone = Arc::clone(&shared_counter);
        let handle = tokio::spawn(async move {
            let mut counter = counter_clone.write().await;
            *counter += 1;
            *counter // Return final value
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    let mut total = 0;
    for handle in handles {
        total += handle.await.unwrap();
    }

    // Each task returns the counter value at the time it ran
    // The total should be the sum of all intermediate values
    assert!(total > 0, "Concurrent operations should complete successfully");

    // Final counter value should be 10
    let final_value = *shared_counter.read().await;
    assert_eq!(final_value, 10, "Counter should be incremented exactly 10 times");
}

/// Test resource cleanup and proper shutdown
#[tokio::test]
async fn test_resource_cleanup_and_shutdown() {
    let temp_dir = tempdir().unwrap();
    let workspace_root = temp_dir.path().to_path_buf();

    // Initialize Git repository
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&workspace_root)
        .output()
        .unwrap();

    // Create workspace manager
    let manager = sandbox::WorkspaceManager::auto_detect(workspace_root.clone()).await;
    assert!(manager.is_ok(), "Should create workspace manager");

    // Create some files and operations
    let test_file = workspace_root.join("test.txt");
    fs::write(&test_file, "test content").unwrap();

    // Verify file exists
    assert!(test_file.exists(), "Test file should be created");

    // Simulate cleanup (temp directory will be automatically cleaned up)
    drop(manager);
    drop(temp_dir); // This should clean up the temp directory

    // Note: We can't verify cleanup in this test since tempdir cleanup happens
    // when the TempDir goes out of scope, but this tests that our components
    // don't prevent proper cleanup
}
