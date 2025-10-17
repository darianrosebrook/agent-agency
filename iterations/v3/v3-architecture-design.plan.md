# Agent Agency V3: Council-Based Arbiter Architecture

## Vision

V3 reimagines agent orchestration as a **constitutional council system** where specialized judge models work together to audit, evaluate, and accept worker outputs. Built ground-up for Apple Silicon with Core ML optimization, this system combines model-native CAWS understanding with runtime enforcement for maximum efficiency.

## Core Architectural Shift

### From V2 → V3

**V2 Architecture:**

- 29 components across multiple layers
- Single orchestrator with distributed responsibilities
- Runtime-only CAWS enforcement
- Generic model pool management

**V3 Architecture:**

- **Council of Judges**: 3-5 specialized models (Constitutional Judge, Technical Auditor, Quality Evaluator, Integration Validator)
- **Worker Pool**: Task execution models with CAWS awareness
- **Research Agent**: Dedicated model for knowledge gathering and context building
- **Apple Silicon Native**: Core ML optimized inference pipeline
- **Hybrid CAWS**: Models trained on CAWS principles + runtime validation layer

## System Components

### 1. The Council (Core Decision Layer)

**Constitutional Judge** (`llama3.3:3b-constitutional-caws`)

- Fine-tuned on CAWS rulebook and constitutional principles
- Evaluates budget compliance, waiver validity, provenance requirements
- Makes binding decisions on CAWS violations
- Optimized for ANE execution (sub-100ms inference)

**Technical Auditor** (`codellama:7b-audit-specialist`)

- Specialized in code quality, security, architecture
- AST analysis, dependency checking, contract validation
- Detects anti-patterns, security vulnerabilities
- GPU-optimized for complex analysis tasks

**Quality Evaluator** (`gemma2:3b-quality-judge`)

- Assesses output correctness, completeness, maintainability
- Runs satisficing checks against acceptance criteria
- Provides feedback for iterative improvement
- Balanced CPU/GPU execution

**Integration Validator** (`mistral:3b-integration-checker`)

- Validates cross-file consistency, API contracts
- Checks database migrations, breaking changes
- Ensures system-wide coherence
- CPU-optimized for fast validation

**Consensus Coordinator** (Lightweight Rust Service)

- Aggregates judge verdicts using weighted voting
- Resolves conflicts through debate protocol
- Enforces unanimous approval for Tier 1 changes
- Maintains audit trail of all council decisions

### 2. Worker Pool (Execution Layer)

**Generalist Workers** (`llama3.3:7b-caws-aware`)

- General-purpose task execution
- Trained to self-check against CAWS rules
- Emit structured outputs with rationale
- 4-8 concurrent workers per workstation

**Specialist Workers** (Domain-specific fine-tunes)

- TypeScript/React specialist
- Python/Backend specialist
- Database/Migration specialist
- Testing/QA specialist
- Each pre-trained on domain patterns + CAWS

### 3. Research Agent (Knowledge Layer)

**Knowledge Seeker** (`llama3.3:13b-research-optimized`)

- Dedicated to information gathering and synthesis
- Vector search integration for context retrieval
- Web scraping and API integration
- Builds comprehensive context for workers
- Offloads "thinking" from worker models

### 4. Orchestration Core (Coordination Layer)

**Task Router** (Rust Service with Core ML bindings)

- Analyzes task complexity and requirements
- Routes to appropriate worker specialists
- Manages concurrent execution pool
- Tracks performance metrics for RL training

**Council Coordinator** (Rust Service)

- Orchestrates judge evaluation workflow
- Implements debate protocol for conflicts
- Enforces CAWS compliance gates
- Generates verdict documents

**Observer Bridge** (Existing v2 component, enhanced)

- Real-time task monitoring and visualization
- Council deliberation transparency
- Performance metrics dashboard

### 5. Apple Silicon Optimization Layer

**Core ML Pipeline** (Swift/C++ Integration)

- Model quantization (INT4/INT8 for judges, INT8/FP16 for workers)
- ANE routing for constitutional judge
- GPU batching for technical auditor
- CPU parallel execution for smaller judges
- Metal shader optimization for embeddings

**Unified Memory Manager** (Rust/Swift)

- Zero-copy model weights sharing
- Intelligent model eviction (LRU + priority)
- Pre-warming for frequent models
- Memory pressure monitoring

**Thermal Management** (Native macOS Integration)

- CPU/GPU/ANE utilization monitoring
- Throttling when thermals exceed 85°C
- Adaptive batch sizing based on temperature
- Background vs foreground execution modes

## Data Flow Architecture

### Task Execution Flow

