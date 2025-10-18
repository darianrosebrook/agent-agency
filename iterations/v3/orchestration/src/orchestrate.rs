use crate::adapter::build_short_circuit_verdict;
use crate::caws_runtime::{
    CawsRuntimeValidator, DefaultValidator, DiffStats, TaskDescriptor, WorkingSpec,
};
use crate::persistence::VerdictWriter;
use crate::provenance::OrchestrationProvenanceEmitter;
use agent_agency_apple_silicon::{
    adaptive_resource_manager::{
        AppleModelRegistry, AppleModelRegistryConfig, SimplePlanner, SystemSensors,
    },
    AllocationPlanner,
};
use agent_agency_council::coordinator::{ConsensusCoordinator, ProvenanceEmitter};
use agent_agency_council::models::{
    AcceptanceCriterion as CouncilAcceptanceCriterion, Environment as CouncilEnvironment,
    RiskTier as CouncilRiskTier, SelfAssessment as CouncilSelfAssessment,
    TaskContext as CouncilTaskContext, TaskScope as CouncilTaskScope,
    TaskSpec as CouncilTaskSpec, WorkerOutput as CouncilWorkerOutput,
};
use agent_agency_council::types::{FinalVerdict, ConsensusResult, CawsWaiver};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};

fn map_risk_tier(tier: u8) -> CouncilRiskTier {
    match tier {
        1 => CouncilRiskTier::Tier1,
        2 => CouncilRiskTier::Tier2,
        3 => CouncilRiskTier::Tier3,
        _ => CouncilRiskTier::Tier3,
    }
}

fn to_task_spec(desc: &TaskDescriptor) -> CouncilTaskSpec {
    CouncilTaskSpec {
        id: uuid::Uuid::new_v4(),
        title: format!("task-{}", desc.task_id),
        description: "Orchestrated task".to_string(),
        risk_tier: map_risk_tier(desc.risk_tier),
        scope: CouncilTaskScope {
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
            .map(|criterion| CouncilAcceptanceCriterion {
                id: uuid::Uuid::new_v4().to_string(),
                description: criterion,
            })
            .collect(),
        context: CouncilTaskContext {
            workspace_root: ".".to_string(),
            git_branch: "main".to_string(),
            recent_changes: vec![],
            dependencies: HashMap::new(),
            environment: CouncilEnvironment::Development,
        },
        worker_output: CouncilWorkerOutput {
            content: String::new(),
            files_modified: vec![],
            rationale: String::new(),
            self_assessment: CouncilSelfAssessment {
                caws_compliance: 0.0,
                quality_score: 0.0,
                confidence: 0.0,
                concerns: vec![],
                improvements: vec![],
                estimated_effort: None,
            },
            metadata: HashMap::new(),
        },
        caws_spec: None,
    }
}

fn record_arm_plan(desc: &TaskDescriptor) {
    let tier = match desc.risk_tier {
        1 => agent_agency_apple_silicon::Tier::T1,
        2 => agent_agency_apple_silicon::Tier::T2,
        _ => agent_agency_apple_silicon::Tier::T3,
    };

    let sensors = SystemSensors::detect();
    let registry = AppleModelRegistry::from_path(std::path::Path::new(
        std::env::var("ARM_MODEL_REGISTRY_JSON")
            .unwrap_or_default()
            .as_str(),
    ))
    .unwrap_or_else(|| {
        AppleModelRegistry::from_config(AppleModelRegistryConfig {
            models: HashMap::new(),
        })
    });
    let planner = SimplePlanner::new(sensors, registry);
    let req = agent_agency_apple_silicon::AllocationRequest {
        model: "gemma-3n-judge".to_string(),
        supported_precisions: vec![
            agent_agency_apple_silicon::Precision::Int8,
            agent_agency_apple_silicon::Precision::Fp16,
        ],
        preferred_devices: vec![],
        tier,
        latency_slo_ms: match tier {
            agent_agency_apple_silicon::Tier::T1 => 30,
            agent_agency_apple_silicon::Tier::T2 => 100,
            agent_agency_apple_silicon::Tier::T3 => 200,
        },
        max_batch_size: match tier {
            agent_agency_apple_silicon::Tier::T1 => 2,
            agent_agency_apple_silicon::Tier::T2 => 8,
            agent_agency_apple_silicon::Tier::T3 => 16,
        },
        workload_hint: agent_agency_apple_silicon::WorkloadHint::JudgeLatencySensitive,
    };
    let allocation = planner.plan(&req);
    info!(
        target: "arm",
        task_id = %desc.task_id,
        device = ?allocation.device,
        precision = ?allocation.precision,
        batch = allocation.batch_size,
        est_ms = allocation.expected_latency_ms,
        "planned Apple Silicon allocation"
    );
}

#[instrument(skip_all, fields(task_id = %desc.task_id, risk_tier = desc.risk_tier))]
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
    _council_circuit_breaker: Option<&Arc<agent_agency_resilience::CircuitBreaker>>,
    _db_circuit_breaker: Option<&Arc<agent_agency_resilience::CircuitBreaker>>,
) -> Result<FinalVerdict> {
    record_arm_plan(desc);
    orch_emitter
        .orchestrate_enter(&desc.task_id, &desc.scope_in, deterministic)
        .await?;

    let validator = DefaultValidator;
    let validation = validator
        .validate(
            spec,
            desc,
            diff,
            &[],
            &[],
            tests_added,
            deterministic,
            vec![],
        )
        .await
        .context("CAWS runtime validation failed")?;

    let short_circuit = build_short_circuit_verdict(&validation);
    orch_emitter
        .validation_result(&desc.task_id, short_circuit.is_some())
        .await?;

    if let Some(ref verdict) = short_circuit {
        warn!(
            target: "orchestrator",
            task_id = %desc.task_id,
            "validation produced short-circuit verdict: {:?}",
            verdict
        );
        emitter.on_judge_verdict(
            uuid::Uuid::nil(),
            "runtime-validator",
            1.0,
            "short_circuit",
            1.0,
        );
    }

    let result: ConsensusResult = coordinator
        .evaluate_task(to_task_spec(desc))
        .await
        .context("council evaluation failed")?;

    writer
        .persist_consensus(&result)
        .await
        .context("persisting final verdict failed")?;

    orch_emitter
        .orchestrate_exit(&desc.task_id, "completed")
        .await?;

    emitter.on_final_verdict(result.task_id, &result.final_verdict);
    Ok(result.final_verdict)
}
