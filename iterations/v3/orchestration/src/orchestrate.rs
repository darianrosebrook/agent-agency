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
    TaskContext as CouncilTaskContext, TaskScope as CouncilTaskScope, TaskSpec as CouncilTaskSpec,
    WorkerOutput as CouncilWorkerOutput,
};
use agent_agency_council::types::{CawsWaiver, ConsensusResult, FinalVerdict};
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

    // Evaluate task with council (may involve LLM calls) - protect with circuit breaker
    let result: ConsensusResult = if let Some(circuit_breaker) = _council_circuit_breaker {
        circuit_breaker
            .execute(|| async {
                coordinator
                    .evaluate_task(to_task_spec(desc))
                    .await
                    .context("council evaluation failed")
            })
            .await
            .context("council evaluation failed due to circuit breaker")?
    } else {
        coordinator
            .evaluate_task(to_task_spec(desc))
            .await
            .context("council evaluation failed")?
    };

    // Persist consensus result to database - protect with circuit breaker
    if let Some(circuit_breaker) = _db_circuit_breaker {
        circuit_breaker
            .execute(|| async {
                writer
                    .persist_consensus(&result)
                    .await
                    .context("persisting final verdict failed")
            })
            .await
            .context("database persistence failed due to circuit breaker")?
    } else {
        writer
            .persist_consensus(&result)
            .await
            .context("persisting final verdict failed")?
    }

    orch_emitter
        .orchestrate_exit(&desc.task_id, "completed")
        .await?;

    emitter.on_final_verdict(result.task_id, &result.final_verdict);
    Ok(result.final_verdict)
}

/// Orchestrator that routes tasks to workers
pub struct Orchestrator {
    client: reqwest::Client,
    worker_endpoint: String,
    progress_tracker: Arc<ProgressTracker>,
}

impl Orchestrator {
    pub fn new(
        config: OrchestratorConfig,
        progress_tracker: Arc<ProgressTracker>,
    ) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        let worker_endpoint = std::env::var("AGENT_AGENCY_WORKER_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:8081".to_string());

        Self {
            client,
            worker_endpoint,
            progress_tracker,
        }
    }

    /// Route a task description to a worker for execution
    pub async fn orchestrate_task(
        &self,
        description: &str,
    ) -> Result<TaskExecutionResult, Box<dyn std::error::Error + Send + Sync>> {
        let task_id = uuid::Uuid::new_v4();

        // Start progress tracking
        self.progress_tracker.start_execution(task_id, "api-submitted".to_string()).await?;

        // Create task execution request
        let request = serde_json::json!({
            "task_id": task_id,
            "description": description,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        let execute_url = format!("{}/execute", self.worker_endpoint.trim_end_matches('/'));

        // Send task to worker
        let response = self.client
            .post(&execute_url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Worker execution failed: {} - {}", response.status(), error_text).into());
        }

        // Parse worker response
        let worker_result: serde_json::Value = response.json().await?;

        // Extract execution details
        let execution_time_ms = worker_result
            .get("execution_time_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let success = worker_result
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Create execution artifacts
        let artifacts = ExecutionArtifacts {
            files_created: vec![],
            files_modified: vec![],
            files_deleted: vec![],
            execution_output: worker_result
                .get("output")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            execution_time_ms,
            worker_id: worker_result
                .get("worker_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
        };

        // Create working spec (minimal for now)
        let working_spec = WorkingSpec {
            id: task_id.to_string(),
            title: format!("Task {}", task_id),
            description: description.to_string(),
            mode: TaskMode::Feature,
            scope: TaskScope {
                in_scope: vec![".".to_string()],
                out_scope: vec![],
            },
            invariants: vec![],
            acceptance_criteria: vec![],
            non_functional_requirements: None,
            validation_results: None,
            metadata: Some(WorkingSpecMetadata {
                created_at: chrono::Utc::now(),
                created_by: "api".to_string(),
                risk_tier: 2,
                change_budget: ChangeBudget {
                    max_files: 10,
                    max_loc: 100,
                },
                blast_radius: BlastRadius {
                    modules: vec![],
                    data_migration: false,
                },
                operational_rollback_slo: "5m".to_string(),
            }),
        };

        // Complete progress tracking
        self.progress_tracker.complete_execution(task_id, success).await?;

        Ok(TaskExecutionResult {
            working_spec,
            artifacts,
            quality_report: None,
        })
    }
}

/// Configuration for the orchestrator
#[derive(Debug, Default)]
pub struct OrchestratorConfig;

/// Result of task execution
#[derive(Debug)]
pub struct TaskExecutionResult {
    pub working_spec: WorkingSpec,
    pub artifacts: ExecutionArtifacts,
    pub quality_report: Option<crate::quality::QualityReport>,
}
