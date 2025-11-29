//! Unified Orchestrator
//!
//! Single entry point coordinating all orchestration components:
//! - Plan generation
//! - Council review
//! - Plan execution via agent-workers
//! - Git worktree management
//! - Council presentation
//! - Refinement loop
//! - Merge and progress tracking
//!
//! @author @darianrosebrook

use anyhow::{anyhow, Result};
use chrono::Utc;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

use agent_agency_contracts::execution_artifacts::ExecutionArtifacts;
use agent_agency_contracts::planning_io::Milestone;
use agent_agency_contracts::types::prelude::*;
use agent_agency_contracts::WorkingSpec;

use crate::council::Council;
use crate::planning::caws_adjudication_cycle::CawsAdjudicationCycle;
use crate::planning::parallel_coordinator::ParallelCoordinator;
use crate::planning::plan_executor::PlanExecutor;
use crate::planning::plan_generator::PlanGenerator;
use crate::planning::plan_types::{
    ExecutionPlan,
    PlanGenerationContext, PlanGenerationStrategy, ResourceInventory,
    TaskDescriptorProvider, WorkingSpecProvider,
};
use crate::planning::refinement_loop::{
    ArtifactValidator, CouncilReviewer, OrchestrationExecutor, ProgressTracker,
    RefinementLoopCoordinator, SpecRefiner,
};
use crate::planning::reflexive_learner::ReflexiveLearner;
use crate::planning::worker_assignment::WorkerAssignmentStrategy;
use crate::planning::worker_lifecycle_manager::WorkerLifecycleManager;
use crate::planning::worktree_manager::WorktreeManager;
use crate::workers::execution_bridge::WorkerExecutionBridge;
use agent_agency_contracts::ExecutionStatus;

#[cfg(feature = "memory")]
use agent_memory::memory_types::TaskContext;
#[cfg(feature = "memory")]
use agent_memory::MemorySystem;

use crate::progress_tracker::turn_level::{
    AgentAction, TaskOutcome, TurnLevelTracker, TurnOutcome, TurnTrajectory,
};

use crate::orchestration::session_manager::SessionManager;

use crate::orchestration::task_state_persistence::{
    ExecutionStateStatus, TaskExecutionState, TaskStatePersistence,
};

use crate::learning::federated_learning::FederatedLearningEngine;

#[cfg(feature = "runtime-optimization")]
use system_federated_ml::{ArbiterPipelineOptimizer, DecisionPipelineConfig};
use agent_model_management::deployment::DeploymentOrchestrator;
use agent_model_management::types::ModelSelection;

/// Unified orchestrator configuration
#[derive(Debug, Clone)]
pub struct UnifiedOrchestratorConfig {
    /// Enable council review
    pub enable_council_review: bool,

    /// Enable refinement loop
    pub enable_refinement: bool,

    /// Enable git worktree isolation
    pub enable_worktree_isolation: bool,

    /// Worktree base directory
    pub worktree_base_path: PathBuf,

    /// Maximum parallel milestones
    pub max_parallel_milestones: usize,
}

impl Default for UnifiedOrchestratorConfig {
    fn default() -> Self {
        Self {
            enable_council_review: true,
            enable_refinement: true,
            enable_worktree_isolation: true,
            worktree_base_path: PathBuf::from("/tmp/agent-agency-worktrees"),
            max_parallel_milestones: 5,
        }
    }
}

/// Execution result from unified orchestrator
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub plan_id: Uuid,
    pub execution_plan: ExecutionPlan,
    pub artifacts: Vec<ExecutionArtifacts>,
    pub final_verdict: Option<agent_agency_contracts::final_verdict::FinalVerdictContract>,
    pub iterations: u32,
    pub quality_scores: Vec<f64>,
}

/// Unified orchestrator - single entry point for all orchestration
pub struct UnifiedOrchestrator {
    config: UnifiedOrchestratorConfig,

    /// Plan generator for creating execution plans
    plan_generator: Arc<PlanGenerator>,

    /// Plan executor for executing plans
    #[allow(dead_code)] // Reserved for future use
    plan_executor: Arc<PlanExecutor>,

    /// Parallel coordinator for parallel milestone execution
    parallel_coordinator: Arc<ParallelCoordinator>,

    /// Council for decision making
    council: Arc<Council>,

    /// Worker execution bridge for delegating to agent-workers
    worker_bridge: Arc<WorkerExecutionBridge>,

    /// Refinement loop coordinator
    refinement_coordinator: Option<Arc<RefinementLoopCoordinator>>,

    /// Worktree manager for git worktree isolation
    worktree_manager: Arc<WorktreeManager>,

    /// CAWS adjudication cycle coordinator
    adjudication_cycle: Option<Arc<CawsAdjudicationCycle>>,

    /// Worker lifecycle manager
    worker_lifecycle_manager: Arc<WorkerLifecycleManager>,

    /// Worker assignment strategy for intelligent worker selection
    #[allow(dead_code)] // Reserved for future use
    worker_assignment_strategy: Option<Arc<WorkerAssignmentStrategy>>,

    /// Reflexive learner for continuous learning from outcomes
    reflexive_learner: Option<Arc<ReflexiveLearner>>,

    /// Memory system for context preservation and retrieval (long-horizon support)
    #[cfg(feature = "memory")]
    memory_system: Option<Arc<MemorySystem>>,
    #[cfg(not(feature = "memory"))]
    #[allow(dead_code)] // Reserved for future use
    memory_system: Option<()>, // Placeholder when memory feature disabled

    /// Active worktrees (worker_id -> worktree_path)
    #[allow(dead_code)] // Reserved for future use
    active_worktrees: Arc<tokio::sync::RwLock<HashMap<Uuid, PathBuf>>>,

    /// Stored context IDs for task resumption (task_id -> context_id)
    #[cfg(feature = "memory")]
    stored_contexts: Arc<tokio::sync::RwLock<HashMap<Uuid, String>>>,

    /// Turn-level progress tracker for long-horizon task support
    turn_level_tracker: Option<Arc<dyn TurnLevelTracker>>,

    /// Session manager for multi-session continuity
    session_manager: Option<Arc<SessionManager>>,

    /// Task state persistence for resumable tasks and crash recovery
    state_persistence: Option<Arc<dyn TaskStatePersistence>>,

    /// Federated learning engine for cross-tenant learning
    federated_learning: Option<Arc<FederatedLearningEngine>>,

    /// Arbiter pipeline optimizer for sub-50ms decision making
    #[cfg(feature = "runtime-optimization")]
    arbiter_optimizer: Option<Arc<ArbiterPipelineOptimizer>>,

    /// Deployment orchestrator for model selection
    deployment_orchestrator: Option<Arc<DeploymentOrchestrator>>,
}

impl UnifiedOrchestrator {
    /// Create a new unified orchestrator
    pub fn new(
        config: UnifiedOrchestratorConfig,
        plan_generator: Arc<PlanGenerator>,
        plan_executor: Arc<PlanExecutor>,
        parallel_coordinator: Arc<ParallelCoordinator>,
        council: Arc<Council>,
        worker_bridge: Arc<WorkerExecutionBridge>,
        refinement_coordinator: Option<Arc<RefinementLoopCoordinator>>,
        worktree_manager: Arc<WorktreeManager>,
        adjudication_cycle: Option<Arc<CawsAdjudicationCycle>>,
        worker_lifecycle_manager: Arc<WorkerLifecycleManager>,
        worker_assignment_strategy: Option<Arc<WorkerAssignmentStrategy>>,
        reflexive_learner: Option<Arc<ReflexiveLearner>>,
        #[cfg(feature = "memory")] memory_system: Option<Arc<MemorySystem>>,
        turn_level_tracker: Option<Arc<dyn TurnLevelTracker>>,
        session_manager: Option<Arc<SessionManager>>,
        state_persistence: Option<Arc<dyn TaskStatePersistence>>,
        federated_learning: Option<Arc<FederatedLearningEngine>>,
        #[cfg(feature = "runtime-optimization")] arbiter_optimizer: Option<
            Arc<ArbiterPipelineOptimizer>,
        >,
        deployment_orchestrator: Option<Arc<DeploymentOrchestrator>>,
    ) -> Self {
        Self {
            config,
            plan_generator,
            plan_executor,
            parallel_coordinator,
            council,
            worker_bridge,
            refinement_coordinator,
            worktree_manager,
            adjudication_cycle,
            worker_lifecycle_manager,
            worker_assignment_strategy,
            reflexive_learner,
            #[cfg(feature = "memory")]
            memory_system,
            #[cfg(not(feature = "memory"))]
            memory_system: None,
            active_worktrees: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            #[cfg(feature = "memory")]
            stored_contexts: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            turn_level_tracker,
            session_manager,
            state_persistence,
            federated_learning,
            #[cfg(feature = "runtime-optimization")]
            arbiter_optimizer,
            deployment_orchestrator,
        }
    }

    /// Preserve iteration context to memory system for long-horizon task support
    #[cfg(feature = "memory")]
    async fn preserve_iteration_context(
        &self,
        task_id: Uuid,
        working_spec: &WorkingSpec,
        execution_plan: &ExecutionPlan,
        artifacts: &[ExecutionArtifacts],
        iteration_number: u32,
    ) -> Result<()> {
        if let Some(ref memory) = self.memory_system {
            // Create TaskContext from current execution state
            let task_context = TaskContext {
                task_id: task_id.to_string(),
                agent_id: "unified_orchestrator".to_string(),
                task_type: "orchestration".to_string(),
                keywords: vec![
                    working_spec.title.clone(),
                    format!("iteration_{}", iteration_number),
                ],
                entities: vec![
                    format!("plan_{}", execution_plan.contract_plan.id),
                    format!("spec_{}", working_spec.id),
                ],
                timestamp: Utc::now(),
                description: format!(
                    "Iteration {} of task {}: {} milestones completed",
                    iteration_number,
                    working_spec.title,
                    artifacts.len()
                ),
            };

            // Store context via context manager
            match memory.context_manager().store_context(&task_context).await {
                Ok(context_id) => {
                    info!(
                        "Preserved iteration context for task {}: {}",
                        task_id, context_id
                    );
                    // Store context_id mapping for later retrieval
                    self.stored_contexts
                        .write()
                        .await
                        .insert(task_id, context_id);
                    Ok(())
                }
                Err(e) => {
                    warn!("Failed to preserve iteration context: {}", e);
                    Err(anyhow::anyhow!("Failed to preserve context: {}", e))
                }
            }
        } else {
            Ok(()) // Memory system not available, skip silently
        }
    }