```
1. User Request → Task Router
2. Router → Research Agent (parallel context gathering)
3. Router → Worker Pool (task assignment based on specialty)
4. Worker → Structured Output + Rationale + Self-Assessment
5. Output → Council of Judges (parallel evaluation)
6. Judges → Individual Verdicts (Constitutional, Technical, Quality, Integration)
7. Consensus Coordinator → Aggregate Verdict
8. If Conflicts → Debate Protocol (judges defend positions)
9. Final Verdict → Accept/Reject/Require Modification
10. If Accepted → Commit with Provenance
11. If Rejected → Feedback to Worker → Retry (max 3 iterations)
```

### Council Deliberation Protocol

```
Phase 1: Independent Evaluation (parallel)
- Each judge receives: task spec, worker output, context
- Each judge emits: verdict (pass/fail/uncertain), reasoning, evidence citations

Phase 2: Conflict Detection
- Consensus Coordinator identifies disagreements
- If unanimous → immediate acceptance/rejection
- If conflicts → proceed to Phase 3

Phase 3: Adversarial Debate
- Dissenting judges present counter-arguments
- Supporting judges defend their positions
- Research Agent provides additional evidence
- 2-3 debate rounds maximum

Phase 4: Final Vote
- Judges revise verdicts based on debate
- Weighted voting (Constitutional Judge 40%, others 20% each)
- Supermajority required for Tier 1 changes (80%+)
- Simple majority for Tier 2/3 (60%+)
```

## Directory Structure

```
iterations/v3/
├── council/                    # Council of judges implementation
│   ├── models/                 # Fine-tuned model definitions
│   │   ├── constitutional.yaml
│   │   ├── technical.yaml
│   │   ├── quality.yaml
│   │   └── integration.yaml
│   ├── coordinator.rs          # Consensus coordination service
│   ├── debate.rs              # Adversarial debate protocol
│   └── verdicts.rs            # Verdict generation and storage
├── workers/                    # Worker pool implementation
│   ├── generalist/            # General-purpose workers
│   ├── specialists/           # Domain-specific workers
│   ├── pool-manager.rs        # Worker lifecycle management
│   └── caws-runtime.rs        # CAWS self-check utilities
├── research/                   # Research agent
│   ├── knowledge-seeker.rs    # Main research coordinator
│   ├── web-scraper.rs         # Web content retrieval
│   ├── vector-search.rs       # Semantic search integration
│   └── context-builder.rs     # Context synthesis
├── orchestration/              # Core coordination
│   ├── task-router.rs         # Intelligent task routing
│   ├── execution-manager.rs   # Parallel execution coordination
│   └── rl-tracker.rs          # Performance tracking for RL
├── apple-silicon/              # Platform optimization
│   ├── core-ml/               # Core ML integration (Swift)
│   │   ├── model-loader.swift
│   │   ├── ane-executor.swift
│   │   └── quantization.swift
│   ├── memory/                # Unified memory management
│   │   ├── allocator.rs
│   │   └── eviction.rs
│   └── thermal/               # Thermal management
│       ├── monitor.swift
│       └── throttle.rs
├── caws/                       # CAWS enforcement
│   ├── runtime-validator/     # Runtime CAWS checks
│   ├── model-training/        # CAWS fine-tuning datasets
│   └── provenance/            # Audit trail generation
├── database/                   # PostgreSQL + pgvector
│   ├── schema.sql             # Simplified v3 schema
│   ├── migrations/            # Version control
│   └── client.rs              # Connection pool manager
├── observer/                   # Monitoring and visualization
│   ├── bridge.rs              # Enhanced observer bridge
│   └── ui/                    # Web dashboard (Next.js)
├── training/                   # Model fine-tuning
│   ├── caws-dataset/          # CAWS training data
│   ├── rl-pipeline/           # Reinforcement learning
│   └── fine-tune-scripts/     # Training automation
├── tests/                      # Comprehensive testing
│   ├── unit/                  # Component tests
│   ├── integration/           # Cross-component tests
│   ├── e2e/                   # End-to-end scenarios
│   └── council/               # Council behavior tests
└── docs/                       # Documentation
    ├── architecture.md        # System design
    ├── council-protocol.md    # Judge coordination
    ├── apple-silicon.md       # Platform optimization
    └── caws-integration.md    # CAWS enforcement
```

## Key Innovations from V2 Learnings

### 1. Council > Single Arbiter

**Problem:** V2's single orchestrator had too many responsibilities

**Solution:** Specialized judge models with clear domains

**Benefit:** Parallel evaluation, faster decisions, better quality

### 2. Model-Native CAWS

**Problem:** Runtime-only enforcement is slow and repetitive

**Solution:** Fine-tune models on CAWS principles with runtime validation

**Benefit:** Workers self-correct, fewer violations, faster iteration

### 3. Apple Silicon Native

**Problem:** V2 treated hardware as generic compute

**Solution:** Core ML pipeline with ANE/GPU/CPU orchestration

**Benefit:** 3-5x faster inference, lower power, better thermals

### 4. Research Agent Separation

**Problem:** Workers spent tokens on information gathering

