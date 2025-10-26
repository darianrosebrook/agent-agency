//! Core Orchestrator Implementation
//!
//! This module contains the main Orchestrator struct and its core execution logic.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{timeout, Duration};
use uuid::Uuid;
use regex::Regex;
use tracing::{debug, info, instrument, warn};

use crate::types::{OrchestratorConfig, TaskExecutionResult, TaskScope, ChangeBudget, BlastRadius, MemoryInformedDecision};
use crate::worker_registry::{WorkerRegistry, StaticWorkerRegistry};
use crate::caws_runtime::{
    CawsRuntimeValidator, DefaultValidator, DiffStats, TaskDescriptor, WorkingSpec,
};
use crate::persistence::VerdictWriter;
use crate::provenance::OrchestrationProvenanceEmitter;
use crate::planning::types::{ExecutionArtifacts, TestResults, CoverageReport, MutationReport, LintReport, TypeCheckReport, ProvenanceRecord};
use parallel_workers::types::{Artifact, ArtifactType, ExecutionMetrics};
use crate::planning::agent::{CriterionPriority, RollbackRisk};
use crate::tracking::ProgressTracker;
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

/// The main Orchestrator
#[derive(Debug)]
pub struct Orchestrator {
    client: reqwest::Client,
    worker_registry: Arc<dyn WorkerRegistry>,
    circuit_breakers: Arc<std::sync::RwLock<HashMap<String, Arc<CircuitBreaker>>>>,
    retry_config: RetryConfig,
    progress_tracker: Arc<ProgressTracker>,
    db_client: Option<Arc<DatabaseClient>>, // Optional for backward compatibility
    parallel_coordinator: Option<Arc<parallel_workers::ParallelCoordinator>>, // Optional parallel execution support
    memory_system: Option<Arc<agent_memory::MemorySystem>>, // Optional memory integration
}

impl Orchestrator {
    /// Create a new orchestrator
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
        parallel_coordinator: Option<Arc<parallel_workers::ParallelCoordinator>>,
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
    pub fn with_parallel_execution(mut self, coordinator: Arc<parallel_workers::ParallelCoordinator>) -> Self {
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

    /// Main orchestration function
    #[instrument(skip(self, desc))]
    pub async fn orchestrate_task(&self, desc: &TaskDescriptor) -> Result<TaskExecutionResult> {
        info!("Starting orchestration for task: {}", desc.task_id);

        // Validate task descriptor
        self.validate_task_descriptor(desc)?;

        // Create working specification
        let mut working_spec = self.create_working_spec(desc).await?;

        // Run CAWS validation
        let caws_result = self.run_caws_validation(&mut working_spec, desc).await?;

        // Run execution pipeline
        let execution_result = self.run_execution_pipeline(&working_spec, desc).await?;

        // Create final result
        let result = TaskExecutionResult {
            working_spec,
            artifacts: execution_result,
        };

        info!("Orchestration completed for task: {}", desc.task_id);
        Ok(result)
    }

    /// Validate task descriptor
    fn validate_task_descriptor(&self, desc: &TaskDescriptor) -> Result<()> {
        if desc.task_id.is_empty() {
            return Err(anyhow::anyhow!("Task ID cannot be empty"));
        }
        if desc.scope_in.is_empty() {
            return Err(anyhow::anyhow!("Task scope cannot be empty"));
        }
        Ok(())
    }

    /// Create initial working specification
    async fn create_working_spec(&self, desc: &TaskDescriptor) -> Result<WorkingSpec> {
        // This would contain the logic to create a working spec from task descriptor
        // For now, return a basic implementation
        Ok(WorkingSpec {
            id: Uuid::new_v4(),
            metadata: WorkingSpecMetadata {
                title: format!("Task {}", desc.task_id),
                description: "Auto-generated working specification".to_string(),
                version: "1.0".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            acceptance_criteria: vec![],
            non_functional_requirements: None,
            rollback_plan: None,
        })
    }

    /// Run CAWS validation
    async fn run_caws_validation(&self, working_spec: &mut WorkingSpec, desc: &TaskDescriptor) -> Result<()> {
        // CAWS validation logic would go here
        debug!("Running CAWS validation for task: {}", desc.task_id);
        Ok(())
    }

    /// Run execution pipeline
    async fn run_execution_pipeline(&self, working_spec: &WorkingSpec, desc: &TaskDescriptor) -> Result<ExecutionArtifacts> {
        debug!("Running execution pipeline for task: {}", desc.task_id);

        // This would contain the actual execution logic
        // For now, return empty artifacts
        Ok(ExecutionArtifacts {
            test_results: None,
            coverage_report: None,
            mutation_report: None,
            lint_report: None,
            type_check_report: None,
            provenance_record: None,
        })
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
}
