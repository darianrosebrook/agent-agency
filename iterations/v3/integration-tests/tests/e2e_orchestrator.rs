#![allow(dead_code)]
//! End-to-end orchestration harness (in progress).
//!
//! Provides a first concrete scenario exercising the orchestrator, council,
//! and provenance layers together. Additional scenarios will be added as the
//! harness matures.

use std::collections::BTreeMap;

use agent_agency_council::{
    coordinator::{ConsensusCoordinator, NoopEmitter},
    CouncilConfig,
};
use agent_agency_council::types::FinalVerdict;
use orchestration::{
    caws_runtime::{DiffStats, TaskDescriptor, WorkingSpec},
    orchestrate::orchestrate_task,
    persistence::InMemoryWriter,
    provenance::OrchestrationProvenanceEmitter,
};

struct EndToEndHarness {
    provenance: OrchestrationProvenanceEmitter,
}

impl EndToEndHarness {
    fn new() -> Self {
        Self {
            provenance: OrchestrationProvenanceEmitter::default(),
        }
    }

    fn coordinator(&self) -> ConsensusCoordinator {
        ConsensusCoordinator::new(CouncilConfig::default())
    }

    fn provenance(&self) -> &OrchestrationProvenanceEmitter {
        &self.provenance
    }
}

fn happy_path_spec() -> WorkingSpec {
    WorkingSpec {
        risk_tier: 2,
        scope_in: vec!["src/".into()],
        change_budget_max_files: 5,
        change_budget_max_loc: 300,
    }
}

fn happy_path_descriptor() -> TaskDescriptor {
    let mut metadata = BTreeMap::new();
    metadata.insert("author".into(), "integration-harness".into());
    metadata.insert("ticket".into(), "E2E-ACCEPT".into());

    TaskDescriptor {
        task_id: "E2E-ACCEPT-001".into(),
        scope_in: vec!["src/lib.rs".into()],
        risk_tier: 2,
        acceptance: Some(vec!["A1: system compiles".into()]),
        metadata: Some(metadata),
    }
}

fn happy_path_diff() -> DiffStats {
    DiffStats {
        files_changed: 1,
        lines_changed: 42,
        touched_paths: vec!["src/lib.rs".into()],
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn e2e_happy_path_accepts_change() {
    let harness = EndToEndHarness::new();
    let mut coordinator = harness.coordinator();
    let writer = InMemoryWriter;
    let council_emitter = NoopEmitter;

    let verdict = orchestrate_task(
        &happy_path_spec(),
        &happy_path_descriptor(),
        &happy_path_diff(),
        true,
        true,
        &mut coordinator,
        &writer,
        &council_emitter,
        harness.provenance(),
        None,
        None,
    )
    .await
    .expect("orchestration should complete");

    match verdict {
        FinalVerdict::Accepted { summary, .. } => {
            assert!(
                summary.contains("Orchestrated task"),
                "expected summary to reference orchestrated task, got: {}",
                summary
            );
        }
        other => panic!("expected accepted verdict, got {:?}", other),
    }

    let events = harness
        .provenance()
        .events_for_task("E2E-ACCEPT-001")
        .await;
    assert!(
        events.iter().any(|e| e.event_type == "session_created"),
        "session_created event missing: {:?}",
        events
    );
    assert!(
        events.iter().any(|e| e.event_type == "validation_result"),
        "validation_result event missing"
    );
    assert!(
        events
            .iter()
            .any(|e| e.event_type == "session_completed"
                && e.payload.get("status") == Some(&serde_json::json!("completed"))),
        "session_completed event missing or has wrong status"
    );
}

/// Placeholder for future short-circuit scenario.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "pending CAWS short-circuit scenario wiring"]
async fn e2e_short_circuit_rejects_change() {
    todo!("Trigger runtime validation failure and assert short-circuit verdict + provenance");
}

/// Placeholder for resilience and retry scenario.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "pending retry injection harness"]
async fn e2e_resilience_retries_persist() {
    todo!("Inject transient persistence failures and verify retries succeed");
}

/// Placeholder for latency benchmarking scenario.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "pending benchmark harness"]
async fn e2e_benchmark_collects_metrics() {
    todo!("Execute multiple orchestration runs and produce benchmark artefact");
}
