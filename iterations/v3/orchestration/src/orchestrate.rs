use crate::adapter::build_short_circuit_verdict;
use crate::caws_runtime::{
    CawsRuntimeValidator, DefaultValidator, DiffStats, TaskDescriptor, WorkingSpec,
};
use crate::persistence::VerdictWriter;
use crate::provenance::OrchestrationProvenanceEmitter;
use crate::planning::types::ExecutionArtifacts;
use crate::tracking::ProgressTracker;
use agent_agency_apple_silicon::{
    adaptive_resource_manager::{
        AppleModelRegistry, AppleModelRegistryConfig, SimplePlanner, SystemSensors,
    },
    AllocationPlanner,
};
use agent_agency_contracts::working_spec::{
    TaskMode, TaskScope, ChangeBudget, BlastRadius, WorkingSpecMetadata,
};
use agent_agency_council::coordinator::{ConsensusCoordinator, ProvenanceEmitter};
use agent_agency_council::models::{
    AcceptanceCriterion as CouncilAcceptanceCriterion, Environment as CouncilEnvironment,
    RiskTier as CouncilRiskTier, SelfAssessment as CouncilSelfAssessment,
    TaskContext as CouncilTaskContext, TaskScope as CouncilTaskScope, TaskSpec as CouncilTaskSpec,
    WorkerOutput as CouncilWorkerOutput,
};
use agent_agency_council::types::{CawsWaiver, ConsensusResult, FinalVerdict};
use agent_agency_resilience::{CircuitBreaker, CircuitBreakerConfig, retry_with_backoff, RetryConfig};
use agent_agency_database::DatabaseClient;
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

/// Worker registry trait for service discovery (P0 requirement)
#[async_trait::async_trait]
pub trait WorkerRegistry: Send + Sync {
    /// Get worker endpoint for a given worker ID
    async fn get_worker_endpoint(&self, worker_id: &str) -> Result<String>;
    /// Report worker health status
    async fn report_worker_health(&self, worker_id: &str, healthy: bool) -> Result<()>;
}

/// Simple static worker registry implementation
pub struct StaticWorkerRegistry {
    default_endpoint: String,
}

impl StaticWorkerRegistry {
    pub fn new(default_endpoint: String) -> Self {
        Self { default_endpoint }
    }
}

#[async_trait::async_trait]
impl WorkerRegistry for StaticWorkerRegistry {
    async fn get_worker_endpoint(&self, _worker_id: &str) -> Result<String> {
        Ok(self.default_endpoint.clone())
    }

    async fn report_worker_health(&self, _worker_id: &str, _healthy: bool) -> Result<()> {
        // Static registry doesn't track health
        Ok(())
    }
}

/// Orchestrator that routes tasks to workers (P0: real worker execution path)
pub struct Orchestrator {
    client: reqwest::Client,
    worker_registry: Arc<dyn WorkerRegistry>,
    circuit_breakers: Arc<std::sync::RwLock<HashMap<String, Arc<CircuitBreaker>>>>,
    retry_config: RetryConfig,
    progress_tracker: Arc<ProgressTracker>,
    db_client: Option<Arc<DatabaseClient>>, // Optional for backward compatibility
}

impl Orchestrator {
    pub fn new(
        config: OrchestratorConfig,
        progress_tracker: Arc<ProgressTracker>,
    ) -> Self {
        Self::new_with_dependencies(
            config,
            progress_tracker,
            None, // Use default worker registry
            None, // Use default circuit breaker config
            None, // Use default retry config
            None, // Use default DB client
        )
    }

    /// Create orchestrator with explicit dependencies (P0: real worker execution path)
    pub fn new_with_dependencies(
        _config: OrchestratorConfig,
        progress_tracker: Arc<ProgressTracker>,
        worker_registry: Option<Arc<dyn WorkerRegistry>>,
        _circuit_breaker_config: Option<CircuitBreakerConfig>,
        retry_config: Option<RetryConfig>,
        db_client: Option<Arc<DatabaseClient>>,
    ) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        let worker_registry = worker_registry.unwrap_or_else(|| {
            let default_endpoint = std::env::var("AGENT_AGENCY_WORKER_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:8081".to_string());
            Arc::new(StaticWorkerRegistry::new(default_endpoint))
        });

