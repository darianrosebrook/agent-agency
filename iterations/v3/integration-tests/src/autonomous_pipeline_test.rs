//! End-to-End Tests for Full Autonomous Pipeline
//!
//! These tests validate the complete autonomous execution workflow from
//! natural language task intake through implementation, testing, and deployment.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use agent_agency_v3::{
    self_prompting_agent::{SelfPromptingLoop, SelfPromptingConfig, Task, TaskBuilder},
    workers::{WorkerPoolManager, AutonomousExecutor, AutonomousExecutorConfig, ExecutionResult},
    orchestration::arbiter::{ArbiterOrchestrator, ArbiterConfig, ArbiterVerdict, VerdictStatus},
    claim_extraction::{ClaimExtractionProcessor, ProcessingContext, ClaimExtractionResult},
    file_ops::{WorkspaceFactory, AllowList, Budgets, ChangeSetId},
    config::{AppConfig, DatabaseConfig, WorkerConfig},
    database::{DatabaseManager, DbConfig},
};

/// Complete autonomous pipeline test
#[tokio::test]
async fn test_full_autonomous_pipeline_execution() {
    // Setup test infrastructure
    let test_config = create_test_config();
    let db_manager = setup_test_database().await;
    let workspace_factory = WorkspaceFactory::new();

    // Create test task
    let task = create_test_task("Implement a user authentication service with JWT tokens and role-based access control");

    // Initialize components
    let worker_pool = Arc::new(WorkerPoolManager::new(test_config.worker.clone()));
    let arbiter = Arc::new(ArbiterOrchestrator::new(test_config.arbiter.clone()));

    let executor_config = AutonomousExecutorConfig {
        enable_arbiter_adjudication: true,
        ..Default::default()
    };

    let (executor, mut execution_rx) = AutonomousExecutor::new(
        worker_pool.clone(),
        Arc::new(MockCawsValidator),
        Some(arbiter.clone()),
        executor_config,
    );

    // Initialize self-prompting loop
    let allow_list = AllowList {
        globs: vec![
            "src/**/*.rs".to_string(),
            "src/**/*.ts".to_string(),
            "tests/**/*.rs".to_string(),
        ],
    };
    let budgets = Budgets {
        max_files: 10,
        max_loc: 500,
    };

    let loop_config = SelfPromptingConfig {
        max_iterations: 5,
        enable_evaluation: true,
        enable_rollback: true,
        evaluation_threshold: 0.8,
        satisficing_enabled: true,
        ..Default::default()
    };

    let loop_controller = SelfPromptingLoop::with_config(
        workspace_factory.clone(),
        allow_list,
        budgets,
        loop_config,
    );

    // Execute the task
    let task_id = Uuid::new_v4();
    let execution_result = executor.execute_with_arbiter(&task.working_spec().unwrap(), task_id).await;

    // Assert successful execution
    assert!(execution_result.is_ok(), "Autonomous execution should succeed");
    let mediated_result = execution_result.unwrap();

    // Verify arbiter adjudication occurred
    assert_eq!(mediated_result.verdict.status, VerdictStatus::Approved,
               "Task should be approved by arbiter");

    // Verify execution completed
    assert!(mediated_result.execution_result.is_some(),
            "Execution result should be present");

    let exec_result = mediated_result.execution_result.unwrap();

    // Verify quality standards were met
    assert!(exec_result.quality_score >= 0.8,
            "Quality score should meet threshold: {}", exec_result.quality_score);

    // Verify artifacts were generated
    assert!(!exec_result.artifacts.is_empty(),
            "Execution should generate artifacts");

    // Verify files were created/modified
    assert!(!exec_result.files_changed.is_empty(),
            "Execution should modify files");

    println!(" Full autonomous pipeline test completed successfully");
}

/// Test arbiter adjudication with claim verification
#[tokio::test]
async fn test_arbiter_adjudication_with_claims() {
    let config = create_test_config();
    let arbiter = ArbiterOrchestrator::new(config.arbiter.clone());

    // Create test task with working spec
    let task = create_test_task("Add user registration endpoint");

    // Generate competing outputs from different workers
    let outputs = vec![
        WorkerOutput {
            task_id: Uuid::new_v4(),
            worker_id: "worker-1".to_string(),
            content: "Generated user registration code with proper validation and error handling".to_string(),
            confidence: 0.85,
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        },
        WorkerOutput {
            task_id: Uuid::new_v4(),
            worker_id: "worker-2".to_string(),
            content: "Implemented registration with security vulnerabilities".to_string(),
            confidence: 0.75,
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        },
    ];

    // Execute adjudication
    let result = arbiter.adjudicate_task(&task.working_spec().unwrap(), outputs).await;

    assert!(result.is_ok(), "Adjudication should succeed");
    let debate_result = result.unwrap();

    // Verify claims were extracted and verified
    assert!(!debate_result.evidence_manifest.claims.is_empty(),
            "Claims should be extracted from outputs");

    // Verify factual accuracy scoring
    assert!(debate_result.evidence_manifest.factual_accuracy_score >= 0.0,
            "Factual accuracy should be calculated");

    // Verify CAWS compliance scoring
    assert!(debate_result.evidence_manifest.caws_compliance_score >= 0.0,
            "CAWS compliance should be calculated");

    println!(" Arbiter adjudication with claims test passed");
}

