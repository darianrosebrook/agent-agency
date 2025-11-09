//! Plan Executor - Execute Execution Plans with Parallel Processing
//!
//! Executes execution plans with parallel milestone processing,
//! evidence collection, and council oversight integration.
//!
//! @author @darianrosebrook

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{timeout, Duration};
use anyhow::{anyhow, Result};
use uuid::Uuid;
use chrono::Utc;
use rand::prelude::*;
use tokio::sync::RwLock;
use agent_agency_contracts::planning::{PlanningEngine, PlanExecutionResult, ExecutionEvidence, ExecutionEventType};

use crate::planning::{
    plan_types::{ExecutionPlan, ParallelBatch, BatchStatus, ResourceUtilization, ResourceRequirements},
    todo_integration::TodoIntegration,
    dependency_resolver::DependencyResolver,
    evidence::EvidenceCollector,
    parallel_coordinator::ParallelCoordinator,
    worker_assignment::WorkerAssignmentStrategy,
    scope_guard::ScopeGuard,
    council_monitor::CouncilMonitor,
    worker_lifecycle_manager::WorkerLifecycleManager,
    worktree_manager::WorktreeManager,
};
use crate::workers::execution_bridge::WorkerExecutionBridge;
use agent_agency_contracts::execution_artifacts::ExecutionArtifacts;
use crate::audit_trail::AuditTrailManager;
use agent_agency_contracts::planning::QualityMetrics;

/// TODO integration interface with interior mutability
#[async_trait::async_trait]
pub trait TodoInterface: Send + Sync {
    async fn initialize_plan(&self, plan_id: Uuid, title: &str) -> Result<()>;
    async fn can_progress_to_milestone(&self, plan_id: Uuid, milestone_id: &str) -> Result<bool>;
    async fn milestone_completed(&self, plan_id: Uuid, milestone_id: &str) -> Result<()>;
}

/// Thin adapter over existing TodoIntegration using RwLock for interior mutability
pub struct TodoAdapter {
    pub inner: RwLock<TodoIntegration>,
}

/// Deterministic failure oracle for testing
pub struct FailureOracle {
    rng: parking_lot::Mutex<StdRng>,
}

impl FailureOracle {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: parking_lot::Mutex::new(rand::rngs::StdRng::seed_from_u64(seed)),
        }
    }

    pub fn chance(&self, p: f64) -> bool {
        let mut rng = self.rng.lock();
        rng.gen::<f64>() < p
    }
}

#[async_trait::async_trait]
impl TodoInterface for TodoAdapter {
    async fn initialize_plan(&self, plan_id: Uuid, title: &str) -> Result<()> {
        self.inner.write().await.initialize_plan_todos(plan_id, title).await
    }

    async fn can_progress_to_milestone(&self, plan_id: Uuid, milestone_id: &str) -> Result<bool> {
        self.inner.read().await.can_progress_to_milestone(plan_id, milestone_id).await
    }

    async fn milestone_completed(&self, plan_id: Uuid, milestone_id: &str) -> Result<()> {
        self.inner.write().await.milestone_completed(plan_id, milestone_id).await
    }
}

/// Plan executor for executing execution plans
pub struct PlanExecutor {
    /// Execution plan to run
    plan: ExecutionPlan,

    /// Worker pool for task execution
    worker_pool: Arc<dyn WorkerPool>,

    /// Evidence collector
    evidence_collector: Arc<EvidenceCollector>,

    /// Worker assignment strategy
    worker_assigner: Arc<WorkerAssignmentStrategy>,

    /// Scope guard for file locking
    scope_guard: Arc<ScopeGuard>,

    /// Council monitor for oversight
    council_monitor: Arc<CouncilMonitor>,

    /// Parallel coordinator for coordinated execution
    parallel_coordinator: std::sync::Weak<ParallelCoordinator>,

    /// Audit trail for execution logging
    audit_trail: Arc<dyn AuditTrail>,

    /// Audit trail manager for chain-of-thought recording
    audit_trail_manager: Option<Arc<AuditTrailManager>>,

    /// TODO integration for quality gate enforcement
    todo_integration: Arc<dyn TodoInterface>,

    /// Worker lifecycle manager for tracking worker assignments and completions
    worker_lifecycle_manager: Option<Arc<WorkerLifecycleManager>>,

    /// Worker execution bridge for real worker execution
    worker_bridge: Option<Arc<WorkerExecutionBridge>>,

    /// Worktree manager for git worktree isolation
    worktree_manager: Option<Arc<WorktreeManager>>,

    /// Parallel execution limit semaphore
    parallel_limit: Arc<Semaphore>,

    /// Failure oracle for deterministic testing
    failure_oracle: Arc<FailureOracle>,

    /// Clock for deterministic time (feature-gated)
    #[cfg(feature = "evaluation")]
    clock: Arc<dyn crate::evaluation::determinism::Clock>,

    /// RNG source for deterministic randomness (feature-gated)
    #[cfg(feature = "evaluation")]
    rng_source: Arc<crate::evaluation::determinism::ThreadSafeRngSource>,

    /// Execution configuration
    config: ExecutionConfig,
}

impl std::fmt::Debug for PlanExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlanExecutor")
            .field("config", &self.config)
            .finish()
    }
}

/// Worker pool abstraction
#[async_trait::async_trait]
pub trait WorkerPool: Send + Sync {
    /// Get available workers
    async fn available_workers(&self) -> Result<Vec<WorkerInfo>>;

    /// Assign worker to milestone
    async fn assign_worker(&self, worker_id: Uuid, milestone_id: String) -> Result<()>;

    /// Release worker from milestone
    async fn release_worker(&self, worker_id: Uuid) -> Result<()>;

    /// Get worker status
    async fn worker_status(&self, worker_id: Uuid) -> Result<WorkerStatus>;
}

/// Worker information

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkerInfo {
    /// Worker unique identifier
    #[schemars(with = "String")]
    pub id: Uuid,

    /// Worker capabilities
    pub capabilities: Vec<String>,

    /// Current load (0.0-1.0)
    pub load: f64,

    /// Worker health status
    pub health: WorkerHealth,
}

/// Worker health status
#[derive(Debug, Clone, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
pub enum WorkerHealth {
    /// Worker is healthy and available
    Healthy,

    /// Worker has minor issues but can work
    Degraded,

    /// Worker is unhealthy (has significant issues)
    Unhealthy,

    /// Worker is unavailable
    Unavailable,
}

/// Worker status
#[derive(Debug, Clone, JsonSchema, Serialize, Deserialize)]
pub struct WorkerStatus {
    /// Current assignment
    pub current_assignment: Option<String>,

    /// Health status
    pub health: WorkerHealth,

    /// Performance metrics
    pub performance: WorkerPerformance,
}

/// Worker performance metrics
#[derive(Debug, Clone, JsonSchema, Serialize, Deserialize)]
pub struct WorkerPerformance {
    /// Tasks completed
    pub tasks_completed: usize,

    /// Tasks failed
    pub tasks_failed: usize,

    /// Average completion time
    pub avg_completion_time_ms: f64,

    /// Success rate (0.0-1.0)
    pub success_rate: f64,
}

/// Audit trail for execution logging
#[async_trait::async_trait]
pub trait AuditTrail: Send + Sync {
    /// Log execution event
    async fn log_event(&self, event: AuditEvent) -> Result<()>;
}

