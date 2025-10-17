use crate::caws_runtime::{CawsRuntimeValidator, DefaultValidator, DiffStats, TaskDescriptor, WorkingSpec};
use crate::adapter::build_short_circuit_verdict;
use anyhow::Result;
use council::contracts as api;
use council::ConsensusCoordinator;
use crate::persistence::VerdictWriter;

fn to_task_spec(desc: &TaskDescriptor) -> council::types::TaskSpec {
    // Minimal mapping stub; extend as TaskSpec evolves
    council::types::TaskSpec {
        id: uuid::Uuid::new_v4(),
        name: Some(format!("task-{}", desc.task_id)),
        description: None,
        risk_tier: desc.risk_tier as u8,
        scope: desc.scope_in.clone(),
        acceptance_criteria: vec![],
        created_at: chrono::Utc::now(),
        ..Default::default()
    }
}

/// Orchestration entry point (simplified):
/// 1) Run runtime validation
/// 2) Short-circuit reject if needed
/// 3) Else run council evaluation
pub async fn orchestrate_task(
    spec: &WorkingSpec,
    desc: &TaskDescriptor,
    diff: &DiffStats,
    tests_added: bool,
    deterministic: bool,
    coordinator: &ConsensusCoordinator,
    writer: &dyn VerdictWriter,
) -> Result<api::FinalVerdict> {
    let validator = DefaultValidator;
    let validation = validator
        .validate(spec, desc, diff, tests_added, deterministic, vec![])
        .await
        .expect("validation failed");

    if let Some(short) = build_short_circuit_verdict(&validation) {
        let result = coordinator
            .evaluate_task_with_validation(
                to_task_spec(desc),
                short.clone(),
            )
            .await?;
        writer.persist_verdict(&desc.task_id, &result.final_verdict).await.ok();
        return Ok(result.final_verdict);
    }

    let result = coordinator
        .evaluate_task(to_task_spec(desc))
        .await?;
    writer.persist_verdict(&desc.task_id, &result.final_verdict).await.ok();
    Ok(result.final_verdict)
}
