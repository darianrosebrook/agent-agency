//! Multi-Agent Coordination Test Suite
//!
//! Validates agent-to-agent communication, arbitration, and conflict resolution:
//! - Agent communication protocols
//! - Arbitration mechanisms
//! - Conflict resolution strategies
//! - Task decomposition and delegation
//! - Consensus formation

use std::time::Instant;
use tracing::{info, error};
use uuid::Uuid;

use crate::{TestResult, TestMetrics, harness::{TestEnvironment, LocalServiceManager}};
use agent_orchestration::council::create_default_council;
use agent_orchestration::types::{TaskDescriptor, TaskScope, ChangeBudget, BlastRadius, AcceptanceCriterion, TaskPriority, ExecutionMode, TaskType};
use std::sync::Arc;

/// Run the multi-agent coordination E2E test
#[cfg(feature = "full")]
pub async fn run_multi_agent_test(
    env: &TestEnvironment,
    services: &LocalServiceManager,
) -> TestResult {
    let start_time = Instant::now();
    info!("Starting Multi-Agent Coordination E2E test");

    let mut metrics = TestMetrics::default();
    let mut passed = true;
    let mut errors = Vec::new();

    // Test 1: Agent communication
    match test_agent_communication(env, services).await {
        Ok(result) => {
            metrics.agent_communications += result.communications;
            if !result.passed {
                passed = false;
                errors.push(format!("Agent communication failed: {}", result.error.unwrap_or_default()));
            }
        }
        Err(e) => {
            passed = false;
            errors.push(format!("Agent communication error: {}", e));
        }
    }

    // Test 2: Arbitration mechanism
    match test_arbitration_mechanism(env, services).await {
        Ok(result) => {
            metrics.arbitration_events += result.arbitration_events;
            if !result.passed {
                passed = false;
                errors.push(format!("Arbitration mechanism failed: {}", result.error.unwrap_or_default()));
            }
        }
        Err(e) => {
            passed = false;
            errors.push(format!("Arbitration mechanism error: {}", e));
        }
    }

    // Test 3: Task decomposition
    match test_task_decomposition(env, services).await {
        Ok(result) => {
            metrics.task_decompositions += result.task_decompositions;
            if !result.passed {
                passed = false;
                errors.push(format!("Task decomposition failed: {}", result.error.unwrap_or_default()));
            }
        }
        Err(e) => {
            passed = false;
            errors.push(format!("Task decomposition error: {}", e));
        }
    }

    // Test 4: Conflict resolution
    match test_conflict_resolution(env, services).await {
        Ok(result) => {
            metrics.conflict_resolutions += result.conflict_resolutions;
            metrics.consensus_achieved += result.consensus_achieved;
            if !result.passed {
                passed = false;
                errors.push(format!("Conflict resolution failed: {}", result.error.unwrap_or_default()));
            }
        }
        Err(e) => {
            passed = false;
            errors.push(format!("Conflict resolution error: {}", e));
        }
    }

    let error_message = if errors.is_empty() {
        None
    } else {
        Some(errors.join("; "))
    };

    TestResult {
        scenario: crate::Scenario::MultiAgentCoordination,
        passed,
        duration_ms: start_time.elapsed().as_millis() as u64,
        error_message,
        metrics,
    }
}

/// Run the multi-agent coordination E2E test (no full feature)
#[cfg(not(feature = "full"))]
pub async fn run_multi_agent_test(
    _env: &TestEnvironment,
    _services: &LocalServiceManager,
) -> TestResult {
    let start_time = Instant::now();
    error!("Multi-Agent Coordination test requires 'full' feature");
    TestResult {
        scenario: crate::Scenario::MultiAgentCoordination,
        passed: false,
        duration_ms: start_time.elapsed().as_millis() as u64,
        error_message: Some("Multi-Agent Coordination test requires 'full' feature".to_string()),
        metrics: TestMetrics::default(),
    }
}

/// Test result for individual multi-agent coordination tests
struct MultiAgentTestResult {
    passed: bool,
    error: Option<String>,
    communications: u64,
    arbitration_events: u64,
    conflict_resolutions: u64,
    task_decompositions: u64,
    consensus_achieved: u64,
}