    /// Retrieve iteration context from memory system for task resumption
    #[cfg(feature = "memory")]
    async fn retrieve_iteration_context(&self, task_id: Uuid) -> Result<Option<TaskContext>> {
        if let Some(ref memory) = self.memory_system {
            // Get stored context_id
            let context_id = {
                let stored = self.stored_contexts.read().await;
                stored.get(&task_id).cloned()
            };

            if let Some(context_id) = context_id {
                match memory.context_manager().retrieve_context(&context_id).await {
                    Ok(context) => {
                        info!(
                            "Retrieved iteration context for task {}: {}",
                            task_id, context_id
                        );
                        Ok(Some(context))
                    }
                    Err(e) => {
                        warn!("Failed to retrieve iteration context: {}", e);
                        Ok(None)
                    }
                }
            } else {
                // Try to retrieve by task_id directly via contextual memory search
                let search_context = TaskContext {
                    task_id: task_id.to_string(),
                    agent_id: "unified_orchestrator".to_string(),
                    task_type: "orchestration".to_string(),
                    keywords: vec![],
                    entities: vec![],
                    timestamp: Utc::now(),
                    description: format!("Searching for context for task {}", task_id),
                };

                match memory
                    .retrieve_contextual_memories(&search_context, 1)
                    .await
                {
                    Ok(memories) => {
                        if let Some(contextual_memory) = memories.first() {
                            // Extract TaskContext from contextual memory
                            // ContextualMemory contains AgentExperience which has task_id, agent_id, context (with description), timestamp
                            let agent_experience = &contextual_memory.memory;

                            // Extract keywords and entities from metadata if available
                            let keywords: Vec<String> = agent_experience
                                .metadata
                                .get("keywords")
                                .and_then(|v| v.as_array())
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                        .collect()
                                })
                                .unwrap_or_else(|| {
                                    // Fallback: extract keywords from context.domain
                                    agent_experience.context.domain.clone()
                                });

                            let entities: Vec<String> = agent_experience
                                .metadata
                                .get("entities")
                                .and_then(|v| v.as_array())
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                        .collect()
                                })
                                .unwrap_or_default();

                            // Create TaskContext from AgentExperience fields
                            let extracted_context = TaskContext {
                                task_id: agent_experience.task_id.clone(),
                                agent_id: agent_experience.agent_id.clone(),
                                task_type: agent_experience.context.task_type.clone(),
                                keywords,
                                entities,
                                timestamp: agent_experience.timestamp,
                                description: agent_experience.context.description.clone(),
                            };

                            tracing::debug!(
                                task_id = %extracted_context.task_id,
                                agent_id = %extracted_context.agent_id,
                                description = %extracted_context.description,
                                "Extracted TaskContext from contextual memory"
                            );

                            Ok(Some(extracted_context))
                        } else {
                            Ok(None)
                        }
                    }
                    Err(e) => {
                        warn!("Failed to search for contextual memories: {}", e);
                        Ok(None)
                    }
                }
            }
        } else {
            Ok(None) // Memory system not available
        }
    }

    /// Execute a plan from a working spec
    ///
    /// This is the single entry point for orchestration:
    /// 1. Generate execution plan from working spec
    /// 2. Council review (CAWS Examination stage)
    /// 3. Execute plan with parallel milestone processing
    /// 4. Council presentation (CAWS Pleading stage)
    /// 5. Refinement loop if needed
    /// 6. Merge and progress tracking (CAWS Publication stage)
    pub async fn execute_plan(&self, working_spec: WorkingSpec) -> Result<ExecutionResult> {
        // Phase 0: Crash recovery - Check for resumable state by working_spec.id
        let mut recovered_state: Option<TaskExecutionState> = None;
        let mut is_resuming = false;
        let plan_id = {
            if let Some(ref persistence) = self.state_persistence {
                // Look for resumable state by checking all resumable tasks
                // and matching by working_spec.id
                let mut found_plan_id = None;
                if let Ok(resumable_task_ids) = persistence.list_resumable_tasks().await {
                    for task_id in resumable_task_ids {
                        if let Ok(Some(state)) = persistence.load_state(task_id).await {
                            // Check if this state matches our working spec
                            if state.working_spec.id == working_spec.id {
                                info!("Found resumable state for working spec {}: {}% complete, phase: {}, iteration {}",
                                    working_spec.id, state.progress_percentage, state.current_phase, state.current_iteration);

                                recovered_state = Some(state.clone());
                                is_resuming = true;
                                found_plan_id = Some(state.task_id);
                                info!("State recovery validated - will resume from phase: {} with plan_id: {}",
                                    recovered_state.as_ref().unwrap().current_phase, state.task_id);
                                break;
                            }
                        }
                    }
                }

                // Use plan_id from recovered state if found, otherwise extract from working_spec.id
                found_plan_id.unwrap_or_else(|| {
                    // Try to extract UUID from working_spec.id (format: TASK-<UUID>)
                    working_spec.id
                        .strip_prefix("TASK-")
                        .and_then(|s| Uuid::parse_str(s).ok())
                        .unwrap_or_else(|| Uuid::new_v4())
                })
            } else {
                // Try to extract UUID from working_spec.id (format: TASK-<UUID>)
                working_spec.id
                    .strip_prefix("TASK-")
                    .and_then(|s| Uuid::parse_str(s).ok())
                    .unwrap_or_else(|| Uuid::new_v4())
            }
        };

        info!(
            "UnifiedOrchestrator: Starting execution for plan {} (resuming: {})",
            plan_id, is_resuming
        );
        
        // Log execution context for debugging
        debug!(
            plan_id = %plan_id,
            working_spec_id = %working_spec.id,
            working_spec_title = %working_spec.title,
            is_resuming = is_resuming,
            "Execution context initialized"
        );

        // Phase 0.1: Initialize execution state (recovered or fresh)
        let mut execution_state = if let Some(state) = recovered_state {
            // Update status to Running if it was paused/crashed
            let mut recovered = state;
            if matches!(
                recovered.status,
                ExecutionStateStatus::Paused | ExecutionStateStatus::Crashed
            ) {
                recovered.status = ExecutionStateStatus::Running;
                recovered.last_updated = Utc::now();
            }
            recovered
        } else {
            TaskExecutionState {
                task_id: plan_id,
                working_spec: working_spec.clone(),
                execution_plan: None,
                artifacts: Vec::new(),
                current_iteration: 0,
                quality_scores: Vec::new(),
                current_phase: "initialization".to_string(),
                progress_percentage: 0.0,
                status: ExecutionStateStatus::Running,
                created_at: Utc::now(),
                last_updated: Utc::now(),
                checkpoint_at: None,
                error: None,
                metadata: std::collections::HashMap::new(),
            }
        };

        // Phase 0: Session management and context retrieval (multi-session continuity)
        let _session_id = if let Some(ref session_mgr) = self.session_manager {
            // Try to get existing session for this task, or create a new one
            let existing_session = session_mgr.get_session_for_task(plan_id).await;

            if let Some(sid) = existing_session {
                info!("Found existing session {} for task {}", sid, plan_id);
                sid
            } else {
                // Create a new session - extract tenant_id from working_spec metadata or use deterministic fallback
                let tenant_id = if let Some(ref metadata) = working_spec.metadata {
                    // Try to extract tenant_id from metadata tags or created_by field
                    // Check if there's a tenant_id in tags (e.g., "tenant:uuid-here")
                    metadata
                        .tags
                        .iter()
                        .find_map(|tag| {
                            if tag.starts_with("tenant:") {
                                tag.strip_prefix("tenant:")
                                    .and_then(|id_str| Uuid::parse_str(id_str).ok())
                            } else {
                                None
                            }
                        })
                        // Or use created_by as tenant identifier (if it's a UUID)
                        .or_else(|| {
                            metadata
                                .created_by
                                .as_ref()
                                .and_then(|created_by| Uuid::parse_str(created_by).ok())
                        })
                        // Or generate deterministic tenant_id from working_spec.id
                        .unwrap_or_else(|| {
                            // Generate deterministic UUID from working_spec.id using a hash
                            use std::collections::hash_map::DefaultHasher;
                            use std::hash::{Hash, Hasher};
                            let mut hasher = DefaultHasher::new();
                            working_spec.id.hash(&mut hasher);
                            let hash = hasher.finish();
                            // Convert hash to UUID (using first 128 bits)
                            Uuid::from_u128(hash as u128)
                        })
                } else {
                    // No metadata - generate deterministic tenant_id from working_spec.id
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::{Hash, Hasher};
                    let mut hasher = DefaultHasher::new();
                    working_spec.id.hash(&mut hasher);
                    let hash = hasher.finish();
                    Uuid::from_u128(hash as u128)
                };

                info!(
                    "Extracted tenant_id {} from working_spec {} (metadata: {:?})",
                    tenant_id,
                    working_spec.id,
                    working_spec.metadata.as_ref().map(|m| &m.created_by)
                );

                match session_mgr
                    .create_session(
                        tenant_id,
                        format!("Session for task {}", plan_id),
                        Some(working_spec.title.clone()),
                    )
                    .await
                {
                    Ok(sid) => {
                        info!("Created new session {} for task {}", sid, plan_id);
                        // Link task to session
                        if let Err(e) = session_mgr.link_task_to_session(plan_id, sid).await {
                            warn!("Failed to link task to session: {}", e);
                        }
                        sid
                    }
                    Err(e) => {
                        warn!("Failed to create session: {}", e);
                        Uuid::nil() // Use nil UUID as fallback
                    }
                }
            }
        } else {
            Uuid::nil() // No session manager available
        };

        // Save initial execution state for status tracking
        if let Some(ref persistence) = self.state_persistence {
            if let Err(e) = persistence.save_state(&execution_state).await {
                warn!("Failed to save initial execution state: {}", e);
            }
        }

        // Phase 0.5: Retrieve cross-session context if session manager is available
        // Store contexts for use in plan generation
        #[cfg(feature = "memory")]
        let cross_session_contexts: Vec<agent_memory::memory_types::TaskContext> = {
            let mut contexts = Vec::new();
            if let Some(ref session_mgr) = self.session_manager {
                if _session_id != Uuid::nil() {
                    if let Ok(retrieved_contexts) = session_mgr
                        .retrieve_cross_session_context(_session_id, 10)
                        .await
                    {
                        if !retrieved_contexts.is_empty() {
                            info!(
                                "Retrieved {} contexts from previous sessions",
                                retrieved_contexts.len()
                            );
                            contexts = retrieved_contexts;

                            // Log insights from cross-session contexts
                            // Each TaskContext represents one task
                            let total_previous_tasks = contexts.len();
                            info!(
                                "Cross-session insights: {} previous tasks across {} sessions",
                                total_previous_tasks,
                                contexts.len()
                            );
                        }
                    }
                }
            }
            contexts
        };

        #[cfg(not(feature = "memory"))]
        let _cross_session_contexts: Vec<()> = Vec::new();

        // Phase 0.6: Try to retrieve previous context for task resumption (long-horizon support)
        #[cfg(feature = "memory")]
        {
            if let Ok(Some(previous_context)) = self.retrieve_iteration_context(plan_id).await {
                info!(
                    "Retrieved previous context for task {}: {}",
                    plan_id, previous_context.description
                );

                // Extract execution state information from previous_context
                // Parse iteration number from keywords (e.g., "iteration_2")
                let extracted_iteration = previous_context.keywords.iter().find_map(|kw| {
                    if kw.starts_with("iteration_") {
                        kw.strip_prefix("iteration_")
                            .and_then(|num_str| num_str.parse::<u32>().ok())
                    } else {
                        None
                    }
                });

                // Extract progress information from description
                // Description format: "Iteration {} of task {}: {} milestones completed"
                let extracted_milestones_completed = previous_context
                    .description
                    .split(": ")
                    .nth(1)
                    .and_then(|part| {
                        part.split_whitespace()
                            .next()
                            .and_then(|num_str| num_str.parse::<usize>().ok())
                    });

                // Extract phase information from description or keywords
                // Try to infer phase from description content
                let extracted_phase = if previous_context.description.contains("planning")
                    || previous_context.description.contains("Planning")
                {
                    Some("planning".to_string())
                } else if previous_context.description.contains("execution")
                    || previous_context.description.contains("Execution")
                {
                    Some("execution".to_string())
                } else if previous_context.description.contains("validation")
                    || previous_context.description.contains("Validation")
                {
                    Some("validation".to_string())
                } else if previous_context.description.contains("completed")
                    || previous_context.description.contains("Completed")
                {
                    Some("completed".to_string())
                } else {
                    None
                };

                // Enhance execution_state with information from previous_context
                if let Some(iteration) = extracted_iteration {
                    if iteration > execution_state.current_iteration {
                        execution_state.current_iteration = iteration;
                        tracing::debug!(
                            task_id = %plan_id,
                            restored_iteration = iteration,
                            "Restored iteration number from previous context"
                        );
                    }
                }

                if let Some(milestones_count) = extracted_milestones_completed {
                    // Update progress percentage based on milestones completed
                    if let Some(ref plan) = execution_state.execution_plan {
                        let total_milestones = plan.milestones.len();
                        if total_milestones > 0 {
                            let new_progress =
                                (milestones_count as f64 / total_milestones as f64) * 100.0;
                            if new_progress > execution_state.progress_percentage {
                                execution_state.progress_percentage = new_progress.min(100.0);
                                tracing::debug!(
                                    task_id = %plan_id,
                                    restored_progress = %execution_state.progress_percentage,
                                    milestones_completed = milestones_count,
                                    total_milestones = total_milestones,
                                    "Restored progress percentage from previous context"
                                );
                            }
                        }
                    }
                }

                if let Some(phase) = extracted_phase {
                    // Update phase if it's more advanced than current phase
                    let phase_priority = |p: &str| -> u8 {
                        match p {
                            "initialization" => 0,
                            "planning" => 1,
                            "execution" => 2,
                            "validation" => 3,
                            "completed" => 4,
                            _ => 1,
                        }
                    };

                    let current_priority = phase_priority(&execution_state.current_phase);
                    let extracted_priority = phase_priority(&phase);

                    if extracted_priority > current_priority {
                        execution_state.current_phase = phase.clone();
                        tracing::debug!(
                            task_id = %plan_id,
                            restored_phase = %execution_state.current_phase,
                            "Restored execution phase from previous context"
                        );
                    }
                }

                // Store context metadata in execution_state.metadata for future reference
                execution_state.metadata.insert(
                    "previous_context_timestamp".to_string(),
                    serde_json::json!(previous_context.timestamp.to_rfc3339()),
                );
                execution_state.metadata.insert(
                    "previous_context_description".to_string(),
                    serde_json::json!(previous_context.description),
                );

                if !previous_context.keywords.is_empty() {
                    execution_state.metadata.insert(
                        "previous_context_keywords".to_string(),
                        serde_json::json!(previous_context.keywords),
                    );
                }

                if !previous_context.entities.is_empty() {
                    execution_state.metadata.insert(
                        "previous_context_entities".to_string(),
                        serde_json::json!(previous_context.entities),
                    );
                }

                tracing::info!(
                    task_id = %plan_id,
                    iteration = execution_state.current_iteration,
                    phase = %execution_state.current_phase,
                    progress = %execution_state.progress_percentage,
                    "Enhanced execution state with information from previous context"
                );
            }
        }

        // Phase 1: Generate execution plan (skip if resuming from later phase)
        let execution_plan = if is_resuming && execution_state.progress_percentage >= 10.0 {
            // We have a recovered execution plan - wrap it back into plan_types::ExecutionPlan
            if let Some(ref contract_plan) = execution_state.execution_plan {
                info!(
                    "Resuming: Using recovered execution plan with {} milestones",
                    contract_plan.milestones.len()
                );
                ExecutionPlan {
                    contract_plan: contract_plan.clone(),
                    orchestration_meta: Default::default(),
                    execution_context: Default::default(),
                    execution_state: None,
                }
            } else {
                return Err(anyhow::anyhow!(
                    "Cannot resume: execution plan missing from recovered state (phase: {}, progress: {}%)",
                    execution_state.current_phase,
                    execution_state.progress_percentage
                ));
            }
        } else {
            // Generate new execution plan
            let plan_gen_start = std::time::Instant::now();
            info!(
                plan_id = %plan_id,
                "Phase 1: Generating execution plan (working spec: {})",
                working_spec.id
            );
            // Create PlanGenerationContext from WorkingSpec
            struct SimpleWorkingSpecProvider {
                spec: WorkingSpec,
            }

            #[async_trait::async_trait]
            impl WorkingSpecProvider for SimpleWorkingSpecProvider {
                async fn get_working_spec(&self) -> Result<WorkingSpec> {
                    Ok(self.spec.clone())
                }
            }

            struct SimpleTaskDescriptorProvider {
                descriptor: TaskDescriptor,
            }

            #[async_trait::async_trait]
            impl TaskDescriptorProvider for SimpleTaskDescriptorProvider {
                async fn get_task_descriptor(&self) -> Result<TaskDescriptor> {
                    Ok(self.descriptor.clone())
                }
            }

            let context = PlanGenerationContext {
                working_spec_provider: Box::new(SimpleWorkingSpecProvider {
                    spec: working_spec.clone(),
                }),
                task_descriptor: Box::new(SimpleTaskDescriptorProvider {
                    descriptor: TaskDescriptor {
                        task_id: plan_id,
                        description: working_spec.description.clone(),
                        execution_mode: ExecutionMode::Auto,
                        blast_radius: agent_agency_contracts::types::planning::BlastRadius {
                            modules: vec![],
                            data_migration: false,
                            external_deps: vec![],
                        },
                        priority: TaskPriority::Medium,
                        change_budget: agent_agency_contracts::planning_io::ChangeBudget {
                            max_files: working_spec
                                .constraints
                                .budget_limits
                                .as_ref()
                                .and_then(|b| b.max_files)
                                .map(|f| f as usize)
                                .unwrap_or(50),
                            max_loc: working_spec
                                .constraints
                                .budget_limits
                                .as_ref()
                                .and_then(|b| b.max_loc)
                                .map(|l| l as usize)
                                .unwrap_or(1000),
                            max_migrations: 0,
                            allow_breaking_changes: false,
                            allow_new_dependencies: false,
                            enforcement_mode:
                                agent_agency_contracts::planning_io::BudgetEnforcement::Strict,
                        },
                        risk_tier: Some(match working_spec.risk_tier {
                            1 => agent_agency_contracts::types::planning::RiskTier::Tier1,
                            2 => agent_agency_contracts::types::planning::RiskTier::Tier2,
                            3 => agent_agency_contracts::types::planning::RiskTier::Tier3,
                            _ => agent_agency_contracts::types::planning::RiskTier::Tier2,
                        }),
                        scope_in: agent_agency_contracts::task_request::ScopeRestrictions {
                            allowed_paths: working_spec
                                .constraints
                                .scope_restrictions
                                .as_ref()
                                .map(|s| s.allowed_paths.clone())
                                .unwrap_or_default(),
                            blocked_paths: vec![],
                        },
                        scope_out: None,
                        acceptance: Some(
                            working_spec
                                .acceptance_criteria
                                .iter()
                                .map(|c| format!("{}: {}", c.given, c.then))
                                .collect::<Vec<_>>()
                                .join("\n"),
                        ),
                    },
                }),
                resource_inventory: ResourceInventory::default(),
                constraints: Default::default(),
                historical_data: {
                    #[cfg(feature = "memory")]
                    {
                        if !cross_session_contexts.is_empty() {
                            // Convert TaskContext to historical planning data
                            // TaskContext has limited fields compared to SessionContext, so we extract what's available
                            let similar_plans: Vec<crate::planning::plan_types::HistoricalPlan> = cross_session_contexts
                                .iter()
                                .map(|ctx| {
                                    // Use task_id as plan identifier (convert string to Uuid if possible)
                                    let _plan_id = uuid::Uuid::parse_str(&ctx.task_id)
                                        .unwrap_or_else(|_| uuid::Uuid::new_v4());

                                    crate::planning::plan_types::HistoricalPlan {
                                        plan_id: _plan_id,
                                        complexity_score: 0.5, // Default - TaskContext doesn't have complexity info
                                        execution_time_ms: 0, // Default - TaskContext doesn't have execution time
                                        successful: true, // Default - assume success if context exists
                                        strategy: "AIAssisted".to_string(), // Default strategy
                                        lessons: ctx.keywords.clone(), // Use keywords as lessons
                                    }
                                })
                                .collect();

                            // Extract execution time patterns from contexts
                            // TaskContext doesn't have execution time metadata, so use defaults
                            let avg_execution_times: HashMap<String, u64> = cross_session_contexts
                                .iter()
                                .map(|ctx| (ctx.task_type.clone(), 0)) // Default to 0
                                .collect();

                            // Extract success rates from contexts
                            // TaskContext doesn't have status, so assume all are successful
                            let success_rates: HashMap<String, f64> = cross_session_contexts
                                .iter()
                                .map(|ctx| (ctx.task_type.clone(), 1.0)) // Default to 1.0 (success)
                                .collect();

                            // Extract failure patterns from task descriptions
                            // Since TaskContext doesn't have status, we can't determine failures
                            // Return empty failure patterns
                            let failure_patterns: Vec<crate::planning::plan_types::FailurePattern> = Vec::new();

                            Some(crate::planning::plan_types::HistoricalPlanningData {
                                similar_plans,
                                avg_execution_times,
                                success_rates,
                                failure_patterns,
                            })
                        } else {
                            None
                        }
                    }
                    #[cfg(not(feature = "memory"))]
                    {
                        None
                    }
                },
                planning_constraints: Default::default(),
                execution_mode: agent_agency_contracts::types::planning::ExecutionMode::Auto,
                planning_strategy: PlanGenerationStrategy::AIAssisted,
            };
            info!(
                plan_id = %plan_id,
                "Waiting on plan generator (may call LLM service)..."
            );
            
            // Generate execution plan with comprehensive error handling
            let execution_plan = match self.plan_generator.generate(&context).await {
                Ok(plan) => {
                    let plan_gen_duration = plan_gen_start.elapsed();
                    info!(
                        plan_id = %plan_id,
                        milestone_count = plan.contract_plan.milestones.len(),
                        duration_ms = plan_gen_duration.as_millis(),
                        "Phase 1 complete: Generated execution plan with {} milestones ({}ms)",
                        plan.contract_plan.milestones.len(),
                        plan_gen_duration.as_millis()
                    );
                    plan
                }
                Err(e) => {
                    let plan_gen_duration = plan_gen_start.elapsed();
                    error!(
                        plan_id = %plan_id,
                        error = %e,
                        duration_ms = plan_gen_duration.as_millis(),
                        "Phase 1 FAILED: Plan generation failed after {}ms: {}",
                        plan_gen_duration.as_millis(),
                        e
                    );
                    
                    // Update execution state with error
                    execution_state.current_phase = "plan_generation_failed".to_string();
                    execution_state.error = Some(format!("Plan generation failed: {}", e));
                    execution_state.status = ExecutionStateStatus::Failed;
                    
                    // Save error state
                    if let Some(ref persistence) = self.state_persistence {
                        if let Err(save_err) = persistence.save_state(&execution_state).await {
                            warn!("Failed to save error state: {}", save_err);
                        }
                    }
                    
                    return Err(anyhow::anyhow!(
                        "Plan generation failed for task {}: {}",
                        plan_id,
                        e
                    ));
                }
            };

            // Update execution state with plan (only if not resuming)
            if !is_resuming {
                execution_state.execution_plan = Some(execution_plan.contract_plan.clone());
                execution_state.current_phase = "plan_generated".to_string();
                execution_state.progress_percentage = 10.0;

                // Create checkpoint after plan generation
                if let Some(ref persistence) = self.state_persistence {
                    if let Err(e) = persistence
                        .create_checkpoint(plan_id, &execution_state)
                        .await
                    {
                        warn!("Failed to create checkpoint after plan generation: {}", e);
                    }
                }
            }

            execution_plan
        };

        // Phase 2: Council plan review (CAWS Examination stage)
        if self.config.enable_council_review {
            let council_start = std::time::Instant::now();
            info!(
                plan_id = %plan_id,
                "Phase 2: Starting council plan review (CAWS Examination)"
            );

            // Create review context for council
            use crate::decision_making::FinalDecision;
            use crate::judge_backup::types::ReviewContext;

            let _review_context = ReviewContext {
                session_id: format!("examination_{}", plan_id),
                working_spec: serde_json::to_string(&working_spec).map_err(|e| {
                    anyhow::anyhow!("Failed to serialize working spec for council review: {}", e)
                })?,
                risk_tier: working_spec.risk_tier as u8,
                previous_reviews: vec![],
                constraints: std::collections::HashMap::new(),
            };

            // Conduct council review of the execution plan
            let council_session = self
                .council
                .conduct_review(working_spec.clone(), _review_context)
                .await
                .map_err(|e| {
                    anyhow::anyhow!("Council plan review (CAWS Examination) failed: {:?}", e)
                })?;

            // Check council decision
            match council_session.final_decision.as_ref() {
                Some(FinalDecision::Proceed { .. }) => {
                    info!("Council approved plan for execution (CAWS Examination passed)");
                }
                Some(FinalDecision::Reject { reason, .. }) => {
                    let rejection_reason =
                        format!("Council rejected plan during CAWS Examination: {}", reason);
                    error!("{}", rejection_reason);
                    return Err(anyhow::anyhow!("{}", rejection_reason));
                }
                Some(FinalDecision::Refine {
                    refinement_directive,
                    ..
                }) => {
                    // Council requests refinement - this will be handled in Phase 5 refinement loop
                    info!(
                        "Council requested plan refinement during CAWS Examination: {:?}",
                        refinement_directive
                    );
                    // Continue to execution - refinement happens in Phase 5 after artifacts are produced
                }
                Some(FinalDecision::Escalate { reason, .. }) => {
                    warn!(
                        "Council escalated decision during CAWS Examination: {}",
                        reason
                    );
                    // Escalation means human review needed - log and proceed with caution
                }
                None => {
                    warn!(
                        "Council review completed but no final decision - proceeding with caution"
                    );
                    // If no decision, log warning but proceed (council may have timed out or failed)
                }
            }

            let council_duration = council_start.elapsed();
            info!(
                plan_id = %plan_id,
                duration_ms = council_duration.as_millis(),
                "Phase 2 complete: Council review finished ({}ms)",
                council_duration.as_millis()
            );

            // Update execution state with council review result
            execution_state.current_phase = "council_examination_complete".to_string();
            execution_state.progress_percentage = 20.0;

            // Store council session info in metadata
            execution_state.metadata.insert(
                "council_examination_session_id".to_string(),
                serde_json::json!(council_session.session_id),
            );
            if let Some(ref decision) = council_session.final_decision {
                execution_state.metadata.insert(
                    "council_examination_decision".to_string(),
                    serde_json::json!(format!("{:?}", decision)),
                );
            }
        }

        // Phase 3: Execute plan with parallel milestone processing (skip if resuming from >= 50%)
        let artifacts = if is_resuming && execution_state.progress_percentage >= 50.0 {
            // We have recovered artifacts - use them
            if !execution_state.artifacts.is_empty() {
                info!(
                    "Resuming: Using {} recovered artifacts from previous execution",
                    execution_state.artifacts.len()
                );
                execution_state.artifacts.clone()
            } else {
                return Err(anyhow::anyhow!(
                    "Cannot resume: artifacts missing from recovered state (phase: {}, progress: {}%)",
                    execution_state.current_phase,
                    execution_state.progress_percentage
                ));
            }
        } else {
            // Execute milestones
            let exec_start = std::time::Instant::now();
            info!(
                plan_id = %plan_id,
                milestone_count = execution_plan.contract_plan.milestones.len(),
                "Phase 3: Starting milestone execution ({} milestones)",
                execution_plan.contract_plan.milestones.len()
            );
            let executed_artifacts = self.execute_plan_milestones(&execution_plan).await?;
            let exec_duration = exec_start.elapsed();
            info!(
                plan_id = %plan_id,
                artifact_count = executed_artifacts.len(),
                duration_ms = exec_duration.as_millis(),
                "Phase 3 complete: Executed {} milestones, produced {} artifacts ({}ms)",
                execution_plan.contract_plan.milestones.len(),
                executed_artifacts.len(),
                exec_duration.as_millis()
            );

            // Update execution state with artifacts
            execution_state.artifacts = executed_artifacts.clone();
            execution_state.current_phase = "milestones_executed".to_string();
            execution_state.progress_percentage = 50.0;

            // Create checkpoint after milestone execution
            if let Some(ref persistence) = self.state_persistence {
                if let Err(e) = persistence
                    .create_checkpoint(plan_id, &execution_state)
                    .await
                {
                    warn!(
                        "Failed to create checkpoint after milestone execution: {}",
                        e
                    );
                }
            }

            executed_artifacts
        };

        // Phase 3.5: Process learning outcomes from execution
        if let Some(ref learner) = self.reflexive_learner {
            info!("Phase 3.5: Processing learning outcomes");
            for (artifact, milestone) in artifacts
                .iter()
                .zip(execution_plan.contract_plan.milestones.iter())
            {
                // Extract worker_id from artifact
                if let Some(worker_id_str) = &artifact.provenance.worker_id {
                    if let Ok(worker_id) = Uuid::parse_str(worker_id_str) {
                        if let Err(e) = learner
                            .process_outcome(artifact, milestone, worker_id)
                            .await
                        {
                            warn!("Failed to process learning outcome: {}", e);
                        }
                    }
                }
            }

            // Submit contribution to federated learning if enabled
            if let Some(ref federated) = self.federated_learning {
                if let Some(ref session_mgr) = self.session_manager {
                    if let Some(session_id) = session_mgr.get_session_for_task(plan_id).await {
                        // Extract trajectories for federated learning
                        let trajectories = if let Some(ref turn_tracker) = self.turn_level_tracker {
                            if let Ok(turns) = turn_tracker.get_turns(plan_id).await {
                                // Convert turns to TurnTrajectory objects
                                if !turns.is_empty() {
                                    // Calculate final outcome from turns
                                    let final_quality =
                                        turns.iter().map(|t| t.outcome.quality_score).sum::<f64>()
                                            / turns.len() as f64;

                                    let final_success = turns.iter().all(|t| t.outcome.success);

                                    // Collect all artifacts from turns
                                    let final_artifacts: Vec<ExecutionArtifacts> = turns
                                        .iter()
                                        .filter_map(|t| t.outcome.artifacts.clone())
                                        .collect();

                                    // Get completion timestamp from last turn
                                    let completed_at = turns
                                        .last()
                                        .map(|t| t.completed_at)
                                        .unwrap_or_else(Utc::now);

                                    let final_outcome = TaskOutcome {
                                        success: final_success,
                                        quality_score: final_quality,
                                        artifacts: final_artifacts,
                                        completed_at,
                                    };

                                    // Create trajectory
                                    let trajectory = TurnTrajectory {
                                        task_id: plan_id,
                                        turns: turns.clone(),
                                        final_outcome,
                                        total_turns: turns.len() as u32,
                                        trajectory_quality: final_quality,
                                    };

                                    vec![trajectory]
                                } else {
                                    vec![]
                                }
                            } else {
                                vec![]
                            }
                        } else {
                            vec![]
                        };

                        // Extract contribution from learner
                        if let Ok(contribution) = federated
                            .extract_contribution(
                                session_id, // Use session_id as tenant_id proxy
                                learner,
                                &trajectories,
                            )
                            .await
                        {
                            if let Err(e) = federated
                                .submit_contribution(session_id, contribution)
                                .await
                            {
                                warn!("Failed to submit federated learning contribution: {}", e);
                            } else {
                                info!("Submitted learning contribution to federated learning");

                                // Check if aggregation round completed and apply aggregated model
                                if let Ok(aggregated_model) =
                                    federated.aggregate_contributions().await
                                {
                                    info!(
                                        "Federated aggregation round {} completed with {} tenants",
                                        aggregated_model.round_id, aggregated_model.tenant_count
                                    );

                                    // Apply aggregated model to learner
                                    if let Err(e) = federated
                                        .apply_to_learner(learner.clone(), &aggregated_model)
                                        .await
                                    {
                                        warn!("Failed to apply aggregated model to learner: {}", e);
                                    } else {
                                        info!("Applied aggregated federated learning model to reflexive learner");
                                    }
                                } else {
                                    // Not enough contributions yet - check for latest model to apply
                                    if let Some(latest_model) = federated.get_latest_model().await {
                                        info!("Applying latest aggregated model (round {}) to learner", latest_model.round_id);
                                        if let Err(e) = federated
                                            .apply_to_learner(learner.clone(), &latest_model)
                                            .await
                                        {
                                            warn!("Failed to apply latest aggregated model: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Phase 3.6: Track turn-level progress (long-horizon support)
        if let Some(ref turn_tracker) = self.turn_level_tracker {
            info!("Phase 3.6: Tracking turn-level progress");
            let turn_number = 1u32; // First turn (execution phase)

            // Create agent action from execution
            let action = AgentAction {
                action_type: "plan_execution".to_string(),
                description: format!("Executed plan with {} milestones", artifacts.len()),
                worker_id: artifacts
                    .first()
                    .and_then(|a| a.provenance.worker_id.as_ref())
                    .and_then(|id| Uuid::parse_str(id).ok()),
                milestone_id: None,
                timestamp: Utc::now(),
                metadata: std::collections::HashMap::new(),
            };

            // Calculate overall quality score from artifacts using comprehensive method
            let quality_score = UnifiedOrchestrationExecutor::calculate_quality_score_static(&artifacts);

            let outcome = TurnOutcome {
                success: artifacts.iter().all(|a| {
                    // Success if all test suites have no failures
                    a.tests.unit_tests.failed == 0
                        && a.tests.integration_tests.failed == 0
                        && a.tests.e2e_tests.failed == 0
                }),
                quality_score,
                artifacts: artifacts.first().cloned(),
                error: None,
                execution_time_ms: artifacts.first().and_then(|a| {
                    a.provenance.completed_at.and_then(|completed| {
                        Some((completed - a.provenance.started_at).num_milliseconds() as u64)
                    })
                }),
                metadata: std::collections::HashMap::new(),
            };

            if let Err(e) = turn_tracker
                .track_turn_progress(plan_id, turn_number, action, outcome)
                .await
            {
                warn!("Failed to track turn-level progress: {}", e);
            }
        }

        // Phase 4: Council presentation (CAWS Pleading stage)
        let mut needs_refinement = false;
        if self.config.enable_council_review {
            info!("Phase 4: Council presentation (CAWS Pleading)");
            if let Some(ref adjudication) = self.adjudication_cycle {
                // Execute full CAWS adjudication cycle
                let adjudication_result = adjudication
                    .execute_cycle(&artifacts, &working_spec, &execution_plan.contract_plan)
                    .await?;

                needs_refinement = adjudication_result.needs_refinement;

                if !adjudication_result.approved && !needs_refinement {
                    return Err(anyhow::anyhow!(
                        "Work rejected by council: {}",
                        adjudication_result.refinement_reason.unwrap_or_default()
                    ));
                }
            }
        }

        // Phase 5: Refinement loop if needed (skip if resuming from >= 80%)
        let (final_verdict, iterations, quality_scores) = if is_resuming
            && execution_state.progress_percentage >= 80.0
        {
            // We have recovered refinement results - use them
            info!(
                "Resuming: Using recovered refinement results (iterations: {}, quality scores: {})",
                execution_state.current_iteration,
                execution_state.quality_scores.len()
            );

            // Extract final verdict from metadata if available
            let verdict = execution_state
                .metadata
                .get("final_verdict")
                .and_then(|v| {
                    serde_json::from_value::<
                        Option<agent_agency_contracts::final_verdict::FinalVerdictContract>,
                    >(v.clone())
                    .ok()
                })
                .flatten();

            (
                verdict,
                execution_state.current_iteration,
                execution_state.quality_scores.clone(),
            )
        } else if self.config.enable_refinement && needs_refinement {
            info!("Phase 5: Refinement loop");

            if let Some(ref refinement_coordinator) = self.refinement_coordinator {
                // Create TaskDescriptor from working spec
                let task_descriptor = TaskDescriptor {
                    task_id: plan_id,
                    description: working_spec.description.clone(),
                    change_budget: working_spec.change_budget.clone(),
                    priority: match working_spec.risk_tier {
                        1 => agent_agency_contracts::types::planning::TaskPriority::Critical,
                        2 => agent_agency_contracts::types::planning::TaskPriority::Normal,
                        3 => agent_agency_contracts::types::planning::TaskPriority::Low,
                        _ => agent_agency_contracts::types::planning::TaskPriority::Normal,
                    },
                    execution_mode: ExecutionMode::Auto,
                    risk_tier: match working_spec.risk_tier {
                        1 => Some(agent_agency_contracts::types::planning::RiskTier::Tier1),
                        2 => Some(agent_agency_contracts::types::planning::RiskTier::Tier2),
                        3 => Some(agent_agency_contracts::types::planning::RiskTier::Tier3),
                        _ => Some(agent_agency_contracts::types::planning::RiskTier::Tier2),
                    },
                    blast_radius: agent_agency_contracts::types::planning::BlastRadius {
                        modules: vec![],
                        data_migration: false,
                        external_deps: vec![],
                    },
                    scope_in: agent_agency_contracts::task_request::ScopeRestrictions {
                        allowed_paths: working_spec.allowed_paths(),
                        blocked_paths: working_spec.blocked_paths(),
                    },
                    scope_out: None,
                    acceptance: Some(
                        working_spec
                            .acceptance_criteria
                            .iter()
                            .map(|c| format!("{}: {}", c.given, c.then))
                            .collect::<Vec<_>>()
                            .join("\n"),
                    ),
                };

                // Create trait implementations
                let executor: Arc<dyn OrchestrationExecutor> =
                    Arc::new(UnifiedOrchestrationExecutor {
                        plan_generator: self.plan_generator.clone(),
                        worker_bridge: self.worker_bridge.clone(),
                        worktree_manager: self.worktree_manager.clone(),
                        worker_lifecycle_manager: self.worker_lifecycle_manager.clone(),
                        deployment_orchestrator: self.deployment_orchestrator.clone(),
                    });

                let validator: Arc<dyn ArtifactValidator> = Arc::new(UnifiedArtifactValidator);

                let council_reviewer: Option<Arc<dyn CouncilReviewer>> =
                    Some(Arc::new(UnifiedCouncilReviewer {
                        council: self.council.clone(),
                    }));

                let spec_refiner: Option<Arc<dyn SpecRefiner>> = Some(Arc::new(UnifiedSpecRefiner::new()));

                // Use RealTimeProgressTracker for actual progress tracking
                let base_progress_tracker: Arc<dyn crate::progress_tracker::ProgressTracker> =
                    Arc::new(crate::progress_tracker::RealTimeProgressTracker::new(None));
                let progress_tracker: Arc<dyn ProgressTracker> = Arc::new(UnifiedProgressTracker {
                    base_tracker: base_progress_tracker,
                });

                // Execute refinement loop
                let refinement_result = refinement_coordinator
                    .execute_refinement_loop(
                        plan_id,
                        working_spec.clone(),
                        &task_descriptor,
                        executor,
                        validator,
                        council_reviewer,
                        spec_refiner,
                        progress_tracker,
                        None, // State persistence - optional
                    )
                    .await?;

                // Preserve context after refinement loop completes (long-horizon support)
                #[cfg(feature = "memory")]
                {
                    if let Err(e) = self
                        .preserve_iteration_context(
                            plan_id,
                            &working_spec,
                            &execution_plan,
                            &artifacts,
                            refinement_result.iterations,
                        )
                        .await
                    {
                        warn!("Failed to preserve context after refinement: {}", e);
                    }
                }

                // Update execution state after refinement
                execution_state.current_iteration = refinement_result.iterations;
                execution_state.quality_scores = refinement_result.quality_scores.clone();
                execution_state.current_phase = "refinement_completed".to_string();
                execution_state.progress_percentage = 80.0;

                // Store final verdict in metadata for resumption
                execution_state.metadata.insert(
                    "final_verdict".to_string(),
                    serde_json::to_value(&refinement_result.final_verdict)
                        .unwrap_or(serde_json::Value::Null),
                );

                // Create checkpoint after refinement
                if let Some(ref persistence) = self.state_persistence {
                    if let Err(e) = persistence
                        .create_checkpoint(plan_id, &execution_state)
                        .await
                    {
                        warn!("Failed to create checkpoint after refinement: {}", e);
                    }
                }

                (
                    Some(refinement_result.final_verdict.clone()),
                    refinement_result.iterations,
                    refinement_result.quality_scores,
                )
            } else {
                (None, 0, Vec::new())
            }
        } else {
            (None, 0, Vec::new())
        };

        // Phase 6: Merge and progress tracking (CAWS Publication stage)
        info!("Phase 6: Merge and progress tracking (CAWS Publication)");

        // Merge worktrees if worktree isolation is enabled and verdict is approved
        if self.config.enable_worktree_isolation {
            let approved = final_verdict
                .as_ref()
                .map(|v| {
                    matches!(
                        v.decision,
                        agent_agency_contracts::final_verdict::FinalDecision::Accept
                    )
                })
                .unwrap_or(false);

            if approved {
                info!("Merging approved worktrees back to main branch");

                // Get all active worktrees from WorktreeManager
                let active_worktrees = self.worktree_manager.list_worktrees().await;

                // Merge each worktree
                for worktree_info in &active_worktrees {
                    info!(
                        "Merging worktree {} (milestone: {})",
                        worktree_info.worktree_id, worktree_info.milestone_id
                    );

                    match self
                        .worktree_manager
                        .merge_worktree(worktree_info.worktree_id)
                        .await
                    {
                        Ok(merge_result) => {
                            if !merge_result.conflicts.is_empty() {
                                warn!(
                                    "Merge conflicts detected in worktree {}: {:?}",
                                    worktree_info.worktree_id, merge_result.conflicts
                                );

                                // Try to resolve conflicts automatically (accept theirs for approved work)
                                if let Err(e) = self.worktree_manager.resolve_conflicts(
                                    worktree_info.worktree_id,
                                    crate::planning::worktree_manager::ConflictResolutionStrategy::AcceptTheirs,
                                ).await {
                                    error!("Failed to resolve conflicts for worktree {}: {}",
                                        worktree_info.worktree_id, e);
                                    // Continue with other worktrees even if one fails
                                }
                            } else {
                                info!(
                                    "Successfully merged worktree {} ({} files changed)",
                                    worktree_info.worktree_id, merge_result.files_changed
                                );
                            }
                        }
                        Err(e) => {
                            error!(
                                "Failed to merge worktree {}: {}",
                                worktree_info.worktree_id, e
                            );
                            // Continue with other worktrees even if one fails
                        }
                    }
                }
            } else {
                info!("Verdict not approved - skipping worktree merge");
            }
        }

        // Cleanup worktrees
        if self.config.enable_worktree_isolation {
            self.worktree_manager.cleanup_all().await?;
        }

        // Preserve final context after execution completes (long-horizon support)
        #[cfg(feature = "memory")]
        {
            if let Err(e) = self
                .preserve_iteration_context(
                    plan_id,
                    &working_spec,
                    &execution_plan,
                    &artifacts,
                    iterations,
                )
                .await
            {
                warn!("Failed to preserve final context: {}", e);
            }
        }

        // Assign credit to turns based on final outcome (long-horizon support)
        if let Some(ref turn_tracker) = self.turn_level_tracker {
            info!("Assigning credit to turns based on final outcome");

            // Get all turns for this task
            if let Ok(turns) = turn_tracker.get_turns(plan_id).await {
                if !turns.is_empty() {
                    // Calculate final quality score
                    let final_quality = quality_scores.last().copied().unwrap_or(0.0);

                    // Create final task outcome
                    let final_outcome = TaskOutcome {
                        success: final_verdict
                            .as_ref()
                            .map(|v| {
                                matches!(
                                    v.decision,
                                    agent_agency_contracts::final_verdict::FinalDecision::Accept
                                )
                            })
                            .unwrap_or(false),
                        quality_score: final_quality,
                        artifacts: artifacts.clone(),
                        completed_at: Utc::now(),
                    };

                    // Assign credit
                    if let Err(e) = turn_tracker
                        .assign_credit(plan_id, turns, final_outcome)
                        .await
                    {
                        warn!("Failed to assign credit to turns: {}", e);
                    }
                }
            }
        }

        // Process learning outcomes via ReflexiveLearner (if available)
        if let Some(ref reflexive_learner) = self.reflexive_learner {
            info!("Processing learning outcomes via ReflexiveLearner");

            // Process each completed milestone with its artifact
            // Match artifacts to milestones by index (artifacts are returned in milestone order)
            for (milestone_index, milestone) in
                execution_plan.contract_plan.milestones.iter().enumerate()
            {
                // Only process completed milestones
                if !matches!(
                    milestone.state,
                    agent_agency_contracts::planning_io::MilestoneState::Completed
                ) {
                    continue;
                }

                // Get corresponding artifact (if available)
                let artifact = artifacts.get(milestone_index).or_else(|| {
                    // Try to find artifact by worker_id match
                    milestone.assigned_workers.first().and_then(|&worker_id| {
                        artifacts.iter().find(|a| {
                            a.provenance
                                .worker_id
                                .as_ref()
                                .and_then(|wid| Uuid::parse_str(wid).ok())
                                .map(|wid| wid == worker_id)
                                .unwrap_or(false)
                        })
                    })
                });

                if let Some(artifact) = artifact {
                    // Extract worker_id from milestone or artifact
                    let worker_id = milestone
                        .assigned_workers
                        .first()
                        .copied()
                        .or_else(|| {
                            artifact
                                .provenance
                                .worker_id
                                .as_ref()
                                .and_then(|wid| Uuid::parse_str(wid).ok())
                        })
                        .unwrap_or_else(Uuid::new_v4);

                    // Process outcome
                    match reflexive_learner
                        .process_outcome(artifact, milestone, worker_id)
                        .await
                    {
                        Ok(adjustments) => {
                            if !adjustments.is_empty() {
                                info!("ReflexiveLearner generated {} routing adjustments for milestone {}",
                                    adjustments.len(), milestone.id);
                                for adjustment in &adjustments {
                                    debug!("Routing adjustment: worker_id={}, performance_adjustment={:.2}, reason={}",
                                        adjustment.worker_id, adjustment.performance_adjustment, adjustment.reason);
                                }
                            }
                        }
                        Err(e) => {
                            warn!(
                                "Failed to process learning outcome for milestone {}: {}",
                                milestone.id, e
                            );
                        }
                    }
                } else {
                    debug!(
                        "No artifact found for milestone {} - skipping ReflexiveLearner processing",
                        milestone.id
                    );
                }
            }
        }

        // Update session context after execution completes (multi-session continuity)
        if let Some(ref session_mgr) = self.session_manager {
            // Get session_id from the beginning of the function
            // Note: session_id was captured in Phase 0, but we need to retrieve it again
            // or store it in a variable accessible here
            if let Some(sid) = session_mgr.get_session_for_task(plan_id).await {
                // Update session metadata with execution results
                let mut metadata = std::collections::HashMap::new();
                metadata.insert("iterations".to_string(), serde_json::json!(iterations));
                metadata.insert(
                    "final_quality".to_string(),
                    serde_json::json!(quality_scores.last().copied().unwrap_or(0.0)),
                );
                metadata.insert(
                    "completed_at".to_string(),
                    serde_json::json!(Utc::now().to_rfc3339()),
                );

                if let Err(e) = session_mgr
                    .update_session_context(
                        sid,
                        crate::orchestration::session_manager::SessionUpdate::Metadata(metadata),
                    )
                    .await
                {
                    warn!("Failed to update session context: {}", e);
                }
            }
        }

        // Update execution state to completed
        execution_state.status = ExecutionStateStatus::Completed;
        execution_state.current_phase = "completed".to_string();
        execution_state.progress_percentage = 100.0;
        execution_state.quality_scores = quality_scores.clone();
        execution_state.current_iteration = iterations;

        // Final checkpoint before completion
        if let Some(ref persistence) = self.state_persistence {
            if let Err(e) = persistence
                .create_checkpoint(plan_id, &execution_state)
                .await
            {
                warn!("Failed to create final checkpoint: {}", e);
            }
            // Optionally delete state after successful completion (or keep for audit)
            // persistence.delete_state(plan_id).await?;
        }

        Ok(ExecutionResult {
            plan_id,
            execution_plan,
            artifacts,
            final_verdict,
            iterations,
            quality_scores,
        })
    }

    /// Get execution status for a plan
    ///
    /// Returns the current execution state if available, or None if not found.
    pub async fn get_execution_status(&self, plan_id: Uuid) -> Result<Option<TaskExecutionState>> {
        if let Some(ref persistence) = self.state_persistence {
            persistence
                .load_state(plan_id)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to load state: {}", e))
        } else {
            Ok(None)
        }
    }

    /// Pause execution of a plan
    ///
    /// Updates the execution state to Paused and creates a checkpoint.
    pub async fn pause_execution(&self, plan_id: Uuid) -> Result<()> {
        if let Some(ref persistence) = self.state_persistence {
            if let Ok(Some(mut state)) = persistence.load_state(plan_id).await {
                state.status = ExecutionStateStatus::Paused;
                state.last_updated = Utc::now();
                persistence
                    .save_state(&state)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to save paused state: {}", e))?;
                persistence
                    .create_checkpoint(plan_id, &state)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to create checkpoint: {}", e))?;
                info!("Paused execution for plan {}", plan_id);
                Ok(())
            } else {
                Err(anyhow::anyhow!("Plan {} not found", plan_id))
            }
        } else {
            Err(anyhow::anyhow!("State persistence not available"))
        }
    }

    /// Resume execution of a paused plan
    ///
    /// Updates the execution state to Running.
    pub async fn resume_execution(&self, plan_id: Uuid) -> Result<()> {
        if let Some(ref persistence) = self.state_persistence {
            if let Ok(Some(mut state)) = persistence.load_state(plan_id).await {
                if state.status != ExecutionStateStatus::Paused {
                    return Err(anyhow::anyhow!(
                        "Plan {} is not paused (status: {:?})",
                        plan_id,
                        state.status
                    ));
                }
                state.status = ExecutionStateStatus::Running;
                state.last_updated = Utc::now();
                persistence
                    .save_state(&state)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to save resumed state: {}", e))?;
                info!("Resumed execution for plan {}", plan_id);
                Ok(())
            } else {
                Err(anyhow::anyhow!("Plan {} not found", plan_id))
            }
        } else {
            Err(anyhow::anyhow!("State persistence not available"))
        }
    }

    /// Cancel execution of a plan
    ///
    /// Updates the execution state to Cancelled and cleans up resources.
    pub async fn cancel_execution(&self, plan_id: Uuid) -> Result<()> {
        if let Some(ref persistence) = self.state_persistence {
            if let Ok(Some(mut state)) = persistence.load_state(plan_id).await {
                // Only cancel if not already completed or cancelled
                if matches!(
                    state.status,
                    ExecutionStateStatus::Completed | ExecutionStateStatus::Cancelled
                ) {
                    return Err(anyhow::anyhow!(
                        "Plan {} cannot be cancelled (status: {:?})",
                        plan_id,
                        state.status
                    ));
                }
                state.status = ExecutionStateStatus::Cancelled;
                state.last_updated = Utc::now();
                state.error = Some("Task cancelled by user".to_string());
                persistence
                    .save_state(&state)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to save cancelled state: {}", e))?;

                // Clean up worktrees for cancelled tasks
                // Note: Worktree cleanup is handled by WorktreeManager, but we should signal cancellation
                info!("Cancelled execution for plan {}", plan_id);
                Ok(())
            } else {
                Err(anyhow::anyhow!("Plan {} not found", plan_id))
            }
        } else {
            Err(anyhow::anyhow!("State persistence not available"))
        }
    }

    /// Execute plan milestones in parallel using ParallelCoordinator
    async fn execute_plan_milestones(
        &self,
        execution_plan: &ExecutionPlan,
    ) -> Result<Vec<ExecutionArtifacts>> {
        // Use ParallelCoordinator to execute plan in parallel
        // Create a mutable copy for ParallelCoordinator
        let mut plan_for_execution = execution_plan.clone();

        // Use ArbiterPipelineOptimizer to make optimized decisions for each milestone
        #[cfg(feature = "runtime-optimization")]
        if let Some(ref optimizer) = self.arbiter_optimizer {
            info!("Using ArbiterPipelineOptimizer for milestone decision optimization");
            for milestone in &mut plan_for_execution.contract_plan.milestones {
                // Create decision input from milestone
                let task_description = format!("{}: {}", milestone.id, milestone.objective);
                let context = format!(
                    "milestone_id={}, dependencies={:?}",
                    milestone.id, milestone.dependencies
                );

                // Make optimized decision
                match optimizer.make_decision(&task_description, &context).await {
                    Ok(decision) => {
                        info!(
                            "Arbiter decision for milestone {}: task_type={}, risk_tier={}, worker_pool={}, confidence={:.2}",
                            milestone.id, decision.task_type, decision.risk_tier, decision.worker_pool, decision.confidence
                        );

                        // Store decision metadata in milestone for later use in worker assignment
                        // The decision result can inform worker selection via worker_assignment_strategy
                        milestone.metadata.insert(
                            "arbiter_task_type".to_string(),
                            serde_json::Value::String(decision.task_type),
                        );
                        milestone.metadata.insert(
                            "arbiter_risk_tier".to_string(),
                            serde_json::Value::String(decision.risk_tier),
                        );
                        milestone.metadata.insert(
                            "arbiter_worker_pool".to_string(),
                            serde_json::Value::String(decision.worker_pool),
                        );
                        milestone.metadata.insert(
                            "arbiter_confidence".to_string(),
                            serde_json::Value::Number(
                                serde_json::Number::from_f64(decision.confidence)
                                    .unwrap_or(serde_json::Number::from(0)),
                            ),
                        );
                    }
                    Err(e) => {
                        warn!(
                            "ArbiterPipelineOptimizer decision failed for milestone {}: {}",
                            milestone.id, e
                        );
                        // Continue execution without optimization
                    }
                }
            }
        }

        // Ensure parallel_batches is populated if empty (plan generator may not have populated it)
        if plan_for_execution
            .execution_context
            .parallel_batches
            .is_empty()
            && !plan_for_execution.contract_plan.milestones.is_empty()
        {
            info!(
                "Populating parallel_batches from {} milestones",
                plan_for_execution.contract_plan.milestones.len()
            );
            // Create a single batch with all milestones
            let milestone_ids: Vec<String> = plan_for_execution
                .contract_plan
                .milestones
                .iter()
                .map(|m| m.id.clone())
                .collect();

            plan_for_execution.execution_context.parallel_batches =
                vec![crate::planning::plan_types::ParallelBatch {
                    batch_index: 0,
                    milestone_ids,
                    status: crate::planning::plan_types::BatchStatus::Pending,
                    started_at: None,
                    completed_at: None,
                    resource_requirements: Default::default(),
                }];
        }

        // Execute plan using ParallelCoordinator
        let parallel_result = self
            .parallel_coordinator
            .execute_plan_parallel(&mut plan_for_execution)
            .await?;

        info!(
            "ParallelCoordinator completed: {} successful, {} failed, {} scope conflicts, {} artifacts collected",
            parallel_result.successful_milestones,
            parallel_result.failed_milestones,
            parallel_result.scope_conflicts,
            parallel_result.artifacts.len()
        );

        // Use artifacts collected during execution from ParallelCoordinator
        let artifacts = if !parallel_result.artifacts.is_empty() {
            parallel_result.artifacts
        } else if parallel_result.successful_milestones > 0 {
            // Fallback: If no artifacts collected but milestones were successful, create minimal artifacts
            // This should not happen in production, but provides safety fallback
            warn!("No artifacts collected from ParallelCoordinator execution - creating minimal artifacts from milestone state");
            let mut fallback_artifacts = Vec::new();
            for milestone in &plan_for_execution.contract_plan.milestones {
                if matches!(
                    milestone.state,
                    agent_agency_contracts::planning_io::MilestoneState::Completed
                ) {
                    let worker_id_str = milestone
                        .assigned_workers
                        .first()
                        .map(|id| id.to_string())
                        .unwrap_or_else(|| milestone.id.clone());

                    let mut artifact = ExecutionArtifacts::default();
                    artifact.task_id = plan_for_execution.contract_plan.id;
                    artifact.working_spec_id =
                        plan_for_execution.contract_plan.working_spec_id.clone();
                    artifact.provenance.worker_id = Some(worker_id_str);
                    artifact.provenance.completed_at = Some(chrono::Utc::now());
                    fallback_artifacts.push(artifact);
                }
            }
            fallback_artifacts
        } else {
            Vec::new()
        };

        Ok(artifacts)
    }

    /// Assign worker to milestone
    #[allow(dead_code)] // Reserved for future use
    async fn assign_worker_to_milestone(&self, milestone: &Milestone) -> Result<Uuid> {
        // Use WorkerAssignmentStrategy if available, otherwise fall back to simple logic
        if let Some(ref assignment_strategy) = self.worker_assignment_strategy {
            assignment_strategy.assign_worker(milestone).await
        } else {
            // Fallback: use first assigned worker or generate new ID
            if let Some(worker_id) = milestone.assigned_workers.first() {
                Ok(*worker_id)
            } else {
                Ok(Uuid::new_v4())
            }
        }
    }
}

// Trait implementations for RefinementLoopCoordinator

/// Orchestration executor implementation for UnifiedOrchestrator
struct UnifiedOrchestrationExecutor {
    plan_generator: Arc<PlanGenerator>,
    worker_bridge: Arc<WorkerExecutionBridge>,
    worktree_manager: Arc<WorktreeManager>,
    worker_lifecycle_manager: Arc<WorkerLifecycleManager>,
    deployment_orchestrator: Option<Arc<DeploymentOrchestrator>>,
}

impl UnifiedOrchestrationExecutor {
    /// Create a comprehensive verdict from execution artifacts and working spec
    /// 
    /// This method aggregates artifacts into a final verdict by:
    /// 1. Validating artifact quality and success
    /// 2. Checking against acceptance criteria from working spec
    /// 3. Creating a verification summary based on artifact outcomes
    /// 4. Generating appropriate decision (Accept/Reject/Modify)
    async fn create_comprehensive_verdict(
        &self,
        artifacts: &[agent_agency_contracts::execution_artifacts::ExecutionArtifacts],
        working_spec: &WorkingSpec,
    ) -> Result<agent_agency_contracts::final_verdict::FinalVerdictContract> {
        use agent_agency_contracts::final_verdict::{FinalDecision, VerificationSummary};

        // Validate artifacts and check against acceptance criteria
        // Check if artifacts have completion timestamps (indicates successful execution)
        // Artifacts with errors or failures would not have completed_at set
        let all_successful = !artifacts.is_empty() && artifacts.iter().all(|artifact| {
            artifact.provenance.completed_at.is_some()
        });

        let successful_count = artifacts.iter()
            .filter(|artifact| artifact.provenance.completed_at.is_some())
            .count();

        // Check acceptance criteria from working spec
        let acceptance_criteria_met = if working_spec.acceptance_criteria.is_empty() {
            // If no explicit acceptance criteria, success is based on artifact completion
            all_successful
        } else {
            // TODO: In a full implementation, we would validate each acceptance criterion
            // against the artifacts. For now, we assume criteria are met if artifacts are successful.
            // This requires:
            // 1. ArtifactValidator implementation to check criteria
            // 2. Claim extraction and verification from artifacts
            // 3. Integration with research/verification pipeline
            all_successful
        };

        // Determine final decision based on artifact outcomes and acceptance criteria
        let decision = if acceptance_criteria_met && all_successful {
            FinalDecision::Accept
        } else if !artifacts.is_empty() && successful_count > 0 {
            // Some artifacts succeeded but not all criteria met - requires modification
            FinalDecision::Modify
        } else {
            FinalDecision::Reject
        };

        // Generate dissent message if needed
        let dissent = if decision == FinalDecision::Accept {
            String::new()
        } else if decision == FinalDecision::Modify {
            format!(
                "{} of {} milestones completed successfully, but acceptance criteria require refinement",
                successful_count,
                artifacts.len()
            )
        } else {
            format!(
                "{} of {} milestones failed or no artifacts produced",
                artifacts.len() - successful_count,
                artifacts.len().max(1)
            )
        };

        // Aggregate test and coverage data from artifacts
        let (total_tests, passed_tests, failed_tests) = artifacts.iter().fold(
            (0, 0, 0),
            |(total, passed, failed), artifact| {
                (
                    total + artifact.tests.unit_tests.total
                        + artifact.tests.integration_tests.total,
                    passed + artifact.tests.unit_tests.passed
                        + artifact.tests.integration_tests.passed,
                    failed + artifact.tests.unit_tests.failed
                        + artifact.tests.integration_tests.failed,
                )
            },
        );

        let avg_line_coverage = if !artifacts.is_empty() {
            artifacts.iter().map(|a| a.coverage.line_coverage).sum::<f64>()
                / artifacts.len() as f64
        } else {
            0.0
        };

        let avg_branch_coverage = if !artifacts.is_empty() {
            artifacts.iter().map(|a| a.coverage.branch_coverage).sum::<f64>()
                / artifacts.len() as f64
        } else {
            0.0
        };

        let total_lint_errors = artifacts.iter().map(|a| a.linting.errors).sum::<u32>();
        let total_lint_warnings = artifacts.iter().map(|a| a.linting.warnings).sum::<u32>();

        // Create votes based on artifact quality analysis
        let votes = self.create_votes_from_artifacts(
            total_tests,
            passed_tests,
            failed_tests,
            avg_line_coverage,
            avg_branch_coverage,
            total_lint_errors,
            total_lint_warnings,
        );

        // Extract remediation steps from artifacts
        let remediation = self.extract_remediation_from_artifacts(artifacts);

        // Extract constitutional references from working spec
        let constitutional_refs = self.extract_constitutional_refs(working_spec);

        // Create verification summary from artifacts
        let claims_total = working_spec.acceptance_criteria.len() as u32;
        let test_success_rate = if total_tests > 0 {
            passed_tests as f64 / total_tests as f64
        } else {
            0.0
        };

        // Estimate verified claims based on test success and coverage
        let claims_verified = if claims_total > 0 {
            let verification_estimate = (test_success_rate * avg_line_coverage * claims_total as f64)
                .ceil() as u32;
            verification_estimate.min(claims_total)
        } else {
            successful_count as u32
        };

        let verification_summary = VerificationSummary {
            claims_total: if claims_total > 0 { claims_total } else { artifacts.len() as u32 },
            claims_verified,
            coverage_pct: (avg_line_coverage * 100.0) as f32,
        };

        // Create final verdict contract
        Ok(agent_agency_contracts::final_verdict::FinalVerdictContract {
            decision,
            votes,
            dissent,
            remediation,
            constitutional_refs,
            verification_summary,
        })
    }

    /// Create vote entries based on artifact quality analysis
    fn create_votes_from_artifacts(
        &self,
        total_tests: u32,
        passed_tests: u32,
        failed_tests: u32,
        avg_line_coverage: f64,
        avg_branch_coverage: f64,
        total_lint_errors: u32,
        total_lint_warnings: u32,
    ) -> Vec<agent_agency_contracts::final_verdict::VoteEntry> {
        use agent_agency_contracts::final_verdict::{VoteEntry, VoteVerdict};

        let mut votes = Vec::new();

        // Test quality vote
        if total_tests > 0 {
            let test_success_rate = passed_tests as f64 / total_tests as f64;
            votes.push(VoteEntry {
                judge_id: "test_quality".to_string(),
                weight: 0.35,
                verdict: if test_success_rate >= 0.95 && failed_tests == 0 {
                    VoteVerdict::Pass
                } else if test_success_rate >= 0.70 {
                    VoteVerdict::Uncertain
                } else {
                    VoteVerdict::Fail
                },
            });
        }

        // Coverage quality vote
        votes.push(VoteEntry {
            judge_id: "coverage_quality".to_string(),
            weight: 0.30,
            verdict: if avg_line_coverage >= 0.80 && avg_branch_coverage >= 0.80 {
                VoteVerdict::Pass
            } else if avg_line_coverage >= 0.60 {
                VoteVerdict::Uncertain
            } else {
                VoteVerdict::Fail
            },
        });

        // Linting quality vote
        votes.push(VoteEntry {
            judge_id: "linting_quality".to_string(),
            weight: 0.20,
            verdict: if total_lint_errors == 0 && total_lint_warnings < 10 {
                VoteVerdict::Pass
            } else if total_lint_errors == 0 && total_lint_warnings < 50 {
                VoteVerdict::Uncertain
            } else {
                VoteVerdict::Fail
            },
        });

        votes
    }

    /// Extract remediation steps from artifact analysis
    fn extract_remediation_from_artifacts(
        &self,
        artifacts: &[agent_agency_contracts::execution_artifacts::ExecutionArtifacts],
    ) -> Vec<String> {
        use agent_agency_contracts::execution_artifacts::IssueSeverity;

        let mut remediation = Vec::new();

        // Remediation from test failures
        for artifact in artifacts {
            if artifact.tests.unit_tests.failed > 0 {
                remediation.push(format!(
                    "Fix {} failing unit test(s) in artifact {}",
                    artifact.tests.unit_tests.failed, artifact.working_spec_id
                ));
            }

            if artifact.tests.integration_tests.failed > 0 {
                remediation.push(format!(
                    "Fix {} failing integration test(s) in artifact {}",
                    artifact.tests.integration_tests.failed, artifact.working_spec_id
                ));
            }

            // Remediation from linting errors
            let critical_errors: Vec<_> = artifact
                .linting
                .issues_by_file
                .values()
                .flatten()
                .filter(|issue| matches!(issue.severity, IssueSeverity::Error))
                .take(5)
                .collect();

            for issue in critical_errors {
                remediation.push(format!(
                    "Fix linting error: {}:{} - {} ({})",
                    issue.line,
                    issue.column.unwrap_or(0),
                    issue.message,
                    issue.code
                ));
            }

            // Remediation from coverage gaps
            if artifact.coverage.line_coverage < 0.80
                && !artifact.coverage.uncovered_lines.is_empty()
            {
                let uncovered_files: Vec<_> = artifact
                    .coverage
                    .uncovered_lines
                    .iter()
                    .take(3)
                    .map(|ul| ul.file.clone())
                    .collect();

                if !uncovered_files.is_empty() {
                    remediation.push(format!(
                        "Improve test coverage for: {}",
                        uncovered_files.join(", ")
                    ));
                }
            }
        }

        remediation
    }

    /// Extract constitutional references from working spec
    fn extract_constitutional_refs(
        &self,
        working_spec: &WorkingSpec,
    ) -> Vec<String> {
        let mut refs = Vec::new();

        // Extract references from acceptance criteria IDs
        for criterion in &working_spec.acceptance_criteria {
            refs.push(format!("AcceptanceCriterion:{}", criterion.id));
        }

        // Extract references from quality gates if available
        if let Some(ref quality_gates) = working_spec.quality_gates {
            if let Some(min_coverage) = quality_gates.min_coverage {
                refs.push(format!("QualityGate:coverage:{}", min_coverage));
            }
            
            // Add coverage requirements from hashmap
            for (test_type, threshold) in &quality_gates.coverage_requirements {
                refs.push(format!("QualityGate:coverage:{}:{}", test_type, threshold));
            }
        }

        refs
    }

    /// Calculate comprehensive quality score from execution artifacts
    /// 
    /// Quality score is a weighted average of:
    /// - Test success rate (35%)
    /// - Code coverage (30%)
    /// - Linting quality (20%)
    /// - Code change statistics (15%)
    fn calculate_quality_score_static(
        artifacts: &[ExecutionArtifacts],
    ) -> f64 {
        if artifacts.is_empty() {
            return 0.0;
        }

        // Aggregate metrics from all artifacts
        let (total_tests, passed_tests, failed_tests) = artifacts.iter().fold(
            (0, 0, 0),
            |(total, passed, failed), artifact| {
                (
                    total + artifact.tests.unit_tests.total
                        + artifact.tests.integration_tests.total,
                    passed + artifact.tests.unit_tests.passed
                        + artifact.tests.integration_tests.passed,
                    failed + artifact.tests.unit_tests.failed
                        + artifact.tests.integration_tests.failed,
                )
            },
        );

        // Test success score (35% weight)
        let test_score = if total_tests > 0 {
            let success_rate = passed_tests as f64 / total_tests as f64;
            // Penalize failed tests more than just success rate
            let failure_penalty = (failed_tests as f64 / total_tests as f64) * 0.5;
            (success_rate - failure_penalty).max(0.0_f64)
        } else {
            0.5_f64 // Neutral score if no tests
        };

        // Coverage score (30% weight)
        let avg_line_coverage = if !artifacts.is_empty() {
            artifacts.iter().map(|a| a.coverage.line_coverage).sum::<f64>()
                / artifacts.len() as f64
        } else {
            0.0
        };

        let avg_branch_coverage = if !artifacts.is_empty() {
            artifacts.iter().map(|a| a.coverage.branch_coverage).sum::<f64>()
                / artifacts.len() as f64
        } else {
            0.0
        };

        let coverage_score = avg_line_coverage * 0.6 + avg_branch_coverage * 0.4;

        // Linting score (20% weight)
        let total_lint_errors = artifacts.iter().map(|a| a.linting.errors).sum::<u32>();
        let total_lint_warnings = artifacts.iter().map(|a| a.linting.warnings).sum::<u32>();
        let total_files = artifacts
            .iter()
            .map(|a| a.linting.issues_by_file.len() as u32)
            .sum::<u32>();

        let lint_score = if total_files > 0 {
            // Base score decreases with errors and warnings
            let error_penalty = (total_lint_errors as f64 / total_files as f64).min(1.0_f64) * 0.7_f64;
            let warning_penalty = (total_lint_warnings as f64 / total_files as f64).min(1.0_f64) * 0.3_f64;
            (1.0_f64 - error_penalty - warning_penalty).max(0.0_f64)
        } else {
            1.0_f64 // Perfect score if no linting issues
        };

        // Code change statistics score (15% weight)
        // Reward well-structured changes, penalize large risky changes
        let mut change_score = 1.0_f64;
        for artifact in artifacts {
            let stats = &artifact.code_changes.statistics;
            let total_lines = stats.lines_added + stats.lines_removed;
            
            // Penalize very large changes (potential refactoring debt)
            if total_lines > 1000 {
                change_score -= 0.1_f64;
            }
            
            // Reward balanced additions/removals
            if stats.lines_added > 0 && stats.lines_removed > 0 {
                let change_ratio = stats.lines_removed as f64 / stats.lines_added as f64;
                // Reward refactoring (0.5-2.0 ratio is good)
                if change_ratio >= 0.5_f64 && change_ratio <= 2.0_f64 {
                    change_score += 0.05_f64;
                }
            }
        }
        change_score = change_score.max(0.0_f64).min(1.0_f64);

        // Weighted combination
        let quality_score: f64 = (test_score * 0.35)
            + (coverage_score * 0.30)
            + (lint_score * 0.20)
            + (change_score * 0.15);

        // Ensure score is in valid range
        quality_score.max(0.0_f64).min(1.0_f64)
    }
}

#[async_trait::async_trait]
impl OrchestrationExecutor for UnifiedOrchestrationExecutor {
    async fn execute_orchestration(
        &self,
        working_spec: &WorkingSpec,
        task_descriptor: &TaskDescriptor,
    ) -> Result<agent_agency_contracts::final_verdict::FinalVerdictContract> {
        // Generate execution plan
        struct SimpleWorkingSpecProvider {
            spec: WorkingSpec,
        }

        #[async_trait::async_trait]
        impl WorkingSpecProvider for SimpleWorkingSpecProvider {
            async fn get_working_spec(&self) -> Result<WorkingSpec> {
                Ok(self.spec.clone())
            }
        }

        struct SimpleTaskDescriptorProvider {
            descriptor: TaskDescriptor,
        }

        #[async_trait::async_trait]
        impl TaskDescriptorProvider for SimpleTaskDescriptorProvider {
            async fn get_task_descriptor(&self) -> Result<TaskDescriptor> {
                Ok(self.descriptor.clone())
            }
        }

        let context = PlanGenerationContext {
            working_spec_provider: Box::new(SimpleWorkingSpecProvider {
                spec: working_spec.clone(),
            }),
            task_descriptor: Box::new(SimpleTaskDescriptorProvider {
                descriptor: task_descriptor.clone(),
            }),
            resource_inventory: ResourceInventory::default(),
            constraints: Default::default(),
            historical_data: None,
            planning_constraints: Default::default(),
            execution_mode: ExecutionMode::Auto,
            planning_strategy: PlanGenerationStrategy::AIAssisted,
        };

        let execution_plan = self.plan_generator.generate(&context).await?;

        // Execute milestones
        let mut artifacts = Vec::new();
        for milestone in &execution_plan.contract_plan.milestones {
            // Select optimal model if deployment orchestrator is available
            let model_id = if let Some(ref deployment) = self.deployment_orchestrator {
                // Create task requirements from milestone
                let requirements = serde_json::json!({
                    "task_id": milestone.id,
                    "objective": milestone.objective,
                    "priority": milestone.priority,
                });
                
                match deployment.select_optimal_model(&requirements).await {
                    Ok(selection) => {
                        info!("Selected optimal model {} for milestone {}", selection.model_id, milestone.id);
                        Some(selection.model_id)
                    }
                    Err(e) => {
                        warn!("Failed to select optimal model: {}", e);
                        None
                    }
                }
            } else {
                None
            };

            let worker_id = Uuid::new_v4();
            self.worker_lifecycle_manager
                .handle_assignment(worker_id, milestone)
                .await?;

            let worktree_path = self
                .worktree_manager
                .create_worktree(milestone, worker_id)
                .await?
                .worktree_path;
            let artifact = self
                .worker_bridge
                .execute_milestone(milestone, &worktree_path, worker_id, model_id)
                .await?;
            self.worker_lifecycle_manager
                .handle_completion(worker_id, artifact.clone())
                .await?;
            artifacts.push(artifact);
        }

        // Create comprehensive verdict from artifacts and working spec
        self.create_comprehensive_verdict(&artifacts, working_spec).await
    }
}

/// Artifact validator implementation
struct UnifiedArtifactValidator;

#[async_trait::async_trait]
impl ArtifactValidator for UnifiedArtifactValidator {
    async fn validate_execution_artifacts(
        &self,
        verdict: &agent_agency_contracts::final_verdict::FinalVerdictContract,
        _task_descriptor: &TaskDescriptor,
    ) -> Result<bool> {
        // Basic validation: check if verdict indicates success
        Ok(matches!(
            verdict.decision,
            agent_agency_contracts::final_verdict::FinalDecision::Accept
        ))
    }
}

/// Council reviewer implementation
struct UnifiedCouncilReviewer {
    council: Arc<Council>,
}

#[async_trait::async_trait]
impl CouncilReviewer for UnifiedCouncilReviewer {
    async fn perform_council_review(
        &self,
        working_spec: &WorkingSpec,
        _task_descriptor: &TaskDescriptor,
    ) -> Result<(bool, bool, String)> {
        // Use council to review the working spec
        use crate::decision_making::FinalDecision;
        use crate::judge_backup::types::ReviewContext;

        let _review_context = ReviewContext {
            session_id: format!("review_{}", Uuid::new_v4()),
            working_spec: serde_json::to_string(working_spec)
                .map_err(|e| anyhow::anyhow!("Failed to serialize working spec: {}", e))?,
            risk_tier: working_spec.risk_tier as u8,
            previous_reviews: vec![],
            constraints: std::collections::HashMap::new(),
        };

        let session = self
            .council
            .conduct_review(working_spec.clone(), _review_context)
            .await
            .map_err(|e| anyhow::anyhow!("Council review failed: {:?}", e))?;

        let approved = session
            .final_decision
            .as_ref()
            .map(|d| matches!(d, FinalDecision::Proceed { .. }))
            .unwrap_or(false);

        let needs_refinement = session
            .final_decision
            .as_ref()
            .map(|d| matches!(d, FinalDecision::Refine { .. }))
            .unwrap_or(false);

        let reason = match session.final_decision.as_ref() {
            Some(FinalDecision::Refine {
                refinement_directive,
                ..
            }) => {
                format!("Refinement required: {:?}", refinement_directive)
            }
            Some(FinalDecision::Reject { reason, .. }) => {
                format!("Rejected: {}", reason)
            }
            _ => String::new(),
        };

        Ok((approved, needs_refinement, reason))
    }
}

/// Spec refiner implementation using intelligent refinement
struct UnifiedSpecRefiner {
    /// Intelligent spec refiner for council feedback-based improvements
    intelligent_refiner: crate::planning::intelligent_spec_refiner::IntelligentSpecRefiner,
}

impl UnifiedSpecRefiner {
    /// Create a new unified spec refiner with intelligent refinement capabilities
    fn new() -> Self {
        Self {
            intelligent_refiner: crate::planning::intelligent_spec_refiner::IntelligentSpecRefiner::new(),
        }
    }
}

impl Default for UnifiedSpecRefiner {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl SpecRefiner for UnifiedSpecRefiner {
    async fn refine_working_spec(
        &self,
        current_spec: &WorkingSpec,
        refinement_reason: &str,
    ) -> Result<WorkingSpec> {
        // Use intelligent refinement based on council feedback
        info!(
            "Refining working spec '{}' using intelligent refinement",
            current_spec.title
        );

        // Parse council feedback into structured directives
        let directive = self.intelligent_refiner.parse_council_feedback(refinement_reason);

        debug!(
            "Parsed {} improvement areas from council feedback: {:?}",
            directive.improvement_areas.len(),
            directive.priority
        );

        // Apply refinements
        let result = self.intelligent_refiner.apply_refinements(current_spec, &directive);

        // Log refinement actions
        for action in &result.actions {
            if action.successful {
                info!("[Refinement] {}: {}", action.area, action.description);
            } else {
                warn!("[Refinement] Failed {}: {}", action.area, action.description);
            }
        }

        // Log unresolved issues for manual review
        if !result.unresolved_issues.is_empty() {
            warn!(
                "Unresolved refinement issues requiring manual review: {:?}",
                result.unresolved_issues
            );
        }

        info!(
            "Intelligent refinement complete. Estimated quality improvement: {:.1}%",
            result.estimated_quality_improvement * 100.0
        );

        Ok(result.refined_spec)
    }
}

/// Progress tracker implementation that delegates to RealTimeProgressTracker
struct UnifiedProgressTracker {
    base_tracker: Arc<dyn crate::progress_tracker::ProgressTracker>,
}

#[async_trait::async_trait]
impl ProgressTracker for UnifiedProgressTracker {
    async fn update_task_progress(
        &self,
        task_id: Uuid,
        progress: f32,
        message: Option<String>,
    ) -> Result<()> {
        // Delegate to RealTimeProgressTracker
        let execution_progress = crate::progress_tracker::ExecutionProgress {
            task_id,
            status: crate::progress_tracker::ExecutionStatus::Running,
            percentage: progress as f64,
            current_phase: message.clone().unwrap_or_else(|| "executing".to_string()),
            total_phases: 5, // Standard 5-phase execution
            current_phase_index: 0,
            started_at: chrono::Utc::now(),
            last_updated: chrono::Utc::now(),
            estimated_completion: None,
            messages: vec![],
            error: None,
            metrics: crate::progress_tracker::ProgressMetrics {
                cpu_usage: 0.0,
                memory_usage: 0,
                network_io: 0,
                disk_io: 0,
                processing_rate: 0.0,
                error_count: 0,
                retry_count: 0,
            },
        };

        self.base_tracker
            .update_progress(task_id, execution_progress)
            .await
            .map_err(|e| anyhow!("Failed to update progress: {}", e))?;

        Ok(())
    }

    async fn update_task_status(
        &self,
        task_id: Uuid,
        status: ExecutionStatus,
        message: Option<String>,
    ) -> Result<()> {
        // Update progress with status information
        let progress = match status {
            ExecutionStatus::Pending => 0.0,
            ExecutionStatus::Running => 50.0,
            ExecutionStatus::Completed => 100.0,
            ExecutionStatus::Failed => 0.0,
            ExecutionStatus::Cancelled => 0.0,
            ExecutionStatus::Timeout => 0.0,
        };

        self.update_task_progress(task_id, progress, message).await
    }

    async fn track_iteration_progress(
        &self,
        task_id: Uuid,
        iteration: u32,
        quality_score: f64,
        improvement_delta: f64,
    ) -> Result<()> {
        // Update progress with iteration information
        let progress = (iteration as f32 * 10.0).min(90.0); // Cap at 90% until final
        let message = format!(
            "Iteration {}: quality={:.2}, improvement={:.2}",
            iteration, quality_score, improvement_delta
        );

        let execution_progress = crate::progress_tracker::ExecutionProgress {
            task_id,
            status: crate::progress_tracker::ExecutionStatus::Running,
            percentage: progress as f64,
            current_phase: message.clone(),
            total_phases: 5,
            current_phase_index: iteration as usize,
            started_at: chrono::Utc::now(),
            last_updated: chrono::Utc::now(),
            estimated_completion: None,
            messages: vec![],
            error: None,
            metrics: crate::progress_tracker::ProgressMetrics {
                cpu_usage: 0.0,
                memory_usage: 0,
                network_io: 0,
                disk_io: 0,
                processing_rate: 0.0,
                error_count: 0,
                retry_count: 0,
            },
        };

        self.base_tracker
            .update_progress(task_id, execution_progress)
            .await
            .map_err(|e| anyhow!("Failed to track iteration progress: {}", e))?;

        Ok(())
    }

    async fn detect_and_report_plateaus(
        &self,
        task_id: Uuid,
        quality_scores: &[f64],
        iteration: u32,
    ) -> Result<()> {
        // TODO: Implement comprehensive quality plateau detection
        //       Currently uses basic stagnant score detection; should implement sophisticated plateau detection using turn-level tracker and statistical analysis for accurate quality trend identification.
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - Plateau detection uses turn-level tracker when available
        // - Statistical analysis identifies meaningful quality trends
        // - Detection thresholds are configurable
        // - False positives are minimized through proper statistical methods
        //
        // DEPENDENCIES:
        // - Turn-level tracker integration (Optional)
        // - Statistical analysis utilities (Required)
        // - Quality score history tracking (Required)
        //
        // ESTIMATED EFFORT: 6-8 hours (medium confidence)
        // PRIORITY: Low
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (quality monitoring enhancement)
        // - Change Budget: ~150 LOC
        // - Reviewer Requirements: Statistical analysis and quality monitoring expertise
        if quality_scores.len() >= 3 {
            let recent_scores = &quality_scores[quality_scores.len().saturating_sub(3)..];
            let avg_recent: f64 = recent_scores.iter().sum::<f64>() / recent_scores.len() as f64;
            let variance: f64 = recent_scores
                .iter()
                .map(|s| (s - avg_recent).powi(2))
                .sum::<f64>()
                / recent_scores.len() as f64;

            if variance < 0.01 {
                warn!(
                    "Plateau detected at iteration {}: quality variance={:.4}",
                    iteration, variance
                );
                // Update progress with plateau warning
                let warning_msg = crate::progress_tracker::ProgressMessage {
                    timestamp: chrono::Utc::now(),
                    level: crate::progress_tracker::MessageLevel::Warning,
                    content: "Quality plateau detected - consider refinement".to_string(),
                    context: Some({
                        let mut ctx = std::collections::HashMap::new();
                        ctx.insert("quality_variance".to_string(), serde_json::json!(variance));
                        ctx.insert(
                            "avg_recent_quality".to_string(),
                            serde_json::json!(avg_recent),
                        );
                        ctx
                    }),
                };
                let execution_progress = crate::progress_tracker::ExecutionProgress {
                    task_id,
                    status: crate::progress_tracker::ExecutionStatus::Running,
                    percentage: (iteration as f64 * 10.0).min(90.0),
                    current_phase: format!(
                        "Iteration {}: Plateau detected (quality variance={:.4})",
                        iteration, variance
                    ),
                    total_phases: 5,
                    current_phase_index: iteration as usize,
                    started_at: chrono::Utc::now(),
                    last_updated: chrono::Utc::now(),
                    estimated_completion: None,
                    messages: vec![warning_msg],
                    error: None,
                    metrics: crate::progress_tracker::ProgressMetrics {
                        cpu_usage: 0.0,
                        memory_usage: 0,
                        network_io: 0,
                        disk_io: 0,
                        processing_rate: 0.0,
                        error_count: 0,
                        retry_count: 0,
                    },
                };

                let _ = self
                    .base_tracker
                    .update_progress(task_id, execution_progress)
                    .await;
            }
        }

        Ok(())
    }
}
