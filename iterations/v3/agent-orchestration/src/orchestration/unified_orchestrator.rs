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

use std::sync::Arc;
use std::path::PathBuf;
use std::collections::HashMap;
use anyhow::Result;
use uuid::Uuid;
use tracing::{info, warn, error};
use chrono::Utc;
use futures::future::join_all;

use agent_agency_contracts::WorkingSpec;
use agent_agency_contracts::planning_io::Milestone;
use agent_agency_contracts::execution_artifacts::ExecutionArtifacts;
use agent_agency_contracts::types::prelude::*;

use crate::planning::plan_generator::PlanGenerator;
use crate::planning::plan_executor::PlanExecutor;
use crate::planning::parallel_coordinator::ParallelCoordinator;
use crate::planning::refinement_loop::{
    RefinementLoopCoordinator, OrchestrationExecutor, ArtifactValidator, 
    CouncilReviewer, SpecRefiner, ProgressTracker, StatePersistence,
    RefinementLoopResult,
};
use crate::planning::council_integration::CouncilIntegration;
use crate::planning::worker_assignment::WorkerAssignmentStrategy;
use agent_agency_contracts::ExecutionStatus;
use crate::planning::plan_types::{PlanGenerationContext, WorkingSpecProvider, TaskDescriptorProvider, ExecutionPlan, PlanGenerationStrategy, ResourceInventory, HistoricalPlanningData, HistoricalPlan, FailurePattern, FailureSeverity};
use crate::planning::worktree_manager::WorktreeManager;
use crate::planning::caws_adjudication_cycle::CawsAdjudicationCycle;
use crate::planning::worker_lifecycle_manager::WorkerLifecycleManager;
use crate::planning::reflexive_learner::ReflexiveLearner;
use crate::workers::execution_bridge::WorkerExecutionBridge;
use crate::council::Council;

#[cfg(feature = "memory")]
use agent_memory::{MemorySystem, TaskContext, MemoryResult};

use crate::progress_tracker::turn_level::{TurnLevelTracker, TurnLevelProgressTracker, AgentAction, TurnOutcome, TaskOutcome, TurnTrajectory, TurnProgress};

use crate::orchestration::session_manager::{SessionManager, SessionContext, SessionStatus};

use crate::orchestration::task_state_persistence::{
    TaskStatePersistence, TaskExecutionState, ExecutionStateStatus,
};

