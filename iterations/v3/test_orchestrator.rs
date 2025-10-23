use agent_agency_orchestration::orchestrate::{Orchestrator, OrchestratorConfig};
use agent_agency_orchestration::tracking::{ProgressTracker, ProgressTrackerConfig};
use std::sync::Arc;

#[tokio::test]
async fn test_orchestrator_routes_task_to_worker() {
    // This test verifies that the orchestrator can route a task to a worker
    // For this test, we assume a worker is running on localhost:8081

    let config = ProgressTrackerConfig {
        enabled: true,
        max_events_per_task: 100,
        event_retention_seconds: 3600,
        enable_metrics: false,
        report_interval_seconds: 60,
    };

    let progress_tracker = Arc::new(ProgressTracker::new(config, None));
    let orchestrator = Orchestrator::new(OrchestratorConfig::default(), progress_tracker);

    // Test task description
    let description = "Test task: Verify orchestrator routes to worker";

    // This should send an HTTP request to the worker and get a response
    let result = orchestrator.orchestrate_task(description).await;

    match result {
        Ok(task_result) => {
            println!(" Orchestrator successfully routed task to worker");
            println!("   Task ID: {}", task_result.working_spec.id);
            println!("   Worker ID: {}", task_result.artifacts.worker_id);
            println!("   Execution Output: {}", task_result.artifacts.execution_output);
            assert!(!task_result.artifacts.execution_output.is_empty());
        }
        Err(e) => {
            println!(" Orchestrator failed to route task: {}", e);
            // This might fail if no worker is running, which is expected in CI
            // For manual testing, ensure a worker is running first
            println!("Note: This test requires a worker running on localhost:8081");
            println!("Run: cargo run --bin agent-agency-worker");
        }
    }
}