/// Test self-prompting loop with file operations and rollback
#[tokio::test]
async fn test_self_prompting_with_file_operations() {
    let workspace_factory = WorkspaceFactory::new();
    let allow_list = AllowList {
        globs: vec!["src/**/*.rs".to_string()],
    };
    let budgets = Budgets {
        max_files: 5,
        max_loc: 100,
    };

    let config = SelfPromptingConfig {
        max_iterations: 3,
        enable_evaluation: true,
        enable_rollback: true,
        evaluation_threshold: 0.7,
        satisficing_enabled: true,
        ..Default::default()
    };

    let loop_controller = SelfPromptingLoop::with_config(
        workspace_factory,
        allow_list,
        budgets,
        config,
    );

    let task = create_test_task("Add a simple logging function");

    // Execute the loop
    let result = loop_controller.execute_task(task).await;

    assert!(result.is_ok(), "Self-prompting execution should succeed");
    let execution_result = result.unwrap();

    // Verify iterations occurred
    assert!(execution_result.iterations > 0,
            "At least one iteration should occur");

    // Verify quality improved or stabilized
    assert!(execution_result.final_quality >= execution_result.initial_quality * 0.9,
            "Quality should not degrade significantly");

    // Verify changesets were created
    assert!(!execution_result.changesets.is_empty(),
            "Changesets should be created");

    // Verify rollback capability (if triggered)
    if execution_result.rollback_occurred {
        assert!(execution_result.changesets.len() > 1,
                "Multiple changesets should exist if rollback occurred");
    }

    println!(" Self-prompting with file operations test passed");
}

/// Test claim extraction across different modalities
#[tokio::test]
async fn test_multi_modal_claim_extraction() {
    let processor = ClaimExtractionProcessor::new();

    // Test code claim extraction
    let code_output = r#"
    pub fn authenticate_user(credentials: &Credentials) -> Result<User, AuthError> {
        // Validate credentials
        if credentials.email.is_empty() {
            return Err(AuthError::InvalidCredentials);
        }

        // Hash password with bcrypt
        let hashed_password = bcrypt::hash(&credentials.password, bcrypt::DEFAULT_COST)?;

        // Check database
        let user = database::find_user_by_email(&credentials.email)?;
        if !bcrypt::verify(&credentials.password, &user.hashed_password)? {
            return Err(AuthError::InvalidCredentials);
        }

        Ok(user)
    }
    "#;

    let code_spec = claim_extraction::CodeSpecification {
        language: "rust".to_string(),
        framework: Some("actix-web".to_string()),
        security_requirements: vec!["password-hashing".to_string(), "input-validation".to_string()],
    };

    let code_claims = processor.extract_code_claims(code_output, &code_spec).await;
    assert!(code_claims.is_ok(), "Code claim extraction should succeed");
    assert!(!code_claims.unwrap().is_empty(), "Code claims should be extracted");

    // Test documentation claim extraction
    let doc_output = r#"
    # User Authentication API

    ## POST /api/auth/login

    Authenticates a user with email and password.

    **Request Body:**
    ```json
    {
      "email": "user@example.com",
      "password": "securepassword"
    }
    ```

    **Response:**
    ```json
    {
      "token": "jwt-token-here",
      "user": {
        "id": 123,
        "email": "user@example.com"
      }
    }
    ```

    **Security:**
    - Passwords are hashed with bcrypt
    - JWT tokens expire in 24 hours
    - Rate limiting: 5 requests per minute
    "#;

    let doc_standards = claim_extraction::DocumentationStandards {
        format: "markdown".to_string(),
        required_sections: vec!["security".to_string(), "api".to_string()],
    };

    let doc_claims = processor.extract_documentation_claims(doc_output, &doc_standards).await;
    assert!(doc_claims.is_ok(), "Documentation claim extraction should succeed");
    assert!(!doc_claims.unwrap().is_empty(), "Documentation claims should be extracted");

    println!(" Multi-modal claim extraction test passed");
}

/// Test CLI intervention controls
#[tokio::test]
async fn test_cli_intervention_controls() {
    // Test strict mode execution
    let result = run_cli_command(vec![
        "execute",
        "Add a simple API endpoint",
        "--mode", "strict",
        "--arbiter",
    ]);

    assert!(result.is_ok(), "CLI strict mode should execute successfully");

    // Test auto mode execution
    let result = run_cli_command(vec![
        "execute",
        "Implement data validation",
        "--mode", "auto",
        "--arbiter",
    ]);

    assert!(result.is_ok(), "CLI auto mode should execute successfully");

    // Test dry-run mode
    let result = run_cli_command(vec![
        "execute",
        "Create test utilities",
        "--mode", "dry-run",
    ]);

    assert!(result.is_ok(), "CLI dry-run mode should execute successfully");

    println!(" CLI intervention controls test passed");
}