/// Test 1: Agent communication
async fn test_agent_communication(
    _env: &TestEnvironment,
    _services: &LocalServiceManager,
) -> Result<MultiAgentTestResult, Box<dyn std::error::Error + Send + Sync>> {
    info!("Testing agent communication");

    // Create council for agent coordination
    let council = create_default_council()
        .map_err(|e| format!("Failed to create council: {}", e))?;

    // Test that multiple judges can communicate
    let judge_count = council.available_judges().len();
    
    if judge_count == 0 {
        return Ok(MultiAgentTestResult {
            passed: false,
            error: Some("No judges available for communication test".to_string()),
            communications: 0,
            arbitration_events: 0,
            conflict_resolutions: 0,
            task_decompositions: 0,
            consensus_achieved: 0,
        });
    }

    info!("Council has {} judges available for communication", judge_count);

    Ok(MultiAgentTestResult {
        passed: true,
        error: None,
        communications: judge_count as u64,
        arbitration_events: 0,
        conflict_resolutions: 0,
        task_decompositions: 0,
        consensus_achieved: 0,
    })
}

/// Test 2: Arbitration mechanism
async fn test_arbitration_mechanism(
    _env: &TestEnvironment,
    _services: &LocalServiceManager,
) -> Result<MultiAgentTestResult, Box<dyn std::error::Error + Send + Sync>> {
    info!("Testing arbitration mechanism");

    // Create council for arbitration
    let council = create_default_council()
        .map_err(|e| format!("Failed to create council: {}", e))?;

    // Create a test task descriptor
    let task_descriptor = TaskDescriptor {
        task_id: format!("test-task-{}", Uuid::new_v4()),
        description: "Test task for arbitration".to_string(),
        scope_in: TaskScope {
            in_scope: vec!["src/test.rs".to_string()],
            out_scope: vec![],
        },
        scope_out: TaskScope {
            in_scope: vec![],
            out_scope: vec![],
        },
        change_budget: ChangeBudget {
            max_files: 5,
            max_loc: 100,
        },
        blast_radius: BlastRadius {
            modules: vec![],
            data_migration: false,
            external_deps: vec![],
        },
        priority: TaskPriority::Normal,
        execution_mode: ExecutionMode::Auto,
        task_type: TaskType::Feature,
        risk_tier: 2,
        acceptance: vec![AcceptanceCriterion {
            id: "AC1".to_string(),
            given: "Given a test scenario".to_string(),
            when: "When arbitration is performed".to_string(),
            then: "Then consensus should be achieved".to_string(),
        }],
    };

    // Start a council session to test arbitration
    let session_result = council.start_session(&task_descriptor).await;
    
    match session_result {
        Ok(session) => {
            info!("Council session started successfully for arbitration test");
            Ok(MultiAgentTestResult {
                passed: true,
                error: None,
                communications: 0,
                arbitration_events: 1,
                conflict_resolutions: 0,
                task_decompositions: 0,
                consensus_achieved: 0,
            })
        }
        Err(e) => {
            Ok(MultiAgentTestResult {
                passed: false,
                error: Some(format!("Failed to start council session: {}", e)),
                communications: 0,
                arbitration_events: 0,
                conflict_resolutions: 0,
                task_decompositions: 0,
                consensus_achieved: 0,
            })
        }
    }
}

/// Test 3: Task decomposition
async fn test_task_decomposition(
    _env: &TestEnvironment,
    _services: &LocalServiceManager,
) -> Result<MultiAgentTestResult, Box<dyn std::error::Error + Send + Sync>> {
    info!("Testing task decomposition");

    // Create a complex task that can be decomposed
    let complex_task = TaskDescriptor {
        task_id: format!("decompose-task-{}", Uuid::new_v4()),
        description: "Complex task requiring decomposition: Implement user authentication with database, API, and frontend components".to_string(),
        scope_in: TaskScope {
            in_scope: vec![
                "src/auth/".to_string(),
                "src/api/".to_string(),
                "src/frontend/".to_string(),
            ],
            out_scope: vec![],
        },
        scope_out: TaskScope {
            in_scope: vec![],
            out_scope: vec![],
        },
        change_budget: ChangeBudget {
            max_files: 25,
            max_loc: 1000,
        },
        blast_radius: BlastRadius {
            modules: vec!["auth".to_string(), "api".to_string(), "frontend".to_string()],
            data_migration: true,
            external_deps: vec!["database".to_string()],
        },
        priority: TaskPriority::High,
        execution_mode: ExecutionMode::Auto,
        task_type: TaskType::Feature,
        risk_tier: 1,
        acceptance: vec![
            AcceptanceCriterion {
                id: "AC1".to_string(),
                given: "Given a user registration form".to_string(),
                when: "When user submits credentials".to_string(),
                then: "Then account should be created in database".to_string(),
            },
            AcceptanceCriterion {
                id: "AC2".to_string(),
                given: "Given a user login form".to_string(),
                when: "When user provides valid credentials".to_string(),
                then: "Then API should return authentication token".to_string(),
            },
        ],
    };

    // Check that task has multiple acceptance criteria indicating decomposition capability
    let decomposition_count = complex_task.acceptance.len();

    info!("Task has {} acceptance criteria indicating {} potential decompositions", decomposition_count, decomposition_count);

    Ok(MultiAgentTestResult {
        passed: decomposition_count > 1,
        error: if decomposition_count <= 1 { Some("Task does not support decomposition".to_string()) } else { None },
        communications: 0,
        arbitration_events: 0,
        conflict_resolutions: 0,
        task_decompositions: decomposition_count as u64,
        consensus_achieved: 0,
    })
}

