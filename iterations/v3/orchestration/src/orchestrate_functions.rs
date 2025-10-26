// Re-export the main orchestration functionality from the decomposed modules
pub use crate::orchestration_core::{orchestrate_task, types, validation, execution, coordination};

// Keep the Orchestrator struct and its implementation for backward compatibility
use std::collections::HashMap;
use uuid::Uuid;
use crate::tracking::ProgressTracker;
use crate::types::{TaskScope, ChangeBudget, BlastRadius, OrchestratorConfig, TaskExecutionResult};
use crate::worker_registry::{WorkerRegistry, StaticWorkerRegistry};
use agent_agency_resilience::{CircuitBreaker, CircuitBreakerConfig, retry, RetryConfig};
use agent_agency_database::DatabaseClient;
use anyhow::{Context, Result};
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};

// Parallel worker system integration
use parallel_workers::{
    ParallelCoordinator, ParallelCoordinatorConfig, ComplexTask,
    integration::{should_route_to_parallel, estimate_parallelization_benefit, convert_to_complex_task},
    OrchestratorHandle,
};
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
    memory_system: Option<Arc<agent_memory::MemorySystem>>, // Optional memory integration
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
        memory_system: Option<Arc<agent_memory::MemorySystem>>,
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
            memory_system,
        }
    }

    /// Enable parallel execution support
    pub fn with_parallel_execution(mut self, coordinator: Arc<ParallelCoordinator>) -> Self {
        self.parallel_coordinator = Some(coordinator);
        self
    }

    /// Enable memory system integration for learning and adaptation
    pub fn with_memory_system(mut self, memory_system: Arc<agent_memory::MemorySystem>) -> Self {
        self.memory_system = Some(memory_system);
        self
    }

    /// Check if parallel execution is available
    pub fn has_parallel_support(&self) -> bool {
        self.parallel_coordinator.is_some()
    }

    /// Check if memory system is available
    pub fn has_memory_support(&self) -> bool {
        self.memory_system.is_some()
    }

    /// Retrieve relevant memories for task execution decisions
    async fn retrieve_execution_memories(
        &self,
        task_description: &str,
        task_type: &str,
    ) -> Vec<agent_memory::AgentExperience> {
        if let Some(ref memory_system) = self.memory_system {
            // Create context for memory retrieval
            let task_context = agent_memory::TaskContext {
                task_id: "orchestrator_decision".to_string(),
                task_type: task_type.to_string(),
                description: format!("Making orchestration decision for: {}", task_description),
                domain: vec!["orchestration".to_string(), "execution".to_string()],
                entities: vec!["orchestrator".to_string()],
                temporal_context: Some(agent_memory::TemporalContext {
                    start_time: chrono::Utc::now(),
                    deadline: None,
                    priority: agent_memory::TaskPriority::High,
                    recurrence_pattern: None,
                }),
                metadata: std::collections::HashMap::new(),
            };

            match memory_system.retrieve_contextual_memories(&task_context, 5).await {
                Ok(memories) => memories,
                Err(e) => {
                    warn!("Failed to retrieve execution memories: {}", e);
                    vec![]
                }
            }
        } else {
            vec![]
        }
    }

    /// Store execution outcome as memory for future learning
    async fn store_execution_memory(
        &self,
        task_id: String,
        task_description: String,
        execution_strategy: String,
        outcome: agent_memory::ExperienceOutcome,
    ) {
        if let Some(ref memory_system) = self.memory_system {
            let task_context = agent_memory::TaskContext {
                task_id: task_id.clone(),
                task_type: "orchestration_execution".to_string(),
                description: format!("Executed task with strategy: {}", execution_strategy),
                domain: vec!["orchestration".to_string(), "learning".to_string()],
                entities: vec!["orchestrator".to_string(), task_id.clone()],
                temporal_context: Some(agent_memory::TemporalContext {
                    start_time: chrono::Utc::now(),
                    deadline: None,
                    priority: agent_memory::TaskPriority::Medium,
                    recurrence_pattern: None,
                }),
                metadata: std::collections::HashMap::new(),
            };

            let experience = agent_memory::AgentExperience {
                id: uuid::Uuid::new_v4(),
                agent_id: "orchestrator".to_string(),
                task_id,
                context: task_context,
                input: serde_json::json!({
                    "task_description": task_description,
                    "execution_strategy": execution_strategy
                }),
                output: serde_json::json!({
                    "outcome": outcome
                }),
                outcome,
                memory_type: agent_memory::MemoryType::Episodic,
                timestamp: chrono::Utc::now(),
                metadata: std::collections::HashMap::new(),
            };

            if let Err(e) = memory_system.store_experience(experience).await {
                warn!("Failed to store execution memory: {}", e);
            }
        }
    }

    /// Analyze execution memories to inform orchestration decisions
    fn analyze_execution_memories(
        &self,
        memories: &[agent_memory::AgentExperience],
        task_description: &str,
    ) -> MemoryInformedDecision {
        if memories.is_empty() {
            return MemoryInformedDecision {
                prefers_parallel: true, // Default to parallel if no memory
                suggested_workers: vec![],
                expected_success_rate: 0.8,
                confidence: 0.0,
            };
        }

        // Analyze past execution outcomes
        let mut parallel_successes = 0;
        let mut parallel_attempts = 0;
        let mut sequential_successes = 0;
        let mut sequential_attempts = 0;
        let mut worker_performance = std::collections::HashMap::new();

        for memory in memories {
            if let Some(strategy) = memory.context.metadata.get("execution_strategy") {
                if strategy == "parallel" {
                    parallel_attempts += 1;
                    if memory.outcome.success {
                        parallel_successes += 1;
                    }
                } else if strategy == "sequential" {
                    sequential_attempts += 1;
                    if memory.outcome.success {
                        sequential_successes += 1;
                    }
                }
            }

            // Track worker performance
            if let Some(worker_id) = memory.context.metadata.get("worker_id") {
                let performance = memory.outcome.performance_score.unwrap_or(0.5);
                worker_performance.entry(worker_id.clone())
                    .or_insert(vec![])
                    .push(performance);
            }
        }

        // Calculate success rates
        let parallel_success_rate = if parallel_attempts > 0 {
            parallel_successes as f32 / parallel_attempts as f32
        } else {
            0.8 // Default assumption
        };

        let sequential_success_rate = if sequential_attempts > 0 {
            sequential_successes as f32 / sequential_attempts as f32
        } else {
            0.8 // Default assumption
        };

        // Determine preference based on historical success
        let prefers_parallel = parallel_success_rate >= sequential_success_rate;

        // Find best performing workers
        let mut worker_scores: Vec<_> = worker_performance.into_iter()
            .map(|(worker_id, scores)| {
                let avg_score = scores.iter().sum::<f32>() / scores.len() as f32;
                (worker_id, avg_score)
            })
            .collect();

        worker_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let suggested_workers = worker_scores.into_iter()
            .take(3)
            .map(|(worker_id, _)| worker_id)
            .collect();

        // Calculate confidence based on sample size
        let total_attempts = parallel_attempts + sequential_attempts;
        let confidence = if total_attempts >= 5 {
            0.9
        } else if total_attempts >= 2 {
            0.7
        } else {
            0.3
        };

        MemoryInformedDecision {
            prefers_parallel,
            suggested_workers,
            expected_success_rate: if prefers_parallel { parallel_success_rate } else { sequential_success_rate },
            confidence,
        }
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

        // Retrieve relevant execution memories to inform decision making
        let execution_memories = self.retrieve_execution_memories(description, "task_execution").await;
        let memory_informed_decision = self.analyze_execution_memories(&execution_memories, description);

        let should_use_parallel = self.parallel_coordinator.is_some() &&
            should_route_to_parallel(description, complexity_score, &ParallelCoordinatorConfig::default()) &&
            memory_informed_decision.prefers_parallel;

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
                    // Store execution outcome in memory for future learning
                    let outcome = agent_memory::ExperienceOutcome {
                        success: result.success,
                        performance_score: Some(if result.success { 0.9 } else { 0.3 }),
                        learned_capabilities: vec!["parallel_execution".to_string()],
                        failure_reasons: if result.success { vec![] } else { vec!["parallel_execution_failed".to_string()] },
                        success_factors: if result.success { vec!["parallel_strategy".to_string()] } else { vec![] },
                        execution_time_ms: Some(result.execution_time.as_millis() as u64),
                        tokens_used: None,
                        feedback: Some(agent_memory::AgentFeedback {
                            quality_score: Some(if result.success { 0.85 } else { 0.4 }),
                            relevance_score: Some(0.9),
                            accuracy_score: Some(if result.success { 0.9 } else { 0.5 }),
                            comments: vec![format!("Parallel execution {}", if result.success { "succeeded" } else { "failed" })],
                            evaluator_id: Some("orchestrator".to_string()),
                        }),
                    };

                    self.store_execution_memory(
                        task_id.to_string(),
                        description.to_string(),
                        "parallel".to_string(),
                        outcome,
                    ).await;

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

        // Store execution outcome in memory for future learning
        let outcome = agent_memory::ExperienceOutcome {
            success,
            performance_score: Some(if success { 0.8 } else { 0.2 }),
            learned_capabilities: vec!["sequential_execution".to_string()],
            failure_reasons: if success { vec![] } else { vec!["sequential_execution_failed".to_string()] },
            success_factors: if success { vec!["sequential_strategy".to_string()] } else { vec![] },
            execution_time_ms: Some(execution_time_ms),
            tokens_used: None,
            feedback: Some(agent_memory::AgentFeedback {
                quality_score: Some(if success { 0.8 } else { 0.3 }),
                relevance_score: Some(0.85),
                accuracy_score: Some(if success { 0.85 } else { 0.4 }),
                comments: vec![format!("Sequential execution {}", if success { "succeeded" } else { "failed" })],
                evaluator_id: Some("orchestrator".to_string()),
            }),
        };

        self.store_execution_memory(
            task_id.to_string(),
            description.to_string(),
            "sequential".to_string(),
            outcome,
        ).await;

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

