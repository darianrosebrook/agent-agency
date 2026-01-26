//! Planning module for task orchestration

pub mod assignment_storage;
pub mod caws_adjudication_cycle;
pub mod caws_complexity_mode;
pub mod caws_debate_scorer;
pub mod caws_integration;
pub mod caws_quality_gates;
pub mod caws_spec_resolver;
pub mod caws_tool_registry;
pub mod council_adapter;
pub mod council_integration;
pub mod council_monitor;
pub mod council_review;
pub mod curriculum_learning;
pub mod data_infrastructure_types;
pub mod database_operations_bridge;
pub mod data_processing_adapter;
pub mod dependency_resolver;
pub mod evidence;
pub mod factory;
pub mod graph_algorithms;
pub mod intelligent_spec_refiner;
pub mod legacy_plan_adapter;
pub mod memory_adapter;
pub mod model_lifecycle;
pub mod orchestrator_integration;
pub mod parallel_coordinator;
pub mod plan_executor;
pub mod plan_generator;
pub mod plan_types;
pub mod planning_engine_impl;
pub mod quality_gates;
pub mod refinement_loop;
pub mod reflexive_learner;
pub mod research_adapter;
pub mod rubric_engineering;
pub mod scope_guard;
pub mod storage;
pub mod task_executor_factory;
pub mod thinking_budget;
pub mod todo_integration;
pub mod todo_template;
pub mod tool_chain_adapter;
pub mod tool_chain_bridge;
pub mod tool_chain_types;
pub mod type_adapters;
pub mod types;
pub mod waiver_integration;
pub mod worker_assignment;
pub mod worker_evolution;
pub mod worker_lifecycle_manager;
pub mod worktree_manager;

// Re-export types for convenience
// Use explicit exports for types that conflict across modules to avoid ambiguous glob re-exports

// From orchestrator_integration - no conflicts
pub use orchestrator_integration::*;

// From plan_executor - export explicitly to avoid conflicts with worker_assignment and storage
pub use plan_executor::{
    AuditEvent, AuditEventType, AuditTrail, FailureOracle, PlanExecutor, TodoAdapter,
    TodoInterface, WorkerPerformance,
};

// From plan_types - export explicitly to avoid conflicts with plan_generator, tool_chain_types, data_infrastructure_types, and todo_integration
pub use plan_types::{
    ActiveExecutionState, BatchStatus, CostLimits, ExecutionContext, ExecutionPlan,
    OrchestrationMetadata, ParallelBatch, PerformanceProfile, PlanGenerationStrategy,
    PlanningConstraints, PlanningContext, PlanningMetrics, PlanningPhase, PlanningSession,
    ResourceInventory, ResourcePreferences, ResourceRequirements, ResourceUtilization,
    TaskDescriptorProvider, TodoIntegration as PlanTypesTodoIntegration, WorkerCapabilities,
    WorkingSpecProvider,
};

// From plan_generator - exclude PlanGenerationStrategy (already exported from plan_types)
pub use plan_generator::PlanGenerator;

// From storage - exclude AuditEvent (already exported from plan_executor)
pub use storage::PlanningStorage;

// From parallel_coordinator - no conflicts
pub use parallel_coordinator::*;

// From worker_assignment - exclude WorkerPerformance (already exported from plan_executor)
pub use worker_assignment::WorkerAssignmentStrategy;

// From evidence - no conflicts
pub use evidence::*;

// From scope_guard - no conflicts
pub use scope_guard::*;

// From council_monitor - no conflicts
pub use council_monitor::*;

// From todo_integration - export explicitly to avoid conflict with plan_types::TodoIntegration
pub use todo_integration::TodoIntegration;

// From council_review - no conflicts
pub use council_review::*;

// From dependency_resolver - no conflicts
pub use dependency_resolver::*;

// From factory - no conflicts
pub use factory::*;

// From legacy_plan_adapter - no conflicts
pub use legacy_plan_adapter::*;

// From tool_chain_bridge - no conflicts
pub use tool_chain_bridge::*;

// From tool_chain_types - exclude PlanningContext and PlanningConstraints (already exported from plan_types)
pub use tool_chain_types::ToolChain;

// From planning_engine_impl - no conflicts
pub use planning_engine_impl::*;

// From type_adapters - no conflicts
pub use type_adapters::*;

// From research_adapter - no conflicts
pub use research_adapter::*;

// From waiver_integration - no conflicts
pub use waiver_integration::*;

// From caws_integration - no conflicts
pub use caws_integration::*;

// From todo_template - no conflicts
pub use todo_template::*;

// From data_infrastructure_types - exclude CostLimits (already exported from plan_types)
pub use data_infrastructure_types::{
    models, CreateAuditTrailEntry, CreateExecutionPlan, CreatePlanningAuditEvent,
    CreatePlanningSession, CreatePlanningTelemetry, CreateWaiver, DatabaseOperations,
    UpdateExecutionPlan, UpdatePlanningSession, UpdateWaiver,
};

// From database_operations_bridge - no conflicts
pub use database_operations_bridge::DatabaseOperationsBridge;

// From refinement_loop - no conflicts
pub use refinement_loop::*;

// From intelligent_spec_refiner - no conflicts
pub use intelligent_spec_refiner::*;

// From council_integration - no conflicts
pub use council_integration::*;

// From worktree_manager - no conflicts
pub use worktree_manager::*;

// From caws_adjudication_cycle - no conflicts
pub use caws_adjudication_cycle::*;

// From caws_tool_registry - export CawsToolRegistry
pub use caws_tool_registry::CawsToolRegistry;

// From worker_lifecycle_manager - no conflicts
pub use worker_lifecycle_manager::*;

// From caws_debate_scorer - no conflicts
pub use caws_debate_scorer::*;
