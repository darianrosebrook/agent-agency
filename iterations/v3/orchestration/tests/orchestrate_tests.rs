use agent_agency_council::types::FinalVerdict;
use council::{ConsensusCoordinator, CouncilConfig, NoopEmitter};
use orchestration::caws_runtime::*;
use orchestration::orchestrate::orchestrate_task;
use orchestration::persistence::InMemoryWriter;
use orchestration::provenance::OrchestrationProvenanceEmitter;

#[tokio::test]
async fn short_circuit_reject_path() {
    let spec = WorkingSpec {
        risk_tier: 2,
        scope_in: vec!["src/".into()],
        change_budget_max_files: 1,
        change_budget_max_loc: 1,
    };
    let desc = TaskDescriptor {
        task_id: "T-99".into(),
        scope_in: vec!["src/".into()],
        risk_tier: 2,
        acceptance: Some(vec!["A1: does X".into()]),
        metadata: None,
    };
    let diff = DiffStats {
        files_changed: 10,
        lines_changed: 100,
        touched_paths: vec!["outside/file.rs".into()],
    };
    let mut coord = ConsensusCoordinator::new(CouncilConfig::default());
    let writer = InMemoryWriter;
    let emitter = OrchestrationProvenanceEmitter::default();
    let verdict = orchestrate_task(
        &spec,
        &desc,
        &diff,
        false,
        false,
        &mut coord,
        &writer,
        &NoopEmitter,
        &emitter,
        None,
        None,
    )
    .await
    .unwrap();
    assert!(matches!(verdict, FinalVerdict::Rejected { .. }));
}
