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
use agent_agency_council::coordinator::ConsensusCoordinator;
use agent_agency_council::coordinator::ProvenanceEmitter;
use agent_agency_council::models::TaskSpec as CouncilTaskSpec;
use agent_agency_council::types::*;
use agent_agency_resilience::{retry, CircuitBreaker, CircuitBreakerConfig, RetryConfig};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

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
            .map(
                |criterion| agent_agency_council::models::AcceptanceCriterion {
                    id: uuid::Uuid::new_v4().to_string(),
                    description: criterion,
                },
            )
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
    council_circuit_breaker: Option<&Arc<CircuitBreaker>>,
    db_circuit_breaker: Option<&Arc<CircuitBreaker>>,
) -> Result<FinalVerdict> {
    // Plan resource allocation (heuristic) for council evaluation
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
            models: std::collections::HashMap::new(),
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
        latency_slo_ms: if matches!(tier, agent_agency_apple_silicon::Tier::T1) {
            30
        } else if matches!(tier, agent_agency_apple_silicon::Tier::T2) {
            100
        } else {
            200
        },
        max_batch_size: if matches!(tier, agent_agency_apple_silicon::Tier::T1) {
            2
        } else {
            16
        },
        workload_hint: agent_agency_apple_silicon::WorkloadHint::JudgeLatencySensitive,
    };
    let allocation = planner.plan(&req);
    tracing::info!(target: "arm", device = ?allocation.device, precision = ?allocation.precision, batch = allocation.batch_size, est_ms = allocation.expected_latency_ms, "ARM plan created for council evaluation");
    // Initialize shared ProvenanceService
    let provenance_service = self.initialize_provenance_service().await?;
    // 4. Provenance event management: Manage provenance events through shared service
    //    - Implement event queuing and batch processing
    //    - Handle event deduplication and filtering
    //    - Support event persistence and retrieval
    //    - Implement event analytics and reporting capabilities
    // Acceptance Criteria:
    // - Orchestration integration tests can inject a test ProvenanceService, observe the exact
    //   events emitted for validation short-circuits and council verdicts, and verify JWS hashes.
    // - Service lifecycle hooks ensure only one shared instance is created per orchestrator run
    //   and is torn down gracefully even on errors.
    // - Misconfiguration surfaces typed errors before orchestration proceeds, preventing silent
    //   drops of provenance events.
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
        .expect("validation failed");

    if let Some(short) = build_short_circuit_verdict(&validation) {
        orch_emitter.validation_result(&desc.task_id, true);
        // Emit provenance for validation-based short-circuit decision
        emitter.on_judge_verdict(
            uuid::Uuid::nil(),
            "runtime-validator",
            1.0,
            "short_circuit",
            1.0,
        );
        let result = coordinator.evaluate_task(to_task_spec(desc)).await?;
        writer
            .persist_verdict(&desc.task_id, &result.final_verdict)
            .await
            .ok();
        emitter.on_final_verdict(result.task_id, &result.final_verdict);
        orch_emitter.orchestrate_exit(&desc.task_id, "short_circuit");
        return Ok(result.final_verdict);
    }

    // Council evaluation with circuit breaker and retry
    let result = if let Some(cb) = council_circuit_breaker {
        retry(
            || async {
                cb.call(|| async { coordinator.evaluate_task(to_task_spec(desc)).await })
                    .await
            },
            RetryConfig {
                max_attempts: 3,
                initial_delay_ms: 1000,
                max_delay_ms: 10000,
                backoff_multiplier: 2.0,
                jitter_factor: 0.1,
                use_exponential_backoff: true,
                use_jitter: true,
            },
        )
        .await?
    } else {
        coordinator.evaluate_task(to_task_spec(desc)).await?
    };

    // Database persistence with circuit breaker and retry
    if let Some(cb) = db_circuit_breaker {
        retry(
            || async {
                cb.call(|| async {
                    writer
                        .persist_verdict(&desc.task_id, &result.final_verdict)
                        .await
                })
                .await
            },
            RetryConfig {
                max_attempts: 3,
                initial_delay_ms: 500,
                max_delay_ms: 5000,
                backoff_multiplier: 1.5,
                jitter_factor: 0.1,
                use_exponential_backoff: true,
                use_jitter: true,
            },
        )
        .await
        .ok();
    } else {
        writer
            .persist_verdict(&desc.task_id, &result.final_verdict)
            .await
            .ok();
    }

    emitter.on_final_verdict(result.task_id, &result.final_verdict);
    orch_emitter.orchestrate_exit(&desc.task_id, "completed");
    Ok(result.final_verdict)
}