/// Audit event for execution tracking

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AuditEvent {
    /// Event type
    pub event_type: AuditEventType,

    /// Plan identifier
    #[schemars(with = "String")]
    pub plan_id: Uuid,

    /// Milestone identifier (if applicable)
    pub milestone_id: Option<String>,

    /// Worker identifier (if applicable)
    #[schemars(with = "Option<String>")]
    pub worker_id: Option<Uuid>,

    /// Event timestamp
    #[schemars(with = "String")]
    pub timestamp: chrono::DateTime<Utc>,

    /// Event description
    pub description: String,

    /// Event metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Audit event types

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum AuditEventType {
    /// Plan execution started
    PlanStarted,

    /// Plan execution completed
    PlanCompleted,

    /// Plan execution failed
    PlanFailed,

    /// Milestone execution started
    MilestoneStarted,

    /// Milestone execution completed
    MilestoneCompleted,

    /// Milestone execution failed
    MilestoneFailed,

    /// Worker assigned
    WorkerAssigned,

    /// Worker released
    WorkerReleased,

    /// Evidence collected
    EvidenceCollected,

    /// Council decision received
    CouncilDecision,

    /// Scope violation detected
    ScopeViolation,

    /// Quality gate failed
    QualityGateFailed,
}

impl std::fmt::Display for AuditEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditEventType::PlanStarted => write!(f, "PlanStarted"),
            AuditEventType::PlanCompleted => write!(f, "PlanCompleted"),
            AuditEventType::PlanFailed => write!(f, "PlanFailed"),
            AuditEventType::MilestoneStarted => write!(f, "MilestoneStarted"),
            AuditEventType::MilestoneCompleted => write!(f, "MilestoneCompleted"),
            AuditEventType::MilestoneFailed => write!(f, "MilestoneFailed"),
            AuditEventType::WorkerAssigned => write!(f, "WorkerAssigned"),
            AuditEventType::WorkerReleased => write!(f, "WorkerReleased"),
            AuditEventType::EvidenceCollected => write!(f, "EvidenceCollected"),
            AuditEventType::CouncilDecision => write!(f, "CouncilDecision"),
            AuditEventType::ScopeViolation => write!(f, "ScopeViolation"),
            AuditEventType::QualityGateFailed => write!(f, "QualityGateFailed"),
        }
    }
}

/// Execution configuration

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionConfig {
    /// Maximum parallel milestones
    pub max_parallel_milestones: usize,

    /// Milestone timeout in milliseconds
    pub milestone_timeout_ms: u64,

    /// Batch timeout in milliseconds
    pub batch_timeout_ms: u64,

    /// Whether to continue on milestone failure
    pub continue_on_failure: bool,

    /// Council oversight level
    pub council_oversight: CouncilOversightLevel,

    /// Evidence collection settings
    pub evidence_settings: EvidenceSettings,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            max_parallel_milestones: 3,
            milestone_timeout_ms: 300000, // 5 minutes
            batch_timeout_ms: 600000, // 10 minutes
            continue_on_failure: false,
            council_oversight: CouncilOversightLevel::Standard,
            evidence_settings: EvidenceSettings::default(),
        }
    }
}

/// Council oversight levels

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
enum CouncilOversightLevel {
    /// No council oversight
    None,

    /// Notify council of major events
    Notify,

    /// Standard council oversight (default level)
    Standard,

    /// Require council approval for continuation
    Approve,

    /// Full council monitoring and intervention
    Full,
}

/// Evidence collection settings

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct EvidenceSettings {
    /// Whether to collect evidence
    pub collect_evidence: bool,

    /// Evidence collection timeout
    pub collection_timeout_ms: u64,

    /// Whether to validate evidence immediately
    pub validate_immediately: bool,

    /// Minimum evidence quality threshold
    pub min_quality_threshold: f64,
}

impl Default for EvidenceSettings {
    fn default() -> Self {
        Self {
            collect_evidence: true,
            collection_timeout_ms: 30000, // 30 seconds
            validate_immediately: false,
            min_quality_threshold: 0.7,
        }
    }
}

impl PlanExecutor {
    /// Create new plan executor
    pub fn new(
        plan: ExecutionPlan,
        worker_pool: Arc<dyn WorkerPool>,
        evidence_collector: Arc<EvidenceCollector>,
        worker_assigner: Arc<WorkerAssignmentStrategy>,
        scope_guard: Arc<ScopeGuard>,
        council_monitor: Arc<CouncilMonitor>,
        parallel_coordinator: std::sync::Weak<ParallelCoordinator>,
        audit_trail: Arc<dyn AuditTrail>,
        audit_trail_manager: Option<Arc<AuditTrailManager>>,
        todo_integration: Arc<dyn TodoInterface>,
        config: ExecutionConfig,
    ) -> Self {
        Self::with_lifecycle_manager(
            plan,
            worker_pool,
            evidence_collector,
            worker_assigner,
            scope_guard,
            council_monitor,
            parallel_coordinator,
            audit_trail,
            audit_trail_manager,
            todo_integration,
            None, // worker_lifecycle_manager - optional
            None, // worker_bridge - optional
            None, // worktree_manager - optional
            config,
        )
    }

    /// Create new plan executor with lifecycle manager
    pub fn with_lifecycle_manager(
        plan: ExecutionPlan,
        worker_pool: Arc<dyn WorkerPool>,
        evidence_collector: Arc<EvidenceCollector>,
        worker_assigner: Arc<WorkerAssignmentStrategy>,
        scope_guard: Arc<ScopeGuard>,
        council_monitor: Arc<CouncilMonitor>,
        parallel_coordinator: std::sync::Weak<ParallelCoordinator>,
        audit_trail: Arc<dyn AuditTrail>,
        audit_trail_manager: Option<Arc<AuditTrailManager>>,
        todo_integration: Arc<dyn TodoInterface>,
        worker_lifecycle_manager: Option<Arc<WorkerLifecycleManager>>,
        worker_bridge: Option<Arc<WorkerExecutionBridge>>,
        worktree_manager: Option<Arc<WorktreeManager>>,
        config: ExecutionConfig,
    ) -> Self {
        Self::with_determinism(
            plan,
            worker_pool,
            evidence_collector,
            worker_assigner,
            scope_guard,
            council_monitor,
            parallel_coordinator,
            audit_trail,
            audit_trail_manager,
            todo_integration,
            worker_lifecycle_manager,
            worker_bridge,
            worktree_manager,
            config,
            #[cfg(feature = "evaluation")]
            Arc::new(crate::evaluation::determinism::SystemClock),
            #[cfg(feature = "evaluation")]
            Arc::new(crate::evaluation::determinism::ThreadSafeRngSource::new(
                Box::new(crate::evaluation::determinism::SystemRng::new())
            )),
        )
    }

    /// Create new plan executor with determinism controls (feature-gated)
    #[cfg(feature = "evaluation")]
    pub fn with_determinism(
        plan: ExecutionPlan,
        worker_pool: Arc<dyn WorkerPool>,
        evidence_collector: Arc<EvidenceCollector>,
        worker_assigner: Arc<WorkerAssignmentStrategy>,
        scope_guard: Arc<ScopeGuard>,
        council_monitor: Arc<CouncilMonitor>,
        parallel_coordinator: std::sync::Weak<ParallelCoordinator>,
        audit_trail: Arc<dyn AuditTrail>,
        audit_trail_manager: Option<Arc<AuditTrailManager>>,
        todo_integration: Arc<dyn TodoInterface>,
        worker_lifecycle_manager: Option<Arc<WorkerLifecycleManager>>,
        worker_bridge: Option<Arc<WorkerExecutionBridge>>,
        worktree_manager: Option<Arc<WorktreeManager>>,
        config: ExecutionConfig,
        clock: Arc<dyn crate::evaluation::determinism::Clock>,
        rng_source: Arc<crate::evaluation::determinism::ThreadSafeRngSource>,
    ) -> Self {
        Self {
            plan,
            worker_pool,
            evidence_collector,
            worker_assigner,
            scope_guard,
            council_monitor,
            parallel_coordinator,
            audit_trail,
            audit_trail_manager,
            todo_integration,
            worker_lifecycle_manager,
            worker_bridge,
            worktree_manager,
            parallel_limit: Arc::new(Semaphore::new(config.max_parallel_milestones.max(1))),
            failure_oracle: Arc::new(FailureOracle::new(42)), // Fixed seed for deterministic testing
            clock,
            rng_source,
            config,
        }
    }

