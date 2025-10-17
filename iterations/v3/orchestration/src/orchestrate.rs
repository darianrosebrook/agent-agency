use crate::caws_runtime::{CawsRuntimeValidator, DefaultValidator, DiffStats, TaskDescriptor, WorkingSpec};
use crate::adapter::build_short_circuit_verdict;
use anyhow::Result;
use agent_agency_council::types::*;
use agent_agency_council::models::TaskSpec as CouncilTaskSpec;
use agent_agency_council::coordinator::ConsensusCoordinator;
use crate::persistence::VerdictWriter;
use agent_agency_apple_silicon::{
    AllocationPlanner,
    adaptive_resource_manager::{
        SystemSensors, AppleModelRegistry, AppleModelRegistryConfig, SimplePlanner
    }
};
use agent_agency_council::coordinator::ProvenanceEmitter;
use crate::provenance::OrchestrationProvenanceEmitter;
use std::collections::HashMap;

fn to_task_spec(desc: &TaskDescriptor) -> CouncilTaskSpec {
    // Expanded mapping to include id/name/risk_tier/scope and deterministic seeds placeholder
    let now = chrono::Utc::now();
    CouncilTaskSpec {
        id: uuid::Uuid::new_v4(),
        title: format!("task-{}", desc.task_id),
        description: "Orchestrated task".to_string(),
        risk_tier: match desc.risk_tier {
            1 => agent_agency_council::models::RiskTier::Tier1,
            2 => agent_agency_council::models::RiskTier::Tier2,
            3 => agent_agency_council::models::RiskTier::Tier3,
            _ => agent_agency_council::models::RiskTier::Tier3,
        },
        scope: agent_agency_council::models::TaskScope {
            files_affected: desc.scope_in.clone(),
            max_files: None,
            max_loc: None,
            domains: vec![],
        },
        acceptance_criteria: desc
            .acceptance
            .clone()
            .unwrap_or_default()
            .into_iter()
            .map(|criterion| agent_agency_council::models::AcceptanceCriterion {
                id: uuid::Uuid::new_v4().to_string(),
                description: criterion,
            })
            .collect(),
        context: agent_agency_council::models::TaskContext {
            workspace_root: ".".to_string(),
            git_branch: "main".to_string(),
            recent_changes: vec![],
            dependencies: HashMap::new(),
            environment: agent_agency_council::models::Environment::Development,
        },
        worker_output: agent_agency_council::models::WorkerOutput {
            content: "".to_string(),
            files_modified: vec![],
            rationale: "".to_string(),
            self_assessment: agent_agency_council::models::SelfAssessment {
                confidence: 0.0,
                concerns: vec![],
                improvements: vec![],
                caws_compliance: 0.0,
                estimated_effort: None,
                quality_score: 0.0,
            },
            metadata: HashMap::new(),
        },
        caws_spec: None,
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
    coordinator: &mut ConsensusCoordinator,
    writer: &dyn VerdictWriter,
    emitter: &dyn ProvenanceEmitter,
    orch_emitter: &OrchestrationProvenanceEmitter,
) -> Result<FinalVerdict> {
    // Plan resource allocation (heuristic) for council evaluation
    let tier = match desc.risk_tier { 
        1 => agent_agency_apple_silicon::Tier::T1, 
        2 => agent_agency_apple_silicon::Tier::T2, 
        _ => agent_agency_apple_silicon::Tier::T3 
    };
    let sensors = SystemSensors::detect();
    let registry = AppleModelRegistry::from_path(std::path::Path::new(
        std::env::var("ARM_MODEL_REGISTRY_JSON").unwrap_or_default().as_str(),
    )).unwrap_or_else(|| AppleModelRegistry::from_config(
        AppleModelRegistryConfig { models: std::collections::HashMap::new() }
    ));
    let planner = SimplePlanner::new(sensors, registry);
    let req = agent_agency_apple_silicon::AllocationRequest {
        model: "gemma-3n-judge".to_string(),
        supported_precisions: vec![agent_agency_apple_silicon::Precision::Int8, agent_agency_apple_silicon::Precision::Fp16],
        preferred_devices: vec![],
        tier,
        latency_slo_ms: if matches!(tier, agent_agency_apple_silicon::Tier::T1) { 30 } else if matches!(tier, agent_agency_apple_silicon::Tier::T2) { 100 } else { 200 },
        max_batch_size: if matches!(tier, agent_agency_apple_silicon::Tier::T1) { 2 } else { 16 },
        workload_hint: agent_agency_apple_silicon::WorkloadHint::JudgeLatencySensitive,
    };
    let allocation = planner.plan(&req);
    tracing::info!(target: "arm", device = ?allocation.device, precision = ?allocation.precision, batch = allocation.batch_size, est_ms = allocation.expected_latency_ms, "ARM plan created for council evaluation");
    // TODO: Wire a shared ProvenanceService into orchestrate context instead of ad-hoc creation
    if let Ok(cfg_json) = std::env::var("PROVENANCE_CONFIG_JSON") {
        if let Ok(_cfg) = serde_json::from_str::<serde_json::Value>(&cfg_json) {
            // Minimal in-memory or existing storage init would go here; using a no-op on error
            // Append telemetry event for ARM plan
            let payload = serde_json::json!({
                "task_id": desc.task_id,
                "tier": desc.risk_tier,
                "workload_hint": "judge",
                "model": req.model,
                "device": format!("{:?}", allocation.device),
                "precision": format!("{:?}", allocation.precision),
                "batch_size": allocation.batch_size,
                "expected_latency_ms": allocation.expected_latency_ms,
            });
            // NOTE: This assumes a ProvenanceService available; replace with actual instance in real wiring
            // provenance_service.append_event("arm.allocation_planned", payload).await.ok();
        }
    }
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
            .evaluate_task(to_task_spec(desc))
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
