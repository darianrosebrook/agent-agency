use crate::adapter::build_short_circuit_verdict;
use crate::caws_runtime::{
    CawsRuntimeValidator, DefaultValidator, DiffStats, TaskDescriptor, WorkingSpec,
};
use crate::persistence::VerdictWriter;
use crate::provenance::OrchestrationProvenanceEmitter;
use crate::planning::types::{ExecutionArtifacts, TestResults, CoverageReport, MutationReport, LintReport, TypeCheckReport, ProvenanceRecord};
use crate::tracking::ProgressTracker;
use agent_agency_apple_silicon::{
    adaptive_resource_manager::{
        AppleModelRegistry, AppleModelRegistryConfig, SimplePlanner, SystemSensors,
    },
    AllocationPlanner,
};
use agent_agency_contracts::working_spec::{
    TaskMode, TaskScope, ChangeBudget, BlastRadius, WorkingSpecMetadata,
    AcceptanceCriterion, CriterionPriority, NonFunctionalRequirements, RollbackPlan,
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
use regex::Regex;

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