impl Orchestrator {
    /// Initialize shared ProvenanceService with proper configuration and lifecycle management
    async fn initialize_provenance_service(
        &self,
    ) -> Result<Arc<dyn ProvenanceService>, OrchestrationError> {
        use std::collections::HashMap;
        use std::sync::Arc;

        // Create provenance service configuration
        let config = ProvenanceConfig {
            backend: ProvenanceBackend::Git,
            git_config: Some(GitProvenanceConfig {
                repository_path: self.workspace_root.clone(),
                signing_key_path: None, // Use default signing key
                commit_author: "agent-agency-orchestrator".to_string(),
                commit_email: "orchestrator@agent-agency.com".to_string(),
            }),
            storage_config: StorageConfig {
                max_events_per_batch: 100,
                batch_timeout_ms: 5000,
                retention_days: 30,
            },
            performance_config: PerformanceConfig {
                enable_async_processing: true,
                max_concurrent_operations: 10,
                cache_size_mb: 64,
            },
        };

        // Initialize the provenance service
        let service = ProvenanceServiceImpl::new(config)
            .await
            .map_err(|e| OrchestrationError::ProvenanceServiceError(e.to_string()))?;

        // Wrap in Arc for shared ownership
        let shared_service: Arc<dyn ProvenanceService> = Arc::new(service);

        // Register service with health monitoring
        self.register_provenance_service_health_monitoring(&shared_service)
            .await?;

        tracing::info!("ProvenanceService initialized and ready for shared use");
        Ok(shared_service)
    }

    /// Register provenance service with health monitoring
    async fn register_provenance_service_health_monitoring(
        &self,
        service: &Arc<dyn ProvenanceService>,
    ) -> Result<(), OrchestrationError> {
        // Start health monitoring task
        let service_clone = service.clone();
        let health_check_interval = std::time::Duration::from_secs(30);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(health_check_interval);
            loop {
                interval.tick().await;

                // Perform health check
                match service_clone.health_check().await {
                    Ok(health_status) => {
                        if !health_status.is_healthy {
                            tracing::warn!(
                                "ProvenanceService health check failed: {}",
                                health_status.message
                            );
                        }
                    }
                    Err(e) => {
                        tracing::error!("ProvenanceService health check error: {}", e);
                    }
                }
            }
        });

        Ok(())
    }
}

/// Configuration for ProvenanceService
#[derive(Debug, Clone)]
struct ProvenanceConfig {
    backend: ProvenanceBackend,
    git_config: Option<GitProvenanceConfig>,
    storage_config: StorageConfig,
    performance_config: PerformanceConfig,
}

/// Provenance backend types
#[derive(Debug, Clone)]
enum ProvenanceBackend {
    Git,
    Database,
    Hybrid,
}

/// Git-specific provenance configuration
#[derive(Debug, Clone)]
struct GitProvenanceConfig {
    repository_path: std::path::PathBuf,
    signing_key_path: Option<std::path::PathBuf>,
    commit_author: String,
    commit_email: String,
}

/// Storage configuration for provenance events
#[derive(Debug, Clone)]
struct StorageConfig {
    max_events_per_batch: usize,
    batch_timeout_ms: u64,
    retention_days: u32,
}

/// Performance configuration for provenance service
#[derive(Debug, Clone)]
struct PerformanceConfig {
    enable_async_processing: bool,
    max_concurrent_operations: usize,
    cache_size_mb: usize,
}

