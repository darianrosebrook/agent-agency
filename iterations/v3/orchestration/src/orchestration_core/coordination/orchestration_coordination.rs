//! Orchestration coordination logic
//!
//! This module handles the high-level coordination of the orchestration process,
//! orchestrating validation, execution, council review, and final verdict combination.

use anyhow::{Context, Result};
use tracing::{debug, info, warn};

/// Main orchestration function that coordinates the entire task execution process
pub async fn orchestrate_task(
    spec: &crate::caws_runtime::WorkingSpec,
    desc: &crate::caws_runtime::TaskDescriptor,
    diff: &crate::caws_runtime::DiffStats,
    tests_added: bool,
    deterministic: bool,
    coordinator: &mut agent_agency_council::ConsensusCoordinator,
    writer: &dyn crate::persistence::VerdictWriter,
    emitter: &dyn agent_agency_council::ProvenanceEmitter,
    orch_emitter: &crate::provenance::OrchestrationProvenanceEmitter,
    _council_circuit_breaker: Option<&std::sync::Arc<agent_agency_resilience::CircuitBreaker>>,
    _db_circuit_breaker: Option<&std::sync::Arc<agent_agency_resilience::CircuitBreaker>>,
) -> Result<agent_agency_council::types::FinalVerdict> {
    use super::validation::validate_orchestration_task;
    use super::execution::{execute_task_with_workers, review_artifacts_with_judges, combine_verdicts};
    use super::types::record_arm_plan;

    record_arm_plan(desc);
    orch_emitter
        .orchestrate_enter(&desc.task_id, &desc.scope_in, deterministic)
        .await?;

    // Step 1: Validate the task
    let validation = validate_orchestration_task(
        spec, desc, diff, tests_added, deterministic, orch_emitter, emitter
    ).await?;

    let short_circuit = crate::adapter::build_short_circuit_verdict(&validation);
    if let Some(verdict) = short_circuit {
        warn!(
            target: "orchestrator",
            task_id = %desc.task_id,
            "validation produced short-circuit verdict: {:?}",
            verdict
        );
        return Ok(verdict);
    }

    // Step 2: Evaluate task with council (may involve LLM calls) - protect with circuit breaker
    let consensus_result: agent_agency_council::types::ConsensusResult;
    if let Some(circuit_breaker) = _council_circuit_breaker {
        consensus_result = circuit_breaker
            .execute(|| async {
                coordinator
                    .evaluate_task(super::types::to_task_spec(desc))
                    .await
                    .context("council evaluation failed")
            })
            .await
            .context("council evaluation failed due to circuit breaker")?;
    } else {
        consensus_result = coordinator
            .evaluate_task(super::types::to_task_spec(desc))
            .await
            .context("council evaluation failed")?;
    };

    // Step 3: Persist consensus result to database - protect with circuit breaker
    if let Some(circuit_breaker) = _db_circuit_breaker {
        circuit_breaker
            .execute(|| async {
                writer
                    .persist_consensus(&consensus_result)
                    .await
                    .context("persisting final verdict failed")
            })
            .await
            .context("database persistence failed due to circuit breaker")?;
    } else {
        writer
            .persist_consensus(&consensus_result)
            .await
            .context("persisting final verdict failed")?;
    }

    // Step 4: Execute task with workers
    let artifacts = execute_task_with_workers(spec, desc).await?;
    orch_emitter
        .task_execution(&desc.task_id, artifacts.execution_id, artifacts.worker_id)
        .await?;

    // Step 5: Review artifacts with judges
    let artifact_verdict = review_artifacts_with_judges(&artifacts, spec, desc, coordinator).await?;
    orch_emitter
        .judge_review(&desc.task_id, artifact_verdict.confidence)
        .await?;

    // Step 6: Combine verdicts
    let final_verdict = combine_verdicts(consensus_result.final_verdict, artifact_verdict);

    // Step 7: Persist final verdict
    writer
        .persist_final_verdict(&final_verdict)
        .await
        .context("persisting final verdict failed")?;

    orch_emitter
        .orchestrate_complete(&desc.task_id, &final_verdict.decision, final_verdict.confidence)
        .await?;

    info!("Orchestration completed for task {} with decision: {}", desc.task_id, final_verdict.decision);
    Ok(final_verdict)
}
