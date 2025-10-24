use std::collections::HashMap;
use uuid::Uuid;

use crate::adapter::build_short_circuit_verdict;
use crate::caws_runtime::{
    CawsRuntimeValidator, DefaultValidator, DiffStats, TaskDescriptor, WorkingSpec,
};
// NEW: Runtime-validator integration
use caws_runtime_validator::integration::{
    OrchestrationIntegration, DefaultOrchestrationIntegration,
    OrchestrationValidationResult, ExecutionDecision, ExecutionMode,
    Violation as RuntimeViolation, ViolationCode as RuntimeViolationCode,
};
use crate::persistence::VerdictWriter;
use crate::provenance::OrchestrationProvenanceEmitter;
use crate::planning::types::{ExecutionArtifacts, TestResults, CoverageReport, MutationReport, LintReport, TypeCheckReport, ProvenanceRecord};
use crate::planning::agent::{CriterionPriority, RollbackRisk};
use crate::tracking::ProgressTracker;
use agent_agency_apple_silicon::{
    AllocationPlanner, AllocationRequest, AllocationPlan, DeviceKind, DeviceSensors,
};

// Stub for SystemSensors until apple_silicon provides it
#[derive(Debug, Clone)]
struct SystemSensors;

impl SystemSensors {
    fn detect() -> Self {
        Self
    }
}

// Stub for AppleModelRegistry until apple_silicon provides it
#[derive(Debug, Clone)]
struct AppleModelRegistry;

impl AppleModelRegistry {
    fn from_path(_path: &std::path::Path) -> Option<Self> {
        Some(Self)
    }

    fn from_config(_config: AppleModelRegistryConfig) -> Self {
        Self
    }
}

// Stub for AppleModelRegistryConfig
#[derive(Debug, Clone)]
struct AppleModelRegistryConfig {
    models: HashMap<String, String>,
}

// Stub for SimplePlanner
#[derive(Debug, Clone)]
struct SimplePlanner;

impl SimplePlanner {
    fn new(_sensors: SystemSensors, _registry: AppleModelRegistry) -> Self {
        Self
    }
}

/// Task scope definition for orchestration boundaries
#[derive(Debug, Clone)]
pub struct TaskScope {
    pub in_scope: Vec<String>,
    pub out_scope: Vec<String>,
}

/// Change budget for orchestration constraints
#[derive(Debug, Clone)]
pub struct ChangeBudget {
    pub max_files: u32,
    pub max_loc: u32,
}

/// Blast radius for orchestration impact analysis
#[derive(Debug, Clone)]
pub struct BlastRadius {
    pub modules: Vec<String>,
    pub data_migration: bool,
    pub external_deps: Vec<String>,
}
use agent_agency_contracts::working_spec::{
    WorkingSpecMetadata, AcceptanceCriterion, NonFunctionalRequirements, RollbackPlan,
};
use agent_agency_council::{ConsensusCoordinator, ProvenanceEmitter};
use agent_agency_council::models::{
    AcceptanceCriterion as CouncilAcceptanceCriterion, Environment as CouncilEnvironment,
    RiskTier as CouncilRiskTier, SelfAssessment as CouncilSelfAssessment,
    TaskContext as CouncilTaskContext, TaskScope as CouncilTaskScope, TaskSpec as CouncilTaskSpec,
    WorkerOutput as CouncilWorkerOutput,
};
use agent_agency_council::types::{CawsWaiver, ConsensusResult, FinalVerdict};
use agent_agency_resilience::{CircuitBreaker, CircuitBreakerConfig, retry, RetryConfig};
use agent_agency_database::DatabaseClient;
use anyhow::{Context, Result};
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};
use regex::Regex;

// Parallel worker system integration
use parallel_workers::{
    ParallelCoordinator, ParallelCoordinatorConfig, ComplexTask,
    integration::{should_route_to_parallel, estimate_parallelization_benefit, convert_to_complex_task},
    OrchestratorHandle,
};