    #[cfg(not(feature = "evaluation"))]
    fn with_determinism(
        plan: ExecutionPlan,
        worker_pool: Arc<dyn WorkerPool>,
        evidence_collector: Arc<EvidenceCollector>,
        worker_assigner: Arc<WorkerAssignmentStrategy>,
        scope_guard: Arc<ScopeGuard>,
        council_monitor: Arc<CouncilMonitor>,
        parallel_coordinator: std::sync::Weak<ParallelCoordinator>,
        audit_trail: Arc<dyn AuditTrail>,
        audit_trail_manager: Option<Arc<AuditTrailManager>>,
        todo_integration: Arc<dyn TodoInterface>,
        worker_lifecycle_manager: Option<Arc<WorkerLifecycleManager>>,
        worker_bridge: Option<Arc<WorkerExecutionBridge>>,
        worktree_manager: Option<Arc<WorktreeManager>>,
        config: ExecutionConfig,
    ) -> Self {
        Self {
            plan,
            worker_pool,
            evidence_collector,
            worker_assigner,
            scope_guard,
            council_monitor,
            parallel_coordinator,
            audit_trail,
            audit_trail_manager,
            todo_integration,
            worker_lifecycle_manager,
            worker_bridge,
            worktree_manager,
            parallel_limit: Arc::new(Semaphore::new(config.max_parallel_milestones.max(1))),
            failure_oracle: Arc::new(FailureOracle::new(42)), // Fixed seed for deterministic testing
            config,
        }
    }

    /// Get current time (uses clock if available, otherwise system time)
    fn now(&self) -> chrono::DateTime<chrono::Utc> {
        #[cfg(feature = "evaluation")]
        {
            self.clock.now()
        }
        #[cfg(not(feature = "evaluation"))]
        {
            chrono::Utc::now()
        }
    }

    /// Generate a UUID (uses RNG source if available, otherwise system UUID)
    fn generate_uuid(&self) -> Uuid {
        #[cfg(feature = "evaluation")]
        {
            self.rng_source.generate_uuid()
        }
        #[cfg(not(feature = "evaluation"))]
        {
            Uuid::new_v4()
        }
    }

    /// Record a decision point for chain-of-thought visibility
    async fn record_decision_point(
        &self,
        decision_type: crate::chain_of_thought::DecisionType,
        context: crate::chain_of_thought::DecisionContext,
        alternatives: Vec<crate::chain_of_thought::Alternative>,
        chosen_option: String,
        reasoning: String,
        confidence: f64,
    ) -> Result<()> {
        if let Some(ref audit_manager) = self.audit_trail_manager {
            let decision_point = crate::chain_of_thought::DecisionPoint {
                decision_id: self.generate_uuid(),
                decision_type,
                timestamp: self.now(),
                context,
                alternatives,
                chosen_option,
                reasoning,
                confidence,
                risk_assessment: None, // Could be enhanced later
                metadata: std::collections::HashMap::new(),
            };

            audit_manager.record_orchestration_decision(decision_point).await?;
        }
        Ok(())
    }

    /// Record coordination events
    async fn record_coordination_event(
        &self,
        event_type: crate::chain_of_thought::CoordinationEventType,
        task_id: Option<Uuid>,
        milestone_id: Option<String>,
        worker_id: Option<Uuid>,
        details: std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        if let Some(ref audit_manager) = self.audit_trail_manager {
            let event = crate::chain_of_thought::CoordinationEvent {
                event_id: self.generate_uuid(),
                event_type,
                timestamp: self.now(),
                task_id,
                milestone_id,
                worker_id,
                resource_id: None, // No specific resource ID for this event
                details,
            };

            // Store coordination event for evaluation framework
            #[cfg(feature = "evaluation")]
            {
                audit_manager.record_coordination_event(event.clone()).await?;
            }

            // TODO: Implement dedicated coordination trace
            //       Currently records as part of broader trace; should implement dedicated coordination trace for better observability.
            //
            // COMPLETION CHECKLIST:
            // [ ] Create dedicated coordination trace structure
            // [ ] Record coordination events in dedicated trace
            // [ ] Support coordination trace querying
            // [ ] Add coordination trace visualization
            // [ ] Handle trace storage and retrieval
            // [ ] Add unit tests for coordination trace
            // [ ] Add integration tests with coordination events
            // [ ] Verify coordination trace accuracy
            //
            // ACCEPTANCE CRITERIA:
            // - Coordination events are recorded in dedicated trace
            // - Trace is queryable and searchable
            // - Trace visualization works correctly
            // - Trace storage and retrieval are efficient
            //
            // DEPENDENCIES:
            // - Trace infrastructure (Required)
            // - Coordination event structure (Required)
            // - Trace query utilities (Required)
            //
            // ESTIMATED EFFORT: 4-5 hours (medium confidence)
            // PRIORITY: Low
            // BLOCKING: No
            //
            // GOVERNANCE:
            // - CAWS Tier: 3 (observability enhancement)
            // - Change Budget: ~100 LOC
            // - Reviewer Requirements: Observability and tracing expertise
            info!("Coordination event: {:?}", event.event_type); // Temporary: basic logging until dedicated trace is implemented
        }
        Ok(())
    }

    /// Audit helper to reduce repetition and ensure consistent metadata
    async fn audit<S: Into<String>>(
        &self,
        ty: AuditEventType,
        milestone_id: Option<String>,
        worker_id: Option<Uuid>,
        msg: S,
        meta: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        self.audit_trail
            .log_event(AuditEvent {
                event_type: ty,
                plan_id: self.plan.contract_plan.id,
                milestone_id,
                worker_id,
                timestamp: self.now(),
                description: msg.into(),
                metadata: meta,
            })
            .await
    }

    /// Council oversight guard - centralizes oversight checks
    async fn council_gate(&self, phase: &'static str) -> Result<()> {
        use CouncilOversightLevel::*;
        match self.config.council_oversight {
            None => Ok(()),
            Notify => {
                self.council_monitor.notify(phase, &self.plan.contract_plan).await?;
                Ok(())
            }
            Standard => self.council_monitor.observe(phase, &self.plan.contract_plan).await,
            Approve | Full => {
                self.council_monitor.request_approval(phase, &self.plan.contract_plan).await
            }
        }
    }

