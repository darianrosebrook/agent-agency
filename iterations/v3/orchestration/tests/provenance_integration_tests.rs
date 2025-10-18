use council::coordinator::NoopEmitter;
use council::{ConsensusCoordinator, CouncilConfig};
use orchestration::caws_runtime::{DiffStats, TaskDescriptor, WorkingSpec};
use orchestration::orchestrate::orchestrate_task;
use orchestration::persistence::InMemoryWriter;
use orchestration::provenance::OrchestrationProvenanceEmitter;

#[tokio::test]
async fn provenance_events_emitted_on_short_circuit() {
    // Arrange a spec that will short-circuit: no tests added and not deterministic with tight budgets
    let spec = WorkingSpec {
        risk_tier: 2,
        scope_in: vec!["src/".into()],
        change_budget_max_files: 1,
        change_budget_max_loc: 1,
    };
    let desc = TaskDescriptor {
        task_id: "T-SHORT".into(),
        scope_in: vec!["src/".into()],
        risk_tier: 2,
        acceptance: None,
        metadata: None,
    };
    let diff = DiffStats {
        files_changed: 2,
        lines_changed: 100,
        touched_paths: vec!["src/lib.rs".into()],
    };
    let mut coord = ConsensusCoordinator::new(CouncilConfig::default());
    let writer = InMemoryWriter;

    let orch_emitter = OrchestrationProvenanceEmitter::default();

    // Council emitter (noop for this test)
    let council_emitter = NoopEmitter;

    // Act
    let _ = orchestrate_task(
        &spec,
        &desc,
        &diff,
        /*tests_added*/ false,
        /*deterministic*/ false,
        &mut coord,
        &writer,
        &council_emitter,
        &orch_emitter,
        None,
        None,
    )
    .await
    .unwrap();

    // Assert orchestration lifecycle events present
    let events = orch_emitter.events_for_task("T-SHORT").await;
    assert!(events.iter().any(|e| e.event_type == "session_created"));
    assert!(events.iter().any(|e| e.event_type == "validation_result"
        && e.payload.get("passed") == Some(&serde_json::json!(true))));
    assert!(events.iter().any(|e| e.event_type == "session_completed"
        && e.payload.get("status") == Some(&serde_json::json!("completed"))));
}

#[tokio::test]
async fn provenance_events_emitted_on_normal_completion() {
    // Arrange a spec that will not short-circuit: within budget, tests added, deterministic
    let spec = WorkingSpec {
        risk_tier: 2,
        scope_in: vec!["src/".into()],
        change_budget_max_files: 10,
        change_budget_max_loc: 1000,
    };
    let desc = TaskDescriptor {
        task_id: "T-NORMAL".into(),
        scope_in: vec!["src/".into()],
        risk_tier: 2,
        acceptance: None,
        metadata: None,
    };
    let diff = DiffStats {
        files_changed: 0,
        lines_changed: 0,
        touched_paths: vec!["src/lib.rs".into()],
    };
    let mut coord = ConsensusCoordinator::new(CouncilConfig::default());
    let writer = InMemoryWriter;

    let orch_emitter = OrchestrationProvenanceEmitter::default();
    let council_emitter = NoopEmitter;

    let _ = orchestrate_task(
        &spec,
        &desc,
        &diff,
        /*tests_added*/ true,
        /*deterministic*/ true,
        &mut coord,
        &writer,
        &council_emitter,
        &orch_emitter,
        None,
        None,
    )
    .await
    .unwrap();

    let events = orch_emitter.events_for_task("T-NORMAL").await;
    assert!(events.iter().any(|e| e.event_type == "session_created"));
    assert!(events.iter().any(|e| e.event_type == "session_completed"));
}