fn map_risk_tier(tier: u8) -> CouncilRiskTier {
    match tier {
        1 => CouncilRiskTier::Low,
        2 => CouncilRiskTier::Medium,
        3 => CouncilRiskTier::High,
        _ => CouncilRiskTier::High,
    }
}

pub fn to_task_spec(desc: &TaskDescriptor) -> CouncilTaskSpec {
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
        1 => agent_agency_apple_silicon::Tier::HighEfficiency,
        2 => agent_agency_apple_silicon::Tier::Balanced,
        _ => agent_agency_apple_silicon::Tier::HighPerformance,
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
            agent_agency_apple_silicon::Tier::HighEfficiency => 30,
            agent_agency_apple_silicon::Tier::Balanced => 100,
            agent_agency_apple_silicon::Tier::HighPerformance => 200,
        },
        max_batch_size: match tier {
            agent_agency_apple_silicon::Tier::HighEfficiency => 2,
            agent_agency_apple_silicon::Tier::Balanced => 8,
            agent_agency_apple_silicon::Tier::HighPerformance => 16,
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

    // DEPRECATED: Legacy validation for backward compatibility
    let _legacy_validator = DefaultValidator;
    let _legacy_validation = _legacy_validator
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
        .context("Legacy CAWS runtime validation failed")?;

    // NEW: Primary validation using runtime-validator
    let runtime_validator = DefaultOrchestrationIntegration::new();
    let runtime_validation = runtime_validator
        .validate_task_execution(
            spec,
            desc,
            diff,
            &[], // patches
            &[], // language_hints
            tests_added,
            deterministic,
            vec![], // waivers
        )
        .await
        .context("Runtime-validator CAWS validation failed")?;

    // Convert runtime validation to legacy format for compatibility
    let validation = crate::caws_runtime::ValidationResult {
        task_id: runtime_validation.task_id,
        snapshot: crate::caws_runtime::ComplianceSnapshot {
            within_scope: runtime_validation.snapshot.within_scope,
            within_budget: runtime_validation.snapshot.within_budget,
            tests_added: runtime_validation.snapshot.tests_added,
            deterministic: runtime_validation.snapshot.deterministic,
        },
        violations: runtime_validation.violations.into_iter().map(|v| {
            crate::caws_runtime::Violation {
                code: match v.code {
                    RuntimeViolationCode::OutOfScope => crate::caws_runtime::ViolationCode::OutOfScope,
                    RuntimeViolationCode::BudgetExceeded => crate::caws_runtime::ViolationCode::BudgetExceeded,
                    RuntimeViolationCode::MissingTests => crate::caws_runtime::ViolationCode::MissingTests,
                    RuntimeViolationCode::NonDeterministic => crate::caws_runtime::ViolationCode::NonDeterministic,
                    RuntimeViolationCode::DisallowedTool => crate::caws_runtime::ViolationCode::DisallowedTool,
                },
                message: v.message,
                remediation: v.remediation,
            }
        }).collect(),
        waivers: runtime_validation.waivers,
        validated_at: runtime_validation.validated_at,
    };

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
#[derive(Debug)]
pub struct Orchestrator {
    client: reqwest::Client,
    worker_registry: Arc<dyn WorkerRegistry>,
    circuit_breakers: Arc<std::sync::RwLock<HashMap<String, Arc<CircuitBreaker>>>>,
    retry_config: RetryConfig,
    progress_tracker: Arc<ProgressTracker>,
    db_client: Option<Arc<DatabaseClient>>, // Optional for backward compatibility
    parallel_coordinator: Option<Arc<ParallelCoordinator>>, // Optional parallel execution support
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
            None, // Use default parallel coordinator
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
        parallel_coordinator: Option<Arc<ParallelCoordinator>>,
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
            parallel_coordinator,
        }
    }

    /// Enable parallel execution support
    pub fn with_parallel_execution(mut self, coordinator: Arc<ParallelCoordinator>) -> Self {
        self.parallel_coordinator = Some(coordinator);
        self
    }

    /// Check if parallel execution is available
    pub fn has_parallel_support(&self) -> bool {
        self.parallel_coordinator.is_some()
    }

    /// Analyze task complexity to determine execution strategy
    fn analyze_task_complexity(&self, description: &str) -> f32 {
        // Use council complexity analysis if available
        // For now, use simple heuristics based on task characteristics

        let desc_lower = description.to_lowercase();
        let mut complexity_score = 0.0;

        // Keywords that indicate high complexity
        let high_complexity_keywords = [
            "refactor", "migrate", "optimize", "parallel", "concurrent",
            "multiple", "complex", "large", "scale", "enterprise",
        ];

        for keyword in &high_complexity_keywords {
            if desc_lower.contains(keyword) {
                complexity_score += 0.2;
            }
        }

        // Length-based complexity (longer descriptions tend to be more complex)
        let length_factor = (description.len() as f32 / 1000.0).min(0.3);
        complexity_score += length_factor;

        // Error-related tasks are highly parallelizable
        if desc_lower.contains("error") || desc_lower.contains("fix") || desc_lower.contains("bug") {
            complexity_score += 0.3;
        }

        complexity_score.min(1.0)
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

        // Check if task should be routed to parallel execution
        let complexity_score = self.analyze_task_complexity(description);
        let should_use_parallel = self.parallel_coordinator.is_some() &&
            should_route_to_parallel(description, complexity_score, &ParallelCoordinatorConfig::default());

        if should_use_parallel {
            info!("Routing task {} to parallel execution (complexity: {:.2})", task_id, complexity_score);

            // P0: Audit trail - Parallel routing
            if let Some(ref db_client) = self.db_client {
                db_client.create_task_audit_event(
                    task_id,
                    "orchestration",
                    "system",
                    "parallel_routing",
                    serde_json::json!({
                        "description": description,
                        "complexity_score": complexity_score,
                        "parallel_benefit": estimate_parallelization_benefit(description, None),
                        "stage": "parallel_coordinator"
                    }),
                ).await.map_err(|e| format!("Failed to audit parallel routing: {}", e))?;
            }

            // Convert to complex task and execute in parallel
            let workspace_root = std::env::current_dir()
                .map_err(|e| format!("Failed to get workspace root: {}", e))?;

            let complex_task = convert_to_complex_task(description.to_string(), workspace_root);

            return match self.parallel_coordinator.as_ref().unwrap().execute_parallel(complex_task.clone()).await {
                Ok(result) => {
                    // Convert parallel result to orchestration result
                    Ok(TaskExecutionResult {
                        task_id,
                        success: result.success,
                        output: result.summary,
                        execution_time_ms: result.execution_time.as_millis() as u64,
                        worker_endpoint: "parallel-coordinator".to_string(),
                        metadata: serde_json::json!({
                            "parallel_execution": true,
                            "subtasks_completed": result.subtasks_completed,
                            "total_subtasks": result.total_subtasks,
                            "quality_scores": result.quality_scores
                        }),
                    })
                }
                Err(e) => {
                    warn!("Parallel execution failed, falling back to sequential: {:?}", e);

                    // Fall back to sequential execution
                    self.execute_sequential_fallback(task_id, description, execution_mode).await
                }
            };
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
            id: uuid::Uuid::new_v4(),
            task_id: uuid::Uuid::parse_str(task_id).unwrap_or_else(|_| uuid::Uuid::new_v4()),
            code_changes: vec![],
            test_results: TestResults::default(),
            coverage: CoverageReport::default(),
            mutation: MutationReport::default(),
            lint: LintReport::default(),
            types: TypeCheckReport::default(),
            provenance: ProvenanceRecord::default(),
            generated_at: chrono::Utc::now(),
        };

        // Generate comprehensive working specification
        let working_spec = self.generate_working_spec(task_id, description, &execution_output).await?;

        // Complete progress tracking
        self.progress_tracker.complete_execution(task_id, success).await?;

        Ok(TaskExecutionResult {
            working_spec,
            artifacts,
            quality_report: None,
        })
    }

    /// Execute task using sequential fallback when parallel execution fails
    async fn execute_sequential_fallback(
        &self,
        task_id: Uuid,
        description: &str,
        execution_mode: Option<&str>,
    ) -> Result<TaskExecutionResult, Box<dyn std::error::Error + Send + Sync>> {
        warn!("Falling back to sequential execution for task {}", task_id);

        // Get worker endpoint (same logic as original orchestrate_task)
        let worker_id = "default-worker";
        let worker_endpoint = self.worker_registry.get_worker_endpoint(worker_id).await
            .map_err(|e| format!("Failed to get worker endpoint: {}", e))?;

        // Create task execution request (same logic)
        let mut request = serde_json::json!({
            "task_id": task_id,
            "prompt": description,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        if let Some(mode) = execution_mode {
            request["execution_mode"] = serde_json::Value::String(mode.to_string());
        }

        // Execute with resilience (same logic as original)
        let circuit_breaker = self.get_or_create_circuit_breaker(worker_id).await;
        let worker_result = self.execute_with_resilience(
            task_id,
            worker_id,
            &worker_endpoint,
            &request,
            &circuit_breaker,
        ).await?;

        // Parse and return result (same logic as original)
        let success = worker_result.get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let output = worker_result.get("output")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let execution_time_ms = worker_result.get("execution_time_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        // Generate working specification
        let working_spec = self.generate_working_spec(task_id, description, &worker_result).await?;

        // Extract artifacts
        let artifacts = worker_result.get("artifacts")
            .and_then(|v| v.as_array())
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|a| a.as_object())
            .map(|obj| {
                let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                let path = obj.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let artifact_type = obj.get("type").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                ExecutionArtifacts {
                    name,
                    path: std::path::PathBuf::from(path),
                    artifact_type,
                    size_bytes: obj.get("size_bytes").and_then(|v| v.as_u64()).unwrap_or(0),
                }
            })
            .collect();

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
        let result = retry(
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

    /// Generate comprehensive working specification with intelligent analysis
    async fn generate_working_spec(
        &self,
        task_id: &str,
        description: &str,
        execution_output: &str,
    ) -> Result<WorkingSpec> {
        // Parse task requirements and generate detailed acceptance criteria
        let acceptance_criteria = self.parse_acceptance_criteria(description, execution_output)?;
        
        // Analyze codebase structure and determine appropriate scope boundaries
        let scope = self.analyze_scope_boundaries(task_id, description)?;
        
        // Identify risk tier based on impact analysis and dependencies
        let risk_tier = self.calculate_risk_tier(&scope, &acceptance_criteria)?;
        
        // Generate specific test requirements and quality gates
        let test_requirements = self.generate_test_requirements(&acceptance_criteria)?;
        
        // Create performance budgets and monitoring requirements
        let performance_budgets = self.create_performance_budgets(&scope)?;
        
        // Identify security and compliance requirements
        let security_requirements = self.identify_security_requirements(&scope)?;
        
        // Generate deployment and rollback specifications
        let rollback_plan = self.generate_rollback_plan(&scope)?;
        
        // Create documentation and maintenance requirements
        let documentation_requirements = self.generate_documentation_requirements(&scope)?;
        
        // Establish success metrics and completion criteria
        let success_metrics = self.establish_success_metrics(&acceptance_criteria)?;

        let change_budget = self.calculate_change_budget(&scope)?;
        
        let working_spec = WorkingSpec {
            risk_tier,
            scope_in: scope.in_scope.clone(),
            change_budget_max_files: change_budget.max_files,
            change_budget_max_loc: change_budget.max_loc,
        };

        Ok(working_spec)
    }

    /// Parse task requirements into structured acceptance criteria
    fn parse_acceptance_criteria(&self, description: &str, execution_output: &str) -> Result<Vec<AcceptanceCriterion>> {
        let mut criteria = Vec::new();
        
        // Extract "Given-When-Then" patterns from description
        let gwt_pattern = Regex::new(r"(?i)(given|when|then)\s+([^.!?]+[.!?])")?;
        let mut current_criterion = None;
        
        for cap in gwt_pattern.captures_iter(description) {
            let clause_type = cap.get(1).unwrap().as_str().to_lowercase();
            let clause_text = cap.get(2).unwrap().as_str().trim().to_string();
            
            match clause_type.as_str() {
                "given" => {
                    if let Some(mut criterion) = current_criterion.take() {
                        criteria.push(criterion);
                    }
                    current_criterion = Some(AcceptanceCriterion {
                        id: format!("AC-{}", criteria.len() + 1),
                        given: clause_text,
                        when: String::new(),
                        then: String::new(),
                        priority: CriterionPriority::MustHave,
                    });
                },
                "when" => {
                    if let Some(ref mut criterion) = current_criterion {
                        criterion.when = clause_text;
                    }
                },
                "then" => {
                    if let Some(ref mut criterion) = current_criterion {
                        criterion.then = clause_text;
                    }
                },
                _ => {}
            }
        }
        
        if let Some(criterion) = current_criterion {
            criteria.push(criterion);
        }
        
        // If no structured criteria found, create from execution output analysis
        if criteria.is_empty() {
            criteria.push(AcceptanceCriterion {
                id: "AC-1".to_string(),
                given: "Task execution environment".to_string(),
                when: format!("Task '{}' is executed", description),
                then: "Task completes successfully with expected output".to_string(),
                priority: CriterionPriority::MustHave,
            });
        }
        
        Ok(criteria)
    }

    /// Analyze codebase structure to determine scope boundaries
    fn analyze_scope_boundaries(&self, task_id: &str, description: &str) -> Result<TaskScope> {
        // Simple heuristic-based scope analysis
        let mut in_scope = Vec::new();
        let mut out_scope = Vec::new();
        
        // Analyze description for file/module patterns
        let file_pattern = Regex::new(r"\b([a-zA-Z0-9_/.-]+\.(rs|ts|js|py|go|java))\b")?;
        let module_pattern = Regex::new(r"\b(src/|lib/|tests/|docs/)\b")?;
        
        for cap in file_pattern.captures_iter(description) {
            let file_path = cap.get(1).unwrap().as_str();
            if !in_scope.contains(&file_path.to_string()) {
                in_scope.push(file_path.to_string());
            }
        }
        
        for cap in module_pattern.captures_iter(description) {
            let module_path = cap.get(1).unwrap().as_str();
            if !in_scope.contains(&module_path.to_string()) {
                in_scope.push(module_path.to_string());
            }
        }
        
        // Default scope if none detected
        if in_scope.is_empty() {
            in_scope.push("src/".to_string());
        }
        
        // Exclude common non-task directories
        out_scope.extend(vec![
            "target/".to_string(),
            "node_modules/".to_string(),
            ".git/".to_string(),
            "dist/".to_string(),
            "build/".to_string(),
        ]);
        
        Ok(TaskScope {
            in_scope,
            out_scope,
        })
    }

    /// Calculate risk tier based on scope and criteria analysis
    fn calculate_risk_tier(&self, scope: &TaskScope, criteria: &[AcceptanceCriterion]) -> u8 {
        let mut risk_score = 2; // Default to Tier 2
        
        // Increase risk for critical modules
        for path in &scope.in_scope {
            if path.contains("auth") || path.contains("security") || path.contains("payment") {
                risk_score = 1; // Tier 1 for critical systems
                break;
            }
        }
        
        // Increase risk for complex acceptance criteria
        if criteria.len() > 5 {
            risk_score = (risk_score - 1).max(1);
        }
        
        // Check for database migration requirements
        if scope.in_scope.iter().any(|p| p.contains("migration") || p.contains("schema")) {
            risk_score = 1; // Tier 1 for database changes
        }
        
        risk_score
    }

    /// Generate test requirements based on acceptance criteria
    fn generate_test_requirements(&self, criteria: &[AcceptanceCriterion]) -> Result<Vec<String>> {
        let mut requirements = Vec::new();
        
        for criterion in criteria {
            requirements.push(format!("Test: {} - {} - {}", criterion.given, criterion.when, criterion.then));
        }
        
        // Add standard test requirements
        requirements.extend(vec![
            "Unit tests for all new functions".to_string(),
            "Integration tests for API endpoints".to_string(),
            "End-to-end tests for critical user flows".to_string(),
            "Performance tests for SLA compliance".to_string(),
        ]);
        
        Ok(requirements)
    }

    /// Create performance budgets based on scope analysis
    fn create_performance_budgets(&self, scope: &TaskScope) -> Result<Vec<String>> {
        let mut budgets = Vec::new();
        
        // API performance budgets
        if scope.in_scope.iter().any(|p| p.contains("api") || p.contains("controller")) {
            budgets.push("API response time P95 < 250ms".to_string());
            budgets.push("API throughput > 1000 req/sec".to_string());
        }
        
        // Database performance budgets
        if scope.in_scope.iter().any(|p| p.contains("database") || p.contains("model")) {
            budgets.push("Database query time P95 < 100ms".to_string());
            budgets.push("Database connection pool utilization < 80%".to_string());
        }
        
        // Default performance budgets
        budgets.extend(vec![
            "Memory usage increase < 10%".to_string(),
            "CPU usage increase < 5%".to_string(),
            "Bundle size increase < 5%".to_string(),
        ]);
        
        Ok(budgets)
    }

    /// Identify security requirements based on scope
    fn identify_security_requirements(&self, scope: &TaskScope) -> Result<Vec<String>> {
        let mut requirements = Vec::new();
        
        // Authentication and authorization
        if scope.in_scope.iter().any(|p| p.contains("auth") || p.contains("security")) {
            requirements.push("Input validation and sanitization".to_string());
            requirements.push("Authentication token validation".to_string());
            requirements.push("Authorization checks for all endpoints".to_string());
        }
        
        // Data handling
        if scope.in_scope.iter().any(|p| p.contains("data") || p.contains("user")) {
            requirements.push("Data encryption at rest and in transit".to_string());
            requirements.push("PII data handling compliance".to_string());
            requirements.push("Audit logging for sensitive operations".to_string());
        }
        
        // Default security requirements
        requirements.extend(vec![
            "Dependency vulnerability scanning".to_string(),
            "Static analysis security testing".to_string(),
            "Rate limiting implementation".to_string(),
        ]);
        
        Ok(requirements)
    }

    /// Generate rollback plan based on scope
    fn generate_rollback_plan(&self, scope: &TaskScope) -> Result<RollbackPlan> {
        let slo = if scope.in_scope.iter().any(|p| p.contains("database") || p.contains("migration")) {
            "10m".to_string() // Longer for database changes
        } else {
            "5m".to_string() // Standard rollback time
        };
        
        Ok(RollbackPlan {
            strategy: "automated".to_string(),
            slo,
            automated_steps: vec![
                "Stop new deployments".to_string(),
                "Revert to previous version".to_string(),
                "Verify system health".to_string(),
            ],
            manual_steps: vec![
                "Notify stakeholders".to_string(),
            ],
            data_impact: "minimal".to_string(),
            downtime_required: false,
            rollback_window_minutes: 5,
        })
    }

    /// Generate documentation requirements
    fn generate_documentation_requirements(&self, scope: &TaskScope) -> Result<Vec<String>> {
        let mut requirements = vec![
            "Update README with new features".to_string(),
            "Document API changes".to_string(),
            "Update deployment procedures".to_string(),
        ];
        
        // Add specific documentation based on scope
        if scope.in_scope.iter().any(|p| p.contains("api")) {
            requirements.push("Update OpenAPI specification".to_string());
        }
        
        if scope.in_scope.iter().any(|p| p.contains("database")) {
            requirements.push("Document schema changes".to_string());
        }
        
        Ok(requirements)
    }

    /// Establish success metrics
    fn establish_success_metrics(&self, criteria: &[AcceptanceCriterion]) -> Result<Vec<String>> {
        let mut metrics = vec![
            "All acceptance criteria met".to_string(),
            "Test coverage > 80%".to_string(),
            "Performance budgets satisfied".to_string(),
            "Security requirements validated".to_string(),
        ];
        
        // Add metrics based on criteria complexity
        if criteria.len() > 3 {
            metrics.push("Integration tests passing".to_string());
        }
        
        Ok(metrics)
    }

    /// Extract title from task description
    fn extract_title_from_description(&self, description: &str) -> String {
        // Take first sentence or first 50 characters, whichever is shorter
        let first_sentence = description.split('.').next().unwrap_or(description);
        if first_sentence.len() > 50 {
            format!("{}...", &first_sentence[..47])
        } else {
            first_sentence.to_string()
        }
    }

    /// Generate system invariants based on scope
    fn generate_invariants(&self, scope: &TaskScope) -> Result<Vec<String>> {
        let mut invariants = vec![
            "System maintains backward compatibility".to_string(),
            "No data loss during implementation".to_string(),
            "Performance does not degrade".to_string(),
        ];
        
        // Add scope-specific invariants
        if scope.in_scope.iter().any(|p| p.contains("database")) {
            invariants.push("Database schema remains consistent".to_string());
        }
        
        if scope.in_scope.iter().any(|p| p.contains("api")) {
            invariants.push("API contracts remain stable".to_string());
        }
        
        Ok(invariants)
    }

    /// Calculate change budget based on scope
    fn calculate_change_budget(&self, scope: &TaskScope) -> Result<ChangeBudget> {
        let file_count = scope.in_scope.len();
        let estimated_loc = file_count * 50; // Rough estimate
        
        Ok(ChangeBudget {
            max_files: (file_count * 2).max(10) as u32,
            max_loc: (estimated_loc * 2).max(500) as u32,
        })
    }

    /// Calculate blast radius based on scope
    fn calculate_blast_radius(&self, scope: &TaskScope) -> Result<BlastRadius> {
        let modules: Vec<String> = scope.in_scope.iter()
            .filter_map(|path| {
                if path.contains('/') {
                    path.split('/').next().map(|s| s.to_string())
                } else {
                    None
                }
            })
            .collect();
        
        let data_migration = scope.in_scope.iter().any(|p| 
            p.contains("migration") || p.contains("schema") || p.contains("database")
        );
        
        Ok(BlastRadius {
            modules,
            data_migration,
        })
    }

    /// Generate reliability requirements
    fn generate_reliability_requirements(&self, scope: &TaskScope) -> Result<Vec<String>> {
        let mut requirements = vec![
            "Error handling for all failure modes".to_string(),
            "Circuit breaker implementation".to_string(),
            "Graceful degradation strategies".to_string(),
        ];
        
        if scope.in_scope.iter().any(|p| p.contains("external") || p.contains("api")) {
            requirements.push("Retry logic with exponential backoff".to_string());
        }
        
        Ok(requirements)
    }

    /// Generate usability requirements
    fn generate_usability_requirements(&self, criteria: &[AcceptanceCriterion]) -> Result<Vec<String>> {
        let mut requirements = vec![
            "Clear error messages for users".to_string(),
            "Consistent user interface patterns".to_string(),
        ];
        
        if criteria.iter().any(|c| c.when.contains("user") || c.then.contains("user")) {
            requirements.push("User experience validation".to_string());
        }
        
        Ok(requirements)
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