        let circuit_breakers = Arc::new(std::sync::RwLock::new(HashMap::new()));

        let retry_config = retry_config.unwrap_or_else(|| RetryConfig {
            max_attempts: 3,
            base_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
            jitter: true,
        });

        Self {
            client,
            worker_registry,
            circuit_breakers,
            retry_config,
            progress_tracker,
            db_client,
        }
    }

    /// Route a task description to a worker for execution (P0: real worker execution path)
    pub async fn orchestrate_task(
        &self,
        description: &str,
        execution_mode: Option<&str>,
    ) -> Result<TaskExecutionResult, Box<dyn std::error::Error + Send + Sync>> {
        let task_id = uuid::Uuid::new_v4();

        // Start progress tracking
        self.progress_tracker.start_execution(task_id, "api-submitted".to_string()).await?;

        // P0: Audit trail - Task enqueued
        if let Some(ref db_client) = self.db_client {
            db_client.create_task_audit_event(
                task_id,
                "orchestration",
                "system",
                "enqueued",
                serde_json::json!({
                    "description": description,
                    "execution_mode": execution_mode,
                    "stage": "worker_routing"
                }),
            ).await.map_err(|e| format!("Failed to audit task enqueue: {}", e))?;
        }

        // Get worker endpoint (MVP: static discovery)
        let worker_id = "default-worker"; // In future, this could be selected based on task requirements
        let worker_endpoint = self.worker_registry.get_worker_endpoint(worker_id).await
            .map_err(|e| format!("Failed to get worker endpoint: {}", e))?;

        // Create task execution request
        let mut request = serde_json::json!({
            "task_id": task_id,
            "prompt": description,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        // Add execution mode if provided
        if let Some(mode) = execution_mode {
            request["execution_mode"] = serde_json::Value::String(mode.to_string());
        }

        let execute_url = format!("{}/execute", worker_endpoint.trim_end_matches('/'));

        // P0: Get or create circuit breaker for this worker
        let circuit_breaker = self.get_or_create_circuit_breaker(worker_id).await;

        // P0: Execute with retry/backoff + circuit breaker
        let worker_result = self.execute_with_resilience(
            task_id,
            worker_id,
            &execute_url,
            &request,
            &circuit_breaker,
        ).await?;

        // Extract execution details from worker response
        let execution_time_ms = worker_result
            .get("execution_time_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let success = worker_result
            .get("exit_code")
            .and_then(|v| v.as_i64())
            .map(|code| code == 0)
            .unwrap_or(false);

        let execution_output = worker_result
            .get("stdout")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let worker_id_response = worker_result
            .get("worker_id")
            .and_then(|v| v.as_str())
            .unwrap_or(worker_id)
            .to_string();

        // Create execution artifacts
        let artifacts = ExecutionArtifacts {
            files_created: vec![],
            files_modified: vec![],
            files_deleted: vec![],
            execution_output,
            execution_time_ms,
            worker_id: worker_id_response,
        };

        // TODO: Implement comprehensive working specification generation
        // - Parse task requirements and generate detailed acceptance criteria
        // - Analyze codebase structure and determine appropriate scope boundaries
        // - Identify risk tier based on impact analysis and dependencies
        // - Generate specific test requirements and quality gates
        // - Create performance budgets and monitoring requirements
        // - Identify security and compliance requirements
        // - Generate deployment and rollback specifications
        // - Create documentation and maintenance requirements
        // - Establish success metrics and completion criteria
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

    /// Get or create circuit breaker for a worker (P0: real worker execution path)
    async fn get_or_create_circuit_breaker(&self, worker_id: &str) -> Arc<CircuitBreaker> {
        // Check if we already have a circuit breaker for this worker
        {
            let breakers = self.circuit_breakers.read().unwrap();
            if let Some(breaker) = breakers.get(worker_id) {
                return breaker.clone();
            }
        }

        // Create new circuit breaker for this worker
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            recovery_timeout_ms: 30000, // 30 seconds
            expected_exceptions: vec![],
            monitoring_enabled: true,
        };

        let breaker = Arc::new(CircuitBreaker::new(config));
        self.circuit_breakers.write().unwrap().insert(worker_id.to_string(), breaker.clone());
        breaker
    }

    /// Execute worker request with resilience (retry/backoff + circuit breaker) (P0 requirement)
    async fn execute_with_resilience(
        &self,
        task_id: Uuid,
        worker_id: &str,
        url: &str,
        request_body: &serde_json::Value,
        circuit_breaker: &Arc<CircuitBreaker>,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        // P0: Audit trail - Execution attempt started
        if let Some(ref db_client) = self.db_client {
            db_client.create_task_audit_event(
                task_id,
                "worker",
                worker_id,
                "exec_attempt",
                serde_json::json!({
                    "worker_endpoint": url,
                    "stage": "execution_attempt"
                }),
            ).await.ok(); // Don't fail if audit fails
        }

        let mut attempt = 0;
        let result = retry_with_backoff(
            &self.retry_config,
            || async {
                attempt += 1;

                // Check circuit breaker
                if let Err(_) = circuit_breaker.call(|| async { Ok(()) }).await {
                    // P0: Audit trail - Circuit breaker open
                    if let Some(ref db_client) = self.db_client {
                        db_client.create_task_audit_event(
                            task_id,
                            "worker",
                            worker_id,
                            "circuit_breaker_open",
                            serde_json::json!({
                                "attempt": attempt,
                                "stage": "circuit_breaker_open"
                            }),
                        ).await.ok();
                    }
                    return Err("Circuit breaker is open".into());
                }

                // Make HTTP request
                match self.client
                    .post(url)
                    .json(request_body)
                    .send()
                    .await
                {
                    Ok(response) => {
                        if response.status().is_success() {
                            // P0: Audit trail - Successful execution
                            if let Some(ref db_client) = self.db_client {
                                db_client.create_task_audit_event(
                                    task_id,
                                    "worker",
                                    worker_id,
                                    "exec_success",
                                    serde_json::json!({
                                        "attempt": attempt,
                                        "response_status": response.status().as_u16(),
                                        "stage": "execution_success"
                                    }),
                                ).await.ok();
                            }

                            // Report worker health
                            self.worker_registry.report_worker_health(worker_id, true).await.ok();

                            // Parse and return response
                            response.json().await.map_err(|e| e.into())
                        } else {
                            // P0: Audit trail - Execution failed
                            if let Some(ref db_client) = self.db_client {
                                db_client.create_task_audit_event(
                                    task_id,
                                    "worker",
                                    worker_id,
                                    "exec_failure",
                                    serde_json::json!({
                                        "attempt": attempt,
                                        "response_status": response.status().as_u16(),
                                        "stage": "execution_failure"
                                    }),
                                ).await.ok();
                            }

                            // Report worker unhealthy
                            self.worker_registry.report_worker_health(worker_id, false).await.ok();

                            Err(format!("Worker returned error: {}", response.status()).into())
                        }
                    }
                    Err(e) => {
                        // P0: Audit trail - Network/timeout error
                        if let Some(ref db_client) = self.db_client {
                            db_client.create_task_audit_event(
                                task_id,
                                "worker",
                                worker_id,
                                "exec_timeout",
                                serde_json::json!({
                                    "attempt": attempt,
                                    "error": e.to_string(),
                                    "stage": "execution_timeout"
                                }),
                            ).await.ok();
                        }

                        // Report worker unhealthy
                        self.worker_registry.report_worker_health(worker_id, false).await.ok();

                        Err(e.into())
                    }
                }
            }
        ).await;

        match result {
            Ok(response) => Ok(response),
            Err(e) => {
                // P0: Audit trail - Final execution failure
                if let Some(ref db_client) = self.db_client {
                    db_client.create_task_audit_event(
                        task_id,
                        "worker",
                        worker_id,
                        "exec_final_failure",
                        serde_json::json!({
                            "attempts": attempt,
                            "final_error": e.to_string(),
                            "stage": "execution_final_failure"
                        }),
                    ).await.ok();
                }
                Err(e)
            }
        }
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
