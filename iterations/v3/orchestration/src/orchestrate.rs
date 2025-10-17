use crate::caws_runtime::{CawsRuntimeValidator, DefaultValidator, DiffStats, TaskDescriptor, WorkingSpec};
use crate::adapter::build_short_circuit_verdict;
use anyhow::Result;
use council::contracts as api;
use council::ConsensusCoordinator;
use crate::persistence::VerdictWriter;
use apple_silicon as silicon;
use council::coordinator::ProvenanceEmitter;
use crate::provenance::OrchestrationProvenanceEmitter;

fn to_task_spec(desc: &TaskDescriptor) -> council::types::TaskSpec {
    // Expanded mapping to include id/name/risk_tier/scope and deterministic seeds placeholder
    let now = chrono::Utc::now();
    council::types::TaskSpec {
        id: uuid::Uuid::new_v4(),
        name: Some(format!("task-{}", desc.task_id)),
        description: Some("Orchestrated task".to_string()),
        risk_tier: desc.risk_tier as u8,
        scope: desc.scope_in.clone(),
        acceptance_criteria: desc
            .acceptance
            .clone()
            .unwrap_or_default(),
        seeds: Some(council::types::Seeds {
            // Use fixed defaults; orchestration should override per-call for determinism in tests
            time_seed: Some(now.timestamp_millis() as u64),
            uuid_seed: Some(0),
            random_seed: Some(42),
        }),
        created_at: now,
        metadata: desc.metadata.clone().unwrap_or_default(),
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
    emitter: &dyn ProvenanceEmitter,
    orch_emitter: &OrchestrationProvenanceEmitter,
) -> Result<api::FinalVerdict> {
    // Plan resource allocation (heuristic) for council evaluation
    let tier = match desc.risk_tier { 1 => silicon::Tier::T1, 2 => silicon::Tier::T2, _ => silicon::Tier::T3 };
    let sensors = silicon::adaptive_resource_manager::SystemSensors::detect();
    let registry = silicon::adaptive_resource_manager::AppleModelRegistry::from_path(std::path::Path::new(
        std::env::var("ARM_MODEL_REGISTRY_JSON").unwrap_or_default().as_str(),
    )).unwrap_or_else(|| silicon::adaptive_resource_manager::AppleModelRegistry::from_config(
        silicon::adaptive_resource_manager::AppleModelRegistryConfig { models: std::collections::HashMap::new() }
    ));
    let planner = silicon::adaptive_resource_manager::SimplePlanner::new(sensors, registry);
    let req = silicon::adaptive_resource_manager::AllocationRequest {
        model: "gemma-3n-judge".to_string(),
        supported_precisions: vec![silicon::Precision::Int8, silicon::Precision::Fp16],
        preferred_devices: vec![],
        tier,
        latency_slo_ms: if matches!(tier, silicon::Tier::T1) { 30 } else if matches!(tier, silicon::Tier::T2) { 100 } else { 200 },
        max_batch_size: if matches!(tier, silicon::Tier::T1) { 2 } else { 16 },
        workload_hint: silicon::WorkloadHint::JudgeLatencySensitive,
    };
    let allocation = planner.plan(&req);
    tracing::info!(target: "arm", device = ?allocation.device, precision = ?allocation.precision, batch = allocation.batch_size, est_ms = allocation.expected_latency_ms, "ARM plan created for council evaluation");
    // Lifecycle enter provenance
    orch_emitter.orchestrate_enter(&desc.task_id, &desc.scope_in, deterministic);
    let validator = DefaultValidator;
    let validation = validator
        .validate(spec, desc, diff, &[], &[], tests_added, deterministic, vec![])
        .await
        .expect("validation failed");

    if let Some(short) = build_short_circuit_verdict(&validation) {
        orch_emitter.validation_result(&desc.task_id, true);
        // Emit provenance for validation-based short-circuit decision
        emitter.on_judge_verdict(uuid::Uuid::nil(), "runtime-validator", 1.0, "short_circuit", 1.0);
        let result = coordinator
            .evaluate_task_with_validation(
                to_task_spec(desc),
                short.clone(),
            )
            .await?;
        writer.persist_verdict(&desc.task_id, &result.final_verdict).await.ok();
        emitter.on_final_verdict(result.task_id, &result.final_verdict);
        orch_emitter.orchestrate_exit(&desc.task_id, "short_circuit");
        return Ok(result.final_verdict);
    }

    let result = coordinator
        .evaluate_task(to_task_spec(desc))
        .await?;
    writer.persist_verdict(&desc.task_id, &result.final_verdict).await.ok();
    emitter.on_final_verdict(result.task_id, &result.final_verdict);
    orch_emitter.orchestrate_exit(&desc.task_id, "completed");
    Ok(result.final_verdict)
}