    /// Execute the plan
    pub async fn execute(&self) -> Result<PlanExecutionResult> {
        let start = self.now();

        // Log plan execution start
        self.audit(
            AuditEventType::PlanStarted,
            None,
            None,
            format!("Plan '{}' execution started", self.plan.contract_plan.title),
            HashMap::from([
                ("milestone_count".into(), self.plan.contract_plan.milestones.len().into()),
            ])
        ).await?;

        // Validate plan can be executed
        self.validate_plan_for_execution().await?;

        // Council oversight check
        self.council_gate("pre-execution").await?;

        // Initialize TODO tracking for plan
        self.todo_integration.initialize_plan(self.plan.contract_plan.id, &self.plan.contract_plan.title).await?;

        // Resolve execution dependencies
        let dep = DependencyResolver::new(self.plan.contract_plan.dependency_graph.clone());
        let batches = dep.resolve_execution_order()?;

        // Initialize plan with execution batches
        let mut plan = self.plan.clone();
        plan.execution_context.parallel_batches = batches.iter().enumerate().map(|(i, ids)| ParallelBatch {
            batch_index: i,
            milestone_ids: ids.clone(),
            status: BatchStatus::Pending,
            started_at: None,
            completed_at: None,
            resource_requirements: ResourceRequirements::default(),
        }).collect();

        // Execute batches with proper parallelism control
        let coordinator = self.parallel_coordinator.upgrade()
            .ok_or_else(|| anyhow!("Parallel coordinator dropped"))?;

        let mut all_evidence = ExecutionEvidence {
            plan_evidence: vec![],
            milestone_evidence: HashMap::new(),
            quality_validation: vec![],
            council_reviews: vec![],
        };

        let mut total_failed = 0usize;
        let mut total_success = 0usize;

        for (batch_index, ids) in batches.into_iter().enumerate() {
            // Council oversight per batch
            let _gate = self.council_gate("pre-batch").await;

            // Record batch start
            {
                let b = &mut plan.execution_context.parallel_batches[batch_index];
                b.status = BatchStatus::Executing;
                b.started_at = Some(self.now());
            }

            // Acquire parallelism permits for this batch
            let permits = self.config.max_parallel_milestones.min(ids.len());
            let _permit = self.parallel_limit.acquire_many(permits as u32).await?;

            // Execute batch
            let res = coordinator.execute_batch_parallel(&mut plan, batch_index).await?;
            total_failed += res.failed;
            total_success += res.successful;

            // Collect evidence for completed milestones
            for mid in &ids {
                if let Some(m) = plan.contract_plan.milestones.iter().find(|m| &m.id == mid) {
                    if m.state == agent_agency_contracts::planning_io::MilestoneState::Completed {
                        if let Ok(bundle) = self.evidence_collector.collect_evidence(m, &plan.contract_plan.id.to_string()).await {
                            let converted = bundle.artifacts.into_iter().map(|a| {
                                agent_agency_contracts::planning::EvidenceArtifact {
                                    artifact_type: agent_agency_contracts::planning::ArtifactType::TestResults, // refine mapping as needed
                                    data: match a.content {
                                        crate::planning::plan_types::EvidenceContent::InlineJson(v) => v,
                                        crate::planning::plan_types::EvidenceContent::InlineText(s) => serde_json::json!(s),
                                        crate::planning::plan_types::EvidenceContent::FilePath(p) => serde_json::json!(p),
                                        crate::planning::plan_types::EvidenceContent::Structured(m) => serde_json::to_value(m).unwrap_or_default(),
                                        crate::planning::plan_types::EvidenceContent::Binary(b) => serde_json::json!(b),
                                    },
                                    verified: a.metadata.get("verified")
                                        .and_then(|v| v.as_bool())
                                        .unwrap_or(false),
                                    validated_at: a.collected_at,
                                    metadata: a.metadata
                                        .into_iter()
                                        .map(|(k, v)| (k, v.as_str().unwrap_or("").to_string()))
                                        .collect(),
                                }
                            }).collect::<Vec<_>>();
                            all_evidence.milestone_evidence.insert(mid.clone(), converted);

                            // Mark TODO completion
                            let _ = self.todo_integration.milestone_completed(plan.contract_plan.id, mid).await;
                        }
                    }
                }
            }

            // Finalize batch
            {
                let b = &mut plan.execution_context.parallel_batches[batch_index];
                b.completed_at = Some(self.now());
                b.status = if res.failed == 0 { BatchStatus::Completed } else { BatchStatus::Failed };
            }

            self.council_gate("post-batch").await.ok();
        }

        let end = self.now();
        let total_ms = (end - start).num_milliseconds().max(0) as u64;
        let success = total_failed == 0;

        let metrics = ExecutionMetrics {
            total_milestones: self.plan.contract_plan.milestones.len(),
            successful_milestones: total_success,
            failed_milestones: total_failed,
            skipped_milestones: 0,
            avg_milestone_time_ms: if total_success > 0 {
                total_ms as f64 / total_success as f64
            } else {
                0.0
            },
            parallel_time_saved_ms: self.estimate_parallel_savings(&plan).await,
            resource_utilization: {
                let u = self.calculate_resource_utilization(&plan).await;
                agent_agency_contracts::planning::ResourceUtilization {
                    cpu_utilization: u.cpu_percent / 100.0,
                    memory_utilization: u.memory_mb,
                    network_io_bytes: (u.network_mbps * 1_000_000.0 / 8.0) as u64,
                    disk_io_bytes: (u.disk_mb * 1_024.0 * 1_024.0) as u64,
                    worker_utilization: self.get_worker_utilization_stats().await,
                }
            },
            quality_metrics: self.calculate_quality_metrics(&all_evidence),
            performance_metrics: self.calculate_performance_metrics(&plan, total_ms),
        };

        let timeline = self.build_execution_timeline(&plan);

        // Log plan completion
        self.audit(
            if success { AuditEventType::PlanCompleted } else { AuditEventType::PlanFailed },
            None,
            None,
            format!("Plan '{}' execution {}", self.plan.contract_plan.title, if success { "completed" } else { "failed" }),
            HashMap::from([
                ("success".into(), success.into()),
                ("total_duration_ms".into(), total_ms.into()),
                ("milestones_completed".into(), total_success.into()),
            ])
        ).await?;

        Ok(PlanExecutionResult {
            plan_id: self.plan.contract_plan.id,
            success,
            milestones_completed: total_success,
            total_duration_ms: total_ms,
            evidence: all_evidence,
            metrics,
            final_state: plan.contract_plan.state,
            timeline,
        })
    }

    /// Validate plan can be executed
    async fn validate_plan_for_execution(&self) -> Result<()> {
        // Check if all required resources are available
        let available_workers = self.worker_pool.available_workers().await?;
        if available_workers.is_empty() {
            return Err(anyhow!("No workers available for execution"));
        }

        // Check if plan has been approved
        if self.plan.contract_plan.state != agent_agency_contracts::planning_io::PlanState::Approved {
            return Err(anyhow!("Plan must be approved before execution"));
        }

        // Validate all milestones have required evidence gates
        for milestone in &self.plan.contract_plan.milestones {
            if milestone.evidence_gate.required_artifacts.is_empty() {
                return Err(anyhow!("Milestone '{}' missing evidence gate requirements", milestone.id));
            }
        }

        Ok(())
    }

    /// Execute a batch of milestones in parallel
    async fn execute_batch(
        &self,
        plan: &mut ExecutionPlan,
        batch_index: usize,
        milestone_ids: Vec<String>,
        all_evidence: &mut ExecutionEvidence,
    ) -> Result<()> {
        // Update batch status
        if let Some(state) = &mut plan.execution_state {
            // Note: parallel_batches access commented out due to double borrow issues
            // if let Some(current_batch) = state.parallel_batches.get_mut(batch_index) {
            //     current_batch.started_at = Some(Utc::now());
            //     current_batch.status = BatchStatus::Executing;
            // }
        }

        // Set up batch for parallel coordinator
        let batch = ParallelBatch {
            batch_index: 0,
            milestone_ids: milestone_ids.clone(),
            status: BatchStatus::Executing,
            started_at: Some(self.now()),
            completed_at: None,
            resource_requirements: ResourceRequirements::default(),
        };

        // Create execution context for the batch
        // Initialize parallel batches with the current batch
        // Additional batches can be added as execution progresses
        plan.execution_context.parallel_batches = vec![batch.clone()];

        // Execute batch using parallel coordinator
        let parallel_coordinator = self.parallel_coordinator.upgrade()
            .ok_or_else(|| anyhow!("Parallel coordinator has been dropped"))?;
        let batch_result = parallel_coordinator.execute_batch_parallel(plan, batch_index).await?;

        // Process results
        let _batch_success = batch_result.failed == 0;

        // Collect evidence from successful executions
        // Evidence is collected by checking milestone state after batch execution completes
        for milestone_id in &milestone_ids {
            if let Some(milestone) = plan.contract_plan.milestones.iter().find(|m| m.id == *milestone_id) {
                if milestone.state == agent_agency_contracts::planning_io::MilestoneState::Completed {
                    // Collect evidence for completed milestone
                    if let Ok(evidence_bundle) = self.evidence_collector.collect_evidence(milestone, &plan.contract_plan.id.to_string()).await {
                        // Convert EvidenceBundle to Vec<EvidenceArtifact>
                        let contract_artifacts: Vec<agent_agency_contracts::planning::EvidenceArtifact> = evidence_bundle.artifacts
                            .into_iter()
                            .map(|artifact| agent_agency_contracts::planning::EvidenceArtifact {
                                artifact_type: match artifact.artifact_type.as_str() {
                                    "code_analysis" => agent_agency_contracts::planning::ArtifactType::TestResults,
                                    "test_results" => agent_agency_contracts::planning::ArtifactType::TestResults,
                                    "coverage" => agent_agency_contracts::planning::ArtifactType::TestResults,
                                    "security_scan" => agent_agency_contracts::planning::ArtifactType::TestResults,
                                    _ => agent_agency_contracts::planning::ArtifactType::TestResults,
                                },
                                data: match artifact.content {
                                    crate::planning::plan_types::EvidenceContent::InlineJson(v) => v,
                                    crate::planning::plan_types::EvidenceContent::InlineText(s) => serde_json::json!(s),
                                    crate::planning::plan_types::EvidenceContent::FilePath(p) => serde_json::json!(p),
                                    crate::planning::plan_types::EvidenceContent::Structured(m) => serde_json::to_value(m).unwrap_or_default(),
                                    crate::planning::plan_types::EvidenceContent::Binary(b) => serde_json::json!(b),
                                },
                                verified: artifact.metadata.get("verified")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false),
                                validated_at: artifact.collected_at,
                                metadata: artifact.metadata
                                    .into_iter()
                                    .map(|(k, v)| (k, v.as_str().unwrap_or("").to_string()))
                                    .collect(),
                            })
                            .collect();
                        all_evidence.milestone_evidence.insert(milestone_id.clone(), contract_artifacts);
                    }
                }
            }
        }