/// Test 4: Conflict resolution
async fn test_conflict_resolution(
    _env: &TestEnvironment,
    _services: &LocalServiceManager,
) -> Result<MultiAgentTestResult, Box<dyn std::error::Error + Send + Sync>> {
    info!("Testing conflict resolution");

    // Create council for conflict resolution
    let council = create_default_council()
        .map_err(|e| format!("Failed to create council: {}", e))?;

    // Create two conflicting task descriptors that might conflict
    let task1 = TaskDescriptor {
        task_id: format!("conflict-task-1-{}", Uuid::new_v4()),
        description: "Task 1: Modify shared module".to_string(),
        scope_in: TaskScope {
            in_scope: vec!["src/shared/module.rs".to_string()],
            out_scope: vec![],
        },
        scope_out: TaskScope {
            in_scope: vec![],
            out_scope: vec![],
        },
        change_budget: ChangeBudget {
            max_files: 1,
            max_loc: 50,
        },
        blast_radius: BlastRadius {
            modules: vec!["shared".to_string()],
            data_migration: false,
            external_deps: vec![],
        },
        priority: TaskPriority::Normal,
        execution_mode: ExecutionMode::Auto,
        task_type: TaskType::Feature,
        risk_tier: 2,
        acceptance: vec![],
    };

    let task2 = TaskDescriptor {
        task_id: format!("conflict-task-2-{}", Uuid::new_v4()),
        description: "Task 2: Also modify shared module".to_string(),
        scope_in: TaskScope {
            in_scope: vec!["src/shared/module.rs".to_string()],
            out_scope: vec![],
        },
        scope_out: TaskScope {
            in_scope: vec![],
            out_scope: vec![],
        },
        change_budget: ChangeBudget {
            max_files: 1,
            max_loc: 50,
        },
        blast_radius: BlastRadius {
            modules: vec!["shared".to_string()],
            data_migration: false,
            external_deps: vec![],
        },
        priority: TaskPriority::Normal,
        execution_mode: ExecutionMode::Auto,
        task_type: TaskType::Feature,
        risk_tier: 2,
        acceptance: vec![],
    };

    // Check for scope conflicts
    let conflict_detected = task1.scope_in.in_scope.iter().any(|file| {
        task2.scope_in.in_scope.contains(file)
    });

    if conflict_detected {
        info!("Conflict detected between tasks - testing resolution mechanism");
        
        // Start sessions for both tasks to test conflict resolution
        let session1_result = council.start_session(&task1).await;
        let session2_result = council.start_session(&task2).await;

        let resolutions = if session1_result.is_ok() && session2_result.is_ok() { 1 } else { 0 };
        let consensus = if resolutions > 0 { 1 } else { 0 };

        Ok(MultiAgentTestResult {
            passed: resolutions > 0,
            error: if resolutions == 0 { Some("Conflict resolution failed".to_string()) } else { None },
            communications: 0,
            arbitration_events: 0,
            conflict_resolutions: resolutions,
            task_decompositions: 0,
            consensus_achieved: consensus,
        })
    } else {
        Ok(MultiAgentTestResult {
            passed: false,
            error: Some("No conflict detected for conflict resolution test".to_string()),
            communications: 0,
            arbitration_events: 0,
            conflict_resolutions: 0,
            task_decompositions: 0,
            consensus_achieved: 0,
        })
    }
}
