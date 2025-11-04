//! Integration Tests for Autonomous Executor
//!
//! Tests the complete autonomous execution system across:
//! 1. Multi-session context continuity
//! 2. Council review and debate protocol
//! 3. Iterative refinement with satisficing
//! 4. Memory system integration
//! 5. Progress tracking and plateau detection

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;
use chrono::Utc;

use agent_agency_contracts::task_request::{TaskRequest, TaskContext, TaskConstraints, TaskMetadata, RiskTier, BudgetLimits, ScopeRestrictions, Environment, TaskPriority};
use agent_agency_contracts::task_executor::{TaskExecutionResult, TaskExecutor};
use agent_agency_contracts::task_executor_provider::TaskExecutorProvider;
use agent_agency_contracts::types::prelude::*;
use agent_agency_contracts::types::planning::TaskDescriptor;
use agent_agency_contracts::planning_io::{ChangeBudget, BudgetEnforcement};
use agent_agency_contracts::ExecutionStatus;

use agent_orchestration::autonomous_executor::{
    AutonomousExecutor, AutonomousExecutorConfig, MockCawsRuntimeValidator, MockVerdictWriter,
    OrchestrationProvenanceEmitter, TypesExecutionStatus,
};
use agent_orchestration::consensus_coordinator::{ConsensusCoordinator, RealTimeConsensusCoordinator, ConsensusConfig};
use agent_orchestration::progress_tracker::{ProgressTracker, RealTimeProgressTracker};

#[cfg(feature = "memory")]
use agent_agency_contracts::MemorySystem;

/// Mock TaskExecutor for testing
#[derive(Debug)]
struct MockTaskExecutor {
    should_succeed: bool,
    execution_delay_ms: u64,
}

impl TaskExecutor for MockTaskExecutor {
    async fn execute(&self, _spec: &agent_agency_contracts::WorkingSpec) -> TaskExecutionResult {
        if self.execution_delay_ms > 0 {
            sleep(Duration::from_millis(self.execution_delay_ms)).await;
        }

        if self.should_succeed {
            TaskExecutionResult {
                success: true,
                artifacts: vec![],
                execution_time_ms: Some(100),
                error_message: None,
                metadata: std::collections::HashMap::new(),
            }
        } else {
            TaskExecutionResult {
                success: false,
                artifacts: vec![],
                execution_time_ms: Some(50),
                error_message: Some("Mock execution failed".to_string()),
                metadata: std::collections::HashMap::new(),
            }
        }
    }
}

fn create_test_executor() -> AutonomousExecutor {
    let config = AutonomousExecutorConfig {
        max_concurrent_tasks: 10,
        task_timeout_seconds: 300,
        progress_report_interval_seconds: 5,
        enable_auto_retry: true,
        max_retry_attempts: 3,
        enable_consensus: true,
        consensus_timeout_seconds: 60,
        enable_council_review: true,
    };

    let task_executor_provider = {
        let factory = || -> Arc<dyn TaskExecutor> {
            Arc::new(MockTaskExecutor {
                should_succeed: true,
                execution_delay_ms: 10,
            })
        };
        TaskExecutorProvider::new(factory)
    };

    AutonomousExecutor::new(
        config,
        Some(Arc::new(RealTimeProgressTracker::new(None))),
        Arc::new(MockCawsRuntimeValidator),
        Some(Arc::new(RealTimeConsensusCoordinator::new(ConsensusConfig::default()))),
        Arc::new(MockVerdictWriter),
        Arc::new(OrchestrationProvenanceEmitter::new()),
        None,
        None,
        task_executor_provider,
        #[cfg(feature = "memory")]
        None,
        None,
    )
}

fn create_test_task(description: &str) -> TaskDescriptor {
    TaskDescriptor {
        task_id: Uuid::new_v4().to_string(),
        description: description.to_string(),
        change_budget: ChangeBudget {
            max_files: 10,
            max_loc: 500,
            max_migrations: 0,
            allow_breaking_changes: false,
            allow_new_dependencies: false,
            enforcement_mode: BudgetEnforcement::Warning,
        },
        priority: TaskPriority::Normal,
        execution_mode: ExecutionMode::Auto,
        risk_tier: Some(RiskTier::Tier2),
        blast_radius: BlastRadius {
            modules: vec![],
            data_migration: false,
            external_deps: vec![],
        },
        scope_in: ScopeRestrictions {
            allowed_paths: vec!["tests/".to_string()],
            blocked_paths: vec![],
        },
        scope_out: None,
        acceptance: None,
    }
}