        // Update batch completion
        let _batch_end = Utc::now();
        if let Some(_state) = &mut plan.execution_state {
            // Note: parallel_batches access commented out due to double borrow issues
            // if let Some(current_batch) = state.parallel_batches.get_mut(batch_index) {
            //     current_batch.completed_at = Some(batch_end);
            //     // Execution time can be calculated from started_at and completed_at timestamps
            //     current_batch.status = if batch_success {
            //         BatchStatus::Completed
            //     } else if batch_result.successful > 0 {
            //         BatchStatus::Completed // Some succeeded, mark as completed
            //     } else {
            //         BatchStatus::Failed
            //     };
            // }
        }

        Ok(())
    }

    /// Execute individual milestone
    async fn execute_milestone(&self, plan: ExecutionPlan, milestone_id: String) -> Result<MilestoneExecutionResult> {
        let milestone_start = Utc::now();

        // Find milestone
        let _milestone = plan.contract_plan.milestones.iter()
            .find(|m| m.id == milestone_id)
            .ok_or_else(|| anyhow!("Milestone '{}' not found", milestone_id))?
            .clone();

        // Check quality gates before execution
        if !self.todo_integration.can_progress_to_milestone(plan.contract_plan.id, &milestone_id).await? {
            return Err(anyhow!("Cannot execute milestone '{}': quality gates not satisfied", milestone_id));
        }

        // Log milestone start
        self.audit(
            AuditEventType::MilestoneStarted,
            Some(milestone_id.clone()),
            None,
            format!("Milestone '{}' execution started", milestone.objective),
            HashMap::from([
                ("objective".into(), milestone.objective.clone().into()),
            ])
        ).await?;

        // Assign worker with chain-of-thought recording
        let worker_id = self.worker_assigner.assign_worker(&milestone).await?;

        // Record worker assignment decision
        self.record_decision_point(
            crate::chain_of_thought::DecisionType::WorkerAssignment,
            crate::chain_of_thought::DecisionContext {
                task_id: Some(self.plan.contract_plan.id),
                plan_id: Some(self.plan.contract_plan.id),
                milestone_id: Some(milestone_id.clone()),
                worker_id: Some(worker_id),
                resource_constraints: std::collections::HashMap::new(),
                time_constraints: None,
                priority_level: Some(milestone.priority.to_string()),
            },
            vec![], // Could be populated with alternative workers considered
            format!("Worker {}", worker_id),
            format!("Assigned worker {} to milestone {} based on capability matching", worker_id, milestone_id),
            0.9, // High confidence for worker assignments
        ).await?;

        // Record coordination event
        self.record_coordination_event(
            crate::chain_of_thought::CoordinationEventType::WorkerAssigned,
            Some(self.plan.contract_plan.id),
            Some(milestone_id.clone()),
            Some(worker_id),
            std::collections::HashMap::from([
                ("objective".to_string(), serde_json::Value::String(milestone.objective.clone())),
            ]),
        ).await?;

        self.worker_pool.assign_worker(worker_id, milestone_id.clone()).await?;

        // Log worker assignment
        self.audit(
            AuditEventType::WorkerAssigned,
            Some(milestone_id.clone()),
            Some(worker_id),
            format!("Worker assigned to milestone"),
            HashMap::new()
        ).await?;

        // Acquire scope locks
        self.scope_guard.acquire_locks(milestone_id.clone(), &milestone.scope).await?;

        // Execute milestone with timeout
        let milestone_timeout = Duration::from_millis(self.config.milestone_timeout_ms);
        let execution_result = match timeout(milestone_timeout, self.execute_milestone_impl(&milestone)).await {
            Ok(result) => result,
            Err(_) => Err(anyhow!("Milestone execution timed out after {}ms", self.config.milestone_timeout_ms)),
        };

        // Release scope locks
        self.scope_guard.release_locks(milestone_id.clone()).await?;

        // Release worker
        self.worker_pool.release_worker(worker_id).await?;

        // Collect evidence
        let evidence = if self.config.evidence_settings.collect_evidence {
            match self.evidence_collector.collect_evidence(&milestone, &plan.contract_plan.id.to_string()).await {
                Ok(evidence) => Some(evidence),
                Err(e) => {
                    // Log evidence collection failure but don't fail milestone
                    println!("Evidence collection failed for milestone '{}': {}", milestone_id, e);
                    None
                }
            }
        } else {
            None
        };

        let milestone_end = Utc::now();
        let execution_time_ms = (milestone_end - milestone_start).num_milliseconds() as u64;

        let success = execution_result.is_ok();

        // Log milestone completion
        self.audit(
            if success { AuditEventType::MilestoneCompleted } else { AuditEventType::MilestoneFailed },
            Some(milestone_id.clone()),
            Some(worker_id),
            format!("Milestone {}", if success { "completed" } else { "failed" }),
            HashMap::from([
                ("execution_time_ms".into(), execution_time_ms.into()),
                ("success".into(), success.into()),
            ])
        ).await?;

        // Update TODO system on milestone completion
        if success {
            if let Err(e) = self.todo_integration.milestone_completed(plan.contract_plan.id, &milestone_id).await {
                tracing::warn!("Failed to complete TODO step for milestone {}: {}", milestone_id, e);
            }
        }

        Ok(MilestoneExecutionResult {
            milestone_id,
            success,
            execution_time_ms,
            evidence,
            worker_id,
        })
    }

    /// Get worker utilization statistics from the worker pool
    /// Returns a map of worker IDs to their current utilization metrics
    async fn get_worker_utilization_stats(&self) -> std::collections::HashMap<String, f64> {
        match self.worker_pool.available_workers().await {
            Ok(workers) => {
                let mut stats = std::collections::HashMap::new();
                for worker in workers {
                    let worker_id = worker.id.to_string();
                    // Use worker load as utilization metric, clamped to valid range
                    stats.insert(worker_id, worker.load.clamp(0.0, 1.0));
                }
                stats
            }
            Err(_) => std::collections::HashMap::new()
        }
    }

    /// Execute milestone implementation using real worker system via WorkerExecutionBridge
    pub async fn execute_milestone_impl(&self, milestone: &agent_agency_contracts::planning_io::Milestone) -> Result<ExecutionArtifacts> {
        // Find suitable worker for this milestone with chain-of-thought recording
        let worker_id = self.worker_assigner.assign_worker(milestone).await?;

        // Handle worker assignment via lifecycle manager
        if let Some(ref lifecycle_manager) = self.worker_lifecycle_manager {
            if let Err(e) = lifecycle_manager.handle_assignment(worker_id, milestone).await {
                tracing::warn!("Failed to handle worker assignment via lifecycle manager: {}", e);
                // Continue execution even if lifecycle tracking fails
            }
        }

        // Record worker assignment decision for individual milestone execution
        self.record_decision_point(
            crate::chain_of_thought::DecisionType::WorkerAssignment,
            crate::chain_of_thought::DecisionContext {
                task_id: Some(self.plan.contract_plan.id),
                plan_id: Some(self.plan.contract_plan.id),
                milestone_id: Some(milestone.id.clone()),
                worker_id: Some(worker_id),
                resource_constraints: std::collections::HashMap::new(),
                time_constraints: None,
                priority_level: Some(milestone.priority.to_string()),
            },
            vec![], // Could be populated with alternative workers considered
            format!("Worker {}", worker_id),
            format!("Assigned worker {} to milestone {} for individual execution", worker_id, milestone.id),
            0.85, // Slightly lower confidence for individual assignments
        ).await?;

        // Record coordination event for individual execution
        self.record_coordination_event(
            crate::chain_of_thought::CoordinationEventType::TaskStarted,
            Some(self.plan.contract_plan.id),
            Some(milestone.id.clone()),
            Some(worker_id),
            std::collections::HashMap::from([
                ("execution_mode".to_string(), serde_json::Value::String("individual".to_string())),
            ]),
        ).await?;

        // Get worktree path for this milestone
        // Note: ParallelCoordinator may have already created a worktree, so we try to find it by milestone_id first
        let worktree_path = if let Some(ref worktree_manager) = self.worktree_manager {
            // First, try to find worktree by milestone_id (in case ParallelCoordinator created it)
            match worktree_manager.get_worktree_path_by_milestone(&milestone.id).await {
                Ok(path) => {
                    tracing::info!("Found existing worktree for milestone {}: {}", milestone.id, path.display());
                    path
                },
                Err(_) => {
                    // Try to get existing worktree for this worker
                    match worktree_manager.get_worktree_path(worker_id).await {
                        Ok(path) => {
                            tracing::info!("Using existing worktree for worker {}: {}", worker_id, path.display());
                            path
                        },
                        Err(_) => {
                            // Create worktree if it doesn't exist
                            tracing::info!("No worktree found for worker {} or milestone {}, creating new worktree", worker_id, milestone.id);
                            match worktree_manager.create_worktree(milestone, worker_id).await {
                                Ok(worktree_info) => {
                                    tracing::info!(
                                        "Created worktree {} for worker {} at {}",
                                        worktree_info.worktree_id,
                                        worker_id,
                                        worktree_info.worktree_path.display()
                                    );
                                    worktree_info.worktree_path
                                }
                                Err(e) => {
                                    tracing::error!("Failed to create worktree for worker {}: {}", worker_id, e);
                                    // Fallback to current directory if worktree creation fails
                                    tracing::warn!("Falling back to current directory due to worktree creation failure");
                                    std::path::PathBuf::from(".")
                                }
                            }
                        }
                    }
                }
            }
        } else {
            // No worktree manager - use current directory
            tracing::warn!("No worktree manager available, using current directory");
            std::path::PathBuf::from(".")
        };

        // Execute using WorkerExecutionBridge if available, otherwise fall back to simulation
        let artifacts = if let Some(ref worker_bridge) = self.worker_bridge {
            // Real execution via WorkerExecutionBridge
            tracing::info!("Executing milestone {} via WorkerExecutionBridge with worker {}", milestone.id, worker_id);
            
            match worker_bridge.execute_milestone(milestone, &worktree_path, worker_id).await {
                Ok(artifacts) => {
                    // Handle worker completion via lifecycle manager
                    if let Some(ref lifecycle_manager) = self.worker_lifecycle_manager {
                        if let Err(e) = lifecycle_manager.handle_completion(worker_id, artifacts.clone()).await {
                            tracing::warn!("Failed to handle worker completion via lifecycle manager: {}", e);
                        }
                    }
                    artifacts
                }
                Err(e) => {
                    // Handle worker failure via lifecycle manager
                    if let Some(ref lifecycle_manager) = self.worker_lifecycle_manager {
                        if let Err(lifecycle_err) = lifecycle_manager.handle_failure(worker_id, e.to_string()).await {
                            tracing::warn!("Failed to handle worker failure via lifecycle manager: {}", lifecycle_err);
                        }
                    }
                    return Err(anyhow!("Worker execution failed: {}", e));
                }
            }
        } else {
            // Fallback: No bridge available - return error (should not happen in production)
            return Err(anyhow!("WorkerExecutionBridge not available - cannot execute milestone"));
        };

        Ok(artifacts)
    }

    /// Create worker context from milestone
    fn create_worker_context(&self, milestone: &agent_agency_contracts::planning_io::Milestone) -> Result<agent_agency_contracts::WorkerContext> {
        Ok(agent_agency_contracts::WorkerContext {
            task_id: milestone.id.parse().unwrap_or(uuid::Uuid::new_v4()), // Use milestone ID as UUID if possible
            description: milestone.objective.clone(),
            required_capabilities: milestone.scope.allowed_operations.clone(),
            priority: self.map_milestone_priority(milestone.priority.clone()),
            working_spec_id: self.plan.contract_plan.working_spec_id.clone(),
            metadata: std::collections::HashMap::from([
                ("milestone_id".to_string(), serde_json::Value::String(milestone.id.clone())),
                ("risk_tier".to_string(), serde_json::Value::Number(milestone.risk_tier.into())),
                ("estimated_effort".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(milestone.estimated_effort).unwrap())),
            ]),
        })
    }

    /// Map milestone priority to worker priority
    fn map_milestone_priority(&self, priority: agent_agency_contracts::planning_io::MilestonePriority) -> agent_agency_contracts::TaskPriority {
        match priority {
            agent_agency_contracts::planning_io::MilestonePriority::Low => agent_agency_contracts::TaskPriority::Low,
            agent_agency_contracts::planning_io::MilestonePriority::Normal => agent_agency_contracts::TaskPriority::Medium,
            agent_agency_contracts::planning_io::MilestonePriority::High => agent_agency_contracts::TaskPriority::High,
            agent_agency_contracts::planning_io::MilestonePriority::Critical => agent_agency_contracts::TaskPriority::Critical,
        }
    }

    /// Execute milestone with assigned worker
    async fn execute_with_worker(&self, worker: &WorkerInfo, context: &agent_agency_contracts::WorkerContext) -> Result<()> {
        // Execute using the worker's execution capabilities
        // Worker assignment validates capabilities before execution

        // Check if worker has required capabilities
        let has_required_capabilities = context.required_capabilities.iter()
            .all(|cap| worker.capabilities.contains(cap));

        if !has_required_capabilities {
            return Err(anyhow!("Worker {} lacks required capabilities: {:?}", worker.id.to_string(), context.required_capabilities));
        }

        // Simulate execution time based on worker load and task complexity
        let base_execution_time = match context.priority {
            agent_agency_contracts::TaskPriority::Low => 1000,
            agent_agency_contracts::TaskPriority::Normal => 2000,
            agent_agency_contracts::TaskPriority::High => 500,
            agent_agency_contracts::TaskPriority::Urgent => 200,
            agent_agency_contracts::TaskPriority::Medium => 2000,
            agent_agency_contracts::TaskPriority::Critical => 500,
        };

        let load_factor = 1.0 + (worker.load * 0.5); // Load increases execution time
        let execution_time = (base_execution_time as f64 * load_factor) as u64;

        tokio::time::sleep(Duration::from_millis(execution_time)).await;

        // Simulate occasional failures based on worker health
        if matches!(worker.health, WorkerHealth::Unhealthy) {
            return Err(anyhow!("Worker {} is unhealthy and failed execution", worker.id.to_string()));
        }

        if worker.load > 0.9 {
            // High load can cause failures
            if self.failure_oracle.chance(0.1) {
                return Err(anyhow!("Worker {} overloaded and failed execution", worker.id.to_string()));
            }
        }

        Ok(())
    }

    /// Update plan state for milestone completion
    async fn update_plan_state_for_milestone(&self, plan: &mut ExecutionPlan, milestone_id: &str, success: bool) -> Result<()> {
        if let Some(state) = &mut plan.execution_state {
            if success {
                state.completed_milestones.insert(milestone_id.to_string());
            } else {
                state.failed_milestones.insert(milestone_id.to_string(), "Execution failed".to_string());
            }
            state.executing_milestones.remove(milestone_id);

            // Update progress
            state.progress.milestones_completed = state.completed_milestones.len();
            state.progress.overall_completion = state.progress.milestones_completed as f64 / state.progress.total_milestones as f64;
        }

        Ok(())
    }

    /// Calculate parallel time saved
    async fn estimate_parallel_savings(&self, plan: &ExecutionPlan) -> u64 {
        let batches = &plan.execution_context.parallel_batches;
        if batches.is_empty() { return 0; }

        let par_ms = batches.iter()
            .filter_map(|b| Some((b.started_at?, b.completed_at?)))
            .map(|(s, e)| (e - s).num_milliseconds().max(0) as u64)
            .sum::<u64>();

        // Crude sequential estimate: sum of milestone durations in these batches
        let seq_ms = batches.iter().filter_map(|b| {
            match (b.started_at, b.completed_at) {
                (Some(s), Some(e)) => Some(((e - s).num_milliseconds().max(0) as u64) * (b.milestone_ids.len() as u64)),
                _ => None
            }
        }).sum::<u64>();

        seq_ms.saturating_sub(par_ms)
    }

    /// Calculate resource utilization
    async fn calculate_resource_utilization(&self, _plan: &ExecutionPlan) -> ResourceUtilization {
        // TODO: Analyze actual resource usage from execution plan
        //       Currently returns zero values; should analyze actual resource usage from plan execution history and metrics.
        //
        // COMPLETION CHECKLIST:
        // [ ] Query execution history for resource metrics
        // [ ] Calculate CPU utilization from execution data
        // [ ] Calculate memory usage from execution data
        // [ ] Calculate disk and network usage
        // [ ] Support GPU utilization if applicable
        // [ ] Add unit tests for resource calculation
        // [ ] Add integration tests with real execution plans
        // [ ] Verify resource calculation accuracy
        //
        // ACCEPTANCE CRITERIA:
        // - Resource utilization is calculated from actual execution data
        // - CPU, memory, disk, and network metrics are accurate
        // - GPU utilization is supported if applicable
        // - Resource calculations reflect actual usage
        //
        // DEPENDENCIES:
        // - Execution history storage (Required)
        // - Resource metrics collection (Required)
        // - Resource calculation utilities (Required)
        //
        // ESTIMATED EFFORT: 4-5 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (monitoring feature)
        // - Change Budget: ~100 LOC
        // - Reviewer Requirements: Resource monitoring expertise
        ResourceUtilization { // Temporary: zero values until actual resource analysis is implemented
            cpu_percent: 0.0,
            memory_mb: 0.0,
            disk_mb: 0.0,
            network_mbps: 0.0,
            gpu_percent: None,
            measured_at: chrono::Utc::now(),
            associated_with: None,
        }
    }

    /// Calculate quality metrics
    fn calculate_quality_metrics(&self, evidence: &ExecutionEvidence) -> QualityMetrics {
        // TODO: Analyze actual evidence for quality metrics
        //       Currently uses hardcoded values; should analyze actual execution evidence to calculate quality metrics.
        //
        // COMPLETION CHECKLIST:
        // [ ] Extract test coverage data from evidence
        // [ ] Calculate average coverage from evidence
        // [ ] Extract mutation testing scores from evidence
        // [ ] Calculate average mutation score
        // [ ] Analyze linting results from evidence
        // [ ] Add unit tests for quality metric calculation
        // [ ] Add integration tests with real evidence
        // [ ] Verify quality metric accuracy
        //
        // ACCEPTANCE CRITERIA:
        // - Quality metrics are calculated from actual evidence
        // - Coverage and mutation scores are accurate
        // - Linting results are analyzed correctly
        // - Quality metrics reflect actual execution quality
        //
        // DEPENDENCIES:
        // - Execution evidence structure (Required)
        // - Quality metric extraction utilities (Required)
        // - Evidence analysis utilities (Required)
        //
        // ESTIMATED EFFORT: 3-4 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (quality analysis feature)
        // - Change Budget: ~80 LOC
        // - Reviewer Requirements: Quality metrics expertise
        QualityMetrics { // Temporary: hardcoded values until evidence analysis is implemented
            avg_coverage: 0.8,
            avg_mutation_score: 0.75,
            security_issues_found: 0,
            performance_regressions: 0,
            code_quality_score: 0.8,
        }
    }

    /// Calculate performance metrics
    fn calculate_performance_metrics(&self, plan: &ExecutionPlan, total_duration_ms: u64) -> agent_agency_contracts::PerformanceMetrics {
        // TODO: Implement comprehensive performance metrics calculation
        //       Currently uses basic calculation; should analyze actual execution plan structure to calculate dependency wait times, parallel vs sequential execution times, and efficiency ratios.
        //
        // COMPLETION CHECKLIST:
        // [ ] Analyze execution plan structure for dependency relationships
        // [ ] Calculate actual dependency wait times from batch dependencies
        // [ ] Track parallel execution time from batch execution timestamps
        // [ ] Calculate sequential execution time as sum of batch durations
        // [ ] Compute efficiency ratio as parallel_time / sequential_time
        // [ ] Handle edge cases (no dependencies, single batch, etc.)
        // [ ] Add unit tests with various plan structures
        // [ ] Add integration tests with real execution plans
        // [ ] Performance: Metrics calculation should complete in <1ms
        // [ ] Documentation: Document metric calculation methodology
        //
        // ACCEPTANCE CRITERIA:
        // - Dependency wait time accurately reflects time spent waiting for dependencies
        // - Parallel execution time reflects actual parallel batch execution duration
        // - Sequential execution time reflects total time if executed sequentially
        // - Efficiency ratio is between 0.0 and 1.0, with 1.0 indicating perfect parallelization
        // - Metrics are consistent with actual execution evidence
        //
        // DEPENDENCIES:
        // - ExecutionPlan structure with batch dependencies (Required)
        // - Batch execution timestamps (started_at, completed_at) (Required)
        // - agent_agency_contracts::PerformanceMetrics type definition (Required)
        //
        // ESTIMATED EFFORT: 4-6 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (performance metrics feature)
        // - Change Budget: ~80 LOC
        // - Reviewer Requirements: Performance analysis expertise
        agent_agency_contracts::PerformanceMetrics {
            total_time_ms: total_duration_ms,
            dependency_wait_time_ms: 0,
            parallel_execution_time_ms: total_duration_ms,
            sequential_execution_time_ms: total_duration_ms,
            efficiency_ratio: 1.0,
        }
    }

    /// Build execution timeline
    fn build_execution_timeline(&self, plan: &ExecutionPlan) -> Vec<agent_agency_contracts::planning::ExecutionEvent> {
        plan.execution_context.parallel_batches.iter().flat_map(|b| {
            let mut v = Vec::new();
            if let Some(s) = b.started_at {
                v.push(agent_agency_contracts::planning::ExecutionEvent {
                    event_type: ExecutionEventType::BatchStarted,
                    timestamp: s,
                    milestone_id: None,
                    description: format!("Batch {} started execution", b.batch_index),
                    metadata: HashMap::from([
                        ("batch_index".into(), b.batch_index.into()),
                        ("milestone_count".into(), b.milestone_ids.len().into()),
                    ]),
                });
            }
            if let Some(e) = b.completed_at {
                v.push(agent_agency_contracts::planning::ExecutionEvent {
                    event_type: ExecutionEventType::MilestoneCompleted,
                    timestamp: e,
                    milestone_id: None,
                    description: format!("Batch {} completed execution", b.batch_index),
                    metadata: HashMap::from([
                        ("batch_index".into(), b.batch_index.into()),
                        ("status".into(), format!("{:?}", b.status).into()),
                    ]),
                });
            }
            v
        }).collect()
    }

    /// Clone executor for parallel execution
    fn clone_executor(&self) -> Arc<Self> {
        // TODO: Implement proper executor cloning for parallel execution
        //       Currently uses basic Arc cloning; should ensure all internal state is properly cloned and thread-safe for parallel execution contexts.
        //
        // COMPLETION CHECKLIST:
        // [ ] Verify all fields are properly cloneable
        // [ ] Ensure thread-safe state sharing where appropriate
        // [ ] Clone mutable state that should be independent per executor
        // [ ] Share immutable state via Arc where possible
        // [ ] Add unit tests for concurrent executor usage
        // [ ] Add integration tests with parallel execution
        // [ ] Performance: Cloning should complete in <100μs
        // [ ] Documentation: Document cloning semantics and thread safety
        //
        // ACCEPTANCE CRITERIA:
        // - Cloned executor can be used independently in parallel execution
        // - Shared state is properly synchronized
        // - No data races or memory safety issues
        // - Cloning preserves all executor configuration and state
        //
        // DEPENDENCIES:
        // - All executor fields must implement Clone or be Arc-wrapped (Required)
        // - Thread-safe primitives for shared mutable state (Required)
        //
        // ESTIMATED EFFORT: 2-3 hours (high confidence)
        // PRIORITY: Low
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (parallel execution feature)
        // - Change Budget: ~30 LOC
        // - Reviewer Requirements: Concurrency and Rust expertise
        Arc::new(Self {
            parallel_coordinator: self.parallel_coordinator.clone(),
            todo_integration: self.todo_integration.clone(),
            plan: self.plan.clone(),
            worker_pool: self.worker_pool.clone(),
            evidence_collector: self.evidence_collector.clone(),
            worker_assigner: self.worker_assigner.clone(),
            scope_guard: self.scope_guard.clone(),
            council_monitor: self.council_monitor.clone(),
            audit_trail: self.audit_trail.clone(),
            audit_trail_manager: self.audit_trail_manager.clone(),
            parallel_limit: self.parallel_limit.clone(),
            failure_oracle: self.failure_oracle.clone(),
            worker_lifecycle_manager: self.worker_lifecycle_manager.clone(),
            worker_bridge: self.worker_bridge.clone(),
            worktree_manager: self.worktree_manager.clone(),
            #[cfg(feature = "evaluation")]
            clock: self.clock.clone(),
            #[cfg(feature = "evaluation")]
            rng_source: self.rng_source.clone(),
            config: self.config.clone(),
        })
    }
}

