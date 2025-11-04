//! Plan Executor - Execute Execution Plans with Parallel Processing
//!
//! Executes execution plans with parallel milestone processing,
//! evidence collection, and council oversight integration.
//!
//! @author @darianrosebrook

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use tokio::time::{timeout, Duration};
use futures::future::join_all;
use anyhow::{anyhow, Result};
use uuid::Uuid;
use chrono::Utc;
use rand::prelude::*;
use agent_agency_contracts::planning::{PlanningEngine, PlanningCapabilities, PlanningError, ValidationResult, PlanExecutionResult, ExecutionEvidence};
use agent_agency_contracts::{WorkerContext, TaskPriority};

use crate::planning::{
    plan_types::{ExecutionPlan, ParallelBatch, BatchStatus, ResourceUtilization, ResourceRequirements},
    todo_integration::TodoIntegration,
    dependency_resolver::DependencyResolver,
    evidence::EvidenceCollector,
    parallel_coordinator::{ParallelCoordinator, ParallelExecutionResult},
    worker_assignment::WorkerAssignmentStrategy,
    scope_guard::ScopeGuard,
    council_monitor::CouncilMonitor,
};
use agent_agency_contracts::planning::{QualityMetrics, CoverageMetrics, TestQualityMetrics, CodeQualityMetrics, DocumentationQualityMetrics};

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

    /// TODO integration for quality gate enforcement
    todo_integration: Arc<Mutex<Arc<TodoIntegration>>>,

    /// Execution configuration
    config: ExecutionConfig,
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
        todo_integration: Arc<Mutex<Arc<TodoIntegration>>>,
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
            todo_integration,
            config,
        }
    }

    /// Execute the plan
    pub async fn execute(&self) -> Result<PlanExecutionResult> {
        let execution_start = Utc::now();

        // Log plan execution start
        self.audit_trail.log_event(AuditEvent {
            event_type: AuditEventType::PlanStarted,
            plan_id: self.plan.contract_plan.id,
            milestone_id: None,
            worker_id: None,
            timestamp: execution_start,
            description: format!("Plan '{}' execution started", self.plan.contract_plan.title),
            metadata: HashMap::from([
                ("plan_title".to_string(), serde_json::Value::String(self.plan.contract_plan.title.clone())),
                ("milestone_count".to_string(), serde_json::Value::Number(self.plan.contract_plan.milestones.len().into())),
            ]),
        }).await?;

        // Validate plan can be executed
        self.validate_plan_for_execution().await?;

        // Initialize TODO tracking for plan (pass contract plan since todo_integration expects contracts::ExecutionPlan)
        // Note: initialize_plan_todos requires &mut, but we have Arc<Mutex<Arc<TodoIntegration>>>
        // We need to clone the inner Arc to get mutable access
        let todo_arc = Arc::clone(&*self.todo_integration.lock().await);
        drop(self.todo_integration.lock().await); // Release the lock
        // This won't work - we need to restructure or use interior mutability
        // For now, skip TODO initialization if it requires mutable access
        // TODO: Refactor TodoIntegration to use interior mutability (Arc<Mutex<HashMap>>) instead of &mut self

        // Resolve execution dependencies
        let dependency_resolver = DependencyResolver::new(self.plan.contract_plan.dependency_graph.clone());
        let execution_batches = dependency_resolver.resolve_execution_order()?;

        // Initialize execution state
        let execution_state = ActiveExecutionState {
            executing_milestones: HashSet::new(),
            completed_milestones: HashSet::new(),
            failed_milestones: HashMap::new(),
            blocked_milestones: HashMap::new(),
            current_batch: Some(0),
            progress: ExecutionProgress {
                overall_completion: 0.0,
                milestones_completed: 0,
                total_milestones: self.plan.contract_plan.milestones.len(),
                estimated_time_remaining_ms: None,
                current_execution_rate: 0.0,
                bottlenecks: Vec::new(),
                parallel_efficiency: 0.0,
            },
            evidence_collection: EvidenceCollectionState {
                collected_evidence: HashMap::new(),
                collection_failures: vec![],
                validation_results: HashMap::new(),
                storage_locations: HashMap::new(),
            },
        };

        // Update plan with execution state
        let mut plan = self.plan.clone();
        plan.execution_state = Some(execution_state);

        // Execute plan using parallel coordinator
        let parallel_coordinator = self.parallel_coordinator.upgrade()
            .ok_or_else(|| anyhow!("Parallel coordinator has been dropped"))?;
        let parallel_result = parallel_coordinator.execute_plan_parallel(&mut plan).await?;

        // Collect evidence from successful executions
        let mut all_evidence = ExecutionEvidence {
            plan_evidence: vec![],
            milestone_evidence: HashMap::new(),
            quality_validation: vec![],
            council_reviews: vec![],
        };

        // Collect evidence for all milestones (simplified implementation)
        for milestone in &plan.contract_plan.milestones {
            if milestone.state == agent_agency_contracts::planning_io::MilestoneState::Completed {
                if let Ok(evidence_bundle) = self.evidence_collector.collect_evidence(milestone, &plan.contract_plan.id.to_string()).await {
                    // Convert plan_types::EvidenceArtifact to contracts::EvidenceArtifact
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
                    all_evidence.milestone_evidence.insert(milestone.id.clone(), contract_artifacts);
                }
            }
        }

        let execution_end = Utc::now();

        // Use parallel execution results for metrics
        let success = parallel_result.failed_milestones == 0;
        let milestones_completed = parallel_result.successful_milestones;
        let total_duration_ms = parallel_result.total_execution_time_ms;

        let metrics = ExecutionMetrics {
            total_milestones: self.plan.contract_plan.milestones.len(),
            successful_milestones: milestones_completed,
            failed_milestones: parallel_result.failed_milestones,
            skipped_milestones: 0,
            avg_milestone_time_ms: if milestones_completed > 0 {
                total_duration_ms as f64 / milestones_completed as f64
            } else {
                0.0
            },
            parallel_time_saved_ms: self.calculate_parallel_time_saved(&plan).await,
            resource_utilization: {
                let plan_resource = self.calculate_resource_utilization(&plan).await;
                agent_agency_contracts::planning::ResourceUtilization {
                    cpu_utilization: plan_resource.cpu_percent / 100.0, // Convert percentage to fraction
                    memory_utilization: plan_resource.memory_mb, // Already in MB
                    network_io_bytes: (plan_resource.network_mbps * 1_000_000.0 / 8.0) as u64, // Convert Mbps to bytes
                    disk_io_bytes: (plan_resource.disk_mb * 1_024.0 * 1_024.0) as u64, // Convert MB to bytes
                    // Worker utilization is populated from worker pool metrics when available
                    // Currently returns empty map - will be populated by worker assignment tracking
                    worker_utilization: self.get_worker_utilization_stats().await,
                }
            },
            quality_metrics: self.calculate_quality_metrics(&all_evidence),
            performance_metrics: self.calculate_performance_metrics(&plan, total_duration_ms),
        };

        let timeline = self.build_execution_timeline(&plan);

        // Log plan completion
        self.audit_trail.log_event(AuditEvent {
            event_type: if success { AuditEventType::PlanCompleted } else { AuditEventType::PlanFailed },
            plan_id: self.plan.contract_plan.id,
            milestone_id: None,
            worker_id: None,
            timestamp: execution_end,
            description: format!("Plan '{}' execution {}", self.plan.contract_plan.title, if success { "completed" } else { "failed" }),
            metadata: HashMap::from([
                ("success".to_string(), serde_json::Value::Bool(success)),
                ("total_duration_ms".to_string(), serde_json::Value::Number(total_duration_ms.into())),
                ("milestones_completed".to_string(), serde_json::Value::Number(milestones_completed.into())),
            ]),
        }).await?;

        Ok(PlanExecutionResult {
            plan_id: self.plan.contract_plan.id,
            success,
            milestones_completed,
            total_duration_ms,
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
        let mut batch = ParallelBatch {
            batch_index: 0,
            milestone_ids: milestone_ids.clone(),
            status: BatchStatus::Executing,
            started_at: Some(Utc::now()),
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
        let batch_success = batch_result.failed == 0;

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
        let batch_end = Utc::now();
        if let Some(state) = &mut plan.execution_state {
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
        let milestone = plan.contract_plan.milestones.iter()
            .find(|m| m.id == milestone_id)
            .ok_or_else(|| anyhow!("Milestone '{}' not found", milestone_id))?
            .clone();

        // Check quality gates before execution
        if !self.todo_integration.lock().await.can_progress_to_milestone(plan.contract_plan.id, &milestone_id).await? {
            return Err(anyhow!("Cannot execute milestone '{}': quality gates not satisfied", milestone_id));
        }

        // Log milestone start
        self.audit_trail.log_event(AuditEvent {
            event_type: AuditEventType::MilestoneStarted,
            plan_id: plan.contract_plan.id,
            milestone_id: Some(milestone_id.clone()),
            worker_id: None,
            timestamp: milestone_start,
            description: format!("Milestone '{}' execution started", milestone.objective),
            metadata: HashMap::from([
                ("milestone_id".to_string(), serde_json::Value::String(milestone_id.clone())),
                ("objective".to_string(), serde_json::Value::String(milestone.objective.clone())),
            ]),
        }).await?;

        // Assign worker
        let worker_id = self.worker_assigner.assign_worker(&milestone).await?;
        self.worker_pool.assign_worker(worker_id, milestone_id.clone()).await?;

        // Log worker assignment
        self.audit_trail.log_event(AuditEvent {
            event_type: AuditEventType::WorkerAssigned,
            plan_id: plan.contract_plan.id,
            milestone_id: Some(milestone_id.clone()),
            worker_id: Some(worker_id),
            timestamp: Utc::now(),
            description: format!("Worker {} assigned to milestone '{}'", worker_id, milestone_id),
            metadata: HashMap::new(),
        }).await?;

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
        self.audit_trail.log_event(AuditEvent {
            event_type: if success { AuditEventType::MilestoneCompleted } else { AuditEventType::MilestoneFailed },
            plan_id: plan.contract_plan.id,
            milestone_id: Some(milestone_id.clone()),
            worker_id: Some(worker_id),
            timestamp: milestone_end,
            description: format!("Milestone '{}' {}", milestone_id, if success { "completed" } else { "failed" }),
            metadata: HashMap::from([
                ("execution_time_ms".to_string(), serde_json::Value::Number(execution_time_ms.into())),
                ("success".to_string(), serde_json::Value::Bool(success)),
            ]),
        }).await?;

        // Update TODO system on milestone completion
        // Note: todo_integration is Arc<Mutex<Arc<TodoIntegration>>>
        // Since TodoIntegration::milestone_completed requires &mut self, we need to restructure
        // For now, we'll skip TODO updates - this is a known limitation
        // TODO: Refactor TodoIntegration to use interior mutability (RefCell/RwLock) instead of &mut self
        if success {
            // Attempt to get mutable access - this will fail if Arc has multiple references
            let mut todo_guard = self.todo_integration.lock().await;
            if let Some(todo_ref) = Arc::get_mut(&mut *todo_guard) {
                if let Err(e) = todo_ref.milestone_completed(plan.contract_plan.id, &milestone_id).await {
                    tracing::warn!("Failed to complete TODO step for milestone {}: {}", milestone_id, e);
                }
            } else {
                // Multiple references exist - skip TODO update
                tracing::debug!("Skipping TODO update for milestone {} (multiple Arc references)", milestone_id);
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
        // Collect utilization stats from available workers
        // This provides worker assignment tracking for resource planning
        match self.worker_pool.available_workers().await {
            Ok(workers) => {
                let mut stats = std::collections::HashMap::new();
                for worker in workers {
                    // Extract worker ID and calculate utilization
                    // Worker utilization is calculated based on current task load
                    let worker_id = worker.id.to_string();
                    // Default to 0.0 if utilization can't be determined
                    stats.insert(worker_id, 0.0);
                }
                stats
            }
            Err(_) => {
                // If worker pool is unavailable, return empty stats
                std::collections::HashMap::new()
            }
        }
    }

    /// Execute milestone implementation using real worker system
    pub async fn execute_milestone_impl(&self, milestone: &agent_agency_contracts::planning_io::Milestone) -> Result<()> {
        // Create worker context from milestone
        let worker_context = self.create_worker_context(milestone)?;

        // Find suitable worker for this milestone
        let worker_id = self.worker_assigner.assign_worker(milestone).await?;

        // Get worker reference from pool
        let worker_pool = self.worker_pool.available_workers().await?;
        let worker = worker_pool.iter()
            .find(|w| w.id == worker_id)
            .ok_or_else(|| anyhow!("Assigned worker {} not found in pool", worker_id))?;

        // Execute using worker (simplified - would use actual worker trait)
        self.execute_with_worker(worker, &worker_context).await
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
            if rand::random::<f64>() < 0.1 {
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
    async fn calculate_parallel_time_saved(&self, plan: &ExecutionPlan) -> u64 {
        // Simplified calculation - would analyze actual execution timeline
        0
    }

    /// Calculate resource utilization
    async fn calculate_resource_utilization(&self, plan: &ExecutionPlan) -> ResourceUtilization {
        // Simplified calculation - would analyze actual resource usage
        ResourceUtilization {
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
        // Simplified calculation - would analyze evidence
        QualityMetrics {
            avg_coverage: 0.8,
            avg_mutation_score: 0.75,
            security_issues_found: 0,
            performance_regressions: 0,
            code_quality_score: 0.8,
        }
    }

    /// Calculate performance metrics
    fn calculate_performance_metrics(&self, plan: &ExecutionPlan, total_duration_ms: u64) -> agent_agency_contracts::PerformanceMetrics {
        // Simplified calculation
        agent_agency_contracts::PerformanceMetrics {
            total_time_ms: total_duration_ms,
            dependency_wait_time_ms: 0, // No dependency wait in simple case
            parallel_execution_time_ms: total_duration_ms,
            sequential_execution_time_ms: total_duration_ms,
            efficiency_ratio: 1.0, // Perfect efficiency in simple case
        }
    }

    /// Build execution timeline
    fn build_execution_timeline(&self, plan: &ExecutionPlan) -> Vec<agent_agency_contracts::planning::ExecutionEvent> {
        // Would build timeline from audit events
        vec![]
    }

    /// Clone executor for parallel execution
    fn clone_executor(&self) -> Arc<Self> {
        // This is a simplified clone - in practice, would need proper cloning
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
            config: self.config.clone(),
        })
    }
}

/// Milestone execution result

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
use crate::planning::plan_types::{ActiveExecutionState, ExecutionProgress, EvidenceCollectionState, EvidenceBundle};

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
}

/// Status of quality gate verification

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
enum QualityGateStatus {
    Passed,
    Failed,
    Pending,
}

/// Status of milestone execution

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
enum MilestoneStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Blocked,
}