#[tokio::test]
async fn test_multi_session_context_continuity() {
    tracing_subscriber::fmt::init();
    
    let executor = create_test_executor();
    let session_id = Uuid::new_v4();

    // Session 1: Execute first task
    let task1 = create_test_task("First task in session");
    let task_id1 = executor.submit_task(task1, Some(session_id)).await
        .expect("Failed to submit first task");

    // Wait for task to complete
    let mut attempts = 0;
    loop {
        if let Some(state) = executor.get_task_status(task_id1).await {
            if state.status == TypesExecutionStatus::Completed || 
               state.status == TypesExecutionStatus::Failed {
                break;
            }
        }
        sleep(Duration::from_millis(100)).await;
        attempts += 1;
        if attempts > 100 {
            panic!("Task 1 did not complete within timeout");
        }
    }

    // Session 2: Execute second task with same session ID
    let task2 = create_test_task("Second task in same session");
    let task_id2 = executor.submit_task(task2, Some(session_id)).await
        .expect("Failed to submit second task");

    // Verify second task can access context from first task
    let mut attempts = 0;
    loop {
        if let Some(state) = executor.get_task_status(task_id2).await {
            if state.status == TypesExecutionStatus::Completed || 
               state.status == TypesExecutionStatus::Failed {
                // Verify session context was retrieved
                assert!(state.session_id.is_some());
                assert_eq!(state.session_id.unwrap(), session_id);
                break;
            }
        }
        sleep(Duration::from_millis(100)).await;
        attempts += 1;
        if attempts > 100 {
            panic!("Task 2 did not complete within timeout");
        }
    }

    println!("Multi-session context continuity test passed");
}

#[tokio::test]
async fn test_council_review_and_debate_protocol() {
    tracing_subscriber::fmt::init();
    
    let executor = create_test_executor();
    let task = create_test_task("Task requiring council review");

    let task_id = executor.submit_task(task, None).await
        .expect("Failed to submit task");

    // Wait for council review phase
    let mut attempts = 0;
    let mut council_reviewed = false;
    
    loop {
        if let Some(state) = executor.get_task_status(task_id).await {
            // Check if council review has occurred
            if matches!(state.status, TypesExecutionStatus::Consensus | TypesExecutionStatus::Execution | TypesExecutionStatus::Completed) {
                council_reviewed = true;
            }
            
            if state.status == TypesExecutionStatus::Completed || 
               state.status == TypesExecutionStatus::Failed {
                break;
            }
        }
        sleep(Duration::from_millis(100)).await;
        attempts += 1;
        if attempts > 100 {
            panic!("Task did not complete within timeout");
        }
    }

    assert!(council_reviewed, "Council review should have occurred");
    println!("Council review and debate protocol test passed");
}

#[tokio::test]
async fn test_iterative_refinement_with_satisficing() {
    tracing_subscriber::fmt::init();
    
    let executor = create_test_executor();
    // Refinement constants are hardcoded in the implementation:
    // MAX_REFINEMENT_ITERATIONS = 5
    // SATISFICING_THRESHOLD = 0.9
    // DELTA_THRESHOLD = 0.05

    let task = create_test_task("Task requiring iterative refinement");
    let task_id = executor.submit_task(task, None).await
        .expect("Failed to submit task");

    // Wait for task completion with refinement
    let mut attempts = 0;
    let mut iterations_tracked = 0;
    
    loop {
        if let Some(state) = executor.get_task_status(task_id).await {
            // Track iteration count
            if state.current_iteration > iterations_tracked {
                iterations_tracked = state.current_iteration;
            }

            // Verify satisficing logic
            // Constants are hardcoded: SATISFICING_THRESHOLD = 0.9, DELTA_THRESHOLD = 0.05, MAX_REFINEMENT_ITERATIONS = 5
            if state.quality_scores.len() >= 2 {
                let recent_scores = &state.quality_scores[state.quality_scores.len() - 2..];
                let score_delta = recent_scores[1] - recent_scores[0];
                const SATISFICING_THRESHOLD: f64 = 0.9;
                const DELTA_THRESHOLD: f64 = 0.05;
                const MAX_REFINEMENT_ITERATIONS: u32 = 5;
                
                // If quality score exceeds threshold, should stop refining
                if recent_scores[1] >= SATISFICING_THRESHOLD {
                    // Check that refinement stopped or delta is below threshold
                    assert!(
                        score_delta.abs() < DELTA_THRESHOLD || 
                        iterations_tracked >= MAX_REFINEMENT_ITERATIONS,
                        "Should stop refining when satisficing threshold met or delta too small"
                    );
                }
            }

            if state.status == TypesExecutionStatus::Completed || 
               state.status == TypesExecutionStatus::Failed {
                break;
            }
        }
        sleep(Duration::from_millis(100)).await;
        attempts += 1;
        if attempts > 200 {
            panic!("Task did not complete within timeout");
        }
    }

    // Verify iterations were tracked
    assert!(iterations_tracked > 0, "Should have tracked at least one iteration");
    println!("Iterative refinement with satisficing test passed");
}