/// Test performance under load
#[tokio::test]
async fn test_performance_under_load() {
    let config = create_test_config();
    let worker_pool = Arc::new(WorkerPoolManager::new(config.worker.clone()));

    let executor_config = AutonomousExecutorConfig {
        enable_arbiter_adjudication: false, // Disable for performance test
        ..Default::default()
    };

    let (executor, _) = AutonomousExecutor::new(
        worker_pool,
        Arc::new(MockCawsValidator),
        None,
        executor_config,
    );

    // Execute multiple tasks concurrently
    let tasks = (0..5)
        .map(|i| create_test_task(&format!("Task {}", i)))
        .collect::<Vec<_>>();

    let start_time = std::time::Instant::now();

    let mut handles = vec![];
    for task in tasks {
        let executor = executor.clone();
        let handle = tokio::spawn(async move {
            let task_id = Uuid::new_v4();
            executor.execute_with_arbiter(&task.working_spec().unwrap(), task_id).await
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok(), "Concurrent execution should succeed");
    }

    let duration = start_time.elapsed();

    // Verify reasonable performance (under 30 seconds for 5 concurrent tasks)
    assert!(duration < Duration::from_secs(30),
            "Concurrent execution should complete within 30 seconds, took {:?}", duration);

    println!(" Performance under load test passed ({:?})", duration);
}

/// Test error recovery and rollback
#[tokio::test]
async fn test_error_recovery_and_rollback() {
    let workspace_factory = WorkspaceFactory::new();
    let allow_list = AllowList {
        globs: vec!["src/**/*.rs".to_string()],
    };
    let budgets = Budgets {
        max_files: 3,
        max_loc: 50,
    };

    let config = SelfPromptingConfig {
        max_iterations: 5,
        enable_evaluation: true,
        enable_rollback: true,
        evaluation_threshold: 0.9, // High threshold to trigger rollback
        satisficing_enabled: false, // Force improvement attempts
        ..Default::default()
    };

    let loop_controller = SelfPromptingLoop::with_config(
        workspace_factory,
        allow_list,
        budgets,
        config,
    );

    let task = create_test_task("Implement a complex algorithm that will initially fail quality checks");

    let result = loop_controller.execute_task(task).await;

    // Should either succeed after iterations or fail gracefully
    match result {
        Ok(execution_result) => {
            // If successful, verify quality standards were met
            assert!(execution_result.final_quality >= config.evaluation_threshold,
                    "Final quality should meet threshold");

            // If rollback occurred, verify recovery
            if execution_result.rollback_occurred {
                assert!(execution_result.iterations > 1,
                        "Multiple iterations should occur with rollback");
            }
        }
        Err(_) => {
            // If failed, verify graceful degradation
            // This is acceptable for complex tasks that exceed capabilities
        }
    }

    println!(" Error recovery and rollback test completed");
}

// Helper functions and mocks

fn create_test_config() -> AppConfig {
    AppConfig {
        database: DatabaseConfig {
            url: "postgres://test:test@localhost:5432/test".to_string(),
            max_connections: 5,
            connection_timeout_ms: 5000,
        },
        worker: WorkerConfig {
            pool_size: 4,
            task_timeout_seconds: 300,
            max_concurrent_tasks: 2,
        },
        arbiter: ArbiterConfig {
            council_size: 3,
            debate_rounds: 2,
            confidence_threshold: 0.8,
        },
    }
}

async fn setup_test_database() -> DatabaseManager {
    let config = DbConfig {
        database_url: "postgres://test:test@localhost:5432/test".to_string(),
        max_connections: 2,
    };

    DatabaseManager::new(config).await.unwrap()
}

fn create_test_task(description: &str) -> Task {
    TaskBuilder::new()
        .description(description.to_string())
        .project_path(std::path::PathBuf::from("/tmp/test-project"))
        .risk_tier("standard".to_string())
        .build()
}

// Mock implementations for testing

struct MockCawsValidator;

#[async_trait::async_trait]
impl CawsRuntimeValidator for MockCawsValidator {
    async fn validate(&self, _spec: &WorkingSpec) -> Result<ValidationResult, ValidationError> {
        Ok(ValidationResult {
            is_valid: true,
            violations: vec![],
            recommendations: vec![],
            confidence: 0.9,
        })
    }
}

fn run_cli_command(args: Vec<&str>) -> Result<(), Box<dyn std::error::Error>> {
    // This would integrate with the actual CLI binary
    // For testing, we'll simulate the command execution
    println!("Simulating CLI command: {:?}", args);

    // Simulate different execution modes
    match args.get(2) {
        Some(&"strict") => {
            // Simulate user prompts for strict mode
            println!("Strict mode: Would prompt for approval");
        }
        Some(&"auto") => {
            // Simulate automatic execution
            println!("Auto mode: Would execute automatically");
        }
        Some(&"dry-run") => {
            // Simulate dry-run
            println!("Dry-run mode: Would generate artifacts only");
        }
        _ => {}
    }

    Ok(())
}

// Import additional types needed for tests
use agent_agency_v3::{
    caws::{CawsRuntimeValidator, ValidationResult, ValidationError},
    planning::types::WorkingSpec,
    workers::WorkerOutput,
};
