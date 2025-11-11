# Orchestration Module

This module provides the unified orchestration system for Agent Agency V3.

## Components

### UnifiedOrchestrator

Main orchestrator that coordinates all system components:
- Planning system
- Worker execution
- Council review
- Worktree management
- State persistence

### UnifiedOrchestratorFactory

Factory for creating fully configured orchestrator instances with all dependencies.

**Automatic Worker Scaffolding:**

When the orchestrator is initialized, it automatically scaffolds standard workers in the database if they don't exist:

- General Purpose Worker
- File Editing Worker
- Code Generation Worker
- Testing Worker
- Documentation Worker

This ensures the orchestrator always has workers available for task execution without manual setup.

**Usage:**

```rust
use agent_orchestration::orchestration::UnifiedOrchestratorFactory;

let orchestrator = UnifiedOrchestratorFactory::create(db_ops).await?;
```

### Worker Scaffolding

Automatic worker registration system (`worker_scaffolding.rs`):

- Checks for existing workers on startup
- Creates standard workers if none exist
- Non-fatal errors (orchestrator continues if scaffolding fails)
- Idempotent (won't create duplicates)

### Session Manager

Manages orchestration sessions and state.

### Task State Persistence

Provides persistence for task execution state, enabling pause/resume/cancel functionality.

### Worker Evolution

Automatic worker creation and refinement system (`worker_evolution.rs`):

The orchestrator includes a **Worker Evolution Engine** that enables agents to craft their own specialized workers based on observed execution patterns. This system:

- **Pattern Detection**: Analyzes learning outcomes to identify recurring task patterns
- **Worker Creation Proposals**: Generates proposals for new specialized workers when patterns suggest a need
- **Worker Refinement Proposals**: Identifies capability gaps and proposes additions to existing workers
- **Automatic Execution**: Evaluates and executes high-confidence proposals automatically
- **Integration with Reflexive Learning**: Processes outcomes from `ReflexiveLearner` to drive evolution

**How It Works:**

1. **Outcome Collection**: `ReflexiveLearner` collects execution outcomes (success rate, quality, task characteristics)
2. **Pattern Analysis**: `WorkerEvolutionEngine` analyzes outcomes to detect patterns:
   - Tasks requiring capabilities that don't exist in any worker → Creation proposal
   - Workers successfully handling tasks with missing capabilities → Refinement proposal
3. **Proposal Generation**: Creates proposals with confidence scores and expected benefits
4. **Evaluation**: Evaluates proposals against thresholds (confidence, benefit, worker limits)
5. **Execution**: Automatically creates or refines workers when proposals meet criteria

**Configuration:**

```rust
use agent_orchestration::planning::worker_evolution::EvolutionConfig;

let config = EvolutionConfig {
    min_creation_confidence: 0.7,    // Minimum confidence for creation
    min_creation_benefit: 0.10,      // Minimum expected benefit
    enable_auto_creation: true,       // Auto-create workers
    enable_auto_refinement: true,    // Auto-refine workers
    max_workers: 50,                  // Maximum worker count
    min_performance_threshold: 0.5,   // Minimum performance for refinement
    min_outcomes_for_refinement: 10,  // Minimum outcomes needed
};
```

**Example:**

When the system observes 15+ tasks requiring "API generation" capability with 70%+ success rate and no suitable worker exists, it automatically creates an "API endpoint generation Specialist" worker with the required capabilities.

See `docs/implementation/worker-evolution-integration.md` for detailed documentation.