**Solution:** Dedicated research model with vector search

**Benefit:** Workers focus on execution, better token efficiency

### 5. Simplified Component Count

**Problem:** 29 v2 components created integration complexity

**Solution:** 15-20 focused v3 components with clear boundaries

**Benefit:** Easier maintenance, faster development, clearer testing

## Performance Targets

### Inference Latency

- Constitutional Judge: <100ms (ANE-optimized)
- Technical Auditor: <500ms (GPU-accelerated)
- Quality Evaluator: <200ms (balanced)
- Integration Validator: <150ms (CPU-optimized)
- Worker Models: <2s per task (parallel execution)

### Throughput

- 10+ concurrent workers on M3 Max (64GB)
- 5+ concurrent workers on M3 Pro (32GB)
- Council evaluation: <1s for Tier 2/3, <3s for Tier 1

### Resource Usage

- Peak memory: 48GB for full system (M3 Max)
- Idle memory: 12GB (base models loaded)
- Thermal: <80°C sustained load
- Power: <30W average (M3 efficiency cores + ANE)

## Implementation Phases

### Phase 1: Foundation (Weeks 1-2)

- Core Rust services (router, coordinator, memory manager)
- Database schema and migration system
- Basic worker pool (single generalist model)
- Simple council (constitutional judge only)
- Observer bridge integration

### Phase 2: Council (Weeks 3-4)

- Complete judge implementation (4 specialized models)
- Debate protocol implementation
- Verdict generation and storage
- Consensus coordination
- Council behavior tests

### Phase 3: Apple Silicon (Weeks 5-6)

- Core ML integration layer
- Model quantization pipeline
- ANE/GPU/CPU routing
- Unified memory management
- Thermal monitoring

### Phase 4: CAWS Integration (Weeks 7-8)

- CAWS training dataset generation
- Model fine-tuning on CAWS principles
- Runtime validator integration
- Provenance tracking
- Hybrid enforcement testing

### Phase 5: Research Agent (Weeks 9-10)

- Knowledge seeker implementation
- Vector search integration
- Web scraping capabilities
- Context synthesis
- Research-worker coordination

### Phase 6: Production Hardening (Weeks 11-12)

- Comprehensive test suite
- Performance benchmarking
- Error handling and recovery
- Monitoring and alerting
- Documentation

## Success Criteria

### Functional

- [ ] Council reaches consensus on 95%+ of decisions
- [ ] Debate protocol resolves conflicts in <5s
- [ ] Workers self-correct CAWS violations 80%+ of time
- [ ] Research agent reduces worker token usage by 40%+
- [ ] System handles 10+ concurrent tasks on M3 Max

### Performance

- [ ] Council evaluation <1s for Tier 2/3 tasks
- [ ] ANE utilization >60% for constitutional judge
- [ ] Memory usage <50GB on M3 Max under full load
- [ ] Sustained operation <80°C thermal
- [ ] 3-5x faster inference vs generic CPU execution

### Quality

- [ ] CAWS compliance rate >95%
- [ ] Test coverage >85% across all components
- [ ] Zero critical security vulnerabilities
- [ ] Complete audit trail for all decisions
- [ ] 99%+ uptime in continuous operation

## Migration from V2

### Reusable Components

- Observer bridge (enhance, don't rebuild)
- Database schema (simplify, migrate)
- Test infrastructure (adapt to council)
- CAWS validator (integrate with fine-tuning)

### Deprecated Components

- Single arbiter orchestrator → Council coordination
- Generic model pool → Specialized judges + workers
- 29 distributed components → 15-20 focused components
- Python/TypeScript mix → Rust core with Swift optimization

### Data Migration

- Agent registry → Worker pool + judge registry
- Performance tracking → RL training dataset
- Provenance chain → Enhanced verdict storage
- Knowledge base → Vector search integration

## Risk Mitigation

### Technical Risks

- **Fine-tuning complexity**: Start with LoRA adapters, iterate
- **Core ML integration**: Use reference implementations, test incrementally
- **Thermal management**: Conservative initial limits, profile real usage
- **Council deadlock**: Timeout mechanisms, fallback to majority vote

### Schedule Risks

- **Model training time**: Pre-train on CAWS dataset, parallel training
- **Apple Silicon unknowns**: Prototype week 1, adjust architecture if needed
- **Integration challenges**: Comprehensive unit tests, integration tests early

## Next Steps

1. **Create v3 directory structure**
2. **Set up Rust project with workspace**
3. **Define model specifications (Modelfiles)**
4. **Implement core router and coordinator**
5. **Build initial constitutional judge**
6. **Test basic council workflow**
7. **Iterate based on performance**

---

**V3 represents a fundamental reimagining**: from distributed orchestration to constitutional council, from runtime enforcement to model-native understanding, from generic compute to Apple Silicon optimization. This is the system we should have built from the start.
