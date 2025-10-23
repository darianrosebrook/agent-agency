//! Simple integration test for worker execution path

use agent_agency_workers::TaskExecutor;
use uuid::Uuid;

#[tokio::test]
async fn test_worker_execution_integration() {
    // Create executor
    let executor = TaskExecutor::new();

    // Create a test task spec
    let task_spec = agent_agency_contracts::working_spec::TaskSpec {
        id: Uuid::new_v4(),
        title: "Integration Test Task".to_string(),
        description: "Test task for worker execution".to_string(),
        risk_tier: agent_agency_contracts::working_spec::RiskTier::Tier3,
        mode: agent_agency_contracts::working_spec::TaskMode::Feature,
        change_budget: agent_agency_contracts::working_spec::ChangeBudget {
            max_files: 10,
            max_loc: 100,
        },
        blast_radius: agent_agency_contracts::working_spec::BlastRadius {
            modules: vec![],
            data_migration: false,
        },
        operational_rollback_slo: "5m".to_string(),
        scope: agent_agency_contracts::working_spec::Scope {
            in_scope: vec![],
            out_scope: vec![],
        },
        invariants: vec![],
        acceptance_criteria: vec!["Task completes successfully".to_string()],
        non_functional_requirements: None,
        validation_results: None,
        metadata: None,
    };

    let worker_id = Uuid::new_v4();

    // Execute task (without circuit breaker for simplicity)
    let result = executor.execute_task(task_spec, worker_id, None).await;

    match result {
        Ok(execution_result) => {
            println!(" Task execution successful!");
            println!("Task ID: {}", execution_result.task_id);
            println!("Worker ID: {}", execution_result.worker_id);
            println!("Status: {:?}", execution_result.status);
            println!("Execution time: {}ms", execution_result.execution_time_ms);

            assert_eq!(execution_result.status, agent_agency_workers::ExecutionStatus::Completed);
            assert!(execution_result.execution_time_ms > 0);
        }
        Err(e) => {
            panic!(" Task execution failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_worker_cancellation_integration() {
    // Create executor
    let executor = TaskExecutor::new();

    let task_id = Uuid::new_v4();
    let worker_id = Uuid::new_v4();

    // Test cancellation
    let cancel_result = executor.cancel_task_execution(task_id, worker_id).await;

    match cancel_result {
        Ok(()) => {
            println!(" Task cancellation successful!");
        }
        Err(e) => {
            panic!(" Task cancellation failed: {}", e);
        }
    }
}