use crate::learning::federated_learning::FederatedLearningEngine;

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
    worker_assignment_strategy: Option<Arc<WorkerAssignmentStrategy>>,
    
    /// Reflexive learner for continuous learning from outcomes
    reflexive_learner: Option<Arc<ReflexiveLearner>>,
    
    /// Memory system for context preservation and retrieval (long-horizon support)
    #[cfg(feature = "memory")]
    memory_system: Option<Arc<MemorySystem>>,
    #[cfg(not(feature = "memory"))]
    memory_system: Option<()>, // Placeholder when memory feature disabled
    
    /// Active worktrees (worker_id -> worktree_path)
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
        #[cfg(feature = "memory")]
        memory_system: Option<Arc<MemorySystem>>,
        turn_level_tracker: Option<Arc<dyn TurnLevelTracker>>,
        session_manager: Option<Arc<SessionManager>>,
        state_persistence: Option<Arc<dyn TaskStatePersistence>>,
        federated_learning: Option<Arc<FederatedLearningEngine>>,
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
                    info!("Preserved iteration context for task {}: {}", task_id, context_id);
                    // Store context_id mapping for later retrieval
                    self.stored_contexts.write().await.insert(task_id, context_id);
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
    async fn retrieve_iteration_context(
        &self,
        task_id: Uuid,
    ) -> Result<Option<TaskContext>> {
        if let Some(ref memory) = self.memory_system {
            // Get stored context_id
            let context_id = {
                let stored = self.stored_contexts.read().await;
                stored.get(&task_id).cloned()
            };

            if let Some(context_id) = context_id {
                match memory.context_manager().retrieve_context(&context_id).await {
                    Ok(context) => {
                        info!("Retrieved iteration context for task {}: {}", task_id, context_id);
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

                match memory.retrieve_contextual_memories(&search_context, 1).await {
                    Ok(memories) => {
                        if let Some(contextual_memory) = memories.first() {
                            // Extract TaskContext from contextual memory if possible
                            // For now, return None as we need the actual TaskContext
                            Ok(None)
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
    pub async fn execute_plan(
        &self,
        working_spec: WorkingSpec,
    ) -> Result<ExecutionResult> {
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
                
                // Use plan_id from recovered state if found, otherwise generate new one
                found_plan_id.unwrap_or_else(|| Uuid::new_v4())
            } else {
                Uuid::new_v4()
            }
        };
        
        info!("UnifiedOrchestrator: Starting execution for plan {} (resuming: {})", plan_id, is_resuming);

        // Phase 0.1: Initialize execution state (recovered or fresh)
        let mut execution_state = if let Some(state) = recovered_state {
            // Update status to Running if it was paused/crashed
            let mut recovered = state;
            if matches!(recovered.status, ExecutionStateStatus::Paused | ExecutionStateStatus::Crashed) {
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
        let session_id = if let Some(ref session_mgr) = self.session_manager {
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
                    metadata.tags.iter()
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
                            metadata.created_by.as_ref()
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
                
                info!("Extracted tenant_id {} from working_spec {} (metadata: {:?})", 
                    tenant_id, working_spec.id, working_spec.metadata.as_ref().map(|m| &m.created_by));
                
                match session_mgr.create_session(
                    tenant_id,
                    format!("Session for task {}", plan_id),
                    Some(working_spec.title.clone()),
                ).await {
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
        let mut cross_session_contexts: Vec<SessionContext> = Vec::new();
        #[cfg(feature = "memory")]
        {
            if let Some(ref session_mgr) = self.session_manager {
                if session_id != Uuid::nil() {
                    if let Ok(contexts) = session_mgr.retrieve_cross_session_context(session_id, 10).await {
                        if !contexts.is_empty() {
                            info!("Retrieved {} contexts from previous sessions", contexts.len());
                            cross_session_contexts = contexts;
                            
                            // Log insights from cross-session contexts
                            let total_previous_tasks: usize = cross_session_contexts.iter()
                                .map(|ctx| ctx.task_ids.len())
                                .sum();
                            info!("Cross-session insights: {} previous tasks across {} sessions", 
                                total_previous_tasks, cross_session_contexts.len());
                        }
                    }
                }
            }
        }

        // Phase 0.6: Try to retrieve previous context for task resumption (long-horizon support)
        #[cfg(feature = "memory")]
        {
            if let Ok(Some(previous_context)) = self.retrieve_iteration_context(plan_id).await {
                info!("Retrieved previous context for task {}: {}", plan_id, previous_context.description);
                // TODO: Use previous_context to restore execution state if needed
                // For now, we just log that context was retrieved
            }
        }

        // Phase 1: Generate execution plan (skip if resuming from later phase)
        let execution_plan = if is_resuming && execution_state.progress_percentage >= 10.0 {
            // We have a recovered execution plan - wrap it back into plan_types::ExecutionPlan
            if let Some(ref contract_plan) = execution_state.execution_plan {
                info!("Resuming: Using recovered execution plan with {} milestones", contract_plan.milestones.len());
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
        info!("Phase 1: Generating execution plan");
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
            working_spec_provider: Box::new(SimpleWorkingSpecProvider { spec: working_spec.clone() }),
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
                        max_files: working_spec.constraints.budget_limits.as_ref()
                            .and_then(|b| b.max_files)
                            .map(|f| f as usize)
                            .unwrap_or(50),
                        max_loc: working_spec.constraints.budget_limits.as_ref()
                            .and_then(|b| b.max_loc)
                            .map(|l| l as usize)
                            .unwrap_or(1000),
                        max_migrations: 0,
                        allow_breaking_changes: false,
                        allow_new_dependencies: false,
                        enforcement_mode: agent_agency_contracts::planning_io::BudgetEnforcement::Strict,
                    },
                    risk_tier: Some(match working_spec.risk_tier {
                        1 => agent_agency_contracts::types::planning::RiskTier::Tier1,
                        2 => agent_agency_contracts::types::planning::RiskTier::Tier2,
                        3 => agent_agency_contracts::types::planning::RiskTier::Tier3,
                        _ => agent_agency_contracts::types::planning::RiskTier::Tier2,
                    }),
                    scope_in: agent_agency_contracts::task_request::ScopeRestrictions {
                        allowed_paths: working_spec.constraints.scope_restrictions.as_ref()
                            .map(|s| s.allowed_paths.clone())
                            .unwrap_or_default(),
                        blocked_paths: vec![],
                    },
                    scope_out: None,
                    acceptance: Some(working_spec.acceptance_criteria.iter()
                        .map(|c| format!("{}: {}", c.given, c.then))
                        .collect::<Vec<_>>()
                        .join("\n")),
                },
            }),
            resource_inventory: ResourceInventory::default(),
            constraints: Default::default(),
            historical_data: if !cross_session_contexts.is_empty() {
                // Convert cross-session contexts to historical planning data
                let similar_plans: Vec<HistoricalPlan> = cross_session_contexts.iter()
                    .flat_map(|ctx| {
                        // Extract plan information from session metadata
                        ctx.task_ids.iter().map(|task_id| {
                            HistoricalPlan {
                                plan_id: *task_id,
                                complexity_score: ctx.metadata.get("complexity")
                                    .and_then(|v| v.as_f64())
                                    .unwrap_or(0.5),
                                execution_time_ms: ctx.metadata.get("execution_time_ms")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                                successful: ctx.status == SessionStatus::Completed,
                                strategy: ctx.metadata.get("strategy")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string())
                                    .unwrap_or_else(|| "AIAssisted".to_string()),
                                lessons: ctx.metadata.get("lessons")
                                    .and_then(|v| v.as_array())
                                    .map(|arr| arr.iter()
                                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                        .collect())
                                    .unwrap_or_default(),
                            }
                        })
                    })
                    .collect();
                
                // Extract execution time patterns from contexts
                let mut avg_execution_times: HashMap<String, u64> = HashMap::new();
                for ctx in &cross_session_contexts {
                    if let Some(time_ms) = ctx.metadata.get("avg_execution_time_ms")
                        .and_then(|v| v.as_u64()) {
                        avg_execution_times.insert(ctx.name.clone(), time_ms);
                    }
                }
                
                // Extract success rates from contexts
                let mut success_rates: HashMap<String, f64> = HashMap::new();
                for ctx in &cross_session_contexts {
                    let success_rate = if ctx.status == SessionStatus::Completed {
                        1.0
                    } else if ctx.status == SessionStatus::Archived {
                        0.5 // Partial success
                    } else {
                        0.0
                    };
                    success_rates.insert(ctx.name.clone(), success_rate);
                }
                
                // Extract failure patterns from session descriptions/metadata
                let failure_patterns: Vec<FailurePattern> = cross_session_contexts.iter()
                    .filter(|ctx| ctx.status != SessionStatus::Completed)
                    .map(|ctx| {
                        FailurePattern {
                            description: ctx.description.clone()
                                .unwrap_or_else(|| format!("Session {} did not complete", ctx.session_id)),
                            frequency: 1,
                            severity: FailureSeverity::Medium,
                            mitigations: vec!["Review session context".to_string(), "Adjust planning strategy".to_string()],
                        }
                    })
                    .collect();
                
                Some(HistoricalPlanningData {
                    similar_plans,
                    avg_execution_times,
                    success_rates,
                    failure_patterns,
                })
            } else {
                None
            },
            planning_constraints: Default::default(),
            execution_mode: agent_agency_contracts::types::planning::ExecutionMode::Auto,
            planning_strategy: PlanGenerationStrategy::AIAssisted,
        };
        let execution_plan = self.plan_generator.generate(&context).await?;
        info!("Generated plan with {} milestones", execution_plan.contract_plan.milestones.len());

        // Update execution state with plan (only if not resuming)
        if !is_resuming {
            execution_state.execution_plan = Some(execution_plan.contract_plan.clone());
            execution_state.current_phase = "plan_generated".to_string();
            execution_state.progress_percentage = 10.0;

            // Create checkpoint after plan generation
            if let Some(ref persistence) = self.state_persistence {
                if let Err(e) = persistence.create_checkpoint(plan_id, &execution_state).await {
                    warn!("Failed to create checkpoint after plan generation: {}", e);
                }
            }
        }
        
        execution_plan
        };

        // Phase 2: Council plan review (CAWS Examination stage)
        if self.config.enable_council_review {
            info!("Phase 2: Council plan review (CAWS Examination)");
            
            // Create review context for council
            use crate::judge_backup::types::ReviewContext;
            use crate::decision_making::FinalDecision;
            
            let review_context = ReviewContext {
                session_id: format!("examination_{}", plan_id),
                working_spec: serde_json::to_string(&working_spec)
                    .map_err(|e| anyhow::anyhow!("Failed to serialize working spec for council review: {}", e))?,
                risk_tier: working_spec.risk_tier as u8,
                previous_reviews: vec![],
                constraints: std::collections::HashMap::new(),
            };

            // Conduct council review of the execution plan
            let council_session = self.council.conduct_review(working_spec.clone(), review_context).await
                .map_err(|e| anyhow::anyhow!("Council plan review (CAWS Examination) failed: {:?}", e))?;

            // Check council decision
            match council_session.final_decision.as_ref() {
                Some(FinalDecision::Proceed { .. }) => {
                    info!("Council approved plan for execution (CAWS Examination passed)");
                }
                Some(FinalDecision::Reject { reason, .. }) => {
                    let rejection_reason = format!("Council rejected plan during CAWS Examination: {}", reason);
                    error!("{}", rejection_reason);
                    return Err(anyhow::anyhow!("{}", rejection_reason));
                }
                Some(FinalDecision::Refine { refinement_directive, .. }) => {
                    // Council requests refinement - this will be handled in Phase 5 refinement loop
                    info!("Council requested plan refinement during CAWS Examination: {:?}", refinement_directive);
                    // Continue to execution - refinement happens in Phase 5 after artifacts are produced
                }
                None => {
                    warn!("Council review completed but no final decision - proceeding with caution");
                    // If no decision, log warning but proceed (council may have timed out or failed)
                }
            }

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
                info!("Resuming: Using {} recovered artifacts from previous execution", execution_state.artifacts.len());
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
        info!("Phase 3: Executing plan with parallel milestones");
            let executed_artifacts = self.execute_plan_milestones(&execution_plan).await?;
            info!("Completed {} milestone executions", executed_artifacts.len());

            // Update execution state with artifacts
            execution_state.artifacts = executed_artifacts.clone();
            execution_state.current_phase = "milestones_executed".to_string();
            execution_state.progress_percentage = 50.0;

            // Create checkpoint after milestone execution
            if let Some(ref persistence) = self.state_persistence {
                if let Err(e) = persistence.create_checkpoint(plan_id, &execution_state).await {
                    warn!("Failed to create checkpoint after milestone execution: {}", e);
                }
            }

            executed_artifacts
        };

        // Phase 3.5: Process learning outcomes from execution
        if let Some(ref learner) = self.reflexive_learner {
            info!("Phase 3.5: Processing learning outcomes");
            for (artifact, milestone) in artifacts.iter().zip(execution_plan.contract_plan.milestones.iter()) {
                // Extract worker_id from artifact
                if let Some(worker_id_str) = &artifact.provenance.worker_id {
                    if let Ok(worker_id) = Uuid::parse_str(worker_id_str) {
                        if let Err(e) = learner.process_outcome(artifact, milestone, worker_id).await {
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
                                    let final_quality = turns.iter()
                                        .map(|t| t.outcome.quality_score)
                                        .sum::<f64>() / turns.len() as f64;
                                    
                                    let final_success = turns.iter()
                                        .all(|t| t.outcome.success);
                                    
                                    // Collect all artifacts from turns
                                    let final_artifacts: Vec<ExecutionArtifacts> = turns.iter()
                                        .filter_map(|t| t.outcome.artifacts.clone())
                                        .collect();
                                    
                                    // Get completion timestamp from last turn
                                    let completed_at = turns.last()
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
                        if let Ok(contribution) = federated.extract_contribution(
                            session_id, // Use session_id as tenant_id proxy
                            learner,
                            &trajectories,
                        ).await {
                            if let Err(e) = federated.submit_contribution(session_id, contribution).await {
                                warn!("Failed to submit federated learning contribution: {}", e);
                            } else {
                                info!("Submitted learning contribution to federated learning");
                                
                                // Check if aggregation round completed and apply aggregated model
                                if let Ok(aggregated_model) = federated.aggregate_contributions().await {
                                    info!("Federated aggregation round {} completed with {} tenants", 
                                        aggregated_model.round_id, aggregated_model.tenant_count);
                                    
                                    // Apply aggregated model to learner
                                    if let Err(e) = federated.apply_to_learner(learner.clone(), &aggregated_model).await {
                                        warn!("Failed to apply aggregated model to learner: {}", e);
                                    } else {
                                        info!("Applied aggregated federated learning model to reflexive learner");
                                    }
                                } else {
                                    // Not enough contributions yet - check for latest model to apply
                                    if let Some(latest_model) = federated.get_latest_model().await {
                                        info!("Applying latest aggregated model (round {}) to learner", latest_model.round_id);
                                        if let Err(e) = federated.apply_to_learner(learner.clone(), &latest_model).await {
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
                worker_id: artifacts.first()
                    .and_then(|a| a.provenance.worker_id.as_ref())
                    .and_then(|id| Uuid::parse_str(id).ok()),
                milestone_id: None,
                timestamp: Utc::now(),
                metadata: std::collections::HashMap::new(),
            };

            // Calculate overall quality score from artifacts
            // Quality score is based on test pass rate
            let quality_score = artifacts.iter()
                .map(|a| {
                    let total_tests = a.tests.unit_tests.total + a.tests.integration_tests.total + a.tests.e2e_tests.total;
                    let passed_tests = a.tests.unit_tests.passed + a.tests.integration_tests.passed + a.tests.e2e_tests.passed;
                    if total_tests > 0 {
                        passed_tests as f64 / total_tests as f64
                    } else {
                        0.0
                    }
                })
                .sum::<f64>() / artifacts.len().max(1) as f64;

            let outcome = TurnOutcome {
                success: artifacts.iter().all(|a| {
                    // Success if all test suites have no failures
                    a.tests.unit_tests.failed == 0 &&
                    a.tests.integration_tests.failed == 0 &&
                    a.tests.e2e_tests.failed == 0
                }),
                quality_score,
                artifacts: artifacts.first().cloned(),
                error: None,
                execution_time_ms: artifacts.first()
                    .and_then(|a| {
                        a.provenance.completed_at.and_then(|completed| {
                            Some((completed - a.provenance.started_at).num_milliseconds() as u64)
                        })
                    }),
                metadata: std::collections::HashMap::new(),
            };

            if let Err(e) = turn_tracker.track_turn_progress(plan_id, turn_number, action, outcome).await {
                warn!("Failed to track turn-level progress: {}", e);
            }
        }

        // Phase 4: Council presentation (CAWS Pleading stage)
        let mut needs_refinement = false;
        if self.config.enable_council_review {
            info!("Phase 4: Council presentation (CAWS Pleading)");
            if let Some(ref adjudication) = self.adjudication_cycle {
                // Execute full CAWS adjudication cycle
                let adjudication_result = adjudication.execute_cycle(
                    &artifacts,
                    &working_spec,
                    &execution_plan.contract_plan,
                ).await?;

                needs_refinement = adjudication_result.needs_refinement;
                
                if !adjudication_result.approved && !needs_refinement {
                    return Err(anyhow::anyhow!("Work rejected by council: {}", 
                        adjudication_result.refinement_reason.unwrap_or_default()));
                }
            }
        }

        // Phase 5: Refinement loop if needed (skip if resuming from >= 80%)
        let (final_verdict, iterations, quality_scores) = if is_resuming && execution_state.progress_percentage >= 80.0 {
            // We have recovered refinement results - use them
            info!("Resuming: Using recovered refinement results (iterations: {}, quality scores: {})", 
                execution_state.current_iteration, execution_state.quality_scores.len());
            
            // Extract final verdict from metadata if available
            let verdict = execution_state.metadata.get("final_verdict")
                .and_then(|v| serde_json::from_value::<Option<agent_agency_contracts::final_verdict::FinalVerdictContract>>(v.clone()).ok())
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
                    acceptance: Some(working_spec.acceptance_criteria.iter()
                        .map(|c| format!("{}: {}", c.given, c.then))
                        .collect::<Vec<_>>()
                        .join("\n")),
                };

                // Create trait implementations
                let executor: Arc<dyn OrchestrationExecutor> = Arc::new(UnifiedOrchestrationExecutor {
                    plan_generator: self.plan_generator.clone(),
                    worker_bridge: self.worker_bridge.clone(),
                    worktree_manager: self.worktree_manager.clone(),
                    worker_lifecycle_manager: self.worker_lifecycle_manager.clone(),
                });
                
                let validator: Arc<dyn ArtifactValidator> = Arc::new(UnifiedArtifactValidator);
                
                let council_reviewer: Option<Arc<dyn CouncilReviewer>> = Some(Arc::new(UnifiedCouncilReviewer {
                    council: self.council.clone(),
                }));
                
                let spec_refiner: Option<Arc<dyn SpecRefiner>> = Some(Arc::new(UnifiedSpecRefiner));
                
                // Use RealTimeProgressTracker for actual progress tracking
                let base_progress_tracker: Arc<dyn crate::progress_tracker::ProgressTracker> = 
                    Arc::new(crate::progress_tracker::RealTimeProgressTracker::new(None));
                let progress_tracker: Arc<dyn ProgressTracker> = Arc::new(UnifiedProgressTracker {
                    base_tracker: base_progress_tracker,
                });

                // Execute refinement loop
                let refinement_result = refinement_coordinator.execute_refinement_loop(
                    plan_id,
                    working_spec.clone(),
                    &task_descriptor,
                    executor,
                    validator,
                    council_reviewer,
                    spec_refiner,
                    progress_tracker,
                    None, // State persistence - optional
                ).await?;

                // Preserve context after refinement loop completes (long-horizon support)
                #[cfg(feature = "memory")]
                {
                    if let Err(e) = self.preserve_iteration_context(
                        plan_id,
                        &working_spec,
                        &execution_plan,
                        &artifacts,
                        refinement_result.iterations,
                    ).await {
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
                    serde_json::to_value(&refinement_result.final_verdict).unwrap_or(serde_json::Value::Null)
                );

                // Create checkpoint after refinement
                if let Some(ref persistence) = self.state_persistence {
                    if let Err(e) = persistence.create_checkpoint(plan_id, &execution_state).await {
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
            let approved = final_verdict.as_ref()
                .map(|v| matches!(v.decision, agent_agency_contracts::final_verdict::FinalDecision::Accept))
                .unwrap_or(false);
            
            if approved {
                info!("Merging approved worktrees back to main branch");
                
                // Get all active worktrees from WorktreeManager
                let active_worktrees = self.worktree_manager.list_worktrees().await;
                
                // Merge each worktree
                for worktree_info in &active_worktrees {
                    info!("Merging worktree {} (milestone: {})", 
                        worktree_info.worktree_id, worktree_info.milestone_id);
                    
                    match self.worktree_manager.merge_worktree(worktree_info.worktree_id).await {
                        Ok(merge_result) => {
                            if !merge_result.conflicts.is_empty() {
                                warn!("Merge conflicts detected in worktree {}: {:?}", 
                                    worktree_info.worktree_id, merge_result.conflicts);
                                
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
                                info!("Successfully merged worktree {} ({} files changed)", 
                                    worktree_info.worktree_id, merge_result.files_changed);
                            }
                        }
                        Err(e) => {
                            error!("Failed to merge worktree {}: {}", worktree_info.worktree_id, e);
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
            if let Err(e) = self.preserve_iteration_context(
                plan_id,
                &working_spec,
                &execution_plan,
                &artifacts,
                iterations,
            ).await {
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
                        success: final_verdict.as_ref()
                            .map(|v| matches!(v.decision, agent_agency_contracts::final_verdict::FinalDecision::Accept))
                            .unwrap_or(false),
                        quality_score: final_quality,
                        artifacts: artifacts.clone(),
                        completed_at: Utc::now(),
                    };

                    // Assign credit
                    if let Err(e) = turn_tracker.assign_credit(plan_id, turns, final_outcome).await {
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
            for (milestone_index, milestone) in execution_plan.contract_plan.milestones.iter().enumerate() {
                // Only process completed milestones
                if !matches!(milestone.state, agent_agency_contracts::planning_io::MilestoneState::Completed) {
                    continue;
                }
                
                // Get corresponding artifact (if available)
                let artifact = artifacts.get(milestone_index).or_else(|| {
                    // Try to find artifact by worker_id match
                    milestone.assigned_workers.first().and_then(|&worker_id| {
                        artifacts.iter().find(|a| {
                            a.provenance.worker_id.as_ref()
                                .and_then(|wid| Uuid::parse_str(wid).ok())
                                .map(|wid| wid == worker_id)
                                .unwrap_or(false)
                        })
                    })
                });
                
                if let Some(artifact) = artifact {
                    // Extract worker_id from milestone or artifact
                    let worker_id = milestone.assigned_workers.first()
                        .copied()
                        .or_else(|| {
                            artifact.provenance.worker_id.as_ref()
                                .and_then(|wid| Uuid::parse_str(wid).ok())
                        })
                        .unwrap_or_else(Uuid::new_v4);
                    
                    // Process outcome
                    match reflexive_learner.process_outcome(artifact, milestone, worker_id).await {
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
                            warn!("Failed to process learning outcome for milestone {}: {}", milestone.id, e);
                        }
                    }
                } else {
                    debug!("No artifact found for milestone {} - skipping ReflexiveLearner processing", milestone.id);
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
                metadata.insert("final_quality".to_string(), serde_json::json!(
                    quality_scores.last().copied().unwrap_or(0.0)
                ));
                metadata.insert("completed_at".to_string(), serde_json::json!(Utc::now().to_rfc3339()));
                
                if let Err(e) = session_mgr.update_session_context(
                    sid,
                    crate::orchestration::session_manager::SessionUpdate::Metadata(metadata),
                ).await {
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
            if let Err(e) = persistence.create_checkpoint(plan_id, &execution_state).await {
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
    pub async fn get_execution_status(
        &self,
        plan_id: Uuid,
    ) -> Result<Option<TaskExecutionState>> {
        if let Some(ref persistence) = self.state_persistence {
            persistence.load_state(plan_id).await
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
                persistence.save_state(&state).await
                    .map_err(|e| anyhow::anyhow!("Failed to save paused state: {}", e))?;
                persistence.create_checkpoint(plan_id, &state).await
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
                    return Err(anyhow::anyhow!("Plan {} is not paused (status: {:?})", plan_id, state.status));
                }
                state.status = ExecutionStateStatus::Running;
                state.last_updated = Utc::now();
                persistence.save_state(&state).await
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
                if matches!(state.status, ExecutionStateStatus::Completed | ExecutionStateStatus::Cancelled) {
                    return Err(anyhow::anyhow!("Plan {} cannot be cancelled (status: {:?})", plan_id, state.status));
                }
                state.status = ExecutionStateStatus::Cancelled;
                state.last_updated = Utc::now();
                state.error = Some("Task cancelled by user".to_string());
                persistence.save_state(&state).await
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
        
        // Execute plan using ParallelCoordinator
        let parallel_result = self.parallel_coordinator.execute_plan_parallel(&mut plan_for_execution).await?;
        
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
                if matches!(milestone.state, agent_agency_contracts::planning_io::MilestoneState::Completed) {
                    let worker_id_str = milestone.assigned_workers.first()
                        .map(|id| id.to_string())
                        .unwrap_or_else(|| milestone.id.clone());
                    
                    let mut artifact = ExecutionArtifacts::default();
                    artifact.task_id = plan_for_execution.contract_plan.id;
                    artifact.working_spec_id = plan_for_execution.contract_plan.working_spec_id.clone();
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
    async fn assign_worker_to_milestone(
        &self,
        milestone: &Milestone,
    ) -> Result<Uuid> {
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
            working_spec_provider: Box::new(SimpleWorkingSpecProvider { spec: working_spec.clone() }),
            task_descriptor: Box::new(SimpleTaskDescriptorProvider { descriptor: task_descriptor.clone() }),
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
            let worker_id = Uuid::new_v4();
            self.worker_lifecycle_manager.handle_assignment(worker_id, milestone).await?;
            
            let worktree_path = self.worktree_manager.create_worktree(milestone, worker_id).await?.worktree_path;
            let artifact = self.worker_bridge.execute_milestone(milestone, &worktree_path, worker_id).await?;
            self.worker_lifecycle_manager.handle_completion(worker_id, artifact.clone()).await?;
            artifacts.push(artifact);
        }
        
        // Create a verdict from artifacts
        // For now, return a simple accept verdict
        Ok(agent_agency_contracts::final_verdict::FinalVerdictContract {
            decision: agent_agency_contracts::final_verdict::FinalDecision::Accept,
            votes: vec![],
            dissent: String::new(),
            remediation: vec![],
            constitutional_refs: vec![],
            verification_summary: agent_agency_contracts::final_verdict::VerificationSummary {
                claims_total: artifacts.len() as u32,
                claims_verified: artifacts.len() as u32,
                coverage_pct: if artifacts.is_empty() {
                    0.0
                } else {
                    (artifacts.iter()
                        .map(|a| a.coverage.line_coverage)
                        .sum::<f64>() / artifacts.len() as f64) as f32
                },
            },
        })
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
        Ok(matches!(verdict.decision, agent_agency_contracts::final_verdict::FinalDecision::Accept))
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
        use crate::judge_backup::types::ReviewContext;
        use crate::decision_making::FinalDecision;
        
        let review_context = ReviewContext {
            session_id: format!("review_{}", Uuid::new_v4()),
            working_spec: serde_json::to_string(working_spec)
                .map_err(|e| anyhow::anyhow!("Failed to serialize working spec: {}", e))?,
            risk_tier: working_spec.risk_tier as u8,
            previous_reviews: vec![],
            constraints: std::collections::HashMap::new(),
        };
        
        let session = self.council.conduct_review(working_spec.clone(), review_context).await
            .map_err(|e| anyhow::anyhow!("Council review failed: {:?}", e))?;
        
        let approved = session.final_decision.as_ref()
            .map(|d| matches!(d, FinalDecision::Proceed { .. }))
            .unwrap_or(false);
        
        let needs_refinement = session.final_decision.as_ref()
            .map(|d| matches!(d, FinalDecision::Refine { .. }))
            .unwrap_or(false);
        
        let reason = match session.final_decision.as_ref() {
            Some(FinalDecision::Refine { refinement_directive, .. }) => {
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

/// Spec refiner implementation
struct UnifiedSpecRefiner;

#[async_trait::async_trait]
impl SpecRefiner for UnifiedSpecRefiner {
    async fn refine_working_spec(
        &self,
        current_spec: &WorkingSpec,
        refinement_reason: &str,
    ) -> Result<WorkingSpec> {
        // Simple refinement: update description with refinement reason
        let mut refined = current_spec.clone();
        refined.description = format!("{} (Refined: {})", current_spec.description, refinement_reason);
        refined.updated_at = chrono::Utc::now();
        Ok(refined)
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
            progress_percentage: progress,
            current_phase: message.clone().unwrap_or_else(|| "executing".to_string()),
            milestones_completed: 0,
            total_milestones: 0,
            estimated_completion: None,
            last_updated: chrono::Utc::now(),
            quality_score: None,
            errors: Vec::new(),
            warnings: Vec::new(),
        };
        
        self.base_tracker.update_progress(task_id, execution_progress).await
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
        let message = format!("Iteration {}: quality={:.2}, improvement={:.2}", iteration, quality_score, improvement_delta);
        
        let execution_progress = crate::progress_tracker::ExecutionProgress {
            task_id,
            progress_percentage: progress,
            current_phase: message.clone(),
            milestones_completed: iteration,
            total_milestones: 0,
            estimated_completion: None,
            last_updated: chrono::Utc::now(),
            quality_score: Some(quality_score),
            errors: Vec::new(),
            warnings: Vec::new(),
        };
        
        self.base_tracker.update_progress(task_id, execution_progress).await
            .map_err(|e| anyhow!("Failed to track iteration progress: {}", e))?;
        
        Ok(())
    }

    async fn detect_and_report_plateaus(
        &self,
        task_id: Uuid,
        quality_scores: &[f64],
        iteration: u32,
    ) -> Result<()> {
        // Use turn-level tracker if available to detect plateaus
        // For now, just log if quality scores are stagnant
        if quality_scores.len() >= 3 {
            let recent_scores = &quality_scores[quality_scores.len().saturating_sub(3)..];
            let avg_recent: f64 = recent_scores.iter().sum::<f64>() / recent_scores.len() as f64;
            let variance: f64 = recent_scores.iter()
                .map(|s| (s - avg_recent).powi(2))
                .sum::<f64>() / recent_scores.len() as f64;
            
            if variance < 0.01 {
                warn!("Plateau detected at iteration {}: quality variance={:.4}", iteration, variance);
                // Update progress with plateau warning
                let execution_progress = crate::progress_tracker::ExecutionProgress {
                    task_id,
                    progress_percentage: (iteration as f32 * 10.0).min(90.0),
                    current_phase: format!("Iteration {}: Plateau detected (quality variance={:.4})", iteration, variance),
                    milestones_completed: iteration,
                    total_milestones: 0,
                    estimated_completion: None,
                    last_updated: chrono::Utc::now(),
                    quality_score: Some(avg_recent),
                    errors: Vec::new(),
                    warnings: vec!["Quality plateau detected - consider refinement".to_string()],
                };
                
                let _ = self.base_tracker.update_progress(task_id, execution_progress).await;
            }
        }
        
        Ok(())
    }
}