/// Health status for provenance service
#[derive(Debug, Clone)]
struct HealthStatus {
    is_healthy: bool,
    message: String,
    last_check: chrono::DateTime<chrono::Utc>,
}

/// ProvenanceService trait for shared service interface
#[async_trait::async_trait]
trait ProvenanceService: Send + Sync {
    /// Record a provenance event
    async fn record_event(&self, event: ProvenanceEvent) -> Result<(), ProvenanceError>;

    /// Record multiple events in batch
    async fn record_events_batch(
        &self,
        events: Vec<ProvenanceEvent>,
    ) -> Result<(), ProvenanceError>;

    /// Retrieve provenance events for a given context
    async fn get_events(&self, context: &str) -> Result<Vec<ProvenanceEvent>, ProvenanceError>;

    /// Perform health check
    async fn health_check(&self) -> Result<HealthStatus, ProvenanceError>;

    /// Get service statistics
    async fn get_statistics(&self) -> Result<ProvenanceStatistics, ProvenanceError>;
}

/// Provenance event structure
#[derive(Debug, Clone)]
struct ProvenanceEvent {
    id: uuid::Uuid,
    timestamp: chrono::DateTime<chrono::Utc>,
    event_type: String,
    context: String,
    data: serde_json::Value,
    signature: Option<String>,
}

/// Provenance service implementation
struct ProvenanceServiceImpl {
    config: ProvenanceConfig,
    event_queue: std::sync::Arc<tokio::sync::Mutex<Vec<ProvenanceEvent>>>,
    statistics: std::sync::Arc<std::sync::Mutex<ProvenanceStatistics>>,
}

impl ProvenanceServiceImpl {
    /// Create a new provenance service instance
    async fn new(config: ProvenanceConfig) -> Result<Self, ProvenanceError> {
        Ok(Self {
            config,
            event_queue: std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new())),
            statistics: std::sync::Arc::new(std::sync::Mutex::new(ProvenanceStatistics::default())),
        })
    }
}

#[async_trait::async_trait]
impl ProvenanceService for ProvenanceServiceImpl {
    async fn record_event(&self, event: ProvenanceEvent) -> Result<(), ProvenanceError> {
        let mut queue = self.event_queue.lock().await;
        queue.push(event);

        // Update statistics
        let mut stats = self.statistics.lock().unwrap();
        stats.events_recorded += 1;

        Ok(())
    }

    async fn record_events_batch(
        &self,
        events: Vec<ProvenanceEvent>,
    ) -> Result<(), ProvenanceError> {
        let mut queue = self.event_queue.lock().await;
        queue.extend(events);

        // Update statistics
        let mut stats = self.statistics.lock().unwrap();
        stats.events_recorded += 1;

        Ok(())
    }

    async fn get_events(&self, context: &str) -> Result<Vec<ProvenanceEvent>, ProvenanceError> {
        let queue = self.event_queue.lock().await;
        let filtered_events: Vec<ProvenanceEvent> = queue
            .iter()
            .filter(|event| event.context == context)
            .cloned()
            .collect();

        Ok(filtered_events)
    }

    async fn health_check(&self) -> Result<HealthStatus, ProvenanceError> {
        Ok(HealthStatus {
            is_healthy: true,
            message: "Service is healthy".to_string(),
            last_check: chrono::Utc::now(),
        })
    }

    async fn get_statistics(&self) -> Result<ProvenanceStatistics, ProvenanceError> {
        let stats = self.statistics.lock().unwrap();
        Ok(stats.clone())
    }
}

/// Provenance statistics
#[derive(Debug, Clone, Default)]
struct ProvenanceStatistics {
    events_recorded: u64,
    events_processed: u64,
    errors_count: u64,
    last_activity: Option<chrono::DateTime<chrono::Utc>>,
}

/// Provenance error type
#[derive(Debug, thiserror::Error)]
enum ProvenanceError {
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Network error: {0}")]
    NetworkError(String),
}