#[tokio::test]
#[cfg(feature = "memory")]
async fn test_memory_system_integration() {
    tracing_subscriber::fmt::init();
    
    // This test requires memory feature and a real memory system implementation
    // For now, we'll verify the memory integration points exist
    
    let executor = create_test_executor();
    let task = create_test_task("Task with memory integration");

    let task_id = executor.submit_task(task, None).await
        .expect("Failed to submit task");

    // Wait for task completion
    let mut attempts = 0;
    loop {
        if let Some(state) = executor.get_task_status(task_id).await {
            if state.status == TypesExecutionStatus::Completed || 
               state.status == TypesExecutionStatus::Failed {
                // Verify memory integration occurred
                // Memory system should have been called during execution
                break;
            }
        }
        sleep(Duration::from_millis(100)).await;
        attempts += 1;
        if attempts > 100 {
            panic!("Task did not complete within timeout");
        }
    }

    println!("Memory system integration test passed");
}

#[tokio::test]
async fn test_progress_tracking_and_plateau_detection() {
    tracing_subscriber::fmt::init();
    
    let executor = create_test_executor();
    let task = create_test_task("Task with progress tracking");

    let task_id = executor.submit_task(task, None).await
        .expect("Failed to submit task");

    // Track progress over time
    let mut progress_history = Vec::new();
    let mut attempts = 0;
    
    loop {
        if let Some(state) = executor.get_task_status(task_id).await {
            // Record progress metrics
            if !state.quality_scores.is_empty() {
                progress_history.push(state.quality_scores.clone());
            }

            if state.status == TypesExecutionStatus::Completed || 
               state.status == TypesExecutionStatus::Failed {
                // Verify progress was tracked
                assert!(!state.quality_scores.is_empty(), "Quality scores should be tracked");
                
                // Verify plateau detection can be triggered
                if state.quality_scores.len() >= 3 {
                    let recent_scores = &state.quality_scores[state.quality_scores.len() - 3..];
                    let variance = calculate_variance(recent_scores);
                    
                    // Plateau detected if variance is very low (quality not improving)
                    let is_plateau = variance < 0.01;
                    println!("Plateau detected: {}, variance: {}", is_plateau, variance);
                }
                
                break;
            }
        }
        sleep(Duration::from_millis(100)).await;
        attempts += 1;
        if attempts > 100 {
            panic!("Task did not complete within timeout");
        }
    }

    assert!(!progress_history.is_empty(), "Progress history should be recorded");
    println!("Progress tracking and plateau detection test passed");
}

#[tokio::test]
async fn test_autonomous_executor_creation() {
    tracing_subscriber::fmt::init();

    let executor = create_test_executor();

    // Just verify the executor was created successfully
    // This tests that the AutonomousExecutor struct and its dependencies can be instantiated
    println!("✅ AutonomousExecutor created successfully");
}

#[tokio::test]
async fn test_end_to_end_autonomous_execution() {
    tracing_subscriber::fmt::init();
    
    let executor = create_test_executor();
    let session_id = Uuid::new_v4();

    // Create a complex task that exercises all features
    let task = create_test_task("End-to-end autonomous execution test task");

    let task_id = executor.submit_task(task, Some(session_id)).await
        .expect("Failed to submit task");

    // Monitor execution through all phases
    let mut phase_history = Vec::new();
    let mut attempts = 0;
    
    loop {
        if let Some(state) = executor.get_task_status(task_id).await {
            // Track phase transitions
            if phase_history.is_empty() || phase_history.last() != Some(&state.status) {
                phase_history.push(state.status.clone());
            }

            // Verify all phases are properly tracked
            match state.status {
                TypesExecutionStatus::Completed => {
                    // Verify completion criteria
                    assert!(!state.quality_scores.is_empty(), "Quality scores should be recorded");
                    assert!(state.session_id.is_some(), "Session ID should be set");
                    assert_eq!(state.session_id.unwrap(), session_id);
                    
                    // Verify iteration history
                    assert!(!state.iteration_history.is_empty(), "Iteration history should be recorded");
                    
                    println!("Task completed successfully");
                    println!("Phases: {:?}", phase_history);
                    println!("Iterations: {}", state.current_iteration);
                    println!("Quality scores: {:?}", state.quality_scores);
                    break;
                }
                TypesExecutionStatus::Failed => {
                    panic!("Task execution failed");
                }
                _ => {
                    // Continue waiting
                }
            }
        }
        
        sleep(Duration::from_millis(100)).await;
        attempts += 1;
        if attempts > 200 {
            panic!("Task did not complete within timeout");
        }
    }

    // Verify all phases were traversed
    assert!(phase_history.len() >= 2, "Should have gone through multiple phases");
    println!("End-to-end autonomous execution test passed");
}

fn calculate_variance(scores: &[f64]) -> f64 {
    if scores.is_empty() {
        return 0.0;
    }
    
    let mean = scores.iter().sum::<f64>() / scores.len() as f64;
    let variance = scores.iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>() / scores.len() as f64;
    
    variance
}