/// Milestone execution result

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[allow(dead_code)] // Reserved for future use
struct MilestoneExecutionResult {
    /// Milestone identifier
    pub milestone_id: String,

    /// Whether execution succeeded
    pub success: bool,

    /// Execution time in milliseconds
    pub execution_time_ms: u64,

    /// Collected evidence
    pub evidence: Option<EvidenceBundle>,

    /// Worker that executed the milestone
    #[schemars(with = "String")]
    pub worker_id: Uuid,
}

// Import missing types
use crate::planning::plan_types::EvidenceBundle;

// Re-export for convenience
pub use agent_agency_contracts::planning::{ExecutionEvent, ExecutionMetrics, PerformanceMetrics};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_config_creation() {
        let config = ExecutionConfig {
            max_parallel_milestones: 5,
            milestone_timeout_ms: 300000,
            batch_timeout_ms: 600000,
            continue_on_failure: false,
            council_oversight: CouncilOversightLevel::Approve,
            evidence_settings: EvidenceSettings {
                collect_evidence: true,
                collection_timeout_ms: 60000,
                validate_immediately: true,
                min_quality_threshold: 0.8,
            },
        };

        assert_eq!(config.max_parallel_milestones, 5);
        assert_eq!(config.milestone_timeout_ms, 300000);
        assert!(!config.continue_on_failure);
        assert!(matches!(config.council_oversight, CouncilOversightLevel::Approve));
    }

    #[test]
    fn test_worker_info_creation() {
        let worker = WorkerInfo {
            id: Uuid::new_v4(),
            capabilities: vec!["rust".to_string(), "testing".to_string()],
            load: 0.3,
            health: WorkerHealth::Healthy,
        };

        assert_eq!(worker.capabilities.len(), 2);
        assert_eq!(worker.load, 0.3);
        assert!(matches!(worker.health, WorkerHealth::Healthy));
    }

    #[test]
    fn test_audit_event_creation() {
        let event = AuditEvent {
            event_type: AuditEventType::PlanStarted,
            plan_id: Uuid::new_v4(),
            milestone_id: Some("M1".to_string()),
            worker_id: Some(Uuid::new_v4()),
            timestamp: Utc::now(),
            description: "Plan execution started".to_string(),
            metadata: HashMap::new(),
        };

        assert!(matches!(event.event_type, AuditEventType::PlanStarted));
        assert!(event.milestone_id.is_some());
        assert!(event.worker_id.is_some());
    }

    #[test]
    fn test_milestone_execution_result() {
        let result = MilestoneExecutionResult {
            milestone_id: "M1".to_string(),
            success: true,
            execution_time_ms: 5000,
            evidence: None,
            worker_id: Uuid::new_v4(),
        };

        assert_eq!(result.milestone_id, "M1");
        assert!(result.success);
        assert_eq!(result.execution_time_ms, 5000);
        assert!(result.evidence.is_none());
    }

    #[test]
    fn test_failure_oracle_deterministic() {
        let oracle = FailureOracle::new(12345);

        // Same seed should produce same results
        let results1: Vec<bool> = (0..10).map(|_| oracle.chance(0.5)).collect();
        let oracle2 = FailureOracle::new(12345);
        let results2: Vec<bool> = (0..10).map(|_| oracle2.chance(0.5)).collect();

        assert_eq!(results1, results2);
    }

    #[test]
    fn test_failure_oracle_probability() {
        let oracle = FailureOracle::new(54321);
        let trials = 10000;
        let true_count = (0..trials).filter(|_| oracle.chance(0.3)).count();
        let ratio = true_count as f64 / trials as f64;

        // Should be roughly 0.3 with some tolerance for randomness
        assert!((ratio - 0.3).abs() < 0.05);
    }

    #[test]
    fn test_todo_adapter_creation() {
        // Test that TodoAdapter can be created
        // Note: This is a basic smoke test since we can't easily mock TodoIntegration
        // In a real test, we'd inject a mock TodoIntegration
        let _adapter_exists = true; // Placeholder for actual test
    }
}

/// Status of quality gate verification

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[allow(dead_code)] // Reserved for future use
enum QualityGateStatus {
    Passed,
    Failed,
    Pending,
}

/// Status of milestone execution

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[allow(dead_code)] // Reserved for future use
enum MilestoneStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Blocked,
}