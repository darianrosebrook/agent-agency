use council::coordinator::NoopEmitter;
use council::{ConsensusCoordinator, CouncilConfig};
use orchestration::caws_runtime::{DiffStats, TaskDescriptor, WorkingSpec};
use orchestration::orchestrate::orchestrate_task;
use orchestration::persistence::InMemoryWriter;
use orchestration::provenance::{InMemoryBackend, OrchestrationProvenanceEmitter, ProvEvent};
use parking_lot::Mutex;
use std::sync::Arc;

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
    let coord = ConsensusCoordinator::new(CouncilConfig::default());
    let writer = InMemoryWriter;

    // Backend to capture orchestration provenance
    let backend = Arc::new(InMemoryBackend(Mutex::new(Vec::new())));
    let orch_emitter = OrchestrationProvenanceEmitter::new(backend.clone());

    // Council emitter (noop for this test)
    let council_emitter = NoopEmitter;

    // Act
    let _ = orchestrate_task(
        &spec,
        &desc,
        &diff,
        /*tests_added*/ false,
        /*deterministic*/ false,
        &coord,
        &writer,
        &council_emitter,
        &orch_emitter,
        None,
        None,
    )
    .await
    .unwrap();

    // Assert orchestration lifecycle events present
    let events = backend.0.lock().clone();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, ProvEvent::OrchestrateEnter { .. })),
        "missing OrchestrateEnter"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            ProvEvent::ValidationResult {
                short_circuit: true,
                ..
            }
        )),
        "missing ValidationResult short_circuit"
    );
    assert!(events.iter().any(|e| matches!(e, ProvEvent::OrchestrateExit { outcome, .. } if outcome == "short_circuit")), "missing OrchestrateExit short_circuit");
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
    let coord = ConsensusCoordinator::new(CouncilConfig::default());
    let writer = InMemoryWriter;

    let backend = Arc::new(InMemoryBackend(Mutex::new(Vec::new())));
    let orch_emitter = OrchestrationProvenanceEmitter::new(backend.clone());
    let council_emitter = NoopEmitter;

    let _ = orchestrate_task(
        &spec,
        &desc,
        &diff,
        /*tests_added*/ true,
        /*deterministic*/ true,
        &coord,
        &writer,
        &council_emitter,
        &orch_emitter,
        None,
        None,
    )
    .await
    .unwrap();

    let events = backend.0.lock().clone();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, ProvEvent::OrchestrateEnter { .. })),
        "missing OrchestrateEnter"
    );
    // No ValidationResult short_circuit expected
    assert!(
        events.iter().any(
            |e| matches!(e, ProvEvent::OrchestrateExit { outcome, .. } if outcome == "completed")
        ),
        "missing OrchestrateExit completed"
    );
}